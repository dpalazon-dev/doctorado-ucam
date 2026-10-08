use research_workbench_core::adapters::sqlite::migrations::{MIGRATIONS, SCHEMA_VERSION};
use research_workbench_core::adapters::sqlite::migrations::{Migration, checksum, migrate};
use research_workbench_core::{
    adapters::{
        sqlite::{actor::DbActor, workflow_repository::SqliteWorkflowRepository},
        windows::paths::LibraryRoot,
    },
    application::{
        unit_of_work::with_transaction,
        workflow::{initialize_processing, invalidate_effective_change},
    },
    modules::workflow::definitions,
    transport::dto::PhaseCode,
};
use rusqlite::{Connection, OptionalExtension};
use std::fs;

#[test]
fn schema_v2_contains_one_atomic_workflow_migration() {
    assert_eq!(SCHEMA_VERSION, 2);
    assert_eq!(
        MIGRATIONS
            .iter()
            .map(|migration| migration.version)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let sql = MIGRATIONS[1].sql;
    assert!(sql.contains("phase_definitions"));
    assert!(sql.contains("paper_phases"));
    assert!(sql.contains("phase_answers"));
    assert!(sql.contains("PRE"));
    assert!(sql.contains("P1"));
    assert!(sql.contains("P2"));
}

#[tokio::test]
async fn fresh_library_applies_schema_v2_and_seeds_canonical_definitions() {
    let dir = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(dir.path().join("library"));
    let actor = DbActor::start(root).unwrap();
    assert_eq!(actor.info().schema_version, 2);
    actor
        .submit(|connection| {
            assert_eq!(
                connection.query_row("SELECT count(*) FROM phase_definitions", [], |row| row
                    .get::<_, i64>(0))?,
                3
            );
            for code in [PhaseCode::PRE, PhaseCode::P1, PhaseCode::P2] {
                let definition = definitions::get(&code, None).unwrap();
                let payload: String = connection.query_row(
                    "SELECT payload_json FROM phase_definitions WHERE code=?1 AND version=1",
                    [serde_json::to_value(&code).unwrap().as_str().unwrap()],
                    |row| row.get(0),
                )?;
                let actual: serde_json::Value = serde_json::from_str(&payload).unwrap();
                assert_eq!(actual, serde_json::to_value(definition).unwrap());
            }
            Ok(())
        })
        .await
        .unwrap();
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn initialization_is_idempotent_and_invalidates_later_started_phases_once() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let paper_id = uuid::Uuid::new_v4().to_string();
    actor.submit({
        let paper_id = paper_id.clone();
        move |connection| {
            connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'Workflow fixture','survey','now','now')", [&paper_id])?;
            with_transaction(connection, |tx| initialize_processing(tx, &SqliteWorkflowRepository, &paper_id))?;
            Ok(())
        }
    }).await.unwrap();
    actor.submit({
        let paper_id = paper_id.clone();
        move |connection| {
            with_transaction(connection, |tx| {
                initialize_processing(tx, &SqliteWorkflowRepository, &paper_id)?;
                tx.execute("UPDATE paper_phases SET state='IN_PROGRESS' WHERE paper_id=?1 AND phase_code='P1'", [&paper_id])?;
                tx.execute("UPDATE paper_phases SET state='IN_PROGRESS',completed_at='kept',accepted_gate_snapshot_json='{}',accepted_gate_snapshot_hash=?2 WHERE paper_id=?1 AND phase_code='P2'", rusqlite::params![paper_id, "0".repeat(64)])?;
                invalidate_effective_change(tx, &SqliteWorkflowRepository, &paper_id, &[PhaseCode::PRE])
            })?;
            Ok(())
        }
    }).await.unwrap();
    let states = actor.submit(move |connection| {
        let mut statement = connection.prepare("SELECT phase_code,state,revision,completed_at,accepted_gate_snapshot_json FROM paper_phases WHERE paper_id=?1 ORDER BY phase_code")?;
        let rows = statement.query_map([paper_id], |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?,row.get::<_,i64>(2)?,row.get::<_,Option<String>>(3)?,row.get::<_,Option<String>>(4)?)))?;
        rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)
    }).await.unwrap();
    assert_eq!(
        states.iter().find(|r| r.0 == "P1").unwrap().1,
        "NEEDS_REVIEW"
    );
    assert_eq!(
        states.iter().find(|r| r.0 == "P2").unwrap().1,
        "NEEDS_REVIEW"
    );
    assert_eq!(states.iter().find(|r| r.0 == "P1").unwrap().2, 3);
    assert_eq!(states.iter().find(|r| r.0 == "P2").unwrap().2, 4);
    assert_eq!(
        states.iter().find(|r| r.0 == "P2").unwrap().3.as_deref(),
        Some("kept")
    );
    assert_eq!(
        states.iter().find(|r| r.0 == "P2").unwrap().4.as_deref(),
        Some("{}")
    );
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn direct_in_progress_input_keeps_state_but_advances_its_clock() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let paper_id = uuid::Uuid::new_v4().to_string();
    actor.submit({
        let paper_id = paper_id.clone();
        move |connection| {
            connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'Clock fixture','survey','now','now')", [&paper_id])?;
            with_transaction(connection, |tx| initialize_processing(tx, &SqliteWorkflowRepository, &paper_id))?;
            with_transaction(connection, |tx| {
                tx.execute("UPDATE paper_phases SET state='IN_PROGRESS' WHERE paper_id=?1 AND phase_code IN ('P1','P2')", [&paper_id])?;
                invalidate_effective_change(tx, &SqliteWorkflowRepository, &paper_id, &[PhaseCode::P1])?;
                initialize_processing(tx, &SqliteWorkflowRepository, &paper_id)
            })
        }
    }).await.unwrap();
    let revisions = actor
        .submit(move |connection| {
            let p1: (String, i64) = connection.query_row(
                "SELECT state,revision FROM paper_phases WHERE paper_id=?1 AND phase_code='P1'",
                [&paper_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            let p2: (String, i64) = connection.query_row(
                "SELECT state,revision FROM paper_phases WHERE paper_id=?1 AND phase_code='P2'",
                [&paper_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            Ok((p1, p2))
        })
        .await
        .unwrap();
    assert_eq!(revisions.0, ("IN_PROGRESS".into(), 2));
    assert_eq!(revisions.1, ("NEEDS_REVIEW".into(), 3));
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn overlapping_inputs_deduplicate_each_phase_clock() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let paper = uuid::Uuid::new_v4().to_string();
    let id = paper.clone();
    actor.submit(move|connection|{
        connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'Overlap fixture','survey','now','now')",[&id])?;
        with_transaction(connection,|tx|initialize_processing(tx,&SqliteWorkflowRepository,&id))?;
        connection.execute("UPDATE paper_phases SET state='IN_PROGRESS' WHERE paper_id=?1 AND phase_code IN ('P1','P2')",[&id])?;
        with_transaction(connection,|tx|invalidate_effective_change(tx,&SqliteWorkflowRepository,&id,&[PhaseCode::PRE,PhaseCode::P1]))
    }).await.unwrap();
    let id = paper.clone();
    let phases=actor.submit(move|c|{let mut s=c.prepare("SELECT phase_code,state,revision FROM paper_phases WHERE paper_id=?1 ORDER BY CASE phase_code WHEN 'PRE' THEN 0 WHEN 'P1' THEN 1 ELSE 2 END")?;let rows=s.query_map([id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?)))?;rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)}).await.unwrap();
    assert_eq!(
        phases,
        vec![
            ("PRE".into(), "IN_PROGRESS".into(), 2),
            ("P1".into(), "NEEDS_REVIEW".into(), 3),
            ("P2".into(), "NEEDS_REVIEW".into(), 4)
        ]
    );
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn initialized_processing_is_noop_after_phase_context_has_advanced() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let paper_id = uuid::Uuid::new_v4().to_string();
    let id = paper_id.clone();
    actor.submit(move|connection|{
        connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'Advanced fixture','survey','now','now')",[&id])?;
        with_transaction(connection,|tx|initialize_processing(tx,&SqliteWorkflowRepository,&id))?;
        tx_update_advanced_context(connection,&id)?;
        with_transaction(connection,|tx|initialize_processing(tx,&SqliteWorkflowRepository,&id))
    }).await.unwrap();
    let state=actor.submit(move|connection|Ok(connection.query_row("SELECT p.current_phase,pp.state,pp.revision FROM papers p JOIN paper_phases pp ON pp.paper_id=p.id AND pp.phase_code='P1' WHERE p.id=?1",[paper_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?)))?)).await.unwrap();
    assert_eq!(state, ("P1".into(), "IN_PROGRESS".into(), 3));
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

fn tx_update_advanced_context(
    connection: &mut rusqlite::Connection,
    paper_id: &str,
) -> Result<(), research_workbench_core::transport::error::AppError> {
    connection.execute(
        "UPDATE papers SET current_phase='P1' WHERE id=?1",
        [paper_id],
    )?;
    let snapshot_hash =
        research_workbench_core::application::library::canonical_hash(&serde_json::json!({}))?;
    connection.execute("UPDATE paper_phases SET state='COMPLETED',revision=2,completed_at='kept',accepted_gate_snapshot_json='{}',accepted_gate_snapshot_hash=?2 WHERE paper_id=?1 AND phase_code='PRE'",rusqlite::params![paper_id,snapshot_hash])?;
    connection.execute("UPDATE paper_phases SET state='IN_PROGRESS',revision=3 WHERE paper_id=?1 AND phase_code='P1'",[paper_id])?;
    Ok(())
}

type V1PaperSnapshot = Vec<(String, String, String, i64, String)>;
type V1UserDataSnapshot = Vec<(String, Vec<serde_json::Value>)>;
type V1LibraryFixture = (
    tempfile::TempDir,
    LibraryRoot,
    Connection,
    V1PaperSnapshot,
    V1UserDataSnapshot,
);

fn snapshot_table(
    connection: &Connection,
    name: &str,
    query: &str,
) -> rusqlite::Result<(String, Vec<serde_json::Value>)> {
    let mut statement = connection.prepare(query)?;
    let columns = (0..statement.column_count())
        .map(|index| statement.column_name(index).map(str::to_owned))
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let rows = statement.query_map([], |row| {
        let mut object = serde_json::Map::new();
        for (index, column) in columns.iter().enumerate() {
            let value = match row.get_ref(index)? {
                rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                rusqlite::types::ValueRef::Integer(value) => serde_json::json!(value),
                rusqlite::types::ValueRef::Real(value) => serde_json::json!(value),
                rusqlite::types::ValueRef::Text(value) => {
                    serde_json::Value::String(String::from_utf8_lossy(value).into_owned())
                }
                rusqlite::types::ValueRef::Blob(value) => serde_json::Value::Array(
                    value.iter().map(|byte| serde_json::json!(byte)).collect(),
                ),
            };
            object.insert(column.clone(), value);
        }
        Ok(serde_json::Value::Object(object))
    })?;
    Ok((name.to_owned(), rows.collect::<Result<Vec<_>, _>>()?))
}

fn library_user_data_snapshot(connection: &Connection) -> rusqlite::Result<V1UserDataSnapshot> {
    Ok(vec![
        snapshot_table(
            connection,
            "venues",
            "SELECT id,name,kind,identifier FROM venues ORDER BY id",
        )?,
        snapshot_table(
            connection,
            "authors",
            "SELECT id,display_name,orcid FROM authors ORDER BY id",
        )?,
        snapshot_table(
            connection,
            "papers",
            "SELECT id,title,doi,year,review_type,domain,url,venue_id,lifecycle,archived_from_lifecycle,revision,active_document_id,created_at,updated_at,last_opened_at FROM papers ORDER BY id",
        )?,
        snapshot_table(
            connection,
            "paper_authors",
            "SELECT paper_id,author_id,author_order FROM paper_authors ORDER BY paper_id,author_order",
        )?,
        snapshot_table(
            connection,
            "documents",
            "SELECT id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status FROM documents ORDER BY id",
        )?,
        snapshot_table(
            connection,
            "reading_positions",
            "SELECT document_id,page_index,zoom,revision,updated_at FROM reading_positions ORDER BY document_id",
        )?,
        snapshot_table(
            connection,
            "app_session",
            "SELECT singleton,last_opened_paper_id FROM app_session ORDER BY singleton",
        )?,
        snapshot_table(
            connection,
            "operation_receipts",
            "SELECT request_id,command,payload_hash,result_json,created_at FROM operation_receipts ORDER BY request_id",
        )?,
        snapshot_table(
            connection,
            "audit_events",
            "SELECT id,request_id,action,entity_id,changes_json,created_at FROM audit_events ORDER BY id",
        )?,
    ])
}

fn populated_v1_library() -> V1LibraryFixture {
    let dir = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(dir.path().join("library"));
    fs::create_dir_all(root.path().join("documents")).unwrap();
    fs::create_dir_all(root.path().join("staging")).unwrap();
    fs::create_dir_all(root.path().join("recovery")).unwrap();
    fs::write(
        root.path().join("library.json"),
        r#"{"formatVersion":1,"libraryId":"00000000-0000-4000-8000-000000000099"}"#,
    )
    .unwrap();
    let connection = Connection::open(root.database()).unwrap();
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .unwrap();
    connection.execute_batch(MIGRATIONS[0].sql).unwrap();
    let library_id = "00000000-0000-4000-8000-000000000099";
    connection
        .execute("INSERT INTO library_identity VALUES(1,?1)", [library_id])
        .unwrap();
    connection
        .execute(
            "INSERT INTO schema_migrations VALUES(1,?1,'v1')",
            [checksum(MIGRATIONS[0].sql)],
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 1).unwrap();
    let fixtures = [
        ("00000000-0000-4000-8000-000000000101", "NEW", None),
        ("00000000-0000-4000-8000-000000000105", "ACTIVE", None),
        (
            "00000000-0000-4000-8000-000000000102",
            "ARCHIVED",
            Some("NEW"),
        ),
        ("00000000-0000-4000-8000-000000000103", "COMPLETED", None),
        (
            "00000000-0000-4000-8000-000000000104",
            "ARCHIVED",
            Some("COMPLETED"),
        ),
    ];
    let mut expected = Vec::new();
    connection
        .execute(
            "INSERT INTO venues(id,name,kind,identifier) VALUES('00000000-0000-4000-8000-000000000301','Journal fixture','journal','ISSN 0000-0000')",
            [],
        )
        .unwrap();
    for (index, (paper_id, lifecycle, archived_from)) in fixtures.into_iter().enumerate() {
        let doc = format!("00000000-0000-4000-8000-{:012}", 201 + index);
        let author = format!("00000000-0000-4000-8000-{:012}", 401 + index);
        let title = format!("Fixture {index}");
        let revision = (index + 7) as i64;
        let updated = format!("updated-{index}");
        connection.execute("INSERT INTO papers(id,title,review_type,domain,venue_id,lifecycle,archived_from_lifecycle,revision,created_at,updated_at,last_opened_at) VALUES(?1,?2,'survey','water','00000000-0000-4000-8000-000000000301',?3,?4,?5,'created',?6,'last-opened')",rusqlite::params![paper_id,title,lifecycle,archived_from,revision,updated]).unwrap();
        connection
            .execute(
                "INSERT INTO authors(id,display_name,orcid) VALUES(?1,?2,?3)",
                rusqlite::params![
                    author,
                    format!("Author {index}"),
                    format!("0000-0000-0000-{index:04}")
                ],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO paper_authors(paper_id,author_id,author_order) VALUES(?1,?2,0)",
                rusqlite::params![paper_id, author],
            )
            .unwrap();
        connection.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,'fixture.pdf',?3,?4,'application/pdf',0,'imported','MISSING')",rusqlite::params![doc,paper_id,format!("documents/{doc}/original.pdf"),"d".repeat(64)]).unwrap();
        connection.execute("INSERT INTO reading_positions(document_id,page_index,zoom,revision,updated_at) VALUES(?1,?2,1.5,?3,'position-updated')",rusqlite::params![doc,index as i64 + 2,index as i64 + 9]).unwrap();
        connection
            .execute(
                "UPDATE papers SET active_document_id=?2 WHERE id=?1",
                rusqlite::params![paper_id, doc],
            )
            .unwrap();
        expected.push((paper_id.into(), title, lifecycle.into(), revision, updated));
    }
    connection
        .execute(
            "UPDATE app_session SET last_opened_paper_id='00000000-0000-4000-8000-000000000101' WHERE singleton=1",
            [],
        )
        .unwrap();
    let user_data_before = library_user_data_snapshot(&connection).unwrap();
    (dir, root, connection, expected, user_data_before)
}

#[test]
fn migration_0002_backfills_populated_v1_without_changing_library_records_and_reopens() {
    let (_dir, root, mut connection, expected, user_data_before) = populated_v1_library();
    migrate(&mut connection, &root, MIGRATIONS).unwrap();
    assert_eq!(
        library_user_data_snapshot(&connection).unwrap(),
        user_data_before
    );
    assert_eq!(
        research_workbench_core::adapters::sqlite::migrations::schema_version(&connection).unwrap(),
        2
    );
    for (id, title, lifecycle, revision, updated) in &expected {
        let actual:(String,String,i64,String,Option<String>,String,i64,String,String,String)=connection.query_row("SELECT p.title,p.lifecycle,p.revision,p.updated_at,p.last_opened_at,d.id,d.size_bytes,d.status,d.sha256,d.relative_path FROM papers p JOIN documents d ON d.id=p.active_document_id WHERE p.id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?))).unwrap();
        assert_eq!(
            (&actual.0, &actual.1, actual.2, &actual.3),
            (title, lifecycle, *revision, updated)
        );
        assert_eq!(actual.4.as_deref(), Some("last-opened"));
        assert_eq!(actual.6, 0);
        assert_eq!(actual.7, "MISSING");
        assert_eq!(actual.8, "d".repeat(64));
        assert!(actual.9.starts_with("documents/"));
        let author_count: i64 = connection.query_row(
            "SELECT count(*) FROM paper_authors pa JOIN authors a ON a.id=pa.author_id WHERE pa.paper_id=?1 AND pa.author_order=0 AND a.display_name LIKE 'Author %'",
            [id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(author_count, 1);
        let venue: (String, String, Option<String>) = connection.query_row(
            "SELECT v.name,v.kind,v.identifier FROM papers p JOIN venues v ON v.id=p.venue_id WHERE p.id=?1",
            [id],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?)),
        ).unwrap();
        assert_eq!(
            venue,
            (
                "Journal fixture".into(),
                "journal".into(),
                Some("ISSN 0000-0000".into())
            )
        );
        let position: (i64, f64, i64, String) = connection.query_row(
            "SELECT rp.page_index,rp.zoom,rp.revision,rp.updated_at FROM reading_positions rp JOIN documents d ON d.id=rp.document_id WHERE d.paper_id=?1",
            [id],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)),
        ).unwrap();
        assert_eq!(
            position.0,
            if id.ends_with("101") {
                2
            } else if id.ends_with("105") {
                3
            } else if id.ends_with("102") {
                4
            } else if id.ends_with("103") {
                5
            } else {
                6
            }
        );
        assert_eq!(position.1, 1.5);
        assert!(position.2 >= 9 && position.2 <= 13);
        assert_eq!(position.3, "position-updated");
        let phases: i64 = connection
            .query_row(
                "SELECT count(*) FROM paper_phases WHERE paper_id=?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(phases, 3);
        let context: (i64, String) = connection
            .query_row(
                "SELECT processing_initialized,current_phase FROM papers WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(context, (1, "PRE".into()));
    }
    assert_eq!(connection.query_row("SELECT archived_from_lifecycle FROM papers WHERE id='00000000-0000-4000-8000-000000000102'",[],|r|r.get::<_,Option<String>>(0)).unwrap().as_deref(),Some("NEW"));
    assert_eq!(connection.query_row("SELECT archived_from_lifecycle FROM papers WHERE id='00000000-0000-4000-8000-000000000104'",[],|r|r.get::<_,Option<String>>(0)).unwrap().as_deref(),Some("COMPLETED"));
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM operation_receipts", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT last_opened_paper_id FROM app_session WHERE singleton=1",
                [],
                |row| row.get::<_, Option<String>>(0)
            )
            .unwrap()
            .as_deref(),
        Some("00000000-0000-4000-8000-000000000101")
    );
    let backups = fs::read_dir(root.backups().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 1);
    let backup_db = Connection::open(backups[0].join("research.sqlite")).unwrap();
    assert_eq!(
        research_workbench_core::adapters::sqlite::migrations::schema_version(&backup_db).unwrap(),
        1
    );
    assert_eq!(
        backup_db
            .query_row("SELECT count(*) FROM papers", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        expected.len() as i64
    );
    assert_eq!(
        library_user_data_snapshot(&backup_db).unwrap(),
        user_data_before
    );
    drop(backup_db);
    drop(connection);
    let reopened = Connection::open(root.database()).unwrap();
    assert_eq!(
        research_workbench_core::adapters::sqlite::migrations::schema_version(&reopened).unwrap(),
        2
    );
    research_workbench_core::adapters::sqlite::migrations::verify_integrity(&reopened).unwrap();
    assert_eq!(
        library_user_data_snapshot(&reopened).unwrap(),
        user_data_before
    );
}

#[test]
fn failed_0002_rolls_back_ddl_seed_backfill_ledger_and_user_version_after_backup() {
    let (_dir, root, mut connection, expected, user_data_before) = populated_v1_library();
    let failed_sql = format!("{}\nTHIS IS AN INJECTED FAILURE;", MIGRATIONS[1].sql);
    let steps = [
        Migration {
            version: 1,
            sql: MIGRATIONS[0].sql,
        },
        Migration {
            version: 2,
            sql: &failed_sql,
        },
    ];
    assert!(migrate(&mut connection, &root, &steps).is_err());
    assert_eq!(
        library_user_data_snapshot(&connection).unwrap(),
        user_data_before
    );
    assert_eq!(
        research_workbench_core::adapters::sqlite::migrations::schema_version(&connection).unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM schema_migrations", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert!(
        connection
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='paper_phases'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .optional()
            .unwrap()
            .is_none()
    );
    for (id, title, lifecycle, revision, updated) in &expected {
        let actual:(String,String,i64,String,i64,String)=connection.query_row("SELECT p.title,p.lifecycle,p.revision,p.updated_at,p.processing_initialized,p.active_document_id FROM papers p WHERE p.id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).unwrap();
        assert_eq!(
            (&actual.0, &actual.1, actual.2, &actual.3, actual.4),
            (title, lifecycle, *revision, updated, 0)
        );
        assert!(!actual.5.is_empty());
    }
    let backups = fs::read_dir(root.backups().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 1);
    let backup = &backups[0];
    assert!(backup.join("library.json").exists());
    assert!(backup.join("manifest.json").exists());
    let backup_db = Connection::open(backup.join("research.sqlite")).unwrap();
    assert_eq!(
        research_workbench_core::adapters::sqlite::migrations::schema_version(&backup_db).unwrap(),
        1
    );
    assert_eq!(
        library_user_data_snapshot(&backup_db).unwrap(),
        user_data_before
    );
    drop(backup_db);
    drop(connection);
    let reopened = Connection::open(root.database()).unwrap();
    assert_eq!(
        research_workbench_core::adapters::sqlite::migrations::schema_version(&reopened).unwrap(),
        1
    );
    assert_eq!(
        library_user_data_snapshot(&reopened).unwrap(),
        user_data_before
    );
}

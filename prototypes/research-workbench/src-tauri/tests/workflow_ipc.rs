use research_workbench_core::{
    adapters::sqlite::workflow_repository::SqliteWorkflowRepository,
    adapters::{
        sqlite::{
            actor::DbActor, settings::ActorSettingsQuery,
            workflow_repository::SqliteWorkflowPersistence,
        },
        windows::paths::LibraryRoot,
    },
    application::{
        library_ports::RecoveryReport, settings::RecoveryStatus, unit_of_work::with_transaction,
        workflow::initialize_processing,
    },
    desktop::{
        lifecycle::{DesktopState, SafeLogger},
        maintenance::MaintenanceCoordinator,
    },
    modules::workflow::service::WorkflowService,
    transport::commands::dispatch,
};
use std::sync::Arc;

fn state(dir: &tempfile::TempDir) -> (DesktopState, DbActor) {
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let maintenance = MaintenanceCoordinator::default();
    let recovery = RecoveryStatus::pending();
    recovery.finish(Ok(RecoveryReport {
        recovered: 0,
        issues: vec![],
    }));
    let service = Arc::new(WorkflowService::new(
        Arc::new(SqliteWorkflowPersistence::new(actor.clone())),
        maintenance.clone(),
        recovery,
        Default::default(),
        Arc::new(
            research_workbench_core::adapters::documents::store::LocalDocumentStore::new(
                actor.library_root().clone(),
                actor.info().library_id.0.clone(),
            )
            .unwrap(),
        ),
    ));
    let state = DesktopState {
        settings: Arc::new(ActorSettingsQuery::new(actor.clone(), maintenance.clone())),
        actor: actor.clone(),
        maintenance,
        logger: Arc::new(SafeLogger::new(dir.path().join("logs")).unwrap()),
        exit: Default::default(),
        library: std::sync::Mutex::new(None),
        reader: std::sync::Mutex::new(None),
        workflow: std::sync::Mutex::new(Some(service)),
    };
    (state, actor)
}

#[tokio::test]
async fn definition_command_uses_dispatcher_and_returns_pinned_definition() {
    let dir = tempfile::tempdir().unwrap();
    let (state, actor) = state(&dir);
    let result = dispatch(
        "main",
        "workflow_get_phase_definition",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000001","phaseCode":"PRE","version":1
        }),
        &state,
    )
    .await;
    assert_eq!(result["ok"], true);
    assert_eq!(result["data"]["code"], "PRE");
    assert_eq!(result["data"]["version"], 1);
    assert_eq!(
        result["data"]["requiredOutputs"].as_array().unwrap().len(),
        6
    );
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn p3_candidate_dispatch_remains_capability_gated() {
    let dir = tempfile::tempdir().unwrap();
    let (state, actor) = state(&dir);
    let summary = dispatch("main", "workflow_get_p3_candidate_summary", serde_json::json!({
        "requestId":"00000000-0000-4000-8000-000000000002","paperId":"00000000-0000-4000-8000-000000000003"
    }), &state).await;
    assert_eq!(summary["ok"], false);
    assert_eq!(summary["error"]["code"], "UnsupportedCapability");
    let set = dispatch(
        "main",
        "workflow_set_p3_candidate",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000006","paperId":"00000000-0000-4000-8000-000000000003",
            "itemId":"00000000-0000-4000-8000-000000000007","selected":true,"priority":1,
            "rationale":"synthetic candidate","noCandidatesJustification":null,"expectedWorkflowRevision":1
        }),
        &state,
    )
    .await;
    assert_eq!(set["ok"], false, "{set}");
    assert_eq!(set["error"]["code"], "UnsupportedCapability", "{set}");
    let receipts = actor
        .submit(|connection| {
            Ok(connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id IN ('00000000-0000-4000-8000-000000000002','00000000-0000-4000-8000-000000000006')",
                [],
                |row| row.get::<_, i64>(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(receipts, 0);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn save_answer_persists_with_cas_replay_and_phase_clock() {
    let dir = tempfile::tempdir().unwrap();
    let (state, actor) = state(&dir);
    let paper_id = uuid::Uuid::new_v4().to_string();
    let id_for_init = paper_id.clone();
    actor.submit(move |connection| {
        connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'IPC fixture','survey','now','now')", [&id_for_init])?;
        with_transaction(connection, |tx| initialize_processing(tx, &SqliteWorkflowRepository, &id_for_init))
    }).await.unwrap();
    let request_id = "00000000-0000-4000-8000-000000000004";
    let args = serde_json::json!({
        "requestId":request_id,"paperId":paper_id,"phaseCode":"PRE","questionKey":"purpose",
        "expectedRevision":0,"answerText":"delimitar incertidumbre","structuredValue":null,
        "resolution":"ANSWERED","explanation":null
    });
    let first = dispatch("main", "workflow_save_phase_answer", args.clone(), &state).await;
    assert_eq!(first["ok"], true);
    assert_eq!(first["data"]["revision"], 1);
    let answers=dispatch("main","workflow_get_phase_answers",serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000012","paperId":paper_id,"phaseCode":"PRE"}),&state).await;
    assert_eq!(answers["ok"], true);
    assert_eq!(answers["data"].as_array().unwrap().len(), 1);
    assert_eq!(answers["data"][0]["questionKey"], "purpose");
    let replay = dispatch("main", "workflow_save_phase_answer", args, &state).await;
    assert_eq!(replay["data"], first["data"]);
    let no_op = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000013","paperId":paper_id,
            "phaseCode":"PRE","questionKey":"purpose","expectedRevision":1,
            "answerText":"delimitar incertidumbre","structuredValue":null,
            "resolution":"ANSWERED","explanation":null
        }),
        &state,
    )
    .await;
    assert_eq!(no_op["ok"], true, "canonical no-op: {no_op}");
    assert_eq!(no_op["data"]["updatedAt"], first["data"]["updatedAt"]);
    let paper_for_audit = paper_id.clone();
    let answer_audit = actor
        .submit(move |connection| {
            Ok((
                connection.query_row(
                    "SELECT count(*) FROM audit_events WHERE action='workflow.answer_changed' AND entity_id=?1",
                    [&paper_for_audit],
                    |row| row.get::<_, i64>(0),
                )?,
                connection.query_row(
                    "SELECT revision FROM paper_phases WHERE paper_id=?1 AND phase_code='PRE'",
                    [&paper_for_audit],
                    |row| row.get::<_, i64>(0),
                )?,
            ))
        })
        .await
        .unwrap();
    assert_eq!(answer_audit, (1, 2));
    let stale_no_op = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000014","paperId":paper_id,
            "phaseCode":"PRE","questionKey":"purpose","expectedRevision":0,
            "answerText":"delimitar incertidumbre","structuredValue":null,
            "resolution":"ANSWERED","explanation":null
        }),
        &state,
    )
    .await;
    assert_eq!(stale_no_op["ok"], false);
    assert_eq!(stale_no_op["error"]["code"], "Conflict");
    let stale_receipt = actor
        .submit(|connection| {
            Ok(connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id='00000000-0000-4000-8000-000000000014'",
                [],
                |row| row.get::<_, i64>(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(stale_receipt, 0);
    let phase = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000005","paperId":paper_id,"phaseCode":"PRE"
        }),
        &state,
    )
    .await;
    assert_eq!(phase["data"]["revision"], 2);
    assert_eq!(phase["data"]["state"], "IN_PROGRESS");
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn save_answer_to_not_started_phase_is_blocked_without_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let (state, actor) = state(&dir);
    let paper_id = uuid::Uuid::new_v4().to_string();
    let id_for_init = paper_id.clone();
    actor.submit(move |connection| {
        connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'IPC fixture','survey','now','now')", [&id_for_init])?;
        with_transaction(connection, |tx| initialize_processing(tx, &SqliteWorkflowRepository, &id_for_init))
    }).await.unwrap();
    let result = dispatch("main", "workflow_save_phase_answer", serde_json::json!({
        "requestId":"00000000-0000-4000-8000-000000000006","paperId":paper_id,"phaseCode":"P1","questionKey":"scope",
        "expectedRevision":0,"answerText":"draft","structuredValue":null,"resolution":"ANSWERED","explanation":null
    }), &state).await;
    assert_eq!(result["ok"], false);
    assert_eq!(result["error"]["code"], "GateBlocked");
    assert_eq!(
        actor
            .submit(|connection| Ok(connection.query_row(
                "SELECT count(*) FROM operation_receipts",
                [],
                |row| row.get::<_, i64>(0)
            )?))
            .await
            .unwrap(),
        0
    );
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn evaluate_gate_uses_live_document_proof_and_does_not_write() {
    let dir = tempfile::tempdir().unwrap();
    let (state, actor) = state(&dir);
    let paper_id = uuid::Uuid::new_v4().to_string();
    let document_id = uuid::Uuid::new_v4().to_string();
    let bytes = b"%PDF-1.4 synthetic gate fixture";
    let document_dir = actor
        .library_root()
        .path()
        .join("documents")
        .join(&document_id);
    std::fs::create_dir_all(&document_dir).unwrap();
    std::fs::write(document_dir.join("original.pdf"), bytes).unwrap();
    let paper_for_insert = paper_id.clone();
    let document_for_insert = document_id.clone();
    actor.submit(move |connection| {
        connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'IPC fixture','survey','now','now')", [&paper_for_insert])?;
        connection.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,'fixture.pdf',?3,?4,'application/pdf',?5,'now','ACTIVE')", rusqlite::params![document_for_insert,paper_for_insert,format!("documents/{document_for_insert}/original.pdf"),"a".repeat(64),bytes.len() as i64])?;
        connection.execute("UPDATE papers SET active_document_id=?2 WHERE id=?1", rusqlite::params![paper_for_insert,document_for_insert])?;
        with_transaction(connection, |tx| initialize_processing(tx, &SqliteWorkflowRepository, &paper_for_insert))?;
        for (key,text,structured) in [("purpose","purpose",None),("uncertainty_target","uncertainty",None),("baseline","baseline",None),("expected_outcome","outcome",None),("desired_depth","depth",None),("review_type","",Some(r#"{"reviewType":"survey"}"#))] {
            tx_insert_answer(connection,&paper_for_insert,"PRE",key,text,structured)?;
        }
        Ok(())
    }).await.unwrap();
    let before = actor.submit(|connection| Ok((connection.query_row("SELECT state,revision,accepted_gate_snapshot_json FROM paper_phases WHERE phase_code='PRE'", [], |r| Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,Option<String>>(2)?)))?, connection.query_row("SELECT count(*) FROM operation_receipts",[],|r|r.get::<_,i64>(0))?))).await.unwrap();
    let result = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000009","paperId":paper_id,"phaseCode":"PRE"
        }),
        &state,
    )
    .await;
    assert_eq!(result["ok"], true, "{result}");
    assert_eq!(result["data"]["complete"], true);
    assert_eq!(result["data"]["phaseRevision"], before.0.1);
    assert_eq!(
        result["data"]["inputSnapshotHash"].as_str().unwrap().len(),
        64
    );
    let after = actor.submit(|connection| Ok((connection.query_row("SELECT state,revision,accepted_gate_snapshot_json FROM paper_phases WHERE phase_code='PRE'", [], |r| Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,Option<String>>(2)?)))?, connection.query_row("SELECT count(*) FROM operation_receipts",[],|r|r.get::<_,i64>(0))?))).await.unwrap();
    assert_eq!(after, before);
    assert_eq!(after.1, 0);
    let advance_args = serde_json::json!({
        "requestId":"00000000-0000-4000-8000-000000000010","paperId":paper_id,"fromPhase":"PRE","expectedPhaseRevision":1
    });
    let advanced = dispatch(
        "main",
        "workflow_advance_phase",
        advance_args.clone(),
        &state,
    )
    .await;
    assert_eq!(advanced["ok"], true, "{advanced}");
    assert_eq!(advanced["data"]["phase"]["state"], "COMPLETED");
    assert_eq!(advanced["data"]["paperLifecycle"], "ACTIVE");
    assert_eq!(advanced["data"]["nextPhase"], "P1");
    let p1_activation = actor.submit({
        let paper_id = paper_id.clone();
        move |connection| {
            Ok(connection.query_row(
                "SELECT p.current_phase,pp.state,pp.revision FROM papers p JOIN paper_phases pp ON pp.paper_id=p.id AND pp.phase_code='P1' WHERE p.id=?1",
                [paper_id],
                |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,i64>(2)?)),
            )?)
        }
    }).await.unwrap();
    assert_eq!(p1_activation, ("P1".into(), "IN_PROGRESS".into(), 3));
    let acceptance_event = actor
        .submit({
            let paper_id = paper_id.clone();
            move |connection| {
                Ok(connection.query_row(
                    "SELECT changes_json FROM audit_events WHERE request_id='00000000-0000-4000-8000-000000000010' AND action='workflow.phase_accepted' AND entity_id=?1",
                    [&paper_id],
                    |row| row.get::<_, String>(0),
                )?)
            }
        })
        .await
        .unwrap();
    let acceptance_event: serde_json::Value = serde_json::from_str(&acceptance_event).unwrap();
    assert_eq!(acceptance_event["phaseCode"], "PRE");
    assert_eq!(acceptance_event["before"]["state"], "IN_PROGRESS");
    assert_eq!(acceptance_event["after"]["state"], "COMPLETED");
    assert_eq!(acceptance_event["contextAfter"], "P1");
    assert_eq!(acceptance_event["destinationBefore"]["phaseCode"], "P1");
    assert_eq!(
        acceptance_event["destinationBefore"]["state"],
        "NOT_STARTED"
    );
    assert_eq!(acceptance_event["destinationBefore"]["revision"], 0);
    assert_eq!(acceptance_event["destinationAfter"]["phaseCode"], "P1");
    assert_eq!(acceptance_event["destinationAfter"]["state"], "IN_PROGRESS");
    assert_eq!(acceptance_event["destinationAfter"]["revision"], 3);
    let touched=dispatch("main","workflow_touch_phase",serde_json::json!({
        "requestId":"00000000-0000-4000-8000-000000000011","paperId":paper_id,"phaseCode":"P1","expectedActivePhaseRevision":3
    }),&state).await;
    assert_eq!(touched["ok"], true, "{touched}");
    assert_eq!(touched["data"]["state"], "IN_PROGRESS");
    let context_events = actor
        .submit({
            let paper_id = paper_id.clone();
            move |connection| {
                Ok(connection.query_row(
                    "SELECT count(*) FROM audit_events WHERE entity_id=?1 AND action='workflow.context_changed'",
                    [&paper_id],
                    |row| row.get::<_, i64>(0),
                )?)
            }
        })
        .await
        .unwrap();
    assert_eq!(context_events, 0, "touch to current phase is a no-op");
    std::fs::remove_file(document_dir.join("original.pdf")).unwrap();
    let archive_paper = paper_id.clone();
    actor.submit(move |connection| {
        connection.execute("UPDATE papers SET lifecycle='ARCHIVED',archived_from_lifecycle='ACTIVE',revision=revision+1 WHERE id=?1", [&archive_paper])?;
        Ok(())
    }).await.unwrap();
    let replay = dispatch("main", "workflow_advance_phase", advance_args, &state).await;
    assert_eq!(replay["ok"], true);
    assert_eq!(replay["data"], advanced["data"]);
    let id = paper_id.clone();
    let durable=actor.submit(move|connection|{
        Ok(connection.query_row(
            "SELECT p.lifecycle,(SELECT accepted_gate_snapshot_hash FROM paper_phases WHERE phase_code='PRE' AND paper_id=p.id),(SELECT count(*) FROM operation_receipts WHERE command='workflow_advance_phase') FROM papers p WHERE p.id=?1",
            [id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,Option<String>>(1)?,r.get::<_,i64>(2)?))
        )?)
    }).await.unwrap();
    assert_eq!(durable.0, "ARCHIVED");
    assert_eq!(durable.1.unwrap().len(), 64);
    assert_eq!(durable.2, 1);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

async fn accepted_pre_fixture() -> (tempfile::TempDir, DesktopState, DbActor, String) {
    let dir = tempfile::tempdir().unwrap();
    let (state, actor) = state(&dir);
    let paper_id = uuid::Uuid::new_v4().to_string();
    let document_id = uuid::Uuid::new_v4().to_string();
    let bytes = b"%PDF-1.4 synthetic P1 fixture";
    let document_dir = actor
        .library_root()
        .path()
        .join("documents")
        .join(&document_id);
    std::fs::create_dir_all(&document_dir).unwrap();
    std::fs::write(document_dir.join("original.pdf"), bytes).unwrap();
    let id = paper_id.clone();
    let doc = document_id.clone();
    actor.submit(move|connection|{
        connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'P1 fixture','survey','now','now')",[&id])?;
        connection.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,'fixture.pdf',?3,?4,'application/pdf',?5,'now','ACTIVE')",rusqlite::params![doc,id,format!("documents/{doc}/original.pdf"),"c".repeat(64),bytes.len() as i64])?;
        connection.execute("UPDATE papers SET active_document_id=?2 WHERE id=?1",rusqlite::params![id,doc])?;
        with_transaction(connection,|tx|initialize_processing(tx,&SqliteWorkflowRepository,&id))?;
        for (key,text,structured) in [("purpose","purpose",None),("uncertainty_target","uncertainty",None),("baseline","baseline",None),("expected_outcome","outcome",None),("desired_depth","depth",None),("review_type","",Some(r#"{"reviewType":"survey"}"#))]{tx_insert_answer(connection,&id,"PRE",key,text,structured)?;}
        Ok(())
    }).await.unwrap();
    let advanced=dispatch("main","workflow_advance_phase",serde_json::json!({"requestId":uuid::Uuid::new_v4().to_string(),"paperId":paper_id,"fromPhase":"PRE","expectedPhaseRevision":1}),&state).await;
    assert_eq!(advanced["ok"], true, "{advanced}");
    let paper = advanced["data"]["phase"]["paperId"]
        .as_str()
        .unwrap()
        .to_owned();
    let touched=dispatch("main","workflow_touch_phase",serde_json::json!({"requestId":uuid::Uuid::new_v4().to_string(),"paperId":paper,"phaseCode":"P1","expectedActivePhaseRevision":3}),&state).await;
    assert_eq!(touched["ok"], true, "{touched}");
    (dir, state, actor, paper)
}

#[tokio::test]
async fn unknown_review_type_gate_becomes_stale_when_paper_metadata_changes() {
    let dir = tempfile::tempdir().unwrap();
    let (state, actor) = state(&dir);
    let paper_id = uuid::Uuid::new_v4().to_string();
    let doc_id = uuid::Uuid::new_v4().to_string();
    let bytes = b"%PDF-1.4 synthetic unknown review fixture";
    let doc_dir = actor.library_root().path().join("documents").join(&doc_id);
    std::fs::create_dir_all(&doc_dir).unwrap();
    std::fs::write(doc_dir.join("original.pdf"), bytes).unwrap();
    let paper = paper_id.clone();
    let doc = doc_id.clone();
    actor.submit(move |connection| {
        connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'Unknown review fixture','unknown','now','now')", [&paper])?;
        connection.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,'fixture.pdf',?3,?4,'application/pdf',?5,'now','ACTIVE')",rusqlite::params![doc,paper,format!("documents/{doc}/original.pdf"),"d".repeat(64),bytes.len() as i64])?;
        connection.execute("UPDATE papers SET active_document_id=?2 WHERE id=?1",rusqlite::params![paper,doc])?;
        with_transaction(connection,|tx|initialize_processing(tx,&SqliteWorkflowRepository,&paper))?;
        for (key,text,structured) in [("purpose","purpose",None),("uncertainty_target","uncertainty",None),("baseline","baseline",None),("expected_outcome","outcome",None),("desired_depth","depth",None)] {
            tx_insert_answer(connection,&paper,"PRE",key,text,structured)?;
        }
        connection.execute("INSERT INTO phase_answers(paper_id,phase_code,question_key,answer_text,structured_value_json,resolution,explanation,revision,updated_at) VALUES(?1,'PRE','review_type','',?2,'UNKNOWN','The bibliographic type is unavailable.',1,'now')",rusqlite::params![paper,r#"{"reviewType":"unknown"}"#])?;
        Ok(())
    }).await.unwrap();
    let initial = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000031","paperId":paper_id,"phaseCode":"PRE"
        }),
        &state,
    )
    .await;
    assert_eq!(initial["ok"], true, "{initial}");
    assert_eq!(initial["data"]["complete"], true);
    let paper = paper_id.clone();
    actor
        .submit(move |connection| {
            connection.execute(
                "UPDATE papers SET review_type='survey' WHERE id=?1",
                [paper],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    let changed = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000032","paperId":paper_id,"phaseCode":"PRE"
        }),
        &state,
    )
    .await;
    assert_eq!(changed["ok"], true, "{changed}");
    assert_eq!(changed["data"]["complete"], false);
    assert_eq!(
        changed["data"]["issues"][0]["requirementKey"],
        "review_type"
    );
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn persisted_definition_pin_controls_workflow_and_other_versions_do_not_repin() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    let baseline = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000051","paperId":paper,"phaseCode":"P1"}),
        &state,
    ).await;
    assert_eq!(baseline["ok"], true, "{baseline}");
    assert_eq!(baseline["data"]["definitionVersion"], 1);
    let install = paper.clone();
    actor.submit(move |connection| {
        connection.execute(
            "INSERT INTO phase_definitions(code,version,payload_json,definition_hash) VALUES('P1',2,'{\"code\":\"P1\",\"version\":2,\"name\":\"unreleased\"}',?1)",
            ["f".repeat(64)],
        )?;
        Ok(())
    }).await.unwrap();
    let unchanged = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000052","paperId":paper,"phaseCode":"P1"}),
        &state,
    ).await;
    assert_eq!(unchanged["ok"], true, "{unchanged}");
    assert_eq!(unchanged["data"]["definitionVersion"], 1);
    assert_eq!(
        unchanged["data"]["inputSnapshotHash"],
        baseline["data"]["inputSnapshotHash"]
    );
    let phase = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000053","paperId":install,"phaseCode":"P1"}),
        &state,
    ).await;
    assert_eq!(phase["data"]["definitionVersion"], 1);
    let set_pin = paper.clone();
    actor.submit(move |connection| {
        connection.execute("UPDATE paper_phases SET definition_version=2 WHERE paper_id=?1 AND phase_code='P1'",[set_pin])?;
        Ok(())
    }).await.unwrap();
    let pinned_phase = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000054","paperId":paper,"phaseCode":"P1"}),
        &state,
    ).await;
    assert_eq!(pinned_phase["ok"], true, "{pinned_phase}");
    assert_eq!(pinned_phase["data"]["definitionVersion"], 2);
    for (command, args) in [
        (
            "workflow_save_phase_answer",
            serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000055","paperId":paper,"phaseCode":"P1","questionKey":"scope","expectedRevision":0,"answerText":"scope","structuredValue":null,"resolution":"ANSWERED","explanation":null}),
        ),
        (
            "workflow_evaluate_gate",
            serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000056","paperId":paper,"phaseCode":"P1"}),
        ),
        (
            "workflow_touch_phase",
            serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000057","paperId":paper,"phaseCode":"P1","expectedActivePhaseRevision":3}),
        ),
        (
            "workflow_advance_phase",
            serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000058","paperId":paper,"fromPhase":"P1","expectedPhaseRevision":3}),
        ),
    ] {
        let result = dispatch("main", command, args.clone(), &state).await;
        assert_eq!(result["ok"], false, "{command}: {result}");
        assert_eq!(
            result["error"]["code"], "UnsupportedCapability",
            "{command}: {result}"
        );
    }
    let receipts = actor.submit(|connection| Ok(connection.query_row("SELECT count(*) FROM operation_receipts WHERE request_id BETWEEN '00000000-0000-4000-8000-000000000055' AND '00000000-0000-4000-8000-000000000058'",[],|row|row.get::<_,i64>(0))?)).await.unwrap();
    assert_eq!(receipts, 0);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn durable_receipt_conflict_precedes_capability_and_new_p2_is_not_receipted() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    let saved = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000061","paperId":paper,"phaseCode":"PRE","questionKey":"purpose","expectedRevision":1,"answerText":"purpose","structuredValue":null,"resolution":"ANSWERED","explanation":null}),
        &state,
    ).await;
    assert_eq!(saved["ok"], true, "{saved}");
    let save_conflict = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000061","paperId":paper,"phaseCode":"P2","questionKey":"main_questions_processed","expectedRevision":0,"answerText":"future","structuredValue":null,"resolution":"ANSWERED","explanation":null}),
        &state,
    ).await;
    assert_eq!(
        save_conflict["error"]["code"], "Conflict",
        "{save_conflict}"
    );
    actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                for (key, text, structured) in [
                    ("scope", "scope", None),
                    ("out_of_scope", "out", None),
                    ("review_type", "method", None),
                    ("literature_cutoff", "cutoff", None),
                    ("core_message", "message", None),
                    (
                        "relevance_decision",
                        "",
                        Some(r#"{"relevance":"sufficient","readingDecision":"light_read"}"#),
                    ),
                ] {
                    tx_insert_answer(connection, &paper, "P1", key, text, structured)?;
                }
                Ok(())
            }
        })
        .await
        .unwrap();
    let advanced = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000062","paperId":paper,"fromPhase":"P1","expectedPhaseRevision":3}),
        &state,
    ).await;
    assert_eq!(advanced["ok"], true, "{advanced}");
    let doc_id = actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                Ok(connection.query_row(
                    "SELECT active_document_id FROM papers WHERE id=?1",
                    [paper],
                    |row| row.get::<_, String>(0),
                )?)
            }
        })
        .await
        .unwrap();
    std::fs::remove_file(
        actor
            .library_root()
            .path()
            .join("documents")
            .join(doc_id)
            .join("original.pdf"),
    )
    .unwrap();
    let conflict = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000062","paperId":paper,"fromPhase":"P2","expectedPhaseRevision":3}),
        &state,
    ).await;
    assert_eq!(conflict["error"]["code"], "Conflict", "{conflict}");
    let unsupported = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000063","paperId":paper,"fromPhase":"P2","expectedPhaseRevision":3}),
        &state,
    ).await;
    assert_eq!(
        unsupported["error"]["code"], "UnsupportedCapability",
        "{unsupported}"
    );
    let p2_save = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000064","paperId":paper,"phaseCode":"P2","questionKey":"main_questions_processed","expectedRevision":0,"answerText":"future","structuredValue":null,"resolution":"ANSWERED","explanation":null}),
        &state,
    ).await;
    assert_eq!(
        p2_save["error"]["code"], "UnsupportedCapability",
        "{p2_save}"
    );
    let p2_evaluation = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000065","paperId":paper,"phaseCode":"P2"}),
        &state,
    )
    .await;
    assert_eq!(p2_evaluation["error"]["code"], "UnsupportedCapability");
    let p2_evaluation_receipts = actor
        .submit(|connection| {
            Ok(connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id='00000000-0000-4000-8000-000000000065'",
                [],
                |row| row.get::<_, i64>(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(p2_evaluation_receipts, 0);
    let receipts = actor.submit(|connection| Ok(connection.query_row("SELECT count(*) FROM operation_receipts WHERE request_id IN ('00000000-0000-4000-8000-000000000063','00000000-0000-4000-8000-000000000064')",[],|row|row.get::<_,i64>(0))?)).await.unwrap();
    assert_eq!(receipts, 0);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn go_back_and_touch_enforce_safe_revision_range_without_receipts() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    for (index, command, args) in [
        (
            0,
            "workflow_go_back_to_phase",
            serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000071","paperId":paper,"targetPhase":"PRE","expectedActivePhaseRevision":9007199254740992_i64}),
        ),
        (
            1,
            "workflow_touch_phase",
            serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000072","paperId":paper,"phaseCode":"P1","expectedActivePhaseRevision":9007199254740992_i64}),
        ),
        (
            2,
            "workflow_go_back_to_phase",
            serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000073","paperId":paper,"targetPhase":"PRE","expectedActivePhaseRevision":-1}),
        ),
        (
            3,
            "workflow_touch_phase",
            serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000074","paperId":paper,"phaseCode":"P1","expectedActivePhaseRevision":-1}),
        ),
    ] {
        let result = dispatch("main", command, args, &state).await;
        assert_eq!(result["ok"], false, "case {index}: {result}");
        assert_eq!(
            result["error"]["code"], "InvalidInput",
            "case {index}: {result}"
        );
    }
    let max_revision = dispatch(
        "main",
        "workflow_go_back_to_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000075","paperId":paper,"targetPhase":"PRE","expectedActivePhaseRevision":9007199254740991_i64}),
        &state,
    ).await;
    assert_eq!(
        max_revision["error"]["code"], "Conflict",
        "MAX is a valid token and reaches CAS: {max_revision}"
    );
    let durable = actor.submit({
        let paper = paper.clone();
        move |connection| Ok((
            connection.query_row("SELECT current_phase FROM papers WHERE id=?1",[&paper],|row|row.get::<_,String>(0))?,
            connection.query_row("SELECT state,revision,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 AND phase_code='P1'",[&paper],|row|Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,Option<String>>(2)?)))?,
            connection.query_row("SELECT count(*) FROM operation_receipts WHERE request_id BETWEEN '00000000-0000-4000-8000-000000000071' AND '00000000-0000-4000-8000-000000000075'",[],|row|row.get::<_,i64>(0))?,
        ))
    }).await.unwrap();
    assert_eq!(durable.0, "P1");
    assert_eq!(durable.1.0, "IN_PROGRESS");
    assert_eq!(durable.1.1, 3);
    assert_eq!(durable.2, 0);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn workflow_clock_overflow_rolls_back_answer_invalidation_and_receipt() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    let paper_for_seed = paper.clone();
    actor
        .submit(move |connection| {
            connection.execute(
                "UPDATE paper_phases SET revision=9007199254740991 WHERE paper_id=?1 AND phase_code='P2'",
                [&paper_for_seed],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    let before = actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                Ok((
                    connection.query_row(
                        "SELECT answer_text,revision,updated_at FROM phase_answers WHERE paper_id=?1 AND phase_code='PRE' AND question_key='purpose'",
                        [&paper],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
                    )?,
                    {
                        let mut statement = connection.prepare(
                            "SELECT phase_code,state,revision,completed_at,accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 AND phase_code IN ('PRE','P1') ORDER BY phase_code",
                        )?;
                        let rows = statement.query_map([&paper], |row| {
                            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, Option<String>>(3)?, row.get::<_, Option<String>>(4)?, row.get::<_, Option<String>>(5)?))
                        })?;
                        rows.collect::<Result<Vec<_>, _>>()?
                    },
                    connection.query_row(
                        "SELECT current_phase FROM papers WHERE id=?1",
                        [&paper],
                        |row| row.get::<_, String>(0),
                    )?,
                ))
            }
        })
        .await
        .unwrap();
    let request_id = "00000000-0000-4000-8000-000000000076";
    let result = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({
            "requestId":request_id,"paperId":paper,"phaseCode":"PRE","questionKey":"purpose",
            "expectedRevision":1,"answerText":"changed before overflow","structuredValue":null,
            "resolution":"ANSWERED","explanation":null
        }),
        &state,
    )
    .await;
    assert_eq!(result["ok"], false, "overflow: {result}");
    assert_eq!(result["error"]["code"], "Conflict", "overflow: {result}");
    let after = actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                Ok((
                    connection.query_row(
                        "SELECT answer_text,revision,updated_at FROM phase_answers WHERE paper_id=?1 AND phase_code='PRE' AND question_key='purpose'",
                        [&paper],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
                    )?,
                    {
                        let mut statement = connection.prepare(
                            "SELECT phase_code,state,revision,completed_at,accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 AND phase_code IN ('PRE','P1') ORDER BY phase_code",
                        )?;
                        let rows = statement.query_map([&paper], |row| {
                            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, Option<String>>(3)?, row.get::<_, Option<String>>(4)?, row.get::<_, Option<String>>(5)?))
                        })?;
                        rows.collect::<Result<Vec<_>, _>>()?
                    },
                    connection.query_row(
                        "SELECT current_phase FROM papers WHERE id=?1",
                        [&paper],
                        |row| row.get::<_, String>(0),
                    )?,
                    connection.query_row(
                        "SELECT count(*) FROM operation_receipts WHERE request_id=?1",
                        [request_id],
                        |row| row.get::<_, i64>(0),
                    )?,
                ))
            }
        })
        .await
        .unwrap();
    assert_eq!(after.0, before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(after.2, before.2);
    assert_eq!(after.3, 0);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn completed_reserved_paper_rejects_workflow_mutations_without_effects() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    let paper_for_seed = paper.clone();
    actor
        .submit(move |connection| {
            connection.execute(
                "UPDATE papers SET lifecycle='COMPLETED' WHERE id=?1",
                [&paper_for_seed],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    let before = actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                Ok((
                    connection.query_row(
                        "SELECT lifecycle,current_phase,revision FROM papers WHERE id=?1",
                        [&paper],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?)),
                    )?,
                    {
                        let mut statement = connection.prepare(
                            "SELECT phase_code,state,revision,accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 ORDER BY phase_code",
                        )?;
                        let rows = statement.query_map([&paper], |row| {
                            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, Option<String>>(3)?, row.get::<_, Option<String>>(4)?))
                        })?;
                        rows.collect::<Result<Vec<_>, _>>()?
                    },
                    connection.query_row(
                        "SELECT answer_text,revision,updated_at FROM phase_answers WHERE paper_id=?1 AND phase_code='PRE' AND question_key='purpose'",
                        [&paper],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
                    )?,
                ))
            }
        })
        .await
        .unwrap();
    let requests = [
        (
            "workflow_save_phase_answer",
            serde_json::json!({
                "requestId":"00000000-0000-4000-8000-000000000077","paperId":paper,"phaseCode":"PRE","questionKey":"purpose",
                "expectedRevision":1,"answerText":"must not be written","structuredValue":null,"resolution":"ANSWERED","explanation":null
            }),
        ),
        (
            "workflow_advance_phase",
            serde_json::json!({
                "requestId":"00000000-0000-4000-8000-000000000078","paperId":paper,"fromPhase":"P1","expectedPhaseRevision":3
            }),
        ),
        (
            "workflow_go_back_to_phase",
            serde_json::json!({
                "requestId":"00000000-0000-4000-8000-000000000079","paperId":paper,"targetPhase":"PRE","expectedActivePhaseRevision":3
            }),
        ),
        (
            "workflow_touch_phase",
            serde_json::json!({
                "requestId":"00000000-0000-4000-8000-000000000080","paperId":paper,"phaseCode":"P1","expectedActivePhaseRevision":3
            }),
        ),
    ];
    for (command, args) in requests {
        let result = dispatch("main", command, args, &state).await;
        assert_eq!(result["ok"], false, "{command}: {result}");
        assert_eq!(
            result["error"]["code"], "UnsupportedCapability",
            "{command}: {result}"
        );
    }
    let after = actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                Ok((
                    connection.query_row(
                        "SELECT lifecycle,current_phase,revision FROM papers WHERE id=?1",
                        [&paper],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?)),
                    )?,
                    {
                        let mut statement = connection.prepare(
                            "SELECT phase_code,state,revision,accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 ORDER BY phase_code",
                        )?;
                        let rows = statement.query_map([&paper], |row| {
                            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, Option<String>>(3)?, row.get::<_, Option<String>>(4)?))
                        })?;
                        rows.collect::<Result<Vec<_>, _>>()?
                    },
                    connection.query_row(
                        "SELECT answer_text,revision,updated_at FROM phase_answers WHERE paper_id=?1 AND phase_code='PRE' AND question_key='purpose'",
                        [&paper],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
                    )?,
                    connection.query_row(
                        "SELECT count(*) FROM operation_receipts WHERE request_id BETWEEN '00000000-0000-4000-8000-000000000077' AND '00000000-0000-4000-8000-000000000080'",
                        [],
                        |row| row.get::<_, i64>(0),
                    )?,
                ))
            }
        })
        .await
        .unwrap();
    assert_eq!(after.0, before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(after.2, before.2);
    assert_eq!(after.3, 0);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn p1_advance_continue_light_read_and_archive_are_distinct_atomic_branches() {
    for (index, decision, expected_lifecycle, expected_next, expected_context) in [
        (0, "continue", "ACTIVE", "P2", "P2"),
        (1, "light_read", "ACTIVE", "null", "P1"),
        (2, "archive", "ARCHIVED", "null", "P1"),
    ] {
        let (_dir, state, actor, paper) = accepted_pre_fixture().await;
        let paper_for_answers = paper.clone();
        actor
            .submit(move |connection| {
                let structured =
                    format!(r#"{{"relevance":"sufficient","readingDecision":"{decision}"}}"#);
                for (key, text, value) in [
                    ("scope", "scope", None),
                    ("out_of_scope", "out", None),
                    ("review_type", "method", None),
                    ("literature_cutoff", "cutoff", None),
                    ("core_message", "message", None),
                    ("relevance_decision", "", Some(structured.as_str())),
                ] {
                    tx_insert_answer(connection, &paper_for_answers, "P1", key, text, value)?;
                }
                Ok(())
            })
            .await
            .unwrap();
        let evaluated=dispatch("main","workflow_evaluate_gate",serde_json::json!({"requestId":uuid::Uuid::new_v4().to_string(),"paperId":paper,"phaseCode":"P1"}),&state).await;
        assert_eq!(evaluated["ok"], true, "branch {index}: {evaluated}");
        assert_eq!(
            evaluated["data"]["complete"], true,
            "branch {index}: {:?}",
            evaluated["data"]["issues"]
        );
        let result=dispatch("main","workflow_advance_phase",serde_json::json!({"requestId":uuid::Uuid::new_v4().to_string(),"paperId":paper,"fromPhase":"P1","expectedPhaseRevision":3}),&state).await;
        assert_eq!(result["ok"], true, "branch {index}: {result}");
        assert_eq!(result["data"]["paperLifecycle"], expected_lifecycle);
        assert_eq!(
            result["data"]["nextPhase"].as_str().unwrap_or("null"),
            expected_next
        );
        let id = result["data"]["phase"]["paperId"]
            .as_str()
            .unwrap()
            .to_owned();
        let context = actor
            .submit(move |connection| {
                Ok(connection.query_row(
                    "SELECT current_phase FROM papers WHERE id=?1",
                    [id],
                    |r| r.get::<_, String>(0),
                )?)
            })
            .await
            .unwrap();
        assert_eq!(context, expected_context);
        if expected_lifecycle == "ARCHIVED" {
            let id = result["data"]["phase"]["paperId"]
                .as_str()
                .unwrap()
                .to_owned();
            let from = actor
                .submit(move |connection| {
                    Ok(connection.query_row(
                        "SELECT archived_from_lifecycle FROM papers WHERE id=?1",
                        [id],
                        |r| r.get::<_, String>(0),
                    )?)
                })
                .await
                .unwrap();
            assert_eq!(from, "ACTIVE");
        }
        actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
    }
}

#[tokio::test]
async fn p1_archive_failure_rolls_back_phase_lifecycle_audit_and_receipt() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    actor.submit(|connection| {
        connection.execute_batch("CREATE TRIGGER fail_workflow_archive BEFORE INSERT ON audit_events WHEN NEW.action='paper.archived' BEGIN SELECT RAISE(ABORT,'injected workflow archive failure'); END;")?;
        Ok(())
    }).await.unwrap();
    actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                for (key, text, structured) in [
                    ("scope", "scope", None),
                    ("out_of_scope", "out", None),
                    ("review_type", "method", None),
                    ("literature_cutoff", "cutoff", None),
                    ("core_message", "message", None),
                    (
                        "relevance_decision",
                        "",
                        Some(r#"{"relevance":"sufficient","readingDecision":"archive"}"#),
                    ),
                ] {
                    tx_insert_answer(connection, &paper, "P1", key, text, structured)?;
                }
                Ok(())
            }
        })
        .await
        .unwrap();
    let request_id = "00000000-0000-4000-8000-000000000077";
    let paper_id = paper.clone();
    let result = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":request_id,"paperId":paper,"fromPhase":"P1","expectedPhaseRevision":3
        }),
        &state,
    )
    .await;
    assert_eq!(
        result["ok"], false,
        "injected archive failure must fail the whole P1 transition: {result}"
    );
    let durable = actor.submit(move |connection| Ok((
        connection.query_row("SELECT lifecycle,current_phase,revision FROM papers WHERE id=?1", [&paper_id], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,i64>(2)?)))?,
        connection.query_row("SELECT state,revision,completed_at,accepted_gate_snapshot_json FROM paper_phases WHERE paper_id=?1 AND phase_code='P1'", [&paper_id], |row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,Option<String>>(2)?,row.get::<_,Option<String>>(3)?)))?,
        connection.query_row("SELECT count(*) FROM operation_receipts WHERE request_id=?1", [request_id], |row| row.get::<_,i64>(0))?,
        connection.query_row("SELECT count(*) FROM audit_events WHERE request_id=?1 AND action IN ('workflow_advance_phase','paper.archived')", [request_id], |row| row.get::<_,i64>(0))?,
    ))).await.unwrap();
    assert_eq!(durable.0, ("ACTIVE".into(), "P1".into(), 1));
    assert_eq!(durable.1, ("IN_PROGRESS".into(), 3, None, None));
    assert_eq!(durable.2, 0);
    assert_eq!(durable.3, 0);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn answer_change_audit_failure_rolls_back_answer_clock_and_receipt() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    actor
        .submit(|connection| {
            connection.execute_batch("CREATE TRIGGER fail_workflow_answer_audit BEFORE INSERT ON audit_events WHEN NEW.action='workflow.answer_changed' BEGIN SELECT RAISE(ABORT,'injected workflow answer audit failure'); END;")?;
            Ok(())
        })
        .await
        .unwrap();
    let request_id = "00000000-0000-4000-8000-000000000078";
    let result = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({
            "requestId":request_id,"paperId":paper,"phaseCode":"PRE",
            "questionKey":"purpose","expectedRevision":1,"answerText":"changed",
            "structuredValue":null,"resolution":"ANSWERED","explanation":null
        }),
        &state,
    )
    .await;
    assert_eq!(
        result["ok"], false,
        "audit failure must abort save: {result}"
    );
    let durable = actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                Ok((
                    connection.query_row(
                        "SELECT answer_text,revision,updated_at FROM phase_answers WHERE paper_id=?1 AND phase_code='PRE' AND question_key='purpose'",
                        [&paper],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
                    )?,
                    connection.query_row(
                        "SELECT phase_code,state,revision,completed_at,accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 AND phase_code='PRE'",
                        [&paper],
                        |row| Ok((row.get::<_, String>(0)?,row.get::<_, String>(1)?,row.get::<_, i64>(2)?,row.get::<_, Option<String>>(3)?,row.get::<_, Option<String>>(4)?,row.get::<_, Option<String>>(5)?)),
                    )?,
                    connection.query_row("SELECT current_phase FROM papers WHERE id=?1", [&paper], |row| row.get::<_, String>(0))?,
                    connection.query_row("SELECT count(*) FROM operation_receipts WHERE request_id=?1", [request_id], |row| row.get::<_, i64>(0))?,
                    connection.query_row("SELECT count(*) FROM audit_events WHERE request_id=?1 AND action='workflow.answer_changed'", [request_id], |row| row.get::<_, i64>(0))?,
                ))
            }
        })
        .await
        .unwrap();
    assert_eq!(durable.0.0, "purpose");
    assert_eq!(durable.0.1, 1);
    assert_eq!(durable.1.1, "COMPLETED");
    assert_eq!(durable.1.2, 2);
    assert!(durable.1.3.is_some() && durable.1.4.is_some() && durable.1.5.is_some());
    assert_eq!(durable.2, "P1");
    assert_eq!(durable.3, 0);
    assert_eq!(durable.4, 0);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn phase_acceptance_and_receipt_failures_rollback_all_workflow_changes() {
    for (index, trigger) in [
        "CREATE TRIGGER injected BEFORE INSERT ON audit_events WHEN NEW.action='workflow.phase_accepted' BEGIN SELECT RAISE(ABORT,'injected acceptance event failure'); END;",
        "CREATE TRIGGER injected BEFORE INSERT ON operation_receipts WHEN NEW.command='workflow_advance_phase' BEGIN SELECT RAISE(ABORT,'injected receipt failure'); END;",
    ]
    .into_iter()
    .enumerate()
    {
        let (_dir, state, actor, paper) = accepted_pre_fixture().await;
        actor
            .submit(move |connection| {
                connection.execute_batch(trigger)?;
                Ok(())
            })
            .await
            .unwrap();
        let paper_for_answers = paper.clone();
        actor
            .submit(move |connection| {
                for (key, text, structured) in [
                    ("scope", "scope", None),
                    ("out_of_scope", "out", None),
                    ("review_type", "method", None),
                    ("literature_cutoff", "cutoff", None),
                    ("core_message", "message", None),
                    (
                        "relevance_decision",
                        "",
                        Some(r#"{"relevance":"sufficient","readingDecision":"continue"}"#),
                    ),
                ] {
                    tx_insert_answer(connection, &paper_for_answers, "P1", key, text, structured)?;
                }
                Ok(())
            })
            .await
            .unwrap();
        let request_id = format!("00000000-0000-4000-8000-00000000008{index}");
        let paper_for_request = paper.clone();
        let result = dispatch(
            "main",
            "workflow_advance_phase",
            serde_json::json!({
                "requestId":request_id,"paperId":paper,"fromPhase":"P1","expectedPhaseRevision":3
            }),
            &state,
        )
        .await;
        assert_eq!(result["ok"], false, "injected failure {index}: {result}");
        let durable = actor
            .submit(move |connection| {
                Ok((
                    connection.query_row(
                        "SELECT current_phase,lifecycle FROM papers WHERE id=?1",
                        [&paper_for_request],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                    )?,
                    connection.query_row(
                        "SELECT state,revision,completed_at,accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 AND phase_code='P1'",
                        [&paper_for_request],
                        |row| Ok((row.get::<_, String>(0)?,row.get::<_, i64>(1)?,row.get::<_, Option<String>>(2)?,row.get::<_, Option<String>>(3)?,row.get::<_, Option<String>>(4)?)),
                    )?,
                    connection.query_row("SELECT count(*) FROM operation_receipts WHERE request_id=?1", [&request_id], |row| row.get::<_, i64>(0))?,
                    connection.query_row("SELECT count(*) FROM audit_events WHERE request_id=?1 AND action IN ('workflow.phase_accepted','workflow_advance_phase')", [&request_id], |row| row.get::<_, i64>(0))?,
                    connection.query_row("SELECT state,revision FROM paper_phases WHERE paper_id=?1 AND phase_code='P2'", [&paper_for_request], |row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?)))?,
                ))
            })
            .await
            .unwrap();
        assert_eq!(durable.0, ("P1".into(), "ACTIVE".into()));
        assert_eq!(durable.1.0, "IN_PROGRESS");
        assert_eq!(durable.1.1, 3);
        assert!(durable.1.2.is_none() && durable.1.3.is_none() && durable.1.4.is_none());
        assert_eq!(durable.2, 0);
        assert_eq!(durable.3, 0);
        assert_eq!(durable.4, ("NOT_STARTED".into(), 0));
        actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
    }
}

#[tokio::test]
async fn editing_started_prior_phase_keeps_later_active_context_and_invalidates_cas() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    let saved=dispatch("main","workflow_save_phase_answer",serde_json::json!({"requestId":uuid::Uuid::new_v4().to_string(),"paperId":paper,"phaseCode":"PRE","questionKey":"purpose","expectedRevision":1,"answerText":"revised purpose","structuredValue":null,"resolution":"ANSWERED","explanation":null}),&state).await;
    assert_eq!(saved["ok"], true, "{saved}");
    let id = saved["data"]["paperId"].as_str().unwrap().to_owned();
    let context =
        actor
            .submit(move |connection| {
                Ok(connection.query_row(
                    "SELECT current_phase FROM papers WHERE id=?1",
                    [id],
                    |r| r.get::<_, String>(0),
                )?)
            })
            .await
            .unwrap();
    assert_eq!(context, "P1");
    let id = saved["data"]["paperId"].as_str().unwrap().to_owned();
    let states=actor.submit(move|connection|{let mut statement=connection.prepare("SELECT phase_code,state,revision FROM paper_phases WHERE paper_id=?1 ORDER BY CASE phase_code WHEN 'PRE' THEN 0 WHEN 'P1' THEN 1 ELSE 2 END")?;let rows=statement.query_map([id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?)))?;rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)}).await.unwrap();
    assert_eq!(
        states,
        vec![
            ("PRE".into(), "NEEDS_REVIEW".into(), 4),
            ("P1".into(), "NEEDS_REVIEW".into(), 5),
            ("P2".into(), "NOT_STARTED".into(), 0)
        ]
    );
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn reaccepting_pre_reactivates_existing_p1_without_resetting_its_data() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                for (key, text, structured) in [
                    ("scope", "scope", None),
                    ("out_of_scope", "out", None),
                    ("review_type", "method", None),
                    ("literature_cutoff", "cutoff", None),
                    ("core_message", "message", None),
                    (
                        "relevance_decision",
                        "",
                        Some(r#"{"relevance":"sufficient","readingDecision":"light_read"}"#),
                    ),
                ] {
                    tx_insert_answer(connection, &paper, "P1", key, text, structured)?;
                }
                Ok(())
            }
        })
        .await
        .unwrap();
    let completed_p1 = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000041","paperId":paper,"fromPhase":"P1","expectedPhaseRevision":3
        }),
        &state,
    ).await;
    assert_eq!(completed_p1["ok"], true, "{completed_p1}");
    let before = actor.submit({
        let paper = paper.clone();
        move |connection| Ok(connection.query_row(
            "SELECT state,revision,accepted_gate_snapshot_json,accepted_gate_snapshot_hash,(SELECT count(*) FROM phase_answers WHERE paper_id=?1 AND phase_code='P1') FROM paper_phases WHERE paper_id=?1 AND phase_code='P1'",
            [paper],
            |row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,Option<String>>(2)?,row.get::<_,Option<String>>(3)?,row.get::<_,i64>(4)?)),
        )?)
    }).await.unwrap();
    assert_eq!(before.0, "COMPLETED");
    assert!(before.2.is_some() && before.3.is_some());
    assert_eq!(before.4, 6);
    let saved = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000042","paperId":paper,"phaseCode":"PRE","questionKey":"purpose","expectedRevision":1,"answerText":"updated purpose","structuredValue":null,"resolution":"ANSWERED","explanation":null
        }),
        &state,
    ).await;
    assert_eq!(saved["ok"], true, "{saved}");
    let active_revision = actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                Ok(connection.query_row(
                    "SELECT revision FROM paper_phases WHERE paper_id=?1 AND phase_code='P1'",
                    [paper],
                    |row| row.get::<_, i64>(0),
                )?)
            }
        })
        .await
        .unwrap();
    let back = dispatch(
        "main",
        "workflow_go_back_to_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000043","paperId":paper,"targetPhase":"PRE","expectedActivePhaseRevision":active_revision
        }),
        &state,
    ).await;
    assert_eq!(back["ok"], true, "{back}");
    let accepted = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000044","paperId":paper,"fromPhase":"PRE","expectedPhaseRevision":back["data"]["activePhaseRevision"]
        }),
        &state,
    ).await;
    assert_eq!(
        accepted["ok"], true,
        "reaccepting PRE should retain an existing P1: {accepted}"
    );
    assert_eq!(accepted["data"]["nextPhase"], "P1");
    let acceptance_event = actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                Ok(connection.query_row(
                    "SELECT changes_json FROM audit_events WHERE request_id='00000000-0000-4000-8000-000000000044' AND action='workflow.phase_accepted' AND entity_id=?1",
                    [&paper],
                    |row| row.get::<_, String>(0),
                )?)
            }
        })
        .await
        .unwrap();
    let acceptance_event: serde_json::Value = serde_json::from_str(&acceptance_event).unwrap();
    assert_eq!(acceptance_event["destinationBefore"]["phaseCode"], "P1");
    assert_eq!(
        acceptance_event["destinationBefore"]["state"],
        "NEEDS_REVIEW"
    );
    assert_eq!(
        acceptance_event["destinationBefore"]["revision"],
        active_revision
    );
    assert_eq!(acceptance_event["destinationAfter"]["phaseCode"], "P1");
    assert_eq!(
        acceptance_event["destinationAfter"]["state"],
        "NEEDS_REVIEW"
    );
    assert!(
        acceptance_event["destinationAfter"]["revision"]
            .as_i64()
            .unwrap()
            > active_revision
    );
    let after = actor.submit({
        let paper = paper.clone();
        move |connection| Ok((
            connection.query_row("SELECT current_phase FROM papers WHERE id=?1", [&paper], |row| row.get::<_,String>(0))?,
            connection.query_row("SELECT state,revision,accepted_gate_snapshot_json,accepted_gate_snapshot_hash,(SELECT count(*) FROM phase_answers WHERE paper_id=?1 AND phase_code='P1') FROM paper_phases WHERE paper_id=?1 AND phase_code='P1'", [&paper], |row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,Option<String>>(2)?,row.get::<_,Option<String>>(3)?,row.get::<_,i64>(4)?)))?,
        ))
    }).await.unwrap();
    assert_eq!(after.0, "P1");
    assert_eq!(after.1.0, "NEEDS_REVIEW");
    assert!(after.1.1 > before.1);
    assert_eq!(after.1.2, before.2);
    assert_eq!(after.1.3, before.3);
    assert_eq!(after.1.4, before.4);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

fn tx_insert_answer(
    connection: &mut rusqlite::Connection,
    paper_id: &str,
    phase: &str,
    key: &str,
    text: &str,
    structured: Option<&str>,
) -> Result<(), research_workbench_core::transport::error::AppError> {
    connection.execute("INSERT INTO phase_answers(paper_id,phase_code,question_key,answer_text,structured_value_json,resolution,explanation,revision,updated_at) VALUES(?1,?2,?3,?4,?5,'ANSWERED',NULL,1,'now')",rusqlite::params![paper_id,phase,key,text,structured])?;
    Ok(())
}

type WorkflowMutationSnapshot = (
    String,
    Option<String>,
    Vec<(
        String,
        String,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
    )>,
    i64,
);

fn workflow_mutation_snapshot(
    connection: &rusqlite::Connection,
    paper_id: &str,
) -> Result<WorkflowMutationSnapshot, rusqlite::Error> {
    let paper = connection.query_row(
        "SELECT lifecycle,current_phase FROM papers WHERE id=?1",
        [paper_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let mut statement = connection.prepare(
        "SELECT phase_code,state,revision,completed_at,accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 ORDER BY CASE phase_code WHEN 'PRE' THEN 0 WHEN 'P1' THEN 1 ELSE 2 END",
    )?;
    let rows = statement.query_map([paper_id], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
        ))
    })?;
    let phases = rows.collect::<Result<Vec<_>, _>>()?;
    let acceptances = connection.query_row(
        "SELECT count(*) FROM audit_events WHERE entity_id=?1 AND action='workflow.phase_accepted'",
        [paper_id],
        |row| row.get(0),
    )?;
    Ok((paper.0, paper.1, phases, acceptances))
}

#[tokio::test]
async fn advance_rejects_unsupported_p1_destination_pin_before_accepting_pre() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    let p1 = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000081","paperId":paper,"phaseCode":"P1"}),
        &state,
    )
    .await;
    let back = dispatch(
        "main",
        "workflow_go_back_to_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000082","paperId":paper,
            "targetPhase":"PRE","expectedActivePhaseRevision":p1["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(back["ok"], true, "go back to PRE: {back}");
    actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                connection.execute(
                    "INSERT INTO phase_definitions(code,version,payload_json,definition_hash) VALUES('P1',2,'{\"code\":\"P1\",\"version\":2,\"name\":\"unreleased\"}',?1)",
                    ["f".repeat(64)],
                )?;
                connection.execute(
                    "UPDATE paper_phases SET definition_version=2 WHERE paper_id=?1 AND phase_code='P1'",
                    [&paper],
                )?;
                Ok(())
            }
        })
        .await
        .unwrap();
    let before = actor
        .submit({
            let paper = paper.clone();
            move |connection| Ok(workflow_mutation_snapshot(connection, &paper)?)
        })
        .await
        .unwrap();
    let rejected = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000083","paperId":paper,
            "fromPhase":"PRE","expectedPhaseRevision":back["data"]["activePhaseRevision"]
        }),
        &state,
    )
    .await;
    assert_eq!(
        rejected["error"]["code"], "UnsupportedCapability",
        "{rejected}"
    );
    let after = actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                Ok((
                    workflow_mutation_snapshot(connection, &paper)?,
                    connection.query_row(
                        "SELECT count(*) FROM operation_receipts WHERE request_id='00000000-0000-4000-8000-000000000083'",
                        [],
                        |row| row.get::<_, i64>(0),
                    )?,
                ))
            }
        })
        .await
        .unwrap();
    assert_eq!(after.0, before);
    assert_eq!(after.1, 0);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn advance_rejects_unsupported_p2_destination_pin_before_accepting_p1() {
    let (_dir, state, actor, paper) = accepted_pre_fixture().await;
    let paper_for_seed = paper.clone();
    actor
        .submit(move |connection| {
            for (key, text, structured) in [
                ("scope", "scope", None),
                ("out_of_scope", "out", None),
                ("review_type", "method", None),
                ("literature_cutoff", "cutoff", None),
                ("core_message", "message", None),
                (
                    "relevance_decision",
                    "",
                    Some(r#"{"relevance":"sufficient","readingDecision":"continue"}"#),
                ),
            ] {
                tx_insert_answer(connection, &paper_for_seed, "P1", key, text, structured)?;
            }
            connection.execute(
                "INSERT INTO phase_definitions(code,version,payload_json,definition_hash) VALUES('P2',2,'{\"code\":\"P2\",\"version\":2,\"name\":\"unreleased\"}',?1)",
                ["f".repeat(64)],
            )?;
            connection.execute(
                "UPDATE paper_phases SET definition_version=2 WHERE paper_id=?1 AND phase_code='P2'",
                [&paper_for_seed],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    let before = actor
        .submit({
            let paper = paper.clone();
            move |connection| Ok(workflow_mutation_snapshot(connection, &paper)?)
        })
        .await
        .unwrap();
    let rejected = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000084","paperId":paper,
            "fromPhase":"P1","expectedPhaseRevision":3
        }),
        &state,
    )
    .await;
    assert_eq!(
        rejected["error"]["code"], "UnsupportedCapability",
        "{rejected}"
    );
    let after = actor
        .submit({
            let paper = paper.clone();
            move |connection| {
                Ok((
                    workflow_mutation_snapshot(connection, &paper)?,
                    connection.query_row(
                        "SELECT count(*) FROM operation_receipts WHERE request_id='00000000-0000-4000-8000-000000000084'",
                        [],
                        |row| row.get::<_, i64>(0),
                    )?,
                ))
            }
        })
        .await
        .unwrap();
    assert_eq!(after.0, before);
    assert_eq!(after.1, 0);
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn go_back_preserves_phase_data_and_forward_touch_requires_accepted_chain() {
    let dir = tempfile::tempdir().unwrap();
    let (state, actor) = state(&dir);
    let paper_id = uuid::Uuid::new_v4().to_string();
    let document_id = uuid::Uuid::new_v4().to_string();
    let id_for_init = paper_id.clone();
    let doc_for_init = document_id.clone();
    actor.submit(move |connection| {
        connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'IPC fixture','survey','now','now')", [&id_for_init])?;
        connection.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,'missing.pdf',?3,?4,'application/pdf',0,'now','MISSING')",rusqlite::params![doc_for_init,id_for_init,format!("documents/{doc_for_init}/original.pdf"),"b".repeat(64)])?;
        connection.execute("UPDATE papers SET active_document_id=?2 WHERE id=?1",rusqlite::params![id_for_init,doc_for_init])?;
        with_transaction(connection, |tx| initialize_processing(tx, &SqliteWorkflowRepository, &id_for_init))?;
        connection.execute("UPDATE papers SET current_phase='P1' WHERE id=?1", [&id_for_init])?;
        connection.execute("UPDATE paper_phases SET state='IN_PROGRESS' WHERE paper_id=?1 AND phase_code='P1'", [&id_for_init])?;
        Ok(())
    }).await.unwrap();
    let back = dispatch("main", "workflow_go_back_to_phase", serde_json::json!({
        "requestId":"00000000-0000-4000-8000-000000000007","paperId":paper_id,"targetPhase":"PRE","expectedActivePhaseRevision":0
    }), &state).await;
    assert_eq!(back["ok"], true);
    assert_eq!(back["data"]["activePhaseCode"], "PRE");
    assert_eq!(back["data"]["activePhaseRevision"], 2);
    let forward = dispatch("main", "workflow_touch_phase", serde_json::json!({
        "requestId":"00000000-0000-4000-8000-000000000008","paperId":paper_id,"phaseCode":"P1","expectedActivePhaseRevision":2
    }), &state).await;
    assert_eq!(forward["ok"], false);
    assert_eq!(forward["error"]["code"], "GateBlocked");
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}

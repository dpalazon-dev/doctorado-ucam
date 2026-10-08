use research_workbench_core::{
    adapters::{
        documents::store::LocalDocumentStore,
        sqlite::{
            actor::DbActor, library_repository::SqliteLibraryPersistence,
            reader_repository::SqliteReaderPersistence, settings::ActorSettingsQuery,
            workflow_repository::SqliteWorkflowPersistence,
        },
        windows::paths::LibraryRoot,
    },
    application::{
        library_ports::RecoveryReport,
        library_ports::{LibraryFuture, NativePdfSelection, SelectedPdf},
        request_registry::RequestRegistry,
        settings::RecoveryStatus,
    },
    desktop::{
        lifecycle::{DesktopState, SafeLogger},
        maintenance::MaintenanceCoordinator,
    },
    modules::{
        library::service::LibraryService, reader::service::ReaderService,
        workflow::service::WorkflowService,
    },
    transport::commands::dispatch,
};
use std::{path::PathBuf, sync::Arc, time::Duration};

struct FixedPdfPicker(PathBuf);

fn valid_pdf() -> Vec<u8> {
    let mut bytes = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for object in [
        b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".as_slice(),
        b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] >>\nendobj\n".as_slice(),
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << >> >>\nendobj\n".as_slice(),
        b"4 0 obj\n<< /Subject (synthetic T04b fixture) >>\nendobj\n".as_slice(),
    ] {
        offsets.push(bytes.len());
        bytes.extend_from_slice(object);
    }
    let xref = bytes.len();
    bytes.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
    for offset in offsets {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!("trailer\n<< /Size 5 /Root 1 0 R /Info 4 0 R >>\nstartxref\n{xref}\n%%EOF\n")
            .as_bytes(),
    );
    bytes
}

impl NativePdfSelection for FixedPdfPicker {
    fn select_pdf(&self) -> LibraryFuture<'_, Option<SelectedPdf>> {
        let path = self.0.clone();
        Box::pin(async move {
            Ok(Some(SelectedPdf {
                path,
                filename: "synthetic-study.pdf".into(),
            }))
        })
    }
}

fn state(dir: &tempfile::TempDir, actor: DbActor, pdf: PathBuf) -> DesktopState {
    let maintenance = MaintenanceCoordinator::default();
    let requests = RequestRegistry::default();
    let recovery = RecoveryStatus::pending();
    recovery.finish(Ok(RecoveryReport {
        recovered: 0,
        issues: vec![],
    }));
    let library_id = actor.info().library_id.0.clone();
    let documents = Arc::new(
        LocalDocumentStore::new(actor.library_root().clone(), library_id.clone()).unwrap(),
    );
    let library = Arc::new(
        LibraryService::new(
            Arc::new(FixedPdfPicker(pdf)),
            documents.clone(),
            Arc::new(SqliteLibraryPersistence::new(
                actor.clone(),
                library_id.clone(),
            )),
            maintenance.clone(),
            requests.clone(),
        )
        .with_recovery_status(recovery.clone()),
    );
    let reader = Arc::new(ReaderService::new(
        Arc::new(SqliteReaderPersistence::new(
            actor.clone(),
            library_id.clone(),
        )),
        documents.clone(),
        maintenance.clone(),
        requests.clone(),
        recovery.clone(),
    ));
    let workflow = Arc::new(WorkflowService::new(
        Arc::new(SqliteWorkflowPersistence::new(actor.clone())),
        maintenance.clone(),
        recovery,
        requests,
        documents,
    ));
    DesktopState {
        settings: Arc::new(ActorSettingsQuery::new(actor.clone(), maintenance.clone())),
        actor,
        maintenance,
        logger: Arc::new(SafeLogger::new(dir.path().join("logs")).unwrap()),
        exit: Default::default(),
        library: std::sync::Mutex::new(Some(library)),
        reader: std::sync::Mutex::new(Some(reader)),
        workflow: std::sync::Mutex::new(Some(workflow)),
    }
}

#[tokio::test]
async fn imported_paper_flows_through_reader_and_pre_p1_workflow_services() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("synthetic.pdf");
    let pdf_bytes = valid_pdf();
    std::fs::write(&pdf, &pdf_bytes).unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let state = state(&dir, actor.clone(), pdf);
    let selected = dispatch(
        "main",
        "library_select_pdf",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000201"}),
        &state,
    )
    .await;
    assert_eq!(selected["ok"], true, "select: {selected}");
    let imported = dispatch(
        "main",
        "library_confirm_import",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000202",
            "importToken":selected["data"]["importToken"],
            "metadata":{"title":"Synthetic paper","authors":["A. Author"],"year":2025,"doi":null,"venue":"Fixture Journal","reviewType":"survey","domain":"water"}
        }),
        &state,
    )
    .await;
    assert_eq!(imported["ok"], true, "import: {imported}");
    let paper_id = imported["data"]["id"].as_str().unwrap().to_owned();
    let opened = dispatch(
        "main",
        "reader_open_paper",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000203","paperId":paper_id
        }),
        &state,
    )
    .await;
    assert_eq!(opened["ok"], true, "reader: {opened}");

    for (index, (key, text, structured)) in [
        ("purpose", "purpose", serde_json::Value::Null),
        ("uncertainty_target", "uncertainty", serde_json::Value::Null),
        ("baseline", "baseline", serde_json::Value::Null),
        ("expected_outcome", "outcome", serde_json::Value::Null),
        ("desired_depth", "depth", serde_json::Value::Null),
        (
            "review_type",
            "",
            serde_json::json!({"reviewType":"survey"}),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let saved = dispatch(
            "main",
            "workflow_save_phase_answer",
            serde_json::json!({
                "requestId":format!("00000000-0000-4000-8000-0000000002{:02}",index+4),
                "paperId":paper_id,"phaseCode":"PRE","questionKey":key,
                "expectedRevision":0,"answerText":text,
                "structuredValue":structured,"resolution":"ANSWERED","explanation":null
            }),
            &state,
        )
        .await;
        assert_eq!(saved["ok"], true, "PRE {key}: {saved}");
    }
    let paper_metadata = dispatch(
        "main",
        "library_get_paper",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000210","paperId":paper_id}),
        &state,
    )
    .await;
    assert_eq!(paper_metadata["ok"], true, "paper read: {paper_metadata}");
    let metadata = |title: &str, authors: Vec<&str>, review_type: &str| {
        serde_json::json!({
            "title":title,"authors":authors,"year":2025,"doi":null,
            "venue":"Fixture Journal","reviewType":review_type,"domain":"water"
        })
    };
    let phase_before_authors = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000221","paperId":paper_id,"phaseCode":"PRE"}),
        &state,
    )
    .await;
    let authors_only = dispatch(
        "main",
        "library_update_metadata",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000222","paperId":paper_id,
            "expectedRevision":paper_metadata["data"]["revision"],
            "metadata":metadata("Synthetic paper",vec!["B. Author"],"survey")
        }),
        &state,
    )
    .await;
    assert_eq!(
        authors_only["ok"], true,
        "authors-only update: {authors_only}"
    );
    let phase_after_authors = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000223","paperId":paper_id,"phaseCode":"PRE"}),
        &state,
    )
    .await;
    assert_eq!(
        phase_after_authors["data"]["revision"],
        phase_before_authors["data"]["revision"]
    );
    let preview = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000224","paperId":paper_id,"phaseCode":"PRE"}),
        &state,
    )
    .await;
    assert_eq!(preview["ok"], true, "PRE preview: {preview}");
    assert_eq!(preview["data"]["complete"], true);
    let title_changed = dispatch(
        "main",
        "library_update_metadata",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000225","paperId":paper_id,
            "expectedRevision":authors_only["data"]["revision"],
            "metadata":metadata("Changed title",vec!["B. Author"],"survey")
        }),
        &state,
    )
    .await;
    assert_eq!(title_changed["ok"], true, "title update: {title_changed}");
    let title_preview = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000236","paperId":paper_id,"phaseCode":"PRE"}),
        &state,
    )
    .await;
    assert_eq!(title_preview["ok"], true, "title preview: {title_preview}");
    assert_ne!(
        title_preview["data"]["inputSnapshotHash"], preview["data"]["inputSnapshotHash"],
        "bibliographic title is part of the effective workflow input"
    );
    let stale_advance = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000226","paperId":paper_id,
            "fromPhase":"PRE","expectedPhaseRevision":preview["data"]["phaseRevision"]
        }),
        &state,
    )
    .await;
    assert_eq!(
        stale_advance["ok"], false,
        "stale PRE token: {stale_advance}"
    );
    assert_eq!(stale_advance["error"]["code"], "Conflict");
    let unknown_metadata = dispatch(
        "main",
        "library_update_metadata",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000232","paperId":paper_id,
            "expectedRevision":title_changed["data"]["revision"],
            "metadata":metadata("Changed title",vec!["B. Author"],"unknown")
        }),
        &state,
    )
    .await;
    assert_eq!(
        unknown_metadata["ok"], true,
        "review-type update: {unknown_metadata}"
    );
    let unknown_answer = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000233","paperId":paper_id,
            "phaseCode":"PRE","questionKey":"review_type","expectedRevision":1,
            "answerText":"","structuredValue":{"reviewType":"unknown"},
            "resolution":"UNKNOWN","explanation":"The bibliographic type is unavailable."
        }),
        &state,
    )
    .await;
    assert_eq!(
        unknown_answer["ok"], true,
        "unknown review type answer: {unknown_answer}"
    );
    let unknown_gate = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000234","paperId":paper_id,"phaseCode":"PRE"}),
        &state,
    )
    .await;
    assert_eq!(
        unknown_gate["ok"], true,
        "unknown review type gate: {unknown_gate}"
    );
    assert_eq!(unknown_gate["data"]["complete"], true);
    assert_ne!(
        unknown_gate["data"]["inputSnapshotHash"], title_preview["data"]["inputSnapshotHash"],
        "review type and its answer affect the effective workflow input"
    );
    let pre_phase = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000211","paperId":paper_id,"phaseCode":"PRE"}),
        &state,
    )
    .await;
    assert_eq!(pre_phase["data"]["state"], "IN_PROGRESS");
    let forward_before_pre_acceptance = dispatch(
        "main",
        "workflow_touch_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000213","paperId":paper_id,
            "phaseCode":"P1","expectedActivePhaseRevision":pre_phase["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(
        forward_before_pre_acceptance["error"]["code"], "GateBlocked",
        "sufficient but unaccepted PRE cannot activate P1: {forward_before_pre_acceptance}"
    );
    let before_pre_acceptance_receipts = actor
        .submit(|connection| {
            Ok(connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id='00000000-0000-4000-8000-000000000213'",
                [],
                |row| row.get::<_, i64>(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(before_pre_acceptance_receipts, 0);
    let pre_advance = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000212","paperId":paper_id,
            "fromPhase":"PRE","expectedPhaseRevision":pre_phase["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(pre_advance["ok"], true, "PRE advance: {pre_advance}");
    assert_eq!(pre_advance["data"]["nextPhase"], "P1");
    let pre_after_acceptance = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000235","paperId":paper_id,"phaseCode":"PRE"}),
        &state,
    )
    .await;
    assert_eq!(pre_after_acceptance["ok"], true);
    assert_eq!(
        pre_after_acceptance["data"]["inputSnapshotHash"],
        unknown_gate["data"]["inputSnapshotHash"]
    );

    for (index, (key, text, structured)) in [
        ("scope", "scope", serde_json::Value::Null),
        ("out_of_scope", "out of scope", serde_json::Value::Null),
        ("review_type", "method", serde_json::Value::Null),
        ("literature_cutoff", "2025", serde_json::Value::Null),
        ("core_message", "message", serde_json::Value::Null),
        (
            "relevance_decision",
            "",
            serde_json::json!({"relevance":"sufficient","readingDecision":"continue"}),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let saved = dispatch(
            "main",
            "workflow_save_phase_answer",
            serde_json::json!({
                "requestId":format!("00000000-0000-4000-8000-0000000002{:02}",index+13),
                "paperId":paper_id,"phaseCode":"P1","questionKey":key,
                "expectedRevision":0,"answerText":text,
                "structuredValue":structured,"resolution":"ANSWERED","explanation":null
            }),
            &state,
        )
        .await;
        assert_eq!(saved["ok"], true, "P1 {key}: {saved}");
    }
    let p1_phase = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000219","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    assert_eq!(p1_phase["ok"], true);
    let p1_answers = dispatch(
        "main",
        "workflow_get_phase_answers",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000236","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    let relevance_answer = p1_answers["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|answer| answer["questionKey"] == "relevance_decision")
        .unwrap();
    let reordered_no_op = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000237","paperId":paper_id,
            "phaseCode":"P1","questionKey":"relevance_decision",
            "expectedRevision":1,"answerText":"",
            "structuredValue":{"readingDecision":"continue","relevance":"sufficient"},
            "resolution":"ANSWERED","explanation":null
        }),
        &state,
    )
    .await;
    assert_eq!(
        reordered_no_op["ok"], true,
        "reordered JSON no-op: {reordered_no_op}"
    );
    assert_eq!(
        reordered_no_op["data"]["updatedAt"],
        relevance_answer["updatedAt"]
    );
    let p1_preview_before_navigation = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000238","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    assert_eq!(p1_preview_before_navigation["ok"], true);
    assert_eq!(p1_preview_before_navigation["data"]["complete"], true);
    let document_id = actor
        .submit({
            let paper_id = paper_id.clone();
            move |connection| {
                Ok(connection.query_row(
                    "SELECT active_document_id FROM papers WHERE id=?1",
                    [&paper_id],
                    |row| row.get::<_, String>(0),
                )?)
            }
        })
        .await
        .unwrap();
    let document_path = actor
        .library_root()
        .path()
        .join("documents")
        .join(document_id)
        .join("original.pdf");
    let p1_before_unavailable = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000231","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    assert_eq!(p1_before_unavailable["ok"], true);
    let available_preview = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000227","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    assert_eq!(
        available_preview["ok"], true,
        "available P1 preview: {available_preview}"
    );
    assert_eq!(available_preview["data"]["complete"], true);
    let forward_before_p1_acceptance = dispatch(
        "main",
        "workflow_touch_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000260","paperId":paper_id,
            "phaseCode":"P2","expectedActivePhaseRevision":p1_before_unavailable["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(
        forward_before_p1_acceptance["error"]["code"], "GateBlocked",
        "complete but unaccepted P1 cannot activate P2: {forward_before_p1_acceptance}"
    );
    let before_p1_acceptance_receipts = actor
        .submit(|connection| {
            Ok(connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id='00000000-0000-4000-8000-000000000260'",
                [],
                |row| row.get::<_, i64>(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(before_p1_acceptance_receipts, 0);
    let accepted_pre = actor
        .submit({
            let paper_id = paper_id.clone();
            move |connection| {
                Ok(connection.query_row(
                    "SELECT accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 AND phase_code='PRE'",
                    [&paper_id],
                    |row| Ok((row.get::<_,Option<String>>(0)?,row.get::<_,Option<String>>(1)?)),
                )?)
            }
        })
        .await
        .unwrap();

    std::fs::remove_file(&document_path).unwrap();
    let unavailable_preview = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000228","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    assert_eq!(
        unavailable_preview["ok"], true,
        "unavailable P1 preview: {unavailable_preview}"
    );
    assert_eq!(unavailable_preview["data"]["complete"], false);
    assert!(
        unavailable_preview["data"]["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["requirementKey"] == "paperHasActiveDocument")
    );
    assert_ne!(
        unavailable_preview["data"]["inputSnapshotHash"],
        available_preview["data"]["inputSnapshotHash"]
    );
    let unavailable_advance = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000229","paperId":paper_id,
            "fromPhase":"P1","expectedPhaseRevision":p1_before_unavailable["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(unavailable_advance["ok"], false);
    assert_eq!(
        unavailable_advance["error"]["code"], "GateBlocked",
        "unavailable document transition: {unavailable_advance}"
    );
    std::fs::write(&document_path, &pdf_bytes).unwrap();
    let recovered_preview = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000230","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    assert_eq!(
        recovered_preview["ok"], true,
        "recovered P1 preview: {recovered_preview}"
    );
    assert_eq!(
        recovered_preview["data"]["inputSnapshotHash"],
        available_preview["data"]["inputSnapshotHash"]
    );
    assert_eq!(recovered_preview["data"]["complete"], true);
    let workflow_after_unavailable = actor
        .submit({
            let paper_id = paper_id.clone();
            move |connection| {
                Ok((
                    connection.query_row(
                        "SELECT accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 AND phase_code='PRE'",
                        [&paper_id],
                        |row| Ok((row.get::<_,Option<String>>(0)?,row.get::<_,Option<String>>(1)?)),
                    )?,
                    connection.query_row(
                        "SELECT state,revision FROM paper_phases WHERE paper_id=?1 AND phase_code='P1'",
                        [&paper_id],
                        |row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?)),
                    )?,
                    connection.query_row(
                        "SELECT count(*) FROM operation_receipts WHERE request_id='00000000-0000-4000-8000-000000000229'",
                        [],
                        |row| row.get::<_,i64>(0),
                    )?,
                ))
            }
        })
        .await
        .unwrap();
    assert_eq!(workflow_after_unavailable.0, accepted_pre);
    assert_eq!(workflow_after_unavailable.1.0, "IN_PROGRESS");
    assert_eq!(
        workflow_after_unavailable.1.1,
        p1_before_unavailable["data"]["revision"]
    );
    assert_eq!(workflow_after_unavailable.2, 0);

    let back = dispatch(
        "main",
        "workflow_go_back_to_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000239","paperId":paper_id,
            "targetPhase":"PRE","expectedActivePhaseRevision":p1_phase["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(back["ok"], true, "back navigation: {back}");
    let retouch = dispatch(
        "main",
        "workflow_touch_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000240","paperId":paper_id,
            "phaseCode":"P1","expectedActivePhaseRevision":back["data"]["activePhaseRevision"]
        }),
        &state,
    )
    .await;
    assert_eq!(retouch["ok"], true, "forward navigation: {retouch}");
    let p1_preview_after_navigation = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000241","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    assert_eq!(p1_preview_after_navigation["ok"], true);
    assert_eq!(
        p1_preview_after_navigation["data"]["inputSnapshotHash"],
        p1_preview_before_navigation["data"]["inputSnapshotHash"]
    );
    let p1_advance = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000220","paperId":paper_id,
            "fromPhase":"P1","expectedPhaseRevision":retouch["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(p1_advance["ok"], true, "P1 advance: {p1_advance}");
    assert_eq!(p1_advance["data"]["nextPhase"], "P2");
    let paper_for_future_p2 = paper_id.clone();
    actor
        .submit(move |connection| {
            connection.execute(
                "INSERT INTO phase_answers(paper_id,phase_code,question_key,answer_text,structured_value_json,resolution,revision,updated_at) VALUES(?1,'P2','main_questions_processed','future fixture answer',NULL,'ANSWERED',1,'fixture-time')",
                [&paper_for_future_p2],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    let p2_phase = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000242","paperId":paper_id,"phaseCode":"P2"}),
        &state,
    )
    .await;
    let return_to_p1 = dispatch(
        "main",
        "workflow_go_back_to_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000243","paperId":paper_id,
            "targetPhase":"P1","expectedActivePhaseRevision":p2_phase["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(return_to_p1["ok"], true, "return to P1: {return_to_p1}");
    let p1_scope_edit = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000244","paperId":paper_id,
            "phaseCode":"P1","questionKey":"scope","expectedRevision":1,
            "answerText":"revised scope","structuredValue":null,
            "resolution":"ANSWERED","explanation":null
        }),
        &state,
    )
    .await;
    assert_eq!(p1_scope_edit["ok"], true, "P1 edit: {p1_scope_edit}");
    let p2_after_edit = actor
        .submit({
            let paper_id = paper_id.clone();
            move |connection| {
                Ok((
                    connection.query_row(
                        "SELECT state,revision FROM paper_phases WHERE paper_id=?1 AND phase_code='P2'",
                        [&paper_id],
                        |row| Ok((row.get::<_, String>(0)?,row.get::<_,i64>(1)?)),
                    )?,
                    connection.query_row(
                        "SELECT answer_text,revision,updated_at FROM phase_answers WHERE paper_id=?1 AND phase_code='P2' AND question_key='main_questions_processed'",
                        [&paper_id],
                        |row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,String>(2)?)),
                    )?,
                ))
            }
        })
        .await
        .unwrap();
    assert_eq!(p2_after_edit.0.0, "NEEDS_REVIEW");
    assert!(p2_after_edit.0.1 > p2_phase["data"]["revision"].as_i64().unwrap());
    assert_eq!(
        p2_after_edit.1,
        ("future fixture answer".into(), 1, "fixture-time".into())
    );
    let edited_p1_preview = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000252","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    assert_eq!(edited_p1_preview["data"]["complete"], true);
    let edited_p1_phase = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000253","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    let forward_after_p1_edit = dispatch(
        "main",
        "workflow_touch_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000254","paperId":paper_id,
            "phaseCode":"P2","expectedActivePhaseRevision":edited_p1_phase["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(
        forward_after_p1_edit["error"]["code"], "GateBlocked",
        "complete but NEEDS_REVIEW P1 cannot activate P2: {forward_after_p1_edit}"
    );
    let after_p1_edit_receipts = actor
        .submit(|connection| {
            Ok(connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id='00000000-0000-4000-8000-000000000254'",
                [],
                |row| row.get::<_, i64>(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(after_p1_edit_receipts, 0);
    let light_read = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000245","paperId":paper_id,
            "phaseCode":"P1","questionKey":"relevance_decision","expectedRevision":1,
            "answerText":"","structuredValue":{"relevance":"sufficient","readingDecision":"light_read"},
            "resolution":"ANSWERED","explanation":null
        }),
        &state,
    )
    .await;
    assert_eq!(light_read["ok"], true, "light-read decision: {light_read}");
    let p1_gate = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000246","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    assert_eq!(p1_gate["ok"], true);
    assert_eq!(p1_gate["data"]["complete"], true);
    assert_ne!(
        p1_gate["data"]["inputSnapshotHash"],
        p1_preview_before_navigation["data"]["inputSnapshotHash"],
        "effective answer changes alter the accepted input snapshot"
    );
    let p1_before_reaccept = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000247","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    let light_read_advance = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000248","paperId":paper_id,
            "fromPhase":"P1","expectedPhaseRevision":p1_before_reaccept["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(
        light_read_advance["ok"], true,
        "light-read acceptance: {light_read_advance}"
    );
    assert_eq!(
        light_read_advance["data"]["nextPhase"],
        serde_json::Value::Null
    );
    let paper_before_archive = dispatch(
        "main",
        "library_get_paper",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000249","paperId":paper_id}),
        &state,
    )
    .await;
    let archived = dispatch(
        "main",
        "library_archive_paper",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000250","paperId":paper_id,
            "expectedRevision":paper_before_archive["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(archived["ok"], true, "archive: {archived}");
    let restored = dispatch(
        "main",
        "library_restore_paper",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000251","paperId":paper_id,
            "expectedRevision":archived["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(restored["ok"], true, "restore: {restored}");
    let archive_decision = dispatch(
        "main",
        "workflow_save_phase_answer",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000261","paperId":paper_id,
            "phaseCode":"P1","questionKey":"relevance_decision","expectedRevision":2,
            "answerText":"","structuredValue":{"relevance":"sufficient","readingDecision":"archive"},
            "resolution":"ANSWERED","explanation":null
        }),
        &state,
    )
    .await;
    assert_eq!(
        archive_decision["ok"], true,
        "archive decision: {archive_decision}"
    );
    let archive_gate = dispatch(
        "main",
        "workflow_evaluate_gate",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000262","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    assert_eq!(archive_gate["data"]["complete"], true);
    let p1_before_archive_decision = dispatch(
        "main",
        "workflow_get_phase",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000263","paperId":paper_id,"phaseCode":"P1"}),
        &state,
    )
    .await;
    let archive_advance = dispatch(
        "main",
        "workflow_advance_phase",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000264","paperId":paper_id,
            "fromPhase":"P1","expectedPhaseRevision":p1_before_archive_decision["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(
        archive_advance["ok"], true,
        "archive decision accepted: {archive_advance}"
    );
    assert_eq!(archive_advance["data"]["paperLifecycle"], "ARCHIVED");
    let acceptance_count_before_restore = actor
        .submit({
            let paper_id = paper_id.clone();
            move |connection| {
                Ok(connection.query_row(
                    "SELECT count(*) FROM audit_events WHERE entity_id=?1 AND action='workflow.phase_accepted'",
                    [&paper_id],
                    |row| row.get::<_, i64>(0),
                )?)
            }
        })
        .await
        .unwrap();
    let archived_paper = dispatch(
        "main",
        "library_get_paper",
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000265","paperId":paper_id}),
        &state,
    )
    .await;
    assert_eq!(archived_paper["data"]["lifecycle"], "ARCHIVED");
    let restored_after_decision = dispatch(
        "main",
        "library_restore_paper",
        serde_json::json!({
            "requestId":"00000000-0000-4000-8000-000000000266","paperId":paper_id,
            "expectedRevision":archived_paper["data"]["revision"]
        }),
        &state,
    )
    .await;
    assert_eq!(
        restored_after_decision["ok"], true,
        "restore after P1 archive: {restored_after_decision}"
    );
    assert_eq!(restored_after_decision["data"]["lifecycle"], "ACTIVE");
    let acceptance_count_after_restore = actor
        .submit({
            let paper_id = paper_id.clone();
            move |connection| {
                Ok(connection.query_row(
                    "SELECT count(*) FROM audit_events WHERE entity_id=?1 AND action='workflow.phase_accepted'",
                    [&paper_id],
                    |row| row.get::<_, i64>(0),
                )?)
            }
        })
        .await
        .unwrap();
    assert_eq!(
        acceptance_count_after_restore, acceptance_count_before_restore,
        "Library restore must not replay the stored P1 archive decision"
    );
    let durable = actor
        .submit({
            let paper_id = paper_id.clone();
            move |connection| {
                Ok((
                    connection.query_row(
                        "SELECT lifecycle,current_phase FROM papers WHERE id=?1",
                        [&paper_id],
                        |row| Ok((row.get::<_, String>(0)?,row.get::<_, String>(1)?)),
                    )?,
                    connection.query_row(
                        "SELECT count(*) FROM audit_events WHERE entity_id=?1 AND action IN ('workflow.answer_changed','workflow.phase_accepted')",
                        [&paper_id],
                        |row| row.get::<_, i64>(0),
                    )?,
                    connection.query_row(
                        "SELECT count(*) FROM audit_events WHERE entity_id=?1 AND action='workflow.context_changed'",
                        [&paper_id],
                        |row| row.get::<_, i64>(0),
                    )?,
                    connection.query_row(
                        "SELECT state FROM paper_phases WHERE paper_id=?1 AND phase_code='P2'",
                        [&paper_id],
                        |row| row.get::<_, String>(0),
                    )?,
                    connection.query_row(
                        "SELECT answer_text,revision,updated_at FROM phase_answers WHERE paper_id=?1 AND phase_code='P2' AND question_key='main_questions_processed'",
                        [&paper_id],
                        |row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,String>(2)?)),
                    )?,
                    connection.query_row(
                        "SELECT structured_value_json FROM phase_answers WHERE paper_id=?1 AND phase_code='P1' AND question_key='relevance_decision'",
                        [&paper_id],
                        |row| row.get::<_, String>(0),
                    )?,
                ))
            }
        })
        .await
        .unwrap();
    assert_eq!(durable.0, ("ACTIVE".into(), "P1".into()));
    assert_eq!(durable.1, 20);
    assert_eq!(durable.2, 3);
    assert_eq!(durable.3, "NEEDS_REVIEW");
    assert_eq!(
        durable.4,
        ("future fixture answer".into(), 1, "fixture-time".into())
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&durable.5).unwrap()["readingDecision"],
        "archive"
    );
    let library_root = actor.library_root().clone();
    drop(state);
    actor.shutdown(Duration::from_secs(5)).unwrap();
    let reopened = DbActor::start(library_root).unwrap();
    let paper_for_reopen = paper_id.clone();
    let accepted = reopened
        .submit(move |connection| {
            let mut statement = connection.prepare(
                "SELECT phase_code,state,definition_version,accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 AND phase_code IN ('PRE','P1') ORDER BY phase_code",
            )?;
            let rows = statement.query_map([paper_for_reopen], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })
        .await
        .unwrap();
    assert_eq!(accepted.len(), 2);
    for (phase, state, version, snapshot_json, snapshot_hash) in accepted {
        assert_eq!(state, "COMPLETED");
        assert_eq!(version, 1);
        let raw = snapshot_json.expect("accepted snapshot survives reopen");
        let expected_hash = snapshot_hash.expect("accepted hash survives reopen");
        let snapshot: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(snapshot["phaseCode"], phase);
        assert_eq!(snapshot["definitionVersion"], 1);
        if phase == "P1" {
            assert_eq!(
                snapshot["answers"]["relevance_decision"]["structuredValue"]["readingDecision"],
                "archive"
            );
        }
        assert_eq!(
            research_workbench_core::application::library::canonical_hash(&snapshot).unwrap(),
            expected_hash
        );
    }
    let context = reopened
        .submit(move |connection| {
            Ok(connection.query_row(
                "SELECT current_phase FROM papers WHERE id=?1",
                [&paper_id],
                |row| row.get::<_, String>(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(context, "P1");
    reopened.shutdown(Duration::from_secs(5)).unwrap();
}

use research_workbench_core::{
    adapters::{
        documents::store::LocalDocumentStore,
        sqlite::{
            actor::DbActor,
            library_repository::SqliteLibraryPersistence,
            reader_repository::SqliteReaderPersistence,
            workflow_repository::{SqliteWorkflowPersistence, SqliteWorkflowRepository},
        },
        windows::paths::LibraryRoot,
    },
    application::{
        library_ports::{LibraryFuture, NativePdfSelection, RecoveryReport, SelectedPdf},
        reader_ports::{
            DocumentAccessOutcome, DocumentBody, DocumentProofAccess, DocumentReadHandle,
            ReaderFuture, RegisteredDocument,
        },
        request_registry::RequestRegistry,
        settings::RecoveryStatus,
        unit_of_work::with_transaction,
        workflow::initialize_processing,
        workflow_ports::{
            PreparedAdvance, PreparedTouch, WorkflowAdmission, WorkflowDocumentProof,
            WorkflowDocumentReference, WorkflowFuture, WorkflowPersistence,
        },
    },
    desktop::maintenance::MaintenanceCoordinator,
    modules::{
        library::service::LibraryService, reader::service::ReaderService,
        workflow::service::WorkflowService,
    },
    transport::{
        dto::*,
        error::{AppError, ErrorCode},
    },
};
use std::{
    future::Future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::{Notify, oneshot};

fn signal_first_poll<F: Future>(
    future: F,
    started: oneshot::Sender<()>,
) -> impl Future<Output = F::Output> {
    let mut future = Box::pin(future);
    let mut started = Some(started);
    std::future::poll_fn(move |cx| {
        let result = future.as_mut().poll(cx);
        if let Some(started) = started.take() {
            let _ = started.send(());
        }
        result
    })
}

struct NoPicker;
impl NativePdfSelection for NoPicker {
    fn select_pdf(&self) -> LibraryFuture<'_, Option<SelectedPdf>> {
        Box::pin(async { Ok(None) })
    }
}

struct GateProof {
    entered: Arc<Notify>,
    release: Arc<Notify>,
    dropped: Arc<AtomicBool>,
    observed_at_drop: Arc<Mutex<Option<String>>>,
    database: std::path::PathBuf,
    paper_id: String,
    outcome: ProofOutcome,
}

#[derive(Clone)]
enum ProofOutcome {
    Available,
    Unavailable,
    Error(ErrorCode),
}

struct GateHandle {
    registered: RegisteredDocument,
    dropped: Arc<AtomicBool>,
    observed_at_drop: Arc<Mutex<Option<String>>>,
    database: std::path::PathBuf,
    paper_id: String,
}

impl DocumentReadHandle for GateHandle {
    fn registered(&self) -> &RegisteredDocument {
        &self.registered
    }
    fn read_all(self: Box<Self>) -> ReaderFuture<'static, DocumentBody> {
        Box::pin(async { Err(AppError::new(ErrorCode::StorageUnavailable)) })
    }
}

impl Drop for GateHandle {
    fn drop(&mut self) {
        let connection = rusqlite::Connection::open(&self.database).unwrap();
        connection.busy_timeout(Duration::ZERO).unwrap();
        let observed = connection.query_row(
            "SELECT p.lifecycle || ':' || d.sha256 || ':' || ph.state || ':' || (SELECT count(*) FROM operation_receipts WHERE command='workflow_advance_phase') FROM papers p JOIN documents d ON d.id=p.active_document_id JOIN paper_phases ph ON ph.paper_id=p.id AND ph.phase_code='PRE' WHERE p.id=?1",
            [&self.paper_id], |row| row.get::<_, String>(0),
        ).unwrap();
        connection
            .execute_batch("BEGIN IMMEDIATE; ROLLBACK;")
            .unwrap();
        *self.observed_at_drop.lock().unwrap() = Some(observed);
        self.dropped.store(true, Ordering::SeqCst);
    }
}

impl DocumentProofAccess for GateProof {
    fn prove_access(
        &self,
        registered: RegisteredDocument,
    ) -> ReaderFuture<'static, DocumentAccessOutcome> {
        let entered = self.entered.clone();
        let release = self.release.clone();
        let dropped = self.dropped.clone();
        let observed_at_drop = self.observed_at_drop.clone();
        let database = self.database.clone();
        let paper_id = self.paper_id.clone();
        let outcome = self.outcome.clone();
        Box::pin(async move {
            entered.notify_one();
            release.notified().await;
            match outcome {
                ProofOutcome::Available => Ok(DocumentAccessOutcome::Available(Box::new(GateHandle { registered, dropped, observed_at_drop, database, paper_id }))),
                ProofOutcome::Unavailable => Ok(DocumentAccessOutcome::Unavailable(research_workbench_core::application::reader_ports::DocumentUnavailableReason::Missing)),
                ProofOutcome::Error(code) => Err(AppError::new(code)),
            }
        })
    }
}

fn service(
    actor: &DbActor,
    maintenance: MaintenanceCoordinator,
    requests: RequestRegistry,
    proof: Arc<dyn DocumentProofAccess>,
) -> WorkflowService {
    service_with_persistence(
        Arc::new(SqliteWorkflowPersistence::new(actor.clone())),
        maintenance,
        requests,
        proof,
    )
}

fn service_with_persistence(
    persistence: Arc<dyn WorkflowPersistence>,
    maintenance: MaintenanceCoordinator,
    requests: RequestRegistry,
    proof: Arc<dyn DocumentProofAccess>,
) -> WorkflowService {
    let recovery = RecoveryStatus::pending();
    recovery.finish(Ok(RecoveryReport {
        recovered: 0,
        issues: vec![],
    }));
    WorkflowService::new(persistence, maintenance, recovery, requests, proof)
}

struct ObserveAdvanceEnqueue {
    inner: Arc<dyn WorkflowPersistence>,
    queued: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl WorkflowPersistence for ObserveAdvanceEnqueue {
    fn writable(&self) -> bool {
        self.inner.writable()
    }
    fn phase(&self, paper_id: UUID, phase: PhaseCode) -> WorkflowFuture<'static, PhaseDto> {
        self.inner.phase(paper_id, phase)
    }
    fn answers(
        &self,
        paper_id: UUID,
        phase: PhaseCode,
    ) -> WorkflowFuture<'static, Vec<PhaseAnswerDto>> {
        self.inner.answers(paper_id, phase)
    }
    fn definition(
        &self,
        phase: PhaseCode,
        version: Option<i64>,
    ) -> WorkflowFuture<'static, PhaseDefinitionDto> {
        self.inner.definition(phase, version)
    }
    fn save_answer(
        &self,
        args: WorkflowSavePhaseAnswerArgs,
    ) -> WorkflowFuture<'static, PhaseAnswerDto> {
        self.inner.save_answer(args)
    }
    fn go_back(
        &self,
        args: WorkflowGoBackToPhaseArgs,
    ) -> WorkflowFuture<'static, WorkflowGoBackToPhaseOutput> {
        self.inner.go_back(args)
    }
    fn prepare_touch(
        &self,
        args: WorkflowTouchPhaseArgs,
    ) -> WorkflowFuture<'static, PreparedTouch> {
        self.inner.prepare_touch(args)
    }
    fn touch(
        &self,
        args: WorkflowTouchPhaseArgs,
        proof: Option<WorkflowDocumentProof>,
        admission: WorkflowAdmission,
    ) -> WorkflowFuture<'static, PhaseDto> {
        self.inner.touch(args, proof, admission)
    }
    fn document_reference(
        &self,
        paper_id: String,
    ) -> WorkflowFuture<'static, WorkflowDocumentReference> {
        self.inner.document_reference(paper_id)
    }
    fn prepare_advance(
        &self,
        args: WorkflowAdvancePhaseArgs,
    ) -> WorkflowFuture<'static, PreparedAdvance> {
        self.inner.prepare_advance(args)
    }
    fn advance(
        &self,
        args: WorkflowAdvancePhaseArgs,
        proof: WorkflowDocumentProof,
        admission: WorkflowAdmission,
    ) -> WorkflowFuture<'static, WorkflowAdvancePhaseOutput> {
        let mut future = self.inner.advance(args, proof, admission);
        let queued = self.queued.clone();
        Box::pin(std::future::poll_fn(move |cx| {
            let result = future.as_mut().poll(cx);
            if result.is_pending()
                && let Some(sender) = queued.lock().unwrap().take()
            {
                let _ = sender.send(());
            }
            result
        }))
    }
    fn evaluate(
        &self,
        args: WorkflowEvaluateGateArgs,
        proof: WorkflowDocumentProof,
        admission: WorkflowAdmission,
    ) -> WorkflowFuture<'static, GateEvaluationDto> {
        self.inner.evaluate(args, proof, admission)
    }
}

fn answers(connection: &rusqlite::Connection, paper_id: &str) -> Result<(), AppError> {
    for (key, text, structured) in [
        ("purpose", "purpose", None),
        ("uncertainty_target", "uncertainty", None),
        ("baseline", "baseline", None),
        ("expected_outcome", "outcome", None),
        ("desired_depth", "depth", None),
        ("review_type", "", Some(r#"{"reviewType":"survey"}"#)),
    ] {
        connection.execute("INSERT INTO phase_answers(paper_id,phase_code,question_key,answer_text,structured_value_json,resolution,revision,updated_at) VALUES(?1,'PRE',?2,?3,?4,'ANSWERED',1,'now')", rusqlite::params![paper_id,key,text,structured])?;
    }
    Ok(())
}

async fn fixture(actor: &DbActor) -> (String, String) {
    let paper = uuid::Uuid::new_v4().to_string();
    let document = uuid::Uuid::new_v4().to_string();
    let p = paper.clone();
    let d = document.clone();
    actor.submit(move |connection| {
        connection.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'Concurrency fixture','survey','now','now')", [&p])?;
        connection.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,'fixture.pdf',?3,?4,'application/pdf',42,'now','ACTIVE')", rusqlite::params![d,p,format!("documents/{d}/original.pdf"),"a".repeat(64)])?;
        connection.execute("UPDATE papers SET active_document_id=?2 WHERE id=?1", rusqlite::params![p,d])?;
        with_transaction(connection, |tx| initialize_processing(tx, &SqliteWorkflowRepository, &p))?;
        answers(connection, &p)
    }).await.unwrap();
    (paper, document)
}

fn args(paper: &str, request: &str) -> WorkflowAdvancePhaseArgs {
    WorkflowAdvancePhaseArgs {
        request_id: UUID(request.into()),
        paper_id: UUID(paper.into()),
        from_phase: PhaseCode::PRE,
        expected_phase_revision: 1,
    }
}

#[tokio::test]
async fn cancellation_keeps_request_and_maintenance_until_reference_conflict_rolls_back() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let (paper, document) = fixture(&actor).await;
    let maintenance = MaintenanceCoordinator::default();
    let requests = RequestRegistry::default();
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let dropped = Arc::new(AtomicBool::new(false));
    let observed_at_drop = Arc::new(Mutex::new(None));
    let proof = Arc::new(GateProof {
        entered: entered.clone(),
        release: release.clone(),
        dropped: dropped.clone(),
        observed_at_drop: observed_at_drop.clone(),
        database: actor.library_root().database(),
        paper_id: paper.clone(),
        outcome: ProofOutcome::Available,
    });
    let (advance_queued_tx, advance_queued_rx) = oneshot::channel();
    let persistence: Arc<dyn WorkflowPersistence> = Arc::new(ObserveAdvanceEnqueue {
        inner: Arc::new(SqliteWorkflowPersistence::new(actor.clone())),
        queued: Arc::new(Mutex::new(Some(advance_queued_tx))),
    });
    let workflow =
        service_with_persistence(persistence, maintenance.clone(), requests.clone(), proof);
    let caller_paper = paper.clone();
    let caller = tokio::spawn({
        let workflow = workflow.clone();
        async move {
            workflow
                .advance_phase(args(&caller_paper, "00000000-0000-4000-8000-000000000041"))
                .await
        }
    });
    tokio::time::timeout(Duration::from_secs(3), entered.notified())
        .await
        .unwrap();
    assert_eq!(maintenance.active_operations(), 1);
    let (actor_started_tx, actor_started_rx) = std::sync::mpsc::channel();
    let (actor_release_tx, actor_release_rx) = std::sync::mpsc::channel();
    let changed_document = document.clone();
    let blocker = actor.submit(move |connection| {
        actor_started_tx.send(()).unwrap();
        actor_release_rx.recv().unwrap();
        connection.execute(
            "UPDATE documents SET sha256=?2 WHERE id=?1",
            rusqlite::params![changed_document, "b".repeat(64)],
        )?;
        Ok(())
    });
    actor_started_rx
        .recv_timeout(Duration::from_secs(3))
        .unwrap();
    release.notify_one();
    tokio::time::timeout(Duration::from_secs(3), advance_queued_rx)
        .await
        .unwrap()
        .unwrap();
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    actor_release_tx.send(()).unwrap();
    blocker.await.unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        while maintenance.active_operations() != 0 || !dropped.load(Ordering::SeqCst) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(
        dropped.load(Ordering::SeqCst),
        "proof handle must be dropped after final reference check and rollback"
    );
    assert_eq!(
        observed_at_drop.lock().unwrap().as_deref(),
        Some(format!("NEW:{}:IN_PROGRESS:0", "b".repeat(64)).as_str()),
        "drop must observe the changed database reference and rolled-back workflow state"
    );
    assert_eq!(maintenance.active_operations(), 0);
    let request = requests
        .acquire(&UUID("00000000-0000-4000-8000-000000000041".into()))
        .await
        .unwrap();
    drop(request);
    let paper_for_check = paper.clone();
    let state = actor.submit(move |connection| Ok((
        connection.query_row("SELECT lifecycle FROM papers WHERE id=?1", [&paper_for_check], |row| row.get::<_, String>(0))?,
        connection.query_row("SELECT count(*) FROM operation_receipts WHERE command='workflow_advance_phase'", [], |row| row.get::<_, i64>(0))?,
        connection.query_row("SELECT state FROM paper_phases WHERE paper_id=?1 AND phase_code='PRE'", [&paper_for_check], |row| row.get::<_, String>(0))?,
    ))).await.unwrap();
    assert_eq!(state, ("NEW".to_owned(), 0, "IN_PROGRESS".to_owned()));
    actor.shutdown(Duration::from_secs(5)).unwrap();
}

struct ImmediateReadHandle(RegisteredDocument);
impl DocumentReadHandle for ImmediateReadHandle {
    fn registered(&self) -> &RegisteredDocument {
        &self.0
    }
    fn read_all(self: Box<Self>) -> ReaderFuture<'static, DocumentBody> {
        Box::pin(async { Err(AppError::new(ErrorCode::StorageUnavailable)) })
    }
}

struct ImmediateReadAccess;
impl research_workbench_core::application::reader_ports::DocumentReadAccess
    for ImmediateReadAccess
{
    fn open_verified(
        &self,
        registered: RegisteredDocument,
    ) -> ReaderFuture<'static, research_workbench_core::application::reader_ports::VerifiedDocument>
    {
        Box::pin(async move { Ok(Box::new(ImmediateReadHandle(registered)) as _) })
    }
}

#[tokio::test]
async fn cancelling_during_proof_keeps_shared_registry_and_operation_owner() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let (paper, _) = fixture(&actor).await;
    let maintenance = MaintenanceCoordinator::default();
    let requests = RequestRegistry::default();
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let dropped = Arc::new(AtomicBool::new(false));
    let observed_at_drop = Arc::new(Mutex::new(None));
    let workflow = service(
        &actor,
        maintenance.clone(),
        requests.clone(),
        Arc::new(GateProof {
            entered: entered.clone(),
            release: release.clone(),
            dropped: dropped.clone(),
            observed_at_drop: observed_at_drop.clone(),
            database: actor.library_root().database(),
            paper_id: paper.clone(),
            outcome: ProofOutcome::Available,
        }),
    );
    let store = Arc::new(
        LocalDocumentStore::new(
            actor.library_root().clone(),
            actor.info().library_id.0.clone(),
        )
        .unwrap(),
    );
    let library = LibraryService::new(
        Arc::new(NoPicker),
        store,
        Arc::new(SqliteLibraryPersistence::new(
            actor.clone(),
            actor.info().library_id.0.clone(),
        )),
        maintenance.clone(),
        requests.clone(),
    );
    let reader = ReaderService::new(
        Arc::new(SqliteReaderPersistence::new(
            actor.clone(),
            actor.info().library_id.0.clone(),
        )),
        Arc::new(ImmediateReadAccess),
        maintenance.clone(),
        requests.clone(),
        RecoveryStatus::default(),
    );
    let request_id = UUID("00000000-0000-4000-8000-000000000044".into());
    let p = paper.clone();
    let workflow_request_id = request_id.0.clone();
    let caller =
        tokio::spawn(async move { workflow.advance_phase(args(&p, &workflow_request_id)).await });
    tokio::time::timeout(Duration::from_secs(3), entered.notified())
        .await
        .unwrap();
    assert_eq!(maintenance.active_operations(), 1);
    let (library_polled_tx, library_polled_rx) = oneshot::channel();
    let (reader_polled_tx, reader_polled_rx) = oneshot::channel();
    let lib_request = request_id.clone();
    let lib_paper = UUID(paper.clone());
    let library_for_task = library.clone();
    let library_task = tokio::spawn(async move {
        signal_first_poll(
            library_for_task.archive_paper(lib_request, lib_paper, 0),
            library_polled_tx,
        )
        .await
    });
    let read_request = request_id.clone();
    let read_paper = UUID(paper.clone());
    let reader_for_task = reader.clone();
    let reader_task = tokio::spawn(async move {
        signal_first_poll(
            reader_for_task.open_paper(read_request, read_paper),
            reader_polled_tx,
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(3), library_polled_rx)
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(3), reader_polled_rx)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        maintenance.active_operations(),
        1,
        "both commands must stop at the shared request registry before they acquire an operation permit"
    );
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    assert_eq!(
        maintenance.active_operations(),
        1,
        "caller cancellation must not release the admitted Workflow operation"
    );
    assert!(matches!(maintenance.begin_maintenance(), Err(error) if error.code == ErrorCode::Busy));
    release.notify_one();
    let workflow_done = tokio::time::timeout(Duration::from_secs(3), async {
        while maintenance.active_operations() != 0 || !dropped.load(Ordering::SeqCst) {
            tokio::task::yield_now().await;
        }
    })
    .await;
    workflow_done.unwrap();
    assert!(
        observed_at_drop
            .lock()
            .unwrap()
            .as_deref()
            .unwrap()
            .starts_with("ACTIVE:")
    );
    let (library_result, reader_result) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(library_task, reader_task)
    })
    .await
    .unwrap();
    assert_eq!(
        library_result.unwrap().unwrap_err().code,
        ErrorCode::Conflict
    );
    let reader_result = reader_result.unwrap();
    assert_eq!(
        reader_result.unwrap_err().code,
        ErrorCode::Conflict,
        "Reader must progress into receipt validation only after Workflow releases the shared request"
    );
    actor.shutdown(Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn unavailable_proof_with_changed_reference_returns_conflict_without_transition() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let (paper, document) = fixture(&actor).await;
    let maintenance = MaintenanceCoordinator::default();
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let workflow = service(
        &actor,
        maintenance.clone(),
        RequestRegistry::default(),
        Arc::new(GateProof {
            entered: entered.clone(),
            release: release.clone(),
            dropped: Arc::new(AtomicBool::new(false)),
            observed_at_drop: Arc::new(Mutex::new(None)),
            database: actor.library_root().database(),
            paper_id: paper.clone(),
            outcome: ProofOutcome::Unavailable,
        }),
    );
    let p = paper.clone();
    let caller = tokio::spawn(async move {
        workflow
            .advance_phase(args(&p, "00000000-0000-4000-8000-000000000042"))
            .await
    });
    tokio::time::timeout(Duration::from_secs(3), entered.notified())
        .await
        .unwrap();
    let d = document.clone();
    actor
        .submit(move |connection| {
            connection.execute(
                "UPDATE documents SET sha256=?2 WHERE id=?1",
                rusqlite::params![d, "c".repeat(64)],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    release.notify_one();
    let error = tokio::time::timeout(Duration::from_secs(3), caller)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::Conflict);
    assert_eq!(maintenance.active_operations(), 0);
    let p = paper.clone();
    let result = actor.submit(move |connection| Ok((
        connection.query_row("SELECT lifecycle FROM papers WHERE id=?1", [&p], |r| r.get::<_, String>(0))?,
        connection.query_row("SELECT state FROM paper_phases WHERE paper_id=?1 AND phase_code='PRE'", [&p], |r| r.get::<_, String>(0))?,
        connection.query_row("SELECT count(*) FROM operation_receipts WHERE command='workflow_advance_phase'", [], |r| r.get::<_, i64>(0))?,
    ))).await.unwrap();
    assert_eq!(result, ("NEW".into(), "IN_PROGRESS".into(), 0));
    actor.shutdown(Duration::from_secs(5)).unwrap();
}

#[tokio::test]
async fn typed_proof_errors_propagate_without_becoming_gate_results() {
    for code in [
        ErrorCode::StorageUnavailable,
        ErrorCode::PathNotAllowed,
        ErrorCode::IntegrityFailure,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
        let (paper, _) = fixture(&actor).await;
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let workflow = service(
            &actor,
            MaintenanceCoordinator::default(),
            RequestRegistry::default(),
            Arc::new(GateProof {
                entered: entered.clone(),
                release: release.clone(),
                dropped: Arc::new(AtomicBool::new(false)),
                observed_at_drop: Arc::new(Mutex::new(None)),
                database: actor.library_root().database(),
                paper_id: paper.clone(),
                outcome: ProofOutcome::Error(code.clone()),
            }),
        );
        let p = paper.clone();
        let caller = tokio::spawn(async move {
            workflow
                .evaluate_gate(WorkflowEvaluateGateArgs {
                    request_id: UUID(uuid::Uuid::new_v4().to_string()),
                    paper_id: UUID(p),
                    phase_code: PhaseCode::PRE,
                })
                .await
        });
        tokio::time::timeout(Duration::from_secs(3), entered.notified())
            .await
            .unwrap();
        release.notify_one();
        let error = tokio::time::timeout(Duration::from_secs(3), caller)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert_eq!(error.code, code);
        let receipts = actor.submit(move |connection| Ok(connection.query_row("SELECT count(*) FROM operation_receipts WHERE command='workflow_evaluate_gate'", [], |r| r.get::<_, i64>(0))?)).await.unwrap();
        assert_eq!(receipts, 0);
        actor.shutdown(Duration::from_secs(5)).unwrap();
    }
}

#[tokio::test]
async fn proof_handle_drop_observes_committed_advance() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let (paper, _) = fixture(&actor).await;
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let dropped = Arc::new(AtomicBool::new(false));
    let observed_at_drop = Arc::new(Mutex::new(None));
    let workflow = service(
        &actor,
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
        Arc::new(GateProof {
            entered: entered.clone(),
            release: release.clone(),
            dropped: dropped.clone(),
            observed_at_drop: observed_at_drop.clone(),
            database: actor.library_root().database(),
            paper_id: paper.clone(),
            outcome: ProofOutcome::Available,
        }),
    );
    let p = paper.clone();
    let caller = tokio::spawn(async move {
        workflow
            .advance_phase(args(&p, "00000000-0000-4000-8000-000000000043"))
            .await
    });
    tokio::time::timeout(Duration::from_secs(3), entered.notified())
        .await
        .unwrap();
    release.notify_one();
    let result = tokio::time::timeout(Duration::from_secs(3), caller)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(result.phase.state, PhaseState::COMPLETED);
    assert!(dropped.load(Ordering::SeqCst));
    let observed = observed_at_drop.lock().unwrap().clone().unwrap();
    assert!(
        observed.starts_with("ACTIVE:"),
        "handle drop must observe durable lifecycle: {observed}"
    );
    assert!(
        observed.contains(":COMPLETED:1"),
        "handle drop must observe committed phase and receipt: {observed}"
    );
    actor.shutdown(Duration::from_secs(5)).unwrap();
}

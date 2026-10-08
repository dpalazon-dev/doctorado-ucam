use research_workbench_core::{
    adapters::{
        documents::store::LocalDocumentStore,
        sqlite::{
            actor::DbActor, library_repository::SqliteLibraryPersistence,
            reader_repository::SqliteReaderPersistence,
        },
        windows::paths::LibraryRoot,
    },
    application::{
        library_ports::{
            CancelPlan, ImportRecord, LibraryFuture, LibraryPersistence, NativePdfSelection,
            PreparedImport, ResourceInspection, SelectedPdf, StageFuture,
        },
        reader_ports::{
            DocumentAccessOutcome, DocumentProofAccess, DocumentReadAccess, OpenPaperCommand,
            ReaderFuture, ReaderPersistence, RegisteredDocument, SaveReadingPositionCommand,
            VerifiedDocument,
        },
    },
    desktop::maintenance::MaintenanceCoordinator,
    modules::{library::service::LibraryService, reader::service::ReaderService},
    transport::dto::{
        OpenPaperDto, PaperDto, PaperMetadataInput, ReadingPositionDto, ReviewType, UUID,
    },
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    future::Future,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
};

fn poll_once<F: Future>(mut future: std::pin::Pin<&mut F>) -> Poll<F::Output> {
    let mut context = Context::from_waker(Waker::noop());
    future.as_mut().poll(&mut context)
}

struct GatedReaderAccess {
    inner: Arc<LocalDocumentStore>,
    entered: std::sync::mpsc::Sender<()>,
    release: Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Receiver<()>>>>,
    handle_acquisitions: Arc<AtomicUsize>,
    handle_drops: Arc<AtomicUsize>,
}
struct ProbeHandle {
    inner: Option<VerifiedDocument>,
    drops: Arc<AtomicUsize>,
}
impl Drop for ProbeHandle {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl research_workbench_core::application::reader_ports::DocumentReadHandle for ProbeHandle {
    fn registered(
        &self,
    ) -> &research_workbench_core::application::reader_ports::RegisteredDocument {
        self.inner.as_ref().unwrap().registered()
    }
    fn read_all(
        mut self: Box<Self>,
    ) -> ReaderFuture<'static, research_workbench_core::application::reader_ports::DocumentBody>
    {
        self.inner.take().unwrap().read_all()
    }
}
impl DocumentReadAccess for GatedReaderAccess {
    fn open_verified(
        &self,
        registered: research_workbench_core::application::reader_ports::RegisteredDocument,
    ) -> ReaderFuture<'static, VerifiedDocument> {
        let inner = self.inner.clone();
        let entered = self.entered.clone();
        let release = self.release.clone();
        let acquisitions = self.handle_acquisitions.clone();
        let drops = self.handle_drops.clone();
        Box::pin(async move {
            let handle = inner.open_verified(registered).await?;
            let handle = Box::new(ProbeHandle {
                inner: Some(handle),
                drops,
            }) as VerifiedDocument;
            acquisitions.fetch_add(1, Ordering::SeqCst);
            let _ = entered.send(());
            let receiver = release.lock().unwrap().take().unwrap();
            let _ = receiver.await;
            Ok(handle)
        })
    }
}

struct ConfirmObservedPersistence {
    inner: Arc<SqliteReaderPersistence>,
    admitted: std::sync::mpsc::Sender<()>,
}
impl ReaderPersistence for ConfirmObservedPersistence {
    fn writable(&self) -> bool {
        self.inner.writable()
    }
    fn prepare_open(&self, command: OpenPaperCommand) -> ReaderFuture<'static, RegisteredDocument> {
        self.inner.prepare_open(command)
    }
    fn confirm_open(
        &self,
        command: OpenPaperCommand,
        verified: VerifiedDocument,
    ) -> ReaderFuture<'static, OpenPaperDto> {
        let mut result = self.inner.confirm_open(command, verified);
        let admitted = self.admitted.clone();
        let mut signaled = false;
        Box::pin(std::future::poll_fn(move |context| {
            let polled = result.as_mut().poll(context);
            if matches!(polled, Poll::Pending) && !signaled {
                signaled = true;
                let _ = admitted.send(());
            }
            polled
        }))
    }
    fn last_opened_paper(&self) -> ReaderFuture<'static, Option<PaperDto>> {
        self.inner.last_opened_paper()
    }
    fn registered_document(&self, document_id: UUID) -> ReaderFuture<'static, RegisteredDocument> {
        self.inner.registered_document(document_id)
    }
    fn reading_position(&self, document_id: UUID) -> ReaderFuture<'static, ReadingPositionDto> {
        self.inner.reading_position(document_id)
    }
    fn save_reading_position(
        &self,
        command: SaveReadingPositionCommand,
    ) -> ReaderFuture<'static, ReadingPositionDto> {
        self.inner.save_reading_position(command)
    }
}

struct NoPicker;
impl NativePdfSelection for NoPicker {
    fn select_pdf(&self) -> LibraryFuture<'_, Option<SelectedPdf>> {
        Box::pin(async { Ok(None) })
    }
}

struct PathPicker {
    path: PathBuf,
}
impl NativePdfSelection for PathPicker {
    fn select_pdf(&self) -> LibraryFuture<'_, Option<SelectedPdf>> {
        let path = self.path.clone();
        Box::pin(async move {
            Ok(Some(SelectedPdf {
                filename: "synthetic-two-page.pdf".into(),
                path,
            }))
        })
    }
}

fn synthetic_two_page_pdf() -> Vec<u8> {
    let objects = [
        "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
        "2 0 obj\n<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>\nendobj\n",
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] /Resources << >> /Contents 5 0 R >>\nendobj\n",
        "4 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] /Resources << >> /Contents 6 0 R >>\nendobj\n",
        "5 0 obj\n<< /Length 0 >>\nstream\n\nendstream\nendobj\n",
        "6 0 obj\n<< /Length 0 >>\nstream\n\nendstream\nendobj\n",
    ];
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0usize];
    for object in objects {
        offsets.push(pdf.len());
        pdf.extend_from_slice(object.as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    pdf
}

struct GatedPromotion {
    inner: Arc<LocalDocumentStore>,
    entered: std::sync::mpsc::Sender<()>,
    release: Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Receiver<()>>>>,
}
impl research_workbench_core::application::library_ports::DocumentStore for GatedPromotion {
    fn stage(&self, id: String, selected: SelectedPdf) -> StageFuture {
        self.inner.stage(id, selected)
    }
    fn inspect_owned(&self, record: ImportRecord) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.inspect_owned(record)
    }
    fn promote(&self, prepared: PreparedImport) -> LibraryFuture<'static, ResourceInspection> {
        let inner = self.inner.clone();
        let entered = self.entered.clone();
        let release = self.release.clone();
        Box::pin(async move {
            let result = inner.promote(prepared).await?;
            let _ = entered.send(());
            let receiver = release.lock().unwrap().take().unwrap();
            let _ = receiver.await;
            Ok(result)
        })
    }
    fn cleanup_owned(&self, plan: CancelPlan) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.cleanup_owned(plan)
    }
}

struct CountingReadAccess {
    inner: Arc<LocalDocumentStore>,
    count: Arc<AtomicUsize>,
}
impl DocumentReadAccess for CountingReadAccess {
    fn open_verified(
        &self,
        registered: research_workbench_core::application::reader_ports::RegisteredDocument,
    ) -> ReaderFuture<'static, VerifiedDocument> {
        self.count.fetch_add(1, Ordering::SeqCst);
        self.inner.open_verified(registered)
    }
}

fn fixture() -> (
    tempfile::TempDir,
    DbActor,
    SqliteReaderPersistence,
    Arc<LocalDocumentStore>,
    UUID,
    UUID,
) {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let paper_id = UUID(uuid::Uuid::new_v4().to_string());
    let document_id = UUID(uuid::Uuid::new_v4().to_string());
    let library_id = actor.info().library_id.clone();
    let relative = format!("documents/{}/original.pdf", document_id.0);
    let bytes = b"%PDF-1.4 synthetic reader fixture";
    let path = actor.library_root().path().join(&relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    let paper = paper_id.0.clone();
    let doc = document_id.0.clone();
    let name = "síntesis.pdf".to_owned();
    let hash = format!("{:x}", Sha256::digest(bytes));
    let rel = relative.clone();
    tauri::async_runtime::block_on(actor.submit(move |connection| {
        let tx=connection.transaction()?;
        tx.execute("INSERT INTO papers(id,title,doi,year,review_type,domain,venue_id,lifecycle,revision,current_phase,processing_initialized,active_document_id,created_at,updated_at,last_opened_at) VALUES(?1,'Prueba de lectura',NULL,2024,'unknown',NULL,NULL,'NEW',0,NULL,0,?2,'2026-10-02T00:00:00Z','2026-10-02T00:00:00Z',NULL)",rusqlite::params![paper,doc])?;
        tx.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,?3,?4,?5,'application/pdf',?6,'2026-10-02T00:00:00Z','ACTIVE')",rusqlite::params![doc,paper,name,rel,hash,bytes.len() as i64])?;
        tx.commit()?;Ok(())
    })).unwrap();
    let store = Arc::new(
        LocalDocumentStore::new(actor.library_root().clone(), library_id.0.clone()).unwrap(),
    );
    let persistence = SqliteReaderPersistence::new(actor.clone(), library_id.0);
    (dir, actor, persistence, store, paper_id, document_id)
}

#[test]
fn open_paper_sets_activity_transactionally_without_changing_bibliographic_revision() {
    let (_dir, actor, persistence, store, paper_id, document_id) = fixture();
    let command = OpenPaperCommand {
        request_id: UUID(uuid::Uuid::new_v4().to_string()),
        paper_id: paper_id.clone(),
    };
    let opened = tauri::async_runtime::block_on(async {
        let registered = persistence.prepare_open(command.clone()).await.unwrap();
        let verified = store.open_verified(registered).await.unwrap();
        persistence
            .confirm_open(command.clone(), verified)
            .await
            .unwrap()
    });
    assert_eq!(opened.paper.id, paper_id);
    assert_eq!(opened.document.id, document_id);
    assert_eq!(opened.paper.revision, 0);
    assert_eq!(opened.paper.updated_at, "2026-10-02T00:00:00Z");
    assert!(opened.paper.last_opened_at.is_some());
    assert!(
        opened
            .paper
            .last_opened_at
            .as_deref()
            .unwrap()
            .ends_with('Z'),
        "new Reader activity timestamps use canonical UTC Z form"
    );
    assert_eq!(opened.reading_position.page_index, 1);
    let expected_last_opened = opened.paper.last_opened_at.clone().unwrap();
    let expected_request_id = command.request_id.0.clone();
    let replayed = tauri::async_runtime::block_on(async {
        let registered = persistence.prepare_open(command.clone()).await.unwrap();
        let verified = store.open_verified(registered).await.unwrap();
        persistence
            .confirm_open(command.clone(), verified)
            .await
            .unwrap()
    });
    assert_eq!(
        replayed, opened,
        "confirmed open replay returns the stored result unchanged"
    );
    let expected_paper_id = paper_id.0.clone();
    tauri::async_runtime::block_on(actor.submit(move |c| {
        assert_eq!(
            c.query_row(
                "SELECT last_opened_paper_id FROM app_session WHERE singleton=1",
                [],
                |r| r.get::<_, String>(0)
            )?,
            expected_paper_id
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM reading_positions", [], |r| r
                .get::<_, i64>(0))?,
            0
        );
        let persisted: String = c.query_row(
            "SELECT last_opened_at FROM papers WHERE id=?1",
            [&expected_paper_id],
            |r| r.get(0),
        )?;
        assert_eq!(persisted, expected_last_opened);
        assert!(persisted.ends_with('Z'));
        let receipt: String = c.query_row(
            "SELECT result_json FROM operation_receipts WHERE request_id=?1",
            [&expected_request_id],
            |r| r.get(0),
        )?;
        let receipt: serde_json::Value = serde_json::from_str(&receipt).unwrap();
        assert_eq!(
            receipt["paper"]["lastOpenedAt"].as_str(),
            Some(expected_last_opened.as_str())
        );
        Ok(())
    }))
    .unwrap();
}

#[test]
fn default_position_is_not_inserted_and_saved_position_is_document_scoped() {
    let (_dir, actor, persistence, _store, _paper, document_id) = fixture();
    let default =
        tauri::async_runtime::block_on(persistence.reading_position(document_id.clone())).unwrap();
    assert_eq!(
        (default.page_index, default.zoom, default.revision),
        (1, 1.0, 0)
    );
    assert_eq!(default.updated_at, "2026-10-02T00:00:00Z");
    let command = SaveReadingPositionCommand {
        request_id: UUID(uuid::Uuid::new_v4().to_string()),
        document_id: document_id.clone(),
        expected_revision: 0,
        page_index: 2,
        zoom: 1.5,
    };
    let saved =
        tauri::async_runtime::block_on(persistence.save_reading_position(command.clone())).unwrap();
    assert_eq!((saved.page_index, saved.zoom, saved.revision), (2, 1.5, 1));
    assert!(
        saved.updated_at.ends_with('Z'),
        "new saved positions use canonical UTC Z form"
    );
    let fetched =
        tauri::async_runtime::block_on(persistence.reading_position(document_id.clone())).unwrap();
    assert_eq!(fetched, saved);
    let replayed =
        tauri::async_runtime::block_on(persistence.save_reading_position(command)).unwrap();
    assert_eq!(
        replayed, saved,
        "position receipt replay preserves its result"
    );
    tauri::async_runtime::block_on(actor.submit(|c| {
        assert_eq!(
            c.query_row("SELECT count(*) FROM reading_positions", [], |r| r
                .get::<_, i64>(0))?,
            1
        );
        Ok(())
    }))
    .unwrap();
}

#[test]
fn legacy_open_utc_timestamp_replay_preserves_the_receipt_and_database_value() {
    let (_dir, actor, persistence, store, paper_id, _document_id) = fixture();
    let command = OpenPaperCommand {
        request_id: UUID(uuid::Uuid::new_v4().to_string()),
        paper_id: paper_id.clone(),
    };
    tauri::async_runtime::block_on(async {
        let registered = persistence.prepare_open(command.clone()).await.unwrap();
        let handle = store.open_verified(registered).await.unwrap();
        persistence
            .confirm_open(command.clone(), handle)
            .await
            .unwrap();
    });
    let legacy_time = "2026-10-03T14:36:22.575865800+00:00";
    let request_for_seed = command.request_id.0.clone();
    let paper_for_seed = paper_id.0.clone();
    let seeded_receipt = tauri::async_runtime::block_on(actor.submit(move |connection| {
        let raw: String = connection.query_row(
            "SELECT result_json FROM operation_receipts WHERE request_id=?1",
            [&request_for_seed],
            |row| row.get(0),
        )?;
        let mut result: serde_json::Value = serde_json::from_str(&raw).map_err(|_| {
            research_workbench_core::transport::error::AppError::new(
                research_workbench_core::transport::error::ErrorCode::IntegrityFailure,
            )
        })?;
        result["paper"]["lastOpenedAt"] = json!(legacy_time);
        let seeded = serde_json::to_string(&result).unwrap();
        connection.execute(
            "UPDATE operation_receipts SET result_json=?1 WHERE request_id=?2",
            rusqlite::params![seeded, request_for_seed],
        )?;
        connection.execute(
            "UPDATE papers SET last_opened_at=?1 WHERE id=?2",
            rusqlite::params![legacy_time, paper_for_seed],
        )?;
        Ok(seeded)
    }))
    .unwrap();
    let before_audit_count = tauri::async_runtime::block_on(actor.submit(|connection| {
        Ok(
            connection.query_row("SELECT COUNT(*) FROM audit_events", [], |row| {
                row.get::<_, i64>(0)
            })?,
        )
    }))
    .unwrap();

    let registered =
        tauri::async_runtime::block_on(persistence.prepare_open(command.clone())).unwrap();
    let handle = tauri::async_runtime::block_on(store.open_verified(registered)).unwrap();
    let replayed =
        tauri::async_runtime::block_on(persistence.confirm_open(command.clone(), handle)).unwrap();
    assert_eq!(replayed.paper.last_opened_at.as_deref(), Some(legacy_time));
    let last_opened = tauri::async_runtime::block_on(persistence.last_opened_paper())
        .unwrap()
        .unwrap();
    assert_eq!(last_opened.last_opened_at.as_deref(), Some(legacy_time));

    let request_for_check = command.request_id.0;
    let paper_for_check = paper_id.0;
    let (receipt_after, last_opened_after, audit_after) =
        tauri::async_runtime::block_on(actor.submit(move |connection| {
            Ok((
                connection.query_row(
                    "SELECT result_json FROM operation_receipts WHERE request_id=?1",
                    [request_for_check],
                    |row| row.get::<_, String>(0),
                )?,
                connection.query_row(
                    "SELECT last_opened_at FROM papers WHERE id=?1",
                    [paper_for_check],
                    |row| row.get::<_, String>(0),
                )?,
                connection.query_row("SELECT COUNT(*) FROM audit_events", [], |row| {
                    row.get::<_, i64>(0)
                })?,
            ))
        }))
        .unwrap();
    assert_eq!(receipt_after, seeded_receipt);
    assert_eq!(last_opened_after, legacy_time);
    assert_eq!(audit_after, before_audit_count);
}

#[test]
fn legacy_position_utc_timestamp_get_and_receipt_replay_preserve_stored_strings() {
    let (_dir, actor, persistence, _store, _paper_id, document_id) = fixture();
    let command = SaveReadingPositionCommand {
        request_id: UUID(uuid::Uuid::new_v4().to_string()),
        document_id: document_id.clone(),
        expected_revision: 0,
        page_index: 2,
        zoom: 1.25,
    };
    tauri::async_runtime::block_on(persistence.save_reading_position(command.clone())).unwrap();
    let legacy_time = "2026-10-03T14:36:22.575865800+00:00";
    let request_for_seed = command.request_id.0.clone();
    let document_for_seed = document_id.0.clone();
    let seeded_receipt = tauri::async_runtime::block_on(actor.submit(move |connection| {
        let raw: String = connection.query_row(
            "SELECT result_json FROM operation_receipts WHERE request_id=?1",
            [&request_for_seed],
            |row| row.get(0),
        )?;
        let mut result: serde_json::Value = serde_json::from_str(&raw).map_err(|_| {
            research_workbench_core::transport::error::AppError::new(
                research_workbench_core::transport::error::ErrorCode::IntegrityFailure,
            )
        })?;
        result["updatedAt"] = json!(legacy_time);
        let seeded = serde_json::to_string(&result).unwrap();
        connection.execute(
            "UPDATE operation_receipts SET result_json=?1 WHERE request_id=?2",
            rusqlite::params![seeded, request_for_seed],
        )?;
        connection.execute(
            "UPDATE reading_positions SET updated_at=?1 WHERE document_id=?2",
            rusqlite::params![legacy_time, document_for_seed],
        )?;
        Ok(seeded)
    }))
    .unwrap();

    let fetched =
        tauri::async_runtime::block_on(persistence.reading_position(document_id.clone())).unwrap();
    assert_eq!(fetched.updated_at, legacy_time);
    let replayed =
        tauri::async_runtime::block_on(persistence.save_reading_position(command.clone())).unwrap();
    assert_eq!(replayed.updated_at, legacy_time);

    let request_for_check = command.request_id.0;
    let document_for_check = document_id.0;
    let (receipt_after, updated_at_after) =
        tauri::async_runtime::block_on(actor.submit(move |connection| {
            Ok((
                connection.query_row(
                    "SELECT result_json FROM operation_receipts WHERE request_id=?1",
                    [request_for_check],
                    |row| row.get::<_, String>(0),
                )?,
                connection.query_row(
                    "SELECT updated_at FROM reading_positions WHERE document_id=?1",
                    [document_for_check],
                    |row| row.get::<_, String>(0),
                )?,
            ))
        }))
        .unwrap();
    assert_eq!(receipt_after, seeded_receipt);
    assert_eq!(updated_at_after, legacy_time);
}

#[test]
fn preexisting_revision_zero_position_can_be_saved_with_expected_zero() {
    let (_dir, actor, persistence, _store, _paper, document_id) = fixture();
    let id = document_id.0.clone();
    tauri::async_runtime::block_on(actor.submit(move|c|{c.execute("INSERT INTO reading_positions(document_id,page_index,zoom,revision,updated_at) VALUES(?1,1,1.0,0,'2026-10-02T00:00:00Z')",[id])?;Ok(())})).unwrap();
    let saved = tauri::async_runtime::block_on(persistence.save_reading_position(
        SaveReadingPositionCommand {
            request_id: UUID(uuid::Uuid::new_v4().to_string()),
            document_id,
            expected_revision: 0,
            page_index: 3,
            zoom: 1.25,
        },
    ))
    .unwrap();
    assert_eq!((saved.page_index, saved.zoom, saved.revision), (3, 1.25, 1));
}

#[test]
fn dropped_reader_caller_keeps_handle_and_shared_request_owner_until_open_commit() {
    verify_dropped_reader_caller_keeps_handle_until_open_finishes(false);
}

#[test]
fn dropped_reader_caller_keeps_handle_and_shared_request_owner_until_open_rollback() {
    verify_dropped_reader_caller_keeps_handle_until_open_finishes(true);
}

fn verify_dropped_reader_caller_keeps_handle_until_open_finishes(rollback: bool) {
    let (_dir, actor, persistence, store, paper_id, _document_id) = fixture();
    if rollback {
        tauri::async_runtime::block_on(actor.submit(|connection| {
            connection.execute_batch("CREATE TRIGGER fail_reader_open_audit BEFORE INSERT ON audit_events WHEN NEW.action='reader.paper_opened' BEGIN SELECT RAISE(ABORT,'injected reader audit failure'); END;")?;
            Ok(())
        })).unwrap();
    }
    let maintenance = MaintenanceCoordinator::default();
    let registry =
        research_workbench_core::application::request_registry::RequestRegistry::default();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let (admitted_tx, admitted_rx) = std::sync::mpsc::channel();
    let (actor_started_tx, actor_started_rx) = std::sync::mpsc::channel();
    let (actor_release_tx, actor_release_rx) = std::sync::mpsc::channel();
    let handle_acquisitions = Arc::new(AtomicUsize::new(0));
    let handle_drops = Arc::new(AtomicUsize::new(0));
    let request_id = UUID(uuid::Uuid::new_v4().to_string());
    let reader = Arc::new(ReaderService::new(
        Arc::new(ConfirmObservedPersistence {
            inner: Arc::new(persistence),
            admitted: admitted_tx,
        }),
        Arc::new(GatedReaderAccess {
            inner: store.clone(),
            entered: entered_tx,
            release: Arc::new(std::sync::Mutex::new(Some(release_rx))),
            handle_acquisitions: handle_acquisitions.clone(),
            handle_drops: handle_drops.clone(),
        }),
        maintenance.clone(),
        registry.clone(),
        research_workbench_core::application::settings::RecoveryStatus::default(),
    ));
    let library = Arc::new(LibraryService::new(
        Arc::new(NoPicker),
        store,
        Arc::new(SqliteLibraryPersistence::new(
            actor.clone(),
            actor.info().library_id.0.clone(),
        )),
        maintenance.clone(),
        registry,
    ));
    let reader_request = request_id.clone();
    let reader_paper = paper_id.clone();
    let caller = tauri::async_runtime::spawn(async move {
        reader.open_paper(reader_request, reader_paper).await
    });
    entered_rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .unwrap();
    assert_eq!(handle_acquisitions.load(Ordering::SeqCst), 1);
    assert_eq!(maintenance.active_operations(), 1);
    let actor_blocker = tauri::async_runtime::spawn({
        let actor = actor.clone();
        async move {
            actor
                .submit(move |_| {
                    actor_started_tx.send(()).unwrap();
                    actor_release_rx.recv().unwrap();
                    Ok(())
                })
                .await
        }
    });
    actor_started_rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .unwrap();
    caller.abort();
    assert!(
        tauri::async_runtime::block_on(caller).is_err(),
        "caller task was aborted before the actor job was unblocked"
    );
    release_tx.send(()).unwrap();
    admitted_rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .unwrap();
    assert_eq!(
        handle_drops.load(Ordering::SeqCst),
        0,
        "the admitted operation still owns its verified file handle after caller cancellation"
    );
    let mut competing = Box::pin(library.archive_paper(request_id.clone(), paper_id.clone(), 0));
    assert!(
        matches!(poll_once(competing.as_mut()), Poll::Pending),
        "the competing Library request waits on the open RequestRegistry owner"
    );
    actor_release_tx.send(()).unwrap();
    tauri::async_runtime::block_on(actor_blocker)
        .unwrap()
        .unwrap();
    maintenance
        .wait_for_idle(std::time::Duration::from_secs(3))
        .unwrap();
    assert_eq!(
        handle_drops.load(Ordering::SeqCst),
        1,
        "the handle is released after the actor finishes commit or rollback"
    );
    let archive_result = tauri::async_runtime::block_on(competing.as_mut());
    if rollback {
        assert!(
            archive_result.is_ok(),
            "a rolled back open releases requestId ownership for a new operation"
        );
    } else {
        assert_eq!(
            archive_result.unwrap_err().code,
            research_workbench_core::transport::error::ErrorCode::Conflict
        );
    }
    let request_id_for_sql = request_id.0.clone();
    tauri::async_runtime::block_on(actor.submit(move |c| {
        let last: Option<String> = c.query_row(
            "SELECT last_opened_paper_id FROM app_session WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        let lifecycle: String = c.query_row("SELECT lifecycle FROM papers", [], |r| r.get(0))?;
        let receipt_count: i64 = c.query_row(
            "SELECT COUNT(*) FROM operation_receipts WHERE request_id=?1",
            [&request_id_for_sql],
            |r| r.get(0),
        )?;
        if rollback {
            assert_eq!(last, None);
            assert_eq!(lifecycle, "ARCHIVED");
            assert_eq!(
                receipt_count, 1,
                "the only durable receipt is the subsequent archive"
            );
        } else {
            assert_eq!(last, Some(paper_id.0));
            assert_eq!(lifecycle, "NEW");
            assert_eq!(receipt_count, 1, "the committed open owns the request ID");
        }
        Ok(())
    }))
    .unwrap();
}

#[test]
fn reader_last_opened_and_position_survive_new_process_reopen() {
    let (_dir, actor, persistence, store, paper_id, document_id) = fixture();
    let command = OpenPaperCommand {
        request_id: UUID(uuid::Uuid::new_v4().to_string()),
        paper_id: paper_id.clone(),
    };
    tauri::async_runtime::block_on(async {
        let registered = persistence.prepare_open(command.clone()).await.unwrap();
        let handle = store.open_verified(registered).await.unwrap();
        persistence.confirm_open(command, handle).await.unwrap();
        persistence
            .save_reading_position(SaveReadingPositionCommand {
                request_id: UUID(uuid::Uuid::new_v4().to_string()),
                document_id: document_id.clone(),
                expected_revision: 0,
                page_index: 2,
                zoom: 1.25,
            })
            .await
            .unwrap();
    });
    let root = actor.library_root().path().to_string_lossy().into_owned();
    let paper_id = paper_id.0;
    let document_id = document_id.0;
    drop(persistence);
    drop(store);
    actor.shutdown(std::time::Duration::from_secs(3)).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "reader_reopen_child_process", "--nocapture"])
        .env("TASK03_REOPEN_ROOT", root)
        .env("TASK03_REOPEN_PAPER", paper_id)
        .env("TASK03_REOPEN_DOCUMENT", document_id)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child process failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn reader_reopen_child_process() {
    let (Ok(root), Ok(paper_id), Ok(document_id)) = (
        std::env::var("TASK03_REOPEN_ROOT"),
        std::env::var("TASK03_REOPEN_PAPER"),
        std::env::var("TASK03_REOPEN_DOCUMENT"),
    ) else {
        return;
    };
    let actor = DbActor::start(LibraryRoot::at(root.into())).unwrap();
    let persistence =
        SqliteReaderPersistence::new(actor.clone(), actor.info().library_id.0.clone());
    let last_opened = tauri::async_runtime::block_on(persistence.last_opened_paper()).unwrap();
    assert_eq!(last_opened.unwrap().id.0, paper_id);
    let position =
        tauri::async_runtime::block_on(persistence.reading_position(UUID(document_id.clone())))
            .unwrap();
    assert_eq!(
        (position.page_index, position.zoom, position.revision),
        (2, 1.25, 1)
    );
    assert_eq!(position.document_id.0, document_id);
    actor.shutdown(std::time::Duration::from_secs(3)).unwrap();
}

#[test]
fn reader_reads_are_blocked_while_maintenance_has_exclusive_access() {
    let (_dir, actor, persistence, store, _paper, document_id) = fixture();
    let maintenance = MaintenanceCoordinator::default();
    let reader = ReaderService::new(
        Arc::new(persistence),
        store,
        maintenance.clone(),
        research_workbench_core::application::request_registry::RequestRegistry::default(),
        research_workbench_core::application::settings::RecoveryStatus::default(),
    );
    let _maintenance = maintenance.begin_maintenance().unwrap();
    assert_eq!(
        tauri::async_runtime::block_on(reader.get_reading_position(document_id))
            .unwrap_err()
            .code,
        research_workbench_core::transport::error::ErrorCode::Busy
    );
    assert_eq!(
        tauri::async_runtime::block_on(reader.get_last_opened_paper())
            .unwrap_err()
            .code,
        research_workbench_core::transport::error::ErrorCode::Busy
    );
    actor.shutdown(std::time::Duration::from_secs(3)).unwrap();
}

#[test]
fn get_paper_does_not_change_last_opened_activity() {
    let (_dir, actor, _reader, _store, paper_id, _document_id) = fixture();
    let library = SqliteLibraryPersistence::new(actor.clone(), actor.info().library_id.0.clone());
    let paper = tauri::async_runtime::block_on(
        library.get_paper(UUID(uuid::Uuid::new_v4().to_string()), paper_id.clone()),
    )
    .unwrap();
    assert_eq!(paper.id, paper_id);
    assert_eq!(paper.last_opened_at, None);
    let last = tauri::async_runtime::block_on(actor.submit(|c| {
        Ok(c.query_row(
            "SELECT last_opened_paper_id FROM app_session WHERE singleton=1",
            [],
            |r| r.get::<_, Option<String>>(0),
        )?)
    }))
    .unwrap();
    assert_eq!(last, None);
}

#[test]
fn open_replay_keeps_original_receipt_and_refuses_a_missing_current_file() {
    let (_dir, actor, persistence, store, paper_id, document_id) = fixture();
    let command = OpenPaperCommand {
        request_id: UUID(uuid::Uuid::new_v4().to_string()),
        paper_id,
    };
    let first = tauri::async_runtime::block_on(async {
        let doc = persistence.prepare_open(command.clone()).await.unwrap();
        let handle = store.open_verified(doc).await.unwrap();
        persistence
            .confirm_open(command.clone(), handle)
            .await
            .unwrap()
    });
    let marker = "2026-10-02T02:00:00Z".to_owned();
    let marker_for_db = marker.clone();
    tauri::async_runtime::block_on(actor.submit(move |c| {
        c.execute("UPDATE papers SET last_opened_at=?1", [marker_for_db])?;
        Ok(())
    }))
    .unwrap();
    let replay = tauri::async_runtime::block_on(async {
        let doc = persistence.prepare_open(command.clone()).await.unwrap();
        let handle = store.open_verified(doc).await.unwrap();
        persistence
            .confirm_open(command.clone(), handle)
            .await
            .unwrap()
    });
    assert_eq!(replay, first);
    let last = tauri::async_runtime::block_on(actor.submit(|c| {
        Ok(c.query_row("SELECT last_opened_at FROM papers", [], |r| {
            r.get::<_, String>(0)
        })?)
    }))
    .unwrap();
    assert_eq!(last, marker);
    let file = actor
        .library_root()
        .path()
        .join(format!("documents/{}/original.pdf", document_id.0));
    fs::remove_file(file).unwrap();
    let doc = tauri::async_runtime::block_on(persistence.prepare_open(command)).unwrap();
    assert!(tauri::async_runtime::block_on(store.open_verified(doc)).is_err());
}

#[test]
fn document_proof_classifies_missing_file_and_intermediate_directory() {
    use research_workbench_core::application::reader_ports::DocumentUnavailableReason;

    let (_dir, actor, persistence, store, _paper_id, document_id) = fixture();
    let registered =
        tauri::async_runtime::block_on(persistence.registered_document(document_id.clone()))
            .unwrap();
    let path = registered.relative_path.clone();
    // The regular reader path keeps its established missing-file error code.
    let absolute = actor.library_root().path().join(path);
    fs::remove_file(absolute).unwrap();
    let legacy = match tauri::async_runtime::block_on(store.open_verified(registered.clone())) {
        Ok(_) => panic!("legacy Reader must reject a missing file"),
        Err(error) => error,
    };
    assert_eq!(
        legacy.code,
        research_workbench_core::transport::error::ErrorCode::NotFound
    );
    assert!(matches!(
        tauri::async_runtime::block_on(store.prove_access(registered.clone())).unwrap(),
        DocumentAccessOutcome::Unavailable(DocumentUnavailableReason::Missing)
    ));
    let mut absent_parent = registered;
    absent_parent.document.id = UUID(uuid::Uuid::new_v4().to_string());
    absent_parent.relative_path = format!("documents/{}/original.pdf", absent_parent.document.id.0);
    absent_parent.size_bytes = 0;
    assert!(matches!(
        tauri::async_runtime::block_on(store.prove_access(absent_parent)).unwrap(),
        DocumentAccessOutcome::Unavailable(DocumentUnavailableReason::Missing)
    ));
}

#[test]
fn document_proof_returns_the_verified_handle_and_keeps_guard_errors_fatal() {
    use research_workbench_core::application::reader_ports::DocumentAccessOutcome;
    use research_workbench_core::transport::error::ErrorCode;

    let (_dir, _actor, persistence, store, _paper_id, document_id) = fixture();
    let registered =
        tauri::async_runtime::block_on(persistence.registered_document(document_id)).unwrap();
    let handle = match tauri::async_runtime::block_on(store.prove_access(registered.clone())) {
        Ok(DocumentAccessOutcome::Available(handle)) => handle,
        Ok(DocumentAccessOutcome::Unavailable(reason)) => {
            panic!("existing synthetic document was unavailable: {reason:?}")
        }
        Err(error) => panic!("existing synthetic document failed proof: {error:?}"),
    };
    assert_eq!(handle.registered(), &registered);
    drop(handle);

    let mut wrong_size = registered.clone();
    wrong_size.size_bytes += 1;
    let error = match tauri::async_runtime::block_on(store.prove_access(wrong_size)) {
        Ok(_) => panic!("a real size mismatch must remain an integrity error"),
        Err(error) => error,
    };
    assert_eq!(error.code, ErrorCode::IntegrityFailure);

    let mut invalid_size = registered;
    invalid_size.size_bytes = -1;
    let error = match tauri::async_runtime::block_on(store.prove_access(invalid_size)) {
        Ok(_) => panic!("an out-of-range registered size must remain a path error"),
        Err(error) => error,
    };
    assert_eq!(error.code, ErrorCode::PathNotAllowed);
}

#[cfg(windows)]
#[test]
fn document_proof_maps_a_blocked_ntfs_file_to_sharing_violation_and_legacy_storage_error() {
    use research_workbench_core::application::reader_ports::{
        DocumentAccessOutcome, DocumentUnavailableReason,
    };
    use research_workbench_core::transport::error::ErrorCode;

    let (_dir, _actor, persistence, store, _paper_id, document_id) = fixture();
    let registered =
        tauri::async_runtime::block_on(persistence.registered_document(document_id)).unwrap();
    let held = match tauri::async_runtime::block_on(store.prove_access(registered.clone())) {
        Ok(DocumentAccessOutcome::Available(handle)) => handle,
        _ => panic!("synthetic NTFS document did not open for the positive control"),
    };
    assert!(matches!(
        tauri::async_runtime::block_on(store.prove_access(registered.clone())).unwrap(),
        DocumentAccessOutcome::Unavailable(DocumentUnavailableReason::SharingViolation)
    ));
    let error = match tauri::async_runtime::block_on(store.open_verified(registered)) {
        Ok(_) => panic!("legacy Reader must keep its storage error for a sharing violation"),
        Err(error) => error,
    };
    assert_eq!(error.code, ErrorCode::StorageUnavailable);
    drop(held);
}

#[test]
fn replay_rejects_document_reassigned_to_another_paper_without_changing_receipt_or_audit() {
    let (_dir, actor, persistence, store, paper_a, document_id) = fixture();
    let command = OpenPaperCommand {
        request_id: UUID(uuid::Uuid::new_v4().to_string()),
        paper_id: paper_a.clone(),
    };
    let request_id = command.request_id.0.clone();
    tauri::async_runtime::block_on(async {
        let registered = persistence.prepare_open(command.clone()).await.unwrap();
        let handle = store.open_verified(registered).await.unwrap();
        persistence
            .confirm_open(command.clone(), handle)
            .await
            .unwrap();
    });
    let paper_b = uuid::Uuid::new_v4().to_string();
    let paper_a_id = paper_a.0.clone();
    let doc_id = document_id.0.clone();
    tauri::async_runtime::block_on(actor.submit(move|connection|{
        let tx=connection.transaction()?;
        tx.execute("INSERT INTO papers(id,title,year,review_type,lifecycle,revision,processing_initialized,active_document_id,created_at,updated_at) VALUES(?1,'Paper B',2025,'unknown','NEW',0,0,?2,'2026-10-02T00:00:00Z','2026-10-02T00:00:00Z')",rusqlite::params![paper_b,doc_id])?;
        tx.execute("UPDATE papers SET active_document_id=NULL WHERE id=?1",[paper_a_id])?;
        tx.execute("UPDATE documents SET paper_id=?1 WHERE id=?2",rusqlite::params![paper_b,doc_id])?;
        tx.commit()?;
        Ok(())
    })).unwrap();
    let receipt_request_id = request_id.clone();
    let paper_id_for_before = paper_a.0.clone();
    let before = tauri::async_runtime::block_on(actor.submit(move |connection| {
        Ok((
            connection.query_row(
                "SELECT result_json FROM operation_receipts WHERE request_id=?1",
                [receipt_request_id],
                |row| row.get::<_, String>(0),
            )?,
            connection.query_row("SELECT COUNT(*) FROM audit_events", [], |row| {
                row.get::<_, i64>(0)
            })?,
            connection.query_row(
                "SELECT last_opened_at FROM papers WHERE id=?1",
                [paper_id_for_before],
                |row| row.get::<_, Option<String>>(0),
            )?,
        ))
    }))
    .unwrap();
    let replay = tauri::async_runtime::block_on(persistence.prepare_open(command));
    assert!(
        replay.is_err(),
        "replay must reject a document now owned by a different paper"
    );
    let paper_id_for_after = paper_a.0.clone();
    let after = tauri::async_runtime::block_on(actor.submit(move |connection| {
        Ok((
            connection.query_row(
                "SELECT result_json FROM operation_receipts WHERE request_id=?1",
                [request_id],
                |row| row.get::<_, String>(0),
            )?,
            connection.query_row("SELECT COUNT(*) FROM audit_events", [], |row| {
                row.get::<_, i64>(0)
            })?,
            connection.query_row(
                "SELECT last_opened_at FROM papers WHERE id=?1",
                [paper_id_for_after],
                |row| row.get::<_, Option<String>>(0),
            )?,
        ))
    }))
    .unwrap();
    assert_eq!(
        before.0, after.0,
        "rejected replay must preserve the durable receipt payload"
    );
    assert_eq!(
        before.1, after.1,
        "rejected replay must not append audit activity"
    );
    assert_eq!(
        before.2, after.2,
        "rejected replay must not touch the old paper activity"
    );
    assert!(!after.0.is_empty(), "durable open receipt remains present");
}

#[test]
fn confirm_open_rechecks_paper_document_association_after_file_verification() {
    let (_dir, actor, persistence, store, paper_a, document_id) = fixture();
    let command = OpenPaperCommand {
        request_id: UUID(uuid::Uuid::new_v4().to_string()),
        paper_id: paper_a.clone(),
    };
    tauri::async_runtime::block_on(async {
        let registered = persistence.prepare_open(command.clone()).await.unwrap();
        let handle = store.open_verified(registered).await.unwrap();
        persistence
            .confirm_open(command.clone(), handle)
            .await
            .unwrap();
    });
    let prepared =
        tauri::async_runtime::block_on(persistence.prepare_open(command.clone())).unwrap();
    let handle = tauri::async_runtime::block_on(store.open_verified(prepared)).unwrap();
    let paper_b = uuid::Uuid::new_v4().to_string();
    let paper_a_id = paper_a.0.clone();
    let doc_id = document_id.0.clone();
    tauri::async_runtime::block_on(actor.submit(move|connection|{let tx=connection.transaction()?;tx.execute("INSERT INTO papers(id,title,year,review_type,lifecycle,revision,processing_initialized,active_document_id,created_at,updated_at) VALUES(?1,'Paper B',2025,'unknown','NEW',0,0,?2,'2026-10-02T00:00:00Z','2026-10-02T00:00:00Z')",rusqlite::params![paper_b,doc_id])?;tx.execute("UPDATE papers SET active_document_id=NULL WHERE id=?1",[paper_a_id])?;tx.execute("UPDATE documents SET paper_id=?1 WHERE id=?2",rusqlite::params![paper_b,doc_id])?;tx.commit()?;Ok(())})).unwrap();
    let receipt_request_id = command.request_id.0.clone();
    let before = tauri::async_runtime::block_on(actor.submit(move |connection| {
        Ok((
            connection.query_row(
                "SELECT result_json FROM operation_receipts WHERE request_id=?1",
                [receipt_request_id],
                |row| row.get::<_, String>(0),
            )?,
            connection.query_row("SELECT COUNT(*) FROM audit_events", [], |row| {
                row.get::<_, i64>(0)
            })?,
        ))
    }))
    .unwrap();
    assert!(
        tauri::async_runtime::block_on(persistence.confirm_open(command.clone(), handle)).is_err()
    );
    let receipt_request_id = command.request_id.0.clone();
    let after = tauri::async_runtime::block_on(actor.submit(move |connection| {
        Ok((
            connection.query_row(
                "SELECT result_json FROM operation_receipts WHERE request_id=?1",
                [receipt_request_id],
                |row| row.get::<_, String>(0),
            )?,
            connection.query_row("SELECT COUNT(*) FROM audit_events", [], |row| {
                row.get::<_, i64>(0)
            })?,
        ))
    }))
    .unwrap();
    assert_eq!(
        before, after,
        "association drift between verification and confirm must not rewrite or duplicate the receipt"
    );
}

#[test]
fn open_failure_rolls_back_activity_session_audit_and_receipt_together() {
    let (_dir, actor, persistence, store, paper_id, _document_id) = fixture();
    tauri::async_runtime::block_on(actor.submit(|connection|{
        connection.execute_batch("CREATE TRIGGER fail_reader_open_audit BEFORE INSERT ON audit_events WHEN NEW.action='reader.paper_opened' BEGIN SELECT RAISE(ABORT,'injected reader audit failure'); END;")?;
        Ok(())
    })).unwrap();
    let command = OpenPaperCommand {
        request_id: UUID(uuid::Uuid::new_v4().to_string()),
        paper_id: paper_id.clone(),
    };
    let registered =
        tauri::async_runtime::block_on(persistence.prepare_open(command.clone())).unwrap();
    let handle = tauri::async_runtime::block_on(store.open_verified(registered)).unwrap();
    assert!(
        tauri::async_runtime::block_on(persistence.confirm_open(command.clone(), handle)).is_err()
    );
    let request_id = command.request_id.0;
    tauri::async_runtime::block_on(actor.submit(move |connection| {
        let last_opened: Option<String> = connection.query_row(
            "SELECT last_opened_at FROM papers WHERE id=?1",
            [paper_id.0],
            |row| row.get(0),
        )?;
        let session: Option<String> = connection.query_row(
            "SELECT last_opened_paper_id FROM app_session WHERE singleton=1",
            [],
            |row| row.get(0),
        )?;
        let audit: i64 = connection.query_row(
            "SELECT COUNT(*) FROM audit_events WHERE request_id=?1",
            [&request_id],
            |row| row.get(0),
        )?;
        let receipt: i64 = connection.query_row(
            "SELECT COUNT(*) FROM operation_receipts WHERE request_id=?1",
            [request_id],
            |row| row.get(0),
        )?;
        assert_eq!(last_opened, None);
        assert_eq!(session, None);
        assert_eq!(audit, 0);
        assert_eq!(receipt, 0);
        Ok(())
    }))
    .unwrap();
}

#[test]
fn position_cas_is_per_document_idempotent_and_rejects_incompatible_receipt_replay() {
    let (_dir, actor, persistence, _store, _paper, document_a) = fixture();
    let id_b = uuid::Uuid::new_v4().to_string();
    let paper_b = uuid::Uuid::new_v4().to_string();
    let relative = format!("documents/{id_b}/original.pdf");
    let bytes = b"%PDF-1.4 second document";
    let path = actor.library_root().path().join(&relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
    let (id_for_sql, paper_for_sql, relative_for_sql, hash) = (
        id_b.clone(),
        paper_b.clone(),
        relative,
        format!("{:x}", Sha256::digest(bytes)),
    );
    tauri::async_runtime::block_on(actor.submit(move|connection|{
        let tx=connection.transaction()?;
        tx.execute("INSERT INTO papers(id,title,year,review_type,lifecycle,revision,processing_initialized,active_document_id,created_at,updated_at) VALUES(?1,'Paper dos',2025,'unknown','NEW',0,0,?2,'2026-10-02T00:00:00Z','2026-10-02T00:00:00Z')",rusqlite::params![paper_for_sql,id_for_sql])?;
        tx.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,'second.pdf',?3,?4,'application/pdf',?5,'2026-10-02T00:00:00Z','ACTIVE')",rusqlite::params![id_for_sql,paper_for_sql,relative_for_sql,hash,bytes.len() as i64])?;
        tx.commit()?;
        Ok(())
    })).unwrap();
    let request = UUID(uuid::Uuid::new_v4().to_string());
    let command = SaveReadingPositionCommand {
        request_id: request.clone(),
        document_id: document_a.clone(),
        expected_revision: 0,
        page_index: 2,
        zoom: 1.5,
    };
    let first =
        tauri::async_runtime::block_on(persistence.save_reading_position(command.clone())).unwrap();
    assert_eq!((first.page_index, first.revision), (2, 1));
    let replay =
        tauri::async_runtime::block_on(persistence.save_reading_position(command.clone())).unwrap();
    assert_eq!(replay, first);
    let stale = tauri::async_runtime::block_on(persistence.save_reading_position(
        SaveReadingPositionCommand {
            request_id: UUID(uuid::Uuid::new_v4().to_string()),
            page_index: 3,
            ..command.clone()
        },
    ))
    .unwrap_err();
    assert_eq!(
        stale.code,
        research_workbench_core::transport::error::ErrorCode::Conflict
    );
    assert_eq!(stale.details.unwrap()["currentRevision"], 1);
    let incompatible = tauri::async_runtime::block_on(persistence.save_reading_position(
        SaveReadingPositionCommand {
            page_index: 3,
            ..command
        },
    ))
    .unwrap_err();
    assert_eq!(
        incompatible.code,
        research_workbench_core::transport::error::ErrorCode::Conflict
    );
    let other = persistence.save_reading_position(SaveReadingPositionCommand {
        request_id: UUID(uuid::Uuid::new_v4().to_string()),
        document_id: UUID(id_b),
        expected_revision: 0,
        page_index: 4,
        zoom: 1.0,
    });
    let other = tauri::async_runtime::block_on(other).unwrap();
    assert_eq!((other.page_index, other.revision), (4, 1));
    let position_a =
        tauri::async_runtime::block_on(persistence.reading_position(document_a)).unwrap();
    assert_eq!(position_a.page_index, 2);
}

#[test]
fn library_import_move_reader_save_reopen_child_archive_restore_preserves_document_state() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("selected-source.pdf");
    let bytes = synthetic_two_page_pdf();
    research_workbench_core::adapters::documents::pdf_probe::validate_pdf(bytes.clone()).unwrap();
    fs::write(&source, &bytes).unwrap();
    let root = LibraryRoot::at(temp.path().join("library"));
    let actor = DbActor::start(root).unwrap();
    let library_id = actor.info().library_id.0.clone();
    let store = Arc::new(
        LocalDocumentStore::new(actor.library_root().clone(), library_id.clone()).unwrap(),
    );
    let maintenance = MaintenanceCoordinator::default();
    let requests =
        research_workbench_core::application::request_registry::RequestRegistry::default();
    let library = LibraryService::new(
        Arc::new(PathPicker {
            path: source.clone(),
        }),
        store.clone(),
        Arc::new(SqliteLibraryPersistence::new(
            actor.clone(),
            library_id.clone(),
        )),
        maintenance.clone(),
        requests.clone(),
    );
    let preview =
        tauri::async_runtime::block_on(library.select_pdf(UUID(uuid::Uuid::new_v4().to_string())))
            .unwrap()
            .unwrap();
    let moved_source = source.with_file_name("source-moved-after-stage.pdf");
    fs::rename(&source, &moved_source).unwrap();
    let imported = tauri::async_runtime::block_on(library.confirm_import(
        UUID(uuid::Uuid::new_v4().to_string()),
        preview.import_token,
        PaperMetadataInput {
            title: "Dos páginas sintéticas".into(),
            authors: vec!["Investigadora de prueba".into()],
            year: Some(2026),
            doi: None,
            venue: None,
            review_type: ReviewType::Unknown,
            domain: None,
        },
        None,
    ))
    .unwrap();
    let reader = ReaderService::new(
        Arc::new(SqliteReaderPersistence::new(
            actor.clone(),
            library_id.clone(),
        )),
        store.clone(),
        maintenance.clone(),
        requests.clone(),
        research_workbench_core::application::settings::RecoveryStatus::default(),
    );
    let opened = tauri::async_runtime::block_on(
        reader.open_paper(UUID(uuid::Uuid::new_v4().to_string()), imported.id.clone()),
    )
    .unwrap();
    assert_eq!(opened.document.id, imported.document_id);
    assert_eq!(
        opened.document.sha256,
        format!("{:x}", Sha256::digest(&bytes))
    );
    let saved =
        tauri::async_runtime::block_on(reader.save_reading_position(SaveReadingPositionCommand {
            request_id: UUID(uuid::Uuid::new_v4().to_string()),
            document_id: imported.document_id.clone(),
            expected_revision: 0,
            page_index: 2,
            zoom: 1.25,
        }))
        .unwrap();
    assert_eq!((saved.page_index, saved.revision), (2, 1));
    let library_root = actor.library_root().path().to_string_lossy().into_owned();
    let paper_id = imported.id.0.clone();
    let document_id = imported.document_id.0.clone();
    drop(reader);
    drop(library);
    drop(store);
    actor.shutdown(std::time::Duration::from_secs(3)).unwrap();
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "reader_reopen_child_process", "--nocapture"])
        .env("TASK03_REOPEN_ROOT", &library_root)
        .env("TASK03_REOPEN_PAPER", &paper_id)
        .env("TASK03_REOPEN_DOCUMENT", &document_id)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "reopen process failed:\n{}\n{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    let reopened = DbActor::start(LibraryRoot::at(library_root.into())).unwrap();
    let store = Arc::new(
        LocalDocumentStore::new(
            reopened.library_root().clone(),
            reopened.info().library_id.0.clone(),
        )
        .unwrap(),
    );
    let library = LibraryService::new(
        Arc::new(NoPicker),
        store.clone(),
        Arc::new(SqliteLibraryPersistence::new(
            reopened.clone(),
            reopened.info().library_id.0.clone(),
        )),
        MaintenanceCoordinator::default(),
        research_workbench_core::application::request_registry::RequestRegistry::default(),
    );
    let archived = tauri::async_runtime::block_on(library.archive_paper(
        UUID(uuid::Uuid::new_v4().to_string()),
        imported.id.clone(),
        imported.revision,
    ))
    .unwrap();
    assert_eq!(archived.document_id, imported.document_id);
    let restored = tauri::async_runtime::block_on(library.restore_paper(
        UUID(uuid::Uuid::new_v4().to_string()),
        imported.id.clone(),
        archived.revision,
    ))
    .unwrap();
    assert_eq!(restored.id, imported.id);
    assert_eq!(restored.document_id, imported.document_id);
    let persistence =
        SqliteReaderPersistence::new(reopened.clone(), reopened.info().library_id.0.clone());
    let document = tauri::async_runtime::block_on(
        persistence.registered_document(imported.document_id.clone()),
    )
    .unwrap();
    let position =
        tauri::async_runtime::block_on(persistence.reading_position(imported.document_id.clone()))
            .unwrap();
    assert_eq!(
        document.document.sha256,
        format!("{:x}", Sha256::digest(&bytes))
    );
    assert_eq!(
        (position.page_index, position.zoom, position.revision),
        (2, 1.25, 1)
    );
    reopened
        .shutdown(std::time::Duration::from_secs(3))
        .unwrap();
    assert!(moved_source.exists());
}

#[test]
fn pending_library_filesystem_saga_owns_shared_request_id_before_reader_access() {
    let (dir, actor, reader_persistence, store, paper_id, _document_id) = fixture();
    let source = dir.path().join("pending-saga.pdf");
    fs::write(&source, synthetic_two_page_pdf()).unwrap();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let library_id = actor.info().library_id.0.clone();
    let maintenance = MaintenanceCoordinator::default();
    let requests =
        research_workbench_core::application::request_registry::RequestRegistry::default();
    let gated = Arc::new(GatedPromotion {
        inner: store.clone(),
        entered: entered_tx,
        release: Arc::new(std::sync::Mutex::new(Some(release_rx))),
    });
    let library = Arc::new(LibraryService::new(
        Arc::new(PathPicker { path: source }),
        gated,
        Arc::new(SqliteLibraryPersistence::new(
            actor.clone(),
            library_id.clone(),
        )),
        maintenance.clone(),
        requests.clone(),
    ));
    let preview =
        tauri::async_runtime::block_on(library.select_pdf(UUID(uuid::Uuid::new_v4().to_string())))
            .unwrap()
            .unwrap();
    let request_id = UUID(uuid::Uuid::new_v4().to_string());
    let library_request = request_id.clone();
    let token = preview.import_token;
    let pending_import = tauri::async_runtime::spawn(async move {
        library
            .confirm_import(
                library_request,
                token,
                PaperMetadataInput {
                    title: "Saga pendiente".into(),
                    authors: vec![],
                    year: None,
                    doi: None,
                    venue: None,
                    review_type: ReviewType::Unknown,
                    domain: None,
                },
                None,
            )
            .await
    });
    entered_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let read_count = Arc::new(AtomicUsize::new(0));
    let reader = ReaderService::new(
        Arc::new(reader_persistence),
        Arc::new(CountingReadAccess {
            inner: store,
            count: read_count.clone(),
        }),
        maintenance.clone(),
        requests,
        research_workbench_core::application::settings::RecoveryStatus::default(),
    );
    let mut competing = Box::pin(reader.open_paper(request_id.clone(), paper_id));
    assert!(
        matches!(poll_once(competing.as_mut()), Poll::Pending),
        "the competing Reader request has polled into the shared requestId lock"
    );
    assert_eq!(
        read_count.load(Ordering::SeqCst),
        0,
        "incompatible Reader request must wait before opening the second document"
    );
    release_tx.send(()).unwrap();
    let imported = tauri::async_runtime::block_on(pending_import)
        .unwrap()
        .unwrap();
    assert_eq!(imported.title, "Saga pendiente");
    let conflict = tauri::async_runtime::block_on(competing.as_mut()).unwrap_err();
    assert_eq!(
        conflict.code,
        research_workbench_core::transport::error::ErrorCode::Conflict
    );
    assert_eq!(read_count.load(Ordering::SeqCst), 0);
    let request = request_id.0;
    tauri::async_runtime::block_on(actor.submit(move |connection| {
        let receipt: (i64, String) = connection.query_row(
            "SELECT COUNT(*),MAX(command) FROM operation_receipts WHERE request_id=?1",
            [&request],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        assert_eq!(receipt, (1, "library_confirm_import".to_owned()));
        let last: Option<String> = connection.query_row(
            "SELECT last_opened_paper_id FROM app_session WHERE singleton=1",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(last, None);
        Ok(())
    }))
    .unwrap();
    actor.shutdown(std::time::Duration::from_secs(3)).unwrap();
}

#[test]
fn protocol_rejects_unregistered_uuid_and_path_segments_before_file_access() {
    use research_workbench_core::modules::reader::protocol::document_id_from_request;
    let unknown = UUID(uuid::Uuid::new_v4().to_string());
    let (_dir, _actor, persistence, _store, _paper, _document) = fixture();
    assert_eq!(
        tauri::async_runtime::block_on(persistence.registered_document(unknown.clone()))
            .unwrap_err()
            .code,
        research_workbench_core::transport::error::ErrorCode::NotFound
    );
    let uri: tauri::http::Uri = format!("research://localhost/{}", unknown.0)
        .parse()
        .unwrap();
    assert!(
        document_id_from_request(
            &uri,
            "main",
            &tauri::http::Method::GET,
            Some("http://tauri.localhost")
        )
        .is_ok()
    );
    let traversal: tauri::http::Uri =
        "research://localhost/%2e%2e/00000000-0000-4000-8000-000000000001"
            .parse()
            .unwrap();
    assert!(
        document_id_from_request(
            &traversal,
            "main",
            &tauri::http::Method::GET,
            Some("http://tauri.localhost")
        )
        .is_err()
    );
}

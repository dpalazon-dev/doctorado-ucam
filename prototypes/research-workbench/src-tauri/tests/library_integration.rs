use research_workbench_core::{
    adapters::{
        documents::store::LocalDocumentStore,
        sqlite::{actor::DbActor, library_repository::SqliteLibraryPersistence},
        windows::paths::LibraryRoot,
    },
    application::library_ports::{
        CancelPlan, DocumentStore, ImportRecord, LibraryFuture, LibraryPersistence,
        NativePdfSelection, PreparedImport, ResourceInspection, SelectedPdf, StageCleanup,
        StageFailure, StageFuture,
    },
    application::{
        request_registry::RequestRegistry,
        settings::{RecoveryStatus, SettingsQuery},
    },
    desktop::maintenance::MaintenanceCoordinator,
    domain::library::{normalize_doi, normalize_metadata},
    modules::library::service::LibraryService,
    transport::dto::{PaperMetadataInput, ReviewType, UUID},
};
use std::{
    collections::VecDeque,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

fn metadata(doi: Option<&str>) -> PaperMetadataInput {
    PaperMetadataInput {
        title: "  Test title  ".into(),
        authors: vec![" Test Author One ".into()],
        year: Some(2024),
        doi: doi.map(str::to_owned),
        venue: Some(" Revista ".into()),
        review_type: ReviewType::Unknown,
        domain: Some(" Agua ".into()),
    }
}

#[derive(Default)]
struct FakePicker {
    selections: Mutex<VecDeque<Option<SelectedPdf>>>,
}
impl NativePdfSelection for FakePicker {
    fn select_pdf(&self) -> LibraryFuture<'_, Option<SelectedPdf>> {
        Box::pin(async move { Ok(self.selections.lock().unwrap().pop_front().flatten()) })
    }
}
struct PartialFailureStore {
    inner: Arc<LocalDocumentStore>,
    root: PathBuf,
    leave_partial: bool,
}
impl DocumentStore for PartialFailureStore {
    fn stage(&self, operation_id: String, _selected: SelectedPdf) -> StageFuture {
        let root = self.root.clone();
        let leave_partial = self.leave_partial;
        Box::pin(async move {
            if leave_partial {
                let dir = root.join("staging").join(operation_id);
                fs::create_dir_all(&dir).unwrap();
                fs::write(dir.join("source.pdf"), b"partial bytes").unwrap();
            }
            Err(StageFailure::new(
                research_workbench_core::transport::error::AppError::new(
                    research_workbench_core::transport::error::ErrorCode::StorageUnavailable,
                ),
                StageCleanup::Pending,
            ))
        })
    }
    fn inspect_owned(&self, record: ImportRecord) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.inspect_owned(record)
    }
    fn promote(&self, prepared: PreparedImport) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.promote(prepared)
    }
    fn cleanup_owned(&self, plan: CancelPlan) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.cleanup_owned(plan)
    }
}
struct GatedDocumentStore {
    inner: Arc<LocalDocumentStore>,
    entered: Arc<std::sync::Barrier>,
    release: Arc<std::sync::Barrier>,
}
impl DocumentStore for GatedDocumentStore {
    fn stage(&self, operation_id: String, selected: SelectedPdf) -> StageFuture {
        let inner = self.inner.clone();
        let entered = self.entered.clone();
        let release = self.release.clone();
        Box::pin(async move {
            tauri::async_runtime::spawn_blocking(move || {
                entered.wait();
                release.wait();
            })
            .await
            .map_err(|_| {
                StageFailure::new(
                    research_workbench_core::transport::error::AppError::new(
                        research_workbench_core::transport::error::ErrorCode::StorageUnavailable,
                    ),
                    StageCleanup::Pending,
                )
            })?;
            inner.stage(operation_id, selected).await
        })
    }
    fn inspect_owned(&self, record: ImportRecord) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.inspect_owned(record)
    }
    fn promote(&self, prepared: PreparedImport) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.promote(prepared)
    }
    fn cleanup_owned(&self, plan: CancelPlan) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.cleanup_owned(plan)
    }
}
struct GatedRecoveryStore {
    inner: Arc<LocalDocumentStore>,
    entered: Arc<std::sync::Barrier>,
    release: Arc<std::sync::Barrier>,
}
impl DocumentStore for GatedRecoveryStore {
    fn stage(&self, operation_id: String, selected: SelectedPdf) -> StageFuture {
        self.inner.stage(operation_id, selected)
    }
    fn inspect_owned(&self, record: ImportRecord) -> LibraryFuture<'static, ResourceInspection> {
        let inner = self.inner.clone();
        let entered = self.entered.clone();
        let release = self.release.clone();
        Box::pin(async move {
            tauri::async_runtime::spawn_blocking(move || {
                entered.wait();
                release.wait();
            })
            .await
            .map_err(|_| {
                research_workbench_core::transport::error::AppError::new(
                    research_workbench_core::transport::error::ErrorCode::StorageUnavailable,
                )
            })?;
            inner.inspect_owned(record).await
        })
    }
    fn promote(&self, prepared: PreparedImport) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.promote(prepared)
    }
    fn cleanup_owned(&self, plan: CancelPlan) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.cleanup_owned(plan)
    }
}
struct GatedCleanupStore {
    inner: Arc<LocalDocumentStore>,
    entered: Arc<std::sync::Barrier>,
    release: Arc<std::sync::Barrier>,
}
impl DocumentStore for GatedCleanupStore {
    fn stage(&self, operation_id: String, selected: SelectedPdf) -> StageFuture {
        self.inner.stage(operation_id, selected)
    }
    fn inspect_owned(&self, record: ImportRecord) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.inspect_owned(record)
    }
    fn promote(&self, prepared: PreparedImport) -> LibraryFuture<'static, ResourceInspection> {
        self.inner.promote(prepared)
    }
    fn cleanup_owned(&self, plan: CancelPlan) -> LibraryFuture<'static, ResourceInspection> {
        let inner = self.inner.clone();
        let entered = self.entered.clone();
        let release = self.release.clone();
        Box::pin(async move {
            tauri::async_runtime::spawn_blocking(move || {
                entered.wait();
                release.wait();
            })
            .await
            .map_err(|_| {
                research_workbench_core::transport::error::AppError::new(
                    research_workbench_core::transport::error::ErrorCode::StorageUnavailable,
                )
            })?;
            inner.cleanup_owned(plan).await
        })
    }
}
struct Fixture {
    _dir: tempfile::TempDir,
    root: LibraryRoot,
    actor: DbActor,
    service: LibraryService,
    picker: Arc<FakePicker>,
    documents: Arc<LocalDocumentStore>,
    persistence: Arc<SqliteLibraryPersistence>,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let requested_root = LibraryRoot::at(dir.path().join("library"));
        let actor = DbActor::start(requested_root).unwrap();
        let root = actor.library_root().clone();
        let picker = Arc::new(FakePicker::default());
        let documents = Arc::new(
            LocalDocumentStore::new(root.clone(), actor.info().library_id.0.clone()).unwrap(),
        );
        let persistence = Arc::new(SqliteLibraryPersistence::new(
            actor.clone(),
            actor.info().library_id.0.clone(),
        ));
        let service = LibraryService::new(
            picker.clone(),
            documents.clone(),
            persistence.clone(),
            MaintenanceCoordinator::default(),
            RequestRegistry::default(),
        );
        Self {
            _dir: dir,
            root,
            actor,
            service,
            picker,
            documents,
            persistence,
        }
    }
    fn select(&self, path: PathBuf) -> research_workbench_core::transport::dto::ImportPreviewDto {
        self.picker
            .selections
            .lock()
            .unwrap()
            .push_back(Some(SelectedPdf {
                filename: path.file_name().unwrap().to_string_lossy().into_owned(),
                path,
            }));
        tauri::async_runtime::block_on(self.service.select_pdf(UUID::new()))
            .unwrap()
            .unwrap()
    }
}

fn assert_latest_stage_failure(fixture: &Fixture, code: &str, cleanup: &str) {
    let (state, raw, staging_path) = tauri::async_runtime::block_on(
        fixture.actor.submit(|connection| {
            connection
                .query_row(
                    "SELECT state,error_json,staging_path FROM import_operations ORDER BY rowid DESC LIMIT 1",
                    [],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?)),
                )
                .map_err(Into::into)
        }),
    )
    .unwrap();
    assert_eq!(state, "FAILED");
    let error: serde_json::Value = serde_json::from_str(raw.as_deref().unwrap()).unwrap();
    assert_eq!(error["version"], 1);
    assert_eq!(error["kind"], "stageFailure");
    assert_eq!(error["code"], code);
    assert_eq!(error["cleanup"], cleanup);
    if cleanup == "DONE" {
        assert!(!fixture.root.path().join(staging_path).exists());
    }
}
fn valid_pdf() -> Vec<u8> {
    let mut bytes = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for object in [b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".as_slice(),b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] >>\nendobj\n".as_slice(),b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << >> >>\nendobj\n".as_slice(),b"4 0 obj\n<< /Subject (/Encrypt is ordinary text) >>\nendobj\n".as_slice()]{offsets.push(bytes.len());bytes.extend_from_slice(object);}
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

fn pdf_with_page_dictionaries(pages: &str, page: &str) -> Vec<u8> {
    let objects = [
        "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_string(),
        format!("2 0 obj\n{pages}\nendobj\n"),
        format!("3 0 obj\n{page}\nendobj\n"),
        "4 0 obj\n<< /Subject (fixture) >>\nendobj\n".to_string(),
    ];
    let mut bytes = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for object in objects {
        offsets.push(bytes.len());
        bytes.extend_from_slice(object.as_bytes());
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

fn incremental_pdf() -> Vec<u8> {
    let mut bytes = valid_pdf();
    let marker = b"startxref\n";
    let marker_at = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
        .unwrap();
    let old_xref = std::str::from_utf8(&bytes[marker_at + marker.len()..])
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let object_offset = bytes.len();
    bytes.extend_from_slice(b"5 0 obj\n<< /Title (incremental) >>\nendobj\n");
    let xref_offset = bytes.len();
    bytes.extend_from_slice(format!("xref\n5 1\n{object_offset:010} 00000 n \ntrailer\n<< /Size 6 /Root 1 0 R /Prev {old_xref} >>\nstartxref\n{xref_offset}\n%%EOF\n").as_bytes());
    bytes
}

fn object_stream_pdf() -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".as_slice(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] >>".as_slice(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << >> >>".as_slice(),
    ];
    let offsets = [0, objects[0].len(), objects[0].len() + objects[1].len()];
    let header = format!("1 0 2 {} 3 {} ", offsets[1], offsets[2]);
    let first = header.len();
    let mut stream = header.into_bytes();
    for object in objects {
        stream.extend_from_slice(object);
    }
    let mut bytes = b"%PDF-1.5\n".to_vec();
    let object_stream_offset = bytes.len();
    bytes.extend_from_slice(
        format!(
            "4 0 obj\n<< /Type /ObjStm /N 3 /First {first} /Length {} >>\nstream\n",
            stream.len()
        )
        .as_bytes(),
    );
    bytes.extend_from_slice(&stream);
    bytes.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_offset = bytes.len();
    let mut entries = Vec::new();
    let mut entry = |kind: u8, field2: u32, field3: u16| {
        entries.push(kind);
        entries.extend_from_slice(&field2.to_be_bytes());
        entries.extend_from_slice(&field3.to_be_bytes());
    };
    entry(0, 0, u16::MAX);
    entry(2, 4, 0);
    entry(2, 4, 1);
    entry(2, 4, 2);
    entry(1, object_stream_offset as u32, 0);
    entry(1, xref_offset as u32, 0);
    bytes.extend_from_slice(
        format!(
            "5 0 obj\n<< /Type /XRef /Size 6 /W [1 4 2] /Root 1 0 R /Length {} >>\nstream\n",
            entries.len()
        )
        .as_bytes(),
    );
    bytes.extend_from_slice(&entries);
    bytes.extend_from_slice(
        format!("\nendstream\nendobj\nstartxref\n{xref_offset}\n%%EOF\n").as_bytes(),
    );
    bytes
}

#[test]
fn duplicate_doi_normalized() {
    assert_eq!(
        normalize_doi(" DOI: HTTPS://DOI.ORG/10.1234/ABC.X ".into()).unwrap(),
        "10.1234/abc.x"
    );
    let normalized = normalize_metadata(metadata(Some("https://doi.org/10.1234/ABC.X"))).unwrap();
    assert_eq!(normalized.doi.as_deref(), Some("10.1234/abc.x"));
}

#[test]
fn metadata_is_trimmed_and_invalid_doi_rejected() {
    let normalized = normalize_metadata(metadata(None)).unwrap();
    assert_eq!(normalized.title, "Test title");
    assert_eq!(normalized.authors, ["Test Author One"]);
    assert!(normalize_doi("doi not real".into()).is_err());
}

#[test]
fn import_survives_original_move() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("fuente original.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source.clone());
    fs::remove_file(source).unwrap();
    let mut input = metadata(Some(" DOI: 10.1234/Test.ID "));
    input.title = "Paper importado".into();
    let request_id = UUID::new();
    let paper = tauri::async_runtime::block_on(fixture.service.confirm_import(
        request_id.clone(),
        preview.import_token,
        input,
        None,
    ))
    .unwrap();
    assert_eq!(paper.title, "Paper importado");
    let document = fixture
        .root
        .path()
        .join(format!("documents/{}/original.pdf", paper.document_id.0));
    assert!(document.is_file());
    assert_eq!(fs::read(document).unwrap(), valid_pdf());
    let initialized=tauri::async_runtime::block_on(fixture.actor.submit({let id=paper.id.0.clone();move|connection|Ok(connection.query_row("SELECT p.processing_initialized,p.current_phase,(SELECT count(*) FROM paper_phases WHERE paper_id=p.id),(SELECT count(*) FROM phase_answers WHERE paper_id=p.id) FROM papers p WHERE p.id=?1",[id],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?)))?)})).unwrap();
    assert_eq!(initialized, (1, "PRE".into(), 3, 0));
    let mut changed_metadata = metadata(None);
    changed_metadata.title = "Updated effective title".into();
    tauri::async_runtime::block_on(fixture.service.update_metadata(
        UUID::new(),
        paper.id.clone(),
        paper.revision,
        changed_metadata,
    ))
    .unwrap();
    let after_metadata=tauri::async_runtime::block_on(fixture.actor.submit({let id=paper.id.0.clone();move|connection|{let mut statement=connection.prepare("SELECT phase_code,state,revision FROM paper_phases WHERE paper_id=?1 ORDER BY CASE phase_code WHEN 'PRE' THEN 0 WHEN 'P1' THEN 1 ELSE 2 END")?;let rows=statement.query_map([id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?)))?;rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)}})).unwrap();
    assert_eq!(
        after_metadata,
        vec![
            ("PRE".into(), "IN_PROGRESS".into(), 2),
            ("P1".into(), "NOT_STARTED".into(), 0),
            ("P2".into(), "NOT_STARTED".into(), 0)
        ]
    );
    let entity_events = tauri::async_runtime::block_on(fixture.actor.submit({
        let paper_id = paper.id.0.clone();
        let request_id = request_id.0;
        move |connection| {
            connection
                .query_row(
                    "SELECT count(*) FROM audit_events WHERE request_id=?1 AND entity_id=?2 AND action='paper.imported'",
                    rusqlite::params![request_id, paper_id],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(Into::into)
        }
    }))
    .unwrap();
    assert_eq!(entity_events, 1);
    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
    let reopened = DbActor::start(fixture.root.clone()).unwrap();
    let saved = tauri::async_runtime::block_on(reopened.submit({
        let id = paper.id.0.clone();
        move |c| {
            c.query_row("SELECT title FROM papers WHERE id=?1", [id], |r| {
                r.get::<_, String>(0)
            })
            .map_err(Into::into)
        }
    }))
    .unwrap();
    assert_eq!(saved, "Updated effective title");
    reopened
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
}

#[test]
fn failed_workflow_initialization_rolls_back_import_confirmation() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("failed-workflow-init.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source);
    let request_id = UUID::new();
    tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
        connection.execute_batch("CREATE TRIGGER fail_workflow_initialization BEFORE INSERT ON paper_phases BEGIN SELECT RAISE(ABORT,'injected workflow initialization failure'); END;")?;
        Ok(())
    })).unwrap();
    let result = tauri::async_runtime::block_on(fixture.service.confirm_import(
        request_id.clone(),
        preview.import_token,
        metadata(None),
        None,
    ));
    assert!(
        result.is_err(),
        "workflow initialization failure must reject the import confirmation"
    );
    let id = request_id.0;
    let durable = tauri::async_runtime::block_on(fixture.actor.submit(move |connection| Ok((
        connection.query_row("SELECT count(*) FROM papers", [], |row| row.get::<_, i64>(0))?,
        connection.query_row("SELECT count(*) FROM documents", [], |row| row.get::<_, i64>(0))?,
        connection.query_row("SELECT count(*) FROM paper_phases", [], |row| row.get::<_, i64>(0))?,
        connection.query_row("SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_confirm_import'", [&id], |row| row.get::<_, i64>(0))?,
        connection.query_row("SELECT count(*) FROM audit_events WHERE request_id=?1 AND action='paper.imported'", [&id], |row| row.get::<_, i64>(0))?,
    )))).unwrap();
    assert_eq!(durable, (0, 0, 0, 0, 0));
    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(5))
        .unwrap();
}

#[test]
fn document_store_requires_the_actor_bound_root_identity() {
    let fixture = Fixture::new();
    let bound = fixture.actor.library_root().clone();
    assert!(bound.is_bound());
    assert!(bound.clone().is_bound());
    let unbound = LibraryRoot::at(bound.path().to_path_buf());
    let result = LocalDocumentStore::new(unbound, fixture.actor.info().library_id.0.clone());
    assert!(matches!(
        result,
        Err(error)
            if error.code == research_workbench_core::transport::error::ErrorCode::PathNotAllowed
    ));

    let replacement = bound.path().with_extension("replaced");
    assert!(fs::rename(bound.path(), replacement).is_err());
    assert!(bound.path().is_dir());
}

#[test]
fn xref_object_stream_and_incremental_pdf_are_accepted() {
    let fixture = Fixture::new();
    let streamed = import_file(
        &fixture,
        &object_stream_pdf(),
        "object-stream.pdf",
        metadata(None),
    );
    let incremented = import_file(
        &fixture,
        &incremental_pdf(),
        "incremental.pdf",
        metadata(None),
    );
    assert_ne!(streamed.document_id, incremented.document_id);
    assert!(
        fixture
            .root
            .path()
            .join(format!("documents/{}/original.pdf", streamed.document_id.0))
            .is_file()
    );
    assert!(
        fixture
            .root
            .path()
            .join(format!(
                "documents/{}/original.pdf",
                incremented.document_id.0
            ))
            .is_file()
    );
}

#[test]
fn invalid_xref_root_and_cyclic_page_tree_never_publish_preview() {
    let fixture = Fixture::new();
    let mut invalids = Vec::new();
    invalids.push(b"%PDF-1.4\n%%EOF\n".to_vec());
    let mut bad_root = valid_pdf();
    let root_at = bad_root
        .windows(b"/Root 1 0 R".len())
        .rposition(|window| window == b"/Root 1 0 R")
        .unwrap();
    bad_root.splice(
        root_at..root_at + b"/Root 1 0 R".len(),
        b"/Root 7 0 R".iter().copied(),
    );
    invalids.push(bad_root);
    let mut missing_page = valid_pdf();
    let kids = b"[3 0 R]";
    let kids_at = missing_page
        .windows(kids.len())
        .position(|window| window == kids)
        .unwrap();
    missing_page.splice(kids_at..kids_at + kids.len(), b"[4 0 R]".iter().copied());
    invalids.push(missing_page);
    let mut empty_kids = valid_pdf();
    let kids_at = empty_kids
        .windows(kids.len())
        .position(|window| window == kids)
        .unwrap();
    empty_kids.splice(kids_at..kids_at + kids.len(), b"[]     ".iter().copied());
    invalids.push(empty_kids);
    let mut cycle = valid_pdf();
    let cycle_at = cycle
        .windows(b"/Kids [3 0 R]".len())
        .position(|window| window == b"/Kids [3 0 R]")
        .unwrap();
    cycle.splice(
        cycle_at..cycle_at + b"/Kids [3 0 R]".len(),
        b"/Kids [2 0 R]".iter().copied(),
    );
    invalids.push(cycle);
    let mut bad_offset = valid_pdf();
    let marker = b"startxref\n";
    let marker_at = bad_offset
        .windows(marker.len())
        .rposition(|window| window == marker)
        .unwrap();
    let digit = marker_at + marker.len();
    bad_offset[digit] = if bad_offset[digit] == b'9' {
        b'0'
    } else {
        bad_offset[digit] + 1
    };
    invalids.push(bad_offset);

    for (index, bytes) in invalids.into_iter().enumerate() {
        let source = fixture
            ._dir
            .path()
            .join(format!("bad-structure-{index}.pdf"));
        fs::write(&source, bytes).unwrap();
        fixture
            .picker
            .selections
            .lock()
            .unwrap()
            .push_back(Some(SelectedPdf {
                filename: format!("bad-{index}.pdf"),
                path: source,
            }));
        let error =
            tauri::async_runtime::block_on(fixture.service.select_pdf(UUID::new())).unwrap_err();
        assert_eq!(
            error.code,
            research_workbench_core::transport::dto::IpcErrorCode::InvalidPdf
        );
        assert_latest_stage_failure(&fixture, "InvalidPdf", "DONE");
    }
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|c| Ok(c.query_row(
            "SELECT count(*) FROM papers",
            [],
            |r| r.get::<_, i64>(0)
        )?)))
        .unwrap(),
        0
    );
}

#[test]
fn encrypt_token_inside_document_metadata_is_not_encryption() {
    let fixture = Fixture::new();
    let paper = import_file(
        &fixture,
        &valid_pdf(),
        "ordinary-encrypt-text.pdf",
        metadata(None),
    );
    assert!(
        fixture
            .root
            .path()
            .join(format!("documents/{}/original.pdf", paper.document_id.0))
            .is_file()
    );
}

#[test]
fn encrypted_pdfs_with_and_without_user_password_are_rejected() {
    let fixture = Fixture::new();
    for (name, bytes) in [
        (
            "encrypted-user.pdf",
            include_bytes!("fixtures/task02/encrypted-user.pdf").as_slice(),
        ),
        (
            "encrypted-empty-user.pdf",
            include_bytes!("fixtures/task02/encrypted-empty-user.pdf").as_slice(),
        ),
    ] {
        let source = fixture._dir.path().join(name);
        fs::write(&source, bytes).unwrap();
        fixture
            .picker
            .selections
            .lock()
            .unwrap()
            .push_back(Some(SelectedPdf {
                filename: name.into(),
                path: source,
            }));
        let error =
            tauri::async_runtime::block_on(fixture.service.select_pdf(UUID::new())).unwrap_err();
        assert_eq!(
            error.code,
            research_workbench_core::transport::dto::IpcErrorCode::InvalidPdf
        );
        assert_latest_stage_failure(&fixture, "InvalidPdf", "DONE");
    }
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|c| Ok(c.query_row(
            "SELECT count(*) FROM papers",
            [],
            |r| r.get::<_, i64>(0)
        )?)))
        .unwrap(),
        0
    );
}

fn import_file(
    fixture: &Fixture,
    bytes: &[u8],
    name: &str,
    metadata: PaperMetadataInput,
) -> research_workbench_core::transport::dto::PaperDto {
    let source = fixture._dir.path().join(name);
    fs::write(&source, bytes).unwrap();
    let preview = fixture.select(source);
    tauri::async_runtime::block_on(fixture.service.confirm_import(
        UUID::new(),
        preview.import_token,
        metadata,
        None,
    ))
    .unwrap()
}

#[test]
fn duplicate_doi_normalized_prevents_second_paper() {
    let fixture = Fixture::new();
    let pdf = valid_pdf();
    let mut first = metadata(Some("doi:10.1234/SAME"));
    first.title = "Original".into();
    let original = import_file(&fixture, &pdf, "first.pdf", first);
    let second = fixture._dir.path().join("second.pdf");
    let mut other = pdf.clone();
    other.extend_from_slice(b" ");
    fs::write(&second, other).unwrap();
    let preview = fixture.select(second);
    let mut duplicate = metadata(Some("https://doi.org/10.1234/same"));
    duplicate.title = "Must not replace".into();
    let error = tauri::async_runtime::block_on(fixture.service.confirm_import(
        UUID::new(),
        preview.import_token,
        duplicate,
        None,
    ))
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::DuplicateDecisionRequired
    );
    let candidate = &error.details.as_ref().unwrap()["candidates"][0];
    assert_eq!(candidate["paperId"], original.id.0);
    assert_eq!(candidate["reasons"], serde_json::json!(["doi"]));
    let count = tauri::async_runtime::block_on(fixture.actor.submit(|c| {
        c.query_row("SELECT count(*) FROM papers", [], |r| r.get::<_, i64>(0))
            .map_err(Into::into)
    }))
    .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn duplicate_hash_requires_decision() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let mut first = metadata(None);
    first.title = "Paper conservado".into();
    let existing = import_file(&fixture, &bytes, "original.pdf", first);
    let source = fixture._dir.path().join("copy.pdf");
    fs::write(&source, &bytes).unwrap();
    let preview = fixture.select(source);
    assert_eq!(preview.candidates.len(), 1);
    assert_eq!(preview.candidates[0].paper_id.0, existing.id.0);
    let mut incoming = metadata(None);
    incoming.title = "Title that must not be merged".into();
    let error = tauri::async_runtime::block_on(fixture.service.confirm_import(
        UUID::new(),
        preview.import_token.clone(),
        incoming.clone(),
        None,
    ))
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::DuplicateDecisionRequired
    );
    let candidate = &error.details.as_ref().unwrap()["candidates"][0];
    assert_eq!(candidate["paperId"], existing.id.0);
    assert_eq!(candidate["reasons"], serde_json::json!(["sha256"]));
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|c| Ok(c.query_row(
            "SELECT count(*) FROM papers",
            [],
            |r| r.get::<_, i64>(0)
        )?)))
        .unwrap(),
        1
    );
}

#[test]
fn duplicate_confirmation_reports_combined_reasons() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let mut original = metadata(Some("doi:10.1234/combined"));
    original.title = "Existing combined duplicate".into();
    let existing = import_file(&fixture, &bytes, "combined-original.pdf", original);
    let source = fixture._dir.path().join("combined-copy.pdf");
    fs::write(&source, bytes).unwrap();
    let preview = fixture.select(source);

    let error = tauri::async_runtime::block_on(fixture.service.confirm_import(
        UUID::new(),
        preview.import_token,
        metadata(Some("https://doi.org/10.1234/combined")),
        None,
    ))
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::DuplicateDecisionRequired
    );
    let candidate = &error.details.as_ref().unwrap()["candidates"][0];
    assert_eq!(candidate["paperId"], existing.id.0);
    assert_eq!(candidate["reasons"], serde_json::json!(["doi", "sha256"]));
}

#[test]
fn late_duplicate_winner_is_reported_by_transactional_commit() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let source = fixture._dir.path().join("late-duplicate-copy.pdf");
    fs::write(&source, &bytes).unwrap();
    let preview = fixture.select(source);
    let request_id = UUID::new();
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        request_id.clone(),
        preview.import_token,
        metadata(None),
        None,
    ))
    .unwrap();
    let verified =
        tauri::async_runtime::block_on(fixture.documents.promote(prepared.clone())).unwrap();
    tauri::async_runtime::block_on(fixture.persistence.record_promoted(prepared.clone())).unwrap();

    let mut winner_metadata = metadata(None);
    winner_metadata.title = "Late duplicate winner".into();
    let winner = import_file(
        &fixture,
        &bytes,
        "late-duplicate-winner.pdf",
        winner_metadata,
    );
    let error = tauri::async_runtime::block_on(
        fixture
            .persistence
            .commit_confirmation(request_id, prepared, verified),
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::DuplicateDecisionRequired
    );
    let candidate = &error.details.as_ref().unwrap()["candidates"][0];
    assert_eq!(candidate["paperId"], winner.id.0);
    assert_eq!(candidate["reasons"], serde_json::json!(["sha256"]));
}

#[test]
fn late_duplicate_loser_cancel_removes_only_its_durably_promoted_file() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let source = fixture._dir.path().join("late-loser.pdf");
    fs::write(&source, &bytes).unwrap();
    let preview = fixture.select(source);
    let request_id = UUID::new();
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        request_id.clone(),
        preview.import_token.clone(),
        metadata(None),
        None,
    ))
    .unwrap();
    let verified =
        tauri::async_runtime::block_on(fixture.documents.promote(prepared.clone())).unwrap();
    tauri::async_runtime::block_on(fixture.persistence.record_promoted(prepared.clone())).unwrap();

    let winner = import_file(&fixture, &bytes, "late-loser-winner.pdf", metadata(None));
    let error = tauri::async_runtime::block_on(fixture.persistence.commit_confirmation(
        request_id.clone(),
        prepared.clone(),
        verified,
    ))
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::DuplicateDecisionRequired
    );
    assert_eq!(
        error.details.unwrap()["candidates"][0]["paperId"],
        winner.id.0
    );

    let cancel_request = UUID::new();
    tauri::async_runtime::block_on(
        fixture
            .service
            .cancel_import(cancel_request.clone(), preview.import_token.clone()),
    )
    .unwrap();
    tauri::async_runtime::block_on(
        fixture
            .service
            .cancel_import(cancel_request.clone(), preview.import_token.clone()),
    )
    .unwrap();

    let loser_path = fixture.root.path().join(&prepared.destination_path);
    assert!(!loser_path.exists());
    let winner_path = fixture
        .root
        .path()
        .join(&winner.document_id.0)
        .join("original.pdf");
    let winner_path = if winner_path.exists() {
        winner_path
    } else {
        fixture
            .root
            .path()
            .join("documents")
            .join(&winner.document_id.0)
            .join("original.pdf")
    };
    assert_eq!(fs::read(winner_path).unwrap(), bytes);
    let (state, error_json, receipt_count, papers) = tauri::async_runtime::block_on(
        fixture.actor.submit({
            let token = preview.import_token.0.clone();
            let cancel_request = cancel_request.0;
            move |connection| {
                let (state, error_json) = connection.query_row(
                    "SELECT state,error_json FROM import_operations WHERE import_token=?1",
                    [&token],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )?;
                let receipts = connection.query_row(
                    "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_cancel_import'",
                    [&cancel_request],
                    |row| row.get::<_, i64>(0),
                )?;
                let papers = connection.query_row(
                    "SELECT count(*) FROM papers",
                    [],
                    |row| row.get::<_, i64>(0),
                )?;
                Ok((state, error_json, receipts, papers))
            }
        }),
    )
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(state, "FAILED");
    assert_eq!(error["promotionConfirmed"], true);
    assert_eq!(error["cleanup"], "DONE");
    assert_eq!(receipt_count, 1);
    assert_eq!(papers, 1);
}

#[test]
fn destination_without_durable_promotion_authority_is_preserved() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let source = fixture._dir.path().join("unpromoted-target.pdf");
    fs::write(&source, &bytes).unwrap();
    let preview = fixture.select(source);
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        UUID::new(),
        preview.import_token.clone(),
        metadata(None),
        None,
    ))
    .unwrap();
    let target = fixture.root.path().join(&prepared.destination_path);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, &bytes).unwrap();

    let error = tauri::async_runtime::block_on(
        fixture
            .service
            .cancel_import(UUID::new(), preview.import_token.clone()),
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::ImportRecoveryRequired
    );
    assert_eq!(fs::read(&target).unwrap(), bytes);
    assert!(
        fixture
            .root
            .path()
            .join(&prepared.record.staging_path)
            .exists()
    );
    let (state, error_json) = tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
        connection
            .query_row(
                "SELECT state,error_json FROM import_operations",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .map_err(Into::into)
    }))
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(state, "FAILED");
    assert_eq!(error["promotionConfirmed"], false);
    assert_eq!(error["cleanup"], "PENDING");
    let (papers, documents) = tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
        connection
            .query_row(
                "SELECT (SELECT count(*) FROM papers),(SELECT count(*) FROM documents)",
                [],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .map_err(Into::into)
    }))
    .unwrap();
    assert_eq!((papers, documents), (0, 0));
}

#[test]
fn durably_promoted_cancel_pending_is_recovered_after_database_reopen() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let source = fixture._dir.path().join("reopen-promoted-loser.pdf");
    fs::write(&source, &bytes).unwrap();
    let preview = fixture.select(source);
    let confirmation_request = UUID::new();
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        confirmation_request.clone(),
        preview.import_token.clone(),
        metadata(None),
        None,
    ))
    .unwrap();
    let proof =
        tauri::async_runtime::block_on(fixture.documents.promote(prepared.clone())).unwrap();
    tauri::async_runtime::block_on(fixture.persistence.record_promoted(prepared.clone())).unwrap();
    import_file(
        &fixture,
        &bytes,
        "reopen-promoted-winner.pdf",
        metadata(None),
    );
    assert_eq!(
        tauri::async_runtime::block_on(fixture.persistence.commit_confirmation(
            confirmation_request,
            prepared.clone(),
            proof,
        ))
        .unwrap_err()
        .code,
        research_workbench_core::transport::dto::IpcErrorCode::DuplicateDecisionRequired
    );

    let cancellation_request = UUID::new();
    tauri::async_runtime::block_on(
        fixture
            .persistence
            .prepare_cancel(cancellation_request.clone(), preview.import_token.clone()),
    )
    .unwrap();
    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
    let actor = DbActor::start(fixture.root.clone()).unwrap();
    let documents = Arc::new(
        LocalDocumentStore::new(fixture.root.clone(), actor.info().library_id.0.clone()).unwrap(),
    );
    let persistence = Arc::new(SqliteLibraryPersistence::new(
        actor.clone(),
        actor.info().library_id.0.clone(),
    ));
    let restarted = LibraryService::new(
        fixture.picker.clone(),
        documents,
        persistence,
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    tauri::async_runtime::block_on(actor.submit(|connection| {
        connection.execute_batch(
            "CREATE TRIGGER fail_import_cancelled BEFORE INSERT ON audit_events WHEN NEW.action='import.cancelled' BEGIN SELECT RAISE(ABORT,'injected cancellation audit failure'); END;",
        )?;
        Ok(())
    }))
    .unwrap();
    let report = tauri::async_runtime::block_on(restarted.reconcile_imports()).unwrap();
    assert_eq!(report.recovered, 0);
    assert_eq!(report.issues.len(), 1);
    assert!(
        !fixture
            .root
            .path()
            .join(&prepared.destination_path)
            .exists()
    );
    let (error_json, receipts, events, papers) = tauri::async_runtime::block_on(actor.submit({
        let token = preview.import_token.0.clone();
        let request_id = cancellation_request.0.clone();
        move |connection| {
            let error_json = connection.query_row(
                "SELECT error_json FROM import_operations WHERE import_token=?1",
                [&token],
                |row| row.get::<_, String>(0),
            )?;
            let receipts = connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_cancel_import'",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )?;
            let events = connection.query_row(
                "SELECT count(*) FROM audit_events WHERE request_id=?1 AND entity_id=(SELECT id FROM import_operations WHERE import_token=?2) AND action='import.cancelled'",
                rusqlite::params![request_id, token],
                |row| row.get::<_, i64>(0),
            )?;
            let papers = connection.query_row("SELECT count(*) FROM papers", [], |row| {
                row.get::<_, i64>(0)
            })?;
            Ok((error_json, receipts, events, papers))
        }
    }))
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(error["promotionConfirmed"], true);
    assert_eq!(error["cleanup"], "PENDING");
    assert_eq!(receipts, 0);
    assert_eq!(events, 0);
    assert_eq!(papers, 1);
    tauri::async_runtime::block_on(actor.submit(|connection| {
        connection.execute_batch("DROP TRIGGER fail_import_cancelled;")?;
        Ok(())
    }))
    .unwrap();
    let report = tauri::async_runtime::block_on(restarted.reconcile_imports()).unwrap();
    assert_eq!(report.recovered, 1);
    assert!(report.issues.is_empty());
    tauri::async_runtime::block_on(
        restarted.cancel_import(cancellation_request.clone(), preview.import_token.clone()),
    )
    .unwrap();
    let (error_json, receipts, events, papers) = tauri::async_runtime::block_on(actor.submit({
        let token = preview.import_token.0.clone();
        let request_id = cancellation_request.0.clone();
        move |connection| {
            let error_json = connection.query_row(
                "SELECT error_json FROM import_operations WHERE import_token=?1",
                [&token],
                |row| row.get::<_, String>(0),
            )?;
            let receipts = connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_cancel_import'",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )?;
            let events = connection.query_row(
                "SELECT count(*) FROM audit_events WHERE request_id=?1 AND action='import.cancelled' AND entity_id=(SELECT id FROM import_operations WHERE import_token=?2)",
                rusqlite::params![request_id, token],
                |row| row.get::<_, i64>(0),
            )?;
            let papers = connection.query_row("SELECT count(*) FROM papers", [], |row| {
                row.get::<_, i64>(0)
            })?;
            Ok((error_json, receipts, events, papers))
        }
    }))
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(error["cleanup"], "DONE");
    assert_eq!(receipts, 1);
    assert_eq!(events, 1);
    assert_eq!(papers, 1);
    actor.shutdown(std::time::Duration::from_secs(2)).unwrap();
}

#[test]
fn cancellation_retry_without_promotion_flag_finishes_staging_only_intent() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("retry-cancel-missing-flag.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source);
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        UUID::new(),
        preview.import_token.clone(),
        metadata(None),
        None,
    ))
    .unwrap();
    let staging_path = fixture.root.path().join(&prepared.record.staging_path);
    let destination_path = fixture
        .root
        .path()
        .join(prepared.record.destination_path.as_ref().unwrap());
    let cancellation_request = UUID::new();
    tauri::async_runtime::block_on(
        fixture
            .persistence
            .prepare_cancel(cancellation_request.clone(), preview.import_token.clone()),
    )
    .unwrap();
    tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0.clone();
        move |connection| {
            connection.execute(
                "UPDATE import_operations SET error_json=json_remove(error_json,'$.promotionConfirmed') WHERE import_token=?1",
                [&token],
            )?;
            Ok(())
        }
    }))
    .unwrap();

    tauri::async_runtime::block_on(
        fixture
            .service
            .cancel_import(cancellation_request.clone(), preview.import_token.clone()),
    )
    .unwrap();
    tauri::async_runtime::block_on(
        fixture
            .service
            .cancel_import(cancellation_request.clone(), preview.import_token.clone()),
    )
    .unwrap();

    assert!(!staging_path.exists());
    assert!(!destination_path.exists());
    let (error_json, receipts, events) = tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0;
        let request_id = cancellation_request.0;
        move |connection| {
            let error_json = connection.query_row(
                "SELECT error_json FROM import_operations WHERE import_token=?1",
                [&token],
                |row| row.get::<_, String>(0),
            )?;
            let receipts = connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_cancel_import'",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )?;
            let events = connection.query_row(
                "SELECT count(*) FROM audit_events WHERE request_id=?1 AND entity_id=(SELECT id FROM import_operations WHERE import_token=?2) AND action='import.cancelled'",
                rusqlite::params![request_id, token],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((error_json, receipts, events))
        }
    }))
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(error["cleanup"], "DONE");
    assert_eq!(error["promotionConfirmed"], false);
    assert_eq!((receipts, events), (1, 1));
}

#[test]
fn missing_promotion_flag_never_authorizes_target_cleanup() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let source = fixture._dir.path().join("promoted-cancel-missing-flag.pdf");
    fs::write(&source, &bytes).unwrap();
    let preview = fixture.select(source);
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        UUID::new(),
        preview.import_token.clone(),
        metadata(None),
        None,
    ))
    .unwrap();
    tauri::async_runtime::block_on(fixture.documents.promote(prepared.clone())).unwrap();
    tauri::async_runtime::block_on(fixture.persistence.record_promoted(prepared.clone())).unwrap();
    let cancellation_request = UUID::new();
    tauri::async_runtime::block_on(
        fixture
            .persistence
            .prepare_cancel(cancellation_request.clone(), preview.import_token.clone()),
    )
    .unwrap();
    tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0.clone();
        move |connection| {
            connection.execute(
                "UPDATE import_operations SET error_json=json_remove(error_json,'$.promotionConfirmed') WHERE import_token=?1",
                [&token],
            )?;
            Ok(())
        }
    }))
    .unwrap();
    let destination = fixture.root.path().join(&prepared.destination_path);
    let staged = fixture.root.path().join(&prepared.record.staging_path);
    assert_eq!(fs::read(&destination).unwrap(), bytes);
    assert!(!staged.exists());

    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
    let actor = DbActor::start(fixture.root.clone()).unwrap();
    let documents = Arc::new(
        LocalDocumentStore::new(fixture.root.clone(), actor.info().library_id.0.clone()).unwrap(),
    );
    let persistence = Arc::new(SqliteLibraryPersistence::new(
        actor.clone(),
        actor.info().library_id.0.clone(),
    ));
    let restarted = LibraryService::new(
        fixture.picker.clone(),
        documents,
        persistence,
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let report = tauri::async_runtime::block_on(restarted.reconcile_imports()).unwrap();
    assert_eq!(report.recovered, 0);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(fs::read(&destination).unwrap(), bytes);
    let (error_json, receipts) = tauri::async_runtime::block_on(actor.submit({
        let token = preview.import_token.0;
        let request_id = cancellation_request.0;
        move |connection| {
            let error_json = connection.query_row(
                "SELECT error_json FROM import_operations WHERE import_token=?1",
                [&token],
                |row| row.get::<_, String>(0),
            )?;
            let receipts = connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_cancel_import'",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((error_json, receipts))
        }
    }))
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(error["cleanup"], "PENDING");
    assert!(error.get("promotionConfirmed").is_none());
    assert_eq!(receipts, 0);
    actor.shutdown(std::time::Duration::from_secs(2)).unwrap();
}

#[test]
fn unknown_cancellation_version_is_preserved_and_cannot_authorize_cleanup() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let source = fixture._dir.path().join("unknown-cancel-wrapper.pdf");
    fs::write(&source, &bytes).unwrap();
    let preview = fixture.select(source);
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        UUID::new(),
        preview.import_token.clone(),
        metadata(None),
        None,
    ))
    .unwrap();
    let proof =
        tauri::async_runtime::block_on(fixture.documents.promote(prepared.clone())).unwrap();
    tauri::async_runtime::block_on(fixture.persistence.record_promoted(prepared.clone())).unwrap();
    let cancellation_request = UUID::new();
    tauri::async_runtime::block_on(
        fixture
            .persistence
            .prepare_cancel(cancellation_request.clone(), preview.import_token.clone()),
    )
    .unwrap();
    assert!(proof.destination_matches);
    tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0.clone();
        move |connection| {
            connection.execute(
                "UPDATE import_operations SET error_json=json_set(error_json,'$.version',99) WHERE import_token=?1",
                [&token],
            )?;
            Ok(())
        }
    }))
    .unwrap();
    let destination = fixture.root.path().join(&prepared.destination_path);
    let before = fs::read(&destination).unwrap();

    let error = tauri::async_runtime::block_on(
        fixture
            .service
            .cancel_import(cancellation_request.clone(), preview.import_token.clone()),
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::ImportRecoveryRequired
    );
    assert_eq!(fs::read(&destination).unwrap(), before);
    let (error_json, receipts) = tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0.clone();
        let request_id = cancellation_request.0;
        move |connection| {
            let error_json = connection.query_row(
                "SELECT error_json FROM import_operations WHERE import_token=?1",
                [&token],
                |row| row.get::<_, String>(0),
            )?;
            let receipts = connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_cancel_import'",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((error_json, receipts))
        }
    }))
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(error["version"], 99);
    assert_eq!(error["cleanup"], "PENDING");
    assert_eq!(error["promotionConfirmed"], true);
    assert_eq!(receipts, 0);

    let mismatch_request = UUID::new();
    let mismatched_token = UUID::new().0;
    tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0.clone();
        let mismatched_token = mismatched_token.clone();
        move |connection| {
            connection.execute(
                "UPDATE import_operations SET error_json=json_set(error_json,'$.version',1,'$.receipt.payload.importToken',?1) WHERE import_token=?2",
                rusqlite::params![mismatched_token, token],
            )?;
            Ok(())
        }
    }))
    .unwrap();

    let error = tauri::async_runtime::block_on(
        fixture
            .service
            .cancel_import(mismatch_request.clone(), preview.import_token.clone()),
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::ImportRecoveryRequired
    );
    assert_eq!(fs::read(&destination).unwrap(), before);
    let (error_json, receipts) = tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0.clone();
        let request_id = mismatch_request.0;
        move |connection| {
            let error_json = connection.query_row(
                "SELECT error_json FROM import_operations WHERE import_token=?1",
                [&token],
                |row| row.get::<_, String>(0),
            )?;
            let receipts = connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_cancel_import'",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((error_json, receipts))
        }
    }))
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(error["version"], 1);
    assert_eq!(error["cleanup"], "PENDING");
    assert_eq!(error["receipt"]["payload"]["importToken"], mismatched_token);
    assert_eq!(receipts, 0);

    let invalid_boolean_request = UUID::new();
    tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0.clone();
        let token_for_payload = preview.import_token.0.clone();
        move |connection| {
            connection.execute(
                "UPDATE import_operations SET error_json=json_set(error_json,'$.receipt.payload.importToken',?1,'$.promotionConfirmed','false') WHERE import_token=?2",
                rusqlite::params![token_for_payload, token],
            )?;
            Ok(())
        }
    }))
    .unwrap();
    let error = tauri::async_runtime::block_on(fixture.service.cancel_import(
        invalid_boolean_request.clone(),
        preview.import_token.clone(),
    ))
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::ImportRecoveryRequired
    );
    assert_eq!(fs::read(&destination).unwrap(), before);
    let (error_json, receipts) = tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0;
        let request_id = invalid_boolean_request.0;
        move |connection| {
            let error_json = connection.query_row(
                "SELECT error_json FROM import_operations WHERE import_token=?1",
                [&token],
                |row| row.get::<_, String>(0),
            )?;
            let receipts = connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_cancel_import'",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((error_json, receipts))
        }
    }))
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(error["promotionConfirmed"], "false");
    assert_eq!(error["cleanup"], "PENDING");
    assert_eq!(receipts, 0);
}

#[test]
fn reuse_existing_keeps_metadata() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let mut first = metadata(None);
    first.title = "Paper conservado".into();
    let existing = import_file(&fixture, &bytes, "original-reuse.pdf", first);
    let source = fixture._dir.path().join("copy-reuse.pdf");
    fs::write(&source, &bytes).unwrap();
    let preview = fixture.select(source);
    let mut incoming = metadata(None);
    incoming.title = "No fusionar".into();
    let resolution = research_workbench_core::transport::dto::DuplicateResolution {
        action: research_workbench_core::transport::dto::DuplicateResolutionAction::ReuseExisting,
        paper_id: existing.id.clone(),
    };
    let request_id = UUID::new();
    let reused = tauri::async_runtime::block_on(fixture.service.confirm_import(
        request_id.clone(),
        preview.import_token,
        incoming,
        Some(resolution),
    ))
    .unwrap();
    assert_eq!(reused.id, existing.id);
    assert_eq!(reused.title, "Paper conservado");
    assert_eq!(reused.document_id, existing.document_id);
    let reuse_events = tauri::async_runtime::block_on(fixture.actor.submit({
        let request_id = request_id.0;
        let paper_id = existing.id.0;
        move |connection| {
            connection
                .query_row(
                    "SELECT count(*) FROM audit_events WHERE request_id=?1 AND entity_id=?2 AND action='paper.import_reused'",
                    rusqlite::params![request_id, paper_id],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(Into::into)
        }
    }))
    .unwrap();
    assert_eq!(reuse_events, 1);
}

#[test]
fn import_token_retry_payload_conflict() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("retry.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source);
    let request = UUID::new();
    let input = metadata(None);
    let first = tauri::async_runtime::block_on(fixture.service.confirm_import(
        request.clone(),
        preview.import_token.clone(),
        input.clone(),
        None,
    ))
    .unwrap();
    let replay = tauri::async_runtime::block_on(fixture.service.confirm_import(
        request.clone(),
        preview.import_token.clone(),
        input,
        None,
    ))
    .unwrap();
    assert_eq!(first.id, replay.id);
    let mut conflict = metadata(None);
    conflict.title = "Other payload".into();
    let error = tauri::async_runtime::block_on(fixture.service.confirm_import(
        request,
        preview.import_token,
        conflict,
        None,
    ))
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::Conflict
    );
}

#[test]
fn replayed_selection_does_not_authorize_token_in_new_service() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("selection-replay.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let request_id = UUID::new();
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            path: source.clone(),
            filename: "selection-replay.pdf".into(),
        }));
    let original = tauri::async_runtime::block_on(fixture.service.select_pdf(request_id.clone()))
        .unwrap()
        .unwrap();

    let new_service = LibraryService::new(
        fixture.picker.clone(),
        fixture.documents.clone(),
        fixture.persistence.clone(),
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let replay = tauri::async_runtime::block_on(new_service.select_pdf(request_id))
        .unwrap()
        .unwrap();
    assert_eq!(replay, original);

    let confirm_error = tauri::async_runtime::block_on(new_service.confirm_import(
        UUID::new(),
        replay.import_token.clone(),
        metadata(None),
        None,
    ))
    .unwrap_err();
    assert_eq!(
        confirm_error.code,
        research_workbench_core::transport::error::ErrorCode::OperationCancelled
    );

    let cancel_error =
        tauri::async_runtime::block_on(new_service.cancel_import(UUID::new(), replay.import_token))
            .unwrap_err();
    assert_eq!(
        cancel_error.code,
        research_workbench_core::transport::error::ErrorCode::OperationCancelled
    );
}

#[test]
fn expired_token_rejected() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("expired.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source);
    let token = preview.import_token.0.clone();
    tauri::async_runtime::block_on(fixture.actor.submit(move|c|{c.execute("UPDATE import_operations SET expires_at='2000-01-01T00:00:00.000Z' WHERE import_token=?1",[token])?;Ok(())})).unwrap();
    let error = tauri::async_runtime::block_on(fixture.service.confirm_import(
        UUID::new(),
        preview.import_token,
        metadata(None),
        None,
    ))
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::OperationCancelled
    );
}

#[test]
fn cancel_removes_only_owned_staging() {
    let fixture = Fixture::new();
    let first = fixture._dir.path().join("one.pdf");
    let second = fixture._dir.path().join("two.pdf");
    fs::write(&first, valid_pdf()).unwrap();
    fs::write(&second, valid_pdf()).unwrap();
    let a = fixture.select(first);
    let b = fixture.select(second);
    let staged_a = tauri::async_runtime::block_on(fixture.actor.submit({
        let token = a.import_token.0.clone();
        move |c| {
            c.query_row(
                "SELECT staging_path FROM import_operations WHERE import_token=?1",
                [token],
                |r| r.get::<_, String>(0),
            )
            .map_err(Into::into)
        }
    }))
    .unwrap();
    let staged_a = fixture.root.path().join(staged_a);
    let staged_b = tauri::async_runtime::block_on(fixture.actor.submit({
        let token = b.import_token.0.clone();
        move |c| {
            c.query_row(
                "SELECT staging_path FROM import_operations WHERE import_token=?1",
                [token],
                |r| r.get::<_, String>(0),
            )
            .map_err(Into::into)
        }
    }))
    .unwrap();
    let staged_b = fixture.root.path().join(staged_b);
    tauri::async_runtime::block_on(fixture.service.cancel_import(UUID::new(), a.import_token))
        .unwrap();
    assert!(!staged_a.exists());
    assert!(staged_b.is_file());
}

#[test]
fn unicode_path_import() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("fuente_日本語 con espacios.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source.clone());
    let paper = tauri::async_runtime::block_on(fixture.service.confirm_import(
        UUID::new(),
        preview.import_token,
        metadata(None),
        None,
    ))
    .unwrap();
    assert!(
        fixture
            .root
            .path()
            .join(format!("documents/{}/original.pdf", paper.document_id.0))
            .is_file()
    );
    assert!(source.exists());
}

#[test]
fn invalid_pdf_no_success() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("invalid.pdf");
    fs::write(&source, b"no es un PDF").unwrap();
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            filename: "invalid.pdf".into(),
            path: source,
        }));
    let error =
        tauri::async_runtime::block_on(fixture.service.select_pdf(UUID::new())).unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::InvalidPdf
    );
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|c| Ok(c.query_row(
            "SELECT count(*) FROM papers",
            [],
            |r| r.get::<_, i64>(0)
        )?)))
        .unwrap(),
        0
    );
    let (state, error_json, staging_path) = tauri::async_runtime::block_on(fixture.actor.submit(
        |connection| {
            connection
                .query_row(
                    "SELECT state,error_json,staging_path FROM import_operations ORDER BY created_at DESC LIMIT 1",
                    [],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?)),
                )
                .map_err(Into::into)
        },
    ))
    .unwrap();
    assert_eq!(state, "FAILED");
    let outcome: serde_json::Value = serde_json::from_str(error_json.as_deref().unwrap()).unwrap();
    assert_eq!(outcome["version"], 1);
    assert_eq!(outcome["kind"], "stageFailure");
    assert_eq!(outcome["code"], "InvalidPdf");
    assert_eq!(outcome["cleanup"], "DONE");
    assert!(!fixture.root.path().join(staging_path).exists());
}

#[test]
fn page_tree_and_first_media_box_must_be_valid() {
    let fixture = Fixture::new();
    let valid = pdf_with_page_dictionaries(
        "<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << >> >>",
    );
    let valid_size = valid.len() as i64;
    let valid_path = fixture._dir.path().join("media-box-control.pdf");
    fs::write(&valid_path, valid).unwrap();
    let preview = fixture.select(valid_path);
    assert_eq!(preview.size_bytes, valid_size);

    let cases = [
        (
            "empty-pages",
            "<< /Type /Pages /Kids [] /Count 0 >>",
            "<< /Type /Page /Parent 2 0 R /Resources << >> >>",
        ),
        (
            "missing-media-box",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /Resources << >> >>",
        ),
        (
            "degenerate-media-box",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 0 0] /Resources << >> >>",
        ),
        (
            "cyclic-media-box-parent",
            "<< /Type /Pages /Parent 2 0 R /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /Resources << >> >>",
        ),
    ];
    for (name, pages, page) in cases {
        let path = fixture._dir.path().join(format!("{name}.pdf"));
        fs::write(&path, pdf_with_page_dictionaries(pages, page)).unwrap();
        fixture
            .picker
            .selections
            .lock()
            .unwrap()
            .push_back(Some(SelectedPdf {
                filename: format!("{name}.pdf"),
                path,
            }));
        let error =
            tauri::async_runtime::block_on(fixture.service.select_pdf(UUID::new())).unwrap_err();
        assert_eq!(
            error.code,
            research_workbench_core::transport::dto::IpcErrorCode::InvalidPdf,
            "fixture {name} must be rejected after parsing its valid xref"
        );
    }

    let inherited = pdf_with_page_dictionaries(
        "<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] >>",
        "<< /Type /Page /Parent 2 0 R /Resources << >> >>",
    );
    let path = fixture._dir.path().join("inherited-media-box.pdf");
    fs::write(&path, inherited).unwrap();
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            filename: "inherited-media-box.pdf".into(),
            path,
        }));
    assert!(
        tauri::async_runtime::block_on(fixture.service.select_pdf(UUID::new()))
            .unwrap()
            .is_some()
    );
}

#[test]
fn source_unreadable() {
    let fixture = Fixture::new();
    let missing = fixture._dir.path().join("missing.pdf");
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            filename: "missing.pdf".into(),
            path: missing,
        }));
    let error =
        tauri::async_runtime::block_on(fixture.service.select_pdf(UUID::new())).unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::SourceUnreadable
    );
    assert_latest_stage_failure(&fixture, "SourceUnreadable", "DONE");
    let (state, error_json) = tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
        connection
            .query_row(
                "SELECT state,error_json FROM import_operations",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .map_err(Into::into)
    }))
    .unwrap();
    let failure: serde_json::Value = serde_json::from_str(error_json.as_deref().unwrap()).unwrap();
    assert_eq!(state, "FAILED");
    assert_eq!(failure["kind"], "stageFailure");
    assert_eq!(failure["code"], "SourceUnreadable");
    assert_eq!(failure["cleanup"], "DONE");
}

#[test]
fn source_open_failure_with_unknown_stage_file_stays_pending_after_reopen() {
    let fixture = Fixture::new();
    let missing_source = fixture._dir.path().join("source-missing-after-pick.pdf");
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            filename: "source-missing-after-pick.pdf".into(),
            path: missing_source,
        }));
    let entered = Arc::new(std::sync::Barrier::new(2));
    let release = Arc::new(std::sync::Barrier::new(2));
    let service = LibraryService::new(
        fixture.picker.clone(),
        Arc::new(GatedDocumentStore {
            inner: fixture.documents.clone(),
            entered: entered.clone(),
            release: release.clone(),
        }),
        fixture.persistence.clone(),
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let selecting =
        tauri::async_runtime::spawn(async move { service.select_pdf(UUID::new()).await });
    entered.wait();
    let (operation_id, staging_path) =
        tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
            connection
                .query_row("SELECT id,staging_path FROM import_operations", [], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(Into::into)
        }))
        .unwrap();
    let unknown_bytes = b"unowned bytes in reserved staging namespace";
    let staged_unknown = fixture.root.path().join(&staging_path);
    fs::create_dir_all(staged_unknown.parent().unwrap()).unwrap();
    fs::write(&staged_unknown, unknown_bytes).unwrap();
    release.wait();
    let error = tauri::async_runtime::block_on(selecting)
        .unwrap()
        .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::error::ErrorCode::SourceUnreadable
    );
    assert_eq!(fs::read(&staged_unknown).unwrap(), unknown_bytes);
    let (state, error_json) = tauri::async_runtime::block_on(fixture.actor.submit({
        let operation_id = operation_id.clone();
        move |connection| {
            connection
                .query_row(
                    "SELECT state,error_json FROM import_operations WHERE id=?1",
                    [&operation_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .map_err(Into::into)
        }
    }))
    .unwrap();
    let failure: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(state, "FAILED");
    assert_eq!(failure["kind"], "stageFailure");
    assert_eq!(failure["code"], "SourceUnreadable");
    assert_eq!(failure["cleanup"], "PENDING");
    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
    let actor = DbActor::start(fixture.root.clone()).unwrap();
    let documents = Arc::new(
        LocalDocumentStore::new(fixture.root.clone(), actor.info().library_id.0.clone()).unwrap(),
    );
    let persistence = Arc::new(SqliteLibraryPersistence::new(
        actor.clone(),
        actor.info().library_id.0.clone(),
    ));
    let restarted = LibraryService::new(
        fixture.picker.clone(),
        documents,
        persistence,
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let report = tauri::async_runtime::block_on(restarted.reconcile_imports()).unwrap();
    assert_eq!(report.recovered, 0);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].operation_id, operation_id);
    assert_eq!(fs::read(&staged_unknown).unwrap(), unknown_bytes);
    let failure_after_reopen = tauri::async_runtime::block_on(actor.submit({
        let operation_id = operation_id.clone();
        move |connection| {
            connection
                .query_row(
                    "SELECT error_json FROM import_operations WHERE id=?1",
                    [&operation_id],
                    |row| row.get::<_, String>(0),
                )
                .map_err(Into::into)
        }
    }))
    .unwrap();
    let failure_after_reopen: serde_json::Value =
        serde_json::from_str(&failure_after_reopen).unwrap();
    assert_eq!(failure_after_reopen["cleanup"], "PENDING");
    actor.shutdown(std::time::Duration::from_secs(2)).unwrap();
}

#[test]
fn size_limit_500_mib() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("too-large.pdf");
    let file = fs::File::create(&source).unwrap();
    file.set_len(524_288_001).unwrap();
    drop(file);
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            filename: "too-large.pdf".into(),
            path: source,
        }));
    let error =
        tauri::async_runtime::block_on(fixture.service.select_pdf(UUID::new())).unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::InvalidInput
    );
    assert_latest_stage_failure(&fixture, "InvalidInput", "DONE");
    let (state, error_json, staging_path) =
        tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
            connection
                .query_row(
                    "SELECT state,error_json,staging_path FROM import_operations",
                    [],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, Option<String>>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    },
                )
                .map_err(Into::into)
        }))
        .unwrap();
    assert_eq!(state, "FAILED");
    let failure: serde_json::Value = serde_json::from_str(error_json.as_deref().unwrap()).unwrap();
    assert_eq!(failure["kind"], "stageFailure");
    assert_eq!(failure["code"], "InvalidInput");
    assert_eq!(failure["cleanup"], "DONE");
    assert!(!fixture.root.path().join(staging_path).exists());
    let (papers, documents) = tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
        connection
            .query_row(
                "SELECT (SELECT count(*) FROM papers),(SELECT count(*) FROM documents)",
                [],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .map_err(Into::into)
    }))
    .unwrap();
    assert_eq!((papers, documents), (0, 0));
    let report = tauri::async_runtime::block_on(fixture.service.reconcile_imports()).unwrap();
    assert_eq!(report.recovered, 0);
    assert!(report.issues.is_empty());
}

#[test]
fn valid_pdf_at_500_mib_limit_is_staged_and_cancelled_cleanly() {
    const MAX_BYTES: u64 = 524_288_000;
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("limit.pdf");
    let mut file = fs::File::create(&source).unwrap();
    let bytes = valid_pdf();
    use std::io::Write;
    file.write_all(&bytes).unwrap();
    // A sparse zero-filled tail follows %%EOF; parser should consume the PDF structure only.
    file.set_len(MAX_BYTES).unwrap();
    drop(file);
    let preview = fixture.select(source);
    assert_eq!(preview.size_bytes as u64, MAX_BYTES);
    tauri::async_runtime::block_on(
        fixture
            .service
            .cancel_import(UUID::new(), preview.import_token),
    )
    .unwrap();
}

#[test]
fn recovery_after_staging_finishes_confirmed_intent() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("recover-staging.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source);
    let request = UUID::new();
    let input = metadata(None);
    tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        request.clone(),
        preview.import_token.clone(),
        input,
        None,
    ))
    .unwrap();
    let recovered = tauri::async_runtime::block_on(fixture.service.reconcile_imports()).unwrap();
    assert_eq!(recovered.recovered, 1);
    assert!(recovered.issues.is_empty());
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|c| Ok(c.query_row(
            "SELECT count(*) FROM papers",
            [],
            |r| r.get::<_, i64>(0)
        )?)))
        .unwrap(),
        1
    );
}

#[test]
fn recovery_of_reuse_confirmation_removes_staging_before_committing() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let existing = import_file(
        &fixture,
        &bytes,
        "reuse-recovery-original.pdf",
        metadata(None),
    );
    let source = fixture._dir.path().join("reuse-recovery-copy.pdf");
    fs::write(&source, &bytes).unwrap();
    let preview = fixture.select(source);
    let request = UUID::new();
    let input = metadata(None);
    let resolution = research_workbench_core::transport::dto::DuplicateResolution {
        action: research_workbench_core::transport::dto::DuplicateResolutionAction::ReuseExisting,
        paper_id: existing.id.clone(),
    };
    tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        request,
        preview.import_token.clone(),
        input.clone(),
        Some(resolution),
    ))
    .unwrap();
    let staging_path = tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0.clone();
        move |connection| {
            Ok(connection.query_row(
                "SELECT staging_path FROM import_operations WHERE import_token=?1",
                [token],
                |row| row.get::<_, String>(0),
            )?)
        }
    }))
    .unwrap();

    let report = tauri::async_runtime::block_on(fixture.service.reconcile_imports()).unwrap();
    assert_eq!(report.recovered, 1, "recovery issues: {:?}", report.issues);
    assert!(report.issues.is_empty());
    assert!(!fixture.root.path().join(staging_path).exists());
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
            Ok(
                connection.query_row("SELECT count(*) FROM papers", [], |row| {
                    row.get::<_, i64>(0)
                })?,
            )
        }))
        .unwrap(),
        1
    );
    assert!(
        fixture
            .root
            .path()
            .join(format!("documents/{}/original.pdf", existing.document_id.0))
            .is_file()
    );
}

#[test]
fn prepared_new_import_with_reserved_destination_can_be_cancelled() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("cancel-prepared.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source.clone());
    let confirmation_request = UUID::new();
    let input = metadata(None);
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        confirmation_request,
        preview.import_token.clone(),
        input,
        None,
    ))
    .unwrap();
    let staging_path = prepared.record.staging_path.clone();
    let destination_path = prepared.record.destination_path.clone().unwrap();
    assert!(!fixture.root.path().join(&destination_path).exists());

    tauri::async_runtime::block_on(
        fixture
            .service
            .cancel_import(UUID::new(), preview.import_token),
    )
    .unwrap();

    let (state, error_json) = tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
        connection
            .query_row(
                "SELECT state,error_json FROM import_operations",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .map_err(Into::into)
    }))
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(state, "FAILED");
    assert_eq!(error["kind"], "cancellation");
    assert_eq!(error["cleanup"], "DONE");
    assert!(!fixture.root.path().join(staging_path).exists());
    assert!(source.exists());
    let (papers, documents) = tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
        connection
            .query_row(
                "SELECT (SELECT count(*) FROM papers),(SELECT count(*) FROM documents)",
                [],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .map_err(Into::into)
    }))
    .unwrap();
    assert_eq!((papers, documents), (0, 0));
}

#[test]
fn prepared_import_cancel_pending_is_recovered_after_restart() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("recover-cancel-prepared.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source.clone());
    let input = metadata(None);
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        UUID::new(),
        preview.import_token.clone(),
        input,
        None,
    ))
    .unwrap();
    let staging_path = prepared.record.staging_path.clone();
    let cancellation_request = UUID::new();
    let plan = tauri::async_runtime::block_on(
        fixture
            .persistence
            .prepare_cancel(cancellation_request.clone(), preview.import_token.clone()),
    )
    .unwrap();
    assert!(plan.record.destination_path.is_some());
    let destination_path = fixture
        .root
        .path()
        .join(plan.record.destination_path.as_ref().unwrap());
    assert!(!destination_path.exists());
    tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0.clone();
        move |connection| {
            connection.execute(
                "UPDATE import_operations SET error_json=json_remove(error_json,'$.promotionConfirmed') WHERE import_token=?1",
                [&token],
            )?;
            Ok(())
        }
    }))
    .unwrap();

    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
    let actor = DbActor::start(fixture.root.clone()).unwrap();
    let documents = Arc::new(
        LocalDocumentStore::new(fixture.root.clone(), actor.info().library_id.0.clone()).unwrap(),
    );
    let persistence = Arc::new(SqliteLibraryPersistence::new(
        actor.clone(),
        actor.info().library_id.0.clone(),
    ));
    let restarted = LibraryService::new(
        fixture.picker.clone(),
        documents,
        persistence,
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let report = tauri::async_runtime::block_on(restarted.reconcile_imports()).unwrap();

    assert_eq!(report.recovered, 1);
    assert!(report.issues.is_empty());
    assert!(!fixture.root.path().join(staging_path).exists());
    assert!(!destination_path.exists());
    assert!(source.exists());
    let (state, error_json) = tauri::async_runtime::block_on(actor.submit(|connection| {
        connection
            .query_row(
                "SELECT state,error_json FROM import_operations",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .map_err(Into::into)
    }))
    .unwrap();
    let error: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(state, "FAILED");
    assert_eq!(error["cleanup"], "DONE");
    assert!(error.get("promotionConfirmed").is_none());
    tauri::async_runtime::block_on(
        restarted.cancel_import(cancellation_request.clone(), preview.import_token.clone()),
    )
    .unwrap();
    let (receipts, events) = tauri::async_runtime::block_on(actor.submit({
        let request_id = cancellation_request.0;
        let token = preview.import_token.0;
        move |connection| {
            let receipts = connection.query_row(
                "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_cancel_import'",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )?;
            let events = connection.query_row(
                "SELECT count(*) FROM audit_events WHERE request_id=?1 AND entity_id=(SELECT id FROM import_operations WHERE import_token=?2) AND action='import.cancelled'",
                rusqlite::params![request_id, token],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((receipts, events))
        }
    }))
    .unwrap();
    assert_eq!((receipts, events), (1, 1));
    actor.shutdown(std::time::Duration::from_secs(2)).unwrap();
}

#[test]
fn foreign_library_intent_cannot_read_or_remove_staged_file() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("foreign-library.pdf");
    let original = valid_pdf();
    fs::write(&source, &original).unwrap();
    fixture.select(source);
    let (operation_id, staging_path) =
        tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
            connection
                .query_row("SELECT id,staging_path FROM import_operations", [], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(Into::into)
        }))
        .unwrap();
    tauri::async_runtime::block_on(fixture.actor.submit(move |connection| {
        connection.execute(
            "UPDATE import_operations SET library_id=?1 WHERE id=?2",
            rusqlite::params![UUID::new().0, operation_id],
        )?;
        Ok(())
    }))
    .unwrap();

    let report = tauri::async_runtime::block_on(fixture.service.reconcile_imports()).unwrap();
    assert_eq!(report.recovered, 0);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].code, "recoveryRequired");
    assert_eq!(
        fs::read(fixture.root.path().join(staging_path)).unwrap(),
        original
    );
}

#[test]
fn cleanup_refuses_intent_referencing_an_existing_document_path() {
    let fixture = Fixture::new();
    let bytes = valid_pdf();
    let existing = import_file(&fixture, &bytes, "ownership-original.pdf", metadata(None));
    let source = fixture._dir.path().join("ownership-copy.pdf");
    fs::write(&source, &bytes).unwrap();
    let preview = fixture.select(source);
    let (operation_id, staging_path) = tauri::async_runtime::block_on(fixture.actor.submit({
        let token = preview.import_token.0.clone();
        move |connection| {
            connection
                .query_row(
                    "SELECT id,staging_path FROM import_operations WHERE import_token=?1",
                    [token],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .map_err(Into::into)
        }
    }))
    .unwrap();
    let existing_document_id = existing.document_id.0.clone();
    let existing_path = format!("documents/{existing_document_id}/original.pdf");
    tauri::async_runtime::block_on(fixture.actor.submit(move |connection| {
        connection.execute(
            "UPDATE import_operations SET reserved_document_id=?1,destination_path=?2 WHERE id=?3",
            rusqlite::params![existing_document_id, existing_path, operation_id],
        )?;
        Ok(())
    }))
    .unwrap();

    let error = tauri::async_runtime::block_on(
        fixture
            .service
            .cancel_import(UUID::new(), preview.import_token),
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::error::ErrorCode::ImportRecoveryRequired
    );
    assert!(fixture.root.path().join(&staging_path).is_file());
    assert_eq!(
        fs::read(
            fixture
                .root
                .path()
                .join(format!("documents/{}/original.pdf", existing.document_id.0))
        )
        .unwrap(),
        bytes,
    );
}

#[test]
fn failed_staging_keeps_intent_pending_when_cleanup_is_unproven() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("partial-stage.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            filename: "partial-stage.pdf".into(),
            path: source,
        }));
    let service = LibraryService::new(
        fixture.picker.clone(),
        Arc::new(PartialFailureStore {
            inner: fixture.documents.clone(),
            root: fixture.root.path().to_path_buf(),
            leave_partial: true,
        }),
        fixture.persistence.clone(),
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );

    let error = tauri::async_runtime::block_on(service.select_pdf(UUID::new())).unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::error::ErrorCode::StorageUnavailable
    );
    let (state, error_json, staging_path) = tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
        connection.query_row(
            "SELECT state,error_json,staging_path FROM import_operations ORDER BY created_at DESC LIMIT 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?)),
        ).map_err(Into::into)
    })).unwrap();
    assert_eq!(state, "FAILED");
    let failure: serde_json::Value = serde_json::from_str(error_json.as_deref().unwrap()).unwrap();
    assert_eq!(failure["kind"], "stageFailure");
    assert_eq!(failure["code"], "StorageUnavailable");
    assert_eq!(failure["cleanup"], "PENDING");
    let staging_file = fixture.root.path().join(staging_path);
    assert!(staging_file.is_file());
    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
    let actor = DbActor::start(fixture.root.clone()).unwrap();
    let persistence = Arc::new(SqliteLibraryPersistence::new(
        actor.clone(),
        actor.info().library_id.0.clone(),
    ));
    let restarted = LibraryService::new(
        fixture.picker.clone(),
        fixture.documents.clone(),
        persistence,
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );

    let report = tauri::async_runtime::block_on(restarted.reconcile_imports()).unwrap();
    assert_eq!(report.recovered, 0);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].code, "recoveryRequired");
    assert_eq!(fs::read(staging_file).unwrap(), b"partial bytes");
    actor.shutdown(std::time::Duration::from_secs(2)).unwrap();
}

#[test]
fn stage_failure_without_residual_is_completed_after_restart() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("no-residual.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            filename: "no-residual.pdf".into(),
            path: source.clone(),
        }));
    let service = LibraryService::new(
        fixture.picker.clone(),
        Arc::new(PartialFailureStore {
            inner: fixture.documents.clone(),
            root: fixture.root.path().to_path_buf(),
            leave_partial: false,
        }),
        fixture.persistence.clone(),
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let error = tauri::async_runtime::block_on(service.select_pdf(UUID::new())).unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::error::ErrorCode::StorageUnavailable
    );
    assert_latest_stage_failure(&fixture, "StorageUnavailable", "PENDING");
    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();

    let actor = DbActor::start(fixture.root.clone()).unwrap();
    let persistence = Arc::new(SqliteLibraryPersistence::new(
        actor.clone(),
        actor.info().library_id.0.clone(),
    ));
    let restarted = LibraryService::new(
        fixture.picker.clone(),
        fixture.documents.clone(),
        persistence,
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let report = tauri::async_runtime::block_on(restarted.reconcile_imports()).unwrap();
    assert_eq!(report.recovered, 1);
    assert!(report.issues.is_empty());
    assert!(source.is_file());
    let (state, error_json, counts) = tauri::async_runtime::block_on(actor.submit(|connection| {
        connection
            .query_row(
                "SELECT state,error_json,(SELECT count(*) FROM papers)+(SELECT count(*) FROM documents) FROM import_operations",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?)),
            )
            .map_err(Into::into)
    }))
    .unwrap();
    let failure: serde_json::Value = serde_json::from_str(&error_json).unwrap();
    assert_eq!(state, "FAILED");
    assert_eq!(failure["cleanup"], "DONE");
    assert_eq!(counts, 0);
    actor.shutdown(std::time::Duration::from_secs(2)).unwrap();
}

#[test]
fn admitted_filesystem_operation_keeps_maintenance_permit_after_caller_drop() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("owned-saga.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            filename: "owned-saga.pdf".into(),
            path: source,
        }));
    let entered = Arc::new(std::sync::Barrier::new(2));
    let release = Arc::new(std::sync::Barrier::new(2));
    let maintenance = MaintenanceCoordinator::default();
    let service = LibraryService::new(
        fixture.picker.clone(),
        Arc::new(GatedDocumentStore {
            inner: fixture.documents.clone(),
            entered: entered.clone(),
            release: release.clone(),
        }),
        fixture.persistence.clone(),
        maintenance.clone(),
        RequestRegistry::default(),
    );
    let second_mutator = service.clone();
    let caller = tauri::async_runtime::spawn(async move { service.select_pdf(UUID::new()).await });
    entered.wait();
    caller.abort();
    maintenance.close();

    assert_eq!(maintenance.active_operations(), 1);
    assert_eq!(
        maintenance
            .wait_for_idle(std::time::Duration::from_millis(10))
            .unwrap_err()
            .code,
        research_workbench_core::transport::error::ErrorCode::Busy
    );
    assert_eq!(
        tauri::async_runtime::block_on(second_mutator.select_pdf(UUID::new()))
            .unwrap_err()
            .code,
        research_workbench_core::transport::error::ErrorCode::Busy
    );
    release.wait();
    maintenance
        .wait_for_idle(std::time::Duration::from_secs(3))
        .unwrap();
    let (staged_hash, staging_path) =
        tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
            connection
                .query_row(
                    "SELECT sha256,staging_path FROM import_operations",
                    [],
                    |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, String>(1)?)),
                )
                .map_err(Into::into)
        }))
        .unwrap();
    assert!(staged_hash.is_some());
    assert!(fixture.root.path().join(staging_path).is_file());
    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
}

#[test]
fn startup_recovery_keeps_maintenance_permit_until_report_after_close() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("startup-recovery.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    fixture.select(source);
    let entered = Arc::new(std::sync::Barrier::new(2));
    let release = Arc::new(std::sync::Barrier::new(2));
    let maintenance = MaintenanceCoordinator::default();
    let recovery_permit = maintenance.begin_maintenance().unwrap();
    let service = LibraryService::new(
        fixture.picker.clone(),
        Arc::new(GatedRecoveryStore {
            inner: fixture.documents.clone(),
            entered: entered.clone(),
            release: release.clone(),
        }),
        fixture.persistence.clone(),
        maintenance.clone(),
        RequestRegistry::default(),
    );
    let recovery = tauri::async_runtime::spawn(async move {
        service.reconcile_imports_with_permit(recovery_permit).await
    });
    entered.wait();
    maintenance.close();
    assert_eq!(
        maintenance
            .wait_for_idle(std::time::Duration::from_millis(10))
            .unwrap_err()
            .code,
        research_workbench_core::transport::error::ErrorCode::Busy
    );
    release.wait();
    let report = tauri::async_runtime::block_on(recovery).unwrap().unwrap();
    assert_eq!(report.issues.len(), 1);
    maintenance
        .wait_for_idle(std::time::Duration::from_secs(3))
        .unwrap();
    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
}

#[test]
fn compatible_overlapping_select_replays_same_result_after_first_finishes() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("duplicate-request.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            filename: "duplicate-request.pdf".into(),
            path: source,
        }));
    let entered = Arc::new(std::sync::Barrier::new(2));
    let release = Arc::new(std::sync::Barrier::new(2));
    let service = LibraryService::new(
        fixture.picker.clone(),
        Arc::new(GatedDocumentStore {
            inner: fixture.documents.clone(),
            entered: entered.clone(),
            release: release.clone(),
        }),
        fixture.persistence.clone(),
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let request = UUID::new();
    let first_service = service.clone();
    let first_request = request.clone();
    let first =
        tauri::async_runtime::spawn(async move { first_service.select_pdf(first_request).await });
    entered.wait();
    let second_service = service.clone();
    let second =
        tauri::async_runtime::spawn(async move { second_service.select_pdf(request).await });
    release.wait();
    let first_result = tauri::async_runtime::block_on(first)
        .unwrap()
        .unwrap()
        .unwrap();
    let second_result = tauri::async_runtime::block_on(second)
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(first_result.import_token, second_result.import_token);
    assert_eq!(first_result, second_result);
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|connection| {
            Ok(
                connection.query_row("SELECT count(*) FROM import_operations", [], |row| {
                    row.get::<_, i64>(0)
                })?,
            )
        }))
        .unwrap(),
        1
    );
}

#[test]
fn incompatible_overlapping_cancel_conflicts_without_touching_second_staging() {
    let fixture = Fixture::new();
    let first_path = fixture._dir.path().join("cancel-first.pdf");
    let second_path = fixture._dir.path().join("cancel-second.pdf");
    fs::write(&first_path, valid_pdf()).unwrap();
    fs::write(&second_path, valid_pdf()).unwrap();
    fixture.picker.selections.lock().unwrap().extend([
        Some(SelectedPdf {
            filename: "cancel-first.pdf".into(),
            path: first_path,
        }),
        Some(SelectedPdf {
            filename: "cancel-second.pdf".into(),
            path: second_path,
        }),
    ]);
    let entered = Arc::new(std::sync::Barrier::new(2));
    let release = Arc::new(std::sync::Barrier::new(2));
    let service = LibraryService::new(
        fixture.picker.clone(),
        Arc::new(GatedCleanupStore {
            inner: fixture.documents.clone(),
            entered: entered.clone(),
            release: release.clone(),
        }),
        fixture.persistence.clone(),
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let first = tauri::async_runtime::block_on(service.select_pdf(UUID::new()))
        .unwrap()
        .unwrap();
    let second = tauri::async_runtime::block_on(service.select_pdf(UUID::new()))
        .unwrap()
        .unwrap();
    let (first_staging, second_staging) = tauri::async_runtime::block_on(fixture.actor.submit({
        let first_token = first.import_token.0.clone();
        let second_token = second.import_token.0.clone();
        move |connection| {
            connection.query_row(
                "SELECT (SELECT staging_path FROM import_operations WHERE import_token=?1),(SELECT staging_path FROM import_operations WHERE import_token=?2)",
                rusqlite::params![first_token, second_token],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            ).map_err(Into::into)
        }
    })).unwrap();
    let request = UUID::new();
    let first_service = service.clone();
    let first_request = request.clone();
    let first_token = first.import_token.clone();
    let first_cancel = tauri::async_runtime::spawn(async move {
        first_service
            .cancel_import(first_request, first_token)
            .await
    });
    entered.wait();
    let second_service = service.clone();
    let second_request = request.clone();
    let second_token = second.import_token.clone();
    let second_cancel = tauri::async_runtime::spawn(async move {
        second_service
            .cancel_import(second_request, second_token)
            .await
    });
    release.wait();
    tauri::async_runtime::block_on(first_cancel)
        .unwrap()
        .unwrap();
    let conflict = tauri::async_runtime::block_on(second_cancel)
        .unwrap()
        .unwrap_err();
    assert_eq!(
        conflict.code,
        research_workbench_core::transport::error::ErrorCode::Conflict
    );
    assert!(!fixture.root.path().join(first_staging).exists());
    assert!(fixture.root.path().join(second_staging).is_file());
}

async fn signal_after_first_poll<F>(
    future: F,
    started: tokio::sync::oneshot::Sender<()>,
) -> F::Output
where
    F: std::future::Future,
{
    let mut future = Box::pin(future);
    let mut started = Some(started);
    std::future::poll_fn(move |context| {
        let result = future.as_mut().poll(context);
        if let Some(started) = started.take() {
            let _ = started.send(());
        }
        result
    })
    .await
}

#[test]
fn same_request_metadata_archive_restore_cannot_race_cancel_cleanup() {
    let fixture = Fixture::new();
    let paper = import_file(&fixture, &valid_pdf(), "request-owner.pdf", metadata(None));
    let source = fixture._dir.path().join("request-owner-staged.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let entered = Arc::new(std::sync::Barrier::new(2));
    let release = Arc::new(std::sync::Barrier::new(2));
    let maintenance = MaintenanceCoordinator::default();
    let service = LibraryService::new(
        fixture.picker.clone(),
        Arc::new(GatedCleanupStore {
            inner: fixture.documents.clone(),
            entered: entered.clone(),
            release: release.clone(),
        }),
        fixture.persistence.clone(),
        maintenance.clone(),
        RequestRegistry::default(),
    );
    let preview = {
        fixture
            .picker
            .selections
            .lock()
            .unwrap()
            .push_back(Some(SelectedPdf {
                filename: source.file_name().unwrap().to_string_lossy().into_owned(),
                path: source,
            }));
        tauri::async_runtime::block_on(service.select_pdf(UUID::new()))
            .unwrap()
            .unwrap()
    };
    let request_id = UUID::new();
    let cancel_service = service.clone();
    let cancel_request = request_id.clone();
    let token = preview.import_token.clone();
    let cancel = tauri::async_runtime::spawn(async move {
        cancel_service.cancel_import(cancel_request, token).await
    });
    entered.wait();

    let (metadata_started, metadata_started_rx) = tokio::sync::oneshot::channel();
    let (archive_started, archive_started_rx) = tokio::sync::oneshot::channel();
    let (restore_started, restore_started_rx) = tokio::sync::oneshot::channel();
    let expected_revision = paper.revision;
    let metadata_service = service.clone();
    let metadata_request = request_id.clone();
    let metadata_paper = paper.id.clone();
    let metadata_revision = expected_revision;
    let update = tauri::async_runtime::spawn(signal_after_first_poll(
        async move {
            metadata_service
                .update_metadata(
                    metadata_request,
                    metadata_paper,
                    metadata_revision,
                    metadata(Some("10.9999/racing")),
                )
                .await
        },
        metadata_started,
    ));
    let archive_service = service.clone();
    let archive_request = request_id.clone();
    let archive_paper = paper.id.clone();
    let archive_revision = expected_revision;
    let archive = tauri::async_runtime::spawn(signal_after_first_poll(
        async move {
            archive_service
                .archive_paper(archive_request, archive_paper, archive_revision)
                .await
        },
        archive_started,
    ));
    let restore_service = service.clone();
    let restore_request = request_id.clone();
    let restore_paper = paper.id.clone();
    let restore_revision = expected_revision;
    let restore = tauri::async_runtime::spawn(signal_after_first_poll(
        async move {
            restore_service
                .restore_paper(restore_request, restore_paper, restore_revision)
                .await
        },
        restore_started,
    ));
    tauri::async_runtime::block_on(async {
        metadata_started_rx.await.unwrap();
        archive_started_rx.await.unwrap();
        restore_started_rx.await.unwrap();
    });

    let (title, lifecycle, receipt_count) = tauri::async_runtime::block_on(
        fixture.actor.submit({
            let paper_id = paper.id.0.clone();
            let request_id = request_id.0.clone();
            move |connection| {
                connection
                    .query_row(
                        "SELECT p.title,p.lifecycle,(SELECT count(*) FROM operation_receipts WHERE request_id=?2) FROM papers p WHERE p.id=?1",
                        rusqlite::params![paper_id, request_id],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?)),
                    )
                    .map_err(Into::into)
            }
        }),
    )
    .unwrap();
    release.wait();
    tauri::async_runtime::block_on(cancel).unwrap().unwrap();
    let errors = [
        tauri::async_runtime::block_on(update).unwrap().unwrap_err(),
        tauri::async_runtime::block_on(archive)
            .unwrap()
            .unwrap_err(),
        tauri::async_runtime::block_on(restore)
            .unwrap()
            .unwrap_err(),
    ];
    assert!(errors.iter().all(|error| {
        error.code == research_workbench_core::transport::error::ErrorCode::Conflict
    }));
    assert_eq!(title, "Test title");
    assert_eq!(lifecycle, "NEW");
    assert_eq!(receipt_count, 0);
    assert_eq!(maintenance.active_operations(), 0);
}

#[test]
fn recovery_after_promote_before_commit() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("recover-promote.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source);
    let request = UUID::new();
    let input = metadata(None);
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        request.clone(),
        preview.import_token,
        input,
        None,
    ))
    .unwrap();
    tauri::async_runtime::block_on(fixture.documents.promote(prepared.clone())).unwrap();
    tauri::async_runtime::block_on(fixture.persistence.record_promoted(prepared)).unwrap();
    let restarted = LibraryService::new(
        fixture.picker.clone(),
        fixture.documents.clone(),
        fixture.persistence.clone(),
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let recovered = tauri::async_runtime::block_on(restarted.reconcile_imports()).unwrap();
    assert_eq!(recovered.recovered, 1);
    assert!(recovered.issues.is_empty());
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|c| Ok(c.query_row(
            "SELECT count(*) FROM papers",
            [],
            |r| r.get::<_, i64>(0)
        )?)))
        .unwrap(),
        1
    );
}

#[test]
fn commit_rollback_keeps_recoverable_intent_without_premature_paper() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("rollback.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source);
    tauri::async_runtime::block_on(fixture.actor.submit(|c| {
        c.execute_batch("CREATE TRIGGER reject_import BEFORE INSERT ON papers BEGIN SELECT RAISE(ABORT,'synthetic test failure'); END;")?;
        Ok(())
    }))
    .unwrap();
    let error = tauri::async_runtime::block_on(fixture.service.confirm_import(
        UUID::new(),
        preview.import_token,
        metadata(None),
        None,
    ))
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::StorageUnavailable
    );
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|c| Ok(c.query_row(
            "SELECT count(*) FROM papers",
            [],
            |r| r.get::<_, i64>(0)
        )?)))
        .unwrap(),
        0
    );
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|c| Ok(c.query_row(
            "SELECT count(*) FROM import_operations WHERE state='PROMOTED'",
            [],
            |r| r.get::<_, i64>(0)
        )?)))
        .unwrap(),
        1
    );

    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
    let actor = DbActor::start(fixture.root.clone()).unwrap();
    tauri::async_runtime::block_on(actor.submit(|c| {
        c.execute_batch("DROP TRIGGER reject_import;")?;
        Ok(())
    }))
    .unwrap();
    let persistence = Arc::new(SqliteLibraryPersistence::new(
        actor.clone(),
        actor.info().library_id.0.clone(),
    ));
    let documents = Arc::new(
        LocalDocumentStore::new(
            actor.library_root().clone(),
            actor.info().library_id.0.clone(),
        )
        .unwrap(),
    );
    let restarted = LibraryService::new(
        fixture.picker.clone(),
        documents,
        persistence,
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let recovered = tauri::async_runtime::block_on(restarted.reconcile_imports()).unwrap();
    assert_eq!(recovered.recovered, 1);
    assert!(recovered.issues.is_empty());
    assert_eq!(
        tauri::async_runtime::block_on(actor.submit(|c| Ok(c.query_row(
            "SELECT count(*) FROM papers",
            [],
            |r| r.get::<_, i64>(0)
        )?)))
        .unwrap(),
        1
    );
    actor.shutdown(std::time::Duration::from_secs(2)).unwrap();
}

#[test]
fn prepare_helper_validates_and_audits_intent_once() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("prepare-helper.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source);
    let request = UUID::new();
    let mut invalid = metadata(None);
    invalid.title = "   ".into();
    let rejected = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        request.clone(),
        preview.import_token.clone(),
        invalid,
        None,
    ))
    .unwrap_err();
    assert_eq!(
        rejected.code,
        research_workbench_core::transport::error::ErrorCode::InvalidInput
    );

    let input = metadata(None);
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        request.clone(),
        preview.import_token.clone(),
        input.clone(),
        None,
    ))
    .unwrap();
    let replay = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        request,
        preview.import_token,
        input,
        None,
    ))
    .unwrap();
    assert_eq!(prepared.paper_id, replay.paper_id);
    let audit_count = tauri::async_runtime::block_on(fixture.actor.submit({
        let operation_id = prepared.record.operation_id;
        move |connection| {
            connection
                .query_row(
                    "SELECT count(*) FROM audit_events WHERE entity_id=?1 AND action='import.confirmation_prepared'",
                    [operation_id],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(Into::into)
        }
    }))
    .unwrap();
    assert_eq!(audit_count, 1);
}

#[test]
fn ambiguous_target_never_deleted() {
    let fixture = Fixture::new();
    let source = fixture._dir.path().join("ambiguous.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    let preview = fixture.select(source);
    let input = metadata(None);
    let prepared = tauri::async_runtime::block_on(fixture.persistence.prepare_confirmation(
        UUID::new(),
        preview.import_token,
        input,
        None,
    ))
    .unwrap();
    let target = fixture.root.path().join(&prepared.destination_path);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    let foreign = b"foreign data must survive";
    fs::write(&target, foreign).unwrap();
    let report = tauri::async_runtime::block_on(fixture.service.reconcile_imports()).unwrap();
    assert_eq!(report.recovered, 0);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(fs::read(target).unwrap(), foreign);
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit(|c| Ok(c.query_row(
            "SELECT count(*) FROM papers",
            [],
            |r| r.get::<_, i64>(0)
        )?)))
        .unwrap(),
        0
    );
}

#[test]
fn stale_revision_rejected_with_wire_current_revision() {
    let fixture = Fixture::new();
    let paper = import_file(&fixture, &valid_pdf(), "revision.pdf", metadata(None));
    let update_request = UUID::new();
    let changed = tauri::async_runtime::block_on(fixture.service.update_metadata(
        update_request.clone(),
        paper.id.clone(),
        paper.revision,
        metadata(None),
    ))
    .unwrap();
    let replay = tauri::async_runtime::block_on(fixture.service.update_metadata(
        update_request.clone(),
        paper.id.clone(),
        paper.revision,
        metadata(None),
    ))
    .unwrap();
    assert_eq!(replay, changed);
    let mut input = metadata(None);
    input.title = "No sobrescribir".into();
    let error = tauri::async_runtime::block_on(fixture.service.update_metadata(
        UUID::new(),
        paper.id,
        paper.revision,
        input,
    ))
    .unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::Conflict
    );
    let wire = serde_json::to_value(research_workbench_core::transport::error::envelope::<()>(
        UUID::new(),
        Err(error),
    ))
    .unwrap();
    assert_eq!(
        wire["error"]["details"]["currentRevision"],
        changed.revision
    );
    let (entity_events, receipt_events) = tauri::async_runtime::block_on(fixture.actor.submit({
        let paper_id = changed.id.0.clone();
        let request_id = update_request.0;
        move |connection| {
            connection
                .query_row(
                    "SELECT count(*),(SELECT count(*) FROM audit_events WHERE request_id=?2 AND entity_id IS NULL AND action='library_update_metadata') FROM audit_events WHERE request_id=?2 AND entity_id=?1 AND action='paper.metadata.updated'",
                    rusqlite::params![paper_id, request_id],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
                )
                .map_err(Into::into)
        }
    }))
    .unwrap();
    assert_eq!((entity_events, receipt_events), (1, 1));
}

#[test]
fn application_library_helper_uses_callers_transaction() {
    use research_workbench_core::{
        adapters::sqlite::library_repository::SqliteLibraryRepository,
        application::{library::update_metadata_in_tx, unit_of_work::with_transaction},
        transport::error::{AppError, ErrorCode},
    };

    let fixture = Fixture::new();
    let paper = import_file(
        &fixture,
        &valid_pdf(),
        "transaction-helper.pdf",
        metadata(None),
    );
    let mut changed = metadata(None);
    changed.title = "must roll back".into();
    let paper_id = paper.id.0.clone();
    let request_id = UUID::new().0;
    let repository = SqliteLibraryRepository;

    let result: Result<(), AppError> =
        tauri::async_runtime::block_on(fixture.actor.submit(move |connection| {
            with_transaction(connection, |tx| {
                update_metadata_in_tx(
                    tx,
                    &repository,
                    &research_workbench_core::adapters::sqlite::workflow_repository::SqliteWorkflowRepository,
                    &request_id,
                    &paper_id,
                    paper.revision,
                    &changed,
                )?;
                Err(AppError::new(ErrorCode::InvalidInput))
            })
        }));
    assert_eq!(result.unwrap_err().code, ErrorCode::InvalidInput);

    let mut invalid_metadata = metadata(None);
    invalid_metadata.title = " \t ".into();
    let invalid_id = paper.id.0.clone();
    let invalid_repository = SqliteLibraryRepository;
    let invalid_revision = paper.revision;
    let invalid_result = tauri::async_runtime::block_on(fixture.actor.submit(move |connection| {
        with_transaction(connection, |tx| {
            update_metadata_in_tx(
                tx,
                &invalid_repository,
                &research_workbench_core::adapters::sqlite::workflow_repository::SqliteWorkflowRepository,
                "00000000-0000-4000-8000-000000000123",
                &invalid_id,
                invalid_revision,
                &invalid_metadata,
            )
            .map(|_| ())
        })
    }));
    assert_eq!(invalid_result.unwrap_err().code, ErrorCode::InvalidInput);

    let unchanged =
        tauri::async_runtime::block_on(fixture.service.get_paper(UUID::new(), paper.id.clone()))
            .unwrap();
    assert_eq!(unchanged.title, "Test title");
    assert_eq!(unchanged.revision, paper.revision);
    assert_eq!(
        tauri::async_runtime::block_on(fixture.actor.submit({
            let paper_id = paper.id.0.clone();
            move |connection| {
                Ok(connection.query_row(
                    "SELECT count(*) FROM audit_events WHERE entity_id=?1 AND action='paper.metadata.updated'",
                    [paper_id],
                    |row| row.get::<_, i64>(0),
                )?)
            }
        }))
        .unwrap(),
        0
    );
}

#[test]
fn archive_restore_preserves_document_authors_and_reading_position() {
    let fixture = Fixture::new();
    let mut input = metadata(None);
    input.authors = vec!["Áutora uno".into(), "Autor Two".into()];
    let paper = import_file(&fixture, &valid_pdf(), "archive.pdf", input);
    tauri::async_runtime::block_on(fixture.actor.submit({
        let document_id = paper.document_id.0.clone();
        move |c| {
            c.execute(
                "INSERT INTO reading_positions(document_id,page_index,zoom,revision,updated_at) VALUES(?1,3,1.25,0,'2026-10-02T00:00:00Z')",
                [document_id],
            )?;
            Ok(())
        }
    }))
    .unwrap();
    let before =
        tauri::async_runtime::block_on(fixture.service.get_paper(UUID::new(), paper.id.clone()))
            .unwrap();
    let archive_request = UUID::new();
    let archived = tauri::async_runtime::block_on(fixture.service.archive_paper(
        archive_request.clone(),
        paper.id.clone(),
        paper.revision,
    ))
    .unwrap();
    let restore_request = UUID::new();
    let restored = tauri::async_runtime::block_on(fixture.service.restore_paper(
        restore_request.clone(),
        paper.id.clone(),
        archived.revision,
    ))
    .unwrap();
    assert_eq!(restored.id, before.id);
    assert_eq!(restored.document_id, before.document_id);
    assert_eq!(restored.authors, before.authors);
    assert_eq!(restored.lifecycle, before.lifecycle);
    let reading_position_count = tauri::async_runtime::block_on(fixture.actor.submit({
        let document_id = paper.document_id.0.clone();
        move |c| {
            Ok(c.query_row(
                "SELECT count(*) FROM reading_positions WHERE document_id=?1 AND page_index=3 AND zoom=1.25",
                [document_id],
                |row| row.get::<_, i64>(0),
            )?)
        }
    }))
    .unwrap();
    assert_eq!(reading_position_count, 1);
    let entity_events = tauri::async_runtime::block_on(fixture.actor.submit({
        let paper_id = paper.id.0.clone();
        let archive_request = archive_request.0;
        let restore_request = restore_request.0;
        move |connection| {
            connection
                .query_row(
                    "SELECT count(*) FROM audit_events WHERE entity_id=?1 AND ((request_id=?2 AND action='paper.archived') OR (request_id=?3 AND action='paper.restored'))",
                    rusqlite::params![paper_id, archive_request, restore_request],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(Into::into)
        }
    }))
    .unwrap();
    assert_eq!(entity_events, 2);
    assert!(
        fixture
            .root
            .path()
            .join(format!("documents/{}/original.pdf", paper.document_id.0))
            .is_file()
    );
}

#[test]
fn authors_order_survives_reopen() {
    let fixture = Fixture::new();
    let mut input = metadata(None);
    input.authors = vec!["Áutora uno".into(), "Autor Two".into(), "作者 三".into()];
    let saved = import_file(&fixture, &valid_pdf(), "authors.pdf", input);
    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
    let reopened = DbActor::start(fixture.root.clone()).unwrap();
    let persistence =
        SqliteLibraryPersistence::new(reopened.clone(), reopened.info().library_id.0.clone());
    let paper =
        tauri::async_runtime::block_on(persistence.get_paper(UUID::new(), saved.id)).unwrap();
    assert_eq!(paper.authors, ["Áutora uno", "Autor Two", "作者 三"]);
    reopened
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
}

#[test]
fn committed_document_missing_is_reported_to_settings_after_recovery() {
    let fixture = Fixture::new();
    let recovery = RecoveryStatus::default();
    let service = fixture
        .service
        .clone()
        .with_recovery_status(recovery.clone());
    let source = fixture._dir.path().join("committed-missing.pdf");
    fs::write(&source, valid_pdf()).unwrap();
    fixture
        .picker
        .selections
        .lock()
        .unwrap()
        .push_back(Some(SelectedPdf {
            filename: "committed-missing.pdf".into(),
            path: source,
        }));
    let preview = tauri::async_runtime::block_on(service.select_pdf(UUID::new()))
        .unwrap()
        .unwrap();
    let paper = tauri::async_runtime::block_on(fixture.service.confirm_import(
        UUID::new(),
        preview.import_token,
        metadata(None),
        None,
    ))
    .unwrap();
    let managed_pdf = fixture
        .root
        .path()
        .join(format!("documents/{}/original.pdf", paper.document_id.0));
    fs::remove_file(managed_pdf).unwrap();

    let report = tauri::async_runtime::block_on(service.reconcile_imports()).unwrap();
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.code == "integrityFailure")
    );
    recovery.finish(Ok(report));
    let settings =
        research_workbench_core::adapters::sqlite::settings::ActorSettingsQuery::with_recovery(
            fixture.actor.clone(),
            MaintenanceCoordinator::default(),
            recovery,
        );
    let status = tauri::async_runtime::block_on(settings.library_status()).unwrap();
    assert!(status.recovery_required);
}

#[test]
fn global_recovery_failure_is_retained_in_settings_and_blocks_mutations() {
    let fixture = Fixture::new();
    let recovery = RecoveryStatus::pending();
    recovery.finish(Err(
        research_workbench_core::transport::error::AppError::new(
            research_workbench_core::transport::error::ErrorCode::StorageUnavailable,
        ),
    ));
    assert_eq!(
        recovery.failure_code(),
        Some(research_workbench_core::transport::error::ErrorCode::StorageUnavailable)
    );
    let service = fixture
        .service
        .clone()
        .with_recovery_status(recovery.clone());
    let error = tauri::async_runtime::block_on(service.select_pdf(UUID::new())).unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::error::ErrorCode::ImportRecoveryRequired
    );
    let settings =
        research_workbench_core::adapters::sqlite::settings::ActorSettingsQuery::with_recovery(
            fixture.actor.clone(),
            MaintenanceCoordinator::default(),
            recovery,
        );
    let status = tauri::async_runtime::block_on(settings.library_status()).unwrap();
    assert!(status.recovery_required);
}

#[test]
fn recovery_enumeration_failure_remains_visible_when_store_query_fails() {
    let fixture = Fixture::new();
    let recovery = RecoveryStatus::pending();
    let service = fixture
        .service
        .clone()
        .with_recovery_status(recovery.clone());
    tauri::async_runtime::block_on(fixture.actor.submit(|c| {
        c.execute_batch("DROP TABLE import_operations")?;
        Ok(())
    }))
    .unwrap();
    let error = tauri::async_runtime::block_on(service.reconcile_imports()).unwrap_err();
    recovery.finish(Err(error));

    let mutation = tauri::async_runtime::block_on(service.select_pdf(UUID::new())).unwrap_err();
    assert_eq!(
        mutation.code,
        research_workbench_core::transport::error::ErrorCode::ImportRecoveryRequired
    );

    let settings =
        research_workbench_core::adapters::sqlite::settings::ActorSettingsQuery::with_recovery(
            fixture.actor.clone(),
            MaintenanceCoordinator::default(),
            recovery.clone(),
        );
    let status = tauri::async_runtime::block_on(settings.library_status()).unwrap();
    assert!(status.recovery_required);
    assert_eq!(
        recovery.failure_code(),
        Some(research_workbench_core::transport::error::ErrorCode::StorageUnavailable)
    );
}

#[test]
fn readonly_future_schema_composition_preserves_root_and_rejects_mutation() {
    use std::collections::BTreeMap;
    fn inventory(
        path: &std::path::Path,
        base: &std::path::Path,
        out: &mut BTreeMap<String, String>,
    ) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let item = entry.path();
            let rel = item
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            if item.is_dir() {
                out.insert(rel.clone(), "directory".into());
                inventory(&item, base, out);
            } else {
                out.insert(
                    rel,
                    format!("{:x}", sha2::Sha256::digest(fs::read(item).unwrap())),
                );
            }
        }
    }
    use sha2::Digest;
    let fixture = Fixture::new();
    fixture
        .actor
        .shutdown(std::time::Duration::from_secs(2))
        .unwrap();
    for folder in ["documents", "staging", "recovery"] {
        let path = fixture.root.path().join(folder);
        if path.exists() {
            fs::remove_dir_all(path).unwrap();
        }
    }
    let c = rusqlite::Connection::open(fixture.root.database()).unwrap();
    c.pragma_update(None, "user_version", 99).unwrap();
    drop(c);
    let mut before = BTreeMap::new();
    inventory(fixture.root.path(), fixture.root.path(), &mut before);
    let actor = DbActor::start(fixture.root.clone()).unwrap();
    assert!(!actor.info().writable);
    let persistence = Arc::new(SqliteLibraryPersistence::new(
        actor.clone(),
        actor.info().library_id.0.clone(),
    ));
    let documents = Arc::new(
        LocalDocumentStore::new(
            actor.library_root().clone(),
            actor.info().library_id.0.clone(),
        )
        .unwrap(),
    );
    let service = LibraryService::new(
        fixture.picker.clone(),
        documents,
        persistence,
        MaintenanceCoordinator::default(),
        RequestRegistry::default(),
    );
    let error = tauri::async_runtime::block_on(service.select_pdf(UUID::new())).unwrap_err();
    assert_eq!(
        error.code,
        research_workbench_core::transport::dto::IpcErrorCode::SchemaTooNew
    );
    for error in [
        tauri::async_runtime::block_on(service.confirm_import(
            UUID::new(),
            UUID::new(),
            metadata(None),
            None,
        ))
        .unwrap_err(),
        tauri::async_runtime::block_on(service.cancel_import(UUID::new(), UUID::new()))
            .unwrap_err(),
        tauri::async_runtime::block_on(service.update_metadata(
            UUID::new(),
            UUID::new(),
            0,
            metadata(None),
        ))
        .unwrap_err(),
        tauri::async_runtime::block_on(service.archive_paper(UUID::new(), UUID::new(), 0))
            .unwrap_err(),
        tauri::async_runtime::block_on(service.restore_paper(UUID::new(), UUID::new(), 0))
            .unwrap_err(),
    ] {
        assert_eq!(
            error.code,
            research_workbench_core::transport::dto::IpcErrorCode::SchemaTooNew
        );
    }
    actor.shutdown(std::time::Duration::from_secs(2)).unwrap();
    let mut after = BTreeMap::new();
    inventory(fixture.root.path(), fixture.root.path(), &mut after);
    assert_eq!(before, after);
}

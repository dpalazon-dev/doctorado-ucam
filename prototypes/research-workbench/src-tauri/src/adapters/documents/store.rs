use crate::{
    adapters::windows::{
        managed_files::{
            ManagedDirectory, ManagedFile, ManagedOpenCause, ManagedOpenError, ManagedRoot,
        },
        paths::LibraryRoot,
    },
    application::library_ports::{
        CancelPlan, DocumentStore, ImportRecord, LibraryFuture, PreparedImport, ResourceInspection,
        SelectedPdf, StageCleanup, StageFailure, StageFuture, StagedPdf,
    },
    application::reader_ports::{
        DocumentAccessOutcome, DocumentBody, DocumentProofAccess, DocumentReadAccess,
        DocumentReadHandle, DocumentUnavailableReason, ReaderFuture, RegisteredDocument,
        VerifiedDocument,
    },
    transport::error::{AppError, ErrorCode},
};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

const MAX_PDF_BYTES: u64 = 524_288_000;
const HASH_BUFFER_BYTES: usize = 64 * 1024;
static PDF_VALIDATION_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn flatten_blocking_result<T, E>(result: Result<Result<T, AppError>, E>) -> Result<T, AppError> {
    result.map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
}

#[derive(Clone)]
pub struct LocalDocumentStore {
    root: LibraryRoot,
    library_id: String,
    #[cfg(test)]
    read_gate: std::sync::Arc<std::sync::Mutex<Option<std::sync::Arc<ReadGate>>>>,
    #[cfg(test)]
    open_failure: std::sync::Arc<std::sync::Mutex<Option<OpenFailureInjection>>>,
}
#[cfg(test)]
pub(crate) struct ReadGate {
    pub entered: std::sync::mpsc::Sender<()>,
    pub release: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
}
#[cfg(test)]
enum OpenFailureInjection {
    Error(ManagedOpenError),
    Panic,
}
impl LocalDocumentStore {
    pub fn new(root: LibraryRoot, library_id: String) -> Result<Self, AppError> {
        root.ensure_bound()?;
        if !is_canonical_uuid(&library_id) {
            return Err(AppError::new(ErrorCode::PathNotAllowed));
        }
        Ok(Self {
            root,
            library_id,
            #[cfg(test)]
            read_gate: std::sync::Arc::new(std::sync::Mutex::new(None)),
            #[cfg(test)]
            open_failure: std::sync::Arc::new(std::sync::Mutex::new(None)),
        })
    }

    #[cfg(test)]
    pub(crate) fn set_read_gate(&self, gate: std::sync::Arc<ReadGate>) {
        *self.read_gate.lock().unwrap() = Some(gate);
    }

    #[cfg(test)]
    fn inject_open_failure_for_test(&self, error: ManagedOpenError) {
        *self.open_failure.lock().unwrap() = Some(OpenFailureInjection::Error(error));
    }

    #[cfg(test)]
    fn inject_open_panic_for_test(&self) {
        *self.open_failure.lock().unwrap() = Some(OpenFailureInjection::Panic);
    }
}

impl DocumentStore for LocalDocumentStore {
    fn stage(&self, operation_id: String, selected: SelectedPdf) -> StageFuture {
        let root = self.root.clone();
        Box::pin(async move {
            tauri::async_runtime::spawn_blocking(move || stage(&root, &operation_id, selected))
                .await
                .map_err(|_| {
                    StageFailure::new(
                        AppError::new(ErrorCode::StorageUnavailable),
                        StageCleanup::Pending,
                    )
                })?
        })
    }
    fn inspect_owned(&self, record: ImportRecord) -> LibraryFuture<'static, ResourceInspection> {
        let root = self.root.clone();
        let library_id = self.library_id.clone();
        Box::pin(async move {
            tauri::async_runtime::spawn_blocking(move || inspect(&root, &library_id, &record))
                .await
                .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
        })
    }
    fn promote(&self, prepared: PreparedImport) -> LibraryFuture<'static, ResourceInspection> {
        let root = self.root.clone();
        let library_id = self.library_id.clone();
        Box::pin(async move {
            tauri::async_runtime::spawn_blocking(move || promote(&root, &library_id, &prepared))
                .await
                .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
        })
    }
    fn cleanup_owned(&self, plan: CancelPlan) -> LibraryFuture<'static, ResourceInspection> {
        let root = self.root.clone();
        let library_id = self.library_id.clone();
        Box::pin(async move {
            tauri::async_runtime::spawn_blocking(move || cleanup(&root, &library_id, &plan))
                .await
                .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
        })
    }
}

struct ManagedDocumentReadHandle {
    registered: RegisteredDocument,
    file: Option<ManagedFile>,
    #[cfg(test)]
    read_gate: Option<std::sync::Arc<ReadGate>>,
}
impl DocumentReadHandle for ManagedDocumentReadHandle {
    fn registered(&self) -> &RegisteredDocument {
        &self.registered
    }
    fn read_all(mut self: Box<Self>) -> ReaderFuture<'static, DocumentBody> {
        Box::pin(async move {
            tauri::async_runtime::spawn_blocking(move || {
                let mut file = self
                    .file
                    .take()
                    .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
                let before = file.size()?;
                if before != self.registered.size_bytes as u64 || before > MAX_PDF_BYTES {
                    return Err(AppError::new(ErrorCode::IntegrityFailure));
                }
                #[cfg(test)]
                if let Some(gate) = self.read_gate.as_ref() {
                    let _ = gate.entered.send(());
                    let _ = gate.release.lock().unwrap().recv();
                }
                let bytes = file.read_bounded(MAX_PDF_BYTES)?;
                let after = file.size()?;
                if after != before || bytes.len() as u64 != before {
                    return Err(AppError::new(ErrorCode::IntegrityFailure));
                }
                let lease: VerifiedDocument = Box::new(ManagedDocumentReadHandle {
                    registered: self.registered,
                    file: Some(file),
                    #[cfg(test)]
                    read_gate: None,
                });
                Ok(DocumentBody { bytes, lease })
            })
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
        })
    }
}
impl DocumentReadAccess for LocalDocumentStore {
    fn open_verified(
        &self,
        registered: RegisteredDocument,
    ) -> ReaderFuture<'static, VerifiedDocument> {
        let proof = self.prove_access(registered);
        Box::pin(async move {
            match proof.await? {
                DocumentAccessOutcome::Available(handle) => Ok(handle),
                DocumentAccessOutcome::Unavailable(reason) => Err(reason.legacy_error()),
            }
        })
    }
}

impl DocumentProofAccess for LocalDocumentStore {
    fn prove_access(
        &self,
        registered: RegisteredDocument,
    ) -> ReaderFuture<'static, DocumentAccessOutcome> {
        let root = self.root.clone();
        let library_id = self.library_id.clone();
        #[cfg(test)]
        let read_gate = self.read_gate.lock().unwrap().clone();
        #[cfg(test)]
        let open_failure = self.open_failure.clone();
        Box::pin(async move {
            flatten_blocking_result(
                tauri::async_runtime::spawn_blocking(move || {
                    let doc_id = &registered.document.id.0;
                    if registered.library_id.0 != library_id
                        || !is_canonical_uuid(doc_id)
                        || registered.document.paper_id.0.is_empty()
                        || registered.document.status
                            != crate::transport::dto::DocumentDtoStatus::ACTIVE
                        || registered.relative_path != format!("documents/{doc_id}/original.pdf")
                        || !(0..=MAX_PDF_BYTES as i64).contains(&registered.size_bytes)
                    {
                        return Err(AppError::new(ErrorCode::PathNotAllowed));
                    }
                    #[cfg(test)]
                    let injection = { open_failure.lock().unwrap().take() };
                    #[cfg(test)]
                    let opened_file = match injection {
                        Some(OpenFailureInjection::Error(error)) => Err(error),
                        Some(OpenFailureInjection::Panic) => {
                            panic!("injected document-open panic")
                        }
                        None => managed_file_open(&root, &registered.relative_path),
                    };
                    #[cfg(not(test))]
                    let opened_file = managed_file_open(&root, &registered.relative_path);
                    let file = match opened_file {
                        Ok(file) => file,
                        Err(error) => return map_document_open_failure(error),
                    };
                    if file.size()? != registered.size_bytes as u64 {
                        return Err(AppError::new(ErrorCode::IntegrityFailure));
                    }
                    Ok(DocumentAccessOutcome::Available(
                        Box::new(ManagedDocumentReadHandle {
                            registered,
                            file: Some(file),
                            #[cfg(test)]
                            read_gate,
                        }) as VerifiedDocument,
                    ))
                })
                .await,
            )
        })
    }
}

impl DocumentUnavailableReason {
    fn legacy_error(self) -> AppError {
        match self {
            Self::Missing => AppError::new(ErrorCode::NotFound),
            Self::AccessDenied | Self::SharingViolation => {
                AppError::new(ErrorCode::StorageUnavailable)
            }
        }
    }
}

impl From<ManagedOpenCause> for DocumentUnavailableReason {
    fn from(cause: ManagedOpenCause) -> Self {
        match cause {
            ManagedOpenCause::Missing => Self::Missing,
            ManagedOpenCause::AccessDenied => Self::AccessDenied,
            ManagedOpenCause::SharingViolation => Self::SharingViolation,
        }
    }
}

fn map_document_open_failure(error: ManagedOpenError) -> Result<DocumentAccessOutcome, AppError> {
    match error {
        ManagedOpenError::Os(cause) => Ok(DocumentAccessOutcome::Unavailable(cause.into())),
        ManagedOpenError::Fatal(error) => Err(error),
    }
}

fn managed_file_open(root: &LibraryRoot, relative: &str) -> Result<ManagedFile, ManagedOpenError> {
    if relative.contains(['\\', ':']) {
        return Err(AppError::new(ErrorCode::PathNotAllowed).into());
    }
    let (parent, name) = relative
        .rsplit_once('/')
        .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?;
    let directory = managed_directory_with_cause(root, parent, false)?;
    directory.open_file_for_document(name)
}

fn managed_directory(
    root: &LibraryRoot,
    relative: &str,
    create: bool,
) -> Result<ManagedDirectory, AppError> {
    managed_directory_with_cause(root, relative, create).map_err(AppError::from)
}

fn managed_directory_with_cause(
    root: &LibraryRoot,
    relative: &str,
    create: bool,
) -> Result<ManagedDirectory, ManagedOpenError> {
    if relative.is_empty() || relative.contains(['\\', ':']) {
        return Err(AppError::new(ErrorCode::PathNotAllowed).into());
    }
    let identity = root.managed_root()?;
    let mut directory = ManagedRoot::directory(&identity)?;
    for component in relative.split('/') {
        directory = if create {
            directory
                .open_child(component, true)
                .map_err(ManagedOpenError::from)?
        } else {
            directory.open_child_for_document(component)?
        };
    }
    Ok(directory)
}

fn managed_file(root: &LibraryRoot, relative: &str, create: bool) -> Result<ManagedFile, AppError> {
    if !create {
        return managed_file_open(root, relative).map_err(AppError::from);
    }
    if relative.contains(['\\', ':']) {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    let (parent, name) = relative
        .rsplit_once('/')
        .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?;
    let directory = managed_directory(root, parent, true)?;
    directory.create_file(name)
}

fn managed_directory_if_exists(
    root: &LibraryRoot,
    relative: &str,
) -> Result<Option<ManagedDirectory>, AppError> {
    if relative.is_empty() || relative.contains(['\\', ':']) {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    let identity = root.managed_root()?;
    let mut directory = ManagedRoot::directory(&identity)?;
    for component in relative.split('/') {
        let child = directory.open_child_if_exists(component)?;
        let Some(child) = child else { return Ok(None) };
        directory = child;
    }
    Ok(Some(directory))
}

fn managed_file_if_exists(
    root: &LibraryRoot,
    relative: &str,
) -> Result<Option<ManagedFile>, AppError> {
    if relative.contains(['\\', ':']) {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    let (parent, name) = relative
        .rsplit_once('/')
        .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?;
    let Some(directory) = managed_directory_if_exists(root, parent)? else {
        return Ok(None);
    };
    directory.open_file_if_exists(name)
}
fn copy_bounded(
    input: &mut impl Read,
    output: &mut impl Write,
    limit: u64,
) -> Result<(u64, String), AppError> {
    let mut hash = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        let n = input
            .read(&mut buffer)
            .map_err(|_| AppError::new(ErrorCode::SourceUnreadable))?;
        if n == 0 {
            break;
        }
        total = total.saturating_add(n as u64);
        if total > limit {
            return Err(AppError::new(ErrorCode::InvalidInput));
        }
        output
            .write_all(&buffer[..n])
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
        hash.update(&buffer[..n]);
    }
    Ok((total, format!("{:x}", hash.finalize())))
}

fn hash_reader(reader: &mut (impl Read + Seek), limit: u64) -> Result<(u64, String), AppError> {
    reader
        .seek(SeekFrom::Start(0))
        .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
    let mut bounded = reader.take(limit.saturating_add(1));
    let mut hash = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; HASH_BUFFER_BYTES];
    loop {
        let count = bounded
            .read(&mut buffer)
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        if total > limit {
            return Err(AppError::new(ErrorCode::InvalidInput));
        }
        hash.update(&buffer[..count]);
    }
    Ok((total, format!("{:x}", hash.finalize())))
}
fn cleanup_if_staging_absent(root: &LibraryRoot, operation_id: &str) -> StageCleanup {
    if !is_canonical_uuid(operation_id) {
        return StageCleanup::Pending;
    }
    let declared_stage = format!("staging/{operation_id}/source.pdf");
    match managed_file_if_exists(root, &declared_stage) {
        Ok(None) => StageCleanup::Done,
        Ok(Some(_)) | Err(_) => StageCleanup::Pending,
    }
}
fn stage(
    root: &LibraryRoot,
    operation_id: &str,
    selected: SelectedPdf,
) -> Result<StagedPdf, StageFailure> {
    let id = uuid::Uuid::parse_str(operation_id).map_err(|_| {
        StageFailure::new(
            AppError::new(ErrorCode::InvalidInput),
            StageCleanup::Pending,
        )
    })?;
    if id.to_string() != operation_id {
        return Err(StageFailure::new(
            AppError::new(ErrorCode::InvalidInput),
            StageCleanup::Pending,
        ));
    }
    if selected.filename.is_empty()
        || selected.filename.len() > 1024
        || selected.filename.contains(['\\', '/'])
    {
        return Err(StageFailure::new(
            AppError::new(ErrorCode::InvalidInput),
            cleanup_if_staging_absent(root, operation_id),
        ));
    }
    let mut input = File::open(&selected.path).map_err(|_| {
        StageFailure::new(
            AppError::new(ErrorCode::SourceUnreadable),
            cleanup_if_staging_absent(root, operation_id),
        )
    })?;
    let dir_rel = format!("staging/{operation_id}");
    let dir = managed_directory(root, &dir_rel, true)
        .map_err(|error| StageFailure::new(error, StageCleanup::Pending))?;
    let relative = format!("{dir_rel}/source.pdf");
    let mut out = dir
        .create_file("source.pdf")
        .map_err(|error| StageFailure::new(error, StageCleanup::Pending))?;
    let (total, sha256) = match copy_bounded(&mut input, out.file_mut(), MAX_PDF_BYTES) {
        Ok(copied) => copied,
        Err(error) => {
            let cleanup = if error.code == ErrorCode::InvalidInput && out.remove().is_ok() {
                StageCleanup::Done
            } else {
                StageCleanup::Pending
            };
            return Err(StageFailure::new(error, cleanup));
        }
    };
    out.sync_all()
        .map_err(|error| StageFailure::new(error, StageCleanup::Pending))?;
    // Bound parser/buffer concurrency per process while remaining on the blocking executor.
    let _validation = PDF_VALIDATION_GATE
        .lock()
        .map_err(|_| StageFailure::new(AppError::new(ErrorCode::Busy), StageCleanup::Pending))?;
    let bytes = read_verified_snapshot(&mut out, total, &sha256)
        .map_err(|error| StageFailure::new(error, StageCleanup::Pending))?;
    if let Err(error) = super::pdf_probe::validate_pdf(bytes) {
        let cleanup = if out.remove().is_ok() {
            StageCleanup::Done
        } else {
            StageCleanup::Pending
        };
        return Err(StageFailure::new(error, cleanup));
    }
    Ok(StagedPdf {
        operation_id: operation_id.to_owned(),
        relative_path: relative,
        original_filename: selected.filename,
        sha256,
        size_bytes: total as i64,
    })
}

fn read_verified_snapshot(
    file: &mut ManagedFile,
    copied_size: u64,
    copied_sha256: &str,
) -> Result<Vec<u8>, AppError> {
    let size_before = file.size()?;
    if size_before > MAX_PDF_BYTES {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    let bytes = file.read_bounded(MAX_PDF_BYTES)?;
    let size_after = file.size()?;
    if size_before != copied_size
        || size_after != copied_size
        || bytes.len() as u64 != copied_size
        || format!("{:x}", Sha256::digest(&bytes)) != copied_sha256
    {
        return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
    }
    Ok(bytes)
}
fn inspect(
    root: &LibraryRoot,
    library_id: &str,
    record: &ImportRecord,
) -> Result<ResourceInspection, AppError> {
    validate_record_namespace(library_id, record)?;
    let staged = record.staging_path.clone();
    let target = record.destination_path.clone();
    let hash = record.sha256.clone();
    let staged_file = managed_file_if_exists(root, &staged)?;
    let staged_exists = staged_file.is_some();
    let staged_matches = match staged_file {
        Some(mut file) => matches_open_file(&mut file, hash.as_deref())?,
        None => false,
    };
    let destination_file = match target.as_deref() {
        Some(path) => managed_file_if_exists(root, path)?,
        None => None,
    };
    let destination_exists = destination_file.is_some();
    let destination_matches = match destination_file {
        Some(mut file) => matches_open_file(&mut file, hash.as_deref())?,
        None => false,
    };
    Ok(ResourceInspection {
        staged_matches,
        destination_matches,
        ambiguous: (staged_exists && !staged_matches)
            || (destination_exists && !destination_matches)
            || (staged_matches && destination_matches),
    })
}
fn is_canonical_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|parsed| parsed.to_string() == value)
}
fn validate_record_namespace(
    expected_library_id: &str,
    record: &ImportRecord,
) -> Result<(), AppError> {
    if !is_canonical_uuid(expected_library_id)
        || record.library_id != expected_library_id
        || !is_canonical_uuid(&record.operation_id)
        || !is_canonical_uuid(&record.token)
        || record.staging_path != format!("staging/{}/source.pdf", record.operation_id)
    {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    if record
        .reserved_paper_id
        .as_deref()
        .is_some_and(|paper_id| !is_canonical_uuid(paper_id))
    {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    match (&record.reserved_document_id, &record.destination_path) {
        (Some(document_id), Some(destination))
            if is_canonical_uuid(document_id)
                && destination == &format!("documents/{document_id}/original.pdf") => {}
        (None, None) => {}
        (Some(document_id), None)
            if is_canonical_uuid(document_id) && is_reuse_confirmation(record) => {}
        _ => return Err(AppError::new(ErrorCode::PathNotAllowed)),
    }
    Ok(())
}
fn is_reuse_confirmation(record: &ImportRecord) -> bool {
    let Some(raw) = record.metadata_json.as_deref() else {
        return false;
    };
    let Ok(intent) = serde_json::from_str::<serde_json::Value>(raw) else {
        return false;
    };
    let resolution = &intent["duplicateResolution"];
    intent["version"] == 1
        && intent["kind"] == "confirmation"
        && resolution["action"] == "reuseExisting"
        && resolution["paperId"].as_str() == record.reserved_paper_id.as_deref()
}
fn matches_open_file(file: &mut ManagedFile, expected: Option<&str>) -> Result<bool, AppError> {
    let Some(expected) = expected else {
        return Ok(false);
    };
    let size_before = file.size()?;
    if size_before > MAX_PDF_BYTES {
        return Ok(false);
    }
    let (size_hashed, hash) = hash_reader(file.file_mut(), MAX_PDF_BYTES)?;
    Ok(size_hashed == size_before && file.size()? == size_before && hash == expected)
}
fn promote(
    root: &LibraryRoot,
    library_id: &str,
    prepared: &PreparedImport,
) -> Result<ResourceInspection, AppError> {
    let record = &prepared.record;
    validate_record_namespace(library_id, record)?;
    if prepared.document_id != record.reserved_document_id.as_deref().unwrap_or_default()
        || prepared.destination_path != record.destination_path.as_deref().unwrap_or_default()
    {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    if prepared.reuse_paper.is_some() {
        return Ok(ResourceInspection {
            staged_matches: false,
            destination_matches: false,
            ambiguous: false,
        });
    }
    let expected = record.sha256.as_deref();
    let mut staged = managed_file(root, &record.staging_path, false)?;
    if !matches_open_file(&mut staged, expected)? {
        return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
    }
    staged.sync_all()?;
    let destination =
        managed_directory(root, &format!("documents/{}", prepared.document_id), true)?;
    let mut promoted = staged.rename_no_replace(&destination, "original.pdf")?;
    // A failed directory barrier leaves the durable intent for recovery. Never report promotion
    // as complete when the platform does not confirm syncing this directory handle.
    destination.sync_all()?;
    let destination_matches = matches_open_file(&mut promoted, expected)?;
    Ok(ResourceInspection {
        staged_matches: false,
        destination_matches,
        ambiguous: false,
    })
}
fn cleanup(
    root: &LibraryRoot,
    library_id: &str,
    plan: &CancelPlan,
) -> Result<ResourceInspection, AppError> {
    validate_record_namespace(library_id, &plan.record)?;
    if plan.staging_path != plan.record.staging_path
        || plan.destination_path != plan.record.destination_path
        || plan.expected_sha256 != plan.record.sha256
    {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    let destination_authorized = destination_cleanup_authorized(plan);
    let mut staged = managed_file_if_exists(root, &plan.staging_path)?;
    let target = plan
        .destination_path
        .as_deref()
        .map(|path| managed_file_if_exists(root, path))
        .transpose()?
        .flatten();
    if let Some(mut target) = target {
        if !destination_authorized || staged.is_some() {
            return Ok(ResourceInspection {
                staged_matches: false,
                destination_matches: false,
                ambiguous: true,
            });
        }
        let expected_size = plan
            .record
            .size_bytes
            .and_then(|size| u64::try_from(size).ok());
        if expected_size.is_none_or(|size| target.size().ok() != Some(size))
            || !matches_open_file(&mut target, plan.expected_sha256.as_deref())?
        {
            return Ok(ResourceInspection {
                staged_matches: false,
                destination_matches: false,
                ambiguous: true,
            });
        }
        target.remove()?;
    }
    if let Some(mut staged) = staged.take() {
        if !matches_open_file(&mut staged, plan.expected_sha256.as_deref())? {
            return Ok(ResourceInspection {
                staged_matches: false,
                destination_matches: false,
                ambiguous: true,
            });
        }
        staged.remove()?;
    }
    if managed_file_if_exists(root, &plan.staging_path)?.is_some()
        || plan
            .destination_path
            .as_deref()
            .map(|path| managed_file_if_exists(root, path))
            .transpose()?
            .flatten()
            .is_some()
    {
        return Ok(ResourceInspection {
            staged_matches: false,
            destination_matches: false,
            ambiguous: true,
        });
    }
    Ok(ResourceInspection {
        staged_matches: false,
        destination_matches: false,
        ambiguous: false,
    })
}

fn destination_cleanup_authorized(plan: &CancelPlan) -> bool {
    if plan.record.state != "FAILED"
        || !crate::application::library::has_durable_promoted_confirmation(&plan.record)
    {
        return false;
    }
    let Some(raw) = plan.record.error_json.as_deref() else {
        return false;
    };
    let Ok(error) = serde_json::from_str::<serde_json::Value>(raw) else {
        return false;
    };
    let Some(request_id) = error["requestId"].as_str() else {
        return false;
    };
    is_canonical_uuid(request_id)
        && error["version"] == 1
        && error["kind"] == "cancellation"
        && error["cleanup"] == "PENDING"
        && error["promotionConfirmed"] == true
        && error["receipt"]["command"] == "library_cancel_import"
        && error["receipt"]["payload"] == plan.payload
        && plan.payload["importToken"] == plan.record.token
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapters::{sqlite::actor::DbActor, windows::paths::LibraryRoot},
        application::reader_ports::DocumentUnavailableReason,
        transport::dto::{DocumentDto, DocumentDtoStatus, UUID},
    };
    use std::io::{Seek, SeekFrom};

    struct InstrumentedReader {
        bytes: Vec<u8>,
        offset: usize,
        largest_buffer: usize,
    }

    fn proof_fixture() -> (
        tempfile::TempDir,
        DbActor,
        LocalDocumentStore,
        RegisteredDocument,
    ) {
        let fixture = tempfile::tempdir().unwrap();
        let actor = DbActor::start(LibraryRoot::at(fixture.path().join("library"))).unwrap();
        let library_id = actor.info().library_id.clone();
        let document_id = UUID("123e4567-e89b-42d3-a456-426614174000".into());
        let paper_id = UUID("123e4567-e89b-42d3-a456-426614174001".into());
        let relative_path = format!("documents/{}/original.pdf", document_id.0);
        let bytes = b"synthetic proof fixture";
        let path = actor.library_root().path().join(&relative_path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        let registered = RegisteredDocument {
            library_id: library_id.clone(),
            document: DocumentDto {
                id: document_id,
                paper_id,
                original_filename: "synthetic.pdf".into(),
                sha256: "0".repeat(64),
                imported_at: "2026-10-03T00:00:00Z".into(),
                status: DocumentDtoStatus::ACTIVE,
            },
            relative_path,
            size_bytes: bytes.len() as i64,
        };
        let store =
            LocalDocumentStore::new(actor.library_root().clone(), library_id.0.clone()).unwrap();
        (fixture, actor, store, registered)
    }

    #[test]
    fn public_proof_and_reader_ports_preserve_injected_open_failures() {
        let (_fixture, _actor, store, registered) = proof_fixture();

        store.inject_open_failure_for_test(ManagedOpenError::Os(ManagedOpenCause::AccessDenied));
        assert!(matches!(
            tauri::async_runtime::block_on(store.prove_access(registered.clone())).unwrap(),
            DocumentAccessOutcome::Unavailable(DocumentUnavailableReason::AccessDenied)
        ));
        store.inject_open_failure_for_test(ManagedOpenError::Os(ManagedOpenCause::AccessDenied));
        let error = match tauri::async_runtime::block_on(store.open_verified(registered.clone())) {
            Ok(_) => panic!("legacy Reader must report the injected denial"),
            Err(error) => error,
        };
        assert_eq!(error.code, ErrorCode::StorageUnavailable);

        for code in [ErrorCode::StorageUnavailable, ErrorCode::NotFound] {
            store
                .inject_open_failure_for_test(ManagedOpenError::Fatal(AppError::new(code.clone())));
            let error = match tauri::async_runtime::block_on(store.prove_access(registered.clone()))
            {
                Ok(_) => panic!("fatal open failures must remain errors"),
                Err(error) => error,
            };
            assert_eq!(error.code, code);

            store
                .inject_open_failure_for_test(ManagedOpenError::Fatal(AppError::new(code.clone())));
            let error =
                match tauri::async_runtime::block_on(store.open_verified(registered.clone())) {
                    Ok(_) => panic!("legacy Reader must preserve fatal open errors"),
                    Err(error) => error,
                };
            assert_eq!(error.code, code);
        }

        store.inject_open_panic_for_test();
        let error = match tauri::async_runtime::block_on(store.prove_access(registered.clone())) {
            Ok(_) => panic!("a blocking open panic must remain an error"),
            Err(error) => error,
        };
        assert_eq!(error.code, ErrorCode::StorageUnavailable);
        store.inject_open_panic_for_test();
        let error = match tauri::async_runtime::block_on(store.open_verified(registered.clone())) {
            Ok(_) => panic!("legacy Reader must report a blocking open panic"),
            Err(error) => error,
        };
        assert_eq!(error.code, ErrorCode::StorageUnavailable);

        // Each injection is consumed by the real open job; normal filesystem proof remains live.
        assert!(matches!(
            tauri::async_runtime::block_on(store.prove_access(registered)).unwrap(),
            DocumentAccessOutcome::Available(_)
        ));
    }

    #[test]
    fn mapping_helper_classifies_denial_and_preserves_guard_errors() {
        let denied =
            map_document_open_failure(ManagedOpenError::Os(ManagedOpenCause::AccessDenied));
        assert!(matches!(
            denied,
            Ok(DocumentAccessOutcome::Unavailable(
                DocumentUnavailableReason::AccessDenied
            ))
        ));
        assert_eq!(
            DocumentUnavailableReason::AccessDenied.legacy_error().code,
            ErrorCode::StorageUnavailable
        );

        for code in [ErrorCode::NotFound, ErrorCode::StorageUnavailable] {
            let error = AppError::new(code.clone());
            let mapped = map_document_open_failure(ManagedOpenError::Fatal(error.clone()));
            assert!(matches!(mapped, Err(actual) if actual == error));
        }
    }

    #[test]
    fn join_helper_maps_blocking_panic_to_storage_error() {
        let joined = tauri::async_runtime::block_on(tauri::async_runtime::spawn_blocking(
            || -> Result<(), AppError> { panic!("injected document-open panic") },
        ));
        let error = flatten_blocking_result(joined).unwrap_err();
        assert_eq!(error.code, ErrorCode::StorageUnavailable);
    }

    #[test]
    fn invalid_operation_id_never_probes_a_staging_namespace() {
        let fixture = tempfile::tempdir().unwrap();
        let root = LibraryRoot::at(fixture.path().to_path_buf());
        let selected = SelectedPdf {
            path: fixture.path().join("missing.pdf"),
            filename: "missing.pdf".into(),
        };

        let failure = stage(&root, "../outside", selected).unwrap_err();

        assert_eq!(failure.error.code, ErrorCode::InvalidInput);
        assert_eq!(failure.cleanup, StageCleanup::Pending);
        assert!(!fixture.path().join("staging").exists());
    }
    impl Read for InstrumentedReader {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            self.largest_buffer = self.largest_buffer.max(buffer.len());
            let count = buffer
                .len()
                .min(self.bytes.len().saturating_sub(self.offset));
            buffer[..count].copy_from_slice(&self.bytes[self.offset..self.offset + count]);
            self.offset += count;
            Ok(count)
        }
    }
    impl Seek for InstrumentedReader {
        fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
            let offset = match position {
                SeekFrom::Start(offset) => offset as i64,
                SeekFrom::Current(offset) => self.offset as i64 + offset,
                SeekFrom::End(offset) => self.bytes.len() as i64 + offset,
            };
            if offset < 0 || offset as usize > self.bytes.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "seek",
                ));
            }
            self.offset = offset as usize;
            Ok(self.offset as u64)
        }
    }

    #[test]
    fn managed_hash_uses_a_fixed_small_read_buffer() {
        let bytes = vec![0x5a; 256 * 1024];
        let expected = format!("{:x}", Sha256::digest(&bytes));
        let mut reader = InstrumentedReader {
            bytes,
            offset: 0,
            largest_buffer: 0,
        };

        let (size, hash) = hash_reader(&mut reader, 256 * 1024).unwrap();

        assert_eq!(size, 256 * 1024);
        assert_eq!(hash, expected);
        assert!(reader.largest_buffer <= 64 * 1024);
    }

    #[test]
    fn managed_hash_rejects_growth_past_its_limit() {
        let mut reader = InstrumentedReader {
            bytes: vec![0x2a; 8],
            offset: 0,
            largest_buffer: 0,
        };

        let error = hash_reader(&mut reader, 4).unwrap_err();

        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert!(reader.largest_buffer <= 64 * 1024);
    }

    struct GrowingReader {
        phase: u8,
    }
    impl Read for GrowingReader {
        fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
            let bytes: &[u8] = match self.phase {
                0 => b"1234",
                1 => b"5678",
                _ => return Ok(0),
            };
            self.phase += 1;
            output[..bytes.len()].copy_from_slice(bytes);
            Ok(bytes.len())
        }
    }
    #[test]
    fn growth_during_read_crosses_limit_deterministically() {
        let mut source = GrowingReader { phase: 0 };
        let mut staged = Vec::new();
        let error = copy_bounded(&mut source, &mut staged, 5).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert_eq!(staged, b"1234");
    }

    #[test]
    fn growth_after_copy_is_rejected_before_pdf_parser_receives_bytes() {
        let fixture = tempfile::tempdir().unwrap();
        let root = ManagedRoot::bind(fixture.path(), true).unwrap();
        let directory = ManagedRoot::directory(&root).unwrap();
        let mut file = directory.create_file("staged.pdf").unwrap();
        file.file_mut().write_all(b"valid-original").unwrap();
        file.sync_all().unwrap();
        let original_hash = format!("{:x}", Sha256::digest(b"valid-original"));
        file.file_mut().write_all(b"-grew").unwrap();

        let error = read_verified_snapshot(&mut file, 14, &original_hash).unwrap_err();
        assert_eq!(error.code, ErrorCode::ImportRecoveryRequired);
    }
}

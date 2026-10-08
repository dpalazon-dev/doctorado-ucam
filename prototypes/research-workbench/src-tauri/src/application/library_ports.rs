use crate::transport::{
    dto::{
        DuplicateResolution, ImportPreviewDto, PageDto, PaperDto, PaperFilterDto,
        PaperMetadataInput, UUID,
    },
    error::AppError,
};
use rusqlite::Transaction;
use std::{future::Future, path::PathBuf, pin::Pin};

pub type LibraryFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;
pub type StageFuture = Pin<Box<dyn Future<Output = Result<StagedPdf, StageFailure>> + Send>>;
#[derive(Debug, Clone)]
pub struct SelectedPdf {
    pub path: PathBuf,
    pub filename: String,
}
#[derive(Debug, Clone)]
pub struct StagedPdf {
    pub operation_id: String,
    pub relative_path: String,
    pub original_filename: String,
    pub sha256: String,
    pub size_bytes: i64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageCleanup {
    Pending,
    Done,
}
#[derive(Debug, Clone)]
pub struct StageFailure {
    pub error: AppError,
    pub cleanup: StageCleanup,
}
impl StageFailure {
    pub fn new(error: AppError, cleanup: StageCleanup) -> Self {
        Self { error, cleanup }
    }
}
#[derive(Debug, Clone)]
pub struct ImportRecord {
    pub operation_id: String,
    pub token: String,
    pub request_id: Option<String>,
    pub library_id: String,
    pub state: String,
    pub staging_path: String,
    pub destination_path: Option<String>,
    pub original_filename: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<i64>,
    pub reserved_paper_id: Option<String>,
    pub reserved_document_id: Option<String>,
    pub metadata_json: Option<String>,
    pub payload_hash: Option<String>,
    pub result_json: Option<String>,
    pub error_json: Option<String>,
    pub expires_at: String,
}
#[derive(Debug, Clone)]
pub struct PreparedImport {
    pub record: ImportRecord,
    pub metadata: PaperMetadataInput,
    pub duplicate_resolution: Option<DuplicateResolution>,
    pub paper_id: String,
    pub document_id: String,
    pub destination_path: String,
    pub confirmation_payload: serde_json::Value,
    pub reuse_paper: Option<PaperDto>,
}
#[derive(Debug, Clone)]
pub struct ResourceInspection {
    pub staged_matches: bool,
    pub destination_matches: bool,
    pub ambiguous: bool,
}
#[derive(Debug, Clone)]
pub struct CancelPlan {
    pub record: ImportRecord,
    pub staging_path: String,
    pub destination_path: Option<String>,
    pub expected_sha256: Option<String>,
    pub payload: serde_json::Value,
}
#[derive(Debug, Clone)]
pub struct RecoveryReport {
    pub recovered: usize,
    pub issues: Vec<RecoveryIssue>,
}
#[derive(Debug, Clone)]
pub struct RecoveryIssue {
    pub operation_id: String,
    pub code: String,
}

pub trait NativePdfSelection: Send + Sync {
    fn select_pdf(&self) -> LibraryFuture<'_, Option<SelectedPdf>>;
}
pub trait DocumentStore: Send + Sync {
    fn stage(&self, operation_id: String, selected: SelectedPdf) -> StageFuture;
    fn inspect_owned(&self, record: ImportRecord) -> LibraryFuture<'static, ResourceInspection>;
    fn promote(&self, prepared: PreparedImport) -> LibraryFuture<'static, ResourceInspection>;
    fn cleanup_owned(&self, plan: CancelPlan) -> LibraryFuture<'static, ResourceInspection>;
}
pub struct ConfirmedRecovery {
    pub request_id: UUID,
    pub prepared: PreparedImport,
}
pub struct NewPaperDocument<'a> {
    pub paper_id: &'a str,
    pub doc_id: &'a str,
    pub metadata: &'a PaperMetadataInput,
    pub sha: &'a str,
    pub size: i64,
    pub filename: &'a str,
    pub relative: &'a str,
    pub now: &'a str,
}

pub struct ImportReservation<'a> {
    pub operation_id: &'a str,
    pub request_id: &'a str,
    pub metadata_json: &'a serde_json::Value,
    pub payload_hash: &'a str,
    pub paper_id: &'a str,
    pub document_id: &'a str,
    pub destination_path: Option<&'a str>,
}
#[derive(Debug, Clone)]
pub struct DuplicateCandidate {
    pub paper_id: String,
    pub title: String,
    pub sha256_match: bool,
    pub doi_match: bool,
}
pub trait LibraryPersistence: Send + Sync {
    fn library_id(&self) -> &str;
    fn writable(&self) -> bool;
    fn begin_import(
        &self,
        request_id: UUID,
        operation_id: String,
        token: String,
        filename: String,
        expires_at: String,
    ) -> LibraryFuture<'static, ()>;
    fn record_stage_failure(
        &self,
        operation_id: String,
        failure: StageFailure,
    ) -> LibraryFuture<'static, ()>;
    fn complete_stage_failure_cleanup(&self, operation_id: String) -> LibraryFuture<'static, ()>;
    fn previous_selection(
        &self,
        request_id: UUID,
    ) -> LibraryFuture<'static, Option<ImportPreviewDto>>;
    fn previous_confirmation(
        &self,
        request_id: UUID,
        token: UUID,
        metadata: PaperMetadataInput,
        resolution: Option<DuplicateResolution>,
    ) -> LibraryFuture<'static, Option<PaperDto>>;
    fn previous_cancel(&self, request_id: UUID, token: UUID) -> LibraryFuture<'static, bool>;
    fn record_staged(
        &self,
        request_id: UUID,
        staged: StagedPdf,
        expires_at: String,
    ) -> LibraryFuture<'static, ImportPreviewDto>;
    fn preview(&self, token: String) -> LibraryFuture<'static, ImportPreviewDto>;
    fn prepare_confirmation(
        &self,
        request_id: UUID,
        token: UUID,
        original_metadata: PaperMetadataInput,
        duplicate_resolution: Option<DuplicateResolution>,
    ) -> LibraryFuture<'static, PreparedImport>;
    fn record_promoted(&self, prepared: PreparedImport) -> LibraryFuture<'static, ()>;
    fn commit_confirmation(
        &self,
        request_id: UUID,
        prepared: PreparedImport,
        verified: ResourceInspection,
    ) -> LibraryFuture<'static, PaperDto>;
    fn prepare_cancel(&self, request_id: UUID, token: UUID) -> LibraryFuture<'static, CancelPlan>;
    fn validate_cleanup_plan(&self, plan: CancelPlan) -> LibraryFuture<'static, ()>;
    fn commit_cancel(
        &self,
        request_id: UUID,
        plan: CancelPlan,
        cleanup: ResourceInspection,
    ) -> LibraryFuture<'static, ()>;
    fn recoverable_imports(&self) -> LibraryFuture<'static, Vec<ImportRecord>>;
    fn confirmed_recovery(
        &self,
        record: ImportRecord,
    ) -> LibraryFuture<'static, Option<ConfirmedRecovery>>;
    fn mark_recovered_cancelled(
        &self,
        record: ImportRecord,
        request_id: UUID,
    ) -> LibraryFuture<'static, ()>;
    fn list_papers(
        &self,
        request_id: UUID,
        filter: PaperFilterDto,
    ) -> LibraryFuture<'static, PageDto<PaperDto>>;
    fn get_paper(&self, request_id: UUID, paper_id: UUID) -> LibraryFuture<'static, PaperDto>;
    fn update_metadata(
        &self,
        request_id: UUID,
        paper_id: UUID,
        expected_revision: i64,
        original_metadata: PaperMetadataInput,
    ) -> LibraryFuture<'static, PaperDto>;
    fn archive_paper(
        &self,
        request_id: UUID,
        paper_id: UUID,
        expected_revision: i64,
    ) -> LibraryFuture<'static, PaperDto>;
    fn restore_paper(
        &self,
        request_id: UUID,
        paper_id: UUID,
        expected_revision: i64,
    ) -> LibraryFuture<'static, PaperDto>;
}

/// Transaction-scoped storage primitives for Library use cases. Implementations may issue SQL,
/// but callers retain ownership of the transaction and its commit boundary.
pub trait LibraryRepository {
    fn current_revision(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
    ) -> Result<Option<i64>, AppError>;
    fn resolve_venue(
        &self,
        tx: &Transaction<'_>,
        name: Option<&str>,
    ) -> Result<Option<String>, AppError>;
    fn update_paper_metadata(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        expected_revision: i64,
        metadata: &PaperMetadataInput,
        venue_id: Option<&str>,
        updated_at: &str,
    ) -> Result<usize, AppError>;
    fn replace_paper_authors(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        authors: &[String],
    ) -> Result<(), AppError>;
    fn lifecycle_state(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
    ) -> Result<Option<(String, Option<String>)>, AppError>;
    fn update_lifecycle(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        expected_revision: i64,
        lifecycle: &str,
        archived_from: Option<&str>,
        updated_at: &str,
    ) -> Result<usize, AppError>;
    fn audit_change(
        &self,
        tx: &Transaction<'_>,
        request_id: &str,
        action: &str,
        entity_id: &str,
        changes: &serde_json::Value,
        created_at: &str,
    ) -> Result<(), AppError>;
    fn paper(&self, tx: &Transaction<'_>, paper_id: &str) -> Result<PaperDto, AppError>;
    fn set_last_opened_at(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        now: &str,
    ) -> Result<usize, AppError>;
    fn import_record(&self, tx: &Transaction<'_>, token: &str) -> Result<ImportRecord, AppError>;
    fn has_duplicate(
        &self,
        tx: &Transaction<'_>,
        doi: Option<&str>,
        sha256: &str,
    ) -> Result<bool, AppError>;
    fn duplicate_candidates(
        &self,
        tx: &Transaction<'_>,
        doi: Option<&str>,
        sha256: &str,
    ) -> Result<Vec<DuplicateCandidate>, AppError>;
    fn reserve_import(
        &self,
        tx: &Transaction<'_>,
        reservation: ImportReservation<'_>,
    ) -> Result<(), AppError>;
    fn insert_paper_document(
        &self,
        tx: &Transaction<'_>,
        document: NewPaperDocument<'_>,
    ) -> Result<(), AppError>;
    fn mark_import_committed(
        &self,
        tx: &Transaction<'_>,
        operation_id: &str,
        paper: &PaperDto,
        updated_at: &str,
    ) -> Result<(), AppError>;
}

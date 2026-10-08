use crate::transport::{
    dto::{DocumentDto, OpenPaperDto, PaperDto, ReadingPositionDto, UUID},
    error::AppError,
};
use rusqlite::Transaction;
use std::{future::Future, pin::Pin};

pub type ReaderFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

#[derive(Clone)]
pub struct OpenPaperCommand {
    pub request_id: UUID,
    pub paper_id: UUID,
}
#[derive(Clone)]
pub struct SaveReadingPositionCommand {
    pub request_id: UUID,
    pub document_id: UUID,
    pub expected_revision: i64,
    pub page_index: i64,
    pub zoom: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredDocument {
    pub library_id: UUID,
    pub document: DocumentDto,
    pub relative_path: String,
    pub size_bytes: i64,
}
pub type VerifiedDocument = Box<dyn DocumentReadHandle + Send>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentUnavailableReason {
    Missing,
    AccessDenied,
    SharingViolation,
}
/// A proof outcome. `Available` owns its verified handle and filesystem pins.
/// `Unavailable` represents only a classified OS open cause; it does not prove
/// registered identity, root membership, or the success of guards requiring a handle.
pub enum DocumentAccessOutcome {
    Available(VerifiedDocument),
    Unavailable(DocumentUnavailableReason),
}
pub trait DocumentProofAccess: Send + Sync {
    /// Fatal validation and guard failures remain `Err`; consumers retain an available
    /// handle for the duration of their operation.
    fn prove_access(
        &self,
        registered: RegisteredDocument,
    ) -> ReaderFuture<'static, DocumentAccessOutcome>;
}
pub trait DocumentReadAccess: Send + Sync {
    fn open_verified(
        &self,
        registered: RegisteredDocument,
    ) -> ReaderFuture<'static, VerifiedDocument>;
}
pub trait DocumentReadHandle: Send {
    fn registered(&self) -> &RegisteredDocument;
    fn read_all(self: Box<Self>) -> ReaderFuture<'static, DocumentBody>;
}
pub struct DocumentBody {
    pub bytes: Vec<u8>,
    pub lease: VerifiedDocument,
}

pub trait ReaderRepository: Send + Sync {
    fn registered_document(
        &self,
        tx: &Transaction<'_>,
        document_id: &str,
    ) -> Result<RegisteredDocument, AppError>;
    fn position(
        &self,
        tx: &Transaction<'_>,
        document_id: &str,
    ) -> Result<Option<ReadingPositionDto>, AppError>;
    fn write_position(
        &self,
        tx: &Transaction<'_>,
        expected_revision: i64,
        next: &ReadingPositionDto,
    ) -> Result<usize, AppError>;
    fn last_opened_paper_id(&self, tx: &Transaction<'_>) -> Result<Option<UUID>, AppError>;
    fn set_last_opened_paper(&self, tx: &Transaction<'_>, paper_id: &str) -> Result<(), AppError>;
    fn audit_position(
        &self,
        tx: &Transaction<'_>,
        request_id: &str,
        position: &ReadingPositionDto,
    ) -> Result<(), AppError>;
}

pub trait ReaderPersistence: Send + Sync {
    fn writable(&self) -> bool;
    fn prepare_open(&self, command: OpenPaperCommand) -> ReaderFuture<'static, RegisteredDocument>;
    fn confirm_open(
        &self,
        command: OpenPaperCommand,
        verified: VerifiedDocument,
    ) -> ReaderFuture<'static, OpenPaperDto>;
    fn last_opened_paper(&self) -> ReaderFuture<'static, Option<PaperDto>>;
    fn registered_document(&self, document_id: UUID) -> ReaderFuture<'static, RegisteredDocument>;
    fn reading_position(&self, document_id: UUID) -> ReaderFuture<'static, ReadingPositionDto>;
    fn save_reading_position(
        &self,
        command: SaveReadingPositionCommand,
    ) -> ReaderFuture<'static, ReadingPositionDto>;
}

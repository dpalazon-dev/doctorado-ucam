use crate::{
    application::reader_ports::{RegisteredDocument, VerifiedDocument},
    application::request_registry::RequestPermit,
    desktop::maintenance::OperationPermit,
    transport::{
        dto::{
            GateEvaluationDto, PhaseAnswerDto, PhaseCode, PhaseDefinitionDto, PhaseDto, UUID,
            WorkflowEvaluateGateArgs,
        },
        error::AppError,
    },
};
use rusqlite::Transaction;
use std::{future::Future, pin::Pin};

pub type WorkflowFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

pub struct WorkflowDocumentReference {
    pub paper_id: String,
    pub active_document_id: String,
    pub registered: RegisteredDocument,
}
pub enum WorkflowDocumentProof {
    Available {
        reference: WorkflowDocumentReference,
        handle: VerifiedDocument,
    },
    Unavailable {
        reference: WorkflowDocumentReference,
    },
}
pub struct WorkflowAdmission {
    pub operation: OperationPermit,
    pub request: RequestPermit,
}
pub enum PreparedAdvance {
    Replay(crate::transport::dto::WorkflowAdvancePhaseOutput),
    NeedsProof(WorkflowDocumentReference),
}
pub enum PreparedTouch {
    Replay(PhaseDto),
    NeedsProof(WorkflowDocumentReference),
    NoProof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowPhaseRecord {
    pub code: PhaseCode,
    pub definition_version: i64,
    pub state: String,
    pub revision: i64,
    pub completed_at: Option<String>,
    pub snapshot_json: Option<String>,
    pub snapshot_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowContext {
    pub initialized: bool,
    pub active_phase: Option<String>,
}

pub struct WorkflowPhaseAcceptance<'a> {
    pub revision: i64,
    pub completed_at: &'a str,
    pub snapshot_json: &'a str,
    pub snapshot_hash: &'a str,
}

/// SQL primitives used by application use cases. The caller owns the transaction boundary.
pub trait WorkflowRepository {
    fn context(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
    ) -> Result<Option<WorkflowContext>, AppError>;
    fn phases(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
    ) -> Result<Vec<WorkflowPhaseRecord>, AppError>;
    fn initialize(&self, tx: &Transaction<'_>, paper_id: &str) -> Result<(), AppError>;
    fn set_context(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        phase: &str,
    ) -> Result<(), AppError>;
    fn set_phase_clock(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        phase: &PhaseCode,
        state: &str,
        revision: i64,
    ) -> Result<(), AppError>;
    fn max_phase_revision(&self, tx: &Transaction<'_>, paper_id: &str) -> Result<i64, AppError>;
    fn answers(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        phase: &PhaseCode,
    ) -> Result<Vec<PhaseAnswerDto>, AppError>;
    fn put_answer(&self, tx: &Transaction<'_>, answer: &PhaseAnswerDto) -> Result<(), AppError>;
    fn set_phase_acceptance(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        phase: &PhaseCode,
        acceptance: WorkflowPhaseAcceptance<'_>,
    ) -> Result<(), AppError>;
    fn set_active_phase(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        phase: &PhaseCode,
    ) -> Result<(), AppError>;
}

pub trait WorkflowPersistence: Send + Sync {
    fn writable(&self) -> bool;
    fn phase(&self, paper_id: UUID, phase: PhaseCode) -> WorkflowFuture<'static, PhaseDto>;
    fn answers(
        &self,
        paper_id: UUID,
        phase: PhaseCode,
    ) -> WorkflowFuture<'static, Vec<PhaseAnswerDto>>;
    fn definition(
        &self,
        phase: PhaseCode,
        version: Option<i64>,
    ) -> WorkflowFuture<'static, PhaseDefinitionDto>;
    fn save_answer(
        &self,
        args: crate::transport::dto::WorkflowSavePhaseAnswerArgs,
    ) -> WorkflowFuture<'static, PhaseAnswerDto>;
    fn go_back(
        &self,
        args: crate::transport::dto::WorkflowGoBackToPhaseArgs,
    ) -> WorkflowFuture<'static, crate::transport::dto::WorkflowGoBackToPhaseOutput>;
    fn prepare_touch(
        &self,
        args: crate::transport::dto::WorkflowTouchPhaseArgs,
    ) -> WorkflowFuture<'static, PreparedTouch>;
    fn touch(
        &self,
        args: crate::transport::dto::WorkflowTouchPhaseArgs,
        proof: Option<WorkflowDocumentProof>,
        admission: WorkflowAdmission,
    ) -> WorkflowFuture<'static, PhaseDto>;
    fn document_reference(
        &self,
        paper_id: String,
    ) -> WorkflowFuture<'static, WorkflowDocumentReference>;
    fn prepare_advance(
        &self,
        args: crate::transport::dto::WorkflowAdvancePhaseArgs,
    ) -> WorkflowFuture<'static, PreparedAdvance>;
    fn advance(
        &self,
        args: crate::transport::dto::WorkflowAdvancePhaseArgs,
        proof: WorkflowDocumentProof,
        admission: WorkflowAdmission,
    ) -> WorkflowFuture<'static, crate::transport::dto::WorkflowAdvancePhaseOutput>;
    fn evaluate(
        &self,
        args: WorkflowEvaluateGateArgs,
        proof: WorkflowDocumentProof,
        admission: WorkflowAdmission,
    ) -> WorkflowFuture<'static, GateEvaluationDto>;
}

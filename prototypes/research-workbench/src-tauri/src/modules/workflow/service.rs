use crate::{
    application::{
        reader_ports::{DocumentAccessOutcome, DocumentProofAccess},
        request_registry::{RequestPermit, RequestRegistry},
        settings::RecoveryStatus,
        workflow_ports::{
            PreparedAdvance, PreparedTouch, WorkflowAdmission, WorkflowDocumentProof,
            WorkflowPersistence,
        },
    },
    desktop::maintenance::{MaintenanceCoordinator, OperationPermit},
    transport::{
        dto::*,
        error::{AppError, ErrorCode},
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct WorkflowService {
    persistence: Arc<dyn WorkflowPersistence>,
    maintenance: MaintenanceCoordinator,
    recovery: RecoveryStatus,
    requests: RequestRegistry,
    documents: Arc<dyn DocumentProofAccess>,
}

impl WorkflowService {
    pub fn new(
        persistence: Arc<dyn WorkflowPersistence>,
        maintenance: MaintenanceCoordinator,
        recovery: RecoveryStatus,
        requests: RequestRegistry,
        documents: Arc<dyn DocumentProofAccess>,
    ) -> Self {
        Self {
            persistence,
            maintenance,
            recovery,
            requests,
            documents,
        }
    }
    pub fn writable(&self) -> bool {
        self.persistence.writable()
    }
    pub async fn get_phase(&self, args: WorkflowGetPhaseArgs) -> Result<PhaseDto, AppError> {
        let _permit = self.maintenance.begin_operation()?;
        if !matches!(
            args.phase_code,
            PhaseCode::PRE | PhaseCode::P1 | PhaseCode::P2
        ) {
            return Err(AppError::new(ErrorCode::UnsupportedCapability));
        }
        self.persistence.phase(args.paper_id, args.phase_code).await
    }
    pub async fn get_phase_answers(
        &self,
        args: WorkflowGetPhaseAnswersArgs,
    ) -> Result<Vec<PhaseAnswerDto>, AppError> {
        let _permit = self.maintenance.begin_operation()?;
        if !matches!(
            args.phase_code,
            PhaseCode::PRE | PhaseCode::P1 | PhaseCode::P2
        ) {
            return Err(AppError::new(ErrorCode::UnsupportedCapability));
        }
        self.persistence
            .answers(args.paper_id, args.phase_code)
            .await
    }
    pub async fn get_phase_definition(
        &self,
        args: WorkflowGetPhaseDefinitionArgs,
    ) -> Result<PhaseDefinitionDto, AppError> {
        let _permit = self.maintenance.begin_operation()?;
        self.persistence
            .definition(args.phase_code, args.version)
            .await
    }
    pub async fn save_phase_answer(
        &self,
        args: WorkflowSavePhaseAnswerArgs,
    ) -> Result<PhaseAnswerDto, AppError> {
        self.ensure_mutable()?;
        let request = self.requests.acquire(&args.request_id).await?;
        let operation = self.maintenance.begin_operation()?;
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service.save_owned(args, request, operation).await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn save_owned(
        &self,
        args: WorkflowSavePhaseAnswerArgs,
        _request: RequestPermit,
        _operation: OperationPermit,
    ) -> Result<PhaseAnswerDto, AppError> {
        self.ensure_mutable()?;
        self.persistence.save_answer(args).await
    }
    pub async fn go_back_to_phase(
        &self,
        args: WorkflowGoBackToPhaseArgs,
    ) -> Result<WorkflowGoBackToPhaseOutput, AppError> {
        self.ensure_mutable()?;
        let request = self.requests.acquire(&args.request_id).await?;
        let operation = self.maintenance.begin_operation()?;
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service.go_back_owned(args, request, operation).await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn go_back_owned(
        &self,
        args: WorkflowGoBackToPhaseArgs,
        _request: RequestPermit,
        _operation: OperationPermit,
    ) -> Result<WorkflowGoBackToPhaseOutput, AppError> {
        self.ensure_mutable()?;
        self.persistence.go_back(args).await
    }
    pub async fn touch_phase(&self, args: WorkflowTouchPhaseArgs) -> Result<PhaseDto, AppError> {
        self.ensure_mutable()?;
        let request = self.requests.acquire(&args.request_id).await?;
        let operation = self.maintenance.begin_operation()?;
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service.touch_owned(args, request, operation).await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn touch_owned(
        &self,
        args: WorkflowTouchPhaseArgs,
        request: RequestPermit,
        operation: OperationPermit,
    ) -> Result<PhaseDto, AppError> {
        self.ensure_mutable()?;
        match self.persistence.prepare_touch(args.clone()).await? {
            PreparedTouch::Replay(output) => Ok(output),
            PreparedTouch::NoProof => {
                self.persistence
                    .touch(args, None, WorkflowAdmission { operation, request })
                    .await
            }
            PreparedTouch::NeedsProof(reference) => {
                let proof = if reference.registered.document.status == DocumentDtoStatus::ACTIVE {
                    match self
                        .documents
                        .prove_access(reference.registered.clone())
                        .await?
                    {
                        DocumentAccessOutcome::Available(handle) => {
                            if *handle.registered() != reference.registered {
                                return Err(AppError::new(ErrorCode::Conflict));
                            }
                            WorkflowDocumentProof::Available { reference, handle }
                        }
                        DocumentAccessOutcome::Unavailable(_) => {
                            WorkflowDocumentProof::Unavailable { reference }
                        }
                    }
                } else {
                    WorkflowDocumentProof::Unavailable { reference }
                };
                self.persistence
                    .touch(args, Some(proof), WorkflowAdmission { operation, request })
                    .await
            }
        }
    }
    pub async fn evaluate_gate(
        &self,
        args: WorkflowEvaluateGateArgs,
    ) -> Result<GateEvaluationDto, AppError> {
        let request = self.requests.acquire(&args.request_id).await?;
        let operation = self.maintenance.begin_operation()?;
        if !matches!(args.phase_code, PhaseCode::PRE | PhaseCode::P1) {
            return Err(AppError::new(ErrorCode::UnsupportedCapability));
        }
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service.evaluate_owned(args, request, operation).await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn evaluate_owned(
        &self,
        args: WorkflowEvaluateGateArgs,
        request: RequestPermit,
        operation: OperationPermit,
    ) -> Result<GateEvaluationDto, AppError> {
        self.ensure_mutable()?;
        let reference = self
            .persistence
            .document_reference(args.paper_id.0.clone())
            .await?;
        let proof = if reference.registered.document.status == DocumentDtoStatus::ACTIVE {
            match self
                .documents
                .prove_access(reference.registered.clone())
                .await?
            {
                DocumentAccessOutcome::Available(handle) => {
                    if *handle.registered() != reference.registered {
                        return Err(AppError::new(ErrorCode::Conflict));
                    }
                    WorkflowDocumentProof::Available { reference, handle }
                }
                DocumentAccessOutcome::Unavailable(_) => {
                    WorkflowDocumentProof::Unavailable { reference }
                }
            }
        } else {
            WorkflowDocumentProof::Unavailable { reference }
        };
        self.persistence
            .evaluate(args, proof, WorkflowAdmission { operation, request })
            .await
    }
    pub async fn advance_phase(
        &self,
        args: WorkflowAdvancePhaseArgs,
    ) -> Result<WorkflowAdvancePhaseOutput, AppError> {
        self.ensure_mutable()?;
        let request = self.requests.acquire(&args.request_id).await?;
        let operation = self.maintenance.begin_operation()?;
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service.advance_owned(args, request, operation).await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn advance_owned(
        &self,
        args: WorkflowAdvancePhaseArgs,
        request: RequestPermit,
        operation: OperationPermit,
    ) -> Result<WorkflowAdvancePhaseOutput, AppError> {
        self.ensure_mutable()?;
        match self.persistence.prepare_advance(args.clone()).await? {
            PreparedAdvance::Replay(output) => Ok(output),
            PreparedAdvance::NeedsProof(reference) => {
                let proof = if reference.registered.document.status == DocumentDtoStatus::ACTIVE {
                    match self
                        .documents
                        .prove_access(reference.registered.clone())
                        .await?
                    {
                        DocumentAccessOutcome::Available(handle) => {
                            if *handle.registered() != reference.registered {
                                return Err(AppError::new(ErrorCode::Conflict));
                            }
                            WorkflowDocumentProof::Available { reference, handle }
                        }
                        DocumentAccessOutcome::Unavailable(_) => {
                            WorkflowDocumentProof::Unavailable { reference }
                        }
                    }
                } else {
                    WorkflowDocumentProof::Unavailable { reference }
                };
                self.persistence
                    .advance(args, proof, WorkflowAdmission { operation, request })
                    .await
            }
        }
    }
    fn ensure_mutable(&self) -> Result<(), AppError> {
        self.recovery.ensure_mutations_allowed()?;
        if !self.persistence.writable() {
            return Err(AppError::new(ErrorCode::SchemaTooNew));
        }
        Ok(())
    }
}

use crate::{
    application::{
        reader_ports::{
            DocumentBody, DocumentReadAccess, OpenPaperCommand, ReaderPersistence,
            SaveReadingPositionCommand,
        },
        request_registry::{RequestPermit, RequestRegistry},
        settings::RecoveryStatus,
    },
    desktop::maintenance::{MaintenanceCoordinator, OperationPermit},
    transport::{
        dto::*,
        error::{AppError, ErrorCode},
    },
};
use std::sync::Arc;

pub struct ReadDocument {
    pub body: DocumentBody,
    pub(crate) permit: OperationPermit,
}

#[derive(Clone)]
pub struct ReaderService {
    persistence: Arc<dyn ReaderPersistence>,
    documents: Arc<dyn DocumentReadAccess>,
    maintenance: MaintenanceCoordinator,
    requests: RequestRegistry,
    recovery: RecoveryStatus,
}
impl ReaderService {
    pub fn new(
        persistence: Arc<dyn ReaderPersistence>,
        documents: Arc<dyn DocumentReadAccess>,
        maintenance: MaintenanceCoordinator,
        requests: RequestRegistry,
        recovery: RecoveryStatus,
    ) -> Self {
        Self {
            persistence,
            documents,
            maintenance,
            requests,
            recovery,
        }
    }
    pub async fn open_paper(
        &self,
        request_id: UUID,
        paper_id: UUID,
    ) -> Result<OpenPaperDto, AppError> {
        let request = self.requests.acquire(&request_id).await?;
        let operation = self.maintenance.begin_operation()?;
        self.recovery.ensure_mutations_allowed()?;
        if !self.persistence.writable() {
            return Err(AppError::new(ErrorCode::SchemaTooNew));
        }
        let service = self.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service
                .open_owned(request_id, paper_id, request, operation)
                .await;
            let _ = tx.send(result);
        });
        rx.await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn open_owned(
        &self,
        request_id: UUID,
        paper_id: UUID,
        _request: RequestPermit,
        _operation: OperationPermit,
    ) -> Result<OpenPaperDto, AppError> {
        self.recovery.ensure_mutations_allowed()?;
        let command = OpenPaperCommand {
            request_id,
            paper_id,
        };
        let record = self.persistence.prepare_open(command.clone()).await?;
        let verified = self.documents.open_verified(record).await?;
        self.persistence.confirm_open(command, verified).await
    }
    pub async fn get_last_opened_paper(&self) -> Result<Option<PaperDto>, AppError> {
        let _operation = self.maintenance.begin_operation()?;
        self.persistence.last_opened_paper().await
    }
    pub async fn get_reading_position(
        &self,
        document_id: UUID,
    ) -> Result<ReadingPositionDto, AppError> {
        let _operation = self.maintenance.begin_operation()?;
        self.persistence.reading_position(document_id).await
    }
    pub async fn save_reading_position(
        &self,
        command: SaveReadingPositionCommand,
    ) -> Result<ReadingPositionDto, AppError> {
        let request = self.requests.acquire(&command.request_id).await?;
        let operation = self.maintenance.begin_operation()?;
        self.recovery.ensure_mutations_allowed()?;
        if !self.persistence.writable() {
            return Err(AppError::new(ErrorCode::SchemaTooNew));
        }
        let service = self.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service.save_owned(command, request, operation).await;
            let _ = tx.send(result);
        });
        rx.await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn save_owned(
        &self,
        command: SaveReadingPositionCommand,
        _request: RequestPermit,
        _operation: OperationPermit,
    ) -> Result<ReadingPositionDto, AppError> {
        self.recovery.ensure_mutations_allowed()?;
        if !self.persistence.writable() {
            return Err(AppError::new(ErrorCode::SchemaTooNew));
        }
        self.persistence.save_reading_position(command).await
    }
    pub async fn read_document(&self, document_id: UUID) -> Result<ReadDocument, AppError> {
        let operation = self.maintenance.begin_operation()?;
        let record = self.persistence.registered_document(document_id).await?;
        let handle = self.documents.open_verified(record).await?;
        let body = handle.read_all().await?;
        Ok(ReadDocument {
            body,
            permit: operation,
        })
    }
}

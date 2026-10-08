use crate::{
    application::library_ports::{
        CancelPlan, DocumentStore, ImportRecord, LibraryPersistence, NativePdfSelection,
        RecoveryIssue, RecoveryReport, ResourceInspection,
    },
    application::request_registry::{RequestPermit, RequestRegistry},
    application::settings::RecoveryStatus,
    desktop::maintenance::{MaintenanceCoordinator, OperationPermit},
    transport::{
        dto::{
            DuplicateResolution, ImportPreviewDto, PageDto, PaperDto, PaperFilterDto,
            PaperMetadataInput, UUID,
        },
        error::{AppError, ErrorCode},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct LibraryService {
    picker: Arc<dyn NativePdfSelection>,
    documents: Arc<dyn DocumentStore>,
    persistence: Arc<dyn LibraryPersistence>,
    maintenance: MaintenanceCoordinator,
    session: String,
    tokens: Arc<Mutex<HashMap<String, TokenAuth>>>,
    inflight: Arc<Mutex<HashSet<String>>>,
    request_registry: RequestRegistry,
    recovery_status: RecoveryStatus,
}
#[derive(Clone)]
struct TokenAuth {
    expires: Instant,
    session: String,
    library_id: String,
}
struct TokenPermit {
    token: String,
    inflight: Arc<Mutex<HashSet<String>>>,
}
impl Drop for TokenPermit {
    fn drop(&mut self) {
        if let Ok(mut tokens) = self.inflight.lock() {
            tokens.remove(&self.token);
        }
    }
}
impl LibraryService {
    pub fn new(
        picker: Arc<dyn NativePdfSelection>,
        documents: Arc<dyn DocumentStore>,
        persistence: Arc<dyn LibraryPersistence>,
        maintenance: MaintenanceCoordinator,
        request_registry: RequestRegistry,
    ) -> Self {
        Self {
            picker,
            documents,
            persistence,
            maintenance,
            session: uuid::Uuid::new_v4().to_string(),
            tokens: Arc::new(Mutex::new(HashMap::new())),
            inflight: Arc::new(Mutex::new(HashSet::new())),
            request_registry,
            recovery_status: RecoveryStatus::default(),
        }
    }
    pub fn with_recovery_status(mut self, recovery_status: RecoveryStatus) -> Self {
        self.recovery_status = recovery_status;
        self
    }
    fn permit(&self) -> Result<OperationPermit, AppError> {
        self.maintenance.begin_operation()
    }
    fn writable(&self) -> Result<(), AppError> {
        if self.persistence.writable() {
            Ok(())
        } else {
            Err(AppError::new(ErrorCode::SchemaTooNew))
        }
    }
    fn lock_token(&self, token: &UUID) -> Result<TokenPermit, AppError> {
        let mut inflight = self
            .inflight
            .lock()
            .map_err(|_| AppError::new(ErrorCode::Busy))?;
        if !inflight.insert(token.0.clone()) {
            return Err(AppError::new(ErrorCode::Busy));
        }
        Ok(TokenPermit {
            token: token.0.clone(),
            inflight: self.inflight.clone(),
        })
    }
    async fn lock_request(&self, request_id: &UUID) -> Result<RequestPermit, AppError> {
        self.request_registry.acquire(request_id).await
    }
    fn token_valid(&self, token: &UUID) -> Result<(), AppError> {
        let tokens = self
            .tokens
            .lock()
            .map_err(|_| AppError::new(ErrorCode::Busy))?;
        if tokens.get(&token.0).is_some_and(|auth| {
            auth.expires > Instant::now()
                && auth.session == self.session
                && auth.library_id == self.persistence.library_id()
        }) {
            Ok(())
        } else {
            Err(AppError::new(ErrorCode::OperationCancelled))
        }
    }
    fn remember_token(&self, token: String) {
        if let Ok(mut tokens) = self.tokens.lock() {
            tokens.insert(
                token,
                TokenAuth {
                    expires: Instant::now() + Duration::from_secs(24 * 60 * 60),
                    session: self.session.clone(),
                    library_id: self.persistence.library_id().to_owned(),
                },
            );
        }
    }
    pub async fn select_pdf(&self, request_id: UUID) -> Result<Option<ImportPreviewDto>, AppError> {
        let request_permit = self.lock_request(&request_id).await?;
        let operation_permit = self.permit()?;
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service
                .select_pdf_owned(request_id, request_permit, operation_permit)
                .await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn select_pdf_owned(
        &self,
        request_id: UUID,
        _request_permit: RequestPermit,
        _operation_permit: OperationPermit,
    ) -> Result<Option<ImportPreviewDto>, AppError> {
        self.recovery_status.ensure_mutations_allowed()?;
        self.writable()?;
        if let Some(preview) = self
            .persistence
            .previous_selection(request_id.clone())
            .await?
        {
            return Ok(Some(preview));
        }
        let Some(selected) = self.picker.select_pdf().await? else {
            return Ok(None);
        };
        let operation = UUID::new();
        let token = UUID::new();
        let expires = chrono::Utc::now() + chrono::Duration::hours(24);
        let expires_at = expires.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        self.persistence
            .begin_import(
                request_id.clone(),
                operation.0.clone(),
                token.0.clone(),
                selected.filename.clone(),
                expires_at.clone(),
            )
            .await?;
        let staged = match self.documents.stage(operation.0.clone(), selected).await {
            Ok(staged) => staged,
            Err(failure) => {
                self.persistence
                    .record_stage_failure(operation.0.clone(), failure.clone())
                    .await?;
                return Err(failure.error);
            }
        };
        match self
            .persistence
            .record_staged(request_id, staged, expires_at)
            .await
        {
            Ok(preview) => {
                self.remember_token(preview.import_token.0.clone());
                Ok(Some(preview))
            }
            Err(error) => Err(error),
        }
    }
    pub async fn confirm_import(
        &self,
        request_id: UUID,
        token: UUID,
        metadata: PaperMetadataInput,
        resolution: Option<DuplicateResolution>,
    ) -> Result<PaperDto, AppError> {
        let request_permit = self.lock_request(&request_id).await?;
        let operation_permit = self.permit()?;
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service
                .confirm_import_owned(
                    request_id,
                    token,
                    metadata,
                    resolution,
                    request_permit,
                    operation_permit,
                )
                .await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn confirm_import_owned(
        &self,
        request_id: UUID,
        token: UUID,
        metadata: PaperMetadataInput,
        resolution: Option<DuplicateResolution>,
        _request_permit: RequestPermit,
        _operation_permit: OperationPermit,
    ) -> Result<PaperDto, AppError> {
        self.recovery_status.ensure_mutations_allowed()?;
        self.writable()?;
        if let Some(paper) = self
            .persistence
            .previous_confirmation(
                request_id.clone(),
                token.clone(),
                metadata.clone(),
                resolution.clone(),
            )
            .await?
        {
            return Ok(paper);
        }
        self.token_valid(&token)?;
        let _token_permit = self.lock_token(&token)?;
        let prepared = self
            .persistence
            .prepare_confirmation(request_id.clone(), token.clone(), metadata, resolution)
            .await?;
        if prepared.record.state == "COMMITTED" {
            return self
                .persistence
                .commit_confirmation(
                    request_id,
                    prepared,
                    ResourceInspection {
                        staged_matches: false,
                        destination_matches: false,
                        ambiguous: false,
                    },
                )
                .await;
        }
        if prepared.reuse_paper.is_some() {
            let plan = self.cancel_recovery_item(prepared.record.clone());
            self.persistence.validate_cleanup_plan(plan.clone()).await?;
            let proof = self.documents.cleanup_owned(plan).await?;
            if proof.ambiguous || proof.staged_matches {
                return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
            }
            let paper = self
                .persistence
                .commit_confirmation(request_id, prepared, proof)
                .await?;
            return Ok(paper);
        }
        let proof = self.documents.promote(prepared.clone()).await?;
        if !proof.destination_matches || proof.ambiguous {
            return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
        }
        self.persistence.record_promoted(prepared.clone()).await?;
        let paper = self
            .persistence
            .commit_confirmation(request_id, prepared, proof)
            .await?;
        Ok(paper)
    }
    pub async fn cancel_import(&self, request_id: UUID, token: UUID) -> Result<(), AppError> {
        let request_permit = self.lock_request(&request_id).await?;
        let operation_permit = self.permit()?;
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service
                .cancel_import_owned(request_id, token, request_permit, operation_permit)
                .await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn cancel_import_owned(
        &self,
        request_id: UUID,
        token: UUID,
        _request_permit: RequestPermit,
        _operation_permit: OperationPermit,
    ) -> Result<(), AppError> {
        self.recovery_status.ensure_mutations_allowed()?;
        self.writable()?;
        if self
            .persistence
            .previous_cancel(request_id.clone(), token.clone())
            .await?
        {
            return Ok(());
        }
        self.token_valid(&token)?;
        let _token_permit = self.lock_token(&token)?;
        let plan = self
            .persistence
            .prepare_cancel(request_id.clone(), token.clone())
            .await?;
        self.persistence.validate_cleanup_plan(plan.clone()).await?;
        let cleanup = self.documents.cleanup_owned(plan.clone()).await?;
        if cleanup.ambiguous || cleanup.staged_matches || cleanup.destination_matches {
            return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
        }
        self.persistence
            .commit_cancel(request_id, plan, cleanup)
            .await?;
        if let Ok(mut tokens) = self.tokens.lock() {
            tokens.remove(&token.0);
        }
        Ok(())
    }
    pub async fn list_papers(
        &self,
        request_id: UUID,
        filter: PaperFilterDto,
    ) -> Result<PageDto<PaperDto>, AppError> {
        let _permit = self.permit()?;
        self.persistence.list_papers(request_id, filter).await
    }
    pub async fn get_paper(&self, request_id: UUID, paper_id: UUID) -> Result<PaperDto, AppError> {
        let _permit = self.permit()?;
        self.persistence.get_paper(request_id, paper_id).await
    }
    pub async fn update_metadata(
        &self,
        request_id: UUID,
        paper_id: UUID,
        revision: i64,
        metadata: PaperMetadataInput,
    ) -> Result<PaperDto, AppError> {
        let request_permit = self.lock_request(&request_id).await?;
        let operation_permit = self.permit()?;
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service
                .update_metadata_owned(
                    request_id,
                    paper_id,
                    revision,
                    metadata,
                    request_permit,
                    operation_permit,
                )
                .await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn update_metadata_owned(
        &self,
        request_id: UUID,
        paper_id: UUID,
        revision: i64,
        metadata: PaperMetadataInput,
        _request_permit: RequestPermit,
        _operation_permit: OperationPermit,
    ) -> Result<PaperDto, AppError> {
        self.recovery_status.ensure_mutations_allowed()?;
        self.writable()?;
        self.persistence
            .update_metadata(request_id, paper_id, revision, metadata)
            .await
    }
    pub async fn archive_paper(
        &self,
        request_id: UUID,
        paper_id: UUID,
        revision: i64,
    ) -> Result<PaperDto, AppError> {
        let request_permit = self.lock_request(&request_id).await?;
        let operation_permit = self.permit()?;
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service
                .archive_paper_owned(
                    request_id,
                    paper_id,
                    revision,
                    request_permit,
                    operation_permit,
                )
                .await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn archive_paper_owned(
        &self,
        request_id: UUID,
        paper_id: UUID,
        revision: i64,
        _request_permit: RequestPermit,
        _operation_permit: OperationPermit,
    ) -> Result<PaperDto, AppError> {
        self.recovery_status.ensure_mutations_allowed()?;
        self.writable()?;
        self.persistence
            .archive_paper(request_id, paper_id, revision)
            .await
    }
    pub async fn restore_paper(
        &self,
        request_id: UUID,
        paper_id: UUID,
        revision: i64,
    ) -> Result<PaperDto, AppError> {
        let request_permit = self.lock_request(&request_id).await?;
        let operation_permit = self.permit()?;
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service
                .restore_paper_owned(
                    request_id,
                    paper_id,
                    revision,
                    request_permit,
                    operation_permit,
                )
                .await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn restore_paper_owned(
        &self,
        request_id: UUID,
        paper_id: UUID,
        revision: i64,
        _request_permit: RequestPermit,
        _operation_permit: OperationPermit,
    ) -> Result<PaperDto, AppError> {
        self.recovery_status.ensure_mutations_allowed()?;
        self.writable()?;
        self.persistence
            .restore_paper(request_id, paper_id, revision)
            .await
    }
    pub async fn reconcile_imports(&self) -> Result<RecoveryReport, AppError> {
        let permit = self.maintenance.begin_maintenance()?;
        self.reconcile_imports_with_permit(permit).await
    }
    pub async fn reconcile_imports_with_permit(
        &self,
        permit: OperationPermit,
    ) -> Result<RecoveryReport, AppError> {
        let service = self.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = service.reconcile_imports_job().await;
            drop(permit);
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
    }
    async fn reconcile_imports_job(&self) -> Result<RecoveryReport, AppError> {
        let records = self.persistence.recoverable_imports().await?;
        let mut report = RecoveryReport {
            recovered: 0,
            issues: Vec::new(),
        };
        for record in records {
            let id = record.operation_id.clone();
            let stage_failure = record
                .error_json
                .as_deref()
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
                .filter(|value| {
                    value["version"] == 1
                        && value["kind"] == "stageFailure"
                        && value["cleanup"] == "PENDING"
                });
            match self.documents.inspect_owned(record.clone()).await {
                Ok(inspection) if stage_failure.is_some() => {
                    if !inspection.ambiguous
                        && !inspection.staged_matches
                        && !inspection.destination_matches
                    {
                        match self
                            .persistence
                            .complete_stage_failure_cleanup(record.operation_id.clone())
                            .await
                        {
                            Ok(()) => report.recovered += 1,
                            Err(_) => report.issues.push(RecoveryIssue {
                                operation_id: id,
                                code: "recoveryRequired".into(),
                            }),
                        }
                    } else {
                        report.issues.push(RecoveryIssue {
                            operation_id: id,
                            code: "recoveryRequired".into(),
                        });
                    }
                }
                Ok(inspection) if inspection.ambiguous => report.issues.push(RecoveryIssue {
                    operation_id: id,
                    code: "ambiguous".into(),
                }),
                Ok(inspection) if record.state == "COMMITTED" => {
                    if record.destination_path.is_some() && !inspection.destination_matches {
                        report.issues.push(RecoveryIssue {
                            operation_id: id,
                            code: "integrityFailure".into(),
                        });
                    }
                }
                Ok(_inspection) if record.state == "FAILED" => {
                    let cancellation = record
                        .error_json
                        .as_deref()
                        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
                        .filter(|value| {
                            value["version"] == 1
                                && value["kind"] == "cancellation"
                                && value["cleanup"] == "PENDING"
                        });
                    if let Some(intent) = cancellation {
                        if let Some(request) =
                            intent["requestId"].as_str().map(|s| UUID(s.to_owned()))
                        {
                            let plan = self.cancel_recovery_item(record.clone());
                            if self
                                .persistence
                                .validate_cleanup_plan(plan.clone())
                                .await
                                .is_err()
                            {
                                report.issues.push(RecoveryIssue {
                                    operation_id: id,
                                    code: "recoveryRequired".into(),
                                });
                                continue;
                            }
                            match self.documents.cleanup_owned(plan).await {
                                Ok(proof)
                                    if !proof.ambiguous
                                        && !proof.staged_matches
                                        && !proof.destination_matches =>
                                {
                                    match self
                                        .persistence
                                        .mark_recovered_cancelled(record.clone(), request)
                                        .await
                                    {
                                        Ok(()) => report.recovered += 1,
                                        Err(_) => report.issues.push(RecoveryIssue {
                                            operation_id: id,
                                            code: "recoveryRequired".into(),
                                        }),
                                    }
                                }
                                _ => report.issues.push(RecoveryIssue {
                                    operation_id: id,
                                    code: "recoveryRequired".into(),
                                }),
                            }
                        } else {
                            report.issues.push(RecoveryIssue {
                                operation_id: id,
                                code: "recoveryRequired".into(),
                            })
                        }
                    } else {
                        report.issues.push(RecoveryIssue {
                            operation_id: id,
                            code: "recoveryRequired".into(),
                        })
                    }
                }
                Ok(inspection) => match self.persistence.confirmed_recovery(record.clone()).await {
                    Ok(Some(recovery)) => {
                        let prepared = recovery.prepared;
                        let mut verified = inspection.clone();
                        if prepared.reuse_paper.is_some() {
                            let plan = self.cancel_recovery_item(record.clone());
                            if self
                                .persistence
                                .validate_cleanup_plan(plan.clone())
                                .await
                                .is_err()
                            {
                                report.issues.push(RecoveryIssue {
                                    operation_id: id,
                                    code: "recoveryRequired".into(),
                                });
                                continue;
                            }
                            match self.documents.cleanup_owned(plan).await {
                                Ok(proof) if !proof.ambiguous && !proof.staged_matches => {
                                    verified = proof;
                                }
                                _ => {
                                    report.issues.push(RecoveryIssue {
                                        operation_id: id,
                                        code: "recoveryRequired".into(),
                                    });
                                    continue;
                                }
                            }
                        } else {
                            if inspection.destination_matches == inspection.staged_matches
                                || (!inspection.destination_matches && !inspection.staged_matches)
                            {
                                report.issues.push(RecoveryIssue {
                                    operation_id: id,
                                    code: "recoveryRequired".into(),
                                });
                                continue;
                            }
                            if inspection.staged_matches {
                                match self.documents.promote(prepared.clone()).await {
                                    Ok(proof) if proof.destination_matches => verified = proof,
                                    _ => {
                                        report.issues.push(RecoveryIssue {
                                            operation_id: id,
                                            code: "recoveryRequired".into(),
                                        });
                                        continue;
                                    }
                                }
                            }
                            if self
                                .persistence
                                .record_promoted(prepared.clone())
                                .await
                                .is_err()
                            {
                                report.issues.push(RecoveryIssue {
                                    operation_id: id,
                                    code: "recoveryRequired".into(),
                                });
                                continue;
                            }
                        }
                        match self
                            .persistence
                            .commit_confirmation(recovery.request_id, prepared, verified)
                            .await
                        {
                            Ok(_) => report.recovered += 1,
                            Err(_) => report.issues.push(RecoveryIssue {
                                operation_id: id,
                                code: "recoveryRequired".into(),
                            }),
                        }
                    }
                    Ok(None) => report.issues.push(RecoveryIssue {
                        operation_id: id,
                        code: "recoveryRequired".into(),
                    }),
                    Err(_) => report.issues.push(RecoveryIssue {
                        operation_id: id,
                        code: "recoveryRequired".into(),
                    }),
                },
                Err(_) => report.issues.push(RecoveryIssue {
                    operation_id: id,
                    code: "recoveryRequired".into(),
                }),
            }
        }
        Ok(report)
    }
    pub fn library_id(&self) -> &str {
        self.persistence.library_id()
    }
    pub fn pending_tokens(&self) -> usize {
        self.tokens.lock().map(|t| t.len()).unwrap_or(0)
    }
    pub fn cancel_recovery_item(&self, record: ImportRecord) -> CancelPlan {
        let payload = record
            .error_json
            .as_deref()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
            .filter(|intent| intent["kind"] == "cancellation")
            .map(|intent| intent["receipt"]["payload"].clone())
            .unwrap_or(serde_json::Value::Null);
        CancelPlan {
            staging_path: record.staging_path.clone(),
            destination_path: record.destination_path.clone(),
            expected_sha256: record.sha256.clone(),
            payload,
            record,
        }
    }
}

pub use super::dto::IpcErrorCode as ErrorCode;
use super::dto::{
    ContractVersion, FailureFlag, IpcErrorDto, IpcFailure, IpcResult, IpcSuccess, SuccessFlag, UUID,
};
#[derive(Debug, Clone, PartialEq)]
pub struct AppError {
    pub code: ErrorCode,
    pub details: Option<super::dto::JsonObject>,
}
impl AppError {
    pub fn new(code: ErrorCode) -> Self {
        Self {
            code,
            details: None,
        }
    }
    pub fn conflict_revision(revision: i64) -> Self {
        Self {
            code: ErrorCode::Conflict,
            details: Some([("currentRevision".into(), serde_json::json!(revision))].into()),
        }
    }
    pub fn message(&self) -> &'static str {
        match self.code {
            ErrorCode::InvalidInput => "The request contains invalid data.",
            ErrorCode::NotFound => "The requested record was not found.",
            ErrorCode::Conflict => "The record or request has changed. Check the details.",
            ErrorCode::Busy => "The library is busy. Try again.",
            ErrorCode::SchemaTooNew => "The library requires a newer version of the app.",
            ErrorCode::UnsupportedCapability => "This feature is not available yet.",
            ErrorCode::PathNotAllowed => "This operation is not allowed.",
            ErrorCode::MigrationFailed => {
                "Could not update the library. The backup has been preserved."
            }
            ErrorCode::BackupFailed => "Could not verify the backup.",
            ErrorCode::IntegrityFailure => "The library needs an integrity check.",
            _ => "Could not complete the operation. Check the library status.",
        }
    }
    pub fn retryable(&self) -> bool {
        matches!(self.code, ErrorCode::Busy | ErrorCode::StorageUnavailable)
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}
impl std::error::Error for AppError {}
impl From<rusqlite::Error> for AppError {
    fn from(_: rusqlite::Error) -> Self {
        Self::new(ErrorCode::StorageUnavailable)
    }
}
impl From<std::io::Error> for AppError {
    fn from(_: std::io::Error) -> Self {
        Self::new(ErrorCode::StorageUnavailable)
    }
}
pub fn envelope<T>(request_id: UUID, result: Result<T, AppError>) -> IpcResult<T> {
    match result {
        Ok(data) => IpcResult::Success(IpcSuccess {
            contract_version: ContractVersion(1),
            request_id,
            ok: SuccessFlag(true),
            data,
        }),
        Err(error) => IpcResult::Failure(IpcFailure {
            contract_version: ContractVersion(1),
            request_id,
            ok: FailureFlag(false),
            error: IpcErrorDto {
                code: error.code.clone(),
                message: error.message().into(),
                retryable: error.retryable(),
                details: error.details,
            },
        }),
    }
}

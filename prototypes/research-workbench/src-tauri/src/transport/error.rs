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
            ErrorCode::InvalidInput => "La solicitud contiene datos no válidos.",
            ErrorCode::NotFound => "No se encontró el registro solicitado.",
            ErrorCode::Conflict => "El registro o la solicitud ha cambiado. Revisa los datos.",
            ErrorCode::Busy => "La biblioteca está ocupada. Inténtalo de nuevo.",
            ErrorCode::SchemaTooNew => {
                "La biblioteca requiere una versión más reciente de la aplicación."
            }
            ErrorCode::UnsupportedCapability => "Esta función todavía no está disponible.",
            ErrorCode::PathNotAllowed => "La operación no está permitida.",
            ErrorCode::MigrationFailed => {
                "No se pudo actualizar la biblioteca. Se conserva la copia de seguridad."
            }
            ErrorCode::BackupFailed => "No se pudo verificar la copia de seguridad.",
            ErrorCode::IntegrityFailure => "La biblioteca necesita una revisión de integridad.",
            _ => "No se pudo completar la operación. Revisa el estado de la biblioteca.",
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

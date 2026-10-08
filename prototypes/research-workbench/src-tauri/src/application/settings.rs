use crate::{
    application::library_ports::{RecoveryIssue, RecoveryReport},
    transport::error::{AppError, ErrorCode},
};
use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
#[derive(Clone)]
pub struct LibrarySummary {
    pub library_id: String,
    pub display_name: String,
    pub root_label: String,
    pub schema_version: i64,
    pub writable: bool,
}
pub struct LibraryStatus {
    pub writable: bool,
    pub active_operations: i64,
    pub recovery_required: bool,
}
pub struct SettingsAppInfo {
    pub app_version: String,
    pub library: LibrarySummary,
    pub capabilities: BTreeMap<String, bool>,
}

#[derive(Clone)]
pub struct RecoveryStatus(Arc<Mutex<RecoveryStatusValue>>);

enum RecoveryStatusValue {
    Ready(Vec<RecoveryIssue>),
    Pending,
    Failed(ErrorCode),
}

impl Default for RecoveryStatus {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(RecoveryStatusValue::Ready(Vec::new()))))
    }
}

impl RecoveryStatus {
    pub fn pending() -> Self {
        Self(Arc::new(Mutex::new(RecoveryStatusValue::Pending)))
    }

    pub fn finish(&self, result: Result<RecoveryReport, AppError>) {
        let value = match result {
            Ok(report) => RecoveryStatusValue::Ready(report.issues),
            Err(error) => RecoveryStatusValue::Failed(error.code),
        };
        match self.0.lock() {
            Ok(mut status) => *status = value,
            Err(poisoned) => *poisoned.into_inner() = value,
        }
    }

    pub fn required(&self) -> bool {
        match self.0.lock() {
            Ok(status) => match &*status {
                RecoveryStatusValue::Ready(issues) => !issues.is_empty(),
                RecoveryStatusValue::Pending | RecoveryStatusValue::Failed(_) => true,
            },
            Err(_) => true,
        }
    }

    pub fn failure_code(&self) -> Option<ErrorCode> {
        match self.0.lock() {
            Ok(status) => match &*status {
                RecoveryStatusValue::Failed(code) => Some(code.clone()),
                _ => None,
            },
            Err(_) => Some(ErrorCode::ImportRecoveryRequired),
        }
    }

    pub fn ensure_mutations_allowed(&self) -> Result<(), AppError> {
        match self.0.lock() {
            Ok(status) => match &*status {
                RecoveryStatusValue::Ready(_) => Ok(()),
                RecoveryStatusValue::Pending => Err(AppError::new(ErrorCode::Busy)),
                RecoveryStatusValue::Failed(_) => {
                    Err(AppError::new(ErrorCode::ImportRecoveryRequired))
                }
            },
            Err(_) => Err(AppError::new(ErrorCode::ImportRecoveryRequired)),
        }
    }
}

pub trait SettingsQuery: Send + Sync {
    fn library_info(&self) -> LibrarySummary;
    fn library_status(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<LibraryStatus, AppError>> + Send + '_>>;
}

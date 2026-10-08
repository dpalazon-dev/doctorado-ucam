use super::managed_files::ManagedRoot;
use crate::transport::error::{AppError, ErrorCode};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
#[derive(Debug, Clone)]
pub struct LibraryRoot {
    path: PathBuf,
    identity: Option<Arc<ManagedRoot>>,
}
impl LibraryRoot {
    pub fn at(path: PathBuf) -> Self {
        Self {
            path,
            identity: None,
        }
    }
    pub(crate) fn create_and_bind(&mut self) -> Result<(), AppError> {
        if self.identity.is_none() {
            self.identity = Some(ManagedRoot::bind(&self.path, true)?);
        }
        Ok(())
    }
    pub fn is_bound(&self) -> bool {
        self.identity.is_some()
    }
    pub(crate) fn ensure_bound(&self) -> Result<&ManagedRoot, AppError> {
        self.identity
            .as_deref()
            .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))
    }
    pub(crate) fn managed_root(&self) -> Result<Arc<ManagedRoot>, AppError> {
        self.identity
            .clone()
            .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))
    }
    pub(crate) fn database_exists(&self) -> Result<bool, AppError> {
        self.ensure_bound()?.database_exists()
    }
    pub(crate) fn create_database_after_lock(&self) -> Result<(), AppError> {
        self.ensure_bound()?.create_database_after_lock()
    }
    pub fn canonical_path(&self) -> Result<&Path, AppError> {
        Ok(self.ensure_bound()?.canonical_path())
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn database(&self) -> PathBuf {
        self.path.join("research.sqlite")
    }
    pub fn backups(&self) -> Result<PathBuf, AppError> {
        Ok(self
            .path
            .parent()
            .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?
            .join("backups"))
    }
    pub fn logs(&self) -> Result<PathBuf, AppError> {
        Ok(self
            .path
            .parent()
            .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?
            .join("logs"))
    }
    pub fn display_label(&self) -> &'static str {
        "Local Research Workbench library"
    }
}
pub fn resolve_data_root() -> Result<LibraryRoot, AppError> {
    #[cfg(debug_assertions)]
    if let Some(path) = std::env::var_os("RESEARCH_WORKBENCH_TEST_ROOT") {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(AppError::new(ErrorCode::PathNotAllowed));
        }
        return Ok(LibraryRoot::at(path));
    }
    let base = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| AppError::new(ErrorCode::StorageUnavailable))?;
    let path = PathBuf::from(base);
    if !path.is_absolute() {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    Ok(LibraryRoot::at(
        path.join("ResearchWorkbench").join("library"),
    ))
}

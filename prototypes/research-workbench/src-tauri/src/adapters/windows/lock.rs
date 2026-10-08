use crate::transport::error::{AppError, ErrorCode};
use fs2::FileExt;
use std::{
    fs::{File, OpenOptions},
    path::Path,
};
/// OS advisory lock is retained on the DB thread until the connection has closed.
pub struct LibraryLock {
    _file: File,
}
impl LibraryLock {
    pub fn acquire(root: &Path) -> Result<Self, AppError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join(".writer.lock"))?;
        file.try_lock_exclusive()
            .map_err(|_| AppError::new(ErrorCode::Busy))?;
        Ok(Self { _file: file })
    }
}

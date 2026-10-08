//! Nonmutating version classification. Sidecars are recovered only in external RAII scratch.
use super::migrations;
use crate::{
    adapters::windows::paths::LibraryRoot,
    transport::error::{AppError, ErrorCode},
};
use rusqlite::{Connection, OpenFlags};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::Read, path::Path};
type Fingerprint = BTreeMap<String, (u64, String)>;
const FILES: [&str; 4] = [
    "research.sqlite",
    "research.sqlite-wal",
    "research.sqlite-shm",
    "research.sqlite-journal",
];
fn fingerprint(root: &Path) -> Result<Fingerprint, AppError> {
    let mut result = BTreeMap::new();
    for name in FILES {
        let mut file = match fs::File::open(root.join(name)) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        let mut hash = Sha256::new();
        let mut size = 0;
        let mut buffer = [0u8; 65536];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            size += n as u64;
            hash.update(&buffer[..n]);
        }
        result.insert(name.to_owned(), (size, format!("{:x}", hash.finalize())));
    }
    Ok(result)
}
pub(super) struct Probe {
    pub connection: Connection,
    pub scratch: Option<tempfile::TempDir>,
    pub version: i64,
    source: Fingerprint,
}
impl Probe {
    pub fn verify_unchanged(&self, root: &LibraryRoot) -> Result<(), AppError> {
        if fingerprint(root.path())? != self.source {
            return Err(AppError::new(ErrorCode::Busy));
        }
        Ok(())
    }
}
pub(super) fn probe(root: &LibraryRoot) -> Result<Probe, AppError> {
    let before = fingerprint(root.path())?;
    let pending = ["research.sqlite-wal", "research.sqlite-journal"]
        .iter()
        .any(|name| before.get(*name).is_some_and(|(size, _)| *size > 0));
    let (connection, scratch) = if pending {
        let dir = tempfile::Builder::new()
            .prefix("research-schema-")
            .tempdir()?;
        for name in FILES
            .into_iter()
            .filter(|name| *name != "research.sqlite-shm")
        {
            if before.contains_key(name) {
                fs::copy(root.path().join(name), dir.path().join(name))?;
            }
        }
        // Verify copied bytes as well as unchanged source before SQLite may recover scratch.
        let copied = fingerprint(dir.path())?;
        if before
            .iter()
            .filter(|(name, _)| name.as_str() != "research.sqlite-shm")
            .any(|(name, hash)| copied.get(name) != Some(hash))
            || fingerprint(root.path())? != before
        {
            return Err(AppError::new(ErrorCode::Busy));
        }
        (
            Connection::open(dir.path().join("research.sqlite"))?,
            Some(dir),
        )
    } else {
        // URI escaping is explicit so '#'/'?' and non-ASCII paths cannot alter parameters.
        let path = root.database().to_string_lossy().replace('\\', "/");
        let escaped = path
            .bytes()
            .map(|b| {
                if b.is_ascii_alphanumeric() || b"/:._-".contains(&b) {
                    (b as char).to_string()
                } else {
                    format!("%{b:02X}")
                }
            })
            .collect::<String>();
        (
            Connection::open_with_flags(
                format!("file:{escaped}?immutable=1"),
                OpenFlags::SQLITE_OPEN_READ_ONLY
                    | OpenFlags::SQLITE_OPEN_URI
                    | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )?,
            None,
        )
    };
    let version = migrations::schema_version(&connection)?;
    let result = Probe {
        connection,
        scratch,
        version,
        source: before,
    };
    result.verify_unchanged(root)?;
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_source_is_busy_and_scratch_is_removed() {
        let dir = tempfile::tempdir().unwrap();
        let root = LibraryRoot::at(dir.path().join("library"));
        fs::create_dir_all(root.path()).unwrap();
        let c = Connection::open(root.database()).unwrap();
        c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; CREATE TABLE fixture(value); INSERT INTO fixture VALUES(1);").unwrap();
        let result = probe(&root).unwrap();
        let scratch = result.scratch.as_ref().unwrap().path().to_owned();
        c.execute("INSERT INTO fixture VALUES(2)", []).unwrap();
        assert_eq!(
            result.verify_unchanged(&root).unwrap_err().code,
            ErrorCode::Busy
        );
        drop(result);
        assert!(!scratch.exists());
    }
}

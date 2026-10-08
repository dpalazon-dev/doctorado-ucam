use crate::{
    adapters::windows::paths::LibraryRoot,
    application::unit_of_work::with_transaction,
    transport::error::{AppError, ErrorCode},
};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path, time::Duration};
pub const SCHEMA_VERSION: i64 = 2;
pub struct Migration<'a> {
    pub version: i64,
    pub sql: &'a str,
}
pub const MIGRATIONS: &[Migration<'static>] = &[
    Migration {
        version: 1,
        sql: include_str!("migrations/0001_library.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("migrations/0002_workflow.sql"),
    },
];
pub fn checksum(sql: &str) -> String {
    format!("{:x}", Sha256::digest(sql.as_bytes()))
}
pub fn verify_integrity(c: &Connection) -> Result<(), AppError> {
    if c.query_row("PRAGMA quick_check", [], |r| r.get::<_, String>(0))? != "ok"
        || c.prepare("PRAGMA foreign_key_check")?.exists([])?
    {
        return Err(AppError::new(ErrorCode::IntegrityFailure));
    }
    Ok(())
}
pub fn schema_version(c: &Connection) -> Result<i64, AppError> {
    Ok(c.pragma_query_value(None, "user_version", |r| r.get(0))?)
}
pub fn verify_ledger(c: &Connection) -> Result<(), AppError> {
    let version = schema_version(c)?;
    if version == 0 {
        return Ok(());
    }
    for m in MIGRATIONS.iter().filter(|m| m.version <= version) {
        let prior: Option<String> = c
            .query_row(
                "SELECT checksum FROM schema_migrations WHERE version=?1",
                [m.version],
                |r| r.get(0),
            )
            .optional()?;
        if prior.as_deref() != Some(checksum(m.sql).as_str()) {
            return Err(AppError::new(ErrorCode::IntegrityFailure));
        }
    }
    Ok(())
}
fn file_hash(path: &Path) -> Result<String, AppError> {
    let mut file = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn reject_reparse(path: &Path) -> Result<(), AppError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(AppError::new(ErrorCode::PathNotAllowed));
        }
    }
    Ok(())
}
fn copy_tree(source: &Path, dest: &Path) -> Result<(), AppError> {
    if !source.exists() {
        return Ok(());
    }
    reject_reparse(source)?;
    if source.is_dir() {
        fs::create_dir(dest)?;
        for e in fs::read_dir(source)? {
            let e = e?;
            copy_tree(&e.path(), &dest.join(e.file_name()))?;
        }
    } else {
        let original_hash = file_hash(source)?;
        fs::copy(source, dest)?;
        if original_hash != file_hash(dest)? {
            return Err(AppError::new(ErrorCode::BackupFailed));
        }
    }
    Ok(())
}
fn inventory(
    base: &Path,
    current: &Path,
    out: &mut Vec<serde_json::Value>,
) -> Result<(), AppError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        reject_reparse(&path)?;
        if path.is_dir() {
            inventory(base, &path, out)?;
        } else {
            let relative = path
                .strip_prefix(base)
                .map_err(|_| AppError::new(ErrorCode::PathNotAllowed))?
                .to_str()
                .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?
                .replace('\\', "/");
            out.push(serde_json::json!({"path":relative,"sizeBytes":entry.metadata()?.len(),"sha256":file_hash(&path)?}));
        }
    }
    Ok(())
}
/// Caller holds the library writer/maintenance lease. Never run inside a SQL transaction.
pub fn snapshot(c: &Connection, root: &LibraryRoot) -> Result<std::path::PathBuf, AppError> {
    if !c.is_autocommit() {
        return Err(AppError::new(ErrorCode::Busy));
    }
    let result: Result<std::path::PathBuf, AppError> = (|| {
        verify_integrity(c)?;
        let backup_id = uuid::Uuid::new_v4().to_string();
        let library_id: String = c.query_row(
            "SELECT library_id FROM library_identity WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        let dest = root.backups()?.join(&backup_id);
        fs::create_dir_all(root.backups()?)?;
        fs::create_dir(&dest)?;
        let mut target = Connection::open(dest.join("research.sqlite"))?;
        {
            let backup = rusqlite::backup::Backup::new(c, &mut target)?;
            backup.run_to_completion(64, Duration::from_millis(10), None)?;
        }
        verify_integrity(&target)?;
        drop(target);
        for name in ["library.json", "documents", "staging", "recovery"] {
            copy_tree(&root.path().join(name), &dest.join(name))?;
        }
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(dest.join("library.json"))?)
                .map_err(|_| AppError::new(ErrorCode::BackupFailed))?;
        if metadata["libraryId"] != library_id || metadata["formatVersion"] != 1 {
            return Err(AppError::new(ErrorCode::BackupFailed));
        }
        let mut statement =
            c.prepare("SELECT relative_path,sha256 FROM documents WHERE status != 'MISSING'")?;
        let docs =
            statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        for doc in docs {
            let (relative, expected) = doc?;
            let path = Path::new(&relative);
            if path.is_absolute()
                || path
                    .components()
                    .any(|part| !matches!(part, std::path::Component::Normal(_)))
                || relative.contains('\\')
                || relative.contains(':')
            {
                return Err(AppError::new(ErrorCode::PathNotAllowed));
            }
            if file_hash(&dest.join(path))? != expected {
                return Err(AppError::new(ErrorCode::BackupFailed));
            }
        }
        let mut files = Vec::new();
        inventory(&dest, &dest, &mut files)?;
        files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
        let manifest = serde_json::json!({"backupId":backup_id,"libraryId":library_id,"kind":"preMigration","appVersion":env!("CARGO_PKG_VERSION"),"schemaVersion":schema_version(c)?,"verified":true,"createdAt":chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis,true),"files":files});
        {
            use std::io::Write;
            let mut file = fs::File::create(dest.join("manifest.json"))?;
            file.write_all(
                &serde_json::to_vec_pretty(&manifest)
                    .map_err(|_| AppError::new(ErrorCode::BackupFailed))?,
            )?;
            file.sync_all()?;
        }
        Ok(dest)
    })();
    result.map_err(|_| AppError::new(ErrorCode::BackupFailed))
}
pub fn migrate(
    c: &mut Connection,
    root: &LibraryRoot,
    steps: &[Migration<'_>],
) -> Result<(), AppError> {
    verify_ledger(c)?;
    let current = schema_version(c)?;
    let pending: Vec<_> = steps.iter().filter(|s| s.version > current).collect();
    if pending.is_empty() {
        return Ok(());
    }
    if current > 0 {
        snapshot(c, root)?;
    }
    with_transaction(c, |tx| {
        for step in pending {
            tx.execute_batch(step.sql)?;
            tx.execute(
                "INSERT INTO schema_migrations(version,checksum,applied_at) VALUES(?1,?2,?3)",
                rusqlite::params![
                    step.version,
                    checksum(step.sql),
                    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
                ],
            )?;
            tx.pragma_update(None, "user_version", step.version)?;
        }
        verify_integrity(tx)?;
        Ok(())
    })
    .map_err(|_| AppError::new(ErrorCode::MigrationFailed))
}
pub fn open_read_only(root: &LibraryRoot) -> Result<Connection, AppError> {
    Ok(Connection::open_with_flags(
        root.database(),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?)
}

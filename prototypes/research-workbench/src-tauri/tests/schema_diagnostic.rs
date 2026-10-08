use research_workbench_core::adapters::{sqlite::actor::DbActor, windows::paths::LibraryRoot};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, time::Duration};
fn inventory(root: &std::path::Path) -> BTreeMap<String, String> {
    fn visit(root: &std::path::Path, path: &std::path::Path, out: &mut BTreeMap<String, String>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            if path.is_dir() {
                out.insert(name, "directory".into());
                visit(root, &path, out);
            } else {
                out.insert(
                    name,
                    format!("{:x}", Sha256::digest(fs::read(path).unwrap())),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}
fn fixture() -> (tempfile::TempDir, LibraryRoot) {
    let dir = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(dir.path().join("library"));
    let actor = DbActor::start(root.clone()).unwrap();
    actor.shutdown(Duration::from_secs(5)).unwrap();
    fs::remove_file(root.path().join(".writer.lock")).unwrap();
    (dir, root)
}
#[test]
fn future_without_lock_preserves_complete_root_inventory() {
    let (_dir, root) = fixture();
    let c = rusqlite::Connection::open(root.database()).unwrap();
    c.pragma_update(None, "user_version", 99).unwrap();
    drop(c);
    let before = inventory(root.path());
    let actor = DbActor::start(root.clone()).unwrap();
    assert!(!actor.info().writable);
    actor.shutdown(Duration::from_secs(5)).unwrap();
    assert_eq!(before, inventory(root.path()));
}
#[cfg(windows)]
struct ReadOnlyFixture(std::path::PathBuf);
#[cfg(windows)]
impl ReadOnlyFixture {
    fn apply(path: &std::path::Path) -> Self {
        assert!(path.starts_with(std::env::temp_dir()));
        assert!(
            std::process::Command::new("icacls")
                .arg(path)
                .args(["/deny", "*S-1-1-0:(OI)(CI)(WD,AD,WEA,WA,DE)", "/Q"])
                .output()
                .unwrap()
                .status
                .success()
        );
        Self(path.to_owned())
    }
}
#[cfg(windows)]
impl Drop for ReadOnlyFixture {
    fn drop(&mut self) {
        assert!(
            std::process::Command::new("icacls")
                .arg(&self.0)
                .args(["/remove:d", "*S-1-1-0", "/T", "/Q"])
                .output()
                .unwrap()
                .status
                .success()
        );
    }
}
#[test]
#[cfg(windows)]
fn future_wal_version_is_classified_without_readonly_root_writes() {
    let (_dir, root) = fixture();
    let c = rusqlite::Connection::open(root.database()).unwrap();
    c.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
    c.pragma_update(None, "user_version", 99).unwrap();
    assert!(
        fs::metadata(root.path().join("research.sqlite-wal"))
            .unwrap()
            .len()
            > 32
    );
    let readonly = ReadOnlyFixture::apply(root.path());
    assert!(fs::write(root.path().join("cannot-write"), b"x").is_err());
    let before = inventory(root.path());
    let actor = DbActor::start(root.clone()).unwrap();
    assert_eq!(actor.info().schema_version, 99);
    assert!(!actor.info().writable);
    actor.shutdown(Duration::from_secs(5)).unwrap();
    assert_eq!(before, inventory(root.path()));
    drop(readonly);
    drop(c);
}
#[test]
fn supported_pending_wal_recovers_with_normal_writer_exclusion() {
    let (_dir, root) = fixture();
    // Separate harness child exits without dropping SQLite, leaving a real committed WAL.
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "pending_wal_fixture_child", "--nocapture"])
        .env("RESEARCH_TEST_WAL_DATABASE", root.database())
        .status()
        .unwrap();
    assert!(status.success());
    assert!(
        fs::metadata(root.path().join("research.sqlite-wal"))
            .unwrap()
            .len()
            > 32
    );
    let actor = DbActor::start(root.clone()).unwrap();
    assert!(actor.info().writable);
    assert!(DbActor::start(root.clone()).is_err());
    assert!(
        tauri::async_runtime::block_on(actor.submit(|c| Ok(c.query_row(
            "SELECT value_json FROM app_settings WHERE key='wal-test'",
            [],
            |r| r.get::<_, String>(0)
        )?)))
        .unwrap()
            == "true"
    );
    actor.shutdown(Duration::from_secs(5)).unwrap();
}
#[test]
fn pending_wal_fixture_child() {
    let Some(path) = std::env::var_os("RESEARCH_TEST_WAL_DATABASE") else {
        return;
    };
    assert!(std::path::Path::new(&path).starts_with(std::env::temp_dir()));
    let c = rusqlite::Connection::open(path).unwrap();
    c.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
    c.execute("INSERT INTO app_settings VALUES('wal-test','true')", [])
        .unwrap();
    std::process::exit(0);
}

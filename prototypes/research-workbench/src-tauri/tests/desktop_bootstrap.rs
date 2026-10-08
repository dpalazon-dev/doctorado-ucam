use research_workbench_core::{
    adapters::{
        sqlite::{
            actor::DbActor,
            migrations::{Migration, migrate},
        },
        windows::{
            lock::LibraryLock,
            paths::{LibraryRoot, resolve_data_root},
        },
    },
    desktop::lifecycle::{LogEvent, SafeLogger},
    transport::{commands::authorize, error::ErrorCode},
};
#[test]
fn data_root_independent_of_cwd() {
    let a = resolve_data_root().unwrap();
    assert!(a.path().is_absolute());
    assert!(!a.path().starts_with(std::env::current_dir().unwrap()));
}
#[test]
fn second_writer_rejected() {
    let d = tempfile::tempdir().unwrap();
    let first = LibraryLock::acquire(d.path()).unwrap();
    let second = LibraryLock::acquire(d.path());
    assert!(second.is_err());
    drop(first);
    assert!(LibraryLock::acquire(d.path()).is_ok());
}
#[test]
fn command_outside_permission_group_rejected() {
    assert!(authorize("main", "arbitrary_sql").is_err());
    assert!(authorize("main", "settings_get_app_info").is_ok());
}
#[test]
fn other_window_command_rejected() {
    assert!(authorize("evil", "settings_get_app_info").is_err());
}
#[test]
fn schema_future_read_only_no_write() {
    let d = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(d.path().join("library"));
    let actor = DbActor::start(root.clone()).unwrap();
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
    let c = rusqlite::Connection::open(root.database()).unwrap();
    c.pragma_update(None, "user_version", 99).unwrap();
    drop(c);
    let before = std::fs::read(root.database()).unwrap();
    let actor = DbActor::start(root.clone()).unwrap();
    assert!(!actor.info().writable);
    let result = tauri::async_runtime::block_on(actor.submit(|c| {
        c.execute("INSERT INTO app_settings VALUES('bad','1')", [])?;
        Ok(())
    }));
    assert!(result.is_err());
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
    assert_eq!(before, std::fs::read(root.database()).unwrap());
}
#[test]
fn migration_failure_keeps_backup() {
    let d = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(d.path().join("library"));
    let actor = DbActor::start(root.clone()).unwrap();
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
    let mut c = rusqlite::Connection::open(root.database()).unwrap();
    let failed = migrate(
        &mut c,
        &root,
        &[Migration {
            version: research_workbench_core::adapters::sqlite::migrations::SCHEMA_VERSION + 1,
            sql: "CREATE TABLE partial(x); THIS IS NOT SQL;",
        }],
    )
    .unwrap_err();
    assert_eq!(failed.code, ErrorCode::MigrationFailed);
    let count: i64 = c
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='partial'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
    assert_eq!(
        c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        research_workbench_core::adapters::sqlite::migrations::SCHEMA_VERSION
    );
    assert!(
        std::fs::read_dir(root.backups().unwrap())
            .unwrap()
            .next()
            .is_some()
    );
}
#[test]
fn logs_exclude_body_and_source_path() {
    let d = tempfile::tempdir().unwrap();
    let log = SafeLogger::new(d.path().to_path_buf()).unwrap();
    for _ in 0..2000 {
        log.record(LogEvent::Started).unwrap();
    }
    log.record(LogEvent::Stopped).unwrap();
    let text = std::fs::read_to_string(d.path().join("desktop.log")).unwrap();
    assert!(!text.contains("source"));
    assert!(!text.contains("body"));
    assert!(!text.contains(&d.path().display().to_string()));
    assert!(text.contains("stopped"));
}
#[tokio::test]
async fn fts5_available_in_bundled_sqlite() {
    let d = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(d.path().join("library"));
    let actor = DbActor::start(root.clone()).unwrap();
    let id = actor.info().library_id.clone();
    actor.submit(|c|{c.execute_batch("CREATE VIRTUAL TABLE temp.probe USING fts5(text, tokenize='unicode61 remove_diacritics 2');INSERT INTO probe VALUES('résumé');")?;assert_eq!(c.query_row("SELECT count(*) FROM probe WHERE probe MATCH 'resume'",[],|r|r.get::<_,i64>(0))?,1);assert_eq!(c.query_row("PRAGMA quick_check",[],|r|r.get::<_,String>(0))?,"ok");assert!(!c.prepare("PRAGMA foreign_key_check")?.exists([])?);Ok(())}).await.unwrap();
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
    let reopened = DbActor::start(root).unwrap();
    assert_eq!(id, reopened.info().library_id);
}

fn acl_app() -> (tempfile::TempDir, tauri::App<tauri::test::MockRuntime>) {
    let d = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(d.path().join("library"));
    let actor = DbActor::start(root).unwrap();
    let state = research_workbench_core::desktop::lifecycle::DesktopState {
        settings: std::sync::Arc::new(
            research_workbench_core::adapters::sqlite::settings::ActorSettingsQuery::new(
                actor.clone(),
                Default::default(),
            ),
        ),
        actor,
        maintenance: Default::default(),
        logger: std::sync::Arc::new(SafeLogger::new(d.path().join("logs")).unwrap()),
        exit: Default::default(),
        library: std::sync::Mutex::new(None),
        reader: std::sync::Mutex::new(None),
        workflow: std::sync::Mutex::new(None),
    };
    let app = tauri::test::mock_builder()
        .manage(state)
        .invoke_handler(research_workbench_core::handle_invoke)
        .build(tauri::generate_context!())
        .unwrap();
    (d, app)
}
fn invoke_settings(
    window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    command: &str,
) -> Result<serde_json::Value, serde_json::Value> {
    invoke_command(
        window,
        command,
        serde_json::json!({"args":{"requestId":"00000000-0000-4000-8000-000000000001"}}),
    )
}
fn invoke_command(
    window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    command: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    tauri::test::get_ipc_response(
        window,
        tauri::webview::InvokeRequest {
            cmd: command.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: "http://tauri.localhost".parse().unwrap(),
            body: tauri::ipc::InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    )
    .map(|b| b.deserialize().unwrap())
}
#[test]
fn tauri_runtime_authority_allows_main_settings_and_denies_other_window() {
    let (_d, app) = acl_app();
    let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let other = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
        .build()
        .unwrap();
    let allowed = invoke_settings(&main, "settings_get_app_info").unwrap();
    assert_eq!(allowed["ok"], true);
    let denied = invoke_settings(&other, "settings_get_app_info");
    assert!(denied.is_err() || denied.unwrap()["ok"] == false);
    let outside = invoke_settings(&main, "arbitrary_sql");
    assert!(outside.is_err() || outside.unwrap()["ok"] == false);
}

#[test]
fn registered_dialog_plugin_stays_outside_main_local_capability_while_library_works() {
    use research_workbench_core::{
        adapters::{
            documents::store::LocalDocumentStore,
            sqlite::library_repository::SqliteLibraryPersistence,
        },
        application::library_ports::NativePdfSelection,
        desktop::maintenance::MaintenanceCoordinator,
        modules::library::service::LibraryService,
    };
    use std::sync::{Arc, Mutex};

    struct EmptyPicker;
    impl NativePdfSelection for EmptyPicker {
        fn select_pdf(
            &self,
        ) -> research_workbench_core::application::library_ports::LibraryFuture<
            '_,
            Option<research_workbench_core::application::library_ports::SelectedPdf>,
        > {
            Box::pin(async { Ok(None) })
        }
    }

    let directory = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(directory.path().join("library"))).unwrap();
    let maintenance = MaintenanceCoordinator::default();
    let library_id = actor.info().library_id.0.clone();
    let service = Arc::new(LibraryService::new(
        Arc::new(EmptyPicker),
        Arc::new(
            LocalDocumentStore::new(actor.library_root().clone(), library_id.clone()).unwrap(),
        ),
        Arc::new(SqliteLibraryPersistence::new(actor.clone(), library_id)),
        maintenance.clone(),
        research_workbench_core::application::request_registry::RequestRegistry::default(),
    ));
    let state = research_workbench_core::desktop::lifecycle::DesktopState {
        settings: Arc::new(
            research_workbench_core::adapters::sqlite::settings::ActorSettingsQuery::new(
                actor.clone(),
                maintenance.clone(),
            ),
        ),
        actor: actor.clone(),
        maintenance,
        logger: Arc::new(SafeLogger::new(directory.path().join("logs")).unwrap()),
        exit: Default::default(),
        library: Mutex::new(Some(service)),
        reader: std::sync::Mutex::new(None),
        workflow: std::sync::Mutex::new(None),
    };
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_dialog::init::<tauri::test::MockRuntime>())
        .manage(state)
        .invoke_handler(research_workbench_core::handle_invoke)
        .build(tauri::generate_context!())
        .unwrap();
    let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let library_result = invoke_command(
        &main,
        "library_select_pdf",
        serde_json::json!({"args":{"requestId":"00000000-0000-4000-8000-000000000011"}}),
    )
    .unwrap();
    assert_eq!(library_result["ok"], true);
    assert!(library_result["data"].is_null());

    let dialog_open = invoke_command(
        &main,
        "plugin:dialog|open",
        serde_json::json!({"options":{}}),
    );
    let dialog_error = dialog_open.unwrap_err().to_string();
    assert!(dialog_error.contains("dialog.open not allowed"));
    assert!(dialog_error.contains("dialog:allow-open"));
    let generic_file_read = invoke_command(
        &main,
        "plugin:fs|read_file",
        serde_json::json!({"path":"research.sqlite"}),
    );
    let fs_error = generic_file_read.unwrap_err().to_string();
    assert!(fs_error.contains("fs.read_file not allowed"));
    assert!(fs_error.contains("Plugin not found"));
    actor.shutdown(std::time::Duration::from_secs(2)).unwrap();
}

fn snapshot_fixture() -> (tempfile::TempDir, LibraryRoot, rusqlite::Connection, String) {
    use sha2::{Digest, Sha256};
    let d = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(d.path().join("library"));
    let actor = DbActor::start(root.clone()).unwrap();
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
    let c = rusqlite::Connection::open(root.database()).unwrap();
    c.pragma_update(None, "foreign_keys", true).unwrap();
    let paper = uuid::Uuid::new_v4().to_string();
    let doc = uuid::Uuid::new_v4().to_string();
    let relative = format!("documents/{doc}/original.pdf");
    let bytes = b"%PDF-1.7\nsynthetic test\n";
    let hash = format!("{:x}", Sha256::digest(bytes));
    std::fs::create_dir_all(root.path().join("documents").join(&doc)).unwrap();
    std::fs::write(root.path().join(&relative), bytes).unwrap();
    c.execute("INSERT INTO papers(id,title,review_type,created_at,updated_at) VALUES(?1,'Synthetic','survey','2026-10-01T12:00:00Z','2026-10-01T12:00:00Z')",[&paper]).unwrap();
    c.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,'synthetic.pdf',?3,?4,'application/pdf',?5,'2026-10-01T12:00:00Z','ACTIVE')",rusqlite::params![doc,paper,relative,hash,bytes.len() as i64]).unwrap();
    (d, root, c, relative)
}
#[test]
fn snapshot_has_verified_resource_hash_manifest() {
    let (_d, root, c, relative) = snapshot_fixture();
    assert!(c.is_autocommit());
    let backup =
        research_workbench_core::adapters::sqlite::migrations::snapshot(&c, &root).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(backup.join("manifest.json")).unwrap()).unwrap();
    let files = manifest["files"].as_array().unwrap();
    assert!(files.iter().any(|f| f["path"] == relative));
    assert!(files.iter().any(|f| f["path"] == "research.sqlite"));
    assert!(files.iter().any(|f| f["path"] == "library.json"));
    assert_eq!(manifest["verified"], true);
    assert!(c.is_autocommit());
    let saved = rusqlite::Connection::open(backup.join("research.sqlite")).unwrap();
    assert_eq!(
        saved
            .query_row("SELECT count(*) FROM documents", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}
#[test]
fn snapshot_rejects_changed_referenced_pdf() {
    let (_d, root, c, relative) = snapshot_fixture();
    std::fs::write(root.path().join(relative), b"externally changed").unwrap();
    let error =
        research_workbench_core::adapters::sqlite::migrations::snapshot(&c, &root).unwrap_err();
    assert_eq!(error.code, ErrorCode::BackupFailed);
}

#[test]
fn startup_diagnostics_are_classified_and_safe() {
    use research_workbench_core::adapters::windows::startup::diagnostic_message;
    use research_workbench_core::transport::error::AppError;
    let busy = diagnostic_message(&AppError::new(ErrorCode::Busy));
    let storage = diagnostic_message(&AppError::new(ErrorCode::StorageUnavailable));
    assert!(busy.1.to_lowercase().contains("another instance"));
    assert!(storage.1.contains("library storage"));
    assert_ne!(busy, storage);
    assert!(!busy.1.contains('\\'));
    assert!(!storage.1.contains("C:"));
}

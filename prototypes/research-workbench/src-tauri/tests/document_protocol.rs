use research_workbench_core::{
    adapters::{
        documents::store::LocalDocumentStore,
        sqlite::{actor::DbActor, reader_repository::SqliteReaderPersistence},
        windows::paths::LibraryRoot,
    },
    application::reader_ports::{DocumentReadAccess, ReaderPersistence},
    application::{request_registry::RequestRegistry, settings::RecoveryStatus},
    desktop::maintenance::MaintenanceCoordinator,
    modules::reader::{protocol::document_id_from_request, service::ReaderService},
    transport::{dto::UUID, error::ErrorCode},
};
use std::{fs, process::Command, sync::Arc};

fn fixture() -> (
    tempfile::TempDir,
    DbActor,
    SqliteReaderPersistence,
    Arc<LocalDocumentStore>,
    UUID,
) {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let library_id = actor.info().library_id.clone();
    let paper_id = uuid::Uuid::new_v4().to_string();
    let document_id = uuid::Uuid::new_v4().to_string();
    let relative = format!("documents/{document_id}/original.pdf");
    let bytes = b"%PDF-1.4 synthetic protocol fixture";
    let path = actor.library_root().path().join(&relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    let (paper, doc, rel, hash) = (
        paper_id,
        document_id.clone(),
        relative,
        format!("{:x}", sha2::Sha256::digest(bytes)),
    );
    tauri::async_runtime::block_on(actor.submit(move|c|{
        let tx=c.transaction()?;
        tx.execute("INSERT INTO papers(id,title,year,review_type,lifecycle,revision,processing_initialized,active_document_id,created_at,updated_at) VALUES(?1,'Synthetic',2024,'unknown','NEW',0,0,?2,'2026-10-02T00:00:00Z','2026-10-02T00:00:00Z')",rusqlite::params![paper,doc])?;
        tx.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,'synthetic.pdf',?3,?4,'application/pdf',?5,'2026-10-02T00:00:00Z','ACTIVE')",rusqlite::params![doc,paper,rel,hash,bytes.len() as i64])?;
        tx.commit()?;Ok(())
    })).unwrap();
    let store = Arc::new(
        LocalDocumentStore::new(actor.library_root().clone(), library_id.0.clone()).unwrap(),
    );
    let persistence = SqliteReaderPersistence::new(actor.clone(), library_id.0);
    (dir, actor, persistence, store, UUID(document_id))
}

#[test]
fn protocol_unknown_uuid_rejected() {
    let (_dir, _actor, persistence, _store, _document) = fixture();
    let unknown = UUID(uuid::Uuid::new_v4().to_string());
    assert_eq!(
        tauri::async_runtime::block_on(persistence.registered_document(unknown))
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
}

#[test]
fn protocol_traversal_rejected() {
    let uri: tauri::http::Uri = "research://localhost/%2e%2e/00000000-0000-4000-8000-000000000001"
        .parse()
        .unwrap();
    assert_eq!(
        document_id_from_request(
            &uri,
            "main",
            &tauri::http::Method::GET,
            Some("http://tauri.localhost")
        )
        .unwrap_err(),
        ErrorCode::PathNotAllowed
    );
    let with_port: tauri::http::Uri =
        "research://localhost:8080/00000000-0000-4000-8000-000000000001"
            .parse()
            .unwrap();
    assert_eq!(
        document_id_from_request(
            &with_port,
            "main",
            &tauri::http::Method::GET,
            Some("http://tauri.localhost")
        )
        .unwrap_err(),
        ErrorCode::PathNotAllowed
    );
}

#[test]
fn protocol_junction_escape_rejected() {
    let (_dir, actor, persistence, store, document_id) = fixture();
    let registered =
        tauri::async_runtime::block_on(persistence.registered_document(document_id.clone()))
            .unwrap();
    let managed_dir = actor
        .library_root()
        .path()
        .join(format!("documents/{}", document_id.0));
    let source = actor
        .library_root()
        .path()
        .join(format!("documents/{}.held", document_id.0));
    let outside = tempfile::tempdir().unwrap();
    fs::remove_file(managed_dir.join("original.pdf")).unwrap();
    fs::rename(&managed_dir, &source).unwrap();
    fs::write(
        outside.path().join("original.pdf"),
        b"outside synthetic PDF",
    )
    .unwrap();
    let command = format!(
        "New-Item -ItemType Junction -Path '{}' -Target '{}' | Out-Null",
        managed_dir.display(),
        outside.path().display()
    );
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &command])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "creating a synthetic NTFS junction must succeed to exercise the boundary"
    );
    let result = tauri::async_runtime::block_on(store.open_verified(registered));
    assert!(
        result.is_err(),
        "a document directory junction must not escape the managed root"
    );
    assert_eq!(
        fs::read(outside.path().join("original.pdf")).unwrap(),
        b"outside synthetic PDF"
    );
    fs::remove_dir(&managed_dir).unwrap();
    fs::rename(source, &managed_dir).unwrap();
}

#[test]
fn protocol_reads_body_from_registered_handle_and_holds_operation_through_response() {
    let (_dir, actor, persistence, store, document_id) = fixture();
    let maintenance = MaintenanceCoordinator::default();
    let service = ReaderService::new(
        Arc::new(persistence),
        store,
        maintenance.clone(),
        RequestRegistry::default(),
        RecoveryStatus::default(),
    );
    let read = tauri::async_runtime::block_on(service.read_document(document_id)).unwrap();
    let expected = b"%PDF-1.4 synthetic protocol fixture";
    assert_eq!(read.body.bytes, expected);
    assert_eq!(maintenance.active_operations(), 1);
    assert_eq!(read.body.bytes.len(), expected.len());
    drop(read);
    maintenance
        .wait_for_idle(std::time::Duration::from_secs(2))
        .unwrap();
    assert_eq!(maintenance.active_operations(), 0);
    actor.shutdown(std::time::Duration::from_secs(3)).unwrap();
}

use sha2::Digest;

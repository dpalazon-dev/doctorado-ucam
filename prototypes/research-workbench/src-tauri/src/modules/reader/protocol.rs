use crate::{
    desktop::lifecycle::DesktopState,
    transport::{dto::UUID, error::ErrorCode},
};
use tauri::{
    Manager,
    http::{HeaderValue, Method, Request, Response, StatusCode, Uri, header},
};

const MAX_PDF_BYTES: usize = 524_288_000;
pub(crate) fn full_pdf_response(
    bytes: Vec<u8>,
    origin: &str,
) -> Result<Response<Vec<u8>>, ErrorCode> {
    if bytes.len() > MAX_PDF_BYTES {
        return Err(ErrorCode::InvalidInput);
    }
    let length = bytes.len();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_LENGTH, length.to_string())
        .header(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_str(origin).map_err(|_| ErrorCode::PathNotAllowed)?,
        )
        .header(header::VARY, "Origin")
        .body(bytes)
        .map_err(|_| ErrorCode::IntegrityFailure)
}

fn deliver_response<L, O, F: FnOnce(Response<Vec<u8>>)>(
    response: Response<Vec<u8>>,
    lease: Option<L>,
    operation: Option<O>,
    respond: F,
) {
    respond(response);
    drop(lease);
    drop(operation);
}
fn allowed_origin(origin: &str) -> bool {
    origin == "http://tauri.localhost"
        || (cfg!(debug_assertions) && origin == "http://localhost:1420")
}

async fn serve_pdf_read<F: FnOnce(Response<Vec<u8>>)>(
    read: Result<crate::modules::reader::service::ReadDocument, crate::transport::error::AppError>,
    origin: &str,
    respond: F,
) {
    let mut held_lease = None;
    let mut held_operation = None;
    let response = match read {
        Ok(read) if read.body.bytes.len() <= MAX_PDF_BYTES => {
            let crate::modules::reader::service::ReadDocument { body, permit } = read;
            let response = full_pdf_response(body.bytes, origin).unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Vec::new())
                    .unwrap()
            });
            held_lease = Some(body.lease);
            held_operation = Some(permit);
            response
        }
        Ok(_) => Response::builder()
            .status(StatusCode::PAYLOAD_TOO_LARGE)
            .body(Vec::new())
            .unwrap(),
        Err(error) => Response::builder()
            .status(match error.code {
                ErrorCode::NotFound => StatusCode::NOT_FOUND,
                ErrorCode::PathNotAllowed => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
            .body(Vec::new())
            .unwrap(),
    };
    deliver_response(response, held_lease, held_operation, respond);
}

pub fn document_id_from_request(
    uri: &Uri,
    window: &str,
    method: &Method,
    origin: Option<&str>,
) -> Result<(UUID, String), ErrorCode> {
    if window != "main" || method != Method::GET {
        return Err(ErrorCode::PathNotAllowed);
    }
    let origin = origin
        .filter(|o| allowed_origin(o))
        .ok_or(ErrorCode::PathNotAllowed)?;
    if uri.scheme_str() != Some("research")
        || uri.authority().map(|a| a.as_str()) != Some("localhost")
        || uri.query().is_some()
        || uri.path().contains('%')
    {
        return Err(ErrorCode::PathNotAllowed);
    }
    let Some(id) = uri.path().strip_prefix('/') else {
        return Err(ErrorCode::PathNotAllowed);
    };
    if id.contains('/') || id.is_empty() {
        return Err(ErrorCode::PathNotAllowed);
    }
    let parsed = uuid::Uuid::parse_str(id).map_err(|_| ErrorCode::PathNotAllowed)?;
    if parsed.to_string() != id {
        return Err(ErrorCode::PathNotAllowed);
    }
    Ok((UUID(id.to_owned()), origin.to_owned()))
}

pub fn register<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.register_asynchronous_uri_scheme_protocol(
        "research",
        |ctx, request: Request<Vec<u8>>, responder| {
            let window = ctx.webview_label().to_owned();
            let parsed = document_id_from_request(
                request.uri(),
                &window,
                request.method(),
                request
                    .headers()
                    .get(header::ORIGIN)
                    .and_then(|v| v.to_str().ok()),
            );
            let app = ctx.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                match parsed {
                    Err(_) => {
                        deliver_response(
                            Response::builder()
                                .status(StatusCode::FORBIDDEN)
                                .body(Vec::new())
                                .unwrap(),
                            None::<()>,
                            None::<()>,
                            |response| responder.respond(response),
                        );
                    }
                    Ok((document_id, origin)) => {
                        let read = async {
                            let state = app.state::<DesktopState>();
                            state.reader()?.read_document(document_id).await
                        }
                        .await;
                        serve_pdf_read(read, &origin, |response| responder.respond(response)).await;
                    }
                }
            });
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    struct DropProbe(Arc<AtomicBool>);
    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn protocol_accepts_only_registered_shape_and_authorized_origin_context() {
        let id = "00000000-0000-4000-8000-000000000001";
        let uri: Uri = format!("research://localhost/{id}").parse().unwrap();
        assert_eq!(
            document_id_from_request(&uri, "main", &Method::GET, Some("http://tauri.localhost"))
                .unwrap()
                .0
                .0,
            id
        );
        for invalid in [
            format!("research://localhost/{id}/x"),
            format!("research://localhost/%2e%2e/{id}"),
            format!("research://localhost/{id}?download=1"),
            "research://remote/00000000-0000-4000-8000-000000000001".to_owned(),
        ] {
            let uri: Uri = invalid.parse().unwrap();
            assert!(
                document_id_from_request(
                    &uri,
                    "main",
                    &Method::GET,
                    Some("http://tauri.localhost")
                )
                .is_err()
            );
        }
        assert!(
            document_id_from_request(
                &format!("research://localhost/{id}").parse().unwrap(),
                "other",
                &Method::GET,
                Some("http://tauri.localhost")
            )
            .is_err()
        );
        assert!(
            document_id_from_request(
                &format!("research://localhost/{id}").parse().unwrap(),
                "main",
                &Method::POST,
                Some("http://tauri.localhost")
            )
            .is_err()
        );
        assert!(
            document_id_from_request(
                &format!("research://localhost/{id}").parse().unwrap(),
                "main",
                &Method::GET,
                Some("http://evil.localhost")
            )
            .is_err()
        );
    }

    #[test]
    fn complete_pdf_response_is_delivered_before_lease_and_operation_release() {
        let maintenance = crate::desktop::maintenance::MaintenanceCoordinator::default();
        let operation = maintenance.begin_operation().unwrap();
        let dropped = Arc::new(AtomicBool::new(false));
        let lease = DropProbe(dropped.clone());
        let bytes = b"%PDF-synthetic".to_vec();
        let response = full_pdf_response(bytes.clone(), "http://tauri.localhost").unwrap();
        deliver_response(response, Some(lease), Some(operation), |response| {
            assert_eq!(maintenance.active_operations(), 1);
            assert!(!dropped.load(Ordering::SeqCst));
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.body(), &bytes);
            assert_eq!(
                response.headers()[header::CONTENT_LENGTH],
                bytes.len().to_string()
            );
            assert_eq!(response.headers()[header::CONTENT_TYPE], "application/pdf");
            assert!(response.headers().get(header::ACCEPT_RANGES).is_none());
        });
        assert!(dropped.load(Ordering::SeqCst));
        assert_eq!(maintenance.active_operations(), 0);
    }

    #[test]
    fn dropped_protocol_receiver_keeps_read_operation_until_blocking_body_read_finishes() {
        use crate::{
            adapters::{
                documents::store::{LocalDocumentStore, ReadGate},
                sqlite::{actor::DbActor, reader_repository::SqliteReaderPersistence},
                windows::paths::LibraryRoot,
            },
            application::{request_registry::RequestRegistry, settings::RecoveryStatus},
            desktop::maintenance::MaintenanceCoordinator,
            modules::reader::service::ReaderService,
        };
        use sha2::{Digest, Sha256};
        use std::{fs, sync::mpsc};

        let temp = tempfile::tempdir().unwrap();
        let actor = DbActor::start(LibraryRoot::at(temp.path().join("library"))).unwrap();
        let library_id = actor.info().library_id.0.clone();
        let paper_id = "00000000-0000-4000-8000-000000000151".to_owned();
        let document_id = "00000000-0000-4000-8000-000000000152".to_owned();
        let relative = format!("documents/{document_id}/original.pdf");
        let bytes = b"%PDF-1.4 synthetic body".to_vec();
        let path = actor.library_root().path().join(&relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &bytes).unwrap();
        let hash = format!("{:x}", Sha256::digest(&bytes));
        let paper_for_db = paper_id.clone();
        let document_for_db = document_id.clone();
        let relative_for_db = relative.clone();
        let hash_for_db = hash.clone();
        tauri::async_runtime::block_on(actor.submit(move |connection| {
            let tx = connection.transaction()?;
            tx.execute("INSERT INTO papers(id,title,year,review_type,lifecycle,revision,processing_initialized,active_document_id,created_at,updated_at) VALUES(?1,'Protocolo sintético',2026,'unknown','NEW',0,0,?2,'2026-10-03T00:00:00Z','2026-10-03T00:00:00Z')", rusqlite::params![paper_for_db, document_for_db])?;
            tx.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,'synthetic.pdf',?3,?4,'application/pdf',?5,'2026-10-03T00:00:00Z','ACTIVE')", rusqlite::params![document_for_db, paper_for_db, relative_for_db, hash_for_db, bytes.len() as i64])?;
            tx.commit()?;
            Ok(())
        })).unwrap();

        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let store = Arc::new(
            LocalDocumentStore::new(actor.library_root().clone(), library_id.clone()).unwrap(),
        );
        store.set_read_gate(Arc::new(ReadGate {
            entered: entered_tx,
            release: std::sync::Mutex::new(release_rx),
        }));
        let maintenance = MaintenanceCoordinator::default();
        let reader = ReaderService::new(
            Arc::new(SqliteReaderPersistence::new(actor.clone(), library_id)),
            store,
            maintenance.clone(),
            RequestRegistry::default(),
            RecoveryStatus::default(),
        );
        let (response_tx, response_rx) = mpsc::channel();
        let callback_called = Arc::new(AtomicBool::new(false));
        let callback_seen = callback_called.clone();
        let operation = tauri::async_runtime::spawn(async move {
            let read = reader
                .read_document(crate::transport::dto::UUID(document_id))
                .await;
            serve_pdf_read(read, "http://tauri.localhost", move |response| {
                callback_seen.store(true, Ordering::SeqCst);
                assert_eq!(response.status(), StatusCode::OK);
                let _ = response_tx.send(response);
            })
            .await;
        });
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(3))
            .unwrap();
        drop(response_rx);
        assert_eq!(
            maintenance.active_operations(),
            1,
            "the production protocol read owns its operation while spawn_blocking is paused inside the actual managed file read"
        );
        assert!(maintenance.begin_maintenance().is_err());
        release_tx.send(()).unwrap();
        tauri::async_runtime::block_on(operation).unwrap();
        assert!(
            callback_called.load(Ordering::SeqCst),
            "the asynchronous responder is attempted after the real file read completes even when its receiver was dropped"
        );
        maintenance
            .wait_for_idle(std::time::Duration::from_secs(3))
            .unwrap();
        assert_eq!(maintenance.active_operations(), 0);
        actor.shutdown(std::time::Duration::from_secs(3)).unwrap();
    }
}

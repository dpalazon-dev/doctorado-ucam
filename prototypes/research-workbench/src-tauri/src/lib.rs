pub mod adapters;
pub mod application;
pub mod desktop;
pub mod domain;
pub mod modules;
pub mod transport;
use tauri::Manager;
pub fn run() -> Result<(), transport::error::AppError> {
    let recovery_status = application::settings::RecoveryStatus::pending();
    let settings_recovery = recovery_status.clone();
    let state = desktop::lifecycle::DesktopState::start(move |actor, maintenance| {
        std::sync::Arc::new(
            adapters::sqlite::settings::ActorSettingsQuery::with_recovery(
                actor,
                maintenance,
                settings_recovery,
            ),
        )
    })?;
    let app = modules::reader::protocol::register(tauri::Builder::default())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(handle_invoke)
        .build(tauri::generate_context!())
        .map_err(|_| {
            transport::error::AppError::new(transport::error::ErrorCode::StorageUnavailable)
        })?;
    let desktop = app.state::<desktop::lifecycle::DesktopState>();
    let picker = std::sync::Arc::new(adapters::windows::pdf_picker::WindowsPdfPicker::new(
        app.handle().clone(),
    ));
    let root = desktop.actor.library_root().clone();
    let documents = std::sync::Arc::new(adapters::documents::store::LocalDocumentStore::new(
        root,
        desktop.actor.info().library_id.0.clone(),
    )?);
    let persistence = std::sync::Arc::new(
        adapters::sqlite::library_repository::SqliteLibraryPersistence::new(
            desktop.actor.clone(),
            desktop.actor.info().library_id.0.clone(),
        ),
    );
    let request_registry = application::request_registry::RequestRegistry::default();
    let library = std::sync::Arc::new(
        modules::library::service::LibraryService::new(
            picker,
            documents.clone(),
            persistence,
            desktop.maintenance.clone(),
            request_registry.clone(),
        )
        .with_recovery_status(recovery_status.clone()),
    );
    let reader_persistence = std::sync::Arc::new(
        adapters::sqlite::reader_repository::SqliteReaderPersistence::new(
            desktop.actor.clone(),
            desktop.actor.info().library_id.0.clone(),
        ),
    );
    let reader = std::sync::Arc::new(modules::reader::service::ReaderService::new(
        reader_persistence,
        documents.clone(),
        desktop.maintenance.clone(),
        request_registry.clone(),
        recovery_status.clone(),
    ));
    let workflow_persistence = std::sync::Arc::new(
        adapters::sqlite::workflow_repository::SqliteWorkflowPersistence::new(
            desktop.actor.clone(),
        ),
    );
    let workflow = std::sync::Arc::new(modules::workflow::service::WorkflowService::new(
        workflow_persistence,
        desktop.maintenance.clone(),
        recovery_status.clone(),
        request_registry,
        documents,
    ));
    let recovery_permit = if desktop.actor.info().writable {
        Some(desktop.maintenance.begin_maintenance()?)
    } else {
        None
    };
    desktop.register_library(library.clone())?;
    desktop.register_reader(reader)?;
    desktop.register_workflow(workflow)?;
    if let Some(recovery_permit) = recovery_permit {
        tauri::async_runtime::spawn(async move {
            let result = library.reconcile_imports_with_permit(recovery_permit).await;
            recovery_status.finish(result);
        });
    } else {
        recovery_status.finish(Ok(application::library_ports::RecoveryReport {
            recovered: 0,
            issues: Vec::new(),
        }));
    }
    app.run(|handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } if !allow_desktop_exit(handle) => {
            api.prevent_exit();
        }
        tauri::RunEvent::WindowEvent {
            event: tauri::WindowEvent::CloseRequested { api, .. },
            ..
        } if !allow_desktop_exit(handle) => {
            api.prevent_close();
        }
        _ => {}
    });
    Ok(())
}
fn allow_desktop_exit<R: tauri::Runtime>(handle: &tauri::AppHandle<R>) -> bool {
    let Some(state) = handle.try_state::<desktop::lifecycle::DesktopState>() else {
        return false;
    };
    let app = handle.clone();
    state.request_exit(
        move || app.exit(0),
        adapters::windows::startup::show_shutdown_waiting,
    )
}
pub fn handle_invoke<R: tauri::Runtime>(invoke: tauri::ipc::Invoke<R>) -> bool {
    let command = invoke.message.command().to_owned();
    let window = invoke.message.webview_ref().label().to_owned();
    let payload = match invoke.message.payload() {
        tauri::ipc::InvokeBody::Json(value) => value.clone(),
        _ => serde_json::Value::Null,
    };
    let args = if payload
        .as_object()
        .is_some_and(|m| m.len() == 1 && m.contains_key("args"))
    {
        payload
            .get("args")
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };
    let app = invoke.message.webview().app_handle().clone();
    let authorized = invoke.acl.is_some();
    invoke.resolver.respond_async(async move {
        let state = app.state::<desktop::lifecycle::DesktopState>();
        let result = transport::commands::dispatch(
            if authorized { &window } else { "unauthorized" },
            &command,
            args,
            &state,
        )
        .await;
        Ok(result)
    });
    true
}

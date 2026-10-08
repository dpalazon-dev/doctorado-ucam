use research_workbench_core::{
    application::settings::{LibraryStatus, LibrarySummary, SettingsQuery},
    modules::settings,
    transport::error::AppError,
};
use std::{future::Future, pin::Pin};
struct FakeSettings;
impl SettingsQuery for FakeSettings {
    fn library_info(&self) -> LibrarySummary {
        LibrarySummary {
            library_id: "00000000-0000-4000-8000-000000000001".into(),
            display_name: "Prueba".into(),
            root_label: "Local".into(),
            schema_version: 99,
            writable: false,
        }
    }
    fn library_status(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<LibraryStatus, AppError>> + Send + '_>> {
        Box::pin(async {
            Ok(LibraryStatus {
                writable: false,
                active_operations: 2,
                recovery_required: true,
            })
        })
    }
}
#[test]
fn settings_service_uses_only_supplied_query_port() {
    let port = FakeSettings;
    let app = settings::get_app_info(&port).unwrap();
    assert!(!app.library.writable);
    assert_eq!(app.library.schema_version, 99);
    assert!(app.capabilities["settings"]);
    assert!(app.capabilities["library"]);
    assert!(app.capabilities["reader"]);
    assert!(app.capabilities["workflow"]);
    let status = tauri::async_runtime::block_on(settings::get_library_status(&port)).unwrap();
    assert!(status.recovery_required);
    assert_eq!(status.active_operations, 2);
}

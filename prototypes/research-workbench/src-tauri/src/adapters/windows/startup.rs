use crate::transport::error::{AppError, ErrorCode};
/// Static diagnostic text avoids leaking setup paths or Tauri error internals.
pub fn diagnostic_message(error: &AppError) -> (&'static str, &'static str) {
    match error.code {
        ErrorCode::Busy => (
            "Biblioteca en uso",
            "Otra instancia de Research Workbench ya utiliza esta biblioteca. Cierra la otra instancia y vuelve a intentarlo.",
        ),
        ErrorCode::StorageUnavailable => (
            "No se pudo abrir la biblioteca",
            "No se pudo acceder al almacenamiento de la biblioteca. Comprueba el espacio y los permisos y vuelve a intentarlo.",
        ),
        _ => ("No se pudo iniciar Research Workbench", error.message()),
    }
}
#[cfg(windows)]
pub fn show_error(error: &AppError) {
    let (title, message) = diagnostic_message(error);
    show_message(title, message);
}
#[cfg(windows)]
fn show_message(title: &str, message: &str) {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(
            owner: *mut std::ffi::c_void,
            text: *const u16,
            caption: *const u16,
            flags: u32,
        ) -> i32;
    }
    let title: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
    let message: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    // SAFETY: both static strings are NUL-terminated UTF-16, remain alive for the call,
    // and a null owner is valid for this process's diagnostic dialog.
    unsafe {
        MessageBoxW(std::ptr::null_mut(), message.as_ptr(), title.as_ptr(), 0x10);
    }
}
#[cfg(not(windows))]
pub fn show_error(error: &AppError) {
    eprintln!("{}", diagnostic_message(error).1);
}
#[cfg(not(windows))]
fn show_message(_title: &str, message: &str) {
    eprintln!("{message}");
}
pub fn show_shutdown_waiting(error: AppError) {
    if error.code == crate::transport::error::ErrorCode::Busy {
        show_message(
            "Cierre pendiente",
            "Hay operaciones pendientes. La aplicación conservará la biblioteca abierta hasta que terminen. No se ha completado el cierre.",
        );
    } else {
        show_error(&error);
    }
}

use crate::{
    application::library_ports::{LibraryFuture, NativePdfSelection, SelectedPdf},
    transport::error::{AppError, ErrorCode},
};
use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;

fn send_selection(
    sender: tokio::sync::oneshot::Sender<Option<SelectedPdf>>,
    picked: Option<SelectedPdf>,
) {
    let _ = sender.send(picked);
}

#[derive(Clone)]
pub struct WindowsPdfPicker {
    app: tauri::AppHandle,
}
impl WindowsPdfPicker {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}
impl NativePdfSelection for WindowsPdfPicker {
    fn select_pdf(&self) -> LibraryFuture<'_, Option<SelectedPdf>> {
        let app = self.app.clone();
        Box::pin(async move {
            let (tx, rx) = tokio::sync::oneshot::channel();
            app.dialog()
                .file()
                .add_filter("Documento PDF", &["pdf"])
                .pick_file(move |selected| {
                    let picked =
                        selected
                            .and_then(|file| file.into_path().ok())
                            .map(|path: PathBuf| {
                                let filename = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("documento.pdf")
                                    .to_owned();
                                SelectedPdf { path, filename }
                            });
                    send_selection(tx, picked);
                });
            rx.await
                .map_err(|_| AppError::new(ErrorCode::OperationCancelled))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_bridge_delivers_selected_path_and_cancellation() {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let selected = SelectedPdf {
            path: PathBuf::from("synthetic.pdf"),
            filename: "synthetic.pdf".into(),
        };
        send_selection(tx, Some(selected.clone()));
        assert_eq!(
            tauri::async_runtime::block_on(rx).unwrap().unwrap().path,
            selected.path
        );

        let (tx, rx) = tokio::sync::oneshot::channel();
        send_selection(tx, None);
        assert!(tauri::async_runtime::block_on(rx).unwrap().is_none());
    }
}

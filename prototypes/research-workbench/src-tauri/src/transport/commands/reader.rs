use crate::{
    application::reader_ports::SaveReadingPositionCommand,
    desktop::lifecycle::DesktopState,
    transport::{dto::*, error::AppError},
};

pub async fn open_paper(
    state: &DesktopState,
    args: ReaderOpenPaperArgs,
) -> Result<OpenPaperDto, AppError> {
    state
        .reader()?
        .open_paper(args.request_id, args.paper_id)
        .await
}
pub async fn last_opened(
    state: &DesktopState,
    _args: ReaderGetLastOpenedPaperArgs,
) -> Result<Option<PaperDto>, AppError> {
    state.reader()?.get_last_opened_paper().await
}
pub async fn reading_position(
    state: &DesktopState,
    args: ReaderGetReadingPositionArgs,
) -> Result<ReadingPositionDto, AppError> {
    state.reader()?.get_reading_position(args.document_id).await
}
pub async fn save_position(
    state: &DesktopState,
    args: ReaderSaveReadingPositionArgs,
) -> Result<ReadingPositionDto, AppError> {
    state
        .reader()?
        .save_reading_position(SaveReadingPositionCommand {
            request_id: args.request_id,
            document_id: args.document_id,
            expected_revision: args.expected_revision,
            page_index: args.page_index,
            zoom: args.zoom,
        })
        .await
}

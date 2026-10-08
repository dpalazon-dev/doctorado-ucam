use crate::{
    desktop::lifecycle::DesktopState,
    transport::{dto::*, error::AppError},
};

pub async fn select_pdf(
    state: &DesktopState,
    args: LibrarySelectPdfArgs,
) -> Result<Option<ImportPreviewDto>, AppError> {
    state.library()?.select_pdf(args.request_id).await
}
pub async fn confirm_import(
    state: &DesktopState,
    args: LibraryConfirmImportArgs,
) -> Result<PaperDto, AppError> {
    state
        .library()?
        .confirm_import(
            args.request_id,
            args.import_token,
            args.metadata,
            args.duplicate_resolution,
        )
        .await
}
pub async fn cancel_import(
    state: &DesktopState,
    args: LibraryCancelImportArgs,
) -> Result<(), AppError> {
    state
        .library()?
        .cancel_import(args.request_id, args.import_token)
        .await
}
pub async fn list_papers(
    state: &DesktopState,
    args: LibraryListPapersArgs,
) -> Result<PageDto<PaperDto>, AppError> {
    state
        .library()?
        .list_papers(args.request_id, args.filter)
        .await
}
pub async fn get_paper(
    state: &DesktopState,
    args: LibraryGetPaperArgs,
) -> Result<PaperDto, AppError> {
    state
        .library()?
        .get_paper(args.request_id, args.paper_id)
        .await
}
pub async fn update_metadata(
    state: &DesktopState,
    args: LibraryUpdateMetadataArgs,
) -> Result<PaperDto, AppError> {
    state
        .library()?
        .update_metadata(
            args.request_id,
            args.paper_id,
            args.expected_revision,
            args.metadata,
        )
        .await
}
pub async fn archive_paper(
    state: &DesktopState,
    args: LibraryArchivePaperArgs,
) -> Result<PaperDto, AppError> {
    state
        .library()?
        .archive_paper(args.request_id, args.paper_id, args.expected_revision)
        .await
}
pub async fn restore_paper(
    state: &DesktopState,
    args: LibraryRestorePaperArgs,
) -> Result<PaperDto, AppError> {
    state
        .library()?
        .restore_paper(args.request_id, args.paper_id, args.expected_revision)
        .await
}

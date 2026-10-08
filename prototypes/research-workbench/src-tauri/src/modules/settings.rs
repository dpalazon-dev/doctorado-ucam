use crate::{
    application::settings::{LibraryStatus, LibrarySummary, SettingsAppInfo, SettingsQuery},
    transport::error::AppError,
};
pub fn get_app_info(query: &dyn SettingsQuery) -> Result<SettingsAppInfo, AppError> {
    Ok(SettingsAppInfo {
        app_version: env!("CARGO_PKG_VERSION").into(),
        library: query.library_info(),
        capabilities: [
            ("settings".into(), true),
            ("library".into(), true),
            ("reader".into(), true),
            ("workflow".into(), true),
            ("knowledge".into(), false),
            ("concepts".into(), false),
            ("relations".into(), false),
            ("provenance".into(), false),
            ("search".into(), false),
            ("portability".into(), false),
            ("switchLibrary".into(), false),
        ]
        .into(),
    })
}
pub fn get_library_info(query: &dyn SettingsQuery) -> Result<LibrarySummary, AppError> {
    Ok(query.library_info())
}
pub async fn get_library_status(query: &dyn SettingsQuery) -> Result<LibraryStatus, AppError> {
    query.library_status().await
}

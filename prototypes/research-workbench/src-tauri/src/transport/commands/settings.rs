use crate::{
    desktop::lifecycle::DesktopState,
    transport::{dto::*, error::AppError},
};
pub fn get_app_info(state: &DesktopState) -> Result<AppInfoDto, AppError> {
    let info = crate::modules::settings::get_app_info(state.settings.as_ref())?;
    Ok(AppInfoDto {
        app_version: info.app_version,
        contract_version: ContractVersion(1),
        schema_version: Some(info.library.schema_version),
        library_id: Some(UUID(info.library.library_id)),
        library_root_label: Some(info.library.root_label),
        state: if info.library.writable {
            AppInfoDtoState::Ready
        } else {
            AppInfoDtoState::ReadOnlyDiagnostic
        },
        capabilities: info.capabilities,
    })
}
pub fn get_library_info(state: &DesktopState) -> Result<LibraryInfoDto, AppError> {
    let info = crate::modules::settings::get_library_info(state.settings.as_ref())?;
    Ok(LibraryInfoDto {
        library_id: UUID(info.library_id),
        display_name: info.display_name,
        root_label: info.root_label,
        schema_version: info.schema_version,
        writable: info.writable,
    })
}
pub async fn get_library_status(
    state: &DesktopState,
) -> Result<SettingsGetLibraryStatusOutput, AppError> {
    let info = crate::modules::settings::get_library_status(state.settings.as_ref()).await?;
    Ok(SettingsGetLibraryStatusOutput {
        writable: info.writable,
        active_operations: info.active_operations,
        recovery_required: info.recovery_required,
    })
}

pub mod library;
pub mod reader;
pub mod settings;
pub mod workflow;
use crate::{
    desktop::lifecycle::DesktopState,
    transport::{
        dto::*,
        error::{AppError, ErrorCode, envelope},
    },
};
use serde_json::Value;
pub const COMMANDS: &[&str] = &[
    "library_select_pdf",
    "library_confirm_import",
    "library_cancel_import",
    "library_list_papers",
    "library_get_paper",
    "library_update_metadata",
    "library_archive_paper",
    "library_restore_paper",
    "reader_open_paper",
    "reader_get_last_opened_paper",
    "reader_get_reading_position",
    "reader_save_reading_position",
    "workflow_get_phase",
    "workflow_get_phase_answers",
    "workflow_get_phase_definition",
    "workflow_save_phase_answer",
    "workflow_evaluate_gate",
    "workflow_advance_phase",
    "workflow_go_back_to_phase",
    "workflow_touch_phase",
    "workflow_set_p3_candidate",
    "workflow_get_p3_candidate_summary",
    "knowledge_create_item",
    "knowledge_update_item",
    "knowledge_archive_item",
    "knowledge_restore_item",
    "knowledge_get_item",
    "knowledge_list_items",
    "concept_suggest",
    "concept_get",
    "concept_list_items",
    "concept_create",
    "concept_update",
    "concept_link",
    "concept_archive",
    "concept_restore",
    "relation_create",
    "relation_update",
    "relation_list",
    "relation_archive",
    "relation_restore",
    "provenance_attach_locator",
    "provenance_update_locator",
    "provenance_get",
    "provenance_check_document_hash",
    "search_library",
    "search_knowledge",
    "export_choose_destination",
    "export_library",
    "export_paper",
    "backup_choose_destination",
    "backup_create",
    "backup_select",
    "backup_choose_restore_target",
    "backup_verify",
    "backup_restore",
    "operation_get_status",
    "operation_cancel",
    "settings_get_app_info",
    "settings_get_library_info",
    "settings_select_library",
    "settings_switch_library",
    "settings_get_library_status",
];
pub fn authorize(window: &str, command: &str) -> Result<(), AppError> {
    if window != "main" || !COMMANDS.contains(&command) {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    Ok(())
}
fn serialize<T: serde::Serialize>(value: T) -> Result<Value, AppError> {
    serde_json::to_value(value).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
}
fn validate_wire(value: &Value) -> Result<(), AppError> {
    if let Value::Object(map) = value {
        for (key, value) in map {
            if matches!(key.as_str(), "__proto__" | "constructor" | "prototype") {
                return Err(AppError::new(ErrorCode::InvalidInput));
            }
            if let Value::String(s) = value {
                let max = match key.as_str() {
                    "title" | "query" => 1000,
                    "answerText" | "bodyText" | "definition" => 20000,
                    "quoteText" | "snippet" => 10000,
                    "contextText" | "justificationText" | "rationale" => 5000,
                    "excerpt" => 500,
                    _ => 20000,
                };
                if s.chars().count() > max || (key == "title" && s.trim().is_empty()) {
                    return Err(AppError::new(ErrorCode::InvalidInput));
                }
            }
            if key == "authors"
                && value.as_array().is_some_and(|a| {
                    a.len() > 100
                        || a.iter()
                            .any(|name| name.as_str().is_none_or(|s| s.trim().is_empty()))
                })
            {
                return Err(AppError::new(ErrorCode::InvalidInput));
            }
            if let Value::Number(n) = value {
                if !n.as_f64().is_some_and(f64::is_finite) {
                    return Err(AppError::new(ErrorCode::InvalidInput));
                }
                let number = n.as_f64().unwrap_or(-1.0);
                if (key == "zoom" && !(0.25..=5.0).contains(&number))
                    || (key == "pageIndex" && (number < 1.0 || number.fract() != 0.0))
                    || (key == "limit" && !(1.0..=500.0).contains(&number))
                    || (key.to_ascii_lowercase().contains("revision")
                        && (number < 0.0 || number.fract() != 0.0))
                {
                    return Err(AppError::new(ErrorCode::InvalidInput));
                }
            }
            validate_wire(value)?;
        }
    }
    if let Value::Array(array) = value {
        for v in array {
            validate_wire(v)?;
        }
    }
    Ok(())
}
/// Closed registry dispatcher. Deserialization failures also return a structured envelope.
pub async fn dispatch(window: &str, command: &str, args: Value, state: &DesktopState) -> Value {
    let request_id =
        serde_json::from_value::<UUID>(args.get("requestId").cloned().unwrap_or(Value::Null))
            .unwrap_or_else(|_| UUID(uuid::Uuid::nil().to_string()));
    let result = async {
        authorize(window, command)?;
        validate_wire(&args)?;
        match command {
            "library_select_pdf" => {
                let input: LibrarySelectPdfArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(library::select_pdf(state, input).await?)
            }
            "library_confirm_import" => {
                let input: LibraryConfirmImportArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(library::confirm_import(state, input).await?)
            }
            "library_cancel_import" => {
                let input: LibraryCancelImportArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                library::cancel_import(state, input).await?;
                serialize(())
            }
            "library_list_papers" => {
                let input: LibraryListPapersArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(library::list_papers(state, input).await?)
            }
            "library_get_paper" => {
                let input: LibraryGetPaperArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(library::get_paper(state, input).await?)
            }
            "library_update_metadata" => {
                let input: LibraryUpdateMetadataArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(library::update_metadata(state, input).await?)
            }
            "library_archive_paper" => {
                let input: LibraryArchivePaperArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(library::archive_paper(state, input).await?)
            }
            "library_restore_paper" => {
                let input: LibraryRestorePaperArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(library::restore_paper(state, input).await?)
            }
            "reader_open_paper" => {
                let input: ReaderOpenPaperArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(reader::open_paper(state, input).await?)
            }
            "reader_get_last_opened_paper" => {
                let input: ReaderGetLastOpenedPaperArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(reader::last_opened(state, input).await?)
            }
            "reader_get_reading_position" => {
                let input: ReaderGetReadingPositionArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(reader::reading_position(state, input).await?)
            }
            "reader_save_reading_position" => {
                let input: ReaderSaveReadingPositionArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(reader::save_position(state, input).await?)
            }
            "workflow_get_phase" => {
                let input: WorkflowGetPhaseArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(workflow::get_phase(state, input).await?)
            }
            "workflow_get_phase_answers" => {
                let input: WorkflowGetPhaseAnswersArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(workflow::get_phase_answers(state, input).await?)
            }
            "workflow_get_phase_definition" => {
                let input: WorkflowGetPhaseDefinitionArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(workflow::get_phase_definition(state, input).await?)
            }
            "workflow_save_phase_answer" => {
                let input: WorkflowSavePhaseAnswerArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(workflow::save_phase_answer(state, input).await?)
            }
            "workflow_evaluate_gate" => {
                let input: WorkflowEvaluateGateArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(workflow::evaluate_gate(state, input).await?)
            }
            "workflow_advance_phase" => {
                let input: WorkflowAdvancePhaseArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(workflow::advance_phase(state, input).await?)
            }
            "workflow_go_back_to_phase" => {
                let input: WorkflowGoBackToPhaseArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(workflow::go_back_to_phase(state, input).await?)
            }
            "workflow_touch_phase" => {
                let input: WorkflowTouchPhaseArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(workflow::touch_phase(state, input).await?)
            }
            "workflow_set_p3_candidate" => {
                let _input: WorkflowSetP3CandidateArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "workflow_get_p3_candidate_summary" => {
                let _input: WorkflowGetP3CandidateSummaryArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "knowledge_create_item" => {
                let _input: KnowledgeCreateItemArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "knowledge_update_item" => {
                let _input: KnowledgeUpdateItemArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "knowledge_archive_item" => {
                let _input: KnowledgeArchiveItemArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "knowledge_restore_item" => {
                let _input: KnowledgeRestoreItemArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "knowledge_get_item" => {
                let _input: KnowledgeGetItemArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "knowledge_list_items" => {
                let _input: KnowledgeListItemsArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "concept_suggest" => {
                let _input: ConceptSuggestArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "concept_get" => {
                let _input: ConceptGetArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "concept_list_items" => {
                let _input: ConceptListItemsArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "concept_create" => {
                let _input: ConceptCreateArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "concept_update" => {
                let _input: ConceptUpdateArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "concept_link" => {
                let _input: ConceptLinkArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "concept_archive" => {
                let _input: ConceptArchiveArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "concept_restore" => {
                let _input: ConceptRestoreArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "relation_create" => {
                let _input: RelationCreateArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "relation_update" => {
                let _input: RelationUpdateArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "relation_list" => {
                let _input: RelationListArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "relation_archive" => {
                let _input: RelationArchiveArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "relation_restore" => {
                let _input: RelationRestoreArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "provenance_attach_locator" => {
                let _input: ProvenanceAttachLocatorArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "provenance_update_locator" => {
                let _input: ProvenanceUpdateLocatorArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "provenance_get" => {
                let _input: ProvenanceGetArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "provenance_check_document_hash" => {
                let _input: ProvenanceCheckDocumentHashArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "search_library" => {
                let _input: SearchLibraryArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "search_knowledge" => {
                let _input: SearchKnowledgeArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "export_choose_destination" => {
                let _input: ExportChooseDestinationArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "export_library" => {
                let _input: ExportLibraryArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "export_paper" => {
                let _input: ExportPaperArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "backup_choose_destination" => {
                let _input: BackupChooseDestinationArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "backup_create" => {
                let _input: BackupCreateArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "backup_select" => {
                let _input: BackupSelectArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "backup_choose_restore_target" => {
                let _input: BackupChooseRestoreTargetArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "operation_get_status" => {
                let _input: OperationGetStatusArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "operation_cancel" => {
                let _input: OperationCancelArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "backup_verify" => {
                let _input: BackupVerifyArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "backup_restore" => {
                let _input: BackupRestoreArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "settings_get_app_info" => {
                let _input: SettingsGetAppInfoArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(settings::get_app_info(state)?)
            }
            "settings_get_library_info" => {
                let _input: SettingsGetLibraryInfoArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(settings::get_library_info(state)?)
            }
            "settings_select_library" => {
                let _input: SettingsSelectLibraryArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "settings_switch_library" => {
                let _input: SettingsSwitchLibraryArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                Err(AppError::new(ErrorCode::UnsupportedCapability))
            }
            "settings_get_library_status" => {
                let _input: SettingsGetLibraryStatusArgs = serde_json::from_value(args)
                    .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                serialize(settings::get_library_status(state).await?)
            }
            _ => Err(AppError::new(ErrorCode::PathNotAllowed)),
        }
    }
    .await;
    serde_json::to_value(envelope(request_id, result)).expect("envelope of JSON values serializes")
}

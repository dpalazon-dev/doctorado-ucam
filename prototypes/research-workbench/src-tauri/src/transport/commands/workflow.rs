use crate::{
    desktop::lifecycle::DesktopState,
    transport::{dto::*, error::AppError},
};

pub async fn get_phase(
    state: &DesktopState,
    args: WorkflowGetPhaseArgs,
) -> Result<PhaseDto, AppError> {
    state.workflow()?.get_phase(args).await
}
pub async fn get_phase_answers(
    state: &DesktopState,
    args: WorkflowGetPhaseAnswersArgs,
) -> Result<Vec<PhaseAnswerDto>, AppError> {
    state.workflow()?.get_phase_answers(args).await
}
pub async fn get_phase_definition(
    state: &DesktopState,
    args: WorkflowGetPhaseDefinitionArgs,
) -> Result<PhaseDefinitionDto, AppError> {
    state.workflow()?.get_phase_definition(args).await
}
pub async fn save_phase_answer(
    state: &DesktopState,
    args: WorkflowSavePhaseAnswerArgs,
) -> Result<PhaseAnswerDto, AppError> {
    state.workflow()?.save_phase_answer(args).await
}
pub async fn go_back_to_phase(
    state: &DesktopState,
    args: WorkflowGoBackToPhaseArgs,
) -> Result<WorkflowGoBackToPhaseOutput, AppError> {
    state.workflow()?.go_back_to_phase(args).await
}
pub async fn touch_phase(
    state: &DesktopState,
    args: WorkflowTouchPhaseArgs,
) -> Result<PhaseDto, AppError> {
    state.workflow()?.touch_phase(args).await
}
pub async fn evaluate_gate(
    state: &DesktopState,
    args: WorkflowEvaluateGateArgs,
) -> Result<GateEvaluationDto, AppError> {
    state.workflow()?.evaluate_gate(args).await
}
pub async fn advance_phase(
    state: &DesktopState,
    args: WorkflowAdvancePhaseArgs,
) -> Result<WorkflowAdvancePhaseOutput, AppError> {
    state.workflow()?.advance_phase(args).await
}

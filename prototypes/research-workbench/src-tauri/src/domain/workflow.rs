use crate::transport::dto::{
    AnswerResolution, GateIssueDto, GateIssueDtoStatus, PhaseCode, PhaseRequiredOutputDto,
    PhaseState, ReadingDecision, RelevanceDecisionValue, ReviewType,
};
use crate::transport::error::{AppError, ErrorCode};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn canonical_hash(value: &Value) -> Result<String, AppError> {
    fn canonical(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut keys = map.keys().collect::<Vec<_>>();
                keys.sort();
                let mut sorted = Map::new();
                for key in keys {
                    sorted.insert(key.clone(), canonical(&map[key]));
                }
                Value::Object(sorted)
            }
            Value::Array(values) => Value::Array(values.iter().map(canonical).collect()),
            other => other.clone(),
        }
    }
    let bytes = serde_json::to_vec(&canonical(value))
        .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredGateAnswer {
    pub answer: NormalizedAnswer,
    pub revision: i64,
}

pub fn evaluate_outputs(
    phase: &PhaseCode,
    outputs: &[PhaseRequiredOutputDto],
    saved: &BTreeMap<String, StoredGateAnswer>,
    paper_review_type: &str,
) -> (Map<String, Value>, Vec<GateIssueDto>) {
    let mut snapshot_answers = Map::new();
    let mut issues = Vec::new();
    for output in outputs {
        let stored = saved.get(&output.key);
        let answer = stored
            .map(|value| value.answer.clone())
            .unwrap_or_else(|| normalize_answer("", None, None, AnswerResolution::PENDING));
        let answer_revision = stored.map_or(0, |value| value.revision);
        let valid = answer_matches_output(phase, &output.key, output, &answer);
        let processed = valid
            && match answer.resolution {
                AnswerResolution::PENDING => false,
                AnswerResolution::UNKNOWN
                    if *phase == PhaseCode::PRE && output.key == "review_type" =>
                {
                    answer
                        .explanation
                        .as_ref()
                        .is_some_and(|value| !value.trim().is_empty())
                        && answer_review_type_matches(&answer, paper_review_type)
                }
                AnswerResolution::UNKNOWN | AnswerResolution::NOTAPPLICABLE => answer
                    .explanation
                    .as_ref()
                    .is_some_and(|value| !value.trim().is_empty()),
                AnswerResolution::ANSWERED => {
                    if *phase == PhaseCode::P1 && output.key == "relevance_decision" {
                        true
                    } else if *phase == PhaseCode::PRE && output.key == "review_type" {
                        answer_review_type_matches(&answer, paper_review_type)
                    } else {
                        !answer.answer_text.trim().is_empty()
                    }
                }
            };
        if !valid {
            issues.push(GateIssueDto {
                requirement_key: output.key.clone(),
                message: "La respuesta no cumple el formato esperado.".into(),
                status: GateIssueDtoStatus::Invalid,
            });
        } else if !processed && output.required {
            issues.push(GateIssueDto {
                requirement_key: output.key.clone(),
                message: "Complete this output to continue.".into(),
                status: if answer.resolution == AnswerResolution::PENDING {
                    GateIssueDtoStatus::Pending
                } else {
                    GateIssueDtoStatus::Invalid
                },
            });
        }
        snapshot_answers.insert(
            output.key.clone(),
            serde_json::json!({
                "answerText": answer.answer_text,
                "structuredValue": answer.structured_value,
                "resolution": answer.resolution,
                "explanation": answer.explanation,
                "revision": answer_revision
            }),
        );
    }
    (snapshot_answers, issues)
}

fn answer_review_type_matches(answer: &NormalizedAnswer, paper_review_type: &str) -> bool {
    answer
        .structured_value
        .as_ref()
        .and_then(|value| value.get("reviewType"))
        .cloned()
        .and_then(|value| serde_json::from_value::<ReviewType>(value).ok())
        .and_then(|value| {
            serde_json::to_value(value)
                .ok()
                .and_then(|value| value.as_str().map(str::to_owned))
        })
        .as_deref()
        == Some(paper_review_type)
}

pub fn document_gate_issue(available: bool, active: bool) -> Option<GateIssueDto> {
    (!available || !active).then(|| GateIssueDto {
        requirement_key: "paperHasActiveDocument".into(),
        message: "The active PDF is unavailable.".into(),
        status: GateIssueDtoStatus::Missing,
    })
}

pub fn accepted_pre_issue(state: &str, snapshot_is_current: bool) -> Option<GateIssueDto> {
    (!snapshot_is_current).then(|| GateIssueDto {
        requirement_key: "accepted_PRE".into(),
        message: "PRE acceptance is missing or no longer matches the current inputs.".into(),
        status: if state == "COMPLETED" {
            GateIssueDtoStatus::Invalid
        } else {
            GateIssueDtoStatus::Missing
        },
    })
}

pub fn accepted_snapshot_is_current(
    state: &str,
    gate_complete: bool,
    accepted_snapshot: Option<&str>,
    accepted_hash: Option<&str>,
    current_snapshot: &str,
    current_hash: &str,
) -> bool {
    state == "COMPLETED"
        && gate_complete
        && accepted_snapshot == Some(current_snapshot)
        && accepted_hash == Some(current_hash)
}

pub fn lifecycle_allows_workflow(lifecycle: &str) -> Result<(), AppError> {
    match lifecycle {
        "NEW" | "ACTIVE" => Ok(()),
        "ARCHIVED" => Err(AppError::new(ErrorCode::GateBlocked)),
        _ => Err(AppError::new(ErrorCode::UnsupportedCapability)),
    }
}

pub fn accepted_chain_allows_forward(
    phase: &PhaseCode,
    reading_decision: Option<&ReadingDecision>,
) -> bool {
    match phase {
        PhaseCode::PRE => true,
        PhaseCode::P1 => matches!(reading_decision, Some(ReadingDecision::Continue)),
        PhaseCode::P2 => true,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvanceDecision {
    StartOrientation,
    ContinueToP2,
    KeepP1Active,
    ArchivePaper,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdvancePlan {
    pub decision: AdvanceDecision,
    pub target: Option<(PhaseCode, PhaseState)>,
    pub activate_paper: bool,
}

pub fn plan_advance(
    from: &PhaseCode,
    reading_decision: Option<&ReadingDecision>,
    lifecycle: &str,
    target_state: Option<&PhaseState>,
) -> Result<AdvancePlan, AppError> {
    lifecycle_allows_workflow(lifecycle)?;
    let decision = advance_decision(from, reading_decision)
        .ok_or_else(|| AppError::new(ErrorCode::UnsupportedCapability))?;
    let target_phase = match decision {
        AdvanceDecision::StartOrientation => Some(PhaseCode::P1),
        AdvanceDecision::ContinueToP2 => Some(PhaseCode::P2),
        AdvanceDecision::KeepP1Active | AdvanceDecision::ArchivePaper => None,
    };
    let target = match (target_phase, target_state) {
        (Some(phase), Some(PhaseState::NOTSTARTED)) => Some((phase, PhaseState::INPROGRESS)),
        (Some(phase), Some(state)) => Some((phase, state.clone())),
        (Some(_), None) | (None, Some(_)) => {
            return Err(AppError::new(ErrorCode::IntegrityFailure));
        }
        (None, None) => None,
    };
    Ok(AdvancePlan {
        activate_paper: decision == AdvanceDecision::StartOrientation && lifecycle == "NEW",
        decision,
        target,
    })
}

pub fn advance_decision(
    phase: &PhaseCode,
    reading_decision: Option<&ReadingDecision>,
) -> Option<AdvanceDecision> {
    match phase {
        PhaseCode::PRE => Some(AdvanceDecision::StartOrientation),
        PhaseCode::P1 => match reading_decision? {
            ReadingDecision::Continue => Some(AdvanceDecision::ContinueToP2),
            ReadingDecision::LightRead => Some(AdvanceDecision::KeepP1Active),
            ReadingDecision::Archive => Some(AdvanceDecision::ArchivePaper),
        },
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedAnswer {
    pub answer_text: String,
    pub structured_value: Option<Value>,
    pub explanation: Option<String>,
    pub resolution: AnswerResolution,
}

pub fn normalize_answer(
    answer_text: &str,
    structured_value: Option<Value>,
    explanation: Option<&str>,
    resolution: AnswerResolution,
) -> NormalizedAnswer {
    fn normalize(value: &str) -> String {
        value
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .trim()
            .to_owned()
    }
    let answer_text = normalize(answer_text);
    let explanation = explanation.map(normalize).filter(|value| !value.is_empty());
    NormalizedAnswer {
        answer_text,
        structured_value,
        explanation,
        resolution,
    }
}

pub fn answer_matches_output(
    phase: &PhaseCode,
    key: &str,
    output: &PhaseRequiredOutputDto,
    answer: &NormalizedAnswer,
) -> bool {
    if !output.allowed_resolutions.contains(&answer.resolution) {
        return false;
    }
    match (phase, key) {
        (PhaseCode::PRE, "review_type") => {
            if answer.resolution == AnswerResolution::PENDING {
                return answer.answer_text.is_empty()
                    && answer.explanation.is_none()
                    && answer.structured_value.as_ref().is_none_or(|value| {
                        value.as_object().is_some_and(|object| {
                            object.len() == 1
                                && object.get("reviewType").cloned().is_some_and(|raw| {
                                    serde_json::from_value::<ReviewType>(raw).is_ok()
                                })
                        })
                    });
            }
            if !matches!(
                answer.resolution,
                AnswerResolution::ANSWERED | AnswerResolution::UNKNOWN
            ) || (answer.resolution == AnswerResolution::UNKNOWN && answer.explanation.is_none())
            {
                return false;
            }
            if !answer.answer_text.is_empty() {
                return false;
            }
            let Some(Value::Object(value)) = &answer.structured_value else {
                return false;
            };
            if value.len() != 1 {
                return false;
            }
            let Some(raw_type) = value.get("reviewType").cloned() else {
                return false;
            };
            let Ok(review_type) = serde_json::from_value::<ReviewType>(raw_type) else {
                return false;
            };
            matches!(
                (&answer.resolution, review_type),
                (AnswerResolution::UNKNOWN, ReviewType::Unknown)
                    | (
                        AnswerResolution::ANSWERED,
                        ReviewType::Survey
                            | ReviewType::TopicalReview
                            | ReviewType::Slr
                            | ReviewType::MappingStudy
                            | ReviewType::Tutorial
                            | ReviewType::Other
                    )
            )
        }
        (PhaseCode::P1, "relevance_decision") => {
            if !answer.answer_text.is_empty() {
                return false;
            }
            match answer.resolution {
                AnswerResolution::PENDING => {
                    answer.explanation.is_none()
                        && answer
                            .structured_value
                            .as_ref()
                            .is_none_or(is_valid_decision)
                }
                AnswerResolution::ANSWERED => answer
                    .structured_value
                    .as_ref()
                    .is_some_and(is_valid_decision),
                _ => false,
            }
        }
        (PhaseCode::P2, _) => false,
        _ => {
            answer_is_valid(answer, output.required)
                && answer.structured_value.is_none()
                && match answer.resolution {
                    AnswerResolution::UNKNOWN | AnswerResolution::NOTAPPLICABLE => {
                        answer.answer_text.is_empty()
                    }
                    _ => true,
                }
        }
    }
}

fn is_valid_decision(value: &Value) -> bool {
    serde_json::from_value::<RelevanceDecisionValue>(value.clone()).is_ok()
}

pub fn answer_is_valid(answer: &NormalizedAnswer, _required: bool) -> bool {
    use AnswerResolution::{ANSWERED, NOTAPPLICABLE, PENDING, UNKNOWN};
    match answer.resolution {
        PENDING => true,
        ANSWERED => !answer.answer_text.is_empty(),
        UNKNOWN | NOTAPPLICABLE => answer.explanation.as_ref().is_some_and(|s| !s.is_empty()),
    }
}

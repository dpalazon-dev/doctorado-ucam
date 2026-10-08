use crate::{
    application::{
        library::{archive_paper_in_tx, canonical_hash},
        library_ports::LibraryRepository,
        workflow_ports::{
            WorkflowDocumentReference, WorkflowPhaseAcceptance, WorkflowPhaseRecord,
            WorkflowRepository,
        },
    },
    domain::workflow::{
        StoredGateAnswer, accepted_chain_allows_forward, accepted_pre_issue,
        accepted_snapshot_is_current, answer_matches_output, document_gate_issue, evaluate_outputs,
        lifecycle_allows_workflow, normalize_answer, plan_advance,
    },
    modules::workflow::definitions,
    transport::{
        dto::{
            GateEvaluationDto, PhaseAnswerDto, PhaseCode, PhaseDto, PhaseState,
            RelevanceDecisionValue, UUID, WorkflowAdvancePhaseArgs, WorkflowAdvancePhaseOutput,
            WorkflowEvaluateGateArgs, WorkflowGoBackToPhaseArgs, WorkflowGoBackToPhaseOutput,
            WorkflowSavePhaseAnswerArgs, WorkflowTouchPhaseArgs,
        },
        error::{AppError, ErrorCode},
    },
};
use rusqlite::Transaction;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const MAX_SAFE_REVISION: i64 = 9_007_199_254_740_991;

fn phase_record<'a>(
    records: &'a [WorkflowPhaseRecord],
    phase: &PhaseCode,
) -> Result<&'a WorkflowPhaseRecord, AppError> {
    let mut matches = records.iter().filter(|record| &record.code == phase);
    let record = matches
        .next()
        .ok_or_else(|| AppError::new(ErrorCode::NotFound))?;
    if matches.next().is_some() {
        return Err(AppError::new(ErrorCode::IntegrityFailure));
    }
    Ok(record)
}

fn pinned_definition(
    records: &[WorkflowPhaseRecord],
    phase: &PhaseCode,
) -> Result<crate::transport::dto::PhaseDefinitionDto, AppError> {
    let record = phase_record(records, phase)?;
    definitions::get(&record.code, Some(record.definition_version)).map_err(|error| {
        if error.code == ErrorCode::NotFound {
            AppError::new(ErrorCode::UnsupportedCapability)
        } else {
            error
        }
    })
}

fn phase_order(phase: &PhaseCode) -> Result<usize, AppError> {
    match phase {
        PhaseCode::PRE => Ok(0),
        PhaseCode::P1 => Ok(1),
        PhaseCode::P2 => Ok(2),
        _ => Err(AppError::new(ErrorCode::UnsupportedCapability)),
    }
}

fn next_clock(
    repository: &dyn WorkflowRepository,
    tx: &Transaction<'_>,
    paper_id: &str,
) -> Result<i64, AppError> {
    repository
        .max_phase_revision(tx, paper_id)?
        .checked_add(1)
        .filter(|revision| *revision <= MAX_SAFE_REVISION)
        .ok_or_else(|| AppError::new(ErrorCode::Conflict))
}

fn phase_state(state: &str) -> Result<PhaseState, AppError> {
    match state {
        "NOT_STARTED" => Ok(PhaseState::NOTSTARTED),
        "IN_PROGRESS" => Ok(PhaseState::INPROGRESS),
        "COMPLETED" => Ok(PhaseState::COMPLETED),
        "NEEDS_REVIEW" => Ok(PhaseState::NEEDSREVIEW),
        _ => Err(AppError::new(ErrorCode::IntegrityFailure)),
    }
}

fn phase_dto(paper_id: &str, record: &WorkflowPhaseRecord) -> Result<PhaseDto, AppError> {
    Ok(PhaseDto {
        paper_id: UUID(paper_id.to_owned()),
        code: record.code.clone(),
        definition_version: record.definition_version,
        state: phase_state(&record.state)?,
        revision: record.revision,
        completed_at: record.completed_at.clone(),
    })
}

fn enum_string<T: serde::Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_value(value)
        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))
}

pub fn evaluate_gate_in_tx(
    tx: &Transaction<'_>,
    workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository,
    args: &WorkflowEvaluateGateArgs,
    reference: &WorkflowDocumentReference,
    available: bool,
) -> Result<(GateEvaluationDto, Value), AppError> {
    if !matches!(args.phase_code, PhaseCode::PRE | PhaseCode::P1) {
        return Err(AppError::new(ErrorCode::UnsupportedCapability));
    }
    let paper = library.paper(tx, &args.paper_id.0)?;
    let records = workflow.phases(tx, &args.paper_id.0)?;
    let record = phase_record(&records, &args.phase_code)?;
    let definition = pinned_definition(&records, &args.phase_code)?;
    let answers = workflow.answers(tx, &args.paper_id.0, &args.phase_code)?;
    let mut saved = BTreeMap::new();
    for answer in answers {
        let normalized = normalize_answer(
            &answer.answer_text,
            answer.structured_value,
            answer.explanation.as_deref(),
            answer.resolution,
        );
        saved.insert(
            answer.question_key,
            StoredGateAnswer {
                answer: normalized,
                revision: answer.revision,
            },
        );
    }
    if saved.keys().any(|key| {
        !definition
            .required_outputs
            .iter()
            .any(|output| output.key == *key)
    }) {
        return Err(AppError::new(ErrorCode::IntegrityFailure));
    }
    let review_type = enum_string(&paper.review_type)?;
    let (snapshot_answers, mut issues) = evaluate_outputs(
        &args.phase_code,
        &definition.required_outputs,
        &saved,
        &review_type,
    );
    if let Some(issue) = document_gate_issue(
        available,
        reference.registered.document.status == crate::transport::dto::DocumentDtoStatus::ACTIVE,
    ) {
        issues.push(issue);
    }
    let document_input = json!({
        "id": reference.active_document_id,
        "status": reference.registered.document.status,
        "sha256": reference.registered.document.sha256,
        "available": available
    });
    let inputs = if args.phase_code == PhaseCode::PRE {
        json!({"title":paper.title,"reviewType":review_type,"document":document_input})
    } else {
        let pre = phase_record(&records, &PhaseCode::PRE)?;
        let pre_args = WorkflowEvaluateGateArgs {
            request_id: args.request_id.clone(),
            paper_id: args.paper_id.clone(),
            phase_code: PhaseCode::PRE,
        };
        let (pre_gate, pre_snapshot) =
            evaluate_gate_in_tx(tx, workflow, library, &pre_args, reference, available)?;
        let serialized = serde_json::to_string(&pre_snapshot)
            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
        let current = accepted_snapshot_is_current(
            &pre.state,
            pre_gate.complete,
            pre.snapshot_json.as_deref(),
            pre.snapshot_hash.as_deref(),
            &serialized,
            &pre_gate.input_snapshot_hash,
        );
        if let Some(issue) = accepted_pre_issue(&pre.state, current) {
            issues.push(issue);
        }
        let pre_definition = pinned_definition(&records, &PhaseCode::PRE)?;
        json!({
            "preAcceptedGateSnapshotHash": pre.snapshot_hash,
            "preDefinitionHash": pre_definition.definition_hash,
            "document": document_input
        })
    };
    let snapshot = json!({
        "snapshotVersion": 1,
        "paperId": args.paper_id.0,
        "phaseCode": args.phase_code,
        "definitionVersion": definition.version,
        "definitionHash": definition.definition_hash,
        "answers": snapshot_answers,
        "inputs": inputs,
        "artifacts": []
    });
    let input_snapshot_hash = canonical_hash(&snapshot)?;
    Ok((
        GateEvaluationDto {
            paper_id: args.paper_id.clone(),
            phase_code: args.phase_code.clone(),
            definition_version: definition.version,
            complete: issues.is_empty(),
            issues,
            phase_revision: record.revision,
            input_snapshot_hash,
            evaluated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        },
        snapshot,
    ))
}

pub fn save_phase_answer_in_tx(
    tx: &Transaction<'_>,
    workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository,
    args: &WorkflowSavePhaseAnswerArgs,
) -> Result<PhaseAnswerDto, AppError> {
    if args.expected_revision < 0 || args.expected_revision > MAX_SAFE_REVISION {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    if !matches!(args.phase_code, PhaseCode::PRE | PhaseCode::P1) {
        return Err(AppError::new(ErrorCode::UnsupportedCapability));
    }
    let paper = library.paper(tx, &args.paper_id.0)?;
    let lifecycle = enum_string(&paper.lifecycle)?;
    lifecycle_allows_workflow(&lifecycle)?;
    let records = workflow.phases(tx, &args.paper_id.0)?;
    let phase = phase_record(&records, &args.phase_code)?;
    if phase.state == "NOT_STARTED" {
        return Err(AppError::new(ErrorCode::GateBlocked));
    }
    let definition = pinned_definition(&records, &args.phase_code)?;
    let output = definition
        .required_outputs
        .iter()
        .find(|output| output.key == args.question_key)
        .ok_or_else(|| AppError::new(ErrorCode::InvalidInput))?;
    let normalized = normalize_answer(
        &args.answer_text,
        args.structured_value.clone(),
        args.explanation.as_deref(),
        args.resolution.clone(),
    );
    if !answer_matches_output(&args.phase_code, &args.question_key, output, &normalized)
        || normalized.answer_text.chars().count() > 20_000
        || normalized
            .explanation
            .as_ref()
            .is_some_and(|text| text.chars().count() > 5_000)
    {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    let mut answers = workflow.answers(tx, &args.paper_id.0, &args.phase_code)?;
    let existing = answers
        .iter()
        .position(|answer| answer.question_key == args.question_key)
        .map(|index| answers.remove(index));
    let current_revision = existing.as_ref().map_or(0, |answer| answer.revision);
    if current_revision != args.expected_revision {
        return Err(AppError::conflict_revision(current_revision));
    }
    let unchanged = existing.as_ref().is_some_and(|answer| {
        answer.answer_text == normalized.answer_text
            && answer.structured_value == normalized.structured_value
            && answer.resolution == normalized.resolution
            && answer.explanation == normalized.explanation
    });
    if unchanged {
        return existing.ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure));
    }
    let revision = current_revision
        .checked_add(1)
        .filter(|value| *value <= MAX_SAFE_REVISION)
        .ok_or_else(|| AppError::new(ErrorCode::Conflict))?;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let answer = PhaseAnswerDto {
        paper_id: args.paper_id.clone(),
        phase_code: args.phase_code.clone(),
        question_key: args.question_key.clone(),
        answer_text: normalized.answer_text,
        structured_value: normalized.structured_value,
        resolution: normalized.resolution,
        explanation: normalized.explanation,
        revision,
        updated_at: now.clone(),
    };
    workflow.put_answer(tx, &answer)?;
    let phases_before = records
        .iter()
        .map(|record| {
            json!({"phaseCode":record.code,"state":record.state,"revision":record.revision})
        })
        .collect::<Vec<_>>();
    invalidate_effective_change(
        tx,
        workflow,
        &args.paper_id.0,
        std::slice::from_ref(&args.phase_code),
    )?;
    let phases_after = workflow.phases(tx, &args.paper_id.0)?;
    library.audit_change(
        tx,
        &args.request_id.0,
        "workflow.answer_changed",
        &args.paper_id.0,
        &json!({
            "phaseCode": args.phase_code,
            "definitionVersion": definition.version,
            "questionKey": args.question_key,
            "before": existing.as_ref().map(|previous| json!({
                "answerText":previous.answer_text,
                "structuredValue":previous.structured_value,
                "resolution":previous.resolution,
                "explanation":previous.explanation,
                "revision":previous.revision
            })),
            "after": {
                "answerText":answer.answer_text,
                "structuredValue":answer.structured_value,
                "resolution":answer.resolution,
                "explanation":answer.explanation,
                "revision":answer.revision
            },
            "phasesBefore":phases_before,
            "phasesAfter":phases_after.iter().map(|record| json!({
                "phaseCode":record.code,"state":record.state,"revision":record.revision
            })).collect::<Vec<_>>()
        }),
        &now,
    )?;
    Ok(answer)
}

pub fn go_back_to_phase_in_tx(
    tx: &Transaction<'_>,
    workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository,
    args: &WorkflowGoBackToPhaseArgs,
) -> Result<WorkflowGoBackToPhaseOutput, AppError> {
    if args.expected_active_phase_revision < 0
        || args.expected_active_phase_revision > MAX_SAFE_REVISION
    {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    let paper = library.paper(tx, &args.paper_id.0)?;
    lifecycle_allows_workflow(&enum_string(&paper.lifecycle)?)?;
    let active = paper
        .active_phase_code
        .as_ref()
        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    let records = workflow.phases(tx, &args.paper_id.0)?;
    let current = phase_record(&records, active)?;
    if current.revision != args.expected_active_phase_revision {
        return Err(AppError::conflict_revision(current.revision));
    }
    if phase_order(&args.target_phase)? > phase_order(active)? {
        return Err(AppError::new(ErrorCode::GateBlocked));
    }
    let target = phase_record(&records, &args.target_phase)?;
    if target.state == "NOT_STARTED" {
        return Err(AppError::new(ErrorCode::GateBlocked));
    }
    if active != &args.target_phase {
        let revision = next_clock(workflow, tx, &args.paper_id.0)?;
        workflow.set_phase_clock(
            tx,
            &args.paper_id.0,
            &args.target_phase,
            &target.state,
            revision,
        )?;
        workflow.set_active_phase(tx, &args.paper_id.0, &args.target_phase)?;
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        library.audit_change(
            tx,
            &args.request_id.0,
            "workflow.context_changed",
            &args.paper_id.0,
            &json!({
                "operation":"goBack",
                "contextBefore":active,
                "contextAfter":args.target_phase,
                "destinationBefore":{"phaseCode":target.code,"state":target.state,"revision":target.revision},
                "destinationAfter":{"phaseCode":target.code,"state":target.state,"revision":revision}
            }),
            &now,
        )?;
        Ok(WorkflowGoBackToPhaseOutput {
            active_phase_code: args.target_phase.clone(),
            active_phase_revision: revision,
        })
    } else {
        Ok(WorkflowGoBackToPhaseOutput {
            active_phase_code: args.target_phase.clone(),
            active_phase_revision: current.revision,
        })
    }
}

struct AcceptedChainInputs<'a> {
    paper_id: &'a str,
    target: &'a PhaseCode,
    reference: &'a WorkflowDocumentReference,
    available: bool,
    request_id: &'a str,
}

fn ensure_accepted_chain_in_tx(
    tx: &Transaction<'_>,
    workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository,
    inputs: AcceptedChainInputs<'_>,
) -> Result<(), AppError> {
    let phases: &[PhaseCode] = match inputs.target {
        PhaseCode::PRE => &[],
        PhaseCode::P1 => &[PhaseCode::PRE],
        PhaseCode::P2 => &[PhaseCode::PRE, PhaseCode::P1],
        _ => return Err(AppError::new(ErrorCode::UnsupportedCapability)),
    };
    for phase in phases {
        let records = workflow.phases(tx, inputs.paper_id)?;
        let accepted = phase_record(&records, phase)?;
        let args = WorkflowEvaluateGateArgs {
            request_id: UUID(inputs.request_id.to_owned()),
            paper_id: UUID(inputs.paper_id.to_owned()),
            phase_code: phase.clone(),
        };
        let (gate, snapshot) = evaluate_gate_in_tx(
            tx,
            workflow,
            library,
            &args,
            inputs.reference,
            inputs.available,
        )?;
        let serialized = serde_json::to_string(&snapshot)
            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
        if !accepted_snapshot_is_current(
            &accepted.state,
            gate.complete,
            accepted.snapshot_json.as_deref(),
            accepted.snapshot_hash.as_deref(),
            &serialized,
            &gate.input_snapshot_hash,
        ) {
            return Err(AppError::new(ErrorCode::GateBlocked));
        }
        if phase == &PhaseCode::P1 {
            let answers = workflow.answers(tx, inputs.paper_id, phase)?;
            let answer = answers
                .iter()
                .find(|answer| answer.question_key == "relevance_decision")
                .ok_or_else(|| AppError::new(ErrorCode::GateBlocked))?;
            let raw = answer
                .structured_value
                .as_ref()
                .ok_or_else(|| AppError::new(ErrorCode::GateBlocked))?;
            let decision: RelevanceDecisionValue = serde_json::from_value(raw.clone())
                .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
            if !accepted_chain_allows_forward(phase, Some(&decision.reading_decision)) {
                return Err(AppError::new(ErrorCode::GateBlocked));
            }
        }
    }
    Ok(())
}

pub fn touch_phase_in_tx(
    tx: &Transaction<'_>,
    workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository,
    args: &WorkflowTouchPhaseArgs,
    proof: Option<(&WorkflowDocumentReference, bool)>,
) -> Result<PhaseDto, AppError> {
    if args.expected_active_phase_revision < 0
        || args.expected_active_phase_revision > MAX_SAFE_REVISION
    {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    if !matches!(
        args.phase_code,
        PhaseCode::PRE | PhaseCode::P1 | PhaseCode::P2
    ) {
        return Err(AppError::new(ErrorCode::UnsupportedCapability));
    }
    let paper = library.paper(tx, &args.paper_id.0)?;
    lifecycle_allows_workflow(&enum_string(&paper.lifecycle)?)?;
    let active = paper
        .active_phase_code
        .as_ref()
        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    let records = workflow.phases(tx, &args.paper_id.0)?;
    let current = phase_record(&records, active)?;
    if current.revision != args.expected_active_phase_revision {
        return Err(AppError::conflict_revision(current.revision));
    }
    let target = phase_record(&records, &args.phase_code)?;
    let definition = pinned_definition(&records, &args.phase_code)?;
    let forward = phase_order(&args.phase_code)? > phase_order(active)?;
    if &args.phase_code != active && !forward && target.state == "NOT_STARTED" {
        return Err(AppError::new(ErrorCode::GateBlocked));
    }
    if forward {
        let (reference, available) = proof.ok_or_else(|| AppError::new(ErrorCode::GateBlocked))?;
        ensure_accepted_chain_in_tx(
            tx,
            workflow,
            library,
            AcceptedChainInputs {
                paper_id: &args.paper_id.0,
                target: &args.phase_code,
                reference,
                available,
                request_id: &args.request_id.0,
            },
        )?;
    }
    if &args.phase_code == active {
        return phase_dto(&args.paper_id.0, target);
    }
    let next_state = if forward && target.state == "NOT_STARTED" {
        "IN_PROGRESS"
    } else {
        target.state.as_str()
    };
    let revision = next_clock(workflow, tx, &args.paper_id.0)?;
    workflow.set_phase_clock(tx, &args.paper_id.0, &args.phase_code, next_state, revision)?;
    workflow.set_active_phase(tx, &args.paper_id.0, &args.phase_code)?;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    library.audit_change(
        tx,
        &args.request_id.0,
        "workflow.context_changed",
        &args.paper_id.0,
        &json!({
            "operation":"touch",
            "contextBefore":active,
            "contextAfter":args.phase_code,
            "destinationBefore":{"phaseCode":target.code,"state":target.state,"revision":target.revision},
            "destinationAfter":{"phaseCode":target.code,"state":next_state,"revision":revision}
        }),
        &now,
    )?;
    Ok(PhaseDto {
        paper_id: args.paper_id.clone(),
        code: args.phase_code.clone(),
        definition_version: definition.version,
        state: phase_state(next_state)?,
        revision,
        completed_at: target.completed_at.clone(),
    })
}

pub fn advance_phase_in_tx(
    tx: &Transaction<'_>,
    workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository,
    args: &WorkflowAdvancePhaseArgs,
    reference: &WorkflowDocumentReference,
    available: bool,
) -> Result<WorkflowAdvancePhaseOutput, AppError> {
    if args.expected_phase_revision < 0 || args.expected_phase_revision > MAX_SAFE_REVISION {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    if !matches!(args.from_phase, PhaseCode::PRE | PhaseCode::P1) {
        return Err(AppError::new(ErrorCode::UnsupportedCapability));
    }
    let paper = library.paper(tx, &args.paper_id.0)?;
    let lifecycle = enum_string(&paper.lifecycle)?;
    lifecycle_allows_workflow(&lifecycle)?;
    let active = paper
        .active_phase_code
        .as_ref()
        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    if active != &args.from_phase {
        return Err(AppError::new(ErrorCode::GateBlocked));
    }
    let records = workflow.phases(tx, &args.paper_id.0)?;
    let origin = phase_record(&records, &args.from_phase)?;
    if origin.revision != args.expected_phase_revision {
        return Err(AppError::conflict_revision(origin.revision));
    }
    let gate_args = WorkflowEvaluateGateArgs {
        request_id: args.request_id.clone(),
        paper_id: args.paper_id.clone(),
        phase_code: args.from_phase.clone(),
    };
    let (mut gate, snapshot) =
        evaluate_gate_in_tx(tx, workflow, library, &gate_args, reference, available)?;
    let mut reading_decision = None;
    if args.from_phase == PhaseCode::P1 {
        let answers = workflow.answers(tx, &args.paper_id.0, &PhaseCode::P1)?;
        let answer = answers
            .iter()
            .find(|answer| answer.question_key == "relevance_decision")
            .ok_or_else(|| AppError::new(ErrorCode::GateBlocked))?;
        let raw = answer
            .structured_value
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::GateBlocked))?;
        let decision: RelevanceDecisionValue = serde_json::from_value(raw.clone())
            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
        reading_decision = Some(decision.reading_decision);
    }
    if !gate.complete {
        return Err(AppError::new(ErrorCode::GateBlocked));
    }
    let decision =
        crate::domain::workflow::advance_decision(&args.from_phase, reading_decision.as_ref())
            .ok_or_else(|| AppError::new(ErrorCode::UnsupportedCapability))?;
    let target_phase = match decision {
        crate::domain::workflow::AdvanceDecision::StartOrientation => Some(PhaseCode::P1),
        crate::domain::workflow::AdvanceDecision::ContinueToP2 => Some(PhaseCode::P2),
        _ => None,
    };
    if let Some(phase) = target_phase.as_ref() {
        pinned_definition(&records, phase)?;
    }
    let target_state = target_phase
        .as_ref()
        .map(|phase| phase_record(&records, phase).and_then(|target| phase_state(&target.state)))
        .transpose()?;
    let plan = plan_advance(
        &args.from_phase,
        reading_decision.as_ref(),
        &lifecycle,
        target_state.as_ref(),
    )?;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let final_phase_revision = next_clock(workflow, tx, &args.paper_id.0)?;
    let snapshot_json =
        serde_json::to_string(&snapshot).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
    workflow.set_phase_acceptance(
        tx,
        &args.paper_id.0,
        &args.from_phase,
        WorkflowPhaseAcceptance {
            revision: final_phase_revision,
            completed_at: &now,
            snapshot_json: &snapshot_json,
            snapshot_hash: &gate.input_snapshot_hash,
        },
    )?;
    gate.phase_revision = final_phase_revision;
    if plan.activate_paper
        && library.update_lifecycle(tx, &args.paper_id.0, paper.revision, "ACTIVE", None, &now)?
            != 1
    {
        return Err(AppError::new(ErrorCode::IntegrityFailure));
    }
    if let Some((phase, target_state)) = &plan.target {
        phase_record(&records, phase)?;
        let target_revision = next_clock(workflow, tx, &args.paper_id.0)?;
        workflow.set_phase_clock(
            tx,
            &args.paper_id.0,
            phase,
            match target_state {
                PhaseState::NOTSTARTED => "NOT_STARTED",
                PhaseState::INPROGRESS => "IN_PROGRESS",
                PhaseState::COMPLETED => "COMPLETED",
                PhaseState::NEEDSREVIEW => "NEEDS_REVIEW",
            },
            target_revision,
        )?;
        workflow.set_active_phase(tx, &args.paper_id.0, phase)?;
    } else if plan.decision == crate::domain::workflow::AdvanceDecision::ArchivePaper {
        archive_paper_in_tx(
            tx,
            library,
            &args.request_id.0,
            &args.paper_id.0,
            paper.revision,
        )?;
    } else {
        workflow.set_active_phase(tx, &args.paper_id.0, &PhaseCode::P1)?;
    }
    let updated_records = workflow.phases(tx, &args.paper_id.0)?;
    let resulting_paper = library.paper(tx, &args.paper_id.0)?;
    let resulting_lifecycle = enum_string(&resulting_paper.lifecycle)?;
    let accepted = phase_record(&updated_records, &args.from_phase)?;
    let destination_before = plan
        .target
        .as_ref()
        .map(|(phase, _)| {
            phase_record(&records, phase).map(|record| {
                json!({"phaseCode":record.code,"state":record.state,"revision":record.revision})
            })
        })
        .transpose()?;
    let destination_after = plan
        .target
        .as_ref()
        .map(|(phase, _)| {
            phase_record(&updated_records, phase).map(|record| {
                json!({"phaseCode":record.code,"state":record.state,"revision":record.revision})
            })
        })
        .transpose()?;
    library.audit_change(
        tx,
        &args.request_id.0,
        "workflow.phase_accepted",
        &args.paper_id.0,
        &json!({
            "phaseCode":args.from_phase,
            "definitionVersion":accepted.definition_version,
            "before":{"state":origin.state,"revision":origin.revision,"completedAt":origin.completed_at,"snapshotHash":origin.snapshot_hash},
            "after":{"state":accepted.state,"revision":accepted.revision,"completedAt":accepted.completed_at,"snapshotHash":accepted.snapshot_hash},
            "decision":match plan.decision {
                crate::domain::workflow::AdvanceDecision::StartOrientation => "StartOrientation",
                crate::domain::workflow::AdvanceDecision::ContinueToP2 => "ContinueToP2",
                crate::domain::workflow::AdvanceDecision::KeepP1Active => "KeepP1Active",
                crate::domain::workflow::AdvanceDecision::ArchivePaper => "ArchivePaper",
            },
            "contextBefore":active,
            "contextAfter":resulting_paper.active_phase_code,
            "lifecycleBefore":lifecycle,
            "lifecycleAfter":resulting_lifecycle,
            "destinationBefore":destination_before,
            "destinationAfter":destination_after
        }),
        &now,
    )?;
    Ok(WorkflowAdvancePhaseOutput {
        phase: phase_dto(&args.paper_id.0, accepted)?,
        gate,
        next_phase: plan.target.map(|(phase, _)| phase),
        paper_lifecycle: serde_json::from_value(Value::String(resulting_lifecycle))
            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?,
    })
}

pub fn initialize_processing(
    tx: &Transaction<'_>,
    repository: &dyn WorkflowRepository,
    paper_id: &str,
) -> Result<(), AppError> {
    let context = repository
        .context(tx, paper_id)?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound))?;
    let phases = repository.phases(tx, paper_id)?;
    if context.initialized {
        let coherent = phases.len() == 3
            && [PhaseCode::PRE, PhaseCode::P1, PhaseCode::P2]
                .iter()
                .all(|code| phases.iter().filter(|p| &p.code == code).count() == 1)
            && phases.iter().all(coherent_phase_record)
            && context.active_phase.as_deref().is_some_and(|active| {
                matches!(active, "PRE" | "P1" | "P2")
                    && phases
                        .iter()
                        .any(|p| format!("{:?}", p.code) == active && p.state != "NOT_STARTED")
            });
        return if coherent {
            Ok(())
        } else {
            Err(AppError::new(ErrorCode::IntegrityFailure))
        };
    }
    if context.active_phase.is_some() || !phases.is_empty() {
        return Err(AppError::new(ErrorCode::IntegrityFailure));
    }
    repository.initialize(tx, paper_id)?;
    repository.set_context(tx, paper_id, "PRE")
}

fn coherent_phase_record(phase: &crate::application::workflow_ports::WorkflowPhaseRecord) -> bool {
    if phase.definition_version != 1
        || phase.revision < 0
        || !matches!(
            phase.state.as_str(),
            "NOT_STARTED" | "IN_PROGRESS" | "COMPLETED" | "NEEDS_REVIEW"
        )
        || ((phase.state == "NOT_STARTED") != (phase.revision == 0))
        || phase.snapshot_json.is_some() != phase.snapshot_hash.is_some()
    {
        return false;
    }
    if phase.state == "NOT_STARTED" && phase.completed_at.is_some() {
        return false;
    }
    if phase.state == "COMPLETED" && (phase.completed_at.is_none() || phase.snapshot_json.is_none())
    {
        return false;
    }
    match (&phase.snapshot_json, &phase.snapshot_hash) {
        (Some(raw), Some(hash)) => serde_json::from_str::<serde_json::Value>(raw)
            .ok()
            .and_then(|value| crate::application::library::canonical_hash(&value).ok())
            .is_some_and(|actual| actual == *hash),
        (None, None) => true,
        _ => false,
    }
}

pub fn invalidate_effective_change(
    tx: &Transaction<'_>,
    repository: &dyn WorkflowRepository,
    paper_id: &str,
    input_phases: &[PhaseCode],
) -> Result<(), AppError> {
    if input_phases.is_empty() {
        return Ok(());
    }
    let mut direct = [false; 3];
    for phase in input_phases {
        let Some(index) = phase_index(phase) else {
            return Err(AppError::new(ErrorCode::UnsupportedCapability));
        };
        direct[index] = true;
    }
    let records = repository.phases(tx, paper_id)?;
    if records.len() != 3 {
        return Err(AppError::new(ErrorCode::IntegrityFailure));
    }
    let mut clock = repository.max_phase_revision(tx, paper_id)?;
    let mut earlier_input = false;
    for (index, phase) in [PhaseCode::PRE, PhaseCode::P1, PhaseCode::P2]
        .iter()
        .enumerate()
    {
        let Some(record) = records.iter().find(|r| &r.code == phase) else {
            return Err(AppError::new(ErrorCode::IntegrityFailure));
        };
        let state = record.state.as_str();
        if !matches!(
            state,
            "NOT_STARTED" | "IN_PROGRESS" | "COMPLETED" | "NEEDS_REVIEW"
        ) || record.revision < 0
        {
            return Err(AppError::new(ErrorCode::IntegrityFailure));
        }
        let dependent = earlier_input;
        let affected = state != "NOT_STARTED" && (dependent || direct[index]);
        if affected {
            clock = clock
                .checked_add(1)
                .filter(|v| *v <= 9_007_199_254_740_991)
                .ok_or_else(|| AppError::new(ErrorCode::Conflict))?;
            let next_state = if dependent || state == "COMPLETED" {
                "NEEDS_REVIEW"
            } else {
                state
            };
            repository.set_phase_clock(tx, paper_id, phase, next_state, clock)?;
        }
        earlier_input |= direct[index];
    }
    Ok(())
}

fn phase_index(phase: &PhaseCode) -> Option<usize> {
    match phase {
        PhaseCode::PRE => Some(0),
        PhaseCode::P1 => Some(1),
        PhaseCode::P2 => Some(2),
        _ => None,
    }
}

use research_workbench_core::transport::dto::{AnswerResolution, PhaseCode};
use research_workbench_core::{domain::workflow, modules::workflow::definitions};
use serde_json::json;

#[test]
fn embedded_phase_definitions_match_the_accepted_payloads_and_hashes() {
    for (code, expected_hash) in [
        (
            PhaseCode::PRE,
            "ee2f0e54c352475dc5896080b3a7ed8f5425b637a3712e4a77f721a942cc4a7f",
        ),
        (
            PhaseCode::P1,
            "8e2c71f5d3f8578ff940c9903086f982aedca69adbb704c8fa3afe6381359150",
        ),
        (
            PhaseCode::P2,
            "2c8059fce6d4beadf442ea9d9f5d063906b5e1542f6fe0972e1c6f85be509393",
        ),
    ] {
        let definition = definitions::get(&code, None).unwrap();
        assert_eq!(definition.version, 1);
        assert_eq!(definition.definition_hash, expected_hash);
        assert_eq!(definitions::get(&code, Some(1)).unwrap(), definition);
    }
}

#[test]
fn advance_plan_selects_target_and_preserves_started_state_without_storage() {
    use research_workbench_core::{
        domain::workflow::{AdvanceDecision, plan_advance},
        transport::{
            dto::{PhaseCode, PhaseState, ReadingDecision},
            error::ErrorCode,
        },
    };

    let initial =
        plan_advance(&PhaseCode::PRE, None, "NEW", Some(&PhaseState::NOTSTARTED)).unwrap();
    assert_eq!(initial.decision, AdvanceDecision::StartOrientation);
    assert_eq!(
        initial.target,
        Some((PhaseCode::P1, PhaseState::INPROGRESS))
    );
    assert!(initial.activate_paper);

    let reaccepted = plan_advance(
        &PhaseCode::PRE,
        None,
        "ACTIVE",
        Some(&PhaseState::NEEDSREVIEW),
    )
    .unwrap();
    assert_eq!(
        reaccepted.target,
        Some((PhaseCode::P1, PhaseState::NEEDSREVIEW))
    );
    assert!(!reaccepted.activate_paper);

    let completed_target = plan_advance(
        &PhaseCode::PRE,
        None,
        "ACTIVE",
        Some(&PhaseState::COMPLETED),
    )
    .unwrap();
    assert_eq!(
        completed_target.target,
        Some((PhaseCode::P1, PhaseState::COMPLETED))
    );

    let continued = plan_advance(
        &PhaseCode::P1,
        Some(&ReadingDecision::Continue),
        "ACTIVE",
        Some(&PhaseState::NEEDSREVIEW),
    )
    .unwrap();
    assert_eq!(continued.decision, AdvanceDecision::ContinueToP2);
    assert_eq!(
        continued.target,
        Some((PhaseCode::P2, PhaseState::NEEDSREVIEW))
    );

    for decision in [ReadingDecision::LightRead, ReadingDecision::Archive] {
        let terminal = plan_advance(&PhaseCode::P1, Some(&decision), "ACTIVE", None).unwrap();
        assert_eq!(terminal.target, None);
        assert!(!terminal.activate_paper);
    }

    assert_eq!(
        plan_advance(&PhaseCode::PRE, None, "NEW", None)
            .unwrap_err()
            .code,
        ErrorCode::IntegrityFailure,
    );
    assert_eq!(
        plan_advance(
            &PhaseCode::PRE,
            None,
            "ARCHIVED",
            Some(&PhaseState::NOTSTARTED)
        )
        .unwrap_err()
        .code,
        ErrorCode::GateBlocked,
    );
}

#[test]
fn answer_normalization_changes_line_endings_and_outer_whitespace_only() {
    let answer = workflow::normalize_answer(
        "  a  b\r\nc\r ",
        None,
        Some("  note\r\n "),
        AnswerResolution::UNKNOWN,
    );
    assert_eq!(answer.answer_text, "a  b\nc");
    assert_eq!(answer.explanation.as_deref(), Some("note"));
}

#[test]
fn unknown_and_not_applicable_require_explanations_and_pending_blocks() {
    for resolution in [AnswerResolution::UNKNOWN, AnswerResolution::NOTAPPLICABLE] {
        let answer = workflow::normalize_answer("", None, None, resolution);
        assert!(!workflow::answer_is_valid(&answer, true));
    }

    let pending = workflow::normalize_answer("", None, None, AnswerResolution::PENDING);
    assert!(workflow::answer_is_valid(&pending, true));
    let explained =
        workflow::normalize_answer("", None, Some("because"), AnswerResolution::UNKNOWN);
    assert!(workflow::answer_is_valid(&explained, true));
}

#[test]
fn p1_decision_is_validated_as_closed_structure_not_answer_prose() {
    let definition = definitions::get(&PhaseCode::P1, None).unwrap();
    let output = definition
        .required_outputs
        .iter()
        .find(|output| output.key == "relevance_decision")
        .unwrap();
    let prose = workflow::normalize_answer("continue", None, None, AnswerResolution::ANSWERED);
    assert!(!workflow::answer_matches_output(
        &PhaseCode::P1,
        "relevance_decision",
        output,
        &prose
    ));
    let structured = workflow::normalize_answer(
        "",
        Some(json!({"relevance":"sufficient","readingDecision":"continue"})),
        None,
        AnswerResolution::ANSWERED,
    );
    assert!(workflow::answer_matches_output(
        &PhaseCode::P1,
        "relevance_decision",
        output,
        &structured
    ));
    let extra = workflow::normalize_answer(
        "",
        Some(json!({"relevance":"sufficient","readingDecision":"continue","extra":true})),
        None,
        AnswerResolution::ANSWERED,
    );
    assert!(!workflow::answer_matches_output(
        &PhaseCode::P1,
        "relevance_decision",
        output,
        &extra
    ));
}

#[test]
fn definition_json_is_not_mutable_through_returned_values() {
    let first = definitions::get(&PhaseCode::PRE, None).unwrap();
    let original_hash = first.definition_hash.clone();
    let mut altered = first;
    altered.name.push_str(" altered");
    assert_eq!(
        definitions::get(&PhaseCode::PRE, None)
            .unwrap()
            .definition_hash,
        original_hash
    );
    assert_ne!(
        altered.name,
        definitions::get(&PhaseCode::PRE, None).unwrap().name
    );
}

#[test]
fn unknown_keys_are_not_silently_accepted_as_phase_answers() {
    let value = json!({"questionKey":"not-a-real-output","resolution":"ANSWERED","answerText":"x"});
    assert!(!definitions::known_output(
        &PhaseCode::PRE,
        value["questionKey"].as_str().unwrap()
    ));
}

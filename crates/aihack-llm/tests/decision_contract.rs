use aihack_ai_contract::ActionIntent;
use aihack_llm::decision::{decision_log_lines, DecisionSource, SuggestedAction};

#[test]
fn decision_log_identifies_the_source_and_action() {
    let suggestion = SuggestedAction {
        action: ActionIntent::Noop,
        rationale: "safe fallback".to_string(),
        source: DecisionSource::Fallback,
    };

    assert_eq!(
        decision_log_lines(&suggestion, Some(false)),
        vec![
            "Decision(Fallback, rejected)".to_string(),
            "Noop :: safe fallback".to_string(),
        ]
    );
}

use serde::Serialize;
use thiserror::Error;

use aihack_ai_contract::{CommandIntent, Direction, EntityKind, ItemKind, RunState, SnapshotHash};
use aihack_runtime::{save::ReplayLineV1, GameClient};

/// Headless runner가 한 accepted turn을 만들기 위해 시도할 command 후보를 제공한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeadlessPolicy {
    WaitV1,
    SurvivalV1,
    ReplayFile,
}

/// Headless 실행의 재현 가능한 결과 요약이다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HeadlessRunReport {
    pub seed: u64,
    pub policy: HeadlessPolicy,
    pub requested_turns: u64,
    pub accepted_turns: u64,
    pub submitted_commands: u64,
    pub final_state: RunState,
    pub final_hash: SnapshotHash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayMismatchField {
    TurnBefore,
    Accepted,
    TurnAdvanced,
    Events,
    OutcomeSnapshotHash,
    NextState,
    SnapshotHashAfter,
}

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
pub enum HeadlessRunError {
    #[error("campaign ascended at turn {turn} before requested target")]
    VictoryBeforeTarget { turn: u64, submitted_commands: u64 },
    #[error("target turn {target} is before loaded turn {turn}")]
    TargetBeforeCurrent { turn: u64, target: u64 },
    #[error("no accepted action at turn {turn} after {attempts} attempts")]
    NoAcceptedAction {
        turn: u64,
        attempts: u8,
        submitted_commands: u64,
    },
    #[error("game ended at turn {turn}")]
    GameOver { turn: u64, submitted_commands: u64 },
    #[error("replay ended before target turn {turn}")]
    ReplayExhausted { turn: u64, submitted_commands: u64 },
    #[error("replay integrity mismatch at line {line}: {field:?}")]
    ReplayMismatch {
        line: usize,
        field: ReplayMismatchField,
        submitted_commands: u64,
    },
}

impl HeadlessRunError {
    pub const fn submitted_commands(&self) -> u64 {
        match self {
            Self::TargetBeforeCurrent { .. } => 0,
            Self::NoAcceptedAction {
                submitted_commands, ..
            }
            | Self::VictoryBeforeTarget {
                submitted_commands, ..
            }
            | Self::GameOver {
                submitted_commands, ..
            }
            | Self::ReplayExhausted {
                submitted_commands, ..
            }
            | Self::ReplayMismatch {
                submitted_commands, ..
            } => *submitted_commands,
        }
    }
}

/// Replay line의 command를 순서대로 적용해 absolute target turn까지 재생한다.
pub fn run_replay_to_turn<C: GameClient + Clone>(
    session: &mut C,
    target_turn: u64,
    replay: &[ReplayLineV1],
) -> Result<HeadlessRunReport, HeadlessRunError> {
    let start_turn = session.revision().turn;
    if target_turn < start_turn {
        return Err(HeadlessRunError::TargetBeforeCurrent {
            turn: start_turn,
            target: target_turn,
        });
    }
    let mut working = session.clone();
    let mut submitted_commands = 0;
    if start_turn < target_turn && matches!(working.run_state(), RunState::Victory { .. }) {
        return Err(HeadlessRunError::VictoryBeforeTarget {
            turn: start_turn,
            submitted_commands,
        });
    }
    for (line_index, line) in replay.iter().enumerate() {
        if working.revision().turn >= target_turn {
            break;
        }
        let line_number = line_index + 1;
        if line.turn_before != working.revision().turn {
            return Err(replay_mismatch(
                line_number,
                ReplayMismatchField::TurnBefore,
                submitted_commands,
            ));
        }
        submitted_commands += 1;
        let actual = working.submit(line.command);
        for (matches, field) in [
            (
                actual.accepted == line.outcome.accepted,
                ReplayMismatchField::Accepted,
            ),
            (
                actual.turn_advanced == line.outcome.turn_advanced,
                ReplayMismatchField::TurnAdvanced,
            ),
            (
                actual.events == line.outcome.events,
                ReplayMismatchField::Events,
            ),
            (
                actual.snapshot_hash == line.outcome.snapshot_hash,
                ReplayMismatchField::OutcomeSnapshotHash,
            ),
            (
                actual.next_state == line.outcome.next_state,
                ReplayMismatchField::NextState,
            ),
            (
                actual.snapshot_hash == line.snapshot_hash_after,
                ReplayMismatchField::SnapshotHashAfter,
            ),
        ] {
            if !matches {
                return Err(replay_mismatch(line_number, field, submitted_commands));
            }
        }
        if working.revision().turn < target_turn
            && matches!(working.run_state(), RunState::Victory { .. })
        {
            return Err(HeadlessRunError::VictoryBeforeTarget {
                turn: working.revision().turn,
                submitted_commands,
            });
        }
        if matches!(working.run_state(), RunState::GameOver { .. }) {
            return Err(HeadlessRunError::GameOver {
                turn: working.revision().turn,
                submitted_commands,
            });
        }
    }
    if working.revision().turn < target_turn {
        return Err(HeadlessRunError::ReplayExhausted {
            turn: working.revision().turn,
            submitted_commands,
        });
    }
    let observation = working.observation();
    let revision = working.revision();
    let report = HeadlessRunReport {
        seed: observation.seed,
        policy: HeadlessPolicy::ReplayFile,
        requested_turns: target_turn,
        accepted_turns: revision.turn - start_turn,
        submitted_commands,
        final_state: working.run_state(),
        final_hash: revision.snapshot_hash,
    };
    *session = working;
    Ok(report)
}

fn replay_mismatch(
    line: usize,
    field: ReplayMismatchField,
    submitted_commands: u64,
) -> HeadlessRunError {
    HeadlessRunError::ReplayMismatch {
        line,
        field,
        submitted_commands,
    }
}

/// 목표 absolute turn까지 진행하며, 한 turn에서 최대 16개 후보만 시도한다.
pub fn run_to_turn<C: GameClient + ?Sized>(
    session: &mut C,
    target_turn: u64,
    policy: HeadlessPolicy,
) -> Result<HeadlessRunReport, HeadlessRunError> {
    run_to_turn_with_trace(session, target_turn, policy).map(|(report, _)| report)
}

/// Replay output을 원하는 adapter를 위해 제출 순서를 함께 반환한다.
pub fn run_to_turn_with_trace<C: GameClient + ?Sized>(
    session: &mut C,
    target_turn: u64,
    policy: HeadlessPolicy,
) -> Result<(HeadlessRunReport, Vec<ReplayLineV1>), HeadlessRunError> {
    let start_turn = session.revision().turn;
    if target_turn < start_turn {
        return Err(HeadlessRunError::TargetBeforeCurrent {
            turn: start_turn,
            target: target_turn,
        });
    }
    let mut submitted_commands = 0;
    let mut trace = Vec::new();

    while session.revision().turn < target_turn {
        if matches!(session.run_state(), RunState::Victory { .. }) {
            return Err(HeadlessRunError::VictoryBeforeTarget {
                turn: session.revision().turn,
                submitted_commands,
            });
        }
        let turn_before = session.revision().turn;
        let mut candidates = policy.candidates(session);
        if policy != HeadlessPolicy::ReplayFile
            && !candidates.contains(&CommandIntent::Wait)
            && session
                .observation()
                .legal_actions
                .contains(&CommandIntent::Wait)
        {
            candidates.truncate(15);
            candidates.push(CommandIntent::Wait);
        }
        let mut advanced = false;
        let mut attempts = 0;

        for command in candidates.into_iter().take(16) {
            attempts += 1;
            submitted_commands += 1;
            let outcome = session.submit(command);
            let accepted = outcome.accepted;
            trace.push(ReplayLineV1 {
                turn_before,
                command,
                snapshot_hash_after: outcome.snapshot_hash.clone(),
                outcome,
            });
            if matches!(session.run_state(), RunState::GameOver { .. }) {
                return Err(HeadlessRunError::GameOver {
                    turn: session.revision().turn,
                    submitted_commands,
                });
            }
            if accepted && session.revision().turn > turn_before {
                advanced = true;
                break;
            }
        }

        if !advanced {
            return Err(HeadlessRunError::NoAcceptedAction {
                turn: turn_before,
                attempts,
                submitted_commands,
            });
        }
    }

    let observation = session.observation();
    let revision = session.revision();
    let report = HeadlessRunReport {
        seed: observation.seed,
        policy,
        requested_turns: target_turn,
        accepted_turns: revision.turn - start_turn,
        submitted_commands,
        final_state: session.run_state(),
        final_hash: revision.snapshot_hash,
    };
    Ok((report, trace))
}

impl HeadlessPolicy {
    pub const fn id(self) -> &'static str {
        match self {
            Self::WaitV1 => "wait-v1",
            Self::SurvivalV1 => "survival-v1",
            Self::ReplayFile => "replay-file",
        }
    }

    pub const fn wait_v1() -> Self {
        Self::WaitV1
    }

    pub const fn survival_v1() -> Self {
        Self::SurvivalV1
    }

    pub fn candidates<C: GameClient + ?Sized>(self, session: &C) -> Vec<CommandIntent> {
        match self {
            Self::WaitV1 => vec![CommandIntent::Wait],
            Self::SurvivalV1 => survival_candidates(session),
            Self::ReplayFile => Vec::new(),
        }
    }
}

fn survival_candidates<C: GameClient + ?Sized>(session: &C) -> Vec<CommandIntent> {
    let observation = session.observation();

    if observation.player.hp.saturating_mul(2) <= observation.player.max_hp {
        if let Some(item) = observation
            .inventory
            .iter()
            .find(|item| item.kind == ItemKind::PotionHealing)
            .map(|item| item.item)
        {
            let command = CommandIntent::Quaff { item };
            if observation.legal_actions.contains(&command) {
                return vec![command];
            }
        }
    }

    for direction in priority_directions() {
        let delta = direction.delta();
        let hostile_adjacent = observation.visible_entities.iter().any(|entity| {
            matches!(entity.kind, EntityKind::Monster(_))
                && entity.pos.x == observation.player_pos.x + delta.dx
                && entity.pos.y == observation.player_pos.y + delta.dy
        });
        let command = CommandIntent::Move(direction);
        if hostile_adjacent && observation.legal_actions.contains(&command) {
            return vec![command];
        }
    }

    if observation.legal_actions.contains(&CommandIntent::Pickup) {
        return vec![CommandIntent::Pickup];
    }

    for direction in priority_directions() {
        let command = CommandIntent::Move(direction);
        if observation.legal_actions.contains(&command) {
            return vec![command];
        }
    }

    vec![CommandIntent::Wait]
}

const fn priority_directions() -> [Direction; 8] {
    [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
        Direction::NorthEast,
        Direction::SouthEast,
        Direction::SouthWest,
        Direction::NorthWest,
    ]
}

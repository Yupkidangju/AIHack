// ============================================================================
// [v2.44.0 R32-1] 턴 엔진 (turn_engine.rs)
// 게임루프 오케스트레이터 — 매 턴 시스템 순서 관리
// ============================================================================

use crate::core::entity::player::Player;
use crate::core::events::{EventQueue, GameEvent};

/// [v2.44.0 R32-1] 턴 처리 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnResult {
    /// 정상: 다음 턴 계속
    Continue,
    /// 플레이어 사망 → 게임 오버
    PlayerDied { cause: String },
    /// 게임 종료 (승천/퇴장)
    GameEnded { ascended: bool },
    /// 입력 대기 (UI 필요)
    WaitingForInput,
}

/// [v2.44.0 R32-1] 턴 처리 순서 (원본: NetHack allmain.c 루프 순서)
///
/// 1. 배고픔 감소 (nutrition burn)
/// 2. 운 감쇠 (luck decay)
/// 3. 상태 타이머 tick (status timers)
/// 4. 플레이어 이동/행동
/// 5. 몬스터 AI (우선순위 큐)
/// 6. 레벨 이벤트 (함정, 지형)
/// 7. HP/에너지 재생
/// 8. 사망 판정
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    NutritionDecay,
    LuckDecay,
    StatusTimers,
    PlayerAction,
    MonsterActions,
    LevelEvents,
    Regeneration,
    DeathCheck,
}

pub const TURN_PHASES: &[TurnPhase] = &[
    TurnPhase::NutritionDecay,
    TurnPhase::LuckDecay,
    TurnPhase::StatusTimers,
    TurnPhase::PlayerAction,
    TurnPhase::MonsterActions,
    TurnPhase::LevelEvents,
    TurnPhase::Regeneration,
    TurnPhase::DeathCheck,
];

/// [v2.44.0 R32-1] 턴 컨텍스트 — 한 턴에 필요한 모든 입력
pub struct TurnContext<'a> {
    pub player: &'a mut Player,
    pub turn_number: u64,
    pub event_queue: &'a mut EventQueue,
}

/// [v2.44.0 R32-1] 각 페이즈 실행 (순수 디스패치)
pub fn execute_phase(phase: TurnPhase, ctx: &mut TurnContext) -> Option<TurnResult> {
    match phase {
        TurnPhase::NutritionDecay => {
            crate::core::systems::bridges::hunger_bridge::tick_nutrition(ctx);
            None
        }
        TurnPhase::LuckDecay => {
            crate::core::systems::bridges::luck_align_bridge::tick_luck(ctx);
            None
        }
        TurnPhase::StatusTimers => {
            // TODO R33: status timer 연결
            None
        }
        TurnPhase::PlayerAction => {
            // 입력 대기 — UI에서 채워줌
            Some(TurnResult::WaitingForInput)
        }
        TurnPhase::MonsterActions => {
            // TODO R34: 몬스터 AI 연결
            None
        }
        TurnPhase::LevelEvents => {
            // TODO R35: 지형 이벤트
            None
        }
        TurnPhase::Regeneration => {
            crate::core::systems::bridges::hunger_bridge::tick_regeneration(ctx);
            None
        }
        TurnPhase::DeathCheck => {
            crate::core::systems::bridges::combat_bridge::check_player_death(ctx)
        }
    }
}

/// [v2.44.0 R32-1] 전체 턴 실행 (플레이어 행동 제외)
pub fn run_turn_systems(ctx: &mut TurnContext) -> TurnResult {
    for &phase in TURN_PHASES {
        if phase == TurnPhase::PlayerAction {
            continue; // 플레이어 행동은 외부에서 주입
        }
        if let Some(result) = execute_phase(phase, ctx) {
            return result;
        }
    }
    TurnResult::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_player() -> Player {
        Player::new()
    }

    fn make_queue() -> EventQueue {
        EventQueue::new()
    }

    #[test]
    fn test_phase_order() {
        assert_eq!(TURN_PHASES[0], TurnPhase::NutritionDecay);
        assert_eq!(TURN_PHASES[7], TurnPhase::DeathCheck);
    }

    #[test]
    fn test_turn_continue() {
        let mut p = make_player();
        let mut q = make_queue();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let result = run_turn_systems(&mut ctx);
        assert_eq!(result, TurnResult::Continue);
    }

    #[test]
    fn test_nutrition_decreases() {
        let mut p = make_player();
        let initial = p.nutrition;
        let mut q = make_queue();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        crate::core::systems::bridges::hunger_bridge::tick_nutrition(&mut ctx);
        assert!(p.nutrition < initial);
    }
}

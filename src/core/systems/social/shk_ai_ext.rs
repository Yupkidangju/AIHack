// ============================================================================
// [v2.27.0 R15-5] 상점 주인 AI (shk_ai_ext.rs)
// 원본: NetHack 3.6.7 shk.c AI 로직 부분
// 상점 주인의 이동, 추적, 교전, 문 관리 AI
// ============================================================================

// =============================================================================
// [1] 상점 주인 행동 유형
// =============================================================================

/// [v2.27.0 R15-5] 상점 주인 행동
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShkAction {
    /// 상점 내 대기
    StandGuard,
    /// 플레이어 추적
    ChaseThief { target_x: i32, target_y: i32 },
    /// 문 닫기 (도둑 차단)
    CloseDoor { door_x: i32, door_y: i32 },
    /// 문 열기 (영업)
    OpenDoor { door_x: i32, door_y: i32 },
    /// 플레이어 공격
    Attack,
    /// 외상 요구 (말풍선)
    DemandPayment { amount: i64 },
    /// 도주 (HP 낮음)
    Flee,
    /// 반갑게 인사 (정상 영업)
    Greet,
}

/// [v2.27.0 R15-5] 상점 주인 AI 입력
#[derive(Debug, Clone)]
pub struct ShkAiInput {
    pub mood_angry: bool,
    pub player_in_shop: bool,
    pub player_has_unpaid: bool,
    pub player_debt: i64,
    pub hp_ratio: f64,
    pub player_distance: i32,
    pub door_position: Option<(i32, i32)>,
    pub player_x: i32,
    pub player_y: i32,
}

// =============================================================================
// [2] AI 결정 (원본: shk.c shkmove, shk_move)
// =============================================================================

/// [v2.27.0 R15-5] 상점 주인 AI (원본: shkmove)
pub fn decide_shk_action(input: &ShkAiInput) -> ShkAction {
    // 도주 (HP 위험)
    if input.hp_ratio < 0.15 {
        return ShkAction::Flee;
    }

    // 분노 상태
    if input.mood_angry {
        if input.player_distance <= 1 {
            return ShkAction::Attack;
        }
        return ShkAction::ChaseThief {
            target_x: input.player_x,
            target_y: input.player_y,
        };
    }

    // 부채 있는 플레이어
    if input.player_in_shop && input.player_debt > 0 {
        return ShkAction::DemandPayment {
            amount: input.player_debt,
        };
    }

    // 미지불 물품 소지 채 떠나려 함
    if !input.player_in_shop && input.player_has_unpaid {
        if let Some((dx, dy)) = input.door_position {
            return ShkAction::CloseDoor {
                door_x: dx,
                door_y: dy,
            };
        }
    }

    // 정상 상태
    if input.player_in_shop {
        return ShkAction::Greet;
    }

    ShkAction::StandGuard
}

// =============================================================================
// [3] 상점 주인 이동 목표 (원본: shk.c find_shktravel_pos)
// =============================================================================

/// [v2.27.0 R15-5] 상점 주인 이동 목표 좌표
pub fn shk_travel_target(shk_x: i32, shk_y: i32, target_x: i32, target_y: i32) -> (i32, i32) {
    let dx = (target_x - shk_x).signum();
    let dy = (target_y - shk_y).signum();
    (shk_x + dx, shk_y + dy)
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn normal_input() -> ShkAiInput {
        ShkAiInput {
            mood_angry: false,
            player_in_shop: true,
            player_has_unpaid: false,
            player_debt: 0,
            hp_ratio: 0.9,
            player_distance: 3,
            door_position: Some((10, 5)),
            player_x: 15,
            player_y: 5,
        }
    }

    #[test]
    fn test_greet() {
        let action = decide_shk_action(&normal_input());
        assert_eq!(action, ShkAction::Greet);
    }

    #[test]
    fn test_demand_payment() {
        let input = ShkAiInput {
            player_debt: 100,
            ..normal_input()
        };
        let action = decide_shk_action(&input);
        assert!(matches!(action, ShkAction::DemandPayment { amount: 100 }));
    }

    #[test]
    fn test_angry_attack() {
        let input = ShkAiInput {
            mood_angry: true,
            player_distance: 1,
            ..normal_input()
        };
        assert_eq!(decide_shk_action(&input), ShkAction::Attack);
    }

    #[test]
    fn test_angry_chase() {
        let input = ShkAiInput {
            mood_angry: true,
            player_distance: 5,
            ..normal_input()
        };
        assert!(matches!(
            decide_shk_action(&input),
            ShkAction::ChaseThief { .. }
        ));
    }

    #[test]
    fn test_flee() {
        let input = ShkAiInput {
            hp_ratio: 0.1,
            ..normal_input()
        };
        assert_eq!(decide_shk_action(&input), ShkAction::Flee);
    }

    #[test]
    fn test_close_door() {
        let input = ShkAiInput {
            player_in_shop: false,
            player_has_unpaid: true,
            ..normal_input()
        };
        assert!(matches!(
            decide_shk_action(&input),
            ShkAction::CloseDoor { .. }
        ));
    }

    #[test]
    fn test_stand_guard() {
        let input = ShkAiInput {
            player_in_shop: false,
            player_has_unpaid: false,
            ..normal_input()
        };
        assert_eq!(decide_shk_action(&input), ShkAction::StandGuard);
    }

    #[test]
    fn test_travel_target() {
        let (nx, ny) = shk_travel_target(10, 5, 15, 8);
        assert_eq!(nx, 11);
        assert_eq!(ny, 6);
    }
}

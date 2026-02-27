// ============================================================================
// [v2.37.0 Phase 101-4] AI 전략 통합 (strategy_phase101_ext.rs)
// 원본: NetHack 3.6.7 src/monmove.c + muse.c 전략적 AI 통합
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] AI 전략 유형 — ai_strategy
// =============================================================================

/// [v2.37.0 101-4] AI 전략 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIStrategy {
    Aggressive, // 공격적 돌진
    Defensive,  // 방어적 후퇴
    Flanking,   // 측면 우회
    Ambush,     // 매복
    Ranged,     // 원거리 공격
    Healing,    // 회복 우선
    Fleeing,    // 도주
    Casting,    // 주문 시전
    ItemUse,    // 아이템 사용
    Summoning,  // 소환
    Guarding,   // 경비
    Patrolling, // 순찰
}

/// [v2.37.0 101-4] AI 상황 판단
#[derive(Debug, Clone)]
pub struct AIContext {
    pub monster_hp_ratio: f64,
    pub monster_level: i32,
    pub player_level: i32,
    pub distance_to_player: i32,
    pub has_ranged: bool,
    pub has_spells: bool,
    pub has_healing: bool,
    pub is_peaceful: bool,
    pub allies_nearby: i32,
    pub is_unique: bool,
}

/// [v2.37.0 101-4] 전략 결정
pub fn decide_strategy(ctx: &AIContext, rng: &mut NetHackRng) -> (AIStrategy, String) {
    if ctx.is_peaceful {
        return (AIStrategy::Patrolling, "평화적으로 순찰 중.".to_string());
    }

    // HP 위급 시
    if ctx.monster_hp_ratio < 0.2 {
        if ctx.has_healing {
            return (AIStrategy::Healing, "위험! 회복 우선.".to_string());
        }
        if ctx.is_unique {
            return (AIStrategy::Fleeing, "유니크 몬스터 도주!".to_string());
        }
        // 약한 몬스터는 50% 확률로 도주
        if rng.rn2(2) == 0 {
            return (AIStrategy::Fleeing, "HP 부족, 도주!".to_string());
        }
    }

    // 레벨 우위 분석
    let level_diff = ctx.monster_level - ctx.player_level;

    // 원거리 공격 가능 & 거리 있으면
    if ctx.has_ranged && ctx.distance_to_player > 3 {
        return (AIStrategy::Ranged, "원거리에서 공격.".to_string());
    }

    // 주문 가능 & 안전 거리
    if ctx.has_spells && ctx.distance_to_player > 1 {
        return (AIStrategy::Casting, "주문 시전.".to_string());
    }

    // 아군이 많으면 측면 공격
    if ctx.allies_nearby >= 3 {
        return (AIStrategy::Flanking, "아군과 함께 측면 공격!".to_string());
    }

    // 레벨 우위 시 공격적
    if level_diff > 3 {
        return (AIStrategy::Aggressive, "압도적 레벨! 공격!".to_string());
    }

    // 레벨 열세 시 방어적
    if level_diff < -5 {
        return (AIStrategy::Defensive, "레벨 열세. 방어적 행동.".to_string());
    }

    // 기본: 공격적
    (AIStrategy::Aggressive, "돌진!".to_string())
}

/// [v2.37.0 101-4] AI 이동 방향 계산
pub fn calculate_move_direction(
    strategy: AIStrategy,
    mx: i32,
    my: i32,
    px: i32,
    py: i32,
    rng: &mut NetHackRng,
) -> (i32, i32) {
    let dx = (px - mx).signum();
    let dy = (py - my).signum();

    match strategy {
        AIStrategy::Aggressive | AIStrategy::Ranged | AIStrategy::Casting => (dx, dy),
        AIStrategy::Fleeing => (-dx, -dy),
        AIStrategy::Defensive => {
            if rng.rn2(3) == 0 {
                (-dx, -dy)
            } else {
                (0, 0)
            }
        }
        AIStrategy::Flanking => {
            if rng.rn2(2) == 0 {
                (dy, -dx)
            } else {
                (-dy, dx)
            }
        }
        AIStrategy::Patrolling | AIStrategy::Guarding => {
            let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
            dirs[rng.rn2(4) as usize]
        }
        _ => (0, 0),
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    fn base_ctx() -> AIContext {
        AIContext {
            monster_hp_ratio: 0.8,
            monster_level: 10,
            player_level: 10,
            distance_to_player: 3,
            has_ranged: false,
            has_spells: false,
            has_healing: false,
            is_peaceful: false,
            allies_nearby: 0,
            is_unique: false,
        }
    }

    #[test]
    fn test_aggressive() {
        let mut rng = test_rng();
        let (s, _) = decide_strategy(&base_ctx(), &mut rng);
        assert_eq!(s, AIStrategy::Aggressive);
    }

    #[test]
    fn test_peaceful() {
        let mut rng = test_rng();
        let mut ctx = base_ctx();
        ctx.is_peaceful = true;
        let (s, _) = decide_strategy(&ctx, &mut rng);
        assert_eq!(s, AIStrategy::Patrolling);
    }

    #[test]
    fn test_low_hp_heal() {
        let mut rng = test_rng();
        let mut ctx = base_ctx();
        ctx.monster_hp_ratio = 0.1;
        ctx.has_healing = true;
        let (s, _) = decide_strategy(&ctx, &mut rng);
        assert_eq!(s, AIStrategy::Healing);
    }

    #[test]
    fn test_ranged() {
        let mut rng = test_rng();
        let mut ctx = base_ctx();
        ctx.has_ranged = true;
        ctx.distance_to_player = 5;
        let (s, _) = decide_strategy(&ctx, &mut rng);
        assert_eq!(s, AIStrategy::Ranged);
    }

    #[test]
    fn test_flanking() {
        let mut rng = test_rng();
        let mut ctx = base_ctx();
        ctx.allies_nearby = 5;
        let (s, _) = decide_strategy(&ctx, &mut rng);
        assert_eq!(s, AIStrategy::Flanking);
    }

    #[test]
    fn test_move_aggressive() {
        let mut rng = test_rng();
        let (dx, dy) = calculate_move_direction(AIStrategy::Aggressive, 0, 0, 5, 5, &mut rng);
        assert_eq!((dx, dy), (1, 1));
    }

    #[test]
    fn test_move_flee() {
        let mut rng = test_rng();
        let (dx, dy) = calculate_move_direction(AIStrategy::Fleeing, 5, 5, 10, 10, &mut rng);
        assert_eq!((dx, dy), (-1, -1));
    }

    #[test]
    fn test_unique_flee() {
        let mut rng = test_rng();
        let mut ctx = base_ctx();
        ctx.monster_hp_ratio = 0.1;
        ctx.is_unique = true;
        let (s, _) = decide_strategy(&ctx, &mut rng);
        assert_eq!(s, AIStrategy::Fleeing);
    }
}

// ============================================================================
// [v2.39.0 Phase 103-5] 멀티플레이어/고스트 통합 (ghost_phase103_ext.rs)
// 원본: NetHack 3.6.7 유령 시스템 + bones 파일 + 멀티플레이어 개념
// 순수 결과 패턴
//
// 구현 범위:
//   - 플레이어 유령(고스트) 생성 (bones 파일)
//   - 고스트 조우 이벤트
//   - 유령의 무덤/인벤토리
//   - 위험 레벨 경고 시스템
//   - 데스 드롭 관리
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 고스트/본즈 시스템 — ghost_bones
// =============================================================================

/// [v2.39.0 103-5] 고스트 데이터
#[derive(Debug, Clone)]
pub struct GhostData {
    pub name: String,
    pub role: String,
    pub level: i32,
    pub death_cause: String,
    pub death_depth: i32,
    pub death_turn: i32,
    pub inventory: Vec<String>,
    pub gold: i64,
    pub hp_at_death: i32,
    pub max_hp: i32,
}

/// [v2.39.0 103-5] 고스트 생성 (사망 시)
pub fn create_ghost(
    name: &str,
    role: &str,
    level: i32,
    cause: &str,
    depth: i32,
    turn: i32,
    items: &[&str],
    gold: i64,
    hp: i32,
    max_hp: i32,
) -> GhostData {
    GhostData {
        name: name.to_string(),
        role: role.to_string(),
        level,
        death_cause: cause.to_string(),
        death_depth: depth,
        death_turn: turn,
        inventory: items.iter().map(|s| s.to_string()).collect(),
        gold,
        hp_at_death: hp,
        max_hp,
    }
}

/// [v2.39.0 103-5] 고스트 조우 이벤트
pub fn encounter_ghost(
    ghost: &GhostData,
    player_level: i32,
    rng: &mut NetHackRng,
) -> (String, Vec<String>, i64) {
    let danger = ghost.level - player_level;

    let message = if danger > 5 {
        format!(
            "⚠️ {}의 유령이 나타났다! (Lv.{} {}) \"{}...\" — 매우 위험!",
            ghost.name, ghost.level, ghost.role, ghost.death_cause
        )
    } else if danger > 0 {
        format!(
            "👻 {}의 유령이 나타났다. (Lv.{} {}) 주의!",
            ghost.name, ghost.level, ghost.role
        )
    } else {
        format!(
            "👻 {}의 유령이 떠돌고 있다. (Lv.{} {})",
            ghost.name, ghost.level, ghost.role
        )
    };

    // 유령 처치 시 드롭 아이템 결정
    let mut drops = Vec::new();
    for item in &ghost.inventory {
        if rng.rn2(3) == 0 {
            // 33% 확률로 드롭
            drops.push(item.clone());
        }
    }

    // 금화 드롭 (50~100%)
    let gold_drop = ghost.gold * (50 + rng.rn2(51) as i64) / 100;

    (message, drops, gold_drop)
}

/// [v2.39.0 103-5] 위험 경고
pub fn danger_warning(player_level: i32, dungeon_depth: i32) -> Option<String> {
    let expected_level = dungeon_depth + 1;
    let gap = expected_level - player_level;

    if gap >= 10 {
        Some("☠️ 이 깊이에서 당신은 극도로 위험하다! 즉시 후퇴하라!".to_string())
    } else if gap >= 5 {
        Some("⚠️ 이 깊이에 비해 레벨이 다소 낮다. 조심하라.".to_string())
    } else if gap >= 3 {
        Some("💡 이 깊이에 비해 약간 약하다. 대비하라.".to_string())
    } else {
        None
    }
}

/// [v2.39.0 103-5] 묘비 메시지 생성
pub fn generate_epitaph(ghost: &GhostData) -> String {
    format!(
        "┌─────────────────────┐\n\
         │      R.I.P.         │\n\
         │  {}                 │\n\
         │  Lv.{} {}           │\n\
         │                     │\n\
         │  {} 에 의해         │\n\
         │  {} 층에서 사망     │\n\
         │                     │\n\
         │  턴: {}             │\n\
         └─────────────────────┘",
        ghost.name, ghost.level, ghost.role, ghost.death_cause, ghost.death_depth, ghost.death_turn
    )
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

    fn sample_ghost() -> GhostData {
        create_ghost(
            "용사",
            "전사",
            15,
            "드래곤",
            25,
            10000,
            &["장검 +3", "미스릴 갑옷", "치유 포션"],
            5000,
            0,
            120,
        )
    }

    #[test]
    fn test_create_ghost() {
        let ghost = sample_ghost();
        assert_eq!(ghost.name, "용사");
        assert_eq!(ghost.inventory.len(), 3);
    }

    #[test]
    fn test_encounter_dangerous() {
        let ghost = sample_ghost();
        let mut rng = test_rng();
        let (msg, _, _) = encounter_ghost(&ghost, 5, &mut rng);
        assert!(msg.contains("위험"));
    }

    #[test]
    fn test_encounter_safe() {
        let ghost = sample_ghost();
        let mut rng = test_rng();
        let (msg, _, _) = encounter_ghost(&ghost, 20, &mut rng);
        assert!(msg.contains("떠돌고"));
    }

    #[test]
    fn test_drops() {
        let ghost = sample_ghost();
        let mut rng = test_rng();
        let (_, drops, gold) = encounter_ghost(&ghost, 15, &mut rng);
        assert!(gold > 0);
        // 드롭은 확률적이므로 크기만 확인
        assert!(drops.len() <= ghost.inventory.len());
    }

    #[test]
    fn test_danger_extreme() {
        let warn = danger_warning(5, 30);
        assert!(warn.is_some());
        assert!(warn.unwrap().contains("극도로"));
    }

    #[test]
    fn test_danger_safe() {
        let warn = danger_warning(20, 15);
        assert!(warn.is_none());
    }

    #[test]
    fn test_epitaph() {
        let ghost = sample_ghost();
        let text = generate_epitaph(&ghost);
        assert!(text.contains("R.I.P."));
        assert!(text.contains("용사"));
    }

    #[test]
    fn test_gold_drop_range() {
        let ghost = sample_ghost();
        let mut rng = test_rng();
        let (_, _, gold) = encounter_ghost(&ghost, 15, &mut rng);
        assert!(gold >= ghost.gold / 2);
        assert!(gold <= ghost.gold);
    }
}

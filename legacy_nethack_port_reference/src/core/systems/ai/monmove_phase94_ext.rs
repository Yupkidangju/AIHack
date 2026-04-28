// ============================================================================
// [v2.30.0 Phase 94-1] 몬스터 AI 확장 (monmove_phase94_ext.rs)
// 원본: NetHack 3.6.7 src/monmove.c L500-2000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 이동 전략 — movement_strategy (monmove.c L500-900)
// =============================================================================

/// [v2.30.0 94-1] 몬스터 행동 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterBehavior {
    Aggressive, // 적극적 추격
    Cautious,   // 조심스러운 접근
    Fleeing,    // 도주
    Wandering,  // 방랑
    Guarding,   // 영역 수호
    Ambush,     // 매복
    Seeking,    // 탐색 (음식/아이템)
    Returning,  // 귀환 (집으로)
}

/// [v2.30.0 94-1] 이동 결정 입력
#[derive(Debug, Clone)]
pub struct MonMoveInput {
    pub monster_x: i32,
    pub monster_y: i32,
    pub player_x: i32,
    pub player_y: i32,
    pub monster_hp_pct: i32,
    pub monster_level: i32,
    pub is_hostile: bool,
    pub is_tame: bool,
    pub is_peaceful: bool,
    pub can_see_player: bool,
    pub has_ranged_attack: bool,
    pub flee_timer: i32,
    pub is_afraid: bool,
}

/// [v2.30.0 94-1] 이동 결정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonMoveResult {
    pub behavior: MonsterBehavior,
    pub target_x: i32,
    pub target_y: i32,
    pub dx: i32,
    pub dy: i32,
    pub special_action: Option<String>,
}

/// [v2.30.0 94-1] 몬스터 이동 전략 결정
/// 원본: monmove.c mon_strategy()
pub fn decide_movement(input: &MonMoveInput, rng: &mut NetHackRng) -> MonMoveResult {
    let dx = (input.player_x - input.monster_x).signum();
    let dy = (input.player_y - input.monster_y).signum();
    let dist = ((input.player_x - input.monster_x).abs() + (input.player_y - input.monster_y).abs())
        as i32;

    // 도주 상태
    if input.is_afraid || input.flee_timer > 0 || input.monster_hp_pct < 15 {
        return MonMoveResult {
            behavior: MonsterBehavior::Fleeing,
            target_x: input.monster_x - dx,
            target_y: input.monster_y - dy,
            dx: -dx,
            dy: -dy,
            special_action: None,
        };
    }

    // 길들인 몬스터 → 플레이어 따라감
    if input.is_tame {
        if dist > 3 {
            return MonMoveResult {
                behavior: MonsterBehavior::Returning,
                target_x: input.player_x,
                target_y: input.player_y,
                dx,
                dy,
                special_action: None,
            };
        }
        return MonMoveResult {
            behavior: MonsterBehavior::Guarding,
            target_x: input.monster_x,
            target_y: input.monster_y,
            dx: 0,
            dy: 0,
            special_action: None,
        };
    }

    // 평화적 → 방랑
    if input.is_peaceful {
        let wdx = rng.rn2(3) - 1;
        let wdy = rng.rn2(3) - 1;
        return MonMoveResult {
            behavior: MonsterBehavior::Wandering,
            target_x: input.monster_x + wdx,
            target_y: input.monster_y + wdy,
            dx: wdx,
            dy: wdy,
            special_action: None,
        };
    }

    // 적대적
    if !input.can_see_player {
        // 플레이어 안보임 → 탐색/방랑
        let wdx = rng.rn2(3) - 1;
        let wdy = rng.rn2(3) - 1;
        return MonMoveResult {
            behavior: MonsterBehavior::Seeking,
            target_x: input.monster_x + wdx,
            target_y: input.monster_y + wdy,
            dx: wdx,
            dy: wdy,
            special_action: None,
        };
    }

    // 원거리 공격 가능 + 적절한 거리
    if input.has_ranged_attack && dist > 2 && dist <= 8 {
        return MonMoveResult {
            behavior: MonsterBehavior::Cautious,
            target_x: input.monster_x,
            target_y: input.monster_y,
            dx: 0,
            dy: 0,
            special_action: Some("원거리 공격".to_string()),
        };
    }

    // HP 낮으면 조심스럽게
    if input.monster_hp_pct < 40 {
        return MonMoveResult {
            behavior: MonsterBehavior::Cautious,
            target_x: input.player_x,
            target_y: input.player_y,
            dx,
            dy,
            special_action: None,
        };
    }

    // 기본: 적극적 추격
    MonMoveResult {
        behavior: MonsterBehavior::Aggressive,
        target_x: input.player_x,
        target_y: input.player_y,
        dx,
        dy,
        special_action: None,
    }
}

// =============================================================================
// [2] 몬스터 아이템 사용 — mon_use_item (monmove.c L1000-1500)
// =============================================================================

/// [v2.30.0 94-1] 몬스터 아이템 사용 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonItemUseResult {
    UsePotion {
        potion_type: String,
    },
    UseWand {
        wand_type: String,
        direction: (i32, i32),
    },
    ThrowItem {
        item: String,
        target: (i32, i32),
    },
    WearArmor {
        slot: String,
    },
    WieldWeapon {
        weapon: String,
    },
    NoAction,
}

/// [v2.30.0 94-1] 몬스터 아이템 사용 판정
/// 원본: monmove.c m_use_item()
pub fn mon_decide_item_use(
    hp_pct: i32,
    has_healing_potion: bool,
    has_wand: bool,
    has_throwable: bool,
    is_in_melee: bool,
    target_dir: (i32, i32),
    rng: &mut NetHackRng,
) -> MonItemUseResult {
    // HP 낮으면 포션 우선
    if hp_pct < 30 && has_healing_potion {
        return MonItemUseResult::UsePotion {
            potion_type: "치유".to_string(),
        };
    }

    // 근접전 아닐 때 원거리
    if !is_in_melee {
        if has_wand && rng.rn2(3) == 0 {
            return MonItemUseResult::UseWand {
                wand_type: "마법 미사일".to_string(),
                direction: target_dir,
            };
        }
        if has_throwable && rng.rn2(2) == 0 {
            return MonItemUseResult::ThrowItem {
                item: "투척물".to_string(),
                target: target_dir,
            };
        }
    }

    MonItemUseResult::NoAction
}

// =============================================================================
// [3] 몬스터 특수 행동 — special_behavior (monmove.c L1500-2000)
// =============================================================================

/// [v2.30.0 94-1] 특수 행동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonSpecialBehavior {
    /// 문 열기/부수기
    OpenDoor { x: i32, y: i32 },
    /// 함정 해제
    DisarmTrap { x: i32, y: i32 },
    /// 아이템 줍기
    PickupItem { x: i32, y: i32, item: String },
    /// 동료 소환
    CallForHelp { count: i32 },
    /// 시체 먹기
    EatCorpse { heal_amount: i32 },
    /// 텔레포트
    TeleportSelf,
    /// 숨기
    Hide,
    /// 특수 행동 없음
    None,
}

/// [v2.30.0 94-1] 특수 행동 판정
pub fn mon_special_behavior(
    can_open_doors: bool,
    can_teleport: bool,
    can_hide: bool,
    near_door: bool,
    near_item: bool,
    hp_pct: i32,
    monster_level: i32,
    rng: &mut NetHackRng,
) -> MonSpecialBehavior {
    // HP 극히 낮으면 텔레포트 도주
    if hp_pct < 10 && can_teleport {
        return MonSpecialBehavior::TeleportSelf;
    }

    // 아이템 근처면 줍기
    if near_item && rng.rn2(3) == 0 {
        return MonSpecialBehavior::PickupItem {
            x: 0,
            y: 0,
            item: "아이템".to_string(),
        };
    }

    // 문 근처면 열기
    if near_door && can_open_doors {
        return MonSpecialBehavior::OpenDoor { x: 0, y: 0 };
    }

    // 높은 레벨 → 도움 부르기
    if monster_level > 15 && rng.rn2(10) == 0 {
        return MonSpecialBehavior::CallForHelp {
            count: rng.rn2(3) + 1,
        };
    }

    // 숨기 가능
    if can_hide && rng.rn2(5) == 0 {
        return MonSpecialBehavior::Hide;
    }

    MonSpecialBehavior::None
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

    fn base_input() -> MonMoveInput {
        MonMoveInput {
            monster_x: 10,
            monster_y: 10,
            player_x: 15,
            player_y: 10,
            monster_hp_pct: 80,
            monster_level: 5,
            is_hostile: true,
            is_tame: false,
            is_peaceful: false,
            can_see_player: true,
            has_ranged_attack: false,
            flee_timer: 0,
            is_afraid: false,
        }
    }

    #[test]
    fn test_aggressive() {
        let mut rng = test_rng();
        let input = base_input();
        let result = decide_movement(&input, &mut rng);
        assert_eq!(result.behavior, MonsterBehavior::Aggressive);
        assert_eq!(result.dx, 1); // 플레이어 방향
    }

    #[test]
    fn test_fleeing() {
        let mut rng = test_rng();
        let mut input = base_input();
        input.is_afraid = true;
        let result = decide_movement(&input, &mut rng);
        assert_eq!(result.behavior, MonsterBehavior::Fleeing);
        assert_eq!(result.dx, -1); // 반대 방향
    }

    #[test]
    fn test_tame_follow() {
        let mut rng = test_rng();
        let mut input = base_input();
        input.is_tame = true;
        input.is_hostile = false;
        let result = decide_movement(&input, &mut rng);
        assert_eq!(result.behavior, MonsterBehavior::Returning);
    }

    #[test]
    fn test_peaceful_wander() {
        let mut rng = test_rng();
        let mut input = base_input();
        input.is_peaceful = true;
        input.is_hostile = false;
        let result = decide_movement(&input, &mut rng);
        assert_eq!(result.behavior, MonsterBehavior::Wandering);
    }

    #[test]
    fn test_ranged_cautious() {
        let mut rng = test_rng();
        let mut input = base_input();
        input.has_ranged_attack = true;
        let result = decide_movement(&input, &mut rng);
        assert_eq!(result.behavior, MonsterBehavior::Cautious);
        assert!(result.special_action.is_some());
    }

    #[test]
    fn test_item_use_heal() {
        let mut rng = test_rng();
        let result = mon_decide_item_use(20, true, false, false, false, (1, 0), &mut rng);
        assert!(matches!(result, MonItemUseResult::UsePotion { .. }));
    }

    #[test]
    fn test_item_use_no_action() {
        let mut rng = test_rng();
        let result = mon_decide_item_use(80, false, false, false, true, (1, 0), &mut rng);
        assert!(matches!(result, MonItemUseResult::NoAction));
    }

    #[test]
    fn test_special_teleport() {
        let mut rng = test_rng();
        let result = mon_special_behavior(false, true, false, false, false, 5, 10, &mut rng);
        assert!(matches!(result, MonSpecialBehavior::TeleportSelf));
    }

    #[test]
    fn test_special_open_door() {
        let mut rng = test_rng();
        let result = mon_special_behavior(true, false, false, true, false, 80, 5, &mut rng);
        assert!(matches!(result, MonSpecialBehavior::OpenDoor { .. }));
    }
}

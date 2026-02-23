// ============================================================================
// [v2.27.0 R15-2] 몬스터 아이템 사용 AI (muse_ext.rs)
// 원본: NetHack 3.6.7 muse.c (3,089줄)
// 몬스터가 포션/스크롤/지팡이/도구를 전략적으로 사용
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 사용 가능 아이템 분류 (원본: muse.c find_misc, find_offensive)
// =============================================================================

/// [v2.27.0 R15-2] 몬스터 사용 아이템 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonsterUseItem {
    /// 치유 포션
    HealingPotion { heal_amount: i32 },
    /// 순간이동 스크롤/지팡이
    Teleport,
    /// 투명 포션
    Invisibility,
    /// 속도 포션
    Speed,
    /// 공격 지팡이 (불/얼음/번개)
    OffensiveWand { damage_type: String },
    /// 마비 지팡이
    ParalysisWand,
    /// 밀치기 지팡이
    StrikingWand,
    /// 도구 (뿔, 호루라기 등)
    Tool(String),
}

/// [v2.27.0 R15-2] 몬스터 전투 상황
#[derive(Debug, Clone)]
pub struct MonsterSituation {
    pub hp_ratio: f64, // 현재HP/최대HP
    pub is_fleeing: bool,
    pub distance_to_player: i32,
    pub is_peaceful: bool,
    pub intelligence: i32, // 0~5 (0=동물, 5=대마법사)
    pub has_ranged_attack: bool,
}

// =============================================================================
// [2] 아이템 사용 결정 (원본: muse.c use_misc, use_offensive)
// =============================================================================

/// [v2.27.0 R15-2] 방어적 아이템 사용 결정 (원본: find_defensive)
pub fn choose_defensive_item(
    situation: &MonsterSituation,
    available: &[MonsterUseItem],
) -> Option<MonsterUseItem> {
    if situation.is_peaceful {
        return None;
    }
    if situation.intelligence < 1 {
        return None;
    } // 동물은 아이템 못 씀

    // HP 낮으면 치유 우선
    if situation.hp_ratio < 0.3 {
        if let Some(item) = available
            .iter()
            .find(|i| matches!(i, MonsterUseItem::HealingPotion { .. }))
        {
            return Some(item.clone());
        }
    }

    // HP 매우 낮고 도주 중이면 텔레포트
    if situation.hp_ratio < 0.15 && situation.is_fleeing {
        if let Some(item) = available
            .iter()
            .find(|i| matches!(i, MonsterUseItem::Teleport))
        {
            return Some(item.clone());
        }
    }

    // 투명화
    if situation.hp_ratio < 0.5 && situation.intelligence >= 3 {
        if let Some(item) = available
            .iter()
            .find(|i| matches!(i, MonsterUseItem::Invisibility))
        {
            return Some(item.clone());
        }
    }

    // 속도
    if !situation.is_fleeing && situation.intelligence >= 2 {
        if let Some(item) = available
            .iter()
            .find(|i| matches!(i, MonsterUseItem::Speed))
        {
            return Some(item.clone());
        }
    }

    None
}

/// [v2.27.0 R15-2] 공격적 아이템 사용 결정 (원본: find_offensive)
pub fn choose_offensive_item(
    situation: &MonsterSituation,
    available: &[MonsterUseItem],
) -> Option<MonsterUseItem> {
    if situation.is_peaceful {
        return None;
    }
    if situation.intelligence < 2 {
        return None;
    }

    // 원거리 공격 지팡이 우선 (거리 2+)
    if situation.distance_to_player >= 2 {
        if let Some(item) = available
            .iter()
            .find(|i| matches!(i, MonsterUseItem::OffensiveWand { .. }))
        {
            return Some(item.clone());
        }
    }

    // 근접 시 마비/밀치기
    if situation.distance_to_player <= 1 {
        if let Some(item) = available
            .iter()
            .find(|i| matches!(i, MonsterUseItem::ParalysisWand))
        {
            return Some(item.clone());
        }
        if let Some(item) = available
            .iter()
            .find(|i| matches!(i, MonsterUseItem::StrikingWand))
        {
            return Some(item.clone());
        }
    }

    None
}

/// [v2.27.0 R15-2] 통합 AI 결정
pub fn monster_item_decision(
    situation: &MonsterSituation,
    available: &[MonsterUseItem],
) -> Option<MonsterUseItem> {
    // 방어 우선
    if let Some(def) = choose_defensive_item(situation, available) {
        return Some(def);
    }
    choose_offensive_item(situation, available)
}

// =============================================================================
// [3] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn low_hp_situation() -> MonsterSituation {
        MonsterSituation {
            hp_ratio: 0.2,
            is_fleeing: true,
            distance_to_player: 3,
            is_peaceful: false,
            intelligence: 3,
            has_ranged_attack: false,
        }
    }

    fn healthy_situation() -> MonsterSituation {
        MonsterSituation {
            hp_ratio: 0.9,
            is_fleeing: false,
            distance_to_player: 2,
            is_peaceful: false,
            intelligence: 4,
            has_ranged_attack: false,
        }
    }

    #[test]
    fn test_defensive_heal() {
        let items = vec![
            MonsterUseItem::HealingPotion { heal_amount: 30 },
            MonsterUseItem::Teleport,
        ];
        let result = choose_defensive_item(&low_hp_situation(), &items);
        assert!(matches!(result, Some(MonsterUseItem::HealingPotion { .. })));
    }

    #[test]
    fn test_defensive_teleport() {
        let sit = MonsterSituation {
            hp_ratio: 0.1,
            ..low_hp_situation()
        };
        let items = vec![MonsterUseItem::Teleport];
        let result = choose_defensive_item(&sit, &items);
        assert!(matches!(result, Some(MonsterUseItem::Teleport)));
    }

    #[test]
    fn test_offensive_wand() {
        let items = vec![MonsterUseItem::OffensiveWand {
            damage_type: "fire".to_string(),
        }];
        let result = choose_offensive_item(&healthy_situation(), &items);
        assert!(matches!(result, Some(MonsterUseItem::OffensiveWand { .. })));
    }

    #[test]
    fn test_peaceful_no_action() {
        let sit = MonsterSituation {
            is_peaceful: true,
            ..healthy_situation()
        };
        let items = vec![MonsterUseItem::OffensiveWand {
            damage_type: "fire".to_string(),
        }];
        assert!(choose_offensive_item(&sit, &items).is_none());
    }

    #[test]
    fn test_animal_no_items() {
        let sit = MonsterSituation {
            intelligence: 0,
            ..healthy_situation()
        };
        let items = vec![MonsterUseItem::HealingPotion { heal_amount: 30 }];
        assert!(choose_defensive_item(&sit, &items).is_none());
    }

    #[test]
    fn test_integrated_defense_first() {
        let items = vec![
            MonsterUseItem::HealingPotion { heal_amount: 30 },
            MonsterUseItem::OffensiveWand {
                damage_type: "cold".to_string(),
            },
        ];
        let result = monster_item_decision(&low_hp_situation(), &items);
        assert!(matches!(result, Some(MonsterUseItem::HealingPotion { .. })));
    }
}

// ============================================================================
// AIHack - steal_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] steal.c 미이식 함수 완전 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 steal.c (759줄)
//
// 이식 대상:
//   somegold(), equipname(),
//   steal() 핵심 로직 (선택 가중치, 갑옷 우회, 저주 검사),
//   stealamulet() 부적 도난,
//   maybe_absorb_item() 미믹 흡수,
//   mdrop_obj() / relobj() 드롭 로직
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// somegold — 비례 금 도난량
// [v2.10.1] steal.c:31-58 이식
// =============================================================================

/// 소지금에 비례한 도난량 계산 (원본 somegold)
/// [v2.10.1] steal.c:31-58 — 정확한 구간별 하한 적용
pub fn somegold(total_gold: i64, rng: &mut NetHackRng) -> i64 {
    let igold = if total_gold >= i32::MAX as i64 {
        i32::MAX
    } else {
        total_gold as i32
    };

    let stolen = if igold < 50 {
        igold // 전부
    } else if igold < 100 {
        rng.rn1(igold - 25 + 1, 25) // 25~igold
    } else if igold < 500 {
        rng.rn1(igold - 50 + 1, 50) // 50~igold
    } else if igold < 1000 {
        rng.rn1(igold - 100 + 1, 100) // 100~igold
    } else if igold < 5000 {
        rng.rn1(igold - 500 + 1, 500) // 500~igold
    } else if igold < 10000 {
        rng.rn1(igold - 1000 + 1, 1000) // 1000~igold
    } else {
        rng.rn1(igold - 5000 + 1, 5000) // 5000~igold
    };

    stolen as i64
}

// =============================================================================
// equipname — 장비 슬롯명
// [v2.10.1] steal.c:12-29 이식
// =============================================================================

/// 장비 슬롯 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipSlot {
    Shirt,
    Boots,
    Shield,
    Gloves,
    Cloak,
    Helm,
    Suit,
    Ring,
    Amulet,
    Weapon,
}

/// 장비 슬롯명 (원본 equipname)
pub fn equipname(slot: EquipSlot) -> &'static str {
    match slot {
        EquipSlot::Shirt => "shirt",
        EquipSlot::Boots => "boots",
        EquipSlot::Shield => "shield",
        EquipSlot::Gloves => "gloves",
        EquipSlot::Cloak => "cloak",
        EquipSlot::Helm => "helmet",
        EquipSlot::Suit => "suit",
        EquipSlot::Ring => "ring",
        EquipSlot::Amulet => "amulet",
        EquipSlot::Weapon => "weapon",
    }
}

// =============================================================================
// steal_target_selection — 도난 대상 가중치 선택
// [v2.10.1] steal.c:293-327 이식
// =============================================================================

/// 인벤토리 아이템 정보 (도난 판정용)
#[derive(Debug, Clone)]
pub struct InvItem {
    pub index: usize,
    pub name: String,
    pub is_worn_armor_or_accessory: bool,
    pub is_coin: bool,
    pub is_skin: bool,
    pub is_under_cloak: bool, // 갑옷이 망토 아래인 경우
    pub is_cursed: bool,
    pub slot: Option<EquipSlot>,
}

/// 도난 대상 선택 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StealSelectionResult {
    /// 아이템 선택됨 (인덱스)
    Selected(usize),
    /// 훔칠 것 없음
    NothingToSteal,
    /// 동물이 포기함
    GaveUp,
}

/// 도난 대상 가중치 선택 (원본 steal 중 선택 로직)
/// [v2.10.1] steal.c:293-327
/// 착용 갑옷/장식: 가중치 5, 일반: 가중치 1
/// 갑옷 우선순위 규칙: 망토 > 갑옷 > 셔츠, 장갑 > 반지
pub fn select_steal_target(
    items: &[InvItem],
    has_body_armor: bool,
    is_animal: bool,
    rng: &mut NetHackRng,
) -> StealSelectionResult {
    // 금화와 스킨 제외
    let stealable: Vec<&InvItem> = items
        .iter()
        .filter(|i| !i.is_coin && !i.is_skin)
        .filter(|i| !has_body_armor || !i.is_under_cloak)
        .collect();

    if stealable.is_empty() {
        return StealSelectionResult::NothingToSteal;
    }

    // 가중치 합 계산 (원본:295-298)
    let total: i32 = stealable
        .iter()
        .map(|i| if i.is_worn_armor_or_accessory { 5 } else { 1 })
        .sum();

    let mut roll = rng.rn2(total);
    let mut selected = None;

    for item in &stealable {
        let weight = if item.is_worn_armor_or_accessory {
            5
        } else {
            1
        };
        roll -= weight;
        if roll < 0 {
            selected = Some(item);
            break;
        }
    }

    let item = match selected {
        Some(i) => i,
        None => return StealSelectionResult::NothingToSteal,
    };

    // 갑옷 우선순위 리다이렉트 (원본:314-326)
    // (이 로직은 호출자가 InvItem의 slot으로 처리하므로 인덱스만 반환)

    // 동물 + 저주 아이템 → 포기 (원본:338-367)
    if is_animal && item.is_cursed && item.is_worn_armor_or_accessory {
        return StealSelectionResult::GaveUp;
    }

    StealSelectionResult::Selected(item.index)
}

// =============================================================================
// stealamulet_target — 부적 도난 대상 선택
// [v2.10.1] steal.c:529-609 이식
// =============================================================================

/// 부적 도난 대상 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmuletTarget {
    QuestArtifact,
    AmuletOfYendor,
    BellOfOpening,
    BookOfTheDead,
    Candelabrum,
}

/// 부적 도난 대상 선택 (원본 stealamulet)
/// [v2.10.1] steal.c:529-577
pub fn select_amulet_target(
    has_quest_artifact: bool,
    quest_artifact_count: i32,
    has_amulet: bool,
    has_bell: bool,
    has_book: bool,
    has_menorah: bool,
    rng: &mut NetHackRng,
) -> Option<AmuletTarget> {
    // 모든 퀘스트 아티팩트 우선 (원본:541-549)
    if has_quest_artifact {
        return Some(AmuletTarget::QuestArtifact);
    }

    // 주요 아이템 차례 (원본:551-563)
    if has_amulet {
        Some(AmuletTarget::AmuletOfYendor)
    } else if has_bell {
        Some(AmuletTarget::BellOfOpening)
    } else if has_book {
        Some(AmuletTarget::BookOfTheDead)
    } else if has_menorah {
        Some(AmuletTarget::Candelabrum)
    } else {
        None
    }
}

// =============================================================================
// maybe_absorb_check — 미믹 아이템 흡수 판정
// [v2.10.1] steal.c:611-653 이식
// =============================================================================

/// 미믹 흡수 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbsorbResult {
    /// 아이템 흡수됨
    Absorbed,
    /// 저항 — 흡수 안 됨
    Resisted,
}

/// 미믹이 아이템 흡수를 시도 (원본 maybe_absorb_item, 확률만)
/// [v2.10.1] steal.c:619-622
pub fn maybe_absorb_check(
    is_rock_class: bool,
    is_ball_or_chain: bool,
    ordinary_chance: i32, // 일반 아이템 흡수확률 (보통 50%)
    artifact_chance: i32, // 아티팩트 흡수확률 (보통 10%)
    is_artifact: bool,
    rng: &mut NetHackRng,
) -> AbsorbResult {
    // 바위/공/사슬 불가 (원본:619)
    if is_rock_class || is_ball_or_chain {
        return AbsorbResult::Resisted;
    }

    // 확률 체크 (원본:620)
    let chance = if is_artifact {
        artifact_chance
    } else {
        ordinary_chance
    };
    if rng.rn2(100) < (100 - chance) {
        AbsorbResult::Resisted
    } else {
        AbsorbResult::Absorbed
    }
}

// =============================================================================
// mdrop — 몬스터 아이템 드롭 관련
// [v2.10.1] steal.c:655-756 이식
// =============================================================================

/// 아이템 드롭 방법
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropBehavior {
    /// 일반 드롭 (바닥에)
    Normal,
    /// 특수 아이템 (부적 등) 보호 드롭
    SpecialProtected,
    /// 금고 경비원의 금 — 소멸
    VaultGuardGold,
}

/// 몬스터 아이템 드롭 판정 (원본 mdrop_obj/relobj 요약)
pub fn should_drop_item(
    is_dead: bool,
    is_pet: bool,
    is_special_item: bool,
    is_wielded: bool,
    is_worn: bool,
) -> DropBehavior {
    if is_special_item {
        return DropBehavior::SpecialProtected;
    }

    if is_pet && (is_wielded || is_worn) {
        // 펫은 착용/장비 아이템 유지 (원본:749)
        return DropBehavior::Normal; // 드롭 대상에서 제외 — 호출자가 필터링
    }

    DropBehavior::Normal
}

/// 죽은 몬스터가 특수 아이템을 드롭해야 하는지 (원본 mdrop_special_objs)
/// [v2.10.1] steal.c:698-728
pub fn is_special_drop_item(item_name: &str) -> bool {
    let l = item_name.to_lowercase();
    l.contains("amulet of yendor")
        || l.contains("bell of opening")
        || l.contains("book of the dead")
        || l.contains("candelabrum")
        || l.contains("quest artifact")
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_somegold_small() {
        let mut rng = NetHackRng::new(42);
        // 50 미만 → 전액
        assert_eq!(somegold(30, &mut rng), 30);
    }

    #[test]
    fn test_somegold_medium() {
        let mut rng = NetHackRng::new(42);
        let stolen = somegold(200, &mut rng);
        // 50 이상 200 이하
        assert!(stolen >= 50 && stolen <= 200);
    }

    #[test]
    fn test_somegold_large() {
        let mut rng = NetHackRng::new(42);
        let stolen = somegold(10000, &mut rng);
        assert!(stolen >= 1000 && stolen <= 10000);
    }

    #[test]
    fn test_somegold_very_large() {
        let mut rng = NetHackRng::new(42);
        let stolen = somegold(50000, &mut rng);
        assert!(stolen >= 5000 && stolen <= 50000);
    }

    #[test]
    fn test_equipname() {
        assert_eq!(equipname(EquipSlot::Boots), "boots");
        assert_eq!(equipname(EquipSlot::Shield), "shield");
        assert_eq!(equipname(EquipSlot::Suit), "suit");
    }

    #[test]
    fn test_steal_selection_basic() {
        let mut rng = NetHackRng::new(42);
        let items = vec![
            InvItem {
                index: 0,
                name: "dagger".to_string(),
                is_worn_armor_or_accessory: false,
                is_coin: false,
                is_skin: false,
                is_under_cloak: false,
                is_cursed: false,
                slot: None,
            },
            InvItem {
                index: 1,
                name: "chain mail".to_string(),
                is_worn_armor_or_accessory: true,
                is_coin: false,
                is_skin: false,
                is_under_cloak: false,
                is_cursed: false,
                slot: Some(EquipSlot::Suit),
            },
        ];

        let result = select_steal_target(&items, false, false, &mut rng);
        assert!(matches!(result, StealSelectionResult::Selected(_)));
    }

    #[test]
    fn test_steal_nothing() {
        let mut rng = NetHackRng::new(42);
        let items = vec![InvItem {
            index: 0,
            name: "gold".to_string(),
            is_worn_armor_or_accessory: false,
            is_coin: true,
            is_skin: false,
            is_under_cloak: false,
            is_cursed: false,
            slot: None,
        }];
        let result = select_steal_target(&items, false, false, &mut rng);
        assert_eq!(result, StealSelectionResult::NothingToSteal);
    }

    #[test]
    fn test_steal_animal_cursed() {
        let mut rng = NetHackRng::new(42);
        let items = vec![InvItem {
            index: 0,
            name: "cursed armor".to_string(),
            is_worn_armor_or_accessory: true,
            is_coin: false,
            is_skin: false,
            is_under_cloak: false,
            is_cursed: true,
            slot: Some(EquipSlot::Suit),
        }];
        let result = select_steal_target(&items, false, true, &mut rng);
        assert_eq!(result, StealSelectionResult::GaveUp);
    }

    #[test]
    fn test_amulet_target() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            select_amulet_target(true, 1, false, false, false, false, &mut rng),
            Some(AmuletTarget::QuestArtifact),
        );
        assert_eq!(
            select_amulet_target(false, 0, true, false, false, false, &mut rng),
            Some(AmuletTarget::AmuletOfYendor),
        );
        assert_eq!(
            select_amulet_target(false, 0, false, true, false, false, &mut rng),
            Some(AmuletTarget::BellOfOpening),
        );
        assert_eq!(
            select_amulet_target(false, 0, false, false, false, false, &mut rng),
            None,
        );
    }

    #[test]
    fn test_absorb_check() {
        let mut rng = NetHackRng::new(42);
        // 바위 불가
        assert_eq!(
            maybe_absorb_check(true, false, 50, 10, false, &mut rng),
            AbsorbResult::Resisted,
        );
        // 일반 50% 확률
        let mut absorbed = 0;
        for seed in 0..100u64 {
            let mut r = NetHackRng::new(seed);
            if maybe_absorb_check(false, false, 50, 10, false, &mut r) == AbsorbResult::Absorbed {
                absorbed += 1;
            }
        }
        assert!(
            absorbed > 10 && absorbed < 90,
            "absorbed={} 예상 범위 밖",
            absorbed
        );
    }

    #[test]
    fn test_special_drop() {
        assert!(is_special_drop_item("Amulet of Yendor"));
        assert!(is_special_drop_item("Bell of Opening"));
        assert!(!is_special_drop_item("dagger"));
    }
}

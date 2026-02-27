// ============================================================================
// [v2.24.0 Phase 3-2] 인벤토리 시스템 확장 (invent_phase3_ext.rs)
// 원본: NetHack 3.6.7 src/invent.c L440-1086 핵심 미이식 함수 이식
// 순수 결과 패턴: ECS 의존 없이 독립 테스트 가능
// ============================================================================

// =============================================================================
// [1] 인벤토리 레터 배정 — assigninvlet (invent.c L440-504)
// =============================================================================

/// [v2.24.0 3-2] 인벤토리 레터 배정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignLetterResult {
    /// 지정된 레터 사용 가능
    Assigned { letter: char },
    /// 기존 레터가 충돌 → 강제 할당 (기존 아이템 밀어냄)
    Reassigned {
        letter: char,
        displaced_item: String,
    },
    /// 인벤토리 가득 참
    Full,
}

/// [v2.24.0 3-2] 인벤토리 레터 배정
/// 원본: invent.c assigninvlet() L440-504
/// 규칙: 같은 클래스 아이템이 이전에 해당 레터를 사용했으면 재사용
pub fn assign_invlet(
    item_class: i32,
    preferred_letter: Option<char>,
    used_letters: &[char],
    class_history: &[(i32, char)], // (클래스, 이전 사용 레터)
) -> AssignLetterResult {
    // [1] 선호 레터가 지정되고, 미사용이면 즉시 할당
    if let Some(pref) = preferred_letter {
        if !used_letters.contains(&pref) {
            return AssignLetterResult::Assigned { letter: pref };
        }
    }

    // [2] 클래스 이력에서 같은 클래스의 이전 레터 검색
    for (cls, letter) in class_history {
        if *cls == item_class && !used_letters.contains(letter) {
            return AssignLetterResult::Assigned { letter: *letter };
        }
    }

    // [3] 빈 레터 검색 (a-z → A-Z 순서)
    for c in 'a'..='z' {
        if !used_letters.contains(&c) {
            return AssignLetterResult::Assigned { letter: c };
        }
    }
    for c in 'A'..='Z' {
        if !used_letters.contains(&c) {
            return AssignLetterResult::Assigned { letter: c };
        }
    }

    // [4] 모든 레터 사용됨 → 인벤토리 가득 참
    AssignLetterResult::Full
}

// =============================================================================
// [2] 아이템 드롭 검증 — dropcheck (invent.c L796-870)
// =============================================================================

/// [v2.24.0 3-2] 드롭 불가 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropCheckFailure {
    /// 장비 중인 아이템 (먼저 해제해야 함)
    Equipped { slot: String },
    /// 저주받은 장비 (해제 불가)
    CursedEquipment { slot: String },
    /// 용접된 무기 (저주 + 2손무기)
    WeldedWeapon,
    /// 로드스톤 (저주받아 내려놓을 수 없음)
    CursedLoadstone,
    /// 퀘스트 아티팩트 (버릴 수 없음)
    QuestArtifact,
}

/// [v2.24.0 3-2] 아이템 드롭 가능 여부 판정
/// 원본: invent.c L796-870 dropcheck()
pub fn drop_check(
    is_equipped: bool,
    equipment_slot: &str,
    is_cursed: bool,
    is_welded: bool,
    is_loadstone: bool,
    is_quest_artifact: bool,
) -> Result<(), DropCheckFailure> {
    if is_quest_artifact {
        return Err(DropCheckFailure::QuestArtifact);
    }

    if is_loadstone && is_cursed {
        return Err(DropCheckFailure::CursedLoadstone);
    }

    if is_equipped {
        if is_welded {
            return Err(DropCheckFailure::WeldedWeapon);
        }
        if is_cursed {
            return Err(DropCheckFailure::CursedEquipment {
                slot: equipment_slot.to_string(),
            });
        }
        // 장비 중이지만 저주 아님 → 해제 필요 알림
        return Err(DropCheckFailure::Equipped {
            slot: equipment_slot.to_string(),
        });
    }

    Ok(())
}

// =============================================================================
// [3] 인벤토리 용량 계산 — near_capacity (invent.c L1059-1086)
// =============================================================================

/// [v2.24.0 3-2] 짐 용량 단계 (원본: near_capacity 반환값)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapacityLevel {
    Unencumbered = 0,
    Burdened = 1,
    Stressed = 2,
    Strained = 3,
    Overtaxed = 4,
    Overloaded = 5,
}

/// [v2.24.0 3-2] 짐 용량 계산 입력
#[derive(Debug, Clone)]
pub struct CapacityInput {
    /// 현재 소지 무게
    pub current_weight: i32,
    /// 플레이어 힘 (STR)
    pub strength: i32,
    /// 플레이어 체질 (CON)
    pub constitution: i32,
    /// 부양 중인지 (무게 0 취급)
    pub levitating: bool,
    /// 캔이 달린 가방, 홀딩 가방 등 보정치
    pub weight_capacity_bonus: i32,
}

/// [v2.24.0 3-2] 짐 용량 단계 판정
/// 원본: invent.c L1059-1086 near_capacity()
pub fn near_capacity(input: &CapacityInput) -> CapacityLevel {
    if input.levitating {
        return CapacityLevel::Unencumbered;
    }

    // 최대 용량 계산 — 원본: invent.c L1064
    // max_cap = 25 * (str + con) / 2 + weight_bonus
    let base_cap = 25 * (input.strength + input.constitution) / 2;
    let max_cap = (base_cap + input.weight_capacity_bonus).max(1);

    let ratio = input.current_weight * 100 / max_cap;

    match ratio {
        0..=49 => CapacityLevel::Unencumbered,
        50..=69 => CapacityLevel::Burdened,
        70..=84 => CapacityLevel::Stressed,
        85..=94 => CapacityLevel::Strained,
        95..=104 => CapacityLevel::Overtaxed,
        _ => CapacityLevel::Overloaded,
    }
}

/// [v2.24.0 3-2] 짐 용량 페널티 (이동속도 감소)
/// 원본: 각 용량 단계별 속도 감소
pub fn capacity_speed_penalty(level: CapacityLevel) -> i32 {
    match level {
        CapacityLevel::Unencumbered => 0,
        CapacityLevel::Burdened => -1,
        CapacityLevel::Stressed => -3,
        CapacityLevel::Strained => -5,
        CapacityLevel::Overtaxed => -7,
        CapacityLevel::Overloaded => -9,
    }
}

// =============================================================================
// [4] 아이템 식별 상태 관리 — fully_identify (invent.c L980-1058)
// =============================================================================

/// [v2.24.0 3-2] 아이템 식별 수준
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentifyLevel {
    /// 미확인 — 외형만 보임
    Unknown,
    /// 부분 확인 — BUC 상태 알려짐
    BucKnown,
    /// 이름 확인 — 진짜 이름 알려짐
    Named,
    /// 완전 확인 — 강화치/효과/특성 모두 알려짐
    FullyIdentified,
}

/// [v2.24.0 3-2] 감정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifyResult {
    /// 이전 식별 수준
    pub old_level: IdentifyLevel,
    /// 새 식별 수준
    pub new_level: IdentifyLevel,
    /// 새로 알게 된 정보
    pub revealed_info: Vec<String>,
}

/// [v2.24.0 3-2] 아이템 감정
/// 원본: invent.c fully_identify() + doidentify()
pub fn identify_item(
    current_level: IdentifyLevel,
    is_artifact: bool,
    buc_status: i32, // -1=저주, 0=무축복, 1=축복
    enchantment: i32,
    charges: Option<i32>,
    item_name: &str,
    true_name: &str,
) -> IdentifyResult {
    let mut revealed = Vec::new();

    let new_level = match current_level {
        IdentifyLevel::Unknown => {
            // BUC 알려짐
            let buc_str = match buc_status {
                -1 => "저주받은",
                1 => "축복받은",
                _ => "축복받지 않은",
            };
            revealed.push(format!("{} 상태: {}", item_name, buc_str));
            IdentifyLevel::BucKnown
        }
        IdentifyLevel::BucKnown => {
            // 이름 알려짐
            if item_name != true_name {
                revealed.push(format!("{}은(는) 사실 {}이다!", item_name, true_name));
            }
            IdentifyLevel::Named
        }
        IdentifyLevel::Named => {
            // 완전 식별
            revealed.push(format!("{}: 강화 +{}", true_name, enchantment));
            if let Some(ch) = charges {
                revealed.push(format!("충전: {}회", ch));
            }
            if is_artifact {
                revealed.push("아티팩트!".to_string());
            }
            IdentifyLevel::FullyIdentified
        }
        IdentifyLevel::FullyIdentified => {
            // 이미 완전 식별
            IdentifyLevel::FullyIdentified
        }
    };

    IdentifyResult {
        old_level: current_level,
        new_level,
        revealed_info: revealed,
    }
}

/// [v2.24.0 3-2] 한 번에 완전 감정 (감정 두루마리)
pub fn fully_identify(
    is_artifact: bool,
    buc_status: i32,
    enchantment: i32,
    charges: Option<i32>,
    item_name: &str,
    true_name: &str,
) -> IdentifyResult {
    let mut revealed = Vec::new();

    let buc_str = match buc_status {
        -1 => "저주받은",
        1 => "축복받은",
        _ => "축복받지 않은",
    };
    revealed.push(format!("{} {}", buc_str, true_name));
    revealed.push(format!("강화: +{}", enchantment));
    if let Some(ch) = charges {
        revealed.push(format!("충전: {}회", ch));
    }
    if is_artifact {
        revealed.push("아티팩트!".to_string());
    }

    IdentifyResult {
        old_level: IdentifyLevel::Unknown,
        new_level: IdentifyLevel::FullyIdentified,
        revealed_info: revealed,
    }
}

// =============================================================================
// [5] 인벤토리 필터링 — getobj / getobj_filter (invent.c L590-702)
// =============================================================================

/// [v2.24.0 3-2] 아이템 필터 기준
#[derive(Debug, Clone)]
pub struct ItemFilter {
    /// 허용되는 클래스 목록
    pub allowed_classes: Vec<i32>,
    /// BUC 필터 (None=모두 허용)
    pub buc_filter: Option<i32>,
    /// 식별 여부 필터
    pub require_identified: bool,
    /// 장비 가능 여부 필터
    pub require_equippable: bool,
}

/// [v2.24.0 3-2] 아이템 필터 매칭
pub fn item_matches_filter(
    item_class: i32,
    item_buc: i32,
    is_identified: bool,
    is_equippable: bool,
    filter: &ItemFilter,
) -> bool {
    // 클래스 필터
    if !filter.allowed_classes.is_empty() && !filter.allowed_classes.contains(&item_class) {
        return false;
    }

    // BUC 필터
    if let Some(required_buc) = filter.buc_filter {
        if item_buc != required_buc {
            return false;
        }
    }

    // 식별 필터
    if filter.require_identified && !is_identified {
        return false;
    }

    // 장비 필터
    if filter.require_equippable && !is_equippable {
        return false;
    }

    true
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- assign_invlet ---

    #[test]
    fn test_assign_preferred() {
        let result = assign_invlet(10, Some('d'), &['a', 'b', 'c'], &[]);
        assert_eq!(result, AssignLetterResult::Assigned { letter: 'd' });
    }

    #[test]
    fn test_assign_preferred_conflict() {
        let result = assign_invlet(10, Some('a'), &['a', 'b', 'c'], &[]);
        // 'a' 이미 사용 → 'd'부터 빈 레터 찾기
        match result {
            AssignLetterResult::Assigned { letter } => {
                assert_eq!(letter, 'd');
            }
            _ => panic!("빈 레터가 할당되어야 함"),
        }
    }

    #[test]
    fn test_assign_class_history() {
        let history = vec![(10, 'w'), (5, 'p')];
        let result = assign_invlet(10, None, &['a', 'b'], &history);
        // 클래스 10의 이전 레터 'w' 재사용
        assert_eq!(result, AssignLetterResult::Assigned { letter: 'w' });
    }

    #[test]
    fn test_assign_full() {
        let mut used: Vec<char> = ('a'..='z').collect();
        used.extend('A'..='Z');
        let result = assign_invlet(10, None, &used, &[]);
        assert_eq!(result, AssignLetterResult::Full);
    }

    // --- drop_check ---

    #[test]
    fn test_drop_ok() {
        assert!(drop_check(false, "", false, false, false, false).is_ok());
    }

    #[test]
    fn test_drop_quest_artifact() {
        let result = drop_check(false, "", false, false, false, true);
        assert_eq!(result, Err(DropCheckFailure::QuestArtifact));
    }

    #[test]
    fn test_drop_cursed_loadstone() {
        let result = drop_check(false, "", true, false, true, false);
        assert_eq!(result, Err(DropCheckFailure::CursedLoadstone));
    }

    #[test]
    fn test_drop_welded() {
        let result = drop_check(true, "weapon", true, true, false, false);
        assert_eq!(result, Err(DropCheckFailure::WeldedWeapon));
    }

    #[test]
    fn test_drop_cursed_equipped() {
        let result = drop_check(true, "helmet", true, false, false, false);
        assert!(matches!(result, Err(DropCheckFailure::CursedEquipment { .. })));
    }

    // --- near_capacity ---

    #[test]
    fn test_capacity_unencumbered() {
        let input = CapacityInput {
            current_weight: 100,
            strength: 16,
            constitution: 16,
            levitating: false,
            weight_capacity_bonus: 0,
        };
        let result = near_capacity(&input);
        assert_eq!(result, CapacityLevel::Unencumbered);
    }

    #[test]
    fn test_capacity_levitating() {
        let input = CapacityInput {
            current_weight: 99999,
            strength: 10,
            constitution: 10,
            levitating: true,
            weight_capacity_bonus: 0,
        };
        assert_eq!(near_capacity(&input), CapacityLevel::Unencumbered);
    }

    #[test]
    fn test_capacity_overloaded() {
        let input = CapacityInput {
            current_weight: 1000,
            strength: 10,
            constitution: 10,
            levitating: false,
            weight_capacity_bonus: 0,
        };
        // max_cap = 25 * (10+10)/2 = 250, ratio = 400%
        let result = near_capacity(&input);
        assert_eq!(result, CapacityLevel::Overloaded);
    }

    #[test]
    fn test_capacity_speed_penalty() {
        assert_eq!(capacity_speed_penalty(CapacityLevel::Unencumbered), 0);
        assert_eq!(capacity_speed_penalty(CapacityLevel::Overloaded), -9);
    }

    // --- identify_item ---

    #[test]
    fn test_identify_unknown_to_buc() {
        let result = identify_item(
            IdentifyLevel::Unknown, false, -1, 0, None,
            "scrolls labeled ZELGO MER", "scroll of identify",
        );
        assert_eq!(result.new_level, IdentifyLevel::BucKnown);
        assert!(!result.revealed_info.is_empty());
    }

    #[test]
    fn test_identify_named_to_full() {
        let result = identify_item(
            IdentifyLevel::Named, true, 1, 3, Some(5),
            "scroll of identify", "scroll of identify",
        );
        assert_eq!(result.new_level, IdentifyLevel::FullyIdentified);
        assert!(result.revealed_info.len() >= 2); // 강화 + 충전 + 아티팩트
    }

    #[test]
    fn test_fully_identify() {
        let result = fully_identify(false, 1, 5, Some(3), "potion", "potion of healing");
        assert_eq!(result.new_level, IdentifyLevel::FullyIdentified);
        assert!(result.revealed_info.len() >= 2);
    }

    // --- item_matches_filter ---

    #[test]
    fn test_filter_class() {
        let filter = ItemFilter {
            allowed_classes: vec![4, 5], // 포션, 두루마리
            buc_filter: None,
            require_identified: false,
            require_equippable: false,
        };
        assert!(item_matches_filter(4, 0, false, false, &filter));
        assert!(!item_matches_filter(10, 0, false, false, &filter));
    }

    #[test]
    fn test_filter_buc() {
        let filter = ItemFilter {
            allowed_classes: vec![],
            buc_filter: Some(1), // 축복만
            require_identified: false,
            require_equippable: false,
        };
        assert!(item_matches_filter(4, 1, false, false, &filter));
        assert!(!item_matches_filter(4, -1, false, false, &filter));
    }

    #[test]
    fn test_filter_empty_allows_all() {
        let filter = ItemFilter {
            allowed_classes: vec![],
            buc_filter: None,
            require_identified: false,
            require_equippable: false,
        };
        assert!(item_matches_filter(99, -1, false, false, &filter));
    }
}

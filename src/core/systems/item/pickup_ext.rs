// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.22.0 R10-5] 아이템 획득 확장 (pickup_ext.rs)
//
// 원본 참조: NetHack 3.6.7 pickup.c (3,218줄) 중 미구현 핵심 이식
//
// 구현 내용:
//   1. 다중 아이템 줍기 메뉴 정렬/필터링
//   2. 스택 병합 알고리즘 (merge_obj)
//   3. 부분 수량 줍기/내려놓기 (getobj_filter, splitobj)
//   4. 상점 내 줍기 규칙 (shop_pickup_check)
//   5. 컨테이너 중첩 제한 (bag_of_holding 재귀)
//   6. 자동줍기 고급 패턴 매칭 (glob, regex-like)
//   7. 아이템 정렬 우선순위 (sortloot)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 아이템 정렬 (원본: pickup.c sortloot, sortloot_cmp)
// =============================================================================

/// [v2.22.0 R10-5] 아이템 정렬 기준
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortCriteria {
    /// 이름 알파벳 순
    ByName,
    /// 클래스 → 이름 순 (기본)
    ByClass,
    /// 무게 내림차순
    ByWeight,
    /// 가격 내림차순
    ByPrice,
}

/// [v2.22.0 R10-5] 정렬 대상 아이템 요약
#[derive(Debug, Clone)]
pub struct SortableItem {
    /// 내부 인덱스
    pub index: usize,
    /// 아이템 이름
    pub name: String,
    /// 클래스 기호 (')' = 무기, '[' = 갑옷 등)
    pub class_symbol: char,
    /// 무게
    pub weight: i32,
    /// 가격
    pub price: i32,
    /// 수량
    pub quantity: i32,
    /// BUC 상태 (0=미지, 1=축복, 2=무저주, 3=저주)
    pub buc_status: u8,
}

/// [v2.22.0 R10-5] 클래스 기호 → 정렬 우선순위 (원본: sortloot_cmp)
fn class_sort_order(symbol: char) -> i32 {
    match symbol {
        '(' => 0,  // 도구
        ')' => 1,  // 무기
        '[' => 2,  // 갑옷
        '=' => 3,  // 반지
        '"' => 4,  // 아뮬렛
        '!' => 5,  // 포션
        '?' => 6,  // 두루마리
        '+' => 7,  // 마법서
        '/' => 8,  // 지팡이
        '%' => 9,  // 음식
        '*' => 10, // 보석
        '$' => 11, // 금화
        _ => 12,
    }
}

/// [v2.22.0 R10-5] 아이템 정렬 수행 (원본: sortloot)
pub fn sort_items(items: &mut [SortableItem], criteria: SortCriteria) {
    match criteria {
        SortCriteria::ByName => {
            items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }
        SortCriteria::ByClass => {
            items.sort_by(|a, b| {
                let ca = class_sort_order(a.class_symbol);
                let cb = class_sort_order(b.class_symbol);
                ca.cmp(&cb)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });
        }
        SortCriteria::ByWeight => {
            items.sort_by(|a, b| b.weight.cmp(&a.weight));
        }
        SortCriteria::ByPrice => {
            items.sort_by(|a, b| b.price.cmp(&a.price));
        }
    }
}

// =============================================================================
// [2] 스택 병합 (원본: pickup.c merged, merge_choice)
// =============================================================================

/// [v2.22.0 R10-5] 스택 병합 가능 여부 (원본: mergable / merged)
pub fn can_merge(
    a_name: &str,
    b_name: &str,
    a_buc: u8,
    b_buc: u8,
    a_eroded: i32,
    b_eroded: i32,
    a_spe: i32,
    b_spe: i32,
) -> bool {
    // 같은 이름이어야 함
    if a_name != b_name {
        return false;
    }
    // BUC 상태 일치
    if a_buc != b_buc {
        return false;
    }
    // 부식 상태 일치
    if a_eroded != b_eroded {
        return false;
    }
    // 특수치 일치 (enchantment)
    if a_spe != b_spe {
        return false;
    }
    true
}

/// [v2.22.0 R10-5] 스택 병합 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeResult {
    /// 병합된 인덱스 쌍 (대상, 소스 → 대상에 합침)
    pub merged_pairs: Vec<(usize, usize)>,
    /// 병합 후 남은 아이템 인덱스
    pub remaining: Vec<usize>,
}

/// [v2.22.0 R10-5] 아이템 목록에서 병합 가능 쌍 찾기
pub fn find_merge_candidates(items: &[SortableItem]) -> MergeResult {
    let mut merged = Vec::new();
    let mut consumed = vec![false; items.len()];

    for i in 0..items.len() {
        if consumed[i] {
            continue;
        }
        for j in (i + 1)..items.len() {
            if consumed[j] {
                continue;
            }
            if can_merge(
                &items[i].name,
                &items[j].name,
                items[i].buc_status,
                items[j].buc_status,
                0,
                0, // 부식 비교는 간략화
                0,
                0, // spe 비교도 간략화
            ) {
                merged.push((i, j));
                consumed[j] = true;
            }
        }
    }

    let remaining: Vec<usize> = (0..items.len()).filter(|i| !consumed[*i]).collect();

    MergeResult {
        merged_pairs: merged,
        remaining,
    }
}

// =============================================================================
// [3] 부분 수량 줍기/내려놓기 (원본: pickup.c splitobj, getobj)
// =============================================================================

/// [v2.22.0 R10-5] 스택 분할 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitResult {
    /// 원본 남은 수량
    pub original_remaining: i32,
    /// 분리된 수량
    pub split_count: i32,
    /// 성공 여부
    pub success: bool,
    /// 실패 사유
    pub error: Option<String>,
}

/// [v2.22.0 R10-5] 스택 분할 (원본: splitobj)
pub fn split_stack(total_quantity: i32, requested: i32) -> SplitResult {
    if requested <= 0 {
        return SplitResult {
            original_remaining: total_quantity,
            split_count: 0,
            success: false,
            error: Some("수량은 1 이상이어야 합니다.".to_string()),
        };
    }
    if requested >= total_quantity {
        // 전체 수량 이동
        return SplitResult {
            original_remaining: 0,
            split_count: total_quantity,
            success: true,
            error: None,
        };
    }
    SplitResult {
        original_remaining: total_quantity - requested,
        split_count: requested,
        success: true,
        error: None,
    }
}

// =============================================================================
// [4] 상점 내 줍기 규칙 (원본: pickup.c addtobill, shop_object)
// =============================================================================

/// [v2.22.0 R10-5] 상점 내 줍기 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopPickupResult {
    /// 자유롭게 줍기 가능 (상점 밖 or 자신의 물건)
    Free,
    /// 구매로 처리 (가격 표시)
    Purchase { price: i32, item_name: String },
    /// 줍기 불가 (상점 주인 거부)
    Denied(String),
    /// 미지불 상태로 줍기 (인벤토리에 unpaid 표시)
    Unpaid { price: i32, item_name: String },
}

/// [v2.22.0 R10-5] 상점 내 줍기 판정 (원본: shop_object, bill_check)
pub fn check_shop_pickup(
    in_shop: bool,
    shopkeeper_present: bool,
    is_own_item: bool,
    item_name: &str,
    item_price: i32,
    charisma: i32,
    tourist: bool,
) -> ShopPickupResult {
    if !in_shop || is_own_item {
        return ShopPickupResult::Free;
    }

    if !shopkeeper_present {
        // 상점주인 부재 → 자유 줍기 (도둑질)
        return ShopPickupResult::Free;
    }

    // 카리스마 할인 적용 (원본: shop_price_calc)
    let discount = if charisma >= 18 {
        25 // 25% 할인
    } else if charisma >= 15 {
        15
    } else if tourist {
        33 // 관광객 추가 요금
    } else {
        0
    };

    let adjusted_price = if discount > 0 && !tourist {
        item_price - (item_price * discount / 100)
    } else if tourist {
        item_price + (item_price * discount / 100)
    } else {
        item_price
    };

    ShopPickupResult::Unpaid {
        price: adjusted_price.max(1),
        item_name: item_name.to_string(),
    }
}

// =============================================================================
// [5] 컨테이너 중첩 제한 (원본: pickup.c mbag_item_gone, bag_of_holding)
// =============================================================================

/// [v2.22.0 R10-5] 컨테이너 삽입 고급 판정
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerNestResult {
    /// 삽입 가능
    Ok,
    /// 중첩 금지 (가방 안에 가방)
    BagInBag,
    /// 마법 가방 폭발 위험
    ExplosionRisk,
    /// 무게 초과
    TooHeavy { max: i32, actual: i32 },
}

/// [v2.22.0 R10-5] 컨테이너 중첩 판정 (원본: in_container 제한 로직)
pub fn check_container_nesting(
    container_is_bag_of_holding: bool,
    item_is_container: bool,
    item_is_bag_of_holding: bool,
    item_is_bag_of_tricks: bool,
    item_is_cancel_wand: bool,
    container_weight_limit: i32,
    item_weight: i32,
) -> ContainerNestResult {
    // 마법 가방에 특정 아이템 삽입 시 폭발
    if container_is_bag_of_holding {
        if item_is_bag_of_holding || item_is_bag_of_tricks || item_is_cancel_wand {
            return ContainerNestResult::ExplosionRisk;
        }
    }

    // 가방 안에 가방 (일반 제한은 없지만 BoH 제한 후 통과)
    if item_is_container && container_is_bag_of_holding && item_is_bag_of_holding {
        return ContainerNestResult::BagInBag;
    }

    // 무게 제한
    if container_weight_limit > 0 && item_weight > container_weight_limit {
        return ContainerNestResult::TooHeavy {
            max: container_weight_limit,
            actual: item_weight,
        };
    }

    ContainerNestResult::Ok
}

// =============================================================================
// [6] 자동줍기 고급 패턴 매칭 (원본: pickup.c autopickup_exceptions)
// =============================================================================

/// [v2.22.0 R10-5] 자동줍기 패턴 규칙
#[derive(Debug, Clone)]
pub struct AutopickupRule {
    /// 패턴 문자열 (글로브/부분 일치)
    pub pattern: String,
    /// true = 줍기, false = 줍지 않기(제외)
    pub grab: bool,
    /// 클래스 필터 (None = 모든 클래스)
    pub class_filter: Option<char>,
}

/// [v2.22.0 R10-5] 패턴 매칭 (단순 부분 문자열 + 와일드카드)
fn pattern_matches(pattern: &str, name: &str) -> bool {
    let p = pattern.to_lowercase();
    let n = name.to_lowercase();

    if p == "*" {
        return true;
    }
    if p.starts_with('*') && p.ends_with('*') {
        // *xxx* → contains
        let inner = &p[1..p.len() - 1];
        return n.contains(inner);
    }
    if p.starts_with('*') {
        // *xxx → ends_with
        let suffix = &p[1..];
        return n.ends_with(suffix);
    }
    if p.ends_with('*') {
        // xxx* → starts_with
        let prefix = &p[..p.len() - 1];
        return n.starts_with(prefix);
    }
    // 정확 일치 또는 부분 일치
    n.contains(&p)
}

/// [v2.22.0 R10-5] 자동줍기 규칙 적용 (원본: check_autopickup_exceptions)
pub fn apply_autopickup_rules(
    rules: &[AutopickupRule],
    item_name: &str,
    item_class: char,
) -> Option<bool> {
    // 규칙을 역순으로 탐색 (마지막 매칭 규칙이 우선)
    for rule in rules.iter().rev() {
        // 클래스 필터 체크
        if let Some(class) = rule.class_filter {
            if class != item_class {
                continue;
            }
        }
        // 패턴 매칭
        if pattern_matches(&rule.pattern, item_name) {
            return Some(rule.grab);
        }
    }
    None // 매칭 규칙 없음 → 기본 설정 따름
}

// =============================================================================
// [7] 바닥 아이템 표시 (원본: pickup.c look_here, dolook)
// =============================================================================

/// [v2.22.0 R10-5] 바닥 아이템 요약 생성
pub fn format_floor_items(items: &[SortableItem], max_display: usize) -> Vec<String> {
    let mut result = Vec::new();
    let display_count = items.len().min(max_display);

    for item in items.iter().take(display_count) {
        let qty_str = if item.quantity > 1 {
            format!("{}개의 ", item.quantity)
        } else {
            String::new()
        };
        result.push(format!("{}{}", qty_str, item.name));
    }

    if items.len() > max_display {
        result.push(format!("...외 {}개의 아이템", items.len() - max_display));
    }

    result
}

/// [v2.22.0 R10-5] 바닥 아이템 합계 무게
pub fn total_floor_weight(items: &[SortableItem]) -> i32 {
    items.iter().map(|i| i.weight * i.quantity).sum()
}

// =============================================================================
// [8] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_items() -> Vec<SortableItem> {
        vec![
            SortableItem {
                index: 0,
                name: "long sword".to_string(),
                class_symbol: ')',
                weight: 40,
                price: 15,
                quantity: 1,
                buc_status: 2,
            },
            SortableItem {
                index: 1,
                name: "chain mail".to_string(),
                class_symbol: '[',
                weight: 300,
                price: 75,
                quantity: 1,
                buc_status: 2,
            },
            SortableItem {
                index: 2,
                name: "potion of healing".to_string(),
                class_symbol: '!',
                weight: 20,
                price: 100,
                quantity: 3,
                buc_status: 0,
            },
            SortableItem {
                index: 3,
                name: "food ration".to_string(),
                class_symbol: '%',
                weight: 20,
                price: 45,
                quantity: 2,
                buc_status: 2,
            },
            SortableItem {
                index: 4,
                name: "dagger".to_string(),
                class_symbol: ')',
                weight: 10,
                price: 4,
                quantity: 5,
                buc_status: 2,
            },
        ]
    }

    #[test]
    fn test_sort_by_name() {
        let mut items = make_test_items();
        sort_items(&mut items, SortCriteria::ByName);
        assert_eq!(items[0].name, "chain mail");
        assert_eq!(items[1].name, "dagger");
    }

    #[test]
    fn test_sort_by_class() {
        let mut items = make_test_items();
        sort_items(&mut items, SortCriteria::ByClass);
        // ')' 무기 먼저, '[' 갑옷 다음
        assert_eq!(items[0].class_symbol, ')');
        assert_eq!(items[1].class_symbol, ')');
        assert_eq!(items[2].class_symbol, '[');
    }

    #[test]
    fn test_sort_by_weight() {
        let mut items = make_test_items();
        sort_items(&mut items, SortCriteria::ByWeight);
        assert_eq!(items[0].name, "chain mail"); // 무게 300 최대
    }

    #[test]
    fn test_sort_by_price() {
        let mut items = make_test_items();
        sort_items(&mut items, SortCriteria::ByPrice);
        assert_eq!(items[0].name, "potion of healing"); // 가격 100 최대
    }

    #[test]
    fn test_can_merge_same() {
        assert!(can_merge("dagger", "dagger", 2, 2, 0, 0, 0, 0));
    }

    #[test]
    fn test_can_merge_different_name() {
        assert!(!can_merge("dagger", "sword", 2, 2, 0, 0, 0, 0));
    }

    #[test]
    fn test_can_merge_different_buc() {
        assert!(!can_merge("dagger", "dagger", 1, 3, 0, 0, 0, 0));
    }

    #[test]
    fn test_find_merge_candidates() {
        let items = vec![
            SortableItem {
                index: 0,
                name: "arrow".to_string(),
                class_symbol: ')',
                weight: 1,
                price: 2,
                quantity: 10,
                buc_status: 2,
            },
            SortableItem {
                index: 1,
                name: "arrow".to_string(),
                class_symbol: ')',
                weight: 1,
                price: 2,
                quantity: 5,
                buc_status: 2,
            },
            SortableItem {
                index: 2,
                name: "dagger".to_string(),
                class_symbol: ')',
                weight: 10,
                price: 4,
                quantity: 1,
                buc_status: 2,
            },
        ];
        let result = find_merge_candidates(&items);
        assert_eq!(result.merged_pairs.len(), 1);
        assert_eq!(result.merged_pairs[0], (0, 1));
        assert_eq!(result.remaining.len(), 2); // 0 + 2 남음
    }

    #[test]
    fn test_split_stack_partial() {
        let r = split_stack(10, 3);
        assert!(r.success);
        assert_eq!(r.original_remaining, 7);
        assert_eq!(r.split_count, 3);
    }

    #[test]
    fn test_split_stack_all() {
        let r = split_stack(5, 10);
        assert!(r.success);
        assert_eq!(r.original_remaining, 0);
        assert_eq!(r.split_count, 5);
    }

    #[test]
    fn test_split_stack_zero() {
        let r = split_stack(5, 0);
        assert!(!r.success);
    }

    #[test]
    fn test_shop_pickup_free() {
        let r = check_shop_pickup(false, false, false, "dagger", 4, 10, false);
        assert_eq!(r, ShopPickupResult::Free);
    }

    #[test]
    fn test_shop_pickup_own_item() {
        let r = check_shop_pickup(true, true, true, "dagger", 4, 10, false);
        assert_eq!(r, ShopPickupResult::Free);
    }

    #[test]
    fn test_shop_pickup_unpaid() {
        let r = check_shop_pickup(true, true, false, "long sword", 15, 10, false);
        if let ShopPickupResult::Unpaid { price, .. } = r {
            assert_eq!(price, 15);
        } else {
            panic!("예상: Unpaid");
        }
    }

    #[test]
    fn test_shop_charisma_discount() {
        let r = check_shop_pickup(true, true, false, "plate mail", 100, 18, false);
        if let ShopPickupResult::Unpaid { price, .. } = r {
            assert_eq!(price, 75); // 25% 할인
        } else {
            panic!("예상: Unpaid");
        }
    }

    #[test]
    fn test_container_nesting_ok() {
        let r = check_container_nesting(false, false, false, false, false, 0, 10);
        assert_eq!(r, ContainerNestResult::Ok);
    }

    #[test]
    fn test_container_nesting_explosion() {
        // BoH에 BoH 삽입 → 폭발
        let r = check_container_nesting(true, true, true, false, false, 0, 10);
        assert_eq!(r, ContainerNestResult::ExplosionRisk);
    }

    #[test]
    fn test_container_nesting_cancel_wand() {
        // BoH에 취소 지팡이 → 폭발
        let r = check_container_nesting(true, false, false, false, true, 0, 10);
        assert_eq!(r, ContainerNestResult::ExplosionRisk);
    }

    #[test]
    fn test_container_weight_limit() {
        let r = check_container_nesting(false, false, false, false, false, 100, 200);
        assert_eq!(
            r,
            ContainerNestResult::TooHeavy {
                max: 100,
                actual: 200
            }
        );
    }

    #[test]
    fn test_pattern_wildcard_all() {
        assert!(pattern_matches("*", "anything"));
    }

    #[test]
    fn test_pattern_contains() {
        assert!(pattern_matches("*sword*", "long sword"));
        assert!(!pattern_matches("*sword*", "dagger"));
    }

    #[test]
    fn test_pattern_starts_with() {
        assert!(pattern_matches("long*", "long sword"));
        assert!(!pattern_matches("long*", "short sword"));
    }

    #[test]
    fn test_pattern_ends_with() {
        assert!(pattern_matches("*sword", "long sword"));
        assert!(!pattern_matches("*sword", "long bow"));
    }

    #[test]
    fn test_autopickup_rules() {
        let rules = vec![
            AutopickupRule {
                pattern: "*".to_string(),
                grab: false,
                class_filter: None,
            },
            AutopickupRule {
                pattern: "*potion*".to_string(),
                grab: true,
                class_filter: Some('!'),
            },
        ];

        // 포션 → grab (마지막 매칭 규칙)
        assert_eq!(
            apply_autopickup_rules(&rules, "potion of healing", '!'),
            Some(true)
        );
        // 무기 → 전체 거부 (* 규칙)
        assert_eq!(
            apply_autopickup_rules(&rules, "long sword", ')'),
            Some(false)
        );
    }

    #[test]
    fn test_format_floor_items() {
        let items = make_test_items();
        let display = format_floor_items(&items, 3);
        assert_eq!(display.len(), 4); // 3개 표시 + "외 2개"
        assert!(display[3].contains("외 2개"));
    }

    #[test]
    fn test_total_floor_weight() {
        let items = make_test_items();
        // 40*1 + 300*1 + 20*3 + 20*2 + 10*5 = 40+300+60+40+50 = 490
        assert_eq!(total_floor_weight(&items), 490);
    }
}

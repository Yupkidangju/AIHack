// ============================================================================
// [v2.33.0 Phase 97-2] 인벤토리 관리 확장 (invent_phase97_ext.rs)
// 원본: NetHack 3.6.7 src/invent.c L500-2000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 인벤토리 정렬/분류 — inventory_sort (invent.c L500-1000)
// =============================================================================

/// [v2.33.0 97-2] 아이템 카테고리
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Ring,
    Amulet,
    Tool,
    Food,
    Potion,
    Scroll,
    Spellbook,
    Wand,
    Coin,
    Gem,
    Rock,
    Ball,
    Chain,
    Corpse,
    Other,
}

/// [v2.33.0 97-2] 인벤토리 아이템
#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub letter: char,
    pub name: String,
    pub category: ItemCategory,
    pub quantity: i32,
    pub weight: i32,
    pub value: i32,
    pub is_equipped: bool,
    pub buc: i32, // -1=저주, 0=무축, 1=축복
    pub is_identified: bool,
}

/// [v2.33.0 97-2] 인벤토리 요약
#[derive(Debug, Clone)]
pub struct InventorySummary {
    pub total_items: i32,
    pub total_weight: i32,
    pub max_weight: i32,
    pub categories: Vec<(ItemCategory, i32)>,
    pub is_encumbered: bool,
    pub encumbrance_level: i32, // 0=없음, 1=부담, 2=힘듦, 3=과부하, 4=초과
}

/// [v2.33.0 97-2] 인벤토리 요약 생성
pub fn inventory_summary(items: &[InventoryItem], str_carry: i32) -> InventorySummary {
    let total_items: i32 = items.iter().map(|i| i.quantity).sum();
    let total_weight: i32 = items.iter().map(|i| i.weight * i.quantity).sum();
    let max_weight = str_carry * 50 + 400;

    let mut cat_counts = std::collections::BTreeMap::new();
    for item in items {
        *cat_counts.entry(item.category).or_insert(0) += item.quantity;
    }
    let categories: Vec<(ItemCategory, i32)> = cat_counts.into_iter().collect();

    let ratio = if max_weight > 0 {
        (total_weight * 100) / max_weight
    } else {
        100
    };
    let encumbrance_level = if ratio < 50 {
        0
    } else if ratio < 70 {
        1
    } else if ratio < 85 {
        2
    } else if ratio < 100 {
        3
    } else {
        4
    };

    InventorySummary {
        total_items,
        total_weight,
        max_weight,
        categories,
        is_encumbered: encumbrance_level > 0,
        encumbrance_level,
    }
}

// =============================================================================
// [2] 아이템 합치기 — merge (invent.c L1000-1300)
// =============================================================================

/// [v2.33.0 97-2] 머지 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    Merged { total_quantity: i32 },
    CannotMerge { reason: String },
}

/// [v2.33.0 97-2] 아이템 합치기 가능 확인
pub fn try_merge_items(item_a: &InventoryItem, item_b: &InventoryItem) -> MergeResult {
    if item_a.name != item_b.name {
        return MergeResult::CannotMerge {
            reason: "다른 아이템이다.".to_string(),
        };
    }
    if item_a.buc != item_b.buc {
        return MergeResult::CannotMerge {
            reason: "축복/저주 상태가 다르다.".to_string(),
        };
    }
    if item_a.is_equipped || item_b.is_equipped {
        return MergeResult::CannotMerge {
            reason: "장착 중인 아이템은 합칠 수 없다.".to_string(),
        };
    }

    MergeResult::Merged {
        total_quantity: item_a.quantity + item_b.quantity,
    }
}

// =============================================================================
// [3] 아이템 선택 프롬프트 — item_select (invent.c L1300-2000)
// =============================================================================

/// [v2.33.0 97-2] 아이템 필터
#[derive(Debug, Clone)]
pub struct ItemFilter {
    pub allowed_categories: Vec<ItemCategory>,
    pub buc_filter: Option<i32>,
    pub identified_only: bool,
    pub equipped_only: bool,
}

/// [v2.33.0 97-2] 필터 적용
pub fn filter_inventory(items: &[InventoryItem], filter: &ItemFilter) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            if !filter.allowed_categories.is_empty()
                && !filter.allowed_categories.contains(&item.category)
            {
                return None;
            }
            if let Some(buc) = filter.buc_filter {
                if item.buc != buc {
                    return None;
                }
            }
            if filter.identified_only && !item.is_identified {
                return None;
            }
            if filter.equipped_only && !item.is_equipped {
                return None;
            }
            Some(idx)
        })
        .collect()
}

/// [v2.33.0 97-2] 인벤토리 글자 할당
pub fn assign_letter(existing_letters: &[char]) -> Option<char> {
    let all_letters: Vec<char> = ('a'..='z').chain('A'..='Z').collect();
    all_letters
        .into_iter()
        .find(|c| !existing_letters.contains(c))
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_item(name: &str, cat: ItemCategory, qty: i32, wt: i32) -> InventoryItem {
        InventoryItem {
            letter: 'a',
            name: name.to_string(),
            category: cat,
            quantity: qty,
            weight: wt,
            value: 10,
            is_equipped: false,
            buc: 0,
            is_identified: true,
        }
    }

    #[test]
    fn test_inventory_summary() {
        let items = vec![
            test_item("검", ItemCategory::Weapon, 1, 40),
            test_item("갑옷", ItemCategory::Armor, 1, 100),
            test_item("식량", ItemCategory::Food, 3, 20),
        ];
        let summary = inventory_summary(&items, 16);
        assert_eq!(summary.total_items, 5);
        assert_eq!(summary.total_weight, 200);
    }

    #[test]
    fn test_encumbrance() {
        let items = vec![test_item("바위", ItemCategory::Rock, 10, 100)];
        let summary = inventory_summary(&items, 10);
        assert!(summary.is_encumbered);
    }

    #[test]
    fn test_merge_success() {
        let a = test_item("화살", ItemCategory::Weapon, 10, 1);
        let b = test_item("화살", ItemCategory::Weapon, 5, 1);
        let result = try_merge_items(&a, &b);
        assert!(matches!(result, MergeResult::Merged { total_quantity: 15 }));
    }

    #[test]
    fn test_merge_different() {
        let a = test_item("화살", ItemCategory::Weapon, 10, 1);
        let b = test_item("볼트", ItemCategory::Weapon, 5, 1);
        let result = try_merge_items(&a, &b);
        assert!(matches!(result, MergeResult::CannotMerge { .. }));
    }

    #[test]
    fn test_filter_category() {
        let items = vec![
            test_item("검", ItemCategory::Weapon, 1, 40),
            test_item("갑옷", ItemCategory::Armor, 1, 100),
            test_item("포션", ItemCategory::Potion, 2, 10),
        ];
        let filter = ItemFilter {
            allowed_categories: vec![ItemCategory::Potion],
            buc_filter: None,
            identified_only: false,
            equipped_only: false,
        };
        let result = filter_inventory(&items, &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 2);
    }

    #[test]
    fn test_assign_letter() {
        let existing = vec!['a', 'b', 'c'];
        let next = assign_letter(&existing);
        assert_eq!(next, Some('d'));
    }

    #[test]
    fn test_no_encumbrance() {
        let items = vec![test_item("반지", ItemCategory::Ring, 1, 1)];
        let summary = inventory_summary(&items, 16);
        assert!(!summary.is_encumbered);
    }
}

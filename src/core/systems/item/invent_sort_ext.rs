// ============================================================================
// [v2.32.0 R20-2] 인벤토리 정렬 (invent_sort_ext.rs)
// 원본: NetHack 3.6.7 invent.c sortloot
// 아이템 정렬 기준, 필터링, 그룹핑
// ============================================================================

/// [v2.32.0 R20-2] 아이템 정렬 키
#[derive(Debug, Clone)]
pub struct SortableItem {
    pub id: u64,
    pub name: String,
    pub oclass: char,   // 아이템 클래스 (*!?/=+"[)% 등)
    pub buc_order: i32, // blessed=0, uncursed=1, cursed=2, unknown=3
    pub enchantment: i32,
    pub weight: i32,
    pub value: i32,
}

/// [v2.32.0 R20-2] 정렬 모드
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    ByClass,  // 클래스 → 이름
    ByName,   // 이름 알파벳
    ByValue,  // 가치 내림차순
    ByWeight, // 무게 오름차순
    Full,     // 클래스 → BUC → 인챈트 → 이름
}

/// [v2.32.0 R20-2] 정렬 (원본: sortloot)
pub fn sort_inventory(items: &mut [SortableItem], mode: SortMode) {
    match mode {
        SortMode::ByClass => {
            items.sort_by(|a, b| a.oclass.cmp(&b.oclass).then(a.name.cmp(&b.name)))
        }
        SortMode::ByName => items.sort_by(|a, b| a.name.cmp(&b.name)),
        SortMode::ByValue => items.sort_by(|a, b| b.value.cmp(&a.value)),
        SortMode::ByWeight => items.sort_by(|a, b| a.weight.cmp(&b.weight)),
        SortMode::Full => items.sort_by(|a, b| {
            a.oclass
                .cmp(&b.oclass)
                .then(a.buc_order.cmp(&b.buc_order))
                .then(b.enchantment.cmp(&a.enchantment))
                .then(a.name.cmp(&b.name))
        }),
    }
}

/// [v2.32.0 R20-2] 클래스별 필터
pub fn filter_by_class(items: &[SortableItem], oclass: char) -> Vec<&SortableItem> {
    items.iter().filter(|i| i.oclass == oclass).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_items() -> Vec<SortableItem> {
        vec![
            SortableItem {
                id: 1,
                name: "dagger".into(),
                oclass: ')',
                buc_order: 1,
                enchantment: 2,
                weight: 10,
                value: 4,
            },
            SortableItem {
                id: 2,
                name: "arrow".into(),
                oclass: ')',
                buc_order: 0,
                enchantment: 0,
                weight: 1,
                value: 2,
            },
            SortableItem {
                id: 3,
                name: "healing".into(),
                oclass: '!',
                buc_order: 1,
                enchantment: 0,
                weight: 20,
                value: 100,
            },
        ]
    }

    #[test]
    fn test_sort_by_name() {
        let mut items = make_items();
        sort_inventory(&mut items, SortMode::ByName);
        assert_eq!(items[0].name, "arrow");
    }

    #[test]
    fn test_sort_by_value() {
        let mut items = make_items();
        sort_inventory(&mut items, SortMode::ByValue);
        assert_eq!(items[0].name, "healing");
    }

    #[test]
    fn test_sort_full() {
        let mut items = make_items();
        sort_inventory(&mut items, SortMode::Full);
        assert_eq!(items[0].oclass, '!'); // 포션 클래스 먼저
    }

    #[test]
    fn test_filter() {
        let items = make_items();
        let weapons = filter_by_class(&items, ')');
        assert_eq!(weapons.len(), 2);
    }
}

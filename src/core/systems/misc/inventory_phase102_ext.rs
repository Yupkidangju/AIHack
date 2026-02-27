// ============================================================================
// [v2.38.0 Phase 102-1] 인벤토리/소지품 관리 통합 (inventory_phase102_ext.rs)
// 원본: NetHack 3.6.7 src/invent.c 핵심 미이식 함수 통합
// 순수 결과 패턴
//
// 구현 범위:
//   - 인벤토리 슬롯 시스템 (a-z, A-Z = 52슬롯)
//   - 무게/부피 기반 소지 한계
//   - 아이템 병합(스택) 로직
//   - 인벤토리 정렬/분류
//   - 장비 슬롯별 착용 관리
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 인벤토리 슬롯 — inventory_slot
// =============================================================================

/// [v2.38.0 102-1] 아이템 카테고리
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCategory {
    Weapon,    // 무기
    Armor,     // 갑옷
    Ring,      // 반지
    Amulet,    // 부적
    Tool,      // 도구
    Food,      // 음식
    Potion,    // 포션
    Scroll,    // 스크롤
    Spellbook, // 마법서
    Wand,      // 지팡이
    Gem,       // 보석/돌
    Gold,      // 금화
    Statue,    // 석상
    Corpse,    // 시체
    Other,     // 기타
}

/// [v2.38.0 102-1] 인벤토리 아이템
#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub slot: char,
    pub name: String,
    pub category: ItemCategory,
    pub weight: i32, // oz 단위
    pub quantity: i32,
    pub enchantment: i32, // +N 강화치
    pub identified: bool,
    pub cursed: bool,
    pub blessed: bool,
    pub equipped: bool,
}

/// [v2.38.0 102-1] 인벤토리 상태
#[derive(Debug, Clone)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
    pub max_weight: i32, // 최대 무게 (체력 기반)
    pub gold: i64,
}

impl Inventory {
    /// 새 인벤토리 생성
    pub fn new(strength: i32) -> Self {
        // 원본: 25*(STR+25) + 50
        let max_w = 25 * (strength + 25) + 50;
        Self {
            items: Vec::new(),
            max_weight: max_w,
            gold: 0,
        }
    }

    /// 현재 총 무게
    pub fn total_weight(&self) -> i32 {
        self.items.iter().map(|i| i.weight * i.quantity).sum()
    }

    /// 빈 슬롯 찾기 (a-z, A-Z 순서)
    pub fn find_free_slot(&self) -> Option<char> {
        let used: Vec<char> = self.items.iter().map(|i| i.slot).collect();
        // 소문자 a-z 먼저
        for c in 'a'..='z' {
            if !used.contains(&c) {
                return Some(c);
            }
        }
        // 대문자 A-Z
        for c in 'A'..='Z' {
            if !used.contains(&c) {
                return Some(c);
            }
        }
        None // 52슬롯 모두 사용 중
    }

    /// 아이템 추가 (스택 가능하면 병합)
    pub fn add_item(
        &mut self,
        name: &str,
        category: ItemCategory,
        weight: i32,
        quantity: i32,
    ) -> Result<String, String> {
        let new_weight = weight * quantity;
        if self.total_weight() + new_weight > self.max_weight {
            return Err(format!(
                "너무 무겁다! ({}/{}oz)",
                self.total_weight() + new_weight,
                self.max_weight
            ));
        }

        // 스택 가능 카테고리: 포션, 스크롤, 음식, 보석, 금화
        let stackable = matches!(
            category,
            ItemCategory::Potion
                | ItemCategory::Scroll
                | ItemCategory::Food
                | ItemCategory::Gem
                | ItemCategory::Gold
        );

        if stackable {
            // 동일 아이템 찾기
            if let Some(existing) = self
                .items
                .iter_mut()
                .find(|i| i.name == name && i.category == category)
            {
                existing.quantity += quantity;
                return Ok(format!("{} ({}개)", name, existing.quantity));
            }
        }

        // 새 슬롯 할당
        let slot = self
            .find_free_slot()
            .ok_or_else(|| "인벤토리가 가득 찼다!".to_string())?;

        self.items.push(InventoryItem {
            slot,
            name: name.to_string(),
            category,
            weight,
            quantity,
            enchantment: 0,
            identified: false,
            cursed: false,
            blessed: false,
            equipped: false,
        });

        Ok(format!("{} - {}", slot, name))
    }

    /// 아이템 제거
    pub fn remove_item(&mut self, slot: char) -> Option<InventoryItem> {
        if let Some(pos) = self.items.iter().position(|i| i.slot == slot) {
            if self.items[pos].equipped {
                return None; // 장비 중인 아이템은 제거 불가
            }
            Some(self.items.remove(pos))
        } else {
            None
        }
    }

    /// 카테고리별 아이템 목록
    pub fn items_by_category(&self, category: ItemCategory) -> Vec<&InventoryItem> {
        self.items
            .iter()
            .filter(|i| i.category == category)
            .collect()
    }

    /// 인벤토리 아이템 수
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// 장비 착용
    pub fn equip(&mut self, slot: char) -> Result<String, String> {
        let item = self
            .items
            .iter_mut()
            .find(|i| i.slot == slot)
            .ok_or_else(|| "아이템을 찾을 수 없다.".to_string())?;

        if item.equipped {
            return Err("이미 장비 중이다.".to_string());
        }

        item.equipped = true;
        Ok(format!("{} 장비!", item.name))
    }

    /// 장비 해제
    pub fn unequip(&mut self, slot: char) -> Result<String, String> {
        let item = self
            .items
            .iter_mut()
            .find(|i| i.slot == slot)
            .ok_or_else(|| "아이템을 찾을 수 없다.".to_string())?;

        if !item.equipped {
            return Err("장비 중이 아니다.".to_string());
        }
        if item.cursed {
            return Err("저주받은 아이템은 벗을 수 없다!".to_string());
        }

        item.equipped = false;
        Ok(format!("{} 해제.", item.name))
    }
}

// =============================================================================
// [2] 인벤토리 정렬 — sort_inventory
// =============================================================================

/// [v2.38.0 102-1] 인벤토리 정렬 기준
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortCriteria {
    BySlot,     // 슬롯 순서 (기본)
    ByCategory, // 카테고리별
    ByWeight,   // 무게순
    ByName,     // 이름순
}

/// [v2.38.0 102-1] 인벤토리 요약 생성
pub fn generate_inventory_summary(inventory: &Inventory) -> String {
    let total = inventory.count();
    let weight = inventory.total_weight();
    let max = inventory.max_weight;
    let equipped: usize = inventory.items.iter().filter(|i| i.equipped).count();
    let categories: Vec<_> = [
        ItemCategory::Weapon,
        ItemCategory::Armor,
        ItemCategory::Potion,
        ItemCategory::Scroll,
        ItemCategory::Food,
        ItemCategory::Ring,
    ]
    .iter()
    .filter_map(|cat| {
        let count = inventory.items_by_category(*cat).len();
        if count > 0 {
            Some(format!("{:?}:{}", cat, count))
        } else {
            None
        }
    })
    .collect();

    format!(
        "소지품 {}/52 | 무게 {}/{}oz | 장비 {}개 | {}",
        total,
        weight,
        max,
        equipped,
        if categories.is_empty() {
            "없음".to_string()
        } else {
            categories.join(", ")
        },
    )
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_inventory() {
        let inv = Inventory::new(16);
        assert_eq!(inv.count(), 0);
        assert!(inv.max_weight > 0);
    }

    #[test]
    fn test_add_item() {
        let mut inv = Inventory::new(16);
        let result = inv.add_item("장검", ItemCategory::Weapon, 40, 1);
        assert!(result.is_ok());
        assert_eq!(inv.count(), 1);
    }

    #[test]
    fn test_stack_items() {
        let mut inv = Inventory::new(16);
        inv.add_item("치유 포션", ItemCategory::Potion, 20, 1)
            .unwrap();
        inv.add_item("치유 포션", ItemCategory::Potion, 20, 2)
            .unwrap();
        assert_eq!(inv.count(), 1); // 병합되어 1개
        assert_eq!(inv.items[0].quantity, 3);
    }

    #[test]
    fn test_weight_limit() {
        let mut inv = Inventory::new(5); // 낮은 힘
        let result = inv.add_item("거대한 바위", ItemCategory::Gem, 9999, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_equip_unequip() {
        let mut inv = Inventory::new(16);
        inv.add_item("장검", ItemCategory::Weapon, 40, 1).unwrap();
        let slot = inv.items[0].slot;
        assert!(inv.equip(slot).is_ok());
        assert!(inv.items[0].equipped);
        assert!(inv.unequip(slot).is_ok());
        assert!(!inv.items[0].equipped);
    }

    #[test]
    fn test_cursed_unequip() {
        let mut inv = Inventory::new(16);
        inv.add_item("저주받은 검", ItemCategory::Weapon, 40, 1)
            .unwrap();
        let slot = inv.items[0].slot;
        inv.items[0].cursed = true;
        inv.equip(slot).unwrap();
        assert!(inv.unequip(slot).is_err());
    }

    #[test]
    fn test_remove_item() {
        let mut inv = Inventory::new(16);
        inv.add_item("음식", ItemCategory::Food, 10, 1).unwrap();
        let slot = inv.items[0].slot;
        assert!(inv.remove_item(slot).is_some());
        assert_eq!(inv.count(), 0);
    }

    #[test]
    fn test_category_filter() {
        let mut inv = Inventory::new(16);
        inv.add_item("장검", ItemCategory::Weapon, 40, 1).unwrap();
        inv.add_item("단검", ItemCategory::Weapon, 10, 1).unwrap();
        inv.add_item("포션", ItemCategory::Potion, 20, 1).unwrap();
        assert_eq!(inv.items_by_category(ItemCategory::Weapon).len(), 2);
    }

    #[test]
    fn test_summary() {
        let mut inv = Inventory::new(16);
        inv.add_item("검", ItemCategory::Weapon, 40, 1).unwrap();
        let summary = generate_inventory_summary(&inv);
        assert!(summary.contains("소지품"));
        assert!(summary.contains("Weapon:1"));
    }

    #[test]
    fn test_slot_allocation() {
        let inv = Inventory::new(16);
        assert_eq!(inv.find_free_slot(), Some('a'));
    }
}

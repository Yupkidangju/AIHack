use serde::{Deserialize, Serialize};

use crate::core::ids::EntityId;

/// [v0.1.0] Phase 4 inventory letter다. `a..=z` 범위만 생성한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InventoryLetter(pub char);

/// [v0.1.0] 획득 순서를 보존하는 inventory entry다. 장착 상태는 Inventory.equipped_melee가 단일 원천이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InventoryEntry {
    pub item: EntityId,
    pub letter: InventoryLetter,
}

/// [v0.1.0] Phase 4 플레이어 inventory다. drop이 없으므로 letter는 재사용하지 않는다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub owner: EntityId,
    pub entries: Vec<InventoryEntry>,
    pub equipped_melee: Option<EntityId>,
    pub next_letter_index: u8,
}

impl Inventory {
    pub fn new(owner: EntityId) -> Self {
        Self {
            owner,
            entries: Vec::new(),
            equipped_melee: None,
            next_letter_index: 0,
        }
    }

    pub fn add_existing_with_next_letter(&mut self, item: EntityId) -> Option<InventoryLetter> {
        let letter = self.next_letter()?;
        self.entries.push(InventoryEntry { item, letter });
        self.next_letter_index += 1;
        Some(letter)
    }

    pub fn remove(&mut self, item: EntityId) -> Option<InventoryEntry> {
        let idx = self.entries.iter().position(|entry| entry.item == item)?;
        if self.equipped_melee == Some(item) {
            self.equipped_melee = None;
        }
        Some(self.entries.remove(idx))
    }

    pub fn contains(&self, item: EntityId) -> bool {
        self.entries.iter().any(|entry| entry.item == item)
    }

    pub fn letter_for(&self, item: EntityId) -> Option<InventoryLetter> {
        self.entries
            .iter()
            .find(|entry| entry.item == item)
            .map(|entry| entry.letter)
    }

    pub fn equip_melee(&mut self, item: EntityId) {
        self.equipped_melee = Some(item);
    }

    fn next_letter(&self) -> Option<InventoryLetter> {
        if self.next_letter_index >= 26 {
            return None;
        }
        Some(InventoryLetter((b'a' + self.next_letter_index) as char))
    }
}

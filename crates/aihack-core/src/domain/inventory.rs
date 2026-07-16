use serde::{Deserialize, Serialize};

use crate::ids::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InventoryLetter(pub char);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InventoryEntry {
    pub item: EntityId,
    pub letter: InventoryLetter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub owner: EntityId,
    pub entries: Vec<InventoryEntry>,
    pub equipped_melee: Option<EntityId>,
    pub equipped_body: Option<EntityId>,
    pub next_letter_index: u8,
}

impl Inventory {
    pub fn new(owner: EntityId) -> Self {
        Self {
            owner,
            entries: Vec::new(),
            equipped_melee: None,
            equipped_body: None,
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
        if self.equipped_body == Some(item) {
            self.equipped_body = None;
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
    pub fn equip_body(&mut self, item: EntityId) {
        self.equipped_body = Some(item);
    }
    fn next_letter(&self) -> Option<InventoryLetter> {
        (self.next_letter_index < 26)
            .then_some(InventoryLetter((b'a' + self.next_letter_index) as char))
    }
}

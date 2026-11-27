use serde::{Deserialize, Serialize};

use crate::{domain::item::EquipmentSlot, ids::EntityId};

pub const MAX_INVENTORY_SLOTS: usize = 52;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_ring: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_shield: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_helm: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_cloak: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_gloves: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_boots: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_right_ring: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_amulet: Option<EntityId>,
    pub next_letter_index: u8,
}

impl Inventory {
    pub fn new(owner: EntityId) -> Self {
        Self {
            owner,
            entries: Vec::new(),
            equipped_melee: None,
            equipped_body: None,
            equipped_ring: None,
            equipped_shield: None,
            equipped_helm: None,
            equipped_cloak: None,
            equipped_gloves: None,
            equipped_boots: None,
            equipped_right_ring: None,
            equipped_amulet: None,
            next_letter_index: 0,
        }
    }
    pub fn letter_for_index(index: u8) -> Option<InventoryLetter> {
        match index {
            0..=25 => Some(InventoryLetter((b'a' + index) as char)),
            26..=51 => Some(InventoryLetter((b'A' + (index - 26)) as char)),
            _ => None,
        }
    }
    pub fn index_for_letter(letter: InventoryLetter) -> Option<u8> {
        match letter.0 {
            'a'..='z' => Some((letter.0 as u8) - b'a'),
            'A'..='Z' => Some(26 + ((letter.0 as u8) - b'A')),
            _ => None,
        }
    }
    pub fn has_free_slot(&self) -> bool {
        self.next_letter().is_some()
    }
    pub fn add_existing_with_next_letter(&mut self, item: EntityId) -> Option<InventoryLetter> {
        let letter = self.next_letter()?;
        let index = Self::index_for_letter(letter)?;
        self.entries.push(InventoryEntry { item, letter });
        self.next_letter_index = self
            .next_letter_index
            .max(index + 1)
            .max(u8::try_from(self.entries.len()).unwrap_or(MAX_INVENTORY_SLOTS as u8))
            .min(MAX_INVENTORY_SLOTS as u8);
        Some(letter)
    }
    pub fn remove(&mut self, item: EntityId) -> Option<InventoryEntry> {
        let idx = self.entries.iter().position(|entry| entry.item == item)?;
        if let Some(slot) = self.slot_of(item) {
            self.set_slot(slot, None);
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
    pub fn equipped_in(&self, slot: EquipmentSlot) -> Option<EntityId> {
        match slot {
            EquipmentSlot::Melee => self.equipped_melee,
            EquipmentSlot::Shield => self.equipped_shield,
            EquipmentSlot::Helm => self.equipped_helm,
            EquipmentSlot::Body => self.equipped_body,
            EquipmentSlot::Cloak => self.equipped_cloak,
            EquipmentSlot::Gloves => self.equipped_gloves,
            EquipmentSlot::Boots => self.equipped_boots,
            EquipmentSlot::LeftRing => self.equipped_ring,
            EquipmentSlot::RightRing => self.equipped_right_ring,
            EquipmentSlot::Amulet => self.equipped_amulet,
        }
    }
    pub fn set_slot(&mut self, slot: EquipmentSlot, item: Option<EntityId>) {
        match slot {
            EquipmentSlot::Melee => self.equipped_melee = item,
            EquipmentSlot::Shield => self.equipped_shield = item,
            EquipmentSlot::Helm => self.equipped_helm = item,
            EquipmentSlot::Body => self.equipped_body = item,
            EquipmentSlot::Cloak => self.equipped_cloak = item,
            EquipmentSlot::Gloves => self.equipped_gloves = item,
            EquipmentSlot::Boots => self.equipped_boots = item,
            EquipmentSlot::LeftRing => self.equipped_ring = item,
            EquipmentSlot::RightRing => self.equipped_right_ring = item,
            EquipmentSlot::Amulet => self.equipped_amulet = item,
        }
    }
    pub fn slot_of(&self, item: EntityId) -> Option<EquipmentSlot> {
        EquipmentSlot::ALL
            .into_iter()
            .find(|slot| self.equipped_in(*slot) == Some(item))
    }
    pub fn equipped_slots(&self) -> [(EquipmentSlot, Option<EntityId>); 10] {
        EquipmentSlot::ALL.map(|slot| (slot, self.equipped_in(slot)))
    }
    pub fn all_defense_items(&self) -> Vec<EntityId> {
        EquipmentSlot::ALL_DEFENSE
            .into_iter()
            .filter_map(|slot| self.equipped_in(slot))
            .collect()
    }
    pub fn equip_melee(&mut self, item: EntityId) {
        self.equipped_melee = Some(item);
    }
    pub fn equip_body(&mut self, item: EntityId) {
        self.equipped_body = Some(item);
    }
    pub fn equip_ring(&mut self, item: EntityId) {
        self.equipped_ring = Some(item);
    }
    fn next_letter(&self) -> Option<InventoryLetter> {
        (0..MAX_INVENTORY_SLOTS as u8).find_map(|index| {
            let candidate = Self::letter_for_index(index)?;
            (!self.entries.iter().any(|entry| entry.letter == candidate)).then_some(candidate)
        })
    }
}

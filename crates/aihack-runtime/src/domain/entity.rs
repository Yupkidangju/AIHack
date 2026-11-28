use std::ops::Deref;

use aihack_core::{
    domain::{
        item::ItemKind,
        monster::{MonsterKind, MonsterTemplate},
    },
    error::EntityAllocationError,
    ids::EntityId,
    position::Pos,
};

pub use aihack_core::domain::entity::{
    ActorKind, ActorStats, Entity, EntityKind, EntityLocation, EntityPayload, Faction, ItemView,
    ItemViewMut,
};

/// Content registry를 알아야 하는 기본 생성만 runtime이 맡는다.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntityStore(aihack_core::domain::entity::EntityStore);

impl Default for EntityStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityStore {
    pub fn new() -> Self {
        Self(aihack_core::domain::entity::EntityStore::new())
    }

    pub(crate) fn inner_mut(&mut self) -> &mut aihack_core::domain::entity::EntityStore {
        &mut self.0
    }

    pub fn spawn_monster(
        &mut self,
        kind: MonsterKind,
        pos: Pos,
    ) -> Result<EntityId, EntityAllocationError> {
        self.0.spawn_monster_with_template(
            kind,
            crate::domain::monster::monster_template(kind),
            pos,
        )
    }

    pub fn spawn_item(
        &mut self,
        kind: ItemKind,
        location: EntityLocation,
    ) -> Result<EntityId, EntityAllocationError> {
        self.0
            .spawn_item_with_data(kind, crate::domain::item::item_data(kind), location)
    }

    pub fn spawn(
        &mut self,
        kind: EntityKind,
        faction: Faction,
        pos: Pos,
        stats: ActorStats,
    ) -> Result<EntityId, EntityAllocationError> {
        match kind {
            EntityKind::Player => self.0.spawn_actor(ActorKind::Player, faction, pos, stats),
            EntityKind::Monster(kind) => {
                self.0
                    .spawn_actor(ActorKind::Monster(kind), faction, pos, stats)
            }
            EntityKind::Item(kind) => self.spawn_item(kind, EntityLocation::on_main_level(pos)),
        }
    }

    pub fn spawn_player(&mut self, pos: Pos) -> Result<EntityId, EntityAllocationError> {
        self.0.spawn_player(pos)
    }

    pub(crate) fn spawn_monster_with_template(
        &mut self,
        kind: MonsterKind,
        template: MonsterTemplate,
        pos: Pos,
    ) -> Result<EntityId, EntityAllocationError> {
        self.0.spawn_monster_with_template(kind, template, pos)
    }

    pub fn spawn_monster_with_faction(
        &mut self,
        kind: MonsterKind,
        template: MonsterTemplate,
        faction: Faction,
        pos: Pos,
    ) -> Result<EntityId, EntityAllocationError> {
        self.0
            .spawn_monster_with_faction(kind, template, faction, pos)
    }

    pub(crate) fn spawn_item_with_data(
        &mut self,
        kind: ItemKind,
        data: aihack_core::domain::item::ItemData,
        location: EntityLocation,
    ) -> Result<EntityId, EntityAllocationError> {
        self.0.spawn_item_with_data(kind, data, location)
    }

    pub fn set_alive(&mut self, id: EntityId, alive: bool) -> bool {
        self.0.set_alive(id, alive)
    }

    pub fn set_pos(&mut self, id: EntityId, pos: Pos) -> bool {
        self.0.set_pos(id, pos)
    }

    pub fn set_actor_location(
        &mut self,
        id: EntityId,
        level: aihack_core::ids::LevelId,
        pos: Pos,
    ) -> bool {
        self.0.set_actor_location(id, level, pos)
    }

    pub fn set_item_location(&mut self, id: EntityId, next: EntityLocation) -> bool {
        self.0.set_item_location(id, next)
    }

    pub fn set_item_letter(
        &mut self,
        id: EntityId,
        letter: aihack_core::domain::inventory::InventoryLetter,
    ) -> bool {
        self.0.set_item_letter(id, letter)
    }

    pub fn actor_stats_mut(&mut self, id: EntityId) -> Option<&mut ActorStats> {
        self.0.actor_stats_mut(id)
    }

    pub fn set_item_charges(&mut self, id: EntityId, charges: Option<u8>) -> bool {
        self.0.set_item_charges(id, charges)
    }

    pub fn set_item_buc(
        &mut self,
        id: EntityId,
        buc: aihack_core::domain::item::BucStatus,
    ) -> bool {
        self.0.set_item_buc(id, buc)
    }

    pub fn set_item_enchantment(&mut self, id: EntityId, enchantment: i8) -> bool {
        self.0.set_item_enchantment(id, enchantment)
    }

    pub fn set_item_buc_known(&mut self, id: EntityId, known: bool) -> bool {
        self.0.set_item_buc_known(id, known)
    }

    pub fn set_item_enchant_known(&mut self, id: EntityId, known: bool) -> bool {
        self.0.set_item_enchant_known(id, known)
    }

    pub fn set_item_quantity(&mut self, id: EntityId, quantity: u16) -> bool {
        self.0.set_item_quantity(id, quantity)
    }

    pub fn set_item_corpse_monster(&mut self, id: EntityId, monster: Option<MonsterKind>) -> bool {
        self.0.set_item_corpse_monster(id, monster)
    }

    pub fn set_item_corpse_age(&mut self, id: EntityId, age: u16) -> bool {
        self.0.set_item_corpse_age(id, age)
    }

    pub fn set_item_artifact_id(&mut self, id: EntityId, artifact: Option<String>) -> bool {
        self.0.set_item_artifact_id(id, artifact)
    }

    pub fn set_item_tool_active(&mut self, id: EntityId, active: bool) -> bool {
        self.0.set_item_tool_active(id, active)
    }

    pub fn set_item_container_locked(&mut self, id: EntityId, locked: bool) -> bool {
        self.0.set_item_container_locked(id, locked)
    }

    pub fn set_item_container_trapped(&mut self, id: EntityId, trapped: bool) -> bool {
        self.0.set_item_container_trapped(id, trapped)
    }

    pub fn set_item_erosion(&mut self, id: EntityId, next: u8) -> bool {
        self.0.set_item_erosion(id, next)
    }

    pub fn push_contained_item(&mut self, container: EntityId, item: EntityId) -> bool {
        self.0.push_contained_item(container, item)
    }

    pub fn remove_contained_item(&mut self, container: EntityId, item: EntityId) -> bool {
        self.0.remove_contained_item(container, item)
    }

    pub fn clear_contained_items(&mut self, container: EntityId) -> bool {
        self.0.clear_contained_items(container)
    }

    pub fn set_item_unpaid(&mut self, id: EntityId, unpaid: bool) -> bool {
        self.0.set_item_unpaid(id, unpaid)
    }

    pub fn set_actor_faction(&mut self, id: EntityId, next: Faction) -> bool {
        self.0.set_actor_faction(id, next)
    }

    pub fn set_actor_kind(&mut self, id: EntityId, next: ActorKind) -> bool {
        self.0.set_actor_kind(id, next)
    }

    pub fn polymorph_monster(
        &mut self,
        id: EntityId,
        new_kind: MonsterKind,
        new_stats: ActorStats,
    ) -> bool {
        self.0.polymorph_monster(id, new_kind, new_stats)
    }

    pub fn set_actor_tameness(&mut self, id: EntityId, next: u8) -> bool {
        self.0.set_actor_tameness(id, next)
    }

    pub fn set_actor_pet_xp(&mut self, id: EntityId, next: u32) -> bool {
        self.0.set_actor_pet_xp(id, next)
    }

    pub fn set_actor_pet_hunger(&mut self, id: EntityId, next: i16) -> bool {
        self.0.set_actor_pet_hunger(id, next)
    }

    pub fn push_monster_inventory(&mut self, actor: EntityId, item: EntityId) -> bool {
        self.0.push_monster_inventory(actor, item)
    }

    pub fn remove_monster_inventory(&mut self, actor: EntityId, item: EntityId) -> bool {
        self.0.remove_monster_inventory(actor, item)
    }

    pub fn clear_monster_inventory(&mut self, actor: EntityId) -> Vec<EntityId> {
        self.0.clear_monster_inventory(actor)
    }

    pub fn clear_item_letter(&mut self, id: EntityId) -> bool {
        self.0.clear_item_letter(id)
    }

    pub fn set_item_kind_and_data(
        &mut self,
        id: EntityId,
        kind: aihack_core::domain::item::ItemKind,
        data: aihack_core::domain::item::ItemData,
    ) -> bool {
        self.0.set_item_kind_and_data(id, kind, data)
    }

    pub fn clear_monsters(&mut self) {
        self.0.clear_monsters();
    }
}

impl Deref for EntityStore {
    type Target = aihack_core::domain::entity::EntityStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

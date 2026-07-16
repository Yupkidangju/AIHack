use std::ops::{Deref, DerefMut};

use aihack_core::{
    domain::{item::ItemKind, monster::MonsterKind},
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

    pub fn spawn_monster(&mut self, kind: MonsterKind, pos: Pos) -> EntityId {
        self.spawn_monster_with_template(kind, crate::domain::monster::monster_template(kind), pos)
    }

    pub fn spawn_item(&mut self, kind: ItemKind, location: EntityLocation) -> EntityId {
        self.spawn_item_with_data(kind, crate::domain::item::item_data(kind), location)
    }

    pub fn spawn(
        &mut self,
        kind: EntityKind,
        faction: Faction,
        pos: Pos,
        stats: ActorStats,
    ) -> EntityId {
        match kind {
            EntityKind::Player => self.spawn_actor(ActorKind::Player, faction, pos, stats),
            EntityKind::Monster(kind) => {
                self.spawn_actor(ActorKind::Monster(kind), faction, pos, stats)
            }
            EntityKind::Item(kind) => self.spawn_item(kind, EntityLocation::on_main_level(pos)),
        }
    }
}

impl Deref for EntityStore {
    type Target = aihack_core::domain::entity::EntityStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EntityStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

use serde::{Deserialize, Serialize};

use crate::{
    core::{
        ids::{EntityId, LevelId},
        position::Pos,
    },
    domain::{
        combat::{AttackProfile, DamageRoll},
        inventory::InventoryLetter,
        item::{item_data, ItemData, ItemKind},
        monster::{monster_ai_kind, monster_template, MonsterAiKind, MonsterKind},
        player::adventurer_template,
    },
};

/// [v0.1.0] Actor payload 내부 종류다. Item은 별도 payload로 분리한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorKind {
    Player,
    Monster(MonsterKind),
}

impl ActorKind {
    /// [v0.1.0] actor가 monster라면 monster kind를 돌려준다.
    pub fn monster_kind(self) -> Option<MonsterKind> {
        match self {
            Self::Monster(kind) => Some(kind),
            Self::Player => None,
        }
    }
}

/// [v0.1.0] 외부 관찰/snapshot에서 쓰는 통합 entity 종류다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    Player,
    Monster(MonsterKind),
    Item(ItemKind),
}

/// [v0.1.0] 전투 관계를 판정하기 위한 최소 faction이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    Player,
    Hostile,
    Neutral,
}

/// [v0.1.0] Phase 3/4 actor stat이다. item payload는 이 값을 갖지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorStats {
    pub hp: i16,
    pub max_hp: i16,
    pub ac: i16,
    pub hit_bonus: i16,
    pub damage_bonus: i16,
    pub damage_reduction: i16,
    pub damage: DamageRoll,
    pub weapon_hit_bonus: i16,
}

/// [v0.1.0] Phase 5 actor/item 공용 위치다. Consumed tombstone은 assigned_letter를 유지한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityLocation {
    OnMap { level: LevelId, pos: Pos },
    Inventory { owner: EntityId },
    Consumed,
}

impl EntityLocation {
    pub fn on_main_level(pos: Pos) -> Self {
        Self::OnMap {
            level: LevelId::main(1),
            pos,
        }
    }

    pub fn map_position(self) -> Option<(LevelId, Pos)> {
        match self {
            Self::OnMap { level, pos } => Some((level, pos)),
            Self::Inventory { .. } | Self::Consumed => None,
        }
    }
}

/// [v0.1.0] actor와 item의 invalid state를 막는 payload 분리 구조다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityPayload {
    Actor {
        kind: ActorKind,
        faction: Faction,
        location: EntityLocation,
        stats: ActorStats,
        alive: bool,
    },
    Item {
        kind: ItemKind,
        data: ItemData,
        location: EntityLocation,
        assigned_letter: Option<InventoryLetter>,
        charges: Option<u8>,
    },
}

pub type ItemView<'a> = (
    ItemKind,
    &'a ItemData,
    EntityLocation,
    Option<InventoryLetter>,
    Option<u8>,
);

pub type ItemViewMut<'a> = (
    &'a ItemData,
    &'a mut EntityLocation,
    &'a mut Option<InventoryLetter>,
    &'a mut Option<u8>,
);

/// [v0.1.0] Phase 4 entity store에 저장되는 통합 엔티티다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub payload: EntityPayload,
}

impl Entity {
    pub fn kind(&self) -> EntityKind {
        match &self.payload {
            EntityPayload::Actor { kind, .. } => match kind {
                ActorKind::Player => EntityKind::Player,
                ActorKind::Monster(kind) => EntityKind::Monster(*kind),
            },
            EntityPayload::Item { kind, .. } => EntityKind::Item(*kind),
        }
    }

    pub fn actor(&self) -> Option<(ActorKind, Faction, LevelId, Pos, &ActorStats, bool)> {
        match &self.payload {
            EntityPayload::Actor {
                kind,
                faction,
                location,
                stats,
                alive,
            } => {
                let (level, pos) = location.map_position()?;
                Some((*kind, *faction, level, pos, stats, *alive))
            }
            EntityPayload::Item { .. } => None,
        }
    }

    pub fn actor_kind(&self) -> Option<ActorKind> {
        match &self.payload {
            EntityPayload::Actor { kind, .. } => Some(*kind),
            EntityPayload::Item { .. } => None,
        }
    }

    pub fn faction(&self) -> Option<Faction> {
        match &self.payload {
            EntityPayload::Actor { faction, .. } => Some(*faction),
            EntityPayload::Item { .. } => None,
        }
    }

    pub fn is_alive_actor(&self) -> bool {
        match &self.payload {
            EntityPayload::Actor { alive, .. } => *alive,
            EntityPayload::Item { .. } => false,
        }
    }

    pub fn monster_ai_kind(&self) -> Option<MonsterAiKind> {
        self.actor_kind()
            .and_then(ActorKind::monster_kind)
            .map(monster_ai_kind)
    }

    pub fn actor_mut(&mut self) -> Option<(&mut EntityLocation, &mut ActorStats, &mut bool)> {
        match &mut self.payload {
            EntityPayload::Actor {
                location,
                stats,
                alive,
                ..
            } => Some((location, stats, alive)),
            EntityPayload::Item { .. } => None,
        }
    }

    pub fn item(&self) -> Option<ItemView<'_>> {
        match &self.payload {
            EntityPayload::Item {
                kind,
                data,
                location,
                assigned_letter,
                charges,
            } => Some((*kind, data, *location, *assigned_letter, *charges)),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_mut(&mut self) -> Option<ItemViewMut<'_>> {
        match &mut self.payload {
            EntityPayload::Item {
                data,
                location,
                assigned_letter,
                charges,
                ..
            } => Some((data, location, assigned_letter, charges)),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn natural_attack_profile(&self) -> Option<AttackProfile> {
        match &self.payload {
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Jackal),
                stats,
                ..
            } => Some(AttackProfile::natural("bite", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Goblin),
                stats,
                ..
            } => Some(AttackProfile::natural("short sword", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::FloatingEye),
                stats,
                ..
            } => Some(AttackProfile::natural("gaze", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Player,
                ..
            } => None,
            EntityPayload::Item { data, .. } => data.attack_profile,
        }
    }
}

/// [v0.1.0] Vec index 안정성을 위해 consumed/dead 엔티티를 즉시 제거하지 않는 저장소다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityStore {
    entities: Vec<Entity>,
    next_id: u32,
}

impl Default for EntityStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityStore {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_id: 1,
        }
    }

    pub fn spawn_player(&mut self, pos: Pos) -> EntityId {
        let template = adventurer_template();
        self.spawn_actor(
            ActorKind::Player,
            Faction::Player,
            pos,
            ActorStats {
                hp: template.hp,
                max_hp: template.hp,
                ac: template.ac,
                hit_bonus: template.hit_bonus,
                damage_bonus: template.damage_bonus,
                damage_reduction: 0,
                damage: template.attack_profile.damage,
                weapon_hit_bonus: template.attack_profile.hit_bonus,
            },
        )
    }

    pub fn spawn_monster(&mut self, kind: MonsterKind, pos: Pos) -> EntityId {
        let template = monster_template(kind);
        self.spawn_actor(
            ActorKind::Monster(kind),
            Faction::Hostile,
            pos,
            ActorStats {
                hp: template.hp,
                max_hp: template.hp,
                ac: template.ac,
                hit_bonus: template.hit_bonus,
                damage_bonus: template.damage_bonus,
                damage_reduction: 0,
                damage: template.attack_profile.damage,
                weapon_hit_bonus: template.attack_profile.hit_bonus,
            },
        )
    }

    pub fn spawn_actor(
        &mut self,
        kind: ActorKind,
        faction: Faction,
        pos: Pos,
        stats: ActorStats,
    ) -> EntityId {
        let id = self.next_entity_id();
        self.entities.push(Entity {
            id,
            payload: EntityPayload::Actor {
                kind,
                faction,
                location: EntityLocation::on_main_level(pos),
                stats,
                alive: true,
            },
        });
        id
    }

    pub fn spawn_item(&mut self, kind: ItemKind, location: EntityLocation) -> EntityId {
        let id = self.next_entity_id();
        self.entities.push(Entity {
            id,
            payload: EntityPayload::Item {
                kind,
                data: item_data(kind),
                location,
                assigned_letter: None,
                charges: item_data(kind).max_charges,
            },
        });
        id
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

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        if id.0 == 0 {
            return None;
        }
        self.entities.iter().find(|entity| entity.id == id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        if id.0 == 0 {
            return None;
        }
        self.entities.iter_mut().find(|entity| entity.id == id)
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn alive_actor_at(&self, level: LevelId, pos: Pos) -> Option<EntityId> {
        self.entities.iter().find_map(|entity| {
            let (_, _, actor_level, actor_pos, _, alive) = entity.actor()?;
            (alive && actor_level == level && actor_pos == pos).then_some(entity.id)
        })
    }

    pub fn alive_entity_at(&self, level: LevelId, pos: Pos) -> Option<EntityId> {
        self.alive_actor_at(level, pos)
    }

    pub fn alive_hostile_at(&self, level: LevelId, pos: Pos) -> Option<EntityId> {
        self.entities.iter().find_map(|entity| {
            let (_, faction, actor_level, actor_pos, _, alive) = entity.actor()?;
            (alive && actor_level == level && actor_pos == pos && faction == Faction::Hostile)
                .then_some(entity.id)
        })
    }

    pub fn hostile_monsters_on_level(&self, level: LevelId) -> Vec<EntityId> {
        let mut ids = self
            .entities
            .iter()
            .filter_map(|entity| {
                let (kind, faction, actor_level, _, _, alive) = entity.actor()?;
                (alive
                    && faction == Faction::Hostile
                    && actor_level == level
                    && matches!(kind, ActorKind::Monster(_)))
                .then_some(entity.id)
            })
            .collect::<Vec<_>>();
        ids.sort_by_key(|id| id.0);
        ids
    }

    pub fn items_at(&self, level: LevelId, pos: Pos) -> Vec<EntityId> {
        let mut items = self
            .entities
            .iter()
            .filter_map(|entity| match entity.item() {
                Some((
                    _,
                    _,
                    EntityLocation::OnMap {
                        level: item_level,
                        pos: item_pos,
                    },
                    _,
                    _,
                )) if item_level == level && item_pos == pos => Some(entity.id),
                _ => None,
            })
            .collect::<Vec<_>>();
        items.sort_by_key(|id| id.0);
        items
    }

    pub fn item_at(&self, level: LevelId, pos: Pos) -> Option<EntityId> {
        self.items_at(level, pos).into_iter().next()
    }

    pub fn inventory_items(&self, owner: EntityId) -> Vec<EntityId> {
        self.entities
            .iter()
            .filter_map(|entity| match entity.item() {
                Some((_, _, EntityLocation::Inventory { owner: item_owner }, _, _))
                    if item_owner == owner =>
                {
                    Some(entity.id)
                }
                _ => None,
            })
            .collect()
    }

    pub fn set_alive(&mut self, id: EntityId, alive: bool) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let Some((_, _, alive_ref)) = entity.actor_mut() else {
            return false;
        };
        *alive_ref = alive;
        true
    }

    pub fn set_pos(&mut self, id: EntityId, pos: Pos) -> bool {
        let Some((level, _)) = self.actor_location(id) else {
            return false;
        };
        self.set_actor_location(id, level, pos)
    }

    pub fn actor_location(&self, id: EntityId) -> Option<(LevelId, Pos)> {
        self.get(id)
            .and_then(|entity| entity.actor().map(|(_, _, level, pos, _, _)| (level, pos)))
    }

    pub fn set_actor_location(&mut self, id: EntityId, level: LevelId, pos: Pos) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let Some((location, _, _)) = entity.actor_mut() else {
            return false;
        };
        *location = EntityLocation::OnMap { level, pos };
        true
    }

    pub fn set_item_location(&mut self, id: EntityId, next: EntityLocation) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let Some((_, location, _, _)) = entity.item_mut() else {
            return false;
        };
        *location = next;
        true
    }

    pub fn set_item_letter(&mut self, id: EntityId, letter: InventoryLetter) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let Some((_, _, assigned_letter, _)) = entity.item_mut() else {
            return false;
        };
        *assigned_letter = Some(letter);
        true
    }

    pub fn actor_stats(&self, id: EntityId) -> Option<&ActorStats> {
        self.get(id)
            .and_then(|entity| entity.actor().map(|(_, _, _, _, stats, _)| stats))
    }

    pub fn actor_stats_mut(&mut self, id: EntityId) -> Option<&mut ActorStats> {
        self.get_mut(id)
            .and_then(|entity| entity.actor_mut().map(|(_, stats, _)| stats))
    }

    pub fn item_data(&self, id: EntityId) -> Option<&ItemData> {
        self.get(id)
            .and_then(|entity| entity.item().map(|(_, data, _, _, _)| data))
    }

    pub fn item_location(&self, id: EntityId) -> Option<EntityLocation> {
        self.get(id)
            .and_then(|entity| entity.item().map(|(_, _, location, _, _)| location))
    }

    pub fn item_letter(&self, id: EntityId) -> Option<InventoryLetter> {
        self.get(id)
            .and_then(|entity| entity.item().and_then(|(_, _, _, letter, _)| letter))
    }

    pub fn item_charges(&self, id: EntityId) -> Option<u8> {
        self.get(id)
            .and_then(|entity| entity.item().and_then(|(_, _, _, _, charges)| charges))
    }

    pub fn set_item_charges(&mut self, id: EntityId, charges: Option<u8>) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let Some((_, _, _, charges_ref)) = entity.item_mut() else {
            return false;
        };
        *charges_ref = charges;
        true
    }

    pub fn clear_monsters(&mut self) {
        for entity in &mut self.entities {
            if matches!(entity.kind(), EntityKind::Monster(_)) {
                if let Some((_, _, alive)) = entity.actor_mut() {
                    *alive = false;
                }
            }
        }
    }

    fn next_entity_id(&mut self) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        id
    }
}

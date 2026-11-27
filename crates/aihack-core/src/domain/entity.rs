use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        combat::{AttackProfile, DamageRoll},
        inventory::InventoryLetter,
        item::{BucStatus, ItemData, ItemKind},
        monster::{MonsterAiKind, MonsterKind, MonsterPassive, MonsterTemplate},
        player::adventurer_template,
        status::Resistances,
    },
    error::EntityAllocationError,
    ids::{EntityId, LevelId},
    position::Pos,
    world::{is_zero_i16, is_zero_u16, is_zero_u32, is_zero_u8},
};

pub const MAX_ABSOLUTE_ACTOR_STAT: i32 = 10_000;
pub const MAX_PET_TAMENESS: u8 = 20;
pub const MAX_PET_XP: u32 = 1_000;
pub const MAX_PET_HUNGER: i16 = 1_500;
pub const DEFAULT_PET_HUNGER: i16 = 600;

fn is_zero_i8(value: &i8) -> bool {
    *value == 0
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn default_quantity_one() -> u16 {
    1
}

fn is_one_u16(value: &u16) -> bool {
    *value == 1
}

/// Actor payload 내부 종류다. Item은 별도 payload로 분리한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorKind {
    Player,
    Monster(MonsterKind),
}

impl ActorKind {
    /// actor가 monster라면 monster kind를 돌려준다.
    pub fn monster_kind(self) -> Option<MonsterKind> {
        match self {
            Self::Monster(kind) => Some(kind),
            Self::Player => None,
        }
    }
}

/// 외부 관찰/snapshot에서 쓰는 통합 entity 종류다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    Player,
    Monster(MonsterKind),
    Item(ItemKind),
}

/// 전투 관계를 판정하기 위한 faction이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    Player,
    Pet,
    Peaceful,
    Shopkeeper,
    Hostile,
    Neutral,
}

/// actor stat이며 item payload는 이 값을 갖지 않는다.
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
    #[serde(default = "default_actor_speed")]
    pub speed: i16,
    #[serde(default, skip_serializing_if = "is_zero_i16")]
    pub energy: i16,
    #[serde(default, skip_serializing_if = "Resistances::is_empty")]
    pub resistances: Resistances,
    #[serde(default)]
    pub ai_kind: Option<MonsterAiKind>,
    #[serde(default)]
    pub passive: Option<MonsterPassive>,
    #[serde(default)]
    pub difficulty: u16,
}

fn default_actor_speed() -> i16 {
    12
}

/// actor/item 공용 위치다. Consumed tombstone은 assigned_letter를 유지한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityLocation {
    OnMap { level: LevelId, pos: Pos },
    Inventory { owner: EntityId },
    InContainer { container: EntityId },
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
            Self::Inventory { .. } | Self::InContainer { .. } | Self::Consumed => None,
        }
    }
}

/// actor와 item의 invalid state를 막는 payload 분리 구조다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityPayload {
    Actor {
        kind: ActorKind,
        faction: Faction,
        location: EntityLocation,
        stats: ActorStats,
        alive: bool,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        monster_inventory: Vec<EntityId>,
        #[serde(default, skip_serializing_if = "is_zero_u8")]
        tameness: u8,
        #[serde(default, skip_serializing_if = "is_zero_u32")]
        pet_xp: u32,
        #[serde(default, skip_serializing_if = "is_zero_i16")]
        pet_hunger: i16,
    },
    Item {
        kind: ItemKind,
        data: ItemData,
        location: EntityLocation,
        assigned_letter: Option<InventoryLetter>,
        charges: Option<u8>,
        #[serde(default, skip_serializing_if = "BucStatus::is_uncursed")]
        buc: BucStatus,
        #[serde(default, skip_serializing_if = "is_zero_i8")]
        enchantment: i8,
        #[serde(default = "default_quantity_one", skip_serializing_if = "is_one_u16")]
        quantity: u16,
        #[serde(default, skip_serializing_if = "is_false")]
        buc_known: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        enchant_known: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        corpse_monster: Option<MonsterKind>,
        #[serde(default, skip_serializing_if = "is_zero_u16")]
        corpse_age: u16,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        contained_items: Vec<EntityId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        artifact_id: Option<String>,
        #[serde(default, skip_serializing_if = "is_false")]
        tool_active: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        container_locked: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        container_trapped: bool,
        #[serde(default, skip_serializing_if = "is_zero_u8")]
        erosion: u8,
        #[serde(default, skip_serializing_if = "is_false")]
        unpaid: bool,
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

/// entity store에 저장되는 통합 엔티티다.
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
                ..
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
        match &self.payload {
            EntityPayload::Actor {
                kind: ActorKind::Monster(kind),
                stats,
                ..
            } => stats.ai_kind.or_else(|| Some(kind.ai_kind())),
            _ => None,
        }
    }

    pub fn monster_passive(&self) -> Option<MonsterPassive> {
        match &self.payload {
            EntityPayload::Actor {
                kind: ActorKind::Monster(kind),
                stats,
                ..
            } => stats.passive.or_else(|| {
                (stats.ai_kind.is_none() && *kind == MonsterKind::FloatingEye)
                    .then_some(MonsterPassive::ParalyzeOnMelee)
            }),
            _ => None,
        }
    }

    pub fn monster_difficulty(&self) -> Option<u16> {
        match &self.payload {
            EntityPayload::Actor {
                kind: ActorKind::Monster(kind),
                stats,
                ..
            } => Some(if stats.difficulty == 0 {
                u16::from(kind.difficulty())
            } else {
                stats.difficulty
            }),
            _ => None,
        }
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
                ..
            } => Some((*kind, data, *location, *assigned_letter, *charges)),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_buc(&self) -> Option<BucStatus> {
        match &self.payload {
            EntityPayload::Item { buc, .. } => Some(*buc),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_enchantment(&self) -> Option<i8> {
        match &self.payload {
            EntityPayload::Item { enchantment, .. } => Some(*enchantment),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_quantity(&self) -> Option<u16> {
        match &self.payload {
            EntityPayload::Item { quantity, .. } => Some(*quantity),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_buc_known(&self) -> Option<bool> {
        match &self.payload {
            EntityPayload::Item { buc_known, .. } => Some(*buc_known),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_enchant_known(&self) -> Option<bool> {
        match &self.payload {
            EntityPayload::Item { enchant_known, .. } => Some(*enchant_known),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_corpse_monster(&self) -> Option<MonsterKind> {
        match &self.payload {
            EntityPayload::Item { corpse_monster, .. } => *corpse_monster,
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_corpse_age(&self) -> Option<u16> {
        match &self.payload {
            EntityPayload::Item { corpse_age, .. } => Some(*corpse_age),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_contained_items(&self) -> Option<&[EntityId]> {
        match &self.payload {
            EntityPayload::Item {
                contained_items, ..
            } => Some(contained_items),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_artifact_id(&self) -> Option<&str> {
        match &self.payload {
            EntityPayload::Item { artifact_id, .. } => artifact_id.as_deref(),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_tool_active(&self) -> Option<bool> {
        match &self.payload {
            EntityPayload::Item { tool_active, .. } => Some(*tool_active),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_container_locked(&self) -> Option<bool> {
        match &self.payload {
            EntityPayload::Item {
                container_locked, ..
            } => Some(*container_locked),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_container_trapped(&self) -> Option<bool> {
        match &self.payload {
            EntityPayload::Item {
                container_trapped, ..
            } => Some(*container_trapped),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn item_erosion(&self) -> Option<u8> {
        match &self.payload {
            EntityPayload::Item { erosion, .. } => Some(*erosion),
            EntityPayload::Actor { .. } => None,
        }
    }

    pub fn actor_monster_inventory(&self) -> Option<&[EntityId]> {
        match &self.payload {
            EntityPayload::Actor {
                monster_inventory, ..
            } => Some(monster_inventory),
            EntityPayload::Item { .. } => None,
        }
    }

    pub fn actor_tameness(&self) -> Option<u8> {
        match &self.payload {
            EntityPayload::Actor { tameness, .. } => Some(*tameness),
            EntityPayload::Item { .. } => None,
        }
    }

    pub fn actor_pet_xp(&self) -> Option<u32> {
        match &self.payload {
            EntityPayload::Actor { pet_xp, .. } => Some(*pet_xp),
            EntityPayload::Item { .. } => None,
        }
    }

    pub fn actor_pet_hunger(&self) -> Option<i16> {
        match &self.payload {
            EntityPayload::Actor { pet_hunger, .. } => Some(*pet_hunger),
            EntityPayload::Item { .. } => None,
        }
    }

    pub fn item_unpaid(&self) -> Option<bool> {
        match &self.payload {
            EntityPayload::Item { unpaid, .. } => Some(*unpaid),
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
                kind: ActorKind::Monster(MonsterKind::Kobold),
                stats,
                ..
            } => Some(AttackProfile::natural("spear", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Orc),
                stats,
                ..
            } => Some(AttackProfile::natural("scimitar", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Ogre),
                stats,
                ..
            } => Some(AttackProfile::natural("club", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::GiantSpider),
                stats,
                ..
            } => Some(AttackProfile::natural("fangs", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::GnollArcher),
                stats,
                ..
            } => Some(AttackProfile::natural("arrow", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Troll),
                stats,
                ..
            } => Some(AttackProfile::natural("claw", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::FireAnt),
                stats,
                ..
            } => Some(AttackProfile::natural("fire bite", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Minotaur),
                stats,
                ..
            } => Some(AttackProfile::natural("gore", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Dragon),
                stats,
                ..
            } => Some(AttackProfile::natural("bite", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Bat),
                stats,
                ..
            } => Some(AttackProfile::natural("bite", stats.damage)),
            EntityPayload::Actor {
                kind:
                    ActorKind::Monster(
                        MonsterKind::Kitten
                        | MonsterKind::Housecat
                        | MonsterKind::LittleDog
                        | MonsterKind::Dog
                        | MonsterKind::RustMonster,
                    ),
                stats,
                ..
            } => Some(AttackProfile::natural("bite", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Pony | MonsterKind::Warhorse),
                stats,
                ..
            } => Some(AttackProfile::natural("kick", stats.damage)),
            EntityPayload::Actor {
                kind:
                    ActorKind::Monster(
                        MonsterKind::Shopkeeper | MonsterKind::TemplePriest | MonsterKind::Watchman,
                    ),
                stats,
                ..
            } => Some(AttackProfile::natural("weapon", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Oracle | MonsterKind::Nymph),
                stats,
                ..
            } => Some(AttackProfile::natural("touch", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Leprechaun),
                stats,
                ..
            } => Some(AttackProfile::natural("claw", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::BlackPudding),
                stats,
                ..
            } => Some(AttackProfile::natural("engulf", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Dwarf | MonsterKind::OrcCaptain),
                stats,
                ..
            } => Some(AttackProfile::natural("axe", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Monster(MonsterKind::Medusa),
                stats,
                ..
            } => Some(AttackProfile::natural("snakes", stats.damage)),
            EntityPayload::Actor {
                kind: ActorKind::Player,
                ..
            } => None,
            EntityPayload::Item { data, .. } => data.attack_profile,
        }
    }
}

/// Vec index 안정성을 위해 consumed/dead 엔티티를 즉시 제거하지 않는 저장소다.
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

    pub fn spawn_player(&mut self, pos: Pos) -> Result<EntityId, EntityAllocationError> {
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
                speed: 12,
                energy: 0,
                resistances: Resistances::NONE,
                ai_kind: None,
                passive: None,
                difficulty: 0,
            },
        )
    }

    pub fn spawn_monster_with_template(
        &mut self,
        kind: MonsterKind,
        template: MonsterTemplate,
        pos: Pos,
    ) -> Result<EntityId, EntityAllocationError> {
        self.spawn_monster_with_faction(kind, template, Faction::Hostile, pos)
    }

    pub fn spawn_monster_with_faction(
        &mut self,
        kind: MonsterKind,
        template: MonsterTemplate,
        faction: Faction,
        pos: Pos,
    ) -> Result<EntityId, EntityAllocationError> {
        self.spawn_actor(
            ActorKind::Monster(kind),
            faction,
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
                speed: template.speed,
                energy: 0,
                resistances: template.resistances,
                ai_kind: Some(template.ai_kind),
                passive: template.passive,
                difficulty: template.difficulty,
            },
        )
    }

    pub fn spawn_actor(
        &mut self,
        kind: ActorKind,
        faction: Faction,
        pos: Pos,
        stats: ActorStats,
    ) -> Result<EntityId, EntityAllocationError> {
        let id = self.next_entity_id()?;
        self.entities.push(Entity {
            id,
            payload: EntityPayload::Actor {
                kind,
                faction,
                location: EntityLocation::on_main_level(pos),
                stats,
                alive: true,
                monster_inventory: Vec::new(),
                tameness: 0,
                pet_xp: 0,
                pet_hunger: 0,
            },
        });
        Ok(id)
    }

    pub fn spawn_item_with_data(
        &mut self,
        kind: ItemKind,
        data: ItemData,
        location: EntityLocation,
    ) -> Result<EntityId, EntityAllocationError> {
        let id = self.next_entity_id()?;
        self.entities.push(Entity {
            id,
            payload: EntityPayload::Item {
                kind,
                data,
                location,
                assigned_letter: None,
                charges: data.max_charges,
                buc: BucStatus::Uncursed,
                enchantment: 0,
                quantity: 1,
                buc_known: false,
                enchant_known: false,
                corpse_monster: None,
                corpse_age: 0,
                contained_items: Vec::new(),
                artifact_id: None,
                tool_active: false,
                container_locked: false,
                container_trapped: false,
                erosion: 0,
                unpaid: false,
            },
        });
        Ok(id)
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

    /// 저장 복원 validator가 allocator 충돌을 검사하기 위한 읽기 전용 값이다.
    pub fn next_id(&self) -> u32 {
        self.next_id
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

    pub fn non_player_actors_on_level(&self, level: LevelId) -> Vec<EntityId> {
        let mut ids = self
            .entities
            .iter()
            .filter_map(|entity| {
                let (kind, _, actor_level, _, _, alive) = entity.actor()?;
                (alive && actor_level == level && matches!(kind, ActorKind::Monster(_)))
                    .then_some(entity.id)
            })
            .collect::<Vec<_>>();
        ids.sort_by_key(|id| id.0);
        ids
    }

    pub fn alive_friendly_at(&self, level: LevelId, pos: Pos) -> Option<EntityId> {
        self.entities.iter().find_map(|entity| {
            let (kind, faction, actor_level, actor_pos, _, alive) = entity.actor()?;
            (alive
                && actor_level == level
                && actor_pos == pos
                && matches!(kind, ActorKind::Monster(_))
                && matches!(
                    faction,
                    Faction::Pet | Faction::Peaceful | Faction::Shopkeeper
                ))
            .then_some(entity.id)
        })
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
        let Some((_, stats, alive_ref)) = entity.actor_mut() else {
            return false;
        };
        // alive/HP는 저장·world invariant의 한 관계다. fixture와 runtime helper도 절반만 바꾸지 않는다.
        if alive && stats.hp <= 0 {
            stats.hp = 1;
        } else if !alive && stats.hp > 0 {
            stats.hp = 0;
        }
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

    pub fn set_item_buc(&mut self, id: EntityId, next: BucStatus) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item { buc, .. } = &mut entity.payload else {
            return false;
        };
        *buc = next;
        true
    }

    pub fn set_item_enchantment(&mut self, id: EntityId, next: i8) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item { enchantment, .. } = &mut entity.payload else {
            return false;
        };
        *enchantment = next;
        true
    }

    pub fn set_item_quantity(&mut self, id: EntityId, next: u16) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item { quantity, .. } = &mut entity.payload else {
            return false;
        };
        *quantity = next;
        true
    }

    pub fn set_item_buc_known(&mut self, id: EntityId, known: bool) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item { buc_known, .. } = &mut entity.payload else {
            return false;
        };
        *buc_known = known;
        true
    }

    pub fn set_item_enchant_known(&mut self, id: EntityId, known: bool) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item { enchant_known, .. } = &mut entity.payload else {
            return false;
        };
        *enchant_known = known;
        true
    }

    pub fn set_item_corpse_monster(&mut self, id: EntityId, monster: Option<MonsterKind>) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item { corpse_monster, .. } = &mut entity.payload else {
            return false;
        };
        *corpse_monster = monster;
        true
    }

    pub fn set_item_corpse_age(&mut self, id: EntityId, age: u16) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item { corpse_age, .. } = &mut entity.payload else {
            return false;
        };
        *corpse_age = age;
        true
    }

    pub fn set_item_artifact_id(&mut self, id: EntityId, artifact: Option<String>) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item { artifact_id, .. } = &mut entity.payload else {
            return false;
        };
        *artifact_id = artifact;
        true
    }

    pub fn set_item_tool_active(&mut self, id: EntityId, active: bool) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item { tool_active, .. } = &mut entity.payload else {
            return false;
        };
        *tool_active = active;
        true
    }

    pub fn set_item_container_locked(&mut self, id: EntityId, locked: bool) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item {
            container_locked, ..
        } = &mut entity.payload
        else {
            return false;
        };
        *container_locked = locked;
        true
    }

    pub fn set_item_container_trapped(&mut self, id: EntityId, trapped: bool) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item {
            container_trapped, ..
        } = &mut entity.payload
        else {
            return false;
        };
        *container_trapped = trapped;
        true
    }

    pub fn set_item_erosion(&mut self, id: EntityId, next: u8) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item { erosion, .. } = &mut entity.payload else {
            return false;
        };
        *erosion = next;
        true
    }

    pub fn push_contained_item(&mut self, container: EntityId, item: EntityId) -> bool {
        let Some(entity) = self.get_mut(container) else {
            return false;
        };
        let EntityPayload::Item {
            contained_items, ..
        } = &mut entity.payload
        else {
            return false;
        };
        if !contained_items.contains(&item) {
            contained_items.push(item);
        }
        true
    }

    pub fn remove_contained_item(&mut self, container: EntityId, item: EntityId) -> bool {
        let Some(entity) = self.get_mut(container) else {
            return false;
        };
        let EntityPayload::Item {
            contained_items, ..
        } = &mut entity.payload
        else {
            return false;
        };
        let before = contained_items.len();
        contained_items.retain(|id| *id != item);
        contained_items.len() != before
    }

    pub fn clear_contained_items(&mut self, container: EntityId) -> bool {
        let Some(entity) = self.get_mut(container) else {
            return false;
        };
        let EntityPayload::Item {
            contained_items, ..
        } = &mut entity.payload
        else {
            return false;
        };
        contained_items.clear();
        true
    }

    pub fn set_item_unpaid(&mut self, id: EntityId, unpaid: bool) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item {
            unpaid: unpaid_ref, ..
        } = &mut entity.payload
        else {
            return false;
        };
        *unpaid_ref = unpaid;
        true
    }

    pub fn set_actor_faction(&mut self, id: EntityId, next: Faction) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Actor { faction, .. } = &mut entity.payload else {
            return false;
        };
        *faction = next;
        true
    }

    pub fn set_actor_kind(&mut self, id: EntityId, next: ActorKind) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Actor { kind, .. } = &mut entity.payload else {
            return false;
        };
        *kind = next;
        true
    }

    pub fn set_actor_tameness(&mut self, id: EntityId, next: u8) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Actor { tameness, .. } = &mut entity.payload else {
            return false;
        };
        *tameness = next.min(MAX_PET_TAMENESS);
        true
    }

    pub fn set_actor_pet_xp(&mut self, id: EntityId, next: u32) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Actor { pet_xp, .. } = &mut entity.payload else {
            return false;
        };
        *pet_xp = next.min(MAX_PET_XP);
        true
    }

    pub fn set_actor_pet_hunger(&mut self, id: EntityId, next: i16) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Actor { pet_hunger, .. } = &mut entity.payload else {
            return false;
        };
        *pet_hunger = next.clamp(0, MAX_PET_HUNGER);
        true
    }

    pub fn push_monster_inventory(&mut self, actor: EntityId, item: EntityId) -> bool {
        let Some(entity) = self.get_mut(actor) else {
            return false;
        };
        let EntityPayload::Actor {
            monster_inventory, ..
        } = &mut entity.payload
        else {
            return false;
        };
        if !monster_inventory.contains(&item) {
            monster_inventory.push(item);
        }
        true
    }

    pub fn remove_monster_inventory(&mut self, actor: EntityId, item: EntityId) -> bool {
        let Some(entity) = self.get_mut(actor) else {
            return false;
        };
        let EntityPayload::Actor {
            monster_inventory, ..
        } = &mut entity.payload
        else {
            return false;
        };
        let before = monster_inventory.len();
        monster_inventory.retain(|id| *id != item);
        monster_inventory.len() != before
    }

    pub fn clear_monster_inventory(&mut self, actor: EntityId) -> Vec<EntityId> {
        let Some(entity) = self.get_mut(actor) else {
            return Vec::new();
        };
        let EntityPayload::Actor {
            monster_inventory, ..
        } = &mut entity.payload
        else {
            return Vec::new();
        };
        std::mem::take(monster_inventory)
    }

    pub fn clear_item_letter(&mut self, id: EntityId) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let Some((_, _, assigned_letter, _)) = entity.item_mut() else {
            return false;
        };
        *assigned_letter = None;
        true
    }

    pub fn set_item_kind_and_data(&mut self, id: EntityId, kind: ItemKind, data: ItemData) -> bool {
        let Some(entity) = self.get_mut(id) else {
            return false;
        };
        let EntityPayload::Item {
            kind: kind_ref,
            data: data_ref,
            charges,
            ..
        } = &mut entity.payload
        else {
            return false;
        };
        *kind_ref = kind;
        *data_ref = data;
        *charges = data.max_charges;
        true
    }

    pub fn clear_monsters(&mut self) {
        for entity in &mut self.entities {
            if matches!(entity.kind(), EntityKind::Monster(_)) {
                if let Some((_, stats, alive)) = entity.actor_mut() {
                    stats.hp = stats.hp.min(0);
                    *alive = false;
                }
            }
        }
    }

    fn next_entity_id(&mut self) -> Result<EntityId, EntityAllocationError> {
        let id = EntityId(self.next_id);
        let next_id = self
            .next_id
            .checked_add(1)
            .ok_or(EntityAllocationError::Exhausted)?;
        self.next_id = next_id;
        Ok(id)
    }
}

#[cfg(test)]
mod allocation_tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::EntityStore;
    use crate::{error::EntityAllocationError, position::Pos};

    #[test]
    fn exhausted_allocator_returns_without_panicking_or_mutating_the_store() {
        let mut store = EntityStore::new();
        store.next_id = u32::MAX;
        let before = store.clone();

        let result = catch_unwind(AssertUnwindSafe(|| store.spawn_player(Pos { x: 1, y: 1 })));

        assert!(result.is_ok(), "allocator exhaustion must be a typed error");
        assert_eq!(result.unwrap(), Err(EntityAllocationError::Exhausted));
        assert_eq!(store, before, "failed allocation must not mutate the store");
    }
}

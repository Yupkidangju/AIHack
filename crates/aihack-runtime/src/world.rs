use std::ops::Deref;

use aihack_content::ContentRegistry;
use aihack_core::{
    domain::{
        inventory::Inventory,
        item::ItemKind,
        level::LevelRegistry,
        map::GameMap,
        status::{HungerState, Status},
    },
    error::ContentError,
    ids::{EntityId, LevelId},
    invariant::{validate_world, InvariantReport, WorldInvariantView},
    movement::MovementWorld,
    position::Pos,
    score::DeathScoreView,
    world::WorldState,
};

use crate::domain::entity::{EntityKind, EntityStore};

pub type SavedWorldV1 = aihack_core::save::SavedWorldV1<EntityStore>;

pub const PHASE3_JACKAL_START: Pos = Pos { x: 6, y: 5 };
pub const PHASE3_GOBLIN_START: Pos = Pos { x: 20, y: 12 };
pub const PHASE4_POTION_START: Pos = Pos { x: 8, y: 5 };
pub const PHASE7_WAND_START_CHARGES: u8 = 3;

#[derive(Debug, Clone)]
pub struct GameWorld {
    state: WorldState<EntityStore>,
    registry: ContentRegistry,
    corpse_jackal_data: aihack_core::domain::item::ItemData,
}

impl PartialEq for GameWorld {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
            && self.registry.content_hash() == other.registry.content_hash()
            && self.corpse_jackal_data == other.corpse_jackal_data
    }
}

impl Eq for GameWorld {}

impl Deref for GameWorld {
    type Target = WorldState<EntityStore>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl GameWorld {
    pub fn from_saved_world(saved: SavedWorldV1) -> Result<Self, aihack_core::error::GameError> {
        crate::save::validate_saved_world(&saved)?;
        Ok(Self::from_saved_world_with_registry(
            saved,
            aihack_content::registry()?,
        )?)
    }

    /// 호환 시나리오의 `ResolveDeath` 직전 HP 0 world만 구성하는 제한 경계다.
    #[cfg(feature = "testing")]
    pub(crate) fn from_depleted_saved_world(
        saved: SavedWorldV1,
    ) -> Result<Self, aihack_core::error::GameError> {
        crate::save::validate_depleted_saved_world(&saved)?;
        Ok(Self::from_saved_world_with_registry(
            saved,
            aihack_content::registry()?,
        )?)
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn try_fixture_phase5() -> Result<Self, ContentError> {
        Self::try_fixture_phase5_with_registry(aihack_content::registry()?)
    }

    pub(crate) fn try_fixture_phase5_with_registry(
        registry: &ContentRegistry,
    ) -> Result<Self, ContentError> {
        let state = crate::bootstrap::initial_world(registry)?;
        let corpse_jackal_data =
            crate::domain::item::try_item_data_from_registry(ItemKind::CorpseJackal, registry)?;
        Ok(Self {
            state,
            registry: registry.clone(),
            corpse_jackal_data,
        })
    }

    pub(crate) fn fixture_phase4() -> Self {
        Self::try_fixture_phase5_with_registry(
            aihack_content::registry().expect("embedded content registry must validate"),
        )
        .expect("embedded content registry must build the default world")
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn fixture_phase5() -> Self {
        Self::try_fixture_phase5()
            .expect("embedded content registry must validate for the default fixture")
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn fixture_without_monsters() -> Self {
        let mut world = Self::fixture_phase5();
        world.state.entities.clear_monsters();
        world
    }

    pub fn validate_invariants(&self) -> InvariantReport {
        validate_world(self)
    }

    pub fn state(&self) -> &WorldState<EntityStore> {
        &self.state
    }

    pub(crate) fn state_mut(&mut self) -> &mut WorldState<EntityStore> {
        &mut self.state
    }

    pub fn current_level(&self) -> LevelId {
        self.current_level
    }

    pub fn player_id(&self) -> EntityId {
        self.player_id
    }

    pub fn levels(&self) -> &LevelRegistry {
        &self.levels
    }

    pub fn entities(&self) -> &EntityStore {
        &self.entities
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub fn current_map(&self) -> &GameMap {
        self.map(self.current_level)
    }

    pub(crate) fn current_map_mut(&mut self) -> &mut GameMap {
        self.map_mut(self.state.current_level)
    }

    pub fn map(&self, level: LevelId) -> &GameMap {
        self.levels
            .map(level)
            .expect("검증된 world는 요청한 level map을 가진다")
    }

    pub(crate) fn map_mut(&mut self, level: LevelId) -> &mut GameMap {
        self.state
            .levels
            .map_mut(level)
            .expect("검증된 world는 갱신할 level map을 가진다")
    }

    pub fn player_location(&self) -> (LevelId, Pos) {
        self.entities
            .actor_location(self.player_id)
            .expect("검증된 world는 player actor 위치를 가진다")
    }

    pub(crate) fn set_player_location(&mut self, level: LevelId, pos: Pos) {
        let player_id = self.player_id;
        assert!(
            self.state
                .entities
                .set_actor_location(player_id, level, pos),
            "검증된 world는 player actor를 가진다"
        );
        self.state.current_level = level;
    }

    pub fn player_pos(&self) -> Pos {
        let (level, pos) = self.player_location();
        debug_assert_eq!(level, self.current_level);
        pos
    }

    pub fn player_alive(&self) -> bool {
        self.entities
            .get(self.player_id)
            .and_then(|entity| entity.actor().map(|(_, _, _, _, _, alive)| alive))
            .unwrap_or(false)
    }

    pub fn current_level_hostile_monsters(&self) -> Vec<EntityId> {
        self.entities.hostile_monsters_on_level(self.current_level)
    }

    pub(crate) fn identify_item_kind(&mut self, kind: ItemKind) {
        if !self.state.identified_items.contains(&kind) {
            self.state.identified_items.push(kind);
            self.state.identified_items.sort_by_key(|kind| *kind as u8);
        }
    }

    pub fn is_item_identified(&self, kind: ItemKind) -> bool {
        self.identified_items.contains(&kind)
    }

    pub fn gold(&self) -> u32 {
        self.gold
    }
    pub fn kill_count(&self) -> u32 {
        self.kill_count
    }

    pub fn carried_weight(&self) -> i16 {
        self.inventory
            .entries
            .iter()
            .filter_map(|entry| self.entities.item_data(entry.item).map(|data| data.weight))
            .fold(0, i16::saturating_add)
    }

    pub fn status(&self) -> Status {
        Status {
            nutrition: self.nutrition,
            luck: self.luck,
            prayer_cooldown: self.prayer_cooldown,
            paralysis_turns: self.paralysis_turns,
            hallucinating: self.hallucinating,
        }
    }

    pub fn hunger_state(&self) -> HungerState {
        self.status().hunger_state()
    }

    pub(crate) fn from_saved_world_with_registry(
        saved: SavedWorldV1,
        registry: &ContentRegistry,
    ) -> Result<Self, ContentError> {
        let corpse_jackal_data =
            crate::domain::item::try_item_data_from_registry(ItemKind::CorpseJackal, registry)?;
        Ok(Self {
            state: saved.into(),
            registry: registry.clone(),
            corpse_jackal_data,
        })
    }

    pub(crate) fn content_registry(&self) -> &ContentRegistry {
        &self.registry
    }

    pub(crate) fn corpse_jackal_data(&self) -> aihack_core::domain::item::ItemData {
        self.corpse_jackal_data
    }
}

impl WorldInvariantView for GameWorld {
    fn current_level_id(&self) -> LevelId {
        self.current_level()
    }
    fn level_exists(&self, level: LevelId) -> bool {
        self.levels.map(level).is_some()
    }
    fn contains_position(&self, level: LevelId, pos: Pos) -> bool {
        self.levels.map(level).is_some_and(|map| map.contains(pos))
    }
    fn player_entity_id(&self) -> EntityId {
        self.player_id
    }
    fn entity_kind(&self, entity: EntityId) -> Option<EntityKind> {
        self.entities.get(entity).map(|entry| entry.kind())
    }
    fn actor_location(&self, entity: EntityId) -> Option<(LevelId, Pos)> {
        self.entities.actor_location(entity)
    }
    fn inventory_owner(&self) -> EntityId {
        self.inventory.owner
    }
}

impl MovementWorld for GameWorld {
    fn map(&self, level: LevelId) -> &GameMap {
        GameWorld::map(self, level)
    }
    fn actor_location(&self, actor: EntityId) -> Option<(LevelId, Pos)> {
        self.entities.actor_location(actor)
    }
    fn alive_actor_at(&self, level: LevelId, pos: Pos) -> Option<EntityId> {
        self.entities.alive_actor_at(level, pos)
    }
}

impl DeathScoreView for GameWorld {
    fn gold_amount(&self) -> u32 {
        self.gold
    }
    fn kill_count(&self) -> u32 {
        self.kill_count
    }
    fn current_level_depth(&self) -> i16 {
        self.current_level().depth
    }
    fn inventory_value(&self) -> u32 {
        self.inventory
            .entries
            .iter()
            .filter_map(|entry| self.entities.item_data(entry.item))
            .map(|data| data.base_price)
            .fold(0, u32::saturating_add)
    }
}

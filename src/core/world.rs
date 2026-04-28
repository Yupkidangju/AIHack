use serde::{Deserialize, Serialize};

use crate::{
    core::{
        ids::{EntityId, LevelId},
        position::Pos,
    },
    domain::{
        entity::{EntityLocation, EntityStore},
        inventory::Inventory,
        item::ItemKind,
        level::{LevelRegistry, PHASE5_LEVEL1_ID},
        map::{GameMap, PHASE2_PLAYER_START},
        monster::MonsterKind,
    },
};

pub const PHASE3_JACKAL_START: Pos = Pos { x: 6, y: 5 };
pub const PHASE3_GOBLIN_START: Pos = Pos { x: 20, y: 12 };
pub const PHASE4_POTION_START: Pos = Pos { x: 8, y: 5 };

/// [v0.1.0] Phase 5 world는 fixed level registry와 단일 entity namespace를 보유한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameWorld {
    pub levels: LevelRegistry,
    current_level: LevelId,
    pub entities: EntityStore,
    pub player_id: EntityId,
    pub inventory: Inventory,
}

impl GameWorld {
    pub fn fixture_phase2() -> Self {
        Self::fixture_phase5()
    }

    pub fn fixture_phase3() -> Self {
        Self::fixture_phase5()
    }

    pub fn fixture_phase4() -> Self {
        Self::fixture_phase5()
    }

    pub fn fixture_phase5() -> Self {
        let mut entities = EntityStore::new();
        let player_id = entities.spawn_player(PHASE2_PLAYER_START);
        entities.spawn_monster(MonsterKind::Jackal, PHASE3_JACKAL_START);
        entities.spawn_monster(MonsterKind::Goblin, PHASE3_GOBLIN_START);
        entities.spawn_item(
            ItemKind::PotionHealing,
            EntityLocation::OnMap {
                level: PHASE5_LEVEL1_ID,
                pos: PHASE4_POTION_START,
            },
        );

        let mut inventory = Inventory::new(player_id);
        let dagger = entities.spawn_item(
            ItemKind::Dagger,
            EntityLocation::Inventory { owner: player_id },
        );
        let dagger_letter = inventory
            .add_existing_with_next_letter(dagger)
            .expect("Phase 5 시작 inventory는 letter 용량을 초과하지 않는다");
        entities.set_item_letter(dagger, dagger_letter);

        let food = entities.spawn_item(
            ItemKind::FoodRation,
            EntityLocation::Inventory { owner: player_id },
        );
        let food_letter = inventory
            .add_existing_with_next_letter(food)
            .expect("Phase 5 시작 inventory는 letter 용량을 초과하지 않는다");
        entities.set_item_letter(food, food_letter);

        Self {
            levels: LevelRegistry::fixture_phase5(),
            current_level: PHASE5_LEVEL1_ID,
            entities,
            player_id,
            inventory,
        }
    }

    pub fn fixture_without_monsters() -> Self {
        let mut world = Self::fixture_phase5();
        world.entities.clear_monsters();
        world
    }

    pub fn current_level(&self) -> LevelId {
        self.current_level
    }

    pub fn current_map(&self) -> &GameMap {
        self.levels
            .map(self.current_level)
            .expect("Phase 5 world는 항상 current level map을 가진다")
    }

    pub fn current_map_mut(&mut self) -> &mut GameMap {
        self.levels
            .map_mut(self.current_level)
            .expect("Phase 5 world는 항상 current level map을 가진다")
    }

    pub fn map(&self, level: LevelId) -> &GameMap {
        self.levels
            .map(level)
            .expect("Phase 5 world는 fixed level map만 조회한다")
    }

    pub fn map_mut(&mut self, level: LevelId) -> &mut GameMap {
        self.levels
            .map_mut(level)
            .expect("Phase 5 world는 fixed level map만 갱신한다")
    }

    pub fn player_location(&self) -> (LevelId, Pos) {
        self.entities
            .actor_location(self.player_id)
            .expect("Phase 5 world는 항상 player actor 위치를 가진다")
    }

    pub fn set_player_location(&mut self, level: LevelId, pos: Pos) {
        assert!(
            self.entities.set_actor_location(self.player_id, level, pos),
            "Phase 5 world는 항상 player actor를 가진다"
        );
        self.current_level = level;
    }

    pub fn player_pos(&self) -> Pos {
        let (level, pos) = self.player_location();
        debug_assert_eq!(level, self.current_level);
        pos
    }

    pub fn set_player_pos(&mut self, pos: Pos) {
        self.set_player_location(self.current_level, pos);
    }

    pub fn player_alive(&self) -> bool {
        self.entities
            .get(self.player_id)
            .and_then(|entity| entity.actor().map(|(_, _, _, _, _, alive)| alive))
            .unwrap_or(false)
    }
}

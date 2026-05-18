use serde::{Deserialize, Serialize};

use crate::{
    core::{
        ids::{EntityId, LevelId},
        position::Pos,
        save::SavedWorldV1,
    },
    domain::{
        combat::DeathCause,
        entity::{EntityLocation, EntityStore},
        inventory::Inventory,
        item::ItemKind,
        level::{LevelRegistry, PHASE5_LEVEL1_ID},
        map::{GameMap, PHASE2_PLAYER_START},
        monster::MonsterKind,
        status::{HungerState, Status},
    },
};

pub const PHASE3_JACKAL_START: Pos = Pos { x: 6, y: 5 };
pub const PHASE3_GOBLIN_START: Pos = Pos { x: 20, y: 12 };
pub const PHASE4_POTION_START: Pos = Pos { x: 8, y: 5 };
pub const PHASE7_WAND_START_CHARGES: u8 = 3;

/// [v0.1.0] Phase 5 world는 fixed level registry와 단일 entity namespace를 보유한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameWorld {
    pub levels: LevelRegistry,
    current_level: LevelId,
    pub entities: EntityStore,
    pub player_id: EntityId,
    pub inventory: Inventory,
    pub nutrition: i16,
    pub luck: i16,
    pub prayer_cooldown: u16,
    pub paralysis_turns: u8,
    pub hallucinating: bool,
    pub kill_count: u32,
    pub gold: u32,
    pub identified_items: Vec<ItemKind>,
    /// [v0.2.0] Phase 16: 플레이어 사망 시 사망 원인을 기록한다.
    /// GameOver 상태 생성 시 이 필드를 읽어 cause를 결정한다.
    pub last_death_cause: Option<DeathCause>,
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

        let wand = entities.spawn_item(
            ItemKind::WandMagicMissile,
            EntityLocation::Inventory { owner: player_id },
        );
        let wand_letter = inventory
            .add_existing_with_next_letter(wand)
            .expect("Phase 7 시작 inventory는 letter 용량을 초과하지 않는다");
        entities.set_item_letter(wand, wand_letter);

        let scroll = entities.spawn_item(
            ItemKind::ScrollReveal,
            EntityLocation::Inventory { owner: player_id },
        );
        let scroll_letter = inventory
            .add_existing_with_next_letter(scroll)
            .expect("Phase 7 시작 inventory는 letter 용량을 초과하지 않는다");
        entities.set_item_letter(scroll, scroll_letter);

        let rock = entities.spawn_item(
            ItemKind::Rock,
            EntityLocation::Inventory { owner: player_id },
        );
        let rock_letter = inventory
            .add_existing_with_next_letter(rock)
            .expect("Phase 7 시작 inventory는 letter 용량을 초과하지 않는다");
        entities.set_item_letter(rock, rock_letter);

        entities.spawn_item(
            ItemKind::ArmorLeather,
            EntityLocation::OnMap {
                level: PHASE5_LEVEL1_ID,
                pos: Pos { x: 7, y: 5 },
            },
        );
        entities.spawn_item(
            ItemKind::ScrollIdentify,
            EntityLocation::OnMap {
                level: PHASE5_LEVEL1_ID,
                pos: Pos { x: 9, y: 5 },
            },
        );
        entities.spawn_item(
            ItemKind::ScrollLevelTeleport,
            EntityLocation::OnMap {
                level: PHASE5_LEVEL1_ID,
                pos: Pos { x: 11, y: 5 },
            },
        );

        Self {
            levels: LevelRegistry::fixture_phase5(),
            current_level: PHASE5_LEVEL1_ID,
            entities,
            player_id,
            inventory,
            nutrition: 900,
            luck: 0,
            prayer_cooldown: 0,
            paralysis_turns: 0,
            hallucinating: false,
            kill_count: 0,
            gold: 0,
            identified_items: Vec::new(),
            last_death_cause: None,
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

    pub fn current_level_hostile_monsters(&self) -> Vec<EntityId> {
        self.entities.hostile_monsters_on_level(self.current_level)
    }

    pub fn identify_item_kind(&mut self, kind: ItemKind) {
        if !self.identified_items.contains(&kind) {
            self.identified_items.push(kind);
            self.identified_items.sort_by_key(|kind| *kind as u8);
        }
    }

    pub fn is_item_identified(&self, kind: ItemKind) -> bool {
        self.identified_items.contains(&kind)
    }

    pub fn carried_weight(&self) -> i16 {
        self.inventory
            .entries
            .iter()
            .filter_map(|entry| self.entities.item_data(entry.item).map(|data| data.weight))
            .sum()
    }

    /// [v0.2.0] Phase 20: 산재한 상태 필드를 Status struct로 묶어 반환한다.
    /// GameWorld 내부 필드 구조는 유지되며, Status는 getter 방식으로 제공된다.
    pub fn status(&self) -> Status {
        Status {
            nutrition: self.nutrition,
            luck: self.luck,
            prayer_cooldown: self.prayer_cooldown,
            paralysis_turns: self.paralysis_turns,
            hallucinating: self.hallucinating,
        }
    }

    /// [v0.2.0] Phase 20: Status를 기반으로 산재한 필드를 일괄 갱신한다.
    /// 이 메서드는 save/load roundtrip 시에만 사용한다.
    pub fn set_status(&mut self, status: Status) {
        self.nutrition = status.nutrition;
        self.luck = status.luck;
        self.prayer_cooldown = status.prayer_cooldown;
        self.paralysis_turns = status.paralysis_turns;
        self.hallucinating = status.hallucinating;
    }

    /// [v0.2.0] Phase 20: 현재 허기 상태를 반환한다.
    pub fn hunger_state(&self) -> HungerState {
        self.status().hunger_state()
    }

    pub fn from_saved_world(saved: SavedWorldV1) -> Self {
        Self {
            levels: saved.levels,
            current_level: saved.current_level,
            entities: saved.entities,
            player_id: saved.player_id,
            inventory: saved.inventory,
            nutrition: saved.nutrition,
            luck: saved.luck,
            prayer_cooldown: saved.prayer_cooldown,
            paralysis_turns: saved.paralysis_turns,
            hallucinating: saved.hallucinating,
            kill_count: saved.kill_count,
            gold: saved.gold,
            identified_items: saved.identified_items,
            last_death_cause: None,
        }
    }
}

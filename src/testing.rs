//! Integration test fixture support.
//!
//! Runtime code must mutate a session through `GameSession::submit`. Tests that
//! need deliberately invalid or unusual preconditions rebuild the session from
//! its persisted fixture representation instead of borrowing the live world.

use crate::{
    core::{ids::LevelId, position::Pos, save::SavedWorldV1, GameSession, RunState},
    domain::{item::ItemKind, map::GameMap, status::Status},
};

pub use aihack_runtime::testing::resolve_depleted_death;

type SavedWorldConfigurator = Box<dyn FnOnce(&mut SavedWorldV1)>;

pub struct SessionBuilder {
    seed: u64,
    run_state: RunState,
    configure: Option<SavedWorldConfigurator>,
}

/// 저장 스키마를 통해서만 테스트 초기 상태를 바꾸는 좁은 fixture 표면이다.
pub struct FixtureWorld<'a> {
    saved: &'a mut SavedWorldV1,
}

impl FixtureWorld<'_> {
    pub fn set_player_pos(&mut self, pos: Pos) {
        let level = self.saved.current_level;
        self.set_player_location(level, pos);
    }

    pub fn set_player_location(&mut self, level: LevelId, pos: Pos) {
        assert!(
            self.saved
                .entities
                .set_actor_location(self.saved.player_id, level, pos),
            "fixture world must retain the player actor"
        );
        self.saved.current_level = level;
    }

    pub fn current_map_mut(&mut self) -> &mut GameMap {
        self.saved
            .levels
            .map_mut(self.saved.current_level)
            .expect("fixture world must retain the current level map")
    }

    pub fn set_status(&mut self, status: Status) {
        self.saved.nutrition = status.nutrition;
        self.saved.luck = status.luck;
        self.saved.prayer_cooldown = status.prayer_cooldown;
        self.saved.paralysis_turns = status.paralysis_turns;
        self.saved.hallucinating = status.hallucinating;
    }

    pub fn set_gold(&mut self, gold: u32) {
        self.saved.gold = gold;
    }

    pub fn set_kill_count(&mut self, kill_count: u32) {
        self.saved.kill_count = kill_count;
    }

    pub fn identify_item_kind(&mut self, kind: ItemKind) {
        if !self.saved.identified_items.contains(&kind) {
            self.saved.identified_items.push(kind);
            self.saved.identified_items.sort_by_key(|kind| *kind as u8);
        }
    }

    pub fn saved(&mut self) -> &mut SavedWorldV1 {
        self.saved
    }
}

impl SessionBuilder {
    pub fn playing(seed: u64) -> Self {
        Self {
            seed,
            run_state: RunState::Playing,
            configure: None,
        }
    }

    pub fn run_state(mut self, run_state: RunState) -> Self {
        self.run_state = run_state;
        self
    }

    pub fn configure_saved_world(
        mut self,
        configure: impl FnOnce(&mut SavedWorldV1) + 'static,
    ) -> Self {
        self.configure = Some(Box::new(configure));
        self
    }

    pub fn build(self) -> GameSession {
        let session = GameSession::new_for_playing(self.seed);
        let mut save = session.to_save_data();
        save.run_state = self.run_state;
        if let Some(configure) = self.configure {
            configure(&mut save.world);
        }
        GameSession::from_save_data(save).expect("test fixture save data must be valid")
    }

    pub fn mutate<T>(
        session: &mut GameSession,
        configure: impl FnOnce(&mut FixtureWorld<'_>) -> T,
    ) -> T {
        let mut save = session.to_save_data();
        let result = configure(&mut FixtureWorld {
            saved: &mut save.world,
        });
        *session = GameSession::from_save_data(save).expect("test fixture save data must be valid");
        result
    }
}

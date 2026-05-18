use serde::{Deserialize, Serialize};

use crate::{
    core::{
        action::CommandIntent, event::GameEvent, ids::EntityId, observation::Observation,
        position::Direction, rng::GameRng, snapshot::GameSnapshot, turn::TurnOutcome,
        world::GameWorld,
    },
    systems::{combat, death, doors, items, monster_ai, movement, projectiles, stairs, traps},
};

/// [v0.1.0] 실행 재현성에 필요한 세션 메타데이터다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMeta {
    pub seed: u64,
}

/// [v0.2.0] Phase 16: spec.md 8.2 계약과 일치시킨다.
/// Title, CharacterCreation, AwaitingDirection, AwaitingInventorySelection, MorePrompt를 추가하고
/// GameOver에 cause와 final_score 필드를 추가한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunState {
    Title,
    CharacterCreation,
    Playing,
    AwaitingDirection {
        action: crate::core::action::DirectionalAction,
    },
    AwaitingInventorySelection {
        action: crate::core::action::InventoryAction,
    },
    MorePrompt,
    GameOver {
        cause: crate::domain::combat::DeathCause,
        final_score: i32,
    },
}

/// [v0.1.0] 새 런타임의 단일 상태 원천이다.
/// Phase 4에서는 map + actor/item store + inventory 상태를 소유한다.
#[derive(Debug, Clone)]
pub struct GameSession {
    pub meta: GameMeta,
    pub rng: GameRng,
    pub turn: u64,
    pub state: RunState,
    pub world: GameWorld,
    pub event_log: Vec<GameEvent>,
}

impl GameSession {
    pub fn new(seed: u64) -> Self {
        Self {
            meta: GameMeta { seed },
            rng: GameRng::new(seed),
            turn: 0,
            state: RunState::Title,
            world: GameWorld::fixture_phase4(),
            event_log: Vec::new(),
        }
    }

    /// [v0.2.0] Phase 16: Title -> CharacterCreation -> Playing로 즉시 전환하여
    /// 기존 테스트와 headless runner의 호환성을 유지한다.
    pub fn new_for_playing(seed: u64) -> Self {
        let mut session = Self::new(seed);
        session.state = RunState::Playing;
        session
    }

    /// [v0.2.0] Phase 16: RunState에 따라 명령 처리를 분기한다.
    pub fn submit(&mut self, intent: CommandIntent) -> TurnOutcome {
        match self.state {
            RunState::Title => self.submit_in_title(intent),
            RunState::CharacterCreation => self.submit_in_character_creation(intent),
            RunState::Playing => self.submit_in_playing(intent),
            RunState::AwaitingDirection { action } => {
                self.submit_in_awaiting_direction(action, intent)
            }
            RunState::AwaitingInventorySelection { action } => {
                self.submit_in_awaiting_inventory(action, intent)
            }
            RunState::MorePrompt => self.submit_in_more_prompt(intent),
            RunState::GameOver { .. } => self.submit_in_game_over(intent),
        }
    }

    fn submit_in_title(&mut self, intent: CommandIntent) -> TurnOutcome {
        match intent {
            CommandIntent::Wait => {
                self.state = RunState::CharacterCreation;
                self.accept_without_turn(vec![crate::core::event::GameEvent::Message {
                    priority: crate::core::event::MessagePriority::Info,
                    text: "Welcome to AIHack".to_string(),
                }])
            }
            CommandIntent::Quit => self.submit_quit(),
            _ => self.reject("press Enter to start or Q to quit".to_string()),
        }
    }

    fn submit_in_character_creation(&mut self, intent: CommandIntent) -> TurnOutcome {
        match intent {
            CommandIntent::Wait => {
                self.state = RunState::Playing;
                self.accept_without_turn(vec![crate::core::event::GameEvent::Message {
                    priority: crate::core::event::MessagePriority::Info,
                    text: "Character created. Good luck!".to_string(),
                }])
            }
            CommandIntent::Quit => self.submit_quit(),
            _ => self.reject("press Enter to confirm or Esc to go back".to_string()),
        }
    }

    fn submit_in_playing(&mut self, intent: CommandIntent) -> TurnOutcome {
        if self.world.paralysis_turns > 0
            && !matches!(intent, CommandIntent::Wait | CommandIntent::Quit)
        {
            return self.reject("player is paralyzed".to_string());
        }
        match intent {
            CommandIntent::Wait => self.submit_wait(),
            CommandIntent::Quit => self.submit_quit(),
            CommandIntent::Move(direction) => self.submit_move(direction),
            CommandIntent::Search => self.submit_search(),
            CommandIntent::Kick(direction) => self.submit_kick(direction),
            CommandIntent::Open(direction) => self.submit_open(direction),
            CommandIntent::Close(direction) => self.submit_close(direction),
            CommandIntent::Pickup => self.submit_pickup(),
            CommandIntent::Drop { item } => self.submit_drop(item),
            CommandIntent::Throw { item, direction } => self.submit_throw(item, direction),
            CommandIntent::ShowInventory => self.accept_without_turn(Vec::new()),
            CommandIntent::Wield { item } => self.submit_wield(item),
            CommandIntent::Wear { item } => self.submit_wear(item),
            CommandIntent::Quaff { item } => self.submit_quaff(item),
            CommandIntent::Zap { item, direction } => self.submit_zap(item, direction),
            CommandIntent::Read { item } => self.submit_read(item),
            CommandIntent::Pray => self.submit_pray(),
            CommandIntent::Descend => self.submit_descend(),
            CommandIntent::Ascend => self.submit_ascend(),
            CommandIntent::AcknowledgeMore => {
                self.reject("no more prompt to acknowledge".to_string())
            }
        }
    }

    fn submit_in_awaiting_direction(
        &mut self,
        action: crate::core::action::DirectionalAction,
        intent: CommandIntent,
    ) -> TurnOutcome {
        self.state = RunState::Playing;
        match intent {
            CommandIntent::Move(direction) => match action {
                crate::core::action::DirectionalAction::Open => self.submit_open(direction),
                crate::core::action::DirectionalAction::Close => self.submit_close(direction),
                crate::core::action::DirectionalAction::Kick => self.submit_kick(direction),
            },
            CommandIntent::Quit => self.submit_quit(),
            _ => {
                self.state = RunState::AwaitingDirection { action };
                self.reject("choose a direction or Esc to cancel".to_string())
            }
        }
    }

    fn submit_in_awaiting_inventory(
        &mut self,
        action: crate::core::action::InventoryAction,
        intent: CommandIntent,
    ) -> TurnOutcome {
        self.state = RunState::Playing;
        match intent {
            CommandIntent::Quit => self.submit_quit(),
            _ => {
                self.state = RunState::AwaitingInventorySelection { action };
                self.reject("choose an item or Esc to cancel".to_string())
            }
        }
    }

    fn submit_in_more_prompt(&mut self, intent: CommandIntent) -> TurnOutcome {
        match intent {
            CommandIntent::AcknowledgeMore => {
                self.state = RunState::Playing;
                self.accept_without_turn(Vec::new())
            }
            _ => self.reject("press any key to continue".to_string()),
        }
    }

    fn submit_in_game_over(&mut self, intent: CommandIntent) -> TurnOutcome {
        match intent {
            CommandIntent::Quit => self.submit_quit(),
            _ => self.reject("run is already game over".to_string()),
        }
    }

    pub fn snapshot(&self) -> GameSnapshot {
        GameSnapshot::from_world(
            self.meta.seed,
            self.turn,
            self.state,
            &self.event_log,
            &self.world,
        )
    }

    pub fn observation(&self) -> Observation {
        Observation::from_world(
            self.meta.seed,
            self.turn,
            self.state,
            &self.event_log,
            &self.world,
        )
    }

    fn submit_wait(&mut self) -> TurnOutcome {
        let next_turn = self.turn + 1;
        self.accept_turn(vec![GameEvent::Waited { turn: next_turn }])
    }

    fn submit_move(&mut self, direction: Direction) -> TurnOutcome {
        let from = self.world.player_pos();
        let to = from.offset(direction.delta());
        if let Some(defender) = self
            .world
            .entities
            .alive_hostile_at(self.world.current_level(), to)
        {
            if movement::is_bump_attack_for_legal_action(&self.world, direction) {
                return self.submit_bump_attack(defender);
            }
        }
        match movement::move_player(&mut self.world, direction) {
            Ok(()) => {
                let to = self.world.player_pos();
                let mut events = vec![GameEvent::EntityMoved {
                    entity: self.world.player_id,
                    from,
                    to,
                }];
                events.extend(traps::trigger_player_trap(&mut self.world));
                let player_id = self.world.player_id;
                events.extend(death::collect_death_events_if_hp_depleted(
                    &mut self.world,
                    player_id,
                    crate::domain::combat::DeathCause::Trap {
                        trap: crate::domain::tile::TrapKind::Pit,
                    },
                ));
                self.state = death::state_after_deaths(&self.world);
                self.accept_turn(events)
            }
            Err(error) => self.reject(format!("{error}")),
        }
    }

    fn submit_bump_attack(&mut self, defender: EntityId) -> TurnOutcome {
        let attacker = self.world.player_id;
        let Some(resolution) =
            combat::resolve_attack(&mut self.world, &mut self.rng, attacker, defender)
        else {
            return self.reject("bump attack target is not attackable".to_string());
        };
        let mut events = vec![combat::attack_event(&resolution)];
        if matches!(
            self.world
                .entities
                .get(defender)
                .map(|entity| entity.kind()),
            Some(crate::domain::entity::EntityKind::Monster(
                crate::domain::monster::MonsterKind::FloatingEye
            ))
        ) {
            self.world.paralysis_turns = self.world.paralysis_turns.max(2);
            events.push(GameEvent::PassiveAttackTriggered {
                source: defender,
                target: attacker,
            });
        }
        events.extend(death::collect_death_events_after_attack(
            &mut self.world,
            attacker,
            defender,
        ));
        self.state = death::state_after_deaths(&self.world);
        self.accept_turn(events)
    }

    fn submit_pickup(&mut self) -> TurnOutcome {
        match items::pickup(&mut self.world) {
            Ok(event) => self.accept_turn(vec![event]),
            Err(error) => self.reject(error),
        }
    }

    fn submit_search(&mut self) -> TurnOutcome {
        let events = traps::search(&mut self.world);
        self.accept_turn(events)
    }

    fn submit_throw(&mut self, item: EntityId, direction: Direction) -> TurnOutcome {
        match projectiles::throw_item(&mut self.world, &mut self.rng, item, direction) {
            Ok(events) => {
                self.state = death::state_after_deaths(&self.world);
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_drop(&mut self, item: EntityId) -> TurnOutcome {
        match items::drop(&mut self.world, item) {
            Ok(event) => self.accept_turn(vec![event]),
            Err(error) => self.reject(error),
        }
    }

    fn submit_wield(&mut self, item: EntityId) -> TurnOutcome {
        match items::wield(&mut self.world, item) {
            Ok(Some(event)) => self.accept_turn(vec![event]),
            Ok(None) => self.accept_without_turn(Vec::new()),
            Err(error) => self.reject(error),
        }
    }

    fn submit_quaff(&mut self, item: EntityId) -> TurnOutcome {
        match items::quaff(&mut self.world, &mut self.rng, item) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_wear(&mut self, item: EntityId) -> TurnOutcome {
        match items::wear(&mut self.world, item) {
            Ok(Some(event)) => self.accept_turn(vec![event]),
            Ok(None) => self.accept_without_turn(Vec::new()),
            Err(error) => self.reject(error),
        }
    }

    fn submit_zap(&mut self, item: EntityId, direction: Direction) -> TurnOutcome {
        match projectiles::zap_wand(&mut self.world, &mut self.rng, item, direction) {
            Ok(events) => {
                self.state = death::state_after_deaths(&self.world);
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_read(&mut self, item: EntityId) -> TurnOutcome {
        match items::read(&mut self.world, item) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_kick(&mut self, direction: Direction) -> TurnOutcome {
        match doors::kick_door(&mut self.world, direction) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(format!("{error}")),
        }
    }

    fn submit_pray(&mut self) -> TurnOutcome {
        if self.world.prayer_cooldown > 0 {
            return self.reject("prayer is on cooldown".to_string());
        }
        self.world.prayer_cooldown = 20;
        self.accept_turn(vec![GameEvent::PrayerOffered {
            entity: self.world.player_id,
            cooldown_after: self.world.prayer_cooldown,
        }])
    }

    fn submit_open(&mut self, direction: Direction) -> TurnOutcome {
        let pos = self.world.player_pos().offset(direction.delta());
        match doors::open_door(&mut self.world, direction) {
            Ok((from, to)) => self.accept_turn(vec![GameEvent::DoorChanged { pos, from, to }]),
            Err(error) => self.reject(format!("{error}")),
        }
    }

    fn submit_close(&mut self, direction: Direction) -> TurnOutcome {
        let pos = self.world.player_pos().offset(direction.delta());
        match doors::close_door(&mut self.world, direction) {
            Ok((from, to)) => self.accept_turn(vec![GameEvent::DoorChanged { pos, from, to }]),
            Err(error) => self.reject(format!("{error}")),
        }
    }

    fn submit_descend(&mut self) -> TurnOutcome {
        match stairs::descend(&mut self.world) {
            Ok(event) => self.accept_turn(vec![event]),
            Err(error) => self.reject(error),
        }
    }

    fn submit_ascend(&mut self) -> TurnOutcome {
        match stairs::ascend(&mut self.world) {
            Ok(event) => self.accept_turn(vec![event]),
            Err(error) => self.reject(error),
        }
    }

    fn submit_quit(&mut self) -> TurnOutcome {
        // [v0.2.0] Phase 16: Quit는 모든 상태에서 종료 요청으로 처리한다.
        // 이미 GameOver 상태가 아니면 GameOver로 전환한다.
        if !matches!(self.state, RunState::GameOver { .. }) {
            self.state = RunState::GameOver {
                cause: crate::domain::combat::DeathCause::Combat {
                    attacker: crate::core::ids::EntityId(0),
                },
                final_score: 0,
            };
        }
        self.accept_without_turn(vec![GameEvent::CommandRejected {
            reason: "quit requested".to_string(),
        }])
    }

    fn accept_turn(&mut self, mut events: Vec<GameEvent>) -> TurnOutcome {
        let next_turn = self.turn + 1;
        events.insert(0, GameEvent::TurnStarted { turn: next_turn });
        self.turn = next_turn;
        self.world.nutrition = self.world.nutrition.saturating_sub(1);
        self.world.prayer_cooldown = self.world.prayer_cooldown.saturating_sub(1);
        if self.world.paralysis_turns > 0 {
            self.world.paralysis_turns -= 1;
        }
        if !matches!(self.state, RunState::GameOver { .. }) {
            events.extend(monster_ai::run_monster_turn(
                &mut self.world,
                &mut self.rng,
                &mut self.state,
            ));
        }
        self.event_log.extend(events.clone());

        TurnOutcome {
            accepted: true,
            turn_advanced: true,
            events,
            snapshot_hash: self.snapshot().stable_hash(),
            next_state: self.state,
        }
    }

    fn accept_without_turn(&mut self, events: Vec<GameEvent>) -> TurnOutcome {
        self.event_log.extend(events.clone());
        TurnOutcome {
            accepted: true,
            turn_advanced: false,
            events,
            snapshot_hash: self.snapshot().stable_hash(),
            next_state: self.state,
        }
    }

    fn reject(&self, reason: String) -> TurnOutcome {
        TurnOutcome {
            accepted: false,
            turn_advanced: false,
            events: vec![GameEvent::CommandRejected { reason }],
            snapshot_hash: self.snapshot().stable_hash(),
            next_state: self.state,
        }
    }
}

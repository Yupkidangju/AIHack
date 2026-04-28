use serde::{Deserialize, Serialize};

use crate::{
    core::{
        action::CommandIntent, event::GameEvent, ids::EntityId, observation::Observation,
        position::Direction, rng::GameRng, snapshot::GameSnapshot, turn::TurnOutcome,
        world::GameWorld,
    },
    systems::{combat, death, doors, items, movement, stairs},
};

/// [v0.1.0] 실행 재현성에 필요한 세션 메타데이터다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMeta {
    pub seed: u64,
}

/// [v0.1.0] Phase 4는 전투 사망 시 GameOver로 전환한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunState {
    Playing,
    GameOver,
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
            state: RunState::Playing,
            world: GameWorld::fixture_phase4(),
            event_log: Vec::new(),
        }
    }

    pub fn submit(&mut self, intent: CommandIntent) -> TurnOutcome {
        if self.state == RunState::GameOver && !matches!(intent, CommandIntent::Quit) {
            return self.reject("run is already game over".to_string());
        }
        match intent {
            CommandIntent::Wait => self.submit_wait(),
            CommandIntent::Quit => self.submit_quit(),
            CommandIntent::Move(direction) => self.submit_move(direction),
            CommandIntent::Open(direction) => self.submit_open(direction),
            CommandIntent::Close(direction) => self.submit_close(direction),
            CommandIntent::Pickup => self.submit_pickup(),
            CommandIntent::ShowInventory => self.accept_without_turn(Vec::new()),
            CommandIntent::Wield { item } => self.submit_wield(item),
            CommandIntent::Quaff { item } => self.submit_quaff(item),
            CommandIntent::Descend => self.submit_descend(),
            CommandIntent::Ascend => self.submit_ascend(),
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
        Observation::from_world(self.meta.seed, self.turn, &self.world)
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
                self.accept_turn(vec![GameEvent::EntityMoved { from, to }])
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
        self.state = RunState::GameOver;
        self.accept_without_turn(vec![GameEvent::CommandRejected {
            reason: "quit requested in Phase 4 headless core".to_string(),
        }])
    }

    fn accept_turn(&mut self, mut events: Vec<GameEvent>) -> TurnOutcome {
        let next_turn = self.turn + 1;
        events.insert(0, GameEvent::TurnStarted { turn: next_turn });
        self.turn = next_turn;
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

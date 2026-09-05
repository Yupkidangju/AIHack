use std::ops::Deref;

use aihack_ai_contract::{ClientRevision, Observation};
use aihack_content::ContentRegistry;
use aihack_core::{
    action::{CommandIntent, DirectionalAction, InventoryAction},
    domain::{combat::DeathCause, monster::MonsterPassive, tile::TrapKind},
    error::ContentError,
    event::{GameEvent, MessagePriority},
    ids::EntityId,
    position::Direction,
    rng::GameRng,
    session::SessionState,
    turn::TurnOutcome,
};

pub use aihack_core::{meta::GameMeta, run_state::RunState};

use crate::{
    observation,
    snapshot::GameSnapshot,
    systems::{
        combat, death, doors, items, monster_ai, movement, projectiles, score, stairs, traps,
    },
    world::GameWorld,
};

/// 결정론적 runtime의 단일 session 상태 원천이다.
#[derive(Debug, Clone)]
pub struct GameSession {
    pub(crate) inner: SessionState<GameWorld>,
    pub(crate) transaction_aborted: bool,
}

impl Deref for GameSession {
    type Target = SessionState<GameWorld>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl GameSession {
    /// Content bootstrap 실패를 호출자에게 돌려주는 production 생성 경계다.
    pub fn try_new(seed: u64) -> Result<Self, ContentError> {
        Self::try_new_with_registry(seed, aihack_content::registry()?)
    }

    pub fn try_new_with_registry(
        seed: u64,
        registry: &ContentRegistry,
    ) -> Result<Self, ContentError> {
        let session = Self {
            inner: SessionState {
                meta: GameMeta { seed },
                rng: GameRng::new(seed),
                turn: 0,
                state: RunState::Title,
                world: GameWorld::try_fixture_phase5_with_registry(registry)?,
                event_log: Vec::new(),
            },
            transaction_aborted: false,
        };
        crate::save::validate_save_data_with_registry(&session.to_save_data(), registry).map_err(
            |error| ContentError::Parse {
                file: "world bootstrap".to_owned(),
                message: error.to_string(),
            },
        )?;
        Ok(session)
    }

    pub fn try_new_for_playing(seed: u64) -> Result<Self, ContentError> {
        Self::try_new_for_playing_with_registry(seed, aihack_content::registry()?)
    }

    pub fn try_new_for_playing_with_registry(
        seed: u64,
        registry: &ContentRegistry,
    ) -> Result<Self, ContentError> {
        let mut session = Self::try_new_with_registry(seed, registry)?;
        session.inner.state = RunState::Playing;
        Ok(session)
    }

    pub fn new(seed: u64) -> Self {
        Self {
            inner: SessionState {
                meta: GameMeta { seed },
                rng: GameRng::new(seed),
                turn: 0,
                state: RunState::Title,
                world: GameWorld::fixture_phase4(),
                event_log: Vec::new(),
            },
            transaction_aborted: false,
        }
    }

    /// Title/CharacterCreation을 건너뛰고 Playing fixture를 만든다.
    pub fn new_for_playing(seed: u64) -> Self {
        let mut session = Self::new(seed);
        session.inner.state = RunState::Playing;
        session
    }

    /// 재현과 저장 식별에 사용하는 초기 난수 시드다.
    pub fn seed(&self) -> u64 {
        self.meta.seed
    }

    /// 현재 승인된 턴 수다.
    pub fn turn(&self) -> u64 {
        self.turn
    }

    /// 현재 실행 상태를 값으로 반환한다.
    pub fn run_state(&self) -> RunState {
        self.state
    }

    /// 최근 게임 이벤트를 읽기 전용으로 제공한다.
    pub fn event_log(&self) -> &[GameEvent] {
        &self.event_log
    }

    /// 현재 월드의 읽기 전용 조회 경계다.
    pub fn world(&self) -> &GameWorld {
        &self.world
    }

    /// working copy에서 명령을 적용하고 invariant가 유효할 때만 전체 상태를 교체한다.
    pub fn submit(&mut self, intent: CommandIntent) -> TurnOutcome {
        let mut transaction = crate::transaction::TurnTransaction::prepare(self);
        let outcome = transaction.apply(intent);
        if transaction.was_aborted() {
            let reason = outcome
                .events
                .iter()
                .find_map(|event| match event {
                    GameEvent::CommandRejected { reason } => Some(reason.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "command rejected".to_string());
            return self.reject(reason);
        }
        let report = transaction.validate();
        if !report.is_valid() {
            return self.reject(crate::transaction::TurnTransaction::invariant_reason(
                &report,
            ));
        }

        *self = transaction.commit();
        outcome
    }

    /// transaction working copy 내부에서만 상태 전이를 수행한다.
    pub(crate) fn submit_uncommitted(&mut self, intent: CommandIntent) -> TurnOutcome {
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
            RunState::GameOver { .. } | RunState::Victory { .. } => {
                self.submit_in_game_over(intent)
            }
        }
    }

    fn submit_in_title(&mut self, intent: CommandIntent) -> TurnOutcome {
        match intent {
            CommandIntent::Wait => {
                self.inner.state = RunState::CharacterCreation;
                self.accept_without_turn(vec![GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Welcome to AIHack".to_string(),
                }])
            }
            CommandIntent::Quit => self.submit_quit(),
            _ => self.reject("press Enter to start or Q to quit".to_string()),
        }
    }

    fn submit_in_character_creation(&mut self, intent: CommandIntent) -> TurnOutcome {
        match intent {
            CommandIntent::StartCampaign { role } => {
                if self.turn != 0 || self.world.campaign.is_some() {
                    return self.reject("campaign requires a fresh creation state".into());
                }
                let registry = self.world.content_registry().clone();
                let state = match crate::bootstrap::campaign_world(&registry, self.seed(), role) {
                    Ok(state) => state,
                    Err(error) => return self.abort_transaction(error.to_string()),
                };
                self.inner.world =
                    match GameWorld::from_saved_world_with_registry((&state).into(), &registry) {
                        Ok(world) => world,
                        Err(error) => return self.abort_transaction(error.to_string()),
                    };
                self.inner.state = RunState::Playing;
                self.accept_without_turn(vec![GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: format!(
                        "{role:?}: recover the Amulet on Main 6 and return to the surface."
                    ),
                }])
            }
            CommandIntent::Wait => {
                self.inner.state = RunState::Playing;
                self.accept_without_turn(vec![GameEvent::Message {
                    priority: MessagePriority::Info,
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
            CommandIntent::StartCampaign { .. } => {
                self.reject("role is chosen only during creation".into())
            }
            CommandIntent::EnterBranch => match stairs::enter_branch(&mut self.inner.world) {
                Ok(event) => self.accept_turn(vec![event]),
                Err(error) => self.reject(error),
            },
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
            CommandIntent::Eat { item } => self.submit_eat(item),
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
        action: DirectionalAction,
        intent: CommandIntent,
    ) -> TurnOutcome {
        self.inner.state = RunState::Playing;
        match intent {
            CommandIntent::Move(direction) => match action {
                DirectionalAction::Open => self.submit_open(direction),
                DirectionalAction::Close => self.submit_close(direction),
                DirectionalAction::Kick => self.submit_kick(direction),
            },
            CommandIntent::AcknowledgeMore => self.accept_without_turn(Vec::new()),
            CommandIntent::Quit => self.submit_quit(),
            _ => {
                self.inner.state = RunState::AwaitingDirection { action };
                self.reject("choose a direction or Esc to cancel".to_string())
            }
        }
    }

    fn submit_in_awaiting_inventory(
        &mut self,
        action: InventoryAction,
        intent: CommandIntent,
    ) -> TurnOutcome {
        self.inner.state = RunState::Playing;
        match intent {
            CommandIntent::Drop { item } if action == InventoryAction::Drop => {
                self.submit_drop(item)
            }
            CommandIntent::Wield { item } if action == InventoryAction::Wield => {
                self.submit_wield(item)
            }
            CommandIntent::Wear { item } if action == InventoryAction::Wear => {
                self.submit_wear(item)
            }
            CommandIntent::Quaff { item } if action == InventoryAction::Quaff => {
                self.submit_quaff(item)
            }
            CommandIntent::Eat { item } if action == InventoryAction::Eat => self.submit_eat(item),
            CommandIntent::Read { item } if action == InventoryAction::Read => {
                self.submit_read(item)
            }
            CommandIntent::AcknowledgeMore => self.accept_without_turn(Vec::new()),
            CommandIntent::Quit => self.submit_quit(),
            _ => {
                self.inner.state = RunState::AwaitingInventorySelection { action };
                self.reject("choose an item or Esc to cancel".to_string())
            }
        }
    }

    fn submit_in_more_prompt(&mut self, intent: CommandIntent) -> TurnOutcome {
        match intent {
            CommandIntent::AcknowledgeMore => {
                self.inner.state = RunState::Playing;
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
        observation::from_world(
            self.meta.seed,
            self.turn,
            self.state,
            &self.event_log,
            &self.world,
        )
    }

    fn submit_wait(&mut self) -> TurnOutcome {
        let next_turn = self.turn.saturating_add(1);
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
        match movement::move_player(&mut self.inner.world, direction) {
            Ok(()) => {
                let to = self.world.player_pos();
                let mut events = vec![GameEvent::EntityMoved {
                    entity: self.world.player_id(),
                    from,
                    to,
                }];
                events.extend(traps::trigger_player_trap(&mut self.inner.world));
                let player_id = self.world.player_id();
                let death_events = match death::collect_death_events_if_hp_depleted(
                    &mut self.inner.world,
                    player_id,
                    DeathCause::Trap {
                        trap: TrapKind::Pit,
                    },
                ) {
                    Ok(events) => events,
                    Err(error) => return self.abort_transaction(error),
                };
                events.extend(death_events);
                self.inner.state =
                    death::state_after_deaths_at(&self.world, self.turn.saturating_add(1));
                self.accept_turn(events)
            }
            Err(error) => self.reject(format!("{error}")),
        }
    }

    fn submit_bump_attack(&mut self, defender: EntityId) -> TurnOutcome {
        let attacker = self.world.player_id();
        let state = &mut self.inner;
        let Some(resolution) =
            combat::resolve_attack(&mut state.world, &mut state.rng, attacker, defender)
        else {
            return self.reject("bump attack target is not attackable".to_string());
        };
        let mut events = vec![combat::attack_event(&resolution)];
        if matches!(
            self.world
                .entities
                .get(defender)
                .and_then(|entity| entity.monster_passive()),
            Some(MonsterPassive::ParalyzeOnMelee)
        ) {
            self.inner.world.state_mut().paralysis_turns = self.world.paralysis_turns.max(2);
            events.push(GameEvent::PassiveAttackTriggered {
                source: defender,
                target: attacker,
            });
        }
        let death_events = match death::collect_death_events_after_attack(
            &mut self.inner.world,
            attacker,
            defender,
        ) {
            Ok(events) => events,
            Err(error) => return self.abort_transaction(error),
        };
        events.extend(death_events);
        self.inner.state = death::state_after_deaths_at(&self.world, self.turn.saturating_add(1));
        self.accept_turn(events)
    }

    fn submit_pickup(&mut self) -> TurnOutcome {
        match items::pickup(&mut self.inner.world) {
            Ok(event) => self.accept_turn(vec![event]),
            Err(error) => self.reject(error),
        }
    }

    fn submit_search(&mut self) -> TurnOutcome {
        let events = traps::search(&mut self.inner.world);
        self.accept_turn(events)
    }

    fn submit_throw(&mut self, item: EntityId, direction: Direction) -> TurnOutcome {
        let state = &mut self.inner;
        match projectiles::throw_item(&mut state.world, &mut state.rng, item, direction) {
            Ok(events) => {
                self.inner.state =
                    death::state_after_deaths_at(&self.world, self.turn.saturating_add(1));
                self.accept_turn(events)
            }
            Err(error) => self.abort_transaction(error),
        }
    }

    fn submit_drop(&mut self, item: EntityId) -> TurnOutcome {
        match items::drop(&mut self.inner.world, item) {
            Ok(event) => self.accept_turn(vec![event]),
            Err(error) => self.reject(error),
        }
    }

    fn submit_wield(&mut self, item: EntityId) -> TurnOutcome {
        match items::wield(&mut self.inner.world, item) {
            Ok(Some(event)) => self.accept_turn(vec![event]),
            Ok(None) => self.accept_without_turn(Vec::new()),
            Err(error) => self.reject(error),
        }
    }

    fn submit_quaff(&mut self, item: EntityId) -> TurnOutcome {
        let state = &mut self.inner;
        match items::quaff(&mut state.world, &mut state.rng, item) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_eat(&mut self, item: EntityId) -> TurnOutcome {
        match items::eat(&mut self.inner.world, item) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_wear(&mut self, item: EntityId) -> TurnOutcome {
        match items::wear(&mut self.inner.world, item) {
            Ok(Some(event)) => self.accept_turn(vec![event]),
            Ok(None) => self.accept_without_turn(Vec::new()),
            Err(error) => self.reject(error),
        }
    }

    fn submit_zap(&mut self, item: EntityId, direction: Direction) -> TurnOutcome {
        let state = &mut self.inner;
        match projectiles::zap_wand(&mut state.world, &mut state.rng, item, direction) {
            Ok(events) => {
                self.inner.state =
                    death::state_after_deaths_at(&self.world, self.turn.saturating_add(1));
                self.accept_turn(events)
            }
            Err(error) => self.abort_transaction(error),
        }
    }

    fn submit_read(&mut self, item: EntityId) -> TurnOutcome {
        match items::read(&mut self.inner.world, item) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_kick(&mut self, direction: Direction) -> TurnOutcome {
        match doors::kick_door(&mut self.inner.world, direction) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(format!("{error}")),
        }
    }

    fn submit_pray(&mut self) -> TurnOutcome {
        if self.world.prayer_cooldown > 0 {
            return self.reject("prayer is on cooldown".to_string());
        }
        self.inner.world.state_mut().prayer_cooldown = 20;
        self.inner.world.state_mut().luck = self.world.luck.saturating_add(1).min(3);
        self.accept_turn(vec![GameEvent::PrayerOffered {
            entity: self.world.player_id(),
            cooldown_after: self.world.prayer_cooldown,
        }])
    }

    fn submit_open(&mut self, direction: Direction) -> TurnOutcome {
        let pos = self.world.player_pos().offset(direction.delta());
        match doors::open_door(&mut self.inner.world, direction) {
            Ok((from, to)) => self.accept_turn(vec![GameEvent::DoorChanged { pos, from, to }]),
            Err(error) => self.reject(format!("{error}")),
        }
    }

    fn submit_close(&mut self, direction: Direction) -> TurnOutcome {
        let pos = self.world.player_pos().offset(direction.delta());
        match doors::close_door(&mut self.inner.world, direction) {
            Ok((from, to)) => self.accept_turn(vec![GameEvent::DoorChanged { pos, from, to }]),
            Err(error) => self.reject(format!("{error}")),
        }
    }

    fn submit_descend(&mut self) -> TurnOutcome {
        match stairs::descend(&mut self.inner.world) {
            Ok(event) => self.accept_turn(vec![event]),
            Err(error) => self.reject(error),
        }
    }

    fn submit_ascend(&mut self) -> TurnOutcome {
        if self.world.campaign.is_some()
            && self.world.current_level() == aihack_core::ids::LevelId::main(1)
            && self.world.current_map().tile(self.world.player_pos()).ok()
                == Some(aihack_core::domain::tile::TileKind::StairsUp)
        {
            if !crate::campaign::has_amulet(&self.world) || !self.world.player_alive() {
                return self.reject("return with the Amulet of Ascension".into());
            }
            let Some(final_score) =
                score::death_score(&self.world, self.turn + 1).checked_add(10_000)
            else {
                return self.abort_transaction("victory score overflows".into());
            };
            self.inner.state = RunState::Victory { final_score };
            return self.accept_turn(vec![GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Ascended! The Amulet has reached the surface.".into(),
            }]);
        }
        match stairs::ascend(&mut self.inner.world) {
            Ok(event) => self.accept_turn(vec![event]),
            Err(error) => self.reject(error),
        }
    }

    fn submit_quit(&mut self) -> TurnOutcome {
        // Quit는 모든 상태에서 종료 요청이며, 기존 GameOver 원인은 보존한다.
        if !matches!(
            self.state,
            RunState::GameOver { .. } | RunState::Victory { .. }
        ) {
            self.inner.state = RunState::GameOver {
                cause: DeathCause::Combat {
                    attacker: EntityId(0),
                },
                final_score: score::death_score(&self.world, self.turn),
            };
        }
        self.accept_without_turn(vec![GameEvent::CommandRejected {
            reason: "quit requested".to_string(),
        }])
    }

    fn accept_turn(&mut self, mut events: Vec<GameEvent>) -> TurnOutcome {
        let next_turn = self.turn.saturating_add(1);
        events.insert(0, GameEvent::TurnStarted { turn: next_turn });
        self.inner.turn = next_turn;
        self.inner.world.state_mut().nutrition = self.world.nutrition.saturating_sub(1);
        self.inner.world.state_mut().prayer_cooldown = self.world.prayer_cooldown.saturating_sub(1);
        if self.world.paralysis_turns > 0 {
            self.inner.world.state_mut().paralysis_turns -= 1;
        }
        if !matches!(
            self.state,
            RunState::GameOver { .. } | RunState::Victory { .. }
        ) {
            let state = &mut self.inner;
            let monster_events = match monster_ai::run_monster_turn(
                &mut state.world,
                &mut state.rng,
                &mut state.state,
                next_turn,
            ) {
                Ok(events) => events,
                Err(error) => return self.abort_transaction(error),
            };
            events.extend(monster_events);
        }
        self.inner.event_log.extend(events.clone());

        TurnOutcome {
            accepted: true,
            turn_advanced: true,
            events,
            snapshot_hash: self.snapshot().stable_hash(),
            next_state: self.state,
        }
    }

    fn accept_without_turn(&mut self, events: Vec<GameEvent>) -> TurnOutcome {
        self.inner.event_log.extend(events.clone());
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

    fn abort_transaction(&mut self, reason: String) -> TurnOutcome {
        self.transaction_aborted = true;
        self.reject(reason)
    }
}

impl crate::client::GameClient for GameSession {
    fn observation(&self) -> Observation {
        GameSession::observation(self)
    }

    fn revision(&self) -> ClientRevision {
        ClientRevision {
            turn: self.turn(),
            snapshot_hash: self.snapshot().stable_hash(),
        }
    }

    fn run_state(&self) -> RunState {
        GameSession::run_state(self)
    }

    fn submit(&mut self, intent: CommandIntent) -> TurnOutcome {
        GameSession::submit(self, intent)
    }
}

#[cfg(test)]
mod allocation_transaction_tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use aihack_core::{action::CommandIntent, ids::EntityId, position::Direction};

    use super::GameSession;

    #[test]
    fn corpse_allocation_exhaustion_rejects_without_panicking_or_committing_partial_state() {
        let mut session = GameSession::new_for_playing(42);
        let state = session.inner.world.state_mut();
        state.entities.set_alive(EntityId(3), false);
        let jackal = state.entities.actor_stats_mut(EntityId(2)).unwrap();
        jackal.hp = 1;
        let player = state.entities.actor_stats_mut(state.player_id).unwrap();
        player.hit_bonus = 100;
        let mut entities = serde_json::to_value(&state.entities).unwrap();
        entities["next_id"] = serde_json::json!(u32::MAX);
        state.entities = serde_json::from_value(entities).unwrap();
        let before = session.to_save_data();

        let result = catch_unwind(AssertUnwindSafe(|| {
            session.submit(CommandIntent::Move(Direction::East))
        }));

        assert!(result.is_ok(), "allocation exhaustion must not panic");
        let outcome = result.unwrap();
        assert!(!outcome.accepted);
        assert_eq!(session.to_save_data(), before);
    }
}

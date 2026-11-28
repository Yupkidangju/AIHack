use std::ops::Deref;

use aihack_ai_contract::{ClientRevision, Observation};
use aihack_content::ContentRegistry;
use aihack_core::{
    action::{
        CommandIntent, DipTarget, DirectionalAction, EngravingMethod, EngravingText,
        InventoryAction, WishChoice,
    },
    campaign::SpellKind,
    domain::{
        combat::DeathCause,
        monster::MonsterPassive,
        status::{HungerState, STARVATION_THRESHOLD},
        tile::TrapKind,
    },
    error::ContentError,
    event::{GameEvent, MessagePriority},
    ids::{BranchId, EntityId},
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
        altar, ascension, combat, containers, death, dip, doors, engrave, items, monster_ai,
        movement, projectiles, score, spells, stairs, terrain, tools, traps,
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
        if transaction.was_aborted() || !outcome.accepted {
            if !transaction.was_aborted()
                && matches!(
                    self.state,
                    RunState::AwaitingDirection { .. }
                        | RunState::AwaitingInventorySelection { .. }
                )
                && outcome.next_state == RunState::Playing
            {
                self.inner.state = RunState::Playing;
            }
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
                        "{role:?}: recover the Amulet in Gehennom 4 and offer it on your Astral high altar."
                    ),
                }])
            }
            CommandIntent::StartCampaignWithBuild {
                role,
                race,
                alignment,
            } => {
                if self.turn != 0 || self.world.campaign.is_some() {
                    return self.reject("campaign requires a fresh creation state".into());
                }
                if !role.is_valid_combo(race, alignment) {
                    return self.reject(format!(
                        "invalid build: {} cannot be {} {}",
                        role.display_name(),
                        alignment.as_str(),
                        race.as_str()
                    ));
                }
                let registry = self.world.content_registry().clone();
                let state = match crate::bootstrap::campaign_world_with_build(
                    &registry,
                    self.seed(),
                    role,
                    race,
                    alignment,
                ) {
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
                        "{} {} {}: recover the Amulet in Gehennom 4 and offer it on your Astral high altar.",
                        alignment.as_str(),
                        race.as_str(),
                        role.display_name()
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
            CommandIntent::StartCampaign { .. } | CommandIntent::StartCampaignWithBuild { .. } => {
                self.reject("role is chosen only during creation".into())
            }
            CommandIntent::EnterBranch => match stairs::enter_branch(&mut self.inner.world) {
                Ok(mut events) => {
                    let entered_to = events.iter().find_map(|event| match event {
                        GameEvent::LevelChanged { entity, to, .. }
                            if *entity == self.world.player_id =>
                        {
                            Some(*to)
                        }
                        _ => None,
                    });
                    if let Some(to) = entered_to {
                        match ascension::on_level_entered(&mut self.inner.world, to) {
                            Ok(pursuit_events) => events.extend(pursuit_events),
                            Err(error) => return self.abort_transaction(error),
                        }
                    }
                    self.accept_turn(events)
                }
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
            CommandIntent::PutOn { item } => self.submit_put_on(item),
            CommandIntent::RemoveRing => self.submit_remove_ring(),
            CommandIntent::TakeOff { slot } => self.submit_take_off(slot),
            CommandIntent::Buy { kind } => self.submit_buy(kind),
            CommandIntent::Sell { item } => self.submit_sell(item),
            CommandIntent::Quaff { item } => self.submit_quaff(item),
            CommandIntent::Eat { item } => self.submit_eat(item),
            CommandIntent::Zap { item, direction } => self.submit_zap(item, direction),
            CommandIntent::Read { item } => self.submit_read(item),
            CommandIntent::Identify { scroll, target } => self.submit_identify(scroll, target),
            CommandIntent::Offer { item } => self.submit_offer(item),
            CommandIntent::Cast { spell, direction } => self.submit_cast(spell, direction),
            CommandIntent::Wish { item, wish } => self.submit_wish(item, wish),
            CommandIntent::Pray => self.submit_pray(),
            CommandIntent::Descend => self.submit_descend(),
            CommandIntent::Ascend => self.submit_ascend(),
            CommandIntent::Apply { item, direction } => self.submit_apply(item, direction),
            CommandIntent::ContainerPut { container, item } => {
                self.submit_container_put(container, item)
            }
            CommandIntent::ContainerTake { container, item } => {
                self.submit_container_take(container, item)
            }
            CommandIntent::Dip { item, target } => self.submit_dip(item, target),
            CommandIntent::Engrave { method, text } => self.submit_engrave(method, text),
            CommandIntent::Drink => self.submit_drink(),
            CommandIntent::Invoke { item } => self.submit_invoke(item),
            CommandIntent::Enhance { skill } => self.submit_enhance(skill),
            CommandIntent::Chat(direction) => self.submit_chat(direction),
            CommandIntent::Pay => self.submit_pay(),
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
            CommandIntent::PutOn { item } if action == InventoryAction::PutOn => {
                self.submit_put_on(item)
            }
            CommandIntent::Sell { item } if action == InventoryAction::Sell => {
                self.submit_sell(item)
            }
            CommandIntent::Quaff { item } if action == InventoryAction::Quaff => {
                self.submit_quaff(item)
            }
            CommandIntent::Eat { item } if action == InventoryAction::Eat => self.submit_eat(item),
            CommandIntent::Read { item } if action == InventoryAction::Read => {
                self.submit_read(item)
            }
            CommandIntent::Offer { item } if action == InventoryAction::Offer => {
                self.submit_offer(item)
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
        let (direction, stagger_event) =
            if self.world.status_timers.confused > 0 || self.world.status_timers.stunned > 0 {
                if self.inner.rng.next_u64().is_multiple_of(2) {
                    let idx = (self.inner.rng.next_u64() as usize) % Direction::ALL.len();
                    (
                        Direction::ALL[idx],
                        Some(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: "You stagger in your confusion!".to_string(),
                        }),
                    )
                } else {
                    (direction, None)
                }
            } else {
                (direction, None)
            };
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
        if self.world.campaign.is_some() {
            if let Some(friendly) = self
                .world
                .entities
                .alive_friendly_at(self.world.current_level(), to)
            {
                return match movement::swap_player_with_friendly(
                    &mut self.inner.world,
                    friendly,
                    direction,
                ) {
                    Ok((old_pos, new_pos)) => {
                        let friendly_name = self
                            .world
                            .entities
                            .get(friendly)
                            .and_then(|e| e.actor_kind())
                            .and_then(|k| k.monster_kind())
                            .map(|mk| {
                                aihack_content::monster_display_name(
                                    mk,
                                    self.world.content_registry(),
                                )
                            })
                            .unwrap_or_else(|| "companion".to_string());
                        let mut events = Vec::new();
                        if let Some(event) = stagger_event {
                            events.push(event);
                        }
                        events.push(GameEvent::EntityMoved {
                            entity: self.world.player_id(),
                            from: old_pos,
                            to: new_pos,
                        });
                        events.push(GameEvent::EntityMoved {
                            entity: friendly,
                            from: new_pos,
                            to: old_pos,
                        });
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: format!("You swap places with the {friendly_name}."),
                        });
                        events.extend(crate::systems::npc::check_shop_theft_on_move_or_stairs(
                            &mut self.inner.world,
                            false,
                        ));
                        self.inner.state =
                            death::state_after_deaths_at(&self.world, self.turn.saturating_add(1));
                        self.accept_turn(events)
                    }
                    Err(error) => self.reject(format!("{error}")),
                };
            }
        }
        if self.world.campaign.is_some()
            && self.world.current_map().tile(to).ok()
                == Some(aihack_core::domain::tile::TileKind::Boulder)
        {
            return match movement::push_boulder(&mut self.inner.world, direction) {
                Ok((old_pos, new_pos, plugged)) => {
                    let mut events = Vec::new();
                    if let Some(event) = stagger_event {
                        events.push(event);
                    }
                    events.push(GameEvent::EntityMoved {
                        entity: self.world.player_id(),
                        from: old_pos,
                        to: new_pos,
                    });
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: if plugged {
                            "You push the boulder into the pit, plugging it!".to_string()
                        } else {
                            "You push the boulder.".to_string()
                        },
                    });
                    events.extend(crate::systems::npc::check_shop_theft_on_move_or_stairs(
                        &mut self.inner.world,
                        false,
                    ));
                    self.inner.state =
                        death::state_after_deaths_at(&self.world, self.turn.saturating_add(1));
                    self.accept_turn(events)
                }
                Err(error) => self.reject(format!("{error}")),
            };
        }
        match movement::move_player(&mut self.inner.world, direction) {
            Ok(()) => {
                let to = self.world.player_pos();
                let mut events = Vec::new();
                if let Some(event) = stagger_event {
                    events.push(event);
                }
                events.push(GameEvent::EntityMoved {
                    entity: self.world.player_id(),
                    from,
                    to,
                });
                events.extend(crate::systems::npc::check_shop_theft_on_move_or_stairs(
                    &mut self.inner.world,
                    false,
                ));
                let (trap_events, triggered_trap) = {
                    let state = &mut self.inner;
                    match traps::trigger_player_trap_with_kind(&mut state.world, &mut state.rng) {
                        Ok(result) => result,
                        Err(error) => return self.abort_transaction(error),
                    }
                };
                events.extend(trap_events);
                let player_id = self.world.player_id();
                let death_events = match death::collect_death_events_if_hp_depleted(
                    &mut self.inner.world,
                    player_id,
                    DeathCause::Trap {
                        trap: triggered_trap.unwrap_or(TrapKind::Pit),
                    },
                ) {
                    Ok(events) => events,
                    Err(error) => return self.abort_transaction(error),
                };
                events.extend(death_events);
                if self.world.player_alive() {
                    let seed = self.meta.seed;
                    let (terrain_events, terrain_cause) = {
                        let state = &mut self.inner;
                        terrain::on_player_enter_tile(seed, &mut state.world, &mut state.rng)
                    };
                    events.extend(terrain_events);
                    if let Some(cause) = terrain_cause {
                        let terrain_death = match death::collect_death_events_if_hp_depleted(
                            &mut self.inner.world,
                            player_id,
                            cause,
                        ) {
                            Ok(events) => events,
                            Err(error) => return self.abort_transaction(error),
                        };
                        events.extend(terrain_death);
                    }
                }
                self.inner.state =
                    death::state_after_deaths_at(&self.world, self.turn.saturating_add(1));
                self.accept_turn(events)
            }
            Err(error) => {
                if stagger_event.is_some()
                    && self.world.carried_weight() <= self.world.carrying_capacity()
                {
                    return self.accept_turn(vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "You bump into an obstacle in your confusion.".to_string(),
                    }]);
                }
                self.reject(format!("{error}"))
            }
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
            let gaze_blocked = self.world.campaign.is_some()
                && (self.world.status_timers.blind > 0
                    || crate::systems::tools::is_player_blindfolded(&self.world));
            if gaze_blocked {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Your blindness shields you from the floating eye's gaze!".to_string(),
                });
            } else {
                self.inner.world.state_mut().paralysis_turns = self.world.paralysis_turns.max(2);
                events.push(GameEvent::PassiveAttackTriggered {
                    source: defender,
                    target: attacker,
                });
            }
        }
        if resolution.hit
            && self.world.campaign.is_some()
            && matches!(
                self.world
                    .entities
                    .get(defender)
                    .and_then(|entity| entity.monster_passive()),
                Some(MonsterPassive::SplitOnHit)
            )
        {
            match crate::systems::monster_ai::try_split_on_hit(&mut self.inner.world, defender) {
                Ok(split_events) => events.extend(split_events),
                Err(error) => return self.abort_transaction(error),
            }
        }
        events.extend(engrave::erode_elbereth_on_attack(&mut self.inner.world));
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
        let floor_item = self
            .world
            .entities
            .item_at(self.world.current_level(), self.world.player_pos());
        match items::pickup(&mut self.inner.world) {
            Ok(event) => {
                let mut events = vec![event];
                if let Some(item) = floor_item {
                    if self.world.campaign.is_some()
                        && self.world.entities.get(item).and_then(|e| e.item_unpaid()) == Some(true)
                        && crate::systems::npc::has_peaceful_shopkeeper_on_level(&self.world)
                    {
                        let price = crate::systems::npc::unpaid_item_price(&self.world, item);
                        let item_name = self
                            .world
                            .entities
                            .item_data(item)
                            .map(|d| {
                                crate::appearance::display_name_for_world_item(
                                    self.meta.seed,
                                    &self.world,
                                    item,
                                    d.kind,
                                )
                            })
                            .unwrap_or_else(|| "item".to_string());
                        let here_debt = crate::systems::npc::shop_debt_on(
                            &self.world,
                            self.world.current_level(),
                        );
                        let debt_note = if self.world.shop_debt > here_debt {
                            format!(
                                "Shop debt: {here_debt} gold (total: {} gold).",
                                self.world.shop_debt
                            )
                        } else {
                            format!("Shop debt: {here_debt} gold.")
                        };
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: format!(
                                "You pick up {item_name} (unpaid, {price} gold). {debt_note}"
                            ),
                        });
                    }
                }
                if let (Some(item), Some(campaign)) = (floor_item, self.world.campaign) {
                    if let Some(def) =
                        crate::systems::artifacts::artifact_def_for_item(&self.world, item)
                    {
                        if crate::systems::artifacts::is_artifact_hostile_to(
                            def,
                            campaign.alignment,
                        ) {
                            let state = &mut self.inner;
                            match crate::systems::artifacts::apply_artifact_blast(
                                &mut state.world,
                                &mut state.rng,
                                def,
                                2,
                                6,
                                "pick up",
                            ) {
                                Ok(blast) => {
                                    events.extend(blast);
                                    self.inner.state = death::state_after_deaths_at(
                                        &self.world,
                                        self.turn.saturating_add(1),
                                    );
                                }
                                Err(error) => return self.abort_transaction(error),
                            }
                        }
                    }
                }
                self.accept_turn(events)
            }
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
        if let Some(campaign) = self.world.campaign {
            if self.world.inventory.contains(item) {
                if let Some(def) =
                    crate::systems::artifacts::artifact_def_for_item(&self.world, item)
                {
                    if crate::systems::artifacts::is_artifact_hostile_to(def, campaign.alignment) {
                        let state = &mut self.inner;
                        return match crate::systems::artifacts::apply_artifact_blast(
                            &mut state.world,
                            &mut state.rng,
                            def,
                            4,
                            4,
                            "wield",
                        ) {
                            Ok(events) => {
                                self.inner.state = death::state_after_deaths_at(
                                    &self.world,
                                    self.turn.saturating_add(1),
                                );
                                self.accept_turn(events)
                            }
                            Err(error) => self.abort_transaction(error),
                        };
                    }
                }
            }
        }
        match items::wield(&mut self.inner.world, item) {
            Ok(Some(event)) => self.accept_turn(vec![event]),
            Ok(None) => self.accept_without_turn(Vec::new()),
            Err(error) => self.reject(error),
        }
    }

    fn submit_quaff(&mut self, item: EntityId) -> TurnOutcome {
        let next_turn = self.turn.saturating_add(1);
        let state = &mut self.inner;
        match items::quaff(&mut state.world, &mut state.rng, item) {
            Ok(events) => {
                state.state = death::state_after_deaths_at(&state.world, next_turn);
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_eat(&mut self, item: EntityId) -> TurnOutcome {
        let state = &mut self.inner;
        match items::eat(&mut state.world, &mut state.rng, item) {
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
        let turn = self.turn;
        let next_turn = turn.saturating_add(1);
        let state = &mut self.inner;
        match items::read_at(&mut state.world, &mut state.rng, item, turn) {
            Ok(events) => {
                self.inner.state = death::state_after_deaths_at(&self.world, next_turn);
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_identify(&mut self, scroll: EntityId, target: EntityId) -> TurnOutcome {
        match items::identify(&mut self.inner.world, scroll, target) {
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
        if self.world.campaign.is_some() {
            let state = &mut self.inner;
            return match altar::resolve_campaign_pray(&mut state.world, &mut state.rng) {
                Ok(events) => {
                    self.inner.state =
                        death::state_after_deaths_at(&self.world, self.turn.saturating_add(1));
                    self.accept_turn(events)
                }
                Err(error) => self.reject(error),
            };
        }
        if self.world.prayer_cooldown > 0 {
            return self.reject("prayer is on cooldown".to_string());
        }
        self.inner.world.state_mut().prayer_cooldown = 20;
        self.inner.world.state_mut().luck = self.world.luck.saturating_add(1).min(3);
        let events = vec![GameEvent::PrayerOffered {
            entity: self.world.player_id(),
            cooldown_after: self.world.prayer_cooldown,
        }];
        self.accept_turn(events)
    }

    fn submit_offer(&mut self, item: EntityId) -> TurnOutcome {
        let next_turn = self.turn.saturating_add(1);
        let astral_victory_eligible =
            self.world.campaign.is_some_and(|campaign| {
                let level = self.world.current_level();
                let pos = self.world.player_pos();
                level == aihack_core::ids::LevelId::new(BranchId::Planes, 5)
                    && self.world.ascension_altars.contains(&(level, pos))
                    && altar::altar_alignment(&self.world) == Some(campaign.alignment)
                    && self.world.entities.item_data(item).is_some_and(|d| {
                        d.kind == aihack_core::domain::item::ItemKind::AmuletAscension
                    })
                    && self.world.player_alive()
            });
        let state = &mut self.inner;
        match altar::offer(&mut state.world, &mut state.rng, item) {
            Ok(events) => {
                if astral_victory_eligible {
                    let Some(final_score) =
                        score::death_score(&self.world, next_turn).checked_add(10_000)
                    else {
                        return self.abort_transaction("victory score overflows".into());
                    };
                    self.inner.state = RunState::Victory { final_score };
                } else {
                    self.inner.state = death::state_after_deaths_at(&self.world, next_turn);
                }
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_invoke(&mut self, item: EntityId) -> TurnOutcome {
        let state = &mut self.inner;
        match crate::systems::artifacts::invoke(&mut state.world, &mut state.rng, item) {
            Ok(events) => {
                self.inner.state =
                    death::state_after_deaths_at(&self.world, self.turn.saturating_add(1));
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_enhance(&mut self, skill: aihack_core::campaign::SkillCategory) -> TurnOutcome {
        match crate::systems::skills::enhance(&mut self.inner.world, skill) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_chat(&mut self, direction: Direction) -> TurnOutcome {
        match crate::systems::npc::chat(self.meta.seed, self.turn, &self.inner.world, direction) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_pay(&mut self) -> TurnOutcome {
        let seed = self.meta.seed;
        let state = &mut self.inner;
        match crate::systems::npc::pay(seed, &mut state.world, &mut state.rng) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_cast(&mut self, spell: SpellKind, direction: Option<Direction>) -> TurnOutcome {
        let turn = self.turn;
        let state = &mut self.inner;
        match spells::cast_at(&mut state.world, &mut state.rng, spell, direction, turn) {
            Ok(events) => {
                self.inner.state =
                    death::state_after_deaths_at(&self.world, self.turn.saturating_add(1));
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_wish(&mut self, item: EntityId, wish: WishChoice) -> TurnOutcome {
        let next_turn = self.turn.saturating_add(1);
        let state = &mut self.inner;
        match projectiles::wish(&mut state.world, &mut state.rng, item, wish) {
            Ok(events) => {
                self.inner.state = death::state_after_deaths_at(&self.world, next_turn);
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
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
            Ok(events) => self.accept_turn(events),
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
            match stairs::enter_planes(&mut self.inner.world) {
                Ok(mut events) => {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "You ascend beyond the dungeon into the Plane of Earth!".into(),
                    });
                    if let Err(error) = ascension::on_level_entered(
                        &mut self.inner.world,
                        aihack_core::ids::LevelId::new(BranchId::Planes, 1),
                    ) {
                        return self.abort_transaction(error);
                    }
                    return self.accept_turn(events);
                }
                Err(error) => return self.reject(error),
            }
        }
        match stairs::ascend(&mut self.inner.world) {
            Ok(mut events) => {
                let entered_to = events.iter().find_map(|event| match event {
                    GameEvent::LevelChanged { entity, to, .. }
                        if *entity == self.world.player_id =>
                    {
                        Some(*to)
                    }
                    _ => None,
                });
                if let Some(to) = entered_to {
                    match ascension::on_level_entered(&mut self.inner.world, to) {
                        Ok(pursuit_events) => events.extend(pursuit_events),
                        Err(error) => return self.abort_transaction(error),
                    }
                }
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_put_on(&mut self, item: EntityId) -> TurnOutcome {
        match items::put_on(&mut self.inner.world, item) {
            Ok(event) => self.accept_turn(event.into_iter().collect()),
            Err(error) => self.reject(error),
        }
    }

    fn submit_remove_ring(&mut self) -> TurnOutcome {
        match items::remove_ring(&mut self.inner.world) {
            Ok(event) => self.accept_turn(vec![event]),
            Err(error) => self.reject(error),
        }
    }

    fn submit_take_off(&mut self, slot: aihack_core::domain::item::EquipmentSlot) -> TurnOutcome {
        match items::take_off(&mut self.inner.world, slot) {
            Ok(event) => self.accept_turn(vec![event]),
            Err(error) => self.reject(error),
        }
    }

    fn submit_buy(&mut self, kind: aihack_core::domain::item::ItemKind) -> TurnOutcome {
        let seed = self.meta.seed;
        match crate::systems::shop::buy(seed, &mut self.inner.world, kind) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_sell(&mut self, item: EntityId) -> TurnOutcome {
        let seed = self.meta.seed;
        match crate::systems::shop::sell(seed, &mut self.inner.world, item) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_apply(&mut self, item: EntityId, direction: Option<Direction>) -> TurnOutcome {
        let state = &mut self.inner;
        match tools::apply(&mut state.world, &mut state.rng, item, direction) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_container_put(&mut self, container: EntityId, item: EntityId) -> TurnOutcome {
        let next_turn = self.turn.saturating_add(1);
        let seed = self.meta.seed;
        let state = &mut self.inner;
        match containers::container_put(seed, &mut state.world, &mut state.rng, container, item) {
            Ok(events) => {
                state.state = death::state_after_deaths_at(&state.world, next_turn);
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_container_take(&mut self, container: EntityId, item: EntityId) -> TurnOutcome {
        let next_turn = self.turn.saturating_add(1);
        let seed = self.meta.seed;
        let state = &mut self.inner;
        match containers::container_take(seed, &mut state.world, &mut state.rng, container, item) {
            Ok(events) => {
                state.state = death::state_after_deaths_at(&state.world, next_turn);
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_dip(&mut self, item: EntityId, target: DipTarget) -> TurnOutcome {
        let next_turn = self.turn.saturating_add(1);
        let seed = self.meta.seed;
        let state = &mut self.inner;
        match dip::dip(seed, &mut state.world, &mut state.rng, item, target) {
            Ok(events) => {
                state.state = death::state_after_deaths_at(&state.world, next_turn);
                self.accept_turn(events)
            }
            Err(error) => self.reject(error),
        }
    }

    fn submit_engrave(&mut self, method: EngravingMethod, text: EngravingText) -> TurnOutcome {
        match engrave::engrave(&mut self.inner.world, method, text) {
            Ok(events) => self.accept_turn(events),
            Err(error) => self.reject(error),
        }
    }

    fn submit_drink(&mut self) -> TurnOutcome {
        let next_turn = self.turn.saturating_add(1);
        let state = &mut self.inner;
        match terrain::drink_from_tile(&mut state.world, &mut state.rng) {
            Ok(events) => {
                state.state = death::state_after_deaths_at(&state.world, next_turn);
                self.accept_turn(events)
            }
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
        let prev_hunger = self.world.hunger_state();
        self.inner.world.state_mut().nutrition = self.world.nutrition.saturating_sub(1);
        self.inner.world.state_mut().prayer_cooldown = self.world.prayer_cooldown.saturating_sub(1);
        if let Some(mut c) = self.inner.world.campaign {
            c.prayer_timeout = c.prayer_timeout.saturating_sub(1);
            c.artifact_invoke_cooldown = c.artifact_invoke_cooldown.saturating_sub(1);
            self.inner.world.state_mut().campaign = Some(c);
        }
        if let Some(campaign) = self.world.campaign {
            if !matches!(
                self.state,
                RunState::GameOver { .. } | RunState::Victory { .. }
            ) {
                if next_turn.is_multiple_of(5) && self.world.mp < self.world.max_mp {
                    self.inner.world.state_mut().mp += 1;
                }
                if self.world.poison_turns == 0 && self.world.nutrition > 50 {
                    let bonus_reduction = campaign.attributes.con_regen_reduction();
                    let floor = if bonus_reduction > 0 { 4 } else { 5 };
                    let regen_interval = u64::from(
                        (15_u8
                            .saturating_sub(campaign.level())
                            .saturating_sub(bonus_reduction))
                        .max(floor),
                    );
                    if next_turn.is_multiple_of(regen_interval) {
                        let player_id = self.world.player_id();
                        if let Some(stats) = self
                            .inner
                            .world
                            .state_mut()
                            .entities
                            .actor_stats_mut(player_id)
                        {
                            if stats.hp > 0 && stats.hp < stats.max_hp {
                                stats.hp += 1;
                            }
                        }
                    }
                }
                let level = self.world.current_level();
                if crate::campaign::has_amulet(&self.world)
                    && level.branch == BranchId::Main
                    && (1..=5).contains(&level.depth)
                    && next_turn.is_multiple_of(15)
                {
                    self.inner.world.state_mut().nutrition = self.world.nutrition.saturating_sub(1);
                }
                let moved_this_turn = events.iter().any(|e| {
                    matches!(e, GameEvent::EntityMoved { entity, .. } if *entity == self.world.player_id)
                });
                let mut levitation_expired = false;
                {
                    let state = self.inner.world.state_mut();
                    let timers = &mut state.status_timers;
                    let mut expiry_msgs = Vec::new();
                    for (timer, msg) in [
                        (&mut timers.blind, "You can see again."),
                        (&mut timers.confused, "You feel less confused now."),
                        (&mut timers.stunned, "You are no longer stunned."),
                        (&mut timers.hallucinating, "Everything looks SO boring now."),
                        (&mut timers.fast, "You feel yourself slowing down."),
                        (&mut timers.levitation, "You float gently to the floor."),
                        (&mut timers.invisible, "Your body returns to visibility."),
                    ] {
                        if *timer > 0 {
                            *timer -= 1;
                            if *timer == 0 {
                                if msg == "You float gently to the floor." {
                                    levitation_expired = true;
                                }
                                expiry_msgs.push(msg);
                            }
                        }
                    }
                    if timers.hallucinating == 0
                        && expiry_msgs.contains(&"Everything looks SO boring now.")
                    {
                        state.hallucinating = false;
                    }
                    for text in expiry_msgs {
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: text.to_string(),
                        });
                    }
                }
                let player_id = self.world.player_id;
                let current_tile = self.world.current_map().tile(self.world.player_pos()).ok();
                if self.world.player_alive()
                    && matches!(
                        current_tile,
                        Some(
                            aihack_core::domain::tile::TileKind::Water
                                | aihack_core::domain::tile::TileKind::Lava
                        )
                    )
                    && (levitation_expired || (!moved_this_turn && !self.world.is_levitating()))
                {
                    let seed = self.meta.seed;
                    let (terrain_events, terrain_cause) = {
                        let state = &mut self.inner;
                        terrain::on_player_enter_tile(seed, &mut state.world, &mut state.rng)
                    };
                    events.extend(terrain_events);
                    if let Some(cause) = terrain_cause {
                        let terrain_death = match death::collect_death_events_if_hp_depleted(
                            &mut self.inner.world,
                            player_id,
                            cause,
                        ) {
                            Ok(events) => events,
                            Err(error) => return self.abort_transaction(error),
                        };
                        events.extend(terrain_death);
                    }
                    self.inner.state = death::state_after_deaths_at(&self.world, next_turn);
                }
                events.extend(tools::tick_active_lamps(&mut self.inner.world));
                events.extend(crate::systems::pets::tick_pets_hunger(
                    &mut self.inner.world,
                    next_turn,
                ));
                {
                    let state = &mut self.inner;
                    engrave::tick_dust_engravings(&mut state.world, &mut state.rng);
                }
                let troll_template = crate::domain::monster::try_monster_template_from_registry(
                    aihack_core::domain::monster::MonsterKind::Troll,
                    self.world.content_registry(),
                )
                .ok();
                {
                    let current_level = self.world.current_level();
                    let state = self.inner.world.state_mut();
                    let corpses: Vec<_> = state
                        .entities
                        .entities()
                        .iter()
                        .filter_map(|e| {
                            let (kind, _, loc, _, _) = e.item()?;
                            if matches!(
                                kind,
                                aihack_core::domain::item::ItemKind::Corpse
                                    | aihack_core::domain::item::ItemKind::CorpseJackal
                            ) && loc != aihack_core::domain::entity::EntityLocation::Consumed
                            {
                                Some((
                                    e.id,
                                    e.item_corpse_monster(),
                                    e.item_corpse_age().unwrap_or(0),
                                    loc,
                                ))
                            } else {
                                None
                            }
                        })
                        .collect();
                    let mut revivals = Vec::new();
                    for (id, monster, age, loc) in corpses {
                        let next_age = age.saturating_add(1);
                        state.entities.set_item_corpse_age(id, next_age);
                        if monster == Some(aihack_core::domain::monster::MonsterKind::Troll)
                            && next_age >= 25
                        {
                            if let aihack_core::domain::entity::EntityLocation::OnMap {
                                level,
                                pos,
                            } = loc
                            {
                                if state.entities.alive_actor_at(level, pos).is_none() {
                                    revivals.push((id, level, pos));
                                }
                            }
                        }
                    }
                    if let Some(template) = troll_template {
                        for (corpse_id, level, pos) in revivals {
                            state.entities.set_item_location(
                                corpse_id,
                                aihack_core::domain::entity::EntityLocation::Consumed,
                            );
                            if let Ok(troll_id) = state.entities.spawn_monster_with_template(
                                aihack_core::domain::monster::MonsterKind::Troll,
                                template,
                                pos,
                            ) {
                                state.entities.set_actor_location(troll_id, level, pos);
                                if level == current_level {
                                    events.push(GameEvent::Message {
                                        priority: MessagePriority::Warning,
                                        text: "The troll corpse shudders and comes back to life!"
                                            .to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        if self.world.paralysis_turns > 0 {
            self.inner.world.state_mut().paralysis_turns -= 1;
        }
        if !matches!(
            self.state,
            RunState::GameOver { .. } | RunState::Victory { .. }
        ) {
            let curr_hunger = self.world.hunger_state();
            if self.world.nutrition <= STARVATION_THRESHOLD {
                let player_id = self.world.player_id();
                if let Some(stats) = self
                    .inner
                    .world
                    .state_mut()
                    .entities
                    .actor_stats_mut(player_id)
                {
                    stats.hp = 0;
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Danger,
                    text: "You starve to death!".to_string(),
                });
                let death_events = match death::collect_death_events_if_hp_depleted(
                    &mut self.inner.world,
                    player_id,
                    DeathCause::Starvation,
                ) {
                    Ok(death_events) => death_events,
                    Err(error) => return self.abort_transaction(error),
                };
                events.extend(death_events);
                self.inner.state = death::state_after_deaths_at(&self.world, next_turn);
            } else if curr_hunger == HungerState::Fainting {
                if prev_hunger != HungerState::Fainting || next_turn.is_multiple_of(10) {
                    self.inner.world.state_mut().paralysis_turns =
                        self.world.paralysis_turns.max(1);
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Warning,
                        text: "You faint from lack of food!".to_string(),
                    });
                }
            } else if curr_hunger == HungerState::Weak && prev_hunger != HungerState::Weak {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "You feel weak from hunger.".to_string(),
                });
            } else if curr_hunger == HungerState::Hungry
                && prev_hunger != HungerState::Hungry
                && matches!(prev_hunger, HungerState::NotHungry | HungerState::Satiated)
            {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "You are beginning to feel hungry.".to_string(),
                });
            }
        }
        if !matches!(
            self.state,
            RunState::GameOver { .. } | RunState::Victory { .. }
        ) && self.world.poison_turns > 0
        {
            if crate::systems::items::has_poison_resistance(&self.world) {
                self.inner.world.state_mut().poison_turns -= 1;
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "You resist the poison.".to_string(),
                });
            } else {
                let state = &mut self.inner;
                state.world.state_mut().poison_turns -= 1;
                let player_id = state.world.player_id();
                if let Some(stats) = state.world.state_mut().entities.actor_stats_mut(player_id) {
                    stats.hp = stats.hp.saturating_sub(1);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "You suffer from poison!".to_string(),
                });
                let death_events = match death::collect_death_events_if_hp_depleted(
                    &mut state.world,
                    player_id,
                    DeathCause::Poison,
                ) {
                    Ok(death_events) => death_events,
                    Err(error) => return self.abort_transaction(error),
                };
                events.extend(death_events);
                state.state = death::state_after_deaths_at(&state.world, next_turn);
            }
        }
        if !matches!(
            self.state,
            RunState::GameOver { .. } | RunState::Victory { .. }
        ) {
            let seed = self.meta.seed;
            let state = &mut self.inner;
            let monster_events = match monster_ai::run_monster_turn(
                seed,
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
        if self.world.campaign.is_some()
            && !matches!(
                self.state,
                RunState::GameOver { .. } | RunState::Victory { .. }
            )
            && self.world.has_searching()
        {
            events.extend(traps::search(&mut self.inner.world));
        }
        self.inner.world.mark_current_level_explored();
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

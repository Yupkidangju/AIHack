use std::collections::BTreeMap;

use aihack_core::{
    action::CommandIntent,
    domain::{
        entity::{EntityKind, EntityLocation},
        inventory::Inventory,
        item::ItemKind,
        monster::{MonsterAiKind, MonsterPassive},
    },
    event::GameEvent,
    ids::EntityId,
    position::Pos,
    run_state::RunState,
    turn::TurnOutcome,
};
use serde::Serialize;

use crate::{systems::score, GameSession};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalWitness {
    FoodNutrition,
    CorpseNutrition,
    ArmorDefense,
    MonsterSpeed,
    MonsterAi,
    MonsterPassive,
    MonsterDifficultyEconomy,
    PrayerLuckCombat,
    GoldScore,
}

pub const REQUIRED_CAUSAL_WITNESSES: [CausalWitness; 9] = [
    CausalWitness::FoodNutrition,
    CausalWitness::CorpseNutrition,
    CausalWitness::ArmorDefense,
    CausalWitness::MonsterSpeed,
    CausalWitness::MonsterAi,
    CausalWitness::MonsterPassive,
    CausalWitness::MonsterDifficultyEconomy,
    CausalWitness::PrayerLuckCombat,
    CausalWitness::GoldScore,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalScenario {
    FoodConsumption,
    CorpseConsumption,
    ArmorWear,
    MonsterSpeedPair,
    MonsterAiPair,
    PassiveCombat,
    DifficultyEconomy,
    PrayerCombat,
    GoldScoreProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalField {
    ItemNutrition,
    ArmorAcBonus,
    MonsterSpeed,
    MonsterAi,
    MonsterPassive,
    MonsterDifficulty,
    PlayerLuck,
    Gold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalConsumer {
    Nutrition,
    PlayerAc,
    MonsterPosition,
    ParalysisTurns,
    Gold,
    AttackRoll,
    FinalScore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum CausalValue {
    None,
    Signed(i64),
    Unsigned(u64),
    Position(Option<Pos>),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CausalWitnessRecord {
    pub witness: CausalWitness,
    pub scenario: CausalScenario,
    pub producer: Option<EntityId>,
    pub field: CausalField,
    pub source_before: CausalValue,
    pub source_after: CausalValue,
    pub consumer: CausalConsumer,
    pub consumer_before: CausalValue,
    pub consumer_after: CausalValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CausalEntityProjection {
    id: EntityId,
    kind: EntityKind,
    pos: Option<Pos>,
    hp: Option<i16>,
    alive: Option<bool>,
    location: Option<EntityLocation>,
    speed: Option<i16>,
    ai_kind: Option<MonsterAiKind>,
    passive: Option<MonsterPassive>,
    difficulty: Option<u16>,
    nutrition: Option<i16>,
    ac_bonus: Option<i16>,
    base_price: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalProjection {
    run_state: RunState,
    player_id: EntityId,
    player_pos: Pos,
    player_hp: i16,
    player_ac: i16,
    inventory: Inventory,
    nutrition: i16,
    luck: i16,
    prayer_cooldown: u16,
    paralysis_turns: u8,
    kill_count: u32,
    gold: u32,
    gold_scores: Option<(i64, i64)>,
    entities: Vec<CausalEntityProjection>,
}

impl CausalProjection {
    pub fn from_session(session: &GameSession) -> Self {
        let world = session.world();
        let player_id = world.player_id();
        let player_stats = world
            .entities()
            .actor_stats(player_id)
            .expect("player actor stats must exist");
        let mut entities = world
            .entities()
            .entities()
            .iter()
            .map(|entity| {
                if let Some((_, _, _, pos, stats, alive)) = entity.actor() {
                    CausalEntityProjection {
                        id: entity.id,
                        kind: entity.kind(),
                        pos: Some(pos),
                        hp: Some(stats.hp),
                        alive: Some(alive),
                        location: None,
                        speed: Some(stats.speed),
                        ai_kind: entity.monster_ai_kind(),
                        passive: entity.monster_passive(),
                        difficulty: entity.monster_difficulty(),
                        nutrition: None,
                        ac_bonus: None,
                        base_price: None,
                    }
                } else {
                    let (_, data, location, _, _) =
                        entity.item().expect("actor가 아니면 item payload여야 한다");
                    CausalEntityProjection {
                        id: entity.id,
                        kind: entity.kind(),
                        pos: None,
                        hp: None,
                        alive: None,
                        location: Some(location),
                        speed: None,
                        ai_kind: None,
                        passive: None,
                        difficulty: None,
                        nutrition: data.nutrition,
                        ac_bonus: Some(data.ac_bonus),
                        base_price: Some(data.base_price),
                    }
                }
            })
            .collect::<Vec<_>>();
        entities.sort_by_key(|entity| entity.id.0);

        let gold_scores = matches!(session.run_state(), RunState::GameOver { .. }).then(|| {
            let (with_gold, without_gold) = score::paired_gold_scores(world, session.turn());
            (i64::from(with_gold), i64::from(without_gold))
        });

        Self {
            run_state: session.run_state(),
            player_id,
            player_pos: world.player_pos(),
            player_hp: player_stats.hp,
            player_ac: player_stats.ac,
            inventory: world.inventory().clone(),
            nutrition: world.nutrition,
            luck: world.luck,
            prayer_cooldown: world.prayer_cooldown,
            paralysis_turns: world.paralysis_turns,
            kill_count: world.kill_count(),
            gold: world.gold(),
            gold_scores,
            entities,
        }
    }

    fn entity(&self, id: EntityId) -> Option<&CausalEntityProjection> {
        self.entities.iter().find(|entity| entity.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct CausalSummary {
    counts: BTreeMap<CausalWitness, u64>,
    records: Vec<CausalWitnessRecord>,
    prayer_luck_pending: bool,
    corpse_produced: bool,
}

impl CausalSummary {
    pub fn observe(
        &mut self,
        before: &CausalProjection,
        command: CommandIntent,
        outcome: &TurnOutcome,
        after: &CausalProjection,
    ) {
        if !outcome.accepted || before == after {
            return;
        }

        match command {
            CommandIntent::Eat { item } => self.observe_eating(before, after, item),
            CommandIntent::Wear { item } => {
                let bonus = before.entity(item).and_then(|entity| entity.ac_bonus);
                if before.inventory.equipped_body != Some(item)
                    && after.inventory.equipped_body == Some(item)
                    && bonus.is_some_and(|value| after.player_ac == before.player_ac - value)
                {
                    self.record(CausalWitnessRecord {
                        witness: CausalWitness::ArmorDefense,
                        scenario: CausalScenario::ArmorWear,
                        producer: Some(item),
                        field: CausalField::ArmorAcBonus,
                        source_before: CausalValue::Signed(0),
                        source_after: CausalValue::Signed(i64::from(bonus.unwrap_or_default())),
                        consumer: CausalConsumer::PlayerAc,
                        consumer_before: CausalValue::Signed(i64::from(before.player_ac)),
                        consumer_after: CausalValue::Signed(i64::from(after.player_ac)),
                    });
                }
            }
            CommandIntent::Pray => {
                if after.luck > before.luck && after.prayer_cooldown > before.prayer_cooldown {
                    self.prayer_luck_pending = true;
                }
            }
            CommandIntent::Quit => {
                if let (RunState::GameOver { final_score, .. }, Some((with_gold, without_gold))) =
                    (after.run_state, after.gold_scores)
                {
                    let gold_delta = with_gold.checked_sub(without_gold);
                    if before.gold > 0
                        && after.gold == before.gold
                        && i64::from(final_score) == with_gold
                        && gold_delta == Some(i64::from(before.gold))
                    {
                        self.record(CausalWitnessRecord {
                            witness: CausalWitness::GoldScore,
                            scenario: CausalScenario::GoldScoreProjection,
                            producer: Some(before.player_id),
                            field: CausalField::Gold,
                            source_before: CausalValue::Unsigned(0),
                            source_after: CausalValue::Unsigned(u64::from(before.gold)),
                            consumer: CausalConsumer::FinalScore,
                            consumer_before: CausalValue::Signed(without_gold),
                            consumer_after: CausalValue::Signed(i64::from(final_score)),
                        });
                    }
                }
            }
            _ => {}
        }

        self.observe_events(before, after, outcome);
    }

    pub fn count(&self, witness: CausalWitness) -> u64 {
        self.counts.get(&witness).copied().unwrap_or(0)
    }

    pub fn total_count(&self) -> u64 {
        self.counts.values().sum()
    }

    pub fn records(&self) -> &[CausalWitnessRecord] {
        &self.records
    }

    pub fn observe_monster_speed_pair(
        &mut self,
        active_before: &CausalProjection,
        active_after: &CausalProjection,
        control_before: &CausalProjection,
        control_after: &CausalProjection,
        entity: EntityId,
    ) {
        let (Some(active_before_entity), Some(active_after_entity)) =
            (active_before.entity(entity), active_after.entity(entity))
        else {
            return;
        };
        let (Some(control_before_entity), Some(control_after_entity)) =
            (control_before.entity(entity), control_after.entity(entity))
        else {
            return;
        };
        if active_before_entity.kind != control_before_entity.kind
            || active_before_entity.ai_kind != control_before_entity.ai_kind
            || active_before_entity.pos != control_before_entity.pos
            || active_before_entity.speed.is_none_or(|speed| speed <= 0)
            || control_before_entity.speed != Some(0)
            || active_after_entity.pos == active_before_entity.pos
            || control_after_entity.pos != control_before_entity.pos
        {
            return;
        }
        self.record(CausalWitnessRecord {
            witness: CausalWitness::MonsterSpeed,
            scenario: CausalScenario::MonsterSpeedPair,
            producer: Some(entity),
            field: CausalField::MonsterSpeed,
            source_before: CausalValue::Signed(i64::from(
                control_before_entity.speed.unwrap_or_default(),
            )),
            source_after: CausalValue::Signed(i64::from(
                active_before_entity.speed.unwrap_or_default(),
            )),
            consumer: CausalConsumer::MonsterPosition,
            consumer_before: CausalValue::Position(control_after_entity.pos),
            consumer_after: CausalValue::Position(active_after_entity.pos),
        });
    }

    pub fn observe_monster_ai_pair(
        &mut self,
        active_before: &CausalProjection,
        active_after: &CausalProjection,
        control_before: &CausalProjection,
        control_after: &CausalProjection,
        entity: EntityId,
    ) {
        let (Some(active_before_entity), Some(active_after_entity)) =
            (active_before.entity(entity), active_after.entity(entity))
        else {
            return;
        };
        let (Some(control_before_entity), Some(control_after_entity)) =
            (control_before.entity(entity), control_after.entity(entity))
        else {
            return;
        };
        if active_before_entity.kind != control_before_entity.kind
            || active_before_entity.speed != control_before_entity.speed
            || active_before_entity.pos != control_before_entity.pos
            || active_before_entity.ai_kind == control_before_entity.ai_kind
            || active_before_entity.ai_kind == Some(MonsterAiKind::Stationary)
            || control_before_entity.ai_kind != Some(MonsterAiKind::Stationary)
            || active_after_entity.pos == active_before_entity.pos
            || control_after_entity.pos != control_before_entity.pos
        {
            return;
        }
        self.record(CausalWitnessRecord {
            witness: CausalWitness::MonsterAi,
            scenario: CausalScenario::MonsterAiPair,
            producer: Some(entity),
            field: CausalField::MonsterAi,
            source_before: CausalValue::Text(format!(
                "{:?}",
                control_before_entity
                    .ai_kind
                    .unwrap_or(MonsterAiKind::Stationary)
            )),
            source_after: CausalValue::Text(format!(
                "{:?}",
                active_before_entity
                    .ai_kind
                    .unwrap_or(MonsterAiKind::Stationary)
            )),
            consumer: CausalConsumer::MonsterPosition,
            consumer_before: CausalValue::Position(control_after_entity.pos),
            consumer_after: CausalValue::Position(active_after_entity.pos),
        });
    }

    pub fn validate_required(&self) -> Result<(), Vec<CausalWitness>> {
        let missing = REQUIRED_CAUSAL_WITNESSES
            .into_iter()
            .filter(|witness| self.count(*witness) == 0)
            .collect::<Vec<_>>();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    pub fn without(mut self, witness: CausalWitness) -> Self {
        self.counts.remove(&witness);
        self.records.retain(|record| record.witness != witness);
        self
    }

    fn observe_eating(
        &mut self,
        before: &CausalProjection,
        after: &CausalProjection,
        item: EntityId,
    ) {
        let Some(before_item) = before.entity(item) else {
            return;
        };
        let Some(after_item) = after.entity(item) else {
            return;
        };
        if after.nutrition <= before.nutrition
            || after_item.location != Some(EntityLocation::Consumed)
        {
            return;
        }
        let source_after = CausalValue::Signed(i64::from(
            before_item
                .nutrition
                .unwrap_or(after.nutrition - before.nutrition),
        ));
        let common = |witness, scenario| CausalWitnessRecord {
            witness,
            scenario,
            producer: Some(item),
            field: CausalField::ItemNutrition,
            source_before: CausalValue::Signed(0),
            source_after: source_after.clone(),
            consumer: CausalConsumer::Nutrition,
            consumer_before: CausalValue::Signed(i64::from(before.nutrition)),
            consumer_after: CausalValue::Signed(i64::from(after.nutrition)),
        };
        match before_item.kind {
            EntityKind::Item(ItemKind::FoodRation) => self.record(common(
                CausalWitness::FoodNutrition,
                CausalScenario::FoodConsumption,
            )),
            EntityKind::Item(ItemKind::CorpseJackal) if self.corpse_produced => {
                self.record(common(
                    CausalWitness::CorpseNutrition,
                    CausalScenario::CorpseConsumption,
                ))
            }
            _ => {}
        }
    }

    fn observe_events(
        &mut self,
        before: &CausalProjection,
        after: &CausalProjection,
        outcome: &TurnOutcome,
    ) {
        if let Some(source) = outcome.events.iter().find_map(|event| match event {
            GameEvent::PassiveAttackTriggered { source, target }
                if *target == before.player_id
                    && before
                        .entity(*source)
                        .and_then(|entity| entity.passive)
                        .is_some() =>
            {
                Some(*source)
            }
            _ => None,
        }) {
            if after.paralysis_turns > before.paralysis_turns {
                let passive = before.entity(source).and_then(|entity| entity.passive);
                self.record(CausalWitnessRecord {
                    witness: CausalWitness::MonsterPassive,
                    scenario: CausalScenario::PassiveCombat,
                    producer: Some(source),
                    field: CausalField::MonsterPassive,
                    source_before: CausalValue::None,
                    source_after: CausalValue::Text(format!("{passive:?}")),
                    consumer: CausalConsumer::ParalysisTurns,
                    consumer_before: CausalValue::Unsigned(u64::from(before.paralysis_turns)),
                    consumer_after: CausalValue::Unsigned(u64::from(after.paralysis_turns)),
                });
            }
        }

        let player_attack_roll = outcome.events.iter().find_map(|event| match event {
            GameEvent::AttackResolved {
                attacker,
                attack_roll,
                ..
            } if *attacker == before.player_id => Some(*attack_roll),
            _ => None,
        });
        if self.prayer_luck_pending && player_attack_roll.is_some() && before.luck > 0 {
            let attack_roll = player_attack_roll.unwrap_or_default();
            self.record(CausalWitnessRecord {
                witness: CausalWitness::PrayerLuckCombat,
                scenario: CausalScenario::PrayerCombat,
                producer: Some(before.player_id),
                field: CausalField::PlayerLuck,
                source_before: CausalValue::Signed(0),
                source_after: CausalValue::Signed(i64::from(before.luck)),
                consumer: CausalConsumer::AttackRoll,
                consumer_before: CausalValue::Signed(i64::from(attack_roll - before.luck)),
                consumer_after: CausalValue::Signed(i64::from(attack_roll)),
            });
            self.prayer_luck_pending = false;
        }

        let killed_with_economy = before.entities.iter().find(|entity| {
            matches!(entity.kind, EntityKind::Monster(_))
                && entity.alive == Some(true)
                && after
                    .entity(entity.id)
                    .is_some_and(|next| next.alive == Some(false))
                && entity
                    .difficulty
                    .is_some_and(|difficulty| after.gold >= before.gold + u32::from(difficulty))
        });
        if let Some(entity) = killed_with_economy.filter(|_| after.kill_count > before.kill_count) {
            let difficulty = entity.difficulty.unwrap_or_default();
            self.record(CausalWitnessRecord {
                witness: CausalWitness::MonsterDifficultyEconomy,
                scenario: CausalScenario::DifficultyEconomy,
                producer: Some(entity.id),
                field: CausalField::MonsterDifficulty,
                source_before: CausalValue::Unsigned(0),
                source_after: CausalValue::Unsigned(u64::from(difficulty)),
                consumer: CausalConsumer::Gold,
                consumer_before: CausalValue::Unsigned(u64::from(before.gold)),
                consumer_after: CausalValue::Unsigned(u64::from(after.gold)),
            });
        }

        let corpse_created = after.entities.iter().any(|entity| {
            entity.kind == EntityKind::Item(ItemKind::CorpseJackal)
                && entity
                    .location
                    .is_some_and(|location| matches!(location, EntityLocation::OnMap { .. }))
                && before.entity(entity.id).is_none()
        });
        self.corpse_produced |= corpse_created;
    }

    fn record(&mut self, record: CausalWitnessRecord) {
        *self.counts.entry(record.witness).or_insert(0) += 1;
        self.records.push(record);
    }
}

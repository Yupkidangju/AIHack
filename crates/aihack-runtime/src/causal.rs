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

use crate::GameSession;

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
                    }
                } else {
                    let (_, _, location, _, _) =
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
                    }
                }
            })
            .collect::<Vec<_>>();
        entities.sort_by_key(|entity| entity.id.0);

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

        self.observe_monster_behavior(before, after);

        match command {
            CommandIntent::Eat { item } => self.observe_eating(before, after, item),
            CommandIntent::Wear { item } => {
                if before.inventory.equipped_body != Some(item)
                    && after.inventory.equipped_body == Some(item)
                    && after.player_ac < before.player_ac
                {
                    self.record(CausalWitness::ArmorDefense);
                }
            }
            CommandIntent::Pray => {
                if after.luck > before.luck && after.prayer_cooldown > before.prayer_cooldown {
                    self.prayer_luck_pending = true;
                }
            }
            CommandIntent::Quit => {
                if before.gold > 0
                    && matches!(
                        after.run_state,
                        RunState::GameOver { final_score, .. }
                            if i64::from(final_score) >= i64::from(before.gold)
                    )
                {
                    self.record(CausalWitness::GoldScore);
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
        match before_item.kind {
            EntityKind::Item(ItemKind::FoodRation) => self.record(CausalWitness::FoodNutrition),
            EntityKind::Item(ItemKind::CorpseJackal) if self.corpse_produced => {
                self.record(CausalWitness::CorpseNutrition)
            }
            _ => {}
        }
    }

    fn observe_monster_behavior(&mut self, before: &CausalProjection, after: &CausalProjection) {
        let moved = before.entities.iter().any(|entity| {
            matches!(entity.kind, EntityKind::Monster(_))
                && entity.alive == Some(true)
                && entity.speed.is_some_and(|speed| speed > 0)
                && entity.ai_kind != Some(MonsterAiKind::Stationary)
                && after
                    .entity(entity.id)
                    .is_some_and(|next| next.pos != entity.pos)
        });
        if moved {
            self.record(CausalWitness::MonsterSpeed);
            self.record(CausalWitness::MonsterAi);
        }
    }

    fn observe_events(
        &mut self,
        before: &CausalProjection,
        after: &CausalProjection,
        outcome: &TurnOutcome,
    ) {
        if outcome.events.iter().any(|event| {
            matches!(
                event,
                GameEvent::PassiveAttackTriggered { source, target }
                    if *target == before.player_id
                        && before.entity(*source).and_then(|entity| entity.passive).is_some()
            )
        }) && after.paralysis_turns > before.paralysis_turns
        {
            self.record(CausalWitness::MonsterPassive);
        }

        if self.prayer_luck_pending
            && outcome.events.iter().any(|event| {
                matches!(
                    event,
                    GameEvent::AttackResolved { attacker, .. } if *attacker == before.player_id
                )
            })
            && before.luck > 0
        {
            self.record(CausalWitness::PrayerLuckCombat);
            self.prayer_luck_pending = false;
        }

        let killed_with_economy = before.entities.iter().any(|entity| {
            matches!(entity.kind, EntityKind::Monster(_))
                && entity.alive == Some(true)
                && after
                    .entity(entity.id)
                    .is_some_and(|next| next.alive == Some(false))
                && entity
                    .difficulty
                    .is_some_and(|difficulty| after.gold >= before.gold + u32::from(difficulty))
        });
        if killed_with_economy && after.kill_count > before.kill_count {
            self.record(CausalWitness::MonsterDifficultyEconomy);
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

    fn record(&mut self, witness: CausalWitness) {
        *self.counts.entry(witness).or_insert(0) += 1;
    }
}

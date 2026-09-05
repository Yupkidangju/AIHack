use serde::{Deserialize, Serialize};

use crate::{
    domain::{entity::EntityStore, tile::TrapKind},
    ids::EntityId,
    rng::GameRng,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DamageRoll {
    pub dice: i16,
    pub sides: i16,
}

impl DamageRoll {
    pub const fn none() -> Self {
        Self { dice: 0, sides: 0 }
    }
    pub const fn new(dice: i16, sides: i16) -> Self {
        Self { dice, sides }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttackProfile {
    #[serde(skip)]
    pub name: &'static str,
    pub hit_bonus: i16,
    pub damage: DamageRoll,
}

impl AttackProfile {
    pub const fn dagger() -> Self {
        Self {
            name: "dagger",
            hit_bonus: 1,
            damage: DamageRoll::new(1, 4),
        }
    }
    pub const fn natural(name: &'static str, damage: DamageRoll) -> Self {
        Self {
            name,
            hit_bonus: 0,
            damage,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeathCause {
    Combat { attacker: EntityId },
    Trap { trap: TrapKind },
}

pub fn roll_die(rng: &mut GameRng, sides: i16) -> i16 {
    assert!(sides > 0, "주사위 면 수는 1 이상이어야 한다");
    (rng.next_u64() % sides as u64) as i16 + 1
}

pub fn roll_damage(
    rng: &mut GameRng,
    damage: DamageRoll,
    damage_bonus: i16,
    damage_reduction: i16,
) -> i16 {
    if damage.dice <= 0 || damage.sides <= 0 {
        return 0;
    }
    let rolled = (0..damage.dice)
        .map(|_| i32::from(roll_die(rng, damage.sides)))
        .sum::<i32>();
    let adjusted = rolled + i32::from(damage_bonus) - i32::from(damage_reduction);
    adjusted.clamp(1, i32::from(i16::MAX)) as i16
}

pub fn attack_roll_value(
    attacker_hit_bonus: i16,
    defender_ac: i16,
    weapon_hit_bonus: i16,
    d20: i16,
) -> (i16, i16, bool) {
    let attack_roll = (i32::from(d20) + i32::from(attacker_hit_bonus) + i32::from(weapon_hit_bonus))
        .clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    let defense =
        (10_i32 + i32::from(defender_ac)).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    (attack_roll, defense, attack_roll >= defense)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackResolution {
    pub attacker: EntityId,
    pub defender: EntityId,
    pub attack_roll: i16,
    pub defense: i16,
    pub hit: bool,
    pub damage: i16,
}

pub fn resolve_attack_with_profile(
    entities: &mut EntityStore,
    rng: &mut GameRng,
    attacker_id: EntityId,
    defender_id: EntityId,
    profile: AttackProfile,
) -> Option<AttackResolution> {
    let attacker = entities.get(attacker_id)?.clone();
    let defender = entities.get(defender_id)?.clone();
    let (_, _, _, _, attacker_stats, attacker_alive) = attacker.actor()?;
    let (_, _, _, _, defender_stats, defender_alive) = defender.actor()?;
    if !attacker_alive || !defender_alive {
        return None;
    }
    let d20 = roll_die(rng, 20);
    let (attack_roll, defense, hit) = attack_roll_value(
        attacker_stats.hit_bonus,
        defender_stats.ac,
        profile.hit_bonus,
        d20,
    );
    let damage = if hit {
        roll_damage(
            rng,
            profile.damage,
            attacker_stats.damage_bonus,
            defender_stats.damage_reduction,
        )
    } else {
        0
    };
    if hit {
        let stats = entities.actor_stats_mut(defender_id)?;
        stats.hp = stats.hp.saturating_sub(damage);
    }
    Some(AttackResolution {
        attacker: attacker_id,
        defender: defender_id,
        attack_roll,
        defense,
        hit,
        damage,
    })
}

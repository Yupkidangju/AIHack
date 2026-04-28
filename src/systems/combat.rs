use crate::{
    core::{event::GameEvent, ids::EntityId, rng::GameRng, world::GameWorld},
    domain::{combat::DamageRoll, entity::Entity, item::UNARMED_ATTACK},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackResolution {
    pub attacker: EntityId,
    pub defender: EntityId,
    pub attack_roll: i16,
    pub defense: i16,
    pub hit: bool,
    pub damage: i16,
}

pub fn resolve_attack(
    world: &mut GameWorld,
    rng: &mut GameRng,
    attacker_id: EntityId,
    defender_id: EntityId,
) -> Option<AttackResolution> {
    let attacker = world.entities.get(attacker_id)?.clone();
    let defender = world.entities.get(defender_id)?.clone();
    let (_, _, _, _, attacker_stats, attacker_alive) = attacker.actor()?;
    let (_, _, _, _, defender_stats, defender_alive) = defender.actor()?;
    if !attacker_alive || !defender_alive {
        return None;
    }

    let profile = attack_profile_for(world, attacker_id, &attacker);
    let d20 = roll_die(rng, 20);
    let attack_roll = d20 + attacker_stats.hit_bonus + profile.hit_bonus;
    let defense = 10 + defender_stats.ac;
    let hit = attack_roll >= defense;
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
        if let Some(defender) = world.entities.actor_stats_mut(defender_id) {
            defender.hp -= damage;
        }
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

pub fn attack_event(resolution: &AttackResolution) -> GameEvent {
    GameEvent::AttackResolved {
        attacker: resolution.attacker,
        defender: resolution.defender,
        attack_roll: resolution.attack_roll,
        defense: resolution.defense,
        hit: resolution.hit,
        damage: resolution.damage,
    }
}

pub fn attack_profile_for(
    world: &GameWorld,
    attacker_id: EntityId,
    attacker: &Entity,
) -> crate::domain::combat::AttackProfile {
    if attacker_id == world.player_id {
        return world
            .inventory
            .equipped_melee
            .and_then(|item| world.entities.item_data(item))
            .and_then(|data| data.attack_profile)
            .unwrap_or(UNARMED_ATTACK);
    }
    attacker.natural_attack_profile().unwrap_or(UNARMED_ATTACK)
}

pub fn attack_roll_value(
    attacker: &Entity,
    defender: &Entity,
    weapon_hit_bonus: i16,
    d20: i16,
) -> (i16, i16, bool) {
    let (_, _, _, _, attacker_stats, _) = attacker.actor().expect("attacker must be actor");
    let (_, _, _, _, defender_stats, _) = defender.actor().expect("defender must be actor");
    let attack_roll = d20 + attacker_stats.hit_bonus + weapon_hit_bonus;
    let defense = 10 + defender_stats.ac;
    (attack_roll, defense, attack_roll >= defense)
}

pub fn roll_damage(
    rng: &mut GameRng,
    damage: DamageRoll,
    damage_bonus: i16,
    damage_reduction: i16,
) -> i16 {
    let rolled = roll_damage_raw(rng, damage);
    if rolled == 0 {
        return 0;
    }
    (rolled + damage_bonus - damage_reduction).max(1)
}

fn roll_damage_raw(rng: &mut GameRng, damage: DamageRoll) -> i16 {
    if damage.dice <= 0 || damage.sides <= 0 {
        return 0;
    }
    (0..damage.dice).map(|_| roll_die(rng, damage.sides)).sum()
}

pub fn roll_die(rng: &mut GameRng, sides: i16) -> i16 {
    assert!(sides > 0, "주사위 면 수는 1 이상이어야 한다");
    (rng.next_u64() % sides as u64) as i16 + 1
}

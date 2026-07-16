use aihack_core::{
    domain::{
        combat::{AttackProfile, DamageRoll},
        entity::Entity,
    },
    event::GameEvent,
    ids::EntityId,
    rng::GameRng,
};

use crate::{domain::item::UNARMED_ATTACK, world::GameWorld};

pub use aihack_core::domain::combat::AttackResolution;

pub fn resolve_attack(
    world: &mut GameWorld,
    rng: &mut GameRng,
    attacker_id: EntityId,
    defender_id: EntityId,
) -> Option<AttackResolution> {
    let attacker = world.entities.get(attacker_id)?.clone();
    let profile = attack_profile_for(world, attacker_id, &attacker);
    resolve_attack_with_profile(world, rng, attacker_id, defender_id, profile)
}

pub fn resolve_attack_with_profile(
    world: &mut GameWorld,
    rng: &mut GameRng,
    attacker_id: EntityId,
    defender_id: EntityId,
    profile: AttackProfile,
) -> Option<AttackResolution> {
    aihack_core::domain::combat::resolve_attack_with_profile(
        &mut world.entities,
        rng,
        attacker_id,
        defender_id,
        profile,
    )
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
) -> AttackProfile {
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
    aihack_core::domain::combat::attack_roll_value(
        attacker_stats.hit_bonus,
        defender_stats.ac,
        weapon_hit_bonus,
        d20,
    )
}

pub fn roll_damage(
    rng: &mut GameRng,
    damage: DamageRoll,
    damage_bonus: i16,
    damage_reduction: i16,
) -> i16 {
    aihack_core::domain::combat::roll_damage(rng, damage, damage_bonus, damage_reduction)
}

pub fn roll_die(rng: &mut GameRng, sides: i16) -> i16 {
    aihack_core::domain::combat::roll_die(rng, sides)
}

use aihack_core::{
    domain::{combat::AttackProfile, entity::Entity},
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
    let mut profile = attack_profile_for(world, attacker_id, &attacker);
    if attacker_id == world.player_id {
        profile.hit_bonus = profile.hit_bonus.saturating_add(world.luck);
    }
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
        world.state_mut().entities.inner_mut(),
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

pub fn roll_die(rng: &mut GameRng, sides: i16) -> i16 {
    aihack_core::domain::combat::roll_die(rng, sides)
}

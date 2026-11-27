use aihack_core::{
    domain::{combat::AttackProfile, entity::Entity, item::BucStatus, status::HungerState},
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
    let mut extra_damage_bonus = 0_i16;
    if attacker_id == world.player_id {
        profile.hit_bonus = profile.hit_bonus.saturating_add(world.luck);
        if let Some(campaign) = world.campaign {
            profile.hit_bonus = profile
                .hit_bonus
                .saturating_add(campaign.attributes.dex_hit_bonus());
            extra_damage_bonus =
                extra_damage_bonus.saturating_add(campaign.attributes.melee_damage_bonus());
        }
        if matches!(
            world.hunger_state(),
            HungerState::Weak | HungerState::Fainting
        ) {
            profile.hit_bonus = profile.hit_bonus.saturating_sub(2);
        }
        if let Some(weapon_id) = world.inventory.equipped_melee {
            if let Some(weapon) = world.entities.get(weapon_id) {
                let enchant = i16::from(weapon.item_enchantment().unwrap_or(0));
                let erosion = i16::from(weapon.item_erosion().unwrap_or(0));
                let buc = weapon.item_buc().unwrap_or(BucStatus::Uncursed);
                let bless_hit = if buc == BucStatus::Blessed { 1 } else { 0 };
                profile.hit_bonus = profile
                    .hit_bonus
                    .saturating_add(enchant)
                    .saturating_add(bless_hit);
                extra_damage_bonus = extra_damage_bonus
                    .saturating_add(enchant)
                    .saturating_sub(erosion);
            }
        }
    }
    resolve_attack_with_profile_and_bonus(
        world,
        rng,
        attacker_id,
        defender_id,
        profile,
        extra_damage_bonus,
    )
}

pub fn resolve_attack_with_profile(
    world: &mut GameWorld,
    rng: &mut GameRng,
    attacker_id: EntityId,
    defender_id: EntityId,
    profile: AttackProfile,
) -> Option<AttackResolution> {
    resolve_attack_with_profile_and_bonus(world, rng, attacker_id, defender_id, profile, 0)
}

pub fn resolve_attack_with_profile_and_bonus(
    world: &mut GameWorld,
    rng: &mut GameRng,
    attacker_id: EntityId,
    defender_id: EntityId,
    profile: AttackProfile,
    extra_damage_bonus: i16,
) -> Option<AttackResolution> {
    aihack_core::domain::combat::resolve_attack_with_profile_and_bonus(
        world.state_mut().entities.inner_mut(),
        rng,
        attacker_id,
        defender_id,
        profile,
        extra_damage_bonus,
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

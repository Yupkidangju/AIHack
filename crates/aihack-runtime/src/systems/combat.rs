use aihack_core::{
    campaign::SkillCategory,
    domain::{
        combat::AttackProfile,
        entity::{ActorKind, Entity},
        item::BucStatus,
        monster::MonsterKind,
        status::HungerState,
    },
    event::GameEvent,
    ids::EntityId,
    rng::GameRng,
};

use crate::{
    domain::item::UNARMED_ATTACK,
    systems::{artifacts, skills},
    world::GameWorld,
};

pub use aihack_core::domain::combat::AttackResolution;

pub fn resolve_attack(
    world: &mut GameWorld,
    rng: &mut GameRng,
    attacker_id: EntityId,
    defender_id: EntityId,
) -> Option<AttackResolution> {
    let attacker = world.entities.get(attacker_id)?.clone();
    let defender_monster = world
        .entities
        .get(defender_id)
        .and_then(|e| match e.actor() {
            Some((ActorKind::Monster(mk), _, _, _, _, alive)) if alive => Some(mk),
            _ => None,
        });
    let mut profile = attack_profile_for(world, attacker_id, &attacker);
    let mut extra_damage_bonus = 0_i16;
    let mut active_skill: Option<SkillCategory> = None;
    let mut active_drains_life = false;
    if attacker_id == world.player_id {
        profile.hit_bonus = profile.hit_bonus.saturating_add(world.luck);
        if let Some(campaign) = world.campaign {
            profile.hit_bonus = profile
                .hit_bonus
                .saturating_add(campaign.attributes.dex_hit_bonus());
            extra_damage_bonus =
                extra_damage_bonus.saturating_add(campaign.attributes.melee_damage_bonus());
            let cat = skills::melee_skill_category(world);
            active_skill = Some(cat);
            let rank = skills::current_rank(world, cat);
            if cat == SkillCategory::BareHands {
                let (u_hit, u_dmg) = campaign.role.unarmed_bonus(campaign.level(), rank);
                profile.hit_bonus = profile.hit_bonus.saturating_add(u_hit);
                extra_damage_bonus = extra_damage_bonus.saturating_add(u_dmg);
            } else {
                profile.hit_bonus = profile.hit_bonus.saturating_add(rank.hit_bonus());
                extra_damage_bonus = extra_damage_bonus.saturating_add(rank.damage_bonus());
            }
        }
        if matches!(
            world.hunger_state(),
            HungerState::Weak | HungerState::Fainting
        ) {
            profile.hit_bonus = profile.hit_bonus.saturating_sub(2);
        }
        if world.campaign.is_some()
            && world
                .entities
                .get(defender_id)
                .and_then(|e| e.actor_invisible())
                == Some(true)
            && !world.has_see_invisible()
        {
            profile.hit_bonus = profile.hit_bonus.saturating_sub(4);
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
            if let Some(def) = artifacts::artifact_def_for_item(world, weapon_id) {
                profile.hit_bonus = profile.hit_bonus.saturating_add(def.hit_bonus);
                extra_damage_bonus = extra_damage_bonus.saturating_add(def.damage_bonus);
                if def.id == "sting"
                    && matches!(
                        defender_monster,
                        Some(
                            MonsterKind::Orc
                                | MonsterKind::OrcCaptain
                                | MonsterKind::Kobold
                                | MonsterKind::Ogre
                        )
                    )
                {
                    extra_damage_bonus = extra_damage_bonus.saturating_add(3);
                }
                active_drains_life = def.drains_life;
            }
            if world.campaign.is_some() {
                if let Some(def_mk) = defender_monster {
                    if let Some(weapon_kind) = world.entities.item_data(weapon_id).map(|d| d.kind) {
                        let reg = world.content_registry();
                        let is_large = aihack_content::monster_is_large(def_mk, reg);
                        if let Some(sized_dmg) =
                            aihack_content::weapon_damage_for_size(weapon_kind, is_large, reg)
                        {
                            profile.damage = sized_dmg;
                        }
                        if aihack_content::item_material(weapon_kind, reg) == Some("silver")
                            && (aihack_content::monster_has_ai_flag(def_mk, "undead", reg)
                                || aihack_content::monster_has_ai_flag(def_mk, "demon", reg))
                        {
                            extra_damage_bonus =
                                extra_damage_bonus.saturating_add(roll_die(rng, 10));
                        }
                    }
                }
            }
        }
    } else if world.campaign.is_some() {
        if let Some(best_weapon_id) = monster_best_carried_weapon(world, &attacker) {
            if let Some(weapon) = world.entities.get(best_weapon_id) {
                if let Some(weapon_profile) =
                    weapon.item().and_then(|(_, d, _, _, _)| d.attack_profile)
                {
                    let natural_max = profile.damage.dice.saturating_mul(profile.damage.sides);
                    let weapon_max = weapon_profile
                        .damage
                        .dice
                        .saturating_mul(weapon_profile.damage.sides);
                    if weapon_max >= natural_max {
                        profile = weapon_profile;
                    }
                    let enchant = i16::from(weapon.item_enchantment().unwrap_or(0));
                    let erosion = i16::from(weapon.item_erosion().unwrap_or(0));
                    profile.hit_bonus = profile.hit_bonus.saturating_add(enchant);
                    extra_damage_bonus = extra_damage_bonus
                        .saturating_add(enchant)
                        .saturating_add(1)
                        .saturating_sub(erosion);
                }
            }
        }
    }
    let resolution = resolve_attack_with_profile_and_bonus(
        world,
        rng,
        attacker_id,
        defender_id,
        profile,
        extra_damage_bonus,
    );
    if attacker_id == world.player_id {
        if let Some(cat) = active_skill {
            let practice = if resolution.as_ref().is_some_and(|res| res.hit) {
                5
            } else {
                3
            };
            skills::record_practice(world, cat, practice);
        }
        if active_drains_life && resolution.as_ref().is_some_and(|res| res.hit) {
            if let Some(stats) = world.state_mut().entities.actor_stats_mut(attacker_id) {
                stats.hp = stats.hp.saturating_add(2).min(stats.max_hp);
            }
        }
    }
    resolution
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

fn monster_best_carried_weapon(world: &GameWorld, attacker: &Entity) -> Option<EntityId> {
    let inv = attacker.actor_monster_inventory()?;
    inv.iter()
        .copied()
        .filter_map(|item_id| {
            let entity = world.entities.get(item_id)?;
            let (_, data, _, _, _) = entity.item()?;
            if data.class != aihack_core::domain::item::ItemClass::Weapon {
                return None;
            }
            let profile = data.attack_profile?;
            let score = i32::from(profile.damage.dice)
                .saturating_mul(i32::from(profile.damage.sides))
                .saturating_add(i32::from(entity.item_enchantment().unwrap_or(0)))
                .saturating_sub(i32::from(entity.item_erosion().unwrap_or(0)));
            Some((item_id, score))
        })
        .max_by_key(|&(_, score)| score)
        .map(|(id, _)| id)
}

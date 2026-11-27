use aihack_core::{
    campaign::SkillCategory,
    domain::{
        combat::{AttackProfile, DamageRoll},
        entity::EntityLocation,
        item::{BucStatus, ItemKind, WandEffect},
        tile::{DoorState, TileKind},
    },
    event::GameEvent,
    ids::EntityId,
    position::{Direction, Pos},
    rng::GameRng,
};

use crate::{
    systems::{combat, death, items, skills},
    world::GameWorld,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectileOutcome {
    pub landing: Pos,
    pub hit_target: Option<EntityId>,
}

pub fn throw_item(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
    direction: Direction,
) -> Result<Vec<GameEvent>, String> {
    let Some((kind, data, _, _, _)) = world.entities.get(item).and_then(|entity| entity.item())
    else {
        return Err("entity is not an item".to_string());
    };
    let data = *data;
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    if !world
        .entities
        .get(item)
        .and_then(|e| e.item_contained_items())
        .is_none_or(<[EntityId]>::is_empty)
    {
        return Err("empty container before throwing".to_string());
    }
    let is_campaign_food_throw = world.campaign.is_some()
        && matches!(
            data.class,
            aihack_core::domain::item::ItemClass::Food
                | aihack_core::domain::item::ItemClass::Corpse
        );
    if data.attack_profile.is_none() && !is_campaign_food_throw {
        return Err("item cannot be thrown".to_string());
    }

    let from = world.player_pos();
    let outcome = trace_path(world, from, direction);
    let level = world.current_level();
    let (qty, buc, enchantment, buc_known, enchant_known) = world
        .entities
        .get(item)
        .map(|entity| {
            (
                entity.item_quantity().unwrap_or(1),
                entity.item_buc().unwrap_or(BucStatus::Uncursed),
                entity.item_enchantment().unwrap_or(0),
                entity.item_buc_known().unwrap_or(false),
                entity.item_enchant_known().unwrap_or(false),
            )
        })
        .unwrap_or((1, BucStatus::Uncursed, 0, false, false));
    let thrown_id = if qty > 1 {
        let spawned = world
            .state_mut()
            .entities
            .spawn_item_with_data(
                kind,
                data,
                EntityLocation::OnMap {
                    level,
                    pos: outcome.landing,
                },
            )
            .map_err(|error| error.to_string())?;
        let entities = &mut world.state_mut().entities;
        entities.set_item_buc(spawned, buc);
        entities.set_item_enchantment(spawned, enchantment);
        entities.set_item_buc_known(spawned, buc_known);
        entities.set_item_enchant_known(spawned, enchant_known);
        entities.set_item_quantity(item, qty - 1);
        spawned
    } else {
        items::remove_inventory_item(world, item)?;
        world.state_mut().entities.set_item_location(
            item,
            EntityLocation::OnMap {
                level,
                pos: outcome.landing,
            },
        );
        item
    };

    let mut events = vec![GameEvent::ItemThrown {
        entity: world.player_id,
        item: thrown_id,
        from,
        to: outcome.landing,
    }];
    if let Some(target) = outcome.hit_target {
        if let Some(feed_events) =
            crate::systems::pets::try_feed_or_tame_by_throw(world, target, thrown_id)
        {
            events.extend(feed_events);
            return Ok(events);
        }
        events.extend(crate::systems::npc::provoke_actor(world, target));
        let mut profile = projectile_profile(kind, data.attack_profile);
        let mut extra_bonus = 0_i16;
        if world.campaign.is_some() {
            profile.hit_bonus = profile.hit_bonus.saturating_add(world.luck);
            if let Some(campaign) = world.campaign {
                profile.hit_bonus = profile
                    .hit_bonus
                    .saturating_add(campaign.attributes.dex_hit_bonus());
            }
            if let Some(thrown_entity) = world.entities.get(thrown_id) {
                let enchant = i16::from(thrown_entity.item_enchantment().unwrap_or(0));
                let erosion = i16::from(thrown_entity.item_erosion().unwrap_or(0));
                let bless_hit = if thrown_entity.item_buc()
                    == Some(aihack_core::domain::item::BucStatus::Blessed)
                {
                    1
                } else {
                    0
                };
                profile.hit_bonus = profile
                    .hit_bonus
                    .saturating_add(enchant)
                    .saturating_add(bless_hit);
                extra_bonus = extra_bonus.saturating_add(enchant).saturating_sub(erosion);
            }
            if let Some(def) = crate::systems::artifacts::artifact_def_for_item(world, thrown_id) {
                profile.hit_bonus = profile.hit_bonus.saturating_add(def.hit_bonus);
                extra_bonus = extra_bonus.saturating_add(def.damage_bonus);
                if def.id == "sting"
                    && matches!(
                        world.entities.get(target).and_then(|e| e.actor_kind()),
                        Some(aihack_core::domain::entity::ActorKind::Monster(
                            aihack_core::domain::monster::MonsterKind::Orc
                                | aihack_core::domain::monster::MonsterKind::Kobold
                                | aihack_core::domain::monster::MonsterKind::Ogre
                        ))
                    )
                {
                    extra_bonus = extra_bonus.saturating_add(3);
                }
            }
        }
        if let Some(cat) = SkillCategory::for_weapon_kind(kind) {
            if world.campaign.is_some() {
                let rank = skills::current_rank(world, cat);
                profile.hit_bonus = profile.hit_bonus.saturating_add(rank.hit_bonus());
                extra_bonus = extra_bonus.saturating_add(rank.damage_bonus());
                skills::record_practice(world, cat, 4);
            }
        }
        if let Some(resolution) = combat::resolve_attack_with_profile_and_bonus(
            world,
            rng,
            world.player_id,
            target,
            profile,
            extra_bonus,
        ) {
            events.push(combat::attack_event(&resolution));
            events.extend(death::collect_death_events_after_attack(
                world,
                world.player_id,
                target,
            )?);
        }
    } else if world.campaign.is_some() {
        if let Some(cat) = SkillCategory::for_weapon_kind(kind) {
            skills::record_practice(world, cat, 2);
        }
    }
    Ok(events)
}

pub fn zap_wand(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
    direction: Direction,
) -> Result<Vec<GameEvent>, String> {
    let Some((_, data, _, _, charges)) = world.entities.get(item).and_then(|entity| entity.item())
    else {
        return Err("entity is not an item".to_string());
    };
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let Some(wand_effect) = data.wand_effect else {
        return Err("item is not a wand".to_string());
    };
    let Some(charges_before) = charges else {
        return Err("wand has no charge state".to_string());
    };
    if charges_before == 0 {
        return Err("wand has no charges".to_string());
    }

    let charges_after = charges_before - 1;
    world
        .state_mut()
        .entities
        .set_item_charges(item, Some(charges_after));
    let outcome = trace_path(world, world.player_pos(), direction);
    let mut events = vec![GameEvent::WandZapped {
        entity: world.player_id,
        item,
        direction,
        charges_after,
    }];
    if let Some(target) = outcome.hit_target {
        events.extend(crate::systems::npc::provoke_actor(world, target));
        if wand_effect == WandEffect::Fire
            && world.entities.actor_stats(target).is_some_and(|stats| {
                stats
                    .resistances
                    .contains(aihack_core::domain::status::Resistances::FIRE)
            })
        {
            events.push(aihack_core::event::GameEvent::Message {
                priority: aihack_core::event::MessagePriority::Info,
                text: "The fire bolt has no effect on the fire-resistant creature!".to_string(),
            });
            return Ok(events);
        }
        let (profile, extra_damage_bonus) = match wand_effect {
            WandEffect::MagicMissile => (
                AttackProfile::natural("magic missile", DamageRoll::new(1, 6)),
                0,
            ),
            WandEffect::Fire => (
                AttackProfile::natural("fire bolt", DamageRoll::new(2, 6)),
                2,
            ),
        };
        if let Some(resolution) = combat::resolve_attack_with_profile_and_bonus(
            world,
            rng,
            world.player_id,
            target,
            profile,
            extra_damage_bonus,
        ) {
            events.push(combat::attack_event(&resolution));
            events.extend(death::collect_death_events_after_attack(
                world,
                world.player_id,
                target,
            )?);
        }
    }
    Ok(events)
}

fn projectile_profile(kind: ItemKind, base: Option<AttackProfile>) -> AttackProfile {
    match kind {
        ItemKind::Rock => AttackProfile::natural("rock", DamageRoll::new(1, 3)),
        _ => base.unwrap_or(AttackProfile::natural("thrown item", DamageRoll::new(1, 2))),
    }
}

fn trace_path(world: &GameWorld, from: Pos, direction: Direction) -> ProjectileOutcome {
    trace_path_with_max_distance(world, from, direction, i16::MAX)
}

pub(crate) fn trace_path_with_max_distance(
    world: &GameWorld,
    from: Pos,
    direction: Direction,
    max_distance: i16,
) -> ProjectileOutcome {
    let mut current = from;
    let mut steps = 0_i16;
    loop {
        if steps >= max_distance {
            return ProjectileOutcome {
                landing: current,
                hit_target: None,
            };
        }
        steps += 1;
        let next = current.offset(direction.delta());
        if !world.current_map().contains(next) {
            return ProjectileOutcome {
                landing: current,
                hit_target: None,
            };
        }
        if let Some(target) = world.entities.alive_actor_at(world.current_level(), next) {
            if target != world.player_id {
                return ProjectileOutcome {
                    landing: next,
                    hit_target: Some(target),
                };
            }
        }
        let Ok(tile) = world.current_map().tile(next) else {
            return ProjectileOutcome {
                landing: current,
                hit_target: None,
            };
        };
        if matches!(
            tile,
            TileKind::Wall
                | TileKind::Door(DoorState::Closed | DoorState::Locked)
                | TileKind::HiddenDoor
        ) {
            return ProjectileOutcome {
                landing: current,
                hit_target: None,
            };
        }
        current = next;
    }
}

#[cfg(test)]
mod tests {
    use aihack_core::{
        domain::{
            combat::{AttackProfile, DamageRoll},
            entity::EntityLocation,
            item::ItemKind,
        },
        position::{Direction, Pos},
        rng::GameRng,
    };

    use super::throw_item;
    use crate::{domain::item::item_data, systems::items, world::GameWorld};

    #[test]
    fn throwing_an_equipped_body_item_uses_the_common_unequip_lifecycle() {
        let mut world = GameWorld::fixture_without_monsters();
        let owner = world.player_id();
        let mut armor_data = item_data(ItemKind::ArmorLeather);
        armor_data.attack_profile = Some(AttackProfile::natural("fixture", DamageRoll::new(1, 4)));
        let armor = world
            .state_mut()
            .entities
            .spawn_item_with_data(
                ItemKind::ArmorLeather,
                armor_data,
                EntityLocation::Inventory { owner },
            )
            .unwrap();
        let letter = world
            .state_mut()
            .inventory
            .add_existing_with_next_letter(armor)
            .unwrap();
        assert!(world.state_mut().entities.set_item_letter(armor, letter));
        assert!(items::wear(&mut world, armor).unwrap().is_some());
        assert_eq!(
            world.entities.actor_stats(owner).unwrap().ac,
            10 - armor_data.ac_bonus
        );

        let mut rng = GameRng::new(42);
        throw_item(&mut world, &mut rng, armor, Direction::East).unwrap();

        assert_eq!(world.inventory.equipped_body, None);
        assert_eq!(world.entities.actor_stats(owner).unwrap().ac, 10);
        assert!(!world.inventory.contains(armor));
        assert_eq!(
            world.entities.item_location(armor),
            Some(EntityLocation::OnMap {
                level: world.current_level(),
                pos: Pos { x: 9, y: 5 },
            })
        );
    }
}

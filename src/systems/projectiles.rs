use crate::{
    core::{
        event::GameEvent,
        ids::EntityId,
        position::{Direction, Pos},
        rng::GameRng,
        world::GameWorld,
    },
    domain::{
        combat::{AttackProfile, DamageRoll},
        entity::EntityLocation,
        item::{ItemKind, WandEffect},
        tile::{DoorState, TileKind},
    },
    systems::{combat, death},
};

/// [v0.1.0] Phase 7 투척/zap 공용 경로 추적 결과다.
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
    if data.attack_profile.is_none() {
        return Err("item cannot be thrown in phase 7".to_string());
    }

    let from = world.player_pos();
    let outcome = trace_path(world, from, direction);
    world.inventory.remove(item);
    world.entities.set_item_location(
        item,
        EntityLocation::OnMap {
            level: world.current_level(),
            pos: outcome.landing,
        },
    );

    let mut events = vec![GameEvent::ItemThrown {
        entity: world.player_id,
        item,
        from,
        to: outcome.landing,
    }];
    if let Some(target) = outcome.hit_target {
        let profile = projectile_profile(kind, data.attack_profile);
        if let Some(resolution) =
            combat::resolve_attack_with_profile(world, rng, world.player_id, target, profile)
        {
            events.push(combat::attack_event(&resolution));
            events.extend(death::collect_death_events_after_attack(
                world,
                world.player_id,
                target,
            ));
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
    if data.wand_effect != Some(WandEffect::MagicMissile) {
        return Err("item is not a wand".to_string());
    }
    let Some(charges_before) = charges else {
        return Err("wand has no charge state".to_string());
    };
    if charges_before == 0 {
        return Err("wand has no charges".to_string());
    }

    let charges_after = charges_before - 1;
    world.entities.set_item_charges(item, Some(charges_after));
    let outcome = trace_path(world, world.player_pos(), direction);
    let mut events = vec![GameEvent::WandZapped {
        entity: world.player_id,
        item,
        direction,
        charges_after,
    }];
    if let Some(target) = outcome.hit_target {
        let profile = AttackProfile::natural("magic missile", DamageRoll::new(1, 6));
        if let Some(resolution) =
            combat::resolve_attack_with_profile(world, rng, world.player_id, target, profile)
        {
            events.push(combat::attack_event(&resolution));
            events.extend(death::collect_death_events_after_attack(
                world,
                world.player_id,
                target,
            ));
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
    let mut current = from;
    loop {
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
            TileKind::Wall | TileKind::Door(DoorState::Closed) | TileKind::HiddenDoor
        ) {
            return ProjectileOutcome {
                landing: current,
                hit_target: None,
            };
        }
        current = next;
    }
}

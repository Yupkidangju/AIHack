use aihack_core::{
    domain::{
        combat::DeathCause,
        entity::{EntityLocation, EntityStore},
        item::{BucStatus, ItemKind},
        tile::TrapKind,
    },
    event::{GameEvent, MessagePriority},
    ids::EntityId,
    rng::GameRng,
};

use crate::{
    systems::{combat::roll_die, death, items},
    world::GameWorld,
};

pub const MAX_CONTAINER_DEPTH: usize = 4;

pub fn recursive_item_weight(entities: &EntityStore, item_id: EntityId, depth: usize) -> i16 {
    let Some(entity) = entities.get(item_id) else {
        return 0;
    };
    let Some((kind, data, _, _, _)) = entity.item() else {
        return 0;
    };
    let qty = i32::from(entity.item_quantity().unwrap_or(1));
    let own = i32::from(data.weight).saturating_mul(qty);
    let mut children_weight = 0_i32;
    if depth < MAX_CONTAINER_DEPTH {
        if let Some(contained) = entity.item_contained_items() {
            for &child in contained {
                children_weight = children_weight.saturating_add(i32::from(recursive_item_weight(
                    entities,
                    child,
                    depth + 1,
                )));
            }
        }
    }
    let adjusted_children = if kind == ItemKind::BagOfHolding {
        match entity.item_buc().unwrap_or(BucStatus::Uncursed) {
            BucStatus::Blessed => children_weight / 4,
            BucStatus::Uncursed => children_weight / 2,
            BucStatus::Cursed => children_weight.saturating_mul(2),
        }
    } else {
        children_weight
    };
    own.saturating_add(adjusted_children)
        .clamp(0, i32::from(i16::MAX)) as i16
}

pub fn raw_contained_weight(entities: &EntityStore, container_id: EntityId, depth: usize) -> i16 {
    if depth >= MAX_CONTAINER_DEPTH {
        return 0;
    }
    let Some(entity) = entities.get(container_id) else {
        return 0;
    };
    let Some(contained) = entity.item_contained_items() else {
        return 0;
    };
    contained
        .iter()
        .map(|&child| recursive_raw_weight(entities, child, depth + 1))
        .fold(0_i32, |acc, w| acc.saturating_add(i32::from(w)))
        .clamp(0, i32::from(i16::MAX)) as i16
}

fn recursive_raw_weight(entities: &EntityStore, item_id: EntityId, depth: usize) -> i16 {
    let Some(entity) = entities.get(item_id) else {
        return 0;
    };
    let Some((_, data, _, _, _)) = entity.item() else {
        return 0;
    };
    let qty = i32::from(entity.item_quantity().unwrap_or(1));
    let own = i32::from(data.weight).saturating_mul(qty);
    let children = if depth < MAX_CONTAINER_DEPTH {
        raw_contained_weight(entities, item_id, depth)
    } else {
        0
    };
    own.saturating_add(i32::from(children))
        .clamp(0, i32::from(i16::MAX)) as i16
}

pub fn recursive_item_value(
    entities: &EntityStore,
    item_id: EntityId,
    depth: usize,
) -> Result<i128, ()> {
    if depth > MAX_CONTAINER_DEPTH {
        return Err(());
    }
    let entity = entities.get(item_id).ok_or(())?;
    let (_, data, _, _, _) = entity.item().ok_or(())?;
    let qty = i128::from(entity.item_quantity().unwrap_or(1));
    let mut total = i128::from(data.base_price).checked_mul(qty).ok_or(())?;
    if let Some(children) = entity.item_contained_items() {
        for &child in children {
            let child_val = recursive_item_value(entities, child, depth + 1)?;
            total = total.checked_add(child_val).ok_or(())?;
        }
    }
    Ok(total)
}

pub fn is_accessible_container(world: &GameWorld, container_id: EntityId) -> bool {
    let Some(entity) = world.entities.get(container_id) else {
        return false;
    };
    let Some((_, data, location, _, _)) = entity.item() else {
        return false;
    };
    if data.container_capacity.is_none() {
        return false;
    }
    world.inventory.contains(container_id)
        || location
            == (EntityLocation::OnMap {
                level: world.current_level(),
                pos: world.player_pos(),
            })
}

fn contains_kind_or_is_kind(
    entities: &EntityStore,
    id: EntityId,
    target: ItemKind,
    depth: usize,
) -> bool {
    if depth > MAX_CONTAINER_DEPTH {
        return false;
    }
    let Some(entity) = entities.get(id) else {
        return false;
    };
    if entity.item().is_some_and(|(k, _, _, _, _)| k == target) {
        return true;
    }
    entity.item_contained_items().is_some_and(|list| {
        list.iter()
            .any(|&c| contains_kind_or_is_kind(entities, c, target, depth + 1))
    })
}

fn is_inside_bag_of_holding(entities: &EntityStore, id: EntityId, depth: usize) -> bool {
    if depth > MAX_CONTAINER_DEPTH {
        return false;
    }
    let Some(entity) = entities.get(id) else {
        return false;
    };
    let Some((kind, _, location, _, _)) = entity.item() else {
        return false;
    };
    if depth > 0 && kind == ItemKind::BagOfHolding {
        return true;
    }
    if let EntityLocation::InContainer { container } = location {
        return is_inside_bag_of_holding(entities, container, depth + 1);
    }
    false
}

fn collect_recursive_items(
    entities: &EntityStore,
    root: EntityId,
    depth: usize,
    out: &mut Vec<EntityId>,
) {
    if depth > MAX_CONTAINER_DEPTH || out.contains(&root) {
        return;
    }
    out.push(root);
    if let Some(children) = entities.get(root).and_then(|e| e.item_contained_items()) {
        for &child in children {
            collect_recursive_items(entities, child, depth + 1, out);
        }
    }
}

pub fn container_put(
    world: &mut GameWorld,
    rng: &mut GameRng,
    container: EntityId,
    item: EntityId,
) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("containers are only available in campaign mode".to_string());
    }
    if container == item {
        return Err("cannot put a container inside itself".to_string());
    }
    if !is_accessible_container(world, container) {
        return Err("container is not accessible here".to_string());
    }
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    if world.inventory.slot_of(item).is_some() {
        return Err("cannot put an equipped item into a container".to_string());
    }
    let item_entity = world
        .entities
        .get(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    let (item_kind, _, _, _, _) = item_entity
        .item()
        .ok_or_else(|| "entity is not an item".to_string())?;
    if item_kind == ItemKind::AmuletAscension {
        return Err("The Amulet of Ascension resists being placed inside a container.".to_string());
    }
    if item_kind == ItemKind::Blindfold
        && item_entity.item_tool_active() == Some(true)
        && item_entity.item_buc() == Some(BucStatus::Cursed)
    {
        world.state_mut().entities.set_item_buc_known(item, true);
        return Err("The blindfold is cursed and stuck to your face!".to_string());
    }

    let container_entity = world
        .entities
        .get(container)
        .ok_or_else(|| "container entity is missing".to_string())?;
    let (container_kind, container_data, _, _, _) = container_entity
        .item()
        .ok_or_else(|| "entity is not a container".to_string())?;
    if container_entity.item_container_locked() == Some(true) {
        return Err("The container is locked.".to_string());
    }
    let capacity = container_data
        .container_capacity
        .ok_or_else(|| "item is not a container".to_string())?;

    // 자기 참조 또는 깊이 초과 방지
    if is_ancestor(&world.entities, item, container, 0) {
        return Err("cannot create a container cycle".to_string());
    }
    let container_depth = ancestor_depth(&world.entities, container);
    let subtree_depth = max_subtree_depth(&world.entities, item, 0);
    if container_depth + 1 + subtree_depth > MAX_CONTAINER_DEPTH {
        return Err("container nesting depth exceeded".to_string());
    }

    // Bag of Holding 중첩 폭발
    let container_has_boh = container_kind == ItemKind::BagOfHolding
        || is_inside_bag_of_holding(&world.entities, container, 0);
    let item_has_boh = contains_kind_or_is_kind(&world.entities, item, ItemKind::BagOfHolding, 0);
    if container_has_boh && item_has_boh {
        let mut destroyed = Vec::new();
        collect_recursive_items(&world.entities, container, 0, &mut destroyed);
        collect_recursive_items(&world.entities, item, 0, &mut destroyed);
        let _ = world.state_mut().inventory.remove(item);
        let _ = world.state_mut().inventory.remove(container);
        for id in destroyed {
            let entities = &mut world.state_mut().entities;
            entities.clear_contained_items(id);
            entities.clear_item_letter(id);
            entities.set_item_tool_active(id, false);
            entities.set_item_location(id, EntityLocation::Consumed);
        }
        let dmg = roll_die(rng, 8) + roll_die(rng, 8);
        let player_id = world.player_id;
        if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
            stats.hp = stats.hp.saturating_sub(dmg);
        }
        let mut events = vec![GameEvent::Message {
            priority: MessagePriority::Danger,
            text: format!("A magical explosion destroys the bag of holding for {dmg} damage!"),
        }];
        events.extend(death::collect_death_events_if_hp_depleted(
            world,
            player_id,
            DeathCause::Explosion,
        )?);
        return Ok(events);
    }

    let current_used = raw_contained_weight(&world.entities, container, 0);
    let added = recursive_raw_weight(&world.entities, item, 0);
    if i32::from(current_used).saturating_add(i32::from(added)) > i32::from(capacity) {
        return Err("container does not have enough capacity".to_string());
    }

    if matches!(item_kind, ItemKind::Blindfold | ItemKind::OilLamp) {
        world.state_mut().entities.set_item_tool_active(item, false);
    }
    items::remove_inventory_item(world, item)?;
    let entities = &mut world.state_mut().entities;
    entities.clear_item_letter(item);
    entities.set_item_location(item, EntityLocation::InContainer { container });
    entities.push_contained_item(container, item);

    let item_name = aihack_content::item_display_name(item_kind, world.content_registry());
    let container_name =
        aihack_content::item_display_name(container_kind, world.content_registry());
    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: format!("You put the {item_name} into the {container_name}."),
    }])
}

pub fn container_take(
    world: &mut GameWorld,
    rng: &mut GameRng,
    container: EntityId,
    item: EntityId,
) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("containers are only available in campaign mode".to_string());
    }
    if !is_accessible_container(world, container) {
        return Err("container is not accessible here".to_string());
    }
    let container_entity = world
        .entities
        .get(container)
        .ok_or_else(|| "container entity is missing".to_string())?;
    let (container_kind, _, _, _, _) = container_entity
        .item()
        .ok_or_else(|| "entity is not a container".to_string())?;
    if container_entity.item_container_locked() == Some(true) {
        return Err("The container is locked.".to_string());
    }
    let mut events = Vec::new();
    if container_entity.item_container_trapped() == Some(true) {
        world
            .state_mut()
            .entities
            .set_item_container_trapped(container, false);
        let dmg = roll_die(rng, 4) + roll_die(rng, 4);
        let player_id = world.player_id;
        if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
            stats.hp = stats.hp.saturating_sub(dmg);
        }
        if !items::has_poison_resistance(world) {
            let next_pois = world.poison_turns.max(3);
            world.state_mut().poison_turns = next_pois;
        }
        events.push(GameEvent::Message {
            priority: MessagePriority::Danger,
            text: format!("A needle trap springs from the chest for {dmg} damage!"),
        });
        events.extend(death::collect_death_events_if_hp_depleted(
            world,
            player_id,
            DeathCause::Trap {
                trap: TrapKind::Dart,
            },
        )?);
        if !world.player_alive() {
            return Ok(events);
        }
    }

    let contained = world
        .entities
        .get(container)
        .and_then(|e| e.item_contained_items())
        .ok_or_else(|| "container has no contents".to_string())?;
    if !contained.contains(&item) {
        return Err("item is not inside that container".to_string());
    }

    let child = world
        .entities
        .get(item)
        .ok_or_else(|| "contained item is missing".to_string())?;
    let (item_kind, _, _, _, _) = child
        .item()
        .ok_or_else(|| "contained entity is not an item".to_string())?;
    let child_buc = child.item_buc().unwrap_or(BucStatus::Uncursed);
    let child_enchant = child.item_enchantment().unwrap_or(0);
    let child_qty = child.item_quantity().unwrap_or(1);
    let child_erosion = child.item_erosion().unwrap_or(0);

    let stack_target = if item_kind.is_stackable() && child_erosion == 0 {
        world.inventory.entries.iter().find_map(|entry| {
            let existing = world.entities.get(entry.item)?;
            let (ek, _, _, _, _) = existing.item()?;
            let eq = existing.item_quantity().unwrap_or(1);
            if ek == item_kind
                && existing.item_buc().unwrap_or(BucStatus::Uncursed) == child_buc
                && existing.item_enchantment().unwrap_or(0) == child_enchant
                && existing.item_erosion().unwrap_or(0) == 0
                && eq.saturating_add(child_qty) <= aihack_core::domain::item::MAX_ITEM_STACK
            {
                Some((entry.item, entry.letter, eq + child_qty))
            } else {
                None
            }
        })
    } else {
        None
    };

    let player_id = world.player_id;
    if let Some((existing_id, letter, next_qty)) = stack_target {
        let entities = &mut world.state_mut().entities;
        entities.remove_contained_item(container, item);
        entities.set_item_quantity(existing_id, next_qty);
        entities.set_item_location(item, EntityLocation::Consumed);
        entities.set_item_quantity(item, 1);
        events.push(GameEvent::ItemPickedUp {
            entity: player_id,
            item: existing_id,
            letter,
        });
    } else {
        let letter = world
            .state_mut()
            .inventory
            .add_existing_with_next_letter(item)
            .ok_or_else(|| "inventory letter capacity exceeded".to_string())?;
        let entities = &mut world.state_mut().entities;
        entities.remove_contained_item(container, item);
        entities.set_item_location(item, EntityLocation::Inventory { owner: player_id });
        entities.set_item_letter(item, letter);
        events.push(GameEvent::ItemPickedUp {
            entity: player_id,
            item,
            letter,
        });
    }

    let item_name = aihack_content::item_display_name(item_kind, world.content_registry());
    let container_name =
        aihack_content::item_display_name(container_kind, world.content_registry());
    events.push(GameEvent::Message {
        priority: MessagePriority::Info,
        text: format!("You take the {item_name} from the {container_name}."),
    });
    Ok(events)
}

fn is_ancestor(
    entities: &EntityStore,
    candidate_ancestor: EntityId,
    mut current: EntityId,
    mut depth: usize,
) -> bool {
    while depth <= MAX_CONTAINER_DEPTH {
        if current == candidate_ancestor {
            return true;
        }
        let Some(EntityLocation::InContainer { container }) = entities.item_location(current)
        else {
            return false;
        };
        current = container;
        depth += 1;
    }
    true
}

fn ancestor_depth(entities: &EntityStore, mut current: EntityId) -> usize {
    let mut depth = 0usize;
    while depth <= MAX_CONTAINER_DEPTH {
        let Some(EntityLocation::InContainer { container }) = entities.item_location(current)
        else {
            break;
        };
        current = container;
        depth += 1;
    }
    depth
}

fn max_subtree_depth(entities: &EntityStore, root: EntityId, depth: usize) -> usize {
    if depth >= MAX_CONTAINER_DEPTH {
        return 0;
    }
    let Some(children) = entities.get(root).and_then(|e| e.item_contained_items()) else {
        return 0;
    };
    children
        .iter()
        .map(|&child| 1 + max_subtree_depth(entities, child, depth + 1))
        .max()
        .unwrap_or(0)
}

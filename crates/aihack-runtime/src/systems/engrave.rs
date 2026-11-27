use aihack_core::{
    action::{EngravingMethod, EngravingText},
    domain::{
        item::{ItemClass, ToolAction, WandEffect},
        level::{Engraving, EngravingKind},
        tile::TileKind,
    },
    event::{GameEvent, MessagePriority},
    position::Pos,
    rng::GameRng,
};

use crate::world::GameWorld;

const MAX_ENGRAVINGS_PER_LEVEL: usize = 32;

pub fn engrave(
    world: &mut GameWorld,
    method: EngravingMethod,
    text: EngravingText,
) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("engraving is only available in campaign mode".to_string());
    }
    let pos = world.player_pos();
    let tile = world
        .current_map()
        .tile(pos)
        .map_err(|_| "current position is invalid".to_string())?;
    if matches!(tile, TileKind::Water | TileKind::Lava) {
        return Err("you cannot engrave on water or lava".to_string());
    }

    let kind = match method {
        EngravingMethod::Fingers => EngravingKind::Dust,
        EngravingMethod::Item(id) => {
            if !world.inventory.contains(id) {
                return Err("engraving tool is not in player inventory".to_string());
            }
            let entity = world
                .entities
                .get(id)
                .ok_or_else(|| "entity is not an item".to_string())?;
            let (_, data, _, _, charges) = entity
                .item()
                .ok_or_else(|| "entity is not an item".to_string())?;
            if data.wand_effect == Some(WandEffect::Fire) {
                let current_charges = charges.unwrap_or(0);
                if current_charges == 0 {
                    return Err("the wand of fire has no charges left".to_string());
                }
                world
                    .state_mut()
                    .entities
                    .set_item_charges(id, Some(current_charges - 1));
                EngravingKind::Burned
            } else if data.class == ItemClass::Weapon || data.tool_action == Some(ToolAction::Dig) {
                EngravingKind::Engraved
            } else {
                return Err("you cannot engrave with that item".to_string());
            }
        }
    };

    let garbled = world.status_timers.confused > 0 || world.status_timers.stunned > 0;
    let final_text = if garbled {
        match text {
            EngravingText::Elbereth => "Elb?r#th".to_string(),
            _ => "?!?".to_string(),
        }
    } else {
        text.as_str().to_string()
    };

    let level_id = world.current_level();
    {
        let level = world
            .state_mut()
            .levels
            .get_mut(level_id)
            .ok_or_else(|| "current level missing".to_string())?;
        if let Some(existing) = level.engravings.iter_mut().find(|e| e.pos == pos) {
            existing.text = final_text.clone();
            existing.kind = kind;
        } else {
            if level.engravings.len() >= MAX_ENGRAVINGS_PER_LEVEL {
                return Err("too many engravings on this level".to_string());
            }
            level.engravings.push(Engraving {
                pos,
                text: final_text.clone(),
                kind,
            });
        }
    }

    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: format!("You write \"{final_text}\" on the floor."),
    }])
}

pub fn has_active_elbereth(world: &GameWorld, pos: Pos) -> bool {
    if world.campaign.is_none() {
        return false;
    }
    world
        .levels
        .get(world.current_level())
        .is_some_and(|level| {
            level
                .engravings
                .iter()
                .any(|e| e.pos == pos && e.text.contains("Elbereth"))
        })
}

pub fn erode_elbereth_on_attack(world: &mut GameWorld) -> Vec<GameEvent> {
    if world.campaign.is_none() {
        return Vec::new();
    }
    let pos = world.player_pos();
    let level_id = world.current_level();
    let Some(level) = world.state_mut().levels.get_mut(level_id) else {
        return Vec::new();
    };
    let Some(idx) = level
        .engravings
        .iter()
        .position(|e| e.pos == pos && e.text.contains("Elbereth"))
    else {
        return Vec::new();
    };
    match level.engravings[idx].kind {
        EngravingKind::Dust => {
            level.engravings.remove(idx);
        }
        EngravingKind::Engraved | EngravingKind::Burned => {
            let entry = &mut level.engravings[idx];
            entry.text.pop();
            if entry.text.is_empty() {
                level.engravings.remove(idx);
            }
        }
    }
    vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: "You smudge the engraving beneath you.".to_string(),
    }]
}

pub fn tick_dust_engravings(world: &mut GameWorld, rng: &mut GameRng) {
    if world.campaign.is_none() {
        return;
    }
    let pos = world.player_pos();
    let level_id = world.current_level();
    let Some(level) = world.state_mut().levels.get_mut(level_id) else {
        return;
    };
    if let Some(idx) = level
        .engravings
        .iter()
        .position(|e| e.pos == pos && e.kind == EngravingKind::Dust)
    {
        if rng.next_u64().is_multiple_of(4) {
            level.engravings[idx].text.pop();
            if level.engravings[idx].text.is_empty() {
                level.engravings.remove(idx);
            }
        }
    }
}

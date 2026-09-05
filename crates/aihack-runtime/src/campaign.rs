use crate::{save::SavedWorldV1, world::GameWorld};
use aihack_core::{
    campaign::MAX_XP,
    domain::{entity::EntityLocation, item::ItemKind},
    event::{GameEvent, MessagePriority},
};

pub(crate) fn award_xp(world: &mut GameWorld, difficulty: u16) -> Vec<GameEvent> {
    let Some(mut campaign) = world.campaign else {
        return Vec::new();
    };
    let before = campaign.level();
    campaign.xp = campaign
        .xp
        .saturating_add(u32::from(difficulty.max(1)) * 10)
        .min(MAX_XP);
    let state = world.state_mut();
    let stats = state
        .entities
        .actor_stats_mut(state.player_id)
        .expect("live player");
    stats.hp = stats
        .hp
        .saturating_add(4 * i16::from(campaign.level() - before))
        .min(campaign.max_hp());
    stats.max_hp = campaign.max_hp();
    stats.hit_bonus = campaign.hit_bonus();
    stats.damage_bonus = campaign.damage_bonus();
    state.campaign = Some(campaign);
    vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: format!("XP {} / level {}", campaign.xp, campaign.level()),
    }]
}

pub(crate) fn validate(world: &SavedWorldV1) -> Result<(), String> {
    let quest_items: Vec<_> = world
        .entities
        .entities()
        .iter()
        .filter(|entity| {
            entity
                .item()
                .is_some_and(|(kind, _, _, _, _)| kind == ItemKind::AmuletAscension)
        })
        .collect();
    let Some(campaign) = world.campaign else {
        return if quest_items.is_empty() {
            Ok(())
        } else {
            Err("quest item requires campaign".into())
        };
    };
    if campaign.xp > MAX_XP || campaign.xp % 10 != 0 {
        return Err("campaign XP out of range".into());
    }
    let stats = world
        .entities
        .actor_stats(world.player_id)
        .ok_or("campaign player missing")?;
    if stats.max_hp != campaign.max_hp()
        || stats.hit_bonus != campaign.hit_bonus()
        || stats.damage_bonus != campaign.damage_bonus()
    {
        return Err("campaign role/XP/stats mismatch".into());
    }
    if quest_items.len() != 1 || quest_items[0].id != campaign.amulet {
        return Err("campaign requires its unique amulet ID and kind".into());
    }
    match quest_items[0].item().unwrap().2 {
        EntityLocation::OnMap { .. } => {}
        EntityLocation::Inventory { owner } if owner == world.player_id => {}
        _ => return Err("campaign amulet cannot be destroyed or owned by another actor".into()),
    }
    let earned: u32 = world
        .entities
        .entities()
        .iter()
        .filter(|entity| entity.monster_difficulty().is_some() && !entity.is_alive_actor())
        .map(|entity| u32::from(entity.monster_difficulty().unwrap().max(1)) * 10)
        .fold(0u32, |total, value| total.saturating_add(value).min(MAX_XP));
    if campaign.xp > earned.min(MAX_XP) {
        return Err("campaign XP exceeds defeated monster budget".into());
    }
    Ok(())
}

pub(crate) fn has_amulet(world: &GameWorld) -> bool {
    world.campaign.is_some_and(|campaign| {
        world.inventory.contains(campaign.amulet)
            && world.entities.get(campaign.amulet).is_some_and(|entity| {
                entity.item().is_some_and(|(kind, _, location, _, _)| {
                    kind == ItemKind::AmuletAscension
                        && location
                            == (EntityLocation::Inventory {
                                owner: world.player_id,
                            })
                })
            })
    })
}

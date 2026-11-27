use crate::{save::SavedWorldV1, systems::skills, world::GameWorld};
use aihack_core::{
    campaign::{
        artifact_def, SkillCategory, SkillRank, ARTIFACT_INVOKE_COOLDOWN_TURNS, MAX_GOD_ANGER,
        MAX_LUCK, MAX_PRAYER_TIMEOUT, MAX_XP, MIN_LUCK,
    },
    domain::{entity::EntityLocation, item::ItemKind},
    event::{GameEvent, MessagePriority},
};
use std::collections::HashSet;

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
    let mp_delta = campaign.role.mp_per_level() * i16::from(campaign.level() - before);
    state.max_mp = campaign.max_mp();
    state.mp = state.mp.saturating_add(mp_delta).min(state.max_mp);
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
        if !quest_items.is_empty() {
            return Err("quest item requires campaign".into());
        }
        if world.mp != 0 || world.max_mp != 0 {
            return Err("legacy mode does not use campaign MP".into());
        }
        if world.ascension_pursuit_stage != 0 {
            return Err("legacy mode cannot have ascension pursuit state".into());
        }
        if !world.intrinsic_resistances.is_empty()
            || !world.status_timers.is_empty()
            || !world.weapon_skills.is_empty()
            || !world.spell_skill.is_empty()
            || !world.skill_practice.is_empty()
            || !world.learned_spells.is_empty()
            || !world.generated_artifacts.is_empty()
            || !world.ascension_altars.is_empty()
            || world.shop_debt > 0
        {
            return Err("legacy mode cannot have campaign P8/extended state".into());
        }
        return Ok(());
    };
    if world.shop_debt > crate::systems::npc::MAX_SHOP_DEBT {
        return Err("campaign shop_debt out of range".into());
    }
    if !campaign
        .role
        .is_valid_combo(campaign.race, campaign.alignment)
    {
        return Err("campaign role/race/alignment combo is invalid".into());
    }
    if !(MIN_LUCK..=MAX_LUCK).contains(&world.luck) {
        return Err("campaign luck out of range".into());
    }
    if !(0..=MAX_GOD_ANGER).contains(&campaign.god_anger) {
        return Err("campaign god_anger out of range".into());
    }
    if campaign.prayer_timeout > MAX_PRAYER_TIMEOUT {
        return Err("campaign prayer_timeout out of range".into());
    }
    if campaign.artifact_invoke_cooldown > ARTIFACT_INVOKE_COOLDOWN_TURNS {
        return Err("campaign artifact_invoke_cooldown out of range".into());
    }
    if world.learned_spells.len() > 32 {
        return Err("learned_spells exceeds cap".into());
    }
    let mut seen_spells = HashSet::new();
    for entry in &world.learned_spells {
        if !seen_spells.insert(entry.spell) {
            return Err("duplicate spell in learned_spells".into());
        }
    }
    if world.ascension_altars.len() > 8 {
        return Err("ascension_altars exceeds cap".into());
    }
    let mut seen_altar_keys = HashSet::new();
    for &(level_id, pos) in &world.ascension_altars {
        if !world
            .levels
            .map(level_id)
            .is_some_and(|map| map.contains(pos))
        {
            return Err("ascension_altars contains invalid level or out-of-bounds pos".into());
        }
        if !seen_altar_keys.insert((level_id, pos)) {
            return Err("duplicate entry in ascension_altars".into());
        }
    }
    for art_id in &world.generated_artifacts {
        if artifact_def(art_id).is_none() {
            return Err(format!("unknown generated artifact id: {art_id}"));
        }
    }
    for (key, rank_raw) in &world.weapon_skills {
        let Some(cat) = SkillCategory::from_key(key) else {
            return Err(format!("unknown weapon skill category: {key}"));
        };
        if cat.is_spell() {
            return Err(format!("spell category {key} stored in weapon_skills"));
        }
        let Some(rank) = SkillRank::from_u8(*rank_raw) else {
            return Err(format!("invalid weapon skill rank {rank_raw} for {key}"));
        };
        if rank > campaign.role.max_skill_rank(cat) {
            return Err(format!("weapon skill {key} exceeds role cap"));
        }
    }
    for (key, rank_raw) in &world.spell_skill {
        let Some(cat) = SkillCategory::from_key(key) else {
            return Err(format!("unknown spell skill category: {key}"));
        };
        if !cat.is_spell() {
            return Err(format!("weapon category {key} stored in spell_skill"));
        }
        let Some(rank) = SkillRank::from_u8(*rank_raw) else {
            return Err(format!("invalid spell skill rank {rank_raw} for {key}"));
        };
        if rank > campaign.role.max_skill_rank(cat) {
            return Err(format!("spell skill {key} exceeds role cap"));
        }
    }
    for key in world.skill_practice.keys() {
        if SkillCategory::from_key(key).is_none() {
            return Err(format!("unknown skill_practice category: {key}"));
        }
    }
    let _ = skills::current_rank;
    if !campaign.attributes.is_valid() {
        return Err("campaign attributes out of range".into());
    }
    if campaign.xp > MAX_XP || campaign.xp % 10 != 0 {
        return Err("campaign XP out of range".into());
    }
    if world.max_mp != campaign.max_mp() || !(0..=world.max_mp).contains(&world.mp) {
        return Err("campaign MP out of range".into());
    }
    if world.ascension_pursuit_stage > 3 {
        return Err("campaign ascension pursuit stage out of range".into());
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

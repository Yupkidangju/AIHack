use crate::{save::SavedWorldV1, systems::skills, world::GameWorld};
use aihack_core::{
    campaign::{
        artifact_def, Alignment, SkillCategory, SkillRank, ARTIFACT_INVOKE_COOLDOWN_TURNS,
        MAX_GOD_ANGER, MAX_LUCK, MAX_PRAYER_TIMEOUT, MAX_XP, MIN_LUCK,
    },
    domain::{
        entity::{ActorKind, EntityLocation, Faction},
        item::ItemKind,
        tile::TileKind,
    },
    event::{GameEvent, MessagePriority},
    ids::{BranchId, LevelId},
    position::Pos,
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

pub(crate) fn validate(world: &SavedWorldV1, turn: Option<u64>) -> Result<(), String> {
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
            || !world.shop_debts.is_empty()
        {
            return Err("legacy mode cannot have campaign P8/extended state".into());
        }
        return Ok(());
    };
    if world.shop_debt > crate::systems::npc::MAX_SHOP_DEBT {
        return Err("campaign shop_debt out of range".into());
    }
    let mut prev_shop_level: Option<LevelId> = None;
    let mut debt_sum = 0_u32;
    for &(shop_lvl, amount) in &world.shop_debts {
        if amount == 0 || !world.levels.contains(shop_lvl) {
            return Err("campaign shop_debts entry has invalid level or zero amount".into());
        }
        if prev_shop_level.is_some_and(|prev| prev >= shop_lvl) {
            return Err("campaign shop_debts entries must be unique and sorted".into());
        }
        prev_shop_level = Some(shop_lvl);
        debt_sum = debt_sum
            .checked_add(amount)
            .ok_or_else(|| "campaign shop_debts sum overflows".to_string())?;
    }
    if debt_sum != world.shop_debt {
        return Err("campaign shop_debt disagrees with per-shop shop_debts ledger".into());
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
    let planes5_id = LevelId::new(BranchId::Planes, 5);
    let planes5 = world
        .levels
        .get(planes5_id)
        .ok_or_else(|| "campaign Planes 5 (Astral Plane) level missing".to_string())?;
    let mut expected_altar_positions: Vec<Pos> = (0..planes5.map.height)
        .flat_map(|y| (0..planes5.map.width).map(move |x| Pos { x, y }))
        .filter(|&pos| planes5.map.tile(pos).ok() == Some(TileKind::Altar))
        .collect();
    expected_altar_positions.sort_by_key(|p| (p.y, p.x));
    if expected_altar_positions.len() != 3 {
        return Err("Planes 5 must contain exactly 3 astral high altars".into());
    }
    let expected_ascension_altars: Vec<(LevelId, Pos)> = expected_altar_positions
        .iter()
        .copied()
        .map(|pos| (planes5_id, pos))
        .collect();
    let mut actual_ascension_altars = world.ascension_altars.clone();
    actual_ascension_altars.sort_by_key(|&(lvl, pos)| (lvl, pos.y, pos.x));
    if world.ascension_altars.len() != 3 || actual_ascension_altars != expected_ascension_altars {
        return Err("ascension_altars must match Planes 5 high altar coordinates".into());
    }
    let expected_alignments = [Alignment::Lawful, Alignment::Neutral, Alignment::Chaotic];
    if planes5.altar_alignments.len() != 3 {
        return Err("Planes 5 altar_alignments must assign all 3 high altars".into());
    }
    for (&pos, expected_align) in expected_altar_positions.iter().zip(expected_alignments) {
        if planes5.altar_alignment(pos) != Some(expected_align) {
            return Err("Planes 5 high altar alignment invariant violated".into());
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
    let dead_monster_xp: u32 = if world.kill_count == 0 {
        0
    } else {
        world
            .entities
            .entities()
            .iter()
            .filter_map(|entity| {
                let (ActorKind::Monster(_), faction, _, _, _, alive) = entity.actor()? else {
                    return None;
                };
                if alive || faction == Faction::Pet {
                    return None;
                }
                let diff = entity.monster_difficulty().unwrap_or(1).max(1);
                Some(u32::from(diff).saturating_mul(10))
            })
            .fold(0_u32, u32::saturating_add)
    };
    let used_gain_level = world.identified_items.contains(&ItemKind::PotionGainLevel)
        && world.entities.entities().iter().any(|entity| {
            entity
                .item()
                .is_some_and(|(kind, _, _, _, _)| kind == ItemKind::PotionGainLevel)
        });
    let stats = world
        .entities
        .actor_stats(world.player_id)
        .ok_or("campaign player missing")?;
    let (extra_hp_cap, extra_mp_cap) = match turn {
        Some(0) => {
            if campaign.xp != 0 || world.kill_count != 0 {
                return Err("turn 0 campaign cannot have earned XP or kills".into());
            }
            if stats.max_hp != campaign.max_hp() || world.max_mp != campaign.max_mp() {
                return Err("turn 0 campaign max_hp/max_mp mismatch".into());
            }
            (0_i16, 0_i16)
        }
        Some(t) => {
            let max_potion_xp = if used_gain_level {
                u32::try_from(t.saturating_mul(20).min(u64::from(MAX_XP))).unwrap_or(MAX_XP)
            } else {
                0
            };
            let max_earned_xp = dead_monster_xp.saturating_add(max_potion_xp).min(MAX_XP);
            if campaign.xp > max_earned_xp {
                return Err("campaign XP exceeds verifiable progression budget".into());
            }
            if used_gain_level && campaign.xp == MAX_XP {
                (
                    i16::try_from(t.saturating_mul(3).min(60)).unwrap_or(60),
                    i16::try_from(t.saturating_mul(2).min(40)).unwrap_or(40),
                )
            } else {
                (0_i16, 0_i16)
            }
        }
        None => {
            let max_potion_xp = if used_gain_level { MAX_XP } else { 0 };
            let max_earned_xp = dead_monster_xp.saturating_add(max_potion_xp).min(MAX_XP);
            if campaign.xp > max_earned_xp {
                return Err("campaign XP exceeds verifiable progression budget".into());
            }
            if used_gain_level && campaign.xp == MAX_XP {
                (60_i16, 40_i16)
            } else {
                (0_i16, 0_i16)
            }
        }
    };
    if world.max_mp < 0
        || world.max_mp > campaign.max_mp().saturating_add(extra_mp_cap)
        || !(0..=world.max_mp).contains(&world.mp)
    {
        return Err("campaign MP out of range".into());
    }
    if world.ascension_pursuit_stage > 5 {
        return Err("campaign ascension pursuit stage out of range".into());
    }
    if stats.max_hp < 4
        || stats.max_hp > campaign.max_hp().saturating_add(extra_hp_cap)
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

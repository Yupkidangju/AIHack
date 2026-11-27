use aihack_ai_contract::SkillObservation;
use aihack_core::{
    campaign::{Role, SkillCategory, SkillRank},
    event::{GameEvent, MessagePriority},
    world::WorldState,
};

use crate::{domain::entity::EntityStore, world::GameWorld};

pub(crate) fn init_role_skills(state: &mut WorldState<EntityStore>, role: Role) {
    state.weapon_skills.clear();
    state.spell_skill.clear();
    state.skill_practice.clear();
    for category in SkillCategory::ALL {
        let rank = role.starting_skill_rank(category).as_u8();
        if category.is_spell() {
            state.spell_skill.insert(category.key().to_owned(), rank);
        } else {
            state.weapon_skills.insert(category.key().to_owned(), rank);
        }
    }
}

pub(crate) fn current_rank(world: &GameWorld, category: SkillCategory) -> SkillRank {
    let raw = if category.is_spell() {
        world.spell_skill.get(category.key()).copied()
    } else {
        world.weapon_skills.get(category.key()).copied()
    };
    raw.and_then(SkillRank::from_u8).unwrap_or_else(|| {
        world
            .campaign
            .map_or(SkillRank::Basic, |c| c.role.starting_skill_rank(category))
    })
}

pub(crate) fn practice_of(world: &GameWorld, category: SkillCategory) -> u16 {
    world
        .skill_practice
        .get(category.key())
        .copied()
        .unwrap_or(0)
}

pub(crate) fn can_enhance(world: &GameWorld, category: SkillCategory) -> bool {
    let Some(campaign) = world.campaign else {
        return false;
    };
    let rank = current_rank(world, category);
    let cap = campaign.role.max_skill_rank(category);
    rank < cap && practice_of(world, category) >= rank.practice_threshold()
}

pub(crate) fn record_practice(world: &mut GameWorld, category: SkillCategory, amount: u16) {
    let Some(campaign) = world.campaign else {
        return;
    };
    if amount == 0 {
        return;
    }
    let rank = current_rank(world, category);
    let cap = campaign.role.max_skill_rank(category);
    if rank >= cap {
        return;
    }
    let key = category.key().to_owned();
    let entry = world.state_mut().skill_practice.entry(key).or_insert(0);
    *entry = entry.saturating_add(amount);
}

pub(crate) fn enhance(
    world: &mut GameWorld,
    category: SkillCategory,
) -> Result<Vec<GameEvent>, String> {
    let Some(campaign) = world.campaign else {
        return Err("skill enhancement is only available in campaign mode".to_string());
    };
    let rank = current_rank(world, category);
    let cap = campaign.role.max_skill_rank(category);
    if rank >= cap {
        return Err(format!(
            "{} is already at maximum rank ({})",
            category.display_name(),
            cap.display_name()
        ));
    }
    let threshold = rank.practice_threshold();
    let practice = practice_of(world, category);
    if practice < threshold {
        return Err(format!(
            "{} requires {threshold} practice ({practice} accumulated)",
            category.display_name()
        ));
    }
    let next_rank = rank
        .next()
        .ok_or_else(|| "skill is already at Grand Master".to_string())?;
    let key = category.key().to_owned();
    let state = world.state_mut();
    let remaining = practice.saturating_sub(threshold);
    if remaining == 0 {
        state.skill_practice.remove(&key);
    } else {
        state.skill_practice.insert(key.clone(), remaining);
    }
    if category.is_spell() {
        state.spell_skill.insert(key, next_rank.as_u8());
    } else {
        state.weapon_skills.insert(key, next_rank.as_u8());
    }
    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: format!(
            "You feel more skilled in {} ({})!",
            category.display_name(),
            next_rank.display_name()
        ),
    }])
}

pub(crate) fn melee_skill_category(world: &GameWorld) -> SkillCategory {
    world
        .inventory
        .equipped_melee
        .and_then(|item_id| world.entities.item_data(item_id))
        .and_then(|data| SkillCategory::for_weapon_kind(data.kind))
        .unwrap_or(SkillCategory::BareHands)
}

pub(crate) fn skill_observations(world: &GameWorld) -> Vec<SkillObservation> {
    let Some(campaign) = world.campaign else {
        return Vec::new();
    };
    SkillCategory::ALL
        .into_iter()
        .map(|category| {
            let rank = current_rank(world, category);
            let cap = campaign.role.max_skill_rank(category);
            let practice = practice_of(world, category);
            let ready = rank < cap && practice >= rank.practice_threshold();
            SkillObservation {
                category,
                rank,
                cap,
                practice,
                ready,
            }
        })
        .collect()
}

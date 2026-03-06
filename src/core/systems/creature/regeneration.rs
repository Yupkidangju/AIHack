// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::entity::player::Player;
use crate::core::entity::{Health, PlayerTag};
use legion::world::SubWorld;
use legion::*;

///
pub fn regeneration_system(ctx: &mut crate::core::context::GameContext) {
    let mut query = <(&mut Player, &mut Health)>::query().filter(component::<PlayerTag>());

    for (p_stats, p_health) in query.iter_mut(ctx.world) {
        // 1. 에너지(PW) 회복: 지혜(WIS) 및 레벨 기반
        let pw_regen_turns = if p_stats.wis.base > 12 { 3 } else { 5 };
        if ctx.turn % pw_regen_turns == 0 {
            if p_stats.energy < p_stats.energy_max {
                let gain = if p_stats.exp_level > 10 { 2 } else { 1 };
                p_stats.energy = (p_stats.energy + gain).min(p_stats.energy_max);
            }
        }

        // 2. 체력(HP) 회복: 건강(CON) 기반 주기 결정
        // NetHack: CON 3-12 (20 turns), 13-18 (20 - (CON-12) turns)
        let mut hp_regen_turns = 20;
        if p_stats.con.base > 12 {
            hp_regen_turns = (20 - (p_stats.con.base - 12)).max(5) as u64;
        }

        if ctx.turn % hp_regen_turns == 0 {
            if p_health.current < p_health.max {
                p_health.current += 1;
                // Player 구조체 내부 HP도 동기화 (UI 표시용)
                p_stats.hp = p_health.current;
            }
        }
    }
}

/// 몬스터 자동 회복 시스템 (원본 monmove.c:mon_regen 이식)
pub fn monster_regeneration_system(ctx: &mut crate::core::context::GameContext) {
    //
    let mut p_query = <&crate::core::entity::Level>::query()
        .filter(component::<crate::core::entity::PlayerTag>());
    let p_level =
        p_query
            .iter(ctx.world)
            .next()
            .map(|l| l.0)
            .unwrap_or(crate::core::dungeon::LevelID::new(
                crate::core::dungeon::DungeonBranch::Main,
                1,
            ));

    use crate::core::entity::capability::MonsterCapability; // [v2.0.0 R6] 의미적 래퍼
    let mut query = <(
        &mut Health,
        &crate::core::entity::Monster,
        &mut crate::core::entity::monster::MonsterState,
        &crate::core::entity::Level,
    )>::query()
    .filter(component::<crate::core::entity::MonsterTag>());

    for (m_health, monster, _m_state, m_level) in query.iter_mut(ctx.world) {
        //
        if m_level.0 != p_level {
            continue;
        }

        if let Some(template) = ctx.assets.monsters.get_by_kind(monster.kind) {
            // NetHack 3.6.7: Moves % 20 == 0 || REGEN 플래그
            if m_health.current < m_health.max
                && (ctx.turn % 20 == 0 || template.has_capability(MonsterCapability::Regen))
            {
                m_health.current += 1;
            }
            // mspec_used 감소는 ai.rs에서 매 턴 처리함
        }
    }
}

// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
//
// [v2.0.0] 몬스터 사망 시 시체/전리품 드롭, 경험치, 레벨업, 게임 오버 전환
// [v2.0.0 R5] Phase R5: GameEvent 발행 ? MonsterDied, ExperienceGained, PlayerDied
// =============================================================================
// SubWorld 제약 사항:
//   - push() 불가 → 시체/금화 생성은 SpawnRequest 또는 외부 처리
//
//   - entry_mut() 가능 (individual Entity에 대해)
// =============================================================================

use crate::core::entity::player::Player;
use crate::core::entity::{CombatStats, Health, MonsterTag, PlayerTag};
use crate::core::events::{EventQueue, GameEvent}; // [v2.0.0 R5] 이벤트 발행
use crate::ui::log::GameLog;
use legion::world::SubWorld;
use legion::*;

///
#[derive(Clone, Debug)]
pub struct CorpseRequest {
    pub monster_name: String,
    pub x: i32,
    pub y: i32,
    pub color: u8,
    pub weight: u32,
    pub corpse_age: u64,
}

///
#[derive(Clone, Debug)]
pub struct ItemDropRequest {
    pub item_entity: Entity,
    pub x: i32,
    pub y: i32,
}

///
#[derive(Clone, Debug, Default)]
pub struct DeathResults {
    pub corpse_requests: Vec<CorpseRequest>,
    pub item_drop_requests: Vec<ItemDropRequest>,
}

#[system]
#[write_component(Player)]
#[read_component(Health)]
#[read_component(CombatStats)]
#[read_component(MonsterTag)]
#[read_component(PlayerTag)]
#[read_component(crate::core::entity::Position)]
#[read_component(crate::core::entity::Monster)]
#[read_component(crate::core::entity::Inventory)]
#[read_component(crate::core::entity::Level)]
pub fn death(
    world: &mut SubWorld,
    #[resource] log: &mut GameLog,
    #[resource] assets: &crate::assets::AssetManager,
    #[resource] rng: &mut crate::util::rng::NetHackRng,
    #[resource] turn: &u64,
    #[resource] game_state: &mut crate::core::game_state::GameState,
    #[resource] event_queue: &mut EventQueue, // [v2.0.0 R5] 이벤트 큐
    #[resource] provider: &crate::core::systems::social::DefaultInteractionProvider, // [R8-3] LLM 교체 포인트
    command_buffer: &mut legion::systems::CommandBuffer,
) {
    // =========================================================================
    //
    // =========================================================================
    let mut player_ent: Option<Entity> = None;
    let mut player_luck = 0i32;
    let mut player_exp_level = 1i32;
    let mut player_level =
        crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1);

    {
        let mut p_query = <(Entity, &Player, &crate::core::entity::Level)>::query()
            .filter(component::<PlayerTag>());

        for (e, p, lvl) in p_query.iter(world) {
            player_ent = Some(*e);
            player_luck = p.luck;
            player_exp_level = p.exp_level;
            player_level = lvl.0;
        }
    }

    // =========================================================================
    // 2. 죽은 몬스터 정보 수집 (원본 mon.c:xkilled)
    // =========================================================================
    struct DeadInfo {
        entity: Entity,
        template_name: String,
        pos_x: i32,
        pos_y: i32,
        level: i32,
        items: Vec<Entity>,
        should_explode: bool,
        explode_adtype: crate::core::entity::monster::DamageType,
        explode_dice: (i32, i32),
    }

    let mut dead_list: Vec<DeadInfo> = Vec::new();

    {
        let mut m_query = <(
            Entity,
            &Health,
            &CombatStats,
            &crate::core::entity::Monster,
            &crate::core::entity::Position,
        )>::query()
        .filter(component::<MonsterTag>());

        for (entity, health, stats, monster, pos) in m_query.iter(world) {
            if health.current <= 0 {
                let mut should_explode = false;
                let mut explode_adtype = crate::core::entity::monster::DamageType::Phys;
                let mut explode_dice = (0i32, 0i32);

                if let Some(template) = assets.monsters.get_by_kind(monster.kind) {
                    for attack in &template.attacks {
                        if attack.atype == crate::core::entity::monster::AttackType::Explode
                            || attack.atype == crate::core::entity::monster::AttackType::Boom
                        {
                            should_explode = true;
                            explode_adtype = attack.adtype;
                            explode_dice = (attack.dice as i32, attack.sides as i32);
                            break;
                        }
                    }
                }

                //
                let items = Vec::new(); // SubWorld에서 Inventory 접근은 별도 Query 필요

                dead_list.push(DeadInfo {
                    entity: *entity,
                    template_name: monster.kind.to_string(),
                    pos_x: pos.x,
                    pos_y: pos.y,
                    level: stats.level,
                    items,
                    should_explode,
                    explode_adtype,
                    explode_dice,
                });
            }
        }
    }

    //
    {
        let mut inv_query =
            <(Entity, &crate::core::entity::Inventory)>::query().filter(component::<MonsterTag>());

        for (entity, inv) in inv_query.iter(world) {
            if let Some(dead) = dead_list.iter_mut().find(|d| d.entity == *entity) {
                dead.items = inv.items.clone();
            }
        }
    }

    // =========================================================================
    // 3. 각 죽은 몬스터 처리 (원본: xkilled + mondead)
    // =========================================================================
    let mut total_xp_gain: u64 = 0;

    for dead in &dead_list {
        let template_opt = assets.monsters.templates.get(&dead.template_name);
        let mon_name = &dead.template_name;

        // --- 3-1. 사망 메시지 ---
        log.add(format!("The {} dies!", mon_name), *turn);

        // [v2.0.0 R5] MonsterDied 이벤트 발행 ? 기존 DeathResults와 병행
        event_queue.push(GameEvent::MonsterDied {
            name: mon_name.clone(),
            killer: "player".to_string(),
            dropped_corpse: template_opt
                .map(|t| crate::core::entity::mon::corpse_chance(t))
                .unwrap_or(false),
            x: dead.pos_x,
            y: dead.pos_y,
            xp_gained: 0, // 사망 시점에는 아직 미확정, ExperienceGained에서 기록
        });

        // --- 3-2. 경험치 계산 (원본: exper.c:experience) ---
        if let Some(template) = template_opt {
            let base_xp = crate::core::systems::exper::experience(template, player_exp_level);
            let adjusted_xp = crate::core::systems::exper::adjusted_experience(
                base_xp,
                player_luck,
                player_exp_level,
                dead.level,
            );
            total_xp_gain += adjusted_xp;
        } else {
            total_xp_gain += (dead.level * 10) as u64;
        }

        // --- 3-3. 아이템 드롭 — CommandBuffer로 위치 변경 (R8-1: DeathResults 대체) ---
        // [v2.20.0 R8] SubWorld 제약 우회: CommandBuffer로 아이템 드롭 위치 컴포넌트 추가
        for &item_ent in &dead.items {
            command_buffer.add_component(
                item_ent,
                crate::core::entity::Position {
                    x: dead.pos_x,
                    y: dead.pos_y,
                },
            );
        }

        // --- 3-4. 시체 생성 — CommandBuffer.push()로 직접 Entity 생성 (R8-1) ---
        // [v2.20.0 R8] DeathResults 브릿지 제거: SubWorld 제약을 CommandBuffer로 완전 해소
        if let Some(template) = template_opt {
            if crate::core::entity::mon::corpse_chance(template) {
                command_buffer.push((
                    crate::core::entity::ItemTag,
                    crate::core::entity::Position {
                        x: dead.pos_x,
                        y: dead.pos_y,
                    },
                    crate::core::entity::Renderable {
                        glyph: '%',
                        color: template.color,
                    },
                    crate::core::entity::Item {
                        kind: crate::generated::ItemKind::from_str(&format!("{} corpse", mon_name)),
                        weight: (template.weight / 10).max(1) as u32,
                        quantity: 1,
                        corpsenm: Some(mon_name.clone()),
                        age: *turn,
                        ..Default::default()
                    },
                    crate::core::entity::Level(player_level),
                ));
            }
        }

        // --- 3-5. 금화 보너스 (인간형 몬스터) ---
        if let Some(template) = template_opt {
            if template.symbol == '@' || template.symbol == 'h' {
                total_xp_gain += (dead.level * 5 + 5) as u64;
            }
        }

        // --- 3-6. 폭발 처리 ---
        if dead.should_explode {
            crate::core::systems::combat::CombatEngine::execute_explosion(
                world,
                (dead.pos_x, dead.pos_y),
                dead.explode_adtype,
                dead.explode_dice,
                log,
                *turn,
                rng,
                assets,
            );
        }

        // --- 3-7. 엔티티 제거 ---
        command_buffer.remove(dead.entity);
    }

    // =========================================================================
    // 4. 경험치 반영 및 레벨업 (원본: exper.c:gain_experience + pluslvl)
    // =========================================================================
    if total_xp_gain > 0 {
        if let Some(p_ent) = player_ent {
            if let Ok(mut p_entry) = world.entry_mut(p_ent) {
                if let Ok(player) = p_entry.get_component_mut::<Player>() {
                    log.add(format!("You gain {} experience.", total_xp_gain), *turn);

                    // [v2.0.0 R5] ExperienceGained 이벤트 발행
                    event_queue.push(GameEvent::ExperienceGained {
                        amount: total_xp_gain,
                    });

                    crate::core::systems::exper::gain_experience(
                        player,
                        total_xp_gain,
                        rng,
                        log,
                        *turn,
                    );
                }
            }
        }
    }

    // =========================================================================
    //
    // =========================================================================
    {
        let mut pq = <(Entity, &Health)>::query().filter(component::<PlayerTag>());
        for (_e, health) in pq.iter(world) {
            if health.current <= 0 {
                // [R8-3] provider 경유 사망 에필로그
                use crate::core::systems::social::InteractionProvider;
                let cause_str = format!("killed on level {} of the Dungeon", player_level.depth);
                let epitaph = provider.generate_death_epitaph(&cause_str, "You");
                log.add_colored(&epitaph, [255, 0, 0], *turn);

                let tombstone = provider.generate_tombstone_text("Adventurer", &cause_str, 0);
                log.add_colored(format!("Tombstone: {}", tombstone), [200, 200, 200], *turn);

                // [v2.0.0 R5] PlayerDied 이벤트 발행
                event_queue.push(GameEvent::PlayerDied {
                    cause: cause_str.clone(),
                });

                *game_state = crate::core::game_state::GameState::GameOver {
                    message: format!(
                        "You were killed on level {} of the Dungeon.",
                        player_level.depth
                    ),
                };
            }
        }
    }
}

// =============================================================================
// [v2.3.1] end.c 핵심 로직 이식
// 원본: nethack-3.6.7/src/end.c (2,092줄)
//
// 사망 원인, 점수 계산, 묘비, DYWYPI, 통계 등
// =============================================================================

/// [v2.3.1] 사망 원인 분류 (원본: done_in_by / killer_format)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeathCause {
    Monster(String), // 몬스터에 의한 사망
    Starvation,      // 굶주림
    Poison,          // 독
    Petrification,   // 석화
    Drowning,        // 익사
    Burning,         // 소사
    Falling,         // 낙사
    Crushing,        // 압사
    Zapping,         // 마법 사망
    Sickness,        // 질병
    Genocide,        // 학살
    SelfDamage,      // 자해
    Trap(String),    // 트랩 사망
    Quit,            // 자발적 종료
    Escaped,         // 탈출 (옌더 아뮬렛 상승)
    Ascended,        // 승천
    Trickery,        // 서버 트릭/버그
    Other(String),   // 기타
}

/// [v2.3.1] 사망 원인 설명 문자열 (원본: killer_format)
pub fn death_cause_description(cause: &DeathCause) -> String {
    match cause {
        DeathCause::Monster(name) => format!("killed by {}", a_an(name)),
        DeathCause::Starvation => "died of starvation".to_string(),
        DeathCause::Poison => "died from poisoning".to_string(),
        DeathCause::Petrification => "turned to stone".to_string(),
        DeathCause::Drowning => "drowned".to_string(),
        DeathCause::Burning => "burned to death".to_string(),
        DeathCause::Falling => "fell to their death".to_string(),
        DeathCause::Crushing => "crushed to death".to_string(),
        DeathCause::Zapping => "killed by a magical blast".to_string(),
        DeathCause::Sickness => "died of illness".to_string(),
        DeathCause::Genocide => "committed genocide".to_string(),
        DeathCause::SelfDamage => "killed themselves".to_string(),
        DeathCause::Trap(name) => format!("killed by {}", a_an(name)),
        DeathCause::Quit => "quit the game".to_string(),
        DeathCause::Escaped => "escaped the dungeon (with the Amulet)".to_string(),
        DeathCause::Ascended => "ascended to demigodhood".to_string(),
        DeathCause::Trickery => "killed by trickery".to_string(),
        DeathCause::Other(msg) => msg.clone(),
    }
}

///
fn a_an(name: &str) -> String {
    let first = name.chars().next().unwrap_or('a');
    if "aeiouAEIOU".contains(first) {
        format!("an {}", name)
    } else {
        format!("a {}", name)
    }
}

/// [v2.3.1] 점수 계산 (원본: end.c calc_score)
pub fn calculate_score(
    exp_level: i32,
    experience: u64,
    gold: u64,
    deepest_level: i32,
    turns: u64,
    has_amulet: bool,
    ascended: bool,
) -> u64 {
    let mut score: u64 = 0;

    // 경험치 기반 (원본: 경험치 전부 점수에 반영)
    score += experience;

    // 금화 (원본: 금화 * 1)
    score += gold;

    // 레벨 보너스 (원본: level * 1000)
    score += (exp_level as u64) * 1000;

    // 깊이 보너스
    score += (deepest_level as u64) * 500;

    // 턴 보너스 (적은 턴이 더 좋음)
    if turns > 0 && turns < 10000 {
        score += (10000 - turns) * 10;
    }

    // 아뮬렛 보너스
    if has_amulet {
        score += 50000;
    }

    // 승천 보너스
    if ascended {
        score += 100000;
    }

    score
}

/// [v2.3.1] 묘비 데이터 (원본: outrip)
#[derive(Debug, Clone)]
pub struct Tombstone {
    pub player_name: String,
    pub role: String,
    pub race: String,
    pub gender: String,
    pub alignment: String,
    pub cause_of_death: String,
    pub score: u64,
    pub level: i32,
    pub max_hp: i32,
    pub turns: u64,
    pub epitaph: String,
}

impl Tombstone {
    /// 묘비 텍스트 생성 (원본: outrip)
    pub fn render(&self) -> Vec<String> {
        vec![
            "                 ----------".to_string(),
            "                /          \\".to_string(),
            "               /    REST    \\".to_string(),
            "              /      IN      \\".to_string(),
            "             /     PEACE      \\".to_string(),
            "            /                  \\".to_string(),
            format!("           |  {:^18}  |", self.player_name),
            format!(
                "           |  {:^18}  |",
                format!("{} {}", self.race, self.role)
            ),
            format!("           |  {:^18}  |", self.cause_of_death),
            format!("           |  {:^18}  |", format!("Score: {}", self.score)),
            format!(
                "           |  {:^18}  |",
                format!("Lvl {} HP {}", self.level, self.max_hp)
            ),
            format!("           |  {:^18}  |", format!("T:{}", self.turns)),
            "           |                    |".to_string(),
            format!("           |  {:^18}  |", self.epitaph),
            "          *|    *  *  *         |*".to_string(),
            " _________)/\\/\\/\\/\\/\\/\\_(/\\__/\\__________".to_string(),
        ]
    }
}

/// [v2.3.1] 사망 통계 (원본: end.c disclose)
#[derive(Debug, Clone, Default)]
pub struct DeathStats {
    /// 총 몬스터 처치 수
    pub monsters_killed: i32,
    /// 가장 많이 처치한 몬스터
    pub most_killed: String,
    /// 가장 많이 처치한 수
    pub most_killed_count: i32,
    /// 방문한 최대 층
    pub deepest_level: i32,
    /// 획득한 총 금화
    pub total_gold: u64,
    /// 사용한 물약 수
    pub potions_used: i32,
    ///
    pub scrolls_read: i32,
    /// 식별된 아이템 수
    pub items_identified: i32,
    /// 사용한 소원 수
    pub wishes_used: i32,
    /// 먹은 음식 수
    pub food_eaten: i32,
}

/// [v2.3.1] DYWYPI 메시지 (원본: do_you_want_your_possessions_identified)
pub fn dywypi_prompt() -> &'static str {
    "Do you want your possessions identified? [ynq]"
}

/// [v2.3.1] 게임 오버 정보 요약
pub fn game_over_summary(player_name: &str, cause: &DeathCause, score: u64, turns: u64) -> String {
    format!(
        "Game over! {} {}. Score: {} (T:{})",
        player_name,
        death_cause_description(cause),
        score,
        turns,
    )
}

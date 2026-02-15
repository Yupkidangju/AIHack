// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::Grid;
use crate::core::entity::player::{Alignment, Player, PlayerClass};
use crate::core::entity::{Health, PlayerTag, Position};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

/// 제단 성향 변경 대기 정보
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingAltarUpdate {
    pub pos: (i32, i32),
    pub new_align: Alignment,
}

/// 원본 NetHack Deity 정보 (role.c) 이식
pub fn get_god_name(role: PlayerClass, align: Alignment) -> &'static str {
    match role {
        PlayerClass::Archeologist => match align {
            Alignment::Lawful => "Camaxtli",
            Alignment::Neutral => "Centeotl",
            Alignment::Chaotic => "Huhetotl",
        },
        PlayerClass::Barbarian => match align {
            Alignment::Lawful => "Mitra",
            Alignment::Neutral => "Crom",
            Alignment::Chaotic => "Set",
        },
        PlayerClass::Healer => match align {
            Alignment::Lawful => "Athena",
            Alignment::Neutral => "Hermes",
            Alignment::Chaotic => "Poseidon",
        },
        PlayerClass::Knight => match align {
            Alignment::Lawful => "Lugh",
            Alignment::Neutral => "Brigit",
            Alignment::Chaotic => "Manannan Mac Lir",
        },
        PlayerClass::Monk => match align {
            Alignment::Lawful => "Shan Lai Ching",
            Alignment::Neutral => "Chih Sung-tzu",
            Alignment::Chaotic => "Huatse",
        },
        PlayerClass::Priest => match align {
            Alignment::Lawful => "Orizaba",
            Alignment::Neutral => "Quetzalcoatl",
            Alignment::Chaotic => "Tezcatlipoca",
        },
        PlayerClass::Rogue => match align {
            Alignment::Lawful => "Ishtar",
            Alignment::Neutral => "Mog",
            Alignment::Chaotic => "Kos",
        },
        PlayerClass::Ranger => match align {
            Alignment::Lawful => "Venus",
            Alignment::Neutral => "Sol",
            Alignment::Chaotic => "Luna",
        },
        PlayerClass::Samurai => match align {
            Alignment::Lawful => "Amaterasu Omikami",
            Alignment::Neutral => "Susanoo",
            Alignment::Chaotic => "Tsukiyomi",
        },
        PlayerClass::Tourist => match align {
            Alignment::Lawful => "Blind Io",
            Alignment::Neutral => "The Lady",
            Alignment::Chaotic => "Offler",
        },
        PlayerClass::Valkyrie => match align {
            Alignment::Lawful => "Tyr",
            Alignment::Neutral => "Odin",
            Alignment::Chaotic => "Loki",
        },
        PlayerClass::Wizard => match align {
            Alignment::Lawful => "Ptah",
            Alignment::Neutral => "Thoth",
            Alignment::Chaotic => "Anhur",
        },
    }
}

/// 기도(Pray) 시스템
/// 원본 NetHack의 pray.c 로직 이식
pub fn try_pray(
    world: &mut SubWorld,
    grid: &Grid,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
) -> bool {
    //
    let (p_pos_val, p_align_val, p_role_val, p_piety_val) = {
        let mut p_query = <(&Player, &Position)>::query().filter(component::<PlayerTag>());
        match p_query.iter(world).next() {
            Some((p, pos)) => (Some(*pos), Some(p.alignment), Some(p.role), Some(p.piety)),
            None => (None, None, None, None),
        }
    };

    if let (Some(p_pos), Some(p_align), Some(p_role), Some(_piety)) =
        (p_pos_val, p_align_val, p_role_val, p_piety_val)
    {
        let god_name = get_god_name(p_role, p_align);
        log.add(format!("당신은 {}에게 기도를 시작합니다.", god_name), turn);

        // 현재 위치 타일 및 제단 성향 확인
        let mut on_altar = false;
        let mut altar_align = Alignment::Neutral;

        if let Some(tile) = grid.get_tile(p_pos.x as usize, p_pos.y as usize) {
            if tile.typ == TileType::Altar {
                on_altar = true;
                altar_align = match tile.altarmask {
                    1 => Alignment::Lawful,
                    2 => Alignment::Neutral,
                    4 => Alignment::Chaotic,
                    _ => Alignment::Neutral,
                };
            }
        }

        // 1. 성수 제작 (Holy Water / Unholy Water) - NetHack: pray.c
        if on_altar {
            let mut potion_query = <(Entity, &Position, &mut crate::core::entity::Item)>::query();
            for (_ent, pos, item) in potion_query.iter_mut(world) {
                if pos.x == p_pos.x
                    && pos.y == p_pos.y
                    && item.kind == crate::generated::ItemKind::Water
                {
                    if altar_align == p_align {
                        if !item.blessed {
                            item.blessed = true;
                            item.cursed = false;
                            log.add("물이 은빛으로 빛납니다!", turn);
                        }
                    } else if altar_align != Alignment::Neutral {
                        if !item.cursed {
                            item.cursed = true;
                            item.blessed = false;
                            log.add("물이 어둡고 탁하게 변합니다.", turn);
                        }
                    }
                }
            }
        }

        //
        let mut p_query = <(
            &mut Player,
            &mut Health,
            &Position,
            &mut crate::core::entity::status::StatusBundle,
        )>::query()
        .filter(component::<PlayerTag>());

        if let Some((p_stats, p_health, _pos, p_status)) = p_query.iter_mut(world).next() {
            // 기도 쿨다운 체크 (u.ublesscnt)
            if p_stats.prayer_cooldown > 0 {
                log.add(
                    format!("{}께서 당신의 너무 빈번한 기도에 분노하셨습니다!", god_name),
                    turn,
                );
                let dmg = rng.rn1(10, 10);
                p_health.current -= dmg;
                p_stats.luck -= 1;
                log.add_colored(
                    format!("당신은 신의 분노로 {}의 피해를 입었습니다!", dmg),
                    [255, 0, 0],
                    turn,
                );
                return true;
            }

            p_stats.prayer_cooldown = rng.rn1(500, 500);

            if on_altar {
                if altar_align == p_stats.alignment {
                    log.add(
                        format!("{}의 존재감이 당신을 평온하게 감쌉니다.", god_name),
                        turn,
                    );
                    p_stats.piety += 10;
                } else {
                    let other_god = get_god_name(p_role, altar_align);
                    log.add(
                        format!(
                            "{}께서 당신이 {}의 제단에 있는 것에 분노하셨습니다!",
                            god_name, other_god
                        ),
                        turn,
                    );
                    let dmg = rng.rn1(6, 5);
                    p_health.current -= dmg;
                    log.add_colored(
                        format!("당신은 신의 분노로 {}의 피해를 입었습니다!", dmg),
                        [255, 0, 0],
                        turn,
                    );
                    p_stats.piety -= 5;
                    p_stats.luck -= 1;
                    return true;
                }
            }

            // 주요 문제 해결 (Troubleshooting) - NetHack: fix_trouble()
            use crate::core::entity::status::StatusFlags;
            let mut fixed = false;

            if p_status.has(StatusFlags::STONING) {
                p_status.remove(StatusFlags::STONING);
                log.add("몸이 다시 유연해지는 것을 느낍니다.", turn);
                fixed = true;
            } else if p_status.has(StatusFlags::SLIMED) {
                p_status.remove(StatusFlags::SLIMED);
                log.add("슬라임이 사라집니다.", turn);
                fixed = true;
            } else if p_status.has(StatusFlags::SICK) || p_status.has(StatusFlags::FOOD_POISONING) {
                p_status.remove(StatusFlags::SICK);
                p_status.remove(StatusFlags::FOOD_POISONING);
                log.add("이제 몸 상태가 훨씬 좋아졌습니다.", turn);
                fixed = true;
            } else if p_health.current < p_health.max / 4 {
                p_health.current = p_health.max;
                log.add("당신의 상처가 기적적으로 치유되었습니다!", turn);
                fixed = true;
            } else if p_stats.nutrition <= 150 {
                p_stats.nutrition = 900;
                log.add(
                    format!("{}께서 당신의 허기를 채워주셨습니다!", god_name),
                    turn,
                );
                fixed = true;
            } else if p_status.has(StatusFlags::BLIND) {
                p_status.remove(StatusFlags::BLIND);
                log.add("시력이 회복되었습니다.", turn);
                fixed = true;
            } else if p_status.has(StatusFlags::CONFUSED) {
                p_status.remove(StatusFlags::CONFUSED);
                log.add("정신이 맑아집니다.", turn);
                fixed = true;
            } else if p_status.has(StatusFlags::STUNNED) {
                p_status.remove(StatusFlags::STUNNED);
                log.add("평형 감각을 되찾았습니다.", turn);
                fixed = true;
            } else if p_status.has(StatusFlags::HALLUCINATING) {
                p_status.remove(StatusFlags::HALLUCINATING);
                log.add("모든 것이 다시 정상으로 보입니다.", turn);
                fixed = true;
            } else if p_stats.str.base < p_stats.str.max
                || p_stats.dex.base < p_stats.dex.max
                || p_stats.con.base < p_stats.con.max
                || p_stats.int.base < p_stats.int.max
                || p_stats.wis.base < p_stats.wis.max
                || p_stats.cha.base < p_stats.cha.max
            {
                p_stats.str.base = p_stats.str.max;
                p_stats.dex.base = p_stats.dex.max;
                p_stats.con.base = p_stats.con.max;
                p_stats.int.base = p_stats.int.max;
                p_stats.wis.base = p_stats.wis.max;
                p_stats.cha.base = p_stats.cha.max;
                log.add("능력치가 회복되는 것을 느낍니다!", turn);
                fixed = true;
            }

            if fixed {
                p_stats.piety -= 20;
                log.add(
                    format!("{}께서 당신의 기도에 응답하셨습니다!", god_name),
                    turn,
                );
            } else {
                log.add(format!("{}께서 당신을 흡족해하십니다.", god_name), turn);
                p_stats.luck = (p_stats.luck + 1).min(10);
            }

            return true;
        }
    }

    false
}

/// 제물 바치기 (#offer)
pub fn try_offer(
    item_ent: Entity,
    world: &mut SubWorld,
    grid: &crate::core::dungeon::Grid,
    assets: &crate::assets::AssetManager,
    rng: &mut crate::util::rng::NetHackRng,
    log: &mut GameLog,
    turn: u64,
    command_buffer: &mut CommandBuffer,
) -> Option<Alignment> {
    let mut player_pos = None;
    let mut p_level =
        crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1);
    let mut p_ent = None;
    let mut new_altar_align = None;

    let mut query = <(Entity, &Position, &crate::core::entity::Level)>::query()
        .filter(component::<PlayerTag>());
    for (ent, pos, lvl) in query.iter(world) {
        player_pos = Some((pos.x, pos.y));
        p_level = lvl.0;
        p_ent = Some(*ent);
    }

    if let (Some((px, py)), Some(player_entity)) = (player_pos, p_ent) {
        if let Some(tile) = grid.get_tile(px as usize, py as usize) {
            if tile.typ != TileType::Altar {
                log.add("제물을 바치려면 제단이 필요합니다.", turn);
                return None;
            }

            // 제물 아이템 확인
            let mut is_corpse = false;
            let mut nutrition = 0;
            let mut corpse_name = None;

            if let Ok(entry) = world.entry_ref(item_ent) {
                if let Ok(item) = entry.get_component::<crate::core::entity::Item>() {
                    if item.kind.is_corpse() {
                        is_corpse = true;
                        nutrition = item.weight * 10; // 대략적인 영양가
                        corpse_name = item.corpsenm.clone();
                    }
                }
            }

            if !is_corpse {
                log.add("그것은 제물로 바칠 수 없습니다!", turn);
                return None;
            }

            // 제단 성향 확인
            let altar_align = match tile.altarmask {
                1 => Alignment::Lawful,
                2 => Alignment::Neutral,
                4 => Alignment::Chaotic,
                _ => Alignment::Neutral,
            };

            log.add(
                format!(
                    "당신은 {}의 시체를 제물로 바칩니다.",
                    corpse_name.as_deref().unwrap_or("이상한")
                ),
                turn,
            );

            // 신의 반응
            let mut p_query = <&mut Player>::query().filter(component::<PlayerTag>());
            if let Some(p) = p_query.iter_mut(world).next() {
                let god_name = get_god_name(p.role, p.alignment);

                if altar_align == p.alignment {
                    log.add(
                        format!("{}께서 당신의 제물에 기뻐하십니다.", god_name),
                        turn,
                    );
                    p.piety += (nutrition / 200).max(1) as i32;
                    p.alignment_record += 1;

                    //
                    if p.luck >= 0 && p.piety >= 20 && rng.rn2(10) == 0 {
                        log.add_colored("신의 은총이 솟구치는 것을 느낍니다!", [255, 215, 0], turn);
                        crate::core::systems::artifact::ArtifactSystem::gift_artifact(
                            p.alignment,
                            &mut World::default(),
                            assets,
                            player_entity,
                            (px, py),
                            p_level,
                            log,
                            turn,
                            command_buffer,
                        );
                        p.piety -= 15;
                    } else if p.piety > 25 {
                        p.luck = (p.luck + 1).min(10);
                        log.add("운이 좋아진 것 같습니다.", turn);
                        p.piety -= 5;
                    }
                } else if tile.altarmask != 0 {
                    // 타 성향 제단: 제단 전환 시도 (개종)
                    if p.piety > 40 && rng.rn2(p.piety) > 30 {
                        log.add_colored(
                            format!("제단이 {}의 성향으로 다시 정렬되었습니다!", god_name),
                            [255, 255, 255],
                            turn,
                        );
                        new_altar_align = Some(p.alignment);
                        p.piety -= 20;
                    } else {
                        let other_god = get_god_name(p.role, altar_align);
                        log.add(format!("{}께서 불쾌해하십니다!", other_god), turn);
                        p.alignment_record -= 2;
                        p.luck = (p.luck - 1).max(-10);

                        // 분노: 번개
                        let dmg = rng.rn1(10, 5);
                        log.add_colored(
                            format!("당신은 신의 분노로 {}의 피해를 입었습니다!", dmg),
                            [255, 0, 0],
                            turn,
                        );
                        //
                        let mut h_query = <&mut Health>::query().filter(component::<PlayerTag>());
                        if let Some(h) = h_query.iter_mut(world).next() {
                            h.current -= dmg;
                        }
                    }
                }
            }

            // 아이템 제거
            command_buffer.remove(item_ent);

            //
            let mut inv_query =
                <&mut crate::core::entity::Inventory>::query().filter(component::<PlayerTag>());
            for inv in inv_query.iter_mut(world) {
                if let Some(pos) = inv.items.iter().position(|&e| e == item_ent) {
                    inv.items.remove(pos);
                }
            }
        }
    }
    new_altar_align
}

// =============================================================================
// [v2.3.3] 기도 시스템 확장 (원본 pray.c: prayer logic)
// =============================================================================

/// 기도 결과 (원본: pray.c prayer_answered)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrayerResult {
    Ignored,    // 무시 (쿨다운 미충족)
    Answered,   // 응답 (문제 해결)
    Pleased,    // 흡족 (운 증가)
    Furious,    // 분노 (잘못된 제단/죄)
    Converted,  // 개종 (성향 변경)
    LastResort, // 최후의 수단 (부활)
}

///
pub fn prayer_success_chance(
    piety: i32,
    alignment_record: i32,
    luck: i32,
    on_aligned_altar: bool,
) -> i32 {
    let mut chance = 50; // 기본 50%

    // 신앙도 보정
    chance += piety / 2;

    // 성향 기록 보정
    chance += alignment_record * 3;

    // 운 보정
    chance += luck * 5;

    // 같은 성향 제단 보너스
    if on_aligned_altar {
        chance += 20;
    }

    chance.clamp(5, 99)
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sin {
    Murder,            // 우호적 존재 살해
    PetAbuse,          // 펫 학대
    Cannibalism,       // 식인
    DesecrateCemetery, // 무덤 훼손
    StealFromShop,     // 상점 절도
    BreakConduct,      // 행동 규범 위반 (채식/평화 등)
    AttackPeaceful,    // 평화적 존재 공격
}

/// 죄별 신앙도 감소량
pub fn sin_piety_penalty(sin: Sin) -> i32 {
    match sin {
        Sin::Murder => -15,
        Sin::PetAbuse => -10,
        Sin::Cannibalism => -20,
        Sin::DesecrateCemetery => -8,
        Sin::StealFromShop => -5,
        Sin::BreakConduct => -3,
        Sin::AttackPeaceful => -7,
    }
}

// =============================================================================
// [v2.3.3] 신의 분노 (원본 pray.c: gods_angry)
// =============================================================================

/// 신의 분노 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DivinePunishment {
    Lightning,    // 번개 (피해)
    Earthquake,   // 지진 (지형 파괴)
    InsectPlague, // 벌레 소환
    Disenchant,   // 장비 부여 해제
    Curse,        // 소지품 저주
    Summon,       // 적대 몬스터 소환
    BlindDeafen,  // 실명 + 청각 상실
}

/// 신의 분노 결정 (원본: god_zaps_you)
pub fn divine_punishment(anger_level: i32, rng: &mut NetHackRng) -> DivinePunishment {
    let severity = (anger_level / 5).min(6) as i32;
    let roll = rng.rn2(7);
    match (roll + severity).min(6) {
        0 => DivinePunishment::BlindDeafen,
        1 => DivinePunishment::Curse,
        2 => DivinePunishment::InsectPlague,
        3 => DivinePunishment::Disenchant,
        4 => DivinePunishment::Summon,
        5 => DivinePunishment::Earthquake,
        _ => DivinePunishment::Lightning,
    }
}

/// 분노 메시지
pub fn punishment_message(p: &DivinePunishment) -> &'static str {
    match p {
        DivinePunishment::Lightning => "A bolt of lightning strikes you!",
        DivinePunishment::Earthquake => "The ground trembles beneath you!",
        DivinePunishment::InsectPlague => "Swarms of insects descend upon you!",
        DivinePunishment::Disenchant => "Your equipment feels less enchanted!",
        DivinePunishment::Curse => "You feel your possessions becoming cursed!",
        DivinePunishment::Summon => "Hostile creatures appear around you!",
        DivinePunishment::BlindDeafen => "You are struck blind and deaf!",
    }
}

/// 분노 데미지
pub fn punishment_damage(p: &DivinePunishment, rng: &mut NetHackRng) -> i32 {
    match p {
        DivinePunishment::Lightning => rng.rn1(20, 10),
        DivinePunishment::Earthquake => rng.rn1(10, 5),
        DivinePunishment::InsectPlague => rng.rn1(6, 3),
        DivinePunishment::Disenchant => 0,
        DivinePunishment::Curse => 0,
        DivinePunishment::Summon => 0,
        DivinePunishment::BlindDeafen => rng.rn1(4, 2),
    }
}

// =============================================================================
// [v2.3.3] 제물 가치 (원본 pray.c: sacrifice value)
// =============================================================================

/// 제물 가치 계산 (원본: sacrifice_value)
pub fn sacrifice_value(
    corpse_nutrition: i32,
    is_unicorn: bool,
    same_alignment_unicorn: bool,
    monster_level: i32,
) -> i32 {
    let mut value = corpse_nutrition / 50;

    // 유니콘 보너스
    if is_unicorn {
        value += 10;
        if same_alignment_unicorn {
            value += 5; // 같은 성향 유니콘 추가 보너스
        }
    }

    // 고레벨 몬스터 보너스
    value += monster_level / 3;

    value.max(1)
}

/// 제물 시 성향 기록 변동
pub fn sacrifice_alignment_change(value: i32, altar_alignment_matches: bool) -> i32 {
    if altar_alignment_matches {
        (value / 3).max(1)
    } else {
        -(value / 2).max(1)
    }
}

// =============================================================================
// [v2.3.3] 기도 통계
// =============================================================================

/// 기도 통계
#[derive(Debug, Clone, Default)]
pub struct PrayerStatistics {
    pub total_prayers: u32,
    pub prayers_answered: u32,
    pub prayers_ignored: u32,
    pub divine_punishments: u32,
    pub conversions: u32,
    pub sacrifices_made: u32,
    pub artifacts_received: u32,
    pub sins_committed: u32,
}

impl PrayerStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_prayer(&mut self, result: &PrayerResult) {
        self.total_prayers += 1;
        match result {
            PrayerResult::Answered | PrayerResult::LastResort => self.prayers_answered += 1,
            PrayerResult::Ignored => self.prayers_ignored += 1,
            PrayerResult::Furious => self.divine_punishments += 1,
            PrayerResult::Converted => self.conversions += 1,
            PrayerResult::Pleased => self.prayers_answered += 1,
        }
    }
}

// =============================================================================
// [v2.3.3] 테스트
// =============================================================================
#[cfg(test)]
mod pray_extended_tests {
    use super::*;

    #[test]
    fn test_prayer_chance_base() {
        let c = prayer_success_chance(0, 0, 0, false);
        assert_eq!(c, 50);
    }

    #[test]
    fn test_prayer_chance_altar() {
        let c = prayer_success_chance(10, 5, 3, true);
        assert!(c > 80);
    }

    #[test]
    fn test_sin_penalty() {
        assert!(sin_piety_penalty(Sin::Cannibalism) < sin_piety_penalty(Sin::StealFromShop));
    }

    #[test]
    fn test_punishment() {
        let mut rng = NetHackRng::new(42);
        let p = divine_punishment(30, &mut rng);
        let msg = punishment_message(&p);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_punishment_damage() {
        let mut rng = NetHackRng::new(42);
        let d = punishment_damage(&DivinePunishment::Lightning, &mut rng);
        assert!(d >= 10 && d <= 30);
    }

    #[test]
    fn test_sacrifice_value() {
        let v = sacrifice_value(500, false, false, 10);
        assert!(v > 0);
        let v_unicorn = sacrifice_value(500, true, true, 10);
        assert!(v_unicorn > v);
    }

    #[test]
    fn test_prayer_stats() {
        let mut stats = PrayerStatistics::new();
        stats.record_prayer(&PrayerResult::Answered);
        stats.record_prayer(&PrayerResult::Furious);
        assert_eq!(stats.total_prayers, 2);
        assert_eq!(stats.prayers_answered, 1);
        assert_eq!(stats.divine_punishments, 1);
    }
}

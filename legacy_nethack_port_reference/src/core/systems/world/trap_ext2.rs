// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-19] 함정 확장2 (trap_ext2.rs)
// 원본: NetHack 3.6.7 trap.c (함정 해제 확률, 마법 함정, 화재/산 피해 판정)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 함정 해제 확률 (원본: trap.c:3944-3973 untrap_prob)
// =============================================================================

/// [v2.22.0 R34-19] 함정 해제 확률 입력
#[derive(Debug, Clone)]
pub struct UntrapInput {
    /// 거미줄 함정인지
    pub is_web: bool,
    /// 플레이어가 거미줄 제작 가능한지 (거미 변신 등)
    pub is_webmaker: bool,
    /// 상태이상 플래그
    pub confused: bool,
    pub hallucinating: bool,
    pub blind: bool,
    pub stunned: bool,
    pub fumbling: bool,
    /// 자기가 만든 함정인지
    pub made_by_player: bool,
    /// 역할 (0: 기타, 1: 로그, 2: 레인저)
    pub role: i32,
    /// 플레이어 레벨
    pub player_level: i32,
    /// 퀘스트 아티팩트 보유 여부
    pub has_quest_artifact: bool,
}

/// [v2.22.0 R34-19] 함정 해제 난이도 계산 (원본: untrap_prob)
/// 반환값이 낮을수록 해제 성공 확률 높음
pub fn calc_untrap_difficulty(input: &UntrapInput) -> i32 {
    let mut chance: i32 = 3;

    // 거미줄 + 거미 아님 → 매우 어려움
    if input.is_web && !input.is_webmaker {
        chance = 30;
    }
    if input.confused || input.hallucinating {
        chance += 1;
    }
    if input.blind {
        chance += 1;
    }
    if input.stunned {
        chance += 2;
    }
    if input.fumbling {
        chance *= 2;
    }
    // 자기 함정 → 더 쉬움
    if input.made_by_player {
        chance -= 1;
    }
    // 역할 보너스
    if input.role == 1 {
        // 로그: 레벨 기반 보너스
        if input.player_level > 15 {
            chance -= 1;
        }
        if input.has_quest_artifact && chance > 1 {
            chance -= 1;
        }
    } else if input.role == 2 && chance > 1 {
        // 레인저: -1
        chance -= 1;
    }

    chance.max(1) // 최소 1
}

/// [v2.22.0 R34-19] 함정 해제 성공 판정 (원본: untrap_prob의 rn2 호출)
pub fn check_untrap_success(input: &UntrapInput, rng: &mut NetHackRng) -> bool {
    let difficulty = calc_untrap_difficulty(input);
    rng.rn2(difficulty) == 0
}

// =============================================================================
// [2] 마법 함정 효과 결정 (원본: trap.c:3194-3305 domagictrap)
// =============================================================================

/// [v2.22.0 R34-19] 마법 함정 효과 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MagicTrapEffect {
    /// 몬스터 소환 + 섬광 + 폭음 (fate 1-9)
    SummonMonsters {
        count: i32,
        blind_duration: i32,
        deaf_duration: i32,
    },
    /// 아무 일도 안 일어남 (fate 10-11)
    Nothing,
    /// 화염 폭발 (fate 12)
    FireBurst,
    /// 오싹한 느낌 (fate 13)
    Shiver,
    /// 먼 울부짖음 (fate 14)
    DistantHowling,
    /// 향수병 (fate 15)
    Homesick,
    /// 가방 흔들림 (fate 16)
    PackShakes,
    /// 타는 냄새 (fate 17)
    SmellCharred,
    /// 피곤함 (fate 18)
    Fatigue,
    /// 매력 상승 + 길들이기 (fate 19)
    TameNearby,
    /// 저주 해제 (fate 20)
    RemoveCurse,
}

/// [v2.22.0 R34-19] 마법 함정 효과 결정 (원본: domagictrap)
pub fn determine_magic_trap_effect(rng: &mut NetHackRng) -> MagicTrapEffect {
    let fate = rng.rnd(20);

    if fate < 10 {
        // 몬스터 소환 + 섬광 + 폭음
        let count = rng.rnd(4);
        let blind_dur = rng.rn1(5, 10);
        let deaf_dur = rng.rn1(20, 30);
        MagicTrapEffect::SummonMonsters {
            count,
            blind_duration: blind_dur,
            deaf_duration: deaf_dur,
        }
    } else {
        match fate {
            10 | 11 => MagicTrapEffect::Nothing,
            12 => MagicTrapEffect::FireBurst,
            13 => MagicTrapEffect::Shiver,
            14 => MagicTrapEffect::DistantHowling,
            15 => MagicTrapEffect::Homesick,
            16 => MagicTrapEffect::PackShakes,
            17 => MagicTrapEffect::SmellCharred,
            18 => MagicTrapEffect::Fatigue,
            19 => MagicTrapEffect::TameNearby,
            20 => MagicTrapEffect::RemoveCurse,
            _ => MagicTrapEffect::Nothing,
        }
    }
}

// =============================================================================
// [3] 화재 피해 보호 확률 (원본: trap.c:3307-3398 fire_damage)
// =============================================================================

/// [v2.22.0 R34-19] 화재 피해 판정 입력
#[derive(Debug, Clone)]
pub struct FireDamageInput {
    /// 운 수치 (-13~13)
    pub luck: i32,
    /// 강제 피해인지
    pub force: bool,
    /// 컨테이너인지
    pub is_container: bool,
    /// 컨테이너 유형 (0: 기타, 1: 아이스박스, 2: 상자, 3: 큰 상자)
    pub container_type: i32,
    /// 내화성인지
    pub is_fireproof: bool,
    /// 가연성인지
    pub is_flammable: bool,
}

/// [v2.22.0 R34-19] 화재 피해 보호 확인 (보호되면 true)
pub fn check_fire_protection(input: &FireDamageInput, rng: &mut NetHackRng) -> bool {
    if input.is_container {
        // 아이스박스: 면역
        if input.container_type == 1 {
            return true;
        }
        let chance = match input.container_type {
            2 => 40, // 상자
            3 => 30, // 큰 상자
            _ => 20,
        };
        if !input.force && (input.luck + 5) > rng.rn2(chance) {
            return true;
        }
        if input.is_flammable && input.is_fireproof {
            return true;
        }
        false
    } else {
        // 일반 아이템: luck+5 > rn2(20)
        if !input.force && (input.luck + 5) > rng.rn2(20) {
            return true;
        }
        false
    }
}

// =============================================================================
// [4] 화염 함정 데미지 계산 (원본: trap.c:3124-3192 dofiretrap)
// =============================================================================

/// [v2.22.0 R34-19] 화염 함정 골렘 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GolemFireType {
    Paper,
    Straw,
    Wood,
    Leather,
    Other,
}

/// [v2.22.0 R34-19] 화염 함정 데미지 계산
/// `is_underwater`: 수중인지
/// `fire_resistant`: 화염 저항인지
/// `golem_type`: 골렘 유형 (변신 시)
/// `max_hp`: 최대 HP
pub fn calc_fire_trap_damage(
    is_underwater: bool,
    fire_resistant: bool,
    golem_type: GolemFireType,
    max_hp: i32,
    rng: &mut NetHackRng,
) -> i32 {
    if is_underwater {
        return rng.rnd(3); // 끓는 물 피해
    }
    if fire_resistant {
        return rng.rn2(2); // 저항: 0 또는 1
    }

    let base = rng.d(2, 4);
    let alt = match golem_type {
        GolemFireType::Paper => max_hp,
        GolemFireType::Straw => max_hp / 2,
        GolemFireType::Wood => max_hp / 4,
        GolemFireType::Leather => max_hp / 8,
        GolemFireType::Other => 0,
    };

    base.max(alt)
}

// =============================================================================
// [5] 산성 피해 판정 (원본: trap.c:3470-3505 acid_damage)
// =============================================================================

/// [v2.22.0 R34-19] 산성 피해 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcidDamageResult {
    /// 그리스로 보호됨
    GreaseProtected,
    /// 스크롤 → 백지화
    ScrollBlanked,
    /// 부식 피해
    Corroded,
    /// 해당 없음
    Nothing,
}

/// [v2.22.0 R34-19] 산성 피해 판정 (원본: acid_damage)
pub fn check_acid_damage(
    is_greased: bool,
    is_scroll: bool,
    is_blank_paper: bool,
) -> AcidDamageResult {
    if is_greased {
        return AcidDamageResult::GreaseProtected;
    }
    if is_scroll && !is_blank_paper {
        return AcidDamageResult::ScrollBlanked;
    }
    AcidDamageResult::Corroded
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_untrap_input() -> UntrapInput {
        UntrapInput {
            is_web: false,
            is_webmaker: false,
            confused: false,
            hallucinating: false,
            blind: false,
            stunned: false,
            fumbling: false,
            made_by_player: false,
            role: 0,
            player_level: 10,
            has_quest_artifact: false,
        }
    }

    #[test]
    fn test_untrap_base_difficulty() {
        let input = default_untrap_input();
        assert_eq!(calc_untrap_difficulty(&input), 3);
    }

    #[test]
    fn test_untrap_web_hard() {
        let mut input = default_untrap_input();
        input.is_web = true;
        assert_eq!(calc_untrap_difficulty(&input), 30);
    }

    #[test]
    fn test_untrap_web_easy_for_spider() {
        let mut input = default_untrap_input();
        input.is_web = true;
        input.is_webmaker = true;
        assert_eq!(calc_untrap_difficulty(&input), 3);
    }

    #[test]
    fn test_untrap_status_penalties() {
        let mut input = default_untrap_input();
        input.confused = true;
        input.blind = true;
        input.stunned = true;
        assert_eq!(calc_untrap_difficulty(&input), 7); // 3+1+1+2
    }

    #[test]
    fn test_untrap_fumbling_doubles() {
        let mut input = default_untrap_input();
        input.fumbling = true;
        assert_eq!(calc_untrap_difficulty(&input), 6); // 3*2
    }

    #[test]
    fn test_untrap_rogue_bonus() {
        let mut input = default_untrap_input();
        input.role = 1;
        input.player_level = 20;
        input.has_quest_artifact = true;
        assert_eq!(calc_untrap_difficulty(&input), 1); // 3-1-1 = 1
    }

    #[test]
    fn test_magic_trap_range() {
        let mut rng = NetHackRng::new(42);
        let effect = determine_magic_trap_effect(&mut rng);
        // 모든 결과가 유효해야 함
        assert!(matches!(
            effect,
            MagicTrapEffect::SummonMonsters { .. }
                | MagicTrapEffect::Nothing
                | MagicTrapEffect::FireBurst
                | MagicTrapEffect::Shiver
                | MagicTrapEffect::DistantHowling
                | MagicTrapEffect::Homesick
                | MagicTrapEffect::PackShakes
                | MagicTrapEffect::SmellCharred
                | MagicTrapEffect::Fatigue
                | MagicTrapEffect::TameNearby
                | MagicTrapEffect::RemoveCurse
        ));
    }

    #[test]
    fn test_magic_trap_distribution() {
        let mut rng = NetHackRng::new(123);
        let mut summon_count = 0;
        for _ in 0..1000 {
            if matches!(
                determine_magic_trap_effect(&mut rng),
                MagicTrapEffect::SummonMonsters { .. }
            ) {
                summon_count += 1;
            }
        }
        // 약 45% (9/20) 확률로 소환
        assert!(
            summon_count > 350 && summon_count < 550,
            "summon ratio: {}",
            summon_count
        );
    }

    #[test]
    fn test_fire_protection_icebox() {
        let mut rng = NetHackRng::new(42);
        let input = FireDamageInput {
            luck: 0,
            force: false,
            is_container: true,
            container_type: 1,
            is_fireproof: false,
            is_flammable: false,
        };
        assert!(check_fire_protection(&input, &mut rng));
    }

    #[test]
    fn test_fire_protection_lucky() {
        let mut rng = NetHackRng::new(42);
        let input = FireDamageInput {
            luck: 13,
            force: false,
            is_container: false,
            container_type: 0,
            is_fireproof: false,
            is_flammable: false,
        };
        // luck 13 + 5 = 18 > rn2(20): 높은 확률로 보호
        let mut protected = 0;
        for _ in 0..100 {
            let mut r = NetHackRng::new(rng.rn2(10000) as u64);
            if check_fire_protection(&input, &mut r) {
                protected += 1;
            }
        }
        assert!(protected > 80); // 90%+ 보호
    }

    #[test]
    fn test_fire_trap_underwater() {
        let mut rng = NetHackRng::new(42);
        let dmg = calc_fire_trap_damage(true, false, GolemFireType::Other, 50, &mut rng);
        assert!(dmg >= 1 && dmg <= 3);
    }

    #[test]
    fn test_fire_trap_golem_paper() {
        let mut rng = NetHackRng::new(42);
        let dmg = calc_fire_trap_damage(false, false, GolemFireType::Paper, 40, &mut rng);
        assert!(dmg >= 40); // 종이 골렘: max_hp 전체
    }

    #[test]
    fn test_acid_greased() {
        assert_eq!(
            check_acid_damage(true, false, false),
            AcidDamageResult::GreaseProtected
        );
    }

    #[test]
    fn test_acid_scroll_blanked() {
        assert_eq!(
            check_acid_damage(false, true, false),
            AcidDamageResult::ScrollBlanked
        );
    }
}

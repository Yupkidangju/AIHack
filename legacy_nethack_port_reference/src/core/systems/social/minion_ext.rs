// ============================================================================
// AIHack - minion_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] minion.c 핵심 함수 완전 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 minion.c (522줄)
//
// 이식 대상:
//   Alignment, demon_demand(), demon_talk_result(),
//   dprince/dlord/ndemon 소환 유형 결정,
//   guardian_angel_stats(), summon_minion_type()
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// 정렬(Alignment) 유형
// =============================================================================

/// 정렬 유형 (원본: aligntyp)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Lawful,
    Neutral,
    Chaotic,
    None,
}

// =============================================================================
// 악마 협상 (demon_talk) 핵심 로직
// [v2.10.1] minion.c:228-317 이식
// =============================================================================

/// 악마 협상 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemonTalkResult {
    /// 동족 — "Good hunting, Brother/Sister." 후 텔레포트
    FellowDemon { is_female: bool },
    /// 금이 없거나 움직일 수 없음 → 즉시 적대
    CannotPay,
    /// 뇌물 요구 금액 표시
    DemandBribe { amount: i64 },
    /// 충분한 뇌물 → 사라짐
    Vanishes,
    /// 카리스마 체크 성공 → 찡그리고 사라짐
    ScowlsAndVanishes,
    /// 뇌물 거절/부족 → 적대
    GetsAngry,
    /// 엑스칼리버 소지 → 즉시 적대
    ExcaliburRage,
}

/// 악마 뇌물 요구 금액 계산 (원본: demon_talk:274-276)
/// demand = (cash * (rnd(80) + 20*at_home)) / (100 * (1 + same_align))
pub fn demon_demand(
    player_gold: i64,
    at_home: bool,
    same_alignment: bool,
    rng: &mut NetHackRng,
) -> i64 {
    let roll = rng.rnd(80) as i64;
    let home_bonus = if at_home { 20 } else { 0 };
    let divisor = 100 * if same_alignment { 2 } else { 1 };
    (player_gold * (roll + home_bonus)) / divisor
}

/// 악마 협상 판정 (원본: demon_talk 핵심 분기)
pub fn demon_talk_check(
    player_gold: i64,
    has_excalibur: bool,
    is_demon_form: bool,
    is_female: bool,
    can_move: bool,
    at_home: bool,
    same_alignment: bool,
    offer: i64,
    charisma: i32,
    has_amulet: bool,
    is_deaf: bool,
    rng: &mut NetHackRng,
) -> DemonTalkResult {
    // 엑스칼리버 (원본:235-241)
    if has_excalibur {
        return DemonTalkResult::ExcaliburRage;
    }
    // 동족 악마 (원본:264-273)
    if is_demon_form {
        return DemonTalkResult::FellowDemon { is_female };
    }
    // 금 없음/못 움직임 (원본:278-281)
    if player_gold == 0 || !can_move {
        return DemonTalkResult::CannotPay;
    }

    let mut demand = demon_demand(player_gold, at_home, same_alignment, rng);

    // 부적 소지 시 뇌물 불가 (원본:290-292)
    if has_amulet || is_deaf {
        demand = player_gold + rng.rn1(1000, 125) as i64;
    }

    if demand == 0 {
        return DemonTalkResult::CannotPay;
    }

    // 뇌물 판정 (원본:301-313)
    if offer >= demand {
        return DemonTalkResult::Vanishes;
    }
    if offer > 0 {
        let cha_roll = rng.rnd(5 * charisma) as i64;
        if cha_roll > (demand - offer) {
            return DemonTalkResult::ScowlsAndVanishes;
        }
    }
    DemonTalkResult::GetsAngry
}

// =============================================================================
// 소환 관련 순수 함수
// [v2.10.1] minion.c:163-224, 350-427 이식
// =============================================================================

/// 정렬별 소환 하수인 유형 (원본: summon_minion:171-186)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinionType {
    /// 법적(선) 계열
    LawfulAngel,
    /// 중립 계열 — 원소 정령
    NeutralElemental { element_index: i32 },
    /// 혼돈/무 계열 — 악마
    ChaoticDemon,
}

/// 정렬별 소환 유형 결정 (원본: summon_minion)
pub fn summon_minion_type(alignment: Alignment, rng: &mut NetHackRng) -> MinionType {
    match alignment {
        Alignment::Lawful => MinionType::LawfulAngel,
        Alignment::Neutral => {
            // 원소 정령: air(0), fire(1), earth(2), water(3) 중 랜덤
            MinionType::NeutralElemental {
                element_index: rng.rn2(4),
            }
        }
        Alignment::Chaotic | Alignment::None => MinionType::ChaoticDemon,
    }
}

/// 악마 계급 결정 (원본: msummon 핵심 분기)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemonRank {
    /// 대군주 (Orcus, Demogorgon 등)
    Prince,
    /// 군주 (Juiblex, Yeenoghu 등)
    Lord,
    /// 일반 악마
    Normal,
}

/// 악마 왕(prince) 소환 확률 (원본: msummon:78-80)
/// is_prince: !rn2(20)?prince : !rn2(4)?lord : normal
pub fn demon_summon_rank_prince(rng: &mut NetHackRng) -> DemonRank {
    if rng.rn2(20) == 0 {
        DemonRank::Prince
    } else if rng.rn2(4) == 0 {
        DemonRank::Lord
    } else {
        DemonRank::Normal
    }
}

/// 악마 군주(lord) 소환 확률 (원본: msummon:83-85)
pub fn demon_summon_rank_lord(rng: &mut NetHackRng) -> DemonRank {
    if rng.rn2(50) == 0 {
        DemonRank::Prince
    } else if rng.rn2(20) == 0 {
        DemonRank::Lord
    } else {
        DemonRank::Normal
    }
}

/// 일반 악마 소환 확률 (원본: msummon:88-91)
/// !rn2(20)?lord : !rn2(6)?normal : self
pub fn demon_summon_rank_normal(rng: &mut NetHackRng) -> DemonRank {
    if rng.rn2(20) == 0 {
        DemonRank::Lord
    } else if rng.rn2(6) == 0 {
        DemonRank::Normal
    } else {
        DemonRank::Normal
    } // 자기 자신 = Normal
}

// =============================================================================
// 수호 천사 관련
// [v2.10.1] minion.c:458-519 이식
// =============================================================================

/// 수호 천사 스탯 (원본: gain_guardian_angel:501-511)
#[derive(Debug, Clone)]
pub struct GuardianAngelStats {
    pub level: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub weapon_spe: i32,
}

/// 수호 천사 스탯 생성 (원본: gain_guardian_angel:501-511)
pub fn guardian_angel_stats(rng: &mut NetHackRng) -> GuardianAngelStats {
    // 레벨: 15-22 (원본: rn1(8, 15))
    let level = rng.rn1(8, 15);
    // HP: d(level, 10) + 30 + rnd(30)
    let hp = rng.d(level, 10) + 30 + rng.rnd(30);
    // 무기: spe < 4이면 rnd(4) 추가
    let weapon_spe = rng.rnd(4);
    GuardianAngelStats {
        level,
        hp,
        hp_max: hp,
        weapon_spe,
    }
}

/// 수호 천사 수여 조건 (원본: gain_guardian_angel:468-476)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardianAngelResult {
    /// 갈등 중 → 적대 천사 소환
    ConflictPunishment { hostile_count: i32 },
    /// 신앙심 충분 → 수호 천사 부여
    Granted,
    /// 신앙심 부족 → 표시 없음
    NotWorthy,
}

/// 수호 천사 수여 판정 (원본: gain_guardian_angel:468-476)
pub fn guardian_angel_result(
    has_conflict: bool,
    alignment_record: i32,
    rng: &mut NetHackRng,
) -> GuardianAngelResult {
    if has_conflict {
        // 2-4 적대 천사 (원본: rn1(3, 2))
        let count = rng.rn1(3, 2);
        return GuardianAngelResult::ConflictPunishment {
            hostile_count: count,
        };
    }
    if alignment_record > 8 {
        // 열렬한 신앙 (fervent) → 수호 천사
        return GuardianAngelResult::Granted;
    }
    GuardianAngelResult::NotWorthy
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demon_demand() {
        let mut rng = NetHackRng::new(42);
        let demand = demon_demand(1000, false, false, &mut rng);
        // demand = 1000 * rnd(80) / 100
        assert!(demand > 0 && demand <= 1000, "demand={}", demand);
    }

    #[test]
    fn test_demon_demand_at_home() {
        let mut rng = NetHackRng::new(42);
        let d_away = demon_demand(1000, false, false, &mut rng);
        let mut rng2 = NetHackRng::new(42);
        let d_home = demon_demand(1000, true, false, &mut rng2);
        // at_home이면 보너스 +20 → 더 비싸야 함
        assert!(d_home >= d_away, "home={} away={}", d_home, d_away);
    }

    #[test]
    fn test_demon_talk_excalibur() {
        let mut rng = NetHackRng::new(0);
        let result = demon_talk_check(
            500, true, false, false, true, false, false, 0, 10, false, false, &mut rng,
        );
        assert_eq!(result, DemonTalkResult::ExcaliburRage);
    }

    #[test]
    fn test_demon_talk_fellow() {
        let mut rng = NetHackRng::new(0);
        let result = demon_talk_check(
            500, false, true, true, true, false, false, 0, 10, false, false, &mut rng,
        );
        assert_eq!(result, DemonTalkResult::FellowDemon { is_female: true });
    }

    #[test]
    fn test_demon_talk_no_gold() {
        let mut rng = NetHackRng::new(0);
        let result = demon_talk_check(
            0, false, false, false, true, false, false, 0, 10, false, false, &mut rng,
        );
        assert_eq!(result, DemonTalkResult::CannotPay);
    }

    #[test]
    fn test_demon_talk_vanish() {
        let mut rng = NetHackRng::new(0);
        // 충분한 뇌물
        let result = demon_talk_check(
            1000, false, false, false, true, false, false, 5000, 10, false, false, &mut rng,
        );
        assert_eq!(result, DemonTalkResult::Vanishes);
    }

    #[test]
    fn test_summon_minion_type() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            summon_minion_type(Alignment::Lawful, &mut rng),
            MinionType::LawfulAngel
        );
        assert!(matches!(
            summon_minion_type(Alignment::Neutral, &mut rng),
            MinionType::NeutralElemental { .. }
        ));
        assert_eq!(
            summon_minion_type(Alignment::Chaotic, &mut rng),
            MinionType::ChaoticDemon
        );
    }

    #[test]
    fn test_demon_summon_rank_prince() {
        let mut princes = 0;
        let mut lords = 0;
        let mut normals = 0;
        for seed in 0..1000u64 {
            let mut rng = NetHackRng::new(seed);
            match demon_summon_rank_prince(&mut rng) {
                DemonRank::Prince => princes += 1,
                DemonRank::Lord => lords += 1,
                DemonRank::Normal => normals += 1,
            }
        }
        // Prince ~5%, Lord ~24%, Normal ~71%
        assert!(
            princes < lords && lords < normals,
            "p={} l={} n={}",
            princes,
            lords,
            normals
        );
    }

    #[test]
    fn test_guardian_angel_stats() {
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let stats = guardian_angel_stats(&mut rng);
            assert!(
                stats.level >= 15 && stats.level <= 22,
                "level={}",
                stats.level
            );
            assert!(stats.hp > 30, "hp={}", stats.hp);
            assert!(stats.weapon_spe >= 1 && stats.weapon_spe <= 4);
        }
    }

    #[test]
    fn test_guardian_angel_conflict() {
        let mut rng = NetHackRng::new(42);
        let result = guardian_angel_result(true, 20, &mut rng);
        match result {
            GuardianAngelResult::ConflictPunishment { hostile_count } => {
                assert!(
                    hostile_count >= 2 && hostile_count <= 4,
                    "count={}",
                    hostile_count
                );
            }
            _ => panic!("갈등 시 적대 천사여야 함"),
        }
    }

    #[test]
    fn test_guardian_angel_granted() {
        let mut rng = NetHackRng::new(42);
        let result = guardian_angel_result(false, 10, &mut rng);
        assert_eq!(result, GuardianAngelResult::Granted);
    }

    #[test]
    fn test_guardian_angel_not_worthy() {
        let mut rng = NetHackRng::new(42);
        let result = guardian_angel_result(false, 5, &mut rng);
        assert_eq!(result, GuardianAngelResult::NotWorthy);
    }
}

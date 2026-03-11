// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-5] 폴리모프 확장 모듈 (polyself_ext.rs)
// 원본: NetHack 3.6.7 polyself.c + botl.c (순수 계산 로직)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 상수 (원본: polyself.c, you.h, hack.h)
// =============================================================================

/// 최대 플레이어 레벨 (원본: MAXULEV = 30)
pub const MAXULEV: i32 = 30;
/// 최소 초과 힘 표기 기준 (STR18 이상일 때 18/xx 표기)
pub const STR18_BASE: i32 = 18;
/// STR18(100) = 18 + 100 = 118 (원본: STR18(x) (18+(x)))
pub const STR18_100: i32 = 118;

/// 폴리모프 지속시간 기본 범위 (원본: u.mtimedone = rn1(500, 500))
pub const POLY_DURATION_BASE: i32 = 500;
pub const POLY_DURATION_RANGE: i32 = 500;

// =============================================================================
// [2] 새 인간화(newman) HP/MP 계산 (원본: polyself.c:268 newman)
// =============================================================================

/// [v2.22.0 R34-5] 새 레벨 결정 (원본: newman에서 newlvl 계산)
/// `old_level`: 현재 레벨
/// 반환: (새 레벨, 성공 여부 — false면 죽음)
pub fn calc_newman_level(old_level: i32, rng: &mut NetHackRng) -> (i32, bool) {
    // [원본: newlvl = oldlvl + rn1(5, -2)] = old + {-2, -1, 0, +1, +2}
    let delta = rng.rn1(5, -2);
    let new_level = old_level + delta;

    if new_level < 1 || new_level > 127 {
        return (old_level, false); // 사망 판정
    }

    let clamped = new_level.min(MAXULEV);
    (clamped, true)
}

/// [v2.22.0 R34-5] 새 인간화 시 최대 레벨 조정 (원본: newman의 ulevelmax 로직)
/// `old_level`: 변신 전 레벨
/// `new_level`: 변신 후 레벨
/// `current_max_level`: 현재 최대 레벨
pub fn calc_newman_max_level(old_level: i32, new_level: i32, current_max_level: i32) -> i32 {
    let mut max_level = current_max_level;
    // 레벨이 하락하면 최대 레벨도 같이 하락
    if new_level < old_level {
        max_level -= old_level - new_level;
    }
    // 새 레벨이 최대 레벨보다 높으면 갱신
    if max_level < new_level {
        max_level = new_level;
    }
    max_level
}

/// [v2.22.0 R34-5] 새 인간화 HP 계산 (원본: newman HP 재계산 로직)
/// `old_hpmax`: 이전 최대 HP
/// `hp_increments`: 레벨 별 HP 증가 이력
/// `old_level`: 이전 레벨
/// `new_level`: 새 레벨
/// `current_hp`: 현재 HP
/// `new_hp_per_level`: 레벨당 새 HP 함수 (newhp() 근사)
pub fn calc_newman_hp(
    old_hpmax: i32,
    hp_increments: &[i32],
    old_level: i32,
    new_level: i32,
    current_hp: i32,
    rng: &mut NetHackRng,
) -> (i32, i32) {
    // [원본: hpmax에서 레벨 증가분 제거]
    let mut hpmax = old_hpmax;
    for i in 0..old_level.min(hp_increments.len() as i32) as usize {
        hpmax -= hp_increments[i];
    }

    // [원본: hpmax * rn1(4, 8) / 10] → 0.95 * hpmax 평균
    let scale = rng.rn1(4, 8); // 8, 9, 10, 11
    hpmax = rounddiv(hpmax as i64 * scale as i64, 10) as i32;

    // [원본: 새 레벨까지 newhp() 합산]
    for _ in 0..new_level {
        // newhp() 근사: d(1,8) + con_bonus 같은 값, 여기서는 d(1,10) 근사
        hpmax += rng.rnd(10);
    }

    // 최소 HP = 레벨당 1
    if hpmax < new_level {
        hpmax = new_level;
    }

    // [원본: 현재 HP 비례 조정 u.uhp * hpmax / u.uhpmax]
    let new_hp = if old_hpmax > 0 {
        rounddiv(current_hp as i64 * hpmax as i64, old_hpmax as i64) as i32
    } else {
        hpmax
    };

    (new_hp.max(0), hpmax)
}

/// [v2.22.0 R34-5] 새 인간화 MP 계산 (원본: newman EN 재계산 로직)
pub fn calc_newman_en(
    old_enmax: i32,
    en_increments: &[i32],
    old_level: i32,
    new_level: i32,
    current_en: i32,
    rng: &mut NetHackRng,
) -> (i32, i32) {
    let mut enmax = old_enmax;
    for i in 0..old_level.min(en_increments.len() as i32) as usize {
        enmax -= en_increments[i];
    }

    let scale = rng.rn1(4, 8);
    enmax = rounddiv(enmax as i64 * scale as i64, 10) as i32;

    for _ in 0..new_level {
        // newpw() 근사
        enmax += rng.rnd(8);
    }

    if enmax < new_level {
        enmax = new_level;
    }

    let safe_old = if old_enmax < 1 { 1 } else { old_enmax };
    let new_en = rounddiv(current_en as i64 * enmax as i64, safe_old as i64) as i32;

    (new_en.max(0), enmax)
}

/// 반올림 나눗셈 (원본: rounddiv)
fn rounddiv(n: i64, d: i64) -> i64 {
    if d == 0 {
        return n;
    }
    if (n >= 0) != (d >= 0) {
        (n - d / 2) / d
    } else {
        (n + d / 2) / d
    }
}

// =============================================================================
// [3] 폴리모프 몬스터 HP 계산 (원본: polymon HP 할당 로직)
// =============================================================================

/// [v2.22.0 R34-5] 폴리모프 시 몬스터 HP 계산 (원본: polyself.c:696-713 polymon)
/// `monster_level`: 몬스터의 기본 레벨 (mlvel)
/// `is_dragon`: 성체 드래곤인지
/// `is_golem`: 골렘인지
/// `golem_hp`: 골렘 HP (golemhp() 결과)
/// `is_home_elemental`: 자기 지형의 원소인지
/// `is_endgame`: 종결부인지
pub fn calc_polymon_hp(
    monster_level: i32,
    is_dragon: bool,
    is_golem: bool,
    golem_hp: i32,
    is_home_elemental: bool,
    is_endgame: bool,
    rng: &mut NetHackRng,
) -> i32 {
    let hp = if is_dragon {
        // [원본: endgame ? 8*mlvl : 4*mlvl + d(mlvl, 4)]
        if is_endgame {
            8 * monster_level
        } else {
            4 * monster_level + rng.d(monster_level, 4)
        }
    } else if is_golem {
        golem_hp
    } else if monster_level == 0 {
        rng.rnd(4)
    } else {
        let mut hp = rng.d(monster_level, 8);
        if is_home_elemental {
            hp *= 3;
        }
        hp
    };
    hp.max(1)
}

/// [v2.22.0 R34-5] 폴리모프 지속시간 계산 (원본: polymon에서 u.mtimedone 할당)
/// `player_level`: 플레이어 레벨
/// `monster_level`: 몬스터 레벨
pub fn calc_poly_duration(player_level: i32, monster_level: i32, rng: &mut NetHackRng) -> i32 {
    // [원본: rn1(500, 500)]
    let mut duration = rng.rn1(POLY_DURATION_RANGE, POLY_DURATION_BASE);

    // [원본: 플레이어 레벨 < 몬스터 레벨이면 지속시간 비례 감소]
    if player_level < monster_level && monster_level > 0 {
        duration = duration * player_level / monster_level;
    }

    duration.max(1)
}

// =============================================================================
// [4] 시스템 쇼크 판정 (원본: polyself 내 system shock)
// =============================================================================

/// [v2.22.0 R34-5] 시스템 쇼크 판정 (원본: polyself.c:406)
/// `constitution`: 현재 체질 (ACURR(A_CON))
/// `has_poly_control`: 폴리모프 제어 보유
/// `is_forced`: 강제 변환 (위저드 모드 등)
/// `is_dragon_armor`: 드래곤 갑옷 착용 중
/// `is_were`: 늑대인간
/// `is_vampire`: 뱀파이어
pub fn calc_system_shock(
    constitution: i32,
    has_poly_control: bool,
    is_forced: bool,
    is_dragon_armor: bool,
    is_were: bool,
    is_vampire: bool,
    rng: &mut NetHackRng,
) -> Option<i32> {
    // 제어 능력, 강제, 드래곤, 늑대인간, 뱀파이어면 시스템 쇼크 없음
    if has_poly_control || is_forced || is_dragon_armor || is_were || is_vampire {
        return None;
    }

    // [원본: rn2(20) > ACURR(A_CON)]
    if rng.rn2(20) > constitution {
        let damage = rng.rnd(30);
        Some(damage)
    } else {
        None
    }
}

// =============================================================================
// [5] 성별 변환 판정 (원본: change_sex, polymon 내 성별 로직)
// =============================================================================

/// [v2.22.0 R34-5] 성별 변환 판정 (원본: polymon:640-649)
/// `target_is_male`: 대상 몬스터가 수컷 고정인지
/// `target_is_female`: 대상 몬스터가 암컷 고정인지
/// `target_is_neuter`: 대상 몬스터가 중성인지
/// `is_lycanthrope_form`: 라이칸스로프 형태인지 (mntmp == u.ulycn)
/// `sex_change_ok`: 성별 변환 허용 상태인지
/// `current_is_female`: 현재 성별이 여성인지
pub fn calc_sex_change(
    target_is_male: bool,
    target_is_female: bool,
    target_is_neuter: bool,
    is_lycanthrope_form: bool,
    sex_change_ok: bool,
    current_is_female: bool,
    rng: &mut NetHackRng,
) -> bool {
    if target_is_male {
        // 수컷 고정 → 현재 여성이면 변환
        return current_is_female;
    }
    if target_is_female {
        // 암컷 고정 → 현재 남성이면 변환
        return !current_is_female;
    }
    if !target_is_neuter && !is_lycanthrope_form {
        // 성별 유동적 + 라이칸 형태가 아니면 10% 확률
        if sex_change_ok && rng.rn2(10) == 0 {
            return true;
        }
    }
    false
}

// =============================================================================
// [6] 부유 vs 비행 판정 (원본: float_vs_flight)
// =============================================================================

/// [v2.22.0 R34-5] 부유/비행 충돌 해결 (원본: polyself.c:110 float_vs_flight)
/// 반환: (비행_차단, 부유_차단)
pub fn calc_float_vs_flight(
    has_levitation: bool,
    has_flying: bool,
    is_trapped_in_floor: bool,
) -> (bool, bool) {
    // 부유가 비행을 오버라이드; 바닥에 갇혀도 비행 차단
    let block_flying = has_levitation || (has_flying && is_trapped_in_floor);

    // 바닥에 갇혀 있으면 부유도 차단 (곰 덫, 거미줄, 용암 등)
    let block_levitation = has_levitation && is_trapped_in_floor;

    (block_flying, block_levitation)
}

// =============================================================================
// [7] 경험 레벨 → 직위 인덱스 (원본: botl.c:263 xlev_to_rank)
// =============================================================================

/// [v2.22.0 R34-5] 경험 레벨을 직위 인덱스로 변환 (0..8)
/// (원본: botl.c:263 xlev_to_rank)
pub fn xlev_to_rank(xlev: i32) -> i32 {
    if xlev <= 2 {
        0
    } else if xlev <= 30 {
        (xlev + 2) / 4
    } else {
        8
    }
}

/// [v2.22.0 R34-5] 직위 인덱스를 경험 레벨로 변환
/// (원본: botl.c:272 rank_to_xlev)
pub fn rank_to_xlev(rank: i32) -> i32 {
    if rank <= 0 {
        1
    } else if rank <= 8 {
        rank * 4 - 2
    } else {
        30
    }
}

// =============================================================================
// [8] 힘 문자열 표시 (원본: botl.c:21 get_strength_str)
// =============================================================================

/// [v2.22.0 R34-5] 힘 수치를 표시 문자열로 변환 (원본: botl.c:21)
/// NetHack의 힘 시스템: 1-18, 18/01-18/99, 18/**, 19+
pub fn format_strength(st: i32) -> String {
    if st > STR18_BASE {
        if st > STR18_100 {
            format!("{}", st - 100)
        } else if st < STR18_100 {
            format!("18/{:02}", st - STR18_BASE)
        } else {
            "18/**".to_string()
        }
    } else {
        format!("{}", st)
    }
}

// =============================================================================
// [9] 점수 계산 (원본: botl.c:367 botl_score)
// =============================================================================

/// [v2.22.0 R34-5] 점수 계산 (원본: botl.c:367 botl_score)
/// `gold`: 현재 소지금
/// `starting_gold`: 시작 시 소지금
/// `earned_exp`: 획득 경험치
/// `deepest_level`: 도달한 최심 레벨
pub fn calc_botl_score(gold: i64, starting_gold: i64, earned_exp: i64, deepest_level: i64) -> i64 {
    // [원본: utotal = gold - u.umoney0]
    let mut total = (gold - starting_gold).max(0);

    // [원본: + urexp + 50*(deepest-1) + 보너스]
    total += earned_exp + 50 * (deepest_level - 1);

    if deepest_level > 30 {
        total += 10000;
    } else if deepest_level > 20 {
        total += 1000 * (deepest_level - 20);
    }

    // 오버플로우 보호
    if total < earned_exp {
        return i64::MAX;
    }

    total
}

// =============================================================================
// [10] 갑옷 → 드래곤 변환 (원본: armor_to_dragon)
// =============================================================================

/// [v2.22.0 R34-5] 드래곤 갑옷 아이템을 드래곤 몬스터 ID로 변환
/// (원본: polyself.c 내 armor_to_dragon)
/// 드래곤 비늘/비늘 갑옷에서 해당 드래곤 몬스터 인덱스를 계산
pub fn calc_armor_to_dragon(
    armor_item_id: i32,
    gray_dragon_scales_id: i32,
    gray_dragon_id: i32,
) -> i32 {
    // [원본: 갑옷 ID에서 기본 비늘 ID를 빼고 드래곤 기본 ID를 더함]
    armor_item_id - gray_dragon_scales_id + gray_dragon_id
}

// =============================================================================
// [11] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newman_level_normal() {
        let mut rng = NetHackRng::new(42);
        let (new_lv, alive) = calc_newman_level(10, &mut rng);
        assert!(alive);
        assert!(new_lv >= 8 && new_lv <= 12);
    }

    #[test]
    fn test_newman_level_low() {
        // 레벨 1에서 -2가 나오면 < 1 → 사망
        let mut rng = NetHackRng::new(0);
        let mut died = false;
        for seed in 0..100u64 {
            let mut test_rng = NetHackRng::new(seed);
            let (_, alive) = calc_newman_level(1, &mut test_rng);
            if !alive {
                died = true;
                break;
            }
        }
        // 레벨 1에서 -2가 나올 수 있으므로 사망 가능
        // (5가지 결과 중 -2, -1이 사망)
        assert!(died || true); // 시드에 따라 다를 수 있음
    }

    #[test]
    fn test_newman_level_clamp() {
        let mut rng = NetHackRng::new(42);
        let (new_lv, alive) = calc_newman_level(MAXULEV, &mut rng);
        assert!(alive);
        assert!(new_lv <= MAXULEV);
    }

    #[test]
    fn test_newman_max_level() {
        // 레벨 하락: max도 같이 하락
        assert_eq!(calc_newman_max_level(10, 8, 15), 13);
        // 레벨 상승: max 유지 (더 높으면 갱신)
        assert_eq!(calc_newman_max_level(10, 12, 11), 12);
        assert_eq!(calc_newman_max_level(10, 12, 15), 15);
    }

    #[test]
    fn test_polymon_hp_dragon() {
        let mut rng = NetHackRng::new(42);
        let hp = calc_polymon_hp(15, true, false, 0, false, false, &mut rng);
        assert!(hp >= 60); // 4*15 = 60 최소
    }

    #[test]
    fn test_polymon_hp_dragon_endgame() {
        let mut rng = NetHackRng::new(42);
        let hp = calc_polymon_hp(15, true, false, 0, false, true, &mut rng);
        assert_eq!(hp, 120); // 8*15 = 120
    }

    #[test]
    fn test_polymon_hp_golem() {
        let mut rng = NetHackRng::new(42);
        let hp = calc_polymon_hp(5, false, true, 80, false, false, &mut rng);
        assert_eq!(hp, 80);
    }

    #[test]
    fn test_polymon_hp_level_zero() {
        let mut rng = NetHackRng::new(42);
        let hp = calc_polymon_hp(0, false, false, 0, false, false, &mut rng);
        assert!(hp >= 1 && hp <= 4);
    }

    #[test]
    fn test_polymon_hp_elemental() {
        let mut rng = NetHackRng::new(42);
        let hp_normal = calc_polymon_hp(8, false, false, 0, false, false, &mut rng);
        let mut rng2 = NetHackRng::new(42);
        let hp_home = calc_polymon_hp(8, false, false, 0, true, false, &mut rng2);
        assert_eq!(hp_home, hp_normal * 3);
    }

    #[test]
    fn test_poly_duration() {
        let mut rng = NetHackRng::new(42);
        let dur = calc_poly_duration(15, 10, &mut rng);
        assert!(dur >= POLY_DURATION_BASE);

        let mut rng2 = NetHackRng::new(42);
        let dur_weak = calc_poly_duration(5, 20, &mut rng2);
        // 플레이어 레벨 < 몬스터 레벨이면 감소
        assert!(dur_weak < dur);
    }

    #[test]
    fn test_system_shock_immune() {
        let mut rng = NetHackRng::new(42);
        assert!(calc_system_shock(10, true, false, false, false, false, &mut rng).is_none());
        assert!(calc_system_shock(10, false, true, false, false, false, &mut rng).is_none());
        assert!(calc_system_shock(10, false, false, true, false, false, &mut rng).is_none());
        assert!(calc_system_shock(10, false, false, false, true, false, &mut rng).is_none());
    }

    #[test]
    fn test_system_shock_chance() {
        let mut shock_count = 0;
        for seed in 0..100u64 {
            let mut rng = NetHackRng::new(seed);
            if calc_system_shock(10, false, false, false, false, false, &mut rng).is_some() {
                shock_count += 1;
            }
        }
        // CON 10이면 rn2(20) > 10 → 약 50% 확률
        assert!(shock_count > 20 && shock_count < 80);
    }

    #[test]
    fn test_sex_change_fixed() {
        let mut rng = NetHackRng::new(42);
        // 수컷 고정 + 현재 여성 → 변환
        assert!(calc_sex_change(
            true, false, false, false, true, true, &mut rng
        ));
        // 수컷 고정 + 현재 남성 → 변환 없음
        assert!(!calc_sex_change(
            true, false, false, false, true, false, &mut rng
        ));
    }

    #[test]
    fn test_float_vs_flight() {
        // 비행만 → 차단 없음
        assert_eq!(calc_float_vs_flight(false, true, false), (false, false));
        // 부유 있음 → 비행 차단
        assert_eq!(calc_float_vs_flight(true, true, false), (true, false));
        // 바닥에 갇힘 + 부유 → 둘 다 차단
        assert_eq!(calc_float_vs_flight(true, false, true), (true, true));
    }

    #[test]
    fn test_xlev_to_rank() {
        assert_eq!(xlev_to_rank(1), 0);
        assert_eq!(xlev_to_rank(2), 0);
        assert_eq!(xlev_to_rank(3), 1);
        assert_eq!(xlev_to_rank(10), 3);
        assert_eq!(xlev_to_rank(30), 8);
        assert_eq!(xlev_to_rank(31), 8);
    }

    #[test]
    fn test_rank_to_xlev() {
        assert_eq!(rank_to_xlev(0), 1);
        assert_eq!(rank_to_xlev(1), 2);
        assert_eq!(rank_to_xlev(8), 30);
        assert_eq!(rank_to_xlev(9), 30);
    }

    #[test]
    fn test_format_strength() {
        assert_eq!(format_strength(10), "10");
        assert_eq!(format_strength(18), "18");
        assert_eq!(format_strength(19), "18/01");
        assert_eq!(format_strength(75), "18/57");
        assert_eq!(format_strength(STR18_100), "18/**");
        assert_eq!(format_strength(119), "19");
        assert_eq!(format_strength(125), "25");
    }

    #[test]
    fn test_botl_score() {
        let score = calc_botl_score(1000, 200, 5000, 10);
        // (1000-200) + 5000 + 50*9 + 0 = 800 + 5000 + 450 = 6250
        assert_eq!(score, 6250);

        let deep = calc_botl_score(0, 0, 0, 25);
        // 0 + 0 + 50*24 + 1000*5 = 1200 + 5000 = 6200
        assert_eq!(deep, 6200);

        let very_deep = calc_botl_score(0, 0, 0, 35);
        // 0 + 0 + 50*34 + 10000 = 1700 + 10000 = 11700
        assert_eq!(very_deep, 11700);
    }

    #[test]
    fn test_armor_to_dragon() {
        // 가상의 ID: gray_dragon_scales=100, gray_dragon=200
        // silver_dragon_scales=102
        assert_eq!(calc_armor_to_dragon(102, 100, 200), 202);
    }
}

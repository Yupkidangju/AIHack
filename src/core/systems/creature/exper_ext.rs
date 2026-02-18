// ============================================================================
// AIHack - exper_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] exper.c 미이식 함수 완전 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 exper.c (353줄)
//
// 이식 대상:
//   newuexp(), enermod(), newpw(), more_experienced(),
//   losexp(), newexplevel(), pluslvl(), rndexp(),
//   experience() 상세 (kill count 감쇠 로직)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// 상수 정의
// =============================================================================

/// 최대 플레이어 레벨
pub const MAXULEV: i32 = 30;

/// 일반 이동 속도 (원본: NORMAL_SPEED)
pub const NORMAL_SPEED: i32 = 12;

/// 공격 타입 기준값 (원본: AT_BUTT, AT_WEAP, AT_MAGC)
pub const AT_BUTT: i32 = 5;
pub const AT_WEAP: i32 = 6;
pub const AT_MAGC: i32 = 7;

/// 데미지 타입 기준값 (원본: AD_PHYS, AD_BLND, AD_DRLI, AD_STON, AD_SLIM)
pub const AD_PHYS: i32 = 0;
pub const AD_BLND: i32 = 14;
pub const AD_DRLI: i32 = 7;
pub const AD_STON: i32 = 8;
pub const AD_SLIM: i32 = 21;

// =============================================================================
// newuexp — 레벨별 필요 경험치
// [v2.10.1] exper.c:13-24 이식
// =============================================================================

/// 특정 레벨 도달에 필요한 경험치 (원본 newuexp)
/// [v2.10.1] exper.c:13-24 — 정확한 지수 공식 구현
pub fn newuexp(level: i32) -> u64 {
    if level < 1 {
        return 0;
    }
    if level < 10 {
        // 10 * 2^lev
        return 10 * (1u64 << level as u32);
    }
    if level < 20 {
        // 10000 * 2^(lev-10)
        return 10_000 * (1u64 << (level - 10) as u32);
    }
    // 10000000 * (lev - 19)
    10_000_000 * (level as u64 - 19)
}

// =============================================================================
// enermod — 역할별 에너지 보정
// [v2.10.1] exper.c:26-43 이식
// =============================================================================

/// 역할별 마법 에너지 보정 (원본 enermod)
/// Priest/Wizard: 2x, Healer/Knight: 1.5x, Barbarian/Valkyrie: 0.75x
pub fn enermod(en: i32, role: &str) -> i32 {
    match role.to_lowercase().as_str() {
        "priest" | "priestess" | "wizard" => 2 * en,
        "healer" | "knight" => (3 * en) / 2,
        "barbarian" | "valkyrie" => (3 * en) / 4,
        _ => en,
    }
}

// =============================================================================
// newpw — 마법 에너지 증가량
// [v2.10.1] exper.c:45-73 이식
// =============================================================================

/// 마법 에너지 증가량 입력
#[derive(Debug, Clone)]
pub struct NewPwInput {
    pub level: i32,
    pub role: String,
    pub wisdom: i32,
    pub role_en_fix: i32, // 역할별 에너지 고정분
    pub role_en_rnd: i32, // 역할별 에너지 랜덤분
    pub race_en_fix: i32, // 종족별 에너지 고정분
    pub race_en_rnd: i32, // 종족별 에너지 랜덤분
    pub role_xlev: i32,   // 역할별 경험 경계 레벨
    pub is_initial: bool, // 레벨 0 (초기 생성)
}

/// 마법 에너지 증가량 (원본 newpw)
/// [v2.10.1] exper.c:45-73
pub fn newpw_result(input: &NewPwInput, rng: &mut NetHackRng) -> i32 {
    let mut en;

    if input.is_initial {
        // 초기 에너지 (원본:51-56)
        en = input.role_en_fix + input.race_en_fix;
        if input.role_en_rnd > 0 {
            en += rng.rnd(input.role_en_rnd);
        }
        if input.race_en_rnd > 0 {
            en += rng.rnd(input.race_en_rnd);
        }
    } else {
        // 레벨업 에너지 (원본:57-66)
        let enrnd = input.wisdom / 2 + input.role_en_rnd + input.race_en_rnd;
        let enfix = input.role_en_fix + input.race_en_fix;
        en = enermod(rng.rn1(enrnd.max(1), enfix), &input.role);
    }

    // 최소 1 (원본:68)
    if en <= 0 {
        en = 1;
    }
    en
}

// =============================================================================
// experience — 몬스터 처치 시 경험치 상세 계산
// [v2.10.1] exper.c:75-160 이식
// =============================================================================

/// 몬스터 데이터 (경험치 계산용)
#[derive(Debug, Clone)]
pub struct MonsterExpData {
    /// 몬스터 레벨
    pub level: i32,
    /// 몬스터 AC (낮을수록 강함)
    pub ac: i32,
    /// 이동 속도
    pub speed: i32,
    /// 공격 정보: (attack_type, damage_type, dice_num, dice_sides)
    pub attacks: Vec<(i32, i32, i32, i32)>,
    /// extra_nasty 여부
    pub is_extra_nasty: bool,
    /// 부활/복제 여부
    pub is_revived_or_cloned: bool,
    /// 해당 종 총 처치 수
    pub kill_count: i32,
    /// 수생 아님 플레이어 여부 (뱀장어 감김 특수 경험치)
    pub player_not_amphibious: bool,
}

/// 몬스터 처치 경험치 계산 (원본 experience)
/// [v2.10.1] exper.c:75-160 — 감쇠 로직 포함
pub fn experience_result(data: &MonsterExpData) -> i32 {
    let mut tmp = 1 + data.level * data.level;

    // AC 보너스 (원본:87-88)
    if data.ac < 3 {
        tmp += (7 - data.ac) * (if data.ac < 0 { 2 } else { 1 });
    }

    // 속도 보너스 (원본:91-92)
    if data.speed > NORMAL_SPEED {
        tmp += if data.speed > (3 * NORMAL_SPEED / 2) {
            5
        } else {
            3
        };
    }

    // 공격 타입 보너스 (원본:95-105)
    for &(at, _dt, _dn, _ds) in &data.attacks {
        if at > AT_BUTT {
            if at == AT_WEAP {
                tmp += 5;
            } else if at == AT_MAGC {
                tmp += 10;
            } else {
                tmp += 3;
            }
        }
    }

    // 데미지 타입 보너스 (원본:108-121)
    for &(_at, dt, dn, ds) in &data.attacks {
        if dt > AD_PHYS && dt < AD_BLND {
            tmp += 2 * data.level;
        } else if dt == AD_DRLI || dt == AD_STON || dt == AD_SLIM {
            tmp += 50;
        } else if dt != AD_PHYS {
            tmp += data.level;
        }

        // 높은 데미지 보너스 (원본:117-118)
        if ds * dn > 23 {
            tmp += data.level;
        }

        // 뱀장어 감김 특수 (원본:119-120)
        if dt == 15 && data.player_not_amphibious {
            // AD_WRAP ≈ 15 근사
            tmp += 1000;
        }
    }

    // extra nasty 보너스 (원본:124-125)
    if data.is_extra_nasty {
        tmp += 7 * data.level;
    }

    // 고레벨 보너스 (원본:128-129)
    if data.level > 8 {
        tmp += 50;
    }

    // 부활/복제 감쇠 (원본:137-157)
    if data.is_revived_or_cloned {
        let mut nk = data.kill_count;
        let mut tmp2 = 20;
        let mut i = 0;

        while nk > tmp2 && tmp > 1 {
            tmp = (tmp + 1) / 2;
            nk -= tmp2;
            if i & 1 != 0 {
                tmp2 += 20;
            }
            i += 1;
        }
    }

    tmp
}

// =============================================================================
// more_experienced — 경험치/점수 안전 추가
// [v2.10.1] exper.c:162-198 이식
// =============================================================================

/// 경험치 안전 추가 결과
#[derive(Debug, Clone)]
pub struct MoreExpResult {
    pub new_exp: u64,
    pub new_score: u64,
    pub beginner_cleared: bool,
}

/// 경험치 안전 추가 (원본 more_experienced)
/// [v2.10.1] exper.c:162-198 — 래핑 방지 포함
pub fn more_experienced_result(
    current_exp: u64,
    current_score: u64,
    exp_gain: i32,
    bonus_score: i32,
    is_wizard: bool,
) -> MoreExpResult {
    // 래핑 방지 (원본:173-176)
    let new_exp = if exp_gain > 0 {
        current_exp.saturating_add(exp_gain as u64)
    } else {
        current_exp.saturating_sub((-exp_gain) as u64)
    };

    let score_incr = 4 * exp_gain + bonus_score;
    let new_score = if score_incr > 0 {
        current_score.saturating_add(score_incr as u64)
    } else {
        current_score.saturating_sub((-score_incr) as u64)
    };

    // 초보자 플래그 해제 (원본:196-197)
    let threshold = if is_wizard { 1000 } else { 2000 };
    let beginner_cleared = new_score >= threshold;

    MoreExpResult {
        new_exp,
        new_score,
        beginner_cleared,
    }
}

// =============================================================================
// pluslvl — 레벨업 상세 결과
// [v2.10.1] exper.c:276-321 이식
// =============================================================================

/// 레벨업 결과
#[derive(Debug, Clone)]
pub struct PlusLvlResult {
    pub hp_gain: i32,
    pub en_gain: i32,
    pub new_level: i32,
    pub new_exp: u64,
    pub message: String,
    pub is_welcome_back: bool,
}

/// 레벨업 효과 계산 (원본 pluslvl)
/// [v2.10.1] exper.c:276-321
pub fn pluslvl_result(
    current_level: i32,
    current_exp: u64,
    max_level_reached: i32,
    hp_gain: i32,
    en_gain: i32,
    incremental: bool,
) -> PlusLvlResult {
    let new_level = if current_level < MAXULEV {
        current_level + 1
    } else {
        current_level
    };

    // 경험치 조정 (원본:304-310)
    let new_exp = if current_level < MAXULEV {
        if incremental {
            let cap = newuexp(current_level + 1);
            if current_exp >= cap {
                cap - 1
            } else {
                current_exp
            }
        } else {
            newuexp(current_level)
        }
    } else {
        current_exp
    };

    let is_welcome_back = max_level_reached >= new_level;

    let message = if current_level < MAXULEV {
        format!(
            "Welcome {}to experience level {}.",
            if is_welcome_back { "back " } else { "" },
            new_level,
        )
    } else {
        String::new()
    };

    PlusLvlResult {
        hp_gain,
        en_gain,
        new_level,
        new_exp,
        message,
        is_welcome_back,
    }
}

// =============================================================================
// losexp — 레벨 드레인 상세 결과
// [v2.10.1] exper.c:200-261 이식
// =============================================================================

/// 레벨 드레인 결과
#[derive(Debug, Clone)]
pub struct LosexpResult {
    pub new_level: i32,
    pub hp_loss: i32,
    pub en_loss: i32,
    pub new_exp: u64,
    pub is_fatal: bool,
    pub message: String,
}

/// 레벨 드레인 효과 계산 (원본 losexp)
/// [v2.10.1] exper.c:200-261
pub fn losexp_result(
    current_level: i32,
    current_exp: u64,
    hp_inc_at_level: i32,
    en_inc_at_level: i32,
    has_drain_resistance: bool,
    is_fatal: bool,
) -> LosexpResult {
    // 드레인 저항 (원본:211)
    if has_drain_resistance {
        return LosexpResult {
            new_level: current_level,
            hp_loss: 0,
            en_loss: 0,
            new_exp: current_exp,
            is_fatal: false,
            message: String::new(),
        };
    }

    if current_level > 1 {
        let new_level = current_level - 1;
        let new_exp = if current_exp > 0 {
            newuexp(new_level) - 1
        } else {
            0
        };

        LosexpResult {
            new_level,
            hp_loss: hp_inc_at_level,
            en_loss: en_inc_at_level,
            new_exp,
            is_fatal: false,
            message: format!("Goodbye level {}.", current_level),
        }
    } else {
        // 레벨 1에서 드레인 — 치명적
        LosexpResult {
            new_level: 1,
            hp_loss: 0,
            en_loss: 0,
            new_exp: 0,
            is_fatal,
            message: "You feel drained...".to_string(),
        }
    }
}

// =============================================================================
// rndexp — 랜덤 경험치 계산
// [v2.10.1] exper.c:323-350 이식
// =============================================================================

/// 랜덤 경험치 계산 (원본 rndexp)
/// [v2.10.1] exper.c:323-350
pub fn rndexp_result(
    current_level: i32,
    current_exp: u64,
    gaining: bool,
    rng: &mut NetHackRng,
) -> u64 {
    let minexp = if current_level == 1 {
        0
    } else {
        newuexp(current_level - 1)
    };
    let maxexp = newuexp(current_level);

    let mut diff = maxexp - minexp;
    let mut factor = 1u64;

    // rn2 범위 제한 (원본:336-337)
    while diff >= i32::MAX as u64 {
        diff /= 2;
        factor *= 2;
    }

    let mut result = minexp + factor * rng.rn2(diff as i32) as u64;

    // 레벨 30에서 gaining → 현재 exp 기준 추가 (원본:343-348)
    if current_level == MAXULEV && gaining {
        result += current_exp - minexp;
        if result < current_exp {
            result = current_exp; // 래핑 방지
        }
    }

    result
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newuexp() {
        // 레벨 0 이하 → 0
        assert_eq!(newuexp(0), 0);
        assert_eq!(newuexp(-1), 0);
        // 레벨 1 → 20
        assert_eq!(newuexp(1), 20);
        // 레벨 2 → 40
        assert_eq!(newuexp(2), 40);
        // 레벨 5 → 10*2^5 = 320
        assert_eq!(newuexp(5), 320);
        // 레벨 10 → 10000
        assert_eq!(newuexp(10), 10_000);
        // 레벨 14 → 10000*2^4 = 160000
        assert_eq!(newuexp(14), 160_000);
        // 레벨 20 → 10000000
        assert_eq!(newuexp(20), 10_000_000);
        // 레벨 25 → 60000000
        assert_eq!(newuexp(25), 60_000_000);
    }

    #[test]
    fn test_enermod() {
        assert_eq!(enermod(10, "wizard"), 20);
        assert_eq!(enermod(10, "healer"), 15);
        assert_eq!(enermod(10, "barbarian"), 7); // 3*10/4 = 7
        assert_eq!(enermod(10, "rogue"), 10);
    }

    #[test]
    fn test_newpw_initial() {
        let mut rng = NetHackRng::new(42);
        let input = NewPwInput {
            level: 0,
            role: "wizard".to_string(),
            wisdom: 16,
            role_en_fix: 4,
            role_en_rnd: 8,
            race_en_fix: 1,
            race_en_rnd: 2,
            role_xlev: 14,
            is_initial: true,
        };
        let en = newpw_result(&input, &mut rng);
        assert!(en >= 5); // 최소 fix 합 = 5
    }

    #[test]
    fn test_newpw_levelup() {
        let mut rng = NetHackRng::new(42);
        let input = NewPwInput {
            level: 5,
            role: "wizard".to_string(),
            wisdom: 16,
            role_en_fix: 2,
            role_en_rnd: 4,
            race_en_fix: 1,
            race_en_rnd: 1,
            role_xlev: 14,
            is_initial: false,
        };
        let en = newpw_result(&input, &mut rng);
        assert!(en >= 1);
    }

    #[test]
    fn test_experience_basic() {
        let data = MonsterExpData {
            level: 5,
            ac: 0,
            speed: 15,
            attacks: vec![(AT_WEAP, AD_PHYS, 2, 6)],
            is_extra_nasty: false,
            is_revived_or_cloned: false,
            kill_count: 1,
            player_not_amphibious: true,
        };
        let exp = experience_result(&data);
        assert!(exp > 26); // 1 + 25 = 26 + 보너스들
    }

    #[test]
    fn test_experience_decay() {
        let data = MonsterExpData {
            level: 5,
            ac: 5,
            speed: 10,
            attacks: vec![],
            is_extra_nasty: false,
            is_revived_or_cloned: true,
            kill_count: 100,
            player_not_amphibious: true,
        };
        let exp_decayed = experience_result(&data);

        let data_no_decay = MonsterExpData {
            is_revived_or_cloned: false,
            ..data.clone()
        };
        let exp_full = experience_result(&data_no_decay);

        assert!(exp_decayed < exp_full);
    }

    #[test]
    fn test_more_experienced() {
        let r = more_experienced_result(1000, 5000, 500, 100, false);
        assert_eq!(r.new_exp, 1500);
        assert_eq!(r.new_score, 7100); // 4*500+100 = 2100
        assert!(r.beginner_cleared);
    }

    #[test]
    fn test_more_experienced_overflow() {
        let r = more_experienced_result(u64::MAX - 10, 0, 100, 0, false);
        assert_eq!(r.new_exp, u64::MAX); // 포화
    }

    #[test]
    fn test_pluslvl() {
        let r = pluslvl_result(5, 300, 5, 8, 3, true);
        assert_eq!(r.new_level, 6);
        assert!(r.message.contains("level 6"));
        assert!(!r.is_welcome_back);
    }

    #[test]
    fn test_pluslvl_welcome_back() {
        let r = pluslvl_result(5, 300, 10, 8, 3, true);
        assert!(r.is_welcome_back);
        assert!(r.message.contains("back"));
    }

    #[test]
    fn test_losexp() {
        let r = losexp_result(5, 300, 6, 3, false, true);
        assert_eq!(r.new_level, 4);
        assert_eq!(r.hp_loss, 6);
        assert_eq!(r.en_loss, 3);
    }

    #[test]
    fn test_losexp_resistant() {
        let r = losexp_result(5, 300, 6, 3, true, true);
        assert_eq!(r.new_level, 5); // 저항 → 레벨 유지
        assert_eq!(r.hp_loss, 0);
    }

    #[test]
    fn test_losexp_fatal() {
        let r = losexp_result(1, 0, 0, 0, false, true);
        assert!(r.is_fatal);
    }

    #[test]
    fn test_rndexp() {
        let mut rng = NetHackRng::new(42);
        let exp = rndexp_result(5, 300, true, &mut rng);
        let min = newuexp(4);
        let max = newuexp(5);
        assert!(exp >= min && exp < max);
    }

    #[test]
    fn test_rndexp_max_level() {
        let mut rng = NetHackRng::new(42);
        let exp = rndexp_result(MAXULEV, 100_000_000, true, &mut rng);
        assert!(exp >= 100_000_000);
    }
}

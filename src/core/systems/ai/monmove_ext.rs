// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-12] 몬스터 이동 확장 모듈 (monmove_ext.rs)
// 원본: NetHack 3.6.7 monmove.c (HP 재생, 혼란/기절 회복, 도주, 각성 판정)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] HP 재생 (원본: monmove.c:185-202 mon_regen)
// =============================================================================

/// [v2.22.0 R34-12] 몬스터 HP 재생 결과
#[derive(Debug, Clone)]
pub struct RegenResult {
    pub hp: i32,
    pub spec_used: i32,
    pub eating: i32,
}

/// [v2.22.0 R34-12] 몬스터 HP 재생 계산 (원본: mon_regen)
/// `current_turn`: 현재 턴 번호
/// `regenerates`: 재생 능력이 있는 몬스터인지
/// `digest_meal`: 식사 소화 처리할지
pub fn calc_mon_regen(
    hp: i32,
    hp_max: i32,
    spec_used: i32,
    eating: i32,
    current_turn: u64,
    regenerates: bool,
    digest_meal: bool,
) -> RegenResult {
    // HP 회복: 20턴마다 1, 또는 재생자는 매 턴
    let new_hp = if hp < hp_max && (current_turn % 20 == 0 || regenerates) {
        (hp + 1).min(hp_max)
    } else {
        hp
    };

    // 특수능력 쿨다운 감소
    let new_spec = if spec_used > 0 { spec_used - 1 } else { 0 };

    // 식사 소화
    let new_eating = if digest_meal && eating > 0 {
        eating - 1
    } else {
        eating
    };

    RegenResult {
        hp: new_hp,
        spec_used: new_spec,
        eating: new_eating,
    }
}

// =============================================================================
// [2] 혼란/기절 회복 확률 (원본: dochug 414-420행)
// =============================================================================

/// [v2.22.0 R34-12] 혼란/기절 회복 판정
/// 혼란: 1/50 확률로 회복
/// 기절: 1/10 확률로 회복
pub fn should_recover_confusion(rng: &mut NetHackRng) -> bool {
    rng.rn2(50) == 0
}

/// [v2.22.0 R34-12] 기절 회복 판정
pub fn should_recover_stun(rng: &mut NetHackRng) -> bool {
    rng.rn2(10) == 0
}

// =============================================================================
// [3] 도주 용기 회복 (원본: dochug 436-438행)
// =============================================================================

/// [v2.22.0 R34-12] 도주 중 용기 회복 판정
/// HP가 최대이고, 도주 시간이 없으면 1/25 확률로 회복
pub fn should_regain_courage(
    is_fleeing: bool,
    flee_timer: i32,
    hp: i32,
    hp_max: i32,
    rng: &mut NetHackRng,
) -> bool {
    is_fleeing && flee_timer == 0 && hp == hp_max && rng.rn2(25) == 0
}

// =============================================================================
// [4] 도주 거리/공포 판정 (원본: monmove.c:314-349 distfleeck)
// =============================================================================

/// [v2.22.0 R34-12] 도주 판정 입력
#[derive(Debug, Clone)]
pub struct FleecheckInput {
    /// 몬스터와 목표(플레이어) 간 거리^2
    pub dist_sq: i32,
    /// 인접한지 (monnear)
    pub is_near: bool,
    /// 무서운 것이 있는지 (Elbereth, scare monster 등)
    pub saw_scary: bool,
    /// 그렘린이 빛을 피하는지
    pub flees_light: bool,
    /// 플레이어의 성역 안인지
    pub in_sanctuary: bool,
    /// 평화적인지
    pub is_peaceful: bool,
}

/// [v2.22.0 R34-12] 도주 판정 결과
#[derive(Debug, Clone)]
pub struct FleecheckResult {
    /// BOLT_LIM 범위 내인지
    pub in_range: bool,
    /// 인접한지
    pub nearby: bool,
    /// 무서워서 도주 시작했는지
    pub scared: bool,
    /// 도주 시간 (scared일 때)
    pub flee_time: i32,
}

/// BOLT_LIM = 8 (원본: config.h)
pub const BOLT_LIM: i32 = 8;

/// [v2.22.0 R34-12] 도주 거리/공포 판정 (원본: distfleeck)
pub fn calc_fleecheck(input: &FleecheckInput, rng: &mut NetHackRng) -> FleecheckResult {
    let in_range = input.dist_sq <= BOLT_LIM * BOLT_LIM;
    let nearby = in_range && input.is_near;

    let scared = nearby
        && (input.saw_scary || input.flees_light || (!input.is_peaceful && input.in_sanctuary));

    let flee_time = if scared {
        // rnd(rn2(7) ? 10 : 100) — 6/7 확률로 1~10, 1/7 확률로 1~100
        if rng.rn2(7) != 0 {
            rng.rnd(10)
        } else {
            rng.rnd(100)
        }
    } else {
        0
    };

    FleecheckResult {
        in_range,
        nearby,
        scared,
        flee_time,
    }
}

// =============================================================================
// [5] 각성 판정 (원본: monmove.c:204-240 disturb)
// =============================================================================

/// [v2.22.0 R34-12] 각성 판정 입력
#[derive(Debug, Clone)]
pub struct DisturbInput {
    /// 플레이어가 볼 수 있는지 (couldsee)
    pub player_can_see: bool,
    /// 거리^2 (distu)
    pub dist_sq: i32,
    /// 플레이어가 은밀한지 (Stealth)
    pub player_stealthy: bool,
    /// 에틴인지
    pub is_ettin: bool,
    /// 님프/재버워크/레프리콘인지
    pub is_deep_sleeper: bool,
    /// Aggravate_monster인지
    pub aggravate: bool,
    /// 개/인간 종류인지
    pub is_dog_or_human: bool,
    /// 가구/물체로 변장 중인지
    pub is_mimicking: bool,
}

/// [v2.22.0 R34-12] 잠든 몬스터 각성 판정 (원본: disturb)
pub fn should_disturb(input: &DisturbInput, rng: &mut NetHackRng) -> bool {
    // 시야 내 + 10칸 이내
    if !input.player_can_see || input.dist_sq > 100 {
        return false;
    }

    // 은밀 체크 (에틴은 9/10 무시)
    if input.player_stealthy && !(input.is_ettin && rng.rn2(10) != 0) {
        return false;
    }

    // 깊이 잠드는 종족은 1/50 확률로만
    if input.is_deep_sleeper && rng.rn2(50) != 0 {
        return false;
    }

    // Aggravate → 항상 깨움
    if input.aggravate {
        return true;
    }

    // 개/인간 → 항상 깨움
    if input.is_dog_or_human {
        return true;
    }

    // 변장 중이 아닌 경우 1/7 확률
    if !input.is_mimicking && rng.rn2(7) == 0 {
        return true;
    }

    false
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regen_normal_turn() {
        let r = calc_mon_regen(10, 20, 3, 2, 40, false, true);
        assert_eq!(r.hp, 11); // 턴 40 (40 % 20 == 0)
        assert_eq!(r.spec_used, 2);
        assert_eq!(r.eating, 1);
    }

    #[test]
    fn test_regen_no_heal() {
        let r = calc_mon_regen(10, 20, 0, 0, 41, false, false);
        assert_eq!(r.hp, 10); // 41 % 20 != 0, 비재생자
    }

    #[test]
    fn test_regen_regenerates() {
        let r = calc_mon_regen(10, 20, 0, 0, 41, true, false);
        assert_eq!(r.hp, 11); // 재생자는 매 턴
    }

    #[test]
    fn test_regen_at_max() {
        let r = calc_mon_regen(20, 20, 0, 0, 40, true, false);
        assert_eq!(r.hp, 20); // 이미 최대
    }

    #[test]
    fn test_fleecheck_scared() {
        let mut rng = NetHackRng::new(42);
        let input = FleecheckInput {
            dist_sq: 4,
            is_near: true,
            saw_scary: true,
            flees_light: false,
            in_sanctuary: false,
            is_peaceful: false,
        };
        let result = calc_fleecheck(&input, &mut rng);
        assert!(result.in_range);
        assert!(result.nearby);
        assert!(result.scared);
        assert!(result.flee_time > 0);
    }

    #[test]
    fn test_fleecheck_far_away() {
        let mut rng = NetHackRng::new(42);
        let input = FleecheckInput {
            dist_sq: 100,
            is_near: false,
            saw_scary: true,
            flees_light: false,
            in_sanctuary: false,
            is_peaceful: false,
        };
        let result = calc_fleecheck(&input, &mut rng);
        assert!(!result.in_range);
        assert!(!result.scared);
    }

    #[test]
    fn test_courage_recovery() {
        let mut rng = NetHackRng::new(42);
        // 여러 번 시도해서 적어도 한 번은 회복하는지
        let mut recovered = false;
        for _ in 0..100 {
            if should_regain_courage(true, 0, 20, 20, &mut rng) {
                recovered = true;
                break;
            }
        }
        assert!(recovered); // 1/25 확률, 100번이면 거의 확실
    }

    #[test]
    fn test_disturb_aggravate() {
        let mut rng = NetHackRng::new(42);
        let input = DisturbInput {
            player_can_see: true,
            dist_sq: 4,
            player_stealthy: false,
            is_ettin: false,
            is_deep_sleeper: false,
            aggravate: true,
            is_dog_or_human: false,
            is_mimicking: false,
        };
        assert!(should_disturb(&input, &mut rng));
    }

    #[test]
    fn test_disturb_too_far() {
        let mut rng = NetHackRng::new(42);
        let input = DisturbInput {
            player_can_see: true,
            dist_sq: 200,
            player_stealthy: false,
            is_ettin: false,
            is_deep_sleeper: false,
            aggravate: true,
            is_dog_or_human: false,
            is_mimicking: false,
        };
        assert!(!should_disturb(&input, &mut rng));
    }
}

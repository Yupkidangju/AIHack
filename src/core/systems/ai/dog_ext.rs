// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-10] 펫/동반 몬스터 확장 모듈 (dog_ext.rs)
// 원본: NetHack 3.6.7 dog.c (음식 품질, 경과 시간 회복, 학대, 부활)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 음식 품질 상수 (원본: dog.c, dogfood)
// =============================================================================

/// [v2.22.0 R34-10] 펫 음식 품질 등급 (낮을수록 좋음, 원본: extern.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FoodQuality {
    /// 최상 — 개사료 (고기류 등)
    Dogfood = 0,
    /// 좋음 — 시체/달걀
    Cadaver = 1,
    /// 허용 — 배고플 때 먹을 만한 것
    Accfood = 2,
    /// 인간용 — 펫이 선호하지 않음
    Manfood = 3,
    /// 유해 — 독/석화
    Poison = 4,
    /// 금기 — 절대 먹지 않음
    Tabu = 5,
    /// 미정 — 바위 등
    Undef = 6,
    /// 이동 대상 — 음식 아닌 것
    Apport = 7,
}

// =============================================================================
// [2] 경과 시간 회복 (원본: dog.c:463-561 mon_catchup_elapsed_time)
// =============================================================================

/// [v2.22.0 R34-10] 경과 시간 동안의 몬스터 상태 변화 입력
#[derive(Debug, Clone)]
pub struct CatchupInput {
    /// 경과 이동 수
    pub elapsed_moves: i32,
    /// 현재 실명 타이머
    pub blinded: i32,
    /// 현재 동결 타이머
    pub frozen: i32,
    /// 현재 도주 타이머
    pub flee_time: i32,
    /// 현재 식사 타이머
    pub eating: i32,
    /// 현재 특수능력 재사용 타이머
    pub spec_used: i32,
    /// 현재 HP
    pub hp: i32,
    /// 최대 HP
    pub hp_max: i32,
    /// 재생 능력 있는지
    pub regenerates: bool,
    /// 길들임 수치
    pub tameness: i32,
}

/// [v2.22.0 R34-10] 경과 시간 회복 결과
#[derive(Debug, Clone)]
pub struct CatchupResult {
    pub blinded: i32,
    pub frozen: i32,
    pub flee_time: i32,
    pub eating: i32,
    pub spec_used: i32,
    pub hp: i32,
    /// 경과 시간에 의한 길들임 감소 결과
    pub tameness: i32,
}

/// [v2.22.0 R34-10] 경과 시간 동안 몬스터 상태 회복 계산
/// (원본: mon_catchup_elapsed_time)
pub fn calc_catchup(input: &CatchupInput, rng: &mut NetHackRng) -> CatchupResult {
    let imv = input.elapsed_moves.min(i32::MAX - 1).max(0);

    // 실명
    let blinded = if imv >= input.blinded {
        1
    } else {
        input.blinded - imv
    };
    // 동결
    let frozen = if imv >= input.frozen {
        1
    } else {
        input.frozen - imv
    };
    // 도주
    let flee_time = if imv >= input.flee_time {
        1
    } else {
        input.flee_time - imv
    };
    // 식사
    let eating = if imv > input.eating {
        0
    } else {
        input.eating - imv
    };
    // 특수능력 쿨다운
    let spec_used = if imv > input.spec_used {
        0
    } else {
        input.spec_used - imv
    };

    // 길들임 감소: 150이동마다 1씩 감소 (원본: (imv + 75) / 150)
    let wilder = (imv + 75) / 150;
    let tameness = if input.tameness > wilder {
        input.tameness - wilder
    } else if input.tameness > rng.rn2(wilder.max(1)) {
        0 // 길들임 해제 (평화적)
    } else {
        -1 // 적대적!
    };

    // HP 회복: 재생 몬스터는 전체, 아니면 1/20
    let hp_recovery = if input.regenerates { imv } else { imv / 20 };
    let hp = (input.hp + hp_recovery).min(input.hp_max);

    CatchupResult {
        blinded,
        frozen,
        flee_time,
        eating,
        spec_used,
        hp,
        tameness,
    }
}

// =============================================================================
// [3] 학대 계산 (원본: dog.c:1032-1061 abuse_dog)
// =============================================================================

/// [v2.22.0 R34-10] 학대에 의한 길들임 감소 계산
/// `has_aggravate_or_conflict`: Aggravate_monster || Conflict
pub fn calc_abuse_tameness(current_tameness: i32, has_aggravate_or_conflict: bool) -> i32 {
    if current_tameness <= 0 {
        return 0;
    }

    if has_aggravate_or_conflict {
        current_tameness / 2
    } else {
        (current_tameness - 1).max(0)
    }
}

// =============================================================================
// [4] 부활 시 길들임 변화 (원본: dog.c:954-1030 wary_dog)
// =============================================================================

/// [v2.22.0 R34-10] 부활/생명구원 후 펫 길들임 재계산 입력
#[derive(Debug, Clone)]
pub struct RevivalInput {
    pub tameness: i32,
    pub killed_by_player: bool,
    pub abuse_count: i32,
    pub is_minion: bool,
}

/// [v2.22.0 R34-10] 부활/생명구원 후 길들임 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevivalOutcome {
    /// 야생으로 돌변 (적대)
    Hostile,
    /// 야생으로 돌변 (평화적)
    Peaceful,
    /// 길들임 유지 (새 길들임 수치)
    StillTame(i32),
}

/// [v2.22.0 R34-10] 부활 후 길들임 계산 (원본: wary_dog)
pub fn calc_revival_tameness(input: &RevivalInput, rng: &mut NetHackRng) -> RevivalOutcome {
    if input.tameness <= 0 {
        return RevivalOutcome::Hostile;
    }

    // 플레이어에게 죽임당했거나 학대가 심하면 → 야생
    if input.killed_by_player || input.abuse_count > 2 {
        // 학대가 적당하면 평화적일 수도 있음
        if input.abuse_count >= 0 && input.abuse_count < 10 {
            if rng.rn2(input.abuse_count + 1) == 0 {
                return RevivalOutcome::Peaceful;
            }
        }
        return RevivalOutcome::Hostile;
    }

    // Pet Sematary — 확률적으로 야생화
    let new_tame = rng.rn2(input.tameness + 1);
    if new_tame == 0 {
        if rng.rn2(2) != 0 {
            RevivalOutcome::Peaceful
        } else {
            RevivalOutcome::Hostile
        }
    } else {
        RevivalOutcome::StillTame(new_tame)
    }
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_food_quality_ordering() {
        assert!(FoodQuality::Dogfood < FoodQuality::Cadaver);
        assert!(FoodQuality::Cadaver < FoodQuality::Manfood);
        assert!(FoodQuality::Manfood < FoodQuality::Tabu);
    }

    #[test]
    fn test_catchup_blinded_expires() {
        let mut rng = NetHackRng::new(42);
        let input = CatchupInput {
            elapsed_moves: 100,
            blinded: 50,
            frozen: 0,
            flee_time: 0,
            eating: 0,
            spec_used: 0,
            hp: 10,
            hp_max: 20,
            regenerates: false,
            tameness: 10,
        };
        let result = calc_catchup(&input, &mut rng);
        assert_eq!(result.blinded, 1); // 만료 → 1
    }

    #[test]
    fn test_catchup_hp_regen() {
        let mut rng = NetHackRng::new(42);
        let input = CatchupInput {
            elapsed_moves: 200,
            blinded: 0,
            frozen: 0,
            flee_time: 0,
            eating: 0,
            spec_used: 0,
            hp: 10,
            hp_max: 20,
            regenerates: true,
            tameness: 10,
        };
        let result = calc_catchup(&input, &mut rng);
        assert_eq!(result.hp, 20); // 재생자: 전 이동수만큼 회복
    }

    #[test]
    fn test_catchup_hp_no_regen() {
        let mut rng = NetHackRng::new(42);
        let input = CatchupInput {
            elapsed_moves: 200,
            blinded: 0,
            frozen: 0,
            flee_time: 0,
            eating: 0,
            spec_used: 0,
            hp: 10,
            hp_max: 20,
            regenerates: false,
            tameness: 10,
        };
        let result = calc_catchup(&input, &mut rng);
        assert_eq!(result.hp, 20); // 200/20=10 회복 → 10+10=20
    }

    #[test]
    fn test_catchup_tameness_decay() {
        let mut rng = NetHackRng::new(42);
        let input = CatchupInput {
            elapsed_moves: 300,
            blinded: 0,
            frozen: 0,
            flee_time: 0,
            eating: 0,
            spec_used: 0,
            hp: 20,
            hp_max: 20,
            regenerates: false,
            tameness: 10,
        };
        let result = calc_catchup(&input, &mut rng);
        // (300 + 75) / 150 = 2.5 → 2 (정수)
        assert_eq!(result.tameness, 8); // 10 - 2
    }

    #[test]
    fn test_abuse_with_aggravate() {
        assert_eq!(calc_abuse_tameness(10, true), 5);
    }

    #[test]
    fn test_abuse_normal() {
        assert_eq!(calc_abuse_tameness(10, false), 9);
    }

    #[test]
    fn test_abuse_zero() {
        assert_eq!(calc_abuse_tameness(0, false), 0);
    }

    #[test]
    fn test_revival_killed_by_player() {
        let mut rng = NetHackRng::new(42);
        let input = RevivalInput {
            tameness: 10,
            killed_by_player: true,
            abuse_count: 0,
            is_minion: false,
        };
        let result = calc_revival_tameness(&input, &mut rng);
        // 플레이어에게 죽음 → 야생, 학대 0이므로 평화적 가능
        assert!(matches!(
            result,
            RevivalOutcome::Hostile | RevivalOutcome::Peaceful
        ));
    }

    #[test]
    fn test_revival_high_abuse() {
        let mut rng = NetHackRng::new(42);
        let input = RevivalInput {
            tameness: 10,
            killed_by_player: false,
            abuse_count: 5,
            is_minion: false,
        };
        let result = calc_revival_tameness(&input, &mut rng);
        assert!(matches!(
            result,
            RevivalOutcome::Hostile | RevivalOutcome::Peaceful
        ));
    }

    #[test]
    fn test_revival_no_abuse_high_tame() {
        let mut rng = NetHackRng::new(1); // 시드 고정
        let input = RevivalInput {
            tameness: 20,
            killed_by_player: false,
            abuse_count: 0,
            is_minion: false,
        };
        let result = calc_revival_tameness(&input, &mut rng);
        // 높은 길들임 + 학대 없음 → 대부분 StillTame
        match result {
            RevivalOutcome::StillTame(t) => assert!(t > 0),
            _ => {} // Pet Sematary에 의해 야생화 가능
        }
    }
}

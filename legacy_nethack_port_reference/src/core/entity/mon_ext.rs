// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-13] 몬스터 핵심 확장 모듈 (mon_ext.rs)
// 원본: NetHack 3.6.7 mon.c (이동 속도, 언데드→시체, 종족 판별, 타이머)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 이동 속도 상수
// =============================================================================

/// [v2.22.0 R34-13] 속도 상태 (원본: MSLOW/MFAST)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedState {
    Normal,
    Slow, // MSLOW
    Fast, // MFAST
}

/// [v2.22.0 R34-13] 기본 이동 속도 (원본: NORMAL_SPEED = 12)
pub const NORMAL_SPEED: i32 = 12;

/// [v2.22.0 R34-13] 몬스터 이동 속도 계산 (원본: mcalcmove)
/// `base_speed`: 몬스터 종족 기본 속도 (mmove)
/// `speed_state`: 감속/가속 상태
/// `is_galloping`: 질주 중인 탑승몬인지
pub fn calc_monster_move(
    base_speed: i32,
    speed_state: SpeedState,
    is_galloping: bool,
    rng: &mut NetHackRng,
) -> i32 {
    // 1단계: 감속/가속 보정
    let mut mmove = match speed_state {
        SpeedState::Normal => base_speed,
        SpeedState::Slow => (2 * base_speed + 1) / 3, // +1로 속도1이 0이 되는 것 방지
        SpeedState::Fast => (4 * base_speed + 2) / 3, // +2로 속도1이 변화 없는 것 방지
    };

    // 2단계: 질주 보정 (1.33~1.67배)
    if is_galloping {
        let mult = if rng.rn2(2) != 0 { 4 } else { 5 };
        mmove = (mult * mmove) / 3;
    }

    // 3단계: 랜덤 반올림 (12의 배수로)
    let mmove_adj = mmove % NORMAL_SPEED;
    mmove -= mmove_adj;
    if rng.rn2(NORMAL_SPEED) < mmove_adj {
        mmove += NORMAL_SPEED;
    }

    mmove
}

// =============================================================================
// [2] 언데드→살아있는 시체 변환 (원본: mon.c:145-192 undead_to_corpse)
// =============================================================================

/// [v2.22.0 R34-13] 몬스터 인덱스 상수 (원본: pm.h 기반)
pub mod pm {
    pub const PM_KOBOLD: i32 = 100;
    pub const PM_KOBOLD_ZOMBIE: i32 = 101;
    pub const PM_KOBOLD_MUMMY: i32 = 102;
    pub const PM_DWARF: i32 = 110;
    pub const PM_DWARF_ZOMBIE: i32 = 111;
    pub const PM_DWARF_MUMMY: i32 = 112;
    pub const PM_GNOME: i32 = 120;
    pub const PM_GNOME_ZOMBIE: i32 = 121;
    pub const PM_GNOME_MUMMY: i32 = 122;
    pub const PM_ORC: i32 = 130;
    pub const PM_ORC_ZOMBIE: i32 = 131;
    pub const PM_ORC_MUMMY: i32 = 132;
    pub const PM_ELF: i32 = 140;
    pub const PM_ELF_ZOMBIE: i32 = 141;
    pub const PM_ELF_MUMMY: i32 = 142;
    pub const PM_HUMAN: i32 = 150;
    pub const PM_HUMAN_ZOMBIE: i32 = 151;
    pub const PM_HUMAN_MUMMY: i32 = 152;
    pub const PM_VAMPIRE: i32 = 153;
    pub const PM_VAMPIRE_LORD: i32 = 154;
    pub const PM_GIANT: i32 = 160;
    pub const PM_GIANT_ZOMBIE: i32 = 161;
    pub const PM_GIANT_MUMMY: i32 = 162;
    pub const PM_ETTIN: i32 = 170;
    pub const PM_ETTIN_ZOMBIE: i32 = 171;
    pub const PM_ETTIN_MUMMY: i32 = 172;
}

/// [v2.22.0 R34-13] 언데드 몬스터→살아있는 시체 인덱스 변환 (원본: undead_to_corpse)
pub fn undead_to_corpse(mndx: i32) -> i32 {
    match mndx {
        pm::PM_KOBOLD_ZOMBIE | pm::PM_KOBOLD_MUMMY => pm::PM_KOBOLD,
        pm::PM_DWARF_ZOMBIE | pm::PM_DWARF_MUMMY => pm::PM_DWARF,
        pm::PM_GNOME_ZOMBIE | pm::PM_GNOME_MUMMY => pm::PM_GNOME,
        pm::PM_ORC_ZOMBIE | pm::PM_ORC_MUMMY => pm::PM_ORC,
        pm::PM_ELF_ZOMBIE | pm::PM_ELF_MUMMY => pm::PM_ELF,
        pm::PM_VAMPIRE | pm::PM_VAMPIRE_LORD | pm::PM_HUMAN_ZOMBIE | pm::PM_HUMAN_MUMMY => {
            pm::PM_HUMAN
        }
        pm::PM_GIANT_ZOMBIE | pm::PM_GIANT_MUMMY => pm::PM_GIANT,
        pm::PM_ETTIN_ZOMBIE | pm::PM_ETTIN_MUMMY => pm::PM_ETTIN,
        _ => mndx, // 언데드 아님 → 원본 반환
    }
}

// =============================================================================
// [3] 퀘스트 가디언→종족/역할 변환 (원본: mon.c:194-263 genus)
// =============================================================================

/// [v2.22.0 R34-13] 퀘스트 가디언 역할 ID
pub mod quest {
    pub const PM_STUDENT: i32 = 200;
    pub const PM_CHIEFTAIN: i32 = 201;
    pub const PM_NEANDERTHAL: i32 = 202;
    pub const PM_ATTENDANT: i32 = 203;
    pub const PM_PAGE: i32 = 204;
    pub const PM_ABBOT: i32 = 205;
    pub const PM_ACOLYTE: i32 = 206;
    pub const PM_HUNTER: i32 = 207;
    pub const PM_THUG: i32 = 208;
    pub const PM_ROSHI: i32 = 209;
    pub const PM_GUIDE: i32 = 210;
    pub const PM_APPRENTICE: i32 = 211;
    pub const PM_WARRIOR: i32 = 212;
}

/// 역할 ID (genus mode=1용)
pub mod role_pm {
    pub const PM_ARCHEOLOGIST: i32 = 300;
    pub const PM_BARBARIAN: i32 = 301;
    pub const PM_CAVEMAN: i32 = 302;
    pub const PM_HEALER: i32 = 303;
    pub const PM_KNIGHT: i32 = 304;
    pub const PM_MONK: i32 = 305;
    pub const PM_PRIEST: i32 = 306;
    pub const PM_RANGER: i32 = 307;
    pub const PM_ROGUE: i32 = 308;
    pub const PM_SAMURAI: i32 = 309;
    pub const PM_TOURIST: i32 = 310;
    pub const PM_WIZARD: i32 = 311;
    pub const PM_VALKYRIE: i32 = 312;
}

/// [v2.22.0 R34-13] 퀘스트 가디언→종족 또는 역할 변환 (원본: genus)
/// `mode`: 0 = 종족, 1 = 역할
pub fn genus(mndx: i32, mode: i32) -> i32 {
    match mndx {
        quest::PM_STUDENT => {
            if mode != 0 {
                role_pm::PM_ARCHEOLOGIST
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_CHIEFTAIN => {
            if mode != 0 {
                role_pm::PM_BARBARIAN
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_NEANDERTHAL => {
            if mode != 0 {
                role_pm::PM_CAVEMAN
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_ATTENDANT => {
            if mode != 0 {
                role_pm::PM_HEALER
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_PAGE => {
            if mode != 0 {
                role_pm::PM_KNIGHT
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_ABBOT => {
            if mode != 0 {
                role_pm::PM_MONK
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_ACOLYTE => {
            if mode != 0 {
                role_pm::PM_PRIEST
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_HUNTER => {
            if mode != 0 {
                role_pm::PM_RANGER
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_THUG => {
            if mode != 0 {
                role_pm::PM_ROGUE
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_ROSHI => {
            if mode != 0 {
                role_pm::PM_SAMURAI
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_GUIDE => {
            if mode != 0 {
                role_pm::PM_TOURIST
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_APPRENTICE => {
            if mode != 0 {
                role_pm::PM_WIZARD
            } else {
                pm::PM_HUMAN
            }
        }
        quest::PM_WARRIOR => {
            if mode != 0 {
                role_pm::PM_VALKYRIE
            } else {
                pm::PM_HUMAN
            }
        }
        _ => mndx, // 기본: 원본 반환
    }
}

// =============================================================================
// [4] 턴당 상태 타이머 감소 (원본: mon.c:707-713 mcalcdistress 타이머部)
// =============================================================================

/// [v2.22.0 R34-13] 몬스터 턴당 타이머 감소
#[derive(Debug, Clone)]
pub struct TimerTickInput {
    pub blinded: i32,
    pub frozen: i32,
    pub flee_timer: i32,
}

/// [v2.22.0 R34-13] 타이머 틱 결과
#[derive(Debug, Clone)]
pub struct TimerTickResult {
    pub blinded: i32,
    pub can_see: bool,
    pub frozen: i32,
    pub can_move: bool,
    pub flee_timer: i32,
    pub is_fleeing: bool,
}

/// [v2.22.0 R34-13] 매 턴 타이머 감소 (원본: mcalcdistress 707-713행)
pub fn tick_timers(input: &TimerTickInput) -> TimerTickResult {
    let new_blind = if input.blinded > 0 {
        input.blinded - 1
    } else {
        0
    };
    let new_frozen = if input.frozen > 0 {
        input.frozen - 1
    } else {
        0
    };
    let new_flee = if input.flee_timer > 0 {
        input.flee_timer - 1
    } else {
        0
    };

    TimerTickResult {
        blinded: new_blind,
        can_see: new_blind == 0,
        frozen: new_frozen,
        can_move: new_frozen == 0,
        flee_timer: new_flee,
        // 도주 타이머가 0이 되면 도주를 멈추되, 원래부터 0이면 유지
        is_fleeing: if input.flee_timer > 0 && new_flee == 0 {
            false
        } else {
            input.flee_timer >= 0
        },
    }
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcalcmove_normal_speed_12() {
        let mut rng = NetHackRng::new(42);
        let speed = calc_monster_move(12, SpeedState::Normal, false, &mut rng);
        assert_eq!(speed, 12); // 정확히 12 → 조정 없음
    }

    #[test]
    fn test_mcalcmove_slow() {
        let mut rng = NetHackRng::new(42);
        let speed = calc_monster_move(12, SpeedState::Slow, false, &mut rng);
        // (2*12+1)/3 = 8, 8%12=8 → 랜덤에 따라 0 또는 12
        assert!(speed == 0 || speed == 12);
    }

    #[test]
    fn test_mcalcmove_fast() {
        let mut rng = NetHackRng::new(42);
        let speed = calc_monster_move(12, SpeedState::Fast, false, &mut rng);
        // (4*12+2)/3 = 16, 16%12=4 → 12 또는 24
        assert!(speed == 12 || speed == 24);
    }

    #[test]
    fn test_mcalcmove_gallop() {
        let mut rng = NetHackRng::new(42);
        let speed = calc_monster_move(12, SpeedState::Normal, true, &mut rng);
        // 질주: (4 or 5)*12/3 = 16 or 20 → 12 또는 24
        assert!(speed == 12 || speed == 24);
    }

    #[test]
    fn test_undead_kobold_zombie() {
        assert_eq!(undead_to_corpse(pm::PM_KOBOLD_ZOMBIE), pm::PM_KOBOLD);
    }

    #[test]
    fn test_undead_vampire_lord() {
        assert_eq!(undead_to_corpse(pm::PM_VAMPIRE_LORD), pm::PM_HUMAN);
    }

    #[test]
    fn test_undead_not_undead() {
        assert_eq!(undead_to_corpse(pm::PM_HUMAN), pm::PM_HUMAN);
    }

    #[test]
    fn test_genus_student_species() {
        assert_eq!(genus(quest::PM_STUDENT, 0), pm::PM_HUMAN);
    }

    #[test]
    fn test_genus_student_role() {
        assert_eq!(genus(quest::PM_STUDENT, 1), role_pm::PM_ARCHEOLOGIST);
    }

    #[test]
    fn test_genus_warrior_role() {
        assert_eq!(genus(quest::PM_WARRIOR, 1), role_pm::PM_VALKYRIE);
    }

    #[test]
    fn test_timer_blind_expires() {
        let input = TimerTickInput {
            blinded: 1,
            frozen: 0,
            flee_timer: 0,
        };
        let result = tick_timers(&input);
        assert_eq!(result.blinded, 0);
        assert!(result.can_see);
    }

    #[test]
    fn test_timer_frozen_tick() {
        let input = TimerTickInput {
            blinded: 0,
            frozen: 5,
            flee_timer: 3,
        };
        let result = tick_timers(&input);
        assert_eq!(result.frozen, 4);
        assert!(!result.can_move);
        assert_eq!(result.flee_timer, 2);
    }
}

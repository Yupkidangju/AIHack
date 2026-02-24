// ============================================================================
// [v2.35.0 R23-2] 저항 체계 (resist_calc_ext.rs)
// 원본: NetHack 3.6.7 mhitu.c/uhitm.c 저항 판정
// 원소/상태 저항, 마법 저항, 누적
// ============================================================================

use bitflags::bitflags;

bitflags! {
    /// [v2.35.0 R23-2] 저항 플래그
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Resistances: u32 {
        const FIRE       = 0b0000_0000_0001;
        const COLD       = 0b0000_0000_0010;
        const SHOCK      = 0b0000_0000_0100;
        const POISON     = 0b0000_0000_1000;
        const ACID       = 0b0000_0001_0000;
        const SLEEP      = 0b0000_0010_0000;
        const DISINT     = 0b0000_0100_0000;
        const PETRIFY    = 0b0000_1000_0000;
        const MAGIC      = 0b0001_0000_0000;
        const DRAIN      = 0b0010_0000_0000;
        const SICK       = 0b0100_0000_0000;
        const STONING    = 0b1000_0000_0000;
    }
}

/// [v2.35.0 R23-2] 저항으로 데미지 감소
pub fn apply_resistance(damage: i32, resist: bool, half_resist: bool) -> i32 {
    if resist {
        0
    } else if half_resist {
        damage / 2
    } else {
        damage
    }
}

/// [v2.35.0 R23-2] 마법 저항 체크 (원본: resist)
pub fn magic_resistance_check(magic_res: bool, difficulty: i32, level: i32) -> bool {
    if magic_res {
        return true;
    }
    // 난이도 vs 레벨로 부분 저항
    level * 3 > difficulty * 2
}

/// [v2.35.0 R23-2] 내성 합산 (장비 + 내재 + 일시)
pub fn combine_resistances(
    intrinsic: Resistances,
    extrinsic: Resistances,
    temporary: Resistances,
) -> Resistances {
    intrinsic | extrinsic | temporary
}

/// [v2.35.0 R23-2] 특정 저항 보유 여부
pub fn has_resistance(combined: Resistances, check: Resistances) -> bool {
    combined.contains(check)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_resist() {
        assert_eq!(apply_resistance(20, true, false), 0);
    }

    #[test]
    fn test_half_resist() {
        assert_eq!(apply_resistance(20, false, true), 10);
    }

    #[test]
    fn test_no_resist() {
        assert_eq!(apply_resistance(20, false, false), 20);
    }

    #[test]
    fn test_combine() {
        let i = Resistances::FIRE;
        let e = Resistances::COLD;
        let t = Resistances::POISON;
        let all = combine_resistances(i, e, t);
        assert!(has_resistance(all, Resistances::FIRE));
        assert!(has_resistance(all, Resistances::COLD));
        assert!(has_resistance(all, Resistances::POISON));
        assert!(!has_resistance(all, Resistances::SHOCK));
    }

    #[test]
    fn test_magic_resist() {
        assert!(magic_resistance_check(true, 30, 1));
        assert!(magic_resistance_check(false, 10, 20));
        assert!(!magic_resistance_check(false, 30, 5));
    }
}

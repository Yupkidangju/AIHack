// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-21] 상태 표시줄 확장 (botl_ext.rs)
// 원본: NetHack 3.6.7 botl.c (상태 표시줄 계산, 능력치 문자열, 짐 표시)
// ============================================================================

// =============================================================================
// [1] 힘 수치 문자열 변환 (원본: botl.c:22-38 get_strength_str)
// =============================================================================

/// [v2.22.0 R34-21] 힘 수치를 표시 문자열로 변환 (원본: get_strength_str)
/// NetHack에서 힘 18 초과 시 "18/XX" 또는 "18/**" 형식
/// STR18(100) = 18 + 100 = 118
pub fn strength_to_string(st: i32) -> String {
    if st > 18 {
        if st > 118 {
            // 18/100 초과 → 순수 숫자 (예: 19, 20...)
            format!("{}", st - 100)
        } else if st < 118 {
            // 18/01 ~ 18/99
            format!("18/{:02}", st - 18)
        } else {
            // 18/100 = 18/**
            "18/**".to_string()
        }
    } else {
        format!("{}", st)
    }
}

// =============================================================================
// [2] 상태 조건 수집 (원본: botl.c:157-199)
// =============================================================================

/// [v2.22.0 R34-21] 플레이어 상태 조건 플래그
#[derive(Debug, Clone, Default)]
pub struct StatusConditionFlags {
    pub stoned: bool,
    pub slimed: bool,
    pub strangled: bool,
    pub food_poisoned: bool,
    pub terminally_ill: bool,
    pub hungry: bool,
    pub hunger_state: i32, // 0: NOT_HUNGRY … 5: STARVED
    pub encumbered: bool,
    pub encumbrance_level: i32, // 0: UNENCUMBERED … 5: OVERLOADED
    pub blind: bool,
    pub deaf: bool,
    pub stunned: bool,
    pub confused: bool,
    pub hallucinating: bool,
    pub levitating: bool,
    pub flying: bool,
    pub riding: bool,
}

/// [v2.22.0 R34-21] 허기 상태 문자열 (원본: eat.c hu_stat)
pub fn hunger_status_label(state: i32) -> &'static str {
    match state {
        0 => "", // NOT_HUNGRY
        1 => "Hungry",
        2 => "Weak",
        3 => "Fainting",
        4 => "Fainted",
        5 => "Starved",
        _ => "",
    }
}

/// [v2.22.0 R34-21] 짐 상태 문자열 (원본: botl.c enc_stat)
pub fn encumbrance_label(level: i32) -> &'static str {
    match level {
        0 => "", // UNENCUMBERED
        1 => "Burdened",
        2 => "Stressed",
        3 => "Strained",
        4 => "Overtaxed",
        5 => "Overloaded",
        _ => "",
    }
}

/// [v2.22.0 R34-21] 상태 조건 문자열 수집 (원본: botl.c:157-199)
pub fn collect_status_conditions(flags: &StatusConditionFlags) -> Vec<&'static str> {
    let mut conditions = Vec::new();

    // 치명적 상태 우선
    if flags.stoned {
        conditions.push("Stone");
    }
    if flags.slimed {
        conditions.push("Slime");
    }
    if flags.strangled {
        conditions.push("Strngl");
    }
    if flags.food_poisoned {
        conditions.push("FoodPois");
    }
    if flags.terminally_ill {
        conditions.push("TermIll");
    }

    // 허기
    if flags.hunger_state > 0 {
        conditions.push(hunger_status_label(flags.hunger_state));
    }

    // 짐 상태
    if flags.encumbrance_level > 0 {
        conditions.push(encumbrance_label(flags.encumbrance_level));
    }

    // 감각/정신
    if flags.blind {
        conditions.push("Blind");
    }
    if flags.deaf {
        conditions.push("Deaf");
    }
    if flags.stunned {
        conditions.push("Stun");
    }
    if flags.confused {
        conditions.push("Conf");
    }
    if flags.hallucinating {
        conditions.push("Hallu");
    }

    // 이동
    if flags.levitating {
        conditions.push("Lev");
    }
    if flags.flying {
        conditions.push("Fly");
    }
    if flags.riding {
        conditions.push("Ride");
    }

    conditions
}

// =============================================================================
// [3] 짐 무게 계산 (원본: hack.c near_capacity + weight_cap)
// =============================================================================

/// [v2.22.0 R34-21] 짐 단계 상수
pub const UNENCUMBERED: i32 = 0;
pub const SLT_ENCUMBER: i32 = 1; // Burdened
pub const MOD_ENCUMBER: i32 = 2; // Stressed
pub const HVY_ENCUMBER: i32 = 3; // Strained
pub const EXT_ENCUMBER: i32 = 4; // Overtaxed
pub const OVERLOADED: i32 = 5;

/// [v2.22.0 R34-21] 최대 적재량 계산 (원본: weight_cap)
/// `str_value`: 힘 수치 (18/XX 포함, 예: 118 = 18/100)
pub fn calc_weight_cap(str_value: i32) -> i32 {
    let mut cap: i64 = if str_value <= 18 {
        25 * str_value as i64
    } else if str_value <= 118 {
        // 18/01~18/100
        450 + 20 * (str_value as i64 - 18)
    } else {
        // 19+
        (str_value as i64 - 100) * 100
    };

    // 최소 적재량 보장
    if cap < 150 {
        cap = 150;
    }

    cap as i32
}

/// [v2.22.0 R34-21] 짐 단계 판정 (원본: near_capacity)
/// `carried_weight`: 현재 들고 있는 총 무게
/// `weight_cap`: 최대 적재량
pub fn calc_encumbrance(carried_weight: i32, weight_cap: i32) -> i32 {
    if weight_cap <= 0 {
        return OVERLOADED;
    }

    // 원본: s/2 < w 기준으로 5단계
    let half_cap = weight_cap / 2;
    let wc = weight_cap;

    if carried_weight <= half_cap {
        UNENCUMBERED
    } else if carried_weight <= half_cap + (wc - half_cap) / 5 {
        SLT_ENCUMBER // Burdened
    } else if carried_weight <= half_cap + 2 * (wc - half_cap) / 5 {
        MOD_ENCUMBER // Stressed
    } else if carried_weight <= half_cap + 3 * (wc - half_cap) / 5 {
        HVY_ENCUMBER // Strained
    } else if carried_weight <= half_cap + 4 * (wc - half_cap) / 5 {
        EXT_ENCUMBER // Overtaxed
    } else {
        OVERLOADED
    }
}

// =============================================================================
// [4] 성향 문자열 (원본: botl.c:87-89)
// =============================================================================

/// [v2.22.0 R34-21] 성향 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Chaotic,
    Neutral,
    Lawful,
}

/// [v2.22.0 R34-21] 성향 표시 문자열
pub fn alignment_label(align: Alignment) -> &'static str {
    match align {
        Alignment::Chaotic => "Chaotic",
        Alignment::Neutral => "Neutral",
        Alignment::Lawful => "Lawful",
    }
}

// =============================================================================
// [5] 상태줄 HP 색상 판정 (커스텀)
// =============================================================================

/// [v2.22.0 R34-21] HP 퍼센트 기반 색상 단계
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpColorLevel {
    /// 100%: 녹색/정상
    Full,
    /// 50-99%: 노란색/주의
    Warning,
    /// 25-49%: 주황색/위험
    Danger,
    /// 1-24%: 빨간색/치명
    Critical,
    /// 0%: 죽음
    Dead,
}

/// [v2.22.0 R34-21] HP 색상 단계 판정
pub fn hp_color_level(hp: i32, max_hp: i32) -> HpColorLevel {
    if max_hp <= 0 || hp <= 0 {
        return HpColorLevel::Dead;
    }
    let pct = hp * 100 / max_hp;
    if pct >= 100 {
        HpColorLevel::Full
    } else if pct >= 50 {
        HpColorLevel::Warning
    } else if pct >= 25 {
        HpColorLevel::Danger
    } else {
        HpColorLevel::Critical
    }
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strength_string_normal() {
        assert_eq!(strength_to_string(10), "10");
        assert_eq!(strength_to_string(18), "18");
    }

    #[test]
    fn test_strength_string_exceptional() {
        assert_eq!(strength_to_string(19), "18/01"); // 18 + 1
        assert_eq!(strength_to_string(68), "18/50"); // 18 + 50
        assert_eq!(strength_to_string(117), "18/99");
        assert_eq!(strength_to_string(118), "18/**"); // 18/100
        assert_eq!(strength_to_string(119), "19"); // 19 이상
    }

    #[test]
    fn test_hunger_labels() {
        assert_eq!(hunger_status_label(0), "");
        assert_eq!(hunger_status_label(1), "Hungry");
        assert_eq!(hunger_status_label(5), "Starved");
    }

    #[test]
    fn test_encumbrance_labels() {
        assert_eq!(encumbrance_label(0), "");
        assert_eq!(encumbrance_label(1), "Burdened");
        assert_eq!(encumbrance_label(5), "Overloaded");
    }

    #[test]
    fn test_collect_conditions_empty() {
        let flags = StatusConditionFlags::default();
        assert!(collect_status_conditions(&flags).is_empty());
    }

    #[test]
    fn test_collect_conditions_multiple() {
        let flags = StatusConditionFlags {
            stoned: true,
            blind: true,
            confused: true,
            hunger_state: 2,
            ..Default::default()
        };
        let conditions = collect_status_conditions(&flags);
        assert!(conditions.contains(&"Stone"));
        assert!(conditions.contains(&"Blind"));
        assert!(conditions.contains(&"Conf"));
        assert!(conditions.contains(&"Weak"));
    }

    #[test]
    fn test_weight_cap() {
        assert_eq!(calc_weight_cap(10), 250); // 25 * 10
        assert_eq!(calc_weight_cap(18), 450); // 25 * 18
        assert_eq!(calc_weight_cap(68), 1450); // 450 + 20 * 50
        assert_eq!(calc_weight_cap(118), 2450); // 450 + 20 * 100
        assert_eq!(calc_weight_cap(119), 1900); // (119-100) * 100
    }

    #[test]
    fn test_encumbrance_unencumbered() {
        assert_eq!(calc_encumbrance(100, 500), UNENCUMBERED); // 100 <= 250
    }

    #[test]
    fn test_encumbrance_burdened() {
        assert_eq!(calc_encumbrance(280, 500), SLT_ENCUMBER); // 250 < 280 <= 300
    }

    #[test]
    fn test_encumbrance_overloaded() {
        assert_eq!(calc_encumbrance(499, 500), OVERLOADED);
    }

    #[test]
    fn test_hp_color_full() {
        assert_eq!(hp_color_level(50, 50), HpColorLevel::Full);
    }

    #[test]
    fn test_hp_color_critical() {
        assert_eq!(hp_color_level(5, 50), HpColorLevel::Critical);
    }

    #[test]
    fn test_hp_color_dead() {
        assert_eq!(hp_color_level(0, 50), HpColorLevel::Dead);
    }

    #[test]
    fn test_alignment_labels() {
        assert_eq!(alignment_label(Alignment::Chaotic), "Chaotic");
        assert_eq!(alignment_label(Alignment::Neutral), "Neutral");
        assert_eq!(alignment_label(Alignment::Lawful), "Lawful");
    }
}

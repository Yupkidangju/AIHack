// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-6] 역할/종족 선택 엔진 확장 모듈 (role_selection_ext.rs)
// 원본: NetHack 3.6.7 role.c (비트마스크 기반 호환성 검증/선택 로직)
// role_ext.rs의 열거형 기반 시스템을 보완하는 저수준 비트마스크 엔진
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 상수 (원본: role.c, role.h 매크로)
// =============================================================================

/// 역할 미선택 (원본: ROLE_NONE = -1)
pub const ROLE_NONE: i32 = -1;
/// 무작위 선택 (원본: ROLE_RANDOM = -2)
pub const ROLE_RANDOM: i32 = -2;

/// 마스크 상수 (원본: role.h)
pub const ROLE_RACEMASK: u32 = 0x0ff8;
pub const ROLE_GENDMASK: u32 = 0x0003;
pub const ROLE_ALIGNMASK: u32 = 0x7000;

/// 성별 플래그
pub const ROLE_MALE: u32 = 0x0001;
pub const ROLE_FEMALE: u32 = 0x0002;

/// 정렬 플래그
pub const ROLE_LAWFUL: u32 = 0x1000;
pub const ROLE_NEUTRAL: u32 = 0x2000;
pub const ROLE_CHAOTIC: u32 = 0x4000;

/// 선택 방법
pub const PICK_RANDOM: i32 = 0;
pub const PICK_RIGID: i32 = 1;

// =============================================================================
// [2] 비트마스크 호환성 검증 (원본: ok_role, ok_race, ok_gend, ok_align)
// =============================================================================

/// [v2.22.0 R34-6] 비트마스크 기반 역할 호환성 판정
/// (원본: role.c:1048 ok_role)
pub fn check_role_compat(
    role_allow: u32,
    race_allow: Option<u32>,
    gender_allow: Option<u32>,
    align_allow: Option<u32>,
    is_filtered: bool,
) -> bool {
    if is_filtered {
        return false;
    }
    if let Some(r) = race_allow {
        if role_allow & r & ROLE_RACEMASK == 0 {
            return false;
        }
    }
    if let Some(g) = gender_allow {
        if role_allow & g & ROLE_GENDMASK == 0 {
            return false;
        }
    }
    if let Some(a) = align_allow {
        if role_allow & a & ROLE_ALIGNMASK == 0 {
            return false;
        }
    }
    true
}

/// [v2.22.0 R34-6] 종족 호환성 (원본: validrace)
pub fn check_valid_race(role_allow: u32, race_allow: u32) -> bool {
    role_allow & race_allow & ROLE_RACEMASK != 0
}

/// [v2.22.0 R34-6] 성별 호환성 (원본: validgend)
pub fn check_valid_gender(role_allow: u32, race_allow: u32, gend_allow: u32) -> bool {
    role_allow & race_allow & gend_allow & ROLE_GENDMASK != 0
}

/// [v2.22.0 R34-6] 정렬 호환성 (원본: validalign)
pub fn check_valid_align(role_allow: u32, race_allow: u32, align_allow: u32) -> bool {
    role_allow & race_allow & align_allow & ROLE_ALIGNMASK != 0
}

// =============================================================================
// [3] 필터링 및 무작위 선택 (원본: randrace, randgend, randalign, pick_*)
// =============================================================================

/// [v2.22.0 R34-6] 호환 종족 필터링 (원본: randrace 패턴)
pub fn filter_races(role_allow: u32, race_allows: &[u32]) -> Vec<usize> {
    race_allows
        .iter()
        .enumerate()
        .filter(|(_, &ra)| role_allow & ra & ROLE_RACEMASK != 0)
        .map(|(i, _)| i)
        .collect()
}

/// [v2.22.0 R34-6] 호환 성별 필터링 (원본: randgend 패턴)
pub fn filter_genders(role_allow: u32, race_allow: u32, gend_allows: &[u32]) -> Vec<usize> {
    gend_allows
        .iter()
        .enumerate()
        .filter(|(_, &ga)| role_allow & race_allow & ga & ROLE_GENDMASK != 0)
        .map(|(i, _)| i)
        .collect()
}

/// [v2.22.0 R34-6] 호환 정렬 필터링 (원본: randalign 패턴)
pub fn filter_aligns(role_allow: u32, race_allow: u32, align_allows: &[u32]) -> Vec<usize> {
    align_allows
        .iter()
        .enumerate()
        .filter(|(_, &aa)| role_allow & race_allow & aa & ROLE_ALIGNMASK != 0)
        .map(|(i, _)| i)
        .collect()
}

/// [v2.22.0 R34-6] 후보에서 선택 (원본: pick_role/pick_race/pick_gend/pick_align 공통 패턴)
pub fn pick_from(candidates: &[usize], pickhow: i32, rng: &mut NetHackRng) -> i32 {
    if candidates.is_empty() {
        return ROLE_NONE;
    }
    if pickhow == PICK_RIGID && candidates.len() > 1 {
        return ROLE_NONE;
    }
    let idx = rng.rn2(candidates.len() as i32) as usize;
    candidates[idx] as i32
}

// =============================================================================
// [4] 다중 제약 조건 역할 선택 (원본: pick_role 전체 로직)
// =============================================================================

/// [v2.22.0 R34-6] 제약 조건 따른 역할 선택 (원본: role.c:1091 pick_role)
/// `role_allows`: 모든 역할의 allow 마스크
/// `race_allows`: 모든 종족의 allow 마스크
/// `gend_allows`: 모든 성별의 allow 마스크
/// `align_allows`: 모든 정렬의 allow 마스크
/// `race_idx`, `gend_idx`, `align_idx`: 선택된 제약 (ROLE_NONE이면 무시)
/// `role_filters`: 각 역할별 필터 상태 (true면 제외)
pub fn pick_role(
    role_allows: &[u32],
    race_allows: &[u32],
    gend_allows: &[u32],
    align_allows: &[u32],
    race_idx: i32,
    gend_idx: i32,
    align_idx: i32,
    role_filters: &[bool],
    pickhow: i32,
    rng: &mut NetHackRng,
) -> i32 {
    let race_allow = if race_idx >= 0 && (race_idx as usize) < race_allows.len() {
        Some(race_allows[race_idx as usize])
    } else {
        None
    };
    let gend_allow = if gend_idx >= 0 && (gend_idx as usize) < gend_allows.len() {
        Some(gend_allows[gend_idx as usize])
    } else {
        None
    };
    let align_allow = if align_idx >= 0 && (align_idx as usize) < align_allows.len() {
        Some(align_allows[align_idx as usize])
    } else {
        None
    };

    let mut candidates = Vec::new();
    for (i, &role_allow) in role_allows.iter().enumerate() {
        let filtered = i < role_filters.len() && role_filters[i];
        if check_role_compat(role_allow, race_allow, gend_allow, align_allow, filtered) {
            candidates.push(i);
        }
    }

    pick_from(&candidates, pickhow, rng)
}

// =============================================================================
// [5] 문자열 매칭 (원본: str2role, str2race, str2gend, str2align)
// =============================================================================

/// [v2.22.0 R34-6] 대소문자 무시 접두사 매칭 (원본: strncmpi)
fn prefix_ci(input: &str, target: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    target.to_lowercase().starts_with(&input.to_lowercase())
}

/// [v2.22.0 R34-6] 문자열 → 인덱스 변환 (원본: str2role 등 공통 패턴)
/// `entries`: (이름, 대체명, 파일코드) 리스트
pub fn str_to_idx(input: &str, entries: &[(&str, Option<&str>, &str)]) -> i32 {
    if input.is_empty() {
        return ROLE_NONE;
    }
    if input == "*" || input == "@" || prefix_ci(input, "random") {
        return ROLE_RANDOM;
    }
    for (i, &(name, alt, code)) in entries.iter().enumerate() {
        if prefix_ci(input, name) {
            return i as i32;
        }
        if let Some(a) = alt {
            if prefix_ci(input, a) {
                return i as i32;
            }
        }
        if input.eq_ignore_ascii_case(code) {
            return i as i32;
        }
    }
    ROLE_NONE
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // 테스트용 마스크: Archeologist = Human(0x0008)|Dwarf(0x0040) + M/F + L/N
    const ARCH: u32 = 0x0048 | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL;
    const HUMAN: u32 = 0x0008 | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL | ROLE_CHAOTIC;
    const ELF: u32 = 0x0010 | ROLE_MALE | ROLE_FEMALE | ROLE_CHAOTIC;
    const DWARF: u32 = 0x0040 | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL;

    #[test]
    fn test_role_compat_basic() {
        assert!(check_role_compat(ARCH, Some(HUMAN), None, None, false));
        assert!(check_role_compat(ARCH, Some(DWARF), None, None, false));
        assert!(!check_role_compat(ARCH, Some(ELF), None, None, false));
    }

    #[test]
    fn test_role_compat_filtered() {
        assert!(!check_role_compat(ARCH, None, None, None, true));
    }

    #[test]
    fn test_role_compat_gender() {
        assert!(check_role_compat(ARCH, None, Some(ROLE_MALE), None, false));
        assert!(check_role_compat(
            ARCH,
            None,
            Some(ROLE_FEMALE),
            None,
            false
        ));
    }

    #[test]
    fn test_role_compat_align() {
        assert!(check_role_compat(
            ARCH,
            None,
            None,
            Some(ROLE_LAWFUL),
            false
        ));
        assert!(!check_role_compat(
            ARCH,
            None,
            None,
            Some(ROLE_CHAOTIC),
            false
        ));
    }

    #[test]
    fn test_valid_race() {
        assert!(check_valid_race(ARCH, HUMAN));
        assert!(!check_valid_race(ARCH, ELF));
        assert!(check_valid_race(ARCH, DWARF));
    }

    #[test]
    fn test_valid_gender() {
        assert!(check_valid_gender(ARCH, HUMAN, ROLE_MALE));
        assert!(check_valid_gender(ARCH, HUMAN, ROLE_FEMALE));
    }

    #[test]
    fn test_valid_align() {
        assert!(check_valid_align(ARCH, HUMAN, ROLE_LAWFUL));
        assert!(check_valid_align(ARCH, HUMAN, ROLE_NEUTRAL));
        // Archeologist는 Chaotic 불가, Human은 Chaotic 가능하지만 AND에서 실패
        assert!(!check_valid_align(ARCH, HUMAN, ROLE_CHAOTIC));
    }

    #[test]
    fn test_filter_races() {
        let races = vec![HUMAN, ELF, DWARF];
        let compat = filter_races(ARCH, &races);
        assert!(compat.contains(&0)); // Human
        assert!(!compat.contains(&1)); // Elf
        assert!(compat.contains(&2)); // Dwarf
    }

    #[test]
    fn test_filter_genders() {
        let genders = vec![ROLE_MALE, ROLE_FEMALE];
        let compat = filter_genders(ARCH, HUMAN, &genders);
        assert_eq!(compat.len(), 2);
    }

    #[test]
    fn test_filter_aligns() {
        let aligns = vec![ROLE_LAWFUL, ROLE_NEUTRAL, ROLE_CHAOTIC];
        let compat = filter_aligns(ARCH, HUMAN, &aligns);
        assert!(compat.contains(&0)); // Lawful
        assert!(compat.contains(&1)); // Neutral
        assert!(!compat.contains(&2)); // Chaotic
    }

    #[test]
    fn test_pick_random() {
        let mut rng = NetHackRng::new(42);
        let cands = vec![0, 2, 5];
        let r = pick_from(&cands, PICK_RANDOM, &mut rng);
        assert!(cands.contains(&(r as usize)));
    }

    #[test]
    fn test_pick_rigid_single() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(pick_from(&[3], PICK_RIGID, &mut rng), 3);
    }

    #[test]
    fn test_pick_rigid_multi() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(pick_from(&[1, 2], PICK_RIGID, &mut rng), ROLE_NONE);
    }

    #[test]
    fn test_pick_empty() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(pick_from(&[], PICK_RANDOM, &mut rng), ROLE_NONE);
    }

    #[test]
    fn test_pick_role_full() {
        let mut rng = NetHackRng::new(42);
        // Barbarian: Human(0x0008)|Orc(0x0800) + M/F + N/C
        let barb: u32 = 0x0808 | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL | ROLE_CHAOTIC;
        let roles = vec![ARCH, barb];
        let race_allows = vec![HUMAN, ELF, DWARF];
        let gend_allows = vec![ROLE_MALE, ROLE_FEMALE];
        let align_allows = vec![ROLE_LAWFUL, ROLE_NEUTRAL, ROLE_CHAOTIC];
        let filters = vec![false, false];

        // Chaotic 제약 → Archeologist 제외, Barbarian만 가능
        let result = pick_role(
            &roles,
            &race_allows,
            &gend_allows,
            &align_allows,
            ROLE_NONE,
            ROLE_NONE,
            2, // align=Chaotic
            &filters,
            PICK_RANDOM,
            &mut rng,
        );
        assert_eq!(result, 1); // Barbarian
    }

    #[test]
    fn test_str_to_idx() {
        let entries = vec![
            ("Archeologist", None, "Arc"),
            ("Barbarian", None, "Bar"),
            ("Caveman", Some("Cavewoman"), "Cav"),
        ];
        assert_eq!(str_to_idx("Arch", &entries), 0);
        assert_eq!(str_to_idx("bar", &entries), 1);
        assert_eq!(str_to_idx("Cavewoman", &entries), 2);
        assert_eq!(str_to_idx("Cav", &entries), 2);
        assert_eq!(str_to_idx("random", &entries), ROLE_RANDOM);
        assert_eq!(str_to_idx("*", &entries), ROLE_RANDOM);
        assert_eq!(str_to_idx("XYZ", &entries), ROLE_NONE);
        assert_eq!(str_to_idx("", &entries), ROLE_NONE);
    }
}

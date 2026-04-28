// ============================================================================
// AIHack - bones_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] bones.c 핵심 함수 완전 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 bones.c (685줄)
//
// 이식 대상:
//   no_bones_level(), can_make_bones(), sanitize_name(),
//   resetobj_for_bones() (아이템 초기화),
//   drop_upon_death() (아이템 드롭/저주 확률),
//   savebones/getbones (핵심 판정 로직)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// no_bones_level — 뼈다귀 레벨 불가 판정
// [v2.10.1] bones.c:19-38 이식
// =============================================================================

/// 레벨 속성 (뼈다귀 판정용)
#[derive(Debug, Clone)]
pub struct LevelInfo {
    /// 던전 번호
    pub dnum: i32,
    /// 던전 레벨 깊이
    pub dlevel: i32,
    /// 특수 레벨 여부
    pub is_special: bool,
    /// 특수 레벨의 뼈다귀 허용 여부
    pub special_allows_bones: bool,
    /// 던전이 뼈다귀를 허용하는지
    pub dungeon_allows_bones: bool,
    /// 최하층 여부
    pub is_bottom_level: bool,
    /// 분기 레벨 여부
    pub is_branch_level: bool,
    /// 지옥 내부 여부
    pub is_in_hell: bool,
    /// 이 던전의 총 레벨 수
    pub dungeon_levels: i32,
    /// 마법 포탈 존재 여부
    pub has_magic_portal: bool,
}

/// 뼈다귀 레벨 불가 여부 (원본 no_bones_level)
/// [v2.10.1] bones.c:19-38
pub fn no_bones_level(level: &LevelInfo) -> bool {
    // 특수 레벨이 뼈다귀 비허용 (원본:29)
    if level.is_special && !level.special_allows_bones {
        return true;
    }
    // 던전 자체가 뼈다귀 비허용 (원본:30)
    if !level.dungeon_allows_bones {
        return true;
    }
    // 최하층 (원본:33)
    if level.is_bottom_level {
        return true;
    }
    // 분기 레벨 (레벨1 제외) (원본:34)
    if level.is_branch_level && level.dlevel > 1 {
        return true;
    }
    // 지옥 소환 레벨 (원본:36-37)
    if level.is_in_hell && level.dlevel == level.dungeon_levels - 1 {
        return true;
    }
    false
}

// =============================================================================
// can_make_bones — 뼈다귀 생성 가능 판정
// [v2.10.1] bones.c:321-352 이식
// =============================================================================

/// 뼈다귀 생성 가능 여부 (원본 can_make_bones)
/// [v2.10.1] bones.c:321-352
pub fn can_make_bones(
    bones_enabled: bool,
    level: &LevelInfo,
    depth: i32,
    is_swallowed: bool,
    is_discover_mode: bool,
    is_wizard_mode: bool,
    rng: &mut NetHackRng,
) -> bool {
    // 뼈다귀 비활성 (원본:327-328)
    if !bones_enabled {
        return false;
    }
    // 불가 레벨 (원본:331-332)
    if no_bones_level(level) {
        return false;
    }
    // 삼켜진 상태 (원본:333-335)
    if is_swallowed {
        return false;
    }
    // 비분기 레벨의 마법 포탈 (원본:336-341)
    if !level.is_branch_level && level.has_magic_portal {
        return false;
    }
    // 깊이 확률 (원본:343-346)
    // 얕은 층일수록 유령 적음: !rn2(1 + depth>>2) → rn2 결과가 0이면 실패
    if depth <= 0 {
        return false;
    }
    let fail_chance = 1 + (depth >> 2);
    if rng.rn2(fail_chance) == 0 && !is_wizard_mode {
        return false;
    }
    // 발견 모드 (원본:349-350)
    if is_discover_mode {
        return false;
    }
    true
}

// =============================================================================
// sanitize_name — 이름 정화
// [v2.10.1] bones.c:204-230 이식
// =============================================================================

/// 뼈다귀 로드 시 이름 정화 (원본 sanitize_name)
/// [v2.10.1] bones.c:204-230
/// 비출력 문자, 제어 문자를 안전한 문자로 교체
pub fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            let code = c as u32;
            if code < 0x20 || code == 0x7F {
                // 비출력/DEL → '.'
                '.'
            } else {
                c
            }
        })
        .collect()
}

// =============================================================================
// resetobj_for_bones — 아이템 뼈다귀 저장 초기화
// [v2.10.1] bones.c:55-202 핵심 이식
// =============================================================================

/// 뼈다귀용 아이템 초기화 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BonesResetAction {
    /// 아이템 유지 (속성 초기화됨)
    Keep,
    /// 아이템 제거 (사용 중이었음)
    Remove,
    /// 아이템 변환 (예: 옌더의 부적 → 가짜)
    Transform { new_type: String },
}

/// 아이템 유형별 뼈다귀 특수 처리 매칭
pub fn resetobj_bones_action(item_name: &str, is_in_use: bool) -> BonesResetAction {
    if is_in_use {
        return BonesResetAction::Remove;
    }

    let l = item_name.to_lowercase();

    // 옌더의 부적 → 가짜 (원본:179-182)
    if l.contains("amulet of yendor") && !l.contains("fake") {
        return BonesResetAction::Transform {
            new_type: "fake amulet of yendor".to_string(),
        };
    }
    // 소환의 촛대 → 밀랍 양초 (원본:183-192)
    if l.contains("candelabrum") {
        return BonesResetAction::Transform {
            new_type: "wax candle".to_string(),
        };
    }
    // 개방의 종 → 일반 종 (원본:193-195)
    if l.contains("bell of opening") {
        return BonesResetAction::Transform {
            new_type: "bell".to_string(),
        };
    }
    // 사자의 서 → 백지 주문서 (원본:196-198)
    if l.contains("book of the dead") {
        return BonesResetAction::Transform {
            new_type: "blank paper".to_string(),
        };
    }

    BonesResetAction::Keep
}

/// 뼈다귀 저장 시 아이템 속성 초기화 결과
#[derive(Debug, Clone)]
pub struct BonesItemReset {
    /// 식별 상태 초기화
    pub known_cleared: bool,
    /// 축복/저주 식별 초기화
    pub bknown_cleared: bool,
    /// 인벤토리 문자 초기화
    pub invlet_cleared: bool,
    /// 비용 초기화
    pub no_charge_cleared: bool,
}

/// 아이템 속성 초기화 (원본:109-117)
pub fn reset_item_properties() -> BonesItemReset {
    BonesItemReset {
        known_cleared: true,
        bknown_cleared: true,
        invlet_cleared: true,
        no_charge_cleared: true,
    }
}

// =============================================================================
// drop_upon_death — 사망 시 아이템 저주/드롭
// [v2.10.1] bones.c:232-269 이식
// =============================================================================

/// 사망 시 아이템 저주 확률 판정 (원본 drop_upon_death:258)
/// 아이템 80% 확률로 저주됨
pub fn should_curse_on_death(rng: &mut NetHackRng) -> bool {
    rng.rn2(5) != 0 // 4/5 확률
}

/// 사망 시 동반 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathRiseType {
    /// 석상으로 변환 — 아이템 석상에 내장
    Statue,
    /// 유령 생성 — 아이템 바닥에 드롭
    Ghost,
    /// 몬스터로 부활 — 아이템 몬스터에게 전달
    MonsterRise { monster_type: i32 },
}

// =============================================================================
// getbones_chance — 뼈다귀 로드 확률
// [v2.10.1] bones.c:577-592 이식
// =============================================================================

/// 뼈다귀 로드 확률 (원본 getbones:590)
/// 33% 확률로 뼈다귀 발견 (위저드 모드: 항상)
pub fn getbones_chance(
    bones_enabled: bool,
    is_discover_mode: bool,
    is_wizard_mode: bool,
    level: &LevelInfo,
    rng: &mut NetHackRng,
) -> bool {
    if is_discover_mode || !bones_enabled {
        return false;
    }
    if no_bones_level(level) {
        return false;
    }
    // 1/3 확률 (원본:590-591)
    if rng.rn2(3) != 0 && !is_wizard_mode {
        return false;
    }
    true
}

// =============================================================================
// 뼈다귀 묘비 정보
// [v2.10.1] bones.c:497-521 이식
// =============================================================================

/// 뼈다귀 묘비 정보 (원본: struct cemetery)
#[derive(Debug, Clone)]
pub struct BonesRecord {
    /// "이름-역할-종족-성별-정렬"
    pub who: String,
    /// 사인
    pub how: String,
    /// 날짜
    pub when: String,
    /// 최종 위치
    pub final_x: i32,
    pub final_y: i32,
    /// 이미 발견되었는지
    pub bones_known: bool,
}

impl BonesRecord {
    /// 묘비 생성 (원본:505-512)
    pub fn new(
        name: &str,
        role: &str,
        race: &str,
        gender: &str,
        alignment: &str,
        cause_of_death: &str,
        date: &str,
        x: i32,
        y: i32,
    ) -> Self {
        Self {
            who: format!("{}-{}-{}-{}-{}", name, role, race, gender, alignment),
            how: cause_of_death.to_string(),
            when: date.to_string(),
            final_x: x,
            final_y: y,
            bones_known: false,
        }
    }
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn basic_level() -> LevelInfo {
        LevelInfo {
            dnum: 0,
            dlevel: 5,
            is_special: false,
            special_allows_bones: false,
            dungeon_allows_bones: true,
            is_bottom_level: false,
            is_branch_level: false,
            is_in_hell: false,
            dungeon_levels: 30,
            has_magic_portal: false,
        }
    }

    #[test]
    fn test_no_bones_normal_level() {
        assert!(!no_bones_level(&basic_level()));
    }

    #[test]
    fn test_no_bones_special() {
        let mut l = basic_level();
        l.is_special = true;
        l.special_allows_bones = false;
        assert!(no_bones_level(&l));
    }

    #[test]
    fn test_no_bones_bottom() {
        let mut l = basic_level();
        l.is_bottom_level = true;
        assert!(no_bones_level(&l));
    }

    #[test]
    fn test_no_bones_branch() {
        let mut l = basic_level();
        l.is_branch_level = true;
        l.dlevel = 5;
        assert!(no_bones_level(&l));
    }

    #[test]
    fn test_no_bones_branch_level1() {
        let mut l = basic_level();
        l.is_branch_level = true;
        l.dlevel = 1;
        assert!(!no_bones_level(&l)); // 레벨1 분기는 허용
    }

    #[test]
    fn test_no_bones_hell_invocation() {
        let mut l = basic_level();
        l.is_in_hell = true;
        l.dungeon_levels = 10;
        l.dlevel = 9; // 마지막 -1
        assert!(no_bones_level(&l));
    }

    #[test]
    fn test_can_make_bones_basic() {
        let l = basic_level();
        // 여러 시드로 테스트 (확률적)
        let mut any_true = false;
        for seed in 0..50u64 {
            let mut rng = NetHackRng::new(seed);
            if can_make_bones(true, &l, 10, false, false, false, &mut rng) {
                any_true = true;
                break;
            }
        }
        assert!(any_true);
    }

    #[test]
    fn test_can_make_bones_disabled() {
        let l = basic_level();
        let mut rng = NetHackRng::new(42);
        assert!(!can_make_bones(
            false, &l, 10, false, false, false, &mut rng
        ));
    }

    #[test]
    fn test_can_make_bones_swallowed() {
        let l = basic_level();
        let mut rng = NetHackRng::new(0);
        assert!(!can_make_bones(true, &l, 10, true, false, false, &mut rng));
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Hello\x01World"), "Hello.World");
        assert_eq!(sanitize_name("Normal Name"), "Normal Name");
        assert_eq!(sanitize_name("Evil\x7FHacker"), "Evil.Hacker");
    }

    #[test]
    fn test_resetobj_bones_amulet() {
        let r = resetobj_bones_action("Amulet of Yendor", false);
        assert!(matches!(r, BonesResetAction::Transform { .. }));
        if let BonesResetAction::Transform { new_type } = r {
            assert!(new_type.contains("fake"));
        }
    }

    #[test]
    fn test_resetobj_bones_candelabrum() {
        let r = resetobj_bones_action("Candelabrum of Invocation", false);
        assert!(matches!(r, BonesResetAction::Transform { .. }));
    }

    #[test]
    fn test_resetobj_bones_bell() {
        let r = resetobj_bones_action("Bell of Opening", false);
        if let BonesResetAction::Transform { new_type } = r {
            assert_eq!(new_type, "bell");
        } else {
            panic!("종이 변환되어야 함");
        }
    }

    #[test]
    fn test_resetobj_bones_normal() {
        assert_eq!(
            resetobj_bones_action("dagger", false),
            BonesResetAction::Keep
        );
    }

    #[test]
    fn test_resetobj_in_use() {
        assert_eq!(
            resetobj_bones_action("dagger", true),
            BonesResetAction::Remove
        );
    }

    #[test]
    fn test_should_curse_rate() {
        let mut cursed = 0;
        for seed in 0..100u64 {
            let mut rng = NetHackRng::new(seed);
            if should_curse_on_death(&mut rng) {
                cursed += 1;
            }
        }
        // 80% ± 넓은 허용범위
        assert!(cursed > 50 && cursed < 100, "cursed={}", cursed);
    }

    #[test]
    fn test_getbones_chance() {
        let l = basic_level();
        let mut found = 0;
        for seed in 0..100u64 {
            let mut rng = NetHackRng::new(seed);
            if getbones_chance(true, false, false, &l, &mut rng) {
                found += 1;
            }
        }
        // 33% ± 넓은 허용
        assert!(found > 10 && found < 60, "found={}", found);
    }

    #[test]
    fn test_bones_record() {
        let rec = BonesRecord::new(
            "Player",
            "Val",
            "Hum",
            "Fem",
            "Neu",
            "killed by a troll",
            "20260218",
            10,
            15,
        );
        assert!(rec.who.contains("Player"));
        assert!(rec.who.contains("Val"));
        assert!(!rec.bones_known);
    }

    #[test]
    fn test_reset_item_properties() {
        let reset = reset_item_properties();
        assert!(reset.known_cleared);
        assert!(reset.bknown_cleared);
        assert!(reset.invlet_cleared);
    }
}

// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-4] 탐지/탐색 확장 모듈 (detect_ext.rs)
// 원본: NetHack 3.6.7 detect.c (순수 판정/계산 로직)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 상수 (원본: detect.c, hack.h 매크로)
// =============================================================================

/// 탐색 영역 반지름 (원본: dosearch0에서 ux-1..ux+1)
pub const SEARCH_RADIUS: i32 = 1;
/// 번개 사정거리 (원본: BOLT_LIM, do_clear_area에 사용)
pub const BOLT_LIM: i32 = 8;
/// 최대 탐색 보너스 (원본: dosearch0에서 fund > 5 -> 5)
pub const MAX_SEARCH_BONUS: i32 = 5;

/// 오브젝트 함정 탐지 결과 (원본: OTRAP_NONE/HERE/THERE)
pub const OTRAP_NONE: u8 = 0;
pub const OTRAP_HERE: u8 = 1;
pub const OTRAP_THERE: u8 = 2;

// =============================================================================
// [2] 레벨 거리 설명 (원본: detect.c:1041 level_distance)
// =============================================================================

/// [v2.22.0 R34-4] 레벨 간 거리를 설명 문자열로 변환
/// (원본: detect.c:1041 level_distance)
/// `depth_diff`: 현재 깊이 - 목표 깊이 (양수면 위, 음수면 아래)
/// `same_dungeon`: 같은 던전 분기 내인지
/// `far_threshold`: 원본의 8 + rn2(3) 대신 미리 계산된 임계값
pub fn calc_level_distance(
    depth_diff: i32,
    same_dungeon: bool,
    far_threshold: i32,
) -> &'static str {
    if depth_diff < 0 {
        // 목표가 아래에 있음
        if depth_diff < -far_threshold {
            if !same_dungeon {
                "far away"
            } else {
                "far below"
            }
        } else if depth_diff < -1 {
            if !same_dungeon {
                "away below you"
            } else {
                "below you"
            }
        } else {
            if !same_dungeon {
                "in the distance"
            } else {
                "just below"
            }
        }
    } else if depth_diff > 0 {
        // 목표가 위에 있음
        if depth_diff > far_threshold {
            if !same_dungeon {
                "far away"
            } else {
                "far above"
            }
        } else if depth_diff > 1 {
            if !same_dungeon {
                "away above you"
            } else {
                "above you"
            }
        } else {
            if !same_dungeon {
                "in the distance"
            } else {
                "just above"
            }
        }
    } else {
        // 같은 깊이
        if !same_dungeon {
            "in the distance"
        } else {
            "near you"
        }
    }
}

// =============================================================================
// [3] 탐색 확률 계산 (원본: dosearch0 핵심 로직)
// =============================================================================

/// [v2.22.0 R34-4] 탐색 보너스 계산 (원본: dosearch0에서 fund 계산)
/// `has_search_artifact`: 탐색 아티팩트 장비 중이고 SPFX_SEARCH 보유
/// `artifact_spe`: 아티팩트의 spe 값
/// `has_lenses`: 렌즈 착용 중이고 비맹인
pub fn calc_search_bonus(has_search_artifact: bool, artifact_spe: i32, has_lenses: bool) -> i32 {
    let mut fund = if has_search_artifact { artifact_spe } else { 0 };
    if has_lenses {
        fund += 2;
    }
    fund.min(MAX_SEARCH_BONUS)
}

/// [v2.22.0 R34-4] 비밀문/통로 발견 확률 (원본: dosearch0에서 rnl(7-fund))
/// `fund`: 탐색 보너스 (0-5)
/// `luck`: 플레이어의 현재 운
/// 반환: true면 발견 실패 (rnl이 0이 아닌 경우)
pub fn calc_secret_door_check(fund: i32, luck: i32, rng: &mut NetHackRng) -> bool {
    let threshold = 7 - fund;
    if threshold <= 0 {
        return false; // 항상 발견
    }
    // rnl(x): luck 보정된 난수 (운이 좋을수록 낮은 값)
    let roll = rnl(threshold, luck, rng);
    roll != 0 // 0이면 발견, 그 외 실패
}

/// [v2.22.0 R34-4] luck 보정 난수 (원본: rnd.c rnl)
/// 운이 양수면 결과가 작아지고(발견 쉬움), 음수면 커짐(발견 어려움)
fn rnl(x: i32, luck: i32, rng: &mut NetHackRng) -> i32 {
    if x <= 0 {
        return 0;
    }
    let mut result = rng.rn2(x);
    // 운이 양수: 결과 감소 (발견 확률 증가)
    if luck > 0 && result != 0 {
        result -= luck.min(result);
    }
    // 운이 음수: 결과 증가 (발견 확률 감소)
    if luck < 0 {
        result -= luck; // luck이 음수이므로 result 증가
        if result >= x {
            result = x - 1;
        }
    }
    result.max(0)
}

/// [v2.22.0 R34-4] 함정 발견 확률 (원본: dosearch0에서 !rnl(8))
pub fn calc_trap_discovery(luck: i32, rng: &mut NetHackRng) -> bool {
    rnl(8, luck, rng) == 0
}

// =============================================================================
// [4] 탐색 영역 좌표 생성 (원본: dosearch0의 3×3 루프)
// =============================================================================

/// [v2.22.0 R34-4] 탐색 대상 좌표 생성 (플레이어 주변 8칸)
/// `ux`, `uy`: 플레이어 위치
/// `max_x`, `max_y`: 맵 크기 제한
pub fn search_adjacent_positions(ux: i32, uy: i32, max_x: i32, max_y: i32) -> Vec<(i32, i32)> {
    let mut positions = Vec::with_capacity(8);
    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = ux + dx;
            let ny = uy + dy;
            if nx >= 0 && nx < max_x && ny >= 0 && ny < max_y {
                positions.push((nx, ny));
            }
        }
    }
    positions
}

// =============================================================================
// [5] 탐지 결과 분류 (원본: gold_detect, food_detect, object_detect,
//     monster_detect, trap_detect의 결과 판정 로직)
// =============================================================================

/// [v2.22.0 R34-4] 탐지 결과 분류
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionResult {
    /// 아무것도 탐지되지 않음 (return 1)
    NothingDetected,
    /// 대상이 플레이어 위치에만 있음 (맵 표시 불필요)
    DetectedHereOnly,
    /// 대상이 다른 곳에 있음 (맵 표시 필요)
    DetectedElsewhere,
}

/// [v2.22.0 R34-4] 금 탐지 결과 판정 (원본: gold_detect 분기 로직)
/// `gold_on_floor_at_hero`: 플레이어 위치 바닥에 금이 있는지
/// `gold_on_floor_elsewhere`: 다른 위치에 금이 있는지
/// `gold_in_inventory`: 플레이어 인벤토리에 금이 있는지
/// `gold_on_monsters`: 몬스터가 금을 소지하는지
/// `is_gold_golem`: 플레이어가 골드 골렘인지
/// `steed_has_gold`: 탈것이 금을 소지하는지
pub fn calc_gold_detection(
    gold_on_floor_at_hero: bool,
    gold_on_floor_elsewhere: bool,
    gold_in_inventory: bool,
    gold_on_monsters: bool,
    is_gold_golem: bool,
    steed_has_gold: bool,
) -> DetectionResult {
    let found_elsewhere = gold_on_floor_elsewhere || gold_on_monsters;

    if found_elsewhere {
        DetectionResult::DetectedElsewhere
    } else if gold_on_floor_at_hero || gold_in_inventory || is_gold_golem || steed_has_gold {
        DetectionResult::DetectedHereOnly
    } else {
        DetectionResult::NothingDetected
    }
}

/// [v2.22.0 R34-4] 음식 탐지 결과 판정 (원본: food_detect 분기 로직)
/// `is_confused`: 혼란 상태 (저주 포함) → POTION_CLASS 탐지
/// `count_here`: 플레이어 위치의 대상 수
/// `count_there`: 다른 위치의 대상 수
pub fn calc_food_detection(count_here: i32, count_there: i32) -> DetectionResult {
    if count_here == 0 && count_there == 0 {
        DetectionResult::NothingDetected
    } else if count_there == 0 {
        DetectionResult::DetectedHereOnly
    } else {
        DetectionResult::DetectedElsewhere
    }
}

/// [v2.22.0 R34-4] 몬스터 탐지 결과 판정 (원본: monster_detect 분기 로직)
/// `living_count`: 살아있는 몬스터 수
/// `has_monster_nearby`: 인접한 몬스터가 있는지
pub fn calc_monster_detection(living_count: i32) -> DetectionResult {
    if living_count == 0 {
        DetectionResult::NothingDetected
    } else {
        DetectionResult::DetectedElsewhere
    }
}

/// [v2.22.0 R34-4] 함정 탐지 결과 판정 (원본: trap_detect 분기 로직)
/// `traps_at_hero`: 플레이어 위치에 함정이 있는지
/// `traps_elsewhere`: 다른 위치에 함정이 있는지
/// `obj_traps_result`: detect_obj_traps의 합산 결과 (OTRAP_* 비트)
/// `door_traps_at_hero`: 플레이어 위치에 문 함정이 있는지
/// `door_traps_elsewhere`: 다른 위치에 문 함정이 있는지
pub fn calc_trap_detection(
    traps_at_hero: bool,
    traps_elsewhere: bool,
    obj_trap_bits: u8,
    door_traps_at_hero: bool,
    door_traps_elsewhere: bool,
) -> DetectionResult {
    let found_elsewhere =
        traps_elsewhere || (obj_trap_bits & OTRAP_THERE) != 0 || door_traps_elsewhere;
    let found_here = traps_at_hero || (obj_trap_bits & OTRAP_HERE) != 0 || door_traps_at_hero;

    if found_elsewhere {
        DetectionResult::DetectedElsewhere
    } else if found_here {
        DetectionResult::DetectedHereOnly
    } else {
        DetectionResult::NothingDetected
    }
}

// =============================================================================
// [6] 수정구슬 사용 계산 (원본: use_crystal_ball 핵심 로직)
// =============================================================================

/// [v2.22.0 R34-4] 수정구슬 사용 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrystalBallResult {
    /// 성공: 탐지 수행
    Detection { detect_type: CrystalBallDetection },
    /// 실패: 과부하 (지능 피해)
    Overload { int_damage: i32 },
    /// 효과 없음 (맹인이거나 환각)
    NoVision,
}

/// [v2.22.0 R34-4] 수정구슬 탐지 타입
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrystalBallDetection {
    Objects,
    Monsters,
    Traps,
    LevelFeature { feature_name: String },
}

/// [v2.22.0 R34-4] 수정구슬 과부하 확률 (원본: use_crystal_ball에서 oops 계산)
/// `spe`: 수정구슬의 spe (0 이상이면 안전, 음수면 위험)
/// `int_current`: 현재 지능
pub fn calc_crystal_ball_oops(spe: i32, int_current: i32, rng: &mut NetHackRng) -> bool {
    // [원본: (sobj->spe > 0) ? rn2(sobj->spe * 3) : 1) ... > (u.uluck + 5)]
    // 단순화: spe > 0이면 oops 확률 낮음, spe <= 0이면 거의 확정
    if spe > 0 {
        rng.rn2(spe * 3) == 0
    } else {
        true
    }
}

/// [v2.22.0 R34-4] 수정구슬 과부하 시 지능 피해
pub fn calc_crystal_ball_int_damage(rng: &mut NetHackRng) -> i32 {
    // 원본: rnd(sobj->oartifact ? 4 : 8)
    rng.rnd(8)
}

// =============================================================================
// [7] 비밀문 변환 유틸리티 (원본: cvt_sdoor_to_door)
// =============================================================================

/// [v2.22.0 R34-4] 타일 타입 상수 (원본: rm.h)
pub const S_DOOR: u8 = 1; // 비밀문
pub const S_CORR: u8 = 2; // 비밀통로
pub const DOOR: u8 = 3; // 일반 문
pub const CORR: u8 = 4; // 일반 통로

/// 문 마스크 상수 (원본: rm.h)
pub const D_NODOOR: u8 = 0x01;
pub const D_BROKEN: u8 = 0x02;
pub const D_ISOPEN: u8 = 0x04;
pub const D_CLOSED: u8 = 0x08;
pub const D_LOCKED: u8 = 0x10;
pub const D_TRAPPED: u8 = 0x20;

/// [v2.22.0 R34-4] 비밀문을 일반 문으로 변환 (원본: detect.c:1433 cvt_sdoor_to_door)
/// `doormask`: 현재 문 마스크
/// 비밀문의 마스크 비트를 일반 문 상태로 변환
pub fn calc_cvt_sdoor_to_door(doormask: u8) -> (u8, u8) {
    // 타입: SDOOR → DOOR
    let new_type = DOOR;

    // 마스크 변환: 비밀문 마스크의 각 비트를 일반 문 마스크로 매핑
    let new_mask = if doormask & D_TRAPPED != 0 {
        D_LOCKED | D_TRAPPED
    } else if doormask & D_LOCKED != 0 {
        D_LOCKED
    } else {
        D_CLOSED
    };

    (new_type, new_mask)
}

// =============================================================================
// [8] 레벨 탐지 대상 테이블 (원본: level_detects[])
// =============================================================================

/// [v2.22.0 R34-4] 수정구슬로 탐지 가능한 특수 레벨
pub const LEVEL_DETECT_TARGETS: [(&str, &str); 4] = [
    ("Delphi", "oracle"),
    ("Medusa's lair", "medusa"),
    ("a castle", "stronghold"),
    ("the Wizard of Yendor's tower", "wizard1"),
];

// =============================================================================
// [9] 탐사 범위 계산 (원본: do_vicinity_map의 범위)
// =============================================================================

/// [v2.22.0 R34-4] 투시(Clairvoyance) 범위 계산
/// (원본: do_vicinity_map에서 사용하는 범위)
/// 반환: (x_min, x_max, y_min, y_max)
pub fn calc_clairvoyance_range(
    ux: i32,
    uy: i32,
    is_blessed: bool,
    max_x: i32,
    max_y: i32,
    rng: &mut NetHackRng,
) -> (i32, i32, i32, i32) {
    // 원본: blessed면 더 넓은 범위
    let range = if is_blessed { 8 } else { 5 };

    // 약간의 랜덤 오프셋
    let dx = rng.rn2(3) - 1; // -1, 0, 1
    let dy = rng.rn2(3) - 1;

    let x_min = (ux - range + dx).max(0);
    let x_max = (ux + range + dx).min(max_x - 1);
    let y_min = (uy - range + dy).max(0);
    let y_max = (uy + range + dy).min(max_y - 1);

    (x_min, x_max, y_min, y_max)
}

// =============================================================================
// [10] 매직 매핑 범위 (원본: do_mapping 범위)
// =============================================================================

/// [v2.22.0 R34-4] 매직 매핑 대상 타일인지 판정
/// (원본: show_map_spot 핵심 로직)
/// `tile_type`: 타일의 유형 ID
/// `is_wall_or_stone`: 벽/돌인지
/// `is_water_or_lava`: 물/용암인지
pub fn should_map_tile(
    tile_type: u8,
    is_wall_or_stone: bool,
    has_door: bool,
    is_explored: bool,
) -> bool {
    // 벽과 돌은 항상 매핑
    if is_wall_or_stone {
        return true;
    }
    // 문이 있는 곳
    if has_door {
        return true;
    }
    // 아직 탐험하지 않은 일반 바닥
    if !is_explored {
        return true;
    }
    false
}

// =============================================================================
// [11] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_distance_below() {
        assert_eq!(calc_level_distance(-1, true, 10), "just below");
        assert_eq!(calc_level_distance(-3, true, 10), "below you");
        assert_eq!(calc_level_distance(-15, true, 10), "far below");
    }

    #[test]
    fn test_level_distance_above() {
        assert_eq!(calc_level_distance(1, true, 10), "just above");
        assert_eq!(calc_level_distance(3, true, 10), "above you");
        assert_eq!(calc_level_distance(15, true, 10), "far above");
    }

    #[test]
    fn test_level_distance_different_dungeon() {
        assert_eq!(calc_level_distance(-1, false, 10), "in the distance");
        assert_eq!(calc_level_distance(-3, false, 10), "away below you");
        assert_eq!(calc_level_distance(-15, false, 10), "far away");
        assert_eq!(calc_level_distance(0, false, 10), "in the distance");
    }

    #[test]
    fn test_level_distance_same_depth() {
        assert_eq!(calc_level_distance(0, true, 10), "near you");
        assert_eq!(calc_level_distance(0, false, 10), "in the distance");
    }

    #[test]
    fn test_search_bonus() {
        assert_eq!(calc_search_bonus(false, 0, false), 0);
        assert_eq!(calc_search_bonus(true, 3, false), 3);
        assert_eq!(calc_search_bonus(true, 3, true), 5); // 3+2=5, clamped
        assert_eq!(calc_search_bonus(true, 10, true), 5); // 10+2=12, clamped to 5
        assert_eq!(calc_search_bonus(false, 0, true), 2);
    }

    #[test]
    fn test_secret_door_check_max_bonus() {
        let mut rng = NetHackRng::new(42);
        // fund=7 → threshold=0 → 항상 발견
        let result = calc_secret_door_check(7, 0, &mut rng);
        assert!(!result); // false = 발견 성공
    }

    #[test]
    fn test_search_adjacent() {
        let positions = search_adjacent_positions(5, 5, 80, 25);
        assert_eq!(positions.len(), 8);
        // 플레이어 위치 (5,5) 자체는 포함되지 않음
        assert!(!positions.contains(&(5, 5)));
    }

    #[test]
    fn test_search_adjacent_corner() {
        // 왼쪽 위 모서리
        let positions = search_adjacent_positions(0, 0, 80, 25);
        assert_eq!(positions.len(), 3);
        assert!(positions.contains(&(1, 0)));
        assert!(positions.contains(&(0, 1)));
        assert!(positions.contains(&(1, 1)));
    }

    #[test]
    fn test_gold_detection_nothing() {
        assert_eq!(
            calc_gold_detection(false, false, false, false, false, false),
            DetectionResult::NothingDetected
        );
    }

    #[test]
    fn test_gold_detection_here() {
        assert_eq!(
            calc_gold_detection(true, false, false, false, false, false),
            DetectionResult::DetectedHereOnly
        );
    }

    #[test]
    fn test_gold_detection_elsewhere() {
        assert_eq!(
            calc_gold_detection(false, true, false, false, false, false),
            DetectionResult::DetectedElsewhere
        );
    }

    #[test]
    fn test_food_detection() {
        assert_eq!(calc_food_detection(0, 0), DetectionResult::NothingDetected);
        assert_eq!(calc_food_detection(3, 0), DetectionResult::DetectedHereOnly);
        assert_eq!(
            calc_food_detection(0, 5),
            DetectionResult::DetectedElsewhere
        );
        assert_eq!(
            calc_food_detection(2, 5),
            DetectionResult::DetectedElsewhere
        );
    }

    #[test]
    fn test_monster_detection() {
        assert_eq!(calc_monster_detection(0), DetectionResult::NothingDetected);
        assert_eq!(
            calc_monster_detection(5),
            DetectionResult::DetectedElsewhere
        );
    }

    #[test]
    fn test_trap_detection() {
        // 아무것도 없음
        assert_eq!(
            calc_trap_detection(false, false, OTRAP_NONE, false, false),
            DetectionResult::NothingDetected
        );
        // 플레이어 위치에만
        assert_eq!(
            calc_trap_detection(true, false, OTRAP_NONE, false, false),
            DetectionResult::DetectedHereOnly
        );
        // 다른 곳에 함정
        assert_eq!(
            calc_trap_detection(false, true, OTRAP_NONE, false, false),
            DetectionResult::DetectedElsewhere
        );
        // 오브젝트 함정이 다른 곳에
        assert_eq!(
            calc_trap_detection(false, false, OTRAP_THERE, false, false),
            DetectionResult::DetectedElsewhere
        );
    }

    #[test]
    fn test_cvt_sdoor_to_door() {
        let (typ, mask) = calc_cvt_sdoor_to_door(D_LOCKED | D_TRAPPED);
        assert_eq!(typ, DOOR);
        assert!(mask & D_LOCKED != 0);
        assert!(mask & D_TRAPPED != 0);

        let (typ2, mask2) = calc_cvt_sdoor_to_door(D_LOCKED);
        assert_eq!(typ2, DOOR);
        assert_eq!(mask2, D_LOCKED);

        let (typ3, mask3) = calc_cvt_sdoor_to_door(0);
        assert_eq!(typ3, DOOR);
        assert_eq!(mask3, D_CLOSED);
    }

    #[test]
    fn test_crystal_ball_good_spe() {
        let mut rng = NetHackRng::new(42);
        // spe=5면 oops 확률은 1/15 정도
        let mut oops_count = 0;
        for _ in 0..100 {
            let mut test_rng = NetHackRng::new(rng.rn2(10000) as u64);
            if calc_crystal_ball_oops(5, 15, &mut test_rng) {
                oops_count += 1;
            }
        }
        // 100회 중 15% 미만이어야 함
        assert!(oops_count < 25);
    }

    #[test]
    fn test_crystal_ball_bad_spe() {
        let mut rng = NetHackRng::new(42);
        // spe=0이면 항상 oops
        assert!(calc_crystal_ball_oops(0, 15, &mut rng));
        assert!(calc_crystal_ball_oops(-1, 15, &mut rng));
    }

    #[test]
    fn test_clairvoyance_range() {
        let mut rng = NetHackRng::new(42);
        let (x_min, x_max, y_min, y_max) = calc_clairvoyance_range(40, 12, false, 80, 25, &mut rng);
        assert!(x_min >= 0 && x_min <= 40);
        assert!(x_max >= 40 && x_max < 80);
        assert!(y_min >= 0 && y_min <= 12);
        assert!(y_max >= 12 && y_max < 25);
    }

    #[test]
    fn test_clairvoyance_blessed_wider() {
        let mut rng = NetHackRng::new(42);
        let (x_min_b, x_max_b, _, _) = calc_clairvoyance_range(40, 12, true, 80, 25, &mut rng);
        let mut rng2 = NetHackRng::new(42);
        let (x_min_n, x_max_n, _, _) = calc_clairvoyance_range(40, 12, false, 80, 25, &mut rng2);
        // 축복된 투시는 기본보다 더 넓어야 함
        assert!(x_max_b - x_min_b >= x_max_n - x_min_n);
    }

    #[test]
    fn test_should_map_tile() {
        assert!(should_map_tile(0, true, false, false)); // 벽
        assert!(should_map_tile(0, false, true, false)); // 문
        assert!(should_map_tile(0, false, false, false)); // 미탐험
        assert!(!should_map_tile(0, false, false, true)); // 이미 탐험
    }

    #[test]
    fn test_rnl_lucky() {
        let mut rng = NetHackRng::new(42);
        // 운이 매우 좋으면 결과가 0에 수렴
        let mut zero_count = 0;
        for _ in 0..100 {
            if rnl(8, 10, &mut rng) == 0 {
                zero_count += 1;
            }
        }
        // 운이 좋으면 0 나올 확률이 높아야 함
        assert!(zero_count > 30);
    }

    #[test]
    fn test_rnl_unlucky() {
        let mut rng = NetHackRng::new(42);
        let mut zero_count = 0;
        for _ in 0..100 {
            if rnl(8, -10, &mut rng) == 0 {
                zero_count += 1;
            }
        }
        // 운이 나쁘면 0 나올 확률이 낮아야 함
        assert!(zero_count < 30);
    }
}

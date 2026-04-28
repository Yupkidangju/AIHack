// teleport_ext.rs — teleport.c 핵심 로직 순수 결과 패턴 이식
// [v2.12.0] 신규 생성: 순간이동 가능 판정/목적지 계산/레벨 텔레포트 등 12개 함수
// 원본: NetHack 3.6.7 src/teleport.c (1,585줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 맵 열 수
const COLNO: i32 = 80;
/// 맵 행 수
const ROWNO: i32 = 21;

// ============================================================
// 열거형
// ============================================================

/// 지형 유형 (순간이동 지점 유효성 판정용)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Floor,
    Corridor,
    Door,
    Stairs,
    Water,
    Lava,
    Air,
    Cloud,
    Wall,
    Stone,
    Tree,
    IronBars,
    Pool,
}

/// 순간이동 거부 사유
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeleportDeny {
    /// 순간이동 불가 레벨
    NoTeleportLevel,
    /// 엔드게임 내
    InEndgame,
    /// 반마법 (Antimagic)
    Antimagic,
    /// 돌에 매립됨
    BuriedBall,
    /// 인접한 동반자 없음
    NoNextToU,
    /// 혼란으로 실패
    ConfusionOverride,
}

/// 순간이동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeleportResult {
    /// 거부됨
    Denied(TeleportDeny),
    /// 성공 — 새 좌표
    Success { x: i32, y: i32 },
    /// 레벨 이동
    LevelChange { new_level: i32 },
}

/// 위치의 유효성 판정 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosCheckResult {
    /// 유효한 위치
    Ok,
    /// 벽 또는 단단한 바위
    BlockedByWall,
    /// 물 (수영 불가)
    BlockedByWater,
    /// 용암
    BlockedByLava,
    /// 이미 몬스터가 있음
    OccupiedByMonster,
    /// 맵 밖
    OutOfBounds,
}

/// 레벨 순간이동 목적지 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelTeleDest {
    /// 같은 레벨 (실패)
    SameLevel,
    /// 하늘 (음수 레벨) — 생존 가능 (비행/부양)
    Sky { can_fly: bool, can_levitate: bool },
    /// 천국 (레벨 <= -10)
    Heaven,
    /// Cloud 9 (레벨 == -9)
    Cloud9,
    /// 특정 레벨
    SpecificLevel { depth: i32 },
}

/// 순간이동 에너지 비용 계산 입력
#[derive(Debug, Clone)]
pub struct TeleportCostInput {
    /// 주문 레벨 (teleport away = 6)
    pub spell_level: i32,
    /// 아뮬렛 보유 여부
    pub has_amulet: bool,
    /// 현재 에너지
    pub current_energy: i32,
}

/// tele_jump_ok 판정에 필요한 입력
#[derive(Debug, Clone)]
pub struct JumpCheckInput {
    /// 출발 좌표
    pub from_x: i32,
    pub from_y: i32,
    /// 도착 좌표
    pub to_x: i32,
    pub to_y: i32,
}

/// 랜덤 순간이동 레벨 계산의 입력
#[derive(Debug, Clone)]
pub struct RandomLevelInput {
    /// 현재 깊이
    pub current_depth: i32,
    /// 던전 시작 깊이
    pub dungeon_start: i32,
    /// 던전 총 레벨 수
    pub dungeon_levels: i32,
    /// 엔드게임 내인지
    pub in_endgame: bool,
    /// 지옥(Gehennom) 내인지
    pub in_hell: bool,
}

// ============================================================
// 1. tele_jump_ok — 순간이동 위치 유효성 (장거리 점프 가능 여부)
// ============================================================

/// 특정 구역 간 장거리 이동 가능 여부 판정
/// 원본: teleport.c tele_jump_ok()
/// 위저드 타워처럼 구역이 분리된 레벨에서 점프 제한
pub fn tele_jump_ok(
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
    tower_bounds: Option<(i32, i32, i32, i32)>, // (lx, ly, hx, hy)
) -> bool {
    // 구역 제한이 없으면 항상 가능
    let (lx, ly, hx, hy) = match tower_bounds {
        Some(bounds) => bounds,
        None => return true,
    };

    let from_in = within_bounded_area(from_x, from_y, lx, ly, hx, hy);
    let to_in = within_bounded_area(to_x, to_y, lx, ly, hx, hy);

    // 같은 구역이어야 이동 가능
    from_in == to_in
}

/// 직사각형 내부인지 확인
fn within_bounded_area(x: i32, y: i32, lx: i32, ly: i32, hx: i32, hy: i32) -> bool {
    x >= lx && x <= hx && y >= ly && y <= hy
}

// ============================================================
// 2. goodpos_terrain_check — 위치 지형 유효성 검사
// ============================================================

/// 지형에 따른 위치 유효성 판정 (순수 지형 판정만)
/// 원본: teleport.c goodpos() 내 지형 관련 코드
pub fn goodpos_terrain_check(
    terrain: TerrainType,
    can_fly: bool,
    can_swim: bool,
    can_walk_on_water: bool,
    is_flyer: bool,
    likes_lava: bool,
) -> PosCheckResult {
    match terrain {
        TerrainType::Wall | TerrainType::Stone => PosCheckResult::BlockedByWall,
        TerrainType::Water | TerrainType::Pool => {
            if can_swim || can_fly || is_flyer || can_walk_on_water {
                PosCheckResult::Ok
            } else {
                PosCheckResult::BlockedByWater
            }
        }
        TerrainType::Lava => {
            if likes_lava || can_fly || is_flyer {
                PosCheckResult::Ok
            } else {
                PosCheckResult::BlockedByLava
            }
        }
        TerrainType::Floor
        | TerrainType::Corridor
        | TerrainType::Door
        | TerrainType::Stairs
        | TerrainType::Air
        | TerrainType::Cloud
        | TerrainType::Tree
        | TerrainType::IronBars => PosCheckResult::Ok,
    }
}

// ============================================================
// 3. is_valid_teleport_pos — 좌표 범위 판정
// ============================================================

/// 좌표가 맵 범위 내인지 확인
/// 원본: isok() 매크로
pub fn is_valid_teleport_pos(x: i32, y: i32) -> bool {
    x >= 1 && x < COLNO - 1 && y >= 0 && y < ROWNO
}

// ============================================================
// 4. random_teleport_level — 랜덤 순간이동 목적지 레벨
// ============================================================

/// 비제어 레벨 순간이동 시 목적 레벨 결정
/// 원본: teleport.c random_teleport_level()
pub fn random_teleport_level(input: &RandomLevelInput, rng: &mut NetHackRng) -> i32 {
    if input.in_endgame {
        return input.current_depth; // 엔드게임에서는 이동 불가
    }

    let nlev: i32;
    let mut max_depth = input.dungeon_start + input.dungeon_levels - 1;

    // 지옥에서는 최소 깊이 = 던전 시작
    let min_depth = if input.in_hell {
        input.dungeon_start
    } else {
        1
    };

    // rn2(max_depth - min_depth + 1) + min_depth
    let range = max_depth - min_depth + 1;
    if range <= 0 {
        return input.current_depth;
    }

    nlev = rng.rn2(range) + min_depth;
    nlev
}

// ============================================================
// 5. teleport_energy_cost — 순간이동 에너지 비용
// ============================================================

/// 순간이동 주문의 에너지 비용
/// 원본: spelleffects() 통해 energy = spellev * 5, teleport_away는 레벨 6
pub fn teleport_energy_cost(spell_level: i32) -> i32 {
    spell_level * 5
}

// ============================================================
// 6. level_tele_dest_calc — 레벨 순간이동 목적지 분류
// ============================================================

/// 레벨 순간이동 목적지 유형 결정
/// 원본: teleport.c level_tele() 내 newlev 처리 로직
pub fn level_tele_dest_calc(
    new_level: i32,
    current_depth: i32,
    can_fly: bool,
    can_levitate: bool,
) -> LevelTeleDest {
    if new_level == current_depth {
        LevelTeleDest::SameLevel
    } else if new_level <= -10 {
        LevelTeleDest::Heaven
    } else if new_level == -9 {
        LevelTeleDest::Cloud9
    } else if new_level < 0 {
        LevelTeleDest::Sky {
            can_fly,
            can_levitate,
        }
    } else {
        LevelTeleDest::SpecificLevel { depth: new_level }
    }
}

// ============================================================
// 7. tele_trap_resist — 순간이동 함정 저항 판정
// ============================================================

/// 순간이동 함정에 대한 저항 여부 판정
/// 원본: teleport.c tele_trap() 내 In_endgame/Antimagic 분기
pub fn tele_trap_resist(in_endgame: bool, has_antimagic: bool) -> bool {
    in_endgame || has_antimagic
}

// ============================================================
// 8. level_tele_trap_resist — 레벨 텔레포트 함정 저항
// ============================================================

/// 레벨 순간이동 함정에 대한 저항 여부 판정
/// 원본: teleport.c level_tele_trap() 내 Antimagic || In_endgame 분기
pub fn level_tele_trap_resist(has_antimagic: bool, in_endgame: bool) -> bool {
    has_antimagic || in_endgame
}

// ============================================================
// 9. controlled_tele_distance — 제어된 순간이동 거리 검증
// ============================================================

/// 제어된 순간이동 목적지의 거리 검증 (distmin)
/// 원본: teleport.c teleds() 내 거리 제한은 없지만 goodpos 판정 필요
pub fn distmin(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();
    dx.max(dy)
}

/// manhattan 거리 (distu 대체용)
pub fn distu(ux: i32, uy: i32, mx: i32, my: i32) -> i32 {
    let dx = ux - mx;
    let dy = uy - my;
    dx * dx + dy * dy
}

// ============================================================
// 10. safe_teleds_attempts — 안전 텔레포트 시도 횟수/전략
// ============================================================

/// 안전 텔레포트 시 랜덤 위치 생성
/// 원본: teleport.c safe_teleds() 내 400회 시도 로직
/// 반환: (x, y) 후보 좌표
pub fn safe_teleds_candidate(rng: &mut NetHackRng) -> (i32, i32) {
    let x = rng.rn1(COLNO - 3, 2); // 2 ~ COLNO-2
    let y = rng.rn2(ROWNO); // 0 ~ ROWNO-1
    (x, y)
}

/// 안전 텔레포트 최대 시도 횟수
pub const SAFE_TELEDS_MAX_TRIES: i32 = 400;

// ============================================================
// 11. vault_tele_check — 금고 순간이동 여부 판정
// ============================================================

/// vault 텔레포트가 필요한지 판정 (한 번만 작동하는 함정)
/// 원본: tele_trap() 내 trap->once 분기
pub fn vault_tele_check(trap_once: bool) -> bool {
    trap_once
}

// ============================================================
// 12. rloc_candidate — 몬스터 재배치 후보 좌표 생성
// ============================================================

/// 몬스터 순간이동 시 랜덤 좌표 후보 생성
/// 원본: teleport.c rloc() 내 랜덤 좌표 생성 로직
/// 반환: (x, y)
pub fn rloc_candidate(rng: &mut NetHackRng) -> (i32, i32) {
    let x = rng.rn1(COLNO - 3, 2);
    let y = rng.rn2(ROWNO);
    (x, y)
}

/// 몬스터 재배치 최대 시도 횟수
pub const RLOC_MAX_TRIES: i32 = 1000;
/// 처음 500회는 rloc_pos_ok, 이후는 goodpos만 확인
pub const RLOC_RELAXED_AFTER: i32 = 500;

// ============================================================
// 보조 함수
// ============================================================

/// 혼란 시 레벨 텔레포트 실패 확률 판정
/// 원본: Confusion && rnl(5) → oops
pub fn confusion_tele_override(confused: bool, rng: &mut NetHackRng) -> bool {
    confused && rng.rn2(5) != 0
}

/// 아뮬렛 에너지 소모량 계산 (순간이동 비용 외 추가 소모)
/// 원본: rnd(2 * energy)
pub fn amulet_energy_drain(energy: i32, rng: &mut NetHackRng) -> i32 {
    if energy <= 0 {
        return 0;
    }
    rng.rnd(2 * energy)
}

// ============================================================
// 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::rng::NetHackRng;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    // --- tele_jump_ok ---
    #[test]
    fn test_jump_ok_no_bounds() {
        assert!(tele_jump_ok(1, 1, 50, 10, None));
    }

    #[test]
    fn test_jump_ok_same_zone() {
        // 둘 다 내부
        assert!(tele_jump_ok(5, 5, 8, 8, Some((3, 3, 10, 10))));
    }

    #[test]
    fn test_jump_ok_different_zone() {
        // from 내부, to 외부 → 거부
        assert!(!tele_jump_ok(5, 5, 15, 15, Some((3, 3, 10, 10))));
    }

    #[test]
    fn test_jump_ok_both_outside() {
        assert!(tele_jump_ok(1, 1, 15, 15, Some((3, 3, 10, 10))));
    }

    // --- goodpos_terrain_check ---
    #[test]
    fn test_terrain_floor() {
        assert_eq!(
            goodpos_terrain_check(TerrainType::Floor, false, false, false, false, false),
            PosCheckResult::Ok
        );
    }

    #[test]
    fn test_terrain_wall() {
        assert_eq!(
            goodpos_terrain_check(TerrainType::Wall, true, true, true, true, true),
            PosCheckResult::BlockedByWall
        );
    }

    #[test]
    fn test_terrain_water_no_swim() {
        assert_eq!(
            goodpos_terrain_check(TerrainType::Water, false, false, false, false, false),
            PosCheckResult::BlockedByWater
        );
    }

    #[test]
    fn test_terrain_water_can_swim() {
        assert_eq!(
            goodpos_terrain_check(TerrainType::Water, false, true, false, false, false),
            PosCheckResult::Ok
        );
    }

    #[test]
    fn test_terrain_lava_likes() {
        assert_eq!(
            goodpos_terrain_check(TerrainType::Lava, false, false, false, false, true),
            PosCheckResult::Ok
        );
    }

    #[test]
    fn test_terrain_lava_no_resist() {
        assert_eq!(
            goodpos_terrain_check(TerrainType::Lava, false, false, false, false, false),
            PosCheckResult::BlockedByLava
        );
    }

    // --- is_valid_teleport_pos ---
    #[test]
    fn test_valid_pos() {
        assert!(is_valid_teleport_pos(10, 10));
        assert!(is_valid_teleport_pos(1, 0));
        assert!(!is_valid_teleport_pos(0, 0));
        assert!(!is_valid_teleport_pos(79, 10));
        assert!(!is_valid_teleport_pos(10, 21));
    }

    // --- random_teleport_level ---
    #[test]
    fn test_random_level_normal() {
        let mut rng = test_rng();
        let input = RandomLevelInput {
            current_depth: 5,
            dungeon_start: 1,
            dungeon_levels: 30,
            in_endgame: false,
            in_hell: false,
        };
        let result = random_teleport_level(&input, &mut rng);
        assert!(result >= 1 && result <= 30, "레벨: {}", result);
    }

    #[test]
    fn test_random_level_endgame() {
        let mut rng = test_rng();
        let input = RandomLevelInput {
            current_depth: 50,
            dungeon_start: 1,
            dungeon_levels: 30,
            in_endgame: true,
            in_hell: false,
        };
        assert_eq!(random_teleport_level(&input, &mut rng), 50);
    }

    #[test]
    fn test_random_level_hell() {
        let mut rng = test_rng();
        let input = RandomLevelInput {
            current_depth: 35,
            dungeon_start: 25,
            dungeon_levels: 15,
            in_endgame: false,
            in_hell: true,
        };
        let result = random_teleport_level(&input, &mut rng);
        assert!(result >= 25 && result <= 39, "지옥 레벨: {}", result);
    }

    // --- teleport_energy_cost ---
    #[test]
    fn test_tele_energy() {
        assert_eq!(teleport_energy_cost(6), 30); // teleport away = level 6
        assert_eq!(teleport_energy_cost(1), 5);
    }

    // --- level_tele_dest_calc ---
    #[test]
    fn test_dest_same_level() {
        let dest = level_tele_dest_calc(5, 5, false, false);
        assert_eq!(dest, LevelTeleDest::SameLevel);
    }

    #[test]
    fn test_dest_heaven() {
        let dest = level_tele_dest_calc(-10, 5, false, false);
        assert_eq!(dest, LevelTeleDest::Heaven);
    }

    #[test]
    fn test_dest_cloud9() {
        let dest = level_tele_dest_calc(-9, 5, false, false);
        assert_eq!(dest, LevelTeleDest::Cloud9);
    }

    #[test]
    fn test_dest_sky_fly() {
        let dest = level_tele_dest_calc(-3, 5, true, false);
        assert_eq!(
            dest,
            LevelTeleDest::Sky {
                can_fly: true,
                can_levitate: false
            }
        );
    }

    #[test]
    fn test_dest_specific() {
        let dest = level_tele_dest_calc(10, 5, false, false);
        assert_eq!(dest, LevelTeleDest::SpecificLevel { depth: 10 });
    }

    // --- tele_trap_resist ---
    #[test]
    fn test_trap_resist_endgame() {
        assert!(tele_trap_resist(true, false));
    }

    #[test]
    fn test_trap_resist_antimagic() {
        assert!(tele_trap_resist(false, true));
    }

    #[test]
    fn test_trap_no_resist() {
        assert!(!tele_trap_resist(false, false));
    }

    // --- distmin / distu ---
    #[test]
    fn test_distmin() {
        assert_eq!(distmin(0, 0, 3, 4), 4);
        assert_eq!(distmin(5, 5, 5, 5), 0);
        assert_eq!(distmin(1, 1, 4, 1), 3);
    }

    #[test]
    fn test_distu() {
        assert_eq!(distu(0, 0, 3, 4), 25);
        assert_eq!(distu(5, 5, 5, 5), 0);
    }

    // --- safe_teleds_candidate ---
    #[test]
    fn test_safe_teleds_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let (x, y) = safe_teleds_candidate(&mut rng);
            assert!(x >= 2 && x < COLNO - 1, "x 범위: {}", x);
            assert!(y >= 0 && y < ROWNO, "y 범위: {}", y);
        }
    }

    // --- vault_tele_check ---
    #[test]
    fn test_vault_tele() {
        assert!(vault_tele_check(true));
        assert!(!vault_tele_check(false));
    }

    // --- rloc_candidate ---
    #[test]
    fn test_rloc_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let (x, y) = rloc_candidate(&mut rng);
            assert!(x >= 2 && x < COLNO - 1, "x: {}", x);
            assert!(y >= 0 && y < ROWNO, "y: {}", y);
        }
    }

    // --- confusion_tele_override ---
    #[test]
    fn test_confusion_not_confused() {
        let mut rng = test_rng();
        assert!(!confusion_tele_override(false, &mut rng));
    }

    #[test]
    fn test_confusion_confused_varies() {
        let mut rng = test_rng();
        let mut overrides = 0;
        for _ in 0..100 {
            if confusion_tele_override(true, &mut rng) {
                overrides += 1;
            }
        }
        // ~80% 확률로 override (rn2(5) != 0 → 4/5)
        assert!(overrides > 50, "혼란 override 횟수: {}", overrides);
    }

    // --- amulet_energy_drain ---
    #[test]
    fn test_amulet_drain() {
        let mut rng = test_rng();
        let drain = amulet_energy_drain(30, &mut rng);
        assert!(drain >= 1 && drain <= 60, "소모: {}", drain);
    }

    #[test]
    fn test_amulet_drain_zero() {
        let mut rng = test_rng();
        assert_eq!(amulet_energy_drain(0, &mut rng), 0);
    }
}

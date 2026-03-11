// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-18] 레벨 생성 확장2 (mklev_ext2.rs)
// 원본: NetHack 3.6.7 mklev.c (광물 배치 확률, 특수 방 선택, 함정 선택)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 광물 배치 확률 계산 (원본: mklev.c:885-985 mineralize의 확률부)
// =============================================================================

/// [v2.22.0 R34-18] 광물 배치 확률 입력
#[derive(Debug, Clone)]
pub struct MineralizeParams {
    /// 기본 금 확률 (음수면 기본값)
    pub gold_prob: i32,
    /// 기본 보석 확률 (음수면 기본값)
    pub gem_prob: i32,
    /// 현재 깊이
    pub depth: i32,
    /// 광산 레벨인지
    pub in_mines: bool,
    /// 퀘스트 레벨인지
    pub in_quest: bool,
    /// 스킵 레벨 체크인지
    pub skip_level_checks: bool,
}

/// [v2.22.0 R34-18] 광물 배치 확률 결과
#[derive(Debug, Clone)]
pub struct MineralizeResult {
    /// 최종 금 확률 (1000 기준)
    pub gold_prob: i32,
    /// 최종 보석 확률 (1000 기준)
    pub gem_prob: i32,
}

/// [v2.22.0 R34-18] 광물 배치 확률 계산 (원본: mineralize의 확률 계산부)
pub fn calc_mineralize_probs(params: &MineralizeParams) -> MineralizeResult {
    // 기본값: 깊이 기반
    let mut gold = if params.gold_prob < 0 {
        20 + params.depth / 3
    } else {
        params.gold_prob
    };
    let mut gem = if params.gem_prob < 0 {
        gold / 4
    } else {
        params.gem_prob
    };

    // 레벨 체크 스킵이 아닐 때만 보정
    if !params.skip_level_checks {
        if params.in_mines {
            // 광산: 금 2배, 보석 3배
            gold *= 2;
            gem *= 3;
        } else if params.in_quest {
            // 퀘스트: 금 1/4, 보석 1/6
            gold /= 4;
            gem /= 6;
        }
    }

    MineralizeResult {
        gold_prob: gold,
        gem_prob: gem,
    }
}

// =============================================================================
// [2] 특수 방 선택 (원본: mklev.c:761-793 makelevel의 mkroom 분기)
// =============================================================================

/// [v2.22.0 R34-18] 특수 방 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialRoomType {
    Shop,
    Court,
    LepreHall,
    Zoo,
    Temple,
    BeeHive,
    Morgue,
    AntHole,
    Barracks,
    Swamp,
    CockNest,
    None,
}

/// [v2.22.0 R34-18] 특수 방 선택 입력
#[derive(Debug, Clone)]
pub struct SpecialRoomInput {
    pub depth: i32,
    pub nroom: i32,
    pub room_threshold: i32,
    pub medusa_depth: i32,
    /// 각종 몬스터 제거 여부 (G_GONE)
    pub leprechaun_gone: bool,
    pub killer_bee_gone: bool,
    pub soldier_gone: bool,
    pub cockatrice_gone: bool,
    pub has_anthole_mon: bool,
}

/// [v2.22.0 R34-18] 특수 방 선택 (원본: makelevel의 mkroom 분기)
/// 각 조건과 확률을 순수하게 평가
pub fn select_special_room(input: &SpecialRoomInput, rng: &mut NetHackRng) -> SpecialRoomType {
    let d = input.depth;

    // 상점: 깊이 2~메두사 사이, 방 임계값 이상, depth에 대해 rn2(depth) < 3
    if d > 1 && d < input.medusa_depth && input.nroom >= input.room_threshold && rng.rn2(d) < 3 {
        return SpecialRoomType::Shop;
    }
    if d > 4 && rng.rn2(6) == 0 {
        return SpecialRoomType::Court;
    }
    if d > 5 && !input.leprechaun_gone && rng.rn2(8) == 0 {
        return SpecialRoomType::LepreHall;
    }
    if d > 6 && rng.rn2(7) == 0 {
        return SpecialRoomType::Zoo;
    }
    if d > 8 && rng.rn2(5) == 0 {
        return SpecialRoomType::Temple;
    }
    if d > 9 && !input.killer_bee_gone && rng.rn2(5) == 0 {
        return SpecialRoomType::BeeHive;
    }
    if d > 11 && rng.rn2(6) == 0 {
        return SpecialRoomType::Morgue;
    }
    if d > 12 && input.has_anthole_mon && rng.rn2(8) == 0 {
        return SpecialRoomType::AntHole;
    }
    if d > 14 && !input.soldier_gone && rng.rn2(4) == 0 {
        return SpecialRoomType::Barracks;
    }
    if d > 15 && rng.rn2(6) == 0 {
        return SpecialRoomType::Swamp;
    }
    if d > 16 && !input.cockatrice_gone && rng.rn2(8) == 0 {
        return SpecialRoomType::CockNest;
    }
    SpecialRoomType::None
}

// =============================================================================
// [3] 함정 선택 (원본: mklev.c:1268-1400 mktrap의 kind 선택 분기)
// =============================================================================

/// [v2.22.0 R34-18] 함정 유형 상수
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapKind {
    Arrow,
    Dart,
    Rock,
    SqueakyBoard,
    BearTrap,
    Landmine,
    RollingBoulder,
    SleepGas,
    Rust,
    Fire,
    Pit,
    SpikedPit,
    Hole,
    TrapDoor,
    Teleport,
    LevelTeleport,
    Web,
    Statue,
    Magic,
    AntiMagic,
    Polymorph,
    None,
}

/// [v2.22.0 R34-18] 로그 레벨 함정 선택 (원본: mktrap의 Is_rogue_level 분기)
pub fn select_rogue_trap(rng: &mut NetHackRng) -> TrapKind {
    match rng.rn2(7) {
        0 => TrapKind::BearTrap,
        1 => TrapKind::Arrow,
        2 => TrapKind::Dart,
        3 => TrapKind::TrapDoor,
        4 => TrapKind::Pit,
        5 => TrapKind::SleepGas,
        6 => TrapKind::Rust,
        _ => TrapKind::BearTrap,
    }
}

/// [v2.22.0 R34-18] 일반 함정 유효성 판정 (원본: mktrap의 switch)
/// `level_difficulty`: 현재 레벨 난이도
/// `no_teleport`: 레벨 텔레포트 불가인지
pub fn is_trap_valid_for_level(kind: i32, level_difficulty: u32, no_teleport: bool) -> bool {
    let lvl = level_difficulty;
    match kind {
        // MAGIC_PORTAL(18), VIBRATING_SQUARE(24) → 항상 무효
        18 | 24 => false,
        // ROLLING_BOULDER(6), SLP_GAS(7) → lvl >= 2
        6 | 7 => lvl >= 2,
        // LEVEL_TELEP(15) → lvl >= 5, 텔레포트 가능
        15 => lvl >= 5 && !no_teleport,
        // SPIKED_PIT(11) → lvl >= 5
        11 => lvl >= 5,
        // LANDMINE(5) → lvl >= 6
        5 => lvl >= 6,
        // WEB(16) → lvl >= 7
        16 => lvl >= 7,
        // STATUE(17) → lvl >= 8
        17 => lvl >= 8,
        // TELEPORT(14) → 텔레포트 가능
        14 => !no_teleport,
        // HOLE(12) → 바닥 아님 검사 필요하지만 여기서는 상위 레벨만 판정
        _ => true,
    }
}

// =============================================================================
// [4] 니체/가구 배치 확률 (원본: mklev.c:800-882)
// =============================================================================

/// [v2.22.0 R34-18] 방 내 가구 배치 확률 계산
/// `depth`: 현재 깊이
pub fn calc_furniture_chances(depth: i32) -> FurnitureChances {
    // 함정 간격: 8 - (level_difficulty/6), 최소 2
    let trap_interval = {
        let x = 8 - depth / 6;
        if x <= 1 {
            2
        } else {
            x
        }
    };
    // 묘비 확률: 1/(80 - depth*2), 최소 1/2
    let grave_chance = {
        let x = 80 - depth * 2;
        if x < 2 {
            2
        } else {
            x
        }
    };

    FurnitureChances {
        trap_interval,
        gold_chance: 3,      // 1/3
        fountain_chance: 10, // 1/10
        sink_chance: 60,     // 1/60
        altar_chance: 60,    // 1/60
        grave_chance,
        statue_chance: 20, // 1/20
        graffiti_depth_factor: 27 + 3 * depth.abs(),
    }
}

/// [v2.22.0 R34-18] 방 내 가구 배치 확률 결과
#[derive(Debug, Clone)]
pub struct FurnitureChances {
    /// 함정 배치 간격 (1/x 확률로 반복)
    pub trap_interval: i32,
    /// 금화 확률 (1/x)
    pub gold_chance: i32,
    /// 샘물 확률 (1/x)
    pub fountain_chance: i32,
    /// 싱크대 확률 (1/x)
    pub sink_chance: i32,
    /// 제단 확률 (1/x)
    pub altar_chance: i32,
    /// 묘비 확률 (1/x)
    pub grave_chance: i32,
    /// 석상 확률 (1/x)
    pub statue_chance: i32,
    /// 낙서 확률 계수
    pub graffiti_depth_factor: i32,
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mineralize_default() {
        let result = calc_mineralize_probs(&MineralizeParams {
            gold_prob: -1,
            gem_prob: -1,
            depth: 15,
            in_mines: false,
            in_quest: false,
            skip_level_checks: false,
        });
        assert_eq!(result.gold_prob, 25); // 20 + 15/3 = 25
        assert_eq!(result.gem_prob, 6); // 25/4 = 6
    }

    #[test]
    fn test_mineralize_mines() {
        let result = calc_mineralize_probs(&MineralizeParams {
            gold_prob: -1,
            gem_prob: -1,
            depth: 12,
            in_mines: true,
            in_quest: false,
            skip_level_checks: false,
        });
        assert_eq!(result.gold_prob, 48); // (20+4)*2 = 48
        assert_eq!(result.gem_prob, 18); // 6*3 = 18
    }

    #[test]
    fn test_mineralize_quest() {
        let result = calc_mineralize_probs(&MineralizeParams {
            gold_prob: -1,
            gem_prob: -1,
            depth: 12,
            in_mines: false,
            in_quest: true,
            skip_level_checks: false,
        });
        assert_eq!(result.gold_prob, 6); // 24/4 = 6
        assert_eq!(result.gem_prob, 1); // 6/6 = 1
    }

    #[test]
    fn test_special_room_shallow() {
        let mut rng = NetHackRng::new(42);
        let room = select_special_room(
            &SpecialRoomInput {
                depth: 3,
                nroom: 6,
                room_threshold: 3,
                medusa_depth: 25,
                leprechaun_gone: false,
                killer_bee_gone: false,
                soldier_gone: false,
                cockatrice_gone: false,
                has_anthole_mon: true,
            },
            &mut rng,
        );
        // 깊이 3: 상점 가능, 기타는 불가
        assert!(matches!(
            room,
            SpecialRoomType::Shop | SpecialRoomType::None
        ));
    }

    #[test]
    fn test_rogue_trap() {
        let mut rng = NetHackRng::new(42);
        let trap = select_rogue_trap(&mut rng);
        let valid = matches!(
            trap,
            TrapKind::BearTrap
                | TrapKind::Arrow
                | TrapKind::Dart
                | TrapKind::TrapDoor
                | TrapKind::Pit
                | TrapKind::SleepGas
                | TrapKind::Rust
        );
        assert!(valid);
    }

    #[test]
    fn test_trap_validity_portal_blocked() {
        assert!(!is_trap_valid_for_level(18, 20, false)); // MAGIC_PORTAL 항상 무효
    }

    #[test]
    fn test_trap_validity_level_telep() {
        assert!(!is_trap_valid_for_level(15, 3, false)); // 레벨 너무 낮음
        assert!(is_trap_valid_for_level(15, 5, false)); // OK
        assert!(!is_trap_valid_for_level(15, 10, true)); // 텔레포트 불가
    }

    #[test]
    fn test_furniture_chances() {
        let fc = calc_furniture_chances(10);
        assert_eq!(fc.trap_interval, 7); // 8 - 10/6 = 7
        assert_eq!(fc.grave_chance, 60); // 80 - 20 = 60
    }

    #[test]
    fn test_furniture_chances_deep() {
        let fc = calc_furniture_chances(50);
        assert_eq!(fc.trap_interval, 2); // 8 - 50/6 < 2 → 2
        assert_eq!(fc.grave_chance, 2); // 80 - 100 < 2 → 2
    }
}

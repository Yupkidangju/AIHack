// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-14] 핵심 이동/판정 확장 모듈 (hack_ext.rs)
// 원본: NetHack 3.6.7 hack.c (대각선 통과, 과로 판정, 굴착 판정)
// ============================================================================

// =============================================================================
// [1] 대각선 통과 판정 (원본: hack.c:670-699 cant_squeeze_thru)
// =============================================================================

/// [v2.22.0 R34-14] 대각선 통과 불가 사유
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqueezeFailure {
    /// 통과 가능
    CanPass,
    /// 몸이 너무 큼
    TooLarge,
    /// 짐이 너무 많음
    TooHeavy,
    /// 소코반 제한
    Sokoban,
}

/// [v2.22.0 R34-14] 대각선 좁은 통로 통과 판정 (원본: cant_squeeze_thru)
/// `is_big`: bigmonst() 결과
/// `is_amorphous_or_special`: amorphous/whirly/noncorporeal/slithy/can_fog 중 하나 이상
/// `inventory_load`: 무게 부하 (inv_weight()+weight_cap() 또는 curr_mon_load)
/// `is_player`: 플레이어인지
/// `in_sokoban`: 소코반 레벨인지
pub fn check_squeeze_thru(
    is_big: bool,
    is_amorphous_or_special: bool,
    inventory_load: i32,
    is_player: bool,
    in_sokoban: bool,
) -> SqueezeFailure {
    // 큰 몬스터: 비정형/소용돌이/비실체가 아니면 못 지나감
    if is_big && !is_amorphous_or_special {
        return SqueezeFailure::TooLarge;
    }

    // 짐이 너무 많으면 (600 초과)
    if inventory_load > 600 {
        return SqueezeFailure::TooHeavy;
    }

    // 소코반: 플레이어만 제한
    if is_player && in_sokoban {
        return SqueezeFailure::Sokoban;
    }

    SqueezeFailure::CanPass
}

// =============================================================================
// [2] 과로 판정 (원본: hack.c:1976-1996 overexertion)
// =============================================================================

/// [v2.22.0 R34-14] 과로 상태
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverexertionResult {
    /// 과로 없음
    Normal,
    /// HP 1 감소
    LoseHp,
    /// 기절 (HP가 1 이하)
    PassOut,
}

/// [v2.22.0 R34-14] 전투 시 과로 판정 (원본: overexertion)
/// `current_turn`: 현재 턴
/// `encumbrance`: 무게 등급 (HVY_ENCUMBER=3 이상이면 과로)
/// `current_hp`: 현재 HP
pub fn check_overexertion(
    current_turn: i64,
    encumbrance: i32,
    current_hp: i32,
) -> OverexertionResult {
    // 3턴 중 2턴 과로 + 중하중 이상
    const HVY_ENCUMBER: i32 = 3;
    if (current_turn % 3) != 0 && encumbrance >= HVY_ENCUMBER {
        if current_hp > 1 {
            return OverexertionResult::LoseHp;
        } else {
            return OverexertionResult::PassOut;
        }
    }
    OverexertionResult::Normal
}

// =============================================================================
// [3] 굴착 가능 판정 (원본: hack.c:639-648 may_dig)
// =============================================================================

/// [v2.22.0 R34-14] 타일 유형 상수
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    StoneWall,
    Tree,
    IronBars,
    Door,
    Room,
    Corridor,
    Other,
}

/// [v2.22.0 R34-14] 타일 벽 정보 플래그
pub const W_NONDIGGABLE: u8 = 0x01;
pub const W_NONPASSWALL: u8 = 0x02;

/// [v2.22.0 R34-14] 굴착 가능 판정 (원본: may_dig)
pub fn may_dig(tile: TileType, wall_info: u8) -> bool {
    match tile {
        TileType::StoneWall | TileType::Tree => (wall_info & W_NONDIGGABLE) == 0,
        _ => true, // 벽/나무가 아니면 항상 가능
    }
}

/// [v2.22.0 R34-14] 통벽 가능 판정 (원본: may_passwall)
pub fn may_passwall(tile: TileType, wall_info: u8) -> bool {
    match tile {
        TileType::StoneWall => (wall_info & W_NONPASSWALL) == 0,
        _ => true,
    }
}

// =============================================================================
// [4] 좌표 거리 계산 (원본: hacklib.c distmin/dist2)
// =============================================================================

/// [v2.22.0 R34-14] 체스보드 거리 (체비셰프 거리, 원본: distmin)
pub fn distmin(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();
    dx.max(dy)
}

/// [v2.22.0 R34-14] 유클리드 거리^2 (원본: dist2)
pub fn dist2(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let dx = x0 - x1;
    let dy = y0 - y1;
    dx * dx + dy * dy
}

/// [v2.22.0 R34-14] 인접 판정 (거리 ≤ 1)
pub fn is_adjacent(x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
    distmin(x0, y0, x1, y1) <= 1
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squeeze_normal() {
        assert_eq!(
            check_squeeze_thru(false, false, 100, true, false),
            SqueezeFailure::CanPass
        );
    }

    #[test]
    fn test_squeeze_too_large() {
        assert_eq!(
            check_squeeze_thru(true, false, 100, true, false),
            SqueezeFailure::TooLarge
        );
    }

    #[test]
    fn test_squeeze_amorphous_passes() {
        assert_eq!(
            check_squeeze_thru(true, true, 100, true, false),
            SqueezeFailure::CanPass
        );
    }

    #[test]
    fn test_squeeze_too_heavy() {
        assert_eq!(
            check_squeeze_thru(false, false, 700, true, false),
            SqueezeFailure::TooHeavy
        );
    }

    #[test]
    fn test_squeeze_sokoban() {
        assert_eq!(
            check_squeeze_thru(false, false, 100, true, true),
            SqueezeFailure::Sokoban
        );
    }

    #[test]
    fn test_overexertion_normal_turn() {
        assert_eq!(check_overexertion(3, 4, 10), OverexertionResult::Normal);
    }

    #[test]
    fn test_overexertion_lose_hp() {
        assert_eq!(check_overexertion(1, 3, 10), OverexertionResult::LoseHp);
    }

    #[test]
    fn test_overexertion_pass_out() {
        assert_eq!(check_overexertion(2, 3, 1), OverexertionResult::PassOut);
    }

    #[test]
    fn test_may_dig_normal_wall() {
        assert!(may_dig(TileType::StoneWall, 0));
    }

    #[test]
    fn test_may_dig_nondiggable() {
        assert!(!may_dig(TileType::StoneWall, W_NONDIGGABLE));
    }

    #[test]
    fn test_may_passwall_normal() {
        assert!(may_passwall(TileType::StoneWall, 0));
    }

    #[test]
    fn test_may_passwall_blocked() {
        assert!(!may_passwall(TileType::StoneWall, W_NONPASSWALL));
    }

    #[test]
    fn test_distmin() {
        assert_eq!(distmin(0, 0, 3, 4), 4);
        assert_eq!(distmin(1, 1, 1, 1), 0);
        assert_eq!(distmin(0, 0, 5, 3), 5);
    }

    #[test]
    fn test_dist2() {
        assert_eq!(dist2(0, 0, 3, 4), 25);
        assert_eq!(dist2(1, 1, 4, 5), 25);
    }

    #[test]
    fn test_is_adjacent() {
        assert!(is_adjacent(5, 5, 5, 6));
        assert!(is_adjacent(5, 5, 6, 6));
        assert!(!is_adjacent(5, 5, 7, 5));
    }
}

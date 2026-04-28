// dig_ext.rs — dig.c 핵심 로직 순수 결과 패턴 이식
// [v2.14.0] 신규 생성: 채굴 진행도/유형/구멍 완료/액체 충전 등 10개 함수
// 원본: NetHack 3.6.7 src/dig.c (2,153줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 채굴 완료 기준 노력치 (아래로)
const DIG_DOWN_COMPLETE: i32 = 250;
/// 채굴 완료 기준 노력치 (옆으로)
const DIG_SIDE_COMPLETE: i32 = 100;

// ============================================================
// 열거형
// ============================================================

/// 채굴 대상 유형
/// 원본: dig.c dig_typ() L141-168
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigType {
    Undiggable,
    Rock,
    Statue,
    Boulder,
    Door,
    Tree,
}

/// 채굴 진행 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigProgressResult {
    /// 아직 진행 중
    Continue,
    /// 완료 (아래 또는 옆 뚫림)
    Complete,
    /// 어질거림으로 실패 (0: 떨어뜨림, 1: 넓은 면, 2: 빗나감)
    Fumbled { fumble_type: i32 },
}

/// 구멍에 채울 액체 유형
/// 원본: dig.c fillholetyp() L503-536
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillType {
    Lava,
    Moat,
    Pool,
    Room, // 액체 없음
}

/// 구멍 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoleType {
    Pit,
    Hole,
}

/// 채굴 불가 사유
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigCheckFailure {
    Stairs,
    Ladder,
    Throne,
    Altar,
    Air,
    Water,
    Nondiggable,
    Boulder,
    PoolOrLava,
    None, // 채굴 가능
}

// ============================================================
// 1. dig_effort_calc — 채굴 노력치 증가량 계산
// ============================================================

/// 한 턴에 추가되는 채굴 노력치
/// 원본: dig.c dig() L300-303
/// 기본: 10 + rn2(5) + abon + spe - erosion + udaminc
pub fn dig_effort_calc(
    abon: i32,
    weapon_spe: i32,
    weapon_erosion: i32,
    damage_inc: i32,
    is_dwarf: bool,
    rng: &mut NetHackRng,
) -> i32 {
    let mut effort = 10 + rng.rn2(5) + abon + weapon_spe - weapon_erosion + damage_inc;
    if is_dwarf {
        effort *= 2;
    }
    effort
}

// ============================================================
// 2. dig_complete_check — 채굴 완료 판정
// ============================================================

/// 현재 노력치로 채굴이 완료되었는지 판정
/// 원본: dig.c dig() L307, L363
pub fn dig_complete_check(effort: i32, digging_down: bool) -> bool {
    let threshold = if digging_down {
        DIG_DOWN_COMPLETE
    } else {
        DIG_SIDE_COMPLETE
    };
    effort > threshold
}

// ============================================================
// 3. holetime — 구멍 완료까지 남은 턴 추정
// ============================================================

/// 상점 주인이 사용하는 구멍 완료 추정 시간
/// 원본: dig.c holetime() L494-500
pub fn holetime(effort: i32) -> i32 {
    (250 - effort) / 20
}

// ============================================================
// 4. fillholetyp — 구멍에 채울 액체 유형 결정
// ============================================================

/// 구멍 주변의 액체 유형에 따라 채울 유형 결정
/// 원본: dig.c fillholetyp() L503-536
pub fn fillholetyp(
    pool_count: i32,
    moat_count: i32,
    lava_count: i32,
    force_fill: bool,
    rng: &mut NetHackRng,
) -> FillType {
    let adj_pool = if force_fill {
        pool_count
    } else {
        pool_count / 3
    };

    if (lava_count > moat_count + adj_pool && rng.rn2(lava_count + 1) != 0)
        || (lava_count > 0 && force_fill)
    {
        FillType::Lava
    } else if (moat_count > 0 && rng.rn2(moat_count + 1) != 0) || (moat_count > 0 && force_fill) {
        FillType::Moat
    } else if (adj_pool > 0 && rng.rn2(adj_pool + 1) != 0) || (adj_pool > 0 && force_fill) {
        FillType::Pool
    } else {
        FillType::Room
    }
}

// ============================================================
// 5. dig_fumble_check — 어질거림 실패 판정
// ============================================================

/// 어질거림 상태에서 채굴 실패 여부와 실패 유형 결정
/// 원본: dig.c dig() L273-297
pub fn dig_fumble_check(is_fumbling: bool, rng: &mut NetHackRng) -> Option<i32> {
    if !is_fumbling {
        return None;
    }

    // 1/3 확률로 어질거림 발동
    if rng.rn2(3) == 0 {
        Some(rng.rn2(3)) // 0=떨어뜨림, 1=넓은 면, 2=빗나감
    } else {
        None
    }
}

// ============================================================
// 6. dig_check_result — 채굴 가능 여부 검사
// ============================================================

/// 특정 위치에서 채굴 가능 여부 검사
/// 원본: dig.c dig_check() L182-237
pub fn dig_check_result(
    on_stairs: bool,
    on_ladder: bool,
    is_throne: bool,
    is_altar: bool,
    is_astral_or_sanctum: bool,
    is_air_level: bool,
    is_water_level: bool,
    is_nondiggable: bool,
    has_boulder: bool,
    is_pool_or_lava: bool,
    by_object: bool,
) -> DigCheckFailure {
    if on_stairs {
        return DigCheckFailure::Stairs;
    }
    if on_ladder {
        return DigCheckFailure::Ladder;
    }
    if is_throne && !by_object {
        return DigCheckFailure::Throne;
    }
    if is_altar && (!by_object || is_astral_or_sanctum) {
        return DigCheckFailure::Altar;
    }
    if is_air_level {
        return DigCheckFailure::Air;
    }
    if is_water_level {
        return DigCheckFailure::Water;
    }
    if is_nondiggable {
        return DigCheckFailure::Nondiggable;
    }
    if has_boulder {
        return DigCheckFailure::Boulder;
    }
    if by_object && is_pool_or_lava {
        return DigCheckFailure::PoolOrLava;
    }
    DigCheckFailure::None
}

// ============================================================
// 7. earth_elemental_spawn — 흙 정령 소환 확률
// ============================================================

/// 대지의 평면에서 채굴 완료 시 몬스터 소환 확률
/// 원본: dig.c dig() L444 — !rn2(3)
pub fn earth_elemental_spawn_check(is_earth_level: bool, rng: &mut NetHackRng) -> bool {
    is_earth_level && rng.rn2(3) == 0
}

/// 소환될 몬스터 유형 결정 (0=대지 정령, 1=Xorn)
/// 원본: dig.c dig() L447-454
pub fn earth_elemental_type(rng: &mut NetHackRng) -> i32 {
    rng.rn2(2)
}

// ============================================================
// 8. pit_trap_turns — 구덩이 함정 턴 계산
// ============================================================

/// 구덩이에 빠졌을 때 갇히는 턴 수
/// 원본: dig.c digactualhole() L628 — rn1(4, 2)
pub fn pit_trap_turns(rng: &mut NetHackRng) -> i32 {
    rng.rn1(4, 2) // 2~5
}

// ============================================================
// 9. tree_fruit_chance — 나무 벌목 시 과일 드랍 확률
// ============================================================

/// 나무를 벌목하면 과일이 드랍할 확률
/// 원본: dig.c dig() L401 — !rn2(5)
pub fn tree_fruit_chance(rng: &mut NetHackRng) -> bool {
    rng.rn2(5) == 0
}

// ============================================================
// 10. bear_trap_dig_result — 곰 함정 속 채굴 결과
// ============================================================

/// 곰 함정에 갇힌 상태에서 채굴 시 자해 vs 파괴
/// 원본: dig.c dig() L327 — rnl(7) > (Fumbling ? 1 : 4)
/// 반환: true면 자해, false면 함정 파괴
pub fn bear_trap_dig_self_hit(is_fumbling: bool, luck: i32, rng: &mut NetHackRng) -> bool {
    let threshold = if is_fumbling { 1 } else { 4 };
    rng.rnl(7, luck) > threshold
}

/// 곰 함정 자해 시 데미지 계산
/// 원본: dig.c dig() L329-334
pub fn bear_trap_self_damage(weapon_damage: i32, damage_bonus: i32, has_boots: bool) -> i32 {
    let mut dmg = weapon_damage + damage_bonus;
    if dmg < 1 {
        dmg = 1;
    } else if has_boots {
        dmg = (dmg + 1) / 2;
    }
    dmg
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

    // --- dig_effort_calc ---
    #[test]
    fn test_effort_normal() {
        let mut rng = test_rng();
        let effort = dig_effort_calc(2, 3, 1, 0, false, &mut rng);
        // 10 + rn2(5) + 2 + 3 - 1 + 0 = 14 + rn2(5) → 14~18
        assert!(effort >= 14 && effort <= 18, "노력치: {}", effort);
    }

    #[test]
    fn test_effort_dwarf() {
        let mut rng = test_rng();
        let effort = dig_effort_calc(2, 3, 1, 0, true, &mut rng);
        // (14+rn2(5))*2 → 28~36
        assert!(effort >= 28 && effort <= 36, "드워프 노력치: {}", effort);
    }

    // --- dig_complete_check ---
    #[test]
    fn test_complete_down() {
        assert!(dig_complete_check(251, true));
        assert!(!dig_complete_check(250, true));
    }

    #[test]
    fn test_complete_side() {
        assert!(dig_complete_check(101, false));
        assert!(!dig_complete_check(100, false));
    }

    // --- holetime ---
    #[test]
    fn test_holetime_calc() {
        assert_eq!(holetime(50), 10);
        assert_eq!(holetime(200), 2);
        assert_eq!(holetime(250), 0);
    }

    // --- fillholetyp ---
    #[test]
    fn test_fill_room() {
        let mut rng = test_rng();
        assert_eq!(fillholetyp(0, 0, 0, false, &mut rng), FillType::Room);
    }

    #[test]
    fn test_fill_lava_dominant() {
        let mut rng = test_rng();
        let mut lava_count = 0;
        for _ in 0..100 {
            if fillholetyp(0, 0, 5, false, &mut rng) == FillType::Lava {
                lava_count += 1;
            }
        }
        assert!(lava_count > 50, "용암 우세 시 결과: {}", lava_count);
    }

    #[test]
    fn test_fill_force() {
        let mut rng = test_rng();
        // force_fill=true, 용암만 있으면 반드시 용암
        assert_eq!(fillholetyp(0, 0, 1, true, &mut rng), FillType::Lava);
    }

    // --- dig_fumble_check ---
    #[test]
    fn test_fumble_not_fumbling() {
        let mut rng = test_rng();
        assert_eq!(dig_fumble_check(false, &mut rng), None);
    }

    #[test]
    fn test_fumble_sometimes() {
        let mut rng = test_rng();
        let mut fumbled = 0;
        for _ in 0..300 {
            if dig_fumble_check(true, &mut rng).is_some() {
                fumbled += 1;
            }
        }
        // ~33% 확률
        assert!(fumbled > 60 && fumbled < 140, "어질거림: {}", fumbled);
    }

    // --- dig_check_result ---
    #[test]
    fn test_check_stairs() {
        assert_eq!(
            dig_check_result(
                true, false, false, false, false, false, false, false, false, false, false
            ),
            DigCheckFailure::Stairs
        );
    }

    #[test]
    fn test_check_ok() {
        assert_eq!(
            dig_check_result(
                false, false, false, false, false, false, false, false, false, false, false
            ),
            DigCheckFailure::None
        );
    }

    #[test]
    fn test_check_throne_by_object() {
        // by_object=true이면 옥좌도 채굴 가능
        assert_eq!(
            dig_check_result(
                false, false, true, false, false, false, false, false, false, false, true
            ),
            DigCheckFailure::None
        );
    }

    // --- earth_elemental_spawn_check ---
    #[test]
    fn test_earth_spawn_non_earth() {
        let mut rng = test_rng();
        assert!(!earth_elemental_spawn_check(false, &mut rng));
    }

    // --- pit_trap_turns ---
    #[test]
    fn test_pit_turns() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let turns = pit_trap_turns(&mut rng);
            assert!(turns >= 2 && turns < 6, "구덩이 턴: {}", turns);
        }
    }

    // --- tree_fruit_chance ---
    #[test]
    fn test_tree_fruit() {
        let mut rng = test_rng();
        let mut fruit = 0;
        for _ in 0..500 {
            if tree_fruit_chance(&mut rng) {
                fruit += 1;
            }
        }
        // ~20% 확률
        assert!(fruit > 60 && fruit < 150, "과일 드랍: {}", fruit);
    }

    // --- bear_trap_dig_self_hit ---
    #[test]
    fn test_bear_trap_normal() {
        let mut rng = test_rng();
        let mut self_hit = 0;
        for _ in 0..100 {
            if bear_trap_dig_self_hit(false, 0, &mut rng) {
                self_hit += 1;
            }
        }
        // rnl(7) > 4 → ~2/7 → ~28%
        assert!(self_hit > 10 && self_hit < 50, "자해: {}", self_hit);
    }

    // --- bear_trap_self_damage ---
    #[test]
    fn test_bear_trap_damage() {
        assert_eq!(bear_trap_self_damage(5, 2, false), 7);
        assert_eq!(bear_trap_self_damage(5, 2, true), 4); // (7+1)/2
        assert_eq!(bear_trap_self_damage(0, -5, false), 1); // 최소 1
    }
}

// trap_ext.rs — trap.c 핵심 로직 순수 결과 패턴 이식
// [v2.16.0] 신규 생성: 함정 데미지/부식/탈출/낙하/독 등 12개 함수
// 원본: NetHack 3.6.7 src/trap.c (5,477줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 최대 부식 수치 (MAX_ERODE)
const MAX_ERODE: i32 = 3;

// ============================================================
// 열거형
// ============================================================

/// 부식 유형
/// 원본: trap.c erode_obj() L139-190
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErodeType {
    Burn,
    Rust,
    Rot,
    Corrode,
}

/// 부식 결과
/// 원본: trap.c erode_obj() 반환값 (ER_*)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErodeResult {
    /// 그리스로 보호됨
    Greased,
    /// 아무 일 없음 (내성, 이미 최대 부식 등)
    Nothing,
    /// 부식 진행됨 (erosion 증가)
    Damaged,
    /// 완전 파괴됨
    Destroyed,
}

/// 화염 방어구 연소 슬롯
/// 원본: trap.c burnarmor() L55-128
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurnArmorSlot {
    Helmet,
    CloakOrBodyOrShirt,
    Shield,
    Gloves,
    Boots,
}

/// 함정 유형
/// 원본: trap.c dotrap() 주요 switch 분기
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    Arrow,
    Dart,
    Rock,
    SqueakyBoard,
    BearTrap,
    SleepGas,
    RustTrap,
    FireTrap,
    Pit,
    SpikedPit,
    Hole,
    Trapdoor,
    Teleporter,
    LevelTeleporter,
    Web,
    Landmine,
    MagicPortal,
    AntiMagic,
    PolyTrap,
    VibratingSquare,
    RollingBoulder,
    StatueTrap,
}

/// 함정 탈출 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapEscapeResult {
    /// 부양/비행으로 회피
    FloatOver,
    /// 민첩하게 탈출 (rn2(5) 성공)
    Escaped,
    /// 함정에 걸림
    Triggered,
}

// ============================================================
// 1. erode_check — 부식 판정 (순수)
// ============================================================

/// 아이템 부식 판정
/// 원본: trap.c erode_obj() L139-280
pub fn erode_check(
    erode_type: ErodeType,
    is_greased: bool,
    check_grease: bool,
    is_vulnerable: bool,
    is_erodeproof: bool,
    is_blessed: bool,
    current_erosion: i32,
    allow_destroy: bool,
    luck: i32,
    rng: &mut NetHackRng,
) -> ErodeResult {
    // 그리스 보호 (화염/부패에는 미적용)
    if check_grease && is_greased {
        return ErodeResult::Greased;
    }

    // 침식 대상 아님
    if !is_vulnerable {
        return ErodeResult::Nothing;
    }

    // 부식 방지 또는 이미 알려진 내성
    if is_erodeproof {
        return ErodeResult::Nothing;
    }

    // 축복 아이템: rnl(4) == 0이면 보호됨
    if is_blessed && rng.rnl(4, luck) == 0 {
        return ErodeResult::Nothing;
    }

    // 부식 진행 가능
    if current_erosion < MAX_ERODE {
        return ErodeResult::Damaged;
    }

    // 최대 부식 → 파괴 가능
    if allow_destroy {
        return ErodeResult::Destroyed;
    }

    ErodeResult::Nothing
}

// ============================================================
// 2. grease_wearoff — 그리스 벗겨짐 확률
// ============================================================

/// 그리스로 보호 후 벗겨질 확률
/// 원본: trap.c grease_protect() L303 — !rn2(2)
pub fn grease_wearoff(rng: &mut NetHackRng) -> bool {
    rng.rn2(2) == 0
}

// ============================================================
// 3. burnarmor_slot — 화염 방어구 연소 슬롯 결정
// ============================================================

/// 화염 피해 시 어떤 방어구 슬롯이 연소 대상인지 결정
/// 원본: trap.c burnarmor() L82 — rn2(5)
pub fn burnarmor_slot(rng: &mut NetHackRng) -> BurnArmorSlot {
    match rng.rn2(5) {
        0 => BurnArmorSlot::Helmet,
        1 => BurnArmorSlot::CloakOrBodyOrShirt,
        2 => BurnArmorSlot::Shield,
        3 => BurnArmorSlot::Gloves,
        4 => BurnArmorSlot::Boots,
        _ => unreachable!(),
    }
}

// ============================================================
// 4. trap_escape_check — 함정 탈출 판정
// ============================================================

/// 이미 확인된 함정에서 플레이어가 탈출 가능한지 판정
/// 원본: trap.c dotrap() L931-949
pub fn trap_escape_check(
    is_levitating: bool,
    is_flying: bool,
    is_fumbling: bool,
    is_plunging: bool,
    is_pit_type: bool,
    is_clinger: bool,
    is_sokoban: bool,
    rng: &mut NetHackRng,
) -> TrapEscapeResult {
    // 소코반에서는 탈출 불가
    if is_sokoban && is_pit_type {
        return TrapEscapeResult::Triggered;
    }

    // 부양/비행 + 구덩이/곰 함정이면 회피
    if (is_levitating || (is_flying && !is_plunging)) && is_pit_type {
        return TrapEscapeResult::FloatOver;
    }

    // 어질거림이면 탈출 불가
    if is_fumbling {
        return TrapEscapeResult::Triggered;
    }

    // 1/5 확률로 탈출 (클링어는 구덩이에서 자동 탈출)
    if rng.rn2(5) == 0 || (is_pit_type && is_clinger) {
        return TrapEscapeResult::Escaped;
    }

    TrapEscapeResult::Triggered
}

// ============================================================
// 5. bear_trap_duration — 곰 함정 지속 턴 계산
// ============================================================

/// 곰 함정에 잡히는 턴 수
/// 원본: trap.c dotrap() L1089 — rn1(4,4) → 4~7턴
pub fn bear_trap_duration(rng: &mut NetHackRng) -> i32 {
    rng.rn1(4, 4)
}

/// 곰 함정 데미지
/// 원본: trap.c dotrap() L1073 — d(2,4)
pub fn bear_trap_damage(rng: &mut NetHackRng) -> i32 {
    rng.d(2, 4)
}

// ============================================================
// 6. pit_damage — 구덩이 데미지 계산
// ============================================================

/// 구덩이 낙하 데미지
/// 원본: trap.c dotrap() L1227-1311
pub fn pit_damage(is_spiked: bool, rng: &mut NetHackRng) -> i32 {
    let base = rng.rnd(6); // 기본 낙하 데미지
    if is_spiked {
        base + rng.rnd(10) // 가시 추가 데미지
    } else {
        base
    }
}

/// 구덩이 탈출 턴 수
/// 원본: trap.c dotrap() L1286 — rn1(6,2) → 2~7턴
pub fn pit_escape_turns(rng: &mut NetHackRng) -> i32 {
    rng.rn1(6, 2)
}

// ============================================================
// 7. dart_poison_chance — 다트 독 부여 확률
// ============================================================

/// 다트 함정의 다트에 독이 있을 확률
/// 원본: trap.c dotrap() L997-998 — !rn2(6)
pub fn dart_poison_chance(rng: &mut NetHackRng) -> bool {
    rng.rn2(6) == 0
}

// ============================================================
// 8. sleep_gas_duration — 수면 가스 지속 턴
// ============================================================

/// 수면 가스 함정 지속 턴 수
/// 원본: trap.c dotrap() L1113 — rnd(25)
pub fn sleep_gas_duration(rng: &mut NetHackRng) -> i32 {
    rng.rnd(25)
}

// ============================================================
// 9. rock_trap_damage — 바위 함정 데미지
// ============================================================

/// 바위 함정 데미지 (헬멧에 따라 감소)
/// 원본: trap.c dotrap() L1027-1049
pub fn rock_trap_damage(has_metal_helmet: bool, rng: &mut NetHackRng) -> i32 {
    let base = rng.d(2, 6);
    if has_metal_helmet {
        2 // 금속 헬멧 장착 시 고정 2 데미지
    } else {
        base
    }
}

// ============================================================
// 10. fall_through_depth — 함정 문/구멍 낙하 깊이
// ============================================================

/// 함정 문이나 구멍으로 낙하 시 떨어지는 깊이 계산
/// 원본: trap.c fall_through() L472-474
/// 현재 레벨에서 25% 확률로 추가 층을 건너뜀
pub fn fall_through_depth(current_level: i32, bottom_level: i32, rng: &mut NetHackRng) -> i32 {
    let mut new_level = current_level;
    loop {
        new_level += 1;
        if rng.rn2(4) != 0 || new_level >= bottom_level {
            break;
        }
    }
    new_level.min(bottom_level)
}

// ============================================================
// 11. arrow_trap_empty — 화살 함정 소진 확률
// ============================================================

/// 한번 발사된 화살 함정이 빈 것(소진)으로 판정
/// 원본: trap.c dotrap() L962 — trap->once && !rn2(15)
pub fn arrow_trap_empty(already_fired: bool, rng: &mut NetHackRng) -> bool {
    already_fired && rng.rn2(15) == 0
}

// ============================================================
// 12. rust_trap_slot — 녹 함정 피격 부위
// ============================================================

/// 녹 함정 발동 시 물이 맞는 부위 결정
/// 원본: trap.c dotrap() L1126 — rn2(5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustTrapSlot {
    Head,     // 0
    LeftArm,  // 1
    RightArm, // 2
    Body,     // 3, 4
}

pub fn rust_trap_slot(rng: &mut NetHackRng) -> RustTrapSlot {
    match rng.rn2(5) {
        0 => RustTrapSlot::Head,
        1 => RustTrapSlot::LeftArm,
        2 => RustTrapSlot::RightArm,
        _ => RustTrapSlot::Body,
    }
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

    // --- erode_check ---
    #[test]
    fn test_erode_greased() {
        let mut rng = test_rng();
        let result = erode_check(
            ErodeType::Rust,
            true,
            true,
            true,
            false,
            false,
            0,
            false,
            0,
            &mut rng,
        );
        assert_eq!(result, ErodeResult::Greased);
    }

    #[test]
    fn test_erode_not_vulnerable() {
        let mut rng = test_rng();
        let result = erode_check(
            ErodeType::Rust,
            false,
            false,
            false,
            false,
            false,
            0,
            false,
            0,
            &mut rng,
        );
        assert_eq!(result, ErodeResult::Nothing);
    }

    #[test]
    fn test_erode_erodeproof() {
        let mut rng = test_rng();
        let result = erode_check(
            ErodeType::Rust,
            false,
            false,
            true,
            true,
            false,
            0,
            false,
            0,
            &mut rng,
        );
        assert_eq!(result, ErodeResult::Nothing);
    }

    #[test]
    fn test_erode_damaged() {
        let mut rng = test_rng();
        let result = erode_check(
            ErodeType::Rust,
            false,
            false,
            true,
            false,
            false,
            0,
            false,
            0,
            &mut rng,
        );
        assert_eq!(result, ErodeResult::Damaged);
    }

    #[test]
    fn test_erode_max_destroy() {
        let mut rng = test_rng();
        let result = erode_check(
            ErodeType::Burn,
            false,
            false,
            true,
            false,
            false,
            MAX_ERODE,
            true,
            0,
            &mut rng,
        );
        assert_eq!(result, ErodeResult::Destroyed);
    }

    // --- grease_wearoff ---
    #[test]
    fn test_grease_50_50() {
        let mut rng = test_rng();
        let mut worn = 0;
        for _ in 0..200 {
            if grease_wearoff(&mut rng) {
                worn += 1;
            }
        }
        assert!(worn > 60 && worn < 140, "그리스 벗겨짐: {}", worn);
    }

    // --- burnarmor_slot ---
    #[test]
    fn test_burnarmor_all_slots() {
        let mut rng = test_rng();
        let mut slots = std::collections::HashSet::new();
        for _ in 0..500 {
            slots.insert(format!("{:?}", burnarmor_slot(&mut rng)));
        }
        assert_eq!(slots.len(), 5, "5가지 슬롯 모두 발생");
    }

    // --- trap_escape_check ---
    #[test]
    fn test_escape_levitating() {
        let mut rng = test_rng();
        let result = trap_escape_check(true, false, false, false, true, false, false, &mut rng);
        assert_eq!(result, TrapEscapeResult::FloatOver);
    }

    #[test]
    fn test_escape_sokoban() {
        let mut rng = test_rng();
        let result = trap_escape_check(false, false, false, false, true, false, true, &mut rng);
        assert_eq!(result, TrapEscapeResult::Triggered);
    }

    // --- bear_trap ---
    #[test]
    fn test_bear_trap_duration() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dur = bear_trap_duration(&mut rng);
            assert!(dur >= 4 && dur < 8, "곰 턴: {}", dur);
        }
    }

    #[test]
    fn test_bear_trap_dmg() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dmg = bear_trap_damage(&mut rng);
            assert!(dmg >= 2 && dmg <= 8, "곰 데미지: {}", dmg);
        }
    }

    // --- pit_damage ---
    #[test]
    fn test_pit_spiked() {
        let mut rng = test_rng();
        let mut spiked_total = 0;
        let mut normal_total = 0;
        for _ in 0..100 {
            spiked_total += pit_damage(true, &mut rng);
            normal_total += pit_damage(false, &mut rng);
        }
        // 가시 구덩이가 평균적으로 더 높은 데미지
        assert!(spiked_total > normal_total, "가시 > 일반");
    }

    // --- dart_poison ---
    #[test]
    fn test_dart_poison() {
        let mut rng = test_rng();
        let mut poisoned = 0;
        for _ in 0..600 {
            if dart_poison_chance(&mut rng) {
                poisoned += 1;
            }
        }
        // ~16.7% 확률
        assert!(poisoned > 60 && poisoned < 140, "독 다트: {}", poisoned);
    }

    // --- sleep_gas ---
    #[test]
    fn test_sleep_gas() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dur = sleep_gas_duration(&mut rng);
            assert!(dur >= 1 && dur <= 25, "수면 턴: {}", dur);
        }
    }

    // --- rock_trap ---
    #[test]
    fn test_rock_helmet() {
        let mut rng = test_rng();
        assert_eq!(rock_trap_damage(true, &mut rng), 2);
    }

    #[test]
    fn test_rock_no_helmet() {
        let mut rng = test_rng();
        let dmg = rock_trap_damage(false, &mut rng);
        assert!(dmg >= 2 && dmg <= 12, "바위 데미지: {}", dmg);
    }

    // --- fall_through_depth ---
    #[test]
    fn test_fall_depth() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let depth = fall_through_depth(5, 25, &mut rng);
            assert!(depth >= 6 && depth <= 25, "낙하: {}", depth);
        }
    }

    #[test]
    fn test_fall_at_bottom() {
        let mut rng = test_rng();
        let depth = fall_through_depth(24, 25, &mut rng);
        assert_eq!(depth, 25);
    }

    // --- arrow_trap_empty ---
    #[test]
    fn test_arrow_not_fired() {
        let mut rng = test_rng();
        assert!(!arrow_trap_empty(false, &mut rng));
    }

    #[test]
    fn test_arrow_fired_rare_empty() {
        let mut rng = test_rng();
        let mut empties = 0;
        for _ in 0..1500 {
            if arrow_trap_empty(true, &mut rng) {
                empties += 1;
            }
        }
        // ~6.7% 확률
        assert!(empties > 50 && empties < 170, "빈 함정: {}", empties);
    }

    // --- rust_trap_slot ---
    #[test]
    fn test_rust_slots() {
        let mut rng = test_rng();
        let mut body_count = 0;
        for _ in 0..500 {
            if rust_trap_slot(&mut rng) == RustTrapSlot::Body {
                body_count += 1;
            }
        }
        // Body = 2/5 = 40%
        assert!(body_count > 140 && body_count < 260, "몸: {}", body_count);
    }
}

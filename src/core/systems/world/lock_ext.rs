// ============================================================================
// AIHack - lock_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] lock.c 미이식 함수 완전 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 lock.c (1,120줄)
//
// 이식 대상:
//   lock_action(), picklock(), breakchestlock(), forcelock(),
//   obstructed(), boxlock(), doorlock(), chest_shatter_msg(),
//   pick_lock() 확률 계산, doforce() 무기 검증,
//   doopen_indir() 문 열기 판정, doclose() 문 닫기 판정
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// 상수 정의
// =============================================================================

/// 자물쇠 따기 도구 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickType {
    /// 자물쇠 따개
    LockPick,
    /// 신용카드
    CreditCard,
    /// 해골 열쇠
    SkeletonKey,
}

/// 자물쇠 따기 대상
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockTarget {
    Door,
    Chest,
    LargeBox,
}

/// 문 상태 (원본: D_NODOOR, D_ISOPEN, D_CLOSED, D_LOCKED, D_BROKEN, D_TRAPPED)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorMask {
    NoDoor,
    Open,
    Closed,
    Locked,
    Broken,
}

// =============================================================================
// lock_action — 작업 문자열 생성
// [v2.10.1] lock.c:46-74 이식
// =============================================================================

/// 현재 자물쇠 작업 설명 생성 (원본 lock_action)
pub fn lock_action_text(
    target: LockTarget,
    is_currently_locked: bool,
    pick_type: PickType,
) -> &'static str {
    if !is_currently_locked {
        // 현재 열려있으면 잠그는 중
        match target {
            LockTarget::Door => "locking the door",
            LockTarget::Chest => "locking the chest",
            LockTarget::LargeBox => "locking the box",
        }
    } else {
        // 잠겨있으면 여는 중
        match pick_type {
            PickType::LockPick | PickType::CreditCard => "picking the lock",
            PickType::SkeletonKey => match target {
                LockTarget::Door => "unlocking the door",
                LockTarget::Chest => "unlocking the chest",
                LockTarget::LargeBox => "unlocking the box",
            },
        }
    }
}

// =============================================================================
// pick_lock_chance — 자물쇠 따기 성공 확률
// [v2.10.1] lock.c:421-433 (상자), 503-515 (문) 이식
// =============================================================================

/// 상자 자물쇠 따기 성공 확률 (원본 pick_lock 중 box 분기)
pub fn box_pick_chance(
    pick_type: PickType,
    dexterity: i32,
    is_rogue: bool,
    is_cursed: bool,
) -> i32 {
    let role_bonus = if is_rogue { 1 } else { 0 };
    let mut ch = match pick_type {
        PickType::CreditCard => dexterity + 20 * role_bonus,
        PickType::LockPick => 4 * dexterity + 25 * role_bonus,
        PickType::SkeletonKey => 75 + dexterity,
    };
    if is_cursed {
        ch /= 2;
    }
    ch
}

/// 문 자물쇠 따기 성공 확률 (원본 pick_lock 중 door 분기)
pub fn door_pick_chance(pick_type: PickType, dexterity: i32, is_rogue: bool) -> i32 {
    let role_bonus = if is_rogue { 1 } else { 0 };
    match pick_type {
        PickType::CreditCard => 2 * dexterity + 20 * role_bonus,
        PickType::LockPick => 3 * dexterity + 30 * role_bonus,
        PickType::SkeletonKey => 70 + dexterity,
    }
}

// =============================================================================
// picklock_result — 자물쇠 따기 턴 처리
// [v2.10.1] lock.c:76-166 이식
// =============================================================================

/// 자물쇠 따기 턴 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PicklockTurnResult {
    /// 아직 진행 중
    StillBusy,
    /// 시간 초과 (50턴)
    GaveUp,
    /// 대상이 이동하여 중단
    TargetMoved,
    /// 함정 발견 (마스터 키)
    TrapFound { was_door: bool },
    /// 성공 — 잠금/해제 완료
    Success { new_state_locked: bool },
    /// 문에 함정 발동
    TrapTriggered,
    /// 문 상태 오류 (문 없음, 열림, 부서짐)
    InvalidDoor { reason: &'static str },
}

/// 자물쇠 따기 턴 처리 (원본 picklock)
/// [v2.10.1] lock.c:76-166
pub fn picklock_turn(
    used_time: i32,
    chance: i32,
    is_door: bool,
    door_state: Option<DoorMask>,
    is_trapped: bool,
    has_magic_key: bool,
    rng: &mut NetHackRng,
) -> PicklockTurnResult {
    // 문 상태 확인 (원본:89-99)
    if is_door {
        if let Some(state) = door_state {
            match state {
                DoorMask::NoDoor => {
                    return PicklockTurnResult::InvalidDoor {
                        reason: "This doorway has no door.",
                    }
                }
                DoorMask::Open => {
                    return PicklockTurnResult::InvalidDoor {
                        reason: "You cannot lock an open door.",
                    }
                }
                DoorMask::Broken => {
                    return PicklockTurnResult::InvalidDoor {
                        reason: "This door is broken.",
                    }
                }
                _ => {}
            }
        }
    }

    // 시간 초과 (원본:102-106)
    if used_time >= 50 {
        return PicklockTurnResult::GaveUp;
    }

    // 확률 체크 (원본:108-109)
    if rng.rn2(100) >= chance {
        return PicklockTurnResult::StillBusy;
    }

    // 마스터 키 함정 탐지 (원본:114-143)
    if is_trapped && has_magic_key {
        return PicklockTurnResult::TrapFound { was_door: is_door };
    }

    // 성공 (원본:145-165)
    if is_trapped {
        PicklockTurnResult::TrapTriggered
    } else if is_door {
        let currently_locked = door_state == Some(DoorMask::Locked);
        PicklockTurnResult::Success {
            new_state_locked: !currently_locked,
        }
    } else {
        PicklockTurnResult::Success {
            new_state_locked: false,
        }
    }
}

// =============================================================================
// forcelock_result — 강제 열기 턴 처리
// [v2.10.1] lock.c:222-264 이식
// =============================================================================

/// 강제 열기 턴 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForcelockTurnResult {
    /// 아직 진행 중
    StillBusy,
    /// 시간 초과
    GaveUp,
    /// 무기 파손
    WeaponBroke,
    /// 성공 — 상자 파괴 여부 포함
    Success { box_destroyed: bool },
}

/// 강제 열기 턴 처리 (원본 forcelock)
/// [v2.10.1] lock.c:222-264
pub fn forcelock_turn(
    used_time: i32,
    chance: i32,
    is_blade: bool,
    weapon_spe: i32,
    weapon_erosion: i32,
    is_cursed: bool,
    rng: &mut NetHackRng,
) -> ForcelockTurnResult {
    // 시간 초과 (원본:229-233)
    if used_time >= 50 {
        return ForcelockTurnResult::GaveUp;
    }

    // 날 무기: 파손 확률 (원본:236-248)
    if is_blade {
        // 파손 확률 = (1000 - spe) 중 (992 - erosion*10) 이상
        // +0 무기: 50턴 동안 생존 확률 ≈ 67%
        let threshold = 992 - weapon_erosion * 10;
        if rng.rn2(1000 - weapon_spe) > threshold && !is_cursed {
            return ForcelockTurnResult::WeaponBroke;
        }
    }

    // 확률 체크 (원본:252-253)
    if rng.rn2(100) >= chance {
        return ForcelockTurnResult::StillBusy;
    }

    // 성공 — 둔기로 강제 열기 시 33% 확률로 상자 파괴 (원본:260)
    let box_destroyed = !is_blade && rng.rn2(3) == 0;
    ForcelockTurnResult::Success { box_destroyed }
}

// =============================================================================
// force_weapon_check — 강제 열기 무기 검증
// [v2.10.1] lock.c:541-553 이식
// =============================================================================

/// 무기 유형 분류 (강제 열기용)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceWeaponType {
    /// 날카로운 무기 (단검~창)
    Blade,
    /// 둔기
    Blunt,
    /// 부적합
    Invalid,
}

/// 강제 열기 무기 검증 (원본 doforce 중 무기 체크)
pub fn check_force_weapon(weapon_name: &str, is_weapon_class: bool) -> ForceWeaponType {
    if !is_weapon_class {
        return ForceWeaponType::Invalid;
    }

    let l = weapon_name.to_lowercase();
    // 날카로운 무기 판별 (원본: P_DAGGER~P_LANCE 범위, 도리깨 제외)
    if l.contains("dagger")
        || l.contains("sword")
        || l.contains("axe")
        || l.contains("spear")
        || l.contains("lance")
        || l.contains("knife")
        || l.contains("blade")
        || l.contains("katana")
        || l.contains("scimitar")
    {
        ForceWeaponType::Blade
    } else if l.contains("flail") || l.contains("whip") {
        ForceWeaponType::Invalid // 도리깨/채찍 불가
    } else {
        ForceWeaponType::Blunt
    }
}

// =============================================================================
// obstructed — 문 방해물 확인
// [v2.10.1] lock.c:741-773 이식
// =============================================================================

/// 문이 막혀있는지 확인 (원본 obstructed)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObstructionCheck {
    Clear,
    MonsterBlocking { name: String },
    ObjectBlocking,
}

/// 방해물 확인 (원본 obstructed)
pub fn check_obstruction(
    has_monster: bool,
    monster_name: Option<&str>,
    has_objects: bool,
) -> ObstructionCheck {
    if has_monster {
        return ObstructionCheck::MonsterBlocking {
            name: monster_name.unwrap_or("Some creature").to_string(),
        };
    }
    if has_objects {
        return ObstructionCheck::ObjectBlocking;
    }
    ObstructionCheck::Clear
}

// =============================================================================
// doopen_result — 문 열기 판정
// [v2.10.1] lock.c:631-739 이식
// =============================================================================

/// 문 열기 시도 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenDoorResult {
    /// 성공 — 문 열림
    Opened,
    /// 성공했지만 함정 발동
    OpenedButTrapped,
    /// 실패 — 문이 버팀
    Resisted,
    /// 문이 이미 열려있음/부서짐/없음
    InvalidState { reason: &'static str },
    /// 플레이어가 너무 작음
    TooSmall,
    /// 문 아님
    NoDoor,
}

/// 문 열기 판정 (원본 doopen_indir)
/// [v2.10.1] lock.c:722-736
pub fn try_open_door(
    door_state: DoorMask,
    is_trapped: bool,
    strength: i32,
    dexterity: i32,
    constitution: i32,
    is_verysmall: bool,
    luck_modifier: i32,
    rng: &mut NetHackRng,
) -> OpenDoorResult {
    // 문 상태 확인
    match door_state {
        DoorMask::NoDoor => {
            return OpenDoorResult::InvalidState {
                reason: "way has no door",
            }
        }
        DoorMask::Open => {
            return OpenDoorResult::InvalidState {
                reason: " is already open",
            }
        }
        DoorMask::Broken => {
            return OpenDoorResult::InvalidState {
                reason: " is broken",
            }
        }
        DoorMask::Locked => {
            return OpenDoorResult::InvalidState {
                reason: " is locked",
            }
        }
        DoorMask::Closed => {}
    }

    // 크기 확인 (원본:716-718)
    if is_verysmall {
        return OpenDoorResult::TooSmall;
    }

    // 판정 (원본:722) — rnl(20) < (STR+DEX+CON)/3
    let stat_total = (strength + dexterity + constitution) / 3;
    let roll = {
        let base = rng.rn2(20);
        // rnl 구현: 행운 보정
        (base - luck_modifier).max(0)
    };

    if roll < stat_total {
        if is_trapped {
            OpenDoorResult::OpenedButTrapped
        } else {
            OpenDoorResult::Opened
        }
    } else {
        OpenDoorResult::Resisted
    }
}

// =============================================================================
// doclose_result — 문 닫기 판정
// [v2.10.1] lock.c:775-870 이식
// =============================================================================

/// 문 닫기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseDoorResult {
    /// 성공
    Closed,
    /// 방해물 있음
    Obstructed,
    /// 문 없음/이미 닫힘 등
    InvalidState { reason: &'static str },
}

/// 문 닫기 판정 (원본 doclose)
pub fn try_close_door(door_state: DoorMask, obstructed: bool) -> CloseDoorResult {
    match door_state {
        DoorMask::NoDoor => CloseDoorResult::InvalidState {
            reason: "This doorway has no door.",
        },
        DoorMask::Broken => CloseDoorResult::InvalidState {
            reason: "This door won't close.",
        },
        DoorMask::Closed | DoorMask::Locked => CloseDoorResult::InvalidState {
            reason: "This door is already closed.",
        },
        DoorMask::Open => {
            if obstructed {
                CloseDoorResult::Obstructed
            } else {
                CloseDoorResult::Closed
            }
        }
    }
}

// =============================================================================
// boxlock/doorlock — 마법 자물쇠 조작
// [v2.10.1] lock.c:872-1071 이식
// =============================================================================

/// 마법 효과 유형 (Knock/Wizard Lock)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagicLockEffect {
    Knock,      // 열기 (마법봉/주문)
    WizardLock, // 잠그기 (마법봉/주문)
}

/// 상자에 마법 효과 적용 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoxlockResult {
    /// 잠금 해제됨
    Unlocked,
    /// 잠금됨
    Locked,
    /// 아무 일 없음 (이미 해당 상태)
    NoChange,
    /// 부서진 상자 — 수리됨
    Repaired,
}

/// 상자에 마법 효과 적용 (원본 boxlock)
pub fn boxlock_result(is_locked: bool, is_broken: bool, effect: MagicLockEffect) -> BoxlockResult {
    match effect {
        MagicLockEffect::Knock => {
            if is_locked {
                BoxlockResult::Unlocked
            } else if is_broken {
                BoxlockResult::Repaired
            } else {
                BoxlockResult::NoChange
            }
        }
        MagicLockEffect::WizardLock => {
            if !is_locked && !is_broken {
                BoxlockResult::Locked
            } else if is_broken {
                BoxlockResult::Repaired
            } else {
                BoxlockResult::NoChange
            }
        }
    }
}

/// 문에 마법 효과 적용 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoorlockResult {
    /// 문 열림
    Opened,
    /// 문 잠금 해제
    Unlocked,
    /// 문 잠금
    Locked,
    /// 문 닫힘
    Closed,
    /// 비밀문 노출
    SecretDoorRevealed,
    /// 아무 일 없음
    NoChange,
    /// 문 파괴됨 (함정)
    Destroyed,
}

/// 문에 마법 효과 적용 (원본 doorlock, 간소화)
pub fn doorlock_result(
    door_state: DoorMask,
    is_trapped: bool,
    effect: MagicLockEffect,
) -> DoorlockResult {
    match effect {
        MagicLockEffect::Knock => match door_state {
            DoorMask::Locked => {
                if is_trapped {
                    DoorlockResult::Destroyed
                } else {
                    DoorlockResult::Unlocked
                }
            }
            DoorMask::Closed => DoorlockResult::Opened,
            _ => DoorlockResult::NoChange,
        },
        MagicLockEffect::WizardLock => match door_state {
            DoorMask::Open => DoorlockResult::Closed,
            DoorMask::Closed => DoorlockResult::Locked,
            _ => DoorlockResult::NoChange,
        },
    }
}

// =============================================================================
// chest_shatter_msg — 파괴 메시지
// [v2.10.1] lock.c:1073-1117 이식
// =============================================================================

/// 상자 파괴 시 아이템 파괴 메시지 (원본 chest_shatter_msg)
pub fn chest_shatter_message(item_class: &str) -> &'static str {
    match item_class {
        "potion" => "You hear a muffled shatter.",
        "scroll" | "spellbook" => "You notice a puff of smoke.",
        "food" => "You smell something nasty.",
        "gem" => "You hear a muffled cracking.",
        "weapon" => "You hear a muffled clang.",
        "tool" => "You hear a muffled thud.",
        _ => "You hear a muffled crash.",
    }
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_action_text() {
        assert_eq!(
            lock_action_text(LockTarget::Door, true, PickType::LockPick),
            "picking the lock"
        );
        assert_eq!(
            lock_action_text(LockTarget::Door, false, PickType::SkeletonKey),
            "locking the door"
        );
        assert_eq!(
            lock_action_text(LockTarget::Chest, true, PickType::SkeletonKey),
            "unlocking the chest"
        );
    }

    #[test]
    fn test_box_pick_chance() {
        // 해골 열쇠: 75 + DEX
        assert_eq!(box_pick_chance(PickType::SkeletonKey, 18, false, false), 93);
        // 도적보너스: LockPick = 4*DEX + 25
        assert_eq!(box_pick_chance(PickType::LockPick, 18, true, false), 97);
        // 저주 시 반감
        assert_eq!(box_pick_chance(PickType::LockPick, 18, false, true), 36);
    }

    #[test]
    fn test_door_pick_chance() {
        assert_eq!(door_pick_chance(PickType::SkeletonKey, 18, false), 88);
        assert_eq!(door_pick_chance(PickType::LockPick, 18, true), 84);
    }

    #[test]
    fn test_picklock_turn_success() {
        let mut rng = NetHackRng::new(42);
        let mut success = false;
        for seed in 0..100u64 {
            let mut r = NetHackRng::new(seed);
            let result = picklock_turn(10, 90, true, Some(DoorMask::Locked), false, false, &mut r);
            if let PicklockTurnResult::Success { .. } = result {
                success = true;
                break;
            }
        }
        assert!(success);
    }

    #[test]
    fn test_picklock_timeout() {
        let mut rng = NetHackRng::new(42);
        let result = picklock_turn(50, 90, true, Some(DoorMask::Locked), false, false, &mut rng);
        assert_eq!(result, PicklockTurnResult::GaveUp);
    }

    #[test]
    fn test_picklock_invalid_door() {
        let mut rng = NetHackRng::new(42);
        let result = picklock_turn(5, 90, true, Some(DoorMask::NoDoor), false, false, &mut rng);
        assert!(matches!(result, PicklockTurnResult::InvalidDoor { .. }));
    }

    #[test]
    fn test_forcelock_turn() {
        let mut rng = NetHackRng::new(42);
        let mut success = false;
        for seed in 0..200u64 {
            let mut r = NetHackRng::new(seed);
            let result = forcelock_turn(10, 50, false, 0, 0, false, &mut r);
            if let ForcelockTurnResult::Success { .. } = result {
                success = true;
                break;
            }
        }
        assert!(success);
    }

    #[test]
    fn test_forcelock_weapon_break() {
        let mut broke = false;
        for seed in 0..500u64 {
            let mut r = NetHackRng::new(seed);
            let result = forcelock_turn(10, 50, true, 0, 5, false, &mut r);
            if result == ForcelockTurnResult::WeaponBroke {
                broke = true;
                break;
            }
        }
        assert!(broke);
    }

    #[test]
    fn test_check_force_weapon() {
        assert_eq!(
            check_force_weapon("long sword", true),
            ForceWeaponType::Blade
        );
        assert_eq!(check_force_weapon("mace", true), ForceWeaponType::Blunt);
        assert_eq!(check_force_weapon("flail", true), ForceWeaponType::Invalid);
        assert_eq!(check_force_weapon("shirt", false), ForceWeaponType::Invalid);
    }

    #[test]
    fn test_obstruction() {
        assert_eq!(
            check_obstruction(false, None, false),
            ObstructionCheck::Clear,
        );
        assert!(matches!(
            check_obstruction(true, Some("orc"), false),
            ObstructionCheck::MonsterBlocking { .. },
        ));
    }

    #[test]
    fn test_open_door() {
        let mut rng = NetHackRng::new(42);
        let mut opened = false;
        for seed in 0..100u64 {
            let mut r = NetHackRng::new(seed);
            if let OpenDoorResult::Opened =
                try_open_door(DoorMask::Closed, false, 18, 16, 14, false, 0, &mut r)
            {
                opened = true;
                break;
            }
        }
        assert!(opened);
    }

    #[test]
    fn test_open_locked_door() {
        let mut rng = NetHackRng::new(42);
        let result = try_open_door(DoorMask::Locked, false, 18, 16, 14, false, 0, &mut rng);
        assert!(matches!(result, OpenDoorResult::InvalidState { .. }));
    }

    #[test]
    fn test_close_door() {
        assert_eq!(
            try_close_door(DoorMask::Open, false),
            CloseDoorResult::Closed
        );
        assert_eq!(
            try_close_door(DoorMask::Open, true),
            CloseDoorResult::Obstructed
        );
        assert!(matches!(
            try_close_door(DoorMask::Closed, false),
            CloseDoorResult::InvalidState { .. }
        ));
    }

    #[test]
    fn test_boxlock_knock() {
        assert_eq!(
            boxlock_result(true, false, MagicLockEffect::Knock),
            BoxlockResult::Unlocked
        );
        assert_eq!(
            boxlock_result(false, true, MagicLockEffect::Knock),
            BoxlockResult::Repaired
        );
        assert_eq!(
            boxlock_result(false, false, MagicLockEffect::Knock),
            BoxlockResult::NoChange
        );
    }

    #[test]
    fn test_boxlock_wizard() {
        assert_eq!(
            boxlock_result(false, false, MagicLockEffect::WizardLock),
            BoxlockResult::Locked
        );
        assert_eq!(
            boxlock_result(true, false, MagicLockEffect::WizardLock),
            BoxlockResult::NoChange
        );
    }

    #[test]
    fn test_doorlock_knock() {
        assert_eq!(
            doorlock_result(DoorMask::Locked, false, MagicLockEffect::Knock),
            DoorlockResult::Unlocked
        );
        assert_eq!(
            doorlock_result(DoorMask::Locked, true, MagicLockEffect::Knock),
            DoorlockResult::Destroyed
        );
        assert_eq!(
            doorlock_result(DoorMask::Closed, false, MagicLockEffect::Knock),
            DoorlockResult::Opened
        );
    }

    #[test]
    fn test_doorlock_wizard() {
        assert_eq!(
            doorlock_result(DoorMask::Open, false, MagicLockEffect::WizardLock),
            DoorlockResult::Closed
        );
        assert_eq!(
            doorlock_result(DoorMask::Closed, false, MagicLockEffect::WizardLock),
            DoorlockResult::Locked
        );
    }

    #[test]
    fn test_chest_shatter_msg() {
        assert!(chest_shatter_message("potion").contains("shatter"));
        assert!(chest_shatter_message("scroll").contains("smoke"));
        assert!(chest_shatter_message("gem").contains("cracking"));
    }
}

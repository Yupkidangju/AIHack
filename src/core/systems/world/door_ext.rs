// ============================================================================
// [v2.32.0 R20-3] 문 로직 (door_ext.rs)
// 원본: NetHack 3.6.7 lock.c/mklev.c 문 확장
// 문 상태 전이, 잠금 해제, 부수기, 비밀문
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.32.0 R20-3] 문 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorState {
    Open,
    Closed,
    Locked,
    Broken,
    Secret,
}

/// [v2.32.0 R20-3] 문 조작 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoorActionResult {
    Success(DoorState),
    Failed(String),
    Jammed, // 걸림
    AlreadyInState,
}

/// [v2.32.0 R20-3] 문 열기 (원본: doopen)
pub fn open_door(current: DoorState) -> DoorActionResult {
    match current {
        DoorState::Closed => DoorActionResult::Success(DoorState::Open),
        DoorState::Locked => DoorActionResult::Failed("문이 잠겨 있다.".to_string()),
        DoorState::Open => DoorActionResult::AlreadyInState,
        DoorState::Broken => DoorActionResult::AlreadyInState,
        DoorState::Secret => DoorActionResult::Failed("문이 보이지 않는다.".to_string()),
    }
}

/// [v2.32.0 R20-3] 문 잠금 해제 (원본: pick_lock)
pub fn pick_lock(current: DoorState, dex: i32, rng: &mut NetHackRng) -> DoorActionResult {
    if current != DoorState::Locked {
        return DoorActionResult::AlreadyInState;
    }
    let chance = 30 + dex * 3;
    if rng.rn2(100) < chance {
        DoorActionResult::Success(DoorState::Closed)
    } else {
        DoorActionResult::Jammed
    }
}

/// [v2.32.0 R20-3] 문 부수기 (원본: kick_door)
pub fn kick_door(current: DoorState, str_stat: i32, rng: &mut NetHackRng) -> DoorActionResult {
    match current {
        DoorState::Open | DoorState::Broken => DoorActionResult::AlreadyInState,
        DoorState::Locked | DoorState::Closed => {
            let chance = 20 + str_stat * 4;
            if rng.rn2(100) < chance {
                DoorActionResult::Success(DoorState::Broken)
            } else {
                DoorActionResult::Failed("문이 꿈쩍도 하지 않는다.".to_string())
            }
        }
        DoorState::Secret => DoorActionResult::Failed("벽밖에 없다...".to_string()),
    }
}

/// [v2.32.0 R20-3] 비밀문 탐색 (원본: findit)
pub fn search_secret(perception: i32, rng: &mut NetHackRng) -> bool {
    let chance = 10 + perception * 2;
    rng.rn2(100) < chance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_closed() {
        assert_eq!(
            open_door(DoorState::Closed),
            DoorActionResult::Success(DoorState::Open)
        );
    }

    #[test]
    fn test_open_locked() {
        assert!(matches!(
            open_door(DoorState::Locked),
            DoorActionResult::Failed(_)
        ));
    }

    #[test]
    fn test_pick_lock() {
        let mut success = 0;
        for s in 0..30 {
            let mut rng = NetHackRng::new(s);
            if let DoorActionResult::Success(_) = pick_lock(DoorState::Locked, 18, &mut rng) {
                success += 1;
            }
        }
        assert!(success > 15);
    }

    #[test]
    fn test_kick_broken() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            kick_door(DoorState::Broken, 18, &mut rng),
            DoorActionResult::AlreadyInState
        );
    }

    #[test]
    fn test_search_secret() {
        let mut found = false;
        for s in 0..50 {
            let mut rng = NetHackRng::new(s);
            if search_secret(15, &mut rng) {
                found = true;
                break;
            }
        }
        assert!(found);
    }
}

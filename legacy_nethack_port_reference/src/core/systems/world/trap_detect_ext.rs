// ============================================================================
// [v2.36.0 R24-1] 함정 판정 확장 (trap_detect_ext.rs)
// 원본: NetHack 3.6.7 detect.c/trap.c 확장
// 함정 감지 확률, 유형별 위험도, 해제 판정
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.36.0 R24-1] 함정 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    ArrowTrap,
    DartTrap,
    RockTrap,
    SqueakyBoard,
    BearTrap,
    LandMine,
    RollingBoulder,
    SleepingGas,
    RustTrap,
    FireTrap,
    Pit,
    SpikedPit,
    Hole,
    TrapDoor,
    Teleporter,
    LevelTeleporter,
    MagicPortal,
    Web,
    StatueTrap,
    PolyTrap,
    AntiMagicField,
}

/// [v2.36.0 R24-1] 함정 위험도
pub fn trap_danger(trap: TrapType) -> i32 {
    match trap {
        TrapType::SqueakyBoard => 1,
        TrapType::ArrowTrap | TrapType::DartTrap => 3,
        TrapType::Web | TrapType::RustTrap => 2,
        TrapType::Pit | TrapType::BearTrap => 4,
        TrapType::SpikedPit | TrapType::RockTrap => 5,
        TrapType::SleepingGas | TrapType::FireTrap => 6,
        TrapType::LandMine | TrapType::RollingBoulder => 7,
        TrapType::Hole | TrapType::TrapDoor => 5,
        TrapType::Teleporter | TrapType::LevelTeleporter => 3,
        TrapType::PolyTrap => 6,
        TrapType::StatueTrap => 4,
        TrapType::AntiMagicField => 2,
        TrapType::MagicPortal => 1,
    }
}

/// [v2.36.0 R24-1] 함정 감지 확률 (원본: findtrap)
pub fn detect_trap_chance(perception: i32, trap_danger_val: i32) -> i32 {
    let base = 10 + perception * 3;
    (base - trap_danger_val * 2).clamp(5, 95)
}

/// [v2.36.0 R24-1] 함정 해제 판정
pub fn disarm_trap(dex: i32, trap: TrapType, rng: &mut NetHackRng) -> bool {
    let difficulty = trap_danger(trap) * 10;
    let chance = 30 + dex * 4 - difficulty;
    rng.rn2(100) < chance.clamp(5, 95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_danger() {
        assert_eq!(trap_danger(TrapType::LandMine), 7);
        assert_eq!(trap_danger(TrapType::SqueakyBoard), 1);
    }

    #[test]
    fn test_detect_chance() {
        let high = detect_trap_chance(18, 1);
        let low = detect_trap_chance(5, 7);
        assert!(high > low);
    }

    #[test]
    fn test_disarm_easy() {
        let mut success = 0;
        for s in 0..50 {
            let mut rng = NetHackRng::new(s);
            if disarm_trap(18, TrapType::SqueakyBoard, &mut rng) {
                success += 1;
            }
        }
        assert!(success > 25);
    }

    #[test]
    fn test_disarm_hard() {
        let mut success = 0;
        for s in 0..30 {
            let mut rng = NetHackRng::new(s);
            if disarm_trap(8, TrapType::LandMine, &mut rng) {
                success += 1;
            }
        }
        assert!(success < 15);
    }

    #[test]
    fn test_all_traps_have_danger() {
        let traps = [
            TrapType::ArrowTrap,
            TrapType::Pit,
            TrapType::LandMine,
            TrapType::Web,
            TrapType::PolyTrap,
            TrapType::AntiMagicField,
        ];
        for t in &traps {
            assert!(trap_danger(*t) >= 1);
        }
    }
}

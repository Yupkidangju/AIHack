// ============================================================================
// [v2.37.0 R25-5] 마법 충전 (recharge_ext.rs)
// 원본: NetHack 3.6.7 zap.c recharge 확장
// 완드/도구 충전, 과충전 폭발, BUC 효과
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.37.0 R25-5] 충전 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RechargeResult {
    Success { new_charges: i32 },
    Overcharged, // 과충전 → 폭발
    AlreadyFull,
    Destroyed, // 충전 불가 아이템 파괴
}

/// [v2.37.0 R25-5] 완드 충전 (원본: recharge)
pub fn recharge_wand(
    current_charges: i32,
    max_charges: i32,
    times_recharged: i32,
    blessed: bool,
    cursed: bool,
    rng: &mut NetHackRng,
) -> RechargeResult {
    // 저주된 스크롤 → 파괴
    if cursed {
        return RechargeResult::Destroyed;
    }

    // 과충전 위험 (이전에 충전한 횟수가 많을수록)
    if times_recharged >= 2 {
        let overcharge_chance = times_recharged * 25;
        if rng.rn2(100) < overcharge_chance {
            return RechargeResult::Overcharged;
        }
    }

    // 이미 만충전
    if current_charges >= max_charges {
        return RechargeResult::AlreadyFull;
    }

    // 충전량 계산
    let bonus = if blessed {
        max_charges // 축복 = 완충
    } else {
        (max_charges + 1) / 2 // 일반 = 반충전
    };

    let new_charges = (current_charges + bonus).min(max_charges);
    RechargeResult::Success { new_charges }
}

/// [v2.37.0 R25-5] 도구 충전 (램프, 뿔)
pub fn recharge_tool(current_fuel: i32, max_fuel: i32, blessed: bool) -> RechargeResult {
    if current_fuel >= max_fuel {
        return RechargeResult::AlreadyFull;
    }
    let fill = if blessed { max_fuel } else { max_fuel / 2 };
    RechargeResult::Success {
        new_charges: (current_fuel + fill).min(max_fuel),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_recharge() {
        let mut rng = NetHackRng::new(42);
        let r = recharge_wand(3, 8, 0, false, false, &mut rng);
        assert!(matches!(r, RechargeResult::Success { .. }));
    }

    #[test]
    fn test_blessed_full() {
        let mut rng = NetHackRng::new(42);
        if let RechargeResult::Success { new_charges } =
            recharge_wand(0, 8, 0, true, false, &mut rng)
        {
            assert_eq!(new_charges, 8);
        }
    }

    #[test]
    fn test_cursed_destroy() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            recharge_wand(3, 8, 0, false, true, &mut rng),
            RechargeResult::Destroyed
        );
    }

    #[test]
    fn test_already_full() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            recharge_wand(8, 8, 0, false, false, &mut rng),
            RechargeResult::AlreadyFull
        );
    }

    #[test]
    fn test_tool() {
        let r = recharge_tool(100, 1500, true);
        if let RechargeResult::Success { new_charges } = r {
            assert_eq!(new_charges, 1500);
        }
    }
}

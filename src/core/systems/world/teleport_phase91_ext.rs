// ============================================================================
// [v2.27.0 Phase 91-4] 텔레포트 확장 (teleport_phase91_ext.rs)
// 원본: NetHack 3.6.7 src/teleport.c L400-1200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 텔레포트 목적지 계산 — teleport_dest (teleport.c L400-600)
// =============================================================================

/// [v2.27.0 91-4] 텔레포트 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeleportResult {
    /// 정상 텔레포트
    Success { x: i32, y: i32 },
    /// 제어된 텔레포트 (플레이어 선택)
    Controlled { x: i32, y: i32, valid: bool },
    /// 텔레포트 불가 (no-teleport 영역)
    Blocked { reason: String },
    /// 같은 위치 (의미 없음)
    SamePosition,
}

/// [v2.27.0 91-4] 텔레포트 목적지 계산
/// 원본: teleport.c tele()
pub fn calculate_teleport_dest(
    current_x: i32,
    current_y: i32,
    has_control: bool,
    desired_x: i32,
    desired_y: i32,
    no_teleport_zone: bool,
    map_width: i32,
    map_height: i32,
    is_walkable: &dyn Fn(i32, i32) -> bool,
    rng: &mut NetHackRng,
) -> TeleportResult {
    // 텔레포트 금지 구역
    if no_teleport_zone {
        return TeleportResult::Blocked {
            reason: "이 장소에서는 텔레포트할 수 없다.".to_string(),
        };
    }

    if has_control {
        // 제어된 텔레포트 — 플레이어 지정 좌표 유효성 검사
        if desired_x < 1
            || desired_x >= map_width - 1
            || desired_y < 1
            || desired_y >= map_height - 1
        {
            return TeleportResult::Controlled {
                x: desired_x,
                y: desired_y,
                valid: false,
            };
        }
        if !is_walkable(desired_x, desired_y) {
            return TeleportResult::Controlled {
                x: desired_x,
                y: desired_y,
                valid: false,
            };
        }
        if desired_x == current_x && desired_y == current_y {
            return TeleportResult::SamePosition;
        }
        return TeleportResult::Controlled {
            x: desired_x,
            y: desired_y,
            valid: true,
        };
    }

    // 무작위 텔레포트 — 유효한 위치 찾기
    for _ in 0..200 {
        let x = rng.rn2(map_width - 2) + 1;
        let y = rng.rn2(map_height - 2) + 1;
        if is_walkable(x, y) && (x != current_x || y != current_y) {
            return TeleportResult::Success { x, y };
        }
    }

    // 200회 시도 실패 → 같은 위치
    TeleportResult::SamePosition
}

// =============================================================================
// [2] 레벨 텔레포트 — level_tele (teleport.c L700-900)
// =============================================================================

/// [v2.27.0 91-4] 레벨 텔레포트 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelTeleResult {
    /// 정상 레벨 변경
    Success { new_level: i32 },
    /// 제어된 레벨 텔레포트
    Controlled { new_level: i32 },
    /// 최하층 너머 (밸리, 위저드 탑 등)
    SpecialLevel { name: String },
    /// 레벨 텔레포트 금지
    Blocked { reason: String },
}

/// [v2.27.0 91-4] 레벨 텔레포트 목적지 계산
pub fn calculate_level_teleport(
    current_level: i32,
    max_level: i32,
    has_control: bool,
    desired_level: i32,
    no_level_tele: bool,
    rng: &mut NetHackRng,
) -> LevelTeleResult {
    if no_level_tele {
        return LevelTeleResult::Blocked {
            reason: "이 던전에서는 레벨 텔레포트가 금지되어 있다.".to_string(),
        };
    }

    if has_control {
        let target = desired_level.max(1).min(max_level);
        if target == current_level {
            return LevelTeleResult::Blocked {
                reason: "현재 레벨과 같다.".to_string(),
            };
        }
        return LevelTeleResult::Controlled { new_level: target };
    }

    // 무작위 레벨
    let random_level = rng.rn2(max_level) + 1;
    if random_level == current_level {
        // 한 번 더 시도
        let retry = rng.rn2(max_level) + 1;
        LevelTeleResult::Success { new_level: retry }
    } else {
        LevelTeleResult::Success {
            new_level: random_level,
        }
    }
}

// =============================================================================
// [3] 텔레포트 부작용 — tele_side_effects (teleport.c L1000-1200)
// =============================================================================

/// [v2.27.0 91-4] 텔레포트 부작용
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeleSideEffect {
    /// 몬스터 교란 (텔레포트 소음)
    DisturbMonsters { radius: i32 },
    /// 기절
    Stun { turns: i32 },
    /// 구속 해제 (곰함정/거미줄)
    FreeFromTrap,
    /// 없음
    None,
}

/// [v2.27.0 91-4] 텔레포트 부작용 판정
pub fn teleport_side_effects(
    was_trapped: bool,
    is_blind: bool,
    rng: &mut NetHackRng,
) -> Vec<TeleSideEffect> {
    let mut effects = Vec::new();

    // 구속 해제
    if was_trapped {
        effects.push(TeleSideEffect::FreeFromTrap);
    }

    // 실명 시 기절 확률
    if is_blind && rng.rn2(3) == 0 {
        effects.push(TeleSideEffect::Stun {
            turns: rng.rn2(3) + 1,
        });
    }

    // 몬스터 교란
    effects.push(TeleSideEffect::DisturbMonsters { radius: 5 });

    effects
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    #[test]
    fn test_random_teleport() {
        let mut rng = test_rng();
        let result =
            calculate_teleport_dest(10, 10, false, 0, 0, false, 80, 21, &|_, _| true, &mut rng);
        assert!(matches!(result, TeleportResult::Success { .. }));
    }

    #[test]
    fn test_controlled_teleport() {
        let mut rng = test_rng();
        let result =
            calculate_teleport_dest(10, 10, true, 20, 15, false, 80, 21, &|_, _| true, &mut rng);
        assert!(matches!(
            result,
            TeleportResult::Controlled { valid: true, .. }
        ));
    }

    #[test]
    fn test_teleport_blocked() {
        let mut rng = test_rng();
        let result =
            calculate_teleport_dest(10, 10, false, 0, 0, true, 80, 21, &|_, _| true, &mut rng);
        assert!(matches!(result, TeleportResult::Blocked { .. }));
    }

    #[test]
    fn test_controlled_invalid_wall() {
        let mut rng = test_rng();
        let result =
            calculate_teleport_dest(10, 10, true, 30, 10, false, 80, 21, &|_, _| false, &mut rng);
        assert!(matches!(
            result,
            TeleportResult::Controlled { valid: false, .. }
        ));
    }

    #[test]
    fn test_level_teleport_random() {
        let mut rng = test_rng();
        let result = calculate_level_teleport(5, 30, false, 0, false, &mut rng);
        assert!(matches!(result, LevelTeleResult::Success { .. }));
    }

    #[test]
    fn test_level_teleport_controlled() {
        let mut rng = test_rng();
        let result = calculate_level_teleport(5, 30, true, 15, false, &mut rng);
        assert!(matches!(
            result,
            LevelTeleResult::Controlled { new_level: 15 }
        ));
    }

    #[test]
    fn test_level_teleport_blocked() {
        let mut rng = test_rng();
        let result = calculate_level_teleport(5, 30, false, 0, true, &mut rng);
        assert!(matches!(result, LevelTeleResult::Blocked { .. }));
    }

    #[test]
    fn test_side_effects_trapped() {
        let mut rng = test_rng();
        let effects = teleport_side_effects(true, false, &mut rng);
        assert!(effects
            .iter()
            .any(|e| matches!(e, TeleSideEffect::FreeFromTrap)));
    }
}

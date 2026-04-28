// ============================================================================
// [v2.27.0 Phase 91-5] 타임아웃/타이머 확장 (timeout_phase91_ext.rs)
// 원본: NetHack 3.6.7 src/timeout.c L300-800 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 상태 이상 타이머 — status_timeout (timeout.c L300-500)
// =============================================================================

/// [v2.27.0 91-5] 상태 이상 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusType {
    Confusion,
    Stun,
    Blind,
    Hallucination,
    Sick,
    Slimed,
    Petrifying,
    Paralyzed,
    Sleeping,
    Levitation,
    Invisibility,
    SeeInvisible,
    SpeedBoost,
    Protection,
    Wounded,
}

/// [v2.27.0 91-5] 상태 타이머 엔트리
#[derive(Debug, Clone)]
pub struct StatusTimer {
    pub status: StatusType,
    pub turns_remaining: i32,
    pub source: String,
    pub severity: i32, // 1=경미, 2=보통, 3=심각, 4=치명
}

/// [v2.27.0 91-5] 타이머 틱 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerTickResult {
    /// 상태 유지
    Continuing { turns_left: i32 },
    /// 상태 종료
    Expired { message: String },
    /// 상태 악화
    Worsened { new_severity: i32, message: String },
    /// 즉사 (석화/질병 완료)
    Fatal { cause: String },
}

/// [v2.27.0 91-5] 상태 타이머 1턴 진행
/// 원본: timeout.c nh_timeout() 내 각 상태별 분기
pub fn tick_status_timer(timer: &StatusTimer, rng: &mut NetHackRng) -> TimerTickResult {
    let new_turns = timer.turns_remaining - 1;

    if new_turns <= 0 {
        // 만료 시 특수 처리
        return match timer.status {
            StatusType::Petrifying => TimerTickResult::Fatal {
                cause: "석화가 완료되었다!".to_string(),
            },
            StatusType::Sick => {
                if timer.severity >= 4 {
                    TimerTickResult::Fatal {
                        cause: "질병으로 사망했다.".to_string(),
                    }
                } else {
                    TimerTickResult::Expired {
                        message: "병이 나았다.".to_string(),
                    }
                }
            }
            StatusType::Slimed => TimerTickResult::Fatal {
                cause: "슬라임으로 변했다!".to_string(),
            },
            StatusType::Confusion => TimerTickResult::Expired {
                message: "혼란이 풀렸다.".to_string(),
            },
            StatusType::Stun => TimerTickResult::Expired {
                message: "기절이 풀렸다.".to_string(),
            },
            StatusType::Blind => TimerTickResult::Expired {
                message: "시야가 돌아왔다!".to_string(),
            },
            StatusType::Hallucination => TimerTickResult::Expired {
                message: "환각이 사라졌다.".to_string(),
            },
            StatusType::Paralyzed => TimerTickResult::Expired {
                message: "다시 움직일 수 있다!".to_string(),
            },
            StatusType::Sleeping => TimerTickResult::Expired {
                message: "잠에서 깨어났다.".to_string(),
            },
            StatusType::Levitation => TimerTickResult::Expired {
                message: "부유가 끝났다.".to_string(),
            },
            StatusType::SpeedBoost => TimerTickResult::Expired {
                message: "속도 증가가 끝났다.".to_string(),
            },
            StatusType::Protection => TimerTickResult::Expired {
                message: "보호 효과가 사라졌다.".to_string(),
            },
            _ => TimerTickResult::Expired {
                message: "효과가 끝났다.".to_string(),
            },
        };
    }

    // 석화/질병 진행 경고
    match timer.status {
        StatusType::Petrifying if new_turns <= 3 => TimerTickResult::Worsened {
            new_severity: 4,
            message: format!("몸이 점점 굳어간다... ({}턴 남음)", new_turns),
        },
        StatusType::Sick if new_turns <= 5 => TimerTickResult::Worsened {
            new_severity: timer.severity + 1,
            message: format!("병이 악화되고 있다... ({}턴 남음)", new_turns),
        },
        StatusType::Slimed if new_turns <= 3 => TimerTickResult::Worsened {
            new_severity: 4,
            message: format!("슬라임이 퍼지고 있다... ({}턴 남음)", new_turns),
        },
        _ => TimerTickResult::Continuing {
            turns_left: new_turns,
        },
    }
}

// =============================================================================
// [2] 아이템 타이머 — obj_timeout (timeout.c L500-700)
// =============================================================================

/// [v2.27.0 91-5] 아이템 타이머 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemTimerType {
    /// 시체 부패
    CorpseRot,
    /// 램프 연료
    LampFuel,
    /// 촛불 연소
    CandleBurn,
    /// 알 부화
    EggHatch,
    /// 구슬 재충전
    OrbRecharge,
    /// 폭탄 타이머
    BombFuse,
}

/// [v2.27.0 91-5] 아이템 타이머 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemTimerResult {
    /// 진행 중
    Continuing { turns_left: i32 },
    /// 시체 부패 완료
    Rotted,
    /// 램프 꺼짐
    LampOut,
    /// 알 부화
    Hatched { monster_type: String },
    /// 촛불 소진
    BurnedOut,
    /// 폭발
    Exploded { damage: i32, radius: i32 },
    /// 재충전 완료
    Recharged,
}

/// [v2.27.0 91-5] 아이템 타이머 1턴 진행
pub fn tick_item_timer(
    timer_type: ItemTimerType,
    turns_remaining: i32,
    rng: &mut NetHackRng,
) -> ItemTimerResult {
    let new_turns = turns_remaining - 1;

    if new_turns <= 0 {
        return match timer_type {
            ItemTimerType::CorpseRot => ItemTimerResult::Rotted,
            ItemTimerType::LampFuel => ItemTimerResult::LampOut,
            ItemTimerType::CandleBurn => ItemTimerResult::BurnedOut,
            ItemTimerType::EggHatch => ItemTimerResult::Hatched {
                monster_type: "해칭 몬스터".to_string(),
            },
            ItemTimerType::OrbRecharge => ItemTimerResult::Recharged,
            ItemTimerType::BombFuse => ItemTimerResult::Exploded {
                damage: rng.rn2(30) + 20,
                radius: 3,
            },
        };
    }

    ItemTimerResult::Continuing {
        turns_left: new_turns,
    }
}

// =============================================================================
// [3] 전역 턴 이벤트 — global_events (timeout.c L700-800)
// =============================================================================

/// [v2.27.0 91-5] 턴 주기 이벤트
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeriodicEvent {
    /// 몬스터 재생 체크
    MonsterRegen,
    /// 플레이어 자연 회복
    PlayerRegen { hp: i32, energy: i32 },
    /// 허기 진행
    HungerTick { drain: i32 },
    /// 함정 재설정
    TrapReset,
    /// 없음
    None,
}

/// [v2.27.0 91-5] 턴 주기 이벤트 판정
pub fn check_periodic_events(
    current_turn: u64,
    player_level: i32,
    hunger_rate: i32,
    rng: &mut NetHackRng,
) -> Vec<PeriodicEvent> {
    let mut events = Vec::new();

    // 매 턴: 허기
    events.push(PeriodicEvent::HungerTick { drain: hunger_rate });

    // 짝수 턴: 몬스터 재생 체크
    if current_turn % 2 == 0 {
        events.push(PeriodicEvent::MonsterRegen);
    }

    // 5턴마다: 플레이어 자연 회복
    if current_turn % 5 == 0 {
        let hp_regen = if player_level >= 10 { 2 } else { 1 };
        let energy_regen = if player_level >= 15 { 2 } else { 1 };
        events.push(PeriodicEvent::PlayerRegen {
            hp: hp_regen,
            energy: energy_regen,
        });
    }

    // 50턴마다: 함정 재설정 확인
    if current_turn % 50 == 0 {
        events.push(PeriodicEvent::TrapReset);
    }

    events
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

    // --- status timer ---

    #[test]
    fn test_confusion_expires() {
        let mut rng = test_rng();
        let timer = StatusTimer {
            status: StatusType::Confusion,
            turns_remaining: 1,
            source: "포션".to_string(),
            severity: 1,
        };
        let result = tick_status_timer(&timer, &mut rng);
        assert!(matches!(result, TimerTickResult::Expired { .. }));
    }

    #[test]
    fn test_petrification_fatal() {
        let mut rng = test_rng();
        let timer = StatusTimer {
            status: StatusType::Petrifying,
            turns_remaining: 1,
            source: "코카트리스".to_string(),
            severity: 4,
        };
        let result = tick_status_timer(&timer, &mut rng);
        assert!(matches!(result, TimerTickResult::Fatal { .. }));
    }

    #[test]
    fn test_petrification_warning() {
        let mut rng = test_rng();
        let timer = StatusTimer {
            status: StatusType::Petrifying,
            turns_remaining: 3,
            source: "코카트리스".to_string(),
            severity: 3,
        };
        let result = tick_status_timer(&timer, &mut rng);
        assert!(matches!(result, TimerTickResult::Worsened { .. }));
    }

    #[test]
    fn test_continuing() {
        let mut rng = test_rng();
        let timer = StatusTimer {
            status: StatusType::Blind,
            turns_remaining: 10,
            source: "포션".to_string(),
            severity: 1,
        };
        let result = tick_status_timer(&timer, &mut rng);
        assert!(matches!(
            result,
            TimerTickResult::Continuing { turns_left: 9 }
        ));
    }

    // --- item timer ---

    #[test]
    fn test_corpse_rot() {
        let mut rng = test_rng();
        let result = tick_item_timer(ItemTimerType::CorpseRot, 1, &mut rng);
        assert!(matches!(result, ItemTimerResult::Rotted));
    }

    #[test]
    fn test_egg_hatch() {
        let mut rng = test_rng();
        let result = tick_item_timer(ItemTimerType::EggHatch, 1, &mut rng);
        assert!(matches!(result, ItemTimerResult::Hatched { .. }));
    }

    #[test]
    fn test_bomb_explode() {
        let mut rng = test_rng();
        let result = tick_item_timer(ItemTimerType::BombFuse, 1, &mut rng);
        assert!(matches!(result, ItemTimerResult::Exploded { .. }));
    }

    // --- periodic events ---

    #[test]
    fn test_periodic_every_turn() {
        let mut rng = test_rng();
        let events = check_periodic_events(1, 10, 1, &mut rng);
        assert!(events
            .iter()
            .any(|e| matches!(e, PeriodicEvent::HungerTick { .. })));
    }

    #[test]
    fn test_periodic_regen() {
        let mut rng = test_rng();
        let events = check_periodic_events(10, 10, 1, &mut rng);
        assert!(events
            .iter()
            .any(|e| matches!(e, PeriodicEvent::PlayerRegen { .. })));
    }

    #[test]
    fn test_periodic_trap_reset() {
        let mut rng = test_rng();
        let events = check_periodic_events(50, 10, 1, &mut rng);
        assert!(events.iter().any(|e| matches!(e, PeriodicEvent::TrapReset)));
    }
}

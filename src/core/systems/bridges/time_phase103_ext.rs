// ============================================================================
// [v2.39.0 Phase 103-2] 시간/턴 관리 통합 (time_phase103_ext.rs)
// 원본: NetHack 3.6.7 src/timeout.c + src/timer.c 시간 관련 통합
// 순수 결과 패턴
//
// 구현 범위:
//   - 턴 카운터 및 시간 경과 관리
//   - 타이머 기반 이벤트 (독, 질병, 속성 변환 등)
//   - 월령/달 위상 계산
//   - 활동 속도 계산 (빠름/보통/느림)
//   - 시간제한 효과 만료 처리
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 턴/시간 관리 — turn_manager
// =============================================================================

/// [v2.39.0 103-2] 타이머 이벤트
#[derive(Debug, Clone)]
pub struct TimerEvent {
    pub name: String,
    pub trigger_turn: i32,
    pub callback_type: TimerCallback,
    pub repeating: bool,
    pub interval: i32,
}

/// [v2.39.0 103-2] 타이머 콜백 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerCallback {
    PoisonDamage,    // 독 데미지
    DiseaseProgress, // 질병 진행
    StoneToFlesh,    // 석화 해제
    PolymorphRevert, // 변이 복구
    HungerTick,      // 배고픔 진행
    TorchBurnout,    // 횃불 소진
    SpellExpire,     // 주문 효과 만료
    TrapReset,       // 함정 리셋
    ShopDebt,        // 상점 빚 이자
    Custom,          // 커스텀
}

/// [v2.39.0 103-2] 시간 관리자
#[derive(Debug, Clone)]
pub struct TimeManager {
    pub current_turn: i32,
    pub timers: Vec<TimerEvent>,
    pub game_start_seed: i64,
    pub moon_phase: i32, // 0~7 달 위상
}

impl TimeManager {
    pub fn new(seed: i64) -> Self {
        let moon = ((seed % 30) as i32).abs() / 4; // 0~7
        Self {
            current_turn: 0,
            timers: Vec::new(),
            game_start_seed: seed,
            moon_phase: moon,
        }
    }

    /// 턴 진행
    pub fn advance_turn(&mut self) -> Vec<(String, TimerCallback)> {
        self.current_turn += 1;
        let mut triggered = Vec::new();

        let mut i = 0;
        while i < self.timers.len() {
            if self.timers[i].trigger_turn <= self.current_turn {
                let event = &self.timers[i];
                triggered.push((event.name.clone(), event.callback_type));

                if event.repeating {
                    self.timers[i].trigger_turn = self.current_turn + self.timers[i].interval;
                    i += 1;
                } else {
                    self.timers.remove(i);
                }
            } else {
                i += 1;
            }
        }

        // 달 위상 갱신 (300턴마다)
        if self.current_turn % 300 == 0 {
            self.moon_phase = (self.moon_phase + 1) % 8;
        }

        triggered
    }

    /// 타이머 등록
    pub fn register_timer(
        &mut self,
        name: &str,
        callback: TimerCallback,
        delay: i32,
        repeating: bool,
        interval: i32,
    ) {
        self.timers.push(TimerEvent {
            name: name.to_string(),
            trigger_turn: self.current_turn + delay,
            callback_type: callback,
            repeating,
            interval,
        });
    }

    /// 특정 타이머 취소
    pub fn cancel_timer(&mut self, name: &str) -> bool {
        let before = self.timers.len();
        self.timers.retain(|t| t.name != name);
        self.timers.len() < before
    }

    /// 활동 타이머 수
    pub fn active_timer_count(&self) -> usize {
        self.timers.len()
    }
}

// =============================================================================
// [2] 속도 계산 — speed_calc
// =============================================================================

/// [v2.39.0 103-2] 이동 속도 계산
pub fn calculate_speed(
    base_speed: i32,  // 기본 속도 (12 = 보통)
    encumbrance: i32, // 짐 무게 (0=없음, 4=최대)
    is_fast: bool,
    is_very_fast: bool,
    is_slow: bool,
    is_hasted: bool,
) -> (i32, String) {
    let mut speed = base_speed;

    if is_very_fast {
        speed += 8;
    } else if is_fast {
        speed += 4;
    }
    if is_hasted {
        speed += 4;
    }
    if is_slow {
        speed -= 6;
    }

    // 무게 패널티
    speed -= encumbrance * 2;

    speed = speed.max(1); // 최소 1

    let desc = match speed {
        1..=6 => "매우 느림",
        7..=10 => "느림",
        11..=14 => "보통",
        15..=18 => "빠름",
        19..=24 => "매우 빠름",
        _ => "초고속",
    };

    (speed, desc.to_string())
}

/// [v2.39.0 103-2] 달 위상 효과
pub fn moon_phase_effect(phase: i32) -> (String, i32) {
    match phase {
        0 => ("신월: 언데드 강화!".to_string(), -2),
        4 => ("보름달: 늑대인간 변이 위험!".to_string(), -3),
        2 | 6 => ("반달: 평범한 밤.".to_string(), 0),
        _ => ("초승달/그믐달: 특별한 일 없음.".to_string(), 0),
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_time_manager() {
        let tm = TimeManager::new(42);
        assert_eq!(tm.current_turn, 0);
        assert!(tm.moon_phase < 8);
    }

    #[test]
    fn test_register_timer() {
        let mut tm = TimeManager::new(42);
        tm.register_timer("독", TimerCallback::PoisonDamage, 5, true, 3);
        assert_eq!(tm.active_timer_count(), 1);
    }

    #[test]
    fn test_trigger_timer() {
        let mut tm = TimeManager::new(42);
        tm.register_timer("독", TimerCallback::PoisonDamage, 3, false, 0);
        for _ in 0..3 {
            tm.advance_turn();
        }
        // 3턴 후 독 타이머 발동+제거
        assert_eq!(tm.active_timer_count(), 0);
    }

    #[test]
    fn test_repeating_timer() {
        let mut tm = TimeManager::new(42);
        tm.register_timer("배고픔", TimerCallback::HungerTick, 2, true, 2);
        tm.advance_turn(); // 1턴
        tm.advance_turn(); // 2턴 → 발동, 다음은 4턴
        assert_eq!(tm.active_timer_count(), 1); // 반복이므로 유지
    }

    #[test]
    fn test_cancel_timer() {
        let mut tm = TimeManager::new(42);
        tm.register_timer("독", TimerCallback::PoisonDamage, 10, false, 0);
        assert!(tm.cancel_timer("독"));
        assert_eq!(tm.active_timer_count(), 0);
    }

    #[test]
    fn test_speed_normal() {
        let (speed, desc) = calculate_speed(12, 0, false, false, false, false);
        assert_eq!(speed, 12);
        assert_eq!(desc, "보통");
    }

    #[test]
    fn test_speed_fast() {
        let (speed, _) = calculate_speed(12, 0, true, false, false, true);
        assert!(speed > 12);
    }

    #[test]
    fn test_speed_encumbered() {
        let (fast, _) = calculate_speed(12, 0, false, false, false, false);
        let (slow, _) = calculate_speed(12, 3, false, false, false, false);
        assert!(fast > slow);
    }

    #[test]
    fn test_moon_phase() {
        let (msg, _) = moon_phase_effect(0);
        assert!(msg.contains("신월"));
        let (msg2, _) = moon_phase_effect(4);
        assert!(msg2.contains("보름달"));
    }
}

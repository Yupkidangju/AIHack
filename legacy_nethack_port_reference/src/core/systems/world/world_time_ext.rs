// ============================================================================
// [v2.37.0 R25-2] 세계 시간 (world_time_ext.rs)
// 원본: NetHack 3.6.7 timeout.c/allmain.c 시간 확장
// 턴 관리, 시간대, 낮/밤, 달 위상
// ============================================================================

/// [v2.37.0 R25-2] 시간대
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    Dawn,  // 새벽 (05:00~07:00)
    Day,   // 낮 (07:00~17:00)
    Dusk,  // 황혼 (17:00~19:00)
    Night, // 밤 (19:00~05:00)
}

/// [v2.37.0 R25-2] 턴→시간 변환
pub fn turn_to_time(turn: u64) -> (i32, i32) {
    let minutes = turn % 1440; // 1440분=24시간
    let hours = (minutes / 60) as i32;
    let mins = (minutes % 60) as i32;
    (hours, mins)
}

/// [v2.37.0 R25-2] 시간대 판정
pub fn time_of_day(turn: u64) -> TimeOfDay {
    let (h, _) = turn_to_time(turn);
    match h {
        5..=6 => TimeOfDay::Dawn,
        7..=16 => TimeOfDay::Day,
        17..=18 => TimeOfDay::Dusk,
        _ => TimeOfDay::Night,
    }
}

/// [v2.37.0 R25-2] 달 위상 (원본: phase_of_the_moon)
pub fn moon_phase(day: u64) -> i32 {
    ((day % 30) as i32 * 12 / 30).clamp(0, 11)
}

/// [v2.37.0 R25-2] 금요일 13일 여부 (원본: friday_13th)
pub fn is_friday_13th(day_of_month: i32, day_of_week: i32) -> bool {
    day_of_month == 13 && day_of_week == 5 // 0=일, 5=금
}

/// [v2.37.0 R25-2] 밤 시야 페널티
pub fn night_vision_penalty(tod: TimeOfDay) -> i32 {
    match tod {
        TimeOfDay::Day => 0,
        TimeOfDay::Dawn | TimeOfDay::Dusk => 1,
        TimeOfDay::Night => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_to_time() {
        assert_eq!(turn_to_time(720), (12, 0)); // 정오
        assert_eq!(turn_to_time(0), (0, 0)); // 자정
    }

    #[test]
    fn test_time_of_day() {
        assert_eq!(time_of_day(720), TimeOfDay::Day); // 12:00
        assert_eq!(time_of_day(0), TimeOfDay::Night); // 00:00
        assert_eq!(time_of_day(360), TimeOfDay::Dawn); // 06:00
    }

    #[test]
    fn test_moon() {
        assert!(moon_phase(15) >= 0);
        assert!(moon_phase(15) <= 11);
    }

    #[test]
    fn test_friday_13th() {
        assert!(is_friday_13th(13, 5));
        assert!(!is_friday_13th(14, 5));
    }

    #[test]
    fn test_night_penalty() {
        assert_eq!(night_vision_penalty(TimeOfDay::Day), 0);
        assert_eq!(night_vision_penalty(TimeOfDay::Night), 3);
    }
}

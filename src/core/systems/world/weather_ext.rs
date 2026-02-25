// ============================================================================
// [v2.38.0 R26-4] 날씨/환경 (weather_ext.rs)
// NetHack 확장: 날씨 시스템 (원본에는 제한적)
// 레벨별 환경 효과, 시야/이동 영향
// ============================================================================

/// [v2.38.0 R26-4] 날씨 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Rain,
    Snow,
    Fog,
    Storm,
    Earthquake, // 일시적
}

/// [v2.38.0 R26-4] 환경 수정자
#[derive(Debug, Clone)]
pub struct EnvironmentModifier {
    pub vision_penalty: i32,
    pub speed_modifier: i32, // 퍼센트 (100=정상)
    pub fire_suppressed: bool,
    pub slip_chance: i32, // 미끄러짐 확률 (0~100)
}

pub fn weather_effects(weather: Weather) -> EnvironmentModifier {
    match weather {
        Weather::Clear => EnvironmentModifier {
            vision_penalty: 0,
            speed_modifier: 100,
            fire_suppressed: false,
            slip_chance: 0,
        },
        Weather::Rain => EnvironmentModifier {
            vision_penalty: 1,
            speed_modifier: 90,
            fire_suppressed: true,
            slip_chance: 5,
        },
        Weather::Snow => EnvironmentModifier {
            vision_penalty: 2,
            speed_modifier: 70,
            fire_suppressed: false,
            slip_chance: 15,
        },
        Weather::Fog => EnvironmentModifier {
            vision_penalty: 4,
            speed_modifier: 95,
            fire_suppressed: false,
            slip_chance: 0,
        },
        Weather::Storm => EnvironmentModifier {
            vision_penalty: 3,
            speed_modifier: 80,
            fire_suppressed: true,
            slip_chance: 10,
        },
        Weather::Earthquake => EnvironmentModifier {
            vision_penalty: 1,
            speed_modifier: 60,
            fire_suppressed: false,
            slip_chance: 30,
        },
    }
}

/// [v2.38.0 R26-4] 레벨 기본 환경
pub fn level_environment(depth: i32, branch: &str) -> Weather {
    match branch {
        "gehennom" => Weather::Clear, // 지옥은 항상 맑음
        "mines" if depth > 10 => Weather::Earthquake,
        "endgame" => Weather::Storm,
        _ => Weather::Clear,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear() {
        let e = weather_effects(Weather::Clear);
        assert_eq!(e.vision_penalty, 0);
        assert_eq!(e.speed_modifier, 100);
    }

    #[test]
    fn test_rain_fire() {
        let e = weather_effects(Weather::Rain);
        assert!(e.fire_suppressed);
    }

    #[test]
    fn test_snow_slow() {
        let e = weather_effects(Weather::Snow);
        assert!(e.speed_modifier < 100);
    }

    #[test]
    fn test_gehennom() {
        assert_eq!(level_environment(30, "gehennom"), Weather::Clear);
    }

    #[test]
    fn test_endgame() {
        assert_eq!(level_environment(1, "endgame"), Weather::Storm);
    }
}

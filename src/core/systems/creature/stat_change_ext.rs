// ============================================================================
// [v2.31.0 R19-5] 능력치 변동 (stat_change_ext.rs)
// 원본: NetHack 3.6.7 attrib.c 확장
// 능력치 증감 규칙, 운동/독서 효과, 노화, 복원
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.31.0 R19-5] 능력치 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Str,
    Dex,
    Con,
    Int,
    Wis,
    Cha,
}

/// [v2.31.0 R19-5] 능력치 변동 원인
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatChangeCause {
    Exercise,       // 운동 (반복 사용)
    PotionGain,     // 포션 of gain ability
    PotionLoss,     // 독, 질병
    Poison,         // 독 (STR 감소)
    Aging,          // 노화
    Curse,          // 저주 효과
    Bless,          // 축복 효과
    RestoreAbility, // 포션 of restore ability
    LevelUp,        // 레벨업 보너스
    UniHorn,        // 유니콘 혼 사용
}

/// [v2.31.0 R19-5] 능력치 변동 결과
#[derive(Debug, Clone)]
pub struct StatChangeResult {
    pub stat: Stat,
    pub old_value: i32,
    pub new_value: i32,
    pub cause: StatChangeCause,
    pub message: String,
}

/// [v2.31.0 R19-5] 능력치 변동 적용 (원본: adjattrib)
pub fn apply_stat_change(
    stat: Stat,
    current: i32,
    delta: i32,
    max_natural: i32,
    cause: StatChangeCause,
) -> StatChangeResult {
    let min_val = 3;
    let new_val = (current + delta).clamp(min_val, max_natural.max(min_val));

    let stat_name = match stat {
        Stat::Str => "힘",
        Stat::Dex => "민첩",
        Stat::Con => "체력",
        Stat::Int => "지능",
        Stat::Wis => "지혜",
        Stat::Cha => "매력",
    };

    let message = if new_val > current {
        format!("{}이(가) 올랐다! ({} → {})", stat_name, current, new_val)
    } else if new_val < current {
        format!("{}이(가) 떨어졌다. ({} → {})", stat_name, current, new_val)
    } else {
        format!("{}은(는) 이미 최대/최소이다.", stat_name)
    };

    StatChangeResult {
        stat,
        old_value: current,
        new_value: new_val,
        cause,
        message,
    }
}

/// [v2.31.0 R19-5] 운동 효과 (원본: exerper)
pub fn exercise_stat(stat: Stat, intensity: i32, rng: &mut NetHackRng) -> i32 {
    // 10% 확률로 +1, 강도에 비례
    if rng.rn2(100) < intensity * 10 {
        1
    } else {
        0
    }
}

/// [v2.31.0 R19-5] 전체 복원 (원본: restore_ability)
pub fn restore_all_stats(current: &[i32; 6], natural_max: &[i32; 6]) -> [i32; 6] {
    let mut restored = *current;
    for i in 0..6 {
        if restored[i] < natural_max[i] {
            restored[i] = natural_max[i];
        }
    }
    restored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_increase() {
        let result = apply_stat_change(Stat::Str, 15, 2, 18, StatChangeCause::PotionGain);
        assert_eq!(result.new_value, 17);
        assert!(result.message.contains("올랐다"));
    }

    #[test]
    fn test_stat_decrease() {
        let result = apply_stat_change(Stat::Str, 15, -3, 18, StatChangeCause::Poison);
        assert_eq!(result.new_value, 12);
    }

    #[test]
    fn test_stat_floor() {
        let result = apply_stat_change(Stat::Dex, 4, -5, 18, StatChangeCause::Curse);
        assert_eq!(result.new_value, 3); // 최소 3
    }

    #[test]
    fn test_stat_ceiling() {
        let result = apply_stat_change(Stat::Con, 17, 5, 18, StatChangeCause::Bless);
        assert_eq!(result.new_value, 18); // 자연 최대
    }

    #[test]
    fn test_restore_all() {
        let current = [10, 12, 8, 15, 14, 11];
        let max = [18, 18, 18, 18, 18, 18];
        let restored = restore_all_stats(&current, &max);
        assert_eq!(restored, [18, 18, 18, 18, 18, 18]);
    }

    #[test]
    fn test_exercise() {
        let mut gained = 0;
        for s in 0..100 {
            let mut rng = NetHackRng::new(s);
            gained += exercise_stat(Stat::Str, 3, &mut rng);
        }
        assert!(gained > 15 && gained < 50); // ~30% 기대
    }
}

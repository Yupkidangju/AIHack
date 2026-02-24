// ============================================================================
// [v2.35.0 R23-1] 채굴 판정 (dig_calc_ext.rs)
// 원본: NetHack 3.6.7 dig.c 확장
// 채굴 속도, 도구 내구도, 지형별 난이도
// ============================================================================

/// [v2.35.0 R23-1] 채굴 도구
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigTool {
    Pickaxe,
    DwarvenMattock,
    WandOfDigging,
    DrumOfEarthquake,
    ForceBreath, // 포스 볼트 등
}

/// [v2.35.0 R23-1] 지형 채굴 난이도
pub fn dig_difficulty(terrain: &str) -> i32 {
    match terrain {
        "rock" => 10,
        "wall" => 15,
        "corridor" => 5,
        "tree" => 20,
        "drawbridge" => 50,
        "undiggable" => i32::MAX,
        _ => 10,
    }
}

/// [v2.35.0 R23-1] 채굴 속도 (턴 수)
pub fn dig_turns(tool: DigTool, difficulty: i32, str_stat: i32) -> i32 {
    if difficulty == i32::MAX {
        return i32::MAX;
    }
    let tool_power = match tool {
        DigTool::Pickaxe => 10,
        DigTool::DwarvenMattock => 15,
        DigTool::WandOfDigging => 100, // 즉시
        DigTool::DrumOfEarthquake => 80,
        DigTool::ForceBreath => 50,
    };
    let str_bonus = (str_stat - 10).max(0);
    let effective = tool_power + str_bonus;
    (difficulty * 10 / effective.max(1)).max(1)
}

/// [v2.35.0 R23-1] 도구 내구도 소모
pub fn tool_wear(tool: DigTool, enchantment: i32) -> i32 {
    let base = match tool {
        DigTool::Pickaxe => 3,
        DigTool::DwarvenMattock => 2,
        _ => 0, // 마법 도구는 소모 없음
    };
    (base - enchantment.min(2)).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty() {
        assert_eq!(dig_difficulty("rock"), 10);
        assert_eq!(dig_difficulty("undiggable"), i32::MAX);
    }

    #[test]
    fn test_wand_instant() {
        let turns = dig_turns(DigTool::WandOfDigging, 15, 10);
        assert_eq!(turns, 1); // 거의 즉시
    }

    #[test]
    fn test_pickaxe_speed() {
        let turns = dig_turns(DigTool::Pickaxe, 10, 18);
        assert!(turns < dig_turns(DigTool::Pickaxe, 10, 10));
    }

    #[test]
    fn test_undiggable() {
        assert_eq!(dig_turns(DigTool::Pickaxe, i32::MAX, 18), i32::MAX);
    }

    #[test]
    fn test_tool_wear() {
        assert_eq!(tool_wear(DigTool::Pickaxe, 0), 3);
        assert_eq!(tool_wear(DigTool::WandOfDigging, 0), 0);
    }
}

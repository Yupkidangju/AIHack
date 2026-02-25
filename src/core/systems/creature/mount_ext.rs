// ============================================================================
// [v2.40.0 R28-3] 몬스터 탈것 (mount_ext.rs)
// 원본: NetHack 3.6.7 steed.c 확장
// 탑승 가능 판정, 탈것 능력치, 전투 보너스
// ============================================================================

/// [v2.40.0 R28-3] 탑승 가능 판정
pub fn can_ride(
    monster_name: &str,
    monster_tame: bool,
    player_riding_skill: i32,
    monster_level: i32,
) -> Result<(), String> {
    if !monster_tame {
        return Err("길들이지 않은 몬스터다.".into());
    }
    if player_riding_skill < 1 {
        return Err("승마 스킬이 없다.".into());
    }

    let rideable = [
        "warhorse",
        "pony",
        "horse",
        "ki-rin",
        "unicorn",
        "dragon",
        "nightmare",
        "wyvern",
    ];
    let is_rideable = rideable
        .iter()
        .any(|r| monster_name.to_lowercase().contains(r));
    if !is_rideable {
        return Err(format!("{}은(는) 탈 수 없다.", monster_name));
    }
    Ok(())
}

/// [v2.40.0 R28-3] 탈것 속도 보너스
pub fn mount_speed_bonus(mount_speed: i32, rider_skill: i32) -> i32 {
    mount_speed + rider_skill * 2
}

/// [v2.40.0 R28-3] 기마 전투 보너스
pub fn mounted_combat_bonus(rider_skill: i32) -> (i32, i32) {
    // (명중 보너스, 데미지 보너스)
    match rider_skill {
        0 => (-2, -2),
        1 => (0, 0),
        2 => (1, 1),
        _ => (2, 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ride_ok() {
        assert!(can_ride("warhorse", true, 1, 5).is_ok());
    }

    #[test]
    fn test_not_tame() {
        assert!(can_ride("warhorse", false, 1, 5).is_err());
    }

    #[test]
    fn test_not_rideable() {
        assert!(can_ride("floating eye", true, 1, 5).is_err());
    }

    #[test]
    fn test_speed() {
        assert_eq!(mount_speed_bonus(20, 3), 26);
    }

    #[test]
    fn test_combat() {
        assert_eq!(mounted_combat_bonus(2), (1, 1));
        assert_eq!(mounted_combat_bonus(0), (-2, -2));
    }
}

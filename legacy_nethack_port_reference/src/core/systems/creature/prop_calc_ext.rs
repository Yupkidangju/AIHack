// ============================================================================
// [v2.33.0 R21-1] 속성 계산 (prop_calc_ext.rs)
// 원본: NetHack 3.6.7 attrib.c/hacklib.c 속성 파생 계산
// AC 합산, 속도 보정, 운반 용량, 에너지 최대
// ============================================================================

/// [v2.33.0 R21-1] AC 합산 (원본: find_ac)
pub fn calculate_ac(
    base_ac: i32,
    armor_bonus: i32,
    shield_bonus: i32,
    ring_bonus: i32,
    spell_bonus: i32,
    dex: i32,
) -> i32 {
    let dex_bonus = if dex >= 16 { (dex - 15) } else { 0 };
    base_ac - armor_bonus - shield_bonus - ring_bonus - spell_bonus - dex_bonus
}

/// [v2.33.0 R21-1] 이동 속도 (원본: calc_speed)
pub fn calculate_speed(
    base_speed: i32,
    encumbrance_level: i32, // 0=없음, 1=짐, 2=과짐, 3=초과짐, 4=압사
    is_hasted: bool,
    is_slowed: bool,
    is_very_fast: bool,
) -> i32 {
    let mut speed = base_speed;
    // 짐 감속
    speed -= encumbrance_level * 3;
    if is_hasted {
        speed += speed / 2;
    }
    if is_very_fast {
        speed += speed / 4;
    }
    if is_slowed {
        speed /= 2;
    }
    speed.max(1)
}

/// [v2.33.0 R21-1] 운반 용량 (원본: calc_capacity)
pub fn carrying_capacity(str_stat: i32, con: i32) -> i32 {
    let base = 25 * (str_stat + con);
    if str_stat >= 18 {
        base + (str_stat - 17) * 50
    } else {
        base
    }
}

/// [v2.33.0 R21-1] 짐 단계 판정
pub fn encumbrance_level(current_weight: i32, capacity: i32) -> i32 {
    let ratio = current_weight * 100 / capacity.max(1);
    match ratio {
        0..=50 => 0,   // 정상
        51..=75 => 1,  // 짐
        76..=90 => 2,  // 과짐
        91..=100 => 3, // 초과짐
        _ => 4,        // 압사
    }
}

/// [v2.33.0 R21-1] 최대 에너지 (원본: calcinit energy)
pub fn max_energy(int: i32, wis: i32, level: i32) -> i32 {
    let base = (int + wis) / 2;
    base + level * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ac() {
        assert_eq!(calculate_ac(10, 5, 2, 1, 0, 18), -1); // 10-5-2-1-0-3=-1
    }

    #[test]
    fn test_speed_haste() {
        assert!(calculate_speed(12, 0, true, false, false) > 12);
    }

    #[test]
    fn test_speed_slow() {
        assert!(calculate_speed(12, 0, false, true, false) < 12);
    }

    #[test]
    fn test_capacity() {
        let cap = carrying_capacity(18, 14);
        assert!(cap > 800);
    }

    #[test]
    fn test_encumbrance() {
        assert_eq!(encumbrance_level(200, 1000), 0);
        assert_eq!(encumbrance_level(800, 1000), 2);
        assert_eq!(encumbrance_level(1200, 1000), 4);
    }

    #[test]
    fn test_energy() {
        assert_eq!(max_energy(18, 16, 10), 37); // (18+16)/2+10*2=17+20
    }
}

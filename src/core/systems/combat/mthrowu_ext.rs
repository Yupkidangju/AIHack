// ============================================================================
// [v2.29.0 R17-5] 몬스터 투척 AI (mthrowu_ext.rs)
// 원본: NetHack 3.6.7 mthrowu.c (870줄)
// 몬스터의 투척 결정, 명중 판정, 포션 투척
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.29.0 R17-5] 투척 가능 아이템
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThrowableItem {
    // 일반 투척
    Dagger,
    Spear,
    Arrow,
    Rock,
    Shuriken,
    // 포션 투척
    PotionAcid,
    PotionBlindness,
    PotionConfusion,
    PotionParalysis,
    PotionSleeping,
}

/// [v2.29.0 R17-5] 투척 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThrowResult {
    Hit { damage: i32 },
    Miss,
    PotionSplash(String),
    OutOfRange,
}

/// [v2.29.0 R17-5] 투척 판정 (원본: thitu)
pub fn throw_hit_check(
    thrower_dex: i32,
    target_ac: i32,
    distance: i32,
    rng: &mut NetHackRng,
) -> bool {
    // 기본 명중: 1d20 + dex보정 >= ac + 거리 페널티
    let roll = rng.rn1(20, 1);
    let dex_bonus = (thrower_dex - 10) / 2;
    let distance_penalty = distance / 3;
    roll + dex_bonus >= target_ac + distance_penalty
}

/// [v2.29.0 R17-5] 투척 데미지 계산
pub fn throw_damage(item: &ThrowableItem, rng: &mut NetHackRng) -> i32 {
    match item {
        ThrowableItem::Dagger => rng.rn1(4, 1),
        ThrowableItem::Spear => rng.rn1(6, 1),
        ThrowableItem::Arrow => rng.rn1(6, 1),
        ThrowableItem::Rock => rng.rn1(3, 1),
        ThrowableItem::Shuriken => rng.rn1(8, 1),
        // 포션은 별도 처리
        _ => 1,
    }
}

/// [v2.29.0 R17-5] 포션 투척 효과
pub fn potion_throw_effect(item: &ThrowableItem) -> Option<String> {
    match item {
        ThrowableItem::PotionAcid => Some("산에 맞았다! (추가 데미지)".to_string()),
        ThrowableItem::PotionBlindness => Some("눈이 보이지 않게 되었다!".to_string()),
        ThrowableItem::PotionConfusion => Some("혼란에 빠졌다!".to_string()),
        ThrowableItem::PotionParalysis => Some("움직일 수 없다!".to_string()),
        ThrowableItem::PotionSleeping => Some("잠이 든다...".to_string()),
        _ => None,
    }
}

/// [v2.29.0 R17-5] 투척 AI 결정 (원본: m_throw)
pub fn decide_throw(
    distance: i32,
    max_range: i32,
    item: &ThrowableItem,
    target_ac: i32,
    thrower_dex: i32,
    rng: &mut NetHackRng,
) -> ThrowResult {
    if distance > max_range {
        return ThrowResult::OutOfRange;
    }

    // 포션: 명중하면 스플래시
    if let Some(effect) = potion_throw_effect(item) {
        if throw_hit_check(thrower_dex, target_ac, distance, rng) {
            return ThrowResult::PotionSplash(effect);
        }
        return ThrowResult::Miss;
    }

    // 일반 투척
    if throw_hit_check(thrower_dex, target_ac, distance, rng) {
        let dmg = throw_damage(item, rng);
        ThrowResult::Hit { damage: dmg }
    } else {
        ThrowResult::Miss
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_check() {
        let mut hits = 0;
        for s in 0..30 {
            let mut rng = NetHackRng::new(s);
            if throw_hit_check(18, 5, 2, &mut rng) {
                hits += 1;
            }
        }
        assert!(hits > 15);
    }

    #[test]
    fn test_damage_dagger() {
        let mut rng = NetHackRng::new(42);
        let dmg = throw_damage(&ThrowableItem::Dagger, &mut rng);
        assert!(dmg >= 1 && dmg <= 4);
    }

    #[test]
    fn test_potion_effect() {
        assert!(potion_throw_effect(&ThrowableItem::PotionAcid).is_some());
        assert!(potion_throw_effect(&ThrowableItem::Rock).is_none());
    }

    #[test]
    fn test_out_of_range() {
        let mut rng = NetHackRng::new(42);
        let result = decide_throw(20, 10, &ThrowableItem::Spear, 5, 15, &mut rng);
        assert_eq!(result, ThrowResult::OutOfRange);
    }

    #[test]
    fn test_decide_potion() {
        let mut found_splash = false;
        for s in 0..30 {
            let mut rng = NetHackRng::new(s);
            if let ThrowResult::PotionSplash(_) =
                decide_throw(3, 10, &ThrowableItem::PotionAcid, 5, 16, &mut rng)
            {
                found_splash = true;
                break;
            }
        }
        assert!(found_splash);
    }
}

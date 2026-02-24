// ============================================================================
// [v2.35.0 R23-4] 분수 효과 (fountain_effect_ext.rs)
// 원본: NetHack 3.6.7 fountain.c 확장
// 분수 음수/담그기 효과, 소원, 뱀, 님프
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.35.0 R23-4] 분수 음수 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrinkEffect {
    Nothing,
    SeeInvisible,
    Poison(i32),
    Heal(i32),
    WaterMoccasin, // 뱀 소환
    WaterNymph,    // 님프 소환 (아이템 도둑)
    Wish,          // 소원! (매우 희귀)
    FountainDries, // 분수 마름
    Oracle,        // 예언
}

/// [v2.35.0 R23-4] 분수 마시기 (원본: drinkfountain)
pub fn drink_fountain(luck: i32, rng: &mut NetHackRng) -> DrinkEffect {
    let roll = rng.rn2(30) - luck.clamp(-3, 3);
    match roll {
        i32::MIN..=-1 => DrinkEffect::Wish,
        0 => DrinkEffect::SeeInvisible,
        1..=3 => DrinkEffect::Heal(rng.rn1(8, 1)),
        4..=6 => DrinkEffect::Poison(rng.rn1(4, 1)),
        7..=9 => DrinkEffect::WaterMoccasin,
        10..=12 => DrinkEffect::WaterNymph,
        13..=15 => DrinkEffect::Oracle,
        16..=20 => DrinkEffect::FountainDries,
        _ => DrinkEffect::Nothing,
    }
}

/// [v2.35.0 R23-4] 분수 담그기 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DipFountainEffect {
    Blessed,   // 성수화
    Cursed,    // 저주물화
    Rust,      // 녹
    Excalibur, // 엑스칼리버 획득!
    Nothing,
}

/// [v2.35.0 R23-4] 분수 담그기 (원본: dipfountain)
pub fn dip_in_fountain(
    item_is_long_sword: bool,
    is_knight: bool,
    level: i32,
    rng: &mut NetHackRng,
) -> DipFountainEffect {
    // 기사 + 롱소드 + 레벨 5+ → 엑스칼리버
    if item_is_long_sword && is_knight && level >= 5 && rng.rn2(3) == 0 {
        return DipFountainEffect::Excalibur;
    }
    let roll = rng.rn2(10);
    match roll {
        0..=2 => DipFountainEffect::Blessed,
        3..=4 => DipFountainEffect::Cursed,
        5..=6 => DipFountainEffect::Rust,
        _ => DipFountainEffect::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drink_outcomes() {
        let mut effects = std::collections::HashSet::new();
        for s in 0..100 {
            let mut rng = NetHackRng::new(s);
            let e = drink_fountain(0, &mut rng);
            effects.insert(format!("{:?}", e));
        }
        assert!(effects.len() >= 4); // 최소 4종 이상 발생
    }

    #[test]
    fn test_drink_lucky_wish() {
        let mut found = false;
        for s in 0..500 {
            let mut rng = NetHackRng::new(s);
            if drink_fountain(3, &mut rng) == DrinkEffect::Wish {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_excalibur() {
        let mut found = false;
        for s in 0..50 {
            let mut rng = NetHackRng::new(s);
            if dip_in_fountain(true, true, 10, &mut rng) == DipFountainEffect::Excalibur {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_no_excalibur_not_knight() {
        for s in 0..30 {
            let mut rng = NetHackRng::new(s);
            assert_ne!(
                dip_in_fountain(true, false, 10, &mut rng),
                DipFountainEffect::Excalibur
            );
        }
    }

    #[test]
    fn test_dip_results() {
        let mut rng = NetHackRng::new(42);
        let r = dip_in_fountain(false, false, 1, &mut rng);
        assert!(matches!(
            r,
            DipFountainEffect::Blessed
                | DipFountainEffect::Cursed
                | DipFountainEffect::Rust
                | DipFountainEffect::Nothing
        ));
    }
}

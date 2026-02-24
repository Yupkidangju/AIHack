// ============================================================================
// [v2.30.0 R18-2] 물약 혼합 (potion_mix_ext.rs)
// 원본: NetHack 3.6.7 potion.c mixtype, dip
// 물약+물약 혼합, 딥핑, 결과 판정
// ============================================================================

/// [v2.30.0 R18-2] 혼합 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MixResult {
    /// 새 물약 생성
    NewPotion(String),
    /// 폭발
    Explosion(i32),
    /// 실패 (안개만)
    Fizzle,
    /// 희석
    Dilute(String),
}

/// [v2.30.0 R18-2] 물약 혼합 규칙 (원본: mixtype)
pub fn mix_potions(potion_a: &str, potion_b: &str) -> MixResult {
    let pair = (potion_a.to_lowercase(), potion_b.to_lowercase());
    match (pair.0.as_str(), pair.1.as_str()) {
        ("healing", "gain energy") | ("gain energy", "healing") => {
            MixResult::NewPotion("extra healing".to_string())
        }
        ("extra healing", "gain energy") | ("gain energy", "extra healing") => {
            MixResult::NewPotion("full healing".to_string())
        }
        ("healing", "speed") | ("speed", "healing") => {
            MixResult::NewPotion("extra healing".to_string())
        }
        ("gain level", "gain level") => MixResult::NewPotion("gain level".to_string()),
        ("acid", "acid") => MixResult::Explosion(20),
        ("water", _) => MixResult::Dilute(pair.1.to_string()),
        (_, "water") => MixResult::Dilute(pair.0.to_string()),
        _ => MixResult::Fizzle,
    }
}

/// [v2.30.0 R18-2] 딥핑 효과 (원본: potion.c dipfountain)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DipResult {
    Blessed,     // 성수에 딥
    Cursed,      // 저주물에 딥
    Poisoned,    // 독약에 딥 (무기)
    Rustproofed, // 기름에 딥
    NoEffect,
}

pub fn dip_item(item_type: &str, potion: &str) -> DipResult {
    match potion {
        "holy water" => DipResult::Blessed,
        "unholy water" => DipResult::Cursed,
        "sickness" | "poison" => DipResult::Poisoned,
        "oil" => DipResult::Rustproofed,
        _ => DipResult::NoEffect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_healing() {
        assert_eq!(
            mix_potions("healing", "gain energy"),
            MixResult::NewPotion("extra healing".to_string())
        );
    }

    #[test]
    fn test_mix_acid_explosion() {
        assert_eq!(mix_potions("acid", "acid"), MixResult::Explosion(20));
    }

    #[test]
    fn test_mix_fizzle() {
        assert_eq!(mix_potions("blindness", "booze"), MixResult::Fizzle);
    }

    #[test]
    fn test_dilute() {
        assert_eq!(
            mix_potions("water", "speed"),
            MixResult::Dilute("speed".to_string())
        );
    }

    #[test]
    fn test_dip_holy() {
        assert_eq!(dip_item("long sword", "holy water"), DipResult::Blessed);
    }
}

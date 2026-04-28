// ============================================================================
// [v2.41.0 R29-1] 완드 효과 (wand_effect_ext.rs)
// 원본: NetHack 3.6.7 zap.c 완드 확장
// 개별 완드 효과, 방향/자기 자신, 반사
// ============================================================================

/// [v2.41.0 R29-1] 완드 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WandEffect {
    MagicMissile(i32),
    Fire(i32),
    Cold(i32),
    Lightning(i32),
    Sleep(i32),
    Death,
    Polymorph,
    Teleport,
    Cancellation,
    MakeInvisible,
    Speed,
    Slow,
    Opening,
    Locking,
    Probing,
    Digging,
    Light,
    Nothing,
    Wishing,
}

pub fn zap_wand(wand_name: &str, at_self: bool) -> WandEffect {
    match wand_name {
        "magic missile" => WandEffect::MagicMissile(if at_self { 4 } else { 12 }),
        "fire" => WandEffect::Fire(if at_self { 6 } else { 18 }),
        "cold" => WandEffect::Cold(if at_self { 6 } else { 18 }),
        "lightning" => WandEffect::Lightning(if at_self { 6 } else { 18 }),
        "sleep" => WandEffect::Sleep(if at_self { 25 } else { 15 }),
        "death" => WandEffect::Death,
        "polymorph" => WandEffect::Polymorph,
        "teleportation" => WandEffect::Teleport,
        "cancellation" => WandEffect::Cancellation,
        "make invisible" => WandEffect::MakeInvisible,
        "speed monster" => WandEffect::Speed,
        "slow monster" => WandEffect::Slow,
        "opening" => WandEffect::Opening,
        "locking" => WandEffect::Locking,
        "probing" => WandEffect::Probing,
        "digging" => WandEffect::Digging,
        "light" => WandEffect::Light,
        "nothing" => WandEffect::Nothing,
        "wishing" => WandEffect::Wishing,
        _ => WandEffect::Nothing,
    }
}

/// [v2.41.0 R29-1] 반사 판정
pub fn beam_reflected(has_reflection: bool) -> bool {
    has_reflection
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fire() {
        assert_eq!(zap_wand("fire", false), WandEffect::Fire(18));
        assert_eq!(zap_wand("fire", true), WandEffect::Fire(6));
    }

    #[test]
    fn test_death() {
        assert_eq!(zap_wand("death", false), WandEffect::Death);
    }

    #[test]
    fn test_wishing() {
        assert_eq!(zap_wand("wishing", false), WandEffect::Wishing);
    }

    #[test]
    fn test_reflect() {
        assert!(beam_reflected(true));
    }

    #[test]
    fn test_unknown() {
        assert_eq!(zap_wand("xyz", false), WandEffect::Nothing);
    }
}

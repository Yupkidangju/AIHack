// ============================================================================
// [v2.30.0 R18-4] 갑옷 강화 (armor_enhance_ext.rs)
// 원본: NetHack 3.6.7 do_wear.c, wield.c enchant
// 인챈트 효과, 과인챈트 위험, 에로드 보호
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.30.0 R18-4] 인챈트 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnchantResult {
    /// 성공 (+1)
    Success { new_enchant: i32 },
    /// 과인챈트 파괴 (원본: +6 이상에서 확률 증가)
    Destroyed,
    /// 이미 최대 (변화 없음)
    AlreadyMax,
    /// 저주 제거 (저주된 갑옷에 축복 스크롤)
    CurseRemoved,
}

/// [v2.30.0 R18-4] 갑옷 인챈트 (원본: seffects SCR_ENCHANT_ARMOR)
pub fn enchant_armor(
    current_enchant: i32,
    is_cursed_scroll: bool,
    is_blessed_scroll: bool,
    item_is_cursed: bool,
    rng: &mut NetHackRng,
) -> EnchantResult {
    // 저주 스크롤 → 인챈트 감소
    if is_cursed_scroll {
        return EnchantResult::Success {
            new_enchant: current_enchant - 1,
        };
    }

    // 축복 스크롤 + 저주 아이템 → 저주 제거
    if is_blessed_scroll && item_is_cursed {
        return EnchantResult::CurseRemoved;
    }

    // 과인챈트 검사 (원본: +6 이상에서 확률적 파괴)
    if current_enchant >= 5 {
        let destroy_chance = (current_enchant - 4) * 15; // +5→15%, +6→30%, +7→45%
        if rng.rn2(100) < destroy_chance {
            return EnchantResult::Destroyed;
        }
    }

    // 최대 +7
    if current_enchant >= 7 {
        return EnchantResult::AlreadyMax;
    }

    let bonus = if is_blessed_scroll { 1 + rng.rn2(2) } else { 1 };
    EnchantResult::Success {
        new_enchant: (current_enchant + bonus).min(7),
    }
}

/// [v2.30.0 R18-4] 에로드 보호 레벨
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErodeProof {
    None,
    Partial,
    Full, // grease 또는 fooproof
}

/// [v2.30.0 R18-4] 에로드 적용
pub fn apply_erosion(current_erosion: i32, max_erosion: i32, protection: ErodeProof) -> i32 {
    match protection {
        ErodeProof::Full => current_erosion, // 보호
        ErodeProof::Partial => (current_erosion + 1).min(max_erosion / 2),
        ErodeProof::None => (current_erosion + 1).min(max_erosion),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enchant_normal() {
        let mut rng = NetHackRng::new(42);
        let result = enchant_armor(3, false, false, false, &mut rng);
        assert!(matches!(result, EnchantResult::Success { new_enchant: 4 }));
    }

    #[test]
    fn test_enchant_cursed_scroll() {
        let mut rng = NetHackRng::new(42);
        let result = enchant_armor(3, true, false, false, &mut rng);
        assert!(matches!(result, EnchantResult::Success { new_enchant: 2 }));
    }

    #[test]
    fn test_enchant_remove_curse() {
        let mut rng = NetHackRng::new(42);
        let result = enchant_armor(0, false, true, true, &mut rng);
        assert_eq!(result, EnchantResult::CurseRemoved);
    }

    #[test]
    fn test_enchant_max() {
        let mut rng = NetHackRng::new(42);
        let result = enchant_armor(7, false, false, false, &mut rng);
        assert_eq!(result, EnchantResult::AlreadyMax);
    }

    #[test]
    fn test_erosion_protected() {
        assert_eq!(apply_erosion(0, 3, ErodeProof::Full), 0);
        assert_eq!(apply_erosion(0, 3, ErodeProof::None), 1);
    }
}

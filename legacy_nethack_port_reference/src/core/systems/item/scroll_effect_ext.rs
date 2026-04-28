// ============================================================================
// [v2.40.0 R28-1] 스크롤 효과 (scroll_effect_ext.rs)
// 원본: NetHack 3.6.7 read.c 스크롤 확장
// 개별 스크롤 효과, BUC, 혼란 변이
// ============================================================================

/// [v2.40.0 R28-1] 스크롤 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrollEffect {
    Identify(i32),      // 감정 수량
    Enchant(String),    // 대상 ("weapon"/"armor")
    RemoveCurse(bool),  // true=전체, false=장비만
    Teleport(bool),     // true=제어, false=랜덤
    CreateMonster(i32), // 소환 수
    Genocide(bool),     // true=종족, false=클래스
    Fire(i32),          // 데미지
    MagicMapping,
    Punishment,
    Destroy(String), // 특정 유형 파괴
    Blank,           // 공백 스크롤
    Confuse,         // 혼란 부여 (다음 공격에)
    Nothing,
}

pub fn read_scroll(name: &str, blessed: bool, cursed: bool, confused: bool) -> ScrollEffect {
    if confused {
        return match name {
            "identify" => ScrollEffect::Nothing,
            "enchant weapon" | "enchant armor" => ScrollEffect::Destroy("equipped".into()),
            "teleportation" => ScrollEffect::Teleport(false),
            _ => ScrollEffect::Confuse,
        };
    }
    match name {
        "identify" => ScrollEffect::Identify(if blessed {
            99
        } else if cursed {
            1
        } else {
            3
        }),
        "enchant weapon" => ScrollEffect::Enchant("weapon".into()),
        "enchant armor" => ScrollEffect::Enchant("armor".into()),
        "remove curse" => ScrollEffect::RemoveCurse(blessed),
        "teleportation" => ScrollEffect::Teleport(!cursed),
        "create monster" => ScrollEffect::CreateMonster(if cursed { 5 } else { 1 }),
        "genocide" => ScrollEffect::Genocide(!cursed),
        "fire" => ScrollEffect::Fire(if blessed { 0 } else { 15 }),
        "magic mapping" => ScrollEffect::MagicMapping,
        "punishment" => ScrollEffect::Punishment,
        "blank paper" => ScrollEffect::Blank,
        _ => ScrollEffect::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify() {
        assert_eq!(
            read_scroll("identify", true, false, false),
            ScrollEffect::Identify(99)
        );
        assert_eq!(
            read_scroll("identify", false, false, false),
            ScrollEffect::Identify(3)
        );
    }

    #[test]
    fn test_remove_curse() {
        assert_eq!(
            read_scroll("remove curse", true, false, false),
            ScrollEffect::RemoveCurse(true)
        );
    }

    #[test]
    fn test_confused() {
        assert_eq!(
            read_scroll("identify", false, false, true),
            ScrollEffect::Nothing
        );
    }

    #[test]
    fn test_fire_blessed() {
        assert_eq!(
            read_scroll("fire", true, false, false),
            ScrollEffect::Fire(0)
        );
    }

    #[test]
    fn test_genocide_cursed() {
        assert_eq!(
            read_scroll("genocide", false, true, false),
            ScrollEffect::Genocide(false)
        );
    }
}

// ============================================================================
// [v2.39.0 R27-2] 원소 공격/방어 (elemental_ext.rs)
// 원본: NetHack 3.6.7 uhitm.c/mhitu.c 원소 확장
// 화염/냉기/전기/독/산 데미지 계산
// ============================================================================

/// [v2.39.0 R27-2] 원소 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Fire,
    Cold,
    Shock,
    Poison,
    Acid,
    Drain,
    Magic,
}

/// [v2.39.0 R27-2] 원소 데미지 계산
pub fn elemental_damage(element: Element, base: i32, resisted: bool, double_vuln: bool) -> i32 {
    let modified = if resisted {
        0
    } else if double_vuln {
        base * 2
    } else {
        base
    };
    modified.max(0)
}

/// [v2.39.0 R27-2] 상태이상 부여 확률
pub fn status_from_element(element: Element) -> Option<(&'static str, i32)> {
    match element {
        Element::Fire => Some(("burning", 3)),
        Element::Cold => Some(("frozen", 2)),
        Element::Shock => Some(("stunned", 1)),
        Element::Poison => Some(("poisoned", 5)),
        Element::Acid => Some(("corroded", 4)),
        Element::Drain => Some(("drained", 1)),
        Element::Magic => None,
    }
}

/// [v2.39.0 R27-2] 장비 부식 판정
pub fn equipment_erode(element: Element, erodeproof: bool) -> bool {
    if erodeproof {
        return false;
    }
    matches!(element, Element::Fire | Element::Acid | Element::Cold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_damage() {
        assert_eq!(elemental_damage(Element::Fire, 10, false, false), 10);
    }

    #[test]
    fn test_resisted() {
        assert_eq!(elemental_damage(Element::Fire, 10, true, false), 0);
    }

    #[test]
    fn test_vulnerable() {
        assert_eq!(elemental_damage(Element::Cold, 10, false, true), 20);
    }

    #[test]
    fn test_status() {
        assert_eq!(status_from_element(Element::Poison), Some(("poisoned", 5)));
        assert_eq!(status_from_element(Element::Magic), None);
    }

    #[test]
    fn test_erode() {
        assert!(equipment_erode(Element::Acid, false));
        assert!(!equipment_erode(Element::Acid, true));
    }
}

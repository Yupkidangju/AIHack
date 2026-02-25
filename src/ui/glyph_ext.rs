// ============================================================================
// [v2.43.0 R31-1] 색상/글리프 매핑 (glyph_ext.rs)
// 원본: NetHack 3.6.7 mapglyph.c 확장
// 글리프→색상, 밝기, 바닥/몬스터/아이템 레이어
// ============================================================================

/// [v2.43.0 R31-1] 렌더 레이어
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GlyphLayer {
    Floor,
    Trap,
    Item,
    Monster,
    Player,
    Effect,
}

/// [v2.43.0 R31-1] 색상
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Brown,
    Blue,
    Magenta,
    Cyan,
    Gray,
    BrightRed,
    BrightGreen,
    Yellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    White,
    NoColor,
}

/// [v2.43.0 R31-1] 몬스터 클래스 색상
pub fn monster_color(mclass: char) -> Color {
    match mclass {
        'D' => Color::Red,           // 드래곤
        'V' => Color::BrightRed,     // 뱀파이어
        'L' => Color::BrightMagenta, // 리치
        'Z' => Color::Gray,          // 좀비
        'A' => Color::Yellow,        // 천사
        '@' => Color::White,         // 인간
        _ => Color::NoColor,
    }
}

/// [v2.43.0 R31-1] 아이템 클래스 색상
pub fn item_color(oclass: char) -> Color {
    match oclass {
        ')' => Color::Cyan,        // 무기
        '[' => Color::Brown,       // 갑옷
        '!' => Color::BrightBlue,  // 포션
        '?' => Color::White,       // 스크롤
        '/' => Color::BrightCyan,  // 완드
        '$' => Color::Yellow,      // 금화
        '*' => Color::BrightGreen, // 보석
        _ => Color::NoColor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_order() {
        assert!(GlyphLayer::Floor < GlyphLayer::Monster);
        assert!(GlyphLayer::Monster < GlyphLayer::Player);
    }

    #[test]
    fn test_monster_color() {
        assert_eq!(monster_color('D'), Color::Red);
        assert_eq!(monster_color('@'), Color::White);
    }

    #[test]
    fn test_item_color() {
        assert_eq!(item_color('$'), Color::Yellow);
    }

    #[test]
    fn test_unknown() {
        assert_eq!(monster_color('~'), Color::NoColor);
    }
}

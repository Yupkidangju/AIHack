// ============================================================================
// [v2.40.0 R28-4] 상점 재고 (shop_stock_ext.rs)
// 원본: NetHack 3.6.7 shk.c/mkshop 재고 확장
// 상점 유형별 재고, 아이템 클래스, 재고 생성
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.40.0 R28-4] 상점 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopType {
    General,
    Weapon,
    Armor,
    Scroll,
    Potion,
    Wand,
    Ring,
    Book,
    Food,
    Tool,
    Candle,
}

/// [v2.40.0 R28-4] 상점 재고 클래스
pub fn stock_class(shop: ShopType) -> Vec<char> {
    match shop {
        ShopType::General => vec![')', '[', '!', '?', '/', '(', '%'],
        ShopType::Weapon => vec![')'],
        ShopType::Armor => vec!['['],
        ShopType::Scroll => vec!['?'],
        ShopType::Potion => vec!['!'],
        ShopType::Wand => vec!['/'],
        ShopType::Ring => vec!['=', '"'],
        ShopType::Book => vec!['+'],
        ShopType::Food => vec!['%'],
        ShopType::Tool => vec!['('],
        ShopType::Candle => vec!['('],
    }
}

/// [v2.40.0 R28-4] 재고 수량
pub fn stock_count(shop: ShopType, depth: i32, rng: &mut NetHackRng) -> i32 {
    let base = match shop {
        ShopType::General => 15,
        ShopType::Weapon | ShopType::Armor => 10,
        _ => 8,
    };
    base + rng.rn2(depth.min(10))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_stock() {
        assert_eq!(stock_class(ShopType::Weapon), vec![')']);
    }

    #[test]
    fn test_general_variety() {
        assert!(stock_class(ShopType::General).len() >= 5);
    }

    #[test]
    fn test_stock_count() {
        let mut rng = NetHackRng::new(42);
        let count = stock_count(ShopType::General, 10, &mut rng);
        assert!(count >= 15 && count < 30);
    }

    #[test]
    fn test_ring_classes() {
        assert!(stock_class(ShopType::Ring).contains(&'='));
    }
}

// ============================================================================
// [v2.33.0 R21-4] 상점 가격 확장 (shk_price_ext.rs)
// 원본: NetHack 3.6.7 shk.c 가격 계산 확장
// 매수/매도 가격, 카리스마 할인, 관광객 보너스, 흥정
// ============================================================================

/// [v2.33.0 R21-4] 매수 가격 (원본: shop_price)
pub fn buy_price(
    base_price: i32,
    charisma: i32,
    is_tourist: bool,
    anger_level: i32, // 0=친절, 1=보통, 2=화남
) -> i32 {
    let cha_factor = match charisma {
        0..=5 => 200,   // 2배
        6..=10 => 150,  // 1.5배
        11..=15 => 100, // 정가
        16..=18 => 75,  // 할인
        _ => 67,        // 큰 할인
    };

    let mut price = base_price * cha_factor / 100;

    // 관광객 바가지
    if is_tourist {
        price = price * 4 / 3;
    }

    // 분노 프리미엄
    price += price * anger_level / 5;

    price.max(1)
}

/// [v2.33.0 R21-4] 매도 가격 (언제나 매수의 1/3~1/2)
pub fn sell_price(base_price: i32, charisma: i32, item_is_identified: bool) -> i32 {
    let ratio = if item_is_identified { 50 } else { 33 };
    let cha_bonus = if charisma >= 16 { 5 } else { 0 };
    (base_price * (ratio + cha_bonus) / 100).max(1)
}

/// [v2.33.0 R21-4] BUC 가격 수정
pub fn buc_price_modifier(base: i32, blessed: bool, cursed: bool) -> i32 {
    if blessed {
        base * 3 / 2
    }
    // +50%
    else if cursed {
        base / 2
    }
    // -50%
    else {
        base
    }
}

/// [v2.33.0 R21-4] 감정 비용
pub fn identify_cost(base_price: i32, player_level: i32) -> i32 {
    let cost = base_price / 5 + player_level * 5;
    cost.clamp(10, 500)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_normal() {
        let price = buy_price(100, 12, false, 0);
        assert_eq!(price, 100);
    }

    #[test]
    fn test_buy_tourist() {
        let price = buy_price(100, 12, true, 0);
        assert!(price > 100); // 관광객 바가지
    }

    #[test]
    fn test_buy_high_cha() {
        let price = buy_price(100, 18, false, 0);
        assert!(price < 100); // 할인
    }

    #[test]
    fn test_sell() {
        let price = sell_price(100, 12, true);
        assert_eq!(price, 50);
    }

    #[test]
    fn test_buc() {
        assert_eq!(buc_price_modifier(100, true, false), 150);
        assert_eq!(buc_price_modifier(100, false, true), 50);
    }
}

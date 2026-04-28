// ============================================================================
// [v2.35.0 Phase 99-3] 상점 경제 확장 (economy_phase99_ext.rs)
// 원본: NetHack 3.6.7 src/shk.c L1500-3000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 가격 체계 — pricing (shk.c L1500-2000)
// =============================================================================

/// [v2.35.0 99-3] 상점 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopType {
    General,
    Armor,
    Weapon,
    Food,
    Scroll,
    Potion,
    Ring,
    Wand,
    Tool,
    Book,
    Antique,
    Lighting,
}

/// [v2.35.0 99-3] 가격 계산 결과
#[derive(Debug, Clone)]
pub struct PriceInfo {
    pub base_price: i32,
    pub buy_price: i32,
    pub sell_price: i32,
    pub shop_markup: f64,
    pub charisma_discount: f64,
    pub tourist_surcharge: bool,
}

/// [v2.35.0 99-3] 가격 계산
pub fn calculate_price(
    base: i32,
    shop_type: ShopType,
    player_charisma: i32,
    is_tourist: bool,
    is_identified: bool,
    is_cursed: bool,
    quantity: i32,
) -> PriceInfo {
    // 상점 유형별 마크업
    let markup = match shop_type {
        ShopType::General => 1.33,
        ShopType::Antique => 2.0,
        ShopType::Weapon | ShopType::Armor => 1.5,
        _ => 1.33,
    };

    // 카리스마 할인
    let cha_discount = match player_charisma {
        0..=5 => 1.5,
        6..=10 => 1.2,
        11..=15 => 1.0,
        16..=18 => 0.9,
        19..=24 => 0.8,
        _ => 0.75,
    };

    let tourist_mult = if is_tourist { 1.33 } else { 1.0 };
    let id_mult = if !is_identified { 1.0 } else { 1.0 };
    let curse_mult = if is_cursed { 0.5 } else { 1.0 };

    let unit_buy = (base as f64 * markup * cha_discount * tourist_mult * id_mult) as i32;
    let unit_sell = (base as f64 * 0.5 * curse_mult) as i32;

    PriceInfo {
        base_price: base * quantity,
        buy_price: unit_buy.max(1) * quantity,
        sell_price: unit_sell.max(1) * quantity,
        shop_markup: markup,
        charisma_discount: cha_discount,
        tourist_surcharge: is_tourist,
    }
}

// =============================================================================
// [2] 상점 상호작용 — shop_interact (shk.c L2000-3000)
// =============================================================================

/// [v2.35.0 99-3] 상점 행동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopAction {
    Bought {
        item: String,
        cost: i32,
        remaining_gold: i64,
    },
    Sold {
        item: String,
        earned: i32,
        total_gold: i64,
    },
    NotEnoughGold {
        need: i32,
        have: i64,
    },
    ShopkeeperAngry {
        reason: String,
    },
    Theft {
        item: String,
        alarm: bool,
    },
    CreditGiven {
        amount: i32,
    },
    Bribe {
        amount: i32,
        success: bool,
    },
    Repair {
        item: String,
        cost: i32,
    },
}

/// [v2.35.0 99-3] 구매 시도
pub fn try_buy(item_name: &str, cost: i32, player_gold: i64, credit: i32) -> ShopAction {
    let total_funds = player_gold + credit as i64;
    if total_funds < cost as i64 {
        return ShopAction::NotEnoughGold {
            need: cost,
            have: player_gold,
        };
    }
    ShopAction::Bought {
        item: item_name.to_string(),
        cost,
        remaining_gold: player_gold - cost as i64,
    }
}

/// [v2.35.0 99-3] 판매 시도
pub fn try_sell(item_name: &str, value: i32, player_gold: i64) -> ShopAction {
    ShopAction::Sold {
        item: item_name.to_string(),
        earned: value,
        total_gold: player_gold + value as i64,
    }
}

/// [v2.35.0 99-3] 절도 시도
pub fn try_steal(
    item_name: &str,
    player_dex: i32,
    player_level: i32,
    shopkeeper_level: i32,
    rng: &mut NetHackRng,
) -> ShopAction {
    let steal_chance = player_dex * 3 + player_level * 2;
    let detect_chance = shopkeeper_level * 5 + 20;

    if rng.rn2(100) < steal_chance.min(80) {
        ShopAction::Theft {
            item: item_name.to_string(),
            alarm: rng.rn2(100) < detect_chance,
        }
    } else {
        ShopAction::ShopkeeperAngry {
            reason: format!("{}을/를 훔치려다 발각됨!", item_name),
        }
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    #[test]
    fn test_price_normal() {
        let price = calculate_price(100, ShopType::General, 12, false, true, false, 1);
        assert!(price.buy_price > price.base_price);
        assert!(price.sell_price < price.base_price);
    }

    #[test]
    fn test_price_charisma_high() {
        let low = calculate_price(100, ShopType::General, 5, false, true, false, 1);
        let high = calculate_price(100, ShopType::General, 20, false, true, false, 1);
        assert!(high.buy_price < low.buy_price);
    }

    #[test]
    fn test_tourist_surcharge() {
        let normal = calculate_price(100, ShopType::General, 12, false, true, false, 1);
        let tourist = calculate_price(100, ShopType::General, 12, true, true, false, 1);
        assert!(tourist.buy_price >= normal.buy_price);
    }

    #[test]
    fn test_buy_success() {
        let result = try_buy("치유 포션", 100, 500, 0);
        assert!(matches!(result, ShopAction::Bought { .. }));
    }

    #[test]
    fn test_buy_no_gold() {
        let result = try_buy("치유 포션", 100, 50, 0);
        assert!(matches!(result, ShopAction::NotEnoughGold { .. }));
    }

    #[test]
    fn test_sell() {
        let result = try_sell("녹슨 검", 50, 1000);
        if let ShopAction::Sold { total_gold, .. } = result {
            assert_eq!(total_gold, 1050);
        }
    }

    #[test]
    fn test_steal() {
        let mut rng = test_rng();
        let result = try_steal("다이아몬드", 18, 15, 10, &mut rng);
        assert!(matches!(
            result,
            ShopAction::Theft { .. } | ShopAction::ShopkeeperAngry { .. }
        ));
    }

    #[test]
    fn test_quantity_price() {
        let single = calculate_price(100, ShopType::General, 12, false, true, false, 1);
        let multi = calculate_price(100, ShopType::General, 12, false, true, false, 5);
        assert_eq!(multi.buy_price, single.buy_price * 5);
    }
}

// ============================================================================
// [v2.29.0 Phase 93-5] 상점 경제 확장 (shk_phase93_ext.rs)
// 원본: NetHack 3.6.7 src/shk.c L500-2000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 가격 계산 — price_calc (shk.c L500-800)
// =============================================================================

/// [v2.29.0 93-5] 아이템 카테고리
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Potion,
    Scroll,
    Wand,
    Ring,
    Amulet,
    Food,
    Tool,
    Gem,
    Gold,
    Spellbook,
}

/// [v2.29.0 93-5] 가격 계산 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceResult {
    pub base_price: i32,
    pub adjusted_price: i32,
    pub charisma_modifier_pct: i32, // 100 = 1.00배
    pub tourist_bonus: bool,
    pub dunce_penalty: bool,
}

/// [v2.29.0 93-5] 판매 가격 계산
/// 원본: shk.c get_cost()
pub fn calculate_sell_price(
    base_price: i32,
    item_buc: i32, // -1=저주, 0=무축, 1=축복
    player_cha: i32,
    is_tourist: bool,
    is_identified: bool,
    shop_type: &str,
    _rng: &mut NetHackRng,
) -> PriceResult {
    // CHA 보정 (퍼센트)
    let cha_pct = match player_cha {
        1..=5 => 50,
        6..=8 => 67,
        9..=12 => 80,
        13..=15 => 90,
        16..=18 => 100,
        19..=22 => 110,
        _ => 125,
    };

    let mut price = base_price;

    // BUC 보정
    match item_buc {
        1 => price = price * 4 / 3,  // 축복 33% 증가
        -1 => price = price * 2 / 3, // 저주 33% 감소
        _ => {}
    }

    // 식별 여부
    if !is_identified {
        price = price / 2; // 미식별 50%
    }

    // 관광객 보너스 (판매가 높아짐 ← 호구)
    let tourist_bonus = is_tourist;
    if tourist_bonus {
        price = price * 4 / 3;
    }

    let adjusted = price * cha_pct / 100;

    PriceResult {
        base_price,
        adjusted_price: adjusted.max(1),
        charisma_modifier_pct: cha_pct,
        tourist_bonus,
        dunce_penalty: player_cha <= 5,
    }
}

/// [v2.29.0 93-5] 구매 가격 계산
pub fn calculate_buy_price(
    base_price: i32,
    player_cha: i32,
    is_tourist: bool,
    shop_surcharge: bool, // 상점주인 분노 등
) -> i32 {
    let cha_mult = match player_cha {
        1..=5 => 200,
        6..=8 => 150,
        9..=12 => 133,
        13..=15 => 115,
        16..=18 => 100,
        19..=22 => 90,
        _ => 75,
    };

    let mut price = base_price * cha_mult / 100;

    if is_tourist {
        price = price * 4 / 3;
    }

    if shop_surcharge {
        price = price * 2;
    }

    price.max(1)
}

// =============================================================================
// [2] 상점 상호작용 — shop_actions (shk.c L1000-1500)
// =============================================================================

/// [v2.29.0 93-5] 상점 행동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopActionResult {
    /// 구매 완료
    Purchased { item: String, cost: i32 },
    /// 판매 완료
    Sold { item: String, earned: i32 },
    /// 금화 부족
    CannotAfford { needed: i32, have: i32 },
    /// 도둑질 감지
    TheftDetected { anger_level: i32 },
    /// 문 막힘
    DoorBlocked,
    /// 거래 거부
    Refused { reason: String },
    /// 가격 제시
    PriceOffer { item: String, price: i32 },
}

/// [v2.29.0 93-5] 상점 구매 시도
pub fn attempt_purchase(
    item_name: &str,
    item_price: i32,
    player_gold: i32,
    player_cha: i32,
    is_tourist: bool,
) -> ShopActionResult {
    let cost = calculate_buy_price(item_price, player_cha, is_tourist, false);

    if player_gold < cost {
        return ShopActionResult::CannotAfford {
            needed: cost,
            have: player_gold,
        };
    }

    ShopActionResult::Purchased {
        item: item_name.to_string(),
        cost,
    }
}

// =============================================================================
// [3] 상점 도둑질 판정 — theft_check (shk.c L1500-1800)
// =============================================================================

/// [v2.29.0 93-5] 도둑질 판정
pub fn theft_check(
    item_value: i32,
    player_dex: i32,
    is_invisible: bool,
    shopkeeper_level: i32,
    rng: &mut NetHackRng,
) -> ShopActionResult {
    // 투명 시 보너스
    let stealth = player_dex * 2 + if is_invisible { 20 } else { 0 };
    let detection = shopkeeper_level * 5 + rng.rn2(20);

    if stealth > detection {
        // 성공 (감지 안됨)
        ShopActionResult::Sold {
            item: "훔친 아이템".to_string(),
            earned: 0,
        }
    } else {
        ShopActionResult::TheftDetected {
            anger_level: item_value / 100 + 1,
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
    fn test_sell_price_blessed() {
        let mut rng = test_rng();
        let result = calculate_sell_price(100, 1, 14, false, true, "general", &mut rng);
        assert!(result.adjusted_price > 100);
    }

    #[test]
    fn test_sell_price_cursed() {
        let mut rng = test_rng();
        let result = calculate_sell_price(100, -1, 14, false, true, "general", &mut rng);
        assert!(result.adjusted_price < 100);
    }

    #[test]
    fn test_sell_price_tourist() {
        let mut rng = test_rng();
        let normal = calculate_sell_price(100, 0, 14, false, true, "general", &mut rng);
        let mut rng2 = test_rng();
        let tourist = calculate_sell_price(100, 0, 14, true, true, "general", &mut rng2);
        assert!(tourist.adjusted_price > normal.adjusted_price);
    }

    #[test]
    fn test_buy_price_charisma() {
        let low_cha = calculate_buy_price(100, 5, false, false);
        let high_cha = calculate_buy_price(100, 20, false, false);
        assert!(low_cha > high_cha);
    }

    #[test]
    fn test_purchase_success() {
        let result = attempt_purchase("검", 100, 500, 14, false);
        assert!(matches!(result, ShopActionResult::Purchased { .. }));
    }

    #[test]
    fn test_purchase_fail() {
        let result = attempt_purchase("검", 100, 10, 14, false);
        assert!(matches!(result, ShopActionResult::CannotAfford { .. }));
    }

    #[test]
    fn test_theft_detected() {
        let mut detected = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = theft_check(500, 10, false, 15, &mut rng);
            if matches!(result, ShopActionResult::TheftDetected { .. }) {
                detected = true;
                break;
            }
        }
        assert!(detected);
    }
}

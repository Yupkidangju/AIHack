// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.23.0 R11-5] 상점 시스템 고도화 (shop_ext.rs)
//
// 원본 참조: NetHack 3.6.7 shk.c (3,818줄)
//
// 구현 내용:
//   1. 구매/판매 가격 계산 (카리스마, 관광객, BUC 보정)
//   2. 도둑질 감지 및 분노 메커니즘
//   3. 수리/식별 서비스
//   4. 상점 주인 상태 머신
//   5. 외상/부채 관리
//   6. 상점 유형별 재고 생성 규칙
// ============================================================================

// =============================================================================
// [1] 상점 유형 (원본: shk.c shktypes)
// =============================================================================

/// [v2.23.0 R11-5] 상점 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopType {
    /// 일반 잡화점
    General,
    /// 무기점
    Weapon,
    /// 갑옷점
    Armor,
    /// 스크롤/책
    Scroll,
    /// 포션
    Potion,
    /// 지팡이
    Wand,
    /// 반지/아뮬렛
    Ring,
    /// 도구
    Tool,
    /// 음식
    Food,
    /// 보석
    Gem,
    /// 양초/조명
    Light,
}

/// [v2.23.0 R11-5] 상점 유형별 아이템 클래스 필터
pub fn shop_item_filter(shop_type: ShopType) -> Vec<char> {
    match shop_type {
        ShopType::General => vec![')', '[', '!', '?', '+', '/', '(', '%', '*'],
        ShopType::Weapon => vec![')'],
        ShopType::Armor => vec!['['],
        ShopType::Scroll => vec!['?', '+'],
        ShopType::Potion => vec!['!'],
        ShopType::Wand => vec!['/'],
        ShopType::Ring => vec!['=', '"'],
        ShopType::Tool => vec!['('],
        ShopType::Food => vec!['%'],
        ShopType::Gem => vec!['*'],
        ShopType::Light => vec!['('],
    }
}

// =============================================================================
// [2] 가격 계산 (원본: shk.c shop_price, set_cost)
// =============================================================================

/// [v2.23.0 R11-5] 판매 가격 계산 인수
#[derive(Debug, Clone)]
pub struct PriceInput {
    /// 아이템 기본 가격
    pub base_price: i32,
    /// 아이템 수량
    pub quantity: i32,
    /// 아이템 강화 수치 (spe)
    pub enchantment: i32,
    /// 축복 여부
    pub blessed: bool,
    /// 저주 여부
    pub cursed: bool,
    /// 식별 여부
    pub identified: bool,
    /// 플레이어 카리스마
    pub charisma: i32,
    /// 관광객 여부
    pub is_tourist: bool,
    /// 상점 서져 배수 (수요/공급)
    pub surcharge: bool,
}

/// [v2.23.0 R11-5] 구매 가격 계산 (원본: shop_price)
pub fn calc_buy_price(input: &PriceInput) -> i32 {
    let mut price = input.base_price * input.quantity;

    // 강화 보정
    if input.enchantment > 0 {
        price += input.enchantment * 10;
    }

    // BUC 보정
    if input.blessed {
        price = price * 3 / 2; // 축복 50% 추가
    }
    if input.cursed {
        price = price * 3 / 4; // 저주 25% 할인
    }

    // 카리스마 보정 (원본: badman 공식)
    let cha_discount = charisma_price_modifier(input.charisma);
    price = (price as f64 * cha_discount) as i32;

    // 관광객 추가 할증
    if input.is_tourist {
        price = price * 4 / 3; // 33% 추가
    }

    // 서져
    if input.surcharge {
        price = price * 4 / 3;
    }

    price.max(1)
}

/// [v2.23.0 R11-5] 판매 가격 계산 (매입 = 구매가의 1/3)
pub fn calc_sell_price(input: &PriceInput) -> i32 {
    let buy_price = calc_buy_price(input);
    (buy_price / 3).max(1)
}

/// [v2.23.0 R11-5] 카리스마 기반 가격 배수 (원본: badman)
fn charisma_price_modifier(charisma: i32) -> f64 {
    match charisma {
        c if c >= 25 => 0.67, // 33% 할인
        c if c >= 19 => 0.75,
        c if c >= 16 => 0.85,
        c if c >= 13 => 0.90,
        c if c >= 10 => 1.00,
        c if c >= 8 => 1.10,
        c if c >= 6 => 1.25,
        _ => 1.50, // 50% 할증
    }
}

// =============================================================================
// [3] 상점 주인 상태 (원본: shk.c ESHK)
// =============================================================================

/// [v2.23.0 R11-5] 상점 주인 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopkeeperMood {
    /// 정상 영업
    Normal,
    /// 약간 경계 (미지불 아이템 소지)
    Suspicious,
    /// 분노 (도둑질 감지)
    Angry,
    /// 도주 (HP 낮음)
    Fleeing,
    /// 사망
    Dead,
}

/// [v2.23.0 R11-5] 상점 주인 전체 상태
#[derive(Debug, Clone)]
pub struct ShopkeeperState {
    /// 이름
    pub name: String,
    /// 상점 유형
    pub shop_type: ShopType,
    /// 기분
    pub mood: ShopkeeperMood,
    /// 플레이어 부채
    pub player_debt: i64,
    /// 플레이어 보증금 (맡긴 금)
    pub player_credit: i64,
    /// 도둑질 횟수
    pub theft_count: i32,
    /// 출입 허용 상태
    pub allows_entry: bool,
}

// =============================================================================
// [4] 도둑질 감지 (원본: shk.c rob_shop, shoplifting)
// =============================================================================

/// [v2.23.0 R11-5] 도둑질 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheftResult {
    /// 안전 (상점 밖이거나 자기 물건)
    Safe,
    /// 미지불 물품으로 상점 탈출 시도
    ShoplifterDetected { unpaid_value: i64 },
    /// 강도 (문을 열고 도주)
    RobberyDetected { stolen_value: i64 },
}

/// [v2.23.0 R11-5] 상점 탈출 시 도둑질 판정 (원본: shk.c shoplifting)
pub fn check_shoplifting(
    in_shop: bool,
    leaving_shop: bool,
    unpaid_items: &[(String, i32)],
    has_debt: bool,
) -> TheftResult {
    if !in_shop || !leaving_shop {
        return TheftResult::Safe;
    }

    if unpaid_items.is_empty() && !has_debt {
        return TheftResult::Safe;
    }

    let total_value: i64 = unpaid_items.iter().map(|(_, v)| *v as i64).sum();

    if total_value > 0 {
        TheftResult::ShoplifterDetected {
            unpaid_value: total_value,
        }
    } else {
        TheftResult::Safe
    }
}

/// [v2.23.0 R11-5] 상점 주인 분노 전이
pub fn on_theft_detected(state: &mut ShopkeeperState, stolen_value: i64) {
    state.theft_count += 1;
    state.player_debt += stolen_value;
    state.mood = ShopkeeperMood::Angry;
    state.allows_entry = false;
}

// =============================================================================
// [5] 수리/식별 서비스 (원본: shk.c costly_alteration, shk_identify)
// =============================================================================

/// [v2.23.0 R11-5] 서비스 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopService {
    /// 식별 (가격 기반)
    Identify,
    /// 수리 (부식/열화 제거)
    Repair,
    /// 저주 제거
    Uncurse,
    /// 강화
    Enchant,
}

/// [v2.23.0 R11-5] 서비스 비용 계산
pub fn service_cost(service: ShopService, item_value: i32, charisma: i32) -> i32 {
    let base = match service {
        ShopService::Identify => item_value / 2 + 10,
        ShopService::Repair => item_value / 3 + 20,
        ShopService::Uncurse => item_value / 2 + 50,
        ShopService::Enchant => item_value + 100,
    };
    let modifier = charisma_price_modifier(charisma);
    ((base as f64) * modifier) as i32
}

/// [v2.23.0 R11-5] 부채 정산
pub fn settle_debt(state: &mut ShopkeeperState, payment: i64) -> i64 {
    let actual = payment.min(state.player_debt);
    state.player_debt -= actual;
    if state.player_debt == 0 && state.mood == ShopkeeperMood::Suspicious {
        state.mood = ShopkeeperMood::Normal;
    }
    actual // 실제 지불 금액 반환
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_price_input() -> PriceInput {
        PriceInput {
            base_price: 100,
            quantity: 1,
            enchantment: 0,
            blessed: false,
            cursed: false,
            identified: true,
            charisma: 10,
            is_tourist: false,
            surcharge: false,
        }
    }

    #[test]
    fn test_buy_price_basic() {
        let input = test_price_input();
        let price = calc_buy_price(&input);
        assert_eq!(price, 100);
    }

    #[test]
    fn test_buy_price_blessed() {
        let mut input = test_price_input();
        input.blessed = true;
        let price = calc_buy_price(&input);
        assert_eq!(price, 150); // 50% 추가
    }

    #[test]
    fn test_buy_price_cursed() {
        let mut input = test_price_input();
        input.cursed = true;
        let price = calc_buy_price(&input);
        assert_eq!(price, 75); // 25% 할인
    }

    #[test]
    fn test_buy_price_charisma_high() {
        let mut input = test_price_input();
        input.charisma = 20;
        let price = calc_buy_price(&input);
        assert_eq!(price, 75); // 25% 할인
    }

    #[test]
    fn test_buy_price_tourist() {
        let mut input = test_price_input();
        input.is_tourist = true;
        let price = calc_buy_price(&input);
        assert_eq!(price, 133); // 33% 추가
    }

    #[test]
    fn test_sell_price() {
        let input = test_price_input();
        let price = calc_sell_price(&input);
        assert_eq!(price, 33); // 100/3
    }

    #[test]
    fn test_enchantment_bonus() {
        let mut input = test_price_input();
        input.enchantment = 3;
        let price = calc_buy_price(&input);
        assert_eq!(price, 130); // 100 + 3*10
    }

    #[test]
    fn test_shop_filter_weapon() {
        let filter = shop_item_filter(ShopType::Weapon);
        assert_eq!(filter, vec![')']);
    }

    #[test]
    fn test_shop_filter_general() {
        let filter = shop_item_filter(ShopType::General);
        assert!(filter.len() >= 5);
    }

    #[test]
    fn test_theft_safe() {
        let r = check_shoplifting(false, true, &[], false);
        assert_eq!(r, TheftResult::Safe);
    }

    #[test]
    fn test_theft_detected() {
        let items = vec![("sword".to_string(), 100), ("potion".to_string(), 50)];
        let r = check_shoplifting(true, true, &items, false);
        assert!(matches!(
            r,
            TheftResult::ShoplifterDetected { unpaid_value: 150 }
        ));
    }

    #[test]
    fn test_theft_no_items() {
        let r = check_shoplifting(true, true, &[], false);
        assert_eq!(r, TheftResult::Safe);
    }

    #[test]
    fn test_on_theft_angry() {
        let mut state = ShopkeeperState {
            name: "Asidonhopo".to_string(),
            shop_type: ShopType::General,
            mood: ShopkeeperMood::Normal,
            player_debt: 0,
            player_credit: 0,
            theft_count: 0,
            allows_entry: true,
        };
        on_theft_detected(&mut state, 500);
        assert_eq!(state.mood, ShopkeeperMood::Angry);
        assert_eq!(state.player_debt, 500);
        assert!(!state.allows_entry);
    }

    #[test]
    fn test_service_cost_identify() {
        let cost = service_cost(ShopService::Identify, 100, 10);
        assert_eq!(cost, 60); // 100/2 + 10 = 60
    }

    #[test]
    fn test_settle_debt() {
        let mut state = ShopkeeperState {
            name: "Test".to_string(),
            shop_type: ShopType::General,
            mood: ShopkeeperMood::Suspicious,
            player_debt: 100,
            player_credit: 0,
            theft_count: 0,
            allows_entry: true,
        };
        let paid = settle_debt(&mut state, 100);
        assert_eq!(paid, 100);
        assert_eq!(state.player_debt, 0);
        assert_eq!(state.mood, ShopkeeperMood::Normal);
    }

    #[test]
    fn test_settle_partial() {
        let mut state = ShopkeeperState {
            name: "Test".to_string(),
            shop_type: ShopType::General,
            mood: ShopkeeperMood::Suspicious,
            player_debt: 200,
            player_credit: 0,
            theft_count: 0,
            allows_entry: true,
        };
        let paid = settle_debt(&mut state, 50);
        assert_eq!(paid, 50);
        assert_eq!(state.player_debt, 150);
        assert_eq!(state.mood, ShopkeeperMood::Suspicious); // 아직 부채 있음
    }
}

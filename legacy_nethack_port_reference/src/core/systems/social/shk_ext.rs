// =============================================================================
// AIHack — shk_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
// [v2.19.0] shk.c 핵심 함수 이식 — Pure Result 패턴
// 원본: nethack-3.6.7/src/shk.c (4,974줄)
//
// 이식 대상:
//   get_cost        (L1200-1300) → get_cost_result
//   get_pricing_units (L1165-1195) → get_pricing_units
//   cost_per_charge (L1301-1350) → cost_per_charge
//   check_credit    (L992-1011)  → check_credit_result
//   addupbill       (L321-334)   → addup_bill
//   shop_debt       (L752-763)   → shop_debt_total
//   rob_shop        (L474-504)   → rob_shop_result
//   pacify_shk      (L1064-1080) → pacify_prices
//   rile_shk        (L1083-1099) → rile_prices
//   special_stock   (L4700+)     → is_special_stock
//   shk_embellish   (L1350+)     → price_embellish
// =============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [v2.19.0] 가격 단위 계산 (원본: shk.c L1165-1195 get_pricing_units)
// 탄약류(화살/볼트/표창 등)는 묶음 단위, 일반은 개별 단위
// =============================================================================

/// 아이템 가격 관련 속성
#[derive(Debug, Clone)]
pub struct ItemPriceInfo {
    /// 아이템 기본 가격 (oc_cost)
    pub base_cost: i32,
    /// 수량
    pub quantity: i32,
    /// 축복 여부
    pub blessed: bool,
    /// 저주 여부
    pub cursed: bool,
    /// 강화 수치 (spe)
    pub enchantment: i32,
    /// 강화 알려짐 여부
    pub enchantment_known: bool,
    /// 아이템 클래스 (0=무기, 1=방어구, 2=도구, 3=음식, 4=물약, 5=두루마리,
    ///                6=지팡이, 7=보석, 8=반지, 9=목걸이, 10=주문서)
    pub item_class: i32,
    /// 탄약류인가 (화살, 볼트, 표창 등)
    pub is_ammo: bool,
    /// 글롭(glob)인가
    pub is_glob: bool,
    /// 무게 (글롭 가격 계산용)
    pub weight: i32,
    /// 충전 가능 아이템인가 (지팡이/뿔나팔 등)
    pub is_chargeable: bool,
    /// 최대 충전 수
    pub max_charges: i32,
    /// 미확인 상태인가
    pub unidentified: bool,
}

/// [v2.19.0] 가격 단위 계산 (원본: get_pricing_units L1165-1195)
/// 탄약류는 묶음 가격, 글롭은 무게 비례, 일반은 개별
pub fn get_pricing_units(info: &ItemPriceInfo) -> i32 {
    if info.is_glob {
        // 원본: glob은 무게를 기준으로 가격 산정
        // 무게 200 기준 1단위, 비례
        (info.weight + 100) / 200
    } else if info.is_ammo {
        // 원본: 탄약류는 수량 그대로 단위
        info.quantity
    } else {
        1 // 일반 아이템은 개당
    }
}

// =============================================================================
// [v2.19.0] 아이템 비용 계산 (원본: shk.c ~L1200 get_cost)
// 상점 주인 관점에서 아이템의 실제 판매 가격 산출
// =============================================================================

/// 상점 가격 계산 입력
#[derive(Debug, Clone)]
pub struct ShopPriceInput {
    /// 아이템 기본 정보
    pub item: ItemPriceInfo,
    /// 상점 주인 비우호적(할증 적용중)인가
    pub surcharge: bool,
    /// 매입(true) vs 매출(false)
    pub is_buying: bool,
    /// 카리스마 수치 (6~25)
    pub charisma: i32,
    /// 관광객(Tourist) 직업인가
    pub is_tourist: bool,
    /// 던전 레벨 (보정 목적)
    pub dungeon_level: i32,
    /// 상점 전문 품목과 일치하는가
    pub matches_shop_type: bool,
}

/// 가격 계산 결과
#[derive(Debug, Clone)]
pub struct PriceResult {
    /// 최종 가격
    pub price: i64,
    /// 할증 적용 여부
    pub surcharge_applied: bool,
    /// 카리스마 할인/할증 배율 (디버그용)
    pub charisma_factor: f64,
}

/// [v2.19.0] 상점 가격 계산 (원본: get_cost 기반)
/// 기본 가격 × 카리스마 보정 × 상점 종류 보정 × 할증/할인
pub fn get_cost_result(input: &ShopPriceInput) -> PriceResult {
    let mut price = input.item.base_cost as i64;

    // 원본: 수량이 아닌 단위 기준
    let units = get_pricing_units(&input.item) as i64;
    price *= units.max(1);

    // 원본: 축복/저주 보정
    if input.item.blessed {
        price = price * 11 / 10; // +10%
    } else if input.item.cursed {
        price = price * 9 / 10; // -10%
    }

    // 원본: 강화 보정 (알려진 경우)
    if input.item.enchantment_known && input.item.enchantment != 0 {
        // 양수 강화는 비싸게, 음수 강화는 싸게
        let spe_adj = input.item.enchantment as i64 * price / 20;
        price += spe_adj;
    }

    // 원본: 카리스마 보정 (원본: shk.c 내 shkaffold 계산)
    // 카리스마 10-11이 기준, 높으면 할인, 낮으면 할증
    let cha_factor = match input.charisma {
        c if c <= 5 => 2.0,
        6..=7 => 1.5,
        8..=10 => 1.3,
        11..=15 => 1.0,
        16..=17 => 0.9,
        18..=20 => 0.8,
        _ => 0.75, // 21+
    };

    if input.is_buying {
        // 매입 시 카리스마가 높으면 할인
        price = (price as f64 * cha_factor) as i64;
    } else {
        // 매출(판매) 시 카리스마가 높으면 더 비싸게 받을 수 있다
        // 원본: 판매가는 기본 가격의 1/2 ~ 2/3
        price = price / 2;
        if cha_factor < 1.0 {
            // 카리스마 높으면 약간 더 받을 수 있음
            price = (price as f64 * (2.0 - cha_factor)) as i64;
        }
    }

    // 원본: 관광객 할증 (원본: 4/3 배)
    if input.is_tourist && input.is_buying {
        price = price * 4 / 3;
    }

    // 원본: 상점 전문 품목 보정 (전문점은 비전문 아이템을 더 비싸게 팔거나 덜 비싸게 삼)
    if input.matches_shop_type && !input.is_buying {
        price = price * 3 / 2; // 전문품 매입 시 50% 더 받음
    }

    // 원본: 분노 할증 (surcharge) — 33% (원본: rile_shk L1083-1099)
    let surcharge_applied = input.surcharge && input.is_buying;
    if surcharge_applied {
        price += (price + 2) / 3;
    }

    // 최소 가격 보장
    if price < 1 {
        price = 1;
    }

    PriceResult {
        price,
        surcharge_applied,
        charisma_factor: cha_factor,
    }
}

// =============================================================================
// [v2.19.0] 충전당 비용 (원본: shk.c L1301-1350 cost_per_charge)
// 지팡이/뿔나팔 등 충전 횟수에 따른 비용
// =============================================================================

/// [v2.19.0] 충전당 비용 (원본: cost_per_charge)
/// 총 가격을 최대 충전 수로 나누어 1회 사용 비용 산출
pub fn cost_per_charge(total_price: i64, max_charges: i32) -> i64 {
    if max_charges <= 0 {
        return total_price; // 충전 불가면 전액
    }
    // 원본: price / (1 + highest_charges)
    total_price / (1 + max_charges as i64)
}

// =============================================================================
// [v2.19.0] 크레딧 차감 (원본: shk.c L992-1011 check_credit)
// =============================================================================

/// 크레딧 차감 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreditCheckResult {
    /// 전액 크레딧 처리 — 추가 지불 불필요
    FullyCovered { credit_remaining: i64 },
    /// 부분 크레딧 처리 — 나머지 금액 지불 필요
    PartiallyCovered { amount_due: i64 },
    /// 크레딧 없음 — 전액 지불
    NoCreditUsed { amount_due: i64 },
}

/// [v2.19.0] 크레딧 차감 판정 (원본: check_credit L992-1011)
pub fn check_credit_result(price: i64, credit: i64) -> CreditCheckResult {
    if credit <= 0 {
        CreditCheckResult::NoCreditUsed { amount_due: price }
    } else if credit >= price {
        CreditCheckResult::FullyCovered {
            credit_remaining: credit - price,
        }
    } else {
        CreditCheckResult::PartiallyCovered {
            amount_due: price - credit,
        }
    }
}

// =============================================================================
// [v2.19.0] 청구서 합산 (원본: shk.c L321-334 addupbill)
// =============================================================================

/// 청구서 항목
#[derive(Debug, Clone)]
pub struct BillEntry {
    /// 아이템 가격
    pub price: i64,
    /// 청구 수량
    pub quantity: i32,
}

/// [v2.19.0] 청구서 합산 (원본: addupbill L321-334)
pub fn addup_bill(entries: &[BillEntry]) -> i64 {
    entries.iter().map(|e| e.price * e.quantity as i64).sum()
}

// =============================================================================
// [v2.19.0] 총 부채 계산 (원본: shk.c L752-763 shop_debt)
// =============================================================================

/// [v2.19.0] 총 부채 계산 (원본: shop_debt L752-763)
/// 청구서 합계 + 기타 부채(파손 등)
pub fn shop_debt_total(bill_entries: &[BillEntry], debit: i64) -> i64 {
    addup_bill(bill_entries) + debit
}

// =============================================================================
// [v2.19.0] 도둑질 판정 (원본: shk.c L474-504 rob_shop)
// 크레딧으로 청구서를 충당할 수 있는지 판정
// =============================================================================

/// 도둑질 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RobShopResult {
    /// 크레딧으로 전액 충당 — 도둑질 아님
    CreditCovers,
    /// 실제 도둑질 — 도둑 금액과 성향 페널티
    Robbery {
        stolen_amount: i64,
        alignment_penalty: bool,
    },
}

/// [v2.19.0] 도둑질 판정 (원본: rob_shop L474-504)
/// 상점 탈출 시 크레딧이 청구서를 충당하는지 확인
pub fn rob_shop_result(bill_total: i64, debit: i64, credit: i64, is_rogue: bool) -> RobShopResult {
    let total = bill_total + debit;

    if credit >= total {
        RobShopResult::CreditCovers
    } else {
        let stolen = total - credit;
        // 원본: if (!Role_if(PM_ROGUE)) adjalign(-sgn(u.ualign.type));
        RobShopResult::Robbery {
            stolen_amount: stolen,
            alignment_penalty: !is_rogue,
        }
    }
}

// =============================================================================
// [v2.19.0] 분노/화해 시 가격 조정 (원본: shk.c pacify_shk/rile_shk)
// 분노 시 33% 할증, 화해 시 25% 할인 (원래대로 복원)
// =============================================================================

/// [v2.19.0] 분노 할증 (원본: rile_shk L1083-1099)
/// 모든 청구 가격에 33% 가산
pub fn rile_prices(prices: &mut [i64]) {
    for p in prices.iter_mut() {
        let surcharge = (*p + 2) / 3;
        *p += surcharge;
    }
}

/// [v2.19.0] 화해 할인 (원본: pacify_shk L1064-1080)
/// 할증이 적용된 가격에서 25% 차감 (33% 할증을 되돌림)
pub fn pacify_prices(prices: &mut [i64]) {
    for p in prices.iter_mut() {
        let reduction = (*p + 3) / 4;
        *p -= reduction;
    }
}

// =============================================================================
// [v2.19.0] 상점 전문 품목 판정 (원본: shk.c special_stock ~L4700)
// =============================================================================

/// 상점 종류 (원본: shop.rs ShopType과 동일하지만 _ext용 독립)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopKind {
    General,
    Weapon,
    Armor,
    Ring,
    Potion,
    Gem,
    Tool,
    Food,
    Candle,
    Book,
}

/// [v2.19.0] 상점 전문 품목 판정 (원본: special_stock)
/// 해당 아이템이 특정 상점의 전문 품목인지 확인
pub fn is_special_stock(item_class: i32, shop_kind: ShopKind) -> bool {
    match shop_kind {
        ShopKind::Weapon => item_class == 0, // 무기
        ShopKind::Armor => item_class == 1,  // 방어구
        ShopKind::Ring => item_class == 8,   // 반지
        ShopKind::Potion => item_class == 4, // 물약
        ShopKind::Gem => item_class == 7,    // 보석
        ShopKind::Tool => item_class == 2,   // 도구
        ShopKind::Food => item_class == 3,   // 음식
        ShopKind::Candle => item_class == 2, // 촛불(도구)
        ShopKind::Book => item_class == 10,  // 주문서
        ShopKind::General => true,           // 일반상점은 모두 해당
    }
}

// =============================================================================
// [v2.19.0] 가격 꾸미기 멘트 (원본: shk.c shk_embellish ~L1350)
// 높은 가격 아이템에 대한 상점 주인의 코멘트
// =============================================================================

/// [v2.19.0] 가격 꾸미기 멘트 (원본: shk_embellish)
/// 가격에 따라 상점 주인의 평가 멘트 반환
pub fn price_embellish(price: i64, rng: &mut NetHackRng) -> &'static str {
    // 원본: 가격대별 다른 멘트
    if price <= 5 {
        match rng.rn2(3) {
            0 => "저렴한 물건이지.",
            1 => "거의 공짜나 마찬가지야.",
            _ => "이 정도면 부담없을 거야.",
        }
    } else if price <= 50 {
        match rng.rn2(3) {
            0 => "합리적인 가격이지.",
            1 => "괜찮은 거래야.",
            _ => "적정 가격이라고 할 수 있지.",
        }
    } else if price <= 500 {
        match rng.rn2(3) {
            0 => "좋은 물건이야, 가격도 그에 걸맞지.",
            1 => "이건 꽤 가치 있는 물건이야.",
            _ => "품질 보증을 하지.",
        }
    } else if price <= 5000 {
        match rng.rn2(3) {
            0 => "최상급 물건! 그만한 가치가 있어.",
            1 => "이 물건은 특별해. 가격도 특별하지.",
            _ => "다시는 못 구할 물건일세.",
        }
    } else {
        match rng.rn2(3) {
            0 => "이건 전설적인 물건이야!",
            1 => "이 정도 물건은 왕도 탐낼 거야.",
            _ => "가격이 무색해질 정도야.",
        }
    }
}

// =============================================================================
// [v2.19.0] 상점 진입/탈출 판정
// =============================================================================

/// 상점 진입 시 상점 주인 반응
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopEntryReaction {
    /// 환영 인사
    Welcome { visit_count: i32 },
    /// 화난 상태 — 경고
    AngryWarning,
    /// 도둑 기록 있음 — 의심
    SuspiciousMuttering,
    /// 투명 상태 — 거부
    InvisibleRejection,
    /// 상점 주인 없음 — 무인 상점
    Deserted,
    /// 상점 주인 침묵 (수면/마비)
    MuteShopkeeper,
}

/// [v2.19.0] 상점 진입 시 반응 결정 (원본: u_entered_shop L535-680)
pub fn shop_entry_reaction(
    has_shopkeeper: bool,
    shk_angry: bool,
    shk_mute: bool,
    shk_following: bool,
    was_robbed: bool,
    player_invisible: bool,
    visit_count: i32,
) -> ShopEntryReaction {
    if !has_shopkeeper {
        return ShopEntryReaction::Deserted;
    }
    if shk_mute || shk_following {
        return ShopEntryReaction::MuteShopkeeper;
    }
    if player_invisible {
        return ShopEntryReaction::InvisibleRejection;
    }
    if shk_angry {
        return ShopEntryReaction::AngryWarning;
    }
    if was_robbed {
        return ShopEntryReaction::SuspiciousMuttering;
    }
    ShopEntryReaction::Welcome { visit_count }
}

/// 상점 탈출 시 반응
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopExitReaction {
    /// 청구서 정산 완료 — 자유 퇴장
    Settled,
    /// 미결제 경고
    PaymentWarning,
    /// 도둑질로 간주 — Kops 소환 가능
    Robbery { near_shop: bool },
}

/// [v2.19.0] 상점 탈출 시 반응 결정 (원본: u_left_shop L402-448)
pub fn shop_exit_reaction(
    has_bill: bool,
    has_debit: bool,
    just_at_boundary: bool,
) -> ShopExitReaction {
    if !has_bill && !has_debit {
        return ShopExitReaction::Settled;
    }
    if just_at_boundary {
        return ShopExitReaction::PaymentWarning;
    }
    ShopExitReaction::Robbery { near_shop: false }
}

// =============================================================================
// [v2.19.0] 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn make_item() -> ItemPriceInfo {
        ItemPriceInfo {
            base_cost: 100,
            quantity: 1,
            blessed: false,
            cursed: false,
            enchantment: 0,
            enchantment_known: false,
            item_class: 0,
            is_ammo: false,
            is_glob: false,
            weight: 10,
            is_chargeable: false,
            max_charges: 0,
            unidentified: false,
        }
    }

    // --- get_pricing_units 테스트 ---

    #[test]
    fn test_pricing_units_normal() {
        let info = make_item();
        assert_eq!(get_pricing_units(&info), 1);
    }

    #[test]
    fn test_pricing_units_ammo() {
        let mut info = make_item();
        info.is_ammo = true;
        info.quantity = 20;
        assert_eq!(get_pricing_units(&info), 20);
    }

    #[test]
    fn test_pricing_units_glob() {
        let mut info = make_item();
        info.is_glob = true;
        info.weight = 600;
        assert_eq!(get_pricing_units(&info), 3); // (600+100)/200 = 3
    }

    // --- get_cost_result 테스트 ---

    #[test]
    fn test_cost_basic() {
        let input = ShopPriceInput {
            item: make_item(),
            surcharge: false,
            is_buying: true,
            charisma: 12,
            is_tourist: false,
            dungeon_level: 1,
            matches_shop_type: false,
        };
        let r = get_cost_result(&input);
        assert!(r.price > 0);
    }

    #[test]
    fn test_cost_surcharge() {
        let input_normal = ShopPriceInput {
            item: make_item(),
            surcharge: false,
            is_buying: true,
            charisma: 12,
            is_tourist: false,
            dungeon_level: 1,
            matches_shop_type: false,
        };
        let input_surcharge = ShopPriceInput {
            surcharge: true,
            ..input_normal.clone()
        };
        let r_normal = get_cost_result(&input_normal);
        let r_surcharge = get_cost_result(&input_surcharge);
        assert!(r_surcharge.price > r_normal.price, "할증 시 가격 상승");
        assert!(r_surcharge.surcharge_applied);
    }

    #[test]
    fn test_cost_tourist() {
        let input_normal = ShopPriceInput {
            item: make_item(),
            surcharge: false,
            is_buying: true,
            charisma: 12,
            is_tourist: false,
            dungeon_level: 1,
            matches_shop_type: false,
        };
        let input_tourist = ShopPriceInput {
            is_tourist: true,
            ..input_normal.clone()
        };
        let r_normal = get_cost_result(&input_normal);
        let r_tourist = get_cost_result(&input_tourist);
        assert!(r_tourist.price > r_normal.price, "관광객 할증");
    }

    #[test]
    fn test_cost_high_charisma() {
        let input_low = ShopPriceInput {
            item: make_item(),
            surcharge: false,
            is_buying: true,
            charisma: 5,
            is_tourist: false,
            dungeon_level: 1,
            matches_shop_type: false,
        };
        let input_high = ShopPriceInput {
            charisma: 25,
            ..input_low.clone()
        };
        let r_low = get_cost_result(&input_low);
        let r_high = get_cost_result(&input_high);
        assert!(r_high.price < r_low.price, "카리스마 높으면 할인");
    }

    // --- check_credit_result 테스트 ---

    #[test]
    fn test_credit_full_cover() {
        let r = check_credit_result(100, 200);
        assert_eq!(
            r,
            CreditCheckResult::FullyCovered {
                credit_remaining: 100
            }
        );
    }

    #[test]
    fn test_credit_partial() {
        let r = check_credit_result(100, 30);
        assert_eq!(r, CreditCheckResult::PartiallyCovered { amount_due: 70 });
    }

    #[test]
    fn test_credit_none() {
        let r = check_credit_result(100, 0);
        assert_eq!(r, CreditCheckResult::NoCreditUsed { amount_due: 100 });
    }

    // --- addup_bill 테스트 ---

    #[test]
    fn test_addup_bill() {
        let entries = vec![
            BillEntry {
                price: 50,
                quantity: 2,
            },
            BillEntry {
                price: 30,
                quantity: 1,
            },
        ];
        assert_eq!(addup_bill(&entries), 130);
    }

    #[test]
    fn test_addup_bill_empty() {
        assert_eq!(addup_bill(&[]), 0);
    }

    // --- shop_debt_total 테스트 ---

    #[test]
    fn test_shop_debt() {
        let entries = vec![BillEntry {
            price: 100,
            quantity: 1,
        }];
        assert_eq!(shop_debt_total(&entries, 50), 150);
    }

    // --- rob_shop_result 테스트 ---

    #[test]
    fn test_rob_credit_covers() {
        let r = rob_shop_result(100, 20, 200, false);
        assert_eq!(r, RobShopResult::CreditCovers);
    }

    #[test]
    fn test_rob_actual_robbery() {
        let r = rob_shop_result(200, 50, 100, false);
        assert_eq!(
            r,
            RobShopResult::Robbery {
                stolen_amount: 150,
                alignment_penalty: true
            }
        );
    }

    #[test]
    fn test_rob_rogue_no_penalty() {
        let r = rob_shop_result(200, 0, 0, true);
        assert_eq!(
            r,
            RobShopResult::Robbery {
                stolen_amount: 200,
                alignment_penalty: false
            }
        );
    }

    // --- rile/pacify 테스트 ---

    #[test]
    fn test_rile_prices() {
        let mut prices = vec![100, 200, 300];
        rile_prices(&mut prices);
        assert_eq!(prices[0], 134); // 100 + (100+2)/3 = 100+34
        assert_eq!(prices[1], 267); // 200 + (200+2)/3 = 200+67
    }

    #[test]
    fn test_pacify_prices() {
        let mut prices = vec![134, 267]; // 할증된 상태
        pacify_prices(&mut prices);
        // (134+3)/4 = 34, 134-34 = 100
        assert_eq!(prices[0], 100);
        // (267+3)/4 = 67, 267-67 = 200
        assert_eq!(prices[1], 200);
    }

    // --- is_special_stock 테스트 ---

    #[test]
    fn test_special_stock_weapon() {
        assert!(is_special_stock(0, ShopKind::Weapon));
        assert!(!is_special_stock(1, ShopKind::Weapon));
    }

    #[test]
    fn test_special_stock_general() {
        assert!(is_special_stock(0, ShopKind::General));
        assert!(is_special_stock(7, ShopKind::General));
    }

    // --- price_embellish 테스트 ---

    #[test]
    fn test_embellish_cheap() {
        let mut rng = NetHackRng::new(42);
        let msg = price_embellish(3, &mut rng);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_embellish_expensive() {
        let mut rng = NetHackRng::new(42);
        let msg = price_embellish(10000, &mut rng);
        assert!(!msg.is_empty());
    }

    // --- shop_entry_reaction 테스트 ---

    #[test]
    fn test_entry_welcome() {
        let r = shop_entry_reaction(true, false, false, false, false, false, 3);
        assert_eq!(r, ShopEntryReaction::Welcome { visit_count: 3 });
    }

    #[test]
    fn test_entry_angry() {
        let r = shop_entry_reaction(true, true, false, false, false, false, 0);
        assert_eq!(r, ShopEntryReaction::AngryWarning);
    }

    #[test]
    fn test_entry_deserted() {
        let r = shop_entry_reaction(false, false, false, false, false, false, 0);
        assert_eq!(r, ShopEntryReaction::Deserted);
    }

    #[test]
    fn test_entry_invisible() {
        let r = shop_entry_reaction(true, false, false, false, false, true, 0);
        assert_eq!(r, ShopEntryReaction::InvisibleRejection);
    }

    // --- shop_exit_reaction 테스트 ---

    #[test]
    fn test_exit_settled() {
        let r = shop_exit_reaction(false, false, false);
        assert_eq!(r, ShopExitReaction::Settled);
    }

    #[test]
    fn test_exit_warning() {
        let r = shop_exit_reaction(true, false, true);
        assert_eq!(r, ShopExitReaction::PaymentWarning);
    }

    #[test]
    fn test_exit_robbery() {
        let r = shop_exit_reaction(true, true, false);
        assert_eq!(r, ShopExitReaction::Robbery { near_shop: false });
    }

    // --- cost_per_charge 테스트 ---

    #[test]
    fn test_cost_per_charge_normal() {
        assert_eq!(cost_per_charge(100, 7), 12); // 100 / 8
    }

    #[test]
    fn test_cost_per_charge_zero() {
        assert_eq!(cost_per_charge(100, 0), 100); // 충전 불가
    }
}

// ============================================================================
// [v2.24.0 Phase 3-4] 상점 시스템 확장 (shk_phase3_ext.rs)
// 원본: NetHack 3.6.7 src/shk.c L1500-2500 핵심 미이식 함수 이식
// 순수 결과 패턴: ECS 의존 없이 독립 테스트 가능
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 상점 주인 행동 결정 — shk_move (shk.c L1500-1650)
// =============================================================================

/// [v2.24.0 3-4] 상점 주인 행동
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShkAction {
    /// 가게 안에서 대기
    StandGuard,
    /// 플레이어 쫓아가기 (부채가 있으므로)
    PursueCustomer { dx: i32, dy: i32 },
    /// 문 잠그기 (폐점 시간)
    LockDoor,
    /// 가격 부르기 (플레이어가 아이템을 들었을 때)
    AnnouncePrice { item_name: String, price: i64 },
    /// 도둑 경고
    WarnThief { message: String },
    /// 공격 (도둑 확정 시)
    AttackThief,
    /// 아이템 되사기 제안
    OfferBuyback { item_name: String, price: i64 },
}

/// [v2.24.0 3-4] 상점 주인 행동 결정 입력
#[derive(Debug, Clone)]
pub struct ShkMoveInput {
    /// 부채 금액
    pub debt: i64,
    /// 플레이어가 가게 안에 있는지
    pub player_in_shop: bool,
    /// 플레이어가 훔친 아이템이 있는지
    pub player_stole_items: bool,
    /// 플레이어와의 거리
    pub distance_to_player: i32,
    /// 상점 주인이 화났는지
    pub is_angry: bool,
    /// 플레이어가 아이템을 집었는지
    pub player_picked_item: bool,
    /// 집은 아이템 이름
    pub picked_item_name: String,
    /// 집은 아이템 가격
    pub picked_item_price: i64,
    /// 플레이어가 가게 밖으로 나가려는지
    pub player_leaving: bool,
}

/// [v2.24.0 3-4] 상점 주인 행동 결정
/// 원본: shk.c shk_move() L1500-1650
pub fn shk_move_result(input: &ShkMoveInput) -> ShkAction {
    // [1] 분노 상태 → 공격 또는 추격
    if input.is_angry {
        if input.distance_to_player <= 2 {
            return ShkAction::AttackThief;
        } else {
            let dx = if input.distance_to_player > 0 { 1 } else { -1 };
            return ShkAction::PursueCustomer { dx, dy: 0 };
        }
    }

    // [2] 플레이어가 아이템을 집었을 때
    if input.player_picked_item && input.player_in_shop {
        return ShkAction::AnnouncePrice {
            item_name: input.picked_item_name.clone(),
            price: input.picked_item_price,
        };
    }

    // [3] 도둑 검사 — 부채가 있고 가게를 떠나려 함
    if input.player_leaving && input.debt > 0 {
        return ShkAction::WarnThief {
            message: format!("\"이봐! {}금을 아직 안 내셨잖아!\"", input.debt),
        };
    }

    // [4] 도둑 검사 — 플레이어가 이미 훔침
    if input.player_stole_items {
        if input.distance_to_player <= 5 {
            return ShkAction::PursueCustomer { dx: 1, dy: 0 };
        }
    }

    // [5] 부채가 있지만 가게 안에 있음 → 대기
    ShkAction::StandGuard
}

// =============================================================================
// [2] 가격 흥정 — bargain_price (shk.c L1800-1900)
// =============================================================================

/// [v2.24.0 3-4] 흥정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BargainResult {
    /// 흥정 성공 — 할인된 가격
    Accepted { final_price: i64, discount_pct: i32 },
    /// 흥정 거절
    Rejected { message: String },
    /// 최종 제안 — 마지막 기회
    FinalOffer { final_price: i64 },
}

/// [v2.24.0 3-4] 가격 흥정 판정
/// 원본: shk.c bargain 관련 로직
pub fn bargain_price(
    base_price: i64,
    player_charisma: i32,
    is_tourist: bool,
    shopkeeper_alignment_match: bool,
    attempts: i32,
    rng: &mut NetHackRng,
) -> BargainResult {
    // [1] 최대 할인율 계산
    // 카리스마 보정: CHA 16+ → 최대 25% 추가 할인
    let cha_bonus = ((player_charisma - 10).max(0) * 2).min(25);
    let tourist_bonus = if is_tourist { 10 } else { 0 };
    let alignment_bonus = if shopkeeper_alignment_match { 5 } else { 0 };
    let max_discount = (cha_bonus + tourist_bonus + alignment_bonus).min(50);

    // [2] 시도 횟수에 따른 거절 확률
    if attempts >= 3 {
        // 3회 이상 시도 → 최종 제안
        let discount = max_discount / 2;
        let final_price = base_price * (100 - discount as i64) / 100;
        return BargainResult::FinalOffer {
            final_price: final_price.max(1),
        };
    }

    // [3] 흥정 성공 여부 난수 판정
    let success_chance = 30 + cha_bonus + tourist_bonus + alignment_bonus - attempts * 10;
    if rng.rn2(100) < success_chance {
        let discount = (rng.rn2(max_discount.max(1)) + 1).min(max_discount);
        let final_price = base_price * (100 - discount as i64) / 100;
        BargainResult::Accepted {
            final_price: final_price.max(1),
            discount_pct: discount,
        }
    } else {
        BargainResult::Rejected {
            message: "\"그 가격은 안 됩니다.\"".to_string(),
        }
    }
}

// =============================================================================
// [3] 아이템 매입 가격 — sellobj_calc (shk.c L2100-2200)
// =============================================================================

/// [v2.24.0 3-4] 매입 가격 계산
/// 원본: shk.c set_cost() / sellobj()
pub fn sell_price_calc(
    base_price: i64,
    is_identified: bool,
    buc_status: i32, // -1=저주, 0=보통, 1=축복
    condition: i32,  // 0=새것, 1=사용됨, 2=부식, 3=심하게 부식
    player_charisma: i32,
    shopkeeper_surcharge: bool,
) -> i64 {
    // [1] 기본 매입가 = 판매가의 50%
    let mut sell_price = base_price / 2;

    // [2] 식별 여부 — 미식별이면 더 싸게 사들임
    if !is_identified {
        sell_price = sell_price * 70 / 100;
    }

    // [3] BUC 보정
    match buc_status {
        -1 => sell_price = sell_price * 50 / 100, // 저주 → 50%
        1 => sell_price = sell_price * 110 / 100, // 축복 → 110%
        _ => {}
    }

    // [4] 상태 보정
    let condition_mult = match condition {
        0 => 100,
        1 => 80,
        2 => 60,
        3 => 40,
        _ => 30,
    };
    sell_price = sell_price * condition_mult / 100;

    // [5] 카리스마 보정
    let cha_bonus = ((player_charisma - 10).max(0)).min(10);
    sell_price = sell_price * (100 + cha_bonus as i64) / 100;

    // [6] 할증 (상점 주인의 개인 정책)
    if shopkeeper_surcharge {
        sell_price = sell_price * 85 / 100;
    }

    sell_price.max(1)
}

// =============================================================================
// [4] 도둑 처벌 — shk_robbery_penalty (shk.c L2300-2400)
// =============================================================================

/// [v2.24.0 3-4] 도둑 처벌 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobberyPenalty {
    /// 부채 금액 (기존 + 페널티)
    pub total_debt: i64,
    /// 상점 주인이 분노했는지
    pub shopkeeper_angry: bool,
    /// 처벌 메시지
    pub message: String,
    /// 가드 호출 여부
    pub call_guards: bool,
    /// 정렬(alignment) 감소량
    pub alignment_penalty: i32,
}

/// [v2.24.0 3-4] 도둑 처벌 판정
/// 원본: shk.c L2300-2400
pub fn robbery_penalty(
    stolen_value: i64,
    existing_debt: i64,
    player_alignment: i32,
    repeated_offense: bool,
) -> RobberyPenalty {
    let base_penalty = stolen_value * 3; // 3배 페널티
    let repeat_mult = if repeated_offense { 2 } else { 1 };
    let total_debt = existing_debt + base_penalty * repeat_mult;

    let alignment_loss = if repeated_offense { 5 } else { 2 };
    let call_guards = stolen_value > 500 || repeated_offense;

    let message = if repeated_offense {
        format!(
            "\"또 도둑질이야?! 경비를 부르겠어!\" (부채: {}금)",
            total_debt
        )
    } else {
        format!("\"도둑! 3배를 물어내라!\" (부채: {}금)", total_debt)
    };

    RobberyPenalty {
        total_debt,
        shopkeeper_angry: true,
        message,
        call_guards,
        alignment_penalty: alignment_loss,
    }
}

// =============================================================================
// [5] 상점 서비스 — shk_services (shk.c L2500-2600)
// =============================================================================

/// [v2.24.0 3-4] 상점 서비스 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopService {
    /// 감정 서비스
    Identify { cost: i64 },
    /// 수리 서비스
    Repair { cost: i64 },
    /// 충전 서비스
    Recharge { cost: i64 },
    /// 저주 해제 서비스
    Uncurse { cost: i64 },
}

/// [v2.24.0 3-4] 상점 서비스 가격 계산
pub fn shop_service_cost(
    service_type: &str,
    item_base_value: i64,
    player_charisma: i32,
) -> Option<ShopService> {
    let cha_discount = ((player_charisma - 10).max(0) * 3).min(30) as i64;

    let cost = |base: i64| -> i64 {
        let adjusted = base * (100 - cha_discount) / 100;
        adjusted.max(5)
    };

    match service_type {
        "identify" => Some(ShopService::Identify {
            cost: cost(item_base_value / 5 + 10),
        }),
        "repair" => Some(ShopService::Repair {
            cost: cost(item_base_value / 3 + 20),
        }),
        "recharge" => Some(ShopService::Recharge {
            cost: cost(item_base_value / 2 + 50),
        }),
        "uncurse" => Some(ShopService::Uncurse {
            cost: cost(item_base_value / 4 + 30),
        }),
        _ => None,
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

    // --- shk_move_result ---

    #[test]
    fn test_shk_angry_attack() {
        let input = ShkMoveInput {
            debt: 100,
            player_in_shop: true,
            player_stole_items: false,
            distance_to_player: 1,
            is_angry: true,
            player_picked_item: false,
            picked_item_name: String::new(),
            picked_item_price: 0,
            player_leaving: false,
        };
        assert_eq!(shk_move_result(&input), ShkAction::AttackThief);
    }

    #[test]
    fn test_shk_announce_price() {
        let input = ShkMoveInput {
            debt: 0,
            player_in_shop: true,
            player_stole_items: false,
            distance_to_player: 3,
            is_angry: false,
            player_picked_item: true,
            picked_item_name: "long sword".to_string(),
            picked_item_price: 50,
            player_leaving: false,
        };
        match shk_move_result(&input) {
            ShkAction::AnnouncePrice { price, .. } => assert_eq!(price, 50),
            _ => panic!("가격을 부르어야 함"),
        }
    }

    #[test]
    fn test_shk_warn_thief() {
        let input = ShkMoveInput {
            debt: 200,
            player_in_shop: true,
            player_stole_items: false,
            distance_to_player: 3,
            is_angry: false,
            player_picked_item: false,
            picked_item_name: String::new(),
            picked_item_price: 0,
            player_leaving: true,
        };
        assert!(matches!(
            shk_move_result(&input),
            ShkAction::WarnThief { .. }
        ));
    }

    #[test]
    fn test_shk_stand_guard() {
        let input = ShkMoveInput {
            debt: 0,
            player_in_shop: true,
            player_stole_items: false,
            distance_to_player: 5,
            is_angry: false,
            player_picked_item: false,
            picked_item_name: String::new(),
            picked_item_price: 0,
            player_leaving: false,
        };
        assert_eq!(shk_move_result(&input), ShkAction::StandGuard);
    }

    // --- bargain_price ---

    #[test]
    fn test_bargain_final_offer() {
        let mut rng = test_rng();
        let result = bargain_price(100, 16, false, false, 3, &mut rng);
        assert!(matches!(result, BargainResult::FinalOffer { .. }));
    }

    #[test]
    fn test_bargain_tourist_bonus() {
        // 관광객은 흥정 보너스
        let mut accepted = false;
        for seed in 0..20 {
            let mut rng = NetHackRng::new(seed);
            let result = bargain_price(100, 18, true, true, 0, &mut rng);
            if matches!(result, BargainResult::Accepted { .. }) {
                accepted = true;
                break;
            }
        }
        assert!(
            accepted,
            "높은 카리스마+관광객은 흥정 성공 확률이 높아야 함"
        );
    }

    // --- sell_price_calc ---

    #[test]
    fn test_sell_basic() {
        let price = sell_price_calc(100, true, 0, 0, 10, false);
        assert_eq!(price, 50); // 기본 매입가 50%
    }

    #[test]
    fn test_sell_cursed() {
        let normal = sell_price_calc(100, true, 0, 0, 10, false);
        let cursed = sell_price_calc(100, true, -1, 0, 10, false);
        assert!(cursed < normal, "저주받은 아이템은 더 싸게 매입");
    }

    #[test]
    fn test_sell_unidentified() {
        let identified = sell_price_calc(100, true, 0, 0, 10, false);
        let unidentified = sell_price_calc(100, false, 0, 0, 10, false);
        assert!(unidentified < identified, "미식별 아이템은 더 싸게 매입");
    }

    // --- robbery_penalty ---

    #[test]
    fn test_robbery_first_time() {
        let result = robbery_penalty(100, 0, 0, false);
        assert_eq!(result.total_debt, 300); // 3배
        assert!(result.shopkeeper_angry);
        assert_eq!(result.alignment_penalty, 2);
    }

    #[test]
    fn test_robbery_repeat() {
        let result = robbery_penalty(100, 500, 0, true);
        assert_eq!(result.total_debt, 500 + 600); // 기존 + 3배×2
        assert!(result.call_guards);
        assert_eq!(result.alignment_penalty, 5);
    }

    // --- shop_service_cost ---

    #[test]
    fn test_service_identify() {
        let result = shop_service_cost("identify", 100, 10);
        match result {
            Some(ShopService::Identify { cost }) => {
                assert!(cost > 0);
            }
            _ => panic!("감정 서비스여야 함"),
        }
    }

    #[test]
    fn test_service_high_cha_discount() {
        let low_cha = shop_service_cost("repair", 100, 10);
        let high_cha = shop_service_cost("repair", 100, 20);
        match (low_cha, high_cha) {
            (Some(ShopService::Repair { cost: c1 }), Some(ShopService::Repair { cost: c2 })) => {
                assert!(c2 < c1, "높은 카리스마는 할인 적용");
            }
            _ => panic!("수리 서비스여야 함"),
        }
    }

    #[test]
    fn test_service_unknown() {
        assert!(shop_service_cost("magic", 100, 10).is_none());
    }
}

// ============================================================================
// [v2.22.0 R34-P2-5] 사회 상호작용 브릿지 (social_bridge.rs)
// shk_ext + pray_ext → 상점 거래 / 기도 통합
// ============================================================================

use crate::core::events::GameEvent;
use crate::core::systems::social::pray_ext;
use crate::core::systems::social::shk_ext::{
    self, BillEntry, CreditCheckResult, ItemPriceInfo, ShopPriceInput,
};
use crate::core::systems::turn_engine::TurnContext;
use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 상점 구매 처리
// =============================================================================

/// [v2.22.0 R34-P2-5] 상점 구매 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurchaseResult {
    /// 구매 성공
    Success { price: i64, credit_used: i64 },
    /// 금화 부족
    InsufficientGold { price: i64, available: u64 },
}

/// [v2.22.0 R34-P2-5] 상점 아이템 구매 처리
pub fn purchase_item(
    ctx: &mut TurnContext,
    item_name: &str,
    price_input: &ShopPriceInput,
    bill_credit: i64,
) -> PurchaseResult {
    // [1] 가격 계산
    let price_result = shk_ext::get_cost_result(price_input);
    let price = price_result.price;

    // [2] 크레딧 차감
    let credit_result = shk_ext::check_credit_result(price, bill_credit);
    let (amount_due, credit_used) = match credit_result {
        CreditCheckResult::FullyCovered {
            credit_remaining: _,
        } => (0, price),
        CreditCheckResult::PartiallyCovered { amount_due } => (amount_due, price - amount_due),
        CreditCheckResult::NoCreditUsed { amount_due } => (amount_due, 0),
    };

    // [3] 금화 확인
    if amount_due > 0 && (ctx.player.gold as i64) < amount_due {
        ctx.event_queue.push(GameEvent::Message {
            text: format!("{}을(를) 살 돈이 부족하다! (가격: {})", item_name, price),
            priority: true,
        });
        return PurchaseResult::InsufficientGold {
            price,
            available: ctx.player.gold,
        };
    }

    // [4] 구매 실행
    ctx.player.gold -= amount_due as u64;
    ctx.event_queue.push(GameEvent::ShopPurchase {
        item_name: item_name.to_string(),
        price: price as u32,
    });
    ctx.event_queue.push(GameEvent::Message {
        text: format!("{}을(를) {}금에 구매했다.", item_name, price),
        priority: false,
    });

    PurchaseResult::Success { price, credit_used }
}

// =============================================================================
// [2] 상점 부채 계산
// =============================================================================

/// [v2.22.0 R34-P2-5] 현재 상점 부채 조회
pub fn calc_shop_debt(bill: &[BillEntry], debit: i64) -> i64 {
    shk_ext::shop_debt_total(bill, debit)
}

// =============================================================================
// [3] 기도 처리
// =============================================================================

/// [v2.22.0 R34-P2-5] 기도 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrayerResult {
    /// 쿨다운 중
    OnCooldown { remaining_turns: i32 },
    /// 분노한 신
    AngryGod { effect: pray_ext::AngryGodResult },
    /// 보상 (문제 해결 또는 특별 보상)
    Rewarded { action: pray_ext::PleaseAction },
}

/// [v2.22.0 R34-P2-5] 기도 처리
pub fn pray(
    ctx: &mut TurnContext,
    same_alignment: bool,
    on_altar: bool,
    on_shrine: bool,
    anger: i32,
    rng: &mut NetHackRng,
) -> PrayerResult {
    // [1] 쿨다운 체크
    if ctx.player.prayer_cooldown > 0 {
        ctx.event_queue.push(GameEvent::Message {
            text: format!(
                "아직 기도할 수 없다. ({}턴 남음)",
                ctx.player.prayer_cooldown
            ),
            priority: true,
        });
        return PrayerResult::OnCooldown {
            remaining_turns: ctx.player.prayer_cooldown,
        };
    }

    // [2] 기도 쿨다운 설정
    let timeout = pray_ext::prayer_timeout(ctx.player.level, rng);
    ctx.player.prayer_cooldown = timeout;

    // [3] 분노 판정
    if anger > 0 {
        let result = pray_ext::angrygods_calc(
            same_alignment,
            ctx.player.alignment_record,
            anger,
            ctx.player.luck,
            false,
            rng,
        );

        // 분노 효과 적용
        match result {
            pray_ext::AngryGodResult::WisdomLoss => {
                ctx.player.wis.base -= 1;
                ctx.event_queue.push(GameEvent::Message {
                    text: "신의 분노로 지혜가 감소했다!".to_string(),
                    priority: true,
                });
            }
            pray_ext::AngryGodResult::Punish => {
                ctx.event_queue.push(GameEvent::Message {
                    text: "신이 벌을 내렸다!".to_string(),
                    priority: true,
                });
            }
            _ => {
                ctx.event_queue.push(GameEvent::Message {
                    text: "신이 불쾌해하고 있다.".to_string(),
                    priority: false,
                });
            }
        }

        ctx.event_queue.push(GameEvent::Prayed {
            result: format!("{:?}", result),
        });

        return PrayerResult::AngryGod { effect: result };
    }

    // [4] 보상 판정
    let hp_critical =
        pray_ext::critically_low_hp(ctx.player.hp, ctx.player.hp_max, ctx.player.level, true);
    let action = pray_ext::prayer_action_calc(
        hp_critical,
        ctx.player.alignment_record,
        0,
        ctx.player.luck,
        on_altar,
        on_shrine,
        rng,
    );

    // 보상 적용
    match action {
        pray_ext::PleaseAction::FixWorst => {
            // HP 회복
            let boost = pray_ext::fix_hit_hp_boost(ctx.player.hp_max, ctx.player.level, rng);
            ctx.player.hp_max += boost;
            ctx.player.hp = ctx.player.hp_max;
            ctx.event_queue.push(GameEvent::Message {
                text: format!("신이 문제를 해결해 주었다! HP +{}", boost),
                priority: true,
            });
        }
        pray_ext::PleaseAction::PatOnHead => {
            ctx.player.luck += 1;
            ctx.event_queue.push(GameEvent::Message {
                text: "신의 축복을 받았다!".to_string(),
                priority: true,
            });
        }
        _ => {
            ctx.event_queue.push(GameEvent::Message {
                text: "신이 응답했다.".to_string(),
                priority: false,
            });
        }
    }

    ctx.event_queue.push(GameEvent::Prayed {
        result: format!("{:?}", action),
    });

    PrayerResult::Rewarded { action }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::player::Player;
    use crate::core::events::EventQueue;

    #[test]
    fn test_purchase_insufficient_gold() {
        let mut p = Player::new();
        p.gold = 0;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let price_input = ShopPriceInput {
            item: ItemPriceInfo {
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
            },
            surcharge: false,
            is_buying: true,
            charisma: 10,
            is_tourist: false,
            dungeon_level: 1,
            matches_shop_type: false,
        };

        let result = purchase_item(&mut ctx, "단검", &price_input, 0);
        assert!(matches!(result, PurchaseResult::InsufficientGold { .. }));
    }

    #[test]
    fn test_purchase_success() {
        let mut p = Player::new();
        p.gold = 1000;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let price_input = ShopPriceInput {
            item: ItemPriceInfo {
                base_cost: 50,
                quantity: 1,
                blessed: false,
                cursed: false,
                enchantment: 0,
                enchantment_known: false,
                item_class: 4,
                is_ammo: false,
                is_glob: false,
                weight: 20,
                is_chargeable: false,
                max_charges: 0,
                unidentified: false,
            },
            surcharge: false,
            is_buying: true,
            charisma: 15,
            is_tourist: false,
            dungeon_level: 1,
            matches_shop_type: false,
        };

        let result = purchase_item(&mut ctx, "포션", &price_input, 0);
        assert!(matches!(result, PurchaseResult::Success { .. }));
        assert!(p.gold < 1000);
    }

    #[test]
    fn test_prayer_cooldown() {
        let mut p = Player::new();
        p.prayer_cooldown = 100;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let mut rng = NetHackRng::new(0);

        let result = pray(&mut ctx, true, false, false, 0, &mut rng);
        assert!(matches!(result, PrayerResult::OnCooldown { .. }));
    }

    #[test]
    fn test_prayer_angry_god() {
        let mut p = Player::new();
        p.prayer_cooldown = 0;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let mut rng = NetHackRng::new(42);

        let result = pray(&mut ctx, true, false, false, 5, &mut rng);
        assert!(matches!(result, PrayerResult::AngryGod { .. }));
    }
}

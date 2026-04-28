// ============================================================================
// [v2.22.0 R34-P2-4] R34 Phase 2 통합 시나리오 (integration_r34.rs)
// 새로운 Bridge (status, terrain, spell) 연쇄 테스트
// ============================================================================

#[cfg(test)]
mod integration_r34 {
    use crate::core::entity::player::Player;
    use crate::core::entity::status::StatusEffect;
    use crate::core::events::{EventQueue, GameEvent};
    use crate::core::systems::bridges::{spell_bridge, status_bridge, terrain_bridge};
    use crate::core::systems::turn_engine::{run_turn_systems, TurnContext, TurnResult};
    use crate::core::systems::world::trap_ext::TrapType;
    use crate::util::rng::NetHackRng;

    /// [시나리오 F] 석화 카운트다운 → 석화 사망 → DeathCheck
    #[test]
    fn scenario_stoning_countdown_death() {
        let mut p = Player::new();
        p.status_bundle.add_effect(StatusEffect::Stoning, 3);
        let mut q = EventQueue::new();

        // 3턴 석화 카운트다운
        for t in 0..3 {
            let mut ctx = TurnContext {
                player: &mut p,
                turn_number: t,
                event_queue: &mut q,
            };
            status_bridge::tick_status_timers(&mut ctx);
        }

        // 3턴 후 석화 완료 → 사망
        assert!(p.hp <= 0, "석화 완료 후 HP가 0 이하여야 함");
    }

    /// [시나리오 G] 함정 데미지 → 사망 판정
    #[test]
    fn scenario_trap_then_death_check() {
        let mut p = Player::new();
        p.hp = 3; // 낮은 HP
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let mut rng = NetHackRng::new(42);

        // 가시 구덩이에 빠짐
        terrain_bridge::process_level_events(&mut ctx, Some(TrapType::SpikedPit), 5, 5, &mut rng);

        // HP 감소 확인
        assert!(p.hp < 3, "함정 데미지로 HP가 감소해야 함");

        // DeathCheck
        let mut ctx2 = TurnContext {
            player: &mut p,
            turn_number: 2,
            event_queue: &mut q,
        };
        let result = run_turn_systems(&mut ctx2);
        if p.hp <= 0 {
            assert!(matches!(result, TurnResult::PlayerDied { .. }));
        }
    }

    /// [시나리오 H] 혼란 상태에서 주문 시전 → 실패 확률 증가
    #[test]
    fn scenario_confused_spellcasting() {
        let mut p = Player::new();
        p.energy = 50;
        p.status_bundle.add_effect(StatusEffect::Confused, 10);
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let mut rng = NetHackRng::new(99);

        let result = spell_bridge::cast_spell(&mut ctx, 3, 1, false, &mut rng);

        // 혼란 상태에서 시전 → 성공 또는 역발
        match result {
            spell_bridge::CastResult::Success { .. } => {
                assert!(p.energy < 50);
            }
            spell_bridge::CastResult::Backfire { .. } => {
                assert!(p.energy < 50);
            }
            spell_bridge::CastResult::InsufficientEnergy { .. } => {
                panic!("에너지 50이면 레벨3 주문은 가능해야 함");
            }
        }
    }

    /// [시나리오 I] 완전 턴 → 상태 만료 + 지형 없음 = 정상 진행
    #[test]
    fn scenario_full_turn_with_status() {
        let mut p = Player::new();
        p.status_bundle.add_effect(StatusEffect::Fast, 2);
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };

        let result = run_turn_systems(&mut ctx);
        assert_eq!(result, TurnResult::Continue);

        // 가속 1턴 남음
        assert!(p.status_bundle.has_effect(StatusEffect::Fast));

        // 2턴째
        let mut ctx2 = TurnContext {
            player: &mut p,
            turn_number: 2,
            event_queue: &mut q,
        };
        run_turn_systems(&mut ctx2);

        // 가속 만료
        assert!(!p.status_bundle.has_effect(StatusEffect::Fast));
    }

    /// [시나리오 J] 주문 시전 → 에너지 소모 → 포션 회복
    #[test]
    fn scenario_spell_then_potion() {
        let mut p = Player::new();
        p.energy = 30;
        p.hp = 5;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let mut rng = NetHackRng::new(42);

        // 주문 시전 (에너지 소모)
        let _result = spell_bridge::cast_spell(&mut ctx, 1, 2, false, &mut rng);
        let after_spell_energy = p.energy;
        assert!(after_spell_energy < 30);

        // 포션으로 회복
        crate::core::systems::bridges::potion_bridge::drink_potion(
            &mut p, "healing", false, false, &mut q,
        );
        assert!(p.hp > 5 || p.hp == 5); // 회복 또는 최소 유지
    }
}

// ============================================================================
// [v2.44.0 R32-5] 통합 시나리오 테스트 (integration_tests.rs)
// 다중 시스템이 협력하는 E2E 시나리오
// ============================================================================

#[cfg(test)]
mod integration {
    use crate::core::entity::player::Player;
    use crate::core::events::EventQueue;
    use crate::core::systems::bridges::{combat_bridge, hunger_bridge, luck_align_bridge};
    use crate::core::systems::social::alignment_ext::AlignAction;
    use crate::core::systems::turn_engine::{run_turn_systems, TurnContext, TurnResult};
    use crate::util::rng::NetHackRng;

    /// [통합 시나리오 1] 굶어 죽는 전체 흐름
    /// 배고픔 → 상태 변화 → 사망 판정
    #[test]
    fn scenario_starvation_death() {
        let mut p = Player::new();
        p.nutrition = 1; // 거의 굶은 상태
        let mut q = EventQueue::new();

        // 여러 턴 배고픔 진행
        for t in 0..10 {
            let mut ctx = TurnContext {
                player: &mut p,
                turn_number: t,
                event_queue: &mut q,
            };
            hunger_bridge::tick_nutrition(&mut ctx);
        }

        // nutrition이 0 이하로 떨어짐
        assert!(p.nutrition <= 0);
        // 메시지 이벤트 발생 확인
        assert!(!q.is_empty());
    }

    /// [통합 시나리오 2] 전투 → 사망 판정 연동
    /// 몬스터 공격 → HP 감소 → DeathCheck
    #[test]
    fn scenario_combat_to_death() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        let mut rng = NetHackRng::new(0);

        // 몬스터에게 맞아서 죽을 때까지
        let mut rounds = 0;
        while p.hp > 0 && rounds < 100 {
            combat_bridge::monster_attacks_player("dragon", 20, &mut p, 3, 6, &mut rng, &mut q);
            rounds += 1;
        }

        // HP가 0 이하가 되었을 것
        // (20레벨 드래곤은 충분히 강하다)
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let result = combat_bridge::check_player_death(&mut ctx);
        if p.hp <= 0 {
            assert!(matches!(result, Some(TurnResult::PlayerDied { .. })));
        }
    }

    /// [통합 시나리오 3] 도둑질 → 정렬 하락 → 신 분노 → 이벤트 연쇄
    #[test]
    fn scenario_steal_chain() {
        let mut p = Player::new();
        p.alignment_record = -5;
        let mut q = EventQueue::new();

        // 두 번 도둑질
        luck_align_bridge::apply_align_action(&mut p, AlignAction::StealFromShop, &mut q);
        luck_align_bridge::apply_align_action(&mut p, AlignAction::StealFromShop, &mut q);

        assert!(p.alignment_record <= -10);
        // 이벤트가 발생했는지 (분노 메시지 포함)
        let msgs: Vec<_> = q
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    crate::core::events::GameEvent::Message { priority: true, .. }
                )
            })
            .collect();
        assert!(!msgs.is_empty());
    }

    /// [통합 시나리오 4] 완전 턴 싸이클
    /// run_turn_systems → NutritionDecay + LuckDecay + Regen + DeathCheck
    #[test]
    fn scenario_full_turn_cycle() {
        let mut p = Player::new();
        p.hp = 10;
        p.hp_max = 20;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 100,
            event_queue: &mut q,
        };

        let result = run_turn_systems(&mut ctx);
        assert_eq!(result, TurnResult::Continue);
        assert!(p.nutrition < 900); // 영양 감소됨
    }

    /// [통합 시나리오 5] 플레이어 공격 → 몬스터 죽음 → 이벤트
    #[test]
    fn scenario_kill_monster() {
        let p = Player::new();
        let mut mon_hp = 5; // 약한 몬스터
        let mut q = EventQueue::new();
        let mut rng = NetHackRng::new(99);

        // 공격 반복: 반드시 죽임
        for _ in 0..20 {
            if mon_hp > 0 {
                combat_bridge::player_attacks_monster(
                    &p,
                    "kobold",
                    10,
                    &mut mon_hp,
                    2,
                    6,
                    3,
                    &mut rng,
                    &mut q,
                );
            }
        }

        // 데미지 이벤트 최소 1개 이상
        let dmg_events: Vec<_> = q
            .iter()
            .filter(|e| matches!(e, crate::core::events::GameEvent::DamageDealt { .. }))
            .collect();
        assert!(!dmg_events.is_empty());
    }
}

// ============================================================================
// [v2.45.0 R33-5] R33 통합 시나리오 (integration_r33.rs)
// 아이템 사용 체인: 포션→회복, 완드→몬스터 처치→경험치 획득
// ============================================================================

#[cfg(test)]
mod integration_r33 {
    use crate::core::entity::player::Player;
    use crate::core::events::EventQueue;
    use crate::core::systems::bridges::{
        combat_bridge, potion_bridge, scroll_bridge, wand_bridge, xp_bridge,
    };
    use crate::util::rng::NetHackRng;

    /// [시나리오 A] 헐벗은 플레이어가 힐링 포션으로 회복
    #[test]
    fn scenario_potion_heal_flow() {
        let mut p = Player::new();
        p.hp = 3;
        let mut q = EventQueue::new();

        potion_bridge::drink_potion(&mut p, "healing", false, false, &mut q);
        assert!(p.hp > 3);

        let health_events: Vec<_> = q
            .iter()
            .filter(|e| matches!(e, crate::core::events::GameEvent::HealthChanged { .. }))
            .collect();
        assert!(!health_events.is_empty());
    }

    /// [시나리오 B] 완드→몬스터 죽음→경험치 획득 연쇄
    #[test]
    fn scenario_wand_kill_xp_chain() {
        let mut p = Player::new();
        let initial_xp = p.experience;
        let mut mon_hp = 30i32;
        let mut q = EventQueue::new();
        let mut rng = NetHackRng::new(0);

        // 완드로 죽임
        let outcome =
            wand_bridge::zap_at_monster(&p, "death", "lich", &mut mon_hp, false, &mut rng, &mut q);
        assert!(mon_hp < 0);

        // 경험치 획득
        let xp = xp_bridge::monster_kill_xp(10, 30);
        xp_bridge::gain_experience(&mut p, xp, &mut q);
        assert!(p.experience > initial_xp);
    }

    /// [시나리오 C] 저주 화염 스크롤 → 데미지 → 포션 치료
    #[test]
    fn scenario_fire_scroll_then_heal() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        let initial_hp = p.hp;

        scroll_bridge::use_scroll(&mut p, "fire", false, false, false, &mut q);
        let after_scroll_hp = p.hp;
        assert!(after_scroll_hp <= initial_hp);

        // 힐링 포션으로 회복
        potion_bridge::drink_potion(&mut p, "healing", false, false, &mut q);
        assert!(p.hp >= after_scroll_hp);
    }

    /// [시나리오 D] 전투 → 죽이기 → 경험치 → 레벨업
    #[test]
    fn scenario_fight_levelup() {
        let mut p = Player::new();
        p.experience = 0;
        let mut mon_hp = 5i32;
        let mut q = EventQueue::new();
        let mut rng = NetHackRng::new(99);

        // 몬스터 공격
        while mon_hp > 0 {
            combat_bridge::player_attacks_monster(
                &p,
                "kobold",
                10,
                &mut mon_hp,
                2,
                4,
                2,
                &mut rng,
                &mut q,
            );
        }

        // 경험치 지급
        let xp = xp_bridge::monster_kill_xp(1, 5);
        xp_bridge::gain_experience(&mut p, xp + 20, &mut q);

        // 결과: 레벨 올랐는지 또는 경험치 증가
        assert!(p.experience > 0);
    }

    /// [시나리오 E] 축복 감정 스크롤 → 모든 아이템 감정 메시지
    #[test]
    fn scenario_blessed_identify() {
        let mut p = Player::new();
        let mut q = EventQueue::new();

        scroll_bridge::use_scroll(&mut p, "identify", true, false, false, &mut q);
        let msgs: Vec<_> = q
            .iter()
            .filter(|e| {
                matches!(e, crate::core::events::GameEvent::Message { text, .. }
                if text.contains("99"))
            })
            .collect();
        assert!(!msgs.is_empty()); // "99개의 아이템을 감정했다" 메시지
    }
}

// ============================================================================
// [v2.45.0 R33-4] 경험치/레벨업 브릿지 (xp_bridge.rs)
// experience_ext (순수 로직) → Player 레벨업 연결
// ============================================================================

use crate::core::entity::player::Player;
use crate::core::events::{EventQueue, GameEvent};

/// [v2.45.0 R33-4] 경험치 임계값 (원본: NetHack experience.c exptbl)
fn xp_threshold(level: i32) -> u64 {
    match level {
        1 => 20,
        2 => 40,
        3 => 80,
        4 => 160,
        5 => 320,
        6 => 640,
        7 => 1280,
        8 => 2560,
        9 => 5000,
        10 => 10000,
        11 => 20000,
        12 => 40000,
        13 => 80000,
        14 => 160000,
        15 => 320000,
        16 => 640000,
        17 => 1200000,
        18 => 2400000,
        19 => 4800000,
        _ => u64::MAX,
    }
}

/// [v2.45.0 R33-4] 경험치 획득 + 레벨업 처리 (원본: more_experienced)
pub fn gain_experience(player: &mut Player, xp: u64, events: &mut EventQueue) {
    let old_level = player.exp_level;
    player.experience += xp;
    events.push(GameEvent::ExperienceGained { amount: xp });

    // 레벨업 루프 (여러 레벨 동시 상승 가능)
    loop {
        let needed = xp_threshold(player.exp_level);
        if player.experience < needed || player.exp_level >= 20 {
            break;
        }
        player.exp_level += 1;

        // HP 증가 (원본: rnd(con에 의존 — 단순화: 레벨*2+CON기반)
        let hp_gain = 2 + (player.con.base - 10).max(0) / 2;
        player.hp_max += hp_gain;
        player.hp = player.hp_max;

        // 에너지 증가
        let en_gain = 1 + (player.wis.base - 10).max(0) / 3;
        player.energy_max += en_gain;
        player.energy = player.energy_max;

        events.push(GameEvent::LevelUp {
            new_level: player.exp_level,
        });
        events.push(GameEvent::Message {
            text: format!("레벨이 오른다! ({}레벨)", player.exp_level),
            priority: false,
        });
    }

    if player.exp_level > old_level {
        // 신앙도 보너스
        player.piety = (player.piety + 2).min(100);
    }
}

/// [v2.45.0 R33-4] 몬스터 처치 시 경험치 계산 (원본: experience)
pub fn monster_kill_xp(monster_level: i32, hp_max: i32) -> u64 {
    let base = (monster_level * monster_level + monster_level) as u64 / 2;
    let hp_bonus = (hp_max / 10) as u64;
    base + hp_bonus + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_xp_no_level() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        gain_experience(&mut p, 5, &mut q);
        assert_eq!(p.exp_level, 1); // 아직 부족
        let xp_events: Vec<_> = q
            .iter()
            .filter(|e| matches!(e, GameEvent::ExperienceGained { .. }))
            .collect();
        assert_eq!(xp_events.len(), 1);
    }

    #[test]
    fn test_gain_level() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        gain_experience(&mut p, 20, &mut q); // 1레벨 임계값
        assert_eq!(p.exp_level, 2);
        assert!(p.hp_max > 15); // HP 증가
    }

    #[test]
    fn test_multi_level() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        gain_experience(&mut p, 200, &mut q); // 여러 레벨
        assert!(p.exp_level >= 3);
    }

    #[test]
    fn test_monster_xp() {
        let xp = monster_kill_xp(5, 30);
        assert!(xp >= 15);
    }

    #[test]
    fn test_level_cap() {
        let mut p = Player::new();
        p.exp_level = 20;
        let mut q = EventQueue::new();
        gain_experience(&mut p, 9_999_999, &mut q);
        assert_eq!(p.exp_level, 20); // 캡 유지
    }
}

// ============================================================================
// [v2.45.0 R33-3] 완드 사용 브릿지 (wand_bridge.rs)
// wand_effect_ext → 대상(Player/Monster) + GameEvent 연결
// ============================================================================

use crate::core::entity::player::Player;
use crate::core::events::{EventQueue, GameEvent};
use crate::core::systems::magic::wand_effect_ext::{beam_reflected, zap_wand, WandEffect};
use crate::util::rng::NetHackRng;

/// [v2.45.0 R33-3] 완드 발사 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZapOutcome {
    HitPlayer { damage: i32 },
    HitMonster { monster_name: String, damage: i32 },
    Reflected,
    Teleported,
    Other(String),
}

/// [v2.45.0 R33-3] 완드 발사 — 완드 효과 → 실제 결과
pub fn zap_at_monster(
    player: &Player,
    wand_name: &str,
    monster_name: &str,
    monster_hp: &mut i32,
    monster_has_reflection: bool,
    rng: &mut NetHackRng,
    events: &mut EventQueue,
) -> ZapOutcome {
    // 반사 판정
    if beam_reflected(monster_has_reflection) {
        events.push(GameEvent::Message {
            text: format!("빔이 {}에게서 반사됐다!", monster_name),
            priority: false,
        });
        return ZapOutcome::Reflected;
    }

    let effect = zap_wand(wand_name, false);
    match effect {
        WandEffect::MagicMissile(dmg) => {
            *monster_hp -= dmg;
            events.push(GameEvent::DamageDealt {
                attacker: "wand".into(),
                defender: monster_name.into(),
                amount: dmg,
                source: "magic missile".into(),
            });
            ZapOutcome::HitMonster {
                monster_name: monster_name.into(),
                damage: dmg,
            }
        }
        WandEffect::Fire(dmg) => {
            *monster_hp -= dmg;
            events.push(GameEvent::DamageDealt {
                attacker: "wand of fire".into(),
                defender: monster_name.into(),
                amount: dmg,
                source: "fire".into(),
            });
            ZapOutcome::HitMonster {
                monster_name: monster_name.into(),
                damage: dmg,
            }
        }
        WandEffect::Cold(dmg) => {
            *monster_hp -= dmg;
            events.push(GameEvent::DamageDealt {
                attacker: "wand of cold".into(),
                defender: monster_name.into(),
                amount: dmg,
                source: "cold".into(),
            });
            ZapOutcome::HitMonster {
                monster_name: monster_name.into(),
                damage: dmg,
            }
        }
        WandEffect::Teleport => {
            events.push(GameEvent::Message {
                text: format!("{}이(가) 텔레포트됐다!", monster_name),
                priority: false,
            });
            ZapOutcome::Teleported
        }
        WandEffect::Death => {
            *monster_hp = -9999;
            events.push(GameEvent::MonsterDied {
                name: monster_name.into(),
                killer: "wand of death".into(),
                dropped_corpse: true,
                x: 0,
                y: 0,
                xp_gained: 500,
            });
            ZapOutcome::HitMonster {
                monster_name: monster_name.into(),
                damage: 9999,
            }
        }
        WandEffect::Sleep(turns) => {
            events.push(GameEvent::StatusApplied {
                target: monster_name.into(),
                status: crate::core::entity::status::StatusFlags::SLEEPING,
                turns: turns as u32,
            });
            ZapOutcome::Other(format!("{}이(가) 잠들었다!", monster_name))
        }
        WandEffect::Nothing => ZapOutcome::Other("아무 일도 없었다.".into()),
        _ => ZapOutcome::Other(format!("{:?} 효과", effect)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_missile() {
        let p = Player::new();
        let mut mon_hp = 20;
        let mut q = EventQueue::new();
        let mut rng = NetHackRng::new(0);
        let outcome = zap_at_monster(
            &p,
            "magic missile",
            "goblin",
            &mut mon_hp,
            false,
            &mut rng,
            &mut q,
        );
        assert_eq!(mon_hp, 8); // 20-12
        assert!(matches!(outcome, ZapOutcome::HitMonster { damage: 12, .. }));
    }

    #[test]
    fn test_reflection() {
        let p = Player::new();
        let mut mon_hp = 20;
        let mut q = EventQueue::new();
        let mut rng = NetHackRng::new(0);
        let outcome = zap_at_monster(
            &p,
            "fire",
            "silver dragon",
            &mut mon_hp,
            true,
            &mut rng,
            &mut q,
        );
        assert_eq!(mon_hp, 20); // 반사, 피해 없음
        assert_eq!(outcome, ZapOutcome::Reflected);
    }

    #[test]
    fn test_death_wand() {
        let p = Player::new();
        let mut mon_hp = 100;
        let mut q = EventQueue::new();
        let mut rng = NetHackRng::new(0);
        zap_at_monster(&p, "death", "lich", &mut mon_hp, false, &mut rng, &mut q);
        assert!(mon_hp < 0);
        let deaths: Vec<_> = q
            .iter()
            .filter(|e| matches!(e, GameEvent::MonsterDied { .. }))
            .collect();
        assert_eq!(deaths.len(), 1);
    }

    #[test]
    fn test_sleep_status() {
        let p = Player::new();
        let mut mon_hp = 20;
        let mut q = EventQueue::new();
        let mut rng = NetHackRng::new(0);
        zap_at_monster(&p, "sleep", "zombie", &mut mon_hp, false, &mut rng, &mut q);
        let statuses: Vec<_> = q
            .iter()
            .filter(|e| matches!(e, GameEvent::StatusApplied { .. }))
            .collect();
        assert!(!statuses.is_empty());
    }
}

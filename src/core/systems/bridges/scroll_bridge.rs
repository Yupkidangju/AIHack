// ============================================================================
// [v2.45.0 R33-2] 스크롤 사용 브릿지 (scroll_bridge.rs)
// scroll_effect_ext → Player / Item / Map + GameEvent 연결
// ============================================================================

use crate::core::entity::player::Player;
use crate::core::events::{EventQueue, GameEvent};
use crate::core::systems::item::scroll_effect_ext::{read_scroll, ScrollEffect};

/// [v2.45.0 R33-2] 스크롤 읽기 — 효과를 실제 상태에 반영
pub fn use_scroll(
    player: &mut Player,
    scroll_name: &str,
    blessed: bool,
    cursed: bool,
    confused: bool,
    events: &mut EventQueue,
) {
    let effect = read_scroll(scroll_name, blessed, cursed, confused);
    apply_scroll_effect(player, effect, scroll_name, events);
}

fn apply_scroll_effect(
    player: &mut Player,
    effect: ScrollEffect,
    scroll_name: &str,
    events: &mut EventQueue,
) {
    match effect {
        ScrollEffect::Identify(count) => {
            events.push(GameEvent::Message {
                text: format!("{}개의 아이템을 감정했다.", count),
                priority: false,
            });
        }
        ScrollEffect::Enchant(target) => {
            events.push(GameEvent::Message {
                text: format!("{}이(가) 빛난다!", target),
                priority: false,
            });
        }
        ScrollEffect::RemoveCurse(all) => {
            events.push(GameEvent::Message {
                text: if all {
                    "장비의 저주가 모두 풀렸다.".into()
                } else {
                    "장비의 저주가 풀렸다.".into()
                },
                priority: false,
            });
        }
        ScrollEffect::Teleport(controlled) => {
            // 텔레포트: 위치 변경은 Map 연결 후 처리 (TODO R35)
            events.push(GameEvent::Message {
                text: if controlled {
                    "어디로 텔레포트 하시겠습니까?".into()
                } else {
                    "갑자기 공간이 뒤틀린다!".into()
                },
                priority: false,
            });
        }
        ScrollEffect::CreateMonster(count) => {
            events.push(GameEvent::Message {
                text: format!("{}마리의 몬스터가 소환됐다!", count),
                priority: true,
            });
        }
        ScrollEffect::MagicMapping => {
            events.push(GameEvent::Message {
                text: "이 층의 지도가 눈앞에 펼쳐진다!".into(),
                priority: false,
            });
        }
        ScrollEffect::Punishment => {
            events.push(GameEvent::Message {
                text: "쇠공이 당신의 발목에 채워졌다!".into(),
                priority: true,
            });
        }
        ScrollEffect::Fire(dmg) => {
            if dmg > 0 {
                player.hp -= dmg;
                events.push(GameEvent::DamageDealt {
                    attacker: "fire scroll".into(),
                    defender: "Player".into(),
                    amount: dmg,
                    source: "fire".into(),
                });
            }
        }
        ScrollEffect::Blank => {
            events.push(GameEvent::Message {
                text: "이 스크롤은 공백이다.".into(),
                priority: false,
            });
        }
        ScrollEffect::Nothing | ScrollEffect::Confuse => {
            events.push(GameEvent::Message {
                text: "아무 일도 일어나지 않았다.".into(),
                priority: false,
            });
        }
        ScrollEffect::Genocide(by_kind) => {
            events.push(GameEvent::Message {
                text: if by_kind {
                    "어떤 종류의 몬스터를 말살하겠습니까?".into()
                } else {
                    "어떤 클래스의 몬스터를 말살하겠습니까?".into()
                },
                priority: false,
            });
        }
        ScrollEffect::Destroy(what) => {
            events.push(GameEvent::Message {
                text: format!("{}이(가) 파괴됐다!", what),
                priority: true,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_scroll() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        use_scroll(&mut p, "identify", false, false, false, &mut q);
        let msgs: Vec<_> = q
            .iter()
            .filter(|e| matches!(e, GameEvent::Message { .. }))
            .collect();
        assert!(!msgs.is_empty());
    }

    #[test]
    fn test_fire_scroll_damages() {
        let mut p = Player::new();
        let hp_before = p.hp;
        let mut q = EventQueue::new();
        use_scroll(&mut p, "fire", false, false, false, &mut q);
        assert!(p.hp < hp_before);
    }

    #[test]
    fn test_fire_scroll_blessed_no_damage() {
        let mut p = Player::new();
        let hp_before = p.hp;
        let mut q = EventQueue::new();
        use_scroll(&mut p, "fire", true, false, false, &mut q);
        assert_eq!(p.hp, hp_before); // 축복 화염 = 0 데미지
    }

    #[test]
    fn test_remove_curse_all() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        use_scroll(&mut p, "remove curse", true, false, false, &mut q);
        let msg = q.iter().find(|e| {
            matches!(e, GameEvent::Message { text, .. }
            if text.contains("모두"))
        });
        assert!(msg.is_some());
    }

    #[test]
    fn test_confused_nothing() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        use_scroll(&mut p, "identify", false, false, true, &mut q); // 혼란 시
        let msg = q.iter().find(|e| {
            matches!(e, GameEvent::Message { text, .. }
            if text.contains("아무"))
        });
        assert!(msg.is_some());
    }
}

// ============================================================================
// [v2.22.0 R34-P2-2] 지형 이벤트 브릿지 (terrain_bridge.rs)
// trap_ext + trap_ext2 + fountain_ext → TurnEngine LevelEvents 페이즈 연결
// ============================================================================

use crate::core::events::{EventQueue, GameEvent};
use crate::core::systems::turn_engine::TurnContext;
use crate::core::systems::world::fountain_ext;
use crate::core::systems::world::trap_ext::{self, TrapEscapeResult, TrapType};
use crate::core::systems::world::trap_ext2;
use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 함정 발동 처리
// =============================================================================

/// [v2.22.0 R34-P2-2] 함정 발동 입력
#[derive(Debug, Clone)]
pub struct TrapTriggerInput {
    pub trap_type: TrapType,
    pub player_ac: i32,
    pub has_metal_helmet: bool,
    pub is_levitating: bool,
    pub is_flying: bool,
    pub is_fumbling: bool,
    pub fire_resistant: bool,
    pub sleep_resistant: bool,
    pub poison_resistant: bool,
    pub x: i32,
    pub y: i32,
}

/// [v2.22.0 R34-P2-2] 함정 발동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapTriggerResult {
    /// 피해 발생
    Damage { amount: i32, description: String },
    /// 상태이상 적용
    StatusEffect { effect: String, turns: i32 },
    /// 텔레포트
    Teleport,
    /// 레벨 이동
    LevelTeleport,
    /// 함정 회피
    Avoided,
    /// 소리만 발생
    Noise { message: String },
}

/// [v2.22.0 R34-P2-2] 함정 발동 처리 (순수 함수)
pub fn process_trap_trigger(input: &TrapTriggerInput, rng: &mut NetHackRng) -> TrapTriggerResult {
    match input.trap_type {
        // 화살/다트/바위 함정 → 데미지
        TrapType::Arrow => {
            let damage = rng.rnd(6);
            TrapTriggerResult::Damage {
                amount: damage,
                description: "화살 함정에 맞았다!".to_string(),
            }
        }
        TrapType::Dart => {
            let damage = rng.rnd(4);
            let poisoned = trap_ext::dart_poison_chance(rng);
            if poisoned && !input.poison_resistant {
                TrapTriggerResult::StatusEffect {
                    effect: "Poisoned".to_string(),
                    turns: 20,
                }
            } else {
                TrapTriggerResult::Damage {
                    amount: damage,
                    description: "다트에 맞았다!".to_string(),
                }
            }
        }
        TrapType::Rock => {
            let damage = trap_ext::rock_trap_damage(input.has_metal_helmet, rng);
            TrapTriggerResult::Damage {
                amount: damage,
                description: if input.has_metal_helmet {
                    "바위가 헬멧에 맞았다!".to_string()
                } else {
                    "바위가 머리에 맞았다!".to_string()
                },
            }
        }
        // 수면 가스
        TrapType::SleepGas => {
            if input.sleep_resistant {
                TrapTriggerResult::Avoided
            } else {
                let turns = trap_ext::sleep_gas_duration(rng);
                TrapTriggerResult::StatusEffect {
                    effect: "Sleeping".to_string(),
                    turns,
                }
            }
        }
        // 곰 함정
        TrapType::BearTrap => {
            let damage = trap_ext::bear_trap_damage(rng);
            TrapTriggerResult::Damage {
                amount: damage,
                description: "곰 함정에 걸렸다!".to_string(),
            }
        }
        // 구덩이 (일반/가시)
        TrapType::Pit | TrapType::SpikedPit => {
            if input.is_levitating || input.is_flying {
                TrapTriggerResult::Avoided
            } else {
                let is_spiked = input.trap_type == TrapType::SpikedPit;
                let damage = trap_ext::pit_damage(is_spiked, rng);
                TrapTriggerResult::Damage {
                    amount: damage,
                    description: if is_spiked {
                        "가시 구덩이에 빠졌다!".to_string()
                    } else {
                        "구덩이에 빠졌다!".to_string()
                    },
                }
            }
        }
        // 화염 함정
        TrapType::FireTrap => {
            let damage = trap_ext2::calc_fire_trap_damage(
                false, // 수중 아님
                input.fire_resistant,
                trap_ext2::GolemFireType::Other,
                100, // 기본 max_hp
                rng,
            );
            if damage > 0 {
                TrapTriggerResult::Damage {
                    amount: damage,
                    description: "화염이 솟아올랐다!".to_string(),
                }
            } else {
                TrapTriggerResult::Avoided
            }
        }
        // 삐걱대는 판자
        TrapType::SqueakyBoard => TrapTriggerResult::Noise {
            message: "삐걱! 발 밑에서 소음이 났다.".to_string(),
        },
        // 텔레포터
        TrapType::Teleporter => TrapTriggerResult::Teleport,
        TrapType::LevelTeleporter => TrapTriggerResult::LevelTeleport,
        // 녹 함정
        TrapType::RustTrap => TrapTriggerResult::Noise {
            message: "녹 가루가 뿌려졌다!".to_string(),
        },
        // 기타
        _ => TrapTriggerResult::Avoided,
    }
}

// =============================================================================
// [2] 지형 이벤트를 이벤트 큐에 적용
// =============================================================================

/// [v2.22.0 R34-P2-2] 함정 발동 결과를 이벤트 큐에 적용
pub fn apply_trap_result(
    result: &TrapTriggerResult,
    trap_type: TrapType,
    x: i32,
    y: i32,
    ctx: &mut TurnContext,
) {
    match result {
        TrapTriggerResult::Damage {
            amount,
            description,
        } => {
            ctx.player.hp -= amount;
            ctx.event_queue.push(GameEvent::TrapTriggered {
                trap_type: format!("{:?}", trap_type),
                x,
                y,
            });
            ctx.event_queue.push(GameEvent::DamageDealt {
                attacker: format!("{:?}", trap_type),
                defender: "Player".to_string(),
                amount: *amount,
                source: "trap".to_string(),
            });
            ctx.event_queue.push(GameEvent::Message {
                text: description.clone(),
                priority: true,
            });
        }
        TrapTriggerResult::StatusEffect { effect, turns } => {
            ctx.event_queue.push(GameEvent::TrapTriggered {
                trap_type: format!("{:?}", trap_type),
                x,
                y,
            });
            ctx.event_queue.push(GameEvent::Message {
                text: format!("{:?} 함정! {} 효과 {}턴.", trap_type, effect, turns),
                priority: true,
            });
        }
        TrapTriggerResult::Teleport => {
            ctx.event_queue.push(GameEvent::TrapTriggered {
                trap_type: "Teleporter".to_string(),
                x,
                y,
            });
            ctx.event_queue.push(GameEvent::Message {
                text: "텔레포트 함정이 발동했다!".to_string(),
                priority: true,
            });
        }
        TrapTriggerResult::LevelTeleport => {
            ctx.event_queue.push(GameEvent::TrapTriggered {
                trap_type: "LevelTeleporter".to_string(),
                x,
                y,
            });
            ctx.event_queue.push(GameEvent::Message {
                text: "레벨 텔레포트 함정!".to_string(),
                priority: true,
            });
        }
        TrapTriggerResult::Noise { message } => {
            ctx.event_queue.push(GameEvent::TrapTriggered {
                trap_type: format!("{:?}", trap_type),
                x,
                y,
            });
            ctx.event_queue.push(GameEvent::Message {
                text: message.clone(),
                priority: false,
            });
        }
        TrapTriggerResult::Avoided => {
            // 회피 → 이벤트 없음
        }
    }
}

// =============================================================================
// [3] LevelEvents 페이즈 진입점
// =============================================================================

/// [v2.22.0 R34-P2-2] LevelEvents 페이즈 처리
/// 현재 플레이어 위치의 함정/지형 효과 처리
pub fn process_level_events(
    ctx: &mut TurnContext,
    current_trap: Option<TrapType>,
    trap_x: i32,
    trap_y: i32,
    rng: &mut NetHackRng,
) {
    // 함정이 있는 경우에만 처리
    if let Some(trap_type) = current_trap {
        let input = TrapTriggerInput {
            trap_type,
            player_ac: ctx.player.ac,
            has_metal_helmet: false, // TODO: 장비 조회
            is_levitating: false,    // TODO: 상태 조회
            is_flying: false,
            is_fumbling: false,
            fire_resistant: false,
            sleep_resistant: false,
            poison_resistant: false,
            x: trap_x,
            y: trap_y,
        };

        let result = process_trap_trigger(&input, rng);
        apply_trap_result(&result, trap_type, trap_x, trap_y, ctx);
    }
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::player::Player;
    use crate::core::events::EventQueue;

    fn default_input(trap_type: TrapType) -> TrapTriggerInput {
        TrapTriggerInput {
            trap_type,
            player_ac: 10,
            has_metal_helmet: false,
            is_levitating: false,
            is_flying: false,
            is_fumbling: false,
            fire_resistant: false,
            sleep_resistant: false,
            poison_resistant: false,
            x: 5,
            y: 5,
        }
    }

    #[test]
    fn test_arrow_trap_damage() {
        let input = default_input(TrapType::Arrow);
        let mut rng = NetHackRng::new(42);
        let result = process_trap_trigger(&input, &mut rng);
        match result {
            TrapTriggerResult::Damage { amount, .. } => {
                assert!(amount >= 1 && amount <= 6);
            }
            _ => panic!("화살 함정은 데미지를 줘야 함"),
        }
    }

    #[test]
    fn test_pit_avoided_when_flying() {
        let mut input = default_input(TrapType::Pit);
        input.is_flying = true;
        let mut rng = NetHackRng::new(0);
        let result = process_trap_trigger(&input, &mut rng);
        assert_eq!(result, TrapTriggerResult::Avoided);
    }

    #[test]
    fn test_sleep_gas_resisted() {
        let mut input = default_input(TrapType::SleepGas);
        input.sleep_resistant = true;
        let mut rng = NetHackRng::new(0);
        let result = process_trap_trigger(&input, &mut rng);
        assert_eq!(result, TrapTriggerResult::Avoided);
    }

    #[test]
    fn test_apply_damage_reduces_hp() {
        let mut p = Player::new();
        let initial_hp = p.hp;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };

        let result = TrapTriggerResult::Damage {
            amount: 5,
            description: "테스트".to_string(),
        };
        apply_trap_result(&result, TrapType::Arrow, 3, 3, &mut ctx);

        assert_eq!(p.hp, initial_hp - 5);
        assert!(!q.is_empty());
    }

    #[test]
    fn test_teleport_trap_emits_event() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };

        let result = TrapTriggerResult::Teleport;
        apply_trap_result(&result, TrapType::Teleporter, 7, 7, &mut ctx);

        let has_trap_event = q.iter().any(|e| {
            matches!(e, GameEvent::TrapTriggered { trap_type, .. } if trap_type == "Teleporter")
        });
        assert!(has_trap_event);
    }

    #[test]
    fn test_process_level_events_with_trap() {
        let mut p = Player::new();
        let initial_hp = p.hp;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let mut rng = NetHackRng::new(42);

        process_level_events(&mut ctx, Some(TrapType::Arrow), 5, 5, &mut rng);

        // 화살 함정 데미지 발생
        assert!(p.hp < initial_hp);
    }

    #[test]
    fn test_process_level_events_no_trap() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let mut rng = NetHackRng::new(0);

        process_level_events(&mut ctx, None, 0, 0, &mut rng);

        assert!(q.is_empty());
    }
}

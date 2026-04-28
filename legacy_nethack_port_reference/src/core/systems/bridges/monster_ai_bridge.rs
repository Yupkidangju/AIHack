// ============================================================================
// [v2.22.0 R34-P2-4] 몬스터 AI 브릿지 (monster_ai_bridge.rs)
// monmove_ext + dogmove_ext + mcastu_ext + muse_ext + dog_ext
// → TurnEngine MonsterActions 페이즈 연결
// ============================================================================

use crate::core::events::GameEvent;
use crate::core::systems::ai::{dogmove_ext, mcastu_ext, monmove_ext, muse_ext};
use crate::core::systems::turn_engine::TurnContext;
use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 상태 (Bridge 전용 경량 구조체)
// =============================================================================

/// [v2.22.0 R34-P2-4] 몬스터 턴 처리용 상태
#[derive(Debug, Clone)]
pub struct MonsterState {
    pub id: u32,
    pub name: String,
    pub hp: i32,
    pub hp_max: i32,
    pub level: i32,
    pub x: i32,
    pub y: i32,
    pub is_tame: bool,
    pub is_peaceful: bool,
    pub is_fleeing: bool,
    pub flee_timer: i32,
    pub is_confused: bool,
    pub is_stunned: bool,
    pub is_sleeping: bool,
    pub regenerates: bool,
    pub can_cast: bool,
    pub intelligence: i32,
    pub spec_used: i32,
    pub eating: i32,
}

/// [v2.22.0 R34-P2-4] 몬스터 행동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonsterAction {
    /// 이동
    Move { dx: i32, dy: i32 },
    /// 공격 (플레이어 대상)
    AttackPlayer { damage: i32, source: String },
    /// 주문 시전
    CastSpell { spell: mcastu_ext::MonsterSpell },
    /// 아이템 사용
    UseItem { item: muse_ext::MonsterUseItem },
    /// 도주
    Flee,
    /// 대기 (행동 없음)
    Wait,
    /// 혼란/기절 회복
    RecoverStatus { recovered: String },
}

// =============================================================================
// [2] 개별 몬스터 턴 처리
// =============================================================================

/// [v2.22.0 R34-P2-4] 단일 몬스터 턴 처리
pub fn process_single_monster(
    monster: &mut MonsterState,
    player_x: i32,
    player_y: i32,
    turn_number: u64,
    rng: &mut NetHackRng,
) -> MonsterAction {
    // [1] HP 재생
    let regen = monmove_ext::calc_mon_regen(
        monster.hp,
        monster.hp_max,
        monster.spec_used,
        monster.eating,
        turn_number,
        monster.regenerates,
        false,
    );
    monster.hp = regen.hp;
    monster.spec_used = regen.spec_used;

    // [2] 혼란/기절 회복
    if monster.is_confused && monmove_ext::should_recover_confusion(rng) {
        monster.is_confused = false;
        return MonsterAction::RecoverStatus {
            recovered: "confusion".to_string(),
        };
    }
    if monster.is_stunned && monmove_ext::should_recover_stun(rng) {
        monster.is_stunned = false;
        return MonsterAction::RecoverStatus {
            recovered: "stun".to_string(),
        };
    }

    // [3] 수면 중이면 각성 판정
    if monster.is_sleeping {
        let dist_sq = (monster.x - player_x).pow(2) + (monster.y - player_y).pow(2);
        let disturb_input = monmove_ext::DisturbInput {
            player_can_see: true,
            dist_sq,
            player_stealthy: false,
            is_ettin: false,
            is_deep_sleeper: false,
            aggravate: false,
            is_dog_or_human: false,
            is_mimicking: false,
        };
        if monmove_ext::should_disturb(&disturb_input, rng) {
            monster.is_sleeping = false;
        } else {
            return MonsterAction::Wait;
        }
    }

    // [4] 도주 판정
    if monster.is_fleeing {
        if monmove_ext::should_regain_courage(
            true,
            monster.flee_timer,
            monster.hp,
            monster.hp_max,
            rng,
        ) {
            monster.is_fleeing = false;
        } else {
            return MonsterAction::Flee;
        }
    }

    // [5] 거리 계산
    let dist = ((monster.x - player_x).abs() + (monster.y - player_y).abs()) as i32;

    // [6] 주문 시전 (지능 1+ 이상이고 주문 가능)
    if monster.can_cast && monster.intelligence >= 1 {
        let hp_ratio = if monster.hp_max > 0 {
            monster.hp as f64 / monster.hp_max as f64
        } else {
            1.0
        };
        let cast_input = mcastu_ext::CastInput {
            caster_level: monster.level,
            caster_hp_ratio: hp_ratio,
            distance_to_target: dist,
            target_has_mr: false,         // TODO: 플레이어 MR 확인
            target_has_reflection: false, // TODO: 반사 확인
            is_covetous: false,           // TODO: 보물 탐욕 확인
        };
        let spell = mcastu_ext::decide_spell(&cast_input, rng);
        return MonsterAction::CastSpell { spell };
    }

    // [7] 근접 공격 가능 → 공격
    if dist <= 1 && !monster.is_peaceful {
        let damage = rng.rn1(monster.level, 1);
        return MonsterAction::AttackPlayer {
            damage,
            source: monster.name.clone(),
        };
    }

    // [8] 플레이어 방향으로 이동
    let dx = (player_x - monster.x).signum();
    let dy = (player_y - monster.y).signum();
    MonsterAction::Move { dx, dy }
}

// =============================================================================
// [3] 전체 몬스터 턴 + 이벤트 발행
// =============================================================================

/// [v2.22.0 R34-P2-4] 전체 몬스터 턴 처리 (TurnEngine MonsterActions 페이즈)
pub fn process_all_monsters(
    ctx: &mut TurnContext,
    monsters: &mut [MonsterState],
    rng: &mut NetHackRng,
) {
    let player_x = ctx.player.x as i32;
    let player_y = ctx.player.y as i32;

    for mon in monsters.iter_mut() {
        let action = process_single_monster(mon, player_x, player_y, ctx.turn_number, rng);

        match &action {
            MonsterAction::AttackPlayer { damage, source } => {
                ctx.player.hp -= damage;
                ctx.event_queue.push(GameEvent::DamageDealt {
                    attacker: source.clone(),
                    defender: "Player".to_string(),
                    amount: *damage,
                    source: "melee".to_string(),
                });
            }
            MonsterAction::CastSpell { spell } => {
                // 공격 주문 → 플레이어에게 피해
                match spell {
                    mcastu_ext::MonsterSpell::MagicMissile { damage }
                    | mcastu_ext::MonsterSpell::FireBolt { damage }
                    | mcastu_ext::MonsterSpell::IceBolt { damage }
                    | mcastu_ext::MonsterSpell::LightningBolt { damage }
                    | mcastu_ext::MonsterSpell::AcidSplash { damage } => {
                        ctx.player.hp -= damage;
                        ctx.event_queue.push(GameEvent::DamageDealt {
                            attacker: mon.name.clone(),
                            defender: "Player".to_string(),
                            amount: *damage,
                            source: format!("{:?}", spell),
                        });
                    }
                    mcastu_ext::MonsterSpell::Confuse { turns } => {
                        ctx.player.status_bundle.make_confused(*turns as u32);
                        ctx.event_queue.push(GameEvent::Message {
                            text: format!("{}의 주문에 혼란되었다! ({}턴)", mon.name, turns),
                            priority: true,
                        });
                    }
                    mcastu_ext::MonsterSpell::Blind { turns } => {
                        ctx.player.status_bundle.make_blinded(*turns as u32);
                        ctx.event_queue.push(GameEvent::Message {
                            text: format!("{}의 주문에 실명당했다! ({}턴)", mon.name, turns),
                            priority: true,
                        });
                    }
                    mcastu_ext::MonsterSpell::DrainLife { amount } => {
                        ctx.player.hp -= amount;
                        ctx.event_queue.push(GameEvent::Message {
                            text: format!("{}이(가) 생명력을 흡수했다! (-{})", mon.name, amount),
                            priority: true,
                        });
                    }
                    _ => {
                        // 방어/소환/저주 주문 → 메시지만
                        ctx.event_queue.push(GameEvent::Message {
                            text: format!("{}이(가) {:?}를 시전했다.", mon.name, spell),
                            priority: false,
                        });
                    }
                }
            }
            MonsterAction::Move { dx, dy } => {
                mon.x += dx;
                mon.y += dy;
            }
            MonsterAction::Flee => {
                // 도주 이동 (플레이어 반대 방향)
                let dx = -(player_x - mon.x).signum();
                let dy = -(player_y - mon.y).signum();
                mon.x += dx;
                mon.y += dy;
            }
            MonsterAction::RecoverStatus { recovered } => {
                ctx.event_queue.push(GameEvent::Message {
                    text: format!("{}이(가) {}에서 회복했다.", mon.name, recovered),
                    priority: false,
                });
            }
            MonsterAction::Wait => {}
            MonsterAction::UseItem { item } => {
                ctx.event_queue.push(GameEvent::Message {
                    text: format!("{}이(가) {:?}를 사용했다.", mon.name, item),
                    priority: false,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::player::Player;
    use crate::core::events::EventQueue;

    fn test_monster() -> MonsterState {
        MonsterState {
            id: 1,
            name: "Goblin".to_string(),
            hp: 10,
            hp_max: 10,
            level: 3,
            x: 5,
            y: 5,
            is_tame: false,
            is_peaceful: false,
            is_fleeing: false,
            flee_timer: 0,
            is_confused: false,
            is_stunned: false,
            is_sleeping: false,
            regenerates: false,
            can_cast: false,
            intelligence: 1,
            spec_used: 0,
            eating: 0,
        }
    }

    #[test]
    fn test_melee_attack() {
        let mut mon = test_monster();
        mon.x = 1;
        mon.y = 0; // 플레이어(0,0)에 인접
        let mut rng = NetHackRng::new(42);
        let action = process_single_monster(&mut mon, 0, 0, 1, &mut rng);
        assert!(matches!(action, MonsterAction::AttackPlayer { .. }));
    }

    #[test]
    fn test_move_toward_player() {
        let mut mon = test_monster();
        mon.x = 10;
        mon.y = 10;
        let mut rng = NetHackRng::new(0);
        let action = process_single_monster(&mut mon, 0, 0, 1, &mut rng);
        assert!(matches!(action, MonsterAction::Move { dx: -1, dy: -1 }));
    }

    #[test]
    fn test_fleeing_monster() {
        let mut mon = test_monster();
        mon.is_fleeing = true;
        mon.flee_timer = 10;
        let mut rng = NetHackRng::new(0);
        let action = process_single_monster(&mut mon, 0, 0, 1, &mut rng);
        assert!(matches!(action, MonsterAction::Flee));
    }

    #[test]
    fn test_sleeping_monster() {
        let mut mon = test_monster();
        mon.is_sleeping = true;
        mon.x = 50;
        mon.y = 50; // 먼 거리
        let mut rng = NetHackRng::new(0);
        let action = process_single_monster(&mut mon, 0, 0, 1, &mut rng);
        // 먼 거리에서는 깨지 않음 → Wait
        assert!(matches!(action, MonsterAction::Wait));
    }

    #[test]
    fn test_caster_spell() {
        let mut mon = test_monster();
        mon.can_cast = true;
        mon.intelligence = 3;
        mon.level = 10;
        mon.x = 5;
        let mut rng = NetHackRng::new(42);
        let action = process_single_monster(&mut mon, 0, 0, 1, &mut rng);
        assert!(matches!(action, MonsterAction::CastSpell { .. }));
    }

    #[test]
    fn test_process_all_attack_damage() {
        let mut p = Player::new();
        let initial_hp = p.hp;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let mut rng = NetHackRng::new(42);
        let mut monsters = vec![{
            let mut m = test_monster();
            m.x = 1;
            m.y = 0; // 인접
            m
        }];

        process_all_monsters(&mut ctx, &mut monsters, &mut rng);

        assert!(p.hp < initial_hp);
        assert!(!q.is_empty());
    }
}

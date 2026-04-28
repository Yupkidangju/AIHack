// ============================================================================
// [v2.29.0 Phase 93-1] 행동 시스템 확장 (hack_phase93_ext.rs)
// 원본: NetHack 3.6.7 src/hack.c L500-2500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 이동 결과 판정 — move_result (hack.c L500-1000)
// =============================================================================

/// [v2.29.0 93-1] 이동 방향
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
    Up,
    Down,
}

/// [v2.29.0 93-1] 이동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveResult {
    /// 정상 이동
    Success {
        new_x: i32,
        new_y: i32,
        turns_used: i32,
    },
    /// 벽/장애물 충돌
    Blocked { reason: String },
    /// 닫힌 문 충돌
    DoorBlocked { x: i32, y: i32, is_locked: bool },
    /// 몬스터와 충돌 (전투 전환)
    MonsterEncounter {
        monster_name: String,
        x: i32,
        y: i32,
    },
    /// 함정 발동
    TrapTriggered { trap_name: String, x: i32, y: i32 },
    /// 수중/용암 진입
    HazardEntry { hazard_type: String, x: i32, y: i32 },
    /// 계단 이용
    StairsUse { direction: String },
    /// 포탈 진입
    PortalEntry { destination: String },
    /// 구속 상태 (곰함정/거미줄)
    Stuck { reason: String },
    /// 이동 불가(마비/수면)
    Paralyzed,
}

/// [v2.29.0 93-1] 이동 입력
#[derive(Debug, Clone)]
pub struct MoveInput {
    pub current_x: i32,
    pub current_y: i32,
    pub direction: MoveDirection,
    pub is_stuck: bool,
    pub is_paralyzed: bool,
    pub is_levitating: bool,
    pub is_swimming: bool,
    pub speed_bonus: i32,
}

/// [v2.29.0 93-1] 이동 결과 판정
/// 원본: hack.c domove()
pub fn resolve_move(
    input: &MoveInput,
    target_x: i32,
    target_y: i32,
    is_walkable: bool,
    has_door: bool,
    door_locked: bool,
    has_monster: Option<&str>,
    has_trap: Option<&str>,
    is_water: bool,
    is_lava: bool,
    is_stairs: bool,
    stairs_dir: &str,
) -> MoveResult {
    // 마비/수면 상태
    if input.is_paralyzed {
        return MoveResult::Paralyzed;
    }

    // 구속 상태
    if input.is_stuck {
        return MoveResult::Stuck {
            reason: "움직일 수 없다!".to_string(),
        };
    }

    // 몬스터 충돌
    if let Some(monster) = has_monster {
        return MoveResult::MonsterEncounter {
            monster_name: monster.to_string(),
            x: target_x,
            y: target_y,
        };
    }

    // 벽/장애물
    if !is_walkable && !has_door {
        return MoveResult::Blocked {
            reason: "벽에 부딪혔다.".to_string(),
        };
    }

    // 닫힌 문
    if has_door {
        return MoveResult::DoorBlocked {
            x: target_x,
            y: target_y,
            is_locked: door_locked,
        };
    }

    // 계단
    if is_stairs {
        return MoveResult::StairsUse {
            direction: stairs_dir.to_string(),
        };
    }

    // 위험 지형
    if is_water && !input.is_levitating && !input.is_swimming {
        return MoveResult::HazardEntry {
            hazard_type: "물".to_string(),
            x: target_x,
            y: target_y,
        };
    }
    if is_lava && !input.is_levitating {
        return MoveResult::HazardEntry {
            hazard_type: "용암".to_string(),
            x: target_x,
            y: target_y,
        };
    }

    // 함정
    if let Some(trap) = has_trap {
        return MoveResult::TrapTriggered {
            trap_name: trap.to_string(),
            x: target_x,
            y: target_y,
        };
    }

    // 정상 이동
    MoveResult::Success {
        new_x: target_x,
        new_y: target_y,
        turns_used: 1,
    }
}

// =============================================================================
// [2] 문 상호작용 — door_action (hack.c L1000-1300)
// =============================================================================

/// [v2.29.0 93-1] 문 행동
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorAction {
    Open,
    Close,
    Kick,
    PickLock,
    ForceLock,
}

/// [v2.29.0 93-1] 문 행동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoorActionResult {
    Opened,
    Closed,
    Locked,
    Broken { debris: bool },
    Failed { reason: String },
    LockPicked,
    Jammed,
}

/// [v2.29.0 93-1] 문 행동 해결
/// 원본: hack.c kickdoor() + doopen() + doclose()
pub fn resolve_door_action(
    action: DoorAction,
    is_locked: bool,
    is_trapped: bool,
    player_str: i32,
    player_dex: i32,
    has_key: bool,
    has_lockpick: bool,
    rng: &mut NetHackRng,
) -> DoorActionResult {
    match action {
        DoorAction::Open => {
            if is_locked {
                DoorActionResult::Failed {
                    reason: "문이 잠겨 있다.".to_string(),
                }
            } else {
                DoorActionResult::Opened
            }
        }
        DoorAction::Close => DoorActionResult::Closed,
        DoorAction::Kick => {
            // STR 기반 성공 확률
            let success_chance = player_str * 3 + rng.rn2(20);
            if success_chance > 30 {
                DoorActionResult::Broken {
                    debris: rng.rn2(3) == 0,
                }
            } else {
                DoorActionResult::Jammed
            }
        }
        DoorAction::PickLock => {
            if !has_lockpick && !has_key {
                return DoorActionResult::Failed {
                    reason: "열쇄나 도구가 없다.".to_string(),
                };
            }
            // DEX 기반 성공 확률
            let pick_chance = player_dex * 3 + if has_key { 30 } else { 10 } + rng.rn2(20);
            if pick_chance > 40 {
                DoorActionResult::LockPicked
            } else {
                DoorActionResult::Failed {
                    reason: "자물쇠를 따는데 실패했다.".to_string(),
                }
            }
        }
        DoorAction::ForceLock => {
            let force_chance = player_str * 4 + rng.rn2(20);
            if force_chance > 45 {
                DoorActionResult::Broken { debris: true }
            } else {
                DoorActionResult::Failed {
                    reason: "문이 꿈쩍도 하지 않는다.".to_string(),
                }
            }
        }
    }
}

// =============================================================================
// [3] 탐색 행동 — search (hack.c L1500-1700)
// =============================================================================

/// [v2.29.0 93-1] 탐색 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchResult {
    FoundTrap { x: i32, y: i32, trap_name: String },
    FoundDoor { x: i32, y: i32 },
    FoundPassage { x: i32, y: i32 },
    NothingFound,
}

/// [v2.29.0 93-1] 주변 탐색
pub fn search_surroundings(
    player_x: i32,
    player_y: i32,
    player_perception: i32,
    hidden_items: &[(i32, i32, &str)], // (x, y, type)
    rng: &mut NetHackRng,
) -> Vec<SearchResult> {
    let mut results = Vec::new();

    for &(hx, hy, htype) in hidden_items {
        // 인접한 것만 탐색
        let dx = (hx - player_x).abs();
        let dy = (hy - player_y).abs();
        if dx > 1 || dy > 1 {
            continue;
        }

        // 탐색 확률: 1/5 + perception 보너스
        let base_chance = 20 + player_perception * 3;
        if rng.rn2(100) < base_chance {
            let result = match htype {
                "trap" => SearchResult::FoundTrap {
                    x: hx,
                    y: hy,
                    trap_name: "함정".to_string(),
                },
                "door" => SearchResult::FoundDoor { x: hx, y: hy },
                "passage" => SearchResult::FoundPassage { x: hx, y: hy },
                _ => continue,
            };
            results.push(result);
        }
    }

    if results.is_empty() {
        results.push(SearchResult::NothingFound);
    }
    results
}

// =============================================================================
// [4] 대기/쉬기 — rest (hack.c L1700-1900)
// =============================================================================

/// [v2.29.0 93-1] 대기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestResult {
    pub hp_recovered: i32,
    pub energy_recovered: i32,
    pub hunger_cost: i32,
    pub turns_used: i32,
    pub interrupted: bool,
    pub interrupt_reason: Option<String>,
}

/// [v2.29.0 93-1] 대기/쉬기 1턴
pub fn rest_one_turn(
    player_level: i32,
    current_hp: i32,
    max_hp: i32,
    current_energy: i32,
    max_energy: i32,
    con_bonus: i32,
    nearby_monster: bool,
    rng: &mut NetHackRng,
) -> RestResult {
    if nearby_monster {
        return RestResult {
            hp_recovered: 0,
            energy_recovered: 0,
            hunger_cost: 1,
            turns_used: 0,
            interrupted: true,
            interrupt_reason: Some("근처에 몬스터가 있다!".to_string()),
        };
    }

    let hp_regen = if current_hp < max_hp {
        let base = if player_level >= 10 { 2 } else { 1 };
        (base + con_bonus / 3).min(max_hp - current_hp)
    } else {
        0
    };

    let en_regen = if current_energy < max_energy {
        let base = if player_level >= 15 { 2 } else { 1 };
        base.min(max_energy - current_energy)
    } else {
        0
    };

    RestResult {
        hp_recovered: hp_regen,
        energy_recovered: en_regen,
        hunger_cost: 1,
        turns_used: 1,
        interrupted: false,
        interrupt_reason: None,
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

    fn base_move() -> MoveInput {
        MoveInput {
            current_x: 10,
            current_y: 10,
            direction: MoveDirection::East,
            is_stuck: false,
            is_paralyzed: false,
            is_levitating: false,
            is_swimming: false,
            speed_bonus: 0,
        }
    }

    #[test]
    fn test_normal_move() {
        let input = base_move();
        let result = resolve_move(
            &input, 11, 10, true, false, false, None, None, false, false, false, "",
        );
        assert!(matches!(
            result,
            MoveResult::Success {
                new_x: 11,
                new_y: 10,
                ..
            }
        ));
    }

    #[test]
    fn test_wall_blocked() {
        let input = base_move();
        let result = resolve_move(
            &input, 11, 10, false, false, false, None, None, false, false, false, "",
        );
        assert!(matches!(result, MoveResult::Blocked { .. }));
    }

    #[test]
    fn test_monster_encounter() {
        let input = base_move();
        let result = resolve_move(
            &input,
            11,
            10,
            true,
            false,
            false,
            Some("고블린"),
            None,
            false,
            false,
            false,
            "",
        );
        assert!(matches!(result, MoveResult::MonsterEncounter { .. }));
    }

    #[test]
    fn test_paralyzed() {
        let mut input = base_move();
        input.is_paralyzed = true;
        let result = resolve_move(
            &input, 11, 10, true, false, false, None, None, false, false, false, "",
        );
        assert!(matches!(result, MoveResult::Paralyzed));
    }

    #[test]
    fn test_door_open() {
        let mut rng = test_rng();
        let result = resolve_door_action(
            DoorAction::Open,
            false,
            false,
            14,
            14,
            false,
            false,
            &mut rng,
        );
        assert!(matches!(result, DoorActionResult::Opened));
    }

    #[test]
    fn test_door_locked() {
        let mut rng = test_rng();
        let result = resolve_door_action(
            DoorAction::Open,
            true,
            false,
            14,
            14,
            false,
            false,
            &mut rng,
        );
        assert!(matches!(result, DoorActionResult::Failed { .. }));
    }

    #[test]
    fn test_door_kick() {
        let mut kicked = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = resolve_door_action(
                DoorAction::Kick,
                true,
                false,
                18,
                14,
                false,
                false,
                &mut rng,
            );
            if matches!(result, DoorActionResult::Broken { .. }) {
                kicked = true;
                break;
            }
        }
        assert!(kicked);
    }

    #[test]
    fn test_search_found() {
        let hidden = vec![(11, 10, "trap"), (9, 10, "door")];
        let mut found = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let results = search_surroundings(10, 10, 15, &hidden, &mut rng);
            if results
                .iter()
                .any(|r| !matches!(r, SearchResult::NothingFound))
            {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_rest_normal() {
        let mut rng = test_rng();
        let result = rest_one_turn(10, 50, 100, 30, 50, 3, false, &mut rng);
        assert!(result.hp_recovered > 0);
        assert!(!result.interrupted);
    }

    #[test]
    fn test_rest_interrupted() {
        let mut rng = test_rng();
        let result = rest_one_turn(10, 50, 100, 30, 50, 3, true, &mut rng);
        assert!(result.interrupted);
        assert_eq!(result.hp_recovered, 0);
    }
}

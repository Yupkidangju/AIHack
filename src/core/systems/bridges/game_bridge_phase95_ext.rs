// ============================================================================
// [v2.31.0 Phase 95-3] 브릿지 통합 확장 (game_bridge_phase95_ext.rs)
// 게임 루프에서 순수 로직 모듈을 호출하는 브릿지 계층 확장
// 순수 결과 패턴 → 상태 적용
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 턴 처리 통합 — turn_process (allmain.c L200-500)
// =============================================================================

/// [v2.31.0 95-3] 턴 처리 단계
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    PlayerInput,
    PlayerAction,
    MonsterActions,
    EnvironmentEffects,
    TimerTick,
    StatusUpdate,
    VisionUpdate,
    HungerCheck,
    TrapCheck,
    EndOfTurn,
}

/// [v2.31.0 95-3] 턴 처리 결과
#[derive(Debug, Clone)]
pub struct TurnResult {
    pub phase_completed: Vec<TurnPhase>,
    pub messages: Vec<String>,
    pub turns_elapsed: i32,
    pub game_over: bool,
    pub game_over_reason: Option<String>,
}

/// [v2.31.0 95-3] 1턴 처리
pub fn process_turn(
    current_turn: i32,
    player_hp: i32,
    player_hunger: i32,
    monster_count: i32,
    active_traps: i32,
    active_timers: i32,
    _rng: &mut NetHackRng,
) -> TurnResult {
    let mut phases = Vec::new();
    let mut messages = Vec::new();
    let mut game_over = false;
    let mut game_over_reason = None;

    // 플레이어 행동
    phases.push(TurnPhase::PlayerAction);

    // 몬스터 행동
    if monster_count > 0 {
        phases.push(TurnPhase::MonsterActions);
        messages.push(format!("{}마리 몬스터 행동", monster_count));
    }

    // 환경 효과
    phases.push(TurnPhase::EnvironmentEffects);

    // 타이머 틱
    if active_timers > 0 {
        phases.push(TurnPhase::TimerTick);
    }

    // 상태 업데이트
    phases.push(TurnPhase::StatusUpdate);

    // 시야 갱신
    phases.push(TurnPhase::VisionUpdate);

    // 배고픔 체크
    phases.push(TurnPhase::HungerCheck);
    if player_hunger <= 0 {
        messages.push("배고파 쓰러졌다!".to_string());
        game_over = true;
        game_over_reason = Some("기아".to_string());
    } else if player_hunger < 50 {
        messages.push("배가 고프다...".to_string());
    }

    // HP 체크
    if player_hp <= 0 {
        game_over = true;
        game_over_reason = Some("사망".to_string());
    }

    // 함정 체크
    if active_traps > 0 {
        phases.push(TurnPhase::TrapCheck);
    }

    phases.push(TurnPhase::EndOfTurn);

    TurnResult {
        phase_completed: phases,
        messages,
        turns_elapsed: 1,
        game_over,
        game_over_reason,
    }
}

// =============================================================================
// [2] 게임 상태 스냅샷 — game_snapshot
// =============================================================================

/// [v2.31.0 95-3] 게임 상태 스냅샷
#[derive(Debug, Clone)]
pub struct GameSnapshot {
    pub turn: i32,
    pub player_name: String,
    pub player_level: i32,
    pub player_hp: i32,
    pub player_hp_max: i32,
    pub player_ac: i32,
    pub player_gold: i64,
    pub dungeon_level: i32,
    pub dungeon_branch: String,
    pub monster_count: i32,
    pub item_count: i32,
    pub score: i64,
}

/// [v2.31.0 95-3] 스냅샷 생성
pub fn create_snapshot(
    turn: i32,
    name: &str,
    level: i32,
    hp: i32,
    hp_max: i32,
    ac: i32,
    gold: i64,
    dlvl: i32,
    branch: &str,
    monsters: i32,
    items: i32,
) -> GameSnapshot {
    let score = (level as i64) * 1000 + gold + (turn as i64 / 10);
    GameSnapshot {
        turn,
        player_name: name.to_string(),
        player_level: level,
        player_hp: hp,
        player_hp_max: hp_max,
        player_ac: ac,
        player_gold: gold,
        dungeon_level: dlvl,
        dungeon_branch: branch.to_string(),
        monster_count: monsters,
        item_count: items,
        score,
    }
}

// =============================================================================
// [3] 세이브/로드 직렬화 — save_format
// =============================================================================

/// [v2.31.0 95-3] 세이브 데이터 헤더
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveHeader {
    pub magic: u32,
    pub version: u32,
    pub player_name: String,
    pub turn: i32,
    pub dungeon_level: i32,
    pub checksum: u32,
}

/// [v2.31.0 95-3] 세이브 헤더 생성
pub fn create_save_header(player_name: &str, turn: i32, dungeon_level: i32) -> SaveHeader {
    // 체크섬: 간단한 해시
    let mut checksum: u32 = 0;
    for b in player_name.bytes() {
        checksum = checksum.wrapping_mul(31).wrapping_add(b as u32);
    }
    checksum = checksum.wrapping_add(turn as u32);
    checksum = checksum.wrapping_add(dungeon_level as u32);

    SaveHeader {
        magic: 0x4E485256, // "NHRV"
        version: 367,      // NetHack 3.6.7
        player_name: player_name.to_string(),
        turn,
        dungeon_level,
        checksum,
    }
}

/// [v2.31.0 95-3] 세이브 헤더 검증
pub fn validate_save_header(header: &SaveHeader) -> bool {
    if header.magic != 0x4E485256 {
        return false;
    }
    if header.version != 367 {
        return false;
    }
    // 체크섬 재계산
    let mut checksum: u32 = 0;
    for b in header.player_name.bytes() {
        checksum = checksum.wrapping_mul(31).wrapping_add(b as u32);
    }
    checksum = checksum.wrapping_add(header.turn as u32);
    checksum = checksum.wrapping_add(header.dungeon_level as u32);

    header.checksum == checksum
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

    #[test]
    fn test_turn_normal() {
        let mut rng = test_rng();
        let result = process_turn(100, 50, 500, 5, 0, 2, &mut rng);
        assert!(!result.game_over);
        assert!(result.phase_completed.contains(&TurnPhase::MonsterActions));
    }

    #[test]
    fn test_turn_death() {
        let mut rng = test_rng();
        let result = process_turn(100, 0, 500, 5, 0, 0, &mut rng);
        assert!(result.game_over);
    }

    #[test]
    fn test_turn_starvation() {
        let mut rng = test_rng();
        let result = process_turn(100, 50, 0, 5, 0, 0, &mut rng);
        assert!(result.game_over);
        assert!(result.game_over_reason.unwrap().contains("기아"));
    }

    #[test]
    fn test_snapshot() {
        let snap = create_snapshot(100, "용사", 10, 50, 100, 5, 1000, 5, "Main", 10, 20);
        assert_eq!(snap.player_name, "용사");
        assert!(snap.score > 0);
    }

    #[test]
    fn test_save_header() {
        let header = create_save_header("용사", 100, 5);
        assert_eq!(header.magic, 0x4E485256);
        assert!(validate_save_header(&header));
    }

    #[test]
    fn test_save_header_invalid() {
        let mut header = create_save_header("용사", 100, 5);
        header.checksum = 0; // 변조
        assert!(!validate_save_header(&header));
    }

    #[test]
    fn test_hunger_warning() {
        let mut rng = test_rng();
        let result = process_turn(100, 50, 30, 0, 0, 0, &mut rng);
        assert!(result.messages.iter().any(|m| m.contains("배가 고프다")));
    }
}

// ============================================================================
// [v2.26.0 R14-5] 게임 루프 확장 (allmain_ext.rs)
// 원본: NetHack 3.6.7 allmain.c (449줄)
// 턴 처리 파이프라인, 속도 시스템, 이벤트 스케줄
// ============================================================================

// =============================================================================
// [1] 턴 단계 (원본: allmain.c moveloop)
// =============================================================================

/// [v2.26.0 R14-5] 턴 처리 단계
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    /// 플레이어 입력 대기
    PlayerInput,
    /// 플레이어 행동 실행
    PlayerAction,
    /// 몬스터 행동 (속도 기반)
    MonsterActions,
    /// 환경 효과 (독, 굶주림, 재생 등)
    EnvironmentEffects,
    /// 턴 후처리 (시야 갱신, UI 업데이트)
    PostTurn,
}

/// [v2.26.0 R14-5] 턴 파이프라인 순서
pub fn turn_phases() -> Vec<TurnPhase> {
    vec![
        TurnPhase::PlayerInput,
        TurnPhase::PlayerAction,
        TurnPhase::MonsterActions,
        TurnPhase::EnvironmentEffects,
        TurnPhase::PostTurn,
    ]
}

// =============================================================================
// [2] 속도 시스템 (원본: allmain.c movemon speed system)
// =============================================================================

/// [v2.26.0 R14-5] 속도 기반 행동 횟수 (원본: NORMAL_SPEED=12)
pub const NORMAL_SPEED: i32 = 12;

/// [v2.26.0 R14-5] 속도→행동 에너지 (원본: movemon 에너지 시스템)
pub fn energy_per_turn(speed: i32) -> i32 {
    // 속도 12 = 턴당 1회 행동 (12 에너지)
    // 속도 24 = 턴당 2회 행동
    // 속도 6 = 2턴마 1회
    speed.max(1)
}

/// [v2.26.0 R14-5] 에너지로 행동 가능 판정
pub fn can_act(energy: i32, cost: i32) -> bool {
    energy >= cost
}

/// [v2.26.0 R14-5] 턴 내 행동 횟수 계산
pub fn actions_per_turn(speed: i32) -> i32 {
    // 속도 12 = 1회, 24 = 2회, 36 = 3회
    (speed / NORMAL_SPEED).max(1)
}

/// [v2.26.0 R14-5] 느린 몬스터 행동 스킵 판정
pub fn should_skip_turn(speed: i32, turn: u64) -> bool {
    if speed >= NORMAL_SPEED {
        return false;
    }
    // 속도 6 = 매 두번째 턴 스킵
    let interval = (NORMAL_SPEED / speed.max(1)) as u64;
    (turn % interval) != 0
}

// =============================================================================
// [3] 환경 이벤트 (원본: allmain.c timeout_process)
// =============================================================================

/// [v2.26.0 R14-5] 주기적 이벤트 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodicEvent {
    /// 배고픔 체크
    HungerCheck,
    /// 재생 (HP/MP)
    Regeneration,
    /// 독 데미지
    PoisonTick,
    /// 슬라임화 진행
    SlimingProgress,
    /// 석화 진행
    StoningProgress,
    /// 텔레포트 (텔레포트 장비)
    RandomTeleport,
    /// 몬스터 생성
    MonsterSpawn,
}

/// [v2.26.0 R14-5] 이벤트 발동 주기 (턴)
pub fn event_interval(event: PeriodicEvent) -> u64 {
    match event {
        PeriodicEvent::HungerCheck => 20,
        PeriodicEvent::Regeneration => 3,
        PeriodicEvent::PoisonTick => 10,
        PeriodicEvent::SlimingProgress => 10,
        PeriodicEvent::StoningProgress => 5,
        PeriodicEvent::RandomTeleport => 100,
        PeriodicEvent::MonsterSpawn => 70,
    }
}

/// [v2.26.0 R14-5] 현재 턴에 발동해야 할 이벤트 목록
pub fn events_due(turn: u64) -> Vec<PeriodicEvent> {
    let all = [
        PeriodicEvent::HungerCheck,
        PeriodicEvent::Regeneration,
        PeriodicEvent::PoisonTick,
        PeriodicEvent::SlimingProgress,
        PeriodicEvent::StoningProgress,
        PeriodicEvent::RandomTeleport,
        PeriodicEvent::MonsterSpawn,
    ];
    all.iter()
        .filter(|e| turn > 0 && turn % event_interval(**e) == 0)
        .cloned()
        .collect()
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_phases() {
        let phases = turn_phases();
        assert_eq!(phases.len(), 5);
        assert_eq!(phases[0], TurnPhase::PlayerInput);
        assert_eq!(phases[4], TurnPhase::PostTurn);
    }

    #[test]
    fn test_actions_per_turn() {
        assert_eq!(actions_per_turn(12), 1);
        assert_eq!(actions_per_turn(24), 2);
        assert_eq!(actions_per_turn(6), 1); // 최소 1
    }

    #[test]
    fn test_skip_turn_slow() {
        // 속도 6 = 2턴에 1번 행동
        assert!(!should_skip_turn(6, 2));
        assert!(should_skip_turn(6, 1));
    }

    #[test]
    fn test_skip_turn_normal() {
        assert!(!should_skip_turn(12, 1));
    }

    #[test]
    fn test_events_due() {
        // 턴 60: 20의 배수(배고픔), 3의 배수(재생), 10의 배수(독)
        let events = events_due(60);
        assert!(events.contains(&PeriodicEvent::HungerCheck));
        assert!(events.contains(&PeriodicEvent::Regeneration));
        assert!(events.contains(&PeriodicEvent::PoisonTick));
    }

    #[test]
    fn test_events_not_due() {
        let events = events_due(7);
        assert!(!events.contains(&PeriodicEvent::HungerCheck));
        assert!(!events.contains(&PeriodicEvent::MonsterSpawn));
    }

    #[test]
    fn test_event_interval() {
        assert_eq!(event_interval(PeriodicEvent::HungerCheck), 20);
        assert_eq!(event_interval(PeriodicEvent::MonsterSpawn), 70);
    }
}

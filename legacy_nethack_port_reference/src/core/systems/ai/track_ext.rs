// ============================================================================
// [v2.28.0 R16-1] 추적 AI (track_ext.rs)
// 원본: NetHack 3.6.7 track.c (140줄) + 몬스터 추적 확장
// 플레이어 이동 이력 기반 추적, 냄새 추적, 청각 추적
// ============================================================================

// =============================================================================
// [1] 이동 이력 (원본: track.c oc_track)
// =============================================================================

/// [v2.28.0 R16-1] 좌표 이력 엔트리
#[derive(Debug, Clone, Copy, Default)]
pub struct TrackPoint {
    pub x: i32,
    pub y: i32,
    pub turn: u64,
}

/// [v2.28.0 R16-1] 플레이어 이동 이력 (원본: track 배열)
#[derive(Debug, Clone)]
pub struct TrackHistory {
    pub points: Vec<TrackPoint>,
    pub max_size: usize,
}

impl TrackHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            points: Vec::new(),
            max_size,
        }
    }

    /// 위치 기록 (원본: oc_to_track)
    pub fn record(&mut self, x: i32, y: i32, turn: u64) {
        self.points.push(TrackPoint { x, y, turn });
        if self.points.len() > self.max_size {
            self.points.remove(0);
        }
    }

    /// 가장 최근 위치
    pub fn last_known(&self) -> Option<TrackPoint> {
        self.points.last().copied()
    }

    /// N턴 이내 위치들
    pub fn recent(&self, within_turns: u64, current_turn: u64) -> Vec<TrackPoint> {
        self.points
            .iter()
            .filter(|p| current_turn.saturating_sub(p.turn) <= within_turns)
            .copied()
            .collect()
    }
}

// =============================================================================
// [2] 냄새 추적 (원본: track.c, dog.c scent)
// =============================================================================

/// [v2.28.0 R16-1] 냄새 강도 계산 (시간 경과에 따라 감소)
pub fn scent_strength(turns_ago: u64) -> f64 {
    if turns_ago == 0 {
        return 1.0;
    }
    (1.0 / (turns_ago as f64 + 1.0)).max(0.0)
}

/// [v2.28.0 R16-1] 냄새 기반 최적 이동 방향
pub fn best_scent_direction(
    monster_x: i32,
    monster_y: i32,
    track: &TrackHistory,
    current_turn: u64,
) -> Option<(i32, i32)> {
    let recent = track.recent(20, current_turn);
    if recent.is_empty() {
        return None;
    }

    // 가장 강한 냄새 지점으로 이동
    let best = recent.iter().max_by(|a, b| {
        let sa = scent_strength(current_turn.saturating_sub(a.turn));
        let sb = scent_strength(current_turn.saturating_sub(b.turn));
        sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
    })?;

    let dx = (best.x - monster_x).signum();
    let dy = (best.y - monster_y).signum();
    if dx == 0 && dy == 0 {
        None
    } else {
        Some((dx, dy))
    }
}

// =============================================================================
// [3] 청각 추적 (원본: monmove.c에서 분리)
// =============================================================================

/// [v2.28.0 R16-1] 소음 감지 (거리 기반)
pub fn can_hear(distance: i32, noise_level: i32) -> bool {
    distance <= noise_level * 2
}

/// [v2.28.0 R16-1] 소음 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseType {
    Combat,    // 전투 소음 (레벨 5)
    SpellCast, // 주문 (레벨 3)
    DoorBreak, // 문 부수기 (레벨 7)
    Footstep,  // 발소리 (레벨 1)
    Digging,   // 채굴 (레벨 6)
}

pub fn noise_level(noise: NoiseType) -> i32 {
    match noise {
        NoiseType::Combat => 5,
        NoiseType::SpellCast => 3,
        NoiseType::DoorBreak => 7,
        NoiseType::Footstep => 1,
        NoiseType::Digging => 6,
    }
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_record() {
        let mut t = TrackHistory::new(5);
        t.record(10, 5, 1);
        t.record(11, 5, 2);
        assert_eq!(t.points.len(), 2);
        assert_eq!(t.last_known().unwrap().x, 11);
    }

    #[test]
    fn test_track_overflow() {
        let mut t = TrackHistory::new(3);
        for i in 0..5 {
            t.record(i, 0, i as u64);
        }
        assert_eq!(t.points.len(), 3);
        assert_eq!(t.points[0].x, 2);
    }

    #[test]
    fn test_recent() {
        let mut t = TrackHistory::new(10);
        t.record(1, 1, 10);
        t.record(2, 2, 50);
        t.record(3, 3, 95);
        let recent = t.recent(10, 100);
        assert_eq!(recent.len(), 1); // 턴 95만 (100-95=5 ≤ 10)
    }

    #[test]
    fn test_scent_decay() {
        assert_eq!(scent_strength(0), 1.0);
        assert!(scent_strength(10) < 0.1);
    }

    #[test]
    fn test_scent_direction() {
        let mut t = TrackHistory::new(10);
        t.record(15, 10, 98);
        t.record(16, 10, 99);
        t.record(17, 10, 100);
        let dir = best_scent_direction(10, 10, &t, 100);
        assert_eq!(dir, Some((1, 0))); // 동쪽으로
    }

    #[test]
    fn test_hearing() {
        assert!(can_hear(8, 5)); // 거리 8 ≤ 5*2=10
        assert!(!can_hear(15, 5)); // 거리 15 > 10
    }

    #[test]
    fn test_noise_level() {
        assert_eq!(noise_level(NoiseType::DoorBreak), 7);
        assert_eq!(noise_level(NoiseType::Footstep), 1);
    }
}

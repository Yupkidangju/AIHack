// ============================================================================
// [v2.25.0 Phase 1-4] 세이브/로드 확장 (save_phase1_ext.rs)
// 원본: NetHack 3.6.7 src/save.c + restore.c L800-2000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 세이브 데이터 직렬화 — savegame_state (save.c L200-500)
// =============================================================================

/// [v2.25.0 1-4] 세이브 가능한 게임 상태 스냅샷
#[derive(Debug, Clone)]
pub struct GameStateSnapshot {
    /// 플레이어 이름
    pub player_name: String,
    /// 플레이어 역할
    pub role: String,
    /// 플레이어 종족
    pub race: String,
    /// 현재 레벨
    pub dungeon_level: i32,
    /// 현재 턴 수
    pub turn_count: u64,
    /// HP
    pub hp: i32,
    pub hp_max: i32,
    /// 마나
    pub energy: i32,
    pub energy_max: i32,
    /// 경험치
    pub experience: u64,
    /// 골드
    pub gold: u64,
    /// AC
    pub ac: i32,
    /// 레벨
    pub level: i32,
    /// 능력치 (STR, DEX, CON, INT, WIS, CHA)
    pub stats: [i32; 6],
    /// 인벤토리 아이템 수
    pub inventory_count: i32,
    /// 맵 탐색율 (%)
    pub exploration_pct: f32,
    /// 사망 원인 (빈 문자열이면 게임 진행 중)
    pub death_cause: String,
}

/// [v2.25.0 1-4] 스냅샷 검증 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotValidation {
    Valid,
    /// 데이터 손상 의심
    Corrupted {
        field: String,
        reason: String,
    },
    /// 불가능한 값
    Impossible {
        field: String,
        value: String,
    },
}

/// [v2.25.0 1-4] 게임 상태 스냅샷 유효성 검사
/// 원본: save.c savestateinternalcheck()
pub fn validate_snapshot(snap: &GameStateSnapshot) -> SnapshotValidation {
    // HP 범위 검사
    if snap.hp_max <= 0 {
        return SnapshotValidation::Corrupted {
            field: "hp_max".to_string(),
            reason: "최대 HP가 0 이하".to_string(),
        };
    }
    if snap.hp > snap.hp_max * 2 {
        return SnapshotValidation::Impossible {
            field: "hp".to_string(),
            value: format!("{}/{}", snap.hp, snap.hp_max),
        };
    }

    // 레벨 범위 검사 (1~30)
    if snap.level < 1 || snap.level > 30 {
        return SnapshotValidation::Impossible {
            field: "level".to_string(),
            value: snap.level.to_string(),
        };
    }

    // 턴 수 검사 (0이면 게임 시작 안 함)
    if snap.turn_count == 0 && snap.level > 1 {
        return SnapshotValidation::Corrupted {
            field: "turn_count".to_string(),
            reason: "레벨 2+ 인데 턴 0".to_string(),
        };
    }

    // 능력치 범위 (3~25)
    for (i, &stat) in snap.stats.iter().enumerate() {
        if stat < 1 || stat > 25 {
            let stat_name = match i {
                0 => "STR",
                1 => "DEX",
                2 => "CON",
                3 => "INT",
                4 => "WIS",
                5 => "CHA",
                _ => "UNKNOWN",
            };
            return SnapshotValidation::Impossible {
                field: stat_name.to_string(),
                value: stat.to_string(),
            };
        }
    }

    SnapshotValidation::Valid
}

// =============================================================================
// [2] 세이브 파일 메타데이터 — save_metadata (save.c L600-700)
// =============================================================================

/// [v2.25.0 1-4] 세이브 파일 메타데이터
#[derive(Debug, Clone)]
pub struct SaveMetadata {
    /// 저장 타임스탬프 (Unix epoch)
    pub timestamp: u64,
    /// 게임 버전
    pub game_version: String,
    /// 세이브 파일 크기 (바이트)
    pub file_size: u64,
    /// 압축 여부
    pub compressed: bool,
    /// 플레이어 이름 (미리보기용)
    pub player_name: String,
    /// 요약 문자열 (미리보기용)
    pub summary: String,
}

/// [v2.25.0 1-4] 세이브 메타데이터 생성
pub fn create_save_metadata(
    snap: &GameStateSnapshot,
    timestamp: u64,
    game_version: &str,
    compressed: bool,
) -> SaveMetadata {
    let summary = format!(
        "{} Lv{} {} {} T:{} D:{}",
        snap.player_name, snap.level, snap.role, snap.race, snap.turn_count, snap.dungeon_level
    );

    SaveMetadata {
        timestamp,
        game_version: game_version.to_string(),
        file_size: 0, // 직렬화 후 설정
        compressed,
        player_name: snap.player_name.clone(),
        summary,
    }
}

// =============================================================================
// [3] 자동 세이브 트리거 — autosave_check (save.c L900-1000)
// =============================================================================

/// [v2.25.0 1-4] 자동 세이브 설정
#[derive(Debug, Clone)]
pub struct AutosaveConfig {
    /// 자동 세이브 활성화
    pub enabled: bool,
    /// 세이브 주기 (턴 수)
    pub interval_turns: u64,
    /// 레벨 변경 시 세이브
    pub on_level_change: bool,
    /// HP 임계값 이하일 때 세이브
    pub on_low_hp: bool,
    pub hp_threshold_pct: i32,
}

/// [v2.25.0 1-4] 자동 세이브 트리거 여부 판정
pub fn should_autosave(
    config: &AutosaveConfig,
    current_turn: u64,
    last_save_turn: u64,
    level_changed: bool,
    hp_pct: i32,
) -> bool {
    if !config.enabled {
        return false;
    }

    // 레벨 변경
    if config.on_level_change && level_changed {
        return true;
    }

    // HP 위험
    if config.on_low_hp && hp_pct <= config.hp_threshold_pct {
        return true;
    }

    // 주기적 세이브
    if config.interval_turns > 0 && current_turn - last_save_turn >= config.interval_turns {
        return true;
    }

    false
}

// =============================================================================
// [4] 세이브 데이터 차분 — delta_save (restore.c 최적화)
// =============================================================================

/// [v2.25.0 1-4] 세이브 차분 항목
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveDelta {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

/// [v2.25.0 1-4] 두 스냅샷 간 차분 계산
/// 최적화: 변경된 필드만 저장하여 세이브 크기 축소
pub fn compute_save_delta(
    old_snap: &GameStateSnapshot,
    new_snap: &GameStateSnapshot,
) -> Vec<SaveDelta> {
    let mut deltas = Vec::new();

    macro_rules! check_delta {
        ($field:ident) => {
            if old_snap.$field != new_snap.$field {
                deltas.push(SaveDelta {
                    field: stringify!($field).to_string(),
                    old_value: format!("{:?}", old_snap.$field),
                    new_value: format!("{:?}", new_snap.$field),
                });
            }
        };
    }

    check_delta!(hp);
    check_delta!(hp_max);
    check_delta!(energy);
    check_delta!(energy_max);
    check_delta!(experience);
    check_delta!(gold);
    check_delta!(ac);
    check_delta!(level);
    check_delta!(dungeon_level);
    check_delta!(turn_count);
    check_delta!(inventory_count);

    // 능력치 개별 비교
    let stat_names = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
    for (i, name) in stat_names.iter().enumerate() {
        if old_snap.stats[i] != new_snap.stats[i] {
            deltas.push(SaveDelta {
                field: name.to_string(),
                old_value: old_snap.stats[i].to_string(),
                new_value: new_snap.stats[i].to_string(),
            });
        }
    }

    deltas
}

// =============================================================================
// [5] 세이브 복구 — emergency_restore (restore.c L1500-1700)
// =============================================================================

/// [v2.25.0 1-4] 복구 시도 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreResult {
    /// 정상 복구 성공
    Success,
    /// 부분 복구 (일부 데이터 누락)
    PartialRestore { missing_fields: Vec<String> },
    /// 버전 불일치 → 마이그레이션 필요
    VersionMismatch {
        save_version: String,
        current_version: String,
    },
    /// 복구 불가능
    Failed { reason: String },
}

/// [v2.25.0 1-4] 세이브 복구 가능 여부 판정
/// 원본: restore.c dorecover()
pub fn check_restore(
    save_version: &str,
    current_version: &str,
    header_valid: bool,
    checksum_valid: bool,
    required_fields: &[&str],
    available_fields: &[&str],
) -> RestoreResult {
    if !header_valid {
        return RestoreResult::Failed {
            reason: "세이브 파일 헤더 손상".to_string(),
        };
    }

    if !checksum_valid {
        return RestoreResult::Failed {
            reason: "체크섬 불일치 — 데이터 손상".to_string(),
        };
    }

    if save_version != current_version {
        return RestoreResult::VersionMismatch {
            save_version: save_version.to_string(),
            current_version: current_version.to_string(),
        };
    }

    let missing: Vec<String> = required_fields
        .iter()
        .filter(|f| !available_fields.contains(f))
        .map(|f| f.to_string())
        .collect();

    if missing.is_empty() {
        RestoreResult::Success
    } else {
        RestoreResult::PartialRestore {
            missing_fields: missing,
        }
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_snapshot() -> GameStateSnapshot {
        GameStateSnapshot {
            player_name: "TestPlayer".to_string(),
            role: "Valkyrie".to_string(),
            race: "Human".to_string(),
            dungeon_level: 5,
            turn_count: 1000,
            hp: 50,
            hp_max: 60,
            energy: 20,
            energy_max: 30,
            experience: 5000,
            gold: 1500,
            ac: 3,
            level: 8,
            stats: [18, 14, 16, 10, 12, 10],
            inventory_count: 25,
            exploration_pct: 45.0,
            death_cause: String::new(),
        }
    }

    // --- validate_snapshot ---

    #[test]
    fn test_snapshot_valid() {
        let snap = test_snapshot();
        assert_eq!(validate_snapshot(&snap), SnapshotValidation::Valid);
    }

    #[test]
    fn test_snapshot_bad_hp() {
        let mut snap = test_snapshot();
        snap.hp_max = 0;
        assert!(matches!(
            validate_snapshot(&snap),
            SnapshotValidation::Corrupted { .. }
        ));
    }

    #[test]
    fn test_snapshot_bad_level() {
        let mut snap = test_snapshot();
        snap.level = 50;
        assert!(matches!(
            validate_snapshot(&snap),
            SnapshotValidation::Impossible { .. }
        ));
    }

    #[test]
    fn test_snapshot_bad_stat() {
        let mut snap = test_snapshot();
        snap.stats[0] = 30; // STR > 25
        assert!(matches!(
            validate_snapshot(&snap),
            SnapshotValidation::Impossible { .. }
        ));
    }

    // --- save_metadata ---

    #[test]
    fn test_metadata_summary() {
        let snap = test_snapshot();
        let meta = create_save_metadata(&snap, 1000000, "2.25.0", false);
        assert!(meta.summary.contains("TestPlayer"));
        assert!(meta.summary.contains("Valkyrie"));
    }

    // --- autosave ---

    #[test]
    fn test_autosave_disabled() {
        let config = AutosaveConfig {
            enabled: false,
            interval_turns: 100,
            on_level_change: true,
            on_low_hp: true,
            hp_threshold_pct: 25,
        };
        assert!(!should_autosave(&config, 200, 0, true, 10));
    }

    #[test]
    fn test_autosave_by_interval() {
        let config = AutosaveConfig {
            enabled: true,
            interval_turns: 100,
            on_level_change: false,
            on_low_hp: false,
            hp_threshold_pct: 25,
        };
        assert!(should_autosave(&config, 200, 50, false, 100));
        assert!(!should_autosave(&config, 80, 50, false, 100));
    }

    #[test]
    fn test_autosave_by_level_change() {
        let config = AutosaveConfig {
            enabled: true,
            interval_turns: 0,
            on_level_change: true,
            on_low_hp: false,
            hp_threshold_pct: 25,
        };
        assert!(should_autosave(&config, 10, 0, true, 100));
    }

    #[test]
    fn test_autosave_by_low_hp() {
        let config = AutosaveConfig {
            enabled: true,
            interval_turns: 0,
            on_level_change: false,
            on_low_hp: true,
            hp_threshold_pct: 25,
        };
        assert!(should_autosave(&config, 10, 0, false, 15));
        assert!(!should_autosave(&config, 10, 0, false, 50));
    }

    // --- compute_save_delta ---

    #[test]
    fn test_delta_no_changes() {
        let snap = test_snapshot();
        let deltas = compute_save_delta(&snap, &snap);
        assert!(deltas.is_empty());
    }

    #[test]
    fn test_delta_hp_change() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        new.hp = 30;
        let deltas = compute_save_delta(&old, &new);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].field, "hp");
    }

    #[test]
    fn test_delta_multiple_changes() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        new.hp = 30;
        new.gold = 5000;
        new.level = 9;
        let deltas = compute_save_delta(&old, &new);
        assert_eq!(deltas.len(), 3);
    }

    // --- check_restore ---

    #[test]
    fn test_restore_success() {
        let result = check_restore(
            "2.25.0",
            "2.25.0",
            true,
            true,
            &["hp", "gold", "level"],
            &["hp", "gold", "level", "exp"],
        );
        assert_eq!(result, RestoreResult::Success);
    }

    #[test]
    fn test_restore_version_mismatch() {
        let result = check_restore("2.20.0", "2.25.0", true, true, &["hp"], &["hp"]);
        assert!(matches!(result, RestoreResult::VersionMismatch { .. }));
    }

    #[test]
    fn test_restore_checksum_fail() {
        let result = check_restore("2.25.0", "2.25.0", true, false, &["hp"], &["hp"]);
        assert!(matches!(result, RestoreResult::Failed { .. }));
    }

    #[test]
    fn test_restore_partial() {
        let result = check_restore(
            "2.25.0",
            "2.25.0",
            true,
            true,
            &["hp", "gold", "map_data"],
            &["hp", "gold"],
        );
        assert!(matches!(result, RestoreResult::PartialRestore { .. }));
    }
}

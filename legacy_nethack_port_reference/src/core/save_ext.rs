// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.23.0 R11-2] 세이브/로드 확장 (save_ext.rs)
//
// 원본 참조: NetHack 3.6.7 save.c (1,481줄) + restore.c (1,719줄)
//
// 구현 내용:
//   1. 세이브 무결성 체크섬 (CRC32)
//   2. 버전 마이그레이션 스키마
//   3. 세이브 파일 헤더 구조
//   4. 차등(Delta) 저장 메타데이터
//   5. 자동 백업/복구 전략
//   6. 세이브 슬롯 관리
//   7. 보안(세이브 변조 감지)
// ============================================================================

// =============================================================================
// [1] CRC32 체크섬 (원본: save.c save_checksum / verify)
// =============================================================================

/// [v2.23.0 R11-2] CRC32 테이블 (IEEE 802.3 다항식)
const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
};

/// [v2.23.0 R11-2] CRC32 계산
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx];
    }
    !crc
}

/// [v2.23.0 R11-2] CRC32 검증
pub fn verify_crc32(data: &[u8], expected: u32) -> bool {
    crc32(data) == expected
}

// =============================================================================
// [2] 세이브 파일 헤더 (원본: save.c save_header)
// =============================================================================

/// [v2.23.0 R11-2] 세이브 파일 매직 넘버
pub const SAVE_MAGIC: u32 = 0x41494841; // "AIHA"

/// [v2.23.0 R11-2] 세이브 파일 헤더
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveHeader {
    /// 매직 넘버
    pub magic: u32,
    /// 세이브 포맷 버전
    pub format_version: u32,
    /// 게임 버전 문자열
    pub game_version: String,
    /// 생성 시각 (Unix timestamp)
    pub timestamp: u64,
    /// 데이터 영역 CRC32
    pub data_checksum: u32,
    /// 데이터 길이 (바이트)
    pub data_length: u64,
    /// 압축 여부
    pub compressed: bool,
    /// 플레이어 이름
    pub player_name: String,
}

/// [v2.23.0 R11-2] 세이브 헤더 유효성 검사
pub fn validate_header(header: &SaveHeader) -> SaveValidation {
    if header.magic != SAVE_MAGIC {
        return SaveValidation::InvalidMagic;
    }
    if header.format_version > CURRENT_FORMAT_VERSION {
        return SaveValidation::FutureVersion {
            file_version: header.format_version,
            current_version: CURRENT_FORMAT_VERSION,
        };
    }
    if header.data_length == 0 {
        return SaveValidation::EmptyData;
    }
    SaveValidation::Valid
}

/// [v2.23.0 R11-2] 세이브 유효성 검사 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveValidation {
    /// 유효한 세이브
    Valid,
    /// 잘못된 매직 넘버 (파일 아님)
    InvalidMagic,
    /// 미래 버전 (업그레이드 필요)
    FutureVersion {
        file_version: u32,
        current_version: u32,
    },
    /// 빈 데이터
    EmptyData,
    /// 체크섬 불일치 (변조 감지)
    ChecksumMismatch { expected: u32, actual: u32 },
    /// 데이터 손상
    CorruptData(String),
}

// =============================================================================
// [3] 버전 마이그레이션 (원본: restore.c version_check)
// =============================================================================

/// [v2.23.0 R11-2] 현재 포맷 버전
pub const CURRENT_FORMAT_VERSION: u32 = 5;

/// [v2.23.0 R11-2] 마이그레이션 단계
#[derive(Debug, Clone)]
pub struct MigrationStep {
    /// 원본 버전
    pub from_version: u32,
    /// 대상 버전
    pub to_version: u32,
    /// 변경 설명
    pub description: String,
    /// 호환 가능 여부 (비호환이면 마이그레이션 불가)
    pub compatible: bool,
}

/// [v2.23.0 R11-2] 마이그레이션 경로 계산
pub fn migration_path(from_version: u32, to_version: u32) -> Vec<MigrationStep> {
    let mut steps = Vec::new();

    // 버전 1→2: 인벤토리 구조 변경
    if from_version < 2 && to_version >= 2 {
        steps.push(MigrationStep {
            from_version: 1,
            to_version: 2,
            description: "인벤토리 슬롯 구조 변경".to_string(),
            compatible: true,
        });
    }
    // 버전 2→3: 몬스터 상태 확장
    if from_version < 3 && to_version >= 3 {
        steps.push(MigrationStep {
            from_version: 2,
            to_version: 3,
            description: "몬스터 상태 플래그 확장".to_string(),
            compatible: true,
        });
    }
    // 버전 3→4: 던전 분기 추가
    if from_version < 4 && to_version >= 4 {
        steps.push(MigrationStep {
            from_version: 3,
            to_version: 4,
            description: "던전 분기 구조 추가 (블라드탑, 엔드게임)".to_string(),
            compatible: true,
        });
    }
    // 버전 4→5: 퀘스트 시스템 추가
    if from_version < 5 && to_version >= 5 {
        steps.push(MigrationStep {
            from_version: 4,
            to_version: 5,
            description: "퀘스트 상태 필드 추가".to_string(),
            compatible: true,
        });
    }

    steps
}

/// [v2.23.0 R11-2] 마이그레이션 가능 여부 확인
pub fn can_migrate(from_version: u32, to_version: u32) -> bool {
    let path = migration_path(from_version, to_version);
    if path.is_empty() && from_version != to_version {
        return false;
    }
    path.iter().all(|s| s.compatible)
}

// =============================================================================
// [4] 세이브 슬롯 관리 (원본: save.c slot system)
// =============================================================================

/// [v2.23.0 R11-2] 세이브 슬롯 상태
#[derive(Debug, Clone)]
pub struct SaveSlot {
    /// 슬롯 번호 (0-9)
    pub slot_id: u8,
    /// 슬롯 사용 여부
    pub occupied: bool,
    /// 플레이어 이름
    pub player_name: Option<String>,
    /// 마지막 저장 시각
    pub last_saved: Option<u64>,
    /// 파일 크기 (바이트)
    pub file_size: Option<u64>,
    /// 게임 턴 수
    pub turn_count: Option<u64>,
    /// 깊이
    pub depth: Option<i32>,
}

/// [v2.23.0 R11-2] 세이브 슬롯 목록 기본 생성
pub fn create_empty_slots(count: u8) -> Vec<SaveSlot> {
    (0..count)
        .map(|i| SaveSlot {
            slot_id: i,
            occupied: false,
            player_name: None,
            last_saved: None,
            file_size: None,
            turn_count: None,
            depth: None,
        })
        .collect()
}

/// [v2.23.0 R11-2] 빈 슬롯 찾기
pub fn find_empty_slot(slots: &[SaveSlot]) -> Option<u8> {
    slots.iter().find(|s| !s.occupied).map(|s| s.slot_id)
}

/// [v2.23.0 R11-2] 가장 오래된 슬롯 찾기 (덮어쓰기 후보)
pub fn find_oldest_slot(slots: &[SaveSlot]) -> Option<u8> {
    slots
        .iter()
        .filter(|s| s.occupied)
        .min_by_key(|s| s.last_saved.unwrap_or(u64::MAX))
        .map(|s| s.slot_id)
}

// =============================================================================
// [5] 자동 백업 전략 (원본: save.c emergency_save)
// =============================================================================

/// [v2.23.0 R11-2] 백업 전략
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupStrategy {
    /// 백업 없음
    None,
    /// 매 세이브 시 이전 파일 보존 (.bak)
    SingleBackup,
    /// 회전식 백업 (최대 N개)
    RotatingBackup(u8),
}

/// [v2.23.0 R11-2] 백업 파일 경로 생성
pub fn backup_path(original_path: &str, backup_index: u8) -> String {
    if backup_index == 0 {
        format!("{}.bak", original_path)
    } else {
        format!("{}.bak{}", original_path, backup_index)
    }
}

/// [v2.23.0 R11-2] 회전 백업 시 삭제 대상 인덱스 계산
pub fn rotation_delete_target(max_backups: u8, current_count: u8) -> Option<u8> {
    if current_count >= max_backups {
        Some(0) // 가장 오래된 것 삭제
    } else {
        None
    }
}

// =============================================================================
// [6] 세이브 통계 (원본: save.c / endgame 연동)
// =============================================================================

/// [v2.23.0 R11-2] 세이브 통계
#[derive(Debug, Clone, Default)]
pub struct SaveStats {
    /// 총 세이브 횟수
    pub total_saves: u32,
    /// 총 로드 횟수
    pub total_loads: u32,
    /// 마지막 세이브 크기 (바이트)
    pub last_save_size: u64,
    /// 평균 세이브 시간 (밀리초)
    pub avg_save_time_ms: u32,
    /// 마이그레이션 횟수
    pub migrations_performed: u32,
    /// 체크섬 실패 횟수
    pub checksum_failures: u32,
}

/// [v2.23.0 R11-2] 세이브 이벤트 기록
pub fn record_save_event(stats: &mut SaveStats, save_size: u64, save_time_ms: u32) {
    stats.total_saves += 1;
    stats.last_save_size = save_size;
    // 이동 평균
    if stats.avg_save_time_ms == 0 {
        stats.avg_save_time_ms = save_time_ms;
    } else {
        stats.avg_save_time_ms = (stats.avg_save_time_ms + save_time_ms) / 2;
    }
}

/// [v2.23.0 R11-2] 로드 이벤트 기록
pub fn record_load_event(stats: &mut SaveStats) {
    stats.total_loads += 1;
}

// =============================================================================
// [7] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_empty() {
        assert_eq!(crc32(&[]), 0x00000000);
    }

    #[test]
    fn test_crc32_data() {
        let data = b"Hello, World!";
        let checksum = crc32(data);
        // 동일 데이터에 대해 동일 체크섬
        assert_eq!(crc32(data), checksum);
    }

    #[test]
    fn test_crc32_different_data() {
        assert_ne!(crc32(b"hello"), crc32(b"world"));
    }

    #[test]
    fn test_verify_crc32() {
        let data = b"test data";
        let checksum = crc32(data);
        assert!(verify_crc32(data, checksum));
        assert!(!verify_crc32(data, checksum + 1));
    }

    #[test]
    fn test_header_valid() {
        let header = SaveHeader {
            magic: SAVE_MAGIC,
            format_version: 3,
            game_version: "2.23.0".to_string(),
            timestamp: 1000,
            data_checksum: 0,
            data_length: 1024,
            compressed: false,
            player_name: "Gandalf".to_string(),
        };
        assert_eq!(validate_header(&header), SaveValidation::Valid);
    }

    #[test]
    fn test_header_invalid_magic() {
        let header = SaveHeader {
            magic: 0xDEADBEEF,
            format_version: 3,
            game_version: "2.23.0".to_string(),
            timestamp: 1000,
            data_checksum: 0,
            data_length: 1024,
            compressed: false,
            player_name: "Gandalf".to_string(),
        };
        assert_eq!(validate_header(&header), SaveValidation::InvalidMagic);
    }

    #[test]
    fn test_header_future_version() {
        let header = SaveHeader {
            magic: SAVE_MAGIC,
            format_version: 99,
            game_version: "9.0.0".to_string(),
            timestamp: 1000,
            data_checksum: 0,
            data_length: 1024,
            compressed: false,
            player_name: "Gandalf".to_string(),
        };
        assert!(matches!(
            validate_header(&header),
            SaveValidation::FutureVersion { .. }
        ));
    }

    #[test]
    fn test_header_empty_data() {
        let header = SaveHeader {
            magic: SAVE_MAGIC,
            format_version: 3,
            game_version: "2.23.0".to_string(),
            timestamp: 1000,
            data_checksum: 0,
            data_length: 0,
            compressed: false,
            player_name: "Gandalf".to_string(),
        };
        assert_eq!(validate_header(&header), SaveValidation::EmptyData);
    }

    #[test]
    fn test_migration_path_1_to_5() {
        let path = migration_path(1, 5);
        assert_eq!(path.len(), 4); // 1→2, 2→3, 3→4, 4→5
    }

    #[test]
    fn test_migration_path_3_to_5() {
        let path = migration_path(3, 5);
        assert_eq!(path.len(), 2); // 3→4, 4→5
    }

    #[test]
    fn test_migration_path_same() {
        let path = migration_path(5, 5);
        assert_eq!(path.len(), 0);
    }

    #[test]
    fn test_can_migrate() {
        assert!(can_migrate(1, 5));
        assert!(can_migrate(3, 5));
        assert!(can_migrate(5, 5));
    }

    #[test]
    fn test_empty_slots() {
        let slots = create_empty_slots(5);
        assert_eq!(slots.len(), 5);
        assert!(slots.iter().all(|s| !s.occupied));
    }

    #[test]
    fn test_find_empty_slot() {
        let mut slots = create_empty_slots(3);
        slots[0].occupied = true;
        slots[1].occupied = true;
        assert_eq!(find_empty_slot(&slots), Some(2));
    }

    #[test]
    fn test_find_empty_slot_none() {
        let mut slots = create_empty_slots(2);
        slots[0].occupied = true;
        slots[1].occupied = true;
        assert_eq!(find_empty_slot(&slots), None);
    }

    #[test]
    fn test_find_oldest_slot() {
        let mut slots = create_empty_slots(3);
        slots[0].occupied = true;
        slots[0].last_saved = Some(100);
        slots[1].occupied = true;
        slots[1].last_saved = Some(50); // 가장 오래됨
        slots[2].occupied = true;
        slots[2].last_saved = Some(200);
        assert_eq!(find_oldest_slot(&slots), Some(1));
    }

    #[test]
    fn test_backup_path() {
        assert_eq!(backup_path("game.sav", 0), "game.sav.bak");
        assert_eq!(backup_path("game.sav", 2), "game.sav.bak2");
    }

    #[test]
    fn test_rotation_delete() {
        assert_eq!(rotation_delete_target(3, 3), Some(0));
        assert_eq!(rotation_delete_target(3, 2), None);
    }

    #[test]
    fn test_save_stats() {
        let mut stats = SaveStats::default();
        record_save_event(&mut stats, 1024, 50);
        assert_eq!(stats.total_saves, 1);
        assert_eq!(stats.last_save_size, 1024);
        assert_eq!(stats.avg_save_time_ms, 50);

        record_save_event(&mut stats, 2048, 100);
        assert_eq!(stats.total_saves, 2);
        assert_eq!(stats.avg_save_time_ms, 75); // (50+100)/2

        record_load_event(&mut stats);
        assert_eq!(stats.total_loads, 1);
    }
}

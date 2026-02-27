// ============================================================================
// [v2.35.0 Phase 99-1] 세이브/로드 확장 (save_phase99_ext.rs)
// 원본: NetHack 3.6.7 src/save.c + restore.c 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 세이브 시스템 — save_game (save.c 핵심)
// =============================================================================

/// [v2.35.0 99-1] 세이브 데이터 구성요소
#[derive(Debug, Clone)]
pub struct SaveData {
    pub version: String,
    pub player_name: String,
    pub player_race: String,
    pub player_role: String,
    pub player_level: i32,
    pub current_hp: i32,
    pub max_hp: i32,
    pub current_mp: i32,
    pub max_mp: i32,
    pub gold: i64,
    pub dungeon_level: i32,
    pub branch: String,
    pub turn_count: i32,
    pub score: i64,
    pub inventory_count: i32,
    pub status_effects: Vec<String>,
    pub conducts: Vec<String>,
    pub checksum: u32,
}

/// [v2.35.0 99-1] 세이브 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveResult {
    Success { file_path: String, size_bytes: i64 },
    Failed { reason: String },
    Corrupted { details: String },
}

/// [v2.35.0 99-1] 체크섬 계산
pub fn calculate_checksum(data: &SaveData) -> u32 {
    let mut hash: u32 = 0;
    hash = hash.wrapping_add(data.player_level as u32 * 31);
    hash = hash.wrapping_add(data.current_hp as u32 * 37);
    hash = hash.wrapping_add(data.max_hp as u32 * 41);
    hash = hash.wrapping_add(data.gold as u32 * 43);
    hash = hash.wrapping_add(data.dungeon_level as u32 * 47);
    hash = hash.wrapping_add(data.turn_count as u32 * 53);
    hash = hash.wrapping_add(data.inventory_count as u32 * 59);
    for (i, ch) in data.player_name.chars().enumerate() {
        hash = hash.wrapping_add((ch as u32) * (61 + i as u32));
    }
    hash
}

/// [v2.35.0 99-1] 세이브 직렬화
pub fn serialize_save(data: &SaveData) -> Result<Vec<u8>, String> {
    if data.player_name.is_empty() {
        return Err("플레이어 이름이 비어있다.".to_string());
    }
    if data.version.is_empty() {
        return Err("버전 정보가 없다.".to_string());
    }

    // 간이 직렬화 (실제로는 bincode 등 사용)
    let header = format!(
        "AIHACK_SAVE|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        data.version,
        data.player_name,
        data.player_race,
        data.player_role,
        data.player_level,
        data.current_hp,
        data.max_hp,
        data.current_mp,
        data.max_mp,
        data.gold,
        data.dungeon_level,
        data.branch,
        data.turn_count,
    );
    Ok(header.into_bytes())
}

/// [v2.35.0 99-1] 세이브 검증
pub fn validate_save(data: &SaveData) -> SaveResult {
    let expected = calculate_checksum(data);
    if data.checksum != expected {
        return SaveResult::Corrupted {
            details: format!("체크섬 불일치: 기대={}, 실제={}", expected, data.checksum),
        };
    }
    if data.current_hp > data.max_hp {
        return SaveResult::Corrupted {
            details: "HP가 최대HP를 초과한다.".to_string(),
        };
    }
    if data.player_level < 1 || data.player_level > 30 {
        return SaveResult::Corrupted {
            details: format!("유효하지 않은 레벨: {}", data.player_level),
        };
    }
    SaveResult::Success {
        file_path: format!("save/{}.nhsave", data.player_name),
        size_bytes: 0,
    }
}

// =============================================================================
// [2] 복원 (로드) — restore_game (restore.c 핵심)
// =============================================================================

/// [v2.35.0 99-1] 로드 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadResult {
    Success {
        player_name: String,
        turn: i32,
    },
    VersionMismatch {
        save_version: String,
        game_version: String,
    },
    FileNotFound,
    Corrupted {
        reason: String,
    },
}

/// [v2.35.0 99-1] 세이브 헤더 파싱
pub fn parse_save_header(raw: &[u8]) -> LoadResult {
    let text = match std::str::from_utf8(raw) {
        Ok(t) => t,
        Err(_) => {
            return LoadResult::Corrupted {
                reason: "UTF-8 디코딩 실패".to_string(),
            }
        }
    };

    if !text.starts_with("AIHACK_SAVE|") {
        return LoadResult::Corrupted {
            reason: "잘못된 파일 형식".to_string(),
        };
    }

    let parts: Vec<&str> = text.split('|').collect();
    if parts.len() < 13 {
        return LoadResult::Corrupted {
            reason: "필드 부족".to_string(),
        };
    }

    let version = parts[1];
    let name = parts[2];
    let turn = parts
        .last()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    // 버전 호환성 확인
    if !version.starts_with("2.") {
        return LoadResult::VersionMismatch {
            save_version: version.to_string(),
            game_version: "2.35.0".to_string(),
        };
    }

    LoadResult::Success {
        player_name: name.to_string(),
        turn,
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_save_data() -> SaveData {
        let mut data = SaveData {
            version: "2.35.0".to_string(),
            player_name: "용사".to_string(),
            player_race: "인간".to_string(),
            player_role: "전사".to_string(),
            player_level: 15,
            current_hp: 80,
            max_hp: 100,
            current_mp: 30,
            max_mp: 50,
            gold: 5000,
            dungeon_level: 10,
            branch: "Main".to_string(),
            turn_count: 20000,
            score: 15000,
            inventory_count: 25,
            status_effects: vec!["가속".to_string()],
            conducts: vec!["Pacifist".to_string()],
            checksum: 0,
        };
        data.checksum = calculate_checksum(&data);
        data
    }

    #[test]
    fn test_checksum() {
        let data = test_save_data();
        let cs1 = calculate_checksum(&data);
        let cs2 = calculate_checksum(&data);
        assert_eq!(cs1, cs2);
    }

    #[test]
    fn test_serialize() {
        let data = test_save_data();
        let bytes = serialize_save(&data);
        assert!(bytes.is_ok());
        let b = bytes.unwrap();
        assert!(b.len() > 0);
    }

    #[test]
    fn test_validate_ok() {
        let data = test_save_data();
        let result = validate_save(&data);
        assert!(matches!(result, SaveResult::Success { .. }));
    }

    #[test]
    fn test_validate_corrupted() {
        let mut data = test_save_data();
        data.checksum = 999;
        let result = validate_save(&data);
        assert!(matches!(result, SaveResult::Corrupted { .. }));
    }

    #[test]
    fn test_parse_header() {
        let data = test_save_data();
        let bytes = serialize_save(&data).unwrap();
        let result = parse_save_header(&bytes);
        assert!(matches!(result, LoadResult::Success { .. }));
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse_save_header(b"INVALID_DATA");
        assert!(matches!(result, LoadResult::Corrupted { .. }));
    }

    #[test]
    fn test_empty_name() {
        let mut data = test_save_data();
        data.player_name = String::new();
        let result = serialize_save(&data);
        assert!(result.is_err());
    }
}

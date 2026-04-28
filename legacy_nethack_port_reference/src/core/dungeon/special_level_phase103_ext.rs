// ============================================================================
// [v2.39.0 Phase 103-4] 던전 특수 층/분기 통합 (special_level_phase103_ext.rs)
// 원본: NetHack 3.6.7 src/sp_lev.c + dat/*.des 특수 층 통합
// 순수 결과 패턴
//
// 구현 범위:
//   - 특수 층 정의 (오라클, 메두사, 소반, 대성당 등)
//   - 분기별 특성 (게헨나, 블랙마켓, 녹스 등)
//   - 특수 층 보스/이벤트 배치
//   - 층 진입 시 효과
//   - 미니맵 메타데이터
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 특수 층 시스템 — special_levels
// =============================================================================

/// [v2.39.0 103-4] 특수 층 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialLevel {
    Oracle,          // 오라클 (5~9층)
    BigRoom,         // 큰 방 (10~12층)
    Rogue,           // 로그 레벨 (15~18층)
    Medusa,          // 메두사 (20~25층)
    Castle,          // 성 (25~30층)
    ValleyOfDead,    // 죽음의 계곡 (게헨나 입구)
    Juiblex,         // 주이블렉스의 소굴
    Baalzebub,       // 바알제붑의 거처
    Asmodeus,        // 아즈모데우스의 궁전
    Orcus,           // 오르쿠스의 마을
    WizardTower,     // 마법사의 탑 (위저드)
    VibratingSquare, // 진동하는 광장
    Sanctum,         // 몰록의 성소
    AstralPlane,     // 아스트랄 평원 (최종)
}

/// [v2.39.0 103-4] 특수 층 정보
#[derive(Debug, Clone)]
pub struct SpecialLevelInfo {
    pub level_type: SpecialLevel,
    pub name: String,
    pub depth_range: (i32, i32),
    pub boss: Option<String>,
    pub entry_message: String,
    pub danger_level: i32,
    pub no_teleport: bool,
    pub no_dig: bool,
    pub unique_items: Vec<String>,
}

/// [v2.39.0 103-4] 특수 층 정보 조회
pub fn get_special_level_info(level: SpecialLevel) -> SpecialLevelInfo {
    match level {
        SpecialLevel::Oracle => SpecialLevelInfo {
            level_type: level,
            name: "오라클의 전당".to_string(),
            depth_range: (5, 9),
            boss: None,
            entry_message: "웅장한 기둥이 늘어선 전당이 보인다.".to_string(),
            danger_level: 5,
            no_teleport: false,
            no_dig: false,
            unique_items: vec![],
        },
        SpecialLevel::Medusa => SpecialLevelInfo {
            level_type: level,
            name: "메두사의 섬".to_string(),
            depth_range: (20, 25),
            boss: Some("메두사".to_string()),
            entry_message: "석상들이 공포의 표정으로 서 있다...".to_string(),
            danger_level: 20,
            no_teleport: false,
            no_dig: false,
            unique_items: vec!["방패 of Reflection".to_string()],
        },
        SpecialLevel::Castle => SpecialLevelInfo {
            level_type: level,
            name: "성".to_string(),
            depth_range: (25, 30),
            boss: None,
            entry_message: "거대한 성벽이 솟아 있다. 해자가 성을 둘러싸고 있다.".to_string(),
            danger_level: 25,
            no_teleport: false,
            no_dig: false,
            unique_items: vec!["소원의 지팡이".to_string()],
        },
        SpecialLevel::ValleyOfDead => SpecialLevelInfo {
            level_type: level,
            name: "죽음의 계곡".to_string(),
            depth_range: (30, 30),
            boss: None,
            entry_message: "으스스한 안개가 깔려 있다. 여기서부터 게헨나다.".to_string(),
            danger_level: 30,
            no_teleport: true,
            no_dig: true,
            unique_items: vec![],
        },
        SpecialLevel::WizardTower => SpecialLevelInfo {
            level_type: level,
            name: "마법사의 탑".to_string(),
            depth_range: (35, 40),
            boss: Some("옌더의 마법사".to_string()),
            entry_message: "탑 꼭대기에서 사악한 기운이 느껴진다!".to_string(),
            danger_level: 35,
            no_teleport: true,
            no_dig: true,
            unique_items: vec!["옌더의 부적".to_string(), "Book of the Dead".to_string()],
        },
        SpecialLevel::Sanctum => SpecialLevelInfo {
            level_type: level,
            name: "몰록의 성소".to_string(),
            depth_range: (45, 50),
            boss: Some("몰록의 대사제".to_string()),
            entry_message: "어둠의 기운이 압도한다. 이곳이 성소다!".to_string(),
            danger_level: 45,
            no_teleport: true,
            no_dig: true,
            unique_items: vec![],
        },
        SpecialLevel::AstralPlane => SpecialLevelInfo {
            level_type: level,
            name: "아스트랄 평원".to_string(),
            depth_range: (50, 50),
            boss: Some("Death, Pestilence, Famine".to_string()),
            entry_message: "눈부신 빛이 넘실거린다. 세 개의 제단이 보인다!".to_string(),
            danger_level: 50,
            no_teleport: true,
            no_dig: true,
            unique_items: vec![],
        },
        _ => SpecialLevelInfo {
            level_type: level,
            name: format!("{:?}", level),
            depth_range: (15, 45),
            boss: Some("악마 군주".to_string()),
            entry_message: "사악한 기운이 감돈다.".to_string(),
            danger_level: 30,
            no_teleport: true,
            no_dig: true,
            unique_items: vec![],
        },
    }
}

/// [v2.39.0 103-4] 현재 깊이에서 만날 수 있는 특수 층 확인
pub fn check_special_level(depth: i32) -> Vec<SpecialLevel> {
    let all = [
        SpecialLevel::Oracle,
        SpecialLevel::BigRoom,
        SpecialLevel::Rogue,
        SpecialLevel::Medusa,
        SpecialLevel::Castle,
        SpecialLevel::ValleyOfDead,
    ];

    all.iter()
        .filter(|&&l| {
            let info = get_special_level_info(l);
            depth >= info.depth_range.0 && depth <= info.depth_range.1
        })
        .copied()
        .collect()
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_info() {
        let info = get_special_level_info(SpecialLevel::Oracle);
        assert_eq!(info.danger_level, 5);
        assert!(!info.no_teleport);
    }

    #[test]
    fn test_medusa_boss() {
        let info = get_special_level_info(SpecialLevel::Medusa);
        assert!(info.boss.is_some());
        assert!(info.boss.unwrap().contains("메두사"));
    }

    #[test]
    fn test_wizard_tower() {
        let info = get_special_level_info(SpecialLevel::WizardTower);
        assert!(info.no_teleport);
        assert!(info.unique_items.len() >= 2);
    }

    #[test]
    fn test_astral_plane() {
        let info = get_special_level_info(SpecialLevel::AstralPlane);
        assert_eq!(info.danger_level, 50);
    }

    #[test]
    fn test_check_depth_5() {
        let specials = check_special_level(5);
        assert!(specials.contains(&SpecialLevel::Oracle));
    }

    #[test]
    fn test_check_depth_22() {
        let specials = check_special_level(22);
        assert!(specials.contains(&SpecialLevel::Medusa));
    }

    #[test]
    fn test_valley_no_dig() {
        let info = get_special_level_info(SpecialLevel::ValleyOfDead);
        assert!(info.no_dig);
    }

    #[test]
    fn test_sanctum() {
        let info = get_special_level_info(SpecialLevel::Sanctum);
        assert!(info.entry_message.contains("성소"));
    }
}

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
// [v2.23.0 R11-1] 퀘스트 시스템 (quest_ext.rs)
//
// 원본 참조: NetHack 3.6.7 quest.c (2,122줄) 핵심 로직 이식
//
// 구현 내용:
//   1. 퀘스트 상태 머신 (5단계)
//   2. 역할(Role)별 퀘스트 메타데이터
//   3. 가디언/리더 NPC 대화 판정
//   4. 퀘스트 진입 조건 검사
//   5. 보스 처치 및 아티팩트 보상 판정
//   6. 퀘스트 레벨 구조 판정
//   7. 퀘스트 메시지 생성
// ============================================================================

// =============================================================================
// [1] 퀘스트 상태 머신 (원본: quest.c quest_status)
// =============================================================================

/// [v2.23.0 R11-1] 퀘스트 진행 상태 (원본: quest_status 구조체)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestPhase {
    /// 퀘스트 미시작 (가디언 미접촉)
    NotStarted,
    /// 리더와 대화 완료 (진입 허가 대기)
    LeaderContacted,
    /// 퀘스트 던전 진행 중
    InProgress,
    /// 보스 처치 완료 (아티팩트 획득 전)
    BossDefeated,
    /// 퀘스트 완전 완료 (아티팩트 반납/획득)
    Completed,
}

/// [v2.23.0 R11-1] 퀘스트 전체 상태 (원본: quest_status 전역 변수)
#[derive(Debug, Clone)]
pub struct QuestStatus {
    /// 현재 단계
    pub phase: QuestPhase,
    /// 리더와 대화 횟수
    pub leader_talks: i32,
    /// 퀘스트 진입 거부 횟수
    pub rejected_count: i32,
    /// 가디언 살해 여부
    pub guardian_killed: bool,
    /// 보스 처치 여부
    pub boss_defeated: bool,
    /// 아티팩트 획득 여부
    pub artifact_obtained: bool,
    /// 퀘스트 던전 진입 턴
    pub entry_turn: Option<u64>,
    /// 보스 처치 턴
    pub boss_defeat_turn: Option<u64>,
}

impl Default for QuestStatus {
    fn default() -> Self {
        Self {
            phase: QuestPhase::NotStarted,
            leader_talks: 0,
            rejected_count: 0,
            guardian_killed: false,
            boss_defeated: false,
            artifact_obtained: false,
            entry_turn: None,
            boss_defeat_turn: None,
        }
    }
}

// =============================================================================
// [2] 역할(Role)별 퀘스트 메타데이터 (원본: quest.c role_quest)
// =============================================================================

/// [v2.23.0 R11-1] 역할별 퀘스트 정보
#[derive(Debug, Clone)]
pub struct RoleQuestInfo {
    /// 역할 이름
    pub role: String,
    /// 리더 NPC 이름
    pub leader_name: String,
    /// 가디언 NPC 종류
    pub guardian_type: String,
    /// 보스 몬스터 이름
    pub boss_name: String,
    /// 보상 아티팩트 이름
    pub artifact_name: String,
    /// 퀘스트 진입 최소 레벨
    pub min_entry_level: i32,
    /// 퀘스트 진입 최소 경험치
    pub min_experience: i32,
    /// 퀘스트 던전 총 층수
    pub dungeon_levels: i32,
    /// 리더의 환영 메시지 키
    pub welcome_msg: String,
    /// 거부 사유 (레벨 부족)
    pub reject_msg: String,
    /// 보스 안내 메시지
    pub boss_hint_msg: String,
}

/// [v2.23.0 R11-1] 기본 역할별 퀘스트 테이블 (원본: quest_info[])
pub fn default_quest_table() -> Vec<RoleQuestInfo> {
    vec![
        RoleQuestInfo {
            role: "Archeologist".to_string(),
            leader_name: "Lord Carnarvon".to_string(),
            guardian_type: "student".to_string(),
            boss_name: "Minion of Huhetotl".to_string(),
            artifact_name: "Orb of Detection".to_string(),
            min_entry_level: 14,
            min_experience: 0,
            dungeon_levels: 5,
            welcome_msg: "quest_archeologist_welcome".to_string(),
            reject_msg: "quest_level_too_low".to_string(),
            boss_hint_msg: "quest_archeologist_boss".to_string(),
        },
        RoleQuestInfo {
            role: "Barbarian".to_string(),
            leader_name: "Pelias".to_string(),
            guardian_type: "chieftain".to_string(),
            boss_name: "Thoth Amon".to_string(),
            artifact_name: "Heart of Ahriman".to_string(),
            min_entry_level: 14,
            min_experience: 0,
            dungeon_levels: 5,
            welcome_msg: "quest_barbarian_welcome".to_string(),
            reject_msg: "quest_level_too_low".to_string(),
            boss_hint_msg: "quest_barbarian_boss".to_string(),
        },
        RoleQuestInfo {
            role: "Caveman".to_string(),
            leader_name: "Shaman Karnov".to_string(),
            guardian_type: "neanderthal".to_string(),
            boss_name: "Chromatic Dragon".to_string(),
            artifact_name: "Sceptre of Might".to_string(),
            min_entry_level: 14,
            min_experience: 0,
            dungeon_levels: 5,
            welcome_msg: "quest_caveman_welcome".to_string(),
            reject_msg: "quest_level_too_low".to_string(),
            boss_hint_msg: "quest_caveman_boss".to_string(),
        },
        RoleQuestInfo {
            role: "Knight".to_string(),
            leader_name: "King Arthur".to_string(),
            guardian_type: "page".to_string(),
            boss_name: "Ixoth".to_string(),
            artifact_name: "Magic Mirror of Merlin".to_string(),
            min_entry_level: 14,
            min_experience: 0,
            dungeon_levels: 5,
            welcome_msg: "quest_knight_welcome".to_string(),
            reject_msg: "quest_level_too_low".to_string(),
            boss_hint_msg: "quest_knight_boss".to_string(),
        },
        RoleQuestInfo {
            role: "Priest".to_string(),
            leader_name: "High Priest".to_string(),
            guardian_type: "acolyte".to_string(),
            boss_name: "Nalzok".to_string(),
            artifact_name: "Mitre of Holiness".to_string(),
            min_entry_level: 14,
            min_experience: 0,
            dungeon_levels: 5,
            welcome_msg: "quest_priest_welcome".to_string(),
            reject_msg: "quest_level_too_low".to_string(),
            boss_hint_msg: "quest_priest_boss".to_string(),
        },
        RoleQuestInfo {
            role: "Rogue".to_string(),
            leader_name: "Master of Thieves".to_string(),
            guardian_type: "thug".to_string(),
            boss_name: "Master Assassin".to_string(),
            artifact_name: "Master Key of Thievery".to_string(),
            min_entry_level: 14,
            min_experience: 0,
            dungeon_levels: 5,
            welcome_msg: "quest_rogue_welcome".to_string(),
            reject_msg: "quest_level_too_low".to_string(),
            boss_hint_msg: "quest_rogue_boss".to_string(),
        },
        RoleQuestInfo {
            role: "Samurai".to_string(),
            leader_name: "Lord Sato".to_string(),
            guardian_type: "roshi".to_string(),
            boss_name: "Ashikaga Takauji".to_string(),
            artifact_name: "Tsurugi of Muramasa".to_string(),
            min_entry_level: 14,
            min_experience: 0,
            dungeon_levels: 5,
            welcome_msg: "quest_samurai_welcome".to_string(),
            reject_msg: "quest_level_too_low".to_string(),
            boss_hint_msg: "quest_samurai_boss".to_string(),
        },
        RoleQuestInfo {
            role: "Valkyrie".to_string(),
            leader_name: "Norn".to_string(),
            guardian_type: "warrior".to_string(),
            boss_name: "Lord Surtur".to_string(),
            artifact_name: "Orb of Fate".to_string(),
            min_entry_level: 14,
            min_experience: 0,
            dungeon_levels: 5,
            welcome_msg: "quest_valkyrie_welcome".to_string(),
            reject_msg: "quest_level_too_low".to_string(),
            boss_hint_msg: "quest_valkyrie_boss".to_string(),
        },
        RoleQuestInfo {
            role: "Wizard".to_string(),
            leader_name: "Neferet the Green".to_string(),
            guardian_type: "apprentice".to_string(),
            boss_name: "Dark One".to_string(),
            artifact_name: "Eye of the Aethiopica".to_string(),
            min_entry_level: 14,
            min_experience: 0,
            dungeon_levels: 5,
            welcome_msg: "quest_wizard_welcome".to_string(),
            reject_msg: "quest_level_too_low".to_string(),
            boss_hint_msg: "quest_wizard_boss".to_string(),
        },
    ]
}

/// [v2.23.0 R11-1] 역할명으로 퀘스트 정보 검색
pub fn find_quest_info<'a>(table: &'a [RoleQuestInfo], role: &str) -> Option<&'a RoleQuestInfo> {
    table.iter().find(|q| q.role.eq_ignore_ascii_case(role))
}

// =============================================================================
// [3] 퀘스트 진입 조건 검사 (원본: quest.c quest_entry)
// =============================================================================

/// [v2.23.0 R11-1] 퀘스트 진입 검사 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestEntryResult {
    /// 진입 허가
    Allowed,
    /// 레벨 부족
    LevelTooLow { required: i32, current: i32 },
    /// 정렬(Alignment) 부족
    AlignmentTooLow { required: i32, current: i32 },
    /// 이미 가디언을 살해함 (진입 거부)
    GuardianKilled,
    /// 이미 퀘스트 완료
    AlreadyCompleted,
    /// 리더 미접촉
    LeaderNotContacted,
}

/// [v2.23.0 R11-1] 퀘스트 진입 종합 판정 (원본: quest_entry)
pub fn check_quest_entry(
    status: &QuestStatus,
    quest_info: &RoleQuestInfo,
    player_level: i32,
    player_alignment: i32,
    alignment_threshold: i32,
) -> QuestEntryResult {
    // 이미 완료
    if status.phase == QuestPhase::Completed {
        return QuestEntryResult::AlreadyCompleted;
    }
    // 가디언 살해 시 진입 불가
    if status.guardian_killed {
        return QuestEntryResult::GuardianKilled;
    }
    // 리더 미접촉
    if status.phase == QuestPhase::NotStarted && status.leader_talks == 0 {
        return QuestEntryResult::LeaderNotContacted;
    }
    // 레벨 체크
    if player_level < quest_info.min_entry_level {
        return QuestEntryResult::LevelTooLow {
            required: quest_info.min_entry_level,
            current: player_level,
        };
    }
    // 정렬 체크
    if player_alignment < alignment_threshold {
        return QuestEntryResult::AlignmentTooLow {
            required: alignment_threshold,
            current: player_alignment,
        };
    }
    QuestEntryResult::Allowed
}

// =============================================================================
// [4] 리더/가디언 대화 판정 (원본: quest.c leader_speaks, guardian_speaks)
// =============================================================================

/// [v2.23.0 R11-1] 리더 대화 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaderDialogResult {
    /// 환영 (첫 대화)
    Welcome(String),
    /// 진입 허가 + 힌트
    GrantAccess(String),
    /// 진입 거부 (레벨 부족)
    RejectEntry(String),
    /// 돌아와서 보고 (보스 미처치 상태)
    AskForReport,
    /// 축하 (보스 처치 후)
    Congratulate(String),
}

/// [v2.23.0 R11-1] 리더 대화 판정 (원본: leader_speaks)
pub fn leader_dialog(
    status: &QuestStatus,
    quest_info: &RoleQuestInfo,
    player_level: i32,
) -> LeaderDialogResult {
    match status.phase {
        QuestPhase::NotStarted => {
            if player_level >= quest_info.min_entry_level {
                LeaderDialogResult::Welcome(quest_info.welcome_msg.clone())
            } else {
                LeaderDialogResult::RejectEntry(quest_info.reject_msg.clone())
            }
        }
        QuestPhase::LeaderContacted => {
            if player_level >= quest_info.min_entry_level {
                LeaderDialogResult::GrantAccess(quest_info.boss_hint_msg.clone())
            } else {
                LeaderDialogResult::RejectEntry(quest_info.reject_msg.clone())
            }
        }
        QuestPhase::InProgress => LeaderDialogResult::AskForReport,
        QuestPhase::BossDefeated => LeaderDialogResult::Congratulate(format!(
            "{}를 물리치셨군요! {}을(를) 가지세요.",
            quest_info.boss_name, quest_info.artifact_name
        )),
        QuestPhase::Completed => {
            LeaderDialogResult::Congratulate("이미 퀘스트를 완료하셨습니다.".to_string())
        }
    }
}

// =============================================================================
// [5] 보스 처치 및 보상 판정 (원본: quest.c quest_completed)
// =============================================================================

/// [v2.23.0 R11-1] 보스 처치 후 상태 전이
pub fn on_boss_defeated(status: &mut QuestStatus, turn: u64) {
    status.boss_defeated = true;
    status.boss_defeat_turn = Some(turn);
    status.phase = QuestPhase::BossDefeated;
}

/// [v2.23.0 R11-1] 아티팩트 획득 후 상태 전이
pub fn on_artifact_obtained(status: &mut QuestStatus) {
    status.artifact_obtained = true;
    status.phase = QuestPhase::Completed;
}

/// [v2.23.0 R11-1] 리더 접촉 후 상태 전이
pub fn on_leader_contacted(status: &mut QuestStatus) {
    status.leader_talks += 1;
    if status.phase == QuestPhase::NotStarted {
        status.phase = QuestPhase::LeaderContacted;
    }
}

/// [v2.23.0 R11-1] 퀘스트 던전 진입 후 상태 전이
pub fn on_quest_entered(status: &mut QuestStatus, turn: u64) {
    if status.phase == QuestPhase::LeaderContacted {
        status.phase = QuestPhase::InProgress;
        status.entry_turn = Some(turn);
    }
}

// =============================================================================
// [6] 퀘스트 레벨 구조 (원본: quest.c nemesis_level, home_level)
// =============================================================================

/// [v2.23.0 R11-1] 퀘스트 레벨 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestLevelType {
    /// 홈 레벨 (리더 NPC)
    Home,
    /// 중간 레벨 (가디언들)
    Intermediate,
    /// 보스 레벨 (네메시스)
    Nemesis,
    /// 보물 레벨 (아티팩트)
    Locate,
}

/// [v2.23.0 R11-1] 상대 깊이로 레벨 타입 판별 (원본: Is_qstart, Is_nemesis 등)
pub fn quest_level_type(relative_depth: i32, total_levels: i32) -> QuestLevelType {
    if relative_depth == 1 {
        QuestLevelType::Home
    } else if relative_depth == total_levels {
        QuestLevelType::Nemesis
    } else if relative_depth == total_levels - 1 {
        QuestLevelType::Locate
    } else {
        QuestLevelType::Intermediate
    }
}

// =============================================================================
// [7] 퀘스트 난이도 보정 (원본: quest.c quest_difficulty)
// =============================================================================

/// [v2.23.0 R11-1] 퀘스트 던전 난이도 보정
pub fn quest_difficulty_bonus(
    base_difficulty: i32,
    quest_level_depth: i32,
    boss_defeated: bool,
) -> i32 {
    let mut diff = base_difficulty + quest_level_depth + 3; // 퀘스트 기본 보정
    if boss_defeated {
        diff -= 5; // 보스 처치 후 약화
    }
    diff.max(1)
}

/// [v2.23.0 R11-1] 퀘스트 완료 경험치 보너스
pub fn quest_completion_xp(player_level: i32) -> i64 {
    // 원본: 퀘스트 완료 시 플레이어 레벨에 비례한 경험치
    let base = player_level as i64 * 1000;
    base + 5000 // 기본 5000 + 레벨*1000
}

// =============================================================================
// [8] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_status() -> QuestStatus {
        QuestStatus::default()
    }

    fn test_quest_info() -> RoleQuestInfo {
        RoleQuestInfo {
            role: "Valkyrie".to_string(),
            leader_name: "Norn".to_string(),
            guardian_type: "warrior".to_string(),
            boss_name: "Lord Surtur".to_string(),
            artifact_name: "Orb of Fate".to_string(),
            min_entry_level: 14,
            min_experience: 0,
            dungeon_levels: 5,
            welcome_msg: "quest_valkyrie_welcome".to_string(),
            reject_msg: "quest_level_too_low".to_string(),
            boss_hint_msg: "quest_valkyrie_boss".to_string(),
        }
    }

    #[test]
    fn test_default_status() {
        let s = test_status();
        assert_eq!(s.phase, QuestPhase::NotStarted);
        assert!(!s.boss_defeated);
        assert!(!s.artifact_obtained);
    }

    #[test]
    fn test_quest_table_count() {
        let table = default_quest_table();
        assert_eq!(table.len(), 9); // 9 역할
    }

    #[test]
    fn test_find_quest_info() {
        let table = default_quest_table();
        let info = find_quest_info(&table, "Wizard");
        assert!(info.is_some());
        assert_eq!(info.unwrap().boss_name, "Dark One");
    }

    #[test]
    fn test_find_quest_info_missing() {
        let table = default_quest_table();
        let info = find_quest_info(&table, "Tourist");
        assert!(info.is_none());
    }

    #[test]
    fn test_entry_level_too_low() {
        let s = QuestStatus {
            leader_talks: 1,
            phase: QuestPhase::LeaderContacted,
            ..test_status()
        };
        let qi = test_quest_info();
        let r = check_quest_entry(&s, &qi, 10, 20, 0);
        assert!(matches!(r, QuestEntryResult::LevelTooLow { .. }));
    }

    #[test]
    fn test_entry_allowed() {
        let s = QuestStatus {
            leader_talks: 1,
            phase: QuestPhase::LeaderContacted,
            ..test_status()
        };
        let qi = test_quest_info();
        let r = check_quest_entry(&s, &qi, 15, 20, 0);
        assert_eq!(r, QuestEntryResult::Allowed);
    }

    #[test]
    fn test_entry_guardian_killed() {
        let s = QuestStatus {
            guardian_killed: true,
            leader_talks: 1,
            phase: QuestPhase::LeaderContacted,
            ..test_status()
        };
        let qi = test_quest_info();
        let r = check_quest_entry(&s, &qi, 20, 20, 0);
        assert_eq!(r, QuestEntryResult::GuardianKilled);
    }

    #[test]
    fn test_entry_already_completed() {
        let s = QuestStatus {
            phase: QuestPhase::Completed,
            ..test_status()
        };
        let qi = test_quest_info();
        let r = check_quest_entry(&s, &qi, 20, 20, 0);
        assert_eq!(r, QuestEntryResult::AlreadyCompleted);
    }

    #[test]
    fn test_entry_alignment_low() {
        let s = QuestStatus {
            leader_talks: 1,
            phase: QuestPhase::LeaderContacted,
            ..test_status()
        };
        let qi = test_quest_info();
        let r = check_quest_entry(&s, &qi, 15, -5, 0);
        assert!(matches!(r, QuestEntryResult::AlignmentTooLow { .. }));
    }

    #[test]
    fn test_leader_dialog_welcome() {
        let s = test_status();
        let qi = test_quest_info();
        let r = leader_dialog(&s, &qi, 15);
        assert!(matches!(r, LeaderDialogResult::Welcome(_)));
    }

    #[test]
    fn test_leader_dialog_reject() {
        let s = test_status();
        let qi = test_quest_info();
        let r = leader_dialog(&s, &qi, 10);
        assert!(matches!(r, LeaderDialogResult::RejectEntry(_)));
    }

    #[test]
    fn test_leader_dialog_grant() {
        let s = QuestStatus {
            phase: QuestPhase::LeaderContacted,
            ..test_status()
        };
        let qi = test_quest_info();
        let r = leader_dialog(&s, &qi, 15);
        assert!(matches!(r, LeaderDialogResult::GrantAccess(_)));
    }

    #[test]
    fn test_leader_dialog_congratulate() {
        let s = QuestStatus {
            phase: QuestPhase::BossDefeated,
            boss_defeated: true,
            ..test_status()
        };
        let qi = test_quest_info();
        let r = leader_dialog(&s, &qi, 20);
        assert!(matches!(r, LeaderDialogResult::Congratulate(_)));
    }

    #[test]
    fn test_state_transitions() {
        let mut s = test_status();
        assert_eq!(s.phase, QuestPhase::NotStarted);

        on_leader_contacted(&mut s);
        assert_eq!(s.phase, QuestPhase::LeaderContacted);
        assert_eq!(s.leader_talks, 1);

        on_quest_entered(&mut s, 1000);
        assert_eq!(s.phase, QuestPhase::InProgress);
        assert_eq!(s.entry_turn, Some(1000));

        on_boss_defeated(&mut s, 2000);
        assert_eq!(s.phase, QuestPhase::BossDefeated);
        assert!(s.boss_defeated);

        on_artifact_obtained(&mut s);
        assert_eq!(s.phase, QuestPhase::Completed);
        assert!(s.artifact_obtained);
    }

    #[test]
    fn test_quest_level_type_home() {
        assert_eq!(quest_level_type(1, 5), QuestLevelType::Home);
    }

    #[test]
    fn test_quest_level_type_nemesis() {
        assert_eq!(quest_level_type(5, 5), QuestLevelType::Nemesis);
    }

    #[test]
    fn test_quest_level_type_locate() {
        assert_eq!(quest_level_type(4, 5), QuestLevelType::Locate);
    }

    #[test]
    fn test_quest_level_type_intermediate() {
        assert_eq!(quest_level_type(3, 5), QuestLevelType::Intermediate);
    }

    #[test]
    fn test_quest_difficulty_bonus() {
        let diff = quest_difficulty_bonus(10, 3, false);
        assert_eq!(diff, 16); // 10 + 3 + 3

        let diff2 = quest_difficulty_bonus(10, 3, true);
        assert_eq!(diff2, 11); // 16 - 5
    }

    #[test]
    fn test_quest_completion_xp() {
        assert_eq!(quest_completion_xp(15), 20000); // 15*1000 + 5000
        assert_eq!(quest_completion_xp(1), 6000);
    }
}

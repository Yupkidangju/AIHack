// ============================================================================
// [v2.37.0 Phase 101-1] 퀘스트/목표 통합 (quest_phase101_ext.rs)
// 원본: NetHack 3.6.7 src/quest.c + questpgr.c 핵심 미이식 함수
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 퀘스트 시스템 — quest_system (quest.c 핵심)
// =============================================================================

/// [v2.37.0 101-1] 퀘스트 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestStatus {
    NotStarted,
    Assigned,
    InProgress,
    NemesisDefeated,
    ArtifactObtained,
    Completed,
    Failed,
}

/// [v2.37.0 101-1] 퀘스트 정보
#[derive(Debug, Clone)]
pub struct QuestInfo {
    pub role: String,
    pub leader_name: String,
    pub nemesis_name: String,
    pub artifact_name: String,
    pub home_level: String,
    pub goal_level: String,
    pub status: QuestStatus,
    pub times_visited: i32,
    pub nemesis_hp: i32,
    pub player_admitted: bool,
}

/// [v2.37.0 101-1] 역할별 퀘스트 생성
pub fn create_quest(role: &str) -> QuestInfo {
    match role {
        "전사" | "barbarian" => QuestInfo {
            role: role.to_string(),
            leader_name: "펠로그난드".to_string(),
            nemesis_name: "서르투르".to_string(),
            artifact_name: "Heart of Ahriman".to_string(),
            home_level: "바바리안 마을".to_string(),
            goal_level: "서르투르의 요새".to_string(),
            status: QuestStatus::NotStarted,
            times_visited: 0,
            nemesis_hp: 500,
            player_admitted: false,
        },
        "마법사" | "wizard" => QuestInfo {
            role: role.to_string(),
            leader_name: "넨쯔가르".to_string(),
            nemesis_name: "다크원".to_string(),
            artifact_name: "Eye of the Aethiopica".to_string(),
            home_level: "마법사 탑".to_string(),
            goal_level: "다크원의 은신처".to_string(),
            status: QuestStatus::NotStarted,
            times_visited: 0,
            nemesis_hp: 400,
            player_admitted: false,
        },
        "기사" | "knight" => QuestInfo {
            role: role.to_string(),
            leader_name: "킹 아서".to_string(),
            nemesis_name: "이쓸디".to_string(),
            artifact_name: "Magic Mirror of Merlin".to_string(),
            home_level: "카멜롯".to_string(),
            goal_level: "이쓸디의 성".to_string(),
            status: QuestStatus::NotStarted,
            times_visited: 0,
            nemesis_hp: 450,
            player_admitted: false,
        },
        "발키리" | "valkyrie" => QuestInfo {
            role: role.to_string(),
            leader_name: "님프".to_string(),
            nemesis_name: "로드 서르투르".to_string(),
            artifact_name: "Orb of Fate".to_string(),
            home_level: "발할라".to_string(),
            goal_level: "서르투르의 영역".to_string(),
            status: QuestStatus::NotStarted,
            times_visited: 0,
            nemesis_hp: 550,
            player_admitted: false,
        },
        _ => QuestInfo {
            role: role.to_string(),
            leader_name: "멘토".to_string(),
            nemesis_name: "악당".to_string(),
            artifact_name: "Quest Artifact".to_string(),
            home_level: "고향".to_string(),
            goal_level: "최종 목표".to_string(),
            status: QuestStatus::NotStarted,
            times_visited: 0,
            nemesis_hp: 300,
            player_admitted: false,
        },
    }
}

/// [v2.37.0 101-1] 입장 자격 확인
pub fn check_quest_admission(
    player_level: i32,
    player_alignment_record: i32,
    quest: &QuestInfo,
) -> (bool, String) {
    if quest.status != QuestStatus::NotStarted && quest.status != QuestStatus::Assigned {
        return (false, "이미 퀘스트가 진행 중입니다.".to_string());
    }
    if player_level < 14 {
        return (
            false,
            format!("레벨이 부족합니다. (현재: {}, 필요: 14)", player_level),
        );
    }
    if player_alignment_record < 0 {
        return (false, "성향 기록이 부족합니다.".to_string());
    }
    (
        true,
        format!("{}이(가) 퀘스트를 부여합니다!", quest.leader_name),
    )
}

/// [v2.37.0 101-1] 네메시스 전투
pub fn fight_nemesis(
    quest: &QuestInfo,
    player_damage: i32,
    rng: &mut NetHackRng,
) -> (bool, i32, String) {
    let nemesis_damage = rng.rn2(30) + 15;
    let remaining_hp = quest.nemesis_hp - player_damage;

    if remaining_hp <= 0 {
        (
            true,
            0,
            format!(
                "{}을(를) 물리쳤다! {}을(를) 획득!",
                quest.nemesis_name, quest.artifact_name
            ),
        )
    } else {
        (
            false,
            nemesis_damage,
            format!(
                "{}이(가) {}의 데미지를 입혔다! (남은 HP: {})",
                quest.nemesis_name, nemesis_damage, remaining_hp
            ),
        )
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

    #[test]
    fn test_create_warrior_quest() {
        let q = create_quest("전사");
        assert_eq!(q.status, QuestStatus::NotStarted);
        assert!(!q.leader_name.is_empty());
    }

    #[test]
    fn test_create_wizard_quest() {
        let q = create_quest("마법사");
        assert!(q.artifact_name.contains("Eye"));
    }

    #[test]
    fn test_admission_ok() {
        let q = create_quest("전사");
        let (ok, _) = check_quest_admission(15, 10, &q);
        assert!(ok);
    }

    #[test]
    fn test_admission_low_level() {
        let q = create_quest("전사");
        let (ok, msg) = check_quest_admission(10, 10, &q);
        assert!(!ok);
        assert!(msg.contains("레벨"));
    }

    #[test]
    fn test_admission_bad_alignment() {
        let q = create_quest("전사");
        let (ok, _) = check_quest_admission(15, -5, &q);
        assert!(!ok);
    }

    #[test]
    fn test_fight_win() {
        let q = create_quest("전사");
        let mut rng = test_rng();
        let (won, _, msg) = fight_nemesis(&q, 600, &mut rng);
        assert!(won);
        assert!(msg.contains("물리쳤다"));
    }

    #[test]
    fn test_fight_lose() {
        let q = create_quest("전사");
        let mut rng = test_rng();
        let (won, dmg, _) = fight_nemesis(&q, 100, &mut rng);
        assert!(!won);
        assert!(dmg > 0);
    }

    #[test]
    fn test_default_quest() {
        let q = create_quest("관광객");
        assert_eq!(q.nemesis_hp, 300);
    }
}

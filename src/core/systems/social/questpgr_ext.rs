// ============================================================================
// [v2.29.0 R17-1] 퀘스트 대사 시스템 (questpgr_ext.rs)
// 원본: NetHack 3.6.7 questpgr.c (685줄)
// 퀘스트 NPC 대사 템플릿, 변수 치환, 진행 기반 대사 분기
// ============================================================================

use std::collections::HashMap;

// =============================================================================
// [1] 대사 변수 치환 (원본: questpgr.c qtext_pronoun)
// =============================================================================

/// [v2.29.0 R17-1] 대사 템플릿 변수 치환
pub fn substitute_vars(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("%{}", key), value);
    }
    result
}

/// [v2.29.0 R17-1] 기본 변수 세트 (원본: qtext 치환 매크로)
pub fn default_quest_vars(
    player_name: &str,
    role: &str,
    deity: &str,
    nemesis: &str,
    artifact: &str,
) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert("n".to_string(), player_name.to_string());
    vars.insert("r".to_string(), role.to_string());
    vars.insert("g".to_string(), deity.to_string());
    vars.insert("N".to_string(), nemesis.to_string());
    vars.insert("A".to_string(), artifact.to_string());
    vars
}

// =============================================================================
// [2] 퀘스트 대사 단계 (원본: questpgr.c qt_com_firstline)
// =============================================================================

/// [v2.29.0 R17-1] 퀘스트 대사 단계
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuestDialogStage {
    /// 첫 방문 (리더 소개)
    FirstVisit,
    /// 자격 불충분
    NotReady,
    /// 퀘스트 부여
    Briefing,
    /// 퀘스트 중 격려
    Encouragement,
    /// 보스 조우
    NemesisEncounter,
    /// 아티팩트 획득
    ArtifactObtained,
    /// 완료 귀환
    QuestComplete,
}

/// [v2.29.0 R17-1] 역할별 대사 데이터
#[derive(Debug, Clone)]
pub struct QuestDialogSet {
    pub role: String,
    pub leader_name: String,
    pub nemesis_name: String,
    pub artifact_name: String,
    pub deity_name: String,
    pub dialogs: HashMap<QuestDialogStage, String>,
}

/// [v2.29.0 R17-1] 대사 조회
pub fn get_dialog(
    dialog_set: &QuestDialogSet,
    stage: QuestDialogStage,
    player_name: &str,
) -> String {
    let template = dialog_set
        .dialogs
        .get(&stage)
        .cloned()
        .unwrap_or_else(|| format!("{} has nothing to say.", dialog_set.leader_name));

    let vars = default_quest_vars(
        player_name,
        &dialog_set.role,
        &dialog_set.deity_name,
        &dialog_set.nemesis_name,
        &dialog_set.artifact_name,
    );
    substitute_vars(&template, &vars)
}

// =============================================================================
// [3] 자격 판정 (원본: questpgr.c not_capable)
// =============================================================================

/// [v2.29.0 R17-1] 퀘스트 자격 판정
pub fn quest_eligible(
    player_level: i32,
    alignment_record: i32,
    min_level: i32,
    min_alignment: i32,
) -> Result<(), String> {
    if player_level < min_level {
        return Err(format!(
            "레벨 {}이 필요합니다 (현재: {})",
            min_level, player_level
        ));
    }
    if alignment_record < min_alignment {
        return Err(format!(
            "정렬 기록 {}이 필요합니다 (현재: {})",
            min_alignment, alignment_record
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute() {
        let mut vars = HashMap::new();
        vars.insert("n".to_string(), "Alice".to_string());
        vars.insert("r".to_string(), "Valkyrie".to_string());
        let result = substitute_vars("Welcome, %n the %r!", &vars);
        assert_eq!(result, "Welcome, Alice the Valkyrie!");
    }

    #[test]
    fn test_quest_eligible_ok() {
        assert!(quest_eligible(14, 10, 14, 5).is_ok());
    }

    #[test]
    fn test_quest_eligible_level() {
        assert!(quest_eligible(10, 10, 14, 5).is_err());
    }

    #[test]
    fn test_quest_eligible_alignment() {
        assert!(quest_eligible(14, 2, 14, 5).is_err());
    }

    #[test]
    fn test_get_dialog() {
        let mut dialogs = HashMap::new();
        dialogs.insert(
            QuestDialogStage::Briefing,
            "Go forth, %n! Defeat %N and retrieve the %A!".to_string(),
        );
        let ds = QuestDialogSet {
            role: "Knight".to_string(),
            leader_name: "King Arthur".to_string(),
            nemesis_name: "Ixoth".to_string(),
            artifact_name: "Magic Mirror".to_string(),
            deity_name: "Lugh".to_string(),
            dialogs,
        };
        let text = get_dialog(&ds, QuestDialogStage::Briefing, "Alice");
        assert!(text.contains("Alice"));
        assert!(text.contains("Ixoth"));
        assert!(text.contains("Magic Mirror"));
    }
}

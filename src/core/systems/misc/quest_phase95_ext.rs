// ============================================================================
// [v2.31.0 Phase 95-1] 퀘스트 시스템 확장 (quest_phase95_ext.rs)
// 원본: NetHack 3.6.7 src/quest.c + questpgr.c L200-1200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 퀘스트 진행 — quest_progress (quest.c L200-600)
// =============================================================================

/// [v2.31.0 95-1] 퀘스트 단계
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestStage {
    NotStarted,
    LeaderMet,
    QuestGiven,
    InProgress,
    NemesisFound,
    NemesisDefeated,
    ArtifactObtained,
    Completed,
    Failed,
}

/// [v2.31.0 95-1] 퀘스트 정보
#[derive(Debug, Clone)]
pub struct QuestInfo {
    pub leader_name: String,
    pub nemesis_name: String,
    pub artifact_name: String,
    pub stage: QuestStage,
    pub alignment_required: i32,
    pub level_required: i32,
    pub attempts: i32,
}

/// [v2.31.0 95-1] 리더 대화 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaderTalkResult {
    Welcome { message: String },
    QuestOffered { description: String },
    NotWorthy { reason: String },
    AlreadyOnQuest,
    QuestComplete { reward: String },
    Disappointed,
}

/// [v2.31.0 95-1] 리더와 대화
/// 원본: quest.c chat_with_leader()
pub fn talk_to_leader(
    quest: &QuestInfo,
    player_level: i32,
    player_alignment: i32,
    _rng: &mut NetHackRng,
) -> LeaderTalkResult {
    match quest.stage {
        QuestStage::NotStarted => LeaderTalkResult::Welcome {
            message: format!("환영한다, 젊은 모험자여. 나는 {}이다.", quest.leader_name),
        },
        QuestStage::LeaderMet => {
            if player_level < quest.level_required {
                return LeaderTalkResult::NotWorthy {
                    reason: format!(
                        "그대는 아직 준비가 되지 않았다. 레벨 {} 이상이 필요하다.",
                        quest.level_required
                    ),
                };
            }
            if player_alignment < quest.alignment_required {
                return LeaderTalkResult::NotWorthy {
                    reason: "그대의 행실이 부족하다.".to_string(),
                };
            }
            LeaderTalkResult::QuestOffered {
                description: format!(
                    "{}을(를) 찾아 {}을(를) 물리치거라.",
                    quest.artifact_name, quest.nemesis_name
                ),
            }
        }
        QuestStage::QuestGiven | QuestStage::InProgress => LeaderTalkResult::AlreadyOnQuest,
        QuestStage::ArtifactObtained | QuestStage::Completed => LeaderTalkResult::QuestComplete {
            reward: "축복과 보상".to_string(),
        },
        _ => LeaderTalkResult::Disappointed,
    }
}

// =============================================================================
// [2] 네메시스 전투 — nemesis_encounter (quest.c L600-900)
// =============================================================================

/// [v2.31.0 95-1] 네메시스 전투 특수 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NemesisEffect {
    SummonMinions { count: i32 },
    TeleportAway,
    HealSelf { amount: i32 },
    CurseEquipment,
    SpecialAttack { damage: i32, element: String },
    Taunt { message: String },
    DropArtifact,
}

/// [v2.31.0 95-1] 네메시스 특수 행동
pub fn nemesis_special_action(
    nemesis_hp_pct: i32,
    turn_count: i32,
    rng: &mut NetHackRng,
) -> NemesisEffect {
    // HP 낮으면 회복 시도
    if nemesis_hp_pct < 20 {
        if rng.rn2(3) == 0 {
            return NemesisEffect::TeleportAway;
        }
        return NemesisEffect::HealSelf {
            amount: rng.rn2(30) + 20,
        };
    }

    // 주기적 소환
    if turn_count % 10 == 0 && rng.rn2(3) == 0 {
        return NemesisEffect::SummonMinions {
            count: rng.rn2(3) + 1,
        };
    }

    // 장비 저주
    if rng.rn2(10) == 0 {
        return NemesisEffect::CurseEquipment;
    }

    // 도발
    if rng.rn2(5) == 0 {
        return NemesisEffect::Taunt {
            message: "어리석은 놈! 네가 나를 이길 수 있을 것 같으냐?".to_string(),
        };
    }

    // 특수 공격
    NemesisEffect::SpecialAttack {
        damage: rng.rn2(20) + 10,
        element: "마법".to_string(),
    }
}

// =============================================================================
// [3] 퀘스트 보상 — quest_reward (questpgr.c L200-400)
// =============================================================================

/// [v2.31.0 95-1] 퀘스트 보상
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestReward {
    pub artifact_name: String,
    pub stat_bonus: Vec<(String, i32)>,
    pub resistance_gained: Vec<String>,
    pub title: String,
}

/// [v2.31.0 95-1] 퀘스트 완료 보상 계산
pub fn calculate_quest_reward(role: &str, _rng: &mut NetHackRng) -> QuestReward {
    match role {
        "전사" | "Barbarian" => QuestReward {
            artifact_name: "Heart of Ahriman".to_string(),
            stat_bonus: vec![("STR".to_string(), 3), ("CON".to_string(), 2)],
            resistance_gained: vec!["독".to_string()],
            title: "대전사장".to_string(),
        },
        "마법사" | "Wizard" => QuestReward {
            artifact_name: "Eye of the Aethiopica".to_string(),
            stat_bonus: vec![("INT".to_string(), 3), ("WIS".to_string(), 2)],
            resistance_gained: vec!["마법".to_string()],
            title: "대마법사".to_string(),
        },
        "기사" | "Knight" => QuestReward {
            artifact_name: "Magic Mirror of Merlin".to_string(),
            stat_bonus: vec![("CHA".to_string(), 3), ("WIS".to_string(), 2)],
            resistance_gained: vec!["반사".to_string()],
            title: "성기사".to_string(),
        },
        "도적" | "Rogue" => QuestReward {
            artifact_name: "Master Key of Thievery".to_string(),
            stat_bonus: vec![("DEX".to_string(), 3)],
            resistance_gained: vec!["경고".to_string()],
            title: "대도적".to_string(),
        },
        _ => QuestReward {
            artifact_name: "퀘스트 아티팩트".to_string(),
            stat_bonus: vec![("STR".to_string(), 1)],
            resistance_gained: vec![],
            title: "영웅".to_string(),
        },
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

    fn test_quest() -> QuestInfo {
        QuestInfo {
            leader_name: "넬쥐".to_string(),
            nemesis_name: "다크 원".to_string(),
            artifact_name: "아에티오피카의 눈".to_string(),
            stage: QuestStage::LeaderMet,
            alignment_required: 10,
            level_required: 14,
            attempts: 0,
        }
    }

    #[test]
    fn test_leader_welcome() {
        let mut rng = test_rng();
        let mut quest = test_quest();
        quest.stage = QuestStage::NotStarted;
        let result = talk_to_leader(&quest, 15, 20, &mut rng);
        assert!(matches!(result, LeaderTalkResult::Welcome { .. }));
    }

    #[test]
    fn test_leader_quest_offer() {
        let mut rng = test_rng();
        let quest = test_quest();
        let result = talk_to_leader(&quest, 15, 20, &mut rng);
        assert!(matches!(result, LeaderTalkResult::QuestOffered { .. }));
    }

    #[test]
    fn test_leader_not_worthy() {
        let mut rng = test_rng();
        let quest = test_quest();
        let result = talk_to_leader(&quest, 5, 20, &mut rng); // 레벨 부족
        assert!(matches!(result, LeaderTalkResult::NotWorthy { .. }));
    }

    #[test]
    fn test_nemesis_heal() {
        let mut healed = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = nemesis_special_action(15, 5, &mut rng);
            if matches!(result, NemesisEffect::HealSelf { .. }) {
                healed = true;
                break;
            }
        }
        assert!(healed);
    }

    #[test]
    fn test_nemesis_teleport() {
        let mut teleported = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = nemesis_special_action(10, 5, &mut rng);
            if matches!(result, NemesisEffect::TeleportAway) {
                teleported = true;
                break;
            }
        }
        assert!(teleported);
    }

    #[test]
    fn test_quest_reward_wizard() {
        let mut rng = test_rng();
        let reward = calculate_quest_reward("마법사", &mut rng);
        assert!(reward.artifact_name.contains("Aethiopica"));
        assert_eq!(reward.title, "대마법사");
    }

    #[test]
    fn test_quest_reward_knight() {
        let mut rng = test_rng();
        let reward = calculate_quest_reward("기사", &mut rng);
        assert!(reward.artifact_name.contains("Merlin"));
    }
}

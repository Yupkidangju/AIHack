// ============================================================================
// [v2.34.0 R22-5] 퀘스트 분기 판정 (quest_branch_ext.rs)
// 원본: NetHack 3.6.7 quest.c/questpgr.c 분기 확장
// 역할별 퀘스트 분기, 보상, 아티팩트 조건
// ============================================================================

/// [v2.34.0 R22-5] 퀘스트 분기 데이터
#[derive(Debug, Clone)]
pub struct QuestBranch {
    pub role: String,
    pub leader: String,
    pub nemesis: String,
    pub artifact: String,
    pub home_level: String,
    pub goal_level: String,
    pub min_level: i32,
    pub min_alignment: i32,
}

/// [v2.34.0 R22-5] 역할별 퀘스트 테이블
pub fn quest_branches() -> Vec<QuestBranch> {
    vec![
        QuestBranch {
            role: "Archeologist".into(),
            leader: "Lord Carnarvon".into(),
            nemesis: "Minion of Huhetotl".into(),
            artifact: "Orb of Detection".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Barbarian".into(),
            leader: "Pelias".into(),
            nemesis: "Thoth Amon".into(),
            artifact: "Heart of Ahriman".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Caveman".into(),
            leader: "Shaman Karnov".into(),
            nemesis: "Chromatic Dragon".into(),
            artifact: "Sceptre of Might".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Knight".into(),
            leader: "King Arthur".into(),
            nemesis: "Ixoth".into(),
            artifact: "Magic Mirror of Merlin".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Monk".into(),
            leader: "Grand Master".into(),
            nemesis: "Master Kaen".into(),
            artifact: "Eyes of the Overworld".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Priest".into(),
            leader: "Arch Priest".into(),
            nemesis: "Nalzok".into(),
            artifact: "Mitre of Holiness".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Ranger".into(),
            leader: "Orion".into(),
            nemesis: "Scorpius".into(),
            artifact: "Longbow of Diana".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Rogue".into(),
            leader: "Master of Thieves".into(),
            nemesis: "Master Assassin".into(),
            artifact: "Master Key of Thievery".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Samurai".into(),
            leader: "Lord Sato".into(),
            nemesis: "Ashikaga Takauji".into(),
            artifact: "Tsurugi of Muramasa".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Tourist".into(),
            leader: "Twoflower".into(),
            nemesis: "Master of Thieves".into(),
            artifact: "Platinum Yendorian Express Card".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Valkyrie".into(),
            leader: "Norn".into(),
            nemesis: "Lord Surtur".into(),
            artifact: "Orb of Fate".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
        QuestBranch {
            role: "Wizard".into(),
            leader: "Neferet the Green".into(),
            nemesis: "Dark One".into(),
            artifact: "Eye of the Aethiopica".into(),
            home_level: "quest_home".into(),
            goal_level: "quest_goal".into(),
            min_level: 14,
            min_alignment: 10,
        },
    ]
}

/// [v2.34.0 R22-5] 역할로 퀘스트 분기 조회
pub fn get_quest_for_role(role: &str) -> Option<QuestBranch> {
    quest_branches()
        .into_iter()
        .find(|q| q.role.eq_ignore_ascii_case(role))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_roles() {
        assert_eq!(quest_branches().len(), 12);
    }

    #[test]
    fn test_knight() {
        let q = get_quest_for_role("Knight").unwrap();
        assert_eq!(q.leader, "King Arthur");
        assert_eq!(q.nemesis, "Ixoth");
    }

    #[test]
    fn test_wizard() {
        let q = get_quest_for_role("Wizard").unwrap();
        assert_eq!(q.artifact, "Eye of the Aethiopica");
    }

    #[test]
    fn test_not_found() {
        assert!(get_quest_for_role("Pokemon Trainer").is_none());
    }
}

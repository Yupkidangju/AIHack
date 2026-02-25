// ============================================================================
// [v2.41.0 R29-2] 아티팩트 테이블 (artifact_table_ext.rs)
// 원본: NetHack 3.6.7 artilist.h 확장
// 아티팩트 데이터, 속성, 역할 제한
// ============================================================================

/// [v2.41.0 R29-2] 아티팩트 데이터
#[derive(Debug, Clone)]
pub struct ArtifactData {
    pub name: &'static str,
    pub base_item: &'static str,
    pub alignment: &'static str,
    pub role: Option<&'static str>,
    pub attack_bonus: i32,
    pub damage_bonus: i32,
    pub special: &'static str,
    pub invokable: bool,
}

pub fn artifact_table() -> Vec<ArtifactData> {
    vec![
        ArtifactData {
            name: "Excalibur",
            base_item: "long sword",
            alignment: "lawful",
            role: Some("Knight"),
            attack_bonus: 5,
            damage_bonus: 10,
            special: "drain resistance, searching",
            invokable: true,
        },
        ArtifactData {
            name: "Stormbringer",
            base_item: "runesword",
            alignment: "chaotic",
            role: None,
            attack_bonus: 5,
            damage_bonus: 8,
            special: "drain life",
            invokable: false,
        },
        ArtifactData {
            name: "Grayswandir",
            base_item: "silver saber",
            alignment: "lawful",
            role: None,
            attack_bonus: 5,
            damage_bonus: 8,
            special: "halve all damage",
            invokable: false,
        },
        ArtifactData {
            name: "Frost Brand",
            base_item: "long sword",
            alignment: "neutral",
            role: None,
            attack_bonus: 3,
            damage_bonus: 6,
            special: "cold damage, cold resist",
            invokable: false,
        },
        ArtifactData {
            name: "Fire Brand",
            base_item: "long sword",
            alignment: "neutral",
            role: None,
            attack_bonus: 3,
            damage_bonus: 6,
            special: "fire damage, fire resist",
            invokable: false,
        },
        ArtifactData {
            name: "Mjollnir",
            base_item: "war hammer",
            alignment: "neutral",
            role: Some("Valkyrie"),
            attack_bonus: 5,
            damage_bonus: 12,
            special: "lightning, returns",
            invokable: false,
        },
        ArtifactData {
            name: "Magicbane",
            base_item: "athame",
            alignment: "neutral",
            role: Some("Wizard"),
            attack_bonus: 3,
            damage_bonus: 4,
            special: "magic resist, scare",
            invokable: false,
        },
        ArtifactData {
            name: "Vorpal Blade",
            base_item: "long sword",
            alignment: "neutral",
            role: None,
            attack_bonus: 5,
            damage_bonus: 4,
            special: "behead",
            invokable: false,
        },
        ArtifactData {
            name: "Sting",
            base_item: "elven dagger",
            alignment: "chaotic",
            role: None,
            attack_bonus: 5,
            damage_bonus: 5,
            special: "orc warning",
            invokable: false,
        },
        ArtifactData {
            name: "Sunsword",
            base_item: "long sword",
            alignment: "lawful",
            role: None,
            attack_bonus: 5,
            damage_bonus: 8,
            special: "vs undead, light",
            invokable: false,
        },
    ]
}

pub fn find_artifact(name: &str) -> Option<ArtifactData> {
    artifact_table()
        .into_iter()
        .find(|a| a.name.eq_ignore_ascii_case(name))
}

pub fn artifacts_for_role(role: &str) -> Vec<ArtifactData> {
    artifact_table()
        .into_iter()
        .filter(|a| a.role.map_or(true, |r| r.eq_ignore_ascii_case(role)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count() {
        assert_eq!(artifact_table().len(), 10);
    }

    #[test]
    fn test_excalibur() {
        let a = find_artifact("Excalibur").unwrap();
        assert_eq!(a.role, Some("Knight"));
        assert!(a.invokable);
    }

    #[test]
    fn test_role_filter() {
        let arts = artifacts_for_role("Wizard");
        assert!(arts.iter().any(|a| a.name == "Magicbane"));
    }

    #[test]
    fn test_vorpal() {
        let a = find_artifact("Vorpal Blade").unwrap();
        assert!(a.special.contains("behead"));
    }

    #[test]
    fn test_not_found() {
        assert!(find_artifact("Lightsaber").is_none());
    }
}

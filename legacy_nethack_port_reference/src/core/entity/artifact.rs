use crate::core::entity::player::Alignment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactTemplate {
    pub name: String,
    pub base_item: String,
    pub alignment: Alignment,
    pub bonus_dice: Option<i32>,
    pub double_damage: bool,
    pub hate_species: Option<String>,
    pub resists: Vec<crate::core::entity::status::StatusFlags>,
    pub intro_msg: String,
}

#[derive(Clone)]
pub struct ArtifactManager {
    pub artifacts: HashMap<String, ArtifactTemplate>,
}

impl ArtifactManager {
    pub fn new() -> Self {
        let mut artifacts = HashMap::new();

        // 1. Excalibur (Lawful, Long Sword)
        artifacts.insert(
            "Excalibur".to_string(),
            ArtifactTemplate {
                name: "Excalibur".to_string(),
                base_item: "long sword".to_string(),
                alignment: Alignment::Lawful,
                bonus_dice: Some(5), // +1d5
                double_damage: false,
                hate_species: Some("demon".to_string()),
                resists: vec![crate::core::entity::status::StatusFlags::SEARCHING],
                intro_msg: "You are now wielding Excalibur!".to_string(),
            },
        );

        // 2. Grayswandir (Lawful, Silver Saber)
        artifacts.insert(
            "Grayswandir".to_string(),
            ArtifactTemplate {
                name: "Grayswandir".to_string(),
                base_item: "silver saber".to_string(),
                alignment: Alignment::Lawful,
                bonus_dice: None,
                double_damage: true,
                hate_species: None,
                resists: vec![crate::core::entity::status::StatusFlags::HALLUC_RES],
                intro_msg: "You are now wielding Grayswandir!".to_string(),
            },
        );

        // 3. Mjollnir (Neutral, War Hammer)
        artifacts.insert(
            "Mjollnir".to_string(),
            ArtifactTemplate {
                name: "Mjollnir".to_string(),
                base_item: "war hammer".to_string(),
                alignment: Alignment::Neutral,
                bonus_dice: Some(24), // +1d24 lightning
                double_damage: false,
                hate_species: None,
                resists: vec![crate::core::entity::status::StatusFlags::SHOCK_RES],
                intro_msg: "You are now wielding Mjollnir!".to_string(),
            },
        );

        Self { artifacts }
    }

    pub fn get_artifact(&self, name: &str) -> Option<&ArtifactTemplate> {
        self.artifacts.get(name)
    }
}

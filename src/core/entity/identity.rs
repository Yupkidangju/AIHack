use crate::util::rng::NetHackRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub actual_name: String,
    pub description: String,
    pub discovered: bool,
    pub call_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityTable {
    /// key: shuffled description (e.g., "ruby"), value: Identity
    pub mapping: HashMap<String, Identity>,
}

impl IdentityTable {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }

    ///
    ///
    pub fn shuffle(
        &mut self,
        rng: &mut NetHackRng,
        items: &HashMap<String, crate::core::entity::object::ItemTemplate>,
    ) {
        // Potion, Scroll, Wand, Ring, Amulet, Spellbook ??..
        //
        //

        let mut potion_descriptions = vec![
            "ruby",
            "pink",
            "orange",
            "yellow",
            "emerald",
            "dark green",
            "cyan",
            "sky blue",
        ];

        //
        //
        for i in (1..potion_descriptions.len()).rev() {
            let j = rng.rn2(i as i32 + 1) as usize;
            potion_descriptions.swap(i, j);
        }

        let mut desc_idx = 0;
        for item in items.values() {
            if item.class == crate::core::entity::object::ItemClass::Potion
                && item.description.is_none()
            {
                if desc_idx < potion_descriptions.len() {
                    self.mapping.insert(
                        potion_descriptions[desc_idx].to_string(),
                        Identity {
                            actual_name: item.name.clone(),
                            description: potion_descriptions[desc_idx].to_string(),
                            discovered: false,
                            call_name: None,
                        },
                    );
                    desc_idx += 1;
                }
            }
        }
    }

    pub fn is_discovered(&self, description: &str) -> bool {
        self.mapping
            .get(description)
            .map_or(false, |id| id.discovered)
    }

    pub fn discover(&mut self, description: &str) {
        if let Some(id) = self.mapping.get_mut(description) {
            id.discovered = true;
        }
    }
}

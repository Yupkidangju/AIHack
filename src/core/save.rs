// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
use crate::core::dungeon::dungeon::Dungeon;
use crate::core::dungeon::Grid;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::serialize::Canon;
use legion::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

///
#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub dungeon: Dungeon,
    pub turn: u64,
    pub game_log: GameLog,
    pub rng: NetHackRng,
    pub identity: crate::core::entity::identity::IdentityTable,
    pub world_data: String,
}

pub struct SaveManager;

impl SaveManager {
    ///
    pub fn create_registry() -> Registry<String> {
        let mut registry = Registry::new();
        registry.register::<crate::core::entity::Position>("position".to_string());
        registry.register::<crate::core::entity::Level>("level".to_string());
        registry.register::<crate::core::entity::Renderable>("renderable".to_string());
        registry.register::<crate::core::entity::PlayerTag>("player_tag".to_string());
        registry.register::<crate::core::entity::MonsterTag>("monster_tag".to_string());
        registry.register::<crate::core::entity::ItemTag>("item_tag".to_string());
        registry.register::<crate::core::entity::ShopkeeperTag>("shopkeeper_tag".to_string());
        registry.register::<crate::core::entity::Shopkeeper>("shopkeeper".to_string());
        registry.register::<crate::core::entity::Monster>("monster".to_string());
        registry.register::<crate::core::entity::Health>("health".to_string());
        registry.register::<crate::core::entity::CombatStats>("combat_stats".to_string());
        registry.register::<crate::core::entity::Inventory>("inventory".to_string());
        registry.register::<crate::core::entity::Item>("item".to_string());
        registry.register::<crate::core::entity::Equipment>("equipment".to_string());
        registry.register::<crate::core::entity::SpellKnowledge>("spell_knowledge".to_string());
        registry.register::<crate::core::entity::Trap>("trap".to_string());
        registry.register::<crate::core::entity::TrapTag>("trap_tag".to_string());
        registry.register::<crate::core::entity::Species>("species".to_string());
        registry.register::<crate::core::entity::Dialogue>("dialogue".to_string());
        registry.register::<crate::core::entity::Talkative>("talkative".to_string());
        registry.register::<crate::core::entity::Artifact>("artifact".to_string());
        registry.register::<crate::core::entity::QuestLeader>("quest_leader".to_string());
        registry.register::<crate::core::entity::player::Player>("player".to_string());
        registry.register::<crate::core::entity::status::StatusBundle>("status_bundle".to_string());
        registry.register::<crate::core::entity::status::Swallowed>("swallowed".to_string());
        registry
    }

    pub fn save(
        path: &str,
        world: &World,
        resources: &Resources,
        dungeon: &Dungeon,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let registry = Self::create_registry();
        let canon = Canon::default();

        //
        let serializable_world = world.as_serializable(any(), &registry, &canon);
        let world_json = serde_json::to_string(&serializable_world)?;

        //
        let turn = resources.get::<u64>().map(|t| *t).unwrap_or(0);
        let game_log = resources
            .get::<crate::ui::log::GameLog>()
            .map(|l| (*l).clone())
            .ok_or("GameLog missing")?;
        let rng = resources
            .get::<crate::util::rng::NetHackRng>()
            .map(|r| (*r).clone())
            .ok_or("RNG missing")?;
        let identity = resources
            .get::<crate::core::entity::identity::IdentityTable>()
            .map(|i| (*i).clone())
            .ok_or("IdentityTable missing")?;

        let save_state = SaveState {
            dungeon: dungeon.clone(),
            turn,
            game_log,
            rng,
            identity,
            world_data: world_json,
        };

        //
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let encoded = serde_json::to_string_pretty(&save_state)?;
        let mut file = File::create(path)?;
        file.write_all(encoded.as_bytes())?;

        Ok(())
    }

    pub fn load(path: &str) -> Result<(World, Resources, Dungeon), Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let save_state: SaveState = serde_json::from_str(&content)?;

        let registry = Self::create_registry();
        let canon = Canon::default();

        //
        let mut deserializer = serde_json::Deserializer::from_str(&save_state.world_data);
        use serde::de::DeserializeSeed;
        let world = registry
            .as_deserialize(&canon)
            .deserialize(&mut deserializer)?;

        let mut resources = Resources::default();
        resources.insert(save_state.turn);
        resources.insert(save_state.game_log);
        resources.insert(save_state.rng);
        resources.insert(save_state.identity);

        let current_grid = save_state
            .dungeon
            .levels
            .get(&save_state.dungeon.current_level)
            .cloned()
            .unwrap_or_else(Grid::new);
        resources.insert(current_grid);
        resources.insert(save_state.dungeon.clone());

        //
        resources.insert(crate::core::systems::talk::Rumors::new());
        resources.insert(None::<crate::core::systems::item_use::ItemAction>);
        resources.insert(None::<crate::core::systems::throw::ThrowAction>);
        resources.insert(None::<crate::core::systems::spell::CastAction>);
        resources.insert(None::<crate::core::dungeon::LevelChange>);
        resources.insert(crate::core::systems::vision::VisionSystem::new());
        resources.insert(crate::core::events::EventQueue::new()); // [v2.0.0 R5]
        resources.insert(crate::core::events::EventHistory::default()); // [v2.0.0 R5]

        Ok((world, resources, save_state.dungeon))
    }
}

// =============================================================================
// [v2.3.1
//
//
//
//
// =============================================================================

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    ///
    pub version: String,
    ///
    pub player_name: String,
    ///
    pub role: String,
    ///
    pub race: String,
    ///
    pub turn: u64,
    ///
    pub dungeon_depth: i32,
    ///
    pub exp_level: i32,
    ///
    pub max_hp: i32,
    ///
    pub timestamp: String,
    ///
    pub file_size: u64,
}

impl SaveMetadata {
    pub fn summary(&self) -> String {
        format!(
            "{} (Lv{} {} {}) T:{} Dlvl:{} HP:{}",
            self.player_name,
            self.exp_level,
            self.race,
            self.role,
            self.turn,
            self.dungeon_depth,
            self.max_hp,
        )
    }
}

///
pub fn list_save_files(save_dir: &str) -> Vec<String> {
    let mut saves = Vec::new();
    if let Ok(entries) = std::fs::read_dir(save_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "nhsave").unwrap_or(false) {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    saves.push(name.to_string());
                }
            }
        }
    }
    saves.sort();
    saves
}

///
pub fn save_file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

///
pub fn delete_save_file(path: &str) -> Result<(), std::io::Error> {
    if Path::new(path).exists() {
        std::fs::remove_file(path)
    } else {
        Ok(())
    }
}

///
pub fn autosave_path(player_name: &str) -> String {
    let save_dir = "saves";
    format!(
        "{}/{}_autosave.nhsave",
        save_dir,
        player_name.to_lowercase()
    )
}

///
///
pub fn bones_path(branch: &str, depth: i32) -> String {
    format!("bones/bon{}{}.bones", branch, depth)
}

///
pub fn bones_file_exists(branch: &str, depth: i32) -> bool {
    Path::new(&bones_path(branch, depth)).exists()
}

///
///
pub fn verify_save_integrity(path: &str) -> Result<bool, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();

    //
    if file_size < 64 {
        return Ok(false);
    }

    //
    let mut file = File::open(path)?;
    let mut header = [0u8; 1];
    file.read_exact(&mut header)?;
    Ok(header[0] == b'{')
}

///
pub fn extract_save_metadata(path: &str) -> Option<SaveMetadata> {
    if let Ok(mut file) = File::open(path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            if let Ok(save_state) = serde_json::from_str::<SaveState>(&content) {
                let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                return Some(SaveMetadata {
                    version: "2.3.1".to_string(),
                    player_name: "Player".to_string(),
                    role: "Unknown".to_string(),
                    race: "Unknown".to_string(),
                    turn: save_state.turn,
                    dungeon_depth: save_state.dungeon.current_level.depth,
                    exp_level: 1,
                    max_hp: 0,
                    timestamp: String::new(),
                    file_size,
                });
            }
        }
    }
    None
}

///
///
pub fn save_bones(
    grid: &Grid,
    player_name: &str,
    cause_of_death: &str,
    branch: &str,
    depth: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = bones_path(branch, depth);
    if let Some(parent) = Path::new(&path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    //
    #[derive(Serialize)]
    struct BonesData {
        player_name: String,
        cause_of_death: String,
        grid: Grid,
    }

    let bones = BonesData {
        player_name: player_name.to_string(),
        cause_of_death: cause_of_death.to_string(),
        grid: grid.clone(),
    };

    let json = serde_json::to_string(&bones)?;
    let mut file = File::create(&path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolIndex {
    Stone = 0,
    VWall = 1,
    HWall = 2,
    TlCorn = 3,
    TrCorn = 4,
    BlCorn = 5,
    BrCorn = 6,
    CrWall = 7,
    TuWall = 8,
    TdWall = 9,
    TlWall = 10,
    TrWall = 11,
    NDoor = 12,
    VODoor = 13,
    HODoor = 14,
    VCDoor = 15,
    HCDoor = 16,
    Bars = 17,
    Tree = 18,
    Room = 19,
    DarkRoom = 20,
    Corr = 21,
    LitCorr = 22,
    UpStair = 23,
    DnStair = 24,
    UpLadder = 25,
    DnLadder = 26,
    Altar = 27,
    Grave = 28,
    Throne = 29,
    Sink = 30,
    Fountain = 31,
    Pool = 32,
    Ice = 33,
    Lava = 34,
    VODBridge = 35,
    HODBridge = 36,
    VCDBridge = 37,
    HCDBridge = 38,
    Air = 39,
    Cloud = 40,
    Water = 41,
}

#[derive(Debug, Clone)]
pub struct SymbolSet {
    pub name: String,
    pub description: String,
    pub symbols: HashMap<SymbolIndex, char>,
}

impl SymbolSet {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            symbols: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolManager {
    pub sets: HashMap<String, SymbolSet>,
    pub current_set: String,
    pub defaults: HashMap<SymbolIndex, char>,
}

impl SymbolManager {
    pub fn new() -> Self {
        let mut inst = Self {
            sets: HashMap::new(),
            current_set: "plain".to_string(),
            defaults: HashMap::new(),
        };
        inst.init_defaults();
        inst
    }

    fn init_defaults(&mut self) {
        let d = &mut self.defaults;
        d.insert(SymbolIndex::Stone, ' ');
        d.insert(SymbolIndex::VWall, '|');
        d.insert(SymbolIndex::HWall, '-');
        d.insert(SymbolIndex::TlCorn, '+');
        d.insert(SymbolIndex::TrCorn, '+');
        d.insert(SymbolIndex::BlCorn, '+');
        d.insert(SymbolIndex::BrCorn, '+');
        d.insert(SymbolIndex::CrWall, '+');
        d.insert(SymbolIndex::TuWall, '+');
        d.insert(SymbolIndex::TdWall, '+');
        d.insert(SymbolIndex::TlWall, '+');
        d.insert(SymbolIndex::TrWall, '+');
        d.insert(SymbolIndex::NDoor, '.');
        d.insert(SymbolIndex::VODoor, '-');
        d.insert(SymbolIndex::HODoor, '|');
        d.insert(SymbolIndex::VCDoor, '+');
        d.insert(SymbolIndex::HCDoor, '+');
        d.insert(SymbolIndex::Room, '.');
        d.insert(SymbolIndex::Corr, '#');
        d.insert(SymbolIndex::LitCorr, '#');
        d.insert(SymbolIndex::UpStair, '<');
        d.insert(SymbolIndex::DnStair, '>');
        d.insert(SymbolIndex::Fountain, '{');
        d.insert(SymbolIndex::Pool, '}');
        d.insert(SymbolIndex::Water, '~');
        d.insert(SymbolIndex::Lava, '~');
    }

    ///
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut current_set: Option<SymbolSet> = None;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with("start:") {
                let name = trimmed["start:".len()..].trim();
                current_set = Some(SymbolSet::new(name));
            } else if trimmed == "finish" {
                if let Some(set) = current_set.take() {
                    self.sets.insert(set.name.clone(), set);
                }
            } else if let Some(ref mut set) = current_set {
                if trimmed.starts_with("Description:") {
                    set.description = trimmed["Description:".len()..].trim().to_string();
                } else if let Some(idx) = trimmed.find(':') {
                    let key = trimmed[..idx].trim();
                    let val = trimmed[idx + 1..].trim();

                    if let Some(sym_idx) = self.map_key_to_index(key) {
                        if let Some(c) = self.parse_symbol_value(val) {
                            set.symbols.insert(sym_idx, c);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn map_key_to_index(&self, key: &str) -> Option<SymbolIndex> {
        match key {
            "S_stone" => Some(SymbolIndex::Stone),
            "S_vwall" => Some(SymbolIndex::VWall),
            "S_hwall" => Some(SymbolIndex::HWall),
            "S_tlcorn" => Some(SymbolIndex::TlCorn),
            "S_trcorn" => Some(SymbolIndex::TrCorn),
            "S_blcorn" => Some(SymbolIndex::BlCorn),
            "S_brcorn" => Some(SymbolIndex::BrCorn),
            "S_crwall" => Some(SymbolIndex::CrWall),
            "S_tuwall" => Some(SymbolIndex::TuWall),
            "S_tdwall" => Some(SymbolIndex::TdWall),
            "S_tlwall" => Some(SymbolIndex::TlWall),
            "S_trwall" => Some(SymbolIndex::TrWall),
            "S_ndoor" => Some(SymbolIndex::NDoor),
            "S_vodoor" => Some(SymbolIndex::VODoor),
            "S_hodoor" => Some(SymbolIndex::HODoor),
            "S_vcdoor" => Some(SymbolIndex::VCDoor),
            "S_hcdoor" => Some(SymbolIndex::HCDoor),
            "S_bars" => Some(SymbolIndex::Bars),
            "S_tree" => Some(SymbolIndex::Tree),
            "S_room" => Some(SymbolIndex::Room),
            "S_corr" => Some(SymbolIndex::Corr),
            "S_litcorr" => Some(SymbolIndex::LitCorr),
            "S_upstair" => Some(SymbolIndex::UpStair),
            "S_dnstair" => Some(SymbolIndex::DnStair),
            "S_fountain" => Some(SymbolIndex::Fountain),
            "S_pool" => Some(SymbolIndex::Pool),
            "S_water" => Some(SymbolIndex::Water),
            "S_air" => Some(SymbolIndex::Air),
            "S_cloud" => Some(SymbolIndex::Cloud),
            _ => None,
        }
    }

    fn parse_symbol_value(&self, val: &str) -> Option<char> {
        let val = if val.starts_with('\'') && val.ends_with('\'') && val.len() >= 3 {
            &val[1..val.len() - 1]
        } else {
            val
        };

        if val.len() == 1 {
            val.chars().next()
        } else if val.starts_with('\\') {
            if val.starts_with("\\x") {
                u32::from_str_radix(&val[2..], 16)
                    .ok()
                    .and_then(std::char::from_u32)
            } else {
                u32::from_str_radix(&val[1..], 8)
                    .ok()
                    .and_then(std::char::from_u32)
            }
        } else {
            val.chars().next()
        }
    }

    pub fn get_char(&self, idx: SymbolIndex) -> char {
        self.sets
            .get(&self.current_set)
            .and_then(|set| set.symbols.get(&idx))
            .cloned()
            .or_else(|| self.defaults.get(&idx).cloned())
            .unwrap_or('?')
    }

    pub fn tile_to_index(&self, tile: crate::core::dungeon::tile::TileType) -> SymbolIndex {
        use crate::core::dungeon::tile::TileType;
        match tile {
            TileType::Stone => SymbolIndex::Stone,
            TileType::VWall => SymbolIndex::VWall,
            TileType::HWall => SymbolIndex::HWall,
            TileType::TlCorner => SymbolIndex::TlCorn,
            TileType::TrCorner => SymbolIndex::TrCorn,
            TileType::BlCorner => SymbolIndex::BlCorn,
            TileType::BrCorner => SymbolIndex::BrCorn,
            TileType::Room => SymbolIndex::Room,
            TileType::Corr => SymbolIndex::Corr,
            TileType::StairsUp => SymbolIndex::UpStair,
            TileType::StairsDown => SymbolIndex::DnStair,
            TileType::Fountain => SymbolIndex::Fountain,
            TileType::Pool => SymbolIndex::Pool,
            TileType::Water => SymbolIndex::Water,
            TileType::LavaPool => SymbolIndex::Lava,
            TileType::Cloud => SymbolIndex::Cloud,
            TileType::Air => SymbolIndex::Air,
            TileType::Door => SymbolIndex::VCDoor,
            TileType::Ladder => SymbolIndex::DnLadder,
            TileType::OpenDoor => SymbolIndex::VODoor,
            _ => SymbolIndex::Stone,
        }
    }

    pub fn get_tile_symbol(&self, tile: crate::core::dungeon::tile::TileType) -> char {
        self.get_char(self.tile_to_index(tile))
    }

    pub fn get_tile_color(&self, tile: crate::core::dungeon::tile::TileType) -> u8 {
        use crate::core::dungeon::tile::TileType;
        match tile {
            TileType::Stone => 8,
            TileType::Room => 7,
            TileType::Corr => 7,
            TileType::Pool | TileType::Water => 4,
            TileType::LavaPool => 1,
            TileType::Fountain => 6,
            _ => 7,
        }
    }
}

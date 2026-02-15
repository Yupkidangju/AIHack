pub mod symbols;

use crate::core::entity::monster::MonsterManager;
use crate::core::entity::object::ItemManager;
use std::fs;
pub use symbols::{SymbolIndex, SymbolManager};

///
#[derive(Clone)]
pub struct AssetManager {
    pub symbols: SymbolManager,
    pub monsters: MonsterManager,
    pub items: ItemManager,
    pub artifacts: crate::core::entity::artifact::ArtifactManager,
    pub rumors: crate::core::systems::talk::Rumors,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            symbols: SymbolManager::new(),
            monsters: MonsterManager::new(),
            items: ItemManager::new(),
            artifacts: crate::core::entity::artifact::ArtifactManager::new(),
            rumors: crate::core::systems::talk::Rumors::new(),
        }
    }

    ///
    pub fn load_defaults(&mut self, base_path: &str) {
        // 1. 심볼 데이터 로드
        let symbols_path = format!("{}/dat/symbols", base_path);
        if let Err(e) = self.symbols.load_from_file(&symbols_path) {
            eprintln!("심볼 파일 로드 실패: {}", e);
        }

        //
        let monsters_path = "assets/data/monsters.toml";
        if let Ok(content) = fs::read_to_string(&monsters_path) {
            match toml::from_str::<MonsterData>(&content) {
                Ok(data) => {
                    for m in data.monsters {
                        self.monsters.templates.insert(m.name.clone(), m);
                    }
                    println!(
                        "Monster 데이터 로드 완료: {} 종",
                        self.monsters.templates.len()
                    );
                }
                Err(e) => eprintln!("Monster TOML 파싱 실패: {}", e),
            }
        } else {
            eprintln!("Monster 파일을 찾을 수 없음: {}", monsters_path);
        }

        //
        let items_path = "assets/data/items.toml";
        if let Ok(content) = fs::read_to_string(&items_path) {
            match toml::from_str::<ItemData>(&content) {
                Ok(data) => {
                    for item in data.items {
                        self.items.templates.insert(item.name.clone(), item);
                    }
                    println!("Item 데이터 로드 완료: {} 종", self.items.templates.len());
                }
                Err(e) => eprintln!("Item TOML 파싱 실패: {}", e),
            }
        } else {
            eprintln!("Item 파일을 찾을 수 없음: {}", items_path);
        }

        // 4. 텍스트 데이터 (Rumors, Oracles 등) 로드
        self.rumors.load_all();
        println!(
            "텍스트 데이터 로드 완료 (Rumors: {}, Oracles: {})",
            self.rumors.true_rumors.len() + self.rumors.false_rumors.len(),
            self.rumors.oracles.len()
        );

        // 5. [v2.0.0 R2-6] enum 기반 조회 인덱스 구축
        self.monsters.build_kind_index();
        self.items.build_kind_index();
    }
}

#[derive(serde::Deserialize)]
struct MonsterData {
    monsters: Vec<crate::core::entity::monster::MonsterTemplate>,
}

#[derive(serde::Deserialize)]
struct ItemData {
    items: Vec<crate::core::entity::object::ItemTemplate>,
}

pub mod artifact;
pub mod capability; // [v2.0.0 R6] MonsterCapability + StatusCategory
pub mod identity;
pub mod mon;
pub mod monster;
pub mod object;
pub mod object_data;
pub mod objnam;
pub mod player;
pub mod player_view; // [v2.0.0 R6] Player God Object 뷰 타입
pub mod skills;
pub mod spawn;
pub mod status;

use legion::Entity;
use serde::{Deserialize, Serialize};

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Level(pub crate::core::dungeon::LevelID);

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Renderable {
    pub glyph: char,
    pub color: u8,
}

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerTag;

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MonsterTag;

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemTag;

/// 상점 주인 태그
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShopkeeperTag;

/// 용기(상자, 가방 등) 태그
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContainerTag;

/// 바위 태그
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoulderTag;

/// 석상 태그
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatueTag;

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ContainerType {
    Normal,       // 일반 상자, 자루
    BagOfHolding, // 마법 가방 (무게 감소)
    IceBox,       // 아이스박스 (부패 지연)
    BagOfTricks,  // 마법의 자루 (몬스터 소환)
}

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContainerProperties {
    pub typ: ContainerType,
    pub capacity: u32, // 최대 수용량 (NetHack에서는 무한대인 경우가 많으나 시스템상 제한)
    pub weight_reduc: u8, // 무게 감소율 (백분율, BoH용)
}

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct InContainerTag {
    pub container: Entity,
}

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shopkeeper {
    pub name: String,
    pub shoproom: u8,
}

/// 구조물 태그 (Phase 50.1)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StructureTag;

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum StructureType {
    CommBase,    // 통신기지
    SupplyDepot, // 보급소
}

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Structure {
    pub typ: StructureType,
    pub integrity: i32, // 구조물 내구도
    pub max_integrity: i32,
}

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StandardBearer {
    pub aura_range: i32, // 광기 버프 범위
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StandardBearerTag;

/// 광기(Frenzy) 버프 상태 - 기수가 근처에 있을 때 활성화
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrenzyTag;

///
/// [v2.0.0 R2] template: String → kind: MonsterKind 전환
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Monster {
    pub kind: crate::generated::MonsterKind,
    pub hostile: bool,
    pub mon_name: Option<String>, // 플레이어가 붙인 몬스터 이름
}

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CombatStats {
    pub ac: i32,
    pub level: i32,
}

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<Entity>,
    pub letter_map: std::collections::HashMap<char, Entity>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            letter_map: std::collections::HashMap::new(),
        }
    }

    pub fn assign_letter(&mut self, entity: Entity) -> char {
        // 이미 할당된 문자가 있는지 확인
        for (&c, &e) in &self.letter_map {
            if e == entity {
                return c;
            }
        }

        // 새로운 문자 할당 (a-z, A-Z)
        for c in ('a'..='z').chain('A'..='Z') {
            if !self.letter_map.contains_key(&c) {
                self.letter_map.insert(c, entity);
                return c;
            }
        }
        '?'
    }

    pub fn get_letter(&self, entity: Entity) -> char {
        for (&c, &e) in &self.letter_map {
            if e == entity {
                return c;
            }
        }
        '?'
    }
}

///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// [v2.0.0 R2] 아이템 종류 (이전: template: String)
    pub kind: crate::generated::ItemKind,
    pub price: u32,
    pub weight: u32,
    pub unpaid: bool,
    pub spe: i8,                   // 강화 수치 (Enchantment, +/-, spe)
    pub blessed: bool,             // 축복 여부
    pub cursed: bool,              // 저주 여부
    pub bknown: bool,              // 축복/저주 인지 여부
    pub known: bool,               // 아이템 종류 인지 여부
    pub dknown: bool,              // 아이템 외형 인지 여부
    pub oeroded: u8,               // 부식 (녹) 정도 (oeroded)
    pub oeroded2: u8,              // 부식 (불/부패) 정도 (oeroded2)
    pub quantity: u32,             // 수량 (quan)
    pub corpsenm: Option<String>,  // 시체인 경우 몬스터 이름 (corpsenm)
    pub age: u64,                  // 생성된 턴 (부패 계산용, age)
    pub oeaten: u16,               // 이미 먹은 양 (oeaten)
    pub olocked: bool,             // 잠김 여부 (olocked)
    pub oopened: bool,             // 열림 여부 (oopened)
    pub user_name: Option<String>, // 플레이어가 붙인 개별 이름 (oname)
    pub artifact: Option<String>,  // 아티팩트 식별자 (oartifact)
    pub owet: u8,                  // 젖음 정도 (owet) - Phase 42
}

impl Default for Item {
    fn default() -> Self {
        Self {
            kind: crate::generated::ItemKind::from_str("unknown"),
            price: 0,
            weight: 0,
            unpaid: false,
            spe: 0,
            blessed: false,
            cursed: false,
            bknown: false,
            known: false,
            dknown: false,
            oeroded: 0,
            oeroded2: 0,
            quantity: 1,
            corpsenm: None,
            age: 0,
            oeaten: 0,
            olocked: false,
            oopened: false,
            user_name: None,
            artifact: None,
            owet: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
    Offhand, // NetHack #twoweapon logic
    Swap,    // NetHack secondary weapon (x command)
    Quiver,  // NetHack projectile slot (f command)
    Head,
    Body,
    Feet,
    Hands,
    Cloak,
    // [v2.2.0
    RingLeft,  // NetHack: uleft
    RingRight, // NetHack: uright
    Amulet,    // NetHack: uamul
    Boots,     // NetHack: uarmf (기존 Feet와 동의어이나 명시성 위해 추가)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Equipment {
    pub slots: std::collections::HashMap<EquipmentSlot, Entity>,
}

impl Equipment {
    pub fn new() -> Self {
        Self {
            slots: std::collections::HashMap::new(),
        }
    }
}

/// 습득한 주문 정보
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearnedSpell {
    pub name: String,
    pub level: i32,
    pub retention: i32, // 남은 기억력 (NetHack 방식)
}

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpellKnowledge {
    pub spells: std::collections::HashMap<char, LearnedSpell>,
}

impl SpellKnowledge {
    pub fn new() -> Self {
        Self {
            spells: std::collections::HashMap::new(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrapType {
    NoTrap,
    Arrow,
    Dart,
    Rock,
    SqueakyBoard,
    BearTrap,
    Landmine,
    RollingBoulder,
    SleepGas,
    Rust,
    Fire,
    Pit,
    SpikedPit,
    Hole,
    TrapDoor,
    Teleport,
    LevelTeleport,
    MagicPortal,
    Web,
    Statue,
    Magic,
    AntiMagic,
    Polymorph,
    VibratingSquare,
}

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trap {
    pub typ: TrapType,
    pub discovered: bool,
}

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrapTag;

/// 엔티티 종 정보 (Polymorph 지원)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Species {
    pub current: String,    // 현재 모습 (monster template name 또는 "player")
    pub original: String,   // 본래 모습
    pub timer: Option<u32>, // 폴리모프 잔여 시간
}

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dialogue {
    pub messages: Vec<String>,
}

/// 대화 가능 태그
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Talkative;

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub alignment: crate::core::entity::player::Alignment,
}

/// 퀘스트 리더 태그
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuestLeader;

/// 매직 맵 요청 태그 (아이템 사용 시 부착)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MagicMapRequest;

///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightSource {
    pub range: i32, // 밝기 반경
    pub lit: bool,  // 현재 켜짐 여부
}

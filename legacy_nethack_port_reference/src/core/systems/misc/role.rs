// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
//
//
// [v1.9.0
// =============================================================================
//
//
//
//

use crate::core::entity::player::{Alignment, PlayerClass, Race};
use serde::{Deserialize, Serialize};

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AttributeArray {
    pub str_val: i32,
    pub int_val: i32,
    pub wis_val: i32,
    pub dex_val: i32,
    pub con_val: i32,
    pub cha_val: i32,
}

impl AttributeArray {
    pub const fn new(s: i32, i: i32, w: i32, d: i32, c: i32, ch: i32) -> Self {
        Self {
            str_val: s,
            int_val: i,
            wis_val: w,
            dex_val: d,
            con_val: c,
            cha_val: ch,
        }
    }
}

///
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HpAdvancement {
    pub init_hp: i32,
    pub init_add: i32,
    pub init_var: i32,
    pub level_hp: i32,
    pub level_add: i32,
    pub level_var: i32,
}

///
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnAdvancement {
    pub init_en: i32,
    pub init_add: i32,
    pub init_var: i32,
    pub level_en: i32,
    pub level_add: i32,
    pub level_var: i32,
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct RoleData {
    pub name: &'static str,
    pub alt_name: Option<&'static str>,
    pub rank_titles: [&'static str; 9],
    pub gods: [&'static str; 3],        // [Lawful, Neutral, Chaotic] ??
    pub filecode: &'static str,
    pub home_base: &'static str,
    pub quest_target: &'static str,
    pub attr_base: AttributeArray,
    pub attr_max: AttributeArray,
    pub hp_adv: HpAdvancement,
    pub en_adv: EnAdvancement,
    pub cutoff_level: i32,
    pub spell_penalty: i32,
    pub spell_stat: SpellStat,
    pub allowed_races: u32,
    pub allowed_aligns: u32,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellStat {
    Intelligence,
    Wisdom,
}

//
pub const MH_HUMAN: u32 = 0x0001;
pub const MH_ELF: u32 = 0x0002;
pub const MH_DWARF: u32 = 0x0004;
pub const MH_GNOME: u32 = 0x0008;
pub const MH_ORC: u32 = 0x0010;

pub const ROLE_LAWFUL: u32 = 0x0100;
pub const ROLE_NEUTRAL: u32 = 0x0200;
pub const ROLE_CHAOTIC: u32 = 0x0400;

// =============================================================================
//
// =============================================================================

///
pub fn get_role_data(role: PlayerClass) -> &'static RoleData {
    match role {
        PlayerClass::Archeologist => &ARCHEOLOGIST,
        PlayerClass::Barbarian => &BARBARIAN,
        PlayerClass::Healer => &HEALER,
        PlayerClass::Knight => &KNIGHT,
        PlayerClass::Monk => &MONK,
        PlayerClass::Priest => &PRIEST,
        PlayerClass::Ranger => &RANGER,
        PlayerClass::Rogue => &ROGUE,
        PlayerClass::Samurai => &SAMURAI,
        PlayerClass::Tourist => &TOURIST,
        PlayerClass::Valkyrie => &VALKYRIE,
        PlayerClass::Wizard => &WIZARD,
    }
}

//
static ARCHEOLOGIST: RoleData = RoleData {
    name: "Archeologist",
    alt_name: None,
    rank_titles: [
        "Digger",
        "Field Worker",
        "Investigator",
        "Exhumer",
        "Excavator",
        "Spelunker",
        "Speleologist",
        "Collector",
        "Curator",
    ],
    gods: ["Quetzalcoatl", "Camaxtli", "Huhetotl"],
    filecode: "Arc",
    home_base: "the College of Archeology",
    quest_target: "the Tomb of the Toltec Kings",
    attr_base: AttributeArray::new(7, 10, 10, 7, 7, 7),
    attr_max: AttributeArray::new(20, 20, 20, 10, 20, 10),
    hp_adv: HpAdvancement {
        init_hp: 11,
        init_add: 0,
        init_var: 0,
        level_hp: 8,
        level_add: 1,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 1,
        init_add: 0,
        init_var: 0,
        level_en: 1,
        level_add: 0,
        level_var: 1,
    },
    cutoff_level: 14,
    spell_penalty: -4,
    spell_stat: SpellStat::Intelligence,
    allowed_races: MH_HUMAN | MH_DWARF | MH_GNOME,
    allowed_aligns: ROLE_LAWFUL | ROLE_NEUTRAL,
};

//
static BARBARIAN: RoleData = RoleData {
    name: "Barbarian",
    alt_name: None,
    rank_titles: [
        "Plunderer",
        "Pillager",
        "Bandit",
        "Brigand",
        "Raider",
        "Reaver",
        "Slayer",
        "Chieftain",
        "Conqueror",
    ],
    gods: ["Mitra", "Crom", "Set"],
    filecode: "Bar",
    home_base: "the Camp of the Duali Tribe",
    quest_target: "the Duali Oasis",
    attr_base: AttributeArray::new(16, 7, 7, 15, 16, 6),
    attr_max: AttributeArray::new(30, 6, 7, 20, 30, 7),
    hp_adv: HpAdvancement {
        init_hp: 14,
        init_add: 0,
        init_var: 0,
        level_hp: 10,
        level_add: 2,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 1,
        init_add: 0,
        init_var: 0,
        level_en: 1,
        level_add: 0,
        level_var: 1,
    },
    cutoff_level: 10,
    spell_penalty: -4,
    spell_stat: SpellStat::Intelligence,
    allowed_races: MH_HUMAN | MH_ORC,
    allowed_aligns: ROLE_NEUTRAL | ROLE_CHAOTIC,
};

//
static HEALER: RoleData = RoleData {
    name: "Healer",
    alt_name: None,
    rank_titles: [
        "Rhizotomist",
        "Empiric",
        "Embalmer",
        "Dresser",
        "Medicus ossium",
        "Herbalist",
        "Magister",
        "Physician",
        "Chirurgeon",
    ],
    gods: ["Athena", "Hermes", "Poseidon"],
    filecode: "Hea",
    home_base: "the Temple of Epidaurus",
    quest_target: "the Temple of Coeus",
    attr_base: AttributeArray::new(7, 7, 13, 7, 11, 16),
    attr_max: AttributeArray::new(15, 20, 20, 15, 25, 5),
    hp_adv: HpAdvancement {
        init_hp: 11,
        init_add: 0,
        init_var: 0,
        level_hp: 8,
        level_add: 1,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 1,
        init_add: 4,
        init_var: 0,
        level_en: 1,
        level_add: 0,
        level_var: 2,
    },
    cutoff_level: 20,
    spell_penalty: -4,
    spell_stat: SpellStat::Wisdom,
    allowed_races: MH_HUMAN | MH_GNOME,
    allowed_aligns: ROLE_NEUTRAL,
};

//
static KNIGHT: RoleData = RoleData {
    name: "Knight",
    alt_name: None,
    rank_titles: [
        "Gallant",
        "Esquire",
        "Bachelor",
        "Sergeant",
        "Knight",
        "Banneret",
        "Chevalier",
        "Seignieur",
        "Paladin",
    ],
    gods: ["Lugh", "Brigit", "Manannan Mac Lir"],
    filecode: "Kni",
    home_base: "Camelot Castle",
    quest_target: "the Isle of Glass",
    attr_base: AttributeArray::new(13, 7, 14, 8, 10, 17),
    attr_max: AttributeArray::new(30, 15, 15, 10, 20, 10),
    hp_adv: HpAdvancement {
        init_hp: 14,
        init_add: 0,
        init_var: 0,
        level_hp: 8,
        level_add: 2,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 1,
        init_add: 4,
        init_var: 0,
        level_en: 1,
        level_add: 0,
        level_var: 2,
    },
    cutoff_level: 10,
    spell_penalty: -4,
    spell_stat: SpellStat::Wisdom,
    allowed_races: MH_HUMAN,
    allowed_aligns: ROLE_LAWFUL,
};

//
static MONK: RoleData = RoleData {
    name: "Monk",
    alt_name: None,
    rank_titles: [
        "Candidate",
        "Novice",
        "Initiate",
        "Student of Stones",
        "Student of Waters",
        "Student of Metals",
        "Student of Winds",
        "Student of Fire",
        "Master",
    ],
    gods: ["Shan Lai Ching", "Chih Sung-tzu", "Huan Ti"],
    filecode: "Mon",
    home_base: "the Monastery of Chan-Sune",
    quest_target: "the Monastery of the Earth-Lord",
    attr_base: AttributeArray::new(10, 7, 8, 8, 7, 7),
    attr_max: AttributeArray::new(25, 10, 20, 20, 15, 10),
    hp_adv: HpAdvancement {
        init_hp: 12,
        init_add: 0,
        init_var: 0,
        level_hp: 8,
        level_add: 1,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 2,
        init_add: 2,
        init_var: 0,
        level_en: 2,
        level_add: 0,
        level_var: 2,
    },
    cutoff_level: 10,
    spell_penalty: -4,
    spell_stat: SpellStat::Wisdom,
    allowed_races: MH_HUMAN,
    allowed_aligns: ROLE_LAWFUL | ROLE_NEUTRAL | ROLE_CHAOTIC,
};

//
static PRIEST: RoleData = RoleData {
    name: "Priest",
    alt_name: Some("Priestess"),
    rank_titles: [
        "Aspirant",
        "Acolyte",
        "Adept",
        "Priest",
        "Curate",
        "Canon",
        "Lama",
        "Patriarch",
        "High Priest",
    ],
    gods: ["", "", ""],
    filecode: "Pri",
    home_base: "the Great Temple",
    quest_target: "the Temple of Nalzok",
    attr_base: AttributeArray::new(7, 7, 10, 7, 7, 7),
    attr_max: AttributeArray::new(15, 10, 30, 15, 20, 10),
    hp_adv: HpAdvancement {
        init_hp: 12,
        init_add: 0,
        init_var: 0,
        level_hp: 8,
        level_add: 1,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 4,
        init_add: 3,
        init_var: 0,
        level_en: 2,
        level_add: 0,
        level_var: 2,
    },
    cutoff_level: 20,
    spell_penalty: -4,
    spell_stat: SpellStat::Wisdom,
    allowed_races: MH_HUMAN | MH_ELF,
    allowed_aligns: ROLE_LAWFUL | ROLE_NEUTRAL | ROLE_CHAOTIC,
};

//
static RANGER: RoleData = RoleData {
    name: "Ranger",
    alt_name: None,
    rank_titles: [
        "Tenderfoot",
        "Lookout",
        "Trailblazer",
        "Reconnoiterer",
        "Scout",
        "Arbalester",
        "Archer",
        "Sharpshooter",
        "Marksman",
    ],
    gods: ["Mercury", "Venus", "Mars"],
    filecode: "Ran",
    home_base: "Orion's camp",
    quest_target: "the cave of the Cyclops",
    attr_base: AttributeArray::new(13, 13, 13, 9, 13, 7),
    attr_max: AttributeArray::new(30, 10, 10, 20, 20, 10),
    hp_adv: HpAdvancement {
        init_hp: 13,
        init_add: 0,
        init_var: 0,
        level_hp: 10,
        level_add: 2,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 1,
        init_add: 0,
        init_var: 0,
        level_en: 1,
        level_add: 0,
        level_var: 1,
    },
    cutoff_level: 12,
    spell_penalty: -4,
    spell_stat: SpellStat::Intelligence,
    allowed_races: MH_HUMAN | MH_ELF | MH_GNOME | MH_ORC,
    allowed_aligns: ROLE_NEUTRAL | ROLE_CHAOTIC,
};

//
static ROGUE: RoleData = RoleData {
    name: "Rogue",
    alt_name: None,
    rank_titles: [
        "Footpad", "Cutpurse", "Rogue", "Pilferer", "Robber", "Burglar", "Filcher", "Magsman",
        "Thief",
    ],
    gods: ["Issek", "Mog", "Kos"],
    filecode: "Rog",
    home_base: "the Thieves' Guild Hall",
    quest_target: "the Assassins' Guild Hall",
    attr_base: AttributeArray::new(7, 7, 7, 10, 7, 6),
    attr_max: AttributeArray::new(20, 10, 10, 30, 20, 10),
    hp_adv: HpAdvancement {
        init_hp: 10,
        init_add: 0,
        init_var: 0,
        level_hp: 8,
        level_add: 1,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 1,
        init_add: 0,
        init_var: 0,
        level_en: 1,
        level_add: 0,
        level_var: 1,
    },
    cutoff_level: 11,
    spell_penalty: -4,
    spell_stat: SpellStat::Intelligence,
    allowed_races: MH_HUMAN | MH_ORC,
    allowed_aligns: ROLE_CHAOTIC,
};

//
static SAMURAI: RoleData = RoleData {
    name: "Samurai",
    alt_name: None,
    rank_titles: [
        "Hatamoto", "Ronin", "Ninja", "Joshu", "Ryoshu", "Kokushu", "Daimyo", "Kuge", "Shogun",
    ],
    gods: ["Amaterasu Omikami", "Raijin", "Susanowo"],
    filecode: "Sam",
    home_base: "the Castle of the Taro Clan",
    quest_target: "the Shogun's Castle",
    attr_base: AttributeArray::new(10, 8, 7, 10, 17, 6),
    attr_max: AttributeArray::new(30, 10, 8, 30, 30, 7),
    hp_adv: HpAdvancement {
        init_hp: 13,
        init_add: 0,
        init_var: 0,
        level_hp: 10,
        level_add: 2,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 1,
        init_add: 0,
        init_var: 0,
        level_en: 1,
        level_add: 0,
        level_var: 1,
    },
    cutoff_level: 11,
    spell_penalty: -4,
    spell_stat: SpellStat::Intelligence,
    allowed_races: MH_HUMAN,
    allowed_aligns: ROLE_LAWFUL,
};

//
static TOURIST: RoleData = RoleData {
    name: "Tourist",
    alt_name: None,
    rank_titles: [
        "Rambler",
        "Sightseer",
        "Excursionist",
        "Peregrinator",
        "Traveler",
        "Journeyer",
        "Voyager",
        "Explorer",
        "Adventurer",
    ],
    gods: ["Blind Io", "The Lady", "Offler"],
    filecode: "Tou",
    home_base: "Ankh-Morpork",
    quest_target: "the Thieves' Guild Hall",
    attr_base: AttributeArray::new(7, 10, 6, 7, 7, 10),
    attr_max: AttributeArray::new(15, 10, 10, 15, 30, 25),
    hp_adv: HpAdvancement {
        init_hp: 8,
        init_add: 0,
        init_var: 0,
        level_hp: 8,
        level_add: 0,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 1,
        init_add: 0,
        init_var: 0,
        level_en: 1,
        level_add: 0,
        level_var: 1,
    },
    cutoff_level: 14,
    spell_penalty: -4,
    spell_stat: SpellStat::Intelligence,
    allowed_races: MH_HUMAN,
    allowed_aligns: ROLE_NEUTRAL,
};

//
static VALKYRIE: RoleData = RoleData {
    name: "Valkyrie",
    alt_name: None,
    rank_titles: [
        "Stripling",
        "Skirmisher",
        "Fighter",
        "Man-at-arms",
        "Warrior",
        "Swashbuckler",
        "Hero",
        "Champion",
        "Lord",
    ],
    gods: ["Tyr", "Odin", "Loki"],
    filecode: "Val",
    home_base: "the Shrine of Destiny",
    quest_target: "the Cave of Surtr",
    attr_base: AttributeArray::new(10, 7, 7, 7, 10, 7),
    attr_max: AttributeArray::new(30, 6, 7, 20, 30, 7),
    hp_adv: HpAdvancement {
        init_hp: 14,
        init_add: 0,
        init_var: 0,
        level_hp: 10,
        level_add: 2,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 1,
        init_add: 0,
        init_var: 0,
        level_en: 1,
        level_add: 0,
        level_var: 1,
    },
    cutoff_level: 10,
    spell_penalty: -4,
    spell_stat: SpellStat::Wisdom,
    allowed_races: MH_HUMAN | MH_DWARF,
    allowed_aligns: ROLE_LAWFUL | ROLE_NEUTRAL,
};

//
static WIZARD: RoleData = RoleData {
    name: "Wizard",
    alt_name: None,
    rank_titles: [
        "Evoker",
        "Conjurer",
        "Thaumaturge",
        "Magician",
        "Enchanter",
        "Sorcerer",
        "Necromancer",
        "Wizard",
        "Mage",
    ],
    gods: ["Ptah", "Thoth", "Anhur"],
    filecode: "Wiz",
    home_base: "the Lonely Tower",
    quest_target: "the Tower of Darkness",
    attr_base: AttributeArray::new(7, 10, 7, 7, 7, 7),
    attr_max: AttributeArray::new(10, 30, 10, 20, 20, 10),
    hp_adv: HpAdvancement {
        init_hp: 10,
        init_add: 0,
        init_var: 0,
        level_hp: 8,
        level_add: 1,
        level_var: 0,
    },
    en_adv: EnAdvancement {
        init_en: 4,
        init_add: 3,
        init_var: 0,
        level_en: 2,
        level_add: 0,
        level_var: 3,
    },
    cutoff_level: 12,
    spell_penalty: 0,
    spell_stat: SpellStat::Intelligence,
    allowed_races: MH_HUMAN | MH_ELF | MH_GNOME | MH_ORC,
    allowed_aligns: ROLE_NEUTRAL | ROLE_CHAOTIC,
};

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct RaceData {
    pub name: &'static str,
    pub adj: &'static str,
    pub filecode: &'static str,
    pub attr_base: AttributeArray,
    pub attr_max: AttributeArray,
    pub hp_bonus: i32,
    pub en_bonus: i32,
    pub allowed_aligns: u32,
}

///
pub fn get_race_data(race: Race) -> &'static RaceData {
    match race {
        Race::Human => &HUMAN_RACE,
        Race::Elf => &ELF_RACE,
        Race::Dwarf => &DWARF_RACE,
        Race::Gnome => &GNOME_RACE,
        Race::Orc => &ORC_RACE,
    }
}

//
static HUMAN_RACE: RaceData = RaceData {
    name: "Human",
    adj: "human",
    filecode: "Hum",
    attr_base: AttributeArray::new(0, 0, 0, 0, 0, 0),
    attr_max: AttributeArray::new(18, 18, 18, 18, 18, 18),
    hp_bonus: 2,
    en_bonus: 1,
    allowed_aligns: ROLE_LAWFUL | ROLE_NEUTRAL | ROLE_CHAOTIC,
};

//
static ELF_RACE: RaceData = RaceData {
    name: "Elf",
    adj: "elven",
    filecode: "Elf",
    attr_base: AttributeArray::new(-1, 0, 0, 1, -1, 1),
    attr_max: AttributeArray::new(16, 20, 20, 18, 16, 18),
    hp_bonus: 1,
    en_bonus: 2,
    allowed_aligns: ROLE_CHAOTIC,
};

//
static DWARF_RACE: RaceData = RaceData {
    name: "Dwarf",
    adj: "dwarven",
    filecode: "Dwa",
    attr_base: AttributeArray::new(0, 0, 0, 0, 2, -1),
    attr_max: AttributeArray::new(18, 16, 16, 20, 20, 16),
    hp_bonus: 4,
    en_bonus: 0,
    allowed_aligns: ROLE_LAWFUL,
};

//
static GNOME_RACE: RaceData = RaceData {
    name: "Gnome",
    adj: "gnomish",
    filecode: "Gno",
    attr_base: AttributeArray::new(0, 1, 0, 1, 0, -2),
    attr_max: AttributeArray::new(18, 19, 18, 18, 18, 18),
    hp_bonus: 1,
    en_bonus: 2,
    allowed_aligns: ROLE_NEUTRAL,
};

//
static ORC_RACE: RaceData = RaceData {
    name: "Orc",
    adj: "orcish",
    filecode: "Orc",
    attr_base: AttributeArray::new(0, -1, 0, 0, 1, -2),
    attr_max: AttributeArray::new(18, 16, 16, 18, 18, 16),
    hp_bonus: 1,
    en_bonus: 1,
    allowed_aligns: ROLE_CHAOTIC,
};

// =============================================================================
//
// =============================================================================

///
///
pub fn rank_of(role: PlayerClass, exp_level: i32) -> &'static str {
    let data = get_role_data(role);
    let idx = match exp_level {
        1..=2 => 0,
        3..=5 => 1,
        6..=9 => 2,
        10..=13 => 3,
        14..=17 => 4,
        18..=21 => 5,
        22..=25 => 6,
        26..=29 => 7,
        _ => 8,
    };
    data.rank_titles[idx]
}

///
pub fn ok_race(role: PlayerClass, race: Race) -> bool {
    let role_data = get_role_data(role);
    let race_bit = match race {
        Race::Human => MH_HUMAN,
        Race::Elf => MH_ELF,
        Race::Dwarf => MH_DWARF,
        Race::Gnome => MH_GNOME,
        Race::Orc => MH_ORC,
    };
    (role_data.allowed_races & race_bit) != 0
}

///
pub fn ok_alignment(role: PlayerClass, align: Alignment) -> bool {
    let role_data = get_role_data(role);
    let align_bit = match align {
        Alignment::Lawful => ROLE_LAWFUL,
        Alignment::Neutral => ROLE_NEUTRAL,
        Alignment::Chaotic => ROLE_CHAOTIC,
    };
    (role_data.allowed_aligns & align_bit) != 0
}

///
pub fn valid_combination(role: PlayerClass, race: Race, align: Alignment) -> bool {
    if !ok_race(role, race) {
        return false;
    }
    if !ok_alignment(role, align) {
        return false;
    }

    //
    let race_data = get_race_data(race);
    let align_bit = match align {
        Alignment::Lawful => ROLE_LAWFUL,
        Alignment::Neutral => ROLE_NEUTRAL,
        Alignment::Chaotic => ROLE_CHAOTIC,
    };
    (race_data.allowed_aligns & align_bit) != 0
}

///
pub fn initial_hp(role: PlayerClass, race: Race) -> i32 {
    let rd = get_role_data(role);
    let rc = get_race_data(race);
    rd.hp_adv.init_hp + rd.hp_adv.init_add + rc.hp_bonus
}

///
pub fn levelup_hp(role: PlayerClass, _race: Race, level: i32) -> i32 {
    let rd = get_role_data(role);
    let base = rd.hp_adv.level_hp;
    let bonus = rd.hp_adv.level_add;
    //
    if level >= rd.cutoff_level {
        (base + bonus) / 2
    } else {
        base + bonus
    }
}

///
pub fn initial_energy(role: PlayerClass, race: Race) -> i32 {
    let rd = get_role_data(role);
    let rc = get_race_data(race);
    rd.en_adv.init_en + rd.en_adv.init_add + rc.en_bonus
}

///
pub fn levelup_energy(role: PlayerClass, _race: Race, level: i32) -> i32 {
    let rd = get_role_data(role);
    let base = rd.en_adv.level_en;
    let bonus = rd.en_adv.level_add;
    if level >= rd.cutoff_level {
        (base + bonus) / 2
    } else {
        base + bonus
    }
}

///
pub fn spell_penalty(role: PlayerClass) -> i32 {
    get_role_data(role).spell_penalty
}

///
pub fn spell_stat(role: PlayerClass) -> SpellStat {
    get_role_data(role).spell_stat
}

///
pub fn god_name(role: PlayerClass, align: Alignment) -> &'static str {
    let data = get_role_data(role);
    match align {
        Alignment::Lawful => data.gods[0],
        Alignment::Neutral => data.gods[1],
        Alignment::Chaotic => data.gods[2],
    }
}

///
pub fn role_if(current: PlayerClass, check: PlayerClass) -> bool {
    current == check
}

///
pub fn all_valid_combinations() -> Vec<(PlayerClass, Race, Alignment)> {
    let roles = [
        PlayerClass::Archeologist,
        PlayerClass::Barbarian,
        PlayerClass::Healer,
        PlayerClass::Knight,
        PlayerClass::Monk,
        PlayerClass::Priest,
        PlayerClass::Ranger,
        PlayerClass::Rogue,
        PlayerClass::Samurai,
        PlayerClass::Tourist,
        PlayerClass::Valkyrie,
        PlayerClass::Wizard,
    ];
    let races = [Race::Human, Race::Elf, Race::Dwarf, Race::Gnome, Race::Orc];
    let aligns = [Alignment::Lawful, Alignment::Neutral, Alignment::Chaotic];

    let mut combos = Vec::new();
    for &role in &roles {
        for &race in &races {
            for &align in &aligns {
                if valid_combination(role, race, align) {
                    combos.push((role, race, align));
                }
            }
        }
    }
    combos
}

// =============================================================================
//
// =============================================================================

///
pub fn exp_for_level(level: i32) -> u64 {
    match level {
        1 => 0,
        2 => 20,
        3 => 40,
        4 => 80,
        5 => 160,
        6 => 320,
        7 => 640,
        8 => 1280,
        9 => 2560,
        10 => 5120,
        11 => 10000,
        12 => 20000,
        13 => 40000,
        14 => 80000,
        15 => 160000,
        16 => 320000,
        17 => 640000,
        18 => 1280000,
        19 => 2560000,
        20 => 5120000,
        21 => 10000000,
        22 => 20000000,
        23 => 40000000,
        24 => 80000000,
        25 => 160000000,
        26 => 320000000,
        27 => 640000000,
        28 => 1280000000,
        29 => 2560000000,
        30 => 5000000000,
        _ => 10000000000,
    }
}

///
pub fn level_from_exp(exp: u64) -> i32 {
    for lv in (1..=30).rev() {
        if exp >= exp_for_level(lv) {
            return lv;
        }
    }
    1
}

///
pub fn monster_kill_exp(monster_level: i32, monster_difficulty: i32) -> u64 {
    //
    let base = (monster_level as u64) * (monster_level as u64);
    let diff_bonus = monster_difficulty as u64;
    base + diff_bonus + 1
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_of() {
        assert_eq!(rank_of(PlayerClass::Valkyrie, 1), "Stripling");
        assert_eq!(rank_of(PlayerClass::Valkyrie, 30), "Lord");
        assert_eq!(rank_of(PlayerClass::Wizard, 1), "Evoker");
        assert_eq!(rank_of(PlayerClass::Wizard, 30), "Mage");
    }

    #[test]
    fn test_ok_race() {
        assert!(ok_race(PlayerClass::Wizard, Race::Human));
        assert!(ok_race(PlayerClass::Wizard, Race::Elf));
        assert!(!ok_race(PlayerClass::Wizard, Race::Dwarf));
        assert!(ok_race(PlayerClass::Knight, Race::Human));
        assert!(!ok_race(PlayerClass::Knight, Race::Orc));
    }

    #[test]
    fn test_ok_alignment() {
        assert!(ok_alignment(PlayerClass::Knight, Alignment::Lawful));
        assert!(!ok_alignment(PlayerClass::Knight, Alignment::Chaotic));
        assert!(ok_alignment(PlayerClass::Rogue, Alignment::Chaotic));
        assert!(!ok_alignment(PlayerClass::Rogue, Alignment::Lawful));
    }

    #[test]
    fn test_valid_combination() {
        assert!(valid_combination(
            PlayerClass::Valkyrie,
            Race::Human,
            Alignment::Neutral
        ));
        assert!(valid_combination(
            PlayerClass::Valkyrie,
            Race::Dwarf,
            Alignment::Lawful
        ));
        assert!(!valid_combination(
            PlayerClass::Valkyrie,
            Race::Elf,
            Alignment::Neutral
        ));
    }

    #[test]
    fn test_initial_hp() {
        let hp = initial_hp(PlayerClass::Barbarian, Race::Human);
        assert!(hp >= 14);
    }

    #[test]
    fn test_exp_table() {
        assert_eq!(exp_for_level(1), 0);
        assert_eq!(exp_for_level(2), 20);
        assert_eq!(level_from_exp(0), 1);
        assert_eq!(level_from_exp(20), 2);
        assert_eq!(level_from_exp(5000000000), 30);
    }

    #[test]
    fn test_all_combos() {
        let combos = all_valid_combinations();
        assert!(combos.len() > 20);
    }

    #[test]
    fn test_god_names() {
        let name = god_name(PlayerClass::Valkyrie, Alignment::Lawful);
        assert_eq!(name, "Tyr");
        let name = god_name(PlayerClass::Valkyrie, Alignment::Chaotic);
        assert_eq!(name, "Loki");
    }
}

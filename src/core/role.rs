// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//!
//!
//!
//!
//!

use serde::{Deserialize, Serialize};

// ============================================================
//
// ============================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Archeologist,
    Barbarian,
    Caveman,
    Healer,
    Knight,
    Monk,
    Priest,
    Rogue,
    Ranger,
    Samurai,
    Tourist,
    Valkyrie,
    Wizard,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Race {
    Human,
    Elf,
    Dwarf,
    Gnome,
    Orc,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Alignment {
    Lawful,
    Neutral,
    Chaotic,
}

// ============================================================
//
// ============================================================

///
#[derive(Debug, Clone, Copy)]
pub struct BaseStats {
    pub str_: i32,
    pub int: i32,
    pub wis: i32,
    pub dex: i32,
    pub con: i32,
    pub cha: i32,
}

///
#[derive(Debug, Clone)]
pub struct RoleData {
    pub role: Role,
    pub name: &'static str,
    pub name_female: Option<&'static str>,
    pub abbrev: &'static str,
    pub base_stats: BaseStats,
    pub base_hp: i32,
    pub base_pw: i32,
    pub base_ac: i32,
    ///
    pub valid_races: &'static [Race],
    ///
    pub valid_genders: &'static [Gender],
    ///
    pub valid_aligns: &'static [Alignment],
    ///
    pub ranks: [&'static str; 9],
    ///
    pub gods: (&'static str, &'static str, &'static str),
}

///
///
pub static ROLES: &[RoleData] = &[
    // 1. Archeologist (role.c L28-69)
    // MH_HUMAN | MH_DWARF | MH_GNOME | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL
    RoleData {
        role: Role::Archeologist,
        name: "Archeologist",
        name_female: None,
        abbrev: "Arc",
        base_stats: BaseStats {
            str_: 7,
            int: 10,
            wis: 10,
            dex: 7,
            con: 7,
            cha: 7,
        },
        base_hp: 11,
        base_pw: 1,
        base_ac: -4,
        valid_races: &[Race::Human, Race::Dwarf, Race::Gnome],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Lawful, Alignment::Neutral],
        ranks: [
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
        gods: ("Quetzalcoatl", "Camaxtli", "Huhetotl"),
    },
    // 2. Barbarian (role.c L70-111)
    // MH_HUMAN | MH_ORC | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL | ROLE_CHAOTIC
    RoleData {
        role: Role::Barbarian,
        name: "Barbarian",
        name_female: None,
        abbrev: "Bar",
        base_stats: BaseStats {
            str_: 16,
            int: 7,
            wis: 7,
            dex: 15,
            con: 16,
            cha: 6,
        },
        base_hp: 14,
        base_pw: 1,
        base_ac: -4,
        valid_races: &[Race::Human, Race::Orc],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Neutral, Alignment::Chaotic],
        ranks: [
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
        gods: ("Mitra", "Crom", "Set"),
    },
    // 3. Caveman (role.c L112-153)
    // MH_HUMAN | MH_DWARF | MH_GNOME | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL
    RoleData {
        role: Role::Caveman,
        name: "Caveman",
        name_female: Some("Cavewoman"),
        abbrev: "Cav",
        base_stats: BaseStats {
            str_: 10,
            int: 7,
            wis: 7,
            dex: 7,
            con: 8,
            cha: 6,
        },
        base_hp: 14,
        base_pw: 1,
        base_ac: -4,
        valid_races: &[Race::Human, Race::Dwarf, Race::Gnome],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Lawful, Alignment::Neutral],
        ranks: [
            "Troglodyte",
            "Aborigine",
            "Wanderer",
            "Vagrant",
            "Wayfarer",
            "Roamer",
            "Nomad",
            "Rover",
            "Pioneer",
        ],
        gods: ("Anu", "Ishtar", "Anshar"),
    },
    // 4. Healer (role.c L154-194)
    // MH_HUMAN | MH_GNOME | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL
    RoleData {
        role: Role::Healer,
        name: "Healer",
        name_female: None,
        abbrev: "Hea",
        base_stats: BaseStats {
            str_: 7,
            int: 7,
            wis: 13,
            dex: 7,
            con: 11,
            cha: 16,
        },
        base_hp: 11,
        base_pw: 1,
        base_ac: -4,
        valid_races: &[Race::Human, Race::Gnome],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Neutral],
        ranks: [
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
        gods: ("Athena", "Hermes", "Poseidon"),
    },
    // 5. Knight (role.c L195-235)
    // MH_HUMAN | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL
    RoleData {
        role: Role::Knight,
        name: "Knight",
        name_female: None,
        abbrev: "Kni",
        base_stats: BaseStats {
            str_: 13,
            int: 7,
            wis: 14,
            dex: 8,
            con: 10,
            cha: 17,
        },
        base_hp: 14,
        base_pw: 1,
        base_ac: -4,
        valid_races: &[Race::Human],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Lawful],
        ranks: [
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
        gods: ("Lugh", "Brigit", "Manannan Mac Lir"),
    },
    // 6. Monk (role.c L236-277)
    // MH_HUMAN | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL | ROLE_CHAOTIC
    RoleData {
        role: Role::Monk,
        name: "Monk",
        name_female: None,
        abbrev: "Mon",
        base_stats: BaseStats {
            str_: 10,
            int: 7,
            wis: 8,
            dex: 8,
            con: 7,
            cha: 7,
        },
        base_hp: 12,
        base_pw: 2,
        base_ac: -4,
        valid_races: &[Race::Human],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Lawful, Alignment::Neutral, Alignment::Chaotic],
        ranks: [
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
        gods: ("Shan Lai Ching", "Chih Sung-tzu", "Huan Ti"),
    },
    // 7. Priest (role.c L278-319)
    // MH_HUMAN | MH_ELF | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL | ROLE_CHAOTIC
    RoleData {
        role: Role::Priest,
        name: "Priest",
        name_female: Some("Priestess"),
        abbrev: "Pri",
        base_stats: BaseStats {
            str_: 7,
            int: 7,
            wis: 10,
            dex: 7,
            con: 7,
            cha: 7,
        },
        base_hp: 12,
        base_pw: 4,
        base_ac: -4,
        valid_races: &[Race::Human, Race::Elf],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Lawful, Alignment::Neutral, Alignment::Chaotic],
        ranks: [
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
        gods: ("", "", ""),
    },
    //
    // MH_HUMAN | MH_ORC | ROLE_MALE | ROLE_FEMALE | ROLE_CHAOTIC
    RoleData {
        role: Role::Rogue,
        name: "Rogue",
        name_female: None,
        abbrev: "Rog",
        base_stats: BaseStats {
            str_: 7,
            int: 7,
            wis: 7,
            dex: 10,
            con: 7,
            cha: 6,
        },
        base_hp: 10,
        base_pw: 1,
        base_ac: -4,
        valid_races: &[Race::Human, Race::Orc],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Chaotic],
        ranks: [
            "Footpad", "Cutpurse", "Rogue", "Pilferer", "Robber", "Burglar", "Filcher", "Magsman",
            "Thief",
        ],
        gods: ("Issek", "Mog", "Kos"),
    },
    // 9. Ranger (role.c L363-418)
    // MH_HUMAN | MH_ELF | MH_GNOME | MH_ORC | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL | ROLE_CHAOTIC
    RoleData {
        role: Role::Ranger,
        name: "Ranger",
        name_female: None,
        abbrev: "Ran",
        base_stats: BaseStats {
            str_: 13,
            int: 13,
            wis: 13,
            dex: 9,
            con: 13,
            cha: 7,
        },
        base_hp: 13,
        base_pw: 1,
        base_ac: -4,
        valid_races: &[Race::Human, Race::Elf, Race::Gnome, Race::Orc],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Neutral, Alignment::Chaotic],
        ranks: [
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
        gods: ("Mercury", "Venus", "Mars"),
    },
    // 10. Samurai (role.c L419-459)
    // MH_HUMAN | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL
    RoleData {
        role: Role::Samurai,
        name: "Samurai",
        name_female: None,
        abbrev: "Sam",
        base_stats: BaseStats {
            str_: 10,
            int: 8,
            wis: 7,
            dex: 10,
            con: 17,
            cha: 6,
        },
        base_hp: 13,
        base_pw: 1,
        base_ac: -4,
        valid_races: &[Race::Human],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Lawful],
        ranks: [
            "Hatamoto", "Ronin", "Ninja", "Joshu", "Ryoshu", "Kokushu", "Daimyo", "Kuge", "Shogun",
        ],
        gods: ("Amaterasu Omikami", "Raijin", "Susanowo"),
    },
    // 11. Tourist (role.c L460-500)
    // MH_HUMAN | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL
    RoleData {
        role: Role::Tourist,
        name: "Tourist",
        name_female: None,
        abbrev: "Tou",
        base_stats: BaseStats {
            str_: 7,
            int: 10,
            wis: 6,
            dex: 7,
            con: 7,
            cha: 10,
        },
        base_hp: 8,
        base_pw: 1,
        base_ac: -4,
        valid_races: &[Race::Human],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Neutral],
        ranks: [
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
        gods: ("Blind Io", "The Lady", "Offler"),
    },
    // 12. Valkyrie (role.c L501-541)
    // MH_HUMAN | MH_DWARF | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL
    RoleData {
        role: Role::Valkyrie,
        name: "Valkyrie",
        name_female: None,
        abbrev: "Val",
        base_stats: BaseStats {
            str_: 10,
            int: 7,
            wis: 7,
            dex: 7,
            con: 10,
            cha: 7,
        },
        base_hp: 14,
        base_pw: 1,
        base_ac: -4,
        valid_races: &[Race::Human, Race::Dwarf],
        valid_genders: &[Gender::Female],
        valid_aligns: &[Alignment::Lawful, Alignment::Neutral],
        ranks: [
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
        gods: ("Tyr", "Odin", "Loki"),
    },
    // 13. Wizard (role.c L542-583)
    // MH_HUMAN | MH_ELF | MH_GNOME | MH_ORC | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL | ROLE_CHAOTIC
    RoleData {
        role: Role::Wizard,
        name: "Wizard",
        name_female: None,
        abbrev: "Wiz",
        base_stats: BaseStats {
            str_: 7,
            int: 10,
            wis: 7,
            dex: 7,
            con: 7,
            cha: 7,
        },
        base_hp: 10,
        base_pw: 4,
        base_ac: -4,
        valid_races: &[Race::Human, Race::Elf, Race::Gnome, Race::Orc],
        valid_genders: &[Gender::Male, Gender::Female],
        valid_aligns: &[Alignment::Neutral, Alignment::Chaotic],
        ranks: [
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
        gods: ("Ptah", "Thoth", "Anhur"),
    },
];

// ============================================================
//
// ============================================================

///
#[derive(Debug, Clone)]
pub struct RaceData {
    pub race: Race,
    pub name: &'static str,
    pub adjective: &'static str,
    pub abbrev: &'static str,
    ///
    pub valid_aligns: &'static [Alignment],
    ///
    pub hp_bonus: i32,
    ///
    pub pw_bonus: i32,
}

///
pub static RACES: &[RaceData] = &[
    // Human (role.c L618-639)
    // MH_HUMAN | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL | ROLE_CHAOTIC
    RaceData {
        race: Race::Human,
        name: "Human",
        adjective: "human",
        abbrev: "Hum",
        valid_aligns: &[Alignment::Lawful, Alignment::Neutral, Alignment::Chaotic],
        hp_bonus: 2,
        pw_bonus: 1,
    },
    // Elf (role.c L640-660)
    // MH_ELF | ROLE_MALE | ROLE_FEMALE | ROLE_CHAOTIC
    RaceData {
        race: Race::Elf,
        name: "Elf",
        adjective: "elven",
        abbrev: "Elf",
        valid_aligns: &[Alignment::Chaotic],
        hp_bonus: 1,
        pw_bonus: 2,
    },
    // Dwarf (role.c L661-681)
    // MH_DWARF | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL
    RaceData {
        race: Race::Dwarf,
        name: "Dwarf",
        adjective: "dwarven",
        abbrev: "Dwa",
        valid_aligns: &[Alignment::Lawful],
        hp_bonus: 4,
        pw_bonus: 0,
    },
    // Gnome (role.c L682-702)
    // MH_GNOME | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL
    RaceData {
        race: Race::Gnome,
        name: "Gnome",
        adjective: "gnomish",
        abbrev: "Gno",
        valid_aligns: &[Alignment::Neutral],
        hp_bonus: 1,
        pw_bonus: 2,
    },
    // Orc (role.c L703-723)
    // MH_ORC | ROLE_MALE | ROLE_FEMALE | ROLE_CHAOTIC
    RaceData {
        race: Race::Orc,
        name: "Orc",
        adjective: "orcish",
        abbrev: "Orc",
        valid_aligns: &[Alignment::Chaotic],
        hp_bonus: 1,
        pw_bonus: 1,
    },
];

// ============================================================
//
// ============================================================

///
pub fn get_role_data(role: Role) -> &'static RoleData {
    ROLES
        .iter()
        .find(|r| r.role == role)
        .expect("[R8] 정적 ROLES 배열에 누락된 직업")
}

///
pub fn get_race_data(race: Race) -> &'static RaceData {
    RACES
        .iter()
        .find(|r| r.race == race)
        .expect("[R8] 정적 RACES 배열에 누락된 종족")
}

///
///
pub fn is_valid_race(role: Role, race: Race) -> bool {
    let rd = get_role_data(role);
    rd.valid_races.contains(&race)
}

///
pub fn is_valid_gender(role: Role, gender: Gender) -> bool {
    let rd = get_role_data(role);
    rd.valid_genders.contains(&gender)
}

///
///
pub fn is_valid_alignment(role: Role, race: Race, align: Alignment) -> bool {
    let role_data = get_role_data(role);
    let race_data = get_race_data(race);
    role_data.valid_aligns.contains(&align) && race_data.valid_aligns.contains(&align)
}

///
pub fn valid_races_for_role(role: Role) -> Vec<Race> {
    get_role_data(role).valid_races.to_vec()
}

///
pub fn valid_alignments_for(role: Role, race: Race) -> Vec<Alignment> {
    let role_data = get_role_data(role);
    let race_data = get_race_data(race);
    role_data
        .valid_aligns
        .iter()
        .filter(|a| race_data.valid_aligns.contains(a))
        .copied()
        .collect()
}

///
pub fn valid_genders_for_role(role: Role) -> Vec<Gender> {
    get_role_data(role).valid_genders.to_vec()
}

///
pub fn all_roles() -> &'static [Role] {
    &[
        Role::Archeologist,
        Role::Barbarian,
        Role::Caveman,
        Role::Healer,
        Role::Knight,
        Role::Monk,
        Role::Priest,
        Role::Rogue,
        Role::Ranger,
        Role::Samurai,
        Role::Tourist,
        Role::Valkyrie,
        Role::Wizard,
    ]
}

///
pub fn all_races() -> &'static [Race] {
    &[Race::Human, Race::Elf, Race::Dwarf, Race::Gnome, Race::Orc]
}

// ============================================================
//
// ============================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharCreationStep {
    SelectRole,
    SelectRace,
    SelectGender,
    SelectAlignment,
    EnterName,
    Confirm,
}

///
#[derive(Debug, Clone)]
pub struct CharCreationChoices {
    pub role: Option<Role>,
    pub race: Option<Race>,
    pub gender: Option<Gender>,
    pub alignment: Option<Alignment>,
    pub name: String,
}

impl CharCreationChoices {
    pub fn new() -> Self {
        Self {
            role: None,
            race: None,
            gender: None,
            alignment: None,
            name: String::new(),
        }
    }

    ///
    pub fn role_display_name(&self) -> String {
        if let Some(role) = self.role {
            let data = get_role_data(role);
            if self.gender == Some(Gender::Female) {
                if let Some(female_name) = data.name_female {
                    return female_name.to_string();
                }
            }
            data.name.to_string()
        } else {
            "???".to_string()
        }
    }
}

// ============================================================
//
// ============================================================

///
///
#[derive(Debug, Clone)]
pub enum AppState {
    ///
    Title,
    ///
    CharacterCreation {
        step: CharCreationStep,
        choices: CharCreationChoices,
    },
    ///
    Playing,
    ///
    GameOver {
        ///
        message: String,
        ///
        score: u64,
        ///
        turns: u64,
        ///
        max_depth: i32,
        ///
        epitaph: String,
    },
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Title
    }
}

// ============================================================
// 7. 직업별 표시 문자 (AIHack UI용)
// ============================================================

impl Role {
    ///
    pub fn icon(&self) -> &'static str {
        match self {
            Role::Archeologist => "[Arc]",
            Role::Barbarian => "[Bar]",
            Role::Caveman => "[Cav]",
            Role::Healer => "[Hea]",
            Role::Knight => "[Kni]",
            Role::Monk => "[Mon]",
            Role::Priest => "[Pri]",
            Role::Rogue => "[Rog]",
            Role::Ranger => "[Ran]",
            Role::Samurai => "[Sam]",
            Role::Tourist => "[Tou]",
            Role::Valkyrie => "[Val]",
            Role::Wizard => "[Wiz]",
        }
    }

    /// 직업의 주요 특성 요약
    pub fn description(&self) -> &'static str {
        match self {
            Role::Archeologist => "Explores dungeons, identifies artifacts",
            Role::Barbarian => "Strong melee fighter, high HP",
            Role::Caveman => "Primitive warrior, sturdy",
            Role::Healer => "Heals allies, alchemy expert",
            Role::Knight => "Chivalrous warrior, rides horses",
            Role::Monk => "Martial artist, fights unarmed",
            Role::Priest => "Divine magic, undead turning",
            Role::Rogue => "Sneaky thief, backstab expert",
            Role::Ranger => "Ranged combat, nature skills",
            Role::Samurai => "Bushido warrior, katana master",
            Role::Tourist => "Unlikely hero, credit card wielder",
            Role::Valkyrie => "Norse warrior maiden, cold resist",
            Role::Wizard => "Master of arcane magic",
        }
    }
}

impl Race {
    /// 종족의 주요 특성 요약
    pub fn description(&self) -> &'static str {
        match self {
            Race::Human => "Versatile, all alignments",
            Race::Elf => "High magic power, infravision",
            Race::Dwarf => "High HP, lawful, tunneling",
            Race::Gnome => "Neutral, gnomish mines friend",
            Race::Orc => "Chaotic, poison resist, infravision",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_role_data(*self).name)
    }
}

impl std::fmt::Display for Race {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_race_data(*self).name)
    }
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gender::Male => write!(f, "Male"),
            Gender::Female => write!(f, "Female"),
        }
    }
}

impl std::fmt::Display for Alignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Alignment::Lawful => write!(f, "Lawful"),
            Alignment::Neutral => write!(f, "Neutral"),
            Alignment::Chaotic => write!(f, "Chaotic"),
        }
    }
}

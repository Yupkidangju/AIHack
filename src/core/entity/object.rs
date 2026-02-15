// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

bitflags! {
    ///
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct ItemBits: u32 {
        const NMKN = 0x0001; // oc_name_known
        const MRG  = 0x0002; // oc_merge
        const USKN = 0x0004; // oc_uses_known
        const PRE  = 0x0008; // oc_pre_discovered
        const MGC  = 0x0010; // oc_magic
        const CHRG = 0x0020; // oc_charged
        const UNIQ = 0x0040; // oc_unique
        const NWSH = 0x0080; // oc_nowish
        const BIG  = 0x0100; // oc_big (bimanual/bulky)
        const TUF  = 0x0200; // oc_tough
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ItemClass {
    Random = 0,
    IllObj = 1,
    Weapon = 2,
    Armor = 3,
    Ring = 4,
    Amulet = 5,
    Tool = 6,
    Food = 7,
    Potion = 8,
    Scroll = 9,
    Spellbook = 10,
    Wand = 11,
    Coin = 12,
    Gem = 13,
    Rock = 14,
    Ball = 15,
    Chain = 16,
    Venom = 17,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Material {
    Liquid = 1,
    Wax = 2,
    Veggy = 3,
    Flesh = 4,
    Paper = 5,
    Cloth = 6,
    Leather = 7,
    Wood = 8,
    Bone = 9,
    DragonHide = 10,
    Iron = 11,
    Metal = 12,
    Copper = 13,
    Silver = 14,
    Gold = 15,
    Platinum = 16,
    Mithril = 17,
    Plastic = 18,
    Glass = 19,
    Gemstone = 20,
    Mineral = 21,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ArmorType {
    Suit = 0,
    Shield = 1,
    Helm = 2,
    Gloves = 3,
    Boots = 4,
    Cloak = 5,
    Shirt = 6,
    None = 255,
}

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTemplate {
    pub name: String,
    pub description: Option<String>,
    pub bits: u32,
    pub dir: u8, // oc_dir (NODIR, IMMEDIATE, RAY)
    pub material: Material,
    pub class: ItemClass,
    pub subtype: i8,    // oc_subtyp (skill, armcat ??
    pub prop: u8,
    pub delay: i8,      // oc_delay
    pub color: u8,      // oc_color
    pub prob: i16,      // oc_prob
    pub weight: u16,    // oc_weight
    pub cost: i16,      // oc_cost
    pub wsdam: i8,      // oc_wsdam (small damage)
    pub wldam: i8,      // oc_wldam (large damage)
    pub oc1: i8,        // oc_oc1 (hitbon, ac)
    pub oc2: i8,        // oc_oc2 (level, can)
    pub nutrition: u16, // oc_nutrition
}

///
pub const NODIR: u8 = 1;
pub const IMMEDIATE: u8 = 2;
pub const RAY: u8 = 3;

///
pub const WHACK: i8 = 0;
pub const PIERCE: i8 = 1;
pub const SLASH: i8 = 2;

use std::collections::HashMap;

///
///
///
#[derive(Clone)]
pub struct ItemManager {
    ///
    pub templates: HashMap<String, ItemTemplate>,
    ///
    kind_index: HashMap<crate::generated::ItemKind, String>,
}

impl ItemManager {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            kind_index: HashMap::new(),
        }
    }

    ///
    pub fn get_template(&self, name: &str) -> Option<&ItemTemplate> {
        self.templates.get(name)
    }

    ///
    ///
    ///
    pub fn get_by_kind(&self, kind: crate::generated::ItemKind) -> Option<&ItemTemplate> {
        self.kind_index
            .get(&kind)
            .and_then(|name| self.templates.get(name))
    }

    pub fn get_templates_by_class(&self, class: ItemClass) -> Vec<&ItemTemplate> {
        self.templates
            .values()
            .filter(|t| t.class == class)
            .collect()
    }

    ///
    ///
    pub fn build_kind_index(&mut self) {
        self.kind_index.clear();
        for (name, _template) in &self.templates {
            let kind = crate::generated::ItemKind::from_str(name);
            //
            if kind != crate::generated::ItemKind::UnknownItem {
                self.kind_index.insert(kind, name.clone());
            }
        }
    }

    ///
    pub fn load_defaults(&mut self) {
        //
    }
}

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

use crate::core::entity::object::{ItemClass, ItemTemplate};
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct MkObjRequest {
    pub class: Option<ItemClass>,
    pub name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub blessed: Option<bool>,
    pub cursed: Option<bool>,
    pub spe: Option<i8>,
    pub quantity: Option<u32>,
    pub no_curse: bool,
}

impl Default for MkObjRequest {
    fn default() -> Self {
        Self {
            class: None,
            name: None,
            x: 0,
            y: 0,
            blessed: None,
            cursed: None,
            spe: None,
            quantity: None,
            no_curse: false,
        }
    }
}

///
#[derive(Debug, Clone)]
pub struct MkObjResult {
    pub template_name: String,
    pub class: ItemClass,
    pub x: i32,
    pub y: i32,
    pub blessed: bool,
    pub cursed: bool,
    pub spe: i8,
    pub quantity: u32,
    pub weight: u32,
    pub price: u32,
    pub known: bool,
    pub bknown: bool,
    pub dknown: bool,
}

// =============================================================================
//
// =============================================================================

///
pub fn random_item_class(rng: &mut NetHackRng) -> ItemClass {
    //
    let roll = rng.rn2(100);
    if roll < 10 {
        ItemClass::Weapon
    } else if roll < 18 {
        ItemClass::Armor
    } else if roll < 22 {
        ItemClass::Ring
    } else if roll < 25 {
        ItemClass::Amulet
    } else if roll < 32 {
        ItemClass::Tool
    } else if roll < 42 {
        ItemClass::Food
    } else if roll < 52 {
        ItemClass::Potion
    } else if roll < 62 {
        ItemClass::Scroll
    } else if roll < 66 {
        ItemClass::Spellbook
    } else if roll < 72 {
        ItemClass::Wand
    } else if roll < 82 {
        ItemClass::Coin
    } else if roll < 92 {
        ItemClass::Gem
    } else {
        ItemClass::Rock
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn determine_buc(rng: &mut NetHackRng, no_curse: bool) -> (bool, bool) {
    let roll = rng.rn2(100);
    if no_curse {
        //
        if roll < 15 {
            (true, false)
        }
        //
        else {
            (false, false)
        }
    } else {
        if roll < 10 {
            (true, false)
        }
        //
        else if roll < 25 {
            (false, true)
        }
        //
        else {
            (false, false)
        }
    }
}

///
pub fn determine_spe(class: ItemClass, blessed: bool, cursed: bool, rng: &mut NetHackRng) -> i8 {
    match class {
        ItemClass::Weapon | ItemClass::Armor => {
            if blessed {
                (rng.rn2(4) + 1) as i8
            } else if cursed {
                -(rng.rn2(4) + 1) as i8
            } else {
                rng.rn2(3) as i8
            }
        }
        ItemClass::Ring => {
            if cursed {
                -(rng.rn2(3) + 1) as i8
            } else {
                (rng.rn2(3) + 1) as i8
            }
        }
        ItemClass::Wand => {
            //
            (rng.rn2(5) + 3) as i8
        }
        _ => 0,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn mkobj(
    request: &MkObjRequest,
    templates: &std::collections::HashMap<String, ItemTemplate>,
    rng: &mut NetHackRng,
) -> Option<MkObjResult> {
    let class = request.class.unwrap_or_else(|| random_item_class(rng));

    //
    if let Some(ref name) = request.name {
        if let Some(tmpl) = templates.get(name) {
            let (bl, cu) = match (request.blessed, request.cursed) {
                (Some(b), Some(c)) => (b, c),
                _ => determine_buc(rng, request.no_curse),
            };
            let spe = request
                .spe
                .unwrap_or_else(|| determine_spe(class, bl, cu, rng));
            let qty = request.quantity.unwrap_or(1);
            return Some(MkObjResult {
                template_name: name.clone(),
                class: tmpl.class,
                x: request.x,
                y: request.y,
                blessed: bl,
                cursed: cu,
                spe,
                quantity: qty,
                weight: tmpl.weight as u32,
                price: tmpl.cost as u32,
                known: false,
                bknown: false,
                dknown: false,
            });
        }
    }

    //
    let class_items: Vec<(&String, &ItemTemplate)> =
        templates.iter().filter(|(_, t)| t.class == class).collect();

    if class_items.is_empty() {
        return None;
    }

    //
    let total_prob: i32 = class_items.iter().map(|(_, t)| t.prob as i32).sum();
    let mut roll = if total_prob > 0 {
        rng.rn2(total_prob)
    } else {
        0
    };

    let mut selected = &class_items[0];
    for item in &class_items {
        roll -= item.1.prob as i32;
        if roll < 0 {
            selected = item;
            break;
        }
    }

    let (bl, cu) = match (request.blessed, request.cursed) {
        (Some(b), Some(c)) => (b, c),
        _ => determine_buc(rng, request.no_curse),
    };
    let spe = request
        .spe
        .unwrap_or_else(|| determine_spe(class, bl, cu, rng));
    let qty = request.quantity.unwrap_or_else(|| match class {
        ItemClass::Coin => (rng.rn2(100) + 1) as u32,
        ItemClass::Gem | ItemClass::Rock => (rng.rn2(3) + 1) as u32,
        _ => 1,
    });

    Some(MkObjResult {
        template_name: selected.0.clone(),
        class,
        x: request.x,
        y: request.y,
        blessed: bl,
        cursed: cu,
        spe,
        quantity: qty,
        weight: selected.1.weight as u32,
        price: selected.1.cost as u32,
        known: false,
        bknown: false,
        dknown: false,
    })
}

// =============================================================================
//
// =============================================================================

///
pub fn mkgold(amount: i32, x: i32, y: i32) -> MkObjResult {
    MkObjResult {
        template_name: "gold piece".into(),
        class: ItemClass::Coin,
        x,
        y,
        blessed: false,
        cursed: false,
        spe: 0,
        quantity: amount.max(1) as u32,
        weight: 1,
        price: 1,
        known: true,
        bknown: true,
        dknown: true,
    }
}

///
pub fn mkcorpse(monster_name: &str, x: i32, y: i32, turn: u64) -> MkObjResult {
    MkObjResult {
        template_name: format!("{} corpse", monster_name),
        class: ItemClass::Food,
        x,
        y,
        blessed: false,
        cursed: false,
        spe: 0,
        quantity: 1,
        weight: 100,
        price: 0,
        known: true,
        bknown: false,
        dknown: true,
    }
}

///
pub fn mkstatue(monster_name: &str, x: i32, y: i32) -> MkObjResult {
    MkObjResult {
        template_name: format!("{} statue", monster_name),
        class: ItemClass::Rock,
        x,
        y,
        blessed: false,
        cursed: false,
        spe: 0,
        quantity: 1,
        weight: 450,
        price: 0,
        known: true,
        bknown: false,
        dknown: true,
    }
}

///
pub fn random_armor(
    templates: &std::collections::HashMap<String, ItemTemplate>,
    rng: &mut NetHackRng,
) -> Option<MkObjResult> {
    let req = MkObjRequest {
        class: Some(ItemClass::Armor),
        ..Default::default()
    };
    mkobj(&req, templates, rng)
}

///
pub fn random_weapon(
    templates: &std::collections::HashMap<String, ItemTemplate>,
    rng: &mut NetHackRng,
) -> Option<MkObjResult> {
    let req = MkObjRequest {
        class: Some(ItemClass::Weapon),
        ..Default::default()
    };
    mkobj(&req, templates, rng)
}

// =============================================================================
//
// =============================================================================

///
pub fn filter_by_difficulty(
    templates: &std::collections::HashMap<String, ItemTemplate>,
    class: ItemClass,
    dungeon_level: i32,
) -> Vec<String> {
    templates
        .iter()
        .filter(|(_, t)| t.class == class && (t.oc2 as i32) <= dungeon_level)
        .map(|(name, _)| name.clone())
        .collect()
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_item_class() {
        let mut rng = NetHackRng::new(42);
        let cls = random_item_class(&mut rng);
        //
        assert!(matches!(
            cls,
            ItemClass::Weapon
                | ItemClass::Armor
                | ItemClass::Ring
                | ItemClass::Amulet
                | ItemClass::Tool
                | ItemClass::Food
                | ItemClass::Potion
                | ItemClass::Scroll
                | ItemClass::Spellbook
                | ItemClass::Wand
                | ItemClass::Coin
                | ItemClass::Gem
                | ItemClass::Rock
        ));
    }

    #[test]
    fn test_determine_buc() {
        let mut rng = NetHackRng::new(42);
        //
        for _ in 0..1000 {
            let (_, cursed) = determine_buc(&mut rng, true);
            assert!(!cursed);
        }
    }

    #[test]
    fn test_mkgold() {
        let g = mkgold(100, 5, 5);
        assert_eq!(g.quantity, 100);
        assert_eq!(g.class, ItemClass::Coin);
    }

    #[test]
    fn test_mkcorpse() {
        let c = mkcorpse("kobold", 3, 3, 100);
        assert!(c.template_name.contains("kobold"));
        assert_eq!(c.class, ItemClass::Food);
    }
}

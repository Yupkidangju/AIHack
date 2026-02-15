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
use crate::assets::AssetManager;
use crate::core::entity::player::Player;
use crate::core::entity::{CombatStats, Health, PlayerTag, Renderable, Species};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;

///
#[legion::system]
#[write_component(Species)]
#[write_component(Health)]
#[write_component(CombatStats)]
#[write_component(Renderable)]
#[write_component(Player)]
#[read_component(PlayerTag)]
pub fn evolution_tick(
    world: &mut SubWorld,
    #[resource] assets: &AssetManager,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] _rng: &mut NetHackRng,
) {
    let mut query = <(Entity, &mut Species)>::query();
    let mut reversions = Vec::new();

    for (ent, species) in query.iter_mut(world) {
        if let Some(timer) = species.timer {
            if timer > 0 {
                species.timer = Some(timer - 1);
            } else {
                reversions.push((*ent, species.original.clone()));
            }
        }
    }

    //
    for (ent, original_name) in reversions {
        revert_polymorph(ent, &original_name, world, assets, log, *turn);
    }
}

pub fn revert_polymorph(
    ent: Entity,
    original_name: &str,
    world: &mut SubWorld,
    assets: &AssetManager,
    log: &mut GameLog,
    turn: u64,
) {
    if let Ok(mut entry) = world.entry_mut(ent) {
        let is_player = entry.get_component::<PlayerTag>().is_ok();

        if is_player {
            log.add("You return to your normal form.", turn);
            //
            if let Ok(species) = entry.get_component_mut::<Species>() {
                species.current = "player".to_string();
                species.timer = None;
            }
            if let Ok(render) = entry.get_component_mut::<Renderable>() {
                render.glyph = '@';
                render.color = 15; // White
            }
            //
        } else {
            //
            if let Some(template) = assets.monsters.templates.get(original_name) {
                if let Ok(species) = entry.get_component_mut::<Species>() {
                    species.current = original_name.to_string();
                    species.timer = None;
                }
                if let Ok(render) = entry.get_component_mut::<Renderable>() {
                    render.glyph = template.symbol;
                    render.color = template.color;
                }
            }
        }
    }
}

///
pub fn drain_level(
    ent: Entity,
    world: &mut SubWorld,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
) {
    if let Ok(mut entry) = world.entry_mut(ent) {
        let is_player = entry.get_component::<PlayerTag>().is_ok();

        if is_player {
            if let Ok(p_stats) = entry.get_component_mut::<Player>() {
                if p_stats.exp_level > 1 {
                    p_stats.exp_level -= 1;
                    log.add("You feel weaker! You lose a level!", turn);

                    //
                    let lose_hp = rng.rn1(8, 2);
                    p_stats.hp_max = (p_stats.hp_max - lose_hp).max(1);
                    p_stats.hp = p_stats.hp.min(p_stats.hp_max);

                    //
                    p_stats.experience = p_stats.experience / 2;
                }
            }
        } else {
            //
            if let Ok(stats) = entry.get_component_mut::<CombatStats>() {
                if stats.level > 1 {
                    stats.level -= 1;
                    log.add("The monster looks diminished.", turn);
                }
            }
        }
    }
}

///
#[legion::system]
#[read_component(crate::core::entity::status::StatusBundle)]
#[write_component(Species)]
#[write_component(Renderable)]
#[read_component(PlayerTag)]
pub fn lycanthropy_tick(
    world: &mut SubWorld,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] rng: &mut NetHackRng,
) {
    use crate::core::entity::status::StatusFlags;

    let mut query = <(
        Entity,
        &crate::core::entity::status::StatusBundle,
        &mut Species,
    )>::query();
    let mut transformations = Vec::new();

    for (ent, status, species) in query.iter_mut(world) {
        if status.has(StatusFlags::LYCANTHROPY) {
            //
            if rng.rn2(80) == 0 {
                if species.current == species.original {
                    //
                    transformations.push((*ent, "werewolf".to_string()));
                } else {
                    //
                    transformations.push((*ent, species.original.clone()));
                }
            }
        }
    }

    for (ent, target_form) in transformations {
        if let Ok(mut entry) = world.entry_mut(ent) {
            let is_player = entry.get_component::<PlayerTag>().is_ok();

            if target_form == "werewolf" {
                if is_player {
                    log.add("You turn into a creature of the night!", *turn);
                    if let Ok(render) = entry.get_component_mut::<Renderable>() {
                        render.glyph = 'd';
                        render.color = 8; // Dark Gray
                    }
                }
                if let Ok(sp) = entry.get_component_mut::<Species>() {
                    sp.current = "werewolf".to_string();
                }
            } else {
                //
                if is_player {
                    log.add("You feel purer.", *turn);
                    if let Ok(render) = entry.get_component_mut::<Renderable>() {
                        render.glyph = '@';
                        render.color = 15;
                    }
                }
                if let Ok(sp) = entry.get_component_mut::<Species>() {
                    sp.current = sp.original.clone();
                }
            }
        }
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
#[derive(Debug, Clone)]
pub struct PolyForm {
    ///
    pub name: String,
    ///
    pub hit_dice: i32,
    ///
    pub symbol: char,
    ///
    pub can_fly: bool,
    ///
    pub can_swim: bool,
    ///
    pub fire_res: bool,
    ///
    pub cold_res: bool,
    ///
    pub poison_res: bool,
    ///
    pub stone_res: bool,
    ///
    pub has_hands: bool,
    ///
    pub attack_damage: i32,
}

///
///
pub fn polymorph_ok(form_name: &str, player_level: i32) -> bool {
    let l = form_name.to_lowercase();

    //
    if is_unique_form(&l) {
        return false;
    }

    //
    if l == "player" || l == "shopkeeper" || l == "guard" || l == "high priest" {
        return false;
    }

    //
    let form_hd = estimate_poly_hd(&l);
    form_hd <= player_level
}

///
fn is_unique_form(name: &str) -> bool {
    //
    matches!(
        name,
        "wizard of yendor"
            | "medusa"
            | "croesus"
            | "death"
            | "pestilence"
            | "famine"
            | "demogorgon"
            | "juiblex"
            | "asmodeus"
            | "baalzebub"
            | "orcus"
            | "yeenoghu"
            | "vlad the impaler"
            | "rodney"
    )
}

///
fn estimate_poly_hd(form_name: &str) -> i32 {
    //
    match form_name {
        "dust vortex" | "newt" | "lichen" => 1,
        "kitten" | "little dog" | "cave spider" => 2,
        "wolf" | "pony" | "gnome" => 3,
        "large dog" | "orc" | "hobgoblin" => 4,
        "warhorse" | "troll" | "ogre" => 6,
        "dragon" | "jabberwock" | "titan" => 15,
        _ => 5,
    }
}

///
pub fn poly_hp(form_hd: i32, current_max_hp: i32, rng: &mut NetHackRng) -> i32 {
    //
    let new_hp = rng.d(form_hd.max(1), 8);
    //
    new_hp.max(1).min(current_max_hp * 2)
}

///
pub fn poly_duration(rng: &mut NetHackRng) -> u32 {
    //
    (rng.rn2(500) + 500) as u32
}

///
///
pub fn poly_equipment_check(
    target_form: &str,
    _has_hands: bool,
    is_body_change: bool,
) -> (i32, Vec<String>) {
    let mut dropped = 0;
    let mut messages = Vec::new();

    if is_body_change {
        let l = target_form.to_lowercase();

        //
        let humanoid = l.contains("human")
            || l.contains("elf")
            || l.contains("dwarf")
            || l.contains("gnome")
            || l.contains("orc");

        if !humanoid {
            //
            messages.push("Your armor falls off!".to_string());
            dropped += 4;

            //
            if !_has_hands {
                messages.push("You drop your weapon!".to_string());
                dropped += 1;
            }
        }

        //
        if !_has_hands {
            messages.push("Your rings slip off!".to_string());
            dropped += 2;
        }
    }

    (dropped, messages)
}

///
pub fn new_body_abilities(form_name: &str) -> Vec<&'static str> {
    let l = form_name.to_lowercase();
    let mut abilities = Vec::new();

    //
    if matches!(
        l.as_str(),
        "bat"
            | "raven"
            | "air elemental"
            | "ki-rin"
            | "couatl"
            | "winged gargoyle"
            | "yellow light"
            | "black light"
            | "gas spore"
            | "floating eye"
    ) {
        abilities.push("flight");
    }

    //
    if matches!(
        l.as_str(),
        "water elemental"
            | "kraken"
            | "electric eel"
            | "giant eel"
            | "shark"
            | "piranha"
            | "jellyfish"
    ) {
        abilities.push("swimming");
    }

    //
    if l.contains("fire")
        || l.contains("red dragon")
        || l.contains("salamander")
        || l.contains("fire giant")
        || l.contains("balrog")
        || l.contains("phoenix")
    {
        abilities.push("fire_resistance");
    }

    //
    if l.contains("snake")
        || l.contains("viper")
        || l.contains("asp")
        || l.contains("scorpion")
        || l.contains("pit viper")
        || l.contains("cobra")
    {
        abilities.push("poison_resistance");
    }

    //
    if l.contains("white dragon")
        || l.contains("ice")
        || l.contains("winter wolf")
        || l.contains("yeti")
        || l.contains("frost giant")
    {
        abilities.push("cold_resistance");
    }

    //
    if l.contains("stalker") || l.contains("invisible") {
        abilities.push("invisibility");
    }

    //
    if l.contains("troll") || l.contains("tengu") {
        abilities.push("regeneration");
    }

    abilities
}

///
pub fn poly_message(form_name: &str, _is_voluntary: bool) -> String {
    let l = form_name.to_lowercase();

    if l.contains("dragon") {
        "You feel a sudden surge of power!".to_string()
    } else if l == "newt" || l == "lichen" {
        "You feel very small...".to_string()
    } else if l.contains("giant") {
        "You feel enormous!".to_string()
    } else if l.contains("elemental") {
        "You merge with the elements!".to_string()
    } else if l.contains("jelly") || l.contains("ooze") || l.contains("slime") {
        "Your body begins to ooze...".to_string()
    } else {
        format!("You turn into {}!", a_or_an(&l))
    }
}

///
fn a_or_an(name: &str) -> String {
    let first = name.chars().next().unwrap_or('a');
    if "aeiou".contains(first) {
        format!("an {}", name)
    } else {
        format!("a {}", name)
    }
}

///
///
pub fn polytrap_effect(player_level: i32, rng: &mut NetHackRng) -> (String, i32) {
    //
    let forms = [
        "newt",
        "kitten",
        "wolf",
        "giant spider",
        "troll",
        "orc",
        "gnome",
        "pony",
        "bat",
        "snake",
    ];
    let idx = rng.rn2(forms.len() as i32) as usize;
    let form = forms[idx];

    let duration = (rng.rn2(100) + 50) as i32;
    let msg = format!(
        "A shimmering polymorph trap transforms you into {}!",
        a_or_an(form)
    );

    (msg, duration)
}

///
pub fn safe_poly_check(current_hp: i32, max_hp: i32, form_hd: i32) -> bool {
    //
    if current_hp < max_hp / 2 {
        return false;
    }
    //
    if form_hd > 20 {
        return false;
    }
    true
}

///
///
pub fn system_shock_check(constitution: i32, rng: &mut NetHackRng) -> bool {
    //
    rng.rn2(20) > constitution
}

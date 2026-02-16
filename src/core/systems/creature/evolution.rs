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
pub fn polytrap_effect(_player_level: i32, rng: &mut NetHackRng) -> (String, i32) {
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

// =============================================================================
// [v2.8.0] polyself.c 대량 이식  신체 부위/골렘/드래곤/성별/기타
// 원본: nethack-3.6.7/src/polyself.c (1,908줄)
// =============================================================================

/// [v2.8.0] 신체 부위 인덱스
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyPart {
    Arm = 0,
    Eye = 1,
    Face = 2,
    Finger = 3,
    Fingertip = 4,
    Foot = 5,
    Hand = 6,
    Handed = 7,
    Head = 8,
    Leg = 9,
    LightHeaded = 10,
    Neck = 11,
    Spine = 12,
    Toe = 13,
    Hair = 14,
    Blood = 15,
    Lung = 16,
    Nose = 17,
    Stomach = 18,
}

/// [v2.8.0] 체형 분류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    Humanoid,
    Jelly,
    Animal,
    Bird,
    Horse,
    Sphere,
    Fungus,
    Vortex,
    Snake,
    Worm,
    Fish,
}

const HUMANOID_PARTS: [&str; 19] = [
    "arm",
    "eye",
    "face",
    "finger",
    "fingertip",
    "foot",
    "hand",
    "handed",
    "head",
    "leg",
    "light headed",
    "neck",
    "spine",
    "toe",
    "hair",
    "blood",
    "lung",
    "nose",
    "stomach",
];
const JELLY_PARTS: [&str; 19] = [
    "pseudopod",
    "dark spot",
    "front",
    "pseudopod extension",
    "pseudopod extremity",
    "pseudopod root",
    "grasp",
    "grasped",
    "cerebral area",
    "lower pseudopod",
    "viscous",
    "middle",
    "surface",
    "pseudopod extremity",
    "ripples",
    "juices",
    "surface",
    "sensor",
    "stomach",
];
const ANIMAL_PARTS: [&str; 19] = [
    "forelimb",
    "eye",
    "face",
    "foreclaw",
    "claw tip",
    "rear claw",
    "foreclaw",
    "clawed",
    "head",
    "rear limb",
    "light headed",
    "neck",
    "spine",
    "rear claw tip",
    "fur",
    "blood",
    "lung",
    "nose",
    "stomach",
];
const BIRD_PARTS: [&str; 19] = [
    "wing",
    "eye",
    "face",
    "wing",
    "wing tip",
    "foot",
    "wing",
    "winged",
    "head",
    "leg",
    "light headed",
    "neck",
    "spine",
    "toe",
    "feathers",
    "blood",
    "lung",
    "bill",
    "stomach",
];
const HORSE_PARTS: [&str; 19] = [
    "foreleg",
    "eye",
    "face",
    "forehoof",
    "hoof tip",
    "rear hoof",
    "forehoof",
    "hooved",
    "head",
    "rear leg",
    "light headed",
    "neck",
    "backbone",
    "rear hoof tip",
    "mane",
    "blood",
    "lung",
    "nose",
    "stomach",
];
const SPHERE_PARTS: [&str; 19] = [
    "appendage",
    "optic nerve",
    "body",
    "tentacle",
    "tentacle tip",
    "lower appendage",
    "tentacle",
    "tentacled",
    "body",
    "lower tentacle",
    "rotational",
    "equator",
    "body",
    "lower tentacle tip",
    "cilia",
    "life force",
    "retina",
    "olfactory nerve",
    "interior",
];
const FUNGUS_PARTS: [&str; 19] = [
    "mycelium",
    "visual area",
    "front",
    "hypha",
    "hypha",
    "root",
    "strand",
    "stranded",
    "cap area",
    "rhizome",
    "sporulated",
    "stalk",
    "root",
    "rhizome tip",
    "spores",
    "juices",
    "gill",
    "gill",
    "interior",
];
const VORTEX_PARTS: [&str; 19] = [
    "region",
    "eye",
    "front",
    "minor current",
    "minor current",
    "lower current",
    "swirl",
    "swirled",
    "central core",
    "lower current",
    "addled",
    "center",
    "currents",
    "edge",
    "currents",
    "life force",
    "center",
    "leading edge",
    "interior",
];
const SNAKE_PARTS: [&str; 19] = [
    "vestigial limb",
    "eye",
    "face",
    "large scale",
    "large scale tip",
    "rear region",
    "scale gap",
    "scale gapped",
    "head",
    "rear region",
    "light headed",
    "neck",
    "length",
    "rear scale",
    "scales",
    "blood",
    "lung",
    "forked tongue",
    "stomach",
];
const WORM_PARTS: [&str; 19] = [
    "anterior segment",
    "light sensitive cell",
    "clitellum",
    "setae",
    "setae",
    "posterior segment",
    "segment",
    "segmented",
    "anterior segment",
    "posterior",
    "over stretched",
    "clitellum",
    "length",
    "posterior setae",
    "setae",
    "blood",
    "skin",
    "prostomium",
    "stomach",
];
const FISH_PARTS: [&str; 19] = [
    "fin",
    "eye",
    "premaxillary",
    "pelvic axillary",
    "pelvic fin",
    "anal fin",
    "pectoral fin",
    "finned",
    "head",
    "peduncle",
    "played out",
    "gills",
    "dorsal fin",
    "caudal fin",
    "scales",
    "blood",
    "gill",
    "nostril",
    "stomach",
];

/// [v2.8.0] 체형에 따른 신체 부위 이름 (원본: polyself.c:1603-1758 mbodypart)
pub fn body_part_name(body_type: BodyType, part: BodyPart) -> &'static str {
    let idx = part as usize;
    if idx >= 19 {
        return "body";
    }
    match body_type {
        BodyType::Humanoid => HUMANOID_PARTS[idx],
        BodyType::Jelly => JELLY_PARTS[idx],
        BodyType::Animal => ANIMAL_PARTS[idx],
        BodyType::Bird => BIRD_PARTS[idx],
        BodyType::Horse => HORSE_PARTS[idx],
        BodyType::Sphere => SPHERE_PARTS[idx],
        BodyType::Fungus => FUNGUS_PARTS[idx],
        BodyType::Vortex => VORTEX_PARTS[idx],
        BodyType::Snake => SNAKE_PARTS[idx],
        BodyType::Worm => WORM_PARTS[idx],
        BodyType::Fish => FISH_PARTS[idx],
    }
}

/// [v2.8.0] 심볼  체형 추론
pub fn body_type_from_symbol(symbol: char) -> BodyType {
    match symbol {
        '@' | 'h' | 'K' | 'G' | 'o' | 'O' | 'T' | 'V' | 'Z' => BodyType::Humanoid,
        'j' | 'P' | 'b' => BodyType::Jelly,
        'B' => BodyType::Bird,
        'u' | 'U' => BodyType::Horse,
        'e' => BodyType::Sphere,
        'F' => BodyType::Fungus,
        'v' | 'E' => BodyType::Vortex,
        'S' | 'N' => BodyType::Snake,
        'w' => BodyType::Worm,
        ';' => BodyType::Fish,
        _ => BodyType::Animal,
    }
}

/// [v2.8.0] 특수 몬스터 신체 부위 보정 (원본: polyself.c:1686-1718)
pub fn special_body_part(monster_name: &str, part: BodyPart) -> Option<&'static str> {
    let l = monster_name.to_lowercase();
    let pi = part as usize;

    if l.contains("dog")
        || l.contains("cat")
        || l.contains("kitten")
        || l.contains("puppy")
        || l.contains("rat")
    {
        match part {
            BodyPart::Hand => return Some("paw"),
            BodyPart::Handed => return Some("pawed"),
            BodyPart::Foot => return Some("rear paw"),
            BodyPart::Arm | BodyPart::Leg => return Some(HORSE_PARTS[pi]),
            _ => {}
        }
    }
    if (l.contains("mumak") || l.contains("mastodon")) && part == BodyPart::Nose {
        return Some("trunk");
    }
    if l.contains("shark") && part == BodyPart::Hair {
        return Some("skin");
    }
    if (l.contains("jellyfish") || l.contains("kraken"))
        && matches!(
            part,
            BodyPart::Arm | BodyPart::Finger | BodyPart::Hand | BodyPart::Foot | BodyPart::Toe
        )
    {
        return Some("tentacle");
    }
    if l.contains("floating eye") && part == BodyPart::Eye {
        return Some("cornea");
    }
    if l.contains("light") || l.contains("will-o") {
        return match part {
            BodyPart::Handed => Some("rayed"),
            BodyPart::Arm | BodyPart::Finger | BodyPart::Fingertip | BodyPart::Hand => Some("ray"),
            _ => Some("beam"),
        };
    }
    None
}

/// [v2.8.0] 데미지 타입 (원본: AD_xxx)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Fire,
    Cold,
    Electricity,
    Acid,
    Poison,
    Magic,
    Disintegrate,
}

/// [v2.8.0] 골렘 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GolemType {
    Flesh,
    Iron,
    Stone,
    Clay,
    Wood,
    Glass,
    Gold,
    Paper,
    None,
}

/// [v2.8.0] 골렘 효과 계산 (원본: polyself.c:1778-1808 ugolemeffects)
pub fn golem_effect(golem: GolemType, dt: DamageType, dmg: i32) -> Option<(i32, &'static str)> {
    match (golem, dt) {
        (GolemType::Flesh, DamageType::Electricity) => {
            let h = (dmg + 5) / 6;
            if h > 0 {
                Some((h, "Strangely, you feel better than before."))
            } else {
                None
            }
        }
        (GolemType::Iron, DamageType::Fire) => {
            if dmg > 0 {
                Some((dmg, "Strangely, you feel better than before."))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// [v2.8.0] 장갑  드래곤 매핑 (원본: polyself.c:1810-1850 armor_to_dragon)
pub fn armor_to_dragon(armor_name: &str) -> Option<&'static str> {
    let l = armor_name.to_lowercase();
    if l.contains("gray dragon") {
        Some("gray dragon")
    } else if l.contains("silver dragon") {
        Some("silver dragon")
    } else if l.contains("red dragon") {
        Some("red dragon")
    } else if l.contains("orange dragon") {
        Some("orange dragon")
    } else if l.contains("white dragon") {
        Some("white dragon")
    } else if l.contains("black dragon") {
        Some("black dragon")
    } else if l.contains("blue dragon") {
        Some("blue dragon")
    } else if l.contains("green dragon") {
        Some("green dragon")
    } else if l.contains("yellow dragon") {
        Some("yellow dragon")
    } else {
        None
    }
}

/// [v2.8.0] 변신 성별 (원본: polyself.c:1767-1776 poly_gender)
pub fn poly_gender(form_name: &str, is_humanoid: bool, original_female: bool) -> i32 {
    let l = form_name.to_lowercase();
    let neuter = l.contains("elemental")
        || l.contains("golem")
        || l.contains("vortex")
        || l.contains("jelly")
        || l.contains("ooze")
        || l.contains("light")
        || l.contains("blob")
        || l.contains("fungus");
    if neuter || !is_humanoid {
        2
    } else if original_female {
        1
    } else {
        0
    }
}

/// [v2.8.0] 변신 인식 결과
#[derive(Debug, Clone)]
pub enum PolyWarning {
    Species(&'static str),
    Races(Vec<&'static str>),
}

/// [v2.8.0] 변신 인식 (원본: polyself.c:1852-1878 polysense)
pub fn polysense(form_name: &str) -> Option<PolyWarning> {
    match form_name.to_lowercase().as_str() {
        "purple worm" => Some(PolyWarning::Species("shrieker")),
        "vampire" | "vampire lord" => Some(PolyWarning::Races(vec!["human", "elf"])),
        _ => None,
    }
}

/// [v2.8.0] 종족 학살 여부 (원본: polyself.c:1880-1890 ugenocided)
pub fn is_role_genocided(role_monsters: &[&str], genocided_list: &[String]) -> bool {
    role_monsters
        .iter()
        .any(|rm| genocided_list.iter().any(|g| g.eq_ignore_ascii_case(rm)))
}

/// [v2.8.0] 사망 시 내면 묘사 (원본: polyself.c:1892-1905 udeadinside)
pub fn dead_inside_feeling(form_name: &str) -> &'static str {
    let l = form_name.to_lowercase();
    let nonliving = l.contains("golem")
        || l.contains("zombie")
        || l.contains("skeleton")
        || l.contains("mummy")
        || l.contains("vampire")
        || l.contains("wraith")
        || l.contains("ghost")
        || l.contains("lich")
        || l.contains("vortex")
        || l.contains("elemental");
    if !nonliving {
        "dead"
    } else if l.contains("golem") || l.contains("vortex") || l.contains("elemental") {
        "empty"
    } else {
        "condemned"
    }
}

/// [v2.8.0] 인간화 HP 복구 (원본: polyself.c:1065-1100 rehumanize)
pub fn rehumanize_hp(original_max_hp: i32, poly_hp: i32, poly_max_hp: i32) -> i32 {
    if poly_max_hp > 0 {
        let r = poly_hp as f64 / poly_max_hp as f64;
        ((original_max_hp as f64 * r) as i32)
            .max(1)
            .min(original_max_hp)
    } else {
        original_max_hp.max(1)
    }
}

/// [v2.8.0] 뱀파이어 변신 가능 대상 (원본: polyself.c:1525-1538 dopoly)
pub fn vampire_shift_forms() -> Vec<&'static str> {
    vec!["vampire bat", "fog cloud", "wolf"]
}

/// [v2.8.0] 숨기 가능 여부 (원본: polyself.c:1455-1523 dohide)
pub fn can_hide(
    form: &str,
    stuck: bool,
    trapped: bool,
    fly: bool,
    ceil: bool,
    obj_below: bool,
    stairs: bool,
) -> Result<&'static str, &'static str> {
    if stuck {
        return Err("You can't hide while being held.");
    }
    let l = form.to_lowercase();
    let ceil_hider = fly || l.contains("piercer") || l.contains("lurker");
    if trapped && (ceil_hider || !l.contains("trapper")) {
        return Err("You can't hide while trapped.");
    }
    if l.contains("eel") {
        return Err("There is no water to hide in here.");
    }
    if (l.contains("trapper") || l.contains("mimic")) && !obj_below {
        return Err("There is nothing to hide under here.");
    }
    if ceil_hider && !ceil {
        return Err("There is nowhere to hide above you.");
    }
    if stairs {
        return Ok("Your camouflage fails to impede access to the stairs.");
    }
    Ok("You are now hiding.")
}

/// [v2.8.0] 마인드 블라스트 데미지 (원본: polyself.c:1540-1575 domindblast)
pub fn mindblast_damage(rng: &mut NetHackRng) -> i32 {
    rng.rn1(15, 1)
}

/// [v2.8.0] 에너지 비용 상수
pub const BREATH_ENERGY_COST: i32 = 15;
pub const SUMMON_ENERGY_COST: i32 = 10;
pub const MINDBLAST_ENERGY_COST: i32 = 10;
pub const MINDBLAST_RANGE_SQ: i32 = 64;

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_part_humanoid() {
        assert_eq!(body_part_name(BodyType::Humanoid, BodyPart::Hand), "hand");
        assert_eq!(body_part_name(BodyType::Humanoid, BodyPart::Eye), "eye");
        assert_eq!(
            body_part_name(BodyType::Humanoid, BodyPart::Stomach),
            "stomach"
        );
    }
    #[test]
    fn test_body_part_jelly() {
        assert_eq!(body_part_name(BodyType::Jelly, BodyPart::Hand), "grasp");
        assert_eq!(body_part_name(BodyType::Jelly, BodyPart::Arm), "pseudopod");
    }
    #[test]
    fn test_body_part_bird() {
        assert_eq!(body_part_name(BodyType::Bird, BodyPart::Hand), "wing");
        assert_eq!(body_part_name(BodyType::Bird, BodyPart::Nose), "bill");
    }
    #[test]
    fn test_body_type_from_symbol() {
        assert_eq!(body_type_from_symbol('@'), BodyType::Humanoid);
        assert_eq!(body_type_from_symbol('B'), BodyType::Bird);
        assert_eq!(body_type_from_symbol('F'), BodyType::Fungus);
    }
    #[test]
    fn test_special_body_part_dog() {
        assert_eq!(special_body_part("large dog", BodyPart::Hand), Some("paw"));
        assert_eq!(special_body_part("kitten", BodyPart::Handed), Some("pawed"));
    }
    #[test]
    fn test_special_body_part_elephant() {
        assert_eq!(special_body_part("mumak", BodyPart::Nose), Some("trunk"));
    }
    #[test]
    fn test_golem_effect_flesh() {
        let r = golem_effect(GolemType::Flesh, DamageType::Electricity, 12);
        assert!(r.is_some());
        assert_eq!(r.unwrap().0, 2);
    }
    #[test]
    fn test_golem_effect_iron() {
        let r = golem_effect(GolemType::Iron, DamageType::Fire, 10);
        assert_eq!(r.unwrap().0, 10);
    }
    #[test]
    fn test_golem_none() {
        assert!(golem_effect(GolemType::Stone, DamageType::Fire, 10).is_none());
    }
    #[test]
    fn test_armor_to_dragon() {
        assert_eq!(
            armor_to_dragon("gray dragon scale mail"),
            Some("gray dragon")
        );
        assert_eq!(armor_to_dragon("leather armor"), None);
    }
    #[test]
    fn test_poly_gender() {
        assert_eq!(poly_gender("orc", true, false), 0);
        assert_eq!(poly_gender("fire elemental", false, false), 2);
    }
    #[test]
    fn test_polysense() {
        assert!(polysense("purple worm").is_some());
        assert!(polysense("vampire").is_some());
        assert!(polysense("orc").is_none());
    }
    #[test]
    fn test_dead_inside() {
        assert_eq!(dead_inside_feeling("human"), "dead");
        assert_eq!(dead_inside_feeling("iron golem"), "empty");
        assert_eq!(dead_inside_feeling("vampire"), "condemned");
    }
    #[test]
    fn test_rehumanize_hp() {
        assert_eq!(rehumanize_hp(100, 25, 50), 50);
        assert_eq!(rehumanize_hp(100, 50, 50), 100);
    }
    #[test]
    fn test_vampire_forms() {
        let f = vampire_shift_forms();
        assert!(f.contains(&"vampire bat"));
        assert!(f.contains(&"wolf"));
    }
    #[test]
    fn test_can_hide() {
        assert!(can_hide("trapper", true, false, false, true, true, false).is_err());
        assert!(can_hide("piercer", false, false, false, true, false, false).is_ok());
    }
}

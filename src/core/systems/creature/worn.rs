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

// =============================================================================
//
// =============================================================================
///
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WornSlot {
    ///
    Armor,
    ///
    Cloak,
    ///
    Helmet,
    ///
    Shield,
    ///
    Gloves,
    ///
    Boots,
    ///
    Shirt,
    ///
    RingLeft,
    ///
    RingRight,
    ///
    Amulet,
    ///
    Tool,
    ///
    Saddle,
}

impl WornSlot {
    ///
    pub const fn to_bit(self) -> u32 {
        match self {
            Self::Armor => 0x00000001,
            Self::Cloak => 0x00000002,
            Self::Helmet => 0x00000004,
            Self::Shield => 0x00000008,
            Self::Gloves => 0x00000010,
            Self::Boots => 0x00000020,
            Self::Shirt => 0x00000040,
            Self::RingLeft => 0x00000080,
            Self::RingRight => 0x00000100,
            Self::Amulet => 0x00000200,
            Self::Tool => 0x00000400,
            Self::Saddle => 0x00000800,
        }
    }

    ///
    pub fn from_bit(bit: u32) -> Option<Self> {
        match bit {
            0x00000001 => Some(Self::Armor),
            0x00000002 => Some(Self::Cloak),
            0x00000004 => Some(Self::Helmet),
            0x00000008 => Some(Self::Shield),
            0x00000010 => Some(Self::Gloves),
            0x00000020 => Some(Self::Boots),
            0x00000040 => Some(Self::Shirt),
            0x00000080 => Some(Self::RingLeft),
            0x00000100 => Some(Self::RingRight),
            0x00000200 => Some(Self::Amulet),
            0x00000400 => Some(Self::Tool),
            0x00000800 => Some(Self::Saddle),
            _ => None,
        }
    }

    ///
    pub fn all() -> &'static [WornSlot] {
        &[
            Self::Armor,
            Self::Cloak,
            Self::Helmet,
            Self::Shield,
            Self::Gloves,
            Self::Boots,
            Self::Shirt,
            Self::RingLeft,
            Self::RingRight,
            Self::Amulet,
            Self::Tool,
            Self::Saddle,
        ]
    }

    ///
    pub fn is_armor(self) -> bool {
        matches!(
            self,
            Self::Armor
                | Self::Cloak
                | Self::Helmet
                | Self::Shield
                | Self::Gloves
                | Self::Boots
                | Self::Shirt
        )
    }

    ///
    pub fn is_accessory(self) -> bool {
        matches!(self, Self::RingLeft | Self::RingRight | Self::Amulet)
    }

    ///
    pub fn is_ring(self) -> bool {
        matches!(self, Self::RingLeft | Self::RingRight)
    }
}

///
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WornSlots(pub u32);

impl WornSlots {
    //
    pub const ARM: u32 = 0x00000001;
    pub const ARMC: u32 = 0x00000002; // 留앺넗
    pub const ARMH: u32 = 0x00000004;
    pub const ARMS: u32 = 0x00000008;
    pub const ARMG: u32 = 0x00000010;
    pub const ARMF: u32 = 0x00000020;
    pub const ARMU: u32 = 0x00000040;
    pub const RINGL: u32 = 0x00000080;
    pub const RINGR: u32 = 0x00000100;
    pub const AMUL: u32 = 0x00000200;
    pub const TOOL: u32 = 0x00000400;
    pub const SADDLE: u32 = 0x00000800;

    ///
    pub const ALL_ARMOR: u32 =
        Self::ARM | Self::ARMC | Self::ARMH | Self::ARMS | Self::ARMG | Self::ARMF | Self::ARMU;

    ///
    pub const ALL_ACCESSORY: u32 = Self::RINGL | Self::RINGR | Self::AMUL;

    //
    pub fn contains(&self, flag: u32) -> bool {
        self.0 & flag != 0
    }

    pub fn set(&mut self, flag: u32) {
        self.0 |= flag;
    }

    pub fn clear(&mut self, flag: u32) {
        self.0 &= !flag;
    }

    //

    ///
    pub fn has(&self, slot: WornSlot) -> bool {
        self.0 & slot.to_bit() != 0
    }

    ///
    pub fn wear(&mut self, slot: WornSlot) {
        self.0 |= slot.to_bit();
    }

    ///
    pub fn unwear(&mut self, slot: WornSlot) {
        self.0 &= !slot.to_bit();
    }

    ///
    pub fn worn_slots(&self) -> Vec<WornSlot> {
        WornSlot::all()
            .iter()
            .copied()
            .filter(|s| self.has(*s))
            .collect()
    }

    ///
    pub fn empty() -> Self {
        Self(0)
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct EquipmentEffects {
    pub fire_res: bool,
    pub cold_res: bool,
    pub shock_res: bool,
    pub poison_res: bool,
    pub acid_res: bool,
    pub sleep_res: bool,
    pub ac_bonus: i32,
    pub speed_bonus: i32,
    pub stealth: bool,
    pub see_invisible: bool,
    pub telepathy: bool,
    pub warning: bool,
    pub searching: bool,
    pub infravision: bool,
    pub flying: bool,
    pub levitation: bool,
    pub swimming: bool,
    pub magical_breathing: bool,
    pub displacement: bool,
    pub reflection: bool,
    pub free_action: bool,
    pub slow_digestion: bool,
    pub half_physical_damage: bool,
    pub half_spell_damage: bool,
    pub regeneration: bool,
    pub energy_regeneration: bool,
    pub protection: i32,
    pub fumbling: bool,
    pub hunger: bool,
    pub aggravate_monster: bool,
    pub conflict: bool,
    pub polymorphing: bool,
    pub polymorph_control: bool,
    pub hallucination: bool,
    pub str_bonus: i32,
    pub dex_bonus: i32,
    pub con_bonus: i32,
    pub int_bonus: i32,
    pub wis_bonus: i32,
    pub cha_bonus: i32,
}

impl EquipmentEffects {
    pub fn new() -> Self {
        Self::default()
    }

    ///
    pub fn combine(&mut self, other: &EquipmentEffects) {
        self.fire_res |= other.fire_res;
        self.cold_res |= other.cold_res;
        self.shock_res |= other.shock_res;
        self.poison_res |= other.poison_res;
        self.acid_res |= other.acid_res;
        self.sleep_res |= other.sleep_res;
        self.ac_bonus += other.ac_bonus;
        self.speed_bonus += other.speed_bonus;
        self.stealth |= other.stealth;
        self.see_invisible |= other.see_invisible;
        self.telepathy |= other.telepathy;
        self.warning |= other.warning;
        self.searching |= other.searching;
        self.infravision |= other.infravision;
        self.flying |= other.flying;
        self.levitation |= other.levitation;
        self.swimming |= other.swimming;
        self.magical_breathing |= other.magical_breathing;
        self.displacement |= other.displacement;
        self.reflection |= other.reflection;
        self.free_action |= other.free_action;
        self.slow_digestion |= other.slow_digestion;
        self.half_physical_damage |= other.half_physical_damage;
        self.half_spell_damage |= other.half_spell_damage;
        self.regeneration |= other.regeneration;
        self.energy_regeneration |= other.energy_regeneration;
        self.protection += other.protection;
        self.fumbling |= other.fumbling;
        self.hunger |= other.hunger;
        self.aggravate_monster |= other.aggravate_monster;
        self.conflict |= other.conflict;
        self.polymorphing |= other.polymorphing;
        self.polymorph_control |= other.polymorph_control;
        self.hallucination |= other.hallucination;
        self.str_bonus += other.str_bonus;
        self.dex_bonus += other.dex_bonus;
        self.con_bonus += other.con_bonus;
        self.int_bonus += other.int_bonus;
        self.wis_bonus += other.wis_bonus;
        self.cha_bonus += other.cha_bonus;
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn armor_effects(name: &str, spe: i32, _blessed: bool, cursed: bool) -> EquipmentEffects {
    let mut fx = EquipmentEffects::new();
    let base_ac = base_armor_ac(name);
    fx.ac_bonus = base_ac + spe;

    //
    match name {
        //
        "gray dragon scale mail" | "gray dragon scales" => {
            fx.reflection = true;
            fx.ac_bonus = 9 + spe;
        }
        "silver dragon scale mail" | "silver dragon scales" => {
            fx.reflection = true;
            fx.ac_bonus = 9 + spe;
        }
        "red dragon scale mail" | "red dragon scales" => {
            fx.fire_res = true;
            fx.ac_bonus = 9 + spe;
        }
        "white dragon scale mail" | "white dragon scales" => {
            fx.cold_res = true;
            fx.ac_bonus = 9 + spe;
        }
        "blue dragon scale mail" | "blue dragon scales" => {
            fx.shock_res = true;
            fx.ac_bonus = 9 + spe;
        }
        "green dragon scale mail" | "green dragon scales" => {
            fx.poison_res = true;
            fx.ac_bonus = 9 + spe;
        }
        "yellow dragon scale mail" | "yellow dragon scales" => {
            fx.acid_res = true;
            fx.ac_bonus = 9 + spe;
        }
        "black dragon scale mail" | "black dragon scales" => {
            fx.ac_bonus = 9 + spe;
        }
        "orange dragon scale mail" | "orange dragon scales" => {
            fx.sleep_res = true;
            fx.ac_bonus = 9 + spe;
        }

        // --- 留앺넗 ---
        "cloak of protection" => {
            fx.protection = 3 + spe;
        }
        "cloak of magic resistance" => {
            fx.half_spell_damage = true;
        }
        "cloak of displacement" => {
            fx.displacement = true;
        }
        "elven cloak" => {
            fx.stealth = true;
        }

        //
        "helm of brilliance" => {
            fx.int_bonus = spe;
            fx.wis_bonus = spe;
        }
        "helm of telepathy" => {
            fx.telepathy = true;
        }
        "cornuthaum" => {
            fx.cha_bonus = 1;
        }

        //
        "gauntlets of power" => {
            fx.str_bonus = 25;
        }
        "gauntlets of dexterity" => {
            fx.dex_bonus = spe;
        }
        "gauntlets of fumbling" => {
            fx.fumbling = true;
        }

        //
        "speed boots" | "boots of speed" => {
            fx.speed_bonus = 8;
        }
        "elven boots" => {
            fx.stealth = true;
        }
        "levitation boots" | "boots of levitation" => {
            fx.levitation = true;
        }
        "fumble boots" | "boots of fumbling" => {
            fx.fumbling = true;
        }
        "water walking boots" | "boots of water walking" => {
            fx.swimming = true;
        }

        //
        "shield of reflection" => {
            fx.reflection = true;
        }

        _ => {}
    }

    if cursed {
        fx.fumbling |= name.contains("boot") || name.contains("gauntlet");
        fx.hunger = true;
    }

    fx
}

///
pub fn ring_effects(name: &str, spe: i32, _blessed: bool, cursed: bool) -> EquipmentEffects {
    let mut fx = EquipmentEffects::new();

    match name {
        "ring of fire resistance" => {
            fx.fire_res = true;
        }
        "ring of cold resistance" => {
            fx.cold_res = true;
        }
        "ring of shock resistance" => {
            fx.shock_res = true;
        }
        "ring of poison resistance" => {
            fx.poison_res = true;
        }
        "ring of free action" => {
            fx.free_action = true;
        }
        "ring of see invisible" => {
            fx.see_invisible = true;
        }
        "ring of levitation" => {
            fx.levitation = true;
        }
        "ring of regeneration" => {
            fx.regeneration = true;
        }
        "ring of searching" => {
            fx.searching = true;
        }
        "ring of stealth" => {
            fx.stealth = true;
        }
        "ring of warning" => {
            fx.warning = true;
        }
        "ring of conflict" => {
            fx.conflict = true;
        }
        "ring of hunger" => {
            fx.hunger = true;
        }
        "ring of aggravate monster" => {
            fx.aggravate_monster = true;
        }
        "ring of slow digestion" => {
            fx.slow_digestion = true;
        }
        "ring of protection" => {
            fx.protection = spe;
        }
        "ring of adornment" => {
            fx.cha_bonus = spe;
        }
        "ring of gain strength" => {
            fx.str_bonus = spe;
        }
        "ring of gain constitution" => {
            fx.con_bonus = spe;
        }
        "ring of polymorphing" => {
            fx.polymorphing = true;
        }
        "ring of polymorph control" => {
            fx.polymorph_control = true;
        }
        _ => {}
    }

    //
    if cursed && spe > 0 {
        fx.str_bonus = -fx.str_bonus;
        fx.dex_bonus = -fx.dex_bonus;
        fx.con_bonus = -fx.con_bonus;
        fx.cha_bonus = -fx.cha_bonus;
        fx.protection = -fx.protection;
    }

    fx
}

///
pub fn amulet_effects(name: &str, _blessed: bool, _cursed: bool) -> EquipmentEffects {
    let mut fx = EquipmentEffects::new();

    match name {
        "amulet of reflection" => {
            fx.reflection = true;
        }
        "amulet of ESP" => {
            fx.telepathy = true;
        }
        "amulet versus poison" => {
            fx.poison_res = true;
        }
        "amulet of magical breathing" => {
            fx.magical_breathing = true;
        }
        _ => {}
    }

    fx
}

// =============================================================================
//
// =============================================================================

///
pub fn base_armor_ac(name: &str) -> i32 {
    match name {
        "plate mail" => 7,
        "crystal plate mail" => 7,
        "splint mail" => 6,
        "banded mail" => 6,
        "bronze plate mail" => 5,
        "chain mail" => 5,
        "scale mail" => 4,
        "ring mail" => 3,
        "studded leather armor" => 3,
        "leather armor" => 2,
        "leather jacket" => 1,
        "elven mithril-coat" => 5,
        "dwarvish mithril-coat" => 6,
        "orcish chain mail" => 4,
        n if n.contains("dragon scale mail") => 9,
        n if n.contains("dragon scales") => 3,
        n if n.contains("cloak") => 1,
        "mummy wrapping" => 0,
        n if n.contains("helm") => 1,
        "dwarvish iron helm" => 2,
        n if n.contains("gauntlets") => 1,
        "leather gloves" => 1,
        n if n.contains("boots") => 1,
        "high boots" => 2,
        "small shield" => 1,
        "large shield" => 2,
        "elven shield" => 2,
        "dwarvish roundshield" => 2,
        "shield of reflection" => 2,
        _ => 0,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn wearing_conflicts_enum(slot: WornSlot, currently_worn: &WornSlots) -> Vec<WornSlot> {
    let mut conflicts = Vec::new();

    match slot {
        WornSlot::Armor => {
            //
            if currently_worn.has(WornSlot::Cloak) {
                conflicts.push(WornSlot::Cloak);
            }
        }
        WornSlot::Shirt => {
            //
            if currently_worn.has(WornSlot::Armor) {
                conflicts.push(WornSlot::Armor);
            }
            if currently_worn.has(WornSlot::Cloak) {
                conflicts.push(WornSlot::Cloak);
            }
        }
        _ => {}
    }

    conflicts
}

///
pub fn removal_conflicts_enum(slot: WornSlot, currently_worn: &WornSlots) -> Vec<WornSlot> {
    let mut conflicts = Vec::new();

    match slot {
        WornSlot::Armor => {
            //
            if currently_worn.has(WornSlot::Cloak) {
                conflicts.push(WornSlot::Cloak);
            }
        }
        WornSlot::Shirt => {
            //
            if currently_worn.has(WornSlot::Armor) {
                conflicts.push(WornSlot::Armor);
            }
            if currently_worn.has(WornSlot::Cloak) {
                conflicts.push(WornSlot::Cloak);
            }
        }
        _ => {}
    }

    conflicts
}

///
pub fn wearing_conflicts(slot: u32, currently_worn: &WornSlots) -> Vec<u32> {
    if let Some(s) = WornSlot::from_bit(slot) {
        wearing_conflicts_enum(s, currently_worn)
            .into_iter()
            .map(|s| s.to_bit())
            .collect()
    } else {
        Vec::new()
    }
}

///
pub fn removal_conflicts(slot: u32, currently_worn: &WornSlots) -> Vec<u32> {
    if let Some(s) = WornSlot::from_bit(slot) {
        removal_conflicts_enum(s, currently_worn)
            .into_iter()
            .map(|s| s.to_bit())
            .collect()
    } else {
        Vec::new()
    }
}

///
pub fn is_stuck(cursed: bool) -> bool {
    cursed
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
///
pub fn update_mon_intrinsics(
    current_effects: &mut EquipmentEffects,
    item_name: &str,
    item_spe: i32,
    item_blessed: bool,
    item_cursed: bool,
    slot: WornSlot,
    wearing: bool,
) {
    let item_fx = match slot {
        s if s.is_armor() => armor_effects(item_name, item_spe, item_blessed, item_cursed),
        s if s.is_ring() => ring_effects(item_name, item_spe, item_blessed, item_cursed),
        WornSlot::Amulet => amulet_effects(item_name, item_blessed, item_cursed),
        _ => EquipmentEffects::new(),
    };

    if wearing {
        //
        current_effects.combine(&item_fx);
    } else {
        //
        //
        current_effects.ac_bonus -= item_fx.ac_bonus;
        current_effects.speed_bonus -= item_fx.speed_bonus;
        current_effects.protection -= item_fx.protection;
        current_effects.str_bonus -= item_fx.str_bonus;
        current_effects.dex_bonus -= item_fx.dex_bonus;
        current_effects.con_bonus -= item_fx.con_bonus;
        current_effects.int_bonus -= item_fx.int_bonus;
        current_effects.wis_bonus -= item_fx.wis_bonus;
        current_effects.cha_bonus -= item_fx.cha_bonus;
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentEvent {
    ///
    Wear { slot: WornSlot, item_name: String },
    ///
    Remove { slot: WornSlot, item_name: String },
    ///
    CursedWear { slot: WornSlot, item_name: String },
    ///
    Destroyed {
        slot: WornSlot,
        item_name: String,
        cause: String,
    },
    ///
    Transformed {
        slot: WornSlot,
        old_name: String,
        new_name: String,
    },
}

///
pub fn equipment_event_message(event: &EquipmentEvent) -> String {
    match event {
        EquipmentEvent::Wear { slot, item_name } => {
            format!("You put on {} ({:?}).", item_name, slot)
        }
        EquipmentEvent::Remove { slot, item_name } => {
            format!("You take off {} ({:?}).", item_name, slot)
        }
        EquipmentEvent::CursedWear { slot, item_name } => {
            format!("The {} welds itself to your body! ({:?})", item_name, slot)
        }
        EquipmentEvent::Destroyed {
            slot,
            item_name,
            cause,
        } => {
            format!("Your {} is destroyed by {}! ({:?})", item_name, cause, slot)
        }
        EquipmentEvent::Transformed {
            old_name, new_name, ..
        } => {
            format!("Your {} transforms into {}!", old_name, new_name)
        }
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn slot_display_name(slot: WornSlot) -> &'static str {
    match slot {
        WornSlot::Armor => "body armor",
        WornSlot::Cloak => "cloak",
        WornSlot::Helmet => "helmet",
        WornSlot::Shield => "shield",
        WornSlot::Gloves => "gloves",
        WornSlot::Boots => "boots",
        WornSlot::Shirt => "shirt",
        WornSlot::RingLeft => "left ring finger",
        WornSlot::RingRight => "right ring finger",
        WornSlot::Amulet => "neck",
        WornSlot::Tool => "face",
        WornSlot::Saddle => "saddle",
    }
}

///
pub fn wear_verb(slot: WornSlot) -> &'static str {
    match slot {
        WornSlot::Armor | WornSlot::Cloak | WornSlot::Shirt => "wear",
        WornSlot::Helmet | WornSlot::Shield | WornSlot::Gloves | WornSlot::Boots => "put on",
        WornSlot::RingLeft | WornSlot::RingRight => "slip on",
        WornSlot::Amulet => "put on",
        WornSlot::Tool => "put on",
        WornSlot::Saddle => "place",
    }
}

///
pub fn remove_verb(slot: WornSlot) -> &'static str {
    match slot {
        WornSlot::Armor | WornSlot::Cloak | WornSlot::Shirt => "take off",
        WornSlot::Helmet | WornSlot::Shield | WornSlot::Gloves | WornSlot::Boots => "take off",
        WornSlot::RingLeft | WornSlot::RingRight => "remove",
        WornSlot::Amulet => "remove",
        WornSlot::Tool => "take off",
        WornSlot::Saddle => "remove",
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct WornItem {
    pub name: String,
    pub slot: WornSlot,
    pub spe: i32,
    pub blessed: bool,
    pub cursed: bool,
}

///
pub fn calculate_total_effects(worn_items: &[WornItem]) -> EquipmentEffects {
    let mut total = EquipmentEffects::new();

    for item in worn_items {
        let fx = match item.slot {
            s if s.is_armor() => armor_effects(&item.name, item.spe, item.blessed, item.cursed),
            s if s.is_ring() => ring_effects(&item.name, item.spe, item.blessed, item.cursed),
            WornSlot::Amulet => amulet_effects(&item.name, item.blessed, item.cursed),
            _ => EquipmentEffects::new(),
        };
        total.combine(&fx);
    }

    total
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn equip_delay(slot: WornSlot) -> i32 {
    match slot {
        WornSlot::Armor => 5,
        WornSlot::Cloak => 2,
        WornSlot::Helmet => 1,
        WornSlot::Shield => 1,
        WornSlot::Gloves => 1,
        WornSlot::Boots => 1,
        WornSlot::Shirt => 5,
        WornSlot::RingLeft | WornSlot::RingRight => 1,
        WornSlot::Amulet => 1,
        WornSlot::Tool => 1,
        WornSlot::Saddle => 3,
    }
}

///
pub fn unequip_delay(slot: WornSlot, cursed: bool) -> Option<i32> {
    if cursed {
        None
    } else {
        Some(equip_delay(slot))
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucStatus {
    Blessed,
    Uncursed,
    Cursed,
    Unknown,
}

///
///
pub fn wearing_reveals_buc(cursed: bool, bknown: bool) -> (BucStatus, bool) {
    if bknown {
        //
        let status = if cursed {
            BucStatus::Cursed
        } else {
            BucStatus::Uncursed
        };
        (status, false)
    } else if cursed {
        //
        (BucStatus::Cursed, true)
    } else {
        (BucStatus::Unknown, false)
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn default_armor_weight(slot: WornSlot) -> i32 {
    match slot {
        WornSlot::Armor => 300,
        WornSlot::Cloak => 10,
        WornSlot::Helmet => 30,
        WornSlot::Shield => 100,
        WornSlot::Gloves => 10,
        WornSlot::Boots => 20,
        WornSlot::Shirt => 5,
        WornSlot::RingLeft | WornSlot::RingRight => 3,
        WornSlot::Amulet => 20,
        WornSlot::Tool => 3,
        WornSlot::Saddle => 200,
    }
}

///
pub fn total_worn_weight(items: &[WornItem]) -> i32 {
    //
    items.iter().map(|i| default_armor_weight(i.slot)).sum()
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dragon_scale_effects() {
        let fx = armor_effects("red dragon scale mail", 0, false, false);
        assert!(fx.fire_res);
        assert_eq!(fx.ac_bonus, 9);
    }

    #[test]
    fn test_ring_effects() {
        let fx = ring_effects("ring of regeneration", 0, false, false);
        assert!(fx.regeneration);

        let fx2 = ring_effects("ring of fire resistance", 0, false, false);
        assert!(fx2.fire_res);
    }

    //
    #[test]
    fn test_worn_slots_legacy() {
        let mut slots = WornSlots(0);
        slots.set(WornSlots::ARM);
        slots.set(WornSlots::ARMC);
        assert!(slots.contains(WornSlots::ARM));
        assert!(slots.contains(WornSlots::ARMC));
        assert!(!slots.contains(WornSlots::ARMH));
        slots.clear(WornSlots::ARM);
        assert!(!slots.contains(WornSlots::ARM));
    }

    #[test]
    fn test_wearing_conflicts_legacy() {
        let worn = WornSlots(WornSlots::ARMC);
        let conflicts = wearing_conflicts(WornSlots::ARM, &worn);
        assert!(conflicts.contains(&WornSlots::ARMC));
    }

    //
    #[test]
    fn test_worn_slot_enum() {
        let mut slots = WornSlots::empty();
        slots.wear(WornSlot::Armor);
        slots.wear(WornSlot::Cloak);
        assert!(slots.has(WornSlot::Armor));
        assert!(slots.has(WornSlot::Cloak));
        assert!(!slots.has(WornSlot::Helmet));
        slots.unwear(WornSlot::Armor);
        assert!(!slots.has(WornSlot::Armor));
    }

    #[test]
    fn test_worn_slot_roundtrip() {
        for &slot in WornSlot::all() {
            let bit = slot.to_bit();
            assert_eq!(WornSlot::from_bit(bit), Some(slot));
        }
    }

    #[test]
    fn test_worn_slots_list() {
        let mut slots = WornSlots::empty();
        slots.wear(WornSlot::Helmet);
        slots.wear(WornSlot::Boots);
        let worn = slots.worn_slots();
        assert_eq!(worn.len(), 2);
        assert!(worn.contains(&WornSlot::Helmet));
        assert!(worn.contains(&WornSlot::Boots));
    }

    #[test]
    fn test_wearing_conflicts_enum() {
        let mut worn = WornSlots::empty();
        worn.wear(WornSlot::Cloak);
        let conflicts = wearing_conflicts_enum(WornSlot::Armor, &worn);
        assert!(conflicts.contains(&WornSlot::Cloak));
    }

    #[test]
    fn test_removal_conflicts_enum() {
        let mut worn = WornSlots::empty();
        worn.wear(WornSlot::Armor);
        worn.wear(WornSlot::Cloak);
        let conflicts = removal_conflicts_enum(WornSlot::Shirt, &worn);
        assert!(conflicts.contains(&WornSlot::Armor));
        assert!(conflicts.contains(&WornSlot::Cloak));
    }

    #[test]
    fn test_worn_slot_categories() {
        assert!(WornSlot::Armor.is_armor());
        assert!(WornSlot::Boots.is_armor());
        assert!(!WornSlot::RingLeft.is_armor());
        assert!(WornSlot::RingLeft.is_accessory());
        assert!(WornSlot::Amulet.is_accessory());
        assert!(!WornSlot::Armor.is_accessory());
        assert!(WornSlot::RingLeft.is_ring());
        assert!(WornSlot::RingRight.is_ring());
        assert!(!WornSlot::Amulet.is_ring());
    }

    //
    #[test]
    fn test_slot_display_names() {
        assert_eq!(slot_display_name(WornSlot::Armor), "body armor");
        assert_eq!(slot_display_name(WornSlot::RingLeft), "left ring finger");
    }

    #[test]
    fn test_equip_delay() {
        assert_eq!(equip_delay(WornSlot::Armor), 5);
        assert_eq!(equip_delay(WornSlot::RingLeft), 1);
        assert!(unequip_delay(WornSlot::Armor, true).is_none());
        assert_eq!(unequip_delay(WornSlot::Armor, false), Some(5));
    }

    #[test]
    fn test_wearing_reveals_buc() {
        let (status, new_info) = wearing_reveals_buc(true, false);
        assert_eq!(status, BucStatus::Cursed);
        assert!(new_info);

        let (status2, new_info2) = wearing_reveals_buc(false, false);
        assert_eq!(status2, BucStatus::Unknown);
        assert!(!new_info2);
    }

    #[test]
    fn test_calculate_total_effects() {
        let items = vec![
            WornItem {
                name: "red dragon scale mail".to_string(),
                slot: WornSlot::Armor,
                spe: 2,
                blessed: true,
                cursed: false,
            },
            WornItem {
                name: "ring of regeneration".to_string(),
                slot: WornSlot::RingLeft,
                spe: 0,
                blessed: false,
                cursed: false,
            },
        ];
        let fx = calculate_total_effects(&items);
        assert!(fx.fire_res);
        assert!(fx.regeneration);
        assert_eq!(fx.ac_bonus, 11); // 9 + 2
    }

    #[test]
    fn test_equipment_event_message() {
        let event = EquipmentEvent::CursedWear {
            slot: WornSlot::Gloves,
            item_name: "gauntlets of fumbling".to_string(),
        };
        let msg = equipment_event_message(&event);
        assert!(msg.contains("welds itself"));
    }
}

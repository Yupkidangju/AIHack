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
// [v2.3.1
//
//
//
//
// =============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponSkill {
    Dagger,
    Knife,
    Axe,
    ShortSword,
    BroadSword,
    LongSword,
    TwoHandedSword,
    Scimitar,
    Saber,
    Club,
    Mace,
    MorningStar,
    Flail,
    Hammer,
    Quarterstaff,
    Polearm,
    Spear,
    Javelin,
    Trident,
    Lance,
    Bow,
    Sling,
    Crossbow,
    Dart,
    Shuriken,
    Boomerang,
    Whip,
    PickAxe,
    Unicorn,
    ///
    BareHanded,
    ///
    MartialArts,
    ///
    AttackSpell,
    HealingSpell,
    DivinationSpell,
    EnchantmentSpell,
    ClericalSpell,
    EscapeSpell,
    MatterSpell,
    ///
    Riding,
    ///
    TwoWeaponCombat,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillLevel {
    ///
    Unskilled = 0,
    ///
    Basic = 1,
    ///
    Skilled = 2,
    ///
    Expert = 3,
    ///
    Master = 4,
    ///
    GrandMaster = 5,
}

impl SkillLevel {
    ///
    pub fn hit_bonus(self) -> i32 {
        match self {
            SkillLevel::Unskilled => -4,
            SkillLevel::Basic => 0,
            SkillLevel::Skilled => 2,
            SkillLevel::Expert => 3,
            SkillLevel::Master => 4,
            SkillLevel::GrandMaster => 5,
        }
    }

    ///
    pub fn damage_bonus(self) -> i32 {
        match self {
            SkillLevel::Unskilled => -2,
            SkillLevel::Basic => 0,
            SkillLevel::Skilled => 1,
            SkillLevel::Expert => 2,
            SkillLevel::Master => 3,
            SkillLevel::GrandMaster => 4,
        }
    }
}

///
#[derive(Debug, Clone)]
pub struct SkillData {
    ///
    pub level: SkillLevel,
    ///
    pub experience: i32,
    ///
    pub max_level: SkillLevel,
    ///
    pub practice_count: i32,
}

impl SkillData {
    pub fn new(max_level: SkillLevel) -> Self {
        Self {
            level: SkillLevel::Unskilled,
            experience: 0,
            max_level,
            practice_count: 0,
        }
    }

    ///
    pub fn add_experience(&mut self, amount: i32) {
        self.experience += amount;
        self.practice_count += 1;
    }

    ///
    pub fn can_advance(&self) -> bool {
        self.level < self.max_level && self.experience >= self.advance_threshold()
    }

    ///
    fn advance_threshold(&self) -> i32 {
        match self.level {
            SkillLevel::Unskilled => 20,
            SkillLevel::Basic => 40,
            SkillLevel::Skilled => 80,
            SkillLevel::Expert => 160,
            SkillLevel::Master => 320,
            SkillLevel::GrandMaster => i32::MAX,
        }
    }

    ///
    pub fn advance(&mut self) -> bool {
        if self.can_advance() {
            self.experience = 0;
            self.level = match self.level {
                SkillLevel::Unskilled => SkillLevel::Basic,
                SkillLevel::Basic => SkillLevel::Skilled,
                SkillLevel::Skilled => SkillLevel::Expert,
                SkillLevel::Expert => SkillLevel::Master,
                SkillLevel::Master => SkillLevel::GrandMaster,
                SkillLevel::GrandMaster => SkillLevel::GrandMaster,
            };
            true
        } else {
            false
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn weapon_type(item_name: &str) -> Option<WeaponSkill> {
    let l = item_name.to_lowercase();

    //
    if l.contains("dagger") || l.contains("stiletto") || l.contains("athame") {
        return Some(WeaponSkill::Dagger);
    }
    if l.contains("knife") || l.contains("scalpel") || l.contains("crysknife") {
        return Some(WeaponSkill::Knife);
    }
    //
    if l.contains("axe") || l.contains("cleaver") {
        return Some(WeaponSkill::Axe);
    }
    //
    if l.contains("two-handed sword") || l.contains("tsurugi") {
        return Some(WeaponSkill::TwoHandedSword);
    }
    if l.contains("long sword") || l.contains("katana") {
        return Some(WeaponSkill::LongSword);
    }
    if l.contains("broad sword") || l.contains("elven broadsword") {
        return Some(WeaponSkill::BroadSword);
    }
    if l.contains("short sword")
        || l.contains("orcish short")
        || l.contains("dwarvish short")
        || l.contains("elven short")
    {
        return Some(WeaponSkill::ShortSword);
    }
    if l.contains("scimitar") {
        return Some(WeaponSkill::Scimitar);
    }
    if l.contains("saber") || l.contains("silver saber") {
        return Some(WeaponSkill::Saber);
    }
    //
    if l.contains("club") || l.contains("aklys") {
        return Some(WeaponSkill::Club);
    }
    if l.contains("mace") {
        return Some(WeaponSkill::Mace);
    }
    if l.contains("morning star") {
        return Some(WeaponSkill::MorningStar);
    }
    if l.contains("flail") {
        return Some(WeaponSkill::Flail);
    }
    if l.contains("hammer") || l.contains("mjollnir") {
        return Some(WeaponSkill::Hammer);
    }
    if l.contains("quarterstaff") || l.contains("staff") {
        return Some(WeaponSkill::Quarterstaff);
    }
    //
    if l.contains("halberd")
        || l.contains("bardiche")
        || l.contains("voulge")
        || l.contains("glaive")
        || l.contains("guisarme")
        || l.contains("lucern")
        || l.contains("partisan")
        || l.contains("ranseur")
        || l.contains("fauchard")
        || l.contains("bill-guisarme")
        || l.contains("bec de corbin")
    {
        return Some(WeaponSkill::Polearm);
    }
    // 李쎈쪟
    if l.contains("spear")
        || l.contains("elven spear")
        || l.contains("dwarvish spear")
        || l.contains("orcish spear")
    {
        return Some(WeaponSkill::Spear);
    }
    if l.contains("javelin") {
        return Some(WeaponSkill::Javelin);
    }
    if l.contains("trident") {
        return Some(WeaponSkill::Trident);
    }
    if l.contains("lance") {
        return Some(WeaponSkill::Lance);
    }
    //
    if l.contains("bow") || l.contains("yumi") {
        return Some(WeaponSkill::Bow);
    }
    if l.contains("sling") {
        return Some(WeaponSkill::Sling);
    }
    if l.contains("crossbow") {
        return Some(WeaponSkill::Crossbow);
    }
    if l.contains("dart") {
        return Some(WeaponSkill::Dart);
    }
    if l.contains("shuriken") {
        return Some(WeaponSkill::Shuriken);
    }
    if l.contains("boomerang") {
        return Some(WeaponSkill::Boomerang);
    }
    //
    if l.contains("whip") || l.contains("rubber hose") {
        return Some(WeaponSkill::Whip);
    }
    if l.contains("pick-axe") || l.contains("pick") || l.contains("mattock") {
        return Some(WeaponSkill::PickAxe);
    }
    if l.contains("unicorn horn") {
        return Some(WeaponSkill::Unicorn);
    }

    None
}

///
pub fn is_blade(item_name: &str) -> bool {
    let l = item_name.to_lowercase();
    l.contains("sword")
        || l.contains("dagger")
        || l.contains("knife")
        || l.contains("scimitar")
        || l.contains("saber")
        || l.contains("axe")
        || l.contains("katana")
        || l.contains("tsurugi")
        || l.contains("stiletto")
        || l.contains("athame")
        || l.contains("scalpel")
        || l.contains("crysknife")
        || l.contains("cleaver")
}

///
pub fn is_launcher(item_name: &str) -> bool {
    let l = item_name.to_lowercase();
    l.contains("bow") || l.contains("sling") || l.contains("crossbow")
}

///
pub fn is_ammo(item_name: &str) -> bool {
    let l = item_name.to_lowercase();
    l.contains("arrow")
        || l.contains("bolt")
        || l.contains("dart")
        || l.contains("shuriken")
        || l.contains("boomerang")
        || l.contains("rock")
}

///
pub fn is_two_handed(item_name: &str) -> bool {
    let l = item_name.to_lowercase();
    l.contains("two-handed")
        || l.contains("battleaxe")
        || l.contains("tsurugi")
        || l.contains("quarterstaff")
        || l.contains("halberd")
        || l.contains("bardiche")
        || l.contains("voulge")
        || l.contains("glaive")
        || l.contains("guisarme")
        || l.contains("lucern")
        || l.contains("partisan")
        || l.contains("ranseur")
        || l.contains("fauchard")
        || l.contains("bill-guisarme")
        || l.contains("bec de corbin")
        || l.contains("bow")
        || l.contains("crossbow")
}

// =============================================================================
//
// =============================================================================

///
///
pub fn weapon_base_damage(item_name: &str) -> (i32, i32) {
    let l = item_name.to_lowercase();

    //
    if l.contains("dagger") || l.contains("stiletto") || l.contains("athame") {
        return (4, 3); // 1d4, 1d3
    }
    if l.contains("knife") || l.contains("scalpel") {
        return (3, 2); // 1d3, 1d2
    }
    //
    if l.contains("two-handed sword") || l.contains("tsurugi") {
        return (12, 18); // 1d12(+bonus), 3d6
    }
    if l.contains("long sword") || l.contains("katana") {
        return (8, 12); // 1d8, 1d12
    }
    if l.contains("broad sword") {
        return (8, 8); // 2d4, 1d6+1
    }
    if l.contains("short sword") {
        return (6, 8); // 1d6, 1d8
    }
    if l.contains("scimitar") || l.contains("saber") {
        return (8, 8); // 1d8, 1d8
    }
    //
    if l.contains("battle") && l.contains("axe") {
        return (8, 12); // 1d8+d4, 1d6+2d4
    }
    if l.contains("axe") {
        return (6, 4); // 1d6, 1d4
    }
    //
    if l.contains("club") {
        return (6, 3); // 1d6, 1d3
    }
    if l.contains("mace") {
        return (8, 6); // 1d6+1, 1d6
    }
    if l.contains("morning star") {
        return (8, 6); // 2d4, 1d6+1
    }
    if l.contains("flail") {
        return (8, 6); // 1d6+1, 2d4
    }
    if l.contains("war hammer") || l.contains("hammer") {
        return (6, 6); // 1d4+1, 1d4
    }
    if l.contains("quarterstaff") || l.contains("staff") {
        return (6, 6); // 1d6, 1d6
    }
    // 李쎈쪟
    if l.contains("spear") {
        return (6, 8); // 1d6, 1d8
    }
    if l.contains("javelin") {
        return (6, 6); // 1d6, 1d6
    }
    if l.contains("trident") {
        return (8, 6); // 1d6+1, 3d4
    }
    if l.contains("lance") {
        return (6, 8); // 1d6, 1d8
    }
    //
    if l.contains("arrow") {
        return (6, 6); // 1d6, 1d6
    }
    if l.contains("bolt") {
        return (8, 6); // 1d4+1, 1d6+1
    }
    if l.contains("dart") {
        return (3, 2); // 1d3, 1d2
    }
    if l.contains("shuriken") {
        return (8, 6); // 1d8, 1d6
    }
    //
    if l.contains("whip") {
        return (2, 2); // 1d2, 1d2
    }
    if l.contains("pick-axe") || l.contains("mattock") {
        return (6, 3); // 1d6, 1d3
    }
    if l.contains("unicorn horn") {
        return (12, 12); // 1d12, 1d12
    }
    if l.contains("crysknife") {
        return (10, 10); // 1d10, 1d10
    }

    //
    (4, 4)
}

///
pub fn enchantment_bonus(enchantment: i32) -> i32 {
    enchantment
}

///
///
pub fn damage_bonus(
    skill_level: SkillLevel,
    enchantment: i32,
    wielded_correctly: bool,
    vs_large: bool,
) -> i32 {
    let skill_bonus = skill_level.damage_bonus();
    let enchant_bonus = enchantment_bonus(enchantment);
    let wield_penalty = if !wielded_correctly { -2 } else { 0 };

    skill_bonus + enchant_bonus + wield_penalty
}

///
pub fn hit_bonus(
    skill_level: SkillLevel,
    enchantment: i32,
    wielded_correctly: bool,
    dexterity: i32,
    experience_level: i32,
) -> i32 {
    let skill_hit = skill_level.hit_bonus();
    let enchant_hit = enchantment;
    let wield_penalty = if !wielded_correctly { -4 } else { 0 };
    //
    let dex_bonus = (dexterity - 14).max(-3).min(3);
    //
    let level_bonus = experience_level / 5;

    skill_hit + enchant_hit + wield_penalty + dex_bonus + level_bonus
}

// =============================================================================
//
// =============================================================================

///
pub fn prefer_ranged_weapon(monster_symbol: char, distance: i32) -> bool {
    //
    if matches!(monster_symbol, 'C' | 'K' | 'O' | 'h' | 'e') && distance > 1 {
        return true;
    }
    distance > 3
}

///
pub fn silver_damage(
    item_name: &str,
    target_is_undead: bool,
    target_is_demon: bool,
    target_is_were: bool,
    rng: &mut NetHackRng,
) -> i32 {
    let l = item_name.to_lowercase();
    if l.contains("silver") || l.contains("silver saber") {
        if target_is_undead || target_is_demon || target_is_were {
            return (rng.rn2(20) + 1) as i32;
        }
    }
    0
}

///
///
pub fn weapon_erode_chance(item_name: &str, is_rustproof: bool, erode_type: ErodeType) -> bool {
    if is_rustproof {
        return false;
    }
    let l = item_name.to_lowercase();
    match erode_type {
        ErodeType::Rust => {
            //
            !l.contains("wooden") && !l.contains("leather") && !l.contains("bone")
        }
        ErodeType::Corrode => {
            //
            !l.contains("mithril") && !l.contains("silver") && !l.contains("wooden")
        }
        ErodeType::Burn => {
            //
            l.contains("wooden") || l.contains("leather") || l.contains("staff")
        }
        ErodeType::Rot => {
            //
            l.contains("leather") || l.contains("wooden") || l.contains("bone")
        }
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErodeType {
    Rust,    // ??
    Corrode,
    Burn,
    Rot,
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponMaterial {
    Iron,
    Steel,
    Silver,
    Mithril,
    Wood,
    Bone,
    Leather,
    Mineral,
    Gemstone,
    Gold,
    Platinum,
}

///
pub fn infer_material(item_name: &str) -> WeaponMaterial {
    let l = item_name.to_lowercase();
    if l.contains("silver") {
        return WeaponMaterial::Silver;
    }
    if l.contains("mithril") || l.contains("elven") {
        return WeaponMaterial::Mithril;
    }
    if l.contains("wooden") || l.contains("staff") || l.contains("club") || l.contains("bow") {
        return WeaponMaterial::Wood;
    }
    if l.contains("bone") {
        return WeaponMaterial::Bone;
    }
    if l.contains("leather") {
        return WeaponMaterial::Leather;
    }
    if l.contains("gold") {
        return WeaponMaterial::Gold;
    }
    if l.contains("diamond") || l.contains("ruby") {
        return WeaponMaterial::Gemstone;
    }
    WeaponMaterial::Iron
}

// =============================================================================
//
// =============================================================================

///
pub fn bare_hands_damage(
    skill_level: SkillLevel,
    player_level: i32,
    is_monk: bool,
    rng: &mut NetHackRng,
) -> i32 {
    let base = if is_monk {
        //
        1 + rng.rn2((player_level / 3 + 1).max(1) as i32) as i32
    } else {
        1 + rng.rn2(2) as i32
    };

    let skill_bonus = match skill_level {
        SkillLevel::Unskilled => 0,
        SkillLevel::Basic => 1,
        SkillLevel::Skilled => 2,
        SkillLevel::Expert => 3,
        SkillLevel::Master => 5,
        SkillLevel::GrandMaster => 7,
    };

    base + skill_bonus
}

///
///
pub fn monk_extra_attacks(skill_level: SkillLevel) -> i32 {
    match skill_level {
        SkillLevel::Master | SkillLevel::GrandMaster => 2,
        SkillLevel::Expert => 1,
        _ => 0,
    }
}

// =============================================================================
//
// =============================================================================

///
///
pub fn role_skill_caps(role: &str) -> Vec<(WeaponSkill, SkillLevel)> {
    match role.to_lowercase().as_str() {
        "archeologist" => vec![
            (WeaponSkill::Whip, SkillLevel::Expert),
            (WeaponSkill::PickAxe, SkillLevel::Expert),
            (WeaponSkill::Saber, SkillLevel::Skilled),
            (WeaponSkill::Dagger, SkillLevel::Basic),
        ],
        "barbarian" => vec![
            (WeaponSkill::TwoHandedSword, SkillLevel::Expert),
            (WeaponSkill::BroadSword, SkillLevel::Skilled),
            (WeaponSkill::Axe, SkillLevel::Expert),
            (WeaponSkill::ShortSword, SkillLevel::Skilled),
            (WeaponSkill::Club, SkillLevel::Skilled),
            (WeaponSkill::Flail, SkillLevel::Skilled),
            (WeaponSkill::Hammer, SkillLevel::Expert),
            (WeaponSkill::Spear, SkillLevel::Skilled),
            (WeaponSkill::BareHanded, SkillLevel::Expert),
        ],
        "caveman" | "cavewoman" => vec![
            (WeaponSkill::Club, SkillLevel::Expert),
            (WeaponSkill::Sling, SkillLevel::Expert),
            (WeaponSkill::Hammer, SkillLevel::Skilled),
            (WeaponSkill::Spear, SkillLevel::Skilled),
            (WeaponSkill::BareHanded, SkillLevel::Expert),
        ],
        "healer" => vec![
            (WeaponSkill::Knife, SkillLevel::Expert),
            (WeaponSkill::Quarterstaff, SkillLevel::Expert),
            (WeaponSkill::Dagger, SkillLevel::Skilled),
            (WeaponSkill::HealingSpell, SkillLevel::Expert),
        ],
        "knight" => vec![
            (WeaponSkill::LongSword, SkillLevel::Expert),
            (WeaponSkill::BroadSword, SkillLevel::Skilled),
            (WeaponSkill::Lance, SkillLevel::Expert),
            (WeaponSkill::Mace, SkillLevel::Skilled),
            (WeaponSkill::Saber, SkillLevel::Skilled),
            (WeaponSkill::Riding, SkillLevel::Expert),
            (WeaponSkill::TwoWeaponCombat, SkillLevel::Skilled),
        ],
        "monk" => vec![
            (WeaponSkill::BareHanded, SkillLevel::GrandMaster),
            (WeaponSkill::MartialArts, SkillLevel::GrandMaster),
            (WeaponSkill::Quarterstaff, SkillLevel::Skilled),
            (WeaponSkill::Spear, SkillLevel::Basic),
            (WeaponSkill::HealingSpell, SkillLevel::Expert),
            (WeaponSkill::ClericalSpell, SkillLevel::Skilled),
        ],
        "priest" | "priestess" => vec![
            (WeaponSkill::Mace, SkillLevel::Expert),
            (WeaponSkill::Club, SkillLevel::Skilled),
            (WeaponSkill::Quarterstaff, SkillLevel::Skilled),
            (WeaponSkill::ClericalSpell, SkillLevel::Expert),
            (WeaponSkill::HealingSpell, SkillLevel::Expert),
        ],
        "ranger" => vec![
            (WeaponSkill::Bow, SkillLevel::Expert),
            (WeaponSkill::Dagger, SkillLevel::Expert),
            (WeaponSkill::Knife, SkillLevel::Skilled),
            (WeaponSkill::ShortSword, SkillLevel::Skilled),
            (WeaponSkill::Spear, SkillLevel::Skilled),
            (WeaponSkill::Crossbow, SkillLevel::Expert),
        ],
        "rogue" => vec![
            (WeaponSkill::Dagger, SkillLevel::Expert),
            (WeaponSkill::Knife, SkillLevel::Expert),
            (WeaponSkill::ShortSword, SkillLevel::Expert),
            (WeaponSkill::Saber, SkillLevel::Skilled),
            (WeaponSkill::Club, SkillLevel::Skilled),
            (WeaponSkill::Crossbow, SkillLevel::Skilled),
            (WeaponSkill::Dart, SkillLevel::Expert),
            (WeaponSkill::TwoWeaponCombat, SkillLevel::Expert),
        ],
        "samurai" => vec![
            (WeaponSkill::LongSword, SkillLevel::Expert),
            (WeaponSkill::ShortSword, SkillLevel::Expert),
            (WeaponSkill::Bow, SkillLevel::Expert),
            (WeaponSkill::Spear, SkillLevel::Skilled),
            (WeaponSkill::Polearm, SkillLevel::Skilled),
            (WeaponSkill::TwoWeaponCombat, SkillLevel::Expert),
            (WeaponSkill::MartialArts, SkillLevel::Expert),
        ],
        "tourist" => vec![
            (WeaponSkill::Dart, SkillLevel::Expert),
            (WeaponSkill::Sling, SkillLevel::Skilled),
            (WeaponSkill::Dagger, SkillLevel::Skilled),
            (WeaponSkill::Whip, SkillLevel::Skilled),
        ],
        "valkyrie" => vec![
            (WeaponSkill::LongSword, SkillLevel::Expert),
            (WeaponSkill::Dagger, SkillLevel::Skilled),
            (WeaponSkill::Axe, SkillLevel::Expert),
            (WeaponSkill::Spear, SkillLevel::Skilled),
            (WeaponSkill::Hammer, SkillLevel::Expert),
            (WeaponSkill::BroadSword, SkillLevel::Skilled),
            (WeaponSkill::TwoWeaponCombat, SkillLevel::Skilled),
        ],
        "wizard" => vec![
            (WeaponSkill::Quarterstaff, SkillLevel::Expert),
            (WeaponSkill::Dagger, SkillLevel::Expert),
            (WeaponSkill::AttackSpell, SkillLevel::Expert),
            (WeaponSkill::EnchantmentSpell, SkillLevel::Expert),
            (WeaponSkill::DivinationSpell, SkillLevel::Expert),
            (WeaponSkill::EscapeSpell, SkillLevel::Expert),
            (WeaponSkill::MatterSpell, SkillLevel::Expert),
        ],
        _ => vec![(WeaponSkill::BareHanded, SkillLevel::Basic)],
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_type() {
        assert_eq!(weapon_type("long sword"), Some(WeaponSkill::LongSword));
        assert_eq!(weapon_type("dagger"), Some(WeaponSkill::Dagger));
        assert_eq!(weapon_type("bow"), Some(WeaponSkill::Bow));
        assert_eq!(weapon_type("random item"), None);
    }

    #[test]
    fn test_is_blade() {
        assert!(is_blade("long sword"));
        assert!(is_blade("dagger"));
        assert!(!is_blade("mace"));
    }

    #[test]
    fn test_skill_advance() {
        let mut skill = SkillData::new(SkillLevel::Expert);
        assert_eq!(skill.level, SkillLevel::Unskilled);
        skill.add_experience(25);
        assert!(skill.can_advance());
        skill.advance();
        assert_eq!(skill.level, SkillLevel::Basic);
    }

    #[test]
    fn test_bare_hands() {
        let mut rng = NetHackRng::new(42);
        let dmg = bare_hands_damage(SkillLevel::Expert, 10, true, &mut rng);
        assert!(dmg >= 1);
    }

    #[test]
    fn test_role_skills() {
        let caps = role_skill_caps("wizard");
        assert!(!caps.is_empty());
        assert!(caps.iter().any(|(s, _)| *s == WeaponSkill::Quarterstaff));
    }
}

// =============================================================================
// [v2.3.4
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponWeight {
    Feather,
    Light,   // 4-12
    Medium,  // 13-40
    Heavy,   // 41-100
    Massive, // 101+
}

///
pub fn weapon_weight_class(weight: i32) -> WeaponWeight {
    if weight <= 3 {
        WeaponWeight::Feather
    } else if weight <= 12 {
        WeaponWeight::Light
    } else if weight <= 40 {
        WeaponWeight::Medium
    } else if weight <= 100 {
        WeaponWeight::Heavy
    } else {
        WeaponWeight::Massive
    }
}

///
pub fn weight_speed_modifier(weight_class: WeaponWeight) -> i32 {
    match weight_class {
        WeaponWeight::Feather => 4,
        WeaponWeight::Light => 2,
        WeaponWeight::Medium => 0,
        WeaponWeight::Heavy => -2,
        WeaponWeight::Massive => -4,
    }
}

///
pub fn weapon_break_chance(item_name: &str, enchantment: i32, is_artifact: bool) -> i32 {
    if is_artifact {
        return 0;
    }
    let l = item_name.to_lowercase();
    let base = if l.contains("glass") {
        30
    } else if l.contains("crystal") {
        15
    } else if l.contains("wooden") {
        10
    } else if l.contains("iron") {
        3
    } else if l.contains("mithril") {
        1
    } else {
        5
    };
    //
    let enchant_mod = if enchantment < 0 {
        (-enchantment * 5) as i32
    } else {
        0
    };
    (base + enchant_mod).min(80)
}

///
pub fn thrown_penalty(item_name: &str, is_designed_for_throwing: bool, strength: i32) -> i32 {
    if is_designed_for_throwing {
        return 0;
    }
    let l = item_name.to_lowercase();
    let weight_penalty = if l.contains("two-handed") || l.contains("battleaxe") {
        -6
    } else if l.contains("sword") || l.contains("mace") {
        -3
    } else {
        -1
    };
    //
    let str_offset = (strength - 14).max(0);
    (weight_penalty + str_offset).min(0)
}

///
pub fn two_weapon_penalty(skill_level: SkillLevel, off_hand_skill: SkillLevel) -> (i32, i32) {
    //
    let main_penalty = match skill_level {
        SkillLevel::Expert => 0,
        SkillLevel::Skilled => -1,
        SkillLevel::Basic => -2,
        _ => -4,
    };
    let off_penalty = match off_hand_skill {
        SkillLevel::Expert => -1,
        SkillLevel::Skilled => -2,
        SkillLevel::Basic => -4,
        _ => -6,
    };
    (main_penalty, off_penalty)
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeaponSpecialEffect {
    None,
    Vorpal, // 李몄닔
    Drain,
    Fire,
    Cold,
    Shock,
    Poison,
    Bisect,
    Stun,
}

///
pub fn weapon_special_effect(item_name: &str) -> WeaponSpecialEffect {
    let l = item_name.to_lowercase();
    if l.contains("vorpal") {
        return WeaponSpecialEffect::Vorpal;
    }
    if l.contains("stormbringer") {
        return WeaponSpecialEffect::Drain;
    }
    if l.contains("fire brand") {
        return WeaponSpecialEffect::Fire;
    }
    if l.contains("frost brand") {
        return WeaponSpecialEffect::Cold;
    }
    if l.contains("mjollnir") {
        return WeaponSpecialEffect::Shock;
    }
    if l.contains("sting") {
        return WeaponSpecialEffect::Poison;
    }
    if l.contains("tsurugi") {
        return WeaponSpecialEffect::Bisect;
    }
    WeaponSpecialEffect::None
}

///
pub fn buc_weapon_modifier(blessed: bool, cursed: bool) -> i32 {
    if blessed {
        1
    } else if cursed {
        -1
    } else {
        0
    }
}

///
pub fn weapon_identify_hint(item_name: &str, enchantment: i32) -> &'static str {
    if enchantment > 0 {
        "It is well-crafted."
    } else if enchantment < 0 {
        "It looks worn and damaged."
    } else {
        "It seems ordinary."
    }
}

///
#[derive(Debug, Clone, Default)]
pub struct WeaponStatistics {
    pub total_hits: u32,
    pub total_misses: u32,
    pub critical_hits: u32,
    pub weapons_broken: u32,
    pub special_effects_triggered: u32,
    pub skills_advanced: u32,
}

impl WeaponStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_hit(&mut self, critical: bool) {
        self.total_hits += 1;
        if critical {
            self.critical_hits += 1;
        }
    }
    pub fn record_miss(&mut self) {
        self.total_misses += 1;
    }
    pub fn accuracy(&self) -> f32 {
        if self.total_hits + self.total_misses == 0 {
            0.0
        } else {
            self.total_hits as f32 / (self.total_hits + self.total_misses) as f32
        }
    }
}

#[cfg(test)]
mod weapon_extended_tests {
    use super::*;

    #[test]
    fn test_weight_class() {
        assert_eq!(weapon_weight_class(2), WeaponWeight::Feather);
        assert_eq!(weapon_weight_class(40), WeaponWeight::Medium);
        assert_eq!(weapon_weight_class(200), WeaponWeight::Massive);
    }

    #[test]
    fn test_speed_modifier() {
        assert!(
            weight_speed_modifier(WeaponWeight::Feather)
                > weight_speed_modifier(WeaponWeight::Heavy)
        );
    }

    #[test]
    fn test_break_chance() {
        assert_eq!(weapon_break_chance("Excalibur", 5, true), 0);
        assert!(weapon_break_chance("glass dagger", 0, false) > 20);
    }

    #[test]
    fn test_two_weapon() {
        let (main_p, off_p) = two_weapon_penalty(SkillLevel::Expert, SkillLevel::Basic);
        assert_eq!(main_p, 0);
        assert_eq!(off_p, -4);
    }

    #[test]
    fn test_special_effect() {
        assert_eq!(
            weapon_special_effect("Vorpal Blade"),
            WeaponSpecialEffect::Vorpal
        );
        assert_eq!(
            weapon_special_effect("long sword"),
            WeaponSpecialEffect::None
        );
    }

    #[test]
    fn test_weapon_stats() {
        let mut s = WeaponStatistics::new();
        s.record_hit(true);
        s.record_miss();
        assert_eq!(s.total_hits, 1);
        assert_eq!(s.critical_hits, 1);
        assert!(s.accuracy() > 0.4);
    }
}

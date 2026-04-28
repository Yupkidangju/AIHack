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

use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClericSpell {
    GeyserOfWater,
    FirePillar,
    Lightning,
    CurseItems,
    Insects,
    BlindYou,
    Paralyze,      // CLC_PARALYZE: 留덈퉬
    Confuse,
    OpenWounds,
    HealSelf,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MageSpell {
    PsiAttack,
    SummonNasty,
    Disappear,
    Stun,
    Haste,
    Clone,
    CurseItems,
    DestroyArmor,
    WeakenYou,
    Death,
    Aggravate,
}

///
#[derive(Debug, Clone)]
pub struct CastResult {
    pub success: bool,
    pub spell_name: String,
    pub damage: i32,
    pub duration: i32,
    pub monsters_summoned: i32,
    pub message: String,
    pub player_affected: bool,
    pub caster_affected: bool,
}

impl CastResult {
    pub fn new(spell: &str, msg: &str) -> Self {
        Self {
            success: true,
            spell_name: spell.to_string(),
            damage: 0,
            duration: 0,
            monsters_summoned: 0,
            message: msg.to_string(),
            player_affected: true,
            caster_affected: false,
        }
    }
    pub fn self_buff(spell: &str, msg: &str) -> Self {
        Self {
            success: true,
            spell_name: spell.to_string(),
            damage: 0,
            duration: 0,
            monsters_summoned: 0,
            message: msg.to_string(),
            player_affected: false,
            caster_affected: true,
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn cast_cleric_spell(
    spell: ClericSpell,
    caster_name: &str,
    caster_level: i32,
    player_mr: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> CastResult {
    match spell {
        ClericSpell::GeyserOfWater => {
            let dmg = rng.rn2(caster_level * 2 + 1) + 10;
            let msg = format!("{} summons a geyser of water!", caster_name);
            log.add_colored(&msg, [100, 150, 255], turn);
            let mut r = CastResult::new("geyser", &msg);
            r.damage = dmg as i32;
            r.duration = rng.rn2(5) as i32;
            r
        }
        ClericSpell::FirePillar => {
            let dmg = rng.rn2(caster_level + 1) + 8;
            let msg = format!("{} invokes a pillar of fire!", caster_name);
            log.add_colored(&msg, [255, 100, 0], turn);
            let mut r = CastResult::new("fire pillar", &msg);
            r.damage = dmg as i32;
            r
        }
        ClericSpell::Lightning => {
            let dmg = rng.rn2(caster_level + 1) + 6;
            let msg = format!("{} hurls a bolt of lightning!", caster_name);
            log.add_colored(&msg, [255, 255, 100], turn);
            let mut r = CastResult::new("lightning", &msg);
            r.damage = dmg as i32;
            r.duration = rng.rn2(3) as i32;
            r
        }
        ClericSpell::CurseItems => {
            //
            if resist_magic(player_mr, caster_level, rng) {
                let msg = format!("{} gestures at you, but you resist!", caster_name);
                log.add(&msg, turn);
                CastResult::new("curse items", &msg)
            } else {
                let msg = format!("{} curses your equipment!", caster_name);
                log.add_colored(&msg, [200, 50, 200], turn);
                CastResult::new("curse items", &msg)
            }
        }
        ClericSpell::Insects => {
            let count = rng.rn2(5) + 3;
            let msg = format!("{} summons a swarm of insects!", caster_name);
            log.add_colored(&msg, [150, 200, 50], turn);
            let mut r = CastResult::new("insects", &msg);
            r.monsters_summoned = count as i32;
            r
        }
        ClericSpell::BlindYou => {
            if resist_magic(player_mr, caster_level, rng) {
                let msg = format!("{} points at you, but you resist!", caster_name);
                log.add(&msg, turn);
                CastResult::new("blind", &msg)
            } else {
                let dur = rng.rn2(50) + 25;
                let msg = format!("{} points at you! You are blinded!", caster_name);
                log.add_colored(&msg, [50, 50, 50], turn);
                let mut r = CastResult::new("blind", &msg);
                r.duration = dur as i32;
                r
            }
        }
        ClericSpell::Paralyze => {
            if resist_magic(player_mr, caster_level, rng) {
                let msg = format!("You resist {}'s spell!", caster_name);
                log.add(&msg, turn);
                CastResult::new("paralyze", &msg)
            } else {
                let dur = rng.rn2(8) + 3;
                let msg = format!("{} chants and you are paralyzed!", caster_name);
                log.add_colored(&msg, [100, 100, 255], turn);
                let mut r = CastResult::new("paralyze", &msg);
                r.duration = dur as i32;
                r
            }
        }
        ClericSpell::Confuse => {
            if resist_magic(player_mr, caster_level, rng) {
                let msg = format!("You resist {}'s spell!", caster_name);
                log.add(&msg, turn);
                CastResult::new("confuse", &msg)
            } else {
                let dur = rng.rn2(30) + 10;
                let msg = format!("{} casts a spell of confusion!", caster_name);
                log.add_colored(&msg, [255, 100, 255], turn);
                let mut r = CastResult::new("confuse", &msg);
                r.duration = dur as i32;
                r
            }
        }
        ClericSpell::OpenWounds => {
            let dmg = rng.rn2(caster_level) + 5;
            let msg = format!("{} opens old wounds! ({} damage)", caster_name, dmg);
            log.add_colored(&msg, [255, 50, 50], turn);
            let mut r = CastResult::new("open wounds", &msg);
            r.damage = dmg as i32;
            r
        }
        ClericSpell::HealSelf => {
            let heal = rng.rn2(caster_level * 2) + 10;
            let msg = format!("{} heals itself!", caster_name);
            log.add(&msg, turn);
            let mut r = CastResult::self_buff("heal self", &msg);
            r.damage = -(heal as i32);
            r
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn cast_mage_spell(
    spell: MageSpell,
    caster_name: &str,
    caster_level: i32,
    player_mr: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> CastResult {
    match spell {
        MageSpell::PsiAttack => {
            if resist_magic(player_mr, caster_level, rng) {
                let msg = format!("{}'s psi bolt bounces off your mental shield!", caster_name);
                log.add(&msg, turn);
                CastResult::new("psi bolt", &msg)
            } else {
                let dmg = rng.rn2(caster_level + 1) + 5;
                let msg = format!("{} attacks your mind! ({} damage)", caster_name, dmg);
                log.add_colored(&msg, [200, 100, 255], turn);
                let mut r = CastResult::new("psi bolt", &msg);
                r.damage = dmg as i32;
                r
            }
        }
        MageSpell::SummonNasty => {
            let count = rng.rn2(5) + 1;
            let msg = format!(
                "{} summons {} nasty monster{}!",
                caster_name,
                count,
                if count != 1 { "s" } else { "" }
            );
            log.add_colored(&msg, [255, 50, 50], turn);
            let mut r = CastResult::new("summon nasty", &msg);
            r.monsters_summoned = count as i32;
            r
        }
        MageSpell::Disappear => {
            let msg = format!("{} becomes invisible!", caster_name);
            log.add(&msg, turn);
            CastResult::self_buff("disappear", &msg)
        }
        MageSpell::Stun => {
            if resist_magic(player_mr, caster_level, rng) {
                let msg = format!("You resist {}'s stunning spell!", caster_name);
                log.add(&msg, turn);
                CastResult::new("stun", &msg)
            } else {
                let dur = rng.rn2(15) + 5;
                let msg = format!("{} casts a stunning spell! You reel...", caster_name);
                log.add_colored(&msg, [255, 200, 100], turn);
                let mut r = CastResult::new("stun", &msg);
                r.duration = dur as i32;
                r
            }
        }
        MageSpell::Haste => {
            let msg = format!("{} speeds up!", caster_name);
            log.add(&msg, turn);
            let mut r = CastResult::self_buff("haste self", &msg);
            r.duration = (rng.rn2(50) + 50) as i32;
            r
        }
        MageSpell::Clone => {
            let msg = format!("{} gestures and a clone appears!", caster_name);
            log.add_colored(&msg, [200, 200, 0], turn);
            let mut r = CastResult::self_buff("clone", &msg);
            r.monsters_summoned = 1;
            r
        }
        MageSpell::CurseItems => {
            if resist_magic(player_mr, caster_level, rng) {
                let msg = "You resist the curse!";
                log.add(msg, turn);
                CastResult::new("curse items", msg)
            } else {
                let msg = format!("{} curses your belongings!", caster_name);
                log.add_colored(&msg, [200, 50, 200], turn);
                CastResult::new("curse items", &msg)
            }
        }
        MageSpell::DestroyArmor => {
            if resist_magic(player_mr, caster_level, rng) {
                let msg = "Your armor glows briefly, then subsides.";
                log.add(msg, turn);
                CastResult::new("destroy armor", msg)
            } else {
                let msg = format!("{} points at your armor and it crumbles!", caster_name);
                log.add_colored(&msg, [255, 100, 100], turn);
                CastResult::new("destroy armor", &msg)
            }
        }
        MageSpell::WeakenYou => {
            if resist_magic(player_mr, caster_level, rng) {
                let msg = format!("You resist {}'s weakening spell!", caster_name);
                log.add(&msg, turn);
                CastResult::new("weaken", &msg)
            } else {
                let msg = format!("{} drains your strength!", caster_name);
                log.add_colored(&msg, [200, 100, 100], turn);
                let mut r = CastResult::new("weaken", &msg);
                r.damage = -(rng.rn2(3) + 1) as i32;
                r
            }
        }
        MageSpell::Death => {
            //
            if resist_magic(player_mr, caster_level, rng) && rng.rn2(3) != 0 {
                let msg = format!(
                    "{} reaches out with a death touch, but you resist!",
                    caster_name
                );
                log.add(&msg, turn);
                CastResult::new("death touch", &msg)
            } else {
                let dmg = rng.rn2(caster_level * 3 + 1) + 15;
                let msg = format!(
                    "{} reaches out and touches you with the finger of death! ({} damage)",
                    caster_name, dmg
                );
                log.add_colored(&msg, [100, 0, 0], turn);
                let mut r = CastResult::new("death touch", &msg);
                r.damage = dmg as i32;
                r
            }
        }
        MageSpell::Aggravate => {
            let msg = format!(
                "{} chants, and you feel that monsters are aware of your presence.",
                caster_name
            );
            log.add_colored(&msg, [255, 200, 50], turn);
            CastResult::new("aggravate", &msg)
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn resist_magic(
    player_mr: i32,
    caster_level: i32,
    rng: &mut NetHackRng,
) -> bool {
    //
    //
    let mr_effective = (player_mr - caster_level * 2).max(0);
    rng.rn2(100) < mr_effective
}

// =============================================================================
//
// =============================================================================

///
pub fn choose_cleric_spell(
    caster_level: i32,
    caster_hp_pct: i32,
    rng: &mut NetHackRng,
) -> ClericSpell {
    //
    if caster_hp_pct < 30 && rng.rn2(3) != 0 {
        return ClericSpell::HealSelf;
    }

    //
    let spells: Vec<ClericSpell> = if caster_level >= 15 {
        vec![
            ClericSpell::GeyserOfWater,
            ClericSpell::FirePillar,
            ClericSpell::Lightning,
            ClericSpell::CurseItems,
            ClericSpell::Insects,
            ClericSpell::Paralyze,
        ]
    } else if caster_level >= 8 {
        vec![
            ClericSpell::FirePillar,
            ClericSpell::Lightning,
            ClericSpell::BlindYou,
            ClericSpell::Confuse,
            ClericSpell::OpenWounds,
            ClericSpell::Insects,
        ]
    } else {
        vec![
            ClericSpell::OpenWounds,
            ClericSpell::Confuse,
            ClericSpell::BlindYou,
            ClericSpell::HealSelf,
        ]
    };

    let idx = rng.rn2(spells.len() as i32) as usize;
    spells[idx]
}

///
pub fn choose_mage_spell(
    caster_level: i32,
    caster_hp_pct: i32,
    is_wizard: bool,
    rng: &mut NetHackRng,
) -> MageSpell {
    //
    if caster_hp_pct < 20 && rng.rn2(2) == 0 {
        return if is_wizard {
            MageSpell::Clone
        } else {
            MageSpell::Disappear
        };
    }

    let spells: Vec<MageSpell> = if is_wizard {
        vec![
            MageSpell::Death,
            MageSpell::SummonNasty,
            MageSpell::Clone,
            MageSpell::CurseItems,
            MageSpell::DestroyArmor,
            MageSpell::Aggravate,
            MageSpell::PsiAttack,
            MageSpell::Haste,
        ]
    } else if caster_level >= 12 {
        vec![
            MageSpell::PsiAttack,
            MageSpell::SummonNasty,
            MageSpell::Stun,
            MageSpell::DestroyArmor,
            MageSpell::CurseItems,
            MageSpell::Haste,
        ]
    } else if caster_level >= 6 {
        vec![
            MageSpell::PsiAttack,
            MageSpell::Stun,
            MageSpell::Disappear,
            MageSpell::WeakenYou,
        ]
    } else {
        vec![MageSpell::PsiAttack, MageSpell::Stun, MageSpell::WeakenYou]
    };

    let idx = rng.rn2(spells.len() as i32) as usize;
    spells[idx]
}

///
pub fn can_monster_cast(
    caster_level: i32,
    caster_confused: bool,
    caster_stunned: bool,
    is_cancelled: bool,
    rng: &mut NetHackRng,
) -> bool {
    if is_cancelled {
        return false;
    }
    if caster_confused || caster_stunned {
        return rng.rn2(3) == 0;
    }
    //
    rng.rn2(5) < caster_level.min(4)
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resist_magic() {
        let mut rng = NetHackRng::new(42);
        let high_mr_resists = (0..100).filter(|_| resist_magic(80, 5, &mut rng)).count();
        assert!(high_mr_resists > 30);
    }

    #[test]
    fn test_heal_self_when_low_hp() {
        let mut rng = NetHackRng::new(42);
        let mut heal_count = 0;
        for _ in 0..100 {
            if choose_cleric_spell(10, 20, &mut rng) == ClericSpell::HealSelf {
                heal_count += 1;
            }
        }
        assert!(heal_count > 30);
    }

    #[test]
    fn test_cancelled_cant_cast() {
        let mut rng = NetHackRng::new(42);
        assert!(!can_monster_cast(20, false, false, true, &mut rng));
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeamType {
    MagicMissile,
    FireBolt,
    ColdBolt,
    Sleep,
    Death,
    Lightning,
    PoisonGas,
    AcidStream,
    Disintegrate,
}

///
#[derive(Debug, Clone)]
pub struct BeamResult {
    pub beam_type: BeamType,
    pub damage: i32,
    pub reflected: bool,
    pub absorbed: bool,
    pub missed: bool,
    pub message: String,
}

///
pub fn cast_beam_spell(
    beam: BeamType,
    caster_name: &str,
    caster_level: i32,
    target_has_reflection: bool,
    target_has_magic_res: bool,
    target_fire_res: bool,
    target_cold_res: bool,
    target_sleep_res: bool,
    target_poison_res: bool,
    target_shock_res: bool,
    target_disintegrate_res: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> BeamResult {
    //
    if target_has_reflection {
        let msg = format!(
            "{}'s {} bounces off your reflection!",
            caster_name,
            beam_name(beam)
        );
        log.add_colored(&msg, [200, 200, 255], turn);
        return BeamResult {
            beam_type: beam,
            damage: 0,
            reflected: true,
            absorbed: false,
            missed: false,
            message: msg,
        };
    }

    //
    let resisted = match beam {
        BeamType::FireBolt => target_fire_res,
        BeamType::ColdBolt => target_cold_res,
        BeamType::Sleep => target_sleep_res,
        BeamType::PoisonGas => target_poison_res,
        BeamType::Lightning => target_shock_res,
        BeamType::MagicMissile => target_has_magic_res,
        BeamType::Death => target_has_magic_res,
        BeamType::Disintegrate => target_disintegrate_res,
        _ => false,
    };

    if resisted {
        let msg = format!("You resist {}'s {}!", caster_name, beam_name(beam));
        log.add(&msg, turn);
        return BeamResult {
            beam_type: beam,
            damage: 0,
            reflected: false,
            absorbed: true,
            missed: false,
            message: msg,
        };
    }

    //
    let base_damage = beam_base_damage(beam, caster_level, rng);
    let msg = format!(
        "{} zaps you with {}! ({} damage)",
        caster_name,
        beam_name(beam),
        base_damage
    );
    log.add_colored(&msg, beam_color(beam), turn);

    BeamResult {
        beam_type: beam,
        damage: base_damage,
        reflected: false,
        absorbed: false,
        missed: false,
        message: msg,
    }
}

///
fn beam_name(beam: BeamType) -> &'static str {
    match beam {
        BeamType::MagicMissile => "a magic missile",
        BeamType::FireBolt => "a bolt of fire",
        BeamType::ColdBolt => "a bolt of cold",
        BeamType::Sleep => "a sleep ray",
        BeamType::Death => "a finger of death",
        BeamType::Lightning => "a bolt of lightning",
        BeamType::PoisonGas => "a blast of poison gas",
        BeamType::AcidStream => "a stream of acid",
        BeamType::Disintegrate => "a disintegration beam",
    }
}

///
fn beam_color(beam: BeamType) -> [u8; 3] {
    match beam {
        BeamType::MagicMissile => [200, 100, 255],
        BeamType::FireBolt => [255, 100, 0],
        BeamType::ColdBolt => [100, 200, 255],
        BeamType::Sleep => [50, 50, 200],
        BeamType::Death => [100, 0, 0],
        BeamType::Lightning => [255, 255, 100],
        BeamType::PoisonGas => [50, 200, 50],
        BeamType::AcidStream => [200, 255, 0],
        BeamType::Disintegrate => [255, 0, 255],
    }
}

///
fn beam_base_damage(beam: BeamType, caster_level: i32, rng: &mut NetHackRng) -> i32 {
    match beam {
        BeamType::MagicMissile => rng.rn2(caster_level + 1) + 5,
        BeamType::FireBolt => rng.rn2(caster_level * 2 + 1) + 8,
        BeamType::ColdBolt => rng.rn2(caster_level * 2 + 1) + 8,
        BeamType::Sleep => 0,
        BeamType::Death => 999,
        BeamType::Lightning => rng.rn2(caster_level * 2 + 1) + 10,
        BeamType::PoisonGas => rng.rn2(15) + 5,
        BeamType::AcidStream => rng.rn2(caster_level + 1) + 6,
        BeamType::Disintegrate => 999,
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn casting_frequency(
    caster_level: i32,
    is_wizard: bool,
    is_priest: bool,
    distance_to_player: i32,
) -> i32 {
    //
    let mut freq = (caster_level / 4).clamp(1, 8);

    //
    if is_wizard {
        freq += 3;
    }
    if is_priest {
        freq += 2;
    }

    //
    if distance_to_player > 5 {
        freq -= 1;
    }
    if distance_to_player > 10 {
        freq -= 1;
    }

    freq.max(1)
}

///
pub fn max_casting_range(caster_level: i32) -> i32 {
    //
    (caster_level / 2 + 3).min(15)
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummonClass {
    Demon,
    Dragon,
    Giant,
    Undead,
    Troll,
    Insect,
    Elemental,
    Naga,
    Golem,
    Angel,
}

///
pub fn pick_summon_class(
    caster_level: i32,
    is_demon: bool,
    is_priest: bool,
    rng: &mut NetHackRng,
) -> SummonClass {
    //
    if is_demon && rng.rn2(3) == 0 {
        return SummonClass::Demon;
    }

    //
    if is_priest && rng.rn2(4) == 0 {
        return if rng.rn2(2) == 0 {
            SummonClass::Angel
        } else {
            SummonClass::Undead
        };
    }

    //
    let pool: Vec<SummonClass> = if caster_level >= 20 {
        vec![
            SummonClass::Demon,
            SummonClass::Dragon,
            SummonClass::Giant,
            SummonClass::Elemental,
        ]
    } else if caster_level >= 12 {
        vec![
            SummonClass::Dragon,
            SummonClass::Troll,
            SummonClass::Giant,
            SummonClass::Naga,
        ]
    } else if caster_level >= 6 {
        vec![
            SummonClass::Troll,
            SummonClass::Undead,
            SummonClass::Insect,
            SummonClass::Golem,
        ]
    } else {
        vec![SummonClass::Insect, SummonClass::Undead, SummonClass::Golem]
    };

    let idx = rng.rn2(pool.len() as i32) as usize;
    pool[idx]
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct SpellProperties {
    pub name: &'static str,
    pub mana_cost: i32,
    pub min_level: i32,
    pub offensive: bool,
    pub requires_los: bool,
    pub can_be_reflected: bool,
    pub can_be_resisted: bool,
}

///
pub fn cleric_spell_properties(spell: ClericSpell) -> SpellProperties {
    match spell {
        ClericSpell::GeyserOfWater => SpellProperties {
            name: "geyser of water",
            mana_cost: 8,
            min_level: 12,
            offensive: true,
            requires_los: true,
            can_be_reflected: false,
            can_be_resisted: false,
        },
        ClericSpell::FirePillar => SpellProperties {
            name: "fire pillar",
            mana_cost: 6,
            min_level: 8,
            offensive: true,
            requires_los: true,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        ClericSpell::Lightning => SpellProperties {
            name: "lightning",
            mana_cost: 5,
            min_level: 6,
            offensive: true,
            requires_los: true,
            can_be_reflected: true,
            can_be_resisted: true,
        },
        ClericSpell::CurseItems => SpellProperties {
            name: "curse items",
            mana_cost: 4,
            min_level: 5,
            offensive: true,
            requires_los: false,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        ClericSpell::Insects => SpellProperties {
            name: "summon insects",
            mana_cost: 5,
            min_level: 7,
            offensive: true,
            requires_los: false,
            can_be_reflected: false,
            can_be_resisted: false,
        },
        ClericSpell::BlindYou => SpellProperties {
            name: "blind",
            mana_cost: 3,
            min_level: 3,
            offensive: true,
            requires_los: true,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        ClericSpell::Paralyze => SpellProperties {
            name: "paralyze",
            mana_cost: 4,
            min_level: 5,
            offensive: true,
            requires_los: true,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        ClericSpell::Confuse => SpellProperties {
            name: "confuse",
            mana_cost: 3,
            min_level: 3,
            offensive: true,
            requires_los: true,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        ClericSpell::OpenWounds => SpellProperties {
            name: "open wounds",
            mana_cost: 2,
            min_level: 1,
            offensive: true,
            requires_los: true,
            can_be_reflected: false,
            can_be_resisted: false,
        },
        ClericSpell::HealSelf => SpellProperties {
            name: "heal self",
            mana_cost: 3,
            min_level: 1,
            offensive: false,
            requires_los: false,
            can_be_reflected: false,
            can_be_resisted: false,
        },
    }
}

///
pub fn mage_spell_properties(spell: MageSpell) -> SpellProperties {
    match spell {
        MageSpell::PsiAttack => SpellProperties {
            name: "psi bolt",
            mana_cost: 3,
            min_level: 3,
            offensive: true,
            requires_los: false,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        MageSpell::SummonNasty => SpellProperties {
            name: "summon nasties",
            mana_cost: 8,
            min_level: 10,
            offensive: true,
            requires_los: false,
            can_be_reflected: false,
            can_be_resisted: false,
        },
        MageSpell::Disappear => SpellProperties {
            name: "disappear",
            mana_cost: 2,
            min_level: 4,
            offensive: false,
            requires_los: false,
            can_be_reflected: false,
            can_be_resisted: false,
        },
        MageSpell::Stun => SpellProperties {
            name: "stun",
            mana_cost: 3,
            min_level: 5,
            offensive: true,
            requires_los: true,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        MageSpell::Haste => SpellProperties {
            name: "haste self",
            mana_cost: 3,
            min_level: 5,
            offensive: false,
            requires_los: false,
            can_be_reflected: false,
            can_be_resisted: false,
        },
        MageSpell::Clone => SpellProperties {
            name: "clone wizard",
            mana_cost: 10,
            min_level: 15,
            offensive: false,
            requires_los: false,
            can_be_reflected: false,
            can_be_resisted: false,
        },
        MageSpell::CurseItems => SpellProperties {
            name: "curse items",
            mana_cost: 4,
            min_level: 7,
            offensive: true,
            requires_los: false,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        MageSpell::DestroyArmor => SpellProperties {
            name: "destroy armor",
            mana_cost: 5,
            min_level: 8,
            offensive: true,
            requires_los: true,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        MageSpell::WeakenYou => SpellProperties {
            name: "weaken",
            mana_cost: 3,
            min_level: 4,
            offensive: true,
            requires_los: true,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        MageSpell::Death => SpellProperties {
            name: "death touch",
            mana_cost: 15,
            min_level: 20,
            offensive: true,
            requires_los: true,
            can_be_reflected: false,
            can_be_resisted: true,
        },
        MageSpell::Aggravate => SpellProperties {
            name: "aggravate",
            mana_cost: 4,
            min_level: 6,
            offensive: true,
            requires_los: false,
            can_be_reflected: false,
            can_be_resisted: false,
        },
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
///
pub fn special_resistance_combo(
    has_magic_res: bool,
    has_reflection: bool,
    has_free_action: bool,
    has_death_res: bool,
    spell_name: &str,
) -> bool {
    match spell_name {
        "death touch" => has_death_res || has_magic_res,
        "paralyze" => has_free_action,
        "blind" => false,
        "stun" => has_free_action,
        "confuse" => has_magic_res && has_free_action,
        "lightning" => has_reflection,
        _ => false,
    }
}

///
pub fn has_enough_mana_cleric(spell: ClericSpell, current_mana: i32) -> bool {
    current_mana >= cleric_spell_properties(spell).mana_cost
}

///
pub fn has_enough_mana_mage(spell: MageSpell, current_mana: i32) -> bool {
    current_mana >= mage_spell_properties(spell).mana_cost
}

// =============================================================================
// [v2.3.2
// =============================================================================
#[cfg(test)]
mod mcastu_extended_tests {
    use super::*;

    #[test]
    fn test_beam_spell_reflection() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = cast_beam_spell(
            BeamType::FireBolt,
            "orc",
            5,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            &mut rng,
            &mut log,
            100,
        );
        assert!(result.reflected);
        assert_eq!(result.damage, 0);
    }

    #[test]
    fn test_beam_spell_resistance() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = cast_beam_spell(
            BeamType::FireBolt,
            "orc",
            5,
            false,
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            &mut rng,
            &mut log,
            100,
        );
        assert!(result.absorbed);
        assert_eq!(result.damage, 0);
    }

    #[test]
    fn test_casting_frequency() {
        assert!(casting_frequency(20, true, false, 3) > casting_frequency(5, false, false, 10));
    }

    #[test]
    fn test_summon_class_selection() {
        let mut rng = NetHackRng::new(42);
        //
        let cls = pick_summon_class(25, false, false, &mut rng);
        assert!(matches!(
            cls,
            SummonClass::Demon | SummonClass::Dragon | SummonClass::Giant | SummonClass::Elemental
        ));
    }

    #[test]
    fn test_spell_properties() {
        let props = cleric_spell_properties(ClericSpell::GeyserOfWater);
        assert_eq!(props.min_level, 12);
        assert!(props.offensive);

        let props2 = mage_spell_properties(MageSpell::Death);
        assert_eq!(props2.mana_cost, 15);
    }

    #[test]
    fn test_special_resistance() {
        assert!(special_resistance_combo(
            true,
            false,
            false,
            false,
            "death touch"
        ));
        assert!(special_resistance_combo(
            false, false, true, false, "paralyze"
        ));
        assert!(!special_resistance_combo(
            false, false, false, false, "blind"
        ));
    }

    #[test]
    fn test_mana_check() {
        assert!(has_enough_mana_cleric(ClericSpell::OpenWounds, 5));
        assert!(!has_enough_mana_mage(MageSpell::Death, 10));
    }
}

// =============================================================================
// [v2.3.4
// =============================================================================

///
pub fn spell_range_cleric(spell: ClericSpell) -> i32 {
    match spell {
        ClericSpell::GeyserOfWater => 5,
        ClericSpell::FirePillar => 6,
        ClericSpell::Lightning => 12,
        ClericSpell::CurseItems => 255,
        ClericSpell::Insects => 8,
        ClericSpell::BlindYou => 10,
        ClericSpell::Paralyze => 7,
        ClericSpell::Confuse => 6,
        ClericSpell::OpenWounds => 4,
        ClericSpell::HealSelf => 0,
    }
}

///
pub fn spell_range_mage(spell: MageSpell) -> i32 {
    match spell {
        MageSpell::PsiAttack => 8,
        MageSpell::SummonNasty => 255,
        MageSpell::Disappear => 0,
        MageSpell::Stun => 6,
        MageSpell::Haste => 0,
        MageSpell::Clone => 0,
        MageSpell::CurseItems => 255,
        MageSpell::DestroyArmor => 10,
        MageSpell::WeakenYou => 8,
        MageSpell::Death => 12,
        MageSpell::Aggravate => 255,
    }
}

///
pub fn spell_cooldown_cleric(spell: ClericSpell) -> i32 {
    match spell {
        ClericSpell::GeyserOfWater => 15,
        ClericSpell::FirePillar => 12,
        ClericSpell::Lightning => 10,
        ClericSpell::CurseItems => 20,
        ClericSpell::Insects => 8,
        ClericSpell::BlindYou => 6,
        ClericSpell::Paralyze => 18,
        ClericSpell::Confuse => 6,
        ClericSpell::OpenWounds => 3,
        ClericSpell::HealSelf => 5,
    }
}

///
pub fn spell_cooldown_mage(spell: MageSpell) -> i32 {
    match spell {
        MageSpell::PsiAttack => 4,
        MageSpell::SummonNasty => 25,
        MageSpell::Disappear => 10,
        MageSpell::Stun => 6,
        MageSpell::Haste => 15,
        MageSpell::Clone => 30,
        MageSpell::CurseItems => 20,
        MageSpell::DestroyArmor => 12,
        MageSpell::WeakenYou => 8,
        MageSpell::Death => 35,
        MageSpell::Aggravate => 20,
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellResistResult {
    FullResist,
    HalfResist,
    NoResist,
    Reflected,
}

///
pub fn cleric_spell_resist(
    spell: ClericSpell,
    magic_resistance: bool,
    fire_resistance: bool,
    shock_resistance: bool,
    free_action: bool,
) -> SpellResistResult {
    match spell {
        ClericSpell::FirePillar => {
            if fire_resistance {
                SpellResistResult::FullResist
            } else if magic_resistance {
                SpellResistResult::HalfResist
            } else {
                SpellResistResult::NoResist
            }
        }
        ClericSpell::Lightning => {
            if shock_resistance {
                SpellResistResult::FullResist
            } else if magic_resistance {
                SpellResistResult::HalfResist
            } else {
                SpellResistResult::NoResist
            }
        }
        ClericSpell::Paralyze => {
            if free_action {
                SpellResistResult::FullResist
            } else if magic_resistance {
                SpellResistResult::HalfResist
            } else {
                SpellResistResult::NoResist
            }
        }
        ClericSpell::BlindYou | ClericSpell::Confuse => {
            if magic_resistance {
                SpellResistResult::HalfResist
            } else {
                SpellResistResult::NoResist
            }
        }
        ClericSpell::CurseItems => {
            if magic_resistance {
                SpellResistResult::HalfResist
            } else {
                SpellResistResult::NoResist
            }
        }
        _ => SpellResistResult::NoResist,
    }
}

///
pub fn mage_spell_resist(
    spell: MageSpell,
    magic_resistance: bool,
    reflection: bool,
    free_action: bool,
) -> SpellResistResult {
    match spell {
        MageSpell::Death => {
            if magic_resistance {
                SpellResistResult::FullResist
            } else {
                SpellResistResult::NoResist
            }
        }
        MageSpell::PsiAttack => {
            if reflection {
                SpellResistResult::Reflected
            } else if magic_resistance {
                SpellResistResult::HalfResist
            } else {
                SpellResistResult::NoResist
            }
        }
        MageSpell::Stun => {
            if free_action {
                SpellResistResult::FullResist
            } else if magic_resistance {
                SpellResistResult::HalfResist
            } else {
                SpellResistResult::NoResist
            }
        }
        MageSpell::DestroyArmor | MageSpell::WeakenYou | MageSpell::CurseItems => {
            if magic_resistance {
                SpellResistResult::HalfResist
            } else {
                SpellResistResult::NoResist
            }
        }
        _ => SpellResistResult::NoResist,
    }
}

///
pub fn spell_synergy_bonus(prev_spell: &str, current_spell: &str) -> i32 {
    let p = prev_spell.to_lowercase();
    let c = current_spell.to_lowercase();
    //
    if p.contains("blind") && c.contains("paralyze") {
        return 30;
    }
    //
    if p.contains("confuse") && c.contains("wound") {
        return 20;
    }
    //
    if p.contains("weaken") && c.contains("destroy") {
        return 25;
    }
    //
    if p.contains("haste") && c.contains("psi") {
        return 15;
    }
    0
}

///
pub fn casting_strategy(
    monster_hp_ratio: f32, // 0.0~1.0
    distance: i32,
    player_hp_ratio: f32,  // 0.0~1.0
    has_allies: bool,
) -> &'static str {
    //
    if monster_hp_ratio < 0.3 {
        if has_allies {
            return "defensive_with_summon";
        }
        return "heal_or_flee";
    }
    //
    if player_hp_ratio < 0.2 {
        return "finish_kill";
    }
    //
    if distance > 8 {
        return "ranged_offense";
    }
    //
    if distance <= 3 {
        return "close_debuff";
    }
    //
    "balanced_offense"
}

///
pub fn spell_threat_level(spell_name: &str) -> i32 {
    let l = spell_name.to_lowercase();
    if l.contains("death") {
        return 10;
    }
    if l.contains("paralyze") || l.contains("geyser") {
        return 8;
    }
    if l.contains("fire pillar") || l.contains("lightning") {
        return 7;
    }
    if l.contains("summon") {
        return 6;
    }
    if l.contains("curse") || l.contains("destroy") {
        return 5;
    }
    if l.contains("blind") || l.contains("confuse") || l.contains("stun") {
        return 4;
    }
    if l.contains("psi") || l.contains("wound") {
        return 3;
    }
    if l.contains("weaken") || l.contains("aggravate") {
        return 2;
    }
    1
}

///
#[derive(Debug, Clone, Default)]
pub struct McastuStatistics {
    pub spells_cast: u32,
    pub spells_resisted: u32,
    pub spells_reflected: u32,
    pub total_spell_damage: i32,
    pub heals_cast: u32,
    pub summons_cast: u32,
}

impl McastuStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_cast(&mut self, damage: i32) {
        self.spells_cast += 1;
        self.total_spell_damage += damage;
    }
    pub fn record_resist(&mut self) {
        self.spells_resisted += 1;
    }
    pub fn record_reflect(&mut self) {
        self.spells_reflected += 1;
    }
    pub fn resist_rate(&self) -> f32 {
        if self.spells_cast == 0 {
            0.0
        } else {
            self.spells_resisted as f32 / self.spells_cast as f32
        }
    }
}

#[cfg(test)]
mod mcastu_advanced_tests {
    use super::*;

    #[test]
    fn test_spell_range() {
        assert!(
            spell_range_cleric(ClericSpell::Lightning)
                > spell_range_cleric(ClericSpell::OpenWounds)
        );
        assert!(spell_range_mage(MageSpell::Death) > spell_range_mage(MageSpell::Stun));
    }

    #[test]
    fn test_spell_cooldown() {
        assert!(spell_cooldown_mage(MageSpell::Death) > spell_cooldown_mage(MageSpell::PsiAttack));
    }

    #[test]
    fn test_cleric_resist() {
        assert_eq!(
            cleric_spell_resist(ClericSpell::FirePillar, false, true, false, false),
            SpellResistResult::FullResist
        );
        assert_eq!(
            cleric_spell_resist(ClericSpell::Paralyze, false, false, false, true),
            SpellResistResult::FullResist
        );
    }

    #[test]
    fn test_mage_resist() {
        assert_eq!(
            mage_spell_resist(MageSpell::PsiAttack, false, true, false),
            SpellResistResult::Reflected
        );
        assert_eq!(
            mage_spell_resist(MageSpell::Death, true, false, false),
            SpellResistResult::FullResist
        );
    }

    #[test]
    fn test_synergy() {
        assert!(spell_synergy_bonus("blind", "paralyze") > 0);
        assert_eq!(spell_synergy_bonus("heal", "attack"), 0);
    }

    #[test]
    fn test_strategy() {
        assert_eq!(casting_strategy(0.1, 5, 0.5, false), "heal_or_flee");
        assert_eq!(casting_strategy(0.8, 2, 0.1, false), "finish_kill");
    }

    #[test]
    fn test_threat() {
        assert!(spell_threat_level("death touch") > spell_threat_level("confuse"));
    }

    #[test]
    fn test_mcastu_stats() {
        let mut s = McastuStatistics::new();
        s.record_cast(20);
        s.record_resist();
        assert_eq!(s.spells_cast, 1);
        assert!(s.resist_rate() > 0.9);
    }
}

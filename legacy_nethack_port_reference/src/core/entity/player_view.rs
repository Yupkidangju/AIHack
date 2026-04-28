// =============================================================================
// [v2.0.0
// =============================================================================
//
//
//
//
//
//
//
//
//
//
//
// =============================================================================

use crate::core::entity::player::{Alignment, HungerState, Player, PlayerClass, Race};

///
///
///
///
#[derive(Debug, Clone)]
pub struct PlayerCombatView {
    pub hp: i32,
    pub hp_max: i32,
    pub ac: i32,
    pub exp_level: i32,
    pub luck: i32,
    pub luck_bonus: i32,
    pub two_weapon: bool,
    pub str_base: i32,
    pub dex_base: i32,
}

impl PlayerCombatView {
    ///
    pub fn from_player(p: &Player) -> Self {
        Self {
            hp: p.hp,
            hp_max: p.hp_max,
            ac: p.ac,
            exp_level: p.exp_level,
            luck: p.luck,
            luck_bonus: p.luck_bonus,
            two_weapon: p.two_weapon,
            str_base: p.str.base,
            dex_base: p.dex.base,
        }
    }

    ///
    pub fn effective_luck(&self) -> i32 {
        (self.luck + self.luck_bonus).clamp(-13, 13)
    }
}

///
///
///
#[derive(Debug, Clone)]
pub struct PlayerSurvivalView {
    pub nutrition: i32,
    pub hunger: HungerState,
    pub equip_hunger_bonus: i32,
    pub prayer_cooldown: i32,
    pub piety: i32,
    pub alignment: Alignment,
    pub alignment_record: i32,
}

impl PlayerSurvivalView {
    ///
    pub fn from_player(p: &Player) -> Self {
        Self {
            nutrition: p.nutrition,
            hunger: p.hunger,
            equip_hunger_bonus: p.equip_hunger_bonus,
            prayer_cooldown: p.prayer_cooldown,
            piety: p.piety,
            alignment: p.alignment,
            alignment_record: p.alignment_record,
        }
    }
}

///
///
///
#[derive(Debug, Clone)]
pub struct PlayerProgressView {
    pub experience: u64,
    pub exp_level: i32,
    pub gold: u64,
    pub role: PlayerClass,
    pub race: Race,
}

impl PlayerProgressView {
    ///
    pub fn from_player(p: &Player) -> Self {
        Self {
            experience: p.experience,
            exp_level: p.exp_level,
            gold: p.gold,
            role: p.role,
            race: p.race,
        }
    }
}

///
///
///
#[derive(Debug, Clone)]
pub struct PlayerAttributeView {
    pub str_base: i32,
    pub str_max: i32,
    pub dex_base: i32,
    pub dex_max: i32,
    pub con_base: i32,
    pub con_max: i32,
    pub int_base: i32,
    pub int_max: i32,
    pub wis_base: i32,
    pub wis_max: i32,
    pub cha_base: i32,
    pub cha_max: i32,
    pub exercise: [i32; 6],
    pub attribute_recovery_turns: i32,
}

impl PlayerAttributeView {
    ///
    pub fn from_player(p: &Player) -> Self {
        Self {
            str_base: p.str.base,
            str_max: p.str.max,
            dex_base: p.dex.base,
            dex_max: p.dex.max,
            con_base: p.con.base,
            con_max: p.con.max,
            int_base: p.int.base,
            int_max: p.int.max,
            wis_base: p.wis.base,
            wis_max: p.wis.max,
            cha_base: p.cha.base,
            cha_max: p.cha.max,
            exercise: p.exercise,
            attribute_recovery_turns: p.attribute_recovery_turns,
        }
    }
}

// =============================================================================
// [v2.0.0
// =============================================================================
//
//
//
//
// =============================================================================

impl PlayerCombatView {
    ///
    pub fn apply_to(&self, p: &mut Player) {
        p.hp = self.hp;
        p.hp_max = self.hp_max;
        p.ac = self.ac;
        p.exp_level = self.exp_level;
        p.luck = self.luck;
        p.luck_bonus = self.luck_bonus;
        p.two_weapon = self.two_weapon;
        p.str.base = self.str_base;
        p.dex.base = self.dex_base;
    }
}

impl PlayerSurvivalView {
    ///
    pub fn apply_to(&self, p: &mut Player) {
        p.nutrition = self.nutrition;
        p.hunger = self.hunger;
        p.equip_hunger_bonus = self.equip_hunger_bonus;
        p.prayer_cooldown = self.prayer_cooldown;
        p.piety = self.piety;
        p.alignment = self.alignment;
        p.alignment_record = self.alignment_record;
    }
}

impl PlayerProgressView {
    ///
    pub fn apply_to(&self, p: &mut Player) {
        p.experience = self.experience;
        p.exp_level = self.exp_level;
        p.gold = self.gold;
        p.role = self.role;
        p.race = self.race;
    }
}

impl PlayerAttributeView {
    ///
    pub fn apply_to(&self, p: &mut Player) {
        p.str.base = self.str_base;
        p.str.max = self.str_max;
        p.dex.base = self.dex_base;
        p.dex.max = self.dex_max;
        p.con.base = self.con_base;
        p.con.max = self.con_max;
        p.int.base = self.int_base;
        p.int.max = self.int_max;
        p.wis.base = self.wis_base;
        p.wis.max = self.wis_max;
        p.cha.base = self.cha_base;
        p.cha.max = self.cha_max;
        p.exercise = self.exercise;
        p.attribute_recovery_turns = self.attribute_recovery_turns;
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_view_roundtrip() {
        let p = Player::new();
        let view = PlayerCombatView::from_player(&p);
        assert_eq!(view.hp, p.hp);
        assert_eq!(view.hp_max, p.hp_max);
        assert_eq!(view.ac, p.ac);
        assert_eq!(view.str_base, p.str.base);
        assert_eq!(view.effective_luck(), p.effective_luck());
    }

    #[test]
    fn test_combat_view_apply() {
        let mut p = Player::new();
        let mut view = p.as_combat_view();
        view.hp = 5;
        view.ac = 3;
        view.apply_to(&mut p);
        assert_eq!(p.hp, 5);
        assert_eq!(p.ac, 3);
    }

    #[test]
    fn test_survival_view_roundtrip() {
        let p = Player::new();
        let view = PlayerSurvivalView::from_player(&p);
        assert_eq!(view.nutrition, p.nutrition);
        assert_eq!(view.hunger, p.hunger);
        assert_eq!(view.piety, p.piety);
    }

    #[test]
    fn test_survival_view_apply() {
        let mut p = Player::new();
        let mut view = p.as_survival_view();
        view.nutrition = 100;
        view.piety = 50;
        view.apply_to(&mut p);
        assert_eq!(p.nutrition, 100);
        assert_eq!(p.piety, 50);
    }

    #[test]
    fn test_progress_view_roundtrip() {
        let p = Player::new();
        let view = PlayerProgressView::from_player(&p);
        assert_eq!(view.experience, p.experience);
        assert_eq!(view.gold, p.gold);
        assert_eq!(view.role, p.role);
    }

    #[test]
    fn test_attribute_view_roundtrip() {
        let p = Player::new();
        let view = PlayerAttributeView::from_player(&p);
        assert_eq!(view.str_base, p.str.base);
        assert_eq!(view.str_max, p.str.max);
        assert_eq!(view.exercise, p.exercise);
    }

    #[test]
    fn test_attribute_view_apply() {
        let mut p = Player::new();
        let mut view = p.as_attribute_view();
        view.str_base = 20;
        view.exercise[0] = 5;
        view.apply_to(&mut p);
        assert_eq!(p.str.base, 20);
        assert_eq!(p.exercise[0], 5);
    }

    #[test]
    fn test_all_views_independent() {
        //
        let mut p = Player::new();

        let mut combat = p.as_combat_view();
        let mut survival = p.as_survival_view();

        combat.hp = 1;
        survival.nutrition = 0;

        combat.apply_to(&mut p);
        assert_eq!(p.hp, 1);
        assert_eq!(p.nutrition, 900);

        survival.apply_to(&mut p);
        assert_eq!(p.nutrition, 0);
        assert_eq!(p.hp, 1);
    }
}

use crate::domain::combat::AttackProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerTemplate {
    pub hp: i16,
    pub ac: i16,
    pub hit_bonus: i16,
    pub damage_bonus: i16,
    pub attack_profile: AttackProfile,
}

pub fn adventurer_template() -> PlayerTemplate {
    PlayerTemplate {
        hp: 16,
        ac: 0,
        hit_bonus: 2,
        damage_bonus: 0,
        attack_profile: AttackProfile::dagger(),
    }
}

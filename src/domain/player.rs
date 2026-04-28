use crate::domain::combat::AttackProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerTemplate {
    pub hp: i16,
    pub ac: i16,
    pub hit_bonus: i16,
    pub damage_bonus: i16,
    pub attack_profile: AttackProfile,
}

/// [v0.1.0] Phase 3 기본 모험가 데이터다. 아이템/장비 엔티티 없이 내장 dagger 프로필을 사용한다.
pub fn adventurer_template() -> PlayerTemplate {
    PlayerTemplate {
        hp: 16,
        ac: 0,
        hit_bonus: 2,
        damage_bonus: 0,
        attack_profile: AttackProfile::dagger(),
    }
}

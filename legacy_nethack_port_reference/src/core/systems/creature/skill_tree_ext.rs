// ============================================================================
// [v2.34.0 R22-2] 스킬 시스템 (skill_tree_ext.rs)
// 원본: NetHack 3.6.7 weapon.c skill (스킬 숙련도)
// 무기/마법 스킬, 숙련도 4단계, 경험 축적
// ============================================================================

/// [v2.34.0 R22-2] 스킬 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    // 무기
    Dagger,
    Knife,
    Axe,
    ShortSword,
    BroadSword,
    LongSword,
    TwoHandedSword,
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
    UnicornHorn,
    // 마법
    AttackSpell,
    HealingSpell,
    DivineSpell,
    EnchantSpell,
    ClericSpell,
    EscapeSpell,
    MatterSpell,
    // 맨손
    BareHands,
    MartialArts,
    // 기타
    Riding,
    TwoWeapon,
}

/// [v2.34.0 R22-2] 숙련도
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Proficiency {
    Restricted, // 사용 불가
    Unskilled,  // 초보
    Basic,      // 기본
    Skilled,    // 숙련
    Expert,     // 전문가
}

/// [v2.34.0 R22-2] 스킬 데이터
#[derive(Debug, Clone)]
pub struct SkillData {
    pub skill: SkillType,
    pub current: Proficiency,
    pub max_allowed: Proficiency,
    pub experience: i32, // 누적 사용 횟수
    pub advance_threshold: i32,
}

impl SkillData {
    pub fn new(skill: SkillType, max: Proficiency) -> Self {
        Self {
            skill,
            current: Proficiency::Unskilled,
            max_allowed: max,
            experience: 0,
            advance_threshold: 100,
        }
    }
}

/// [v2.34.0 R22-2] 스킬 사용 시 경험 축적
pub fn gain_skill_experience(skill: &mut SkillData, amount: i32) {
    skill.experience += amount;
}

/// [v2.34.0 R22-2] 스킬 레벨업 가능 여부
pub fn can_advance(skill: &SkillData) -> bool {
    skill.current < skill.max_allowed && skill.experience >= skill.advance_threshold
}

/// [v2.34.0 R22-2] 스킬 레벨업
pub fn advance_skill(skill: &mut SkillData) -> bool {
    if !can_advance(skill) {
        return false;
    }
    skill.current = match skill.current {
        Proficiency::Restricted => Proficiency::Unskilled,
        Proficiency::Unskilled => Proficiency::Basic,
        Proficiency::Basic => Proficiency::Skilled,
        Proficiency::Skilled => Proficiency::Expert,
        Proficiency::Expert => return false,
    };
    skill.experience = 0;
    skill.advance_threshold = (skill.advance_threshold as f64 * 1.5) as i32;
    true
}

/// [v2.34.0 R22-2] 숙련도 보너스 (데미지/명중)
pub fn proficiency_bonus(prof: Proficiency) -> (i32, i32) {
    match prof {
        Proficiency::Restricted => (-4, -4),
        Proficiency::Unskilled => (-2, -2),
        Proficiency::Basic => (0, 0),
        Proficiency::Skilled => (1, 1),
        Proficiency::Expert => (2, 3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skill() {
        let s = SkillData::new(SkillType::LongSword, Proficiency::Expert);
        assert_eq!(s.current, Proficiency::Unskilled);
    }

    #[test]
    fn test_advance() {
        let mut s = SkillData::new(SkillType::LongSword, Proficiency::Expert);
        gain_skill_experience(&mut s, 100);
        assert!(can_advance(&s));
        assert!(advance_skill(&mut s));
        assert_eq!(s.current, Proficiency::Basic);
    }

    #[test]
    fn test_max_cap() {
        let mut s = SkillData::new(SkillType::Dagger, Proficiency::Basic);
        s.experience = 1000;
        advance_skill(&mut s); // unskilled → basic
        assert_eq!(s.current, Proficiency::Basic);
        // 여기서 basic이 max이므로 더 오를 수 없음
        s.experience = 1000;
        assert!(!can_advance(&s));
    }

    #[test]
    fn test_bonus() {
        assert_eq!(proficiency_bonus(Proficiency::Expert), (2, 3));
        assert_eq!(proficiency_bonus(Proficiency::Unskilled), (-2, -2));
    }

    #[test]
    fn test_restricted() {
        let s = SkillData::new(SkillType::MartialArts, Proficiency::Restricted);
        assert!(!can_advance(&s));
    }
}

// ============================================================================
// [v2.25.0 R13-2] 몬스터 데이터 쿼리 (mondata_ext.rs)
// 원본: NetHack 3.6.7 mondata.c (1,396줄)
// 몬스터 속성 쿼리 40종+, 크기/무게/식성, 상성 매트릭스
// ============================================================================

// =============================================================================
// [1] 몬스터 능력 플래그 (원본: mondata.c is_flyer, is_swimmer 등)
// =============================================================================

use bitflags::bitflags;

bitflags! {
    /// [v2.25.0 R13-2] 몬스터 능력 플래그
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MonAbility: u64 {
        const FLIES         = 1 << 0;
        const SWIMS         = 1 << 1;
        const BREATHLESS    = 1 << 2;
        const AMPHIBIOUS    = 1 << 3;
        const PASSES_WALLS  = 1 << 4;
        const TUNNELS       = 1 << 5;
        const NEEDSPICK     = 1 << 6;
        const CONCEALS      = 1 << 7;
        const HIDES         = 1 << 8;
        const TELEPATHIC    = 1 << 9;
        const ACID_BLOOD    = 1 << 10;
        const POISONOUS     = 1 << 11;
        const REGENERATES   = 1 << 12;
        const SEE_INVISIBLE = 1 << 13;
        const INFRAVISIBLE  = 1 << 14;
        const IS_UNDEAD     = 1 << 15;
        const IS_DEMON      = 1 << 16;
        const IS_DRAGON     = 1 << 17;
        const IS_GIANT      = 1 << 18;
        const IS_GOLEM      = 1 << 19;
        const IS_HUMAN      = 1 << 20;
        const IS_ELF        = 1 << 21;
        const IS_DWARF      = 1 << 22;
        const IS_ORC        = 1 << 23;
        const IS_GNOME      = 1 << 24;
        const NO_CORPSE     = 1 << 25;
        const VENOM_SPIT    = 1 << 26;
        const STALK         = 1 << 27;
        const MERC           = 1 << 28;
        const COLLECT       = 1 << 29;
        const SHAPESHIFTER  = 1 << 30;
        const NOPOLY        = 1 << 31;
    }
}

// =============================================================================
// [2] 몬스터 크기 (원본: mondata.c verysmall, bigmonst)
// =============================================================================

/// [v2.25.0 R13-2] 몬스터 크기
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MonSize {
    Tiny = 0,
    Small = 1,
    Medium = 2,
    Large = 3,
    Huge = 4,
    Gigantic = 5,
}

/// [v2.25.0 R13-2] 크기 판정
pub fn is_verysmall(size: MonSize) -> bool {
    size <= MonSize::Small
}
pub fn is_bigmonst(size: MonSize) -> bool {
    size >= MonSize::Large
}

// =============================================================================
// [3] 식성 (원본: mondata.c carnivorous, herbivorous)
// =============================================================================

/// [v2.25.0 R13-2] 식성
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Diet {
    Carnivore,
    Herbivore,
    Omnivore,
    Metallivore,
}

pub fn eats_meat(diet: Diet) -> bool {
    matches!(diet, Diet::Carnivore | Diet::Omnivore)
}
pub fn eats_plants(diet: Diet) -> bool {
    matches!(diet, Diet::Herbivore | Diet::Omnivore)
}
pub fn eats_metal(diet: Diet) -> bool {
    matches!(diet, Diet::Metallivore)
}

// =============================================================================
// [4] 몬스터 데이터 레코드
// =============================================================================

/// [v2.25.0 R13-2] 몬스터 데이터 레코드
#[derive(Debug, Clone)]
pub struct MonsterData {
    pub name: String,
    pub symbol: char,
    pub level: i32,
    pub speed: i32,
    pub ac: i32,
    pub weight: i32,
    pub size: MonSize,
    pub diet: Diet,
    pub abilities: MonAbility,
    pub alignment: i32,
    pub frequency: i32,
}

/// [v2.25.0 R13-2] 속성 쿼리 편의 함수
impl MonsterData {
    pub fn can_fly(&self) -> bool {
        self.abilities.contains(MonAbility::FLIES)
    }
    pub fn can_swim(&self) -> bool {
        self.abilities.contains(MonAbility::SWIMS)
    }
    pub fn is_undead(&self) -> bool {
        self.abilities.contains(MonAbility::IS_UNDEAD)
    }
    pub fn is_demon(&self) -> bool {
        self.abilities.contains(MonAbility::IS_DEMON)
    }
    pub fn is_dragon(&self) -> bool {
        self.abilities.contains(MonAbility::IS_DRAGON)
    }
    pub fn is_human(&self) -> bool {
        self.abilities.contains(MonAbility::IS_HUMAN)
    }
    pub fn has_acid_blood(&self) -> bool {
        self.abilities.contains(MonAbility::ACID_BLOOD)
    }
    pub fn is_poisonous(&self) -> bool {
        self.abilities.contains(MonAbility::POISONOUS)
    }
    pub fn regenerates(&self) -> bool {
        self.abilities.contains(MonAbility::REGENERATES)
    }
    pub fn passes_walls(&self) -> bool {
        self.abilities.contains(MonAbility::PASSES_WALLS)
    }
    pub fn is_shapeshifter(&self) -> bool {
        self.abilities.contains(MonAbility::SHAPESHIFTER)
    }
}

// =============================================================================
// [5] 상성 매트릭스 (원본: mondata.c hates_silver, vulnerable_to)
// =============================================================================

/// [v2.25.0 R13-2] 은 무기 취약도
pub fn hates_silver(abilities: MonAbility) -> bool {
    abilities.intersects(MonAbility::IS_UNDEAD | MonAbility::IS_DEMON | MonAbility::SHAPESHIFTER)
}

/// [v2.25.0 R13-2] 엘프/드워프/오크 상성
pub fn racial_enemy(a: MonAbility, b: MonAbility) -> bool {
    (a.contains(MonAbility::IS_ELF) && b.contains(MonAbility::IS_ORC))
        || (a.contains(MonAbility::IS_ORC) && b.contains(MonAbility::IS_ELF))
        || (a.contains(MonAbility::IS_DWARF) && b.contains(MonAbility::IS_ORC))
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dragon() -> MonsterData {
        MonsterData {
            name: "red dragon".to_string(),
            symbol: 'D',
            level: 15,
            speed: 9,
            ac: -1,
            weight: 4500,
            size: MonSize::Gigantic,
            diet: Diet::Omnivore,
            abilities: MonAbility::FLIES | MonAbility::IS_DRAGON | MonAbility::SEE_INVISIBLE,
            alignment: -4,
            frequency: 1,
        }
    }

    #[test]
    fn test_dragon_abilities() {
        let d = test_dragon();
        assert!(d.can_fly());
        assert!(d.is_dragon());
        assert!(!d.is_undead());
    }

    #[test]
    fn test_size() {
        assert!(is_verysmall(MonSize::Tiny));
        assert!(is_bigmonst(MonSize::Huge));
        assert!(!is_bigmonst(MonSize::Medium));
    }

    #[test]
    fn test_diet() {
        assert!(eats_meat(Diet::Carnivore));
        assert!(eats_meat(Diet::Omnivore));
        assert!(!eats_meat(Diet::Herbivore));
        assert!(eats_metal(Diet::Metallivore));
    }

    #[test]
    fn test_hates_silver() {
        assert!(hates_silver(MonAbility::IS_UNDEAD));
        assert!(hates_silver(MonAbility::IS_DEMON));
        assert!(!hates_silver(MonAbility::IS_DRAGON));
    }

    #[test]
    fn test_racial_enemy() {
        assert!(racial_enemy(MonAbility::IS_ELF, MonAbility::IS_ORC));
        assert!(!racial_enemy(MonAbility::IS_ELF, MonAbility::IS_HUMAN));
    }
}

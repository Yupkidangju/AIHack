// ============================================================================
// [v2.33.0 R21-2] 소환/생성 확장 (summon_ext.rs)
// 원본: NetHack 3.6.7 makemon.c/wizard.c 소환 확장
// 악마 소환, 위자드 소환, 제노사이드, 난수 몬스터
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.33.0 R21-2] 소환 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummonType {
    DemonGate { demon_name: String },
    WizardNasties { count: i32 },
    ScrollSummon { monster_class: char },
    AltarSummon { angel: bool },
    TrapSummon { count: i32 },
    GenocideReverse { monster_name: String },
}

/// [v2.33.0 R21-2] 악마 소환 (원본: msummon)
pub fn summon_demon(depth: i32, rng: &mut NetHackRng) -> SummonType {
    let demons = [
        "Juiblex",
        "Yeenoghu",
        "Orcus",
        "Baalzebub",
        "Asmodeus",
        "Demogorgon",
    ];
    let idx = rng.rn2(demons.len() as i32) as usize;
    SummonType::DemonGate {
        demon_name: demons[idx].to_string(),
    }
}

/// [v2.33.0 R21-2] 위자드 소환 (원본: nasty)
pub fn wizard_nasties(wizard_level: i32, rng: &mut NetHackRng) -> SummonType {
    let count = 1 + rng.rn2((wizard_level / 5).max(1));
    SummonType::WizardNasties { count }
}

/// [v2.33.0 R21-2] 제노사이드 대상 수 (원본: do_genocide)
pub fn genocide_count(monster_name: &str, is_blessed: bool) -> i32 {
    if is_blessed {
        1
    }
    // 축복 = 전체 종 하나
    else if monster_name.len() == 1 {
        1
    }
    // 클래스 문자
    else {
        0
    }
}

/// [v2.33.0 R21-2] 랜덤 몬스터 등급 (깊이 기반)
pub fn random_monster_difficulty(depth: i32, rng: &mut NetHackRng) -> i32 {
    let base = depth / 2;
    let variation = rng.rn2(5);
    (base + variation).max(1).min(30)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demon_summon() {
        let mut rng = NetHackRng::new(42);
        let result = summon_demon(30, &mut rng);
        assert!(matches!(result, SummonType::DemonGate { .. }));
    }

    #[test]
    fn test_wizard_nasties() {
        let mut rng = NetHackRng::new(42);
        if let SummonType::WizardNasties { count } = wizard_nasties(25, &mut rng) {
            assert!(count >= 1 && count <= 6);
        }
    }

    #[test]
    fn test_genocide_blessed() {
        assert_eq!(genocide_count("L", true), 1);
    }

    #[test]
    fn test_random_difficulty() {
        let mut rng = NetHackRng::new(42);
        let diff = random_monster_difficulty(20, &mut rng);
        assert!(diff >= 1 && diff <= 30);
    }
}

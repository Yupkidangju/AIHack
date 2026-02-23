// ============================================================================
// [v2.26.0 R14-3] 소문/오라클 (rumors_ext.rs)
// 원본: NetHack 3.6.7 rumors.c (290줄) + oracle 로직
// 소문 카테고리, 오라클 상담, 진위 판별
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 소문 구조 (원본: rumors.c getrumor)
// =============================================================================

/// [v2.26.0 R14-3] 소문 진위
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RumorTruth {
    True,
    False,
}

/// [v2.26.0 R14-3] 소문 엔트리
#[derive(Debug, Clone)]
pub struct Rumor {
    pub text: String,
    pub truth: RumorTruth,
}

/// [v2.26.0 R14-3] 진짜 소문 테이블 (원본 발췌)
pub const TRUE_RUMORS: &[&str] = &[
    "\"strstrng%\" is strstrng!",
    "A blessed tin of foo always gives positive effects.",
    "A ring of levitation will save you from a pit.",
    "Elbereth protects you from most monsters.",
    "Excalibur can be obtained from a fountain.",
    "Feeding a dog or cat a tripe ration will tame it.",
    "Tins of floating eye meat grant telepathy.",
    "Wearing a blindfold grants some protection from nymphs.",
    "You can pray safely every 300-500 turns.",
    "You can turn monsters to stone with a cockatrice corpse.",
];

/// [v2.26.0 R14-3] 거짓 소문 테이블 (원본 발췌)
pub const FALSE_RUMORS: &[&str] = &[
    "Always attack a floating eye in melee.",
    "Elbereth written in the dust lasts forever.",
    "Eating a cockatrice corpse makes you immune to petrification.",
    "Shopkeepers always have unlimited money.",
    "You can safely quaff from any fountain.",
    "Wands of wishing have unlimited charges.",
    "Prayer always works when your HP is low.",
    "Kicking a sink always gives useful items.",
];

/// [v2.26.0 R14-3] 랜덤 소문 (원본: getrumor)
pub fn get_rumor(want_true: bool, rng: &mut NetHackRng) -> Rumor {
    if want_true {
        let idx = rng.rn2(TRUE_RUMORS.len() as i32) as usize;
        Rumor {
            text: TRUE_RUMORS[idx].to_string(),
            truth: RumorTruth::True,
        }
    } else {
        let idx = rng.rn2(FALSE_RUMORS.len() as i32) as usize;
        Rumor {
            text: FALSE_RUMORS[idx].to_string(),
            truth: RumorTruth::False,
        }
    }
}

/// [v2.26.0 R14-3] 운에 따라 진짜/거짓 결정 (원본: 행운 양수면 진짜 확률 UP)
pub fn rumor_by_luck(luck: i32, rng: &mut NetHackRng) -> Rumor {
    let true_chance = (50 + luck * 5).clamp(10, 90);
    let roll = rng.rn2(100);
    get_rumor(roll < true_chance, rng)
}

// =============================================================================
// [2] 오라클 상담 (원본: rumors.c outrumor, oracle)
// =============================================================================

/// [v2.26.0 R14-3] 오라클 상담 등급
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleConsultation {
    /// 저렴한 상담 (일반 소문)
    Minor,
    /// 비싼 상담 (정확한 정보)
    Major,
}

/// [v2.26.0 R14-3] 오라클 비용
pub fn oracle_cost(consultation: OracleConsultation, depth: i32) -> i32 {
    match consultation {
        OracleConsultation::Minor => 50 + depth * 5,
        OracleConsultation::Major => 500 + depth * 50,
    }
}

/// [v2.26.0 R14-3] 오라클 상담 결과
pub fn consult_oracle(consultation: OracleConsultation, rng: &mut NetHackRng) -> Rumor {
    match consultation {
        OracleConsultation::Minor => rumor_by_luck(0, rng),
        OracleConsultation::Major => get_rumor(true, rng), // Major는 항상 진실
    }
}

// =============================================================================
// [3] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_rumor() {
        let mut rng = NetHackRng::new(42);
        let r = get_rumor(true, &mut rng);
        assert_eq!(r.truth, RumorTruth::True);
    }

    #[test]
    fn test_false_rumor() {
        let mut rng = NetHackRng::new(42);
        let r = get_rumor(false, &mut rng);
        assert_eq!(r.truth, RumorTruth::False);
    }

    #[test]
    fn test_luck_affects_truth() {
        // 높은 행운 → 대부분 진짜
        let mut true_count = 0;
        for seed in 0..20 {
            let mut rng = NetHackRng::new(seed);
            if rumor_by_luck(8, &mut rng).truth == RumorTruth::True {
                true_count += 1;
            }
        }
        assert!(true_count > 10); // 대부분 진짜
    }

    #[test]
    fn test_oracle_minor() {
        let cost = oracle_cost(OracleConsultation::Minor, 10);
        assert_eq!(cost, 100); // 50 + 10*5
    }

    #[test]
    fn test_oracle_major_always_true() {
        let mut rng = NetHackRng::new(42);
        let r = consult_oracle(OracleConsultation::Major, &mut rng);
        assert_eq!(r.truth, RumorTruth::True);
    }
}

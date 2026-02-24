// ============================================================================
// [v2.30.0 R18-1] 보석 판별 (gem_ext.rs)
// 원본: NetHack 3.6.7 o_init.c gems, objnam.c gem naming
// 보석/유리 구분, 가치 판정, 유니콘 교환
// ============================================================================

/// [v2.30.0 R18-1] 보석 등급
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemGrade {
    Precious,     // 진짜 보석 (높은 가치)
    SemiPrecious, // 반보석
    Worthless,    // 유리 (무가치)
}

/// [v2.30.0 R18-1] 보석 데이터
#[derive(Debug, Clone)]
pub struct GemData {
    pub name: &'static str,
    pub grade: GemGrade,
    pub base_value: i32,
    pub hardness: i32, // 1~10
}

/// [v2.30.0 R18-1] 보석 테이블 (원본 발췌)
pub fn gem_table() -> Vec<GemData> {
    vec![
        GemData {
            name: "diamond",
            grade: GemGrade::Precious,
            base_value: 4000,
            hardness: 10,
        },
        GemData {
            name: "ruby",
            grade: GemGrade::Precious,
            base_value: 3500,
            hardness: 9,
        },
        GemData {
            name: "sapphire",
            grade: GemGrade::Precious,
            base_value: 3000,
            hardness: 9,
        },
        GemData {
            name: "emerald",
            grade: GemGrade::Precious,
            base_value: 2500,
            hardness: 8,
        },
        GemData {
            name: "opal",
            grade: GemGrade::SemiPrecious,
            base_value: 800,
            hardness: 6,
        },
        GemData {
            name: "garnet",
            grade: GemGrade::SemiPrecious,
            base_value: 700,
            hardness: 7,
        },
        GemData {
            name: "amethyst",
            grade: GemGrade::SemiPrecious,
            base_value: 600,
            hardness: 7,
        },
        GemData {
            name: "jade",
            grade: GemGrade::SemiPrecious,
            base_value: 300,
            hardness: 6,
        },
        GemData {
            name: "worthless piece of white glass",
            grade: GemGrade::Worthless,
            base_value: 0,
            hardness: 5,
        },
        GemData {
            name: "worthless piece of blue glass",
            grade: GemGrade::Worthless,
            base_value: 0,
            hardness: 5,
        },
        GemData {
            name: "worthless piece of red glass",
            grade: GemGrade::Worthless,
            base_value: 0,
            hardness: 5,
        },
        GemData {
            name: "worthless piece of green glass",
            grade: GemGrade::Worthless,
            base_value: 0,
            hardness: 5,
        },
    ]
}

/// [v2.30.0 R18-1] 유니콘 보석 교환 판정 (원본: mon.c unicorn_gem)
pub fn unicorn_gem_value(gem_grade: GemGrade, unicorn_alignment_match: bool) -> i32 {
    match (gem_grade, unicorn_alignment_match) {
        (GemGrade::Precious, true) => 5,  // 행운 +5
        (GemGrade::Precious, false) => 1, // 행운 +1
        (GemGrade::SemiPrecious, _) => 0, // 효과 없음
        (GemGrade::Worthless, _) => -3,   // 행운 감소!
    }
}

/// [v2.30.0 R18-1] 미감별 보석 이름 (원본: gem descriptions)
pub fn unidentified_gem_name(color: &str) -> String {
    format!("{} gem", color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gem_table() {
        let table = gem_table();
        assert_eq!(table.len(), 12);
        assert_eq!(table[0].name, "diamond");
    }

    #[test]
    fn test_precious_value() {
        assert_eq!(unicorn_gem_value(GemGrade::Precious, true), 5);
    }

    #[test]
    fn test_worthless_penalty() {
        assert_eq!(unicorn_gem_value(GemGrade::Worthless, true), -3);
    }

    #[test]
    fn test_unid_name() {
        assert_eq!(unidentified_gem_name("red"), "red gem");
    }
}

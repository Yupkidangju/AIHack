// ============================================================================
// [v2.39.0 Phase 103-3] 능력치/속성 통합 (attribute_phase103_ext.rs)
// 원본: NetHack 3.6.7 src/attrib.c + src/role.c 능력치 통합
// 순수 결과 패턴
//
// 구현 범위:
//   - 6대 능력치 (STR/DEX/CON/INT/WIS/CHA)
//   - 능력치 변동 (운동, 중독, 질병, 아이템, 소원)
//   - 능력치 기반 보너스 계산
//   - 능력치 회복/저하
//   - 역할별 초기 능력치
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 능력치 시스템 — attribute_system
// =============================================================================

/// [v2.39.0 103-3] 능력치
#[derive(Debug, Clone)]
pub struct Attributes {
    pub strength: i32,     // 힘 (3~25, 18/**포함)
    pub dexterity: i32,    // 민첩 (3~25)
    pub constitution: i32, // 체력 (3~25)
    pub intelligence: i32, // 지능 (3~25)
    pub wisdom: i32,       // 지혜 (3~25)
    pub charisma: i32,     // 매력 (3~25)
    pub max_str: i32,
    pub max_dex: i32,
    pub max_con: i32,
    pub max_int: i32,
    pub max_wis: i32,
    pub max_cha: i32,
}

impl Attributes {
    /// 역할별 초기 능력치
    pub fn for_role(role: &str, rng: &mut NetHackRng) -> Self {
        let (s, d, c, i, w, ch) = match role {
            "전사" | "barbarian" => (16 + rng.rn2(3), 13 + rng.rn2(2), 16 + rng.rn2(2), 8, 8, 10),
            "마법사" | "wizard" => (8, 10, 10 + rng.rn2(2), 18 + rng.rn2(2), 14, 10),
            "도적" | "rogue" => (10, 16 + rng.rn2(3), 12, 12, 10, 14),
            "기사" | "knight" => (14, 12, 14, 10, 14, 16),
            "성직자" | "priest" => (10, 10, 12, 12, 18 + rng.rn2(2), 14),
            "발키리" | "valkyrie" => (16 + rng.rn2(2), 14, 16, 8, 10, 12),
            "수도승" | "monk" => (12, 16, 14, 10, 16, 10),
            "궁수" | "ranger" => (12, 16 + rng.rn2(2), 14, 12, 12, 10),
            _ => (12, 12, 12, 12, 12, 12),
        };
        Attributes {
            strength: s,
            dexterity: d,
            constitution: c,
            intelligence: i,
            wisdom: w,
            charisma: ch,
            max_str: s + 3,
            max_dex: d + 3,
            max_con: c + 3,
            max_int: i + 3,
            max_wis: w + 3,
            max_cha: ch + 3,
        }
    }

    /// 능력치 변동
    pub fn modify(&mut self, stat: &str, amount: i32) -> String {
        let (val, max) = match stat {
            "힘" | "str" => (&mut self.strength, self.max_str),
            "민첩" | "dex" => (&mut self.dexterity, self.max_dex),
            "체력" | "con" => (&mut self.constitution, self.max_con),
            "지능" | "int" => (&mut self.intelligence, self.max_int),
            "지혜" | "wis" => (&mut self.wisdom, self.max_wis),
            "매력" | "cha" => (&mut self.charisma, self.max_cha),
            _ => return "알 수 없는 능력치.".to_string(),
        };

        let old = *val;
        *val = (*val + amount).max(3).min(max);

        if amount > 0 {
            format!("{}이(가) 강해진 느낌이 든다! ({} → {})", stat, old, *val)
        } else if amount < 0 {
            format!("{}이(가) 약해진 느낌이 든다... ({} → {})", stat, old, *val)
        } else {
            "아무 변화도 없다.".to_string()
        }
    }

    /// 모든 능력치 복구 (소원/복원 주문)
    pub fn restore_all(&mut self) -> String {
        let mut restored = Vec::new();
        if self.strength < self.max_str {
            self.strength = self.max_str;
            restored.push("힘");
        }
        if self.dexterity < self.max_dex {
            self.dexterity = self.max_dex;
            restored.push("민첩");
        }
        if self.constitution < self.max_con {
            self.constitution = self.max_con;
            restored.push("체력");
        }
        if self.intelligence < self.max_int {
            self.intelligence = self.max_int;
            restored.push("지능");
        }
        if self.wisdom < self.max_wis {
            self.wisdom = self.max_wis;
            restored.push("지혜");
        }
        if self.charisma < self.max_cha {
            self.charisma = self.max_cha;
            restored.push("매력");
        }

        if restored.is_empty() {
            "복구할 능력치가 없다.".to_string()
        } else {
            format!("{} 복구됨!", restored.join(", "))
        }
    }
}

/// [v2.39.0 103-3] 능력치 기반 보너스
pub fn attribute_bonus(stat: i32) -> i32 {
    match stat {
        3 => -3,
        4..=5 => -2,
        6..=7 => -1,
        8..=15 => 0,
        16..=17 => 1,
        18..=20 => 2,
        21..=23 => 3,
        24..=25 => 4,
        _ => 0,
    }
}

/// [v2.39.0 103-3] 운반 하중 (힘 기반)
pub fn carry_capacity(strength: i32) -> i32 {
    match strength {
        3..=5 => 200,
        6..=10 => 400,
        11..=15 => 800,
        16..=17 => 1000,
        18 => 1200,
        19..=20 => 1400,
        21..=23 => 1800,
        _ => 2000,
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    #[test]
    fn test_warrior_attrs() {
        let mut rng = test_rng();
        let attrs = Attributes::for_role("전사", &mut rng);
        assert!(attrs.strength >= 16);
        assert!(attrs.constitution >= 16);
    }

    #[test]
    fn test_wizard_attrs() {
        let mut rng = test_rng();
        let attrs = Attributes::for_role("마법사", &mut rng);
        assert!(attrs.intelligence >= 18);
    }

    #[test]
    fn test_modify_increase() {
        let mut rng = test_rng();
        let mut attrs = Attributes::for_role("전사", &mut rng);
        let old = attrs.strength;
        let msg = attrs.modify("힘", 2);
        assert!(attrs.strength >= old);
        assert!(msg.contains("강해진"));
    }

    #[test]
    fn test_modify_decrease() {
        let mut rng = test_rng();
        let mut attrs = Attributes::for_role("전사", &mut rng);
        let msg = attrs.modify("힘", -5);
        assert!(msg.contains("약해진"));
    }

    #[test]
    fn test_clamp_min() {
        let mut rng = test_rng();
        let mut attrs = Attributes::for_role("전사", &mut rng);
        attrs.modify("지능", -100);
        assert_eq!(attrs.intelligence, 3); // 최소 3
    }

    #[test]
    fn test_restore_all() {
        let mut rng = test_rng();
        let mut attrs = Attributes::for_role("전사", &mut rng);
        attrs.modify("힘", -5);
        let msg = attrs.restore_all();
        assert!(msg.contains("복구"));
    }

    #[test]
    fn test_attribute_bonus() {
        assert_eq!(attribute_bonus(3), -3);
        assert_eq!(attribute_bonus(12), 0);
        assert_eq!(attribute_bonus(18), 2);
    }

    #[test]
    fn test_carry_capacity() {
        let weak = carry_capacity(5);
        let strong = carry_capacity(20);
        assert!(strong > weak);
    }
}

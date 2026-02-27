// ============================================================================
// [v2.38.0 Phase 102-3] 마법/주문 통합 (spellbook_phase102_ext.rs)
// 원본: NetHack 3.6.7 src/spell.c 핵심 미이식 함수 통합
// 순수 결과 패턴
//
// 구현 범위:
//   - 주문 학습 (마법서 공부)
//   - 주문 시전 (마나 소비/성공률)
//   - 주문 망각 (시간 경과)
//   - 주문 레벨/난이도 체계
//   - 주문 카테고리별 효과
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 주문 시스템 — spell_system
// =============================================================================

/// [v2.38.0 102-3] 주문 카테고리
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellSchool {
    Attack,      // 공격 마법
    Healing,     // 치유 마법
    Divination,  // 예지 마법
    Enchantment, // 강화 마법
    Clerical,    // 성직 마법
    Escape,      // 탈출 마법
    Matter,      // 물질 마법
}

/// [v2.38.0 102-3] 주문 정보
#[derive(Debug, Clone)]
pub struct Spell {
    pub name: String,
    pub school: SpellSchool,
    pub level: i32, // 1~7
    pub mana_cost: i32,
    pub turns_studied: i32, // 공부한 턴 수
    pub retention: i32,     // 잔여 기억 턴 (20000에서 감소)
    pub known: bool,        // 습득 여부
}

/// [v2.38.0 102-3] 주문서
pub struct Spellbook {
    pub spells: Vec<Spell>,
    pub max_spells: usize,
}

impl Spellbook {
    pub fn new() -> Self {
        Self {
            spells: Vec::new(),
            max_spells: 52,
        }
    }

    /// 주문 학습
    pub fn learn_spell(
        &mut self,
        name: &str,
        school: SpellSchool,
        level: i32,
        intelligence: i32,
        rng: &mut NetHackRng,
    ) -> Result<String, String> {
        if self.spells.len() >= self.max_spells {
            return Err("주문 목록이 가득 찼다!".to_string());
        }

        // 이미 알고 있는 주문인지 확인
        if let Some(existing) = self.spells.iter_mut().find(|s| s.name == name) {
            existing.retention = 20000; // 기억 갱신
            existing.turns_studied += 100;
            return Ok(format!("{} 주문 복습 완료! (기억 갱신)", name));
        }

        // 학습 성공률 = (지능 * 5 - 주문레벨 * 10)%
        let success_rate = (intelligence * 5 - level * 10).max(5).min(95);
        let roll = rng.rn2(100);

        if roll >= success_rate {
            return Err(format!(
                "{} 주문 학습 실패. (성공률: {}%)",
                name, success_rate
            ));
        }

        let mana_cost = level * 5 + 3;

        self.spells.push(Spell {
            name: name.to_string(),
            school,
            level,
            mana_cost,
            turns_studied: 100,
            retention: 20000,
            known: true,
        });

        Ok(format!(
            "{} 주문을 습득했다! (Lv.{}, 마나 {})",
            name, level, mana_cost
        ))
    }

    /// 주문 시전
    pub fn cast_spell(
        &self,
        name: &str,
        current_mana: i32,
        caster_level: i32,
        rng: &mut NetHackRng,
    ) -> Result<(i32, String), String> {
        let spell = self
            .spells
            .iter()
            .find(|s| s.name == name && s.known)
            .ok_or_else(|| format!("{} 주문을 모른다.", name))?;

        if spell.retention <= 0 {
            return Err(format!("{} 주문을 잊어버렸다!", name));
        }

        if current_mana < spell.mana_cost {
            return Err(format!(
                "마나가 부족하다! (필요: {}, 현재: {})",
                spell.mana_cost, current_mana
            ));
        }

        // 시전 성공률 = min(95, 70 + 시전자레벨 * 3 - 주문레벨 * 5)
        let success = (70 + caster_level * 3 - spell.level * 5).max(5).min(95);
        let roll = rng.rn2(100);

        if roll >= success {
            let wasted = spell.mana_cost / 2;
            return Ok((
                wasted,
                format!("{} 시전 실패! (마나 {} 소비)", name, wasted),
            ));
        }

        let effect = match spell.school {
            SpellSchool::Attack => format!("{}d{} 데미지!", spell.level, 6 + caster_level / 3),
            SpellSchool::Healing => format!("{}d{} 회복!", spell.level + 1, 8),
            SpellSchool::Divination => "주변이 밝혀진다!".to_string(),
            SpellSchool::Enchantment => "강화 효과 적용!".to_string(),
            SpellSchool::Escape => "순간이동!".to_string(),
            SpellSchool::Clerical => "신성한 힘이 발동한다!".to_string(),
            SpellSchool::Matter => "물질 변환 발동!".to_string(),
        };

        Ok((spell.mana_cost, format!("{} 시전 성공! {}", name, effect)))
    }

    /// 시간 경과에 따른 주문 망각
    pub fn decay_retention(&mut self, turns_elapsed: i32) {
        for spell in &mut self.spells {
            spell.retention -= turns_elapsed;
            if spell.retention <= 0 {
                spell.known = false;
                spell.retention = 0;
            }
        }
    }

    /// 알고 있는 주문 수
    pub fn known_count(&self) -> usize {
        self.spells.iter().filter(|s| s.known).count()
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

    /// 테스트용 직접 스펠 삽입 (RNG 의존 제거)
    fn insert_spell(book: &mut Spellbook, name: &str, school: SpellSchool, level: i32) {
        book.spells.push(Spell {
            name: name.to_string(),
            school,
            level,
            mana_cost: level * 5 + 3,
            turns_studied: 100,
            retention: 20000,
            known: true,
        });
    }

    #[test]
    fn test_learn_spell() {
        // 지능 20, 레벨 1 → 성공률 90%, 거의 항상 성공
        let mut book = Spellbook::new();
        let mut rng = NetHackRng::new(1); // 학습 보장 시드
        let result = book.learn_spell("마법 미사일", SpellSchool::Attack, 1, 20, &mut rng);
        assert!(result.is_ok());
        assert_eq!(book.known_count(), 1);
    }

    #[test]
    fn test_learn_high_level() {
        let mut book = Spellbook::new();
        let mut rng = test_rng();
        // 낮은 지능+고레벨 주문 — 성공 또는 실패 모두 허용
        let _result = book.learn_spell("핑거 오브 데스", SpellSchool::Attack, 7, 8, &mut rng);
    }

    #[test]
    fn test_cast_success() {
        let mut book = Spellbook::new();
        insert_spell(&mut book, "치유", SpellSchool::Healing, 1);
        // 레벨 10, 주문레벨 1 → 성공률 95%, rng2 roll은 대부분 성공
        let mut rng2 = NetHackRng::new(1);
        let result = book.cast_spell("치유", 50, 10, &mut rng2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cast_no_mana() {
        let mut book = Spellbook::new();
        insert_spell(&mut book, "화염구", SpellSchool::Attack, 3);
        let mut rng = test_rng();
        let result = book.cast_spell("화염구", 1, 10, &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_cast_unknown() {
        let book = Spellbook::new();
        let mut rng = test_rng();
        let result = book.cast_spell("미지의 주문", 50, 10, &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_retention_decay() {
        let mut book = Spellbook::new();
        insert_spell(&mut book, "빛", SpellSchool::Divination, 1);
        assert_eq!(book.known_count(), 1);
        book.decay_retention(25000);
        assert_eq!(book.known_count(), 0);
    }

    #[test]
    fn test_refresh_retention() {
        let mut book = Spellbook::new();
        insert_spell(&mut book, "빛", SpellSchool::Divination, 1);
        book.decay_retention(15000);
        // 복습 (이미 존재하므로 기억 갱신)
        let mut rng = test_rng();
        let result = book.learn_spell("빛", SpellSchool::Divination, 1, 18, &mut rng);
        assert!(result.is_ok());
        assert_eq!(book.known_count(), 1);
    }

    #[test]
    fn test_mana_cost_scaling() {
        let mut book = Spellbook::new();
        insert_spell(&mut book, "약한 주문", SpellSchool::Attack, 1);
        insert_spell(&mut book, "강한 주문", SpellSchool::Attack, 5);
        let weak = &book.spells[0];
        let strong = &book.spells[1];
        assert!(strong.mana_cost > weak.mana_cost);
    }
}

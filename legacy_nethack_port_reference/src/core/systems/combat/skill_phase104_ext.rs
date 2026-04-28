// ============================================================================
// [v2.40.0 Phase 104-3] 스킬/직업 통합 (skill_phase104_ext.rs)
// 원본: NetHack 3.6.7 src/weapon.c + src/role.c 스킬 시스템 통합
// 순수 결과 패턴
//
// 구현 범위:
//   - 무기 스킬 체계 (미숙련~대가)
//   - 스킬 경험치 / 레벨업
//   - 스킬 슬롯 관리
//   - 직업별 스킬 제한
//   - 스킬 보너스 계산
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.40.0 104-3] 스킬 숙련도
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillLevel {
    Unskilled,   // 미숙련
    Basic,       // 기본
    Skilled,     // 숙련
    Expert,      // 전문
    Master,      // 대가
    GrandMaster, // 대종사 (특수)
}

/// [v2.40.0 104-3] 스킬 카테고리
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCategory {
    Dagger,       // 단검류
    Sword,        // 검류
    BroadSword,   // 광검류
    TwoHandSword, // 양손검
    Axe,          // 도끼류
    Polearm,      // 폴암
    Mace,         // 메이스류
    Whip,         // 채찍
    Bow,          // 활
    Crossbow,     // 석궁
    Sling,        // 투석기
    MartialArts,  // 격투
    Shield,       // 방패
    Riding,       // 기승
    TwoWeapon,    // 이도류
    BareHands,    // 맨손
}

/// [v2.40.0 104-3] 스킬 데이터
#[derive(Debug, Clone)]
pub struct Skill {
    pub category: SkillCategory,
    pub level: SkillLevel,
    pub xp: i32,               // 현재 경험치
    pub max_level: SkillLevel, // 역할별 최대 숙련도
}

/// [v2.40.0 104-3] 스킬 매니저
#[derive(Debug, Clone)]
pub struct SkillManager {
    pub skills: Vec<Skill>,
    pub available_slots: i32, // 사용 가능한 스킬 포인트
}

impl SkillManager {
    /// 역할별 초기 스킬 세트
    pub fn for_role(role: &str) -> Self {
        let skills = match role {
            "전사" | "barbarian" => vec![
                Skill {
                    category: SkillCategory::Sword,
                    level: SkillLevel::Basic,
                    xp: 0,
                    max_level: SkillLevel::Expert,
                },
                Skill {
                    category: SkillCategory::Axe,
                    level: SkillLevel::Basic,
                    xp: 0,
                    max_level: SkillLevel::Expert,
                },
                Skill {
                    category: SkillCategory::TwoHandSword,
                    level: SkillLevel::Unskilled,
                    xp: 0,
                    max_level: SkillLevel::Master,
                },
                Skill {
                    category: SkillCategory::Shield,
                    level: SkillLevel::Unskilled,
                    xp: 0,
                    max_level: SkillLevel::Skilled,
                },
                Skill {
                    category: SkillCategory::BareHands,
                    level: SkillLevel::Unskilled,
                    xp: 0,
                    max_level: SkillLevel::Expert,
                },
            ],
            "도적" | "rogue" => vec![
                Skill {
                    category: SkillCategory::Dagger,
                    level: SkillLevel::Skilled,
                    xp: 0,
                    max_level: SkillLevel::Expert,
                },
                Skill {
                    category: SkillCategory::Sword,
                    level: SkillLevel::Basic,
                    xp: 0,
                    max_level: SkillLevel::Skilled,
                },
                Skill {
                    category: SkillCategory::TwoWeapon,
                    level: SkillLevel::Unskilled,
                    xp: 0,
                    max_level: SkillLevel::Expert,
                },
                Skill {
                    category: SkillCategory::Crossbow,
                    level: SkillLevel::Unskilled,
                    xp: 0,
                    max_level: SkillLevel::Basic,
                },
            ],
            "수도승" | "monk" => vec![
                Skill {
                    category: SkillCategory::MartialArts,
                    level: SkillLevel::Skilled,
                    xp: 0,
                    max_level: SkillLevel::GrandMaster,
                },
                Skill {
                    category: SkillCategory::BareHands,
                    level: SkillLevel::Basic,
                    xp: 0,
                    max_level: SkillLevel::GrandMaster,
                },
            ],
            _ => vec![
                Skill {
                    category: SkillCategory::Sword,
                    level: SkillLevel::Unskilled,
                    xp: 0,
                    max_level: SkillLevel::Skilled,
                },
                Skill {
                    category: SkillCategory::BareHands,
                    level: SkillLevel::Unskilled,
                    xp: 0,
                    max_level: SkillLevel::Basic,
                },
            ],
        };

        SkillManager {
            skills,
            available_slots: 1,
        }
    }

    /// 스킬 경험치 추가
    pub fn gain_xp(&mut self, category: SkillCategory, amount: i32) {
        if let Some(skill) = self.skills.iter_mut().find(|s| s.category == category) {
            skill.xp += amount;
        }
    }

    /// 스킬 레벨 향상 (슬롯 소비)
    pub fn enhance_skill(&mut self, category: SkillCategory) -> Result<String, String> {
        if self.available_slots <= 0 {
            return Err("사용 가능한 스킬 포인트가 없다.".to_string());
        }

        let skill = self
            .skills
            .iter_mut()
            .find(|s| s.category == category)
            .ok_or_else(|| "해당 스킬이 없다.".to_string())?;

        if skill.level >= skill.max_level {
            return Err(format!("{:?} 스킬은 이미 최대 숙련도다.", category));
        }

        let xp_needed = match skill.level {
            SkillLevel::Unskilled => 20,
            SkillLevel::Basic => 50,
            SkillLevel::Skilled => 100,
            SkillLevel::Expert => 200,
            SkillLevel::Master => 400,
            SkillLevel::GrandMaster => 999,
        };

        if skill.xp < xp_needed {
            return Err(format!("경험치 부족 ({}/{})", skill.xp, xp_needed));
        }

        skill.level = match skill.level {
            SkillLevel::Unskilled => SkillLevel::Basic,
            SkillLevel::Basic => SkillLevel::Skilled,
            SkillLevel::Skilled => SkillLevel::Expert,
            SkillLevel::Expert => SkillLevel::Master,
            SkillLevel::Master => SkillLevel::GrandMaster,
            SkillLevel::GrandMaster => SkillLevel::GrandMaster,
        };
        skill.xp = 0;
        self.available_slots -= 1;

        Ok(format!(
            "{:?} 스킬이 {:?}(으)로 향상!",
            category, skill.level
        ))
    }

    /// 스킬 보너스 (명중/데미지)
    pub fn skill_bonus(level: SkillLevel) -> (i32, i32) {
        match level {
            SkillLevel::Unskilled => (-2, -1),
            SkillLevel::Basic => (0, 0),
            SkillLevel::Skilled => (1, 1),
            SkillLevel::Expert => (2, 2),
            SkillLevel::Master => (3, 3),
            SkillLevel::GrandMaster => (4, 4),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warrior_skills() {
        let sm = SkillManager::for_role("전사");
        assert!(sm.skills.len() >= 4);
    }

    #[test]
    fn test_monk_martial_arts() {
        let sm = SkillManager::for_role("수도승");
        let ma = sm
            .skills
            .iter()
            .find(|s| s.category == SkillCategory::MartialArts);
        assert!(ma.is_some());
        assert_eq!(ma.unwrap().max_level, SkillLevel::GrandMaster);
    }

    #[test]
    fn test_gain_xp() {
        let mut sm = SkillManager::for_role("전사");
        sm.gain_xp(SkillCategory::Sword, 50);
        let sword = sm
            .skills
            .iter()
            .find(|s| s.category == SkillCategory::Sword)
            .unwrap();
        assert_eq!(sword.xp, 50);
    }

    #[test]
    fn test_enhance() {
        let mut sm = SkillManager::for_role("전사");
        sm.gain_xp(SkillCategory::TwoHandSword, 100);
        let result = sm.enhance_skill(SkillCategory::TwoHandSword);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enhance_no_slots() {
        let mut sm = SkillManager::for_role("전사");
        sm.available_slots = 0;
        sm.gain_xp(SkillCategory::Sword, 100);
        let result = sm.enhance_skill(SkillCategory::Sword);
        assert!(result.is_err());
    }

    #[test]
    fn test_enhance_max() {
        let mut sm = SkillManager::for_role("수도승");
        // 격투를 이미 대종사로 만들기
        sm.skills[0].level = SkillLevel::GrandMaster;
        let result = sm.enhance_skill(SkillCategory::MartialArts);
        assert!(result.is_err());
    }

    #[test]
    fn test_bonus() {
        let (hit, dmg) = SkillManager::skill_bonus(SkillLevel::Expert);
        assert_eq!(hit, 2);
        assert_eq!(dmg, 2);
    }
}

// ============================================================================
// [v2.33.0 Phase 97-1] 음식/영양 확장 (eat_phase97_ext.rs)
// 원본: NetHack 3.6.7 src/eat.c L800-2500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 시체 효과 — corpse_effects (eat.c L800-1500)
// =============================================================================

/// [v2.33.0 97-1] 시체 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpseType {
    Normal,
    Poisonous,
    Acidic,
    Petrifying,
    Tainted,
    Cannibalism,
    Polymorph,
    Teleport,
    Invisible,
    Lycanthrope,
    Giant,
    Dragon,
}

/// [v2.33.0 97-1] 시체 섭취 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpseEffect {
    Nutrition {
        amount: i32,
    },
    Poison {
        damage: i32,
        stat_loss: Option<(String, i32)>,
    },
    Acid {
        damage: i32,
    },
    Petrifying {
        turns_until_stone: i32,
    },
    Disease {
        disease: String,
    },
    GainResistance {
        resistance: String,
    },
    GainStrength {
        amount: i32,
    },
    Polymorph,
    Teleportitis,
    Invisibility {
        turns: i32,
    },
    Lycanthropy {
        were_type: String,
    },
    CannibalismPenalty {
        alignment_loss: i32,
    },
    NoEffect,
}

/// [v2.33.0 97-1] 시체 섭취
/// 원본: eat.c eatcorpse()
pub fn eat_corpse(
    corpse: &CorpseType,
    monster_level: i32,
    has_poison_resist: bool,
    has_stone_resist: bool,
    is_same_race: bool,
    turns_old: i32,
    rng: &mut NetHackRng,
) -> CorpseEffect {
    // 상한 시체
    if turns_old > 50 && *corpse == CorpseType::Normal {
        if rng.rn2(5) == 0 {
            return CorpseEffect::Disease {
                disease: "식중독".to_string(),
            };
        }
    }

    // 동족 섭취
    if is_same_race {
        return CorpseEffect::CannibalismPenalty { alignment_loss: 15 };
    }

    match corpse {
        CorpseType::Normal => CorpseEffect::Nutrition {
            amount: monster_level * 10 + 50,
        },
        CorpseType::Poisonous => {
            if has_poison_resist {
                CorpseEffect::Nutrition {
                    amount: monster_level * 8 + 30,
                }
            } else {
                CorpseEffect::Poison {
                    damage: rng.rn2(15) + 5,
                    stat_loss: if rng.rn2(3) == 0 {
                        Some(("STR".to_string(), rng.rn2(3) + 1))
                    } else {
                        None
                    },
                }
            }
        }
        CorpseType::Acidic => CorpseEffect::Acid {
            damage: rng.rn2(10) + 3,
        },
        CorpseType::Petrifying => {
            if has_stone_resist {
                CorpseEffect::Nutrition { amount: 60 }
            } else {
                CorpseEffect::Petrifying {
                    turns_until_stone: 5,
                }
            }
        }
        CorpseType::Tainted => CorpseEffect::Disease {
            disease: "터미널 질병".to_string(),
        },
        CorpseType::Polymorph => CorpseEffect::Polymorph,
        CorpseType::Teleport => CorpseEffect::Teleportitis,
        CorpseType::Invisible => CorpseEffect::Invisibility {
            turns: rng.rn2(100) + 50,
        },
        CorpseType::Lycanthrope => CorpseEffect::Lycanthropy {
            were_type: "늑대인간".to_string(),
        },
        CorpseType::Giant => CorpseEffect::GainStrength {
            amount: rng.rn2(2) + 1,
        },
        CorpseType::Dragon => {
            let resists = ["화염", "냉기", "독", "전기", "수면", "마법 해제"];
            let idx = rng.rn2(resists.len() as i32) as usize;
            CorpseEffect::GainResistance {
                resistance: resists[idx].to_string(),
            }
        }
        CorpseType::Cannibalism => CorpseEffect::CannibalismPenalty { alignment_loss: 20 },
    }
}

// =============================================================================
// [2] 음식 속성 — food_properties (eat.c L1500-2000)
// =============================================================================

/// [v2.33.0 97-1] 음식 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoodType {
    Ration,
    Bread,
    Fruit,
    Meat,
    Egg,
    Candy,
    Fortune,
    Tin,
    Corpse,
    TripeRation,
    Lembas,
    Mimic,
}

/// [v2.33.0 97-1] 음식 영양 계산
pub fn food_nutrition(food: &FoodType) -> i32 {
    match food {
        FoodType::Ration => 800,
        FoodType::Bread => 200,
        FoodType::Fruit => 250,
        FoodType::Meat => 400,
        FoodType::Egg => 80,
        FoodType::Candy => 100,
        FoodType::Fortune => 40,
        FoodType::Tin => 300,
        FoodType::Corpse => 100, // 기본값, 실제로는 몬스터 레벨 기반
        FoodType::TripeRation => 200,
        FoodType::Lembas => 800,
        FoodType::Mimic => 0,
    }
}

/// [v2.33.0 97-1] 음식 섭취 턴 수
pub fn eating_turns(food: &FoodType) -> i32 {
    match food {
        FoodType::Ration | FoodType::Lembas => 5,
        FoodType::Corpse | FoodType::Meat => 3,
        FoodType::Tin => 6,
        FoodType::TripeRation => 2,
        _ => 1,
    }
}

// =============================================================================
// [3] 채취/쿡킹 — cooking (eat.c L2000-2500)
// =============================================================================

/// [v2.33.0 97-1] 조리 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CookingResult {
    Cooked {
        nutrition_bonus: i32,
        removes_poison: bool,
    },
    Burnt {
        nutrition_loss: i32,
    },
    Destroyed,
}

/// [v2.33.0 97-1] 시체 조리
pub fn cook_corpse(
    is_poisonous: bool,
    fire_source: &str, // "화염 함정", "지팡이", "드래곤 브레스"
    rng: &mut NetHackRng,
) -> CookingResult {
    let intensity = match fire_source {
        "드래곤 브레스" => 3,
        "화염 함정" => 2,
        _ => 1,
    };

    if intensity >= 3 && rng.rn2(3) == 0 {
        return CookingResult::Destroyed;
    }

    if intensity >= 2 && rng.rn2(4) == 0 {
        return CookingResult::Burnt { nutrition_loss: 50 };
    }

    CookingResult::Cooked {
        nutrition_bonus: 50,
        removes_poison: is_poisonous,
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
    fn test_normal_corpse() {
        let mut rng = test_rng();
        let result = eat_corpse(&CorpseType::Normal, 10, false, false, false, 5, &mut rng);
        assert!(matches!(result, CorpseEffect::Nutrition { .. }));
    }

    #[test]
    fn test_poison_resist() {
        let mut rng = test_rng();
        let result = eat_corpse(&CorpseType::Poisonous, 10, true, false, false, 5, &mut rng);
        assert!(matches!(result, CorpseEffect::Nutrition { .. }));
    }

    #[test]
    fn test_petrifying() {
        let mut rng = test_rng();
        let result = eat_corpse(&CorpseType::Petrifying, 5, false, false, false, 5, &mut rng);
        assert!(matches!(result, CorpseEffect::Petrifying { .. }));
    }

    #[test]
    fn test_dragon_resistance() {
        let mut rng = test_rng();
        let result = eat_corpse(&CorpseType::Dragon, 20, false, false, false, 5, &mut rng);
        assert!(matches!(result, CorpseEffect::GainResistance { .. }));
    }

    #[test]
    fn test_cannibalism() {
        let mut rng = test_rng();
        let result = eat_corpse(&CorpseType::Normal, 10, false, false, true, 5, &mut rng);
        assert!(matches!(result, CorpseEffect::CannibalismPenalty { .. }));
    }

    #[test]
    fn test_food_nutrition() {
        assert_eq!(food_nutrition(&FoodType::Ration), 800);
        assert_eq!(food_nutrition(&FoodType::Lembas), 800);
    }

    #[test]
    fn test_cooking() {
        let mut rng = test_rng();
        let result = cook_corpse(true, "지팡이", &mut rng);
        assert!(matches!(result, CookingResult::Cooked { .. }));
    }

    #[test]
    fn test_eating_turns() {
        assert_eq!(eating_turns(&FoodType::Ration), 5);
        assert_eq!(eating_turns(&FoodType::Candy), 1);
    }
}

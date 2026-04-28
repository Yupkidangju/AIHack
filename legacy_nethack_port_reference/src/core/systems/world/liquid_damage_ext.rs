// ============================================================================
// [v2.24.0 Phase 50-3] 액체 피해 / 환경 파괴 (liquid_damage_ext.rs)
// 원본: NetHack 3.6.7 src/dig.c L1618-1760 — 용암/물에 의한 아이템 파괴
// 순수 결과 패턴: ECS 의존 없이 독립 테스트 가능
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 아이템 재질
// =============================================================================

/// [v2.24.0 50-3] 아이템 재질 (파괴 내성 판정용)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemMaterial {
    /// 금속 — 용암에 녹을 수 있음, 물에는 안전
    Metal,
    /// 나무 — 용암에 탐, 물에 부패
    Wood,
    /// 가죽 — 용암에 탐, 물에 부패
    Leather,
    /// 천 — 용암에 탐, 물에 젖음
    Cloth,
    /// 유리 — 용암에 녹음, 물에 안전
    Glass,
    /// 돌/보석 — 용암에 안전, 물에 안전
    Stone,
    /// 종이 — 용암에 탐, 물에 파괴
    Paper,
    /// 액체 (포션) — 용암에 증발, 물에 희석
    Liquid,
    /// 식품 — 용암에 탐, 물에 부패
    Food,
    /// 뼈 — 용암에 탐, 물에 안전
    Bone,
    /// 용 비늘 — 내화성
    DragonHide,
    /// 마법 — 내구도 높음
    Magical,
}

// =============================================================================
// [2] 용암 피해 — lava_damage (dig.c L1618-1688)
// =============================================================================

/// [v2.24.0 50-3] 용암 피해 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LavaDamageResult {
    /// 아이템 파괴됨
    Destroyed { message: String },
    /// 아이템이 내화성으로 생존
    Survived { message: String },
    /// 아이템이 열에 의해 손상 (enchantment 감소 등)
    Damaged {
        erosion_increase: i32,
        message: String,
    },
}

/// [v2.24.0 50-3] 용암 아이템 파괴 판정
/// 원본: dig.c lava_damage() L1618-1688
pub fn lava_damage_result(
    material: ItemMaterial,
    is_fireproof: bool,
    is_artifact: bool,
    enchantment: i32,
    rng: &mut NetHackRng,
) -> LavaDamageResult {
    // [1] 아티팩트는 절대 파괴되지 않음
    if is_artifact {
        return LavaDamageResult::Survived {
            message: "아티팩트가 용암의 열기를 견뎌냈다!".to_string(),
        };
    }

    // [2] 방화 아이템은 생존 — 원본 dig.c L1632
    if is_fireproof {
        return LavaDamageResult::Survived {
            message: "방화 처리된 아이템이 용암에서 생존했다.".to_string(),
        };
    }

    // [3] 재질별 판정
    match material {
        // 내화성 재질
        ItemMaterial::Stone | ItemMaterial::DragonHide | ItemMaterial::Magical => {
            LavaDamageResult::Survived {
                message: "이 재질은 용암에 견딜 수 있다.".to_string(),
            }
        }

        // 금속 — 높은 인챈트는 생존, 낮으면 녹음
        // 원본 dig.c L1645-1660
        ItemMaterial::Metal => {
            if enchantment >= 3 || rng.rn2(4) == 0 {
                LavaDamageResult::Damaged {
                    erosion_increase: 1,
                    message: "금속이 용암의 열에 약간 손상되었다.".to_string(),
                }
            } else {
                LavaDamageResult::Destroyed {
                    message: "금속 아이템이 용암에 녹아내렸다!".to_string(),
                }
            }
        }

        // 유리 — 반드시 녹음
        ItemMaterial::Glass => LavaDamageResult::Destroyed {
            message: "유리 아이템이 용암에 녹아내렸다!".to_string(),
        },

        // 가연성 재질 — 확실한 파괴
        ItemMaterial::Wood
        | ItemMaterial::Leather
        | ItemMaterial::Cloth
        | ItemMaterial::Paper
        | ItemMaterial::Food
        | ItemMaterial::Bone => LavaDamageResult::Destroyed {
            message: "아이템이 용암에 타버렸다!".to_string(),
        },

        // 액체 — 증발
        ItemMaterial::Liquid => LavaDamageResult::Destroyed {
            message: "포션이 용암의 열에 증발했다!".to_string(),
        },
    }
}

// =============================================================================
// [3] 물 피해 — water_damage (dig.c L1693-1760)
// =============================================================================

/// [v2.24.0 50-3] 물 피해 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaterDamageResult {
    /// 아이템 파괴됨
    Destroyed { message: String },
    /// 아이템 생존 (방수 또는 내수성)
    Survived { message: String },
    /// 아이템 손상 (침식 등)
    Damaged {
        erosion_increase: i32,
        message: String,
    },
    /// 포션이 물에 희석됨
    Diluted { message: String },
}

/// [v2.24.0 50-3] 물 아이템 파괴 판정
/// 원본: dig.c water_damage() L1693-1760
pub fn water_damage_result(
    material: ItemMaterial,
    is_waterproof: bool,
    is_artifact: bool,
    already_diluted: bool,
    rng: &mut NetHackRng,
) -> WaterDamageResult {
    // [1] 아티팩트는 절대 파괴되지 않음
    if is_artifact {
        return WaterDamageResult::Survived {
            message: "아티팩트가 물에 젖지 않았다!".to_string(),
        };
    }

    // [2] 방수 아이템은 생존
    if is_waterproof {
        return WaterDamageResult::Survived {
            message: "방수 처리된 아이템이 물에서 생존했다.".to_string(),
        };
    }

    // [3] 재질별 판정
    match material {
        // 내수성 재질
        ItemMaterial::Stone
        | ItemMaterial::Metal
        | ItemMaterial::Glass
        | ItemMaterial::DragonHide
        | ItemMaterial::Bone
        | ItemMaterial::Magical => WaterDamageResult::Survived {
            message: "이 재질은 물에 영향을 받지 않는다.".to_string(),
        },

        // 종이 (두루마리, 마법서) — 파괴 확률 높음
        // 원본: dig.c L1710-1725
        ItemMaterial::Paper => {
            if rng.rn2(3) == 0 {
                WaterDamageResult::Survived {
                    message: "두루마리가 간신히 물을 피했다.".to_string(),
                }
            } else {
                WaterDamageResult::Destroyed {
                    message: "두루마리가 물에 젖어 글자가 지워졌다!".to_string(),
                }
            }
        }

        // 액체 (포션) — 희석
        // 원본: dig.c L1728-1745
        ItemMaterial::Liquid => {
            if already_diluted {
                // 이미 희석된 포션은 물이 됨 (파괴)
                WaterDamageResult::Destroyed {
                    message: "희석된 포션이 완전히 물이 되었다!".to_string(),
                }
            } else {
                WaterDamageResult::Diluted {
                    message: "포션이 물에 희석되었다.".to_string(),
                }
            }
        }

        // 가죽/천 — 침식
        ItemMaterial::Leather | ItemMaterial::Cloth => {
            if rng.rn2(2) == 0 {
                WaterDamageResult::Damaged {
                    erosion_increase: 1,
                    message: "아이템이 물에 젖어 손상되었다.".to_string(),
                }
            } else {
                WaterDamageResult::Survived {
                    message: "아이템이 약간 젖었지만 괜찮다.".to_string(),
                }
            }
        }

        // 나무 — 겹침 손상
        ItemMaterial::Wood => WaterDamageResult::Damaged {
            erosion_increase: 1,
            message: "나무 아이템이 물에 젖어 부풀었다.".to_string(),
        },

        // 식품 — 부패 (파괴)
        ItemMaterial::Food => {
            if rng.rn2(5) == 0 {
                WaterDamageResult::Survived {
                    message: "식품이 물에 살짝 젖었다.".to_string(),
                }
            } else {
                WaterDamageResult::Destroyed {
                    message: "식품이 물에 젖어 부패했다!".to_string(),
                }
            }
        }
    }
}

// =============================================================================
// [4] 환경 아이템 파괴 통합 인터페이스
// =============================================================================

/// [v2.24.0 50-3] 환경 피해 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentType {
    Lava,
    Water,
    Acid,
}

/// [v2.24.0 50-3] 환경 피해 통합 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvDamageResult {
    Destroyed { message: String },
    Survived { message: String },
    Damaged { erosion: i32, message: String },
    Diluted { message: String },
}

/// [v2.24.0 50-3] 환경에 의한 아이템 파괴 통합
pub fn env_destroy_item(
    env: EnvironmentType,
    material: ItemMaterial,
    is_proofed: bool,
    is_artifact: bool,
    enchantment: i32,
    already_diluted: bool,
    rng: &mut NetHackRng,
) -> EnvDamageResult {
    match env {
        EnvironmentType::Lava => {
            let result = lava_damage_result(material, is_proofed, is_artifact, enchantment, rng);
            match result {
                LavaDamageResult::Destroyed { message } => EnvDamageResult::Destroyed { message },
                LavaDamageResult::Survived { message } => EnvDamageResult::Survived { message },
                LavaDamageResult::Damaged {
                    erosion_increase,
                    message,
                } => EnvDamageResult::Damaged {
                    erosion: erosion_increase,
                    message,
                },
            }
        }
        EnvironmentType::Water => {
            let result =
                water_damage_result(material, is_proofed, is_artifact, already_diluted, rng);
            match result {
                WaterDamageResult::Destroyed { message } => EnvDamageResult::Destroyed { message },
                WaterDamageResult::Survived { message } => EnvDamageResult::Survived { message },
                WaterDamageResult::Damaged {
                    erosion_increase,
                    message,
                } => EnvDamageResult::Damaged {
                    erosion: erosion_increase,
                    message,
                },
                WaterDamageResult::Diluted { message } => EnvDamageResult::Diluted { message },
            }
        }
        EnvironmentType::Acid => {
            // 산성 피해 — 모든 재질에 침식, 아티팩트만 예외
            if is_artifact {
                return EnvDamageResult::Survived {
                    message: "아티팩트가 산성을 견뎌냈다!".to_string(),
                };
            }

            match material {
                ItemMaterial::Stone | ItemMaterial::Glass | ItemMaterial::DragonHide => {
                    EnvDamageResult::Survived {
                        message: "이 재질은 산에 견딜 수 있다.".to_string(),
                    }
                }
                ItemMaterial::Metal => {
                    if rng.rn2(3) == 0 {
                        EnvDamageResult::Destroyed {
                            message: "금속이 산에 녹아내렸다!".to_string(),
                        }
                    } else {
                        EnvDamageResult::Damaged {
                            erosion: 2,
                            message: "금속이 산에 부식되었다.".to_string(),
                        }
                    }
                }
                _ => EnvDamageResult::Destroyed {
                    message: "아이템이 산에 녹아내렸다!".to_string(),
                },
            }
        }
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

    // --- lava_damage_result ---

    #[test]
    fn test_lava_artifact_survives() {
        let mut rng = test_rng();
        let result = lava_damage_result(ItemMaterial::Wood, false, true, 0, &mut rng);
        assert!(matches!(result, LavaDamageResult::Survived { .. }));
    }

    #[test]
    fn test_lava_fireproof_survives() {
        let mut rng = test_rng();
        let result = lava_damage_result(ItemMaterial::Paper, true, false, 0, &mut rng);
        assert!(matches!(result, LavaDamageResult::Survived { .. }));
    }

    #[test]
    fn test_lava_stone_survives() {
        let mut rng = test_rng();
        let result = lava_damage_result(ItemMaterial::Stone, false, false, 0, &mut rng);
        assert!(matches!(result, LavaDamageResult::Survived { .. }));
    }

    #[test]
    fn test_lava_paper_destroyed() {
        let mut rng = test_rng();
        let result = lava_damage_result(ItemMaterial::Paper, false, false, 0, &mut rng);
        assert!(matches!(result, LavaDamageResult::Destroyed { .. }));
    }

    #[test]
    fn test_lava_glass_destroyed() {
        let mut rng = test_rng();
        let result = lava_damage_result(ItemMaterial::Glass, false, false, 0, &mut rng);
        assert!(matches!(result, LavaDamageResult::Destroyed { .. }));
    }

    #[test]
    fn test_lava_metal_high_ench_survives() {
        let mut rng = test_rng();
        let result = lava_damage_result(ItemMaterial::Metal, false, false, 5, &mut rng);
        // 인챈트 >= 3 이면 손상만 (파괴 안 됨)
        assert!(matches!(result, LavaDamageResult::Damaged { .. }));
    }

    // --- water_damage_result ---

    #[test]
    fn test_water_artifact_survives() {
        let mut rng = test_rng();
        let result = water_damage_result(ItemMaterial::Paper, false, true, false, &mut rng);
        assert!(matches!(result, WaterDamageResult::Survived { .. }));
    }

    #[test]
    fn test_water_metal_survives() {
        let mut rng = test_rng();
        let result = water_damage_result(ItemMaterial::Metal, false, false, false, &mut rng);
        assert!(matches!(result, WaterDamageResult::Survived { .. }));
    }

    #[test]
    fn test_water_potion_diluted() {
        let mut rng = test_rng();
        let result = water_damage_result(ItemMaterial::Liquid, false, false, false, &mut rng);
        assert!(matches!(result, WaterDamageResult::Diluted { .. }));
    }

    #[test]
    fn test_water_diluted_potion_destroyed() {
        let mut rng = test_rng();
        let result = water_damage_result(ItemMaterial::Liquid, false, false, true, &mut rng);
        assert!(matches!(result, WaterDamageResult::Destroyed { .. }));
    }

    // --- env_destroy_item ---

    #[test]
    fn test_env_acid_metal() {
        let mut rng = test_rng();
        let result = env_destroy_item(
            EnvironmentType::Acid,
            ItemMaterial::Metal,
            false,
            false,
            0,
            false,
            &mut rng,
        );
        // 산성 + 금속 → 파괴 또는 손상
        assert!(
            matches!(result, EnvDamageResult::Destroyed { .. })
                || matches!(result, EnvDamageResult::Damaged { .. })
        );
    }

    #[test]
    fn test_env_acid_stone_survives() {
        let mut rng = test_rng();
        let result = env_destroy_item(
            EnvironmentType::Acid,
            ItemMaterial::Stone,
            false,
            false,
            0,
            false,
            &mut rng,
        );
        assert!(matches!(result, EnvDamageResult::Survived { .. }));
    }

    #[test]
    fn test_env_lava_via_unified() {
        let mut rng = test_rng();
        let result = env_destroy_item(
            EnvironmentType::Lava,
            ItemMaterial::Paper,
            false,
            false,
            0,
            false,
            &mut rng,
        );
        assert!(matches!(result, EnvDamageResult::Destroyed { .. }));
    }

    #[test]
    fn test_env_water_potion_dilute() {
        let mut rng = test_rng();
        let result = env_destroy_item(
            EnvironmentType::Water,
            ItemMaterial::Liquid,
            false,
            false,
            0,
            false,
            &mut rng,
        );
        assert!(matches!(result, EnvDamageResult::Diluted { .. }));
    }
}

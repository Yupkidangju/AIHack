// ============================================================================
// [v2.34.0 Phase 98-1] 파괴/교란 확장 (destroy_phase98_ext.rs)
// 원본: NetHack 3.6.7 src/uhitm.c + zap.c 아이템/지형 파괴 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 아이템 파괴 — item_destroy (uhitm.c/zap.c 파괴 분기)
// =============================================================================

/// [v2.34.0 98-1] 파괴 원소
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestroyElement {
    Fire,
    Cold,
    Lightning,
    Acid,
    Petrification,
    Disintegration,
    Cancellation,
}

/// [v2.34.0 98-1] 아이템 카테고리 (파괴 대상)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestroyCategory {
    Potion,
    Scroll,
    Spellbook,
    Wand,
    Ring,
    Amulet,
    Food,
    Armor,
    Weapon,
    Tool,
    Other,
}

/// [v2.34.0 98-1] 파괴 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestroyResult {
    Destroyed { item_name: String, count: i32 },
    Damaged { item_name: String, erosion: i32 },
    Resistant { reason: String },
    Exploded { item_name: String, damage: i32 },
    Frozen { item_name: String },
    Melted { item_name: String },
    NoEffect,
}

/// [v2.34.0 98-1] 원소에 의한 아이템 파괴
pub fn destroy_item_by_element(
    category: DestroyCategory,
    item_name: &str,
    element: DestroyElement,
    quantity: i32,
    is_fireproof: bool,
    rng: &mut NetHackRng,
) -> DestroyResult {
    // 방어 속성 체크
    if is_fireproof && element == DestroyElement::Fire {
        return DestroyResult::Resistant {
            reason: "방화 처리되어 있다.".to_string(),
        };
    }

    match (element, category) {
        // 화염: 포션/스크롤/마법서/음식 파괴
        (DestroyElement::Fire, DestroyCategory::Scroll | DestroyCategory::Spellbook) => {
            let destroyed = rng.rn2(quantity) + 1;
            DestroyResult::Destroyed {
                item_name: item_name.to_string(),
                count: destroyed.min(quantity),
            }
        }
        (DestroyElement::Fire, DestroyCategory::Potion) => {
            if rng.rn2(3) == 0 {
                DestroyResult::Exploded {
                    item_name: item_name.to_string(),
                    damage: rng.rn2(6) + 2,
                }
            } else {
                let destroyed = rng.rn2(quantity) + 1;
                DestroyResult::Destroyed {
                    item_name: item_name.to_string(),
                    count: destroyed.min(quantity),
                }
            }
        }
        (DestroyElement::Fire, DestroyCategory::Food) => DestroyResult::Destroyed {
            item_name: item_name.to_string(),
            count: quantity,
        },
        // 냉기: 포션 동결
        (DestroyElement::Cold, DestroyCategory::Potion) => {
            if rng.rn2(3) == 0 {
                DestroyResult::Exploded {
                    item_name: item_name.to_string(),
                    damage: rng.rn2(4) + 1,
                }
            } else {
                DestroyResult::Frozen {
                    item_name: item_name.to_string(),
                }
            }
        }
        // 번개: 지팡이/반지 과충전
        (DestroyElement::Lightning, DestroyCategory::Wand) => {
            if rng.rn2(4) == 0 {
                DestroyResult::Exploded {
                    item_name: item_name.to_string(),
                    damage: rng.rn2(10) + 5,
                }
            } else {
                DestroyResult::NoEffect
            }
        }
        (DestroyElement::Lightning, DestroyCategory::Ring) => DestroyResult::Damaged {
            item_name: item_name.to_string(),
            erosion: 1,
        },
        // 산: 갑옷/무기 부식
        (DestroyElement::Acid, DestroyCategory::Armor | DestroyCategory::Weapon) => {
            DestroyResult::Damaged {
                item_name: item_name.to_string(),
                erosion: rng.rn2(2) + 1,
            }
        }
        // 석화: 전부
        (DestroyElement::Petrification, _) => DestroyResult::Destroyed {
            item_name: item_name.to_string(),
            count: quantity,
        },
        // 소멸: 전부
        (DestroyElement::Disintegration, _) => DestroyResult::Destroyed {
            item_name: item_name.to_string(),
            count: quantity,
        },
        // 취소: 마법 제거
        (
            DestroyElement::Cancellation,
            DestroyCategory::Wand | DestroyCategory::Ring | DestroyCategory::Amulet,
        ) => {
            DestroyResult::Damaged {
                item_name: item_name.to_string(),
                erosion: 0, // 마법 속성 제거
            }
        }
        _ => DestroyResult::NoEffect,
    }
}

// =============================================================================
// [2] 지형 파괴 — terrain_destroy
// =============================================================================

/// [v2.34.0 98-1] 지형 파괴 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainDestroyResult {
    WallDestroyed { x: i32, y: i32 },
    FloorDestroyed { x: i32, y: i32, creates_pit: bool },
    DoorDestroyed { x: i32, y: i32 },
    TreeBurnt { x: i32, y: i32 },
    WaterFrozen { x: i32, y: i32 },
    LavaHardened { x: i32, y: i32 },
    IceMelted { x: i32, y: i32 },
    NoEffect,
}

/// [v2.34.0 98-1] 지형 파괴
pub fn destroy_terrain(
    terrain: &str,
    element: DestroyElement,
    power: i32,
    x: i32,
    y: i32,
    rng: &mut NetHackRng,
) -> TerrainDestroyResult {
    match (terrain, element) {
        ("벽", DestroyElement::Disintegration) => TerrainDestroyResult::WallDestroyed { x, y },
        ("벽", _) if power > 20 => {
            if rng.rn2(3) == 0 {
                TerrainDestroyResult::WallDestroyed { x, y }
            } else {
                TerrainDestroyResult::NoEffect
            }
        }
        ("문", DestroyElement::Fire) => TerrainDestroyResult::DoorDestroyed { x, y },
        ("문", DestroyElement::Disintegration) => TerrainDestroyResult::DoorDestroyed { x, y },
        ("나무", DestroyElement::Fire) => TerrainDestroyResult::TreeBurnt { x, y },
        ("물", DestroyElement::Cold) => TerrainDestroyResult::WaterFrozen { x, y },
        ("용암", DestroyElement::Cold) => TerrainDestroyResult::LavaHardened { x, y },
        ("얼음", DestroyElement::Fire) => TerrainDestroyResult::IceMelted { x, y },
        ("바닥", DestroyElement::Disintegration) => TerrainDestroyResult::FloorDestroyed {
            x,
            y,
            creates_pit: true,
        },
        _ => TerrainDestroyResult::NoEffect,
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
    fn test_fire_scroll() {
        let mut rng = test_rng();
        let result = destroy_item_by_element(
            DestroyCategory::Scroll,
            "식별 스크롤",
            DestroyElement::Fire,
            3,
            false,
            &mut rng,
        );
        assert!(matches!(result, DestroyResult::Destroyed { .. }));
    }

    #[test]
    fn test_fireproof() {
        let mut rng = test_rng();
        let result = destroy_item_by_element(
            DestroyCategory::Scroll,
            "스크롤",
            DestroyElement::Fire,
            1,
            true,
            &mut rng,
        );
        assert!(matches!(result, DestroyResult::Resistant { .. }));
    }

    #[test]
    fn test_cold_potion() {
        let mut rng = test_rng();
        let result = destroy_item_by_element(
            DestroyCategory::Potion,
            "치유 포션",
            DestroyElement::Cold,
            1,
            false,
            &mut rng,
        );
        assert!(matches!(
            result,
            DestroyResult::Frozen { .. } | DestroyResult::Exploded { .. }
        ));
    }

    #[test]
    fn test_disintegration() {
        let mut rng = test_rng();
        let result = destroy_item_by_element(
            DestroyCategory::Armor,
            "갑옷",
            DestroyElement::Disintegration,
            1,
            false,
            &mut rng,
        );
        assert!(matches!(result, DestroyResult::Destroyed { .. }));
    }

    #[test]
    fn test_acid_corrosion() {
        let mut rng = test_rng();
        let result = destroy_item_by_element(
            DestroyCategory::Armor,
            "갑옷",
            DestroyElement::Acid,
            1,
            false,
            &mut rng,
        );
        assert!(matches!(result, DestroyResult::Damaged { .. }));
    }

    #[test]
    fn test_terrain_fire_door() {
        let mut rng = test_rng();
        let result = destroy_terrain("문", DestroyElement::Fire, 10, 5, 5, &mut rng);
        assert!(matches!(result, TerrainDestroyResult::DoorDestroyed { .. }));
    }

    #[test]
    fn test_terrain_freeze_water() {
        let mut rng = test_rng();
        let result = destroy_terrain("물", DestroyElement::Cold, 10, 5, 5, &mut rng);
        assert!(matches!(result, TerrainDestroyResult::WaterFrozen { .. }));
    }

    #[test]
    fn test_no_effect() {
        let mut rng = test_rng();
        let result = destroy_item_by_element(
            DestroyCategory::Other,
            "돌",
            DestroyElement::Fire,
            1,
            false,
            &mut rng,
        );
        assert!(matches!(result, DestroyResult::NoEffect));
    }
}

// ============================================================================
// [v2.37.0 Phase 101-3] 지형/환경 통합 (environ_phase101_ext.rs)
// 원본: NetHack 3.6.7 전반 환경 상호작용 통합 (fountain, sink, throne, grave)
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 지형 상호작용 유형 — terrain_interact
// =============================================================================

/// [v2.37.0 101-3] 지형 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainFeature {
    Fountain,
    Sink,
    Throne,
    Grave,
    Altar,
    Tree,
    Pool,
    Lava,
    Ice,
    Cloud,
    Air,
}

/// [v2.37.0 101-3] 상호작용 결과
#[derive(Debug, Clone)]
pub struct TerrainInteraction {
    pub feature: TerrainFeature,
    pub action: String,
    pub outcome: String,
    pub hp_change: i32,
    pub items_generated: Vec<String>,
    pub monster_spawned: Option<String>,
    pub feature_destroyed: bool,
}

/// [v2.37.0 101-3] 분수 상호작용
pub fn interact_fountain(action: &str, rng: &mut NetHackRng) -> TerrainInteraction {
    let roll = rng.rn2(30);
    let (outcome, hp, items, monster, destroyed) = match action {
        "마시기" => match roll {
            0..=5 => ("시원한 물을 마셨다.".to_string(), 1, vec![], None, false),
            6..=8 => ("독이 든 물이었다!".to_string(), -5, vec![], None, false),
            9..=11 => ("물이 마법처럼 빛난다!".to_string(), 5, vec![], None, false),
            12..=14 => (
                "바닥에서 동전이 올라온다!".to_string(),
                0,
                vec!["금화".to_string()],
                None,
                false,
            ),
            15..=17 => (
                "물의 요정이 나타났다!".to_string(),
                0,
                vec![],
                Some("물의 요정".to_string()),
                false,
            ),
            18..=20 => (
                "갑자기 소원이 생겼다!".to_string(),
                0,
                vec!["소원".to_string()],
                None,
                true,
            ),
            _ => (
                "아무 일도 일어나지 않았다.".to_string(),
                0,
                vec![],
                None,
                false,
            ),
        },
        "손 씻기" => match roll {
            0..=10 => (
                "반지가 빠졌다!".to_string(),
                0,
                vec!["반지".to_string()],
                None,
                false,
            ),
            _ => ("손을 깨끗이 씻었다.".to_string(), 0, vec![], None, false),
        },
        _ => ("분수를 바라보았다.".to_string(), 0, vec![], None, false),
    };

    TerrainInteraction {
        feature: TerrainFeature::Fountain,
        action: action.to_string(),
        outcome,
        hp_change: hp,
        items_generated: items,
        monster_spawned: monster,
        feature_destroyed: destroyed,
    }
}

/// [v2.37.0 101-3] 왕좌 상호작용
pub fn interact_throne(rng: &mut NetHackRng) -> TerrainInteraction {
    let roll = rng.rn2(20);
    let (outcome, hp, items) = match roll {
        0..=3 => ("왕좌에 앉았지만 아무 일도 없다.".to_string(), 0, vec![]),
        4..=6 => (
            "보석이 굴러 나왔다!".to_string(),
            0,
            vec!["보석".to_string()],
        ),
        7..=9 => ("왕의 유령이 나타나 찬사를 보낸다!".to_string(), 0, vec![]),
        10..=12 => ("전기 충격을 받았다!".to_string(), -10, vec![]),
        13..=15 => ("왕좌가 텔레포트를 부여한다!".to_string(), 0, vec![]),
        16..=18 => ("왕관이 나타났다!".to_string(), 0, vec!["왕관".to_string()]),
        _ => ("왕좌가 부서졌다!".to_string(), 0, vec![]),
    };

    TerrainInteraction {
        feature: TerrainFeature::Throne,
        action: "앉기".to_string(),
        outcome,
        hp_change: hp,
        items_generated: items,
        monster_spawned: None,
        feature_destroyed: roll >= 19,
    }
}

/// [v2.37.0 101-3] 무덤 상호작용
pub fn interact_grave(rng: &mut NetHackRng) -> TerrainInteraction {
    let roll = rng.rn2(10);
    let (outcome, monster, items) = match roll {
        0..=2 => (
            "무덤을 파헤치자 보물이 나왔다!".to_string(),
            None,
            vec!["보물 상자".to_string()],
        ),
        3..=5 => (
            "좀비가 깨어났다!".to_string(),
            Some("좀비".to_string()),
            vec![],
        ),
        6..=7 => (
            "유령이 나타났다!".to_string(),
            Some("유령".to_string()),
            vec![],
        ),
        _ => ("빈 무덤이었다.".to_string(), None, vec![]),
    };

    TerrainInteraction {
        feature: TerrainFeature::Grave,
        action: "파기".to_string(),
        outcome,
        hp_change: 0,
        items_generated: items,
        monster_spawned: monster,
        feature_destroyed: true,
    }
}

/// [v2.37.0 101-3] 환경 피해 계산
pub fn environmental_damage(
    terrain: TerrainFeature,
    has_resistance: bool,
    has_levitation: bool,
) -> (i32, String) {
    match terrain {
        TerrainFeature::Lava => {
            if has_levitation {
                (0, "용암 위를 떠다닌다.".to_string())
            } else if has_resistance {
                (5, "용암이 뜨겁지만 견딜 만하다.".to_string())
            } else {
                (100, "용암에 빠졌다! 치명적!".to_string())
            }
        }
        TerrainFeature::Pool => {
            if has_levitation {
                (0, "물 위를 떠다닌다.".to_string())
            } else {
                (0, "헤엄치고 있다.".to_string())
            }
        }
        TerrainFeature::Ice => {
            if has_resistance {
                (0, "얼음 위를 안전하게 걷는다.".to_string())
            } else {
                (2, "미끄러졌다!".to_string())
            }
        }
        _ => (0, "안전한 지형이다.".to_string()),
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
    fn test_fountain_drink() {
        let mut rng = test_rng();
        let result = interact_fountain("마시기", &mut rng);
        assert_eq!(result.feature, TerrainFeature::Fountain);
        assert!(!result.outcome.is_empty());
    }

    #[test]
    fn test_fountain_wash() {
        let mut rng = test_rng();
        let result = interact_fountain("손 씻기", &mut rng);
        assert!(result.outcome.len() > 0);
    }

    #[test]
    fn test_throne() {
        let mut rng = test_rng();
        let result = interact_throne(&mut rng);
        assert_eq!(result.feature, TerrainFeature::Throne);
    }

    #[test]
    fn test_grave() {
        let mut rng = test_rng();
        let result = interact_grave(&mut rng);
        assert!(result.feature_destroyed);
    }

    #[test]
    fn test_lava_deadly() {
        let (dmg, _) = environmental_damage(TerrainFeature::Lava, false, false);
        assert_eq!(dmg, 100);
    }

    #[test]
    fn test_lava_resist() {
        let (dmg, _) = environmental_damage(TerrainFeature::Lava, true, false);
        assert!(dmg < 100);
    }

    #[test]
    fn test_lava_levitate() {
        let (dmg, _) = environmental_damage(TerrainFeature::Lava, false, true);
        assert_eq!(dmg, 0);
    }

    #[test]
    fn test_ice_slip() {
        let (dmg, msg) = environmental_damage(TerrainFeature::Ice, false, false);
        assert!(dmg > 0);
        assert!(msg.contains("미끄러"));
    }
}

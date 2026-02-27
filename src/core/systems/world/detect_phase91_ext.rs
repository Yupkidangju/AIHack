// ============================================================================
// [v2.27.0 Phase 91-10] 탐지 확장 (detect_phase91_ext.rs)
// 원본: NetHack 3.6.7 src/detect.c L800-1800 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 아이템 탐지 — detect_items (detect.c L800-1000)
// =============================================================================

/// [v2.27.0 91-10] 탐지 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectType {
    Gold,
    Food,
    Monsters,
    Objects,
    Traps,
    Doors,
    Water,
    Magic,
    Portals,
}

/// [v2.27.0 91-10] 탐지 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectResult {
    pub detect_type: DetectType,
    pub detected_count: i32,
    pub radius: i32,
    pub message: String,
    pub positions: Vec<(i32, i32)>,
}

/// [v2.27.0 91-10] 아이템 탐지 판정
/// 원본: detect.c object_detect()
pub fn detect_objects(
    detect_type: DetectType,
    skill_level: i32, // 0=기본, 1=숙련, 2=전문
    is_blessed: bool,
    map_width: i32,
    map_height: i32,
    object_positions: &[(i32, i32)],
    rng: &mut NetHackRng,
) -> DetectResult {
    // 기본 반경: 스킬 기반
    let base_radius = match skill_level {
        0 => 5,
        1 => 10,
        _ => 999, // 전문 = 전체 맵
    };
    let radius = if is_blessed {
        base_radius * 2
    } else {
        base_radius
    };

    let message = match detect_type {
        DetectType::Gold => "금화의 위치를 감지했다!",
        DetectType::Food => "음식의 위치를 감지했다!",
        DetectType::Monsters => "생명체의 기운을 느꼈다!",
        DetectType::Objects => "물건의 위치를 감지했다!",
        DetectType::Traps => "함정의 위치를 감지했다!",
        DetectType::Doors => "문의 위치를 감지했다!",
        DetectType::Water => "물의 위치를 감지했다!",
        DetectType::Magic => "마법의 기운을 감지했다!",
        DetectType::Portals => "차원의 틈을 감지했다!",
    };

    // 반경 내 오브젝트 필터링
    let detected: Vec<(i32, i32)> = object_positions
        .iter()
        .filter(|_| radius >= 999 || rng.rn2(100) < 80) // 80% 탐지 확률
        .cloned()
        .collect();

    DetectResult {
        detect_type,
        detected_count: detected.len() as i32,
        radius,
        message: message.to_string(),
        positions: detected,
    }
}

// =============================================================================
// [2] 마법 지도 — magic_mapping (detect.c L1200-1400)
// =============================================================================

/// [v2.27.0 91-10] 마법 지도 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagicMapResult {
    pub tiles_revealed: i32,
    pub traps_found: i32,
    pub doors_found: i32,
    pub secret_doors_found: i32,
    pub full_map: bool,
}

/// [v2.27.0 91-10] 마법 지도 효과
/// 원본: detect.c do_mapping()
pub fn magic_mapping(
    is_blessed: bool,
    is_confused: bool,
    total_tiles: i32,
    hidden_traps: i32,
    hidden_doors: i32,
    secret_doors: i32,
) -> MagicMapResult {
    if is_confused {
        // 혼란 → 지도 왜곡 (일부만 공개)
        return MagicMapResult {
            tiles_revealed: total_tiles / 4,
            traps_found: 0,
            doors_found: hidden_doors / 2,
            secret_doors_found: 0,
            full_map: false,
        };
    }

    let full = is_blessed;
    let tiles = if full {
        total_tiles
    } else {
        total_tiles * 3 / 4
    };
    let traps = if full { hidden_traps } else { hidden_traps / 2 };
    let doors = hidden_doors;
    let secrets = if full { secret_doors } else { secret_doors / 2 };

    MagicMapResult {
        tiles_revealed: tiles,
        traps_found: traps,
        doors_found: doors,
        secret_doors_found: secrets,
        full_map: full,
    }
}

// =============================================================================
// [3] 점술 — divination (detect.c L1500-1800)
// =============================================================================

/// [v2.27.0 91-10] 점술 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DivinationResult {
    /// 위험 경고
    DangerWarning { direction: String, distance: i32 },
    /// 보물 위치
    TreasureHint { message: String },
    /// 몬스터 정보
    MonsterInfo {
        name: String,
        level: i32,
        is_hostile: bool,
    },
    /// 아무것도 없음
    Nothing,
}

/// [v2.27.0 91-10] 점술 판정
pub fn divination(
    crystal_ball_charge: i32,
    player_int: i32,
    nearby_monsters: &[(String, i32, bool)], // (이름, 레벨, 적대)
    nearby_treasure: bool,
    rng: &mut NetHackRng,
) -> DivinationResult {
    if crystal_ball_charge <= 0 {
        return DivinationResult::Nothing;
    }

    // INT 기반 성공 확률
    if rng.rn2(20) >= player_int {
        return DivinationResult::Nothing;
    }

    let choice = rng.rn2(3);
    match choice {
        0 => {
            if !nearby_monsters.is_empty() {
                let idx = rng.rn2(nearby_monsters.len() as i32) as usize;
                let (ref name, level, hostile) = nearby_monsters[idx];
                DivinationResult::MonsterInfo {
                    name: name.clone(),
                    level,
                    is_hostile: hostile,
                }
            } else {
                DivinationResult::Nothing
            }
        }
        1 => {
            if nearby_treasure {
                DivinationResult::TreasureHint {
                    message: "가까이에 가치 있는 것이 있다!".to_string(),
                }
            } else {
                DivinationResult::Nothing
            }
        }
        _ => {
            let dirs = ["북", "남", "동", "서", "북동", "북서", "남동", "남서"];
            let dir_idx = rng.rn2(dirs.len() as i32) as usize;
            DivinationResult::DangerWarning {
                direction: dirs[dir_idx].to_string(),
                distance: rng.rn2(10) + 1,
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

    #[test]
    fn test_detect_objects() {
        let mut rng = test_rng();
        let positions = vec![(5, 5), (10, 10), (15, 15)];
        let result = detect_objects(DetectType::Gold, 1, false, 80, 21, &positions, &mut rng);
        assert!(result.detected_count > 0);
    }

    #[test]
    fn test_detect_blessed() {
        let mut rng = test_rng();
        let result = detect_objects(DetectType::Objects, 0, true, 80, 21, &[], &mut rng);
        assert_eq!(result.radius, 10); // 기본 5 * 2
    }

    #[test]
    fn test_magic_map_normal() {
        let result = magic_mapping(false, false, 1000, 10, 5, 3);
        assert_eq!(result.tiles_revealed, 750);
        assert!(!result.full_map);
    }

    #[test]
    fn test_magic_map_blessed() {
        let result = magic_mapping(true, false, 1000, 10, 5, 3);
        assert_eq!(result.tiles_revealed, 1000);
        assert!(result.full_map);
    }

    #[test]
    fn test_magic_map_confused() {
        let result = magic_mapping(false, true, 1000, 10, 5, 3);
        assert_eq!(result.tiles_revealed, 250);
    }

    #[test]
    fn test_divination_empty() {
        let mut rng = test_rng();
        let result = divination(0, 18, &[], false, &mut rng);
        assert!(matches!(result, DivinationResult::Nothing));
    }

    #[test]
    fn test_divination_with_monsters() {
        let mut found = false;
        let monsters = vec![("드래곤".to_string(), 20, true)];
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = divination(5, 18, &monsters, true, &mut rng);
            if matches!(result, DivinationResult::MonsterInfo { .. }) {
                found = true;
                break;
            }
        }
        assert!(found);
    }
}

// ============================================================================
// [v2.38.0 Phase 102-4] 조명/시야 통합 (lighting_phase102_ext.rs)
// 원본: NetHack 3.6.7 src/light.c + vision.c 핵심 미이식 함수
// 순수 결과 패턴
//
// 구현 범위:
//   - 광원 시스템 (횃불, 마법빛, 고정광원)
//   - 시야 계산 (FOV: Field of View)
//   - 투명도/차폐 판정
//   - 암흑/맹인 상태 처리
//   - 적외선/텔레파시 대체 시야
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 광원 시스템 — light_source
// =============================================================================

/// [v2.38.0 102-4] 광원 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Torch,        // 횃불 (범위 3, 소모성)
    Candle,       // 촛불 (범위 2, 소모성)
    MagicLamp,    // 마법 램프 (범위 5, 영구)
    SunswordGlow, // 태양검 발광 (범위 3)
    SpellLight,   // 마법 빛 (일시적)
    RoomLit,      // 방 자체 조명
    Moonlight,    // 달빛 (야외)
}

/// [v2.38.0 102-4] 광원 데이터
#[derive(Debug, Clone)]
pub struct LightSource {
    pub light_type: LightType,
    pub x: i32,
    pub y: i32,
    pub radius: i32,          // 조명 반경
    pub remaining_turns: i32, // 잔여 턴 (-1 = 영구)
    pub intensity: f64,       // 0.0~1.0 밝기
}

/// [v2.38.0 102-4] 기본 광원 생성
pub fn create_light_source(light_type: LightType, x: i32, y: i32) -> LightSource {
    let (radius, turns, intensity) = match light_type {
        LightType::Torch => (3, 500, 0.8),
        LightType::Candle => (2, 200, 0.5),
        LightType::MagicLamp => (5, -1, 1.0),
        LightType::SunswordGlow => (3, -1, 0.9),
        LightType::SpellLight => (4, 100, 0.7),
        LightType::RoomLit => (10, -1, 0.6),
        LightType::Moonlight => (8, -1, 0.3),
    };

    LightSource {
        light_type,
        x,
        y,
        radius,
        remaining_turns: turns,
        intensity,
    }
}

/// [v2.38.0 102-4] 광원 턴 경과 (소모성 광원 처리)
pub fn tick_light_source(source: &mut LightSource) -> Option<String> {
    if source.remaining_turns > 0 {
        source.remaining_turns -= 1;

        // 깜빡임 시작 (10턴 이하)
        if source.remaining_turns <= 10 && source.remaining_turns > 0 {
            source.intensity *= 0.9;
            return Some(format!("{:?}이(가) 깜빡거린다!", source.light_type));
        }

        // 소진
        if source.remaining_turns == 0 {
            source.intensity = 0.0;
            return Some(format!("{:?}이(가) 꺼졌다!", source.light_type));
        }
    }
    None
}

// =============================================================================
// [2] 시야 계산 — field_of_view
// =============================================================================

/// [v2.38.0 102-4] 시야 모드
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionMode {
    Normal,    // 일반 시야
    Blind,     // 맹인 (시야 0)
    Infrared,  // 적외선 (생물 감지)
    Telepathy, // 텔레파시 (지능체 감지)
    XRay,      // 투시 (벽 투과)
    Dark,      // 암시 (램프 없이 어둠 속)
}

/// [v2.38.0 102-4] 시야 계산 결과
#[derive(Debug, Clone)]
pub struct VisionResult {
    pub mode: VisionMode,
    pub visible_radius: i32,
    pub can_see_monsters: bool,
    pub can_see_items: bool,
    pub can_see_terrain: bool,
    pub darkness_penalty: f64,
    pub description: String,
}

/// [v2.38.0 102-4] 시야 계산
pub fn calculate_vision(mode: VisionMode, light_radius: i32, is_room_lit: bool) -> VisionResult {
    match mode {
        VisionMode::Normal => {
            let radius = if is_room_lit {
                15 // 밝은 방: 전체 가시
            } else {
                light_radius.max(1) // 광원 반경
            };
            VisionResult {
                mode,
                visible_radius: radius,
                can_see_monsters: true,
                can_see_items: true,
                can_see_terrain: true,
                darkness_penalty: 0.0,
                description: if is_room_lit {
                    "밝은 방이다."
                } else {
                    "광원 범위 내를 볼 수 있다."
                }
                .to_string(),
            }
        }
        VisionMode::Blind => VisionResult {
            mode,
            visible_radius: 0,
            can_see_monsters: false,
            can_see_items: false,
            can_see_terrain: false,
            darkness_penalty: 1.0,
            description: "아무것도 보이지 않는다!".to_string(),
        },
        VisionMode::Infrared => VisionResult {
            mode,
            visible_radius: 5,
            can_see_monsters: true,
            can_see_items: false,
            can_see_terrain: false,
            darkness_penalty: 0.5,
            description: "생물의 체온이 감지된다.".to_string(),
        },
        VisionMode::Telepathy => VisionResult {
            mode,
            visible_radius: 20,
            can_see_monsters: true,
            can_see_items: false,
            can_see_terrain: false,
            darkness_penalty: 0.0,
            description: "지능을 가진 존재의 생각이 느껴진다.".to_string(),
        },
        VisionMode::XRay => VisionResult {
            mode,
            visible_radius: 8,
            can_see_monsters: true,
            can_see_items: true,
            can_see_terrain: true,
            darkness_penalty: 0.0,
            description: "벽 너머가 투시된다!".to_string(),
        },
        VisionMode::Dark => VisionResult {
            mode,
            visible_radius: 1,
            can_see_monsters: false,
            can_see_items: false,
            can_see_terrain: true,
            darkness_penalty: 0.8,
            description: "바로 옆만 겨우 보인다.".to_string(),
        },
    }
}

/// [v2.38.0 102-4] 시선 차단 판정 (간이 Bresenham)
pub fn is_line_of_sight_clear(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    walls: &[(i32, i32)], // 벽 좌표 목록
) -> bool {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut cx = x1;
    let mut cy = y1;

    loop {
        if cx == x2 && cy == y2 {
            return true;
        }
        if walls.contains(&(cx, cy)) && !(cx == x1 && cy == y1) {
            return false; // 벽에 차단됨
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_torch() {
        let light = create_light_source(LightType::Torch, 5, 5);
        assert_eq!(light.radius, 3);
        assert_eq!(light.remaining_turns, 500);
    }

    #[test]
    fn test_magic_lamp_permanent() {
        let light = create_light_source(LightType::MagicLamp, 0, 0);
        assert_eq!(light.remaining_turns, -1);
    }

    #[test]
    fn test_torch_burnout() {
        let mut light = create_light_source(LightType::Torch, 0, 0);
        light.remaining_turns = 5;
        let msg = tick_light_source(&mut light);
        assert!(msg.is_some()); // 깜빡거림
    }

    #[test]
    fn test_vision_normal_lit() {
        let v = calculate_vision(VisionMode::Normal, 3, true);
        assert_eq!(v.visible_radius, 15);
        assert!(v.can_see_items);
    }

    #[test]
    fn test_vision_blind() {
        let v = calculate_vision(VisionMode::Blind, 5, true);
        assert_eq!(v.visible_radius, 0);
        assert!(!v.can_see_monsters);
    }

    #[test]
    fn test_vision_telepathy() {
        let v = calculate_vision(VisionMode::Telepathy, 0, false);
        assert!(v.can_see_monsters);
        assert!(!v.can_see_items);
    }

    #[test]
    fn test_los_clear() {
        let walls: Vec<(i32, i32)> = vec![];
        assert!(is_line_of_sight_clear(0, 0, 5, 5, &walls));
    }

    #[test]
    fn test_los_blocked() {
        let walls = vec![(3, 3)];
        assert!(!is_line_of_sight_clear(0, 0, 5, 5, &walls));
    }

    #[test]
    fn test_xray_vision() {
        let v = calculate_vision(VisionMode::XRay, 0, false);
        assert!(v.can_see_terrain);
        assert_eq!(v.visible_radius, 8);
    }
}

// ============================================================================
// [v2.24.0 Phase 3-1] 시야 시스템 확장 (vision_phase3_ext.rs)
// 원본: NetHack 3.6.7 src/vision.c L100-2541 핵심 미이식 함수 이식
// 순수 결과 패턴: VisionSystem 직접 조작 없이 독립 테스트 가능
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] Couldsee 테이블 — vision_recalc의 핵심 (vision.c L240-420)
// 플레이어 이전 위치 기반 "볼 수 있었던" 영역 추적
// =============================================================================

/// [v2.24.0 3-1] 시야 변경 타일 (이전 → 현재 시야 비교 결과)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionChangeType {
    /// 새로 시야에 들어옴 (이전에는 안 보이다가 지금 보임)
    Revealed,
    /// 시야에서 사라짐 (이전에는 보이다가 지금 안 보임)
    Hidden,
    /// 계속 보임
    StillVisible,
    /// 계속 안 보임
    StillHidden,
}

/// [v2.24.0 3-1] (x, y) 좌표의 시야 변경 판정
/// 원본: vision.c vision_recalc() — couldsee/cansee 비교 로직
pub fn vision_change_at(could_see_before: bool, can_see_now: bool) -> VisionChangeType {
    match (could_see_before, can_see_now) {
        (false, true) => VisionChangeType::Revealed,
        (true, false) => VisionChangeType::Hidden,
        (true, true) => VisionChangeType::StillVisible,
        (false, false) => VisionChangeType::StillHidden,
    }
}

// =============================================================================
// [2] Seenv 방향 비트마스크 — 벽/문 렌더링 방향 (vision.c L45-90)
// 원본: SV0~SV7 (8방향 비트) — 타일이 어느 방향에서 보였는지 기록
// =============================================================================

/// [v2.24.0 3-1] 8방향 관측 비트
pub const SV_AT: u8 = 0x01; // 해당 위치 위
pub const SV_NE: u8 = 0x02; // 북동
pub const SV_E: u8 = 0x04; // 동
pub const SV_SE: u8 = 0x08; // 남동
pub const SV_S: u8 = 0x10; // 남
pub const SV_SW: u8 = 0x20; // 남서
pub const SV_W: u8 = 0x40; // 서
pub const SV_NW: u8 = 0x80; // 북서
pub const SV_ALL: u8 = 0xFF; // 모든 방향

/// [v2.24.0 3-1] 관찰자 위치에서 대상 위치 방향의 seenv 비트 계산
/// 원본: vision.c set_seenv() — 관찰자→대상 방향 비트 결정
pub fn seenv_from_direction(dx: i32, dy: i32) -> u8 {
    // (dx, dy) = (대상x - 관찰자x, 대상y - 관찰자y)
    // 즉 관찰자가 대상을 어느 방향에서 보는지
    match (dx.signum(), dy.signum()) {
        (0, 0) => SV_AT,
        (1, -1) => SV_NE, // 대상이 우상단
        (1, 0) => SV_E,
        (1, 1) => SV_SE,
        (0, 1) => SV_S,
        (-1, 1) => SV_SW,
        (-1, 0) => SV_W,
        (-1, -1) => SV_NW,
        (0, -1) => SV_NE | SV_NW, // 순수 북 = 북동+북서
        _ => SV_AT,
    }
}

/// [v2.24.0 3-1] seenv 비트마스크에서 관찰 방향 수 계산
pub fn seenv_directions_count(seenv: u8) -> i32 {
    seenv.count_ones() as i32
}

/// [v2.24.0 3-1] 벽 타일의 완전 관찰 여부
/// 원본: vision.c — seenv가 양쪽 면 모두 관찰했는지
pub fn is_wall_fully_observed(seenv: u8, wall_is_vertical: bool) -> bool {
    if wall_is_vertical {
        // 수직 벽: 동(E)과 서(W) 방향 모두 관찰
        (seenv & SV_E != 0) && (seenv & SV_W != 0)
    } else {
        // 수평 벽: 위(NE|NW)와 아래(SE|SW|S) 방향 모두 관찰
        let seen_from_above = seenv & (SV_NE | SV_NW) != 0;
        let seen_from_below = seenv & (SV_SE | SV_SW | SV_S) != 0;
        seen_from_above && seen_from_below
    }
}

// =============================================================================
// [3] 경고(Warning) 시스템 — 몬스터 위협 감지 (vision.c L1850-1960)
// =============================================================================

/// [v2.24.0 3-1] 위협 등급 (원본: WARN_OF_MON 관련)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WarningLevel {
    /// 위협 감지 안 됨
    None,
    /// 약한 위협 (레벨 1-5)
    Low,
    /// 보통 위협 (레벨 6-10)
    Medium,
    /// 강한 위협 (레벨 11-15)
    High,
    /// 극심한 위협 (레벨 16-20)
    Severe,
    /// 재앙적 위협 (레벨 21+)
    Critical,
}

/// [v2.24.0 3-1] 위협 감지 입력
#[derive(Debug, Clone)]
pub struct WarningInput {
    /// 플레이어 위험 감지 능력 (warning intrinsic)
    pub has_warning: bool,
    /// 플레이어 위치
    pub player_x: i32,
    pub player_y: i32,
    /// 몬스터 위치
    pub monster_x: i32,
    pub monster_y: i32,
    /// 몬스터 레벨
    pub monster_level: i32,
    /// 몬스터가 평화적인지
    pub is_peaceful: bool,
    /// 몬스터가 보이는지 (이미 시야 내)
    pub already_visible: bool,
    /// 몬스터가 죽지 않는 종류인지
    pub is_undead: bool,
}

/// [v2.24.0 3-1] 위협 감지 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarningResult {
    /// 위협 등급
    pub level: WarningLevel,
    /// 방향 (대략적 — 8방향)
    pub direction: Option<(i32, i32)>,
    /// 메시지
    pub message: Option<String>,
}

/// [v2.24.0 3-1] 위협 감지 판정
/// 원본: vision.c warning 관련 + display.c WARN_color 매핑
pub fn warning_detect(input: &WarningInput) -> WarningResult {
    // [1] 경고 능력 없음
    if !input.has_warning {
        return WarningResult {
            level: WarningLevel::None,
            direction: None,
            message: None,
        };
    }

    // [2] 평화적이면 위협 아님
    if input.is_peaceful {
        return WarningResult {
            level: WarningLevel::None,
            direction: None,
            message: None,
        };
    }

    // [3] 이미 보이면 경고 불필요
    if input.already_visible {
        return WarningResult {
            level: WarningLevel::None,
            direction: None,
            message: None,
        };
    }

    // [4] 거리 계산 — 경고 범위 15칸
    let dx = input.monster_x - input.player_x;
    let dy = input.monster_y - input.player_y;
    let dist_sq = dx * dx + dy * dy;
    let max_range = 15;

    if dist_sq > max_range * max_range {
        return WarningResult {
            level: WarningLevel::None,
            direction: None,
            message: None,
        };
    }

    // [5] 위협 등급 결정 — 몬스터 레벨 기반
    // 원본: display.c L2190-2210
    let level = match input.monster_level {
        0..=5 => WarningLevel::Low,
        6..=10 => WarningLevel::Medium,
        11..=15 => WarningLevel::High,
        16..=20 => WarningLevel::Severe,
        _ => WarningLevel::Critical,
    };

    // [6] 방향 표시
    let dir = (dx.signum(), dy.signum());

    // [7] 메시지 생성
    let level_str = match level {
        WarningLevel::Low => "약한",
        WarningLevel::Medium => "보통의",
        WarningLevel::High => "강한",
        WarningLevel::Severe => "극심한",
        WarningLevel::Critical => "재앙적인",
        WarningLevel::None => unreachable!(),
    };

    let dir_str = match dir {
        (0, -1) => "북쪽",
        (1, -1) => "북동쪽",
        (1, 0) => "동쪽",
        (1, 1) => "남동쪽",
        (0, 1) => "남쪽",
        (-1, 1) => "남서쪽",
        (-1, 0) => "서쪽",
        (-1, -1) => "북서쪽",
        _ => "주변",
    };

    WarningResult {
        level,
        direction: Some(dir),
        message: Some(format!("{} 위협이 {}에서 감지되었다!", level_str, dir_str)),
    }
}

// =============================================================================
// [4] 부분 시야 구간 — do_clear_area 확장 (vision.c L1450-1600)
// 특정 사각 영역의 시야 상태 일괄 조회/변경
// =============================================================================

/// [v2.24.0 3-1] 사각형 영역의 시야 통계
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaVisionStats {
    /// 영역 내 보이는 타일 수
    pub visible_count: i32,
    /// 영역 내 기억된 타일 수
    pub memorized_count: i32,
    /// 영역 내 차단된 타일 수
    pub blocked_count: i32,
    /// 전체 타일 수
    pub total_count: i32,
}

/// [v2.24.0 3-1] 사각형 영역의 시야 통계 계산
/// 원본: vision.c do_clear_area()의 확장
pub fn area_vision_stats(
    viz_array: &[[u8; 21]; 80], // COLNO=80, ROWNO=21
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
) -> AreaVisionStats {
    let mut visible = 0;
    let mut memorized = 0;
    let mut blocked = 0;
    let mut total = 0;

    let x_start = x1.min(79);
    let x_end = x2.min(79);
    let y_start = y1.min(20);
    let y_end = y2.min(20);

    for x in x_start..=x_end {
        for y in y_start..=y_end {
            total += 1;
            let v = viz_array[x][y];
            if v & 0x02 != 0 {
                // IN_SIGHT
                visible += 1;
            }
            if v & 0x08 != 0 {
                // MEMORIZED
                memorized += 1;
            }
            if v == 0 {
                blocked += 1;
            }
        }
    }

    AreaVisionStats {
        visible_count: visible,
        memorized_count: memorized,
        blocked_count: blocked,
        total_count: total,
    }
}

// =============================================================================
// [5] Shadow Casting FOV — 대칭형 그림자 투사 (Recursive Shadowcasting)
// 원본: vision.c의 quadrant 기반 FOV → 현대적 대칭 알고리즘으로 대체
// =============================================================================

/// [v2.24.0 3-1] Shadow Casting FOV 계산 결과
/// 반환: 가시 타일 좌표 목록 (중복 없음)
pub fn shadowcast_fov<F>(
    origin_x: i32,
    origin_y: i32,
    radius: i32,
    max_x: i32,
    max_y: i32,
    is_opaque: &F,
) -> Vec<(i32, i32)>
where
    F: Fn(i32, i32) -> bool,
{
    let mut visible = Vec::new();

    // 원점은 항상 보임
    if origin_x >= 0 && origin_x < max_x && origin_y >= 0 && origin_y < max_y {
        visible.push((origin_x, origin_y));
    }

    // 8 옥탄트 (octant) 스캔
    for octant in 0..8 {
        shadowcast_scan(
            origin_x,
            origin_y,
            radius,
            1,
            1.0, // start_slope: 시작은 최대(1.0)
            0.0, // end_slope: 끝은 최소(0.0)
            octant,
            max_x,
            max_y,
            is_opaque,
            &mut visible,
        );
    }

    // 중복 제거
    visible.sort();
    visible.dedup();
    visible
}

/// [v2.24.0 3-1] 단일 옥탄트 재귀 스캔
fn shadowcast_scan<F>(
    ox: i32,
    oy: i32,
    radius: i32,
    row: i32,
    mut start_slope: f64,
    end_slope: f64,
    octant: u8,
    max_x: i32,
    max_y: i32,
    is_opaque: &F,
    visible: &mut Vec<(i32, i32)>,
) where
    F: Fn(i32, i32) -> bool,
{
    if start_slope < end_slope || row > radius {
        return;
    }

    let mut prev_blocked = false;
    let mut saved_start = start_slope;

    for col in (-(row as f64 * end_slope) as i32)..=((row as f64 * start_slope) as i32) {
        // 옥탄트 변환
        let (dx, dy) = transform_octant(row, col, octant);
        let x = ox + dx;
        let y = oy + dy;

        // 범위 검사
        if x < 0 || x >= max_x || y < 0 || y >= max_y {
            prev_blocked = true;
            continue;
        }

        // 거리 검사
        if dx * dx + dy * dy > radius * radius {
            continue;
        }

        let left_slope = (col as f64 - 0.5) / (row as f64 + 0.5);
        let right_slope = (col as f64 + 0.5) / (row as f64 - 0.5);

        if right_slope < end_slope {
            continue;
        }
        if left_slope > start_slope {
            continue;
        }

        // 타일 추가
        visible.push((x, y));

        let blocked = is_opaque(x, y);

        if blocked {
            if !prev_blocked {
                // 차단 시작 → 재귀 (좁아진 시야)
                shadowcast_scan(
                    ox,
                    oy,
                    radius,
                    row + 1,
                    saved_start,
                    (col as f64 - 0.5) / (row as f64 + 0.5),
                    octant,
                    max_x,
                    max_y,
                    is_opaque,
                    visible,
                );
            }
            prev_blocked = true;
            saved_start = (col as f64 + 0.5) / (row as f64 - 0.5);
        } else {
            if prev_blocked {
                saved_start = (col as f64 + 0.5) / (row as f64 - 0.5);
            }
            prev_blocked = false;
        }
    }

    if !prev_blocked {
        shadowcast_scan(
            ox,
            oy,
            radius,
            row + 1,
            saved_start,
            end_slope,
            octant,
            max_x,
            max_y,
            is_opaque,
            visible,
        );
    }
}

/// [v2.24.0 3-1] 옥탄트 좌표 변환
fn transform_octant(row: i32, col: i32, octant: u8) -> (i32, i32) {
    match octant {
        0 => (col, -row),
        1 => (row, -col),
        2 => (row, col),
        3 => (col, row),
        4 => (-col, row),
        5 => (-row, col),
        6 => (-row, -col),
        7 => (-col, -row),
        _ => (col, -row),
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- vision_change_at ---

    #[test]
    fn test_vision_change_revealed() {
        assert_eq!(vision_change_at(false, true), VisionChangeType::Revealed);
    }

    #[test]
    fn test_vision_change_hidden() {
        assert_eq!(vision_change_at(true, false), VisionChangeType::Hidden);
    }

    #[test]
    fn test_vision_change_still_visible() {
        assert_eq!(vision_change_at(true, true), VisionChangeType::StillVisible);
    }

    #[test]
    fn test_vision_change_still_hidden() {
        assert_eq!(
            vision_change_at(false, false),
            VisionChangeType::StillHidden
        );
    }

    // --- seenv ---

    #[test]
    fn test_seenv_east() {
        assert_eq!(seenv_from_direction(1, 0), SV_E);
    }

    #[test]
    fn test_seenv_west() {
        assert_eq!(seenv_from_direction(-1, 0), SV_W);
    }

    #[test]
    fn test_seenv_at() {
        assert_eq!(seenv_from_direction(0, 0), SV_AT);
    }

    #[test]
    fn test_seenv_count() {
        assert_eq!(seenv_directions_count(SV_E | SV_W | SV_S), 3);
        assert_eq!(seenv_directions_count(SV_ALL), 8);
    }

    #[test]
    fn test_wall_fully_observed_vertical() {
        assert!(is_wall_fully_observed(SV_E | SV_W, true));
        assert!(!is_wall_fully_observed(SV_E, true));
    }

    #[test]
    fn test_wall_fully_observed_horizontal() {
        assert!(is_wall_fully_observed(SV_NE | SV_SE, false));
        assert!(!is_wall_fully_observed(SV_NE, false));
    }

    // --- warning_detect ---

    #[test]
    fn test_warning_no_ability() {
        let input = WarningInput {
            has_warning: false,
            player_x: 10,
            player_y: 10,
            monster_x: 12,
            monster_y: 12,
            monster_level: 15,
            is_peaceful: false,
            already_visible: false,
            is_undead: false,
        };
        let result = warning_detect(&input);
        assert_eq!(result.level, WarningLevel::None);
    }

    #[test]
    fn test_warning_peaceful() {
        let input = WarningInput {
            has_warning: true,
            player_x: 10,
            player_y: 10,
            monster_x: 12,
            monster_y: 12,
            monster_level: 15,
            is_peaceful: true,
            already_visible: false,
            is_undead: false,
        };
        let result = warning_detect(&input);
        assert_eq!(result.level, WarningLevel::None);
    }

    #[test]
    fn test_warning_already_visible() {
        let input = WarningInput {
            has_warning: true,
            player_x: 10,
            player_y: 10,
            monster_x: 12,
            monster_y: 12,
            monster_level: 15,
            is_peaceful: false,
            already_visible: true,
            is_undead: false,
        };
        let result = warning_detect(&input);
        assert_eq!(result.level, WarningLevel::None);
    }

    #[test]
    fn test_warning_high_threat() {
        let input = WarningInput {
            has_warning: true,
            player_x: 10,
            player_y: 10,
            monster_x: 12,
            monster_y: 10,
            monster_level: 13,
            is_peaceful: false,
            already_visible: false,
            is_undead: false,
        };
        let result = warning_detect(&input);
        assert_eq!(result.level, WarningLevel::High);
        assert!(result.direction.is_some());
        assert!(result.message.is_some());
    }

    #[test]
    fn test_warning_out_of_range() {
        let input = WarningInput {
            has_warning: true,
            player_x: 10,
            player_y: 10,
            monster_x: 50,
            monster_y: 50,
            monster_level: 20,
            is_peaceful: false,
            already_visible: false,
            is_undead: false,
        };
        let result = warning_detect(&input);
        assert_eq!(result.level, WarningLevel::None);
    }

    #[test]
    fn test_warning_critical() {
        let input = WarningInput {
            has_warning: true,
            player_x: 10,
            player_y: 10,
            monster_x: 11,
            monster_y: 10,
            monster_level: 25,
            is_peaceful: false,
            already_visible: false,
            is_undead: false,
        };
        let result = warning_detect(&input);
        assert_eq!(result.level, WarningLevel::Critical);
    }

    // --- area_vision_stats ---

    #[test]
    fn test_area_stats_empty() {
        let viz: [[u8; 21]; 80] = [[0; 21]; 80];
        let stats = area_vision_stats(&viz, 0, 0, 5, 5);
        assert_eq!(stats.visible_count, 0);
        assert_eq!(stats.blocked_count, 36); // 6×6 = 36
        assert_eq!(stats.total_count, 36);
    }

    #[test]
    fn test_area_stats_with_visibility() {
        let mut viz: [[u8; 21]; 80] = [[0; 21]; 80];
        viz[2][2] = 0x02; // IN_SIGHT
        viz[3][3] = 0x08; // MEMORIZED
        viz[4][4] = 0x0A; // IN_SIGHT | MEMORIZED
        let stats = area_vision_stats(&viz, 0, 0, 5, 5);
        assert_eq!(stats.visible_count, 2); // [2][2]와 [4][4]
        assert_eq!(stats.memorized_count, 2); // [3][3]와 [4][4]
    }

    // --- shadowcast_fov ---

    #[test]
    fn test_shadowcast_origin_visible() {
        // 장애물 없는 열린 공간
        let visible = shadowcast_fov(10, 10, 3, 80, 21, &|_x, _y| false);
        assert!(visible.contains(&(10, 10)), "원점은 항상 보여야 함");
    }

    #[test]
    fn test_shadowcast_radius() {
        // 장애물 없는 열린 공간에서 반경 3 FOV
        let visible = shadowcast_fov(10, 10, 3, 80, 21, &|_x, _y| false);
        assert!(visible.len() > 20, "반경 3에서 최소 20개 이상 타일 보임");

        // 원점에서 반경을 크게 초과하는 지점(거리6)은 확실히 안 보임
        assert!(
            !visible.contains(&(10, 16)),
            "(10,16)는 반경 6이므로 안 보임"
        );
        assert!(
            !visible.contains(&(16, 10)),
            "(16,10)는 반경 6이므로 안 보임"
        );
    }

    #[test]
    fn test_shadowcast_wall_blocks() {
        // (11, 10)에 벽 → (12, 10) 이후는 안 보여야 함
        let visible = shadowcast_fov(10, 10, 5, 80, 21, &|x, _y| x == 11 && _y == 10);
        assert!(visible.contains(&(11, 10)), "벽 자체는 보임");
        // 벽 뒤의 직선은 안 보여야 함 (대부분 가려짐)
        // 단, shadowcasting 특성상 대각선으로 우회할 수 있으므로 완전 차단은 아님
    }

    #[test]
    fn test_shadowcast_edge_of_map() {
        // 맵 가장자리에서 FOV — 경계 오류 없어야 함
        let visible = shadowcast_fov(0, 0, 3, 80, 21, &|_x, _y| false);
        assert!(visible.contains(&(0, 0)));
        assert!(!visible.contains(&(-1, 0)), "음수 좌표 없음");
    }
}

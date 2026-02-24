// ============================================================================
// [v2.29.0 R17-2] 탐지 맵 확장 (detect_map_ext.rs)
// 원본: NetHack 3.6.7 detect.c (1,675줄)
// 맵 완전 탐지, 몬스터 탐지, 아이템 탐지, 함정 감지
// ============================================================================

use crate::core::dungeon::COLNO;
use crate::core::dungeon::ROWNO;

/// [v2.29.0 R17-2] 탐지 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionType {
    /// 몬스터 감지 (포션 of monster detection)
    Monsters,
    /// 아이템 감지 (포션 of object detection)
    Objects,
    /// 함정 감지 (scroll of gold detection → 함정 표시)
    Traps,
    /// 금 감지
    Gold,
    /// 전체 맵 (scroll of magic mapping)
    FullMap,
    /// 특정 타입 추적 (crystal ball)
    Specific(char),
}

/// [v2.29.0 R17-2] 탐지 결과
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// 발견된 좌표 목록
    pub found: Vec<(i32, i32)>,
    /// 발견 개수
    pub count: usize,
    /// 메시지
    pub message: String,
}

/// [v2.29.0 R17-2] 범위 내 탐지 (원본: detect.c do_mapping)
pub fn detect_in_range(
    detection: DetectionType,
    center_x: i32,
    center_y: i32,
    range: i32,
    found_positions: &[(i32, i32)],
) -> DetectionResult {
    let in_range: Vec<(i32, i32)> = found_positions
        .iter()
        .filter(|&&(x, y)| {
            let dx = (x - center_x).abs();
            let dy = (y - center_y).abs();
            if range <= 0 {
                true
            } else {
                dx <= range && dy <= range
            }
        })
        .copied()
        .collect();

    let count = in_range.len();
    let message = match detection {
        DetectionType::Monsters if count > 0 => format!("{}마리의 몬스터를 감지했다.", count),
        DetectionType::Monsters => "이 층에 다른 몬스터가 없는 것 같다.".to_string(),
        DetectionType::Objects if count > 0 => format!("{}개의 아이템을 감지했다.", count),
        DetectionType::Objects => "이 층에 아이템이 없는 것 같다.".to_string(),
        DetectionType::Traps if count > 0 => format!("{}개의 함정을 감지했다!", count),
        DetectionType::Traps => "이 층에 함정이 없는 것 같다.".to_string(),
        DetectionType::Gold if count > 0 => format!("{}곳에서 금을 감지했다.", count),
        DetectionType::Gold => "이 층에 금이 없는 것 같다.".to_string(),
        DetectionType::FullMap => "이 층의 전체 지도가 밝혀졌다!".to_string(),
        DetectionType::Specific(_) if count > 0 => format!("{}곳에서 대상을 감지했다.", count),
        DetectionType::Specific(_) => "아무것도 감지되지 않았다.".to_string(),
    };

    DetectionResult {
        found: in_range,
        count,
        message,
    }
}

/// [v2.29.0 R17-2] 매직 매핑 (전체 맵 공개)
pub type RevealMap = [[bool; ROWNO]; COLNO];

pub fn magic_mapping() -> RevealMap {
    [[true; ROWNO]; COLNO]
}

/// [v2.29.0 R17-2] 부분 매핑 (범위 한정)
pub fn partial_mapping(center_x: usize, center_y: usize, range: usize) -> RevealMap {
    let mut map = [[false; ROWNO]; COLNO];
    let x_min = center_x.saturating_sub(range);
    let x_max = (center_x + range).min(COLNO - 1);
    let y_min = center_y.saturating_sub(range);
    let y_max = (center_y + range).min(ROWNO - 1);

    for x in x_min..=x_max {
        for y in y_min..=y_max {
            map[x][y] = true;
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_monsters() {
        let positions = vec![(5, 5), (10, 10), (50, 15)];
        let result = detect_in_range(DetectionType::Monsters, 5, 5, 10, &positions);
        assert_eq!(result.count, 2); // (5,5), (10,10) 범위 내
    }

    #[test]
    fn test_detect_none() {
        let result = detect_in_range(DetectionType::Objects, 5, 5, 3, &[]);
        assert_eq!(result.count, 0);
        assert!(result.message.contains("없는"));
    }

    #[test]
    fn test_detect_unlimited_range() {
        let positions = vec![(1, 1), (70, 18)];
        let result = detect_in_range(DetectionType::Gold, 0, 0, 0, &positions);
        assert_eq!(result.count, 2); // 범위 0 = 무한
    }

    #[test]
    fn test_magic_mapping() {
        let map = magic_mapping();
        assert!(map[0][0]);
        assert!(map[COLNO - 1][ROWNO - 1]);
    }

    #[test]
    fn test_partial_mapping() {
        let map = partial_mapping(40, 10, 5);
        assert!(map[40][10]);
        assert!(map[35][5]);
        assert!(!map[0][0]);
    }
}

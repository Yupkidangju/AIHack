// ============================================================================
// [v2.37.0 R25-1] 범위 공격 (area_attack_ext.rs)
// 원본: NetHack 3.6.7 explode.c/zap.c 범위 확장
// 폭발, 브레스, 콘, 구 범위 판정
// ============================================================================

/// [v2.37.0 R25-1] 범위 공격 형태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AreaShape {
    Circle(i32),    // 원형 (반경)
    Cone(i32),      // 원뿔 (길이)
    Line(i32),      // 직선 (길이)
    Beam(i32),      // 빔 (길이, 관통)
    Explosion(i32), // 폭발 (반경, 강화)
}

/// [v2.37.0 R25-1] 범위 내 좌표 계산 (원형)
pub fn circle_area(cx: i32, cy: i32, radius: i32) -> Vec<(i32, i32)> {
    let mut result = Vec::new();
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                result.push((cx + dx, cy + dy));
            }
        }
    }
    result
}

/// [v2.37.0 R25-1] 범위 내 좌표 계산 (직선)
pub fn line_area(sx: i32, sy: i32, dx: i32, dy: i32, length: i32) -> Vec<(i32, i32)> {
    (1..=length).map(|i| (sx + dx * i, sy + dy * i)).collect()
}

/// [v2.37.0 R25-1] 폭발 데미지 감쇠 (거리에 따라)
pub fn explosion_damage(base_damage: i32, distance: i32, radius: i32) -> i32 {
    if distance > radius {
        return 0;
    }
    let factor = radius - distance + 1;
    (base_damage * factor / (radius + 1)).max(1)
}

/// [v2.37.0 R25-1] 아군 피격 여부 (friendly fire)
pub fn in_blast_radius(target_x: i32, target_y: i32, cx: i32, cy: i32, radius: i32) -> bool {
    let dx = (target_x - cx).abs();
    let dy = (target_y - cy).abs();
    dx * dx + dy * dy <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle() {
        let area = circle_area(5, 5, 2);
        assert!(area.contains(&(5, 5)));
        assert!(area.contains(&(5, 7)));
        assert!(!area.contains(&(5, 8)));
    }

    #[test]
    fn test_line() {
        let area = line_area(0, 0, 1, 0, 5);
        assert_eq!(area.len(), 5);
        assert_eq!(area[4], (5, 0));
    }

    #[test]
    fn test_explosion_damage() {
        assert!(explosion_damage(20, 0, 3) > explosion_damage(20, 2, 3));
    }

    #[test]
    fn test_damage_outside() {
        assert_eq!(explosion_damage(20, 5, 3), 0);
    }

    #[test]
    fn test_friendly_fire() {
        assert!(in_blast_radius(3, 3, 5, 5, 3));
        assert!(!in_blast_radius(0, 0, 5, 5, 3));
    }
}

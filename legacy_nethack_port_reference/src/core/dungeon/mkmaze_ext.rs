// ============================================================================
// [v2.25.0 R13-4] 미로 생성 (mkmaze_ext.rs)
// 원본: NetHack 3.6.7 mkmaze.c (1,561줄)
// 미로 생성 알고리즘, Vlad's Tower, 특수 지형
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 미로 그리드 (원본: mkmaze.c maze generation)
// =============================================================================

/// [v2.25.0 R13-4] 미로 셀
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MazeCell {
    Wall,
    Path,
    Door,
    StairsUp,
    StairsDown,
}

/// [v2.25.0 R13-4] DFS 기반 미로 생성 (원본: walkfrom)
pub fn generate_maze(width: usize, height: usize, rng: &mut NetHackRng) -> Vec<Vec<MazeCell>> {
    let mut grid = vec![vec![MazeCell::Wall; height]; width];

    // 시작점 (홀수 좌표)
    let sx = 1;
    let sy = 1;
    grid[sx][sy] = MazeCell::Path;

    // DFS 스택
    let mut stack: Vec<(usize, usize)> = vec![(sx, sy)];

    while let Some(&(cx, cy)) = stack.last() {
        // 이동 가능 방향 수집 (2칸 단위)
        let mut neighbors = Vec::new();
        let dirs: [(i32, i32); 4] = [(0, -2), (0, 2), (-2, 0), (2, 0)];
        for (dx, dy) in &dirs {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx > 0
                && (nx as usize) < width - 1
                && ny > 0
                && (ny as usize) < height - 1
                && grid[nx as usize][ny as usize] == MazeCell::Wall
            {
                neighbors.push((nx as usize, ny as usize));
            }
        }

        if neighbors.is_empty() {
            stack.pop();
        } else {
            // 랜덤 방향 선택
            let idx = rng.rn2(neighbors.len() as i32) as usize;
            let (nx, ny) = neighbors[idx];

            // 중간 벽 제거
            let mx = (cx + nx) / 2;
            let my = (cy + ny) / 2;
            grid[mx][my] = MazeCell::Path;
            grid[nx][ny] = MazeCell::Path;

            stack.push((nx, ny));
        }
    }

    grid
}

// =============================================================================
// [2] 미로 통계
// =============================================================================

/// [v2.25.0 R13-4] 미로 통로 비율
pub fn maze_path_ratio(grid: &[Vec<MazeCell>]) -> f64 {
    let mut paths = 0;
    let mut total = 0;
    for col in grid {
        for cell in col {
            total += 1;
            if *cell == MazeCell::Path {
                paths += 1;
            }
        }
    }
    if total == 0 {
        0.0
    } else {
        paths as f64 / total as f64
    }
}

/// [v2.25.0 R13-4] 미로 데드엔드 카운트
pub fn count_dead_ends(grid: &[Vec<MazeCell>]) -> usize {
    let width = grid.len();
    let height = if width > 0 { grid[0].len() } else { 0 };
    let mut count = 0;

    for x in 1..width.saturating_sub(1) {
        for y in 1..height.saturating_sub(1) {
            if grid[x][y] != MazeCell::Path {
                continue;
            }
            let mut openings = 0;
            if grid[x - 1][y] == MazeCell::Path {
                openings += 1;
            }
            if grid[x + 1][y] == MazeCell::Path {
                openings += 1;
            }
            if grid[x][y - 1] == MazeCell::Path {
                openings += 1;
            }
            if grid[x][y + 1] == MazeCell::Path {
                openings += 1;
            }
            if openings == 1 {
                count += 1;
            }
        }
    }
    count
}

// =============================================================================
// [3] Vlad's Tower 구조 (원본: mkmaze.c setup_waterlevel 등)
// =============================================================================

/// [v2.25.0 R13-4] 특수 미로 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialMazeType {
    /// 기본 미로
    Standard,
    /// 블라드 탑 (작은 층)
    VladsTower,
    /// 미노타우르 미로 (거대)
    MinotaurMaze,
    /// 물 레벨 (엔드게임)
    WaterLevel,
    /// 지구 레벨 (엔드게임)
    EarthLevel,
    /// 불 레벨
    FireLevel,
    /// 공기 레벨
    AirLevel,
    /// 아스트랄 평원
    AstralPlane,
}

/// [v2.25.0 R13-4] 특수 미로 크기 결정
pub fn special_maze_size(maze_type: SpecialMazeType) -> (usize, usize) {
    match maze_type {
        SpecialMazeType::Standard => (80, 21),
        SpecialMazeType::VladsTower => (40, 12),
        SpecialMazeType::MinotaurMaze => (80, 21),
        SpecialMazeType::WaterLevel => (80, 21),
        SpecialMazeType::EarthLevel => (80, 21),
        SpecialMazeType::FireLevel => (80, 21),
        SpecialMazeType::AirLevel => (80, 21),
        SpecialMazeType::AstralPlane => (80, 21),
    }
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_maze() {
        let mut rng = NetHackRng::new(42);
        let grid = generate_maze(21, 13, &mut rng);
        assert_eq!(grid.len(), 21);
        assert_eq!(grid[0].len(), 13);
        assert_eq!(grid[1][1], MazeCell::Path); // 시작점
    }

    #[test]
    fn test_maze_has_paths() {
        let mut rng = NetHackRng::new(42);
        let grid = generate_maze(21, 13, &mut rng);
        let ratio = maze_path_ratio(&grid);
        assert!(ratio > 0.1); // 최소 10% 통로
        assert!(ratio < 0.6); // 미로니까 60% 미만
    }

    #[test]
    fn test_maze_connectivity() {
        let mut rng = NetHackRng::new(42);
        let grid = generate_maze(21, 13, &mut rng);
        // 통로 존재 확인
        let path_count: usize = grid
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| **c == MazeCell::Path)
            .count();
        assert!(path_count > 10);
    }

    #[test]
    fn test_dead_ends() {
        let mut rng = NetHackRng::new(42);
        let grid = generate_maze(21, 13, &mut rng);
        let dead_ends = count_dead_ends(&grid);
        assert!(dead_ends > 0); // 미로에는 막다른 길이 있어야
    }

    #[test]
    fn test_special_maze_size() {
        assert_eq!(special_maze_size(SpecialMazeType::VladsTower), (40, 12));
        assert_eq!(special_maze_size(SpecialMazeType::Standard), (80, 21));
    }
}

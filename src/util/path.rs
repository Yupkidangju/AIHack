use crate::core::dungeon::{Grid, COLNO, ROWNO};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    pos: (i32, i32),
    cost: i32,
    priority: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.pos.cmp(&other.pos))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct PathFinder;

impl PathFinder {
    ///
    ///
    pub fn find_path(
        grid: &Grid,
        start: (i32, i32),
        goal: (i32, i32),
        can_pass: impl Fn(&Grid, i32, i32) -> bool,
    ) -> Option<Vec<(i32, i32)>> {
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
        let mut g_score: HashMap<(i32, i32), i32> = HashMap::new();

        g_score.insert(start, 0);
        open_set.push(Node {
            pos: start,
            cost: 0,
            priority: Self::heuristic(start, goal),
        });

        while let Some(current) = open_set.pop() {
            if current.pos == goal {
                return Some(Self::reconstruct_path(came_from, goal));
            }

            if let Some(&score) = g_score.get(&current.pos) {
                if current.cost > score {
                    continue;
                }
            }

            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let nx = current.pos.0 + dx;
                    let ny = current.pos.1 + dy;

                    if nx < 0 || ny < 0 || nx >= COLNO as i32 || ny >= ROWNO as i32 {
                        continue;
                    }

                    if !can_pass(grid, nx, ny) && (nx, ny) != goal {
                        continue;
                    }

                    let tentative_g = g_score[&current.pos] + 1;
                    if tentative_g < *g_score.get(&(nx, ny)).unwrap_or(&i32::MAX) {
                        came_from.insert((nx, ny), current.pos);
                        g_score.insert((nx, ny), tentative_g);
                        open_set.push(Node {
                            pos: (nx, ny),
                            cost: tentative_g,
                            priority: tentative_g + Self::heuristic((nx, ny), goal),
                        });
                    }
                }
            }
        }

        None
    }

    fn heuristic(a: (i32, i32), b: (i32, i32)) -> i32 {
        //
        (a.0 - b.0).abs().max((a.1 - b.1).abs())
    }

    fn reconstruct_path(
        came_from: HashMap<(i32, i32), (i32, i32)>,
        mut current: (i32, i32),
    ) -> Vec<(i32, i32)> {
        let mut path = vec![current];
        while let Some(&prev) = came_from.get(&current) {
            current = prev;
            path.push(current);
        }
        path.reverse();
        path
    }
}

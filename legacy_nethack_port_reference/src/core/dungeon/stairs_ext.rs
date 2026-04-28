// ============================================================================
// [v2.33.0 R21-5] 사다리/구멍 (stairs_ext.rs)
// 원본: NetHack 3.6.7 mklev.c/do.c 계단/사다리/구멍
// 레벨 전환, 구멍 생성, 사다리 판정
// ============================================================================

/// [v2.33.0 R21-5] 수직 이동 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalLink {
    StairsUp,
    StairsDown,
    LadderUp,
    LadderDown,
    Hole,            // 구멍 (아래로만)
    TrapDoor,        // 함정문 (아래로만)
    MagicPortal,     // 마법 포탈 (퀘스트 등)
    LevelTeleporter, // 레벨 텔레포트
}

/// [v2.33.0 R21-5] 이동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StairResult {
    Move {
        target_depth: i32,
        link_type: VerticalLink,
    },
    Blocked(String),
    BranchChange {
        branch: String,
        depth: i32,
    },
}

/// [v2.33.0 R21-5] 계단 사용 (원본: goto_level)
pub fn use_stairs(
    link: VerticalLink,
    current_depth: i32,
    max_depth: i32,
    is_levitation: bool,
) -> StairResult {
    // 부유 중이면 구멍 못 빠짐
    if is_levitation && matches!(link, VerticalLink::Hole | VerticalLink::TrapDoor) {
        return StairResult::Blocked("부유 중이라 떨어지지 않는다.".to_string());
    }

    match link {
        VerticalLink::StairsUp | VerticalLink::LadderUp => {
            if current_depth <= 1 {
                StairResult::Blocked("더 올라갈 수 없다.".to_string())
            } else {
                StairResult::Move {
                    target_depth: current_depth - 1,
                    link_type: link,
                }
            }
        }
        VerticalLink::StairsDown
        | VerticalLink::LadderDown
        | VerticalLink::Hole
        | VerticalLink::TrapDoor => {
            if current_depth >= max_depth {
                StairResult::Blocked("더 내려갈 수 없다.".to_string())
            } else {
                StairResult::Move {
                    target_depth: current_depth + 1,
                    link_type: link,
                }
            }
        }
        VerticalLink::MagicPortal => StairResult::BranchChange {
            branch: "quest".to_string(),
            depth: 1,
        },
        VerticalLink::LevelTeleporter => StairResult::Move {
            target_depth: current_depth,
            link_type: link,
        },
    }
}

/// [v2.33.0 R21-5] 구멍 낙하 데미지
pub fn fall_damage(depth_fallen: i32) -> i32 {
    (depth_fallen * 3).min(30)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stairs_down() {
        let r = use_stairs(VerticalLink::StairsDown, 5, 50, false);
        assert!(matches!(
            r,
            StairResult::Move {
                target_depth: 6,
                ..
            }
        ));
    }

    #[test]
    fn test_stairs_up_floor() {
        let r = use_stairs(VerticalLink::StairsUp, 1, 50, false);
        assert!(matches!(r, StairResult::Blocked(_)));
    }

    #[test]
    fn test_hole_levitate() {
        let r = use_stairs(VerticalLink::Hole, 5, 50, true);
        assert!(matches!(r, StairResult::Blocked(_)));
    }

    #[test]
    fn test_portal() {
        let r = use_stairs(VerticalLink::MagicPortal, 5, 50, false);
        assert!(matches!(r, StairResult::BranchChange { .. }));
    }

    #[test]
    fn test_fall_damage() {
        assert_eq!(fall_damage(3), 9);
        assert_eq!(fall_damage(15), 30); // 최대 30
    }
}

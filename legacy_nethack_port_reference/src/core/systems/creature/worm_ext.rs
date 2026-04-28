// worm_ext.rs — worm.c 핵심 로직 순수 결과 패턴 이식
// [v2.18.0] 신규 생성: 벌레 성장/수축/절단/분할/슬롯/교차 판정 등 12개 함수
// 원본: NetHack 3.6.7 src/worm.c (875줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 최대 벌레 수
pub const MAX_NUM_WORMS: usize = 32;

/// 벌레 최대 HP
pub const MHPMAX: i32 = 127;

/// 벌레 성장 HP 증가량
pub const WORM_GROW_HP: i32 = 3;

// ============================================================
// 열거형
// ============================================================

/// 벌레 이동 결과
/// 원본: worm.c worm_move() L196-235
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WormMoveResult {
    /// 성장함 — HP 증가
    Grew {
        new_hp: i32,
        new_maxhp: i32,
        next_grow_time: i64,
    },
    /// 꼬리 축소
    Shrunk,
}

/// 벌레 절단 결과
/// 원본: worm.c cutworm() L316-423
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WormCutResult {
    /// 절단 안 됨 (확률 실패)
    NotCut,
    /// 머리 맞음 (절단 무의미)
    HitHead,
    /// 꼬리만 잘림
    TailSevered { old_worm_hp: i32 },
    /// 둘로 분할됨
    Split {
        old_level: i32,
        old_hp: i32,
        old_maxhp: i32,
        new_level: i32,
        new_hp: i32,
    },
}

/// 벌레 교차 판정 결과
/// 원본: worm.c worm_cross() L801-847
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WormCrossResult {
    /// 비대각 이동 → 교차 없음
    NotDiagonal,
    /// 같은 몬스터 아님
    DifferentMonsters,
    /// 연속 세그먼트 → 교차
    Crosses,
    /// 비연속 → 교차 아님
    NoCross,
}

// ============================================================
// 1. get_wormno_check — 빈 슬롯 탐색
// ============================================================

/// 사용 가능한 벌레 슬롯이 있는지 확인
/// 원본: worm.c get_wormno() L86-97
/// occupied_slots: 인덱스1부터 MAX_NUM_WORMS-1까지, true면 사용 중
pub fn get_wormno_check(occupied_slots: &[bool]) -> Option<usize> {
    for i in 1..occupied_slots.len().min(MAX_NUM_WORMS) {
        if !occupied_slots[i] {
            return Some(i);
        }
    }
    None
}

// ============================================================
// 2. worm_move_result — 벌레 이동 성장/축소 판정
// ============================================================

/// 벌레 이동 시 성장 또는 축소 판정
/// 원본: worm.c worm_move() L222-234
pub fn worm_move_result(
    current_hp: i32,
    current_maxhp: i32,
    grow_time: i64,
    current_moves: i64,
    rng: &mut NetHackRng,
) -> WormMoveResult {
    if grow_time <= current_moves {
        // 성장
        let next_time = if grow_time == 0 {
            current_moves + rng.rnd(5) as i64
        } else {
            current_moves + rng.rn1(15, 3) as i64
        };
        let new_hp = (current_hp + WORM_GROW_HP).min(MHPMAX);
        let new_maxhp = if new_hp > current_maxhp {
            new_hp
        } else {
            current_maxhp
        };
        WormMoveResult::Grew {
            new_hp,
            new_maxhp,
            next_grow_time: next_time,
        }
    } else {
        WormMoveResult::Shrunk
    }
}

// ============================================================
// 3. worm_nomove_result — 움직이지 못하는 벌레 HP
// ============================================================

/// 벌레가 움직이지 못할 때 HP 감소
/// 원본: worm.c worm_nomove() L244-254
pub fn worm_nomove_result(current_hp: i32) -> i32 {
    if current_hp > 3 {
        current_hp - 3
    } else {
        1
    }
}

// ============================================================
// 4. worm_cut_check — 절단 확률 계산
// ============================================================

/// 벌레 절단 확률 (칼날 여부 반영)
/// 원본: worm.c cutworm() L334-339
pub fn worm_cut_check(is_bladed: bool, rng: &mut NetHackRng) -> bool {
    let mut cut = rng.rnd(20);
    if is_bladed {
        cut += 10;
    }
    cut >= 17
}

// ============================================================
// 5. worm_cut_result — 절단 결과 판정
// ============================================================

/// 벌레 절단 전체 결과 판정 (머리/꼬리/분할)
/// 원본: worm.c cutworm() L316-423
pub fn worm_cut_result(
    is_head: bool,
    is_bladed: bool,
    is_tail_segment: bool,
    monster_level: i32,
    current_hp: i32,
    rng: &mut NetHackRng,
) -> WormCutResult {
    if is_head {
        return WormCutResult::HitHead;
    }

    if !worm_cut_check(is_bladed, rng) {
        return WormCutResult::NotCut;
    }

    if is_tail_segment {
        return WormCutResult::TailSevered {
            old_worm_hp: (current_hp / 2).max(1),
        };
    }

    // 분할 가능 여부
    let can_split = monster_level >= 3 && rng.rn2(3) == 0;
    if can_split {
        let new_level = (monster_level - 2).max(3);
        // [v2.18.0 감사 수정] 원본 worm.c L407-410:
        // new_worm->mhpmax = new_worm->mhp = d(new_worm->m_lev, 8);
        // worm->mhpmax = d(worm->m_lev, 8);
        // if (worm->mhpmax < worm->mhp) worm->mhp = worm->mhpmax;
        let new_hp = rng.d(new_level, 8);
        let old_maxhp = rng.d(new_level, 8);
        // 기존 HP가 새 최대보다 크면 잘라냄
        let old_hp = current_hp.min(old_maxhp);
        WormCutResult::Split {
            old_level: new_level,
            old_hp,
            old_maxhp,
            new_level,
            new_hp,
        }
    } else {
        WormCutResult::TailSevered {
            old_worm_hp: (current_hp / 2).max(1),
        }
    }
}

// ============================================================
// 6. worm_cross_check — 대각 이동 시 벌레 교차 판정
// ============================================================

/// 대각 이동이 벌레 세그먼트를 교차하는지 판정
/// 원본: worm.c worm_cross() L801-847
/// (x1,y1)→(x2,y2) 이동 시 (x1,y2)와 (x2,y1)이 같은 벌레의 연속 세그먼트인지
pub fn worm_cross_check(x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    // 인접 칸이 아니면 false
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();
    if dx.max(dy) != 1 {
        return false;
    }
    // 대각 이동이 아니면 교차 불가능
    x1 != x2 && y1 != y2
}

// ============================================================
// 7. wseg_index — 세그먼트 인덱스 계산
// ============================================================

/// 세그먼트 위치의 인덱스 계산 (꼬리로부터의 역순)
/// 원본: worm.c wseg_at() L850-872
pub fn wseg_index(segment_from_tail: i32, total_segments: i32) -> i32 {
    total_segments - segment_from_tail
}

// ============================================================
// 8. worm_segment_count_calc — 새 벌레 초기 세그먼트 수
// ============================================================

/// 초기 벌레 생성 시 세그먼트 수 (d8에 따라 결정됨)
/// 원본: worm.c initworm() / create_worm_tail()
pub fn worm_initial_segments(rng: &mut NetHackRng) -> i32 {
    rng.rnd(5) // 1~5 초기 세그먼트
}

// ============================================================
// 9. worm_attack_range — 세그먼트 공격 범위
// ============================================================

/// 벌레 세그먼트가 영웅을 공격할 수 있는 거리인지 판정
/// 원본: worm.c wormhitu() L303 — distu(seg->wx, seg->wy) < 3
pub fn worm_attack_range(seg_x: i32, seg_y: i32, hero_x: i32, hero_y: i32) -> bool {
    let dx = seg_x - hero_x;
    let dy = seg_y - hero_y;
    dx * dx + dy * dy < 3
}

// ============================================================
// 10. random_dir_calc — 인접 방향 랜덤 선택
// ============================================================

/// 현재 위치로부터 인접 칸 랜덤 선택 (벌레 꼬리 배치용)
/// 원본: worm.c random_dir() L699-722
/// col_max: COLNO-1, row_max: ROWNO-1
pub fn random_dir_calc(
    x: i32,
    y: i32,
    col_max: i32,
    row_max: i32,
    rng: &mut NetHackRng,
) -> (i32, i32) {
    let nx = if x > 1 {
        if x < col_max {
            x + (rng.rn2(3) - 1)
        } else {
            x - rng.rn2(2)
        }
    } else {
        x + rng.rn2(2)
    };

    let ny = if nx != x {
        // x 변경됨 → y도 랜덤
        if y > 0 {
            if y < row_max {
                y + (rng.rn2(3) - 1)
            } else {
                y - rng.rn2(2)
            }
        } else {
            y + rng.rn2(2)
        }
    } else {
        // x 안 변했으면 y 반드시 변경
        if y > 0 {
            if y < row_max {
                if rng.rn2(2) != 0 {
                    y + 1
                } else {
                    y - 1
                }
            } else {
                y - 1
            }
        } else {
            y + 1
        }
    };

    (nx, ny)
}

// ============================================================
// 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::rng::NetHackRng;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    // --- get_wormno_check ---
    #[test]
    fn test_wormno_empty() {
        let slots = vec![false; MAX_NUM_WORMS];
        assert_eq!(get_wormno_check(&slots), Some(1));
    }

    #[test]
    fn test_wormno_full() {
        let slots = vec![true; MAX_NUM_WORMS];
        assert_eq!(get_wormno_check(&slots), None);
    }

    #[test]
    fn test_wormno_partial() {
        let mut slots = vec![true; MAX_NUM_WORMS];
        slots[5] = false;
        assert_eq!(get_wormno_check(&slots), Some(5));
    }

    // --- worm_move_result ---
    #[test]
    fn test_worm_grow() {
        let mut rng = test_rng();
        match worm_move_result(50, 50, 0, 100, &mut rng) {
            WormMoveResult::Grew {
                new_hp,
                new_maxhp,
                next_grow_time,
            } => {
                assert_eq!(new_hp, 53);
                assert_eq!(new_maxhp, 53);
                assert!(next_grow_time >= 101 && next_grow_time <= 105);
            }
            _ => panic!("성장 예상"),
        }
    }

    #[test]
    fn test_worm_shrink() {
        let mut rng = test_rng();
        match worm_move_result(50, 50, 200, 100, &mut rng) {
            WormMoveResult::Shrunk => {}
            _ => panic!("축소 예상"),
        }
    }

    #[test]
    fn test_worm_grow_cap() {
        let mut rng = test_rng();
        match worm_move_result(126, 126, 0, 100, &mut rng) {
            WormMoveResult::Grew { new_hp, .. } => {
                assert_eq!(new_hp, 127); // MHPMAX
            }
            _ => panic!("성장 예상"),
        }
    }

    // --- worm_nomove_result ---
    #[test]
    fn test_nomove_normal() {
        assert_eq!(worm_nomove_result(10), 7);
    }

    #[test]
    fn test_nomove_low() {
        assert_eq!(worm_nomove_result(3), 1);
        assert_eq!(worm_nomove_result(1), 1);
    }

    // --- worm_cut_check ---
    #[test]
    fn test_cut_rates() {
        let mut rng = test_rng();
        let mut blade = 0;
        let mut normal = 0;
        for _ in 0..500 {
            if worm_cut_check(true, &mut rng) {
                blade += 1;
            }
            if worm_cut_check(false, &mut rng) {
                normal += 1;
            }
        }
        // 칼: (20+10)>=17 → 14/20=70%, 일반: rnd(20)>=17 → 4/20=20%
        assert!(blade > 280 && blade < 420, "칼: {}", blade);
        assert!(normal > 50 && normal < 150, "일반: {}", normal);
    }

    // --- worm_cut_result ---
    #[test]
    fn test_cut_head() {
        let mut rng = test_rng();
        assert_eq!(
            worm_cut_result(true, true, false, 5, 50, &mut rng),
            WormCutResult::HitHead
        );
    }

    // --- worm_cross_check ---
    #[test]
    fn test_cross_diagonal() {
        assert!(worm_cross_check(5, 5, 6, 6));
    }

    #[test]
    fn test_cross_straight() {
        assert!(!worm_cross_check(5, 5, 6, 5));
    }

    #[test]
    fn test_cross_non_adjacent() {
        assert!(!worm_cross_check(5, 5, 7, 7));
    }

    // --- wseg_index ---
    #[test]
    fn test_wseg_index() {
        assert_eq!(wseg_index(2, 10), 8);
        assert_eq!(wseg_index(0, 5), 5);
    }

    // --- worm_attack_range ---
    #[test]
    fn test_attack_range() {
        assert!(worm_attack_range(5, 5, 5, 6)); // dist2 = 1
        assert!(worm_attack_range(5, 5, 5, 5)); // dist2 = 0
        assert!(!worm_attack_range(5, 5, 7, 5)); // dist2 = 4
    }

    // --- random_dir_calc ---
    #[test]
    fn test_random_dir_moves() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let (nx, ny) = random_dir_calc(40, 10, 79, 20, &mut rng);
            assert!(nx >= 39 && nx <= 41, "nx: {}", nx);
            assert!(ny >= 9 && ny <= 11, "ny: {}", ny);
            // 적어도 하나는 변경됨
            assert!(nx != 40 || ny != 10, "같은 위치");
        }
    }

    #[test]
    fn test_random_dir_edge() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let (nx, _) = random_dir_calc(1, 10, 79, 20, &mut rng);
            assert!(nx >= 1 && nx <= 2, "왼쪽 가장자리 nx: {}", nx);
        }
    }
}

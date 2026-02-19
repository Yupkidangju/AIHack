// region_ext.rs — region.c 핵심 로직 순수 결과 패턴 이식
// [v2.17.0] 신규 생성: 구역 AABB/포함 판정, 가스 구름 데미지/면역, 위험도, 바운딩박스 등 10개 함수
// 원본: NetHack 3.6.7 src/region.c (1,129줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 몬스터 배열 증가 단위
pub const MONST_INC: usize = 5;

// ============================================================
// 구조체
// ============================================================

/// 사각형 구역
/// 원본: region.c NhRect 구조체
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub lx: i32,
    pub ly: i32,
    pub hx: i32,
    pub hy: i32,
}

/// 바운딩 박스 계산 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundingBox {
    pub lx: i32,
    pub ly: i32,
    pub hx: i32,
    pub hy: i32,
}

// ============================================================
// 열거형
// ============================================================

/// 가스 구름 피해 결과
/// 원본: region.c inside_gas_cloud() L955-1029
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasCloudEffect {
    /// 면역 (비호흡/비생물/수중)
    Immune,
    /// 독 내성 — 기침만
    PoisonResist,
    /// 데미지 적용
    Damaged(i32),
}

/// 구역 위험도 (기도 시 체크)
/// 원본: region.c region_danger() L1067-1091
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionDanger {
    /// 위험한 가스 구름 내에 있음
    InDanger(i32),
    /// 안전함
    Safe,
}

/// 가스 구름 약화/소멸 (expire_gas_cloud 결과)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasExpire {
    /// 약화 — 데미지 절반, TTL 리셋
    Weaken { new_damage: i32, new_ttl: i32 },
    /// 완전 소멸
    Gone,
}

// ============================================================
// 1. inside_rect — 사각형 내 포함 판정
// ============================================================

/// 좌표가 사각형 내에 있는지 판정
/// 원본: region.c inside_rect() L56-62
pub fn inside_rect(r: &Rect, x: i32, y: i32) -> bool {
    x >= r.lx && x <= r.hx && y >= r.ly && y <= r.hy
}

// ============================================================
// 2. inside_region — 복수 사각형 구역 내 포함 판정
// ============================================================

/// 좌표가 복수 사각형 중 하나에 포함되는지 판정
/// 원본: region.c inside_region() L67-80
pub fn inside_region(bounding_box: &Rect, rects: &[Rect], x: i32, y: i32) -> bool {
    if !inside_rect(bounding_box, x, y) {
        return false;
    }
    for r in rects {
        if inside_rect(r, x, y) {
            return true;
        }
    }
    false
}

// ============================================================
// 3. compute_bounding_box — 바운딩 박스 계산
// ============================================================

/// 사각형 목록에서 바운딩 박스 계산
/// 원본: region.c create_region() L85-116
pub fn compute_bounding_box(rects: &[Rect]) -> BoundingBox {
    if rects.is_empty() {
        return BoundingBox {
            lx: 80,
            ly: 21,
            hx: 0,
            hy: 0,
        }; // COLNO, ROWNO
    }

    let mut bb = BoundingBox {
        lx: rects[0].lx,
        ly: rects[0].ly,
        hx: rects[0].hx,
        hy: rects[0].hy,
    };

    for r in rects.iter().skip(1) {
        if r.lx < bb.lx {
            bb.lx = r.lx;
        }
        if r.ly < bb.ly {
            bb.ly = r.ly;
        }
        if r.hx > bb.hx {
            bb.hx = r.hx;
        }
        if r.hy > bb.hy {
            bb.hy = r.hy;
        }
    }

    bb
}

// ============================================================
// 4. gas_cloud_rects — 가스 구름 사각형 생성
// ============================================================

/// 가스 구름의 다이아몬드 형태 사각형 목록 생성
/// 원본: region.c create_gas_cloud() L1031-1053
pub fn gas_cloud_rects(cx: i32, cy: i32, radius: i32) -> Vec<Rect> {
    let mut rects = Vec::new();
    let mut lx = cx;
    let mut hx = cx;
    let mut ly = cy - (radius - 1);
    let mut hy = cy + (radius - 1);

    for _ in 0..radius {
        rects.push(Rect { lx, ly, hx, hy });
        lx -= 1;
        hx += 1;
        ly += 1;
        hy -= 1;
    }
    rects
}

// ============================================================
// 5. gas_cloud_hero_effect — 가스 구름 플레이어 효과 판정
// ============================================================

/// 가스 구름이 플레이어에게 미치는 효과 판정
/// 원본: region.c inside_gas_cloud() L971-988
pub fn gas_cloud_hero_effect(
    damage: i32,
    is_invulnerable: bool,
    is_nonliving: bool,
    is_breathless: bool,
    is_underwater: bool,
    has_poison_resist: bool,
    rng: &mut NetHackRng,
) -> GasCloudEffect {
    // 면역 조건
    if is_invulnerable || is_nonliving || is_breathless || is_underwater {
        return GasCloudEffect::Immune;
    }

    // 독 내성 시 기침만
    if has_poison_resist {
        return GasCloudEffect::PoisonResist;
    }

    // 실제 데미지: rnd(dam) + 5
    let dmg = rng.rnd(damage) + 5;
    GasCloudEffect::Damaged(dmg)
}

// ============================================================
// 6. gas_cloud_monster_effect — 가스 구름 몬스터 효과 판정
// ============================================================

/// 가스 구름이 몬스터에게 미치는 효과 판정
/// 원본: region.c inside_gas_cloud() L989-1029
pub fn gas_cloud_monster_effect(
    damage: i32,
    is_nonliving: bool,
    is_vampshifter: bool,
    is_breathless: bool,
    is_eel_in_water: bool,
    has_poison_breath: bool,
    has_poison_resist: bool,
    rng: &mut NetHackRng,
) -> GasCloudEffect {
    // 면역: 비생물/뱀파이어변신/비호흡/물속장어/독가스브레스
    if is_nonliving || is_vampshifter || is_breathless || is_eel_in_water || has_poison_breath {
        return GasCloudEffect::Immune;
    }

    if has_poison_resist {
        return GasCloudEffect::PoisonResist;
    }

    let dmg = rng.rnd(damage) + 5;
    GasCloudEffect::Damaged(dmg)
}

// ============================================================
// 7. gas_expire — 가스 구름 소멸/약화 판정
// ============================================================

/// 가스 구름 TTL 만료 시 소멸 또는 약화 판정
/// 원본: region.c expire_gas_cloud() L933-953
pub fn gas_expire(damage: i32) -> GasExpire {
    if damage >= 5 {
        GasExpire::Weaken {
            new_damage: damage / 2,
            new_ttl: 2,
        }
    } else {
        GasExpire::Gone
    }
}

// ============================================================
// 8. region_danger_check — 기도 시 구역 위험도 체크
// ============================================================

/// 플레이어가 위험한 구역에 있는지 판정 (기도 목적)
/// 원본: region.c region_danger() L1067-1091
pub fn region_danger_check(
    is_in_gas_cloud: bool,
    is_nonliving: bool,
    is_breathless: bool,
    has_poison_resist: bool,
) -> bool {
    if !is_in_gas_cloud {
        return false;
    }
    if is_nonliving || is_breathless {
        return false;
    }
    if has_poison_resist {
        return false;
    }
    true
}

// ============================================================
// 9. gas_cloud_initial_ttl — 가스 구름 초기 TTL
// ============================================================

/// 가스 구름 초기 생존 시간
/// 원본: region.c create_gas_cloud() L1054 — rn1(3,4) → 4~6
pub fn gas_cloud_initial_ttl(rng: &mut NetHackRng) -> i32 {
    rng.rn1(3, 4)
}

// ============================================================
// 10. region_ttl_adjust — 구역 TTL 시간 보정 (세이브/로드)
// ============================================================

/// 세이브/로드 시 경과 시간에 따른 TTL 보정
/// 원본: region.c rest_regions() L742-744
pub fn region_ttl_adjust(original_ttl: i64, elapsed_turns: i64) -> i64 {
    if original_ttl < 0 {
        original_ttl // 영구 구역
    } else if original_ttl > elapsed_turns {
        original_ttl - elapsed_turns
    } else {
        0 // 만료됨
    }
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

    // --- inside_rect ---
    #[test]
    fn test_inside_rect_center() {
        let r = Rect {
            lx: 5,
            ly: 5,
            hx: 10,
            hy: 10,
        };
        assert!(inside_rect(&r, 7, 7));
    }

    #[test]
    fn test_inside_rect_edge() {
        let r = Rect {
            lx: 5,
            ly: 5,
            hx: 10,
            hy: 10,
        };
        assert!(inside_rect(&r, 5, 5));
        assert!(inside_rect(&r, 10, 10));
    }

    #[test]
    fn test_outside_rect() {
        let r = Rect {
            lx: 5,
            ly: 5,
            hx: 10,
            hy: 10,
        };
        assert!(!inside_rect(&r, 4, 7));
        assert!(!inside_rect(&r, 11, 7));
    }

    // --- inside_region ---
    #[test]
    fn test_inside_region_multi_rect() {
        let rects = vec![
            Rect {
                lx: 0,
                ly: 0,
                hx: 5,
                hy: 5,
            },
            Rect {
                lx: 10,
                ly: 10,
                hx: 15,
                hy: 15,
            },
        ];
        let bb = Rect {
            lx: 0,
            ly: 0,
            hx: 15,
            hy: 15,
        };
        assert!(inside_region(&bb, &rects, 3, 3));
        assert!(inside_region(&bb, &rects, 12, 12));
        assert!(!inside_region(&bb, &rects, 7, 7)); // 사이 공간
    }

    #[test]
    fn test_inside_region_outside_bb() {
        let rects = vec![Rect {
            lx: 5,
            ly: 5,
            hx: 10,
            hy: 10,
        }];
        let bb = Rect {
            lx: 5,
            ly: 5,
            hx: 10,
            hy: 10,
        };
        assert!(!inside_region(&bb, &rects, 2, 2));
    }

    // --- compute_bounding_box ---
    #[test]
    fn test_bounding_box() {
        let rects = vec![
            Rect {
                lx: 3,
                ly: 4,
                hx: 8,
                hy: 9,
            },
            Rect {
                lx: 1,
                ly: 6,
                hx: 10,
                hy: 7,
            },
        ];
        let bb = compute_bounding_box(&rects);
        assert_eq!(bb.lx, 1);
        assert_eq!(bb.ly, 4);
        assert_eq!(bb.hx, 10);
        assert_eq!(bb.hy, 9);
    }

    #[test]
    fn test_bounding_box_empty() {
        let bb = compute_bounding_box(&[]);
        assert_eq!(bb.lx, 80);
        assert_eq!(bb.hy, 0);
    }

    // --- gas_cloud_rects ---
    #[test]
    fn test_gas_cloud_shape() {
        let rects = gas_cloud_rects(10, 10, 3);
        assert_eq!(rects.len(), 3);
        // 첫 번째: 중앙 열, 상하 2칸
        assert_eq!(
            rects[0],
            Rect {
                lx: 10,
                ly: 8,
                hx: 10,
                hy: 12
            }
        );
        // 두 번째: 좌우 1칸 확장, 상하 축소
        assert_eq!(
            rects[1],
            Rect {
                lx: 9,
                ly: 9,
                hx: 11,
                hy: 11
            }
        );
        // 세 번째: 좌우 2칸 확장, 중앙 1줄
        assert_eq!(
            rects[2],
            Rect {
                lx: 8,
                ly: 10,
                hx: 12,
                hy: 10
            }
        );
    }

    // --- gas_cloud_hero_effect ---
    #[test]
    fn test_gas_hero_immune() {
        let mut rng = test_rng();
        let e = gas_cloud_hero_effect(10, false, true, false, false, false, &mut rng);
        assert_eq!(e, GasCloudEffect::Immune);
    }

    #[test]
    fn test_gas_hero_resist() {
        let mut rng = test_rng();
        let e = gas_cloud_hero_effect(10, false, false, false, false, true, &mut rng);
        assert_eq!(e, GasCloudEffect::PoisonResist);
    }

    #[test]
    fn test_gas_hero_damaged() {
        let mut rng = test_rng();
        for _ in 0..50 {
            match gas_cloud_hero_effect(10, false, false, false, false, false, &mut rng) {
                GasCloudEffect::Damaged(d) => {
                    assert!(d >= 6 && d <= 15, "데미지: {}", d);
                }
                _ => panic!("데미지 예상"),
            }
        }
    }

    // --- gas_cloud_monster_effect ---
    #[test]
    fn test_gas_monster_immune_breathless() {
        let mut rng = test_rng();
        let e = gas_cloud_monster_effect(10, false, false, true, false, false, false, &mut rng);
        assert_eq!(e, GasCloudEffect::Immune);
    }

    #[test]
    fn test_gas_monster_damaged() {
        let mut rng = test_rng();
        match gas_cloud_monster_effect(8, false, false, false, false, false, false, &mut rng) {
            GasCloudEffect::Damaged(d) => {
                assert!(d >= 6 && d <= 13, "몬스터 데미지: {}", d);
            }
            _ => panic!("데미지 예상"),
        }
    }

    // --- gas_expire ---
    #[test]
    fn test_gas_expire_weaken() {
        match gas_expire(10) {
            GasExpire::Weaken {
                new_damage,
                new_ttl,
            } => {
                assert_eq!(new_damage, 5);
                assert_eq!(new_ttl, 2);
            }
            _ => panic!("약화 예상"),
        }
    }

    #[test]
    fn test_gas_expire_gone() {
        assert_eq!(gas_expire(4), GasExpire::Gone);
    }

    // --- region_danger_check ---
    #[test]
    fn test_danger_in_cloud() {
        assert!(region_danger_check(true, false, false, false));
    }

    #[test]
    fn test_danger_resist() {
        assert!(!region_danger_check(true, false, false, true));
    }

    #[test]
    fn test_danger_not_in_cloud() {
        assert!(!region_danger_check(false, false, false, false));
    }

    // --- gas_cloud_initial_ttl ---
    #[test]
    fn test_gas_ttl_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let ttl = gas_cloud_initial_ttl(&mut rng);
            assert!(ttl >= 4 && ttl <= 6, "TTL: {}", ttl);
        }
    }

    // --- region_ttl_adjust ---
    #[test]
    fn test_ttl_adjust_permanent() {
        assert_eq!(region_ttl_adjust(-1, 100), -1);
    }

    #[test]
    fn test_ttl_adjust_remaining() {
        assert_eq!(region_ttl_adjust(50, 30), 20);
    }

    #[test]
    fn test_ttl_adjust_expired() {
        assert_eq!(region_ttl_adjust(10, 20), 0);
    }

    #[test]
    fn test_ttl_adjust_exact() {
        assert_eq!(region_ttl_adjust(10, 10), 0);
    }
}

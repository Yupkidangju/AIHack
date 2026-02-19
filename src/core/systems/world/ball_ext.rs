// ball_ext.rs — ball.c 핵심 로직 순수 결과 패턴 이식
// [v2.18.0] 신규 생성: 쇠공/쇠사슬 낙하 데미지, 끌기 확률, 계단 끌림, 탈출 등 10개 함수
// 원본: NetHack 3.6.7 src/ball.c (1,115줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 함정 유형 (ball.c drop_ball에서 사용)
pub const TT_PIT: i32 = 0;
pub const TT_WEB: i32 = 1;
pub const TT_LAVA: i32 = 2;
pub const TT_BEARTRAP: i32 = 3;
pub const TT_INFLOOR: i32 = 4;
pub const TT_BURIEDBALL: i32 = 5;

/// BC 비트마스크
pub const BC_BALL: i32 = 0x01;
pub const BC_CHAIN: i32 = 0x02;

/// BC 순서
pub const BCPOS_DIFFER: i32 = 0;
pub const BCPOS_CHAIN: i32 = 1;
pub const BCPOS_BALL: i32 = 2;

// ============================================================
// 열거형
// ============================================================

/// 쇠공 낙하 결과
/// 원본: ball.c ballfall() L43-65
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BallFallResult {
    /// 맞지 않음
    Missed,
    /// 금속 투구 착용 — 데미지 3 (원본: ball.c L57 dmg=3)
    MetalHelm(i32),
    /// 일반 투구 또는 미착용 — 전체 데미지 (25~31)
    FullDamage(i32),
}

/// 끌기 방향
/// 원본: ball.c drag_down() L997-1041
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragDownResult {
    /// 앞으로 끌림 — 계단 아래로 끌려감 (데미지)
    DraggedForward(i32),
    /// 앞으로 던졌지만 안 맞음 — 아무 일 없음
    ForwardNoEffect,
    /// 뒤에서 충돌 — 데미지
    SmashedBack(i32),
    /// 뒤에서 끌림 — 추가 데미지
    DraggedBack(i32),
    /// 뒤에서 안 맞고 안 끌림
    BackNoEffect,
}

/// 쇠공으로 인한 함정 탈출 결과
/// 원본: ball.c drop_ball() L908-937
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BallTrapEscape {
    /// 구덩이에서 끌려나옴
    PulledFromPit,
    /// 거미줄에서 끌려나옴 (거미줄 파괴)
    PulledFromWeb,
    /// 용암에서 끌려나옴
    PulledFromLava,
    /// 곰 함정에서 끌려나옴 (다리 손상)
    PulledFromBearTrap { side_is_left: bool, leg_damage: i32 },
}

/// 쇠공 위치 관계
/// 원본: ball.c bc_order() L361-379
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcOrder {
    Differ,
    ChainOnTop,
    BallOnTop,
}

// ============================================================
// 1. ballfall_damage — 함정 문으로 쇠공 낙하 데미지
// ============================================================

/// 함정 문으로 떨어질 때 쇠공 데미지 판정
/// 원본: ball.c ballfall() L43-65
/// - 쇠공이 같은 위치면 맞지 않음
/// - 쇠공을 쥐고 있으면 맞지 않음 (wielded)
/// - 그 외 4/5 확률로 맞음
/// - 금속 투구: 데미지 3, 아니면 rn1(7,25) = 25~31
pub fn ballfall_damage(
    ball_same_pos: bool,
    is_wielded: bool,
    has_metal_helm: bool,
    rng: &mut NetHackRng,
) -> BallFallResult {
    // 같은 위치면 안 맞음
    if ball_same_pos {
        return BallFallResult::Missed;
    }
    // 쥐고 있으면 안 맞음
    if is_wielded {
        return BallFallResult::Missed;
    }
    // 4/5 확률
    if rng.rn2(5) == 0 {
        return BallFallResult::Missed;
    }

    let dmg = rng.rn1(7, 25); // 25~31
    if has_metal_helm {
        BallFallResult::MetalHelm(3)
    } else {
        BallFallResult::FullDamage(dmg)
    }
}

// ============================================================
// 2. drag_down_result — 계단에서 쇠공 끌림 확률
// ============================================================

/// 계단에서 쇠공에 의한 끌림 결과 판정
/// 원본: ball.c drag_down() L997-1041
pub fn drag_down_result(
    is_carried: bool,
    is_wielded: bool,
    has_weapon: bool,
    rng: &mut NetHackRng,
) -> DragDownResult {
    // 앞으로 떨어질 조건: 소지 + (무기=공 또는 무기 없음 또는 1/3)
    let forward = is_carried && (is_wielded || !has_weapon || rng.rn2(3) == 0);

    if forward {
        // 5/6 확률로 끌림
        if rng.rn2(6) != 0 {
            let dmg = rng.rnd(6);
            DragDownResult::DraggedForward(dmg)
        } else {
            DragDownResult::ForwardNoEffect
        }
    } else {
        // 1/2 확률로 충돌
        let mut drag_chance: i32 = 3;

        if rng.rn2(2) != 0 {
            let smash_dmg = rng.rnd(20);
            drag_chance -= 2;
            // 남은 drag_chance (1) >= rnd(6)?
            if drag_chance >= rng.rnd(6) {
                let drag_dmg = rng.rnd(3);
                // 충돌 + 끌림
                DragDownResult::DraggedBack(smash_dmg + drag_dmg)
            } else {
                DragDownResult::SmashedBack(smash_dmg)
            }
        } else {
            // 안 맞았지만 끌릴 수 있음
            if drag_chance >= rng.rnd(6) {
                let drag_dmg = rng.rnd(3);
                DragDownResult::DraggedBack(drag_dmg)
            } else {
                DragDownResult::BackNoEffect
            }
        }
    }
}

// ============================================================
// 3. ball_trap_escape — 쇠공에 의한 함정 탈출
// ============================================================

/// 쇠공 떨어질 때 현재 함정에서 탈출 가능한지 판정
/// 원본: ball.c drop_ball() L908-937
pub fn ball_trap_escape(trap_type: i32, rng: &mut NetHackRng) -> Option<BallTrapEscape> {
    match trap_type {
        TT_INFLOOR | TT_BURIEDBALL => None, // 탈출 불가
        TT_PIT => Some(BallTrapEscape::PulledFromPit),
        TT_WEB => Some(BallTrapEscape::PulledFromWeb),
        TT_LAVA => Some(BallTrapEscape::PulledFromLava),
        TT_BEARTRAP => {
            let side_is_left = rng.rn2(3) != 0;
            let leg_damage = rng.rn1(1000, 500); // 500~1499 턴 부상
            Some(BallTrapEscape::PulledFromBearTrap {
                side_is_left,
                leg_damage,
            })
        }
        _ => None,
    }
}

// ============================================================
// 4. litter_drop_check — 계단 끌림 시 소지품 낙하 확률
// ============================================================

/// 무게 용량 대비 아이템 낙하 확률 판정
/// 원본: ball.c litter() L976-994
pub fn litter_drop_check(item_weight: i32, weight_capacity: i32, rng: &mut NetHackRng) -> bool {
    rng.rnd(weight_capacity) <= item_weight
}

// ============================================================
// 5. bc_felt_mask — 맹인 상태 감지 마스크
// ============================================================

/// 맹인 상태에서 쇠공/쇠사슬 감지 마스크 결정
/// 원본: ball.c set_bc() L388-394
pub fn bc_felt_mask(ball_on_floor: bool) -> i32 {
    if ball_on_floor {
        BC_BALL | BC_CHAIN
    } else {
        BC_CHAIN
    }
}

// ============================================================
// 6. chain_position_for_dist — 거리에 따른 사슬 위치 보정
// ============================================================

/// 영웅과 쇠공 사이 거리 제곱에 따른 사슬 위치 결정 방식
/// 원본: ball.c drag_ball() L648-779, dist2에 따른 분기
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainPositionRule {
    /// 사이 위치 (대각 2칸)
    Midpoint,
    /// 두 후보 중 선택
    ChooseOne,
    /// 현재 위치 유지 가능
    KeepCurrent,
    /// 당기기로 전환
    ForceDrag,
}

pub fn chain_position_for_dist(dist_sq: i32) -> ChainPositionRule {
    match dist_sq {
        8 => ChainPositionRule::Midpoint,
        5 => ChainPositionRule::ChooseOne,
        4 => ChainPositionRule::KeepCurrent,
        0 | 1 | 2 => ChainPositionRule::KeepCurrent,
        _ => ChainPositionRule::ForceDrag,
    }
}

// ============================================================
// 7. drag_encumbrance_check — 끌기 무게 초과 확인
// ============================================================

/// 짐이 너무 무거워 쇠공을 끌 수 없는지 판정
/// 원본: ball.c drag_ball() L787 — near_capacity() > SLT_ENCUMBER
/// SLT_ENCUMBER = 1
pub fn drag_encumbrance_check(near_capacity: i32) -> bool {
    near_capacity > 1 // SLT_ENCUMBER
}

// ============================================================
// 8. ball_smack_tohit — 쇠공 충돌 명중 확인
// ============================================================

/// 쇠공이 몬스터에게 명중하는지 판정
/// 원본: ball.c drag_ball() L813
/// tmp = -2 + Luck + find_mac(victim) + omon_adj(...)
pub fn ball_smack_tohit(luck: i32, victim_ac: i32, omon_adj: i32, rng: &mut NetHackRng) -> bool {
    let tmp = -2 + luck + victim_ac + omon_adj;
    let dieroll = rng.rnd(20);
    tmp >= dieroll
}

// ============================================================
// 9. worm_grow_interval — 벌레 성장 주기
// ============================================================

/// 벌레 다음 성장 시간 결정
/// 원본: worm.c worm_move() L222-226
pub fn worm_grow_interval(current_time: i64, first_growth: bool, rng: &mut NetHackRng) -> i64 {
    if first_growth {
        current_time + rng.rnd(5) as i64
    } else {
        current_time + rng.rn1(15, 3) as i64
    }
}

// ============================================================
// 10. worm_cut_chance — 벌레 절단 확률
// ============================================================

/// 벌레를 칼로 잘라 분할할 확률
/// 원본: worm.c cutworm() L334-339
pub fn worm_cut_chance(is_cuttier: bool, rng: &mut NetHackRng) -> bool {
    let mut cut = rng.rnd(20);
    if is_cuttier {
        cut += 10;
    }
    cut >= 17
}

// ============================================================
// 11. worm_split_check — 벌레 분할 (꼬리 생존) 확률
// ============================================================

/// 절단된 벌레 꼬리가 새 벌레로 생존하는지 판정
/// 원본: worm.c cutworm() L373 — m_lev >= 3 && !rn2(3)
pub fn worm_split_check(monster_level: i32, rng: &mut NetHackRng) -> bool {
    monster_level >= 3 && rng.rn2(3) == 0
}

// ============================================================
// 12. worm_nomove_hp — 움직이지 못하는 벌레 HP 감소
// ============================================================

/// 벌레가 움직이지 못할 때 HP 감소량
/// 원본: worm.c worm_nomove() L250-253
pub fn worm_nomove_hp(current_hp: i32) -> i32 {
    if current_hp > 3 {
        current_hp - 3
    } else {
        1
    }
}

// ============================================================
// 13. worm_grow_hp — 벌레 성장 시 HP 증가
// ============================================================

/// 벌레 성장 시 HP +3, 최대 127까지
/// 원본: worm.c worm_move() L227-231
pub fn worm_grow_hp(current_hp: i32, current_max: i32) -> (i32, i32) {
    let new_hp = (current_hp + 3).min(127); // MHPMAX
    let new_max = if new_hp > current_max {
        new_hp
    } else {
        current_max
    };
    (new_hp, new_max)
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

    // --- ballfall_damage ---
    #[test]
    fn test_ballfall_same_pos() {
        let mut rng = test_rng();
        assert_eq!(
            ballfall_damage(true, false, false, &mut rng),
            BallFallResult::Missed
        );
    }

    #[test]
    fn test_ballfall_wielded() {
        let mut rng = test_rng();
        assert_eq!(
            ballfall_damage(false, true, false, &mut rng),
            BallFallResult::Missed
        );
    }

    #[test]
    fn test_ballfall_metal_helm() {
        let mut rng = test_rng();
        let mut metal = 0;
        for _ in 0..100 {
            match ballfall_damage(false, false, true, &mut rng) {
                BallFallResult::MetalHelm(d) => {
                    assert_eq!(d, 3);
                    metal += 1;
                }
                _ => {}
            }
        }
        // 4/5 확률로 맞고, 맞으면 항상 MetalHelm
        assert!(metal > 60, "금속 투구: {}", metal);
    }

    #[test]
    fn test_ballfall_full_dmg() {
        let mut rng = test_rng();
        for _ in 0..100 {
            match ballfall_damage(false, false, false, &mut rng) {
                BallFallResult::FullDamage(d) => {
                    assert!(d >= 25 && d <= 31, "데미지: {}", d);
                }
                _ => {}
            }
        }
    }

    // --- drag_down_result ---
    #[test]
    fn test_drag_forward() {
        let mut rng = test_rng();
        let mut dragged = 0;
        for _ in 0..200 {
            match drag_down_result(true, true, true, &mut rng) {
                DragDownResult::DraggedForward(d) => {
                    assert!(d >= 1 && d <= 6);
                    dragged += 1;
                }
                DragDownResult::ForwardNoEffect => {}
                _ => panic!("앞방향 예상"),
            }
        }
        // 5/6 ≈ 83%
        assert!(dragged > 130, "앞끌림: {}", dragged);
    }

    #[test]
    fn test_drag_backward() {
        let mut rng = test_rng();
        let mut events = 0;
        for _ in 0..200 {
            match drag_down_result(false, false, true, &mut rng) {
                DragDownResult::SmashedBack(_)
                | DragDownResult::DraggedBack(_)
                | DragDownResult::BackNoEffect => {
                    events += 1;
                }
                _ => panic!("뒤방향 예상"),
            }
        }
        assert_eq!(events, 200);
    }

    // --- ball_trap_escape ---
    #[test]
    fn test_trap_pit() {
        let mut rng = test_rng();
        assert_eq!(
            ball_trap_escape(TT_PIT, &mut rng),
            Some(BallTrapEscape::PulledFromPit)
        );
    }

    #[test]
    fn test_trap_web() {
        let mut rng = test_rng();
        assert_eq!(
            ball_trap_escape(TT_WEB, &mut rng),
            Some(BallTrapEscape::PulledFromWeb)
        );
    }

    #[test]
    fn test_trap_beartrap() {
        let mut rng = test_rng();
        if let Some(BallTrapEscape::PulledFromBearTrap { leg_damage, .. }) =
            ball_trap_escape(TT_BEARTRAP, &mut rng)
        {
            assert!(
                leg_damage >= 500 && leg_damage <= 1499,
                "다리 손상: {}",
                leg_damage
            );
        } else {
            panic!("곰 함정 탈출 예상");
        }
    }

    #[test]
    fn test_trap_infloor() {
        let mut rng = test_rng();
        assert!(ball_trap_escape(TT_INFLOOR, &mut rng).is_none());
    }

    // --- litter_drop_check ---
    #[test]
    fn test_litter_heavy() {
        let mut rng = test_rng();
        let mut dropped = 0;
        for _ in 0..100 {
            if litter_drop_check(500, 100, &mut rng) {
                dropped += 1;
            }
        }
        // 무거운 아이템 → 거의 100% 낙하
        assert!(dropped > 95, "무거운 낙하: {}", dropped);
    }

    #[test]
    fn test_litter_light() {
        let mut rng = test_rng();
        let mut dropped = 0;
        for _ in 0..100 {
            if litter_drop_check(1, 1000, &mut rng) {
                dropped += 1;
            }
        }
        // 가벼운 아이템 → 거의 0% 낙하
        assert!(dropped < 5, "가벼운 낙하: {}", dropped);
    }

    // --- bc_felt_mask ---
    #[test]
    fn test_felt_mask() {
        assert_eq!(bc_felt_mask(true), BC_BALL | BC_CHAIN);
        assert_eq!(bc_felt_mask(false), BC_CHAIN);
    }

    // --- chain_position ---
    #[test]
    fn test_chain_position() {
        assert_eq!(chain_position_for_dist(8), ChainPositionRule::Midpoint);
        assert_eq!(chain_position_for_dist(5), ChainPositionRule::ChooseOne);
        assert_eq!(chain_position_for_dist(4), ChainPositionRule::KeepCurrent);
        assert_eq!(chain_position_for_dist(1), ChainPositionRule::KeepCurrent);
        assert_eq!(chain_position_for_dist(20), ChainPositionRule::ForceDrag);
    }

    // --- drag_encumbrance_check ---
    #[test]
    fn test_encumbrance() {
        assert!(!drag_encumbrance_check(0));
        assert!(!drag_encumbrance_check(1));
        assert!(drag_encumbrance_check(2));
    }

    // --- ball_smack_tohit ---
    #[test]
    fn test_smack_hit() {
        let mut rng = test_rng();
        let mut hits = 0;
        for _ in 0..200 {
            if ball_smack_tohit(5, 10, 2, &mut rng) {
                hits += 1;
            }
        }
        // tmp = -2+5+10+2 = 15, 15/20 = 75%
        assert!(hits > 120 && hits < 180, "명중: {}", hits);
    }

    // --- worm functions ---
    #[test]
    fn test_worm_grow_interval() {
        let mut rng = test_rng();
        let t = worm_grow_interval(100, true, &mut rng);
        assert!(t >= 101 && t <= 105, "첫 성장: {}", t);
        let t2 = worm_grow_interval(100, false, &mut rng);
        assert!(t2 >= 103 && t2 <= 117, "재성장: {}", t2);
    }

    #[test]
    fn test_worm_cut_chance() {
        let mut rng = test_rng();
        let mut cuts_blade = 0;
        let mut cuts_normal = 0;
        for _ in 0..200 {
            if worm_cut_chance(true, &mut rng) {
                cuts_blade += 1;
            }
            if worm_cut_chance(false, &mut rng) {
                cuts_normal += 1;
            }
        }
        // 칼: 7-20 절단 = 70%, 일반: 17-20 = 20%
        assert!(
            cuts_blade > cuts_normal,
            "칼>{}: {}",
            cuts_normal,
            cuts_blade
        );
    }

    #[test]
    fn test_worm_split_check() {
        let mut rng = test_rng();
        // 레벨 < 3: 절대 불가
        assert!(!worm_split_check(2, &mut rng));
        let mut splits = 0;
        for _ in 0..300 {
            if worm_split_check(5, &mut rng) {
                splits += 1;
            }
        }
        // ~33%
        assert!(splits > 60 && splits < 150, "분할: {}", splits);
    }

    #[test]
    fn test_worm_nomove_hp() {
        assert_eq!(worm_nomove_hp(10), 7);
        assert_eq!(worm_nomove_hp(3), 1);
        assert_eq!(worm_nomove_hp(1), 1);
    }

    #[test]
    fn test_worm_grow_hp() {
        assert_eq!(worm_grow_hp(50, 50), (53, 53));
        assert_eq!(worm_grow_hp(125, 125), (127, 127)); // 최대
        assert_eq!(worm_grow_hp(30, 50), (33, 50));
    }
}

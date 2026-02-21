// =============================================================================
// AIHack — apply_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
// [v2.19.0] apply.c 핵심 함수 이식 — Pure Result 패턴
// 원본: nethack-3.6.7/src/apply.c (3,812줄)
//
// 이식 대상:
//   camera       (L53-89)   → camera_flash_result
//   towel        (L91-175)  → towel_use_result
//   leashable    (L598-605) → leashable_check
//   jump_range   (L2200+)   → jump_range_result
//   tinning_kit  (L1800+)   → tinning_result
//   figurine     (L1900+)   → figurine_result
//   whip         (L2600+)   → whip_use_result
//   pole         (L2700+)   → polearm_range_check
//   cream_pie    (L2800+)   → cream_pie_result
//   grapple      (L3000+)   → grapple_result
// =============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [v2.19.0] 1. camera_flash_result — 카메라 플래시 효과
// 원본: apply.c use_camera() L53-89
// =============================================================================

/// 카메라 플래시 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CameraFlashResult {
    /// 충전 없음 — 아무 일 없음
    NoCharges,
    /// 저주된 카메라 — 자기 자신에게 플래시
    CursedSelfFlash,
    /// 자신을 향해 촬영
    SelfFlash,
    /// 삼킨 상태 — 위장 촬영
    StomachPhoto,
    /// 위/아래 방향 — 바닥/천장 촬영
    FloorCeilingPhoto { looking_down: bool },
    /// 몬스터에게 플래시 명중
    HitMonster,
    /// 빗나감 (몬스터 없음)
    Missed,
}

/// [v2.19.0] 카메라 플래시 결과 판정 (원본: use_camera L53-89)
pub fn camera_flash_result(
    charges_left: i32,
    is_cursed: bool,
    is_underwater: bool,
    is_swallowed: bool,
    dx: i32,
    dy: i32,
    dz: i32,
    has_target_monster: bool,
    rng: &mut NetHackRng,
) -> CameraFlashResult {
    // 수중 사용 불가 — 호출자가 차단
    if is_underwater {
        return CameraFlashResult::NoCharges;
    }

    if charges_left <= 0 {
        return CameraFlashResult::NoCharges;
    }

    // 저주된 카메라: 50% 자기 플래시
    if is_cursed && rng.rn2(2) == 0 {
        return CameraFlashResult::CursedSelfFlash;
    }

    // 삼킨 상태
    if is_swallowed {
        return CameraFlashResult::StomachPhoto;
    }

    // 위/아래 방향
    if dz != 0 {
        return CameraFlashResult::FloorCeilingPhoto {
            looking_down: dz > 0,
        };
    }

    // 자신을 향해
    if dx == 0 && dy == 0 {
        return CameraFlashResult::SelfFlash;
    }

    // 몬스터 대상
    if has_target_monster {
        CameraFlashResult::HitMonster
    } else {
        CameraFlashResult::Missed
    }
}

// =============================================================================
// [v2.19.0] 2. towel_use_result — 수건 사용 효과
// 원본: apply.c use_towel() L91-175
// =============================================================================

/// 수건 사용 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TowelResult {
    /// 손 안 비었음 — 사용 불가
    NoFreeHand,
    /// 착용 중 — 사용 불가
    WearingIt,
    /// 저주됨 — 손이 미끌미끌 (rn2(3)==2)
    CursedSlippery,
    /// 저주됨 — 얼굴에 더러움 (rn2(3)==1)
    CursedGunkFace,
    /// 미끄러운 손 닦기 성공
    WipedGlib,
    /// 얼굴 닦기 성공
    WipedFace,
    /// 이미 깨끗함
    AlreadyClean,
}

/// [v2.19.0] 수건 사용 결과 판정 (원본: use_towel L91-175)
pub fn towel_use_result(
    has_free_hand: bool,
    is_wearing_it: bool,
    is_cursed: bool,
    hands_are_glib: bool,
    face_is_creamed: bool,
    rng: &mut NetHackRng,
) -> TowelResult {
    if !has_free_hand {
        return TowelResult::NoFreeHand;
    }
    if is_wearing_it {
        return TowelResult::WearingIt;
    }

    // 저주된 수건
    if is_cursed {
        match rng.rn2(3) {
            2 => return TowelResult::CursedSlippery,
            1 => return TowelResult::CursedGunkFace,
            _ => {} // 0이면 정상 사용
        }
    }

    // 미끄러운 손 닦기
    if hands_are_glib {
        return TowelResult::WipedGlib;
    }

    // 얼굴 닦기
    if face_is_creamed {
        return TowelResult::WipedFace;
    }

    TowelResult::AlreadyClean
}

// =============================================================================
// [v2.19.0] 3. leashable_check — 목줄 대상 판정
// 원본: apply.c leashable() L598-605
// =============================================================================

/// [v2.19.0] 목줄 대상 판정 (원본: leashable L598-605)
/// 긴 벌레, 비고체, 사지 없고 머리 없는 몬스터는 불가
pub fn leashable_check(
    is_long_worm: bool,
    is_unsolid: bool,
    has_limbs: bool,
    has_head: bool,
) -> bool {
    !is_long_worm && !is_unsolid && (has_limbs || has_head)
}

// =============================================================================
// [v2.19.0] 4. jump_range_result — 점프 범위 결정
// 원본: apply.c get_valid_jump_position() ~L2200
// =============================================================================

/// [v2.19.0] 점프 가능 범위 (원본: L2200)
/// 기본 2칸, 점프 부츠/기술에 따라 최대 4칸
pub fn jump_range(
    has_jump_boots: bool,
    jump_skill_level: i32, // 0=없음, 1=기본, 2=숙련, 3=전문
    encumbrance: i32,      // 0=없음, 1~4단계
) -> i32 {
    let base = if has_jump_boots {
        if jump_skill_level >= 3 {
            4
        } else if jump_skill_level >= 2 {
            3
        } else {
            2
        }
    } else if jump_skill_level >= 1 {
        2
    } else {
        0 // 점프 능력 없음
    };

    // 짐 과다 시 범위 감소
    let penalty = encumbrance.min(2);
    (base - penalty).max(0)
}

// =============================================================================
// [v2.19.0] 5. tinning_result — 통조림 도구 결과
// 원본: apply.c use_tinning_kit() ~L1800
// =============================================================================

/// 통조림 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TinningResult {
    /// 통조림 성공 (재료 소비, 통조림 생산)
    Success { turns_needed: i32 },
    /// 축복된 통조림: 1턴만에 완료
    BlessedInstant,
    /// 충전 없음
    NoCharges,
    /// 대상 시체 없음
    NoCorpse,
}

/// [v2.19.0] 통조림 결과 (원본: use_tinning_kit ~L1800)
pub fn tinning_result(
    has_charges: bool,
    has_corpse: bool,
    is_blessed: bool,
    monster_size: i32, // 0=작음, 1=중간, 2=큼, 3=거대
    rng: &mut NetHackRng,
) -> TinningResult {
    if !has_charges {
        return TinningResult::NoCharges;
    }
    if !has_corpse {
        return TinningResult::NoCorpse;
    }
    if is_blessed {
        return TinningResult::BlessedInstant;
    }
    // 원본: 크기에 따라 턴 수 결정
    let turns = match monster_size {
        0 => rng.rn1(3, 2),  // 2~4턴
        1 => rng.rn1(5, 4),  // 4~8턴
        2 => rng.rn1(8, 6),  // 6~13턴
        _ => rng.rn1(10, 8), // 8~17턴 (거대)
    };
    TinningResult::Success {
        turns_needed: turns,
    }
}

// =============================================================================
// [v2.19.0] 6. figurine_result — 조각상 사용 결과
// 원본: apply.c use_figurine() ~L1900
// =============================================================================

/// 조각상 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FigurineResult {
    /// 몬스터 소환 성공
    SummonSuccess,
    /// 위치 부적합 (가구 위, 물 위 등)
    InvalidLocation,
    /// 저주됨 — 적대적 몬스터
    CursedHostile,
}

/// [v2.19.0] 조각상 사용 결과 (원본: use_figurine ~L1900)
pub fn figurine_result(is_cursed: bool, location_valid: bool) -> FigurineResult {
    if !location_valid {
        return FigurineResult::InvalidLocation;
    }
    if is_cursed {
        FigurineResult::CursedHostile
    } else {
        FigurineResult::SummonSuccess
    }
}

// =============================================================================
// [v2.19.0] 7. whip_use_result — 채찍 사용 결과
// 원본: apply.c use_whip() ~L2600
// =============================================================================

/// 채찍 사용 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhipResult {
    /// 무기 낚아채기 성공
    DisarmSuccess,
    /// 아이템 끌어오기 성공
    PullItem,
    /// 몬스터 끌어오기 (작은 몬스터)
    PullMonster,
    /// 자신이 끌려감 (큰 몬스터)
    PulledToward,
    /// 빗나감
    Miss,
    /// 자기 공격 (저주됨)
    SelfHit,
}

/// [v2.19.0] 채찍 사용 결과 (원본: use_whip ~L2600)
pub fn whip_use_result(
    is_cursed: bool,
    target_has_weapon: bool,
    target_size: i32, // 0=작, 1=중, 2=큰, 3=거대
    player_strength: i32,
    skill_level: i32, // 0~3
    rng: &mut NetHackRng,
) -> WhipResult {
    // 저주된 채찍: 1/5 확률로 자기 공격
    if is_cursed && rng.rn2(5) == 0 {
        return WhipResult::SelfHit;
    }

    // 명중 판정: skill + str/3 + rnd(20) > 12
    let hit_roll = skill_level + player_strength / 3 + rng.rnd(20);
    if hit_roll <= 12 {
        return WhipResult::Miss;
    }

    // 무기 낚아채기 (대상이 무기 보유)
    if target_has_weapon && rng.rn2(3) == 0 {
        return WhipResult::DisarmSuccess;
    }

    // 크기에 따라 끌기/끌려가기
    if target_size <= 1 {
        WhipResult::PullMonster
    } else {
        WhipResult::PulledToward
    }
}

// =============================================================================
// [v2.19.0] 8. polearm_range_check — 장창 사거리 판정
// 원본: apply.c get_valid_polearm_position() ~L2700
// =============================================================================

/// [v2.19.0] 장창 사거리 판정 (원본: L2700)
/// 장창은 2칸~ 범위에서만 사용 가능 (1칸 이내 불가)
pub fn polearm_range_check(dx: i32, dy: i32, max_range: i32) -> bool {
    let dist = dx.abs().max(dy.abs()); // 체비셰프 거리
    dist >= 2 && dist <= max_range
}

/// [v2.19.0] 장창 최대 사거리 (스킬에 따라)
pub fn polearm_max_range(skill_level: i32) -> i32 {
    match skill_level {
        0 => 4, // 미숙
        1 => 4, // 기본
        2 => 5, // 숙련
        _ => 8, // 전문
    }
}

// =============================================================================
// [v2.19.0] 9. cream_pie_result — 크림 파이 사용 결과
// 원본: apply.c use_cream_pie() ~L2800
// =============================================================================

/// 크림 파이 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreamPieResult {
    /// 자기 얼굴에 파이
    SelfBlind { turns: i32 },
    /// 몬스터에게 파이
    HitMonster { blind_turns: i32 },
    /// 빗나감
    Miss,
}

/// [v2.19.0] 크림 파이 결과 (원본: use_cream_pie ~L2800)
pub fn cream_pie_result(is_self: bool, has_target: bool, rng: &mut NetHackRng) -> CreamPieResult {
    if is_self {
        // 원본: rnd(25) 턴 눈 가려짐
        CreamPieResult::SelfBlind { turns: rng.rnd(25) }
    } else if has_target {
        CreamPieResult::HitMonster {
            blind_turns: rng.rnd(25),
        }
    } else {
        CreamPieResult::Miss
    }
}

// =============================================================================
// [v2.19.0] 10. grapple_result — 갈고리 사용 결과
// 원본: apply.c use_grapple() ~L3000
// =============================================================================

/// 갈고리 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrappleResult {
    /// 대상 끌어오기
    PullItem,
    /// 자신 끌기 (벽/가구 방향)
    PullSelf,
    /// 몬스터 공격
    HitMonster,
    /// 빗나감 / 대상 없음
    Miss,
    /// 장애물에 걸림
    Stuck,
}

/// [v2.19.0] 갈고리 결과 (원본: use_grapple ~L3000)
pub fn grapple_result(
    has_target_item: bool,
    has_target_monster: bool,
    has_target_terrain: bool,
    is_terrain_pullable: bool,
    skill_level: i32,
    rng: &mut NetHackRng,
) -> GrappleResult {
    // 명중 판정: skill + rnd(20) > 10
    let hit_roll = skill_level + rng.rnd(20);
    if hit_roll <= 10 {
        return GrappleResult::Miss;
    }

    if has_target_monster {
        return GrappleResult::HitMonster;
    }

    if has_target_item {
        return GrappleResult::PullItem;
    }

    if has_target_terrain {
        if is_terrain_pullable {
            GrappleResult::PullSelf
        } else {
            GrappleResult::Stuck
        }
    } else {
        GrappleResult::Miss
    }
}

// =============================================================================
// [v2.19.0] 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    // --- camera_flash ---
    #[test]
    fn test_camera_no_charges() {
        let mut rng = test_rng();
        let r = camera_flash_result(0, false, false, false, 1, 0, 0, false, &mut rng);
        assert_eq!(r, CameraFlashResult::NoCharges);
    }

    #[test]
    fn test_camera_stomach() {
        let mut rng = test_rng();
        let r = camera_flash_result(5, false, false, true, 1, 0, 0, false, &mut rng);
        assert_eq!(r, CameraFlashResult::StomachPhoto);
    }

    #[test]
    fn test_camera_hit_monster() {
        let mut rng = test_rng();
        let r = camera_flash_result(5, false, false, false, 1, 0, 0, true, &mut rng);
        assert_eq!(r, CameraFlashResult::HitMonster);
    }

    #[test]
    fn test_camera_self() {
        let mut rng = test_rng();
        let r = camera_flash_result(5, false, false, false, 0, 0, 0, false, &mut rng);
        assert_eq!(r, CameraFlashResult::SelfFlash);
    }

    // --- towel ---
    #[test]
    fn test_towel_no_hand() {
        let mut rng = test_rng();
        assert_eq!(
            towel_use_result(false, false, false, false, false, &mut rng),
            TowelResult::NoFreeHand
        );
    }

    #[test]
    fn test_towel_wearing() {
        let mut rng = test_rng();
        assert_eq!(
            towel_use_result(true, true, false, false, false, &mut rng),
            TowelResult::WearingIt
        );
    }

    #[test]
    fn test_towel_wipe_glib() {
        let mut rng = test_rng();
        assert_eq!(
            towel_use_result(true, false, false, true, false, &mut rng),
            TowelResult::WipedGlib
        );
    }

    #[test]
    fn test_towel_clean() {
        let mut rng = test_rng();
        assert_eq!(
            towel_use_result(true, false, false, false, false, &mut rng),
            TowelResult::AlreadyClean
        );
    }

    // --- leashable ---
    #[test]
    fn test_leashable_normal() {
        assert!(leashable_check(false, false, true, true));
    }

    #[test]
    fn test_leashable_long_worm() {
        assert!(!leashable_check(true, false, true, true));
    }

    #[test]
    fn test_leashable_unsolid() {
        assert!(!leashable_check(false, true, true, true));
    }

    #[test]
    fn test_leashable_no_limbs_no_head() {
        assert!(!leashable_check(false, false, false, false));
    }

    // --- jump_range ---
    #[test]
    fn test_jump_no_ability() {
        assert_eq!(jump_range(false, 0, 0), 0);
    }

    #[test]
    fn test_jump_boots_basic() {
        assert_eq!(jump_range(true, 1, 0), 2);
    }

    #[test]
    fn test_jump_boots_expert() {
        assert_eq!(jump_range(true, 3, 0), 4);
    }

    #[test]
    fn test_jump_encumbered() {
        assert_eq!(jump_range(true, 3, 2), 2);
    }

    // --- tinning ---
    #[test]
    fn test_tinning_no_charges() {
        let mut rng = test_rng();
        assert_eq!(
            tinning_result(false, true, false, 1, &mut rng),
            TinningResult::NoCharges
        );
    }

    #[test]
    fn test_tinning_blessed() {
        let mut rng = test_rng();
        assert_eq!(
            tinning_result(true, true, true, 2, &mut rng),
            TinningResult::BlessedInstant
        );
    }

    #[test]
    fn test_tinning_normal() {
        let mut rng = test_rng();
        match tinning_result(true, true, false, 1, &mut rng) {
            TinningResult::Success { turns_needed } => {
                assert!(
                    turns_needed >= 4 && turns_needed <= 8,
                    "턴: {}",
                    turns_needed
                );
            }
            _ => panic!("정상 통조림 실패"),
        }
    }

    // --- figurine ---
    #[test]
    fn test_figurine_success() {
        assert_eq!(figurine_result(false, true), FigurineResult::SummonSuccess);
    }

    #[test]
    fn test_figurine_invalid() {
        assert_eq!(
            figurine_result(false, false),
            FigurineResult::InvalidLocation
        );
    }

    #[test]
    fn test_figurine_cursed() {
        assert_eq!(figurine_result(true, true), FigurineResult::CursedHostile);
    }

    // --- whip ---
    #[test]
    fn test_whip_miss() {
        // 낮은 힘/스킬로 빗나감 확인
        let mut rng = NetHackRng::new(999);
        let mut misses = 0;
        for _ in 0..100 {
            if matches!(
                whip_use_result(false, false, 1, 5, 0, &mut rng),
                WhipResult::Miss
            ) {
                misses += 1;
            }
        }
        assert!(misses > 0, "빗나감 있어야 함");
    }

    // --- polearm ---
    #[test]
    fn test_polearm_too_close() {
        assert!(!polearm_range_check(1, 0, 4));
    }

    #[test]
    fn test_polearm_in_range() {
        assert!(polearm_range_check(2, 0, 4));
    }

    #[test]
    fn test_polearm_too_far() {
        assert!(!polearm_range_check(5, 0, 4));
    }

    #[test]
    fn test_polearm_max_range() {
        assert_eq!(polearm_max_range(0), 4);
        assert_eq!(polearm_max_range(2), 5);
        assert_eq!(polearm_max_range(3), 8);
    }

    // --- cream_pie ---
    #[test]
    fn test_pie_self() {
        let mut rng = test_rng();
        match cream_pie_result(true, false, &mut rng) {
            CreamPieResult::SelfBlind { turns } => {
                assert!(turns >= 1 && turns <= 25, "눈가림 턴: {}", turns);
            }
            _ => panic!("자기 파이"),
        }
    }

    #[test]
    fn test_pie_hit() {
        let mut rng = test_rng();
        match cream_pie_result(false, true, &mut rng) {
            CreamPieResult::HitMonster { blind_turns } => {
                assert!(blind_turns >= 1 && blind_turns <= 25);
            }
            _ => panic!("몬스터 파이"),
        }
    }

    #[test]
    fn test_pie_miss() {
        let mut rng = test_rng();
        assert_eq!(
            cream_pie_result(false, false, &mut rng),
            CreamPieResult::Miss
        );
    }

    // --- grapple ---
    #[test]
    fn test_grapple_monster() {
        let mut rng = test_rng();
        // 높은 스킬로 명중 보장
        let r = grapple_result(false, true, false, false, 10, &mut rng);
        assert_eq!(r, GrappleResult::HitMonster);
    }

    #[test]
    fn test_grapple_item() {
        let mut rng = test_rng();
        let r = grapple_result(true, false, false, false, 10, &mut rng);
        assert_eq!(r, GrappleResult::PullItem);
    }

    #[test]
    fn test_grapple_terrain_pull() {
        let mut rng = test_rng();
        let r = grapple_result(false, false, true, true, 10, &mut rng);
        assert_eq!(r, GrappleResult::PullSelf);
    }

    #[test]
    fn test_grapple_terrain_stuck() {
        let mut rng = test_rng();
        let r = grapple_result(false, false, true, false, 10, &mut rng);
        assert_eq!(r, GrappleResult::Stuck);
    }
}

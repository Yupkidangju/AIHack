// detect_ext.rs — detect.c 핵심 로직 순수 결과 패턴 이식
// [v2.12.0] 신규 생성: 레벨 거리/수정구 실패/탐색/해독 등 12개 함수
// 원본: NetHack 3.6.7 src/detect.c (2,033줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 열거형
// ============================================================

/// 레벨 거리 표현 (level_distance)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelDistanceDesc {
    /// 같은 던전에서 바로 위
    JustAbove,
    /// 같은 던전에서 위
    Above,
    /// 같은 던전에서 먼 위
    FarAbove,
    /// 같은 던전에서 바로 아래
    JustBelow,
    /// 같은 던전에서 아래
    Below,
    /// 같은 던전에서 먼 아래
    FarBelow,
    /// 같은 위치 (같은 던전)
    NearYou,
    /// 먼 거리 (다른 던전)
    InTheDistance,
    /// 다른 던전에서 위
    AwayAbove,
    /// 다른 던전에서 아래
    AwayBelow,
    /// 다른 던전에서 먼 위
    FarAway,
}

/// 수정구 실패 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrystalBallOopsEffect {
    /// "너무 복잡해서 이해할 수 없음"
    TooMuchComprehend,
    /// 혼란
    Confused { duration: i32 },
    /// 시야 손상 (실명 저항 없으면)
    VisionDamaged { duration: i32, resisted: bool },
    /// 환각
    Hallucinated { duration: i32 },
    /// 폭발 (비아티팩트만)
    Explode { damage: i32 },
}

/// 수정구 환각 메시지 인덱스
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrystalHallucMsg {
    GroovyGlobs,
    PsychedelicColors,
    SinisterLight,
    GoldfishRocks,
    Snowflakes,
    Kaleidoscope,
}

/// 탐지 유형 (무엇을 탐지하는지)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionType {
    Gold,
    Food,
    Object,
    Monster,
    Trap,
}

/// 탐지 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionResult {
    /// 아무것도 탐지되지 않음
    NothingDetected,
    /// 플레이어 위치에서만 탐지
    OnlyUnderPlayer,
    /// 떨어진 곳에서 탐지 (맵 표시 필요)
    DistantDetection { count: i32 },
}

/// 수색 성공 판정에 필요한 입력
#[derive(Debug, Clone)]
pub struct SearchInput {
    /// 플레이어 레벨
    pub player_level: i32,
    /// 행운 보정치
    pub luck: i32,
    /// 수색 술 장착 여부 (lenses 등)
    pub has_search_tool: bool,
    /// 탐지 마법 효과 활성 여부
    pub has_searching: bool,
}

/// 함정 탐지 위치
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjTrapLocation {
    /// 아무것도 없음
    None,
    /// 플레이어 위치
    Here,
    /// 다른 위치
    There,
    /// 양쪽 다
    Both,
}

// ============================================================
// 1. level_distance_desc — 레벨 거리 설명 문자열 결정
// ============================================================

/// 현재 레벨과 대상 레벨 사이의 거리 설명 결정
/// 원본: detect.c level_distance()
/// depth_diff: depth(&u.uz) - depth(where), positive = 대상이 위에 있음
/// in_same_dungeon: 같은 던전인지
pub fn level_distance_desc(
    depth_diff: i32,
    in_same_dungeon: bool,
    rng: &mut NetHackRng,
) -> LevelDistanceDesc {
    let threshold = 8 + rng.rn2(3);

    if depth_diff < 0 {
        // 대상이 아래에 있음
        if depth_diff < -threshold {
            if !in_same_dungeon {
                LevelDistanceDesc::FarAway
            } else {
                LevelDistanceDesc::FarBelow
            }
        } else if depth_diff < -1 {
            if !in_same_dungeon {
                LevelDistanceDesc::AwayBelow
            } else {
                LevelDistanceDesc::Below
            }
        } else if !in_same_dungeon {
            LevelDistanceDesc::InTheDistance
        } else {
            LevelDistanceDesc::JustBelow
        }
    } else if depth_diff > 0 {
        // 대상이 위에 있음
        if depth_diff > threshold {
            if !in_same_dungeon {
                LevelDistanceDesc::FarAway
            } else {
                LevelDistanceDesc::FarAbove
            }
        } else if depth_diff > 1 {
            if !in_same_dungeon {
                LevelDistanceDesc::AwayAbove
            } else {
                LevelDistanceDesc::Above
            }
        } else if !in_same_dungeon {
            LevelDistanceDesc::InTheDistance
        } else {
            LevelDistanceDesc::JustAbove
        }
    } else if !in_same_dungeon {
        LevelDistanceDesc::InTheDistance
    } else {
        LevelDistanceDesc::NearYou
    }
}

/// level_distance_desc 결과를 문자열로 변환
pub fn level_distance_str(desc: LevelDistanceDesc) -> &'static str {
    match desc {
        LevelDistanceDesc::JustAbove => "just above",
        LevelDistanceDesc::Above => "above you",
        LevelDistanceDesc::FarAbove => "far above",
        LevelDistanceDesc::JustBelow => "just below",
        LevelDistanceDesc::Below => "below you",
        LevelDistanceDesc::FarBelow => "far below",
        LevelDistanceDesc::NearYou => "near you",
        LevelDistanceDesc::InTheDistance => "in the distance",
        LevelDistanceDesc::AwayAbove => "away above you",
        LevelDistanceDesc::AwayBelow => "away below you",
        LevelDistanceDesc::FarAway => "far away",
    }
}

// ============================================================
// 2. crystal_ball_oops — 수정구 사용 실패 효과
// ============================================================

/// 수정구 oops 효과 결정 (spe > 0인 상태에서 실패 시)
/// 원본: detect.c use_crystal_ball() 내 oops 분기
/// is_artifact: 아티팩트 여부 (4가지 vs 5가지 결과)
pub fn crystal_ball_oops(is_artifact: bool, rng: &mut NetHackRng) -> CrystalBallOopsEffect {
    let max_roll = if is_artifact { 4 } else { 5 };
    let roll = rng.rnd(max_roll);

    match roll {
        1 => CrystalBallOopsEffect::TooMuchComprehend,
        2 => CrystalBallOopsEffect::Confused {
            duration: rng.rnd(100),
        },
        3 => CrystalBallOopsEffect::VisionDamaged {
            duration: rng.rnd(100),
            resisted: false, // 호출자가 저항 여부 확인
        },
        4 => CrystalBallOopsEffect::Hallucinated {
            duration: rng.rnd(100),
        },
        _ => CrystalBallOopsEffect::Explode {
            damage: rng.rnd(30),
        },
    }
}

// ============================================================
// 3. crystal_ball_halluc_msg — 환각 수정구 메시지
// ============================================================

/// 환각 상태에서 수정구를 볼 때의 메시지 결정
/// 원본: detect.c use_crystal_ball() 내 Hallucination 분기
pub fn crystal_ball_halluc_msg(rng: &mut NetHackRng) -> CrystalHallucMsg {
    match rng.rnd(6) {
        1 => CrystalHallucMsg::GroovyGlobs,
        2 => CrystalHallucMsg::PsychedelicColors,
        3 => CrystalHallucMsg::SinisterLight,
        4 => CrystalHallucMsg::GoldfishRocks,
        5 => CrystalHallucMsg::Snowflakes,
        _ => CrystalHallucMsg::Kaleidoscope,
    }
}

// ============================================================
// 4. crystal_ball_oops_check — 수정구 실패 판정
// ============================================================

/// 수정구 사용 시 실패(oops) 여부 판정
/// 원본: oops = (rnd(20) > ACURR(A_INT) || obj->cursed)
pub fn crystal_ball_oops_check(intelligence: i32, is_cursed: bool, rng: &mut NetHackRng) -> bool {
    rng.rnd(20) > intelligence || is_cursed
}

// ============================================================
// 5. crystal_ball_gaze_delay — 수정구 응시 지연
// ============================================================

/// 수정구 응시에 소요되는 턴 수 반환
/// 원본: nomul(-rnd(10))
pub fn crystal_ball_gaze_delay(rng: &mut NetHackRng) -> i32 {
    rng.rnd(10)
}

// ============================================================
// 6. search_success_chance — 수색 성공 확률 판정
// ============================================================

/// 주변 수색 시 비밀문/함정 발견 확률 계산
/// 원본: detect.c dosearch0() 내 판정 로직
/// 반환: 성공에 필요한 rn2 범위 (작을수록 높은 확률)
pub fn search_chance(player_level: i32, luck: i32, has_search_tool: bool) -> i32 {
    // 원본: rn2(7 - (level/6 + luck)) → 범위가 0이면 자동 성공
    let mut chance = 7 - (player_level / 6 + luck);

    // 수색 도구(렌즈 등)가 있으면 2 감소
    if has_search_tool {
        chance -= 2;
    }

    // 최소 1 (0이면 자동 성공이므로)
    chance.max(1)
}

/// 수색 성공 여부 판정
/// 원본: !rn2(chance)
pub fn search_succeeds(chance: i32, rng: &mut NetHackRng) -> bool {
    if chance <= 0 {
        return true; // 자동 성공
    }
    rng.rn2(chance) == 0
}

// ============================================================
// 7. trapped_chest_glyph_check — 함정 상자 글리프 판정
// ============================================================

/// 함정 글리프가 함정 상자를 나타내는지 판정
/// 원본: detect.c trapped_chest_at() 전반부
pub fn trapped_chest_glyph_check(
    trap_type: i32,
    is_hallucinating: bool,
    rng: &mut NetHackRng,
) -> bool {
    const BEAR_TRAP: i32 = 4; // 곰 함정

    // 곰 함정이 아니면 상자 함정이 아님
    if trap_type != BEAR_TRAP {
        return false;
    }
    // 환각 시 95% 확률로 상자 함정이 아님
    if is_hallucinating && rng.rn2(20) != 0 {
        return false;
    }
    true
}

// ============================================================
// 8. sense_trap_object — 함정 감지 시 가짜 오브젝트 유형 결정
// ============================================================

/// 환각/저주 상태에서 함정 감지 시 어떤 오브젝트로 표시할지 결정
/// 원본: detect.c sense_trap() 내 Hallucination / src_cursed 분기
/// 반환: (is_gold, quantity) — is_gold이면 금 표시
pub fn sense_trap_fake_gold_quantity(
    is_hallucinating: bool,
    is_gold: bool,
    rng: &mut NetHackRng,
) -> i32 {
    if !is_hallucinating && is_gold {
        // 저주 → 금화
        rng.rnd(10)
    } else if is_gold {
        rng.rnd(10)
    } else {
        // 비금 오브젝트: merge 가능하면 rnd(2), 아니면 1
        // 여기서는 단순화: rnd(2)
        rng.rnd(2)
    }
}

// ============================================================
// 9. obj_trap_combine — 오브젝트 함정 위치 결합
// ============================================================

/// 두 개의 ObjTrapLocation을 결합 (비트 OR 연산)
/// 원본: detect.c detect_obj_traps() 내 result |= 로직
pub fn obj_trap_combine(a: ObjTrapLocation, b: ObjTrapLocation) -> ObjTrapLocation {
    let va = match a {
        ObjTrapLocation::None => 0,
        ObjTrapLocation::Here => 1,
        ObjTrapLocation::There => 2,
        ObjTrapLocation::Both => 3,
    };
    let vb = match b {
        ObjTrapLocation::None => 0,
        ObjTrapLocation::Here => 1,
        ObjTrapLocation::There => 2,
        ObjTrapLocation::Both => 3,
    };
    match va | vb {
        0 => ObjTrapLocation::None,
        1 => ObjTrapLocation::Here,
        2 => ObjTrapLocation::There,
        _ => ObjTrapLocation::Both,
    }
}

// ============================================================
// 10. food_detect_class — 음식 탐지 시 실제 탐지 클래스
// ============================================================

/// 혼란/저주 시 음식 탐지가 물약 탐지로 바뀜
/// 원본: food_detect() 내 confused → POTION_CLASS
pub fn food_detect_class(confused: bool, is_cursed: bool) -> &'static str {
    if confused || is_cursed {
        "potion" // 혼란/저주 시 물약 탐지
    } else {
        "food"
    }
}

// ============================================================
// 11. gold_detect_message — 금 탐지 실패 시 메시지 분기
// ============================================================

/// 금 탐지 실패 시 적절한 메시지 카테고리 결정
/// 원본: detect.c gold_detect() 내 !known 분기
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoldDetectFailMsg {
    /// 골드 골렘 형태
    GoldGolem,
    /// 인벤토리에 금이 있음
    HasGold,
    /// 탈것에 금이 있음
    SteedHasGold,
    /// 금이 없음
    NothingAtAll,
}

pub fn gold_detect_fail_message(
    is_gold_golem: bool,
    has_own_gold: bool,
    steed_has_gold: bool,
) -> GoldDetectFailMsg {
    if is_gold_golem {
        GoldDetectFailMsg::GoldGolem
    } else if has_own_gold {
        GoldDetectFailMsg::HasGold
    } else if steed_has_gold {
        GoldDetectFailMsg::SteedHasGold
    } else {
        GoldDetectFailMsg::NothingAtAll
    }
}

// ============================================================
// 12. monster_detect_wakeup — 저주 몬스터 탐지 시 각성
// ============================================================

/// 저주 몬스터 탐지 시 잠든/마비 몬스터를 깨울지 판정
/// 원본: detect.c monster_detect() 내 otmp->cursed 분기
pub fn monster_detect_wakeup(is_cursed: bool, is_sleeping: bool, cant_move: bool) -> bool {
    is_cursed && (is_sleeping || cant_move)
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

    // --- level_distance_desc ---
    #[test]
    fn test_level_distance_same_level() {
        let mut rng = test_rng();
        let desc = level_distance_desc(0, true, &mut rng);
        assert_eq!(desc, LevelDistanceDesc::NearYou);
    }

    #[test]
    fn test_level_distance_just_below() {
        let mut rng = test_rng();
        let desc = level_distance_desc(-1, true, &mut rng);
        assert_eq!(desc, LevelDistanceDesc::JustBelow);
    }

    #[test]
    fn test_level_distance_far_below() {
        let mut rng = test_rng();
        let desc = level_distance_desc(-15, true, &mut rng);
        assert_eq!(desc, LevelDistanceDesc::FarBelow);
    }

    #[test]
    fn test_level_distance_different_dungeon() {
        let mut rng = test_rng();
        let desc = level_distance_desc(0, false, &mut rng);
        assert_eq!(desc, LevelDistanceDesc::InTheDistance);
    }

    #[test]
    fn test_level_distance_far_above_other() {
        let mut rng = test_rng();
        let desc = level_distance_desc(15, false, &mut rng);
        assert_eq!(desc, LevelDistanceDesc::FarAway);
    }

    // --- level_distance_str ---
    #[test]
    fn test_distance_str() {
        assert_eq!(level_distance_str(LevelDistanceDesc::NearYou), "near you");
        assert_eq!(level_distance_str(LevelDistanceDesc::FarBelow), "far below");
        assert_eq!(
            level_distance_str(LevelDistanceDesc::InTheDistance),
            "in the distance"
        );
    }

    // --- crystal_ball_oops ---
    #[test]
    fn test_crystal_ball_oops_effects() {
        let mut rng = test_rng();
        let effect = crystal_ball_oops(false, &mut rng);
        match effect {
            CrystalBallOopsEffect::TooMuchComprehend
            | CrystalBallOopsEffect::Confused { .. }
            | CrystalBallOopsEffect::VisionDamaged { .. }
            | CrystalBallOopsEffect::Hallucinated { .. }
            | CrystalBallOopsEffect::Explode { .. } => {}
        }
    }

    #[test]
    fn test_crystal_ball_artifact_no_explode() {
        // 아티팩트는 폭발 안 함 (roll 1~4만)
        let mut rng = test_rng();
        for _ in 0..50 {
            let effect = crystal_ball_oops(true, &mut rng);
            if let CrystalBallOopsEffect::Explode { .. } = effect {
                panic!("아티팩트 수정구는 폭발하면 안 됨");
            }
        }
    }

    // --- crystal_ball_oops_check ---
    #[test]
    fn test_oops_check_cursed() {
        let mut rng = test_rng();
        assert!(
            crystal_ball_oops_check(25, true, &mut rng),
            "저주면 항상 실패"
        );
    }

    #[test]
    fn test_oops_check_high_int() {
        let mut rng = test_rng();
        // 지능 25, 비저주 → rnd(20) > 25는 불가능
        let oops = crystal_ball_oops_check(25, false, &mut rng);
        assert!(!oops, "지능 25는 실패 불가");
    }

    // --- crystal_ball_gaze_delay ---
    #[test]
    fn test_gaze_delay() {
        let mut rng = test_rng();
        let delay = crystal_ball_gaze_delay(&mut rng);
        assert!(delay >= 1 && delay <= 10);
    }

    // --- search_chance ---
    #[test]
    fn test_search_chance_base() {
        let chance = search_chance(1, 0, false);
        assert_eq!(chance, 7); // 7 - (1/6 + 0) = 7
    }

    #[test]
    fn test_search_chance_high_level() {
        let chance = search_chance(30, 3, true);
        // 7 - (30/6 + 3) - 2 = 7 - 8 - 2 = -3 → max(1)
        assert_eq!(chance, 1);
    }

    #[test]
    fn test_search_succeeds_auto() {
        let mut rng = test_rng();
        assert!(search_succeeds(0, &mut rng), "chance 0 → 자동 성공");
    }

    // --- trapped_chest_glyph_check ---
    #[test]
    fn test_trapped_chest_bear_trap() {
        let mut rng = test_rng();
        assert!(trapped_chest_glyph_check(4, false, &mut rng));
    }

    #[test]
    fn test_trapped_chest_not_bear_trap() {
        let mut rng = test_rng();
        assert!(!trapped_chest_glyph_check(1, false, &mut rng));
    }

    // --- obj_trap_combine ---
    #[test]
    fn test_combine_none() {
        assert_eq!(
            obj_trap_combine(ObjTrapLocation::None, ObjTrapLocation::None),
            ObjTrapLocation::None
        );
    }

    #[test]
    fn test_combine_here_there() {
        assert_eq!(
            obj_trap_combine(ObjTrapLocation::Here, ObjTrapLocation::There),
            ObjTrapLocation::Both
        );
    }

    // --- food_detect_class ---
    #[test]
    fn test_food_normal() {
        assert_eq!(food_detect_class(false, false), "food");
    }

    #[test]
    fn test_food_confused() {
        assert_eq!(food_detect_class(true, false), "potion");
    }

    // --- gold_detect_fail_message ---
    #[test]
    fn test_gold_fail_golem() {
        assert_eq!(
            gold_detect_fail_message(true, false, false),
            GoldDetectFailMsg::GoldGolem
        );
    }

    #[test]
    fn test_gold_fail_nothing() {
        assert_eq!(
            gold_detect_fail_message(false, false, false),
            GoldDetectFailMsg::NothingAtAll
        );
    }

    // --- monster_detect_wakeup ---
    #[test]
    fn test_wakeup_cursed_sleeping() {
        assert!(monster_detect_wakeup(true, true, false));
    }

    #[test]
    fn test_wakeup_not_cursed() {
        assert!(!monster_detect_wakeup(false, true, true));
    }

    // --- crystal_ball_halluc_msg ---
    #[test]
    fn test_halluc_msg() {
        let mut rng = test_rng();
        let msg = crystal_ball_halluc_msg(&mut rng);
        // 어떤 메시지든 유효
        match msg {
            CrystalHallucMsg::GroovyGlobs
            | CrystalHallucMsg::PsychedelicColors
            | CrystalHallucMsg::SinisterLight
            | CrystalHallucMsg::GoldfishRocks
            | CrystalHallucMsg::Snowflakes
            | CrystalHallucMsg::Kaleidoscope => {}
        }
    }
}

// timeout_ext.rs — timeout.c 핵심 로직 순수 결과 패턴 이식
// [v2.17.0] 신규 생성: 상태이상 대화/카운트다운, 행운 감쇠, 랜턴/양초 연소, 알 부화 등 12개 함수
// 원본: NetHack 3.6.7 src/timeout.c (2,429줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 최대 알 부화 시간
pub const MAX_EGG_HATCH_TIME: i32 = 200;

/// 램프/등불 연소 임계값
pub const LAMP_THRESHOLDS: &[i32] = &[150, 100, 50, 25, 0];
/// 양초 연소 임계값
pub const CANDLE_THRESHOLDS: &[i32] = &[75, 15, 0];
/// 석화 카운트다운 단계 (5..1)
pub const STONED_STAGES: i32 = 5;
/// 구토 카운트다운 단계 (14→0)
pub const VOMITING_MAX: i32 = 14;

// ============================================================
// 열거형
// ============================================================

/// 석화 진행 단계 메시지
/// 원본: timeout.c stoned_dialogue() L102-108
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StonedStage {
    SlowingDown,      // 5
    LimbsStiffening,  // 4
    LimbsTurnedStone, // 3
    TurnedToStone,    // 2
    Statue,           // 1
    None,
}

/// 구토 진행 단계
/// 원본: timeout.c vomiting_dialogue() L162-168
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VomitingStage {
    MildlyNauseated,  // 14
    SlightlyConfused, // 11
    CantThink,        // 8
    IncrediblySick,   // 5
    AboutToVomit,     // 2
    Vomit,            // 0
    Transition,       // 기타 (혼란/기절 적용)
    None,
}

/// 질식 진행 단계
/// 원본: timeout.c choke_dialogue() L235-249
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChokeStage {
    HardToBreathe,   // 5
    GaspingForAir,   // 4
    NoLongerBreathe, // 3
    TurningColor,    // 2
    Suffocate,       // 1
    None,
}

/// 랜턴/양초 연소 단계
/// 원본: timeout.c burn_object() L1129-1408
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurnStage {
    /// 깜빡임 (age=150,100,50)
    Flickering,
    /// 상당히 깜빡임 (age=50)
    FlickeringConsiderably,
    /// 꺼지려 함 (age=25)
    AboutToGoOut,
    /// 꺼짐 (age=0)
    BurnedOut,
    /// 양초가 짧아짐 (age=75)
    GettingShort,
    /// 양초 불꽃 약해짐 (age=15)
    FlameFlickersLow,
    /// 이상 없음
    Normal,
}

/// 행운 감쇠 방향
/// 원본: timeout.c nh_timeout() L491-505
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuckDecay {
    Decrease,
    Increase,
    NoChange,
}

/// 알 부화 시간 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EggHatchResult {
    /// 부화 시간 결정됨
    HatchAt(i32),
    /// 부화 불가 (확률상 실패)
    NoHatch,
}

// ============================================================
// 1. stoned_stage — 석화 카운트다운 단계 판정
// ============================================================

/// 석화 타이머에서 현재 단계 메시지 결정
/// 원본: timeout.c stoned_dialogue() L110-159
pub fn stoned_stage(timer_remaining: i32) -> StonedStage {
    match timer_remaining {
        5 => StonedStage::SlowingDown,
        4 => StonedStage::LimbsStiffening,
        3 => StonedStage::LimbsTurnedStone,
        2 => StonedStage::TurnedToStone,
        1 => StonedStage::Statue,
        _ => StonedStage::None,
    }
}

// ============================================================
// 2. vomiting_stage — 구토 카운트다운 단계 판정
// ============================================================

/// 구토 타이머에서 현재 단계 메시지 결정
/// 원본: timeout.c vomiting_dialogue() L170-233
/// [주의] timer에서 -1 하여 전달해야 함 (nhtimeout이 아직 감소 전)
pub fn vomiting_stage(timer_minus_one: i32) -> VomitingStage {
    match timer_minus_one {
        14 => VomitingStage::MildlyNauseated,
        11 => VomitingStage::SlightlyConfused,
        8 => VomitingStage::CantThink,
        5 => VomitingStage::IncrediblySick,
        2 => VomitingStage::AboutToVomit,
        0 => VomitingStage::Vomit,
        6 | 9 => VomitingStage::Transition,
        _ => VomitingStage::None,
    }
}

// ============================================================
// 3. choke_stage — 질식 카운트다운 단계 판정
// ============================================================

/// 질식(Strangled) 타이머에서 현재 단계 메시지 결정
/// 원본: timeout.c choke_dialogue() L251-269
pub fn choke_stage(timer_remaining: i32) -> ChokeStage {
    match timer_remaining {
        5 => ChokeStage::HardToBreathe,
        4 => ChokeStage::GaspingForAir,
        3 => ChokeStage::NoLongerBreathe,
        2 => ChokeStage::TurningColor,
        1 => ChokeStage::Suffocate,
        _ => ChokeStage::None,
    }
}

// ============================================================
// 4. luck_decay — 행운 감쇠 방향 결정
// ============================================================

/// 행운 타이머 감쇠 방향 결정
/// 원본: timeout.c nh_timeout() L491-505
pub fn luck_decay(
    current_luck: i32,
    base_luck: i32,
    has_luckstone: bool,
    stone_luck_negative: bool,
    stone_luck_positive: bool,
) -> LuckDecay {
    let no_stone = !has_luckstone && !stone_luck_negative && !stone_luck_positive;

    if current_luck > base_luck && (no_stone || stone_luck_negative) {
        LuckDecay::Decrease
    } else if current_luck < base_luck && (no_stone || stone_luck_positive) {
        LuckDecay::Increase
    } else {
        LuckDecay::NoChange
    }
}

// ============================================================
// 5. luck_decay_interval — 행운 감쇠 주기
// ============================================================

/// 행운이 감쇠하는 턴 간격
/// 원본: timeout.c nh_timeout() L492
pub fn luck_decay_interval(has_amulet: bool, has_gods_anger: bool) -> i32 {
    if has_amulet || has_gods_anger {
        300
    } else {
        600
    }
}

// ============================================================
// 6. base_luck — 기본 행운값 계산
// ============================================================

/// 달과 금요일13에 따른 기본 행운값
/// 원본: timeout.c nh_timeout() L486-489
pub fn base_luck(is_full_moon: bool, is_friday_13: bool) -> i32 {
    let mut base = if is_full_moon { 1 } else { 0 };
    if is_friday_13 {
        base -= 1;
    }
    base
}

// ============================================================
// 7. lamp_burn_stage — 랜턴/오일 램프 연소 단계
// ============================================================

/// 랜턴/오일 램프의 연소 age에 따른 단계
/// 원본: timeout.c burn_object() L1213-1267
pub fn lamp_burn_stage(age: i32) -> BurnStage {
    match age {
        150 | 100 => BurnStage::Flickering,
        50 => BurnStage::FlickeringConsiderably,
        25 => BurnStage::AboutToGoOut,
        0 => BurnStage::BurnedOut,
        _ => BurnStage::Normal,
    }
}

// ============================================================
// 8. candle_burn_stage — 양초 연소 단계
// ============================================================

/// 양초의 연소 age에 따른 단계
/// 원본: timeout.c burn_object() L1285-1398
pub fn candle_burn_stage(age: i32) -> BurnStage {
    match age {
        75 => BurnStage::GettingShort,
        15 => BurnStage::FlameFlickersLow,
        0 => BurnStage::BurnedOut,
        _ => BurnStage::Normal,
    }
}

// ============================================================
// 9. lamp_next_turns — 랜턴 다음 이벤트까지 남은 턴
// ============================================================

/// 램프/등불의 현재 age에서 다음 이벤트까지 연소 턴 수
/// 원본: timeout.c begin_burn() L1464-1477
pub fn lamp_next_turns(age: i32) -> i32 {
    if age > 150 {
        age - 150
    } else if age > 100 {
        age - 100
    } else if age > 50 {
        age - 50
    } else if age > 25 {
        age - 25
    } else {
        age
    }
}

// ============================================================
// 10. candle_next_turns — 양초 다음 이벤트까지 남은 턴
// ============================================================

/// 양초의 현재 age에서 다음 이벤트까지 연소 턴 수
/// 원본: timeout.c begin_burn() L1483-1489
pub fn candle_next_turns(age: i32) -> i32 {
    if age > 75 {
        age - 75
    } else if age > 15 {
        age - 15
    } else {
        age
    }
}

// ============================================================
// 11. egg_hatch_time — 알 부화 시간 결정
// ============================================================

/// 알 부화 시간 확률적 결정
/// 원본: timeout.c attach_egg_hatch_timeout() L762-789
/// 151부터 200까지 시도하여 rnd(i) > 150이면 부화
pub fn egg_hatch_time(rng: &mut NetHackRng) -> EggHatchResult {
    for i in (MAX_EGG_HATCH_TIME - 50 + 1)..=MAX_EGG_HATCH_TIME {
        if rng.rnd(i) > 150 {
            return EggHatchResult::HatchAt(i);
        }
    }
    EggHatchResult::NoHatch
}

// ============================================================
// 12. fig_transform_time — 인형 변신 시간
// ============================================================

/// 인형의 변신 시간 결정
/// 원본: timeout.c attach_fig_transform_timeout() L978-994
pub fn fig_transform_time(rng: &mut NetHackRng) -> i32 {
    rng.rnd(9000) + 200
}

// ============================================================
// 13. oil_diluted_turns — 희석된 기름 연소 시간
// ============================================================

/// 희석된 기름 물약의 연소 시간 보정
/// 원본: timeout.c begin_burn() L1459-1460
pub fn oil_diluted_turns(age: i32, is_diluted: bool) -> i32 {
    if is_diluted {
        (3 * age + 2) / 4
    } else {
        age
    }
}

// ============================================================
// 14. storm_strikes — 폭풍 번개 횟수
// ============================================================

/// 에어 레벨 폭풍에서 번개 타격 횟수 결정
/// 원본: timeout.c do_storms() L1594
/// nstrike = rnd(64), 횟수 = 7 - log2(nstrike) (nstrike가 64 이하면서 2배씩 증가)
pub fn storm_strike_count(rng: &mut NetHackRng) -> i32 {
    let start = rng.rnd(64);
    let mut count = 0;
    let mut n = start;
    while n <= 64 {
        count += 1;
        n *= 2;
    }
    count
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

    // --- stoned_stage ---
    #[test]
    fn test_stoned_stages() {
        assert_eq!(stoned_stage(5), StonedStage::SlowingDown);
        assert_eq!(stoned_stage(4), StonedStage::LimbsStiffening);
        assert_eq!(stoned_stage(3), StonedStage::LimbsTurnedStone);
        assert_eq!(stoned_stage(2), StonedStage::TurnedToStone);
        assert_eq!(stoned_stage(1), StonedStage::Statue);
        assert_eq!(stoned_stage(0), StonedStage::None);
        assert_eq!(stoned_stage(10), StonedStage::None);
    }

    // --- vomiting_stage ---
    #[test]
    fn test_vomiting_stages() {
        assert_eq!(vomiting_stage(14), VomitingStage::MildlyNauseated);
        assert_eq!(vomiting_stage(11), VomitingStage::SlightlyConfused);
        assert_eq!(vomiting_stage(8), VomitingStage::CantThink);
        assert_eq!(vomiting_stage(5), VomitingStage::IncrediblySick);
        assert_eq!(vomiting_stage(2), VomitingStage::AboutToVomit);
        assert_eq!(vomiting_stage(0), VomitingStage::Vomit);
        assert_eq!(vomiting_stage(6), VomitingStage::Transition);
        assert_eq!(vomiting_stage(9), VomitingStage::Transition);
        assert_eq!(vomiting_stage(7), VomitingStage::None);
    }

    // --- choke_stage ---
    #[test]
    fn test_choke_stages() {
        assert_eq!(choke_stage(5), ChokeStage::HardToBreathe);
        assert_eq!(choke_stage(4), ChokeStage::GaspingForAir);
        assert_eq!(choke_stage(3), ChokeStage::NoLongerBreathe);
        assert_eq!(choke_stage(2), ChokeStage::TurningColor);
        assert_eq!(choke_stage(1), ChokeStage::Suffocate);
        assert_eq!(choke_stage(0), ChokeStage::None);
    }

    // --- luck_decay ---
    #[test]
    fn test_luck_decrease() {
        let r = luck_decay(3, 0, false, false, false);
        assert_eq!(r, LuckDecay::Decrease);
    }

    #[test]
    fn test_luck_increase() {
        let r = luck_decay(-2, 0, false, false, false);
        assert_eq!(r, LuckDecay::Increase);
    }

    #[test]
    fn test_luck_no_change_stone() {
        // 축복받은 행운석: 좋은 행운 유지
        let r = luck_decay(3, 0, true, false, true);
        assert_eq!(r, LuckDecay::NoChange);
    }

    #[test]
    fn test_luck_no_change_equal() {
        let r = luck_decay(0, 0, false, false, false);
        assert_eq!(r, LuckDecay::NoChange);
    }

    // --- luck_decay_interval ---
    #[test]
    fn test_luck_interval_normal() {
        assert_eq!(luck_decay_interval(false, false), 600);
    }

    #[test]
    fn test_luck_interval_amulet() {
        assert_eq!(luck_decay_interval(true, false), 300);
    }

    // --- base_luck ---
    #[test]
    fn test_base_luck_normal() {
        assert_eq!(base_luck(false, false), 0);
    }

    #[test]
    fn test_base_luck_full_moon() {
        assert_eq!(base_luck(true, false), 1);
    }

    #[test]
    fn test_base_luck_friday13() {
        assert_eq!(base_luck(false, true), -1);
    }

    #[test]
    fn test_base_luck_both() {
        assert_eq!(base_luck(true, true), 0);
    }

    // --- lamp_burn_stage ---
    #[test]
    fn test_lamp_stages() {
        assert_eq!(lamp_burn_stage(150), BurnStage::Flickering);
        assert_eq!(lamp_burn_stage(100), BurnStage::Flickering);
        assert_eq!(lamp_burn_stage(50), BurnStage::FlickeringConsiderably);
        assert_eq!(lamp_burn_stage(25), BurnStage::AboutToGoOut);
        assert_eq!(lamp_burn_stage(0), BurnStage::BurnedOut);
        assert_eq!(lamp_burn_stage(80), BurnStage::Normal);
    }

    // --- candle_burn_stage ---
    #[test]
    fn test_candle_stages() {
        assert_eq!(candle_burn_stage(75), BurnStage::GettingShort);
        assert_eq!(candle_burn_stage(15), BurnStage::FlameFlickersLow);
        assert_eq!(candle_burn_stage(0), BurnStage::BurnedOut);
        assert_eq!(candle_burn_stage(30), BurnStage::Normal);
    }

    // --- lamp_next_turns ---
    #[test]
    fn test_lamp_turns() {
        assert_eq!(lamp_next_turns(200), 50);
        assert_eq!(lamp_next_turns(120), 20);
        assert_eq!(lamp_next_turns(60), 10);
        assert_eq!(lamp_next_turns(30), 5);
        assert_eq!(lamp_next_turns(10), 10);
    }

    // --- candle_next_turns ---
    #[test]
    fn test_candle_turns() {
        assert_eq!(candle_next_turns(100), 25);
        assert_eq!(candle_next_turns(50), 35);
        assert_eq!(candle_next_turns(10), 10);
    }

    // --- egg_hatch_time ---
    #[test]
    fn test_egg_hatch_range() {
        let mut rng = test_rng();
        let mut hatched = 0;
        for _ in 0..100 {
            match egg_hatch_time(&mut rng) {
                EggHatchResult::HatchAt(t) => {
                    assert!(t >= 151 && t <= 200, "부화 시간: {}", t);
                    hatched += 1;
                }
                EggHatchResult::NoHatch => {}
            }
        }
        // 부화 확률 99.9993%, 100번 중 거의 100번 부화
        assert!(hatched > 95, "부화: {}/100", hatched);
    }

    // --- fig_transform_time ---
    #[test]
    fn test_fig_time() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let t = fig_transform_time(&mut rng);
            assert!(t >= 201 && t <= 9200, "인형 변신: {}", t);
        }
    }

    // --- oil_diluted_turns ---
    #[test]
    fn test_oil_normal() {
        assert_eq!(oil_diluted_turns(100, false), 100);
    }

    #[test]
    fn test_oil_diluted() {
        // (3*100+2)/4 = 75
        assert_eq!(oil_diluted_turns(100, true), 75);
    }

    // --- storm_strike_count ---
    #[test]
    fn test_storm_count() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let c = storm_strike_count(&mut rng);
            assert!(c >= 1 && c <= 7, "번개: {}", c);
        }
    }
}

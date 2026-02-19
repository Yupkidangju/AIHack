// steed_ext.rs — steed.c 핵심 로직 순수 결과 패턴 이식
// [v2.13.0] 신규 생성: 안장 장착 확률/탑승 실패/낙마/질주/기술 연습 등 10개 함수
// 원본: NetHack 3.6.7 src/steed.c (781줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 최대 플레이어 레벨 (MAXULEV)
const MAXULEV: i32 = 30;

// ============================================================
// 열거형
// ============================================================

/// 기승 스킬 레벨
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RidingSkill {
    Restricted,
    Unskilled,
    Basic,
    Skilled,
    Expert,
}

/// 탈것 크기
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gigantic,
}

/// 탈것 종류 심볼 (안장 가능 확인용)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteedSymbol {
    Quadruped,  // S_QUADRUPED
    Unicorn,    // S_UNICORN
    Angel,      // S_ANGEL
    Centaur,    // S_CENTAUR
    Dragon,     // S_DRAGON
    Jabberwock, // S_JABBERWOCK
    Other,
}

/// 낙마 사유
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DismountReason {
    ByChoice,
    Thrown,
    Fell,
    Poly,
    Engulfed,
    Bones,
    Generic,
}

/// 탈것 깨우기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteedWakeResult {
    /// 수면 해제 또는 마비 해제
    Awakened,
    /// 수면 해제만 (마비는 남음, 기간 단축)
    Stirred { remaining_frozen: i32 },
    /// 반응 없음
    NoResponse,
    /// 이미 활동 중
    AlreadyAwake,
}

// ============================================================
// 1. can_saddle_check — 안장 장착 가능 여부
// ============================================================

/// 몬스터에 안장을 얹을 수 있는지 판정
/// 원본: steed.c can_saddle()
pub fn can_saddle_check(
    symbol: SteedSymbol,
    size: MonsterSize,
    is_humanoid: bool,
    is_centaur: bool,
    is_amorphous: bool,
    is_noncorporeal: bool,
    is_whirly: bool,
    is_unsolid: bool,
) -> bool {
    // 유효한 탈것 심볼
    let valid_symbol = matches!(
        symbol,
        SteedSymbol::Quadruped
            | SteedSymbol::Unicorn
            | SteedSymbol::Angel
            | SteedSymbol::Centaur
            | SteedSymbol::Dragon
            | SteedSymbol::Jabberwock
    );

    // 크기 중형 이상
    let size_ok = matches!(
        size,
        MonsterSize::Medium | MonsterSize::Large | MonsterSize::Huge | MonsterSize::Gigantic
    );

    // 인간형이 아니거나 켄타우로스
    let shape_ok = !is_humanoid || is_centaur;

    valid_symbol
        && size_ok
        && shape_ok
        && !is_amorphous
        && !is_noncorporeal
        && !is_whirly
        && !is_unsolid
}

// ============================================================
// 2. saddle_chance_calc — 안장 장착 성공률
// ============================================================

/// 안장 장착 성공률 계산
/// 원본: steed.c use_saddle() L93-126
pub fn saddle_chance_calc(
    dexterity: i32,
    charisma: i32,
    tame_level: i32,
    player_level: i32,
    is_knight: bool,
    riding_skill: RidingSkill,
    is_confused: bool,
    is_fumbling: bool,
    is_glib: bool,
    has_riding_gloves: bool,
    has_riding_boots: bool,
    is_cursed_saddle: bool,
    monster_level: i32,
) -> i32 {
    let mut chance = dexterity + charisma / 2 + 2 * tame_level;

    // 길들여진 정도에 따른 레벨 보너스
    chance += player_level * (if tame_level > 0 { 20 } else { 5 });

    // 야생이면 몬스터 레벨 패널티
    if tame_level == 0 {
        chance -= 10 * monster_level;
    }

    // 기사 보너스
    if is_knight {
        chance += 20;
    }

    // 기승 스킬 보너스
    match riding_skill {
        RidingSkill::Restricted | RidingSkill::Unskilled => chance -= 20,
        RidingSkill::Basic => {}
        RidingSkill::Skilled => chance += 15,
        RidingSkill::Expert => chance += 30,
    }

    // 상태이상 패널티
    if is_confused || is_fumbling || is_glib {
        chance -= 20;
    } else if has_riding_gloves || has_riding_boots {
        // 기승 장비 보너스 (어질거림 없을 때만)
        chance += 10;
    }

    // 저주된 안장
    if is_cursed_saddle {
        chance -= 50;
    }

    chance
}

// ============================================================
// 3. mount_slip_damage — 탑승 실패 시 데미지
// ============================================================

/// 탑승 실패 시 데미지 계산
/// 원본: steed.c mount_steed() L337 — rn1(5, 10)
pub fn mount_slip_damage(rng: &mut NetHackRng) -> i32 {
    rng.rn1(5, 10) // 10~14
}

// ============================================================
// 4. mount_slip_check — 탑승 미끄러짐 판정
// ============================================================

/// 탑승 시 미끄러짐 여부 판정
/// 원본: steed.c mount_steed() L322-324
pub fn mount_slip_check(
    player_level: i32,
    tame_level: i32,
    is_confused: bool,
    is_fumbling: bool,
    is_glib: bool,
    is_wounded_legs: bool,
    saddle_cursed: bool,
    rng: &mut NetHackRng,
) -> bool {
    is_confused
        || is_fumbling
        || is_glib
        || is_wounded_legs
        || saddle_cursed
        || (player_level + tame_level < rng.rnd(MAXULEV / 2 + 5))
}

// ============================================================
// 5. dismount_damage — 낙마 데미지
// ============================================================

/// 낙마 또는 추락 시 데미지 계산
/// 원본: steed.c dismount_steed() L505 — rn1(10, 10)
pub fn dismount_damage(rng: &mut NetHackRng) -> i32 {
    rng.rn1(10, 10) // 10~19
}

/// 낙마 시 다리 부상 기간
/// 원본: steed.c dismount_steed() L506 — rn1(5, 5)
pub fn dismount_leg_damage_duration(rng: &mut NetHackRng) -> i32 {
    rng.rn1(5, 5) // 5~9
}

// ============================================================
// 6. steed_gallop_duration — 킥 후 질주 기간
// ============================================================

/// 탈것 킥 시 질주(gallop) 기간
/// 원본: steed.c kick_steed() L422 — rn1(20, 30)
pub fn steed_gallop_duration(rng: &mut NetHackRng) -> i32 {
    rng.rn1(20, 30) // 30~49
}

// ============================================================
// 7. exercise_steed_check — 기승 기술 연습 판정
// ============================================================

/// 100턴마다 기승 기술 연습 발생 여부
/// 원본: steed.c exercise_steed() L369
pub fn exercise_steed_check(ride_turns: i32) -> bool {
    ride_turns >= 100
}

// ============================================================
// 8. steed_wake_calc — 수면/마비 탈것 깨우기
// ============================================================

/// 수면/마비 탈것을 킥으로 깨우기 시도
/// 원본: steed.c kick_steed() L384-407
pub fn steed_wake_calc(
    is_sleeping: bool,
    can_move: bool,
    frozen: i32,
    rng: &mut NetHackRng,
) -> SteedWakeResult {
    if !is_sleeping && can_move {
        return SteedWakeResult::AlreadyAwake;
    }

    // 50% 확률로 반응
    if (can_move || frozen > 0) && rng.rn2(2) == 0 {
        if can_move && is_sleeping {
            // 수면 해제
            return SteedWakeResult::Awakened;
        } else if frozen > 2 {
            // 마비 기간 2 감소
            return SteedWakeResult::Stirred {
                remaining_frozen: frozen - 2,
            };
        } else {
            // 마비 완전 해제
            return SteedWakeResult::Awakened;
        }
    }

    SteedWakeResult::NoResponse
}

// ============================================================
// 9. steed_tame_decrease — 킥 시 친밀도 감소
// ============================================================

/// 탈것을 차면 친밀도가 1 감소하고, 친밀도 0이면 낙마
/// 원본: steed.c kick_steed() L410-418
/// 반환: (new_tame, should_dismount)
pub fn steed_tame_decrease(tame: i32, player_level: i32, rng: &mut NetHackRng) -> (i32, bool) {
    let new_tame = if tame > 0 { tame - 1 } else { 0 };

    let should_dismount = new_tame == 0 || (player_level + new_tame < rng.rnd(MAXULEV / 2 + 5));

    (new_tame, should_dismount)
}

// ============================================================
// 10. maybewakesteed — 안장/탑승 시 탈것 깨우기
// ============================================================

/// 안장 장착 또는 탑승 시 수면/마비 탈것 깨우기 시도
/// 원본: steed.c maybewakesteed() L692-715
pub fn maybewakesteed(is_sleeping: bool, frozen: i32, rng: &mut NetHackRng) -> (bool, i32) {
    // 수면은 항상 해제
    let new_sleeping = false;

    if frozen == 0 {
        return (new_sleeping, 0);
    }

    // 마비: 기간 절반, 1/(기간절반) 확률로 완전 해제
    let half_frozen = (frozen + 1) / 2;
    if rng.rn2(half_frozen) == 0 {
        // 완전 해제
        (new_sleeping, 0)
    } else {
        // 기간만 절반으로
        (new_sleeping, half_frozen)
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

    // --- can_saddle_check ---
    #[test]
    fn test_saddle_dragon() {
        assert!(can_saddle_check(
            SteedSymbol::Dragon,
            MonsterSize::Huge,
            false,
            false,
            false,
            false,
            false,
            false
        ));
    }

    #[test]
    fn test_saddle_too_small() {
        assert!(!can_saddle_check(
            SteedSymbol::Unicorn,
            MonsterSize::Small,
            false,
            false,
            false,
            false,
            false,
            false
        ));
    }

    #[test]
    fn test_saddle_humanoid_not_centaur() {
        assert!(!can_saddle_check(
            SteedSymbol::Angel,
            MonsterSize::Medium,
            true,
            false,
            false,
            false,
            false,
            false
        ));
    }

    #[test]
    fn test_saddle_centaur() {
        assert!(can_saddle_check(
            SteedSymbol::Centaur,
            MonsterSize::Large,
            true,
            true,
            false,
            false,
            false,
            false
        ));
    }

    #[test]
    fn test_saddle_amorphous() {
        assert!(!can_saddle_check(
            SteedSymbol::Quadruped,
            MonsterSize::Large,
            false,
            false,
            true,
            false,
            false,
            false
        ));
    }

    // --- saddle_chance_calc ---
    #[test]
    fn test_saddle_chance_knight() {
        let chance = saddle_chance_calc(
            18,
            16,
            10,
            15,
            true,
            RidingSkill::Expert,
            false,
            false,
            false,
            false,
            false,
            false,
            5,
        );
        // 18+8+20 + 15*20 + 20 + 30 = 396
        assert!(chance > 300, "기사 전문가 높은 확률: {}", chance);
    }

    #[test]
    fn test_saddle_chance_wild() {
        let chance = saddle_chance_calc(
            10,
            10,
            0,
            5,
            false,
            RidingSkill::Unskilled,
            true,
            false,
            false,
            false,
            false,
            false,
            15,
        );
        // 10+5+0 + 5*5 - 150 - 20 - 20 = -150
        assert!(chance < 0, "야생 고레벨 몬스터 부정 확률: {}", chance);
    }

    // --- mount_slip_damage ---
    #[test]
    fn test_slip_damage_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dmg = mount_slip_damage(&mut rng);
            assert!(dmg >= 10 && dmg < 15, "미끄러짐 데미지: {}", dmg);
        }
    }

    // --- mount_slip_check ---
    #[test]
    fn test_slip_confused() {
        let mut rng = test_rng();
        assert!(mount_slip_check(
            10, 5, true, false, false, false, false, &mut rng
        ));
    }

    // --- dismount_damage ---
    #[test]
    fn test_dismount_damage_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dmg = dismount_damage(&mut rng);
            assert!(dmg >= 10 && dmg < 20, "낙마 데미지: {}", dmg);
        }
    }

    // --- steed_gallop_duration ---
    #[test]
    fn test_gallop_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dur = steed_gallop_duration(&mut rng);
            assert!(dur >= 30 && dur < 50, "질주 기간: {}", dur);
        }
    }

    // --- exercise_steed_check ---
    #[test]
    fn test_exercise() {
        assert!(exercise_steed_check(100));
        assert!(exercise_steed_check(150));
        assert!(!exercise_steed_check(99));
    }

    // --- steed_wake_calc ---
    #[test]
    fn test_wake_already_awake() {
        let mut rng = test_rng();
        assert_eq!(
            steed_wake_calc(false, true, 0, &mut rng),
            SteedWakeResult::AlreadyAwake
        );
    }

    #[test]
    fn test_wake_sleeping() {
        let mut rng = test_rng();
        let mut awakened = false;
        for _ in 0..100 {
            if let SteedWakeResult::Awakened = steed_wake_calc(true, true, 0, &mut rng) {
                awakened = true;
                break;
            }
        }
        assert!(awakened, "수면 해제 발생");
    }

    // --- steed_tame_decrease ---
    #[test]
    fn test_tame_to_zero() {
        let mut rng = test_rng();
        let (new_tame, dismount) = steed_tame_decrease(1, 1, &mut rng);
        assert_eq!(new_tame, 0);
        assert!(dismount, "친밀도 0 → 낙마");
    }

    // --- maybewakesteed ---
    #[test]
    fn test_maybe_wake_no_frozen() {
        let mut rng = test_rng();
        let (sleeping, frozen) = maybewakesteed(true, 0, &mut rng);
        assert!(!sleeping);
        assert_eq!(frozen, 0);
    }

    #[test]
    fn test_maybe_wake_frozen() {
        let mut rng = test_rng();
        let (_, frozen) = maybewakesteed(true, 10, &mut rng);
        // 절반(5) 또는 0
        assert!(frozen == 0 || frozen == 5, "마비 결과: {}", frozen);
    }
}

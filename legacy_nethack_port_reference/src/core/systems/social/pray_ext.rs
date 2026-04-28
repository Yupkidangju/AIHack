// pray_ext.rs — pray.c 핵심 로직 순수 결과 패턴 이식
// [v2.16.0] 신규 생성: 기도 결과/신의 분노/문제 판정/대관식 등 10개 함수
// 원본: NetHack 3.6.7 src/pray.c (2,303줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

const PIOUS: i32 = 20;
const DEVOUT: i32 = 14;
const FERVENT: i32 = 9;
const STRIDENT: i32 = 4;

// ============================================================
// 열거형
// ============================================================

/// 문제 심각도 (양수: 심각, 음수: 경미, 0: 없음)
/// 원본: pray.c in_trouble() L182-269
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trouble {
    // === 심각한 문제 (양수) ===
    Stoned = 14,
    Slimed = 13,
    Strangled = 12,
    Lava = 11,
    Sick = 10,
    Starving = 9,
    Region = 8,
    Hit = 7,
    Lycanthrope = 6,
    Collapsing = 5,
    StuckInWall = 4,
    CursedLevitation = 3,
    UnuseableHands = 2,
    CursedBlindfold = 1,
    // === 없음 ===
    None = 0,
}

/// 경미한 문제 (음수 값으로 분리)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinorTrouble {
    Punished,
    Fumbling,
    CursedItems,
    Saddle,
    Blind,
    Poisoned,
    WoundedLegs,
    Hungry,
    Stunned,
    Confused,
    Hallucination,
}

/// 신의 분노 결과
/// 원본: pray.c angrygods() L669-743
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngryGodResult {
    /// 0-1: 불쾌함 메시지만
    Displeased,
    /// 2-3: 지혜 감소 + 경험치 손실
    WisdomLoss,
    /// 4-5: 검은 빛 + 저주
    DarkCurse,
    /// 6: 공 + 사슬 형벌
    Punish,
    /// 7-8: 미니언 소환 + 사형 선고
    SummonMinion,
    /// 9+: 번개 + 분해 광선
    DivineWrath,
}

/// 신의 보상 레벨
/// 원본: pray.c pleased() L923-1008
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PleaseAction {
    /// 아무것도 안 함
    Nothing,
    /// 최악의 문제 1개 해결
    FixWorst,
    /// 모든 심각한 문제 해결
    FixAllMajor,
    /// 경미한 문제도 해결
    FixMinorToo,
    /// 모든 문제 해결
    FixAll,
    /// 특별 보상 (축복, 강화, 경험치 등)
    PatOnHead,
}

/// 특별 보상 내용
/// 원본: pray.c pleased() L1015-1145
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialBoon {
    /// 아무것도 안 함
    Nothing,
    /// 무기 수리/축복/강화
    BlessWeapon,
    /// 골든 글로우 (HP/레벨 보너스)
    GoldenGlow,
    /// 성채 힌트
    CastleHint,
    /// 아이템 식별
    Enlightenment,
}

/// HP 위험 판정을 위한 제수 수준
/// 원본: pray.c critically_low_hp() L100-141
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpDivisorRank {
    Rank0to1, // 제수 5
    Rank2to3, // 제수 6
    Rank4to5, // 제수 7
    Rank6to7, // 제수 8
    Rank8Up,  // 제수 9
}

// ============================================================
// 1. critically_low_hp — HP 위험 판정
// ============================================================

/// 현재 HP가 위험 수준인지 판정
/// 원본: pray.c critically_low_hp() L100-141
pub fn critically_low_hp(
    current_hp: i32,
    max_hp: i32,
    player_level: i32,
    only_if_injured: bool,
) -> bool {
    if only_if_injured && current_hp >= max_hp {
        return false;
    }

    // 레벨 기반 maxhp 상한
    let hp_limit = 15 * player_level;
    let effective_max = max_hp.min(hp_limit);

    // 레벨에 따른 제수 결정
    let rank = xlev_to_rank(player_level);
    let divisor = match rank {
        0 | 1 => 5,
        2 | 3 => 6,
        4 | 5 => 7,
        6 | 7 => 8,
        _ => 9,
    };

    current_hp <= 5 || current_hp * divisor <= effective_max
}

/// 레벨을 랭크로 변환 (1~30 → 0~8)
/// 원본: extern xlev_to_rank()
fn xlev_to_rank(level: i32) -> i32 {
    if level <= 2 {
        0
    } else if level <= 5 {
        1
    } else if level <= 9 {
        2
    } else if level <= 13 {
        3
    } else if level <= 17 {
        4
    } else if level <= 21 {
        5
    } else if level <= 25 {
        6
    } else if level <= 29 {
        7
    } else {
        8
    }
}

// ============================================================
// 2. angrygods_calc — 신의 분노 결과 결정
// ============================================================

/// 신의 분노 시 효과 결정
/// 원본: pray.c angrygods() L669-743
pub fn angrygods_calc(
    same_alignment: bool,
    alignment_record: i32,
    anger: i32,
    luck: i32,
    is_already_punished: bool,
    rng: &mut NetHackRng,
) -> AngryGodResult {
    // 최대 분노 계산
    let maxanger = if !same_alignment {
        let v = alignment_record / 2 + if luck > 0 { -luck / 3 } else { -luck };
        v.max(1).min(15)
    } else {
        let v = 3 * anger
            + if luck > 0 || alignment_record >= STRIDENT {
                -luck / 3
            } else {
                -luck
            };
        v.max(1).min(15)
    };

    match rng.rn2(maxanger) {
        0 | 1 => AngryGodResult::Displeased,
        2 | 3 => AngryGodResult::WisdomLoss,
        6 => {
            if !is_already_punished {
                AngryGodResult::Punish
            } else {
                AngryGodResult::DarkCurse
            }
        }
        4 | 5 => AngryGodResult::DarkCurse,
        7 | 8 => AngryGodResult::SummonMinion,
        _ => AngryGodResult::DivineWrath,
    }
}

// ============================================================
// 3. prayer_action_calc — 기도 보상 수준 결정
// ============================================================

/// 기도 시 신의 보상 수준 결정
/// 원본: pray.c pleased() L958-1008
pub fn prayer_action_calc(
    has_trouble: bool,
    alignment_record: i32,
    initial_trouble: i32,
    luck: i32,
    on_altar: bool,
    on_shrine: bool,
    rng: &mut NetHackRng,
) -> PleaseAction {
    // 문제 없고 경건 → 특별 보상
    if !has_trouble && alignment_record >= DEVOUT {
        if initial_trouble == 0 {
            return PleaseAction::PatOnHead;
        }
        return PleaseAction::Nothing;
    }

    let prayer_luck = luck.max(-1);
    let shrine_bonus = if on_shrine { 1 } else { 0 };
    let altar_bonus = if on_altar { 3 + shrine_bonus } else { 2 };
    let mut action = rng.rn1(prayer_luck + altar_bonus, 1);

    if !on_altar {
        action = action.min(3);
    }
    if alignment_record < STRIDENT {
        action = if alignment_record > 0 || rng.rnl(2, luck) == 0 {
            1
        } else {
            0
        };
    }

    match action.min(5) {
        5 => PleaseAction::PatOnHead,
        4 => PleaseAction::FixAll,
        3 => PleaseAction::FixMinorToo,
        2 => PleaseAction::FixAllMajor,
        1 => PleaseAction::FixWorst,
        _ => PleaseAction::Nothing,
    }
}

// ============================================================
// 4. pat_on_head_boon — 특별 보상 결정
// ============================================================

/// 기도 특별 보상 내용 결정
/// 원본: pray.c pleased() L1015-1145
pub fn pat_on_head_boon(
    luck: i32,
    has_weapon: bool,
    has_heard_tune: i32,
    opened_drawbridge: bool,
    entered_gehennom: bool,
    rng: &mut NetHackRng,
) -> SpecialBoon {
    let roll = rng.rn2(((luck + 6) >> 1).max(1));
    match roll {
        0 => SpecialBoon::Nothing,
        1 => {
            if has_weapon {
                SpecialBoon::BlessWeapon
            } else {
                SpecialBoon::Nothing
            }
        }
        3 => {
            if !opened_drawbridge && !entered_gehennom && has_heard_tune < 2 {
                SpecialBoon::CastleHint
            } else {
                SpecialBoon::GoldenGlow
            }
        }
        2 => SpecialBoon::GoldenGlow,
        _ => SpecialBoon::Enlightenment,
    }
}

// ============================================================
// 5. prayer_timeout — 기도 쿨다운 계산
// ============================================================

/// 기도 쿨다운 턴 계산
/// 원본: pray.c angrygods() L741 — rnz(300)
pub fn prayer_timeout(player_level: i32, rng: &mut NetHackRng) -> i32 {
    rng.rnz(300, player_level)
}

// ============================================================
// 6. fix_hit_hp — HP 문제 해결 시 회복량
// ============================================================

/// HP 위험 해소 시 최대HP 증가량
/// 원본: pray.c fix_worst_trouble() L387-397
pub fn fix_hit_hp_boost(current_max: i32, player_level: i32, rng: &mut NetHackRng) -> i32 {
    let target = player_level * 5 + 11;
    if current_max < target {
        rng.rnd(5)
    } else {
        0
    }
}

// ============================================================
// 7. collapsing_strength — 무너짐 문제 심각도
// ============================================================

/// 힘 손실 정도 (AMAX-ABASE > 3이면 무너짐)
pub fn collapsing_severity(str_max: i32, str_base: i32) -> bool {
    str_max - str_base > 3
}

/// 무너짐 메시지 강도
pub fn collapsing_message_level(str_max: i32, str_base: i32) -> bool {
    str_max - str_base > 6
}

// ============================================================
// 8. wall_phasing_duration — 벽 통과 부여 턴
// ============================================================

/// 벽에 갇힌 문제 해결 시 부여되는 벽 통과 지속 턴
/// 원본: pray.c fix_worst_trouble() L429 — d(4,4)+4
pub fn wall_phasing_duration(rng: &mut NetHackRng) -> i32 {
    rng.d(4, 4) + 4 // 8~20
}

// ============================================================
// 9. golden_glow_hp — 골든 글로우 HP 보너스
// ============================================================

/// 골든 글로우 보상 시 HP 증가량
/// 원본: pray.c pleased() L1098-1100
pub fn golden_glow_hp_bonus() -> i32 {
    5
}

// ============================================================
// 10. satisfied_message — 만족도 메시지 수준
// ============================================================

/// 기도 시 신의 만족도 메시지 수준 결정
/// 원본: pray.c pleased() L930-935
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatisfactionLevel {
    WellPleased,
    Pleased,
    Satisfied,
}

pub fn satisfaction_level(alignment_record: i32) -> SatisfactionLevel {
    if alignment_record >= DEVOUT {
        SatisfactionLevel::WellPleased
    } else if alignment_record >= STRIDENT {
        SatisfactionLevel::Pleased
    } else {
        SatisfactionLevel::Satisfied
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

    // --- critically_low_hp ---
    #[test]
    fn test_hp_critical_low() {
        assert!(critically_low_hp(3, 50, 10, false));
        assert!(critically_low_hp(5, 50, 10, false));
    }

    #[test]
    fn test_hp_not_critical() {
        assert!(!critically_low_hp(30, 50, 10, false));
    }

    #[test]
    fn test_hp_not_injured() {
        assert!(!critically_low_hp(50, 50, 10, true));
    }

    #[test]
    fn test_hp_divisor_high_level() {
        // 레벨 30: 제수 9, maxhp=100: 100/9≈11
        assert!(critically_low_hp(10, 100, 30, false)); // 10*9=90 < 100
        assert!(!critically_low_hp(15, 100, 30, false)); // 15*9=135 > 100
    }

    // --- angrygods_calc ---
    #[test]
    fn test_angry_displeased() {
        let mut rng = test_rng();
        // maxanger=1이면 rn2(1)=0 → Displeased
        let result = angrygods_calc(true, 0, 0, 0, false, &mut rng);
        assert_eq!(result, AngryGodResult::Displeased);
    }

    #[test]
    fn test_angry_varied() {
        let mut rng = test_rng();
        let mut types = std::collections::HashSet::new();
        for _ in 0..500 {
            let result = angrygods_calc(true, 5, 5, 0, false, &mut rng);
            types.insert(format!("{:?}", result));
        }
        assert!(types.len() >= 3, "분노 결과 다양성: {}", types.len());
    }

    // --- prayer_action_calc ---
    #[test]
    fn test_prayer_devout_no_trouble() {
        let mut rng = test_rng();
        let result = prayer_action_calc(false, DEVOUT, 0, 3, true, true, &mut rng);
        assert_eq!(result, PleaseAction::PatOnHead);
    }

    #[test]
    fn test_prayer_low_align() {
        let mut rng = test_rng();
        // 낮은 정렬이면 FixWorst 또는 Nothing
        let result = prayer_action_calc(true, 1, 5, 0, false, false, &mut rng);
        assert!(
            result == PleaseAction::FixWorst || result == PleaseAction::Nothing,
            "낮은 정렬: {:?}",
            result
        );
    }

    // --- pat_on_head_boon ---
    #[test]
    fn test_boon_low_luck() {
        let mut rng = test_rng();
        // luck=0 → rn2(3) = 0,1,2
        let result = pat_on_head_boon(0, true, 0, false, false, &mut rng);
        assert!(
            result != SpecialBoon::Enlightenment, // roll < 3 유지
            "보상: {:?}",
            result
        );
    }

    // --- prayer_timeout ---
    #[test]
    fn test_timeout_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let t = prayer_timeout(10, &mut rng);
            assert!(t > 0, "쿨다운: {}", t);
        }
    }

    // --- fix_hit_hp_boost ---
    #[test]
    fn test_hp_boost_needed() {
        let mut rng = test_rng();
        let boost = fix_hit_hp_boost(20, 10, &mut rng); // 20 < 10*5+11=61
        assert!(boost >= 1 && boost <= 5, "HP 증가: {}", boost);
    }

    #[test]
    fn test_hp_boost_not_needed() {
        let mut rng = test_rng();
        let boost = fix_hit_hp_boost(100, 10, &mut rng); // 100 >= 61
        assert_eq!(boost, 0);
    }

    // --- collapsing ---
    #[test]
    fn test_collapsing() {
        assert!(collapsing_severity(18, 10));
        assert!(!collapsing_severity(18, 16));
    }

    // --- wall_phasing ---
    #[test]
    fn test_wall_phasing() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dur = wall_phasing_duration(&mut rng);
            assert!(dur >= 8 && dur <= 20, "벽 통과: {}", dur);
        }
    }

    // --- satisfaction ---
    #[test]
    fn test_satisfaction() {
        assert_eq!(satisfaction_level(20), SatisfactionLevel::WellPleased);
        assert_eq!(satisfaction_level(10), SatisfactionLevel::Pleased);
        assert_eq!(satisfaction_level(2), SatisfactionLevel::Satisfied);
    }
}

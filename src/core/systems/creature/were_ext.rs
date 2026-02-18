// ============================================================================
// AIHack - were_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] were.c 핵심 함수 완전 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 were.c (222줄)
//
// 이식 대상:
//   counter_were(), were_beastie(), were_change_chance(),
//   were_summon_type(), were_heal_amount()
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// 수인 유형 정의
// [v2.10.1] were.c 전체 PM 매핑 이식
// =============================================================================

/// 수인(Lycanthrope) 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WereType {
    Werewolf,
    HumanWerewolf,
    Werejackal,
    HumanWerejackal,
    Wererat,
    HumanWererat,
}

impl WereType {
    /// 인간 형태인지 확인
    pub fn is_human_form(&self) -> bool {
        matches!(
            self,
            WereType::HumanWerewolf | WereType::HumanWerejackal | WereType::HumanWererat
        )
    }

    /// 동물 형태인지 확인
    pub fn is_beast_form(&self) -> bool {
        !self.is_human_form()
    }
}

// =============================================================================
// counter_were — 수인 대항 형태
// [v2.10.1] were.c:45-65 이식
// =============================================================================

/// 수인의 반대 형태 반환 (원본 counter_were)
/// 인간 ↔ 동물 형태 전환 매핑
pub fn counter_were(were_type: WereType) -> Option<WereType> {
    match were_type {
        WereType::Werewolf => Some(WereType::HumanWerewolf),
        WereType::HumanWerewolf => Some(WereType::Werewolf),
        WereType::Werejackal => Some(WereType::HumanWerejackal),
        WereType::HumanWerejackal => Some(WereType::Werejackal),
        WereType::Wererat => Some(WereType::HumanWererat),
        WereType::HumanWererat => Some(WereType::Wererat),
    }
}

// =============================================================================
// were_beastie — 유사 몬스터의 수인 유형 판정
// [v2.10.1] were.c:67-92 이식
// =============================================================================

/// 몬스터 이름 기반 수인 유형 판정 (원본 were_beastie)
/// 쥐/자칼/늑대 계열 → 해당 수인 유형
pub fn were_beastie(monster_name: &str) -> Option<WereType> {
    let lower = monster_name.to_lowercase();

    // 쥐 계열 → 웨어래트
    if lower.contains("wererat")
        || lower.contains("sewer rat")
        || lower.contains("giant rat")
        || lower.contains("rabid rat")
    {
        return Some(WereType::Wererat);
    }
    // 자칼 계열 → 웨어자칼
    if lower.contains("werejackal")
        || lower.contains("jackal")
        || lower.contains("fox")
        || lower.contains("coyote")
    {
        return Some(WereType::Werejackal);
    }
    // 늑대 계열 → 웨어울프
    if lower.contains("werewolf")
        || lower.contains("wolf")
        || lower.contains("warg")
        || lower.contains("winter wolf")
    {
        return Some(WereType::Werewolf);
    }
    None
}

// =============================================================================
// were_change_chance — 수인 변신 확률
// [v2.10.1] were.c:8-43 핵심 확률 이식
// =============================================================================

/// 달의 위상
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonPhase {
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    FullMoon,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

/// 수인 변신 확률 (원본 were_change:12-40)
/// 인간 형태에서 동물로 변신할 확률 분모 반환
/// 밤: 보름달 1/3, 일반 1/30
/// 낮: 보름달 1/10, 일반 1/50
/// 동물에서 인간으로: 1/30 (고정)
pub fn were_change_chance(
    is_human_form: bool,
    is_night: bool,
    moon_phase: MoonPhase,
    has_protection: bool,
) -> i32 {
    if has_protection && is_human_form {
        // 형태변환 보호 → 변신 안 함 (분모 0 = 불가)
        return 0;
    }
    if !is_human_form {
        // 동물 → 인간: 1/30 또는 보호 시 즉시
        if has_protection {
            return 1;
        } // 즉시 변신
        return 30;
    }
    // 인간 → 동물
    if is_night {
        if moon_phase == MoonPhase::FullMoon {
            3
        } else {
            30
        }
    } else {
        if moon_phase == MoonPhase::FullMoon {
            10
        } else {
            50
        }
    }
}

/// 실제 변신 여부 판정 (확률 적용)
pub fn should_were_change(
    is_human_form: bool,
    is_night: bool,
    moon_phase: MoonPhase,
    has_protection: bool,
    rng: &mut NetHackRng,
) -> bool {
    let chance = were_change_chance(is_human_form, is_night, moon_phase, has_protection);
    if chance <= 0 {
        return false;
    }
    if chance == 1 {
        return true;
    }
    rng.rn2(chance) == 0
}

// =============================================================================
// were_summon_type — 수인 소환 몬스터 유형
// [v2.10.1] were.c:124-173 이식
// =============================================================================

/// 수인 소환 몬스터 종류와 이름
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WereSummonResult {
    pub monster_name: String,
    pub group_name: String,
}

/// 수인 유형별 소환 몬스터 결정 (원본 were_summon:139-162)
pub fn were_summon_type(were_type: WereType, rng: &mut NetHackRng) -> WereSummonResult {
    match were_type {
        WereType::Wererat | WereType::HumanWererat => {
            // 원본: rn2(3) ? sewer_rat : rn2(3) ? giant_rat : rabid_rat
            let r = rng.rn2(9);
            let name = if r < 6 {
                "sewer rat"
            } else if r < 8 {
                "giant rat"
            } else {
                "rabid rat"
            };
            WereSummonResult {
                monster_name: name.to_string(),
                group_name: "rat".to_string(),
            }
        }
        WereType::Werejackal | WereType::HumanWerejackal => {
            // 원본: rn2(7) ? jackal : rn2(3) ? coyote : fox
            let r = rng.rn2(21);
            let name = if r < 15 {
                "jackal"
            } else if r < 19 {
                "coyote"
            } else {
                "fox"
            };
            WereSummonResult {
                monster_name: name.to_string(),
                group_name: "jackal".to_string(),
            }
        }
        WereType::Werewolf | WereType::HumanWerewolf => {
            // 원본: rn2(5) ? wolf : rn2(2) ? warg : winter_wolf
            let r = rng.rn2(10);
            let name = if r < 6 {
                "wolf"
            } else if r < 8 {
                "warg"
            } else {
                "winter wolf"
            };
            WereSummonResult {
                monster_name: name.to_string(),
                group_name: "wolf".to_string(),
            }
        }
    }
}

/// 소환 횟수 결정 (원본: rnd(5))
pub fn were_summon_count(rng: &mut NetHackRng) -> i32 {
    rng.rnd(5)
}

// =============================================================================
// were_heal — 변신 시 치유량
// [v2.10.1] were.c:117-118 이식
// =============================================================================

/// 수인 변신 시 회복량 (원본: (mhpmax - mhp) / 4)
pub fn were_heal_amount(current_hp: i32, max_hp: i32) -> i32 {
    (max_hp - current_hp) / 4
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_were() {
        assert_eq!(
            counter_were(WereType::Werewolf),
            Some(WereType::HumanWerewolf)
        );
        assert_eq!(
            counter_were(WereType::HumanWerewolf),
            Some(WereType::Werewolf)
        );
        assert_eq!(
            counter_were(WereType::Werejackal),
            Some(WereType::HumanWerejackal)
        );
        assert_eq!(
            counter_were(WereType::HumanWerejackal),
            Some(WereType::Werejackal)
        );
        assert_eq!(
            counter_were(WereType::Wererat),
            Some(WereType::HumanWererat)
        );
        assert_eq!(
            counter_were(WereType::HumanWererat),
            Some(WereType::Wererat)
        );
    }

    #[test]
    fn test_were_beastie() {
        assert_eq!(were_beastie("sewer rat"), Some(WereType::Wererat));
        assert_eq!(were_beastie("giant rat"), Some(WereType::Wererat));
        assert_eq!(were_beastie("jackal"), Some(WereType::Werejackal));
        assert_eq!(were_beastie("coyote"), Some(WereType::Werejackal));
        assert_eq!(were_beastie("wolf"), Some(WereType::Werewolf));
        assert_eq!(were_beastie("warg"), Some(WereType::Werewolf));
        assert_eq!(were_beastie("human"), None);
    }

    #[test]
    fn test_were_change_chance() {
        // 인간 + 밤 + 보름달 → 1/3
        assert_eq!(
            were_change_chance(true, true, MoonPhase::FullMoon, false),
            3
        );
        // 인간 + 밤 + 일반 → 1/30
        assert_eq!(
            were_change_chance(true, true, MoonPhase::NewMoon, false),
            30
        );
        // 인간 + 낮 + 보름달 → 1/10
        assert_eq!(
            were_change_chance(true, false, MoonPhase::FullMoon, false),
            10
        );
        // 동물 → 인간: 1/30
        assert_eq!(
            were_change_chance(false, true, MoonPhase::FullMoon, false),
            30
        );
        // 보호 + 인간 → 불가
        assert_eq!(were_change_chance(true, true, MoonPhase::FullMoon, true), 0);
        // 보호 + 동물 → 즉시
        assert_eq!(were_change_chance(false, true, MoonPhase::NewMoon, true), 1);
    }

    #[test]
    fn test_should_were_change_full_moon() {
        let mut changed = 0;
        for seed in 0..300u64 {
            let mut rng = NetHackRng::new(seed);
            if should_were_change(true, true, MoonPhase::FullMoon, false, &mut rng) {
                changed += 1;
            }
        }
        // 1/3 확률 → 약 100 예상
        assert!(changed > 50 && changed < 200, "changed={}", changed);
    }

    #[test]
    fn test_were_summon_type() {
        let mut rng = NetHackRng::new(42);
        let result = were_summon_type(WereType::Wererat, &mut rng);
        assert_eq!(result.group_name, "rat");
        assert!(["sewer rat", "giant rat", "rabid rat"].contains(&result.monster_name.as_str()));
    }

    #[test]
    fn test_were_summon_distribution() {
        let mut rats = 0;
        let mut giants = 0;
        let mut rabids = 0;
        for seed in 0..900u64 {
            let mut rng = NetHackRng::new(seed);
            let r = were_summon_type(WereType::Wererat, &mut rng);
            match r.monster_name.as_str() {
                "sewer rat" => rats += 1,
                "giant rat" => giants += 1,
                "rabid rat" => rabids += 1,
                _ => panic!("예상치 못한 몬스터: {}", r.monster_name),
            }
        }
        // sewer rat 약 66%, giant rat 약 22%, rabid rat 약 11%
        assert!(
            rats > giants && giants > rabids,
            "rats={} giants={} rabids={}",
            rats,
            giants,
            rabids
        );
    }

    #[test]
    fn test_were_heal_amount() {
        assert_eq!(were_heal_amount(50, 100), 12);
        assert_eq!(were_heal_amount(100, 100), 0);
        assert_eq!(were_heal_amount(10, 100), 22);
    }

    #[test]
    fn test_were_summon_count() {
        for seed in 0..50u64 {
            let mut rng = NetHackRng::new(seed);
            let count = were_summon_count(&mut rng);
            assert!(count >= 1 && count <= 5);
        }
    }

    #[test]
    fn test_is_human_form() {
        assert!(WereType::HumanWerewolf.is_human_form());
        assert!(!WereType::Werewolf.is_human_form());
    }
}

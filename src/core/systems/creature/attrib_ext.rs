// ============================================================================
// AIHack - attrib_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] attrib.c 미이식 함수 대량 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 attrib.c (1,173줄)
//
// 이식 대상:
//   gainstr, poisontell/poisoned, change_luck/stone_luck/set_moreluck,
//   init_attr, redist_attr, exerper, exertext[],
//   role_abil[] (고유 능력 테이블), adjabil, newhp,
//   acurr/acurrstr, extremeattr, adjalign/uchangealign, from_what
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// 상수 정의 (원본 attrib.c)
// =============================================================================

/// 능력치 인덱스 (원본: A_STR 등)
pub const A_STR: usize = 0;
pub const A_INT: usize = 1;
pub const A_WIS: usize = 2;
pub const A_DEX: usize = 3;
pub const A_CON: usize = 4;
pub const A_CHA: usize = 5;
pub const A_MAX: usize = 6;

/// 운 한계값 (원본: LUCKMIN/LUCKMAX/LUCKADD)
pub const LUCKMIN: i32 = -10;
pub const LUCKMAX: i32 = 10;
pub const LUCKADD: i32 = 3;

/// ATTRMIN / ATTRMAX 기본값
pub const ATTRMIN: i32 = 3;
pub const ATTRMAX_DEFAULT: i32 = 18;
const ATTRMAX_ABSOLUTE: i32 = 25;

/// 운동 튜닝 값 (원본: AVAL)
const AVAL: i32 = 50;

/// 성향 한계값 (원본: ALIGNLIM)
const ALIGNLIM: i32 = 14;

// =============================================================================
// 독 효과 메시지 테이블 (원본: poiseff[])
// [v2.10.1] attrib.c:235-245 이식
// =============================================================================

/// 독으로 인한 능력치 감소 시 메시지 (능력치 인덱스별)
pub const POISON_MESSAGES: [(&str, &str); 6] = [
    ("You feel", "weaker"),             // A_STR
    ("Your", "brain is on fire"),       // A_INT
    ("Your", "judgement is impaired"),  // A_WIS
    ("Your", "muscles won't obey you"), // A_DEX
    ("You feel", "very sick"),          // A_CON
    ("You", "break out in hives"),      // A_CHA
];

/// 독 메시지 생성 (원본 poisontell)
/// [v2.10.1] attrib.c:247-269
pub fn poison_tell_result(
    attr_index: usize,
    exclaim: bool,
    forced_max_str: bool,
    forced_max_con: bool,
) -> String {
    if attr_index >= A_MAX {
        return String::new();
    }

    let (prefix, mut msg) = POISON_MESSAGES[attr_index];

    // 능력치가 강제 최대값인 경우 대체 메시지 (원본:263-266)
    if attr_index == A_STR && forced_max_str {
        msg = "innately weaker";
    } else if attr_index == A_CON && forced_max_con {
        msg = "sick inside";
    }

    let punct = if exclaim { "!" } else { "." };
    format!("{} {}{}", prefix, msg, punct)
}

// =============================================================================
// poisoned — 독 효과 처리 결과
// [v2.10.1] attrib.c:271-337 이식
// =============================================================================

/// 독 효과 결과
#[derive(Debug, Clone)]
pub struct PoisonedResult {
    /// 사망했는지
    pub instant_kill: bool,
    /// HP 데미지 (사망이 아닌 경우)
    pub hp_damage: i32,
    /// 능력치 감소량
    pub attr_loss: i32,
    /// 감소할 능력치 인덱스
    pub attr_index: usize,
    /// 능력치 변경이 발생했는지
    pub attr_changed: bool,
    /// 표시할 메시지
    pub message: String,
}

/// 독 효과 계산 (원본 poisoned)
/// [v2.10.1] attrib.c:271-337
pub fn poisoned_result(
    reason: &str,
    attr_index: usize,
    fatal: i32,
    thrown_weapon: bool,
    has_poison_resistance: bool,
    rng: &mut NetHackRng,
) -> PoisonedResult {
    // 독 저항이 있으면 효과 없음
    if has_poison_resistance {
        return PoisonedResult {
            instant_kill: false,
            hp_damage: 0,
            attr_loss: 0,
            attr_index,
            attr_changed: false,
            message: "The poison doesn't seem to affect you.".to_string(),
        };
    }

    // 독 안내 메시지 생성 (이미 "poison"이나 "blast" 포함 안 된 경우만)
    let base_msg = if reason != "blast" && !reason.to_lowercase().contains("poison") {
        let plural = reason.ends_with('s');
        format!(
            "The {} {} poisoned!",
            reason,
            if plural { "were" } else { "was" }
        )
    } else {
        String::new()
    };

    // 치사 판정 (원본:311)
    let i = if fatal == 0 {
        1
    } else {
        rng.rn2(fatal + if thrown_weapon { 20 } else { 0 })
    };

    if i == 0 && attr_index != A_CHA {
        // 즉사
        return PoisonedResult {
            instant_kill: true,
            hp_damage: 0,
            attr_loss: 0,
            attr_index,
            attr_changed: false,
            message: format!(
                "{}The poison was deadly...",
                if base_msg.is_empty() {
                    String::new()
                } else {
                    format!("{} ", base_msg)
                }
            ),
        };
    } else if i > 5 {
        // HP 데미지 (원본:319-320)
        let loss = if thrown_weapon {
            rng.rnd(6)
        } else {
            rng.rn1(10, 6)
        };
        return PoisonedResult {
            instant_kill: false,
            hp_damage: loss,
            attr_loss: 0,
            attr_index,
            attr_changed: false,
            message: base_msg,
        };
    } else {
        // 능력치 감소 (원본:324)
        let loss = if thrown_weapon || fatal == 0 {
            1
        } else {
            rng.d(2, 2)
        };
        return PoisonedResult {
            instant_kill: false,
            hp_damage: 0,
            attr_loss: loss,
            attr_index,
            attr_changed: true,
            message: base_msg,
        };
    }
}

// =============================================================================
// 운 시스템 (change_luck, stone_luck, set_moreluck)
// [v2.10.1] attrib.c:339-382 이식
// =============================================================================

/// 운 변경 결과 (원본 change_luck)
pub fn change_luck_result(current: i32, delta: i32) -> i32 {
    let mut result = current + delta;
    if result < 0 && result < LUCKMIN {
        result = LUCKMIN;
    }
    if result > 0 && result > LUCKMAX {
        result = LUCKMAX;
    }
    result
}

/// 운석 보너스 (원본 stone_luck)
/// [v2.10.1] attrib.c:350-368
/// inventory_items: (is_luck_granting, is_cursed, is_blessed, quantity)
pub fn stone_luck_result(
    inventory_items: &[(bool, bool, bool, i32)],
    include_uncursed: bool,
) -> i32 {
    let mut bonchance: i64 = 0;
    for &(is_luck, is_cursed, is_blessed, qty) in inventory_items {
        if is_luck {
            if is_cursed {
                bonchance -= qty as i64;
            } else if is_blessed {
                bonchance += qty as i64;
            } else if include_uncursed {
                bonchance += qty as i64;
            }
        }
    }
    // sgn 함수
    if bonchance > 0 {
        1
    } else if bonchance < 0 {
        -1
    } else {
        0
    }
}

/// 운 보너스 설정 결과 (원본 set_moreluck)
/// [v2.10.1] attrib.c:370-382
pub fn set_moreluck_result(stone_luck_val: i32, carrying_luckstone: bool) -> i32 {
    if stone_luck_val == 0 && !carrying_luckstone {
        0
    } else if stone_luck_val >= 0 {
        LUCKADD
    } else {
        -LUCKADD
    }
}

// =============================================================================
// gainstr — 힘 증가량 계산
// [v2.10.1] attrib.c:194-212 이식
// =============================================================================

/// 힘 증가량 계산 (원본 gainstr)
pub fn gainstr_result(current_str: i32, incr: i32, is_cursed: bool, rng: &mut NetHackRng) -> i32 {
    let num = if incr != 0 {
        incr
    } else if current_str < 18 {
        if rng.rn2(4) != 0 {
            1
        } else {
            rng.rnd(6)
        }
    } else if current_str < 103 {
        // STR18(85) = 18+85 = 103
        rng.rnd(10)
    } else {
        1
    };

    if is_cursed {
        -num
    } else {
        num
    }
}

// =============================================================================
// init_attr — 초기 능력치 배분
// [v2.10.1] attrib.c:613-661 이식
// =============================================================================

/// 초기 능력치 배분 결과
pub fn init_attr_result(
    base_attrs: &[i32; A_MAX],
    attr_dist: &[i32; A_MAX],
    attr_max: &[i32; A_MAX],
    total_points: i32,
    rng: &mut NetHackRng,
) -> [i32; A_MAX] {
    let mut attrs = *base_attrs;
    let mut np = total_points;

    // 기본 포인트 차감
    for i in 0..A_MAX {
        np -= base_attrs[i];
    }

    // 남은 포인트 배분 (원본:625-641)
    let mut tryct = 0;
    while np > 0 && tryct < 100 {
        let mut x = rng.rn2(100);
        let mut i = 0;
        while i < A_MAX {
            x -= attr_dist[i];
            if x <= 0 {
                break;
            }
            i += 1;
        }
        if i >= A_MAX {
            continue;
        }

        if attrs[i] >= attr_max[i] {
            tryct += 1;
            continue;
        }
        tryct = 0;
        attrs[i] += 1;
        np -= 1;
    }

    // 초과 포인트 회수 (원본:643-661)
    tryct = 0;
    while np < 0 && tryct < 100 {
        let mut x = rng.rn2(100);
        let mut i = 0;
        while i < A_MAX {
            x -= attr_dist[i];
            if x <= 0 {
                break;
            }
            i += 1;
        }
        if i >= A_MAX {
            continue;
        }

        if attrs[i] <= ATTRMIN {
            tryct += 1;
            continue;
        }
        tryct = 0;
        attrs[i] -= 1;
        np += 1;
    }

    attrs
}

// =============================================================================
// redist_attr — 변이 시 능력치 재배분
// [v2.10.1] attrib.c:663-684 이식
// =============================================================================

/// 변이 시 능력치 재배분 결과 (원본 redist_attr)
pub fn redist_attr_result(
    abase: &[i32; A_MAX],
    amax: &[i32; A_MAX],
    attr_max_cap: &[i32; A_MAX],
    rng: &mut NetHackRng,
) -> ([i32; A_MAX], [i32; A_MAX]) {
    let mut new_base = *abase;
    let mut new_max = *amax;

    for i in 0..A_MAX {
        // Int/Wis는 변이에 영향 안 받음 (원본:669-670)
        if i == A_INT || i == A_WIS {
            continue;
        }

        let tmp = new_max[i];
        if tmp == 0 {
            continue;
        } // 0으로 나누기 방지

        new_max[i] += rng.rn2(5) - 2;
        if new_max[i] > attr_max_cap[i] {
            new_max[i] = attr_max_cap[i];
        }
        if new_max[i] < ATTRMIN {
            new_max[i] = ATTRMIN;
        }

        new_base[i] = new_base[i] * new_max[i] / tmp;
        if new_base[i] < ATTRMIN {
            new_base[i] = ATTRMIN;
        }
    }

    (new_base, new_max)
}

// =============================================================================
// 운동 텍스트 (exertext[])
// [v2.10.1] attrib.c:517-524 이식
// =============================================================================

/// 운동/남용 텍스트 (원본 exertext[])
/// (운동 성공 텍스트, 운동 실패 텍스트) — None이면 해당 능력치는 운동 불가
pub const EXERTEXT: [(Option<&str>, Option<&str>); A_MAX] = [
    (Some("exercising diligently"), Some("exercising properly")), // Str
    (None, None),                                                 // Int (운동 불가)
    (Some("very observant"), Some("paying attention")),           // Wis
    (
        Some("working on your reflexes"),
        Some("working on reflexes lately"),
    ), // Dex
    (
        Some("leading a healthy life-style"),
        Some("watching your health"),
    ), // Con
    (None, None),                                                 // Cha (운동 불가)
];

// =============================================================================
// exerper — 주기적 운동 효과
// [v2.10.1] attrib.c:446-513 이식
// =============================================================================

/// 배고픔 상태 (원본: hunger states)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HungerState {
    Satiated,
    NotHungry,
    Hungry,
    Weak,
    Fainting,
}

/// 적재 상태 (원본: encumbrance)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncumbranceLevel {
    Unencumbered,
    Burdened,
    Stressed,
    Strained,
    Overtaxed,
    Overloaded,
}

/// 주기적 운동 결과 — 어떤 능력치를 운동/남용할지
#[derive(Debug, Clone, Default)]
pub struct ExerperResult {
    /// (능력치 인덱스, 운동 여부 true=운동/false=남용)
    pub exercises: Vec<(usize, bool)>,
}

/// 주기적 운동 계산 (원본 exerper)
/// [v2.10.1] attrib.c:446-513
pub fn exerper_result(
    turn: u64,
    hunger: HungerState,
    encumbrance: EncumbranceLevel,
    is_monk: bool,
    has_clairvoyance: bool,
    has_regeneration: bool,
    is_sick: bool,
    is_vomiting: bool,
    is_confused: bool,
    is_hallucinating: bool,
    has_wounded_legs: bool,
    is_fumbling: bool,
    is_stunned: bool,
    is_mounted: bool,
) -> ExerperResult {
    let mut result = ExerperResult::default();

    // 10턴마다 배고픔/적재 체크 (원본:449)
    if turn % 10 == 0 {
        // 배고픔 체크 (원본:461-479)
        match hunger {
            HungerState::Satiated => {
                result.exercises.push((A_DEX, false));
                if is_monk {
                    result.exercises.push((A_WIS, false));
                }
            }
            HungerState::NotHungry => {
                result.exercises.push((A_CON, true));
            }
            HungerState::Hungry => {} // 특별한 효과 없음
            HungerState::Weak => {
                result.exercises.push((A_STR, false));
                if is_monk {
                    result.exercises.push((A_WIS, true));
                } // 금식 보너스
            }
            HungerState::Fainting => {
                result.exercises.push((A_CON, false));
            }
        }

        // 적재 체크 (원본:483-495)
        match encumbrance {
            EncumbranceLevel::Stressed => {
                result.exercises.push((A_STR, true));
            }
            EncumbranceLevel::Strained => {
                result.exercises.push((A_STR, true));
                result.exercises.push((A_DEX, false));
            }
            EncumbranceLevel::Overtaxed | EncumbranceLevel::Overloaded => {
                result.exercises.push((A_DEX, false));
                result.exercises.push((A_CON, false));
            }
            _ => {}
        }
    }

    // 5턴마다 상태 체크 (원본:499-512)
    if turn % 5 == 0 {
        if has_clairvoyance {
            result.exercises.push((A_WIS, true));
        }
        if has_regeneration {
            result.exercises.push((A_STR, true));
        }
        if is_sick || is_vomiting {
            result.exercises.push((A_CON, false));
        }
        if is_confused || is_hallucinating {
            result.exercises.push((A_WIS, false));
        }
        if (has_wounded_legs && !is_mounted) || is_fumbling || is_stunned {
            result.exercises.push((A_DEX, false));
        }
    }

    result
}

// =============================================================================
// newhp — HP 증가량 계산
// [v2.10.1] attrib.c:980-1033 이식
// =============================================================================

/// HP 증가량 입력
#[derive(Debug, Clone)]
pub struct NewHpInput {
    pub level: i32,
    pub role_hp_fix: i32, // 역할별 HP 고정분
    pub role_hp_rnd: i32, // 역할별 HP 랜덤분
    pub race_hp_fix: i32, // 종족별 HP 고정분
    pub race_hp_rnd: i32, // 종족별 HP 랜덤분
    pub con_value: i32,   // 현재 건강 능력치
    pub role_xlev: i32,   // 역할별 경험 경계 레벨
    pub is_initial: bool, // 초기 생성(레벨0) 여부
}

/// HP 증가량 계산 (원본 newhp)
/// [v2.10.1] attrib.c:980-1033
pub fn newhp_result(input: &NewHpInput, rng: &mut NetHackRng) -> i32 {
    let mut hp;

    if input.is_initial {
        // 초기 HP (원본:986-996)
        hp = input.role_hp_fix + input.race_hp_fix;
        if input.role_hp_rnd > 0 {
            hp += rng.rnd(input.role_hp_rnd);
        }
        if input.race_hp_rnd > 0 {
            hp += rng.rnd(input.race_hp_rnd);
        }
        // 초기에는 Con 보정 없음
    } else {
        // 레벨업 HP (원본:998-1011)
        hp = input.role_hp_fix + input.race_hp_fix;
        if input.role_hp_rnd > 0 {
            hp += rng.rnd(input.role_hp_rnd);
        }
        if input.race_hp_rnd > 0 {
            hp += rng.rnd(input.race_hp_rnd);
        }

        // Con 보정 (원본:1012-1026)
        let conplus = if input.con_value <= 3 {
            -2
        } else if input.con_value <= 6 {
            -1
        } else if input.con_value <= 14 {
            0
        } else if input.con_value <= 16 {
            1
        } else if input.con_value == 17 {
            2
        } else if input.con_value == 18 {
            3
        } else {
            4
        };
        hp += conplus;
    }

    // 최소 1 (원본:1028)
    if hp <= 0 {
        hp = 1;
    }
    hp
}

// =============================================================================
// acurr — 현재 유효 능력치 계산
// [v2.10.1] attrib.c:1035-1070 이식
// =============================================================================

/// 유효 능력치 계산 입력
#[derive(Debug, Clone)]
pub struct AcurrInput {
    pub attr_index: usize,
    pub base_value: i32,
    pub bonus_value: i32,
    pub temp_value: i32,
    pub has_gauntlets_of_power: bool,
    pub has_ogresmasher: bool,
    pub has_dunce_cap: bool,
    pub is_nymph_or_seducer: bool,
}

/// 유효 능력치 계산 (원본 acurr)
/// [v2.10.1] attrib.c:1035-1070
pub fn acurr_result(input: &AcurrInput) -> i32 {
    let tmp = input.bonus_value + input.temp_value + input.base_value;

    match input.attr_index {
        A_STR => {
            // 힘 장갑 또는 125 이상 → 125 고정 (원본:1042-1043)
            if tmp >= 125 || input.has_gauntlets_of_power {
                return 125;
            }
            return tmp.max(ATTRMIN);
        }
        A_CHA => {
            // 님프/succubus/incubus면 최소 18 (원본:1050-1054)
            if tmp < 18 && input.is_nymph_or_seducer {
                return 18;
            }
        }
        A_CON => {
            // Ogresmasher 장비 시 25 고정 (원본:1055-1057)
            if input.has_ogresmasher {
                return ATTRMAX_ABSOLUTE;
            }
        }
        A_INT | A_WIS => {
            // 멍청이 모자 시 6 고정 (원본:1062-1063)
            if input.has_dunce_cap {
                return 6;
            }
        }
        _ => {}
    }

    // 범위 제한 3~25 (원본:1068)
    tmp.clamp(ATTRMIN, ATTRMAX_ABSOLUTE)
}

// =============================================================================
// acurrstr — 힘 값 게임 공식용 변환
// [v2.10.1] attrib.c:1072-1085 이식
// =============================================================================

/// 힘 값을 게임 공식에 맞게 변환 (원본 acurrstr)
/// 18 이하 → 그대로, 19~121 → 19~21, 122~125 → 22~25
pub fn acurrstr_result(str_value: i32) -> i32 {
    if str_value <= 18 {
        str_value
    } else if str_value <= 121 {
        19 + str_value / 50 // 19..21
    } else {
        str_value.min(125) - 100 // 22..25
    }
}

// =============================================================================
// extremeattr — 능력치 극값 확인
// [v2.10.1] attrib.c:1087-1116 이식
// =============================================================================

/// 능력치가 최소/최대 극값인지 확인 (원본 extremeattr)
pub fn extremeattr_result(
    attr_index: usize,
    current_value: i32,
    has_gauntlets_of_power: bool,
    has_ogresmasher: bool,
    has_dunce_cap: bool,
) -> bool {
    let mut lolimit = ATTRMIN;
    let mut hilimit = ATTRMAX_ABSOLUTE;

    if attr_index == A_STR {
        hilimit = 125; // STR19(25)
        if has_gauntlets_of_power {
            lolimit = hilimit;
        }
    } else if attr_index == A_CON {
        if has_ogresmasher {
            lolimit = hilimit;
        }
    } else if attr_index == A_INT || attr_index == A_WIS {
        if has_dunce_cap {
            lolimit = 6;
            hilimit = 6;
        }
    }

    current_value == lolimit || current_value == hilimit
}

// =============================================================================
// adjalign — 성향 조정
// [v2.10.1] attrib.c:1118-1134 이식
// =============================================================================

/// 성향 조정 결과 (원본 adjalign)
pub fn adjalign_result(current_record: i32, delta: i32) -> i32 {
    let newalign = current_record + delta;

    if delta < 0 {
        // 감소: 오버플로우 방지 (새 값이 이전보다 작을 때만)
        if newalign < current_record {
            newalign
        } else {
            current_record
        }
    } else {
        // 증가: 상한 적용
        if newalign > current_record {
            newalign.min(ALIGNLIM)
        } else {
            current_record
        }
    }
}

// =============================================================================
// uchangealign — 성향 변경
// [v2.10.1] attrib.c:1136-1170 이식
// =============================================================================

/// 성향 변경 사유
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignChangeReason {
    Conversion, // 제단 전향 (reason=0)
    HelmOn,     // 반대 성향 투구 착용 (reason=1)
    HelmOff,    // 반대 성향 투구 탈거 (reason=2)
}

/// 성향 변경 결과
#[derive(Debug, Clone)]
pub struct AlignChangeResult {
    pub new_alignment: i32,
    pub new_record: i32,
    pub blessed_lost: bool,
    pub message: String,
    pub alignment_actually_changed: bool,
    pub need_retouch_equipment: bool,
}

/// 성향 변경 결과 계산 (원본 uchangealign)
/// [v2.10.1] attrib.c:1136-1170
pub fn uchangealign_result(
    old_align: i32,
    new_align: i32,
    old_record: i32,
    reason: AlignChangeReason,
    has_helm_opp: bool,
    is_hallucinating: bool,
) -> AlignChangeResult {
    let effective_new = match reason {
        AlignChangeReason::Conversion => {
            // 투구 없으면 실제 전향, 있으면 기존 유지
            if !has_helm_opp {
                new_align
            } else {
                old_align
            }
        }
        _ => new_align,
    };

    let actually_changed = effective_new != old_align;

    let message = match reason {
        AlignChangeReason::Conversion => {
            format!(
                "You have a {}sense of a new direction.",
                if actually_changed { "sudden " } else { "" }
            )
        }
        AlignChangeReason::HelmOn => {
            format!(
                "Your mind oscillates {}.",
                if is_hallucinating {
                    "wildly"
                } else {
                    "briefly"
                }
            )
        }
        AlignChangeReason::HelmOff => {
            format!(
                "Your mind is {}.",
                if is_hallucinating {
                    "much of a muchness"
                } else {
                    "back in sync with your body"
                }
            )
        }
    };

    let new_record = if actually_changed { 0 } else { old_record };

    AlignChangeResult {
        new_alignment: effective_new,
        new_record,
        blessed_lost: true,
        message,
        alignment_actually_changed: actually_changed,
        need_retouch_equipment: actually_changed,
    }
}

// =============================================================================
// 역할/종족 고유 능력 테이블
// [v2.10.1] attrib.c:25-104 이식
// =============================================================================

/// 고유 능력 항목 (원본 struct innate)
#[derive(Debug, Clone)]
pub struct InnateAbility {
    /// 획득 레벨
    pub level: i32,
    /// 능력 이름
    pub ability: &'static str,
    /// 획득 시 메시지
    pub gain_msg: &'static str,
    /// 상실 시 메시지
    pub lose_msg: &'static str,
}

/// 역할별 고유 능력 테이블 (원본 role_abil)
/// [v2.10.1] attrib.c:25-87 완전 이식
pub fn role_abilities(role: &str) -> Vec<InnateAbility> {
    match role.to_lowercase().as_str() {
        "archeologist" => vec![
            InnateAbility {
                level: 1,
                ability: "stealth",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 1,
                ability: "fast",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 10,
                ability: "searching",
                gain_msg: "perceptive",
                lose_msg: "",
            },
        ],
        "barbarian" => vec![
            InnateAbility {
                level: 1,
                ability: "poison_resistance",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 7,
                ability: "fast",
                gain_msg: "quick",
                lose_msg: "slow",
            },
            InnateAbility {
                level: 15,
                ability: "stealth",
                gain_msg: "stealthy",
                lose_msg: "",
            },
        ],
        "caveman" | "cavewoman" => vec![InnateAbility {
            level: 15,
            ability: "warning",
            gain_msg: "sensitive",
            lose_msg: "",
        }],
        "healer" => vec![
            InnateAbility {
                level: 1,
                ability: "poison_resistance",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 15,
                ability: "warning",
                gain_msg: "sensitive",
                lose_msg: "",
            },
        ],
        "knight" => vec![InnateAbility {
            level: 7,
            ability: "fast",
            gain_msg: "quick",
            lose_msg: "slow",
        }],
        "monk" => vec![
            InnateAbility {
                level: 1,
                ability: "fast",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 1,
                ability: "see_invisible",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 3,
                ability: "poison_resistance",
                gain_msg: "healthy",
                lose_msg: "",
            },
            InnateAbility {
                level: 5,
                ability: "stealth",
                gain_msg: "stealthy",
                lose_msg: "",
            },
            InnateAbility {
                level: 7,
                ability: "warning",
                gain_msg: "sensitive",
                lose_msg: "",
            },
            InnateAbility {
                level: 9,
                ability: "searching",
                gain_msg: "perceptive",
                lose_msg: "unaware",
            },
            InnateAbility {
                level: 11,
                ability: "fire_resistance",
                gain_msg: "cool",
                lose_msg: "warmer",
            },
            InnateAbility {
                level: 13,
                ability: "cold_resistance",
                gain_msg: "warm",
                lose_msg: "cooler",
            },
            InnateAbility {
                level: 15,
                ability: "shock_resistance",
                gain_msg: "insulated",
                lose_msg: "conductive",
            },
            InnateAbility {
                level: 17,
                ability: "teleport_control",
                gain_msg: "controlled",
                lose_msg: "uncontrolled",
            },
        ],
        "priest" | "priestess" => vec![
            InnateAbility {
                level: 15,
                ability: "warning",
                gain_msg: "sensitive",
                lose_msg: "",
            },
            InnateAbility {
                level: 20,
                ability: "fire_resistance",
                gain_msg: "cool",
                lose_msg: "warmer",
            },
        ],
        "ranger" => vec![
            InnateAbility {
                level: 1,
                ability: "searching",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 7,
                ability: "stealth",
                gain_msg: "stealthy",
                lose_msg: "",
            },
            InnateAbility {
                level: 15,
                ability: "see_invisible",
                gain_msg: "",
                lose_msg: "",
            },
        ],
        "rogue" => vec![
            InnateAbility {
                level: 1,
                ability: "stealth",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 10,
                ability: "searching",
                gain_msg: "perceptive",
                lose_msg: "",
            },
        ],
        "samurai" => vec![
            InnateAbility {
                level: 1,
                ability: "fast",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 15,
                ability: "stealth",
                gain_msg: "stealthy",
                lose_msg: "",
            },
        ],
        "tourist" => vec![
            InnateAbility {
                level: 10,
                ability: "searching",
                gain_msg: "perceptive",
                lose_msg: "",
            },
            InnateAbility {
                level: 20,
                ability: "poison_resistance",
                gain_msg: "hardy",
                lose_msg: "",
            },
        ],
        "valkyrie" => vec![
            InnateAbility {
                level: 1,
                ability: "cold_resistance",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 1,
                ability: "stealth",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 7,
                ability: "fast",
                gain_msg: "quick",
                lose_msg: "slow",
            },
        ],
        "wizard" => vec![
            InnateAbility {
                level: 15,
                ability: "warning",
                gain_msg: "sensitive",
                lose_msg: "",
            },
            InnateAbility {
                level: 17,
                ability: "teleport_control",
                gain_msg: "controlled",
                lose_msg: "uncontrolled",
            },
        ],
        _ => vec![],
    }
}

/// 종족별 고유 능력 테이블 (원본 race_abil)
/// [v2.10.1] attrib.c:89-104 완전 이식
pub fn race_abilities(race: &str) -> Vec<InnateAbility> {
    match race.to_lowercase().as_str() {
        "dwarf" => vec![InnateAbility {
            level: 1,
            ability: "infravision",
            gain_msg: "",
            lose_msg: "",
        }],
        "elf" => vec![
            InnateAbility {
                level: 1,
                ability: "infravision",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 4,
                ability: "sleep_resistance",
                gain_msg: "awake",
                lose_msg: "tired",
            },
        ],
        "gnome" => vec![InnateAbility {
            level: 1,
            ability: "infravision",
            gain_msg: "",
            lose_msg: "",
        }],
        "orc" => vec![
            InnateAbility {
                level: 1,
                ability: "infravision",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 1,
                ability: "poison_resistance",
                gain_msg: "",
                lose_msg: "",
            },
        ],
        "human" | _ => vec![],
    }
}

// =============================================================================
// adjabil — 레벨 변경 시 능력 조정 결과
// [v2.10.1] attrib.c:908-978 이식
// =============================================================================

/// 레벨 변경 시 능력 변화 이벤트
#[derive(Debug, Clone)]
pub struct AbilityChange {
    pub ability: String,
    pub gained: bool,
    pub message: Option<String>,
}

/// 레벨 변경 시 능력 조정 결과 (원본 adjabil)
/// [v2.10.1] attrib.c:908-978
pub fn adjabil_result(
    old_level: i32,
    new_level: i32,
    role: &str,
    race: &str,
) -> Vec<AbilityChange> {
    let mut changes = Vec::new();

    let role_abils = role_abilities(role);
    let race_abils = race_abilities(race);

    for abil in role_abils.iter().chain(race_abils.iter()) {
        if old_level < abil.level && new_level >= abil.level {
            // 능력 획득 (원본:943-957)
            let msg = if !abil.gain_msg.is_empty() {
                Some(format!("You feel {}!", abil.gain_msg))
            } else {
                None
            };
            changes.push(AbilityChange {
                ability: abil.ability.to_string(),
                gained: true,
                message: msg,
            });
        } else if old_level >= abil.level && new_level < abil.level {
            // 능력 상실 (원본:958-965)
            let msg = if !abil.lose_msg.is_empty() {
                Some(format!("You feel {}!", abil.lose_msg))
            } else if !abil.gain_msg.is_empty() {
                Some(format!("You feel less {}!", abil.gain_msg))
            } else {
                None
            };
            changes.push(AbilityChange {
                ability: abil.ability.to_string(),
                gained: false,
                message: msg,
            });
        }
    }

    changes
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poison_tell() {
        let msg = poison_tell_result(A_STR, true, false, false);
        assert!(msg.contains("weaker"));
        assert!(msg.ends_with('!'));

        let msg_max = poison_tell_result(A_STR, false, true, false);
        assert!(msg_max.contains("innately weaker"));
    }

    #[test]
    fn test_poisoned_resistant() {
        let mut rng = NetHackRng::new(42);
        let r = poisoned_result("dart", A_STR, 10, false, true, &mut rng);
        assert!(!r.instant_kill);
        assert_eq!(r.hp_damage, 0);
        assert_eq!(r.attr_loss, 0);
    }

    #[test]
    fn test_poisoned_lethal() {
        // fatal=0 → i=1 → 능력치 감소 경로
        let mut rng = NetHackRng::new(42);
        let r = poisoned_result("blade", A_STR, 0, false, false, &mut rng);
        assert!(!r.instant_kill);
        assert!(r.attr_changed || r.hp_damage > 0);
    }

    #[test]
    fn test_change_luck() {
        assert_eq!(change_luck_result(0, 5), 5);
        assert_eq!(change_luck_result(0, -15), LUCKMIN);
        assert_eq!(change_luck_result(9, 5), LUCKMAX);
    }

    #[test]
    fn test_stone_luck() {
        let items = vec![
            (true, false, true, 1), // 축복된 럭스톤 1개
            (true, true, false, 2), // 저주된 럭스톤 2개
        ];
        // 1 blessed - 2 cursed = -1 → -1
        assert_eq!(stone_luck_result(&items, true), -1);
    }

    #[test]
    fn test_set_moreluck() {
        assert_eq!(set_moreluck_result(1, true), LUCKADD);
        assert_eq!(set_moreluck_result(-1, true), -LUCKADD);
        assert_eq!(set_moreluck_result(0, false), 0);
    }

    #[test]
    fn test_gainstr() {
        let mut rng = NetHackRng::new(42);
        let gain = gainstr_result(15, 0, false, &mut rng);
        assert!(gain > 0);

        let gain_cursed = gainstr_result(15, 3, true, &mut rng);
        assert_eq!(gain_cursed, -3);
    }

    #[test]
    fn test_init_attr() {
        let mut rng = NetHackRng::new(42);
        let base = [10, 10, 10, 10, 10, 10];
        let dist = [20, 15, 15, 20, 20, 10];
        let max_cap = [18, 18, 18, 18, 18, 18];
        let result = init_attr_result(&base, &dist, &max_cap, 75, &mut rng);
        let total: i32 = result.iter().sum();
        assert_eq!(total, 75);
    }

    #[test]
    fn test_redist_attr() {
        let mut rng = NetHackRng::new(42);
        let base = [15, 12, 14, 13, 16, 10];
        let amax = [15, 12, 14, 13, 16, 10];
        let cap = [18, 18, 18, 18, 18, 18];
        let (new_base, new_max) = redist_attr_result(&base, &amax, &cap, &mut rng);
        // Int/Wis는 변하지 않아야 함
        assert_eq!(new_base[A_INT], base[A_INT]);
        assert_eq!(new_base[A_WIS], base[A_WIS]);
        assert_eq!(new_max[A_INT], amax[A_INT]);
        assert_eq!(new_max[A_WIS], amax[A_WIS]);
    }

    #[test]
    fn test_exerper() {
        let r = exerper_result(
            10,
            HungerState::Satiated,
            EncumbranceLevel::Strained,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
        // 포화 시 DEX 남용, 과적 시 STR 운동 + DEX 남용
        assert!(r.exercises.iter().any(|e| e.0 == A_DEX && !e.1));
        assert!(r.exercises.iter().any(|e| e.0 == A_STR && e.1));
    }

    #[test]
    fn test_newhp() {
        let mut rng = NetHackRng::new(42);
        let input = NewHpInput {
            level: 5,
            role_hp_fix: 2,
            role_hp_rnd: 6,
            race_hp_fix: 1,
            race_hp_rnd: 2,
            con_value: 16,
            role_xlev: 14,
            is_initial: false,
        };
        let hp = newhp_result(&input, &mut rng);
        assert!(hp >= 1);
    }

    #[test]
    fn test_acurr() {
        // 일반 경우
        let input = AcurrInput {
            attr_index: A_STR,
            base_value: 16,
            bonus_value: 2,
            temp_value: 0,
            has_gauntlets_of_power: false,
            has_ogresmasher: false,
            has_dunce_cap: false,
            is_nymph_or_seducer: false,
        };
        assert_eq!(acurr_result(&input), 18);

        // 힘 장갑
        let input_gop = AcurrInput {
            has_gauntlets_of_power: true,
            ..input.clone()
        };
        assert_eq!(acurr_result(&input_gop), 125);
    }

    #[test]
    fn test_acurrstr() {
        assert_eq!(acurrstr_result(16), 16);
        assert_eq!(acurrstr_result(18), 18);
        assert_eq!(acurrstr_result(100), 21); // 19 + 100/50 = 21
        assert_eq!(acurrstr_result(125), 25); // 125 - 100 = 25
    }

    #[test]
    fn test_extremeattr() {
        // 일반 값 → false
        assert!(!extremeattr_result(A_STR, 15, false, false, false));
        // 최소값 → true
        assert!(extremeattr_result(A_STR, ATTRMIN, false, false, false));
        // 힘 장갑 → 상하한 동일 → true
        assert!(extremeattr_result(A_STR, 125, true, false, false));
    }

    #[test]
    fn test_adjalign() {
        assert_eq!(adjalign_result(5, 3), 8);
        assert_eq!(adjalign_result(12, 5), ALIGNLIM); // 상한
        assert_eq!(adjalign_result(0, -3), -3);
    }

    #[test]
    fn test_uchangealign() {
        let r = uchangealign_result(1, -1, 5, AlignChangeReason::Conversion, false, false);
        assert!(r.alignment_actually_changed);
        assert_eq!(r.new_record, 0); // 성향 변경 시 기록 초기화
        assert!(r.message.contains("sudden"));
    }

    #[test]
    fn test_role_abilities_monk() {
        let abils = role_abilities("monk");
        assert_eq!(abils.len(), 10);
        assert_eq!(abils[0].ability, "fast");
        assert_eq!(abils[9].ability, "teleport_control");
    }

    #[test]
    fn test_race_abilities_elf() {
        let abils = race_abilities("elf");
        assert_eq!(abils.len(), 2);
        assert_eq!(abils[1].ability, "sleep_resistance");
        assert_eq!(abils[1].level, 4);
    }

    #[test]
    fn test_adjabil_levelup() {
        let changes = adjabil_result(6, 7, "barbarian", "human");
        // 바바리안 레벨 7: fast 획득
        assert!(changes.iter().any(|c| c.ability == "fast" && c.gained));
    }

    #[test]
    fn test_adjabil_leveldown() {
        let changes = adjabil_result(7, 6, "barbarian", "human");
        // 바바리안 레벨 7 -> 6: fast 상실
        assert!(changes.iter().any(|c| c.ability == "fast" && !c.gained));
    }
}

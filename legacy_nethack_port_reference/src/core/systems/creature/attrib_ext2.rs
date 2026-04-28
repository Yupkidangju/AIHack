// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-2] 능력치 관리 확장 모듈 (attrib_ext2.rs)
// 원본: NetHack 3.6.7 attrib.c (순수 계산 로직)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 상수 및 열거형 (원본: attrib.c/you.h 매크로)
// =============================================================================

/// 능력치 인덱스 (원본: A_STR, A_INT, A_WIS, A_DEX, A_CON, A_CHA)
pub const A_STR: usize = 0;
pub const A_INT: usize = 1;
pub const A_WIS: usize = 2;
pub const A_DEX: usize = 3;
pub const A_CON: usize = 4;
pub const A_CHA: usize = 5;
pub const A_MAX: usize = 6;

/// 능력치 이름 (원본: attrname[])
pub const ATTR_NAMES: [&str; A_MAX] = [
    "strength",
    "intelligence",
    "wisdom",
    "dexterity",
    "constitution",
    "charisma",
];

/// 기본 능력치 최솟값 (원본: ATTRMIN, 보통 3)
pub const ATTRMIN_DEFAULT: i32 = 3;
/// 기본 능력치 최댓값 (원본: ATTRMAX, 보통 18)
pub const ATTRMAX_DEFAULT: i32 = 18;
/// 절대 최댓값 (25: 갑옷/마법 보정 포함)
pub const ATTRMAX_ABSOLUTE: i32 = 25;

/// 행운 범위 (원본: LUCKMIN=-10, LUCKMAX=10, LUCKADD=3)
pub const LUCKMIN: i32 = -10;
pub const LUCKMAX: i32 = 10;
pub const LUCKADD: i32 = 3;

/// 훈련 값 임계치 (원본: AVAL = 50)
pub const AVAL: i32 = 50;

/// [v2.22.0 R34-2] 배고픔 상태 (원본: hunger.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HungerState {
    Satiated = 0,
    NotHungry = 1,
    Hungry = 2,
    Weak = 3,
    Fainting = 4,
    Fainted = 5,
    Starved = 6,
}

/// [v2.22.0 R34-2] 짐 부하 상태 (원본: hack.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EncumbranceLevel {
    Unencumbered = 0,
    Burdened = 1,
    Stressed = 2,
    Strained = 3,
    Overtaxed = 4,
    Overloaded = 5,
}

/// [v2.22.0 R34-2] 훈련 대상 능력치에 대한 증감 요청
#[derive(Debug, Clone)]
pub struct ExerciseRequest {
    pub attr_index: usize,
    pub increase: bool,
}

// =============================================================================
// [2] 능력치 스냅샷 (순수 함수에서 사용할 불변 데이터)
// =============================================================================

/// [v2.22.0 R34-2] 플레이어의 능력치 상태 스냅샷
/// 원본의 전역 변수(ABASE, AMAX, ATEMP, ATIME, AEXE 등)를 구조체로 캡슐화
#[derive(Debug, Clone)]
pub struct AttributeSnapshot {
    /// 기본 능력치 [0..5]
    pub abase: [i32; A_MAX],
    /// 최대 능력치 (도달 가능한 최고치)
    pub amax: [i32; A_MAX],
    /// 임시 보정 (배고픔 등에 의한)
    pub atemp: [i32; A_MAX],
    /// 임시 보정 지속 타이머
    pub atime: [i32; A_MAX],
    /// 훈련 축적치 (양수=성장, 음수=퇴화)
    pub aexe: [i32; A_MAX],
    /// 능력치별 최솟값 (종족/상태에 따라 가변)
    pub attr_min: [i32; A_MAX],
    /// 능력치별 최댓값 (종족/상태에 따라 가변)
    pub attr_max: [i32; A_MAX],
}

impl Default for AttributeSnapshot {
    fn default() -> Self {
        Self {
            abase: [10; A_MAX],
            amax: [10; A_MAX],
            atemp: [0; A_MAX],
            atime: [0; A_MAX],
            aexe: [0; A_MAX],
            attr_min: [ATTRMIN_DEFAULT; A_MAX],
            attr_max: [ATTRMAX_DEFAULT; A_MAX],
        }
    }
}

impl AttributeSnapshot {
    /// 현재 유효 능력치 (base + temp)
    /// 원본: ACURR(x) = ABASE(x) + ATEMP(x) + ABON(x) (여기서는 ABON 생략)
    pub fn current(&self, idx: usize) -> i32 {
        let raw = self.abase[idx] + self.atemp[idx];
        raw.clamp(self.attr_min[idx], ATTRMAX_ABSOLUTE)
    }
}

// =============================================================================
// [3] 능력치 조정 결과 (원본: adjattrib 반환 + 부수효과)
// =============================================================================

/// [v2.22.0 R34-2] 능력치 조정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjAttribResult {
    /// 성공적으로 변경됨
    Changed {
        attr_index: usize,
        old_value: i32,
        new_value: i32,
        new_base: i32,
        new_max: i32,
    },
    /// 변경 없음 (이미 최대/최소이거나 Fixed_abil)
    NoChange { reason: &'static str },
}

/// [v2.22.0 R34-2] 능력치 조정 순수 계산 (원본: attrib.c:112 adjattrib)
/// `incr`: 양수면 증가, 음수면 감소
/// `fixed_abil`: 능력치 고정 상태인지
/// `dunce_cap_worn`: 바보모자 착용 중인지
pub fn calc_adjattrib(
    snap: &AttributeSnapshot,
    ndx: usize,
    incr: i32,
    fixed_abil: bool,
    dunce_cap_worn: bool,
    rng: &mut NetHackRng,
) -> (AdjAttribResult, AttributeSnapshot) {
    let mut new_snap = snap.clone();

    // 능력치 고정 또는 변화량 0
    if fixed_abil || incr == 0 {
        return (
            AdjAttribResult::NoChange {
                reason: "능력치 고정 또는 변화량 0",
            },
            new_snap,
        );
    }

    // 바보모자: 지능/지혜 변경 불가
    if (ndx == A_INT || ndx == A_WIS) && dunce_cap_worn {
        return (
            AdjAttribResult::NoChange {
                reason: "바보모자 착용으로 지능/지혜 변경 불가",
            },
            new_snap,
        );
    }

    let old_base = new_snap.abase[ndx];
    let old_max = new_snap.amax[ndx];
    let old_current = new_snap.current(ndx);

    // 기본 능력치 변경
    new_snap.abase[ndx] += incr;

    if incr > 0 {
        // 증가: base가 max를 넘으면 max도 같이 올림
        if new_snap.abase[ndx] > new_snap.amax[ndx] {
            new_snap.amax[ndx] = new_snap.abase[ndx];
            if new_snap.amax[ndx] > new_snap.attr_max[ndx] {
                new_snap.abase[ndx] = new_snap.attr_max[ndx];
                new_snap.amax[ndx] = new_snap.attr_max[ndx];
            }
        }
    } else {
        // 감소: base가 min 아래로 가면 max에서 일부 깎음
        if new_snap.abase[ndx] < new_snap.attr_min[ndx] {
            let excess = new_snap.attr_min[ndx] - new_snap.abase[ndx];
            let decr = rng.rn2(excess + 1);
            new_snap.abase[ndx] = new_snap.attr_min[ndx];
            new_snap.amax[ndx] -= decr;
            if new_snap.amax[ndx] < new_snap.attr_min[ndx] {
                new_snap.amax[ndx] = new_snap.attr_min[ndx];
            }
        }
    }

    let new_current = new_snap.current(ndx);

    if new_current == old_current {
        (
            AdjAttribResult::NoChange {
                reason: "유효 능력치 변화 없음",
            },
            new_snap,
        )
    } else {
        (
            AdjAttribResult::Changed {
                attr_index: ndx,
                old_value: old_current,
                new_value: new_current,
                new_base: new_snap.abase[ndx],
                new_max: new_snap.amax[ndx],
            },
            new_snap,
        )
    }
}

// =============================================================================
// [4] 운(Luck) 관련 순수 함수 (원본: change_luck, stone_luck)
// =============================================================================

/// [v2.22.0 R34-2] 운 변경 (원본: attrib.c:339 change_luck)
pub fn calc_change_luck(current_luck: i32, delta: i32) -> i32 {
    let mut new_luck = current_luck + delta;
    if new_luck < 0 && new_luck < LUCKMIN {
        new_luck = LUCKMIN;
    }
    if new_luck > 0 && new_luck > LUCKMAX {
        new_luck = LUCKMAX;
    }
    new_luck
}

/// [v2.22.0 R34-2] 행운석 보정 계산 (원본: attrib.c:350 stone_luck)
/// `luck_items`: (축복 여부, 저주 여부, 수량) 목록
/// `parameter`: true면 일반 아이템도 +1 보정
pub fn calc_stone_luck(luck_items: &[(bool, bool, i32)], parameter: bool) -> i32 {
    let mut bonchance: i64 = 0;
    for &(blessed, cursed, qty) in luck_items {
        if cursed {
            bonchance -= qty as i64;
        } else if blessed {
            bonchance += qty as i64;
        } else if parameter {
            bonchance += qty as i64;
        }
    }
    // sgn
    if bonchance > 0 {
        1
    } else if bonchance < 0 {
        -1
    } else {
        0
    }
}

/// [v2.22.0 R34-2] 행운 보정 설정 (원본: attrib.c:370 set_moreluck)
/// `stone_luck_val`: stone_luck(TRUE)의 반환값
/// `carrying_luckstone`: 인벤토리에 행운석을 소지 중인지
pub fn calc_moreluck(stone_luck_val: i32, carrying_luckstone: bool) -> i32 {
    if stone_luck_val == 0 && !carrying_luckstone {
        0
    } else if stone_luck_val >= 0 {
        LUCKADD
    } else {
        -LUCKADD
    }
}

// =============================================================================
// [5] 능력치 초기화/재분배 (원본: init_attr, redist_attr)
// =============================================================================

/// [v2.22.0 R34-2] 능력치 초기화 (원본: attrib.c:613 init_attr)
/// `total_points`: 분배할 총 포인트
/// `base_attrs`: 직업 기본 능력치 [A_MAX]
/// `attr_dist`: 직업별 능력치 분배 확률 [A_MAX] (합계 100)
/// `attr_max`: 능력치별 최댓값 [A_MAX]
pub fn calc_init_attr(
    total_points: i32,
    base_attrs: &[i32; A_MAX],
    attr_dist: &[i32; A_MAX],
    attr_max: &[i32; A_MAX],
    rng: &mut NetHackRng,
) -> ([i32; A_MAX], [i32; A_MAX]) {
    let mut abase = *base_attrs;
    let mut amax = *base_attrs;
    let mut np = total_points;

    // 기본값에서 총 포인트 차감
    for i in 0..A_MAX {
        np -= base_attrs[i];
    }

    // 남은 포인트 분배 (양수)
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
        if abase[i] >= attr_max[i] {
            tryct += 1;
            continue;
        }
        tryct = 0;
        abase[i] += 1;
        amax[i] += 1;
        np -= 1;
    }

    // 초과 포인트 회수 (음수)
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
        if abase[i] <= ATTRMIN_DEFAULT {
            tryct += 1;
            continue;
        }
        tryct = 0;
        abase[i] -= 1;
        amax[i] -= 1;
        np += 1;
    }

    (abase, amax)
}

/// [v2.22.0 R34-2] 능력치 재분배 - 폴리모프 시 (원본: attrib.c:663 redist_attr)
/// 지능/지혜는 변경하지 않음 (폴리모프가 정신은 바꾸지 않으므로)
pub fn calc_redist_attr(snap: &AttributeSnapshot, rng: &mut NetHackRng) -> AttributeSnapshot {
    let mut new_snap = snap.clone();

    for i in 0..A_MAX {
        // 지능과 지혜는 건드리지 않음
        if i == A_INT || i == A_WIS {
            continue;
        }

        let old_max = new_snap.amax[i];
        if old_max == 0 {
            continue;
        }

        // max를 ±2 범위에서 변경
        let delta = rng.rn2(5) - 2;
        new_snap.amax[i] += delta;
        if new_snap.amax[i] > new_snap.attr_max[i] {
            new_snap.amax[i] = new_snap.attr_max[i];
        }
        if new_snap.amax[i] < new_snap.attr_min[i] {
            new_snap.amax[i] = new_snap.attr_min[i];
        }

        // base를 비례 조정
        new_snap.abase[i] = new_snap.abase[i] * new_snap.amax[i] / old_max;
        if new_snap.abase[i] < new_snap.attr_min[i] {
            new_snap.abase[i] = new_snap.attr_min[i];
        }
    }

    new_snap
}

// =============================================================================
// [6] 훈련(Exercise) 시스템 (원본: exercise, exerper)
// =============================================================================

/// [v2.22.0 R34-2] 훈련 축적 계산 (원본: attrib.c:412 exercise)
/// 지능/카리스마는 훈련 불가, 폴리모프 중 지혜 외 훈련 불가
pub fn calc_exercise(
    attr_index: usize,
    increase: bool,
    current_aexe: i32,
    current_acurr: i32,
    is_polymorphed: bool,
    rng: &mut NetHackRng,
) -> i32 {
    // 지능/카리스마는 훈련 불가
    if attr_index == A_INT || attr_index == A_CHA {
        return current_aexe;
    }

    // 폴리모프 중 지혜 외 훈련 불가
    if is_polymorphed && attr_index != A_WIS {
        return current_aexe;
    }

    if current_aexe.abs() < AVAL {
        if increase {
            // 성장: 높은 능력치일수록 어려움 (19 - ACURR) / 19 확률
            if rng.rn2(19) > current_acurr {
                return current_aexe + 1;
            }
        } else {
            // 퇴화: 50% 확률로 -1
            return current_aexe - rng.rn2(2);
        }
    }

    current_aexe
}

/// [v2.22.0 R34-2] 배고픔/짐에 의한 주기적 훈련 요청 생성
/// (원본: attrib.c:446 exerper)
/// 10턴마다 배고픔/짐 체크, 5턴마다 상태 체크
pub fn calc_exerper(
    turn: i64,
    hunger: HungerState,
    encumbrance: EncumbranceLevel,
    is_monk: bool,
    has_clairvoyance: bool,
    has_regeneration: bool,
    is_sick_or_vomiting: bool,
    is_confused_or_hallucinating: bool,
    has_wounded_legs_no_steed: bool,
    is_fumbling: bool,
    is_stunned: bool,
) -> Vec<ExerciseRequest> {
    let mut requests = Vec::new();

    // 10턴 주기: 배고픔/짐 체크
    if turn % 10 == 0 {
        // 배고픔
        match hunger {
            HungerState::Satiated => {
                requests.push(ExerciseRequest {
                    attr_index: A_DEX,
                    increase: false,
                });
                if is_monk {
                    requests.push(ExerciseRequest {
                        attr_index: A_WIS,
                        increase: false,
                    });
                }
            }
            HungerState::NotHungry => {
                requests.push(ExerciseRequest {
                    attr_index: A_CON,
                    increase: true,
                });
            }
            HungerState::Weak => {
                requests.push(ExerciseRequest {
                    attr_index: A_STR,
                    increase: false,
                });
                if is_monk {
                    requests.push(ExerciseRequest {
                        attr_index: A_WIS,
                        increase: true,
                    });
                }
            }
            HungerState::Fainting | HungerState::Fainted => {
                requests.push(ExerciseRequest {
                    attr_index: A_CON,
                    increase: false,
                });
            }
            _ => {}
        }

        // 짐
        match encumbrance {
            EncumbranceLevel::Stressed => {
                requests.push(ExerciseRequest {
                    attr_index: A_STR,
                    increase: true,
                });
            }
            EncumbranceLevel::Strained => {
                requests.push(ExerciseRequest {
                    attr_index: A_STR,
                    increase: true,
                });
                requests.push(ExerciseRequest {
                    attr_index: A_DEX,
                    increase: false,
                });
            }
            EncumbranceLevel::Overtaxed | EncumbranceLevel::Overloaded => {
                requests.push(ExerciseRequest {
                    attr_index: A_DEX,
                    increase: false,
                });
                requests.push(ExerciseRequest {
                    attr_index: A_CON,
                    increase: false,
                });
            }
            _ => {}
        }
    }

    // 5턴 주기: 상태 체크
    if turn % 5 == 0 {
        if has_clairvoyance {
            requests.push(ExerciseRequest {
                attr_index: A_WIS,
                increase: true,
            });
        }
        if has_regeneration {
            requests.push(ExerciseRequest {
                attr_index: A_STR,
                increase: true,
            });
        }
        if is_sick_or_vomiting {
            requests.push(ExerciseRequest {
                attr_index: A_CON,
                increase: false,
            });
        }
        if is_confused_or_hallucinating {
            requests.push(ExerciseRequest {
                attr_index: A_WIS,
                increase: false,
            });
        }
        if has_wounded_legs_no_steed || is_fumbling || is_stunned {
            requests.push(ExerciseRequest {
                attr_index: A_DEX,
                increase: false,
            });
        }
    }

    requests
}

/// [v2.22.0 R34-2] 훈련 결과 확인 (원본: attrib.c:526 exerchk 핵심 로직)
/// 충분한 훈련이 축적되면 능력치 변경 시도
pub fn calc_exerchk_result(
    attr_index: usize,
    aexe: i32,
    abase: i32,
    attr_min: i32,
    attr_max_capped: i32, // min(ATTRMAX, 18)
    is_polymorphed: bool,
    rng: &mut NetHackRng,
) -> Option<(i32, i32)> {
    // 훈련 축적 없음
    if aexe == 0 {
        return None;
    }

    let mod_val = if aexe > 0 { 1 } else { -1 };

    // 이미 최대/최소에 도달
    if (aexe < 0 && abase <= attr_min) || (aexe > 0 && abase >= attr_max_capped) {
        // 훈련은 감쇠만
        let new_aexe = (aexe.abs() / 2) * mod_val;
        return Some((0, new_aexe));
    }

    // 폴리모프 중 지혜 외 불가
    if is_polymorphed && attr_index != A_WIS {
        let new_aexe = (aexe.abs() / 2) * mod_val;
        return Some((0, new_aexe));
    }

    // 체크: AVAL 기반 확률 (지혜는 더 쉬움)
    let threshold = if attr_index != A_WIS {
        (aexe.abs() * 2) / 3
    } else {
        aexe.abs()
    };

    if rng.rn2(AVAL) > threshold {
        // 실패 — 훈련 감쇠만
        let new_aexe = (aexe.abs() / 2) * mod_val;
        return Some((0, new_aexe));
    }

    // 성공 — 능력치 +1/-1 변경 + 훈련 초기화
    Some((mod_val, 0))
}

// =============================================================================
// [7] 독 피해 계산 (원본: attrib.c:271 poisoned 핵심 로직)
// =============================================================================

/// [v2.22.0 R34-2] 독 피해 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoisonResult {
    /// 독 저항으로 무효
    Resisted,
    /// 즉사 (fatal roll == 0)
    InstantDeath,
    /// HP 감소
    HpDamage { amount: i32 },
    /// 능력치 감소
    StatLoss { attr_index: usize, amount: i32 },
}

/// [v2.22.0 R34-2] 독 피해 계산 (원본: attrib.c:311-328)
/// `typ`: 영향받는 능력치 인덱스
/// `fatal`: 치명도 (0이면 능력치만 감소, 그 외 즉사 확률 존재)
/// `thrown_weapon`: 투척 무기면 덜 치명적
/// `has_poison_resistance`: 독 저항 여부
pub fn calc_poison_damage(
    typ: usize,
    fatal: i32,
    thrown_weapon: bool,
    has_poison_resistance: bool,
    rng: &mut NetHackRng,
) -> PoisonResult {
    if has_poison_resistance {
        return PoisonResult::Resisted;
    }

    let i = if fatal == 0 {
        1 // 비치명적: 능력치만 감소
    } else {
        rng.rn2(fatal + if thrown_weapon { 20 } else { 0 })
    };

    if i == 0 && typ != A_CHA {
        // 즉사 (카리스마 독은 즉사 안 함, 원본 동일)
        PoisonResult::InstantDeath
    } else if i > 5 {
        // HP 피해
        let loss = if thrown_weapon {
            rng.rnd(6)
        } else {
            rng.rn1(10, 6)
        };
        PoisonResult::HpDamage { amount: loss }
    } else {
        // 능력치 감소
        let loss = if thrown_weapon || fatal == 0 {
            1
        } else {
            // d(2,2) = 2d2
            rng.rnd(2) + rng.rnd(2)
        };
        PoisonResult::StatLoss {
            attr_index: typ,
            amount: loss,
        }
    }
}

// =============================================================================
// [8] 직업별 내재 능력 테이블 (원본: attrib.c:25-104 innate 구조체들)
// =============================================================================

/// [v2.22.0 R34-2] 내재 능력 항목 (원본: struct innate)
#[derive(Debug, Clone)]
pub struct InnateAbility {
    /// 획득 레벨
    pub level: i32,
    /// 능력 이름 (예: "stealth", "fire_resistance")
    pub ability_name: &'static str,
    /// 획득 시 메시지
    pub gain_msg: &'static str,
    /// 상실 시 메시지
    pub lose_msg: &'static str,
}

/// [v2.22.0 R34-2] 직업별 내재 능력 테이블 반환 (원본: role_abil)
pub fn get_role_abilities(role_name: &str) -> Vec<InnateAbility> {
    match role_name {
        "Archeologist" => vec![
            InnateAbility {
                level: 1,
                ability_name: "stealth",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 1,
                ability_name: "fast",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 10,
                ability_name: "searching",
                gain_msg: "perceptive",
                lose_msg: "",
            },
        ],
        "Barbarian" => vec![
            InnateAbility {
                level: 1,
                ability_name: "poison_resistance",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 7,
                ability_name: "fast",
                gain_msg: "quick",
                lose_msg: "slow",
            },
            InnateAbility {
                level: 15,
                ability_name: "stealth",
                gain_msg: "stealthy",
                lose_msg: "",
            },
        ],
        "Caveman" => vec![InnateAbility {
            level: 15,
            ability_name: "warning",
            gain_msg: "sensitive",
            lose_msg: "",
        }],
        "Knight" => vec![InnateAbility {
            level: 7,
            ability_name: "fast",
            gain_msg: "quick",
            lose_msg: "slow",
        }],
        "Monk" => vec![
            InnateAbility {
                level: 1,
                ability_name: "fast",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 1,
                ability_name: "sleep_resistance",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 3,
                ability_name: "see_invisible",
                gain_msg: "see clearly",
                lose_msg: "see less clearly",
            },
            InnateAbility {
                level: 5,
                ability_name: "poison_resistance",
                gain_msg: "healthy",
                lose_msg: "less healthy",
            },
            InnateAbility {
                level: 7,
                ability_name: "warning",
                gain_msg: "sensitive",
                lose_msg: "",
            },
            InnateAbility {
                level: 9,
                ability_name: "searching",
                gain_msg: "perceptive",
                lose_msg: "less perceptive",
            },
            InnateAbility {
                level: 11,
                ability_name: "fire_resistance",
                gain_msg: "cool",
                lose_msg: "warmer",
            },
            InnateAbility {
                level: 13,
                ability_name: "cold_resistance",
                gain_msg: "warm",
                lose_msg: "cooler",
            },
            InnateAbility {
                level: 15,
                ability_name: "shock_resistance",
                gain_msg: "insulated",
                lose_msg: "conductive",
            },
            InnateAbility {
                level: 17,
                ability_name: "teleport_control",
                gain_msg: "controlled",
                lose_msg: "uncontrolled",
            },
        ],
        "Ranger" => vec![
            InnateAbility {
                level: 1,
                ability_name: "searching",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 7,
                ability_name: "stealth",
                gain_msg: "stealthy",
                lose_msg: "",
            },
            InnateAbility {
                level: 15,
                ability_name: "see_invisible",
                gain_msg: "see clearly",
                lose_msg: "see less clearly",
            },
        ],
        "Rogue" => vec![
            InnateAbility {
                level: 1,
                ability_name: "stealth",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 10,
                ability_name: "searching",
                gain_msg: "perceptive",
                lose_msg: "less perceptive",
            },
        ],
        "Valkyrie" => vec![
            InnateAbility {
                level: 1,
                ability_name: "cold_resistance",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 1,
                ability_name: "stealth",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 7,
                ability_name: "fast",
                gain_msg: "quick",
                lose_msg: "slow",
            },
        ],
        "Wizard" => vec![
            InnateAbility {
                level: 15,
                ability_name: "warning",
                gain_msg: "sensitive",
                lose_msg: "",
            },
            InnateAbility {
                level: 17,
                ability_name: "teleport_control",
                gain_msg: "controlled",
                lose_msg: "uncontrolled",
            },
        ],
        _ => vec![],
    }
}

/// [v2.22.0 R34-2] 종족별 내재 능력 테이블 반환
pub fn get_race_abilities(race_name: &str) -> Vec<InnateAbility> {
    match race_name {
        "Dwarf" => vec![InnateAbility {
            level: 1,
            ability_name: "infravision",
            gain_msg: "",
            lose_msg: "",
        }],
        "Elf" => vec![
            InnateAbility {
                level: 1,
                ability_name: "infravision",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 4,
                ability_name: "sleep_resistance",
                gain_msg: "awake",
                lose_msg: "tired",
            },
        ],
        "Gnome" => vec![InnateAbility {
            level: 1,
            ability_name: "infravision",
            gain_msg: "",
            lose_msg: "",
        }],
        "Orc" => vec![
            InnateAbility {
                level: 1,
                ability_name: "infravision",
                gain_msg: "",
                lose_msg: "",
            },
            InnateAbility {
                level: 1,
                ability_name: "poison_resistance",
                gain_msg: "",
                lose_msg: "",
            },
        ],
        "Human" => vec![],
        _ => vec![],
    }
}

/// [v2.22.0 R34-2] 레벨업/다운 시 획득/상실 능력 조회
/// (원본: attrib.c:908 adjabil 핵심 로직)
pub fn calc_adjabil(
    old_level: i32,
    new_level: i32,
    role_name: &str,
    race_name: &str,
) -> (Vec<&'static str>, Vec<&'static str>) {
    let mut gained = Vec::new();
    let mut lost = Vec::new();

    let check = |abilities: &[InnateAbility]| -> (Vec<&'static str>, Vec<&'static str>) {
        let mut g = Vec::new();
        let mut l = Vec::new();
        for abil in abilities {
            if old_level < abil.level && new_level >= abil.level {
                // 레벨업 → 능력 획득
                g.push(abil.ability_name);
            } else if old_level >= abil.level && new_level < abil.level {
                // 레벨다운 → 능력 상실
                l.push(abil.ability_name);
            }
        }
        (g, l)
    };

    let role_abils = get_role_abilities(role_name);
    let (rg, rl) = check(&role_abils);
    gained.extend(rg);
    lost.extend(rl);

    let race_abils = get_race_abilities(race_name);
    let (rg2, rl2) = check(&race_abils);
    gained.extend(rg2);
    lost.extend(rl2);

    (gained, lost)
}

// =============================================================================
// [9] 독 메시지 테이블 (원본: poiseff[])
// =============================================================================

/// [v2.22.0 R34-2] 독 효과 메시지 (원본: poiseff[] 테이블)
pub fn poison_effect_message(attr_index: usize) -> (&'static str, &'static str) {
    match attr_index {
        A_STR => ("You feel", "weaker"),
        A_INT => ("Your", "brain is on fire"),
        A_WIS => ("Your", "judgement is impaired"),
        A_DEX => ("Your", "muscles won't obey you"),
        A_CON => ("You feel", "very sick"),
        A_CHA => ("You", "break out in hives"),
        _ => ("You feel", "strange"),
    }
}

// =============================================================================
// [10] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_luck_clamp() {
        // 정상 범위 내
        assert_eq!(calc_change_luck(0, 3), 3);
        assert_eq!(calc_change_luck(0, -3), -3);
        // 최대 범위 클램핑
        assert_eq!(calc_change_luck(8, 5), LUCKMAX);
        assert_eq!(calc_change_luck(-8, -5), LUCKMIN);
        // 0 → 음수 변경 시 LUCKMIN 미만 방지
        assert_eq!(calc_change_luck(-9, -2), LUCKMIN);
    }

    #[test]
    fn test_stone_luck() {
        // 축복된 행운석 = +1
        let items = vec![(true, false, 1)];
        assert_eq!(calc_stone_luck(&items, false), 1);

        // 저주된 행운석 = -1
        let items = vec![(false, true, 1)];
        assert_eq!(calc_stone_luck(&items, false), -1);

        // 보통 행운석: parameter=false면 0, parameter=true면 +1
        let items = vec![(false, false, 1)];
        assert_eq!(calc_stone_luck(&items, false), 0);
        assert_eq!(calc_stone_luck(&items, true), 1);
    }

    #[test]
    fn test_moreluck() {
        assert_eq!(calc_moreluck(1, true), LUCKADD);
        assert_eq!(calc_moreluck(-1, true), -LUCKADD);
        assert_eq!(calc_moreluck(0, false), 0);
        assert_eq!(calc_moreluck(0, true), LUCKADD);
    }

    #[test]
    fn test_poison_resistance() {
        let mut rng = NetHackRng::new(42);
        let result = calc_poison_damage(A_STR, 10, false, true, &mut rng);
        assert_eq!(result, PoisonResult::Resisted);
    }

    #[test]
    fn test_poison_damage_types() {
        let mut rng = NetHackRng::new(12345);
        // 비치명적 독 (fatal=0): 항상 능력치 감소
        let result = calc_poison_damage(A_STR, 0, false, false, &mut rng);
        match result {
            PoisonResult::StatLoss { attr_index, amount } => {
                assert_eq!(attr_index, A_STR);
                assert_eq!(amount, 1); // thrown_weapon=false, fatal=0 → 1
            }
            _ => panic!("비치명적 독은 StatLoss여야 함"),
        }
    }

    #[test]
    fn test_init_attr() {
        let mut rng = NetHackRng::new(42);
        let base = [10, 10, 10, 10, 10, 10];
        let dist = [20, 10, 15, 20, 25, 10]; // 합계 100
        let max = [18, 18, 18, 18, 18, 18];

        let (abase, amax) = calc_init_attr(75, &base, &dist, &max, &mut rng);

        // 총합이 75여야 함
        let total: i32 = abase.iter().sum();
        assert_eq!(total, 75);

        // 모든 능력치가 범위 내
        for i in 0..A_MAX {
            assert!(abase[i] >= ATTRMIN_DEFAULT);
            assert!(abase[i] <= max[i]);
            assert_eq!(abase[i], amax[i]); // 초기화 시 base == max
        }
    }

    #[test]
    fn test_redist_attr() {
        let mut rng = NetHackRng::new(42);
        let mut snap = AttributeSnapshot::default();
        snap.abase = [16, 14, 12, 15, 14, 10];
        snap.amax = [16, 14, 12, 15, 14, 10];

        let new_snap = calc_redist_attr(&snap, &mut rng);

        // 지능/지혜는 변경 안 됨
        assert_eq!(new_snap.abase[A_INT], snap.abase[A_INT]);
        assert_eq!(new_snap.amax[A_INT], snap.amax[A_INT]);
        assert_eq!(new_snap.abase[A_WIS], snap.abase[A_WIS]);
        assert_eq!(new_snap.amax[A_WIS], snap.amax[A_WIS]);
    }

    #[test]
    fn test_adjattrib_increase() {
        let mut rng = NetHackRng::new(42);
        let snap = AttributeSnapshot::default();

        let (result, new_snap) = calc_adjattrib(&snap, A_STR, 2, false, false, &mut rng);
        match result {
            AdjAttribResult::Changed {
                old_value,
                new_value,
                ..
            } => {
                assert_eq!(old_value, 10);
                assert_eq!(new_value, 12);
            }
            _ => panic!("능력치 증가가 반영되어야 함"),
        }
        assert_eq!(new_snap.abase[A_STR], 12);
    }

    #[test]
    fn test_adjattrib_fixed_abil() {
        let mut rng = NetHackRng::new(42);
        let snap = AttributeSnapshot::default();

        let (result, _) = calc_adjattrib(&snap, A_STR, 2, true, false, &mut rng);
        assert!(matches!(result, AdjAttribResult::NoChange { .. }));
    }

    #[test]
    fn test_adjattrib_dunce_cap() {
        let mut rng = NetHackRng::new(42);
        let snap = AttributeSnapshot::default();

        let (result, _) = calc_adjattrib(&snap, A_INT, 2, false, true, &mut rng);
        assert!(matches!(result, AdjAttribResult::NoChange { .. }));
    }

    #[test]
    fn test_exercise_int_cha_blocked() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(calc_exercise(A_INT, true, 0, 10, false, &mut rng), 0);
        assert_eq!(calc_exercise(A_CHA, true, 0, 10, false, &mut rng), 0);
    }

    #[test]
    fn test_exercise_polymorphed() {
        let mut rng = NetHackRng::new(42);
        // 폴리모프 중 STR 훈련 불가
        assert_eq!(calc_exercise(A_STR, true, 0, 10, true, &mut rng), 0);
        // 폴리모프 중 WIS 훈련은 가능
        let result = calc_exercise(A_WIS, true, 0, 5, true, &mut rng);
        // 결과는 RNG에 따라 다를 수 있으나, 차단되지 않으면 0 또는 1
        assert!(result >= 0 && result <= 1);
    }

    #[test]
    fn test_exerper_hunger() {
        let requests = calc_exerper(
            10,
            HungerState::Satiated,
            EncumbranceLevel::Unencumbered,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
        // 포만 상태: DEX 감소
        assert!(requests
            .iter()
            .any(|r| r.attr_index == A_DEX && !r.increase));
    }

    #[test]
    fn test_exerper_encumbered() {
        let requests = calc_exerper(
            10,
            HungerState::NotHungry,
            EncumbranceLevel::Strained,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
        // 과중: STR 증가 + DEX 감소
        assert!(requests.iter().any(|r| r.attr_index == A_STR && r.increase));
        assert!(requests
            .iter()
            .any(|r| r.attr_index == A_DEX && !r.increase));
    }

    #[test]
    fn test_role_abilities() {
        let monk = get_role_abilities("Monk");
        assert_eq!(monk.len(), 10);
        assert_eq!(monk[0].ability_name, "fast");
        assert_eq!(monk[9].ability_name, "teleport_control");
        assert_eq!(monk[9].level, 17);
    }

    #[test]
    fn test_race_abilities() {
        let elf = get_race_abilities("Elf");
        assert_eq!(elf.len(), 2);
        assert_eq!(elf[1].ability_name, "sleep_resistance");
        assert_eq!(elf[1].level, 4);
    }

    #[test]
    fn test_adjabil_levelup() {
        let (gained, lost) = calc_adjabil(6, 7, "Barbarian", "Human");
        assert!(gained.contains(&"fast"));
        assert!(lost.is_empty());
    }

    #[test]
    fn test_adjabil_leveldown() {
        let (gained, lost) = calc_adjabil(7, 6, "Barbarian", "Human");
        assert!(gained.is_empty());
        assert!(lost.contains(&"fast"));
    }

    #[test]
    fn test_poison_messages() {
        let (prefix, msg) = poison_effect_message(A_STR);
        assert_eq!(prefix, "You feel");
        assert_eq!(msg, "weaker");

        let (prefix, msg) = poison_effect_message(A_INT);
        assert_eq!(prefix, "Your");
        assert_eq!(msg, "brain is on fire");
    }

    #[test]
    fn test_exerchk_no_exercise() {
        let mut rng = NetHackRng::new(42);
        let result = calc_exerchk_result(A_STR, 0, 10, 3, 18, false, &mut rng);
        assert!(result.is_none());
    }
}

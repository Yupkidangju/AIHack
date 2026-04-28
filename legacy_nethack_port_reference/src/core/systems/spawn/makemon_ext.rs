// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.21.0 R34-1] 몬스터 생성 확장 모듈 (makemon_ext.rs)
// 원본: NetHack 3.6.7 makemon.c (2,319줄)
// 몬스터 생성/복제/성장/정렬/종 관리 순수 함수
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 원소 레벨 판정 (원본: makemon.c:33-75 is_home_elemental, wrong_elem_type)
// =============================================================================

/// [v2.21.0 R34-1] 원소 레벨 타입 (원본: dungeon.h Is_*level 매크로)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementalLevel {
    /// 일반 레벨 (원소 레벨 아님)
    Normal,
    /// 공기 레벨
    Air,
    /// 불 레벨
    Fire,
    /// 지구 레벨
    Earth,
    /// 물 레벨
    Water,
}

/// [v2.21.0 R34-1] 원소 타입 (원본: monst.c PM_*_ELEMENTAL)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementalType {
    Air,
    Fire,
    Earth,
    Water,
}

/// [v2.21.0 R34-1] 몬스터가 현재 원소 레벨의 "홈" 원소인지 판정
/// 원본: makemon.c:33-49 is_home_elemental()
/// 원소 몬스터가 해당 원소 레벨에 있을 때만 true
pub fn is_home_elemental(
    monster_class: char,
    elemental_type: Option<ElementalType>,
    current_level: ElementalLevel,
) -> bool {
    // S_ELEMENTAL = 'E' (원소 몬스터 클래스)
    if monster_class != 'E' {
        return false;
    }
    match (elemental_type, current_level) {
        (Some(ElementalType::Air), ElementalLevel::Air) => true,
        (Some(ElementalType::Fire), ElementalLevel::Fire) => true,
        (Some(ElementalType::Earth), ElementalLevel::Earth) => true,
        (Some(ElementalType::Water), ElementalLevel::Water) => true,
        _ => false,
    }
}

/// [v2.21.0 R34-1] 몬스터 능력 플래그 (wrong_elem_type 판정용)
#[derive(Debug, Clone, Copy, Default)]
pub struct MonsterAbilities {
    /// 수영 가능 (원본: is_swimmer)
    pub can_swim: bool,
    /// 화염 저항 (원본: pm_resistance MR_FIRE)
    pub fire_resistant: bool,
    /// 비행 가능 (원본: is_flyer)
    pub can_fly: bool,
    /// 부유 가능 (원본: is_floater)
    pub can_float: bool,
    /// 무정형 (원본: amorphous)
    pub amorphous: bool,
    /// 비실체 (원본: noncorporeal)
    pub noncorporeal: bool,
    /// 회오리 형태 (원본: is_whirly)
    pub is_whirly: bool,
    /// 트래퍼 계열 (원본: S_TRAPPER)
    pub is_trapper: bool,
}

/// [v2.21.0 R34-1] 해당 몬스터가 현재 원소 레벨에서 존재할 수 없는지 판정
/// 원본: makemon.c:51-75 wrong_elem_type()
/// true = 이 레벨에서 생성 불가
pub fn wrong_elem_type(
    monster_class: char,
    elemental_type: Option<ElementalType>,
    current_level: ElementalLevel,
    abilities: &MonsterAbilities,
) -> bool {
    // 원소 몬스터는 자기 레벨이 아니면 생성 불가
    if monster_class == 'E' {
        return !is_home_elemental(monster_class, elemental_type, current_level);
    }

    match current_level {
        ElementalLevel::Earth => {
            // 지구 레벨: 제한 없음 (원본: no restrictions?)
            false
        }
        ElementalLevel::Water => {
            // 물 레벨: 수영 가능한 몬스터만
            !abilities.can_swim
        }
        ElementalLevel::Fire => {
            // 불 레벨: 화염 저항 필수
            !abilities.fire_resistant
        }
        ElementalLevel::Air => {
            // 공기 레벨: 비행/부유/무정형/비실체/회오리만
            // 단, 트래퍼 계열은 비행해도 불가 (원본: ptr->mlet != S_TRAPPER)
            let can_exist = (abilities.can_fly && !abilities.is_trapper)
                || abilities.can_float
                || abilities.amorphous
                || abilities.noncorporeal
                || abilities.is_whirly;
            !can_exist
        }
        ElementalLevel::Normal => false,
    }
}

// =============================================================================
// [2] HP 계산 (원본: makemon.c:954-978, 1966-1996 monhp_per_lvl, golemhp)
// =============================================================================

/// [v2.21.0 R34-1] 골렘 타입별 사전 정의 HP (원본: makemon.c:1966-1996 golemhp)
/// 원본의 switch문을 Rust match로 1:1 재현
pub fn golemhp(golem_name: &str) -> i32 {
    match golem_name {
        "straw golem" => 20,
        "paper golem" => 20,
        "rope golem" => 30,
        "gold golem" => 40,
        "leather golem" => 40,
        "wood golem" => 50,
        "flesh golem" => 40,
        "clay golem" => 50,
        "stone golem" => 60,
        "glass golem" => 60,
        "iron golem" => 80,
        _ => 0,
    }
}

/// [v2.21.0 R34-1] 레벨 드레인/스톰브링어 HP 변화량 (원본: makemon.c:954-978)
/// 원본: monhp_per_lvl() — 몬스터 레벨당 HP 변화를 계산하는 순수 함수
pub fn monhp_per_lvl(
    is_golem: bool,
    golem_name: &str,
    monster_level: i32,
    monster_class: char,
    is_adult_dragon: bool,
    rng: &mut NetHackRng,
) -> i32 {
    if is_golem {
        // 골렘: 총 HP를 몬스터 레벨로 나눈 값
        let total = golemhp(golem_name);
        if monster_level > 0 {
            total / monster_level
        } else {
            total
        }
    } else if monster_level > 49 {
        // 특수 고레벨 몬스터: d4+4 (5..8)
        4 + rng.rn2(4) + 1
    } else if monster_class == 'D' && is_adult_dragon {
        // 성체 드래곤: 4..8
        4 + rng.rn2(5)
    } else if monster_level == 0 {
        // 레벨 0 몬스터: d4
        rng.rn2(4) + 1
    } else {
        // 기본: d8
        rng.rn2(8) + 1
    }
}

// =============================================================================
// [3] 종 관리 (원본: makemon.c:912-952, 1417-1428 propagate, mbirth_limit)
// =============================================================================

/// [v2.21.0 R34-1] 종 출생 제한 (원본: makemon.c:1417-1428 mbirth_limit)
/// MAXMONNO = 120 (원본: include/monst.h)
pub const MAXMONNO: u8 = 120;

/// 특수 몬스터별 생성 제한 수.
/// 원본: Nazgul=9, Erinyes=3, 기타=MAXMONNO
pub fn mbirth_limit(monster_name: &str) -> u8 {
    match monster_name {
        "Nazgul" => 9,
        "Erinyes" => 3,
        _ => MAXMONNO,
    }
}

/// [v2.21.0 R34-1] 종 번식 결과 (원본: makemon.c:912-952 propagate)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropagateResult {
    /// 번식 성공 여부
    pub success: bool,
    /// 출생 카운트 증가 후 값
    pub new_born_count: u8,
    /// 절멸 여부 (출생한계 도달)
    pub extinct: bool,
}

/// [v2.21.0 R34-1] 종 번식 판정 (원본: makemon.c:912-952 propagate)
/// 특정 종의 출생 수가 한계에 도달했는지 체크하고, 한계 초과 시 절멸 처리
///
/// # 매개변수
/// - `born_count`: 현재까지 출생한 수 (mvitals[].born)
/// - `is_gone`: 이미 제노사이드/절멸 상태인지 (G_GONE)
/// - `is_unique`: 유니크 몬스터인지 (G_UNIQ)
/// - `is_high_priest`: 대신관인지 (PM_HIGH_PRIEST, 유니크 예외)
/// - `is_nogen`: G_NOGEN 플래그
/// - `monster_name`: 몬스터 이름 (mbirth_limit 조회용)
/// - `tally`: 출생 카운트를 증가시킬지
/// - `ghostly`: 유령 복원인지
pub fn propagate(
    born_count: u8,
    is_gone: bool,
    is_unique: bool,
    is_high_priest: bool,
    is_nogen: bool,
    monster_name: &str,
    tally: bool,
    ghostly: bool,
) -> PropagateResult {
    let lim = mbirth_limit(monster_name);
    // 번식 성공: 출생 수가 한계 미만이고 절멸/제노사이드가 아닐 때
    let success = (born_count as i32) < (lim as i32) && !is_gone;

    // 유니크 몬스터는 생성 즉시 절멸 처리 (대신관 제외)
    let mut extinct = is_gone;
    if is_unique && !is_high_priest {
        extinct = true;
    }

    // 출생 카운트 증가 (tally이고, 255 미만이고, 유령복원이 아니거나 성공인 경우)
    let mut new_born = born_count;
    if new_born < 255 && tally && (!ghostly || success) {
        new_born = new_born.saturating_add(1);
    }

    // 출생 한계 도달 시 절멸 처리 (G_NOGEN이 아닌 경우)
    if (new_born as i32) >= (lim as i32) && !is_nogen && !extinct {
        extinct = true;
    }

    PropagateResult {
        success,
        new_born_count: new_born,
        extinct,
    }
}

// =============================================================================
// [4] 레벨 보정 (원본: makemon.c:1755-1788 adj_lev)
// =============================================================================

/// [v2.21.0 R34-1] 몬스터 난이도 보정 (원본: makemon.c:1755-1788 adj_lev)
/// 던전 깊이와 플레이어 레벨에 따라 몬스터 레벨을 보정
///
/// 원본 로직:
/// - 유니크(G_UNIQ): 원래 레벨 유지
/// - 일반: base_level ± level_difficulty()/5 범위에서 랜덤 보정
/// - level_difficulty() = depth + player_level - 1
pub fn adj_lev(
    base_level: i32,
    is_unique: bool,
    dungeon_depth: i32,
    player_level: i32,
    rng: &mut NetHackRng,
) -> i32 {
    if is_unique {
        // 유니크 몬스터: 원래 레벨 유지 (원본: return ptr->mlevel)
        return base_level;
    }

    // level_difficulty = depth + player_level - 1 (원본: cmd.c level_difficulty)
    let zlevel = dungeon_depth + player_level - 1;
    let adjust = zlevel.max(0) / 5;

    if adjust > 0 {
        // 원본: ptr->mlevel + rn2(adj)
        // 범위: [base_level, base_level + adjust - 1]
        let bonus = if adjust > 1 { rng.rn2(adjust) } else { 0 };
        let result = base_level + bonus;
        // 상한 경계: 원본에서 49를 초과하지 않도록 (50+는 특수 HP)
        result.min(49)
    } else {
        base_level
    }
}

// =============================================================================
// [5] 정렬 편향 (원본: makemon.c:1479-1512 align_shift)
// =============================================================================

/// [v2.21.0 R34-1] 던전 정렬 타입 (원본: align.h AM_*)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonAlignment {
    None,
    Lawful,
    Neutral,
    Chaotic,
}

/// ALIGNWEIGHT 상수 (원본: include/monst.h)
const ALIGNWEIGHT: i32 = 4;

/// [v2.21.0 R34-1] 정렬 편향 계산 (원본: makemon.c:1479-1512 align_shift)
/// 던전 정렬과 몬스터 정렬을 비교하여 0~5 범위의 확률 편향 반환
///
/// 질서 던전: 선한 몬스터 +, 악한 몬스터 -
/// 혼돈 던전: 악한 몬스터 +, 선한 몬스터 -
/// 중립 던전: 중립에 가까울수록 +
pub fn align_shift(monster_alignment: i32, dungeon_align: DungeonAlignment) -> i32 {
    match dungeon_align {
        DungeonAlignment::None => 0,
        DungeonAlignment::Lawful => {
            // (maligntyp + 20) / (2 * ALIGNWEIGHT)
            (monster_alignment + 20) / (2 * ALIGNWEIGHT)
        }
        DungeonAlignment::Neutral => {
            // (20 - abs(maligntyp)) / ALIGNWEIGHT
            (20 - monster_alignment.abs()) / ALIGNWEIGHT
        }
        DungeonAlignment::Chaotic => {
            // (-(maligntyp - 20)) / (2 * ALIGNWEIGHT)
            (-(monster_alignment - 20)) / (2 * ALIGNWEIGHT)
        }
    }
}

// =============================================================================
// [6] 평화 판정 (원본: makemon.c:1998-2042 peace_minded)
// =============================================================================

/// [v2.21.0 R34-1] 몬스터 평화 성향 판정 컨텍스트
#[derive(Debug, Clone)]
pub struct PeaceMindedContext {
    /// 몬스터 정렬 타입 (maligntyp)
    pub monster_alignment: i32,
    /// 플레이어 정렬 타입 (u.ualign.type)
    pub player_alignment: i32,
    /// 항상 적대 (always_hostile 매크로)
    pub always_hostile: bool,
    /// 항상 평화 (always_peaceful 매크로)
    pub always_peaceful: bool,
    /// 인간 종족 선택 시 발동 (Race_if(PM_HUMAN) 확인)
    pub player_is_human: bool,
    /// 엘프 종족 선택 시 발동
    pub player_is_elf: bool,
}

/// [v2.21.0 R34-1] 몬스터가 평화적으로 생성될지 판정 (원본: makemon.c:1998-2042)
/// true = 평화적으로 생성, false = 적대적으로 생성
pub fn peace_minded(ctx: &PeaceMindedContext, rng: &mut NetHackRng) -> bool {
    // 항상 적대
    if ctx.always_hostile {
        return false;
    }
    // 항상 평화
    if ctx.always_peaceful {
        return true;
    }

    // 정렬 기반 판정 (원본: sgn(ptr->maligntyp) == sgn(u.ualign.type))
    let m_sign = ctx.monster_alignment.signum();
    let p_sign = ctx.player_alignment.signum();

    // 같은 정렬이면 평화적, 다른 정렬이면 적대적
    // 중립(0)인 경우: 1/2 확률
    if m_sign == 0 && p_sign == 0 {
        // 양쪽 중립: 평화적
        return true;
    }
    if m_sign == 0 || p_sign == 0 {
        // 한쪽만 중립: 50% 확률
        return rng.rn2(2) == 0;
    }
    // 같은 정렬이면 평화, 다르면 적대
    m_sign == p_sign
}

// =============================================================================
// [7] 정렬 악성 설정 (원본: makemon.c:2044-2101 set_malign)
// =============================================================================

/// [v2.21.0 R34-1] 몬스터 악성치(malign) 계산 컨텍스트
#[derive(Debug, Clone)]
pub struct MalignContext {
    /// 몬스터 정렬 (maligntyp)
    pub monster_alignment: i32,
    /// 평화적인 몬스터인지
    pub is_peaceful: bool,
    /// 항상 평화적인 몬스터인지
    pub always_peaceful: bool,
    /// 몬스터 레벨
    pub monster_level: i32,
}

/// [v2.21.0 R34-1] 몬스터 악성치 계산 (원본: makemon.c:2044-2101 set_malign)
/// 양수 = 죽이면 좋음, 음수 = 죽이면 나쁨
///
/// 규칙:
/// - 평화 몬스터 죽이기 = 나쁨 (음수)
/// - 항상-평화 몬스터 죽이기 = 더 나쁨
/// - 적대 몬스터 죽이기 = 좋거나 0
pub fn calc_malign(ctx: &MalignContext) -> i32 {
    if ctx.is_peaceful {
        // 평화 몬스터 죽이기: -3 * max(5, monster_alignment)
        // 항상-평화인 경우 추가 페널티
        let base = ctx.monster_alignment.abs().max(5);
        let malign = -(3 * base);
        if ctx.always_peaceful {
            // 추가 페널티: -(2 * monster_level + 10)
            malign - (2 * ctx.monster_level + 10)
        } else {
            malign
        }
    } else {
        // 적대 몬스터 죽이기: max(0, monster_level)
        // 선한 몬스터 적대적 → 죽여도 보상 없음
        if ctx.monster_alignment > 0 {
            0
        } else {
            // 악한 몬스터 → 레벨에 비례한 보상
            ctx.monster_level.max(0)
        }
    }
}

// =============================================================================
// [8] 몬스터 성장 (원본: makemon.c:1789-1911 grow_up)
// =============================================================================

/// [v2.21.0 R34-1] 몬스터 성장 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrowUpResult {
    /// HP만 증가 (레벨 미달)
    HpGain { hp_increase: i32, new_level: i32 },
    /// 형태 변화 (baby → adult 등)
    Evolved {
        new_form: String,
        new_level: i32,
        hp_increase: i32,
    },
    /// 진화 실패로 사망 (폴리모프 관련)
    DiedFromEvolution,
    /// 이미 최대 레벨
    NoChange,
}

/// [v2.21.0 R34-1] 몬스터 성장 계산 (원본: makemon.c:1789-1911 grow_up)
///
/// 적 처치 또는 포션 복용으로 경험치를 획득한 몬스터의 성장을 처리
/// - victim_level: 처치한 적의 레벨 (None이면 포션/시체 복용)
/// - current_level: 현재 몬스터 레벨
/// - current_hp_max: 현재 최대 HP
/// - evolved_form: 성장 시 변할 형태 이름 (little_to_big 결과, 없으면 None)
/// - evolved_form_level: 성장 형태의 기본 레벨
pub fn grow_up_calc(
    victim_level: Option<i32>,
    current_level: i32,
    current_hp_max: i32,
    evolved_form: Option<&str>,
    evolved_form_level: i32,
    is_player_monster: bool,
    rng: &mut NetHackRng,
) -> GrowUpResult {
    // 레벨 제한 결정 (원본: lines 1847-1857)
    let lev_limit = if is_player_monster {
        30 // 플레이어 몬스터: 30 제한
    } else if current_level < 5 {
        5 // 최소 5
    } else if current_level > 49 {
        if evolved_form_level > 49 {
            50
        } else {
            49
        }
    } else {
        49 // 일반 상한
    };

    // HP 증가량 계산
    let (hp_increase, can_level_up) = match victim_level {
        Some(vlev) => {
            // 적 처치: HP 한계 = (1 + current_level) * 8
            let hp_threshold = (1 + current_level) * 8;
            let max_increase = (rng.rn2(vlev + 1) + 1).max(1);
            let capped = if current_hp_max + max_increase > hp_threshold + 1 {
                ((hp_threshold + 1) - current_hp_max).max(0)
            } else {
                max_increase
            };
            let cur_increase = if capped > 1 { rng.rn2(capped) } else { 0 };
            // HP 한계 초과 시 레벨업
            (cur_increase, current_hp_max + cur_increase >= hp_threshold)
        }
        None => {
            // 포션/시체: 무조건 레벨업 (원본: lines 1837-1846)
            if current_level >= lev_limit {
                return GrowUpResult::NoChange;
            }
            let hp_gain = rng.rn2(8) + 1; // d8
            (hp_gain, true)
        }
    };

    if !can_level_up || current_level + 1 > lev_limit {
        return GrowUpResult::HpGain {
            hp_increase,
            new_level: current_level,
        };
    }

    let new_level = current_level + 1;

    // 성장 형태가 있고 새 레벨이 해당 형태 기본 레벨 이상이면 진화
    if let Some(form) = evolved_form {
        if new_level >= evolved_form_level && form != "" {
            return GrowUpResult::Evolved {
                new_form: form.to_string(),
                new_level,
                hp_increase,
            };
        }
    }

    GrowUpResult::HpGain {
        hp_increase,
        new_level,
    }
}

// =============================================================================
// [9] 클래스별 몬스터 생성 (원본: makemon.c:1638-1722 mkclass)
// =============================================================================

/// [v2.21.0 R34-1] 클래스별 몬스터 선택 입력 컨텍스트
#[derive(Debug, Clone)]
pub struct MonsterCandidate {
    /// 몬스터 이름
    pub name: String,
    /// 몬스터 클래스 문자
    pub class_char: char,
    /// 난이도 (difficulty)
    pub difficulty: i32,
    /// 정렬 (maligntyp)
    pub alignment: i32,
    /// 생성 플래그 (geno)
    pub geno_flags: u32,
    /// 제거/절멸 상태 (G_GONE)
    pub is_gone: bool,
}

/// G_NOGEN, G_UNIQ 비트 마스크 (원본: monst.h)
pub const G_NOGEN: u32 = 0x08;
pub const G_UNIQ: u32 = 0x10;

/// [v2.21.0 R34-1] 클래스별 몬스터 선택 (원본: makemon.c:1638-1722 mkclass)
/// 주어진 몬스터 클래스에서 적절한 난이도의 몬스터를 랜덤 선택
///
/// # 매개변수
/// - `candidates`: 해당 클래스의 모든 몬스터 리스트 (난이도 오름차순)
/// - `max_level`: 최대 허용 난이도 (level_difficulty 기반)
/// - `alignment_filter`: 정렬 제한 (None이면 무제한)
pub fn mkclass_select(
    candidates: &[MonsterCandidate],
    max_level: i32,
    alignment_filter: Option<i32>,
    rng: &mut NetHackRng,
) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }

    // 후보 필터링 및 가중치 부여 (원본: lines 1685-1711)
    let mask = G_NOGEN | G_UNIQ;
    let mut weights: Vec<(usize, i32)> = Vec::new();
    let mut total_weight = 0i32;

    for (idx, c) in candidates.iter().enumerate() {
        // 정렬 필터 (원본: atyp != A_NONE)
        if let Some(align) = alignment_filter {
            if c.alignment.signum() != align.signum() {
                continue;
            }
        }
        // G_GONE 체크
        if c.is_gone {
            continue;
        }
        // G_NOGEN | G_UNIQ 체크
        if c.geno_flags & mask != 0 {
            continue;
        }
        // 난이도 제한: toostrong 체크 (첫 후보는 예외)
        if c.difficulty > max_level && !weights.is_empty() {
            continue;
        }

        // 가중치: 난이도가 낮을수록 높은 가중치 (원본: k = zlevel - adj_lev, nums[last] = k+1)
        let k = (max_level - c.difficulty).max(0);
        let w = (k + 1).min(127);
        if w > 0 {
            weights.push((idx, w));
            total_weight += w;
        }
    }

    if total_weight <= 0 {
        return None;
    }

    // 가중 랜덤 선택 (원본: for(num=rnd(num); first<last; ...) )
    let mut roll = rng.rn2(total_weight) + 1;
    for (idx, w) in &weights {
        roll -= w;
        if roll <= 0 {
            return Some(candidates[*idx].name.clone());
        }
    }

    // 폴백: 마지막 후보
    weights.last().map(|(idx, _)| candidates[*idx].name.clone())
}

// =============================================================================
// [10] 몬스터 복제 (원본: makemon.c:802-910 clone_mon)
// =============================================================================

/// [v2.21.0 R34-1] 복제 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloneMonResult {
    /// 복제 성공 여부
    pub success: bool,
    /// 원본 몬스터의 새 HP (원본 - 분배분)
    pub original_new_hp: i32,
    /// 복제체의 HP
    pub clone_hp: i32,
    /// 복제체의 최대 HP
    pub clone_hp_max: i32,
    /// 복제체의 평화 여부
    pub clone_peaceful: bool,
}

/// [v2.21.0 R34-1] 몬스터 복제 HP 분배 (원본: makemon.c:853-858 clone_mon)
/// 원본 HP를 절반으로 분배 (홀수면 원본이 1점 더 가져감)
///
/// # 매개변수
/// - `original_hp`: 원본 현재 HP
/// - `original_hp_max`: 원본 최대 HP
/// - `is_extinct`: 해당 종이 절멸 상태인지
/// - `is_peaceful`: 원본 평화적 여부
/// - `player_luck`: 플레이어 운 (평화 상태 결정에 사용)
pub fn clone_mon_calc(
    original_hp: i32,
    original_hp_max: i32,
    is_extinct: bool,
    is_peaceful: bool,
    player_luck: i32,
    rng: &mut NetHackRng,
) -> CloneMonResult {
    // 복제 실패: HP 1 이하 또는 절멸 상태
    if original_hp <= 1 || is_extinct {
        return CloneMonResult {
            success: false,
            original_new_hp: original_hp,
            clone_hp: 0,
            clone_hp_max: 0,
            clone_peaceful: false,
        };
    }

    // HP 분배: 복제체 = original / 2, 원본은 나머지 (원본이 홀수일 때 1점 더)
    let clone_hp = original_hp / 2;
    let original_new_hp = original_hp - clone_hp;

    // 평화 상태 결정 (원본: lines 880-884)
    let clone_peaceful = if is_peaceful {
        // max(2 + luck, 2) 분의 1 확률로 적대 전환
        let threshold = (2 + player_luck).max(2);
        rng.rn2(threshold) != 0
    } else {
        false
    };

    CloneMonResult {
        success: true,
        original_new_hp,
        clone_hp,
        clone_hp_max: original_hp_max,
        clone_peaceful,
    }
}

// =============================================================================
// [11] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_home_elemental 테스트 ---

    #[test]
    fn test_home_elemental_fire_on_fire_level() {
        // 불 원소가 불 레벨에 있으면 true
        assert!(is_home_elemental(
            'E',
            Some(ElementalType::Fire),
            ElementalLevel::Fire
        ));
    }

    #[test]
    fn test_home_elemental_fire_on_water_level() {
        // 불 원소가 물 레벨에 있으면 false
        assert!(!is_home_elemental(
            'E',
            Some(ElementalType::Fire),
            ElementalLevel::Water
        ));
    }

    #[test]
    fn test_home_elemental_non_elemental() {
        // 원소가 아닌 몬스터는 항상 false
        assert!(!is_home_elemental('D', None, ElementalLevel::Fire));
    }

    // --- wrong_elem_type 테스트 ---

    #[test]
    fn test_wrong_elem_water_level_no_swim() {
        // 물 레벨에서 수영 불가 몬스터 → 생성 불가
        let abilities = MonsterAbilities::default();
        assert!(wrong_elem_type(
            'Z',
            None,
            ElementalLevel::Water,
            &abilities
        ));
    }

    #[test]
    fn test_wrong_elem_water_level_swimmer() {
        // 물 레벨에서 수영 가능 몬스터 → 생성 가능
        let abilities = MonsterAbilities {
            can_swim: true,
            ..Default::default()
        };
        assert!(!wrong_elem_type(
            'Z',
            None,
            ElementalLevel::Water,
            &abilities
        ));
    }

    #[test]
    fn test_wrong_elem_air_level_flyer() {
        // 공기 레벨에서 비행 가능(트래퍼 아닌) 몬스터 → 생성 가능
        let abilities = MonsterAbilities {
            can_fly: true,
            ..Default::default()
        };
        assert!(!wrong_elem_type('B', None, ElementalLevel::Air, &abilities));
    }

    #[test]
    fn test_wrong_elem_air_level_trapper_cannot() {
        // 공기 레벨에서 트래퍼(비행이지만) → 생성 불가
        let abilities = MonsterAbilities {
            can_fly: true,
            is_trapper: true,
            ..Default::default()
        };
        assert!(wrong_elem_type('t', None, ElementalLevel::Air, &abilities));
    }

    // --- golemhp 테스트 ---

    #[test]
    fn test_golemhp() {
        assert_eq!(golemhp("iron golem"), 80);
        assert_eq!(golemhp("flesh golem"), 40);
        assert_eq!(golemhp("straw golem"), 20);
        assert_eq!(golemhp("unknown"), 0);
    }

    // --- monhp_per_lvl 테스트 ---

    #[test]
    fn test_monhp_per_lvl_golem() {
        let mut rng = NetHackRng::new(42);
        // 철 골렘(HP 80), 레벨 16 → 80/16 = 5
        assert_eq!(
            monhp_per_lvl(true, "iron golem", 16, 'G', false, &mut rng),
            5
        );
    }

    #[test]
    fn test_monhp_per_lvl_level_zero() {
        let mut rng = NetHackRng::new(42);
        // 레벨 0 몬스터 → d4 (1~4)
        let hp = monhp_per_lvl(false, "", 0, 'r', false, &mut rng);
        assert!(hp >= 1 && hp <= 4);
    }

    // --- mbirth_limit 테스트 ---

    #[test]
    fn test_mbirth_limit() {
        assert_eq!(mbirth_limit("Nazgul"), 9);
        assert_eq!(mbirth_limit("Erinyes"), 3);
        assert_eq!(mbirth_limit("goblin"), MAXMONNO);
    }

    // --- propagate 테스트 ---

    #[test]
    fn test_propagate_success() {
        let result = propagate(0, false, false, false, false, "goblin", true, false);
        assert!(result.success);
        assert_eq!(result.new_born_count, 1);
        assert!(!result.extinct);
    }

    #[test]
    fn test_propagate_unique_extinct() {
        // 유니크 몬스터 → 생성 후 즉시 절멸
        let result = propagate(0, false, true, false, false, "Medusa", true, false);
        assert!(result.success);
        assert!(result.extinct);
    }

    #[test]
    fn test_propagate_high_priest_exception() {
        // 대신관은 유니크이지만 절멸되지 않음
        let result = propagate(0, false, true, true, false, "high priest", true, false);
        assert!(result.success);
        assert!(!result.extinct);
    }

    #[test]
    fn test_propagate_already_gone() {
        let result = propagate(100, true, false, false, false, "goblin", true, false);
        assert!(!result.success);
    }

    // --- adj_lev 테스트 ---

    #[test]
    fn test_adj_lev_unique() {
        let mut rng = NetHackRng::new(42);
        // 유니크: 원래 레벨 유지
        assert_eq!(adj_lev(30, true, 10, 10, &mut rng), 30);
    }

    #[test]
    fn test_adj_lev_normal() {
        let mut rng = NetHackRng::new(42);
        // 일반 몬스터 레벨 5, 깊이 15, 플레이어 10
        // zlevel=24, adjust=4
        let result = adj_lev(5, false, 15, 10, &mut rng);
        assert!(result >= 5 && result <= 8); // 5 + rn2(4)
    }

    // --- align_shift 테스트 ---

    #[test]
    fn test_align_shift_lawful_good_monster() {
        // 질서 던전 + 선한 몬스터(+20) → (20+20)/(2*4) = 5
        assert_eq!(align_shift(20, DungeonAlignment::Lawful), 5);
    }

    #[test]
    fn test_align_shift_chaotic_evil_monster() {
        // 혼돈 던전 + 악한 몬스터(-20) → (-(-20-20))/(2*4) = 5
        assert_eq!(align_shift(-20, DungeonAlignment::Chaotic), 5);
    }

    #[test]
    fn test_align_shift_neutral_neutral() {
        // 중립 던전 + 중립 몬스터(0) → (20-0)/4 = 5
        assert_eq!(align_shift(0, DungeonAlignment::Neutral), 5);
    }

    #[test]
    fn test_align_shift_none() {
        assert_eq!(align_shift(10, DungeonAlignment::None), 0);
    }

    // --- peace_minded 테스트 ---

    #[test]
    fn test_peace_minded_always_hostile() {
        let mut rng = NetHackRng::new(42);
        let ctx = PeaceMindedContext {
            monster_alignment: 0,
            player_alignment: 0,
            always_hostile: true,
            always_peaceful: false,
            player_is_human: false,
            player_is_elf: false,
        };
        assert!(!peace_minded(&ctx, &mut rng));
    }

    #[test]
    fn test_peace_minded_always_peaceful() {
        let mut rng = NetHackRng::new(42);
        let ctx = PeaceMindedContext {
            monster_alignment: -10,
            player_alignment: 10,
            always_hostile: false,
            always_peaceful: true,
            player_is_human: false,
            player_is_elf: false,
        };
        assert!(peace_minded(&ctx, &mut rng));
    }

    #[test]
    fn test_peace_minded_same_alignment() {
        let mut rng = NetHackRng::new(42);
        let ctx = PeaceMindedContext {
            monster_alignment: 5,
            player_alignment: 10,
            always_hostile: false,
            always_peaceful: false,
            player_is_human: false,
            player_is_elf: false,
        };
        // 양쪽 양수 정렬 → 평화
        assert!(peace_minded(&ctx, &mut rng));
    }

    // --- calc_malign 테스트 ---

    #[test]
    fn test_malign_peaceful_monster() {
        let ctx = MalignContext {
            monster_alignment: 10,
            is_peaceful: true,
            always_peaceful: false,
            monster_level: 5,
        };
        let malign = calc_malign(&ctx);
        assert!(malign < 0); // 평화 몬스터 죽이기 = 나쁨
    }

    #[test]
    fn test_malign_hostile_evil() {
        let ctx = MalignContext {
            monster_alignment: -10,
            is_peaceful: false,
            always_peaceful: false,
            monster_level: 8,
        };
        let malign = calc_malign(&ctx);
        assert!(malign > 0); // 적대적 악한 몬스터 죽이기 = 좋음
    }

    #[test]
    fn test_malign_hostile_good() {
        let ctx = MalignContext {
            monster_alignment: 10,
            is_peaceful: false,
            always_peaceful: false,
            monster_level: 5,
        };
        let malign = calc_malign(&ctx);
        assert_eq!(malign, 0); // 적대적 선한 몬스터 = 보상 없음
    }

    // --- grow_up_calc 테스트 ---

    #[test]
    fn test_grow_up_kill_hp_gain() {
        let mut rng = NetHackRng::new(42);
        let result = grow_up_calc(Some(5), 3, 20, None, 0, false, &mut rng);
        match result {
            GrowUpResult::HpGain { hp_increase, .. } => {
                assert!(hp_increase >= 0);
            }
            _ => {} // 레벨업도 가능
        }
    }

    #[test]
    fn test_grow_up_potion_no_change_at_max() {
        let mut rng = NetHackRng::new(42);
        // 이미 최대 레벨 (49)
        let result = grow_up_calc(None, 49, 100, None, 0, false, &mut rng);
        assert_eq!(result, GrowUpResult::NoChange);
    }

    // --- clone_mon_calc 테스트 ---

    #[test]
    fn test_clone_low_hp_fails() {
        let mut rng = NetHackRng::new(42);
        let result = clone_mon_calc(1, 10, false, false, 0, &mut rng);
        assert!(!result.success);
    }

    #[test]
    fn test_clone_success_hp_split() {
        let mut rng = NetHackRng::new(42);
        let result = clone_mon_calc(20, 25, false, false, 0, &mut rng);
        assert!(result.success);
        assert_eq!(result.clone_hp, 10); // 20/2
        assert_eq!(result.original_new_hp, 10); // 20-10
        assert_eq!(result.clone_hp_max, 25);
    }

    #[test]
    fn test_clone_extinct_fails() {
        let mut rng = NetHackRng::new(42);
        let result = clone_mon_calc(20, 25, true, false, 0, &mut rng);
        assert!(!result.success);
    }

    // --- mkclass_select 테스트 ---

    #[test]
    fn test_mkclass_select_empty() {
        let mut rng = NetHackRng::new(42);
        let result = mkclass_select(&[], 10, None, &mut rng);
        assert!(result.is_none());
    }

    #[test]
    fn test_mkclass_select_single() {
        let mut rng = NetHackRng::new(42);
        let candidates = vec![MonsterCandidate {
            name: "goblin".to_string(),
            class_char: 'o',
            difficulty: 1,
            alignment: -3,
            geno_flags: 0,
            is_gone: false,
        }];
        let result = mkclass_select(&candidates, 10, None, &mut rng);
        assert_eq!(result, Some("goblin".to_string()));
    }

    #[test]
    fn test_mkclass_select_filters_gone() {
        let mut rng = NetHackRng::new(42);
        let candidates = vec![
            MonsterCandidate {
                name: "extinct_orc".to_string(),
                class_char: 'o',
                difficulty: 1,
                alignment: -3,
                geno_flags: 0,
                is_gone: true, // 절멸됨
            },
            MonsterCandidate {
                name: "hobgoblin".to_string(),
                class_char: 'o',
                difficulty: 2,
                alignment: -3,
                geno_flags: 0,
                is_gone: false,
            },
        ];
        let result = mkclass_select(&candidates, 10, None, &mut rng);
        assert_eq!(result, Some("hobgoblin".to_string()));
    }

    // --- 통계적 테스트: 클래스 선택 분포 ---

    #[test]
    fn test_mkclass_select_distribution() {
        // 100번 반복하여 다양한 몬스터가 선택되는지 확인
        let candidates = vec![
            MonsterCandidate {
                name: "kobold".to_string(),
                class_char: 'k',
                difficulty: 1,
                alignment: -3,
                geno_flags: 0,
                is_gone: false,
            },
            MonsterCandidate {
                name: "large kobold".to_string(),
                class_char: 'k',
                difficulty: 3,
                alignment: -3,
                geno_flags: 0,
                is_gone: false,
            },
            MonsterCandidate {
                name: "kobold lord".to_string(),
                class_char: 'k',
                difficulty: 5,
                alignment: -3,
                geno_flags: 0,
                is_gone: false,
            },
        ];

        let mut seen_names = std::collections::HashSet::new();
        for seed in 0..100 {
            let mut rng = NetHackRng::new(seed);
            if let Some(name) = mkclass_select(&candidates, 10, None, &mut rng) {
                seen_names.insert(name);
            }
        }
        // 100번 중 최소 2종류 이상 나와야 함
        assert!(seen_names.len() >= 2, "분포 다양성 부족: {:?}", seen_names);
    }
}

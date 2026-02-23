// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.22.0 R10-3] 몬스터 생성 확장 엔진 (spawn_ext.rs)
//
// 원본 참조: NetHack 3.6.7 makemon.c (2,156줄) 핵심 알고리즘 이식
//
// 구현 내용:
//   1. enexto() — 안전한 빈 좌표 탐색 (나선형 탐색)
//   2. rndmonst() — 난이도 기반 몬스터 선택 (깊이/분기 보정)
//   3. 유일 몬스터 / 제노사이드 검사
//   4. 초기 HP 계산 (원본 공식 재현)
//   5. 초기 충성도/평화도 판정
//   6. 초기 속도 계산
//   7. 몬스터 분류/그룹 생성 정책
//   8. 특수 생성 조건 (지옥/비지옥, 분기 전용 등)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] enexto — 안전한 빈 좌표 탐색 (원본: makemon.c enexto, goodpos)
// =============================================================================

/// [v2.22.0 R10-3] 좌표 안전성 체크 입력
#[derive(Debug, Clone)]
pub struct GoodPosInput {
    /// 맵 폭
    pub map_width: i32,
    /// 맵 높이
    pub map_height: i32,
}

/// [v2.22.0 R10-3] 좌표의 "좋은 위치" 여부 판별 (원본: goodpos)
/// `is_passable`: (x, y) → 해당 좌표가 이동 가능한 지형인지
/// `is_occupied`: (x, y) → 해당 좌표에 이미 엔티티가 있는지
pub fn goodpos<F1, F2>(
    x: i32,
    y: i32,
    input: &GoodPosInput,
    mut is_passable: F1,
    mut is_occupied: F2,
) -> bool
where
    F1: FnMut(i32, i32) -> bool,
    F2: FnMut(i32, i32) -> bool,
{
    // 범위 체크
    if x < 1 || x >= input.map_width - 1 || y < 0 || y >= input.map_height {
        return false;
    }
    // 이동 가능 지형인지
    if !is_passable(x, y) {
        return false;
    }
    // 이미 점유된 좌표인지
    if is_occupied(x, y) {
        return false;
    }
    true
}

/// [v2.22.0 R10-3] 안전한 빈 좌표 탐색 — 나선형 확장 (원본: enexto)
/// 기준 좌표 (cx, cy) 근처에서 배치 가능한 좌표를 나선형으로 탐색
pub fn enexto<F1, F2>(
    cx: i32,
    cy: i32,
    input: &GoodPosInput,
    is_passable: F1,
    is_occupied: F2,
) -> Option<(i32, i32)>
where
    F1: FnMut(i32, i32) -> bool + Clone,
    F2: FnMut(i32, i32) -> bool + Clone,
{
    // 기준 좌표 자체가 좋은 위치인지 먼저 확인
    if goodpos(cx, cy, input, is_passable.clone(), is_occupied.clone()) {
        return Some((cx, cy));
    }

    // 나선형 탐색 (최대 반경 15)
    for radius in 1i32..=15 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                // 현재 반경 테두리만 검사 (이전 반경은 이미 검사함)
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }
                let nx = cx + dx;
                let ny = cy + dy;
                if goodpos(nx, ny, input, is_passable.clone(), is_occupied.clone()) {
                    return Some((nx, ny));
                }
            }
        }
    }
    None
}

// =============================================================================
// [2] rndmonst — 난이도 기반 몬스터 선택 (원본: makemon.c rndmonst)
// =============================================================================

/// [v2.22.0 R10-3] 몬스터 난이도 선택 입력
#[derive(Debug, Clone)]
pub struct MonsterSelectInput {
    /// 현재 던전 깊이
    pub depth: i32,
    /// 분기 보정 (Mines → +2, Gehennom → +5 등)
    pub branch_bonus: i32,
    /// 플레이어 레벨 (고레벨 시 강한 몹 출현)
    pub player_level: i32,
}

/// [v2.22.0 R10-3] 몬스터 선택 후보 엔트리
#[derive(Debug, Clone)]
pub struct MonsterCandidate {
    /// 템플릿 인덱스
    pub index: usize,
    /// 몬스터 이름
    pub name: String,
    /// 몬스터 기본 난이도
    pub difficulty: i32,
    /// 출현 빈도 가중치 (G_FREQ)
    pub frequency: i32,
    /// 유일 몬스터 여부
    pub is_unique: bool,
    /// 생성 불가 (G_NOGEN) 여부
    pub is_nogen: bool,
    /// 지옥 전용 여부
    pub hell_only: bool,
    /// 제노사이드 여부
    pub genocided: bool,
}

/// [v2.22.0 R10-3] rndmonst 난이도 기반 몬스터 선택 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonsterSelectResult {
    /// 선택된 몬스터 인덱스
    Selected(usize),
    /// 후보 없음
    NoCandidates,
}

/// [v2.22.0 R10-3] 난이도 기반 후보 필터링 (원본: rndmonst 핵심 로직)
pub fn filter_candidates(
    candidates: &[MonsterCandidate],
    input: &MonsterSelectInput,
    in_hell: bool,
) -> Vec<(usize, i32)> {
    let effective_depth = input.depth + input.branch_bonus;
    // 난이도 상한: 깊이 + (깊이/10) + 2 (원본 공식)
    let max_difficulty = effective_depth + (effective_depth / 10) + 2;

    candidates
        .iter()
        .filter(|c| {
            // 생성 불가 필터
            if c.is_nogen || c.genocided || c.is_unique {
                return false;
            }
            // 난이도 필터
            if c.difficulty > max_difficulty {
                return false;
            }
            // 지옥 전용 필터
            if c.hell_only && !in_hell {
                return false;
            }
            true
        })
        .map(|c| (c.index, c.frequency.max(1)))
        .collect()
}

/// [v2.22.0 R10-3] 가중 랜덤 선택 (원본: rndmonst)
pub fn select_weighted(weighted: &[(usize, i32)], rng: &mut NetHackRng) -> MonsterSelectResult {
    if weighted.is_empty() {
        return MonsterSelectResult::NoCandidates;
    }
    let total: i32 = weighted.iter().map(|(_, w)| w).sum();
    if total <= 0 {
        return MonsterSelectResult::NoCandidates;
    }
    let mut pick = rng.rn2(total);
    for &(idx, weight) in weighted {
        if pick < weight {
            return MonsterSelectResult::Selected(idx);
        }
        pick -= weight;
    }
    MonsterSelectResult::Selected(weighted[0].0)
}

/// [v2.22.0 R10-3] rndmonst 통합 API
pub fn rndmonst(
    candidates: &[MonsterCandidate],
    input: &MonsterSelectInput,
    in_hell: bool,
    rng: &mut NetHackRng,
) -> MonsterSelectResult {
    let filtered = filter_candidates(candidates, input, in_hell);
    select_weighted(&filtered, rng)
}

// =============================================================================
// [3] 유일 몬스터 / 제노사이드 검사 (원본: makemon.c is_unique, mvitals)
// =============================================================================

/// [v2.22.0 R10-3] 유일 몬스터 생성 가능 여부 (원본: can_gen_unique)
pub fn can_gen_unique(monster_name: &str, already_generated: &[String], is_unique: bool) -> bool {
    if !is_unique {
        return true; // 유일 몬스터가 아닌 경우 항상 생성 가능
    }
    // 이미 생성된 유일 몬스터는 다시 생성 불가
    !already_generated.iter().any(|n| n == monster_name)
}

/// [v2.22.0 R10-3] 제노사이드 검사 (원본: mvitals[].mvflags & G_GENOD)
pub fn is_genocided(monster_name: &str, genocided_list: &[String]) -> bool {
    genocided_list.iter().any(|g| g == monster_name)
}

// =============================================================================
// [4] 초기 HP 계산 (원본: makemon.c, mhp = d(m_lev, 8))
// =============================================================================

/// [v2.22.0 R10-3] 몬스터 초기 HP 계산 (원본: newmonhp)
/// m_lev: 조정된 몬스터 레벨, base_lev: 템플릿 기본 레벨
pub fn calc_initial_hp(m_lev: i32, base_lev: i32, rng: &mut NetHackRng) -> i32 {
    if m_lev <= 0 {
        // 레벨 0 이하: 1d4
        return rng.rn1(4, 1).max(1);
    }
    // d(m_lev, 8) 기본 HP
    let mut hp = 0;
    for _ in 0..m_lev {
        hp += rng.rn1(8, 1);
    }
    // 높은 레벨 보정: 기본 레벨보다 실제 레벨이 크면 추가 HP
    if m_lev > base_lev {
        let bonus_dice = m_lev - base_lev;
        for _ in 0..bonus_dice {
            hp += rng.rn1(4, 1);
        }
    }
    hp.max(1)
}

/// [v2.22.0 R10-3] 몬스터 조정 레벨 계산 (원본: makemon)
/// 던전 깊이가 기본 레벨보다 깊으면 레벨 올림
pub fn calc_adjusted_level(base_level: i32, dungeon_depth: i32) -> i32 {
    let mut m_lev = base_level;
    if dungeon_depth > m_lev + 2 {
        m_lev += (dungeon_depth - m_lev) / 2;
    }
    m_lev.max(1)
}

// =============================================================================
// [5] 초기 충성도/평화도 판정 (원본: makemon.c, peace_minded)
// =============================================================================

/// [v2.22.0 R10-3] 평화 판정 입력
#[derive(Debug, Clone)]
pub struct PeaceInput {
    /// 몬스터가 기본적으로 평화로운지
    pub is_peaceful_template: bool,
    /// 플레이어의 Alignment (양수=질서, 음수=혼돈)
    pub player_alignment: i32,
    /// 몬스터의 Alignment (양수=질서, 음수=혼돈)
    pub monster_alignment: i32,
    /// 같은 종족 여부
    pub same_race: bool,
    /// MM_PEACEFUL 플래그로 강제 평화인지
    pub force_peaceful: bool,
}

/// [v2.22.0 R10-3] 평화 판정 (원본: peace_minded)
pub fn is_peace_minded(input: &PeaceInput) -> bool {
    if input.force_peaceful {
        return true;
    }
    if input.is_peaceful_template {
        return true;
    }
    // 같은 종족이면 평화
    if input.same_race {
        return true;
    }
    // 같은 성향이면 약간의 확률로 평화
    if input.player_alignment > 0 && input.monster_alignment > 0 {
        return true; // 둘 다 질서
    }
    false
}

// =============================================================================
// [6] 초기 속도 계산 (원본: makemon.c, mcalcmove)
// =============================================================================

/// [v2.22.0 R10-3] 이동 속도 계산 (원본: mcalcmove)
/// base_speed: 템플릿 기본 속도 (NORMAL_SPEED = 12)
pub fn calc_move_speed(base_speed: i32) -> i32 {
    // 원본: 12가 기본, 한 턴에 1타일 이동
    // 속도 > 12: 추가 행동 기회, 속도 < 12: 행동 건너뜀
    base_speed.max(1).min(30) // 최소 1, 최대 30
}

/// [v2.22.0 R10-3] 이동 에너지 소모량 (원본: movement_per_turn)
/// NORMAL_SPEED(12) 기준으로 12 에너지 획득, 이동에 12 소모
pub fn movement_energy(speed: i32) -> i32 {
    // 매 턴 speed만큼 에너지 획득, 행동 시 12 소모
    speed
}

// =============================================================================
// [7] 몬스터 그룹 생성 정책 (원본: makemon.c m_initgrp)
// =============================================================================

/// [v2.22.0 R10-3] 그룹 크기 결정 (원본: m_initgrp)
pub fn calc_group_size(geno_flags: u32, rng: &mut NetHackRng) -> i32 {
    const G_SGROUP: u32 = 0x0200;
    const G_LGROUP: u32 = 0x0400;

    if (geno_flags & G_LGROUP) != 0 {
        // 대그룹: 1d(6) + 2
        rng.rn1(6, 1) + 2
    } else if (geno_flags & G_SGROUP) != 0 {
        // 소그룹: 1d(3) + 1
        rng.rn1(3, 1) + 1
    } else {
        0 // 그룹 없음
    }
}

/// [v2.22.0 R10-3] 그룹 생성 가능 여부 (MM_NOGRP 체크)
pub fn should_create_group(mm_flags: u32, geno_flags: u32) -> bool {
    const MM_NOGRP: u32 = 0x01;
    const G_SGROUP: u32 = 0x0200;
    const G_LGROUP: u32 = 0x0400;

    if (mm_flags & MM_NOGRP) != 0 {
        return false;
    }
    (geno_flags & G_SGROUP) != 0 || (geno_flags & G_LGROUP) != 0
}

// =============================================================================
// [8] 특수 생성 조건 (원본: makemon.c, hell/non-hell, branch-specific)
// =============================================================================

/// [v2.22.0 R10-3] 분기별 생성 제한 검사
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnRestriction {
    /// 어디서든 생성 가능
    Anywhere,
    /// 지옥(Gehennom)에서만
    HellOnly,
    /// 지옥 아닌 곳에서만
    NonHellOnly,
    /// 광산(Mines)에서만
    MinesOnly,
    /// 퀘스트 분기에서만
    QuestOnly,
}

/// [v2.22.0 R10-3] 분기 제한 통과 여부
pub fn passes_branch_restriction(
    restriction: SpawnRestriction,
    is_hell: bool,
    is_mines: bool,
    is_quest: bool,
) -> bool {
    match restriction {
        SpawnRestriction::Anywhere => true,
        SpawnRestriction::HellOnly => is_hell,
        SpawnRestriction::NonHellOnly => !is_hell,
        SpawnRestriction::MinesOnly => is_mines,
        SpawnRestriction::QuestOnly => is_quest,
    }
}

/// [v2.22.0 R10-3] 야간 몬스터 생성 보정 (원본: night_creature)
pub fn is_night_creature(monster_name: &str) -> bool {
    matches!(
        monster_name,
        "vampire" | "vampire lord" | "vampire mage" | "werewolf" | "werejackal" | "wererat"
    )
}

/// [v2.22.0 R10-3] 수중 생성 가능 여부
pub fn can_spawn_in_water(monster_symbol: char) -> bool {
    matches!(monster_symbol, ';' | 'E' | 'P')
}

/// [v2.22.0 R10-3] 용암 위 생성 가능 여부
pub fn can_spawn_in_lava(monster_symbol: char) -> bool {
    matches!(monster_symbol, 'E' | '&')
}

// =============================================================================
// [9] makemon 결과 (Pure Result 패턴)
// =============================================================================

/// [v2.22.0 R10-3] makemon 순수 결과 (ECS 비의존)
#[derive(Debug, Clone)]
pub struct MakeMonResult {
    /// 생성 좌표
    pub x: i32,
    pub y: i32,
    /// 템플릿 인덱스
    pub template_index: usize,
    /// 초기 HP
    pub hp: i32,
    /// 조정된 레벨
    pub adjusted_level: i32,
    /// 평화 여부
    pub peaceful: bool,
    /// 이동 속도
    pub speed: i32,
    /// 그룹 크기 (0 = 단독)
    pub group_size: i32,
}

/// [v2.22.0 R10-3] makemon 순수 생성 계획 산출 (ECS 비의존)
pub fn plan_makemon(
    candidates: &[MonsterCandidate],
    select_input: &MonsterSelectInput,
    peace_input: &PeaceInput,
    base_speed: i32,
    geno_flags: u32,
    mm_flags: u32,
    spawn_x: i32,
    spawn_y: i32,
    in_hell: bool,
    rng: &mut NetHackRng,
) -> Option<MakeMonResult> {
    // 몬스터 선택
    let select_result = rndmonst(candidates, select_input, in_hell, rng);
    let template_index = match select_result {
        MonsterSelectResult::Selected(idx) => idx,
        MonsterSelectResult::NoCandidates => return None,
    };

    let candidate = &candidates[template_index];

    // 레벨 & HP 계산
    let adjusted_level = calc_adjusted_level(candidate.difficulty, select_input.depth);
    let hp = calc_initial_hp(adjusted_level, candidate.difficulty, rng);

    // 평화 판정
    let peaceful = is_peace_minded(peace_input);

    // 속도
    let speed = calc_move_speed(base_speed);

    // 그룹
    let group_size = if should_create_group(mm_flags, geno_flags) {
        calc_group_size(geno_flags, rng)
    } else {
        0
    };

    Some(MakeMonResult {
        x: spawn_x,
        y: spawn_y,
        template_index,
        hp,
        adjusted_level,
        peaceful,
        speed,
        group_size,
    })
}

// =============================================================================
// [10] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidates() -> Vec<MonsterCandidate> {
        vec![
            MonsterCandidate {
                index: 0,
                name: "grid bug".to_string(),
                difficulty: 1,
                frequency: 5,
                is_unique: false,
                is_nogen: false,
                hell_only: false,
                genocided: false,
            },
            MonsterCandidate {
                index: 1,
                name: "jackal".to_string(),
                difficulty: 1,
                frequency: 3,
                is_unique: false,
                is_nogen: false,
                hell_only: false,
                genocided: false,
            },
            MonsterCandidate {
                index: 2,
                name: "kobold".to_string(),
                difficulty: 2,
                frequency: 4,
                is_unique: false,
                is_nogen: false,
                hell_only: false,
                genocided: false,
            },
            MonsterCandidate {
                index: 3,
                name: "orc".to_string(),
                difficulty: 3,
                frequency: 3,
                is_unique: false,
                is_nogen: false,
                hell_only: false,
                genocided: false,
            },
            MonsterCandidate {
                index: 4,
                name: "troll".to_string(),
                difficulty: 10,
                frequency: 2,
                is_unique: false,
                is_nogen: false,
                hell_only: false,
                genocided: false,
            },
            MonsterCandidate {
                index: 5,
                name: "dragon".to_string(),
                difficulty: 20,
                frequency: 1,
                is_unique: false,
                is_nogen: false,
                hell_only: false,
                genocided: false,
            },
            MonsterCandidate {
                index: 6,
                name: "Medusa".to_string(),
                difficulty: 15,
                frequency: 0,
                is_unique: true,
                is_nogen: false,
                hell_only: false,
                genocided: false,
            },
            MonsterCandidate {
                index: 7,
                name: "pit fiend".to_string(),
                difficulty: 12,
                frequency: 2,
                is_unique: false,
                is_nogen: false,
                hell_only: true,
                genocided: false,
            },
        ]
    }

    #[test]
    fn test_goodpos_valid() {
        let input = GoodPosInput {
            map_width: 80,
            map_height: 21,
        };
        assert!(goodpos(5, 5, &input, |_, _| true, |_, _| false));
    }

    #[test]
    fn test_goodpos_out_of_bounds() {
        let input = GoodPosInput {
            map_width: 80,
            map_height: 21,
        };
        assert!(!goodpos(0, 0, &input, |_, _| true, |_, _| false)); // x=0 → 범위 밖
        assert!(!goodpos(80, 10, &input, |_, _| true, |_, _| false)); // x=80 초과
    }

    #[test]
    fn test_goodpos_occupied() {
        let input = GoodPosInput {
            map_width: 80,
            map_height: 21,
        };
        assert!(!goodpos(5, 5, &input, |_, _| true, |_, _| true));
    }

    #[test]
    fn test_enexto_direct() {
        let input = GoodPosInput {
            map_width: 80,
            map_height: 21,
        };
        let result = enexto(10, 10, &input, |_, _| true, |_, _| false);
        assert_eq!(result, Some((10, 10)));
    }

    #[test]
    fn test_enexto_spiral() {
        let input = GoodPosInput {
            map_width: 80,
            map_height: 21,
        };
        // 중심(10, 10)이 점유되어 있을 때 근처 탐색
        let result = enexto(10, 10, &input, |_, _| true, |x, y| x == 10 && y == 10);
        assert!(result.is_some());
        let (nx, ny) = result.unwrap();
        assert!((nx - 10).abs() <= 1 && (ny - 10).abs() <= 1);
    }

    #[test]
    fn test_filter_candidates_depth1() {
        let candidates = make_candidates();
        let input = MonsterSelectInput {
            depth: 1,
            branch_bonus: 0,
            player_level: 1,
        };
        let filtered = filter_candidates(&candidates, &input, false);
        // 깊이 1 → max_difficulty = 1 + 0 + 2 = 3
        // grid bug(1), jackal(1), kobold(2), orc(3) 통과. troll(10), dragon(20) 불통
        // Medusa(unique), pit fiend(hell_only, diff=12) 불통
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    fn test_filter_candidates_hell() {
        let candidates = make_candidates();
        let input = MonsterSelectInput {
            depth: 15,
            branch_bonus: 0,
            player_level: 15,
        };
        let filtered = filter_candidates(&candidates, &input, true);
        // 지옥: pit fiend(hell_only=true, diff=12) 통과
        assert!(filtered.iter().any(|(idx, _)| *idx == 7));
    }

    #[test]
    fn test_filter_candidates_no_hell() {
        let candidates = make_candidates();
        let input = MonsterSelectInput {
            depth: 15,
            branch_bonus: 0,
            player_level: 15,
        };
        let filtered = filter_candidates(&candidates, &input, false);
        // 지옥 아님: pit fiend 불통
        assert!(!filtered.iter().any(|(idx, _)| *idx == 7));
    }

    #[test]
    fn test_select_weighted() {
        let weighted = vec![(0, 5), (1, 3), (2, 4)];
        let mut rng = NetHackRng::new(42);
        let result = select_weighted(&weighted, &mut rng);
        assert!(matches!(result, MonsterSelectResult::Selected(_)));
    }

    #[test]
    fn test_select_weighted_empty() {
        let weighted: Vec<(usize, i32)> = vec![];
        let mut rng = NetHackRng::new(42);
        let result = select_weighted(&weighted, &mut rng);
        assert_eq!(result, MonsterSelectResult::NoCandidates);
    }

    #[test]
    fn test_can_gen_unique() {
        assert!(can_gen_unique("Medusa", &[], true));
        assert!(!can_gen_unique("Medusa", &["Medusa".to_string()], true));
        assert!(can_gen_unique("orc", &[], false));
    }

    #[test]
    fn test_is_genocided() {
        assert!(is_genocided(
            "orc",
            &["orc".to_string(), "kobold".to_string()]
        ));
        assert!(!is_genocided("troll", &["orc".to_string()]));
    }

    #[test]
    fn test_calc_initial_hp() {
        let mut rng = NetHackRng::new(42);
        let hp = calc_initial_hp(5, 3, &mut rng);
        assert!(hp >= 5); // 최소 d(5,8) = 5
    }

    #[test]
    fn test_calc_initial_hp_zero_level() {
        let mut rng = NetHackRng::new(42);
        let hp = calc_initial_hp(0, 0, &mut rng);
        assert!(hp >= 1 && hp <= 4); // 1d4
    }

    #[test]
    fn test_calc_adjusted_level() {
        assert_eq!(calc_adjusted_level(3, 3), 3);
        assert_eq!(calc_adjusted_level(3, 10), 6); // 3 + (10-3)/2 = 6
        assert_eq!(calc_adjusted_level(1, 1), 1);
    }

    #[test]
    fn test_peace_minded() {
        // 강제 평화
        assert!(is_peace_minded(&PeaceInput {
            is_peaceful_template: false,
            player_alignment: -5,
            monster_alignment: -5,
            same_race: false,
            force_peaceful: true,
        }));
        // 같은 종족
        assert!(is_peace_minded(&PeaceInput {
            is_peaceful_template: false,
            player_alignment: 5,
            monster_alignment: -5,
            same_race: true,
            force_peaceful: false,
        }));
        // 적대적
        assert!(!is_peace_minded(&PeaceInput {
            is_peaceful_template: false,
            player_alignment: 5,
            monster_alignment: -5,
            same_race: false,
            force_peaceful: false,
        }));
    }

    #[test]
    fn test_calc_move_speed() {
        assert_eq!(calc_move_speed(12), 12);
        assert_eq!(calc_move_speed(0), 1); // 최소 1
        assert_eq!(calc_move_speed(50), 30); // 최대 30
    }

    #[test]
    fn test_group_size() {
        let mut rng = NetHackRng::new(42);
        let size = calc_group_size(0x0400, &mut rng); // G_LGROUP
        assert!(size >= 3 && size <= 8); // 1d6+2
    }

    #[test]
    fn test_should_create_group() {
        assert!(!should_create_group(0x01, 0x0200)); // MM_NOGRP → 불가
        assert!(should_create_group(0x00, 0x0200)); // G_SGROUP
        assert!(should_create_group(0x00, 0x0400)); // G_LGROUP
        assert!(!should_create_group(0x00, 0x0000)); // 그룹 아님
    }

    #[test]
    fn test_branch_restriction() {
        assert!(passes_branch_restriction(
            SpawnRestriction::Anywhere,
            false,
            false,
            false
        ));
        assert!(passes_branch_restriction(
            SpawnRestriction::HellOnly,
            true,
            false,
            false
        ));
        assert!(!passes_branch_restriction(
            SpawnRestriction::HellOnly,
            false,
            false,
            false
        ));
        assert!(passes_branch_restriction(
            SpawnRestriction::MinesOnly,
            false,
            true,
            false
        ));
    }

    #[test]
    fn test_night_creature() {
        assert!(is_night_creature("vampire"));
        assert!(is_night_creature("werewolf"));
        assert!(!is_night_creature("orc"));
    }

    #[test]
    fn test_water_lava_spawn() {
        assert!(can_spawn_in_water(';'));
        assert!(!can_spawn_in_water('o'));
        assert!(can_spawn_in_lava('&'));
        assert!(!can_spawn_in_lava('o'));
    }

    #[test]
    fn test_plan_makemon() {
        let candidates = make_candidates();
        let select_input = MonsterSelectInput {
            depth: 5,
            branch_bonus: 0,
            player_level: 5,
        };
        let peace_input = PeaceInput {
            is_peaceful_template: false,
            player_alignment: 5,
            monster_alignment: -5,
            same_race: false,
            force_peaceful: false,
        };
        let mut rng = NetHackRng::new(42);

        let result = plan_makemon(
            &candidates,
            &select_input,
            &peace_input,
            12,
            0x0000,
            0x00,
            10,
            10,
            false,
            &mut rng,
        );
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 10);
        assert!(r.hp >= 1);
        assert!(!r.peaceful);
        assert_eq!(r.speed, 12);
        assert_eq!(r.group_size, 0); // 그룹 플래그 없음
    }
}

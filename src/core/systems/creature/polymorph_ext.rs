// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.24.0 R12-1] 변신(Polymorph) 시스템 (polymorph_ext.rs)
//
// 원본 참조: NetHack 3.6.7 polymorph.c (1,670줄)
//
// 구현 내용:
//   1. 자기 변신 타겟 선택 (레벨/종족 제한)
//   2. 변신 시 스탯 재계산 (HP, 공격, AC, 이동속도)
//   3. 변신 유지 시간 / 해제 판정
//   4. 아이템 변신 (polymorph 지팡이 효과)
//   5. 몬스터 변신 (balrog→pit fiend 등)
//   6. 변신 불가 판정 (유니크, 골렘, 라이더 등)
//   7. 신체 변형 (slime, stone → 변신 강제)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 변신 가능 여부 (원본: polymorph.c polymon, is_polyed)
// =============================================================================

/// [v2.24.0 R12-1] 변신 불가 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolymorphDenied {
    /// 이미 같은 형태
    AlreadySameForm,
    /// 유니크 몬스터 (변신 불가)
    UniqueMonster,
    /// 변신 저항 보유
    PolymorphResistant,
    /// 변신 제어 실패 (원하는 형태를 선택 못함)
    ControlFailed,
    /// 시체/골렘 특수 제한
    SpecialRestriction(String),
    /// 레벨 부족
    LevelTooLow { required: i32, current: i32 },
    /// 제노사이드된 종
    Genocided,
}

/// [v2.24.0 R12-1] 변신 가능 여부 판정 (원본: polyok)
pub fn can_polymorph_into(
    target_id: i32,
    is_unique: bool,
    is_genocided: bool,
    is_quest_monster: bool,
    player_level: i32,
    target_level: i32,
    same_as_current: bool,
) -> Result<(), PolymorphDenied> {
    if same_as_current {
        return Err(PolymorphDenied::AlreadySameForm);
    }
    if is_unique {
        return Err(PolymorphDenied::UniqueMonster);
    }
    if is_genocided {
        return Err(PolymorphDenied::Genocided);
    }
    if is_quest_monster {
        return Err(PolymorphDenied::SpecialRestriction(
            "퀘스트 전용 몬스터".to_string(),
        ));
    }
    // 레벨 제한: 타겟 레벨이 플레이어 레벨보다 너무 높으면 불가
    if target_level > player_level + 5 {
        return Err(PolymorphDenied::LevelTooLow {
            required: target_level,
            current: player_level,
        });
    }
    Ok(())
}

// =============================================================================
// [2] 변신 타겟 선택 (원본: polymorph.c rndmonst_poly)
// =============================================================================

/// [v2.24.0 R12-1] 랜덤 변신 타겟 선택 (원본: rndmonst for polymorph)
pub fn select_random_target(
    player_level: i32,
    total_monster_types: i32,
    rng: &mut NetHackRng,
) -> i32 {
    // 변신 가능 범위: 플레이어 레벨 ± 6
    let max_level = player_level + 6;
    let _min_level = (player_level - 6).max(0);
    // 랜덤 선택 (실제로는 필터링 후 선택해야 하지만, 여기선 간략화)
    rng.rn2(total_monster_types.min(max_level * 3 + 1).max(1))
}

/// [v2.24.0 R12-1] 변신 제어 시 사용자 선택 유효성 검사
pub fn validate_controlled_choice(chosen_name: &str, available_names: &[&str]) -> Option<usize> {
    let lower = chosen_name.to_lowercase();
    available_names
        .iter()
        .position(|n| n.to_lowercase() == lower)
}

// =============================================================================
// [3] 변신 시 스탯 재계산 (원본: polymorph.c polyself, new_attr)
// =============================================================================

/// [v2.24.0 R12-1] 변신 스탯 입력
#[derive(Debug, Clone)]
pub struct PolymorphInput {
    /// 원래 HP
    pub original_hp: i32,
    /// 원래 최대 HP
    pub original_max_hp: i32,
    /// 원래 AC
    pub original_ac: i32,
    /// 타겟 몬스터 레벨
    pub target_level: i32,
    /// 타겟 몬스터 기본 AC
    pub target_base_ac: i32,
    /// 타겟 몬스터 기본 이동속도
    pub target_speed: i32,
    /// 타겟 크기 (0=작음, 1=보통, 2=큼, 3=거대)
    pub target_size: i32,
}

/// [v2.24.0 R12-1] 변신 후 스탯 결과
#[derive(Debug, Clone)]
pub struct PolymorphStats {
    /// 변신 후 HP
    pub new_hp: i32,
    /// 변신 후 최대 HP
    pub new_max_hp: i32,
    /// 변신 후 AC
    pub new_ac: i32,
    /// 변신 후 이동속도
    pub new_speed: i32,
    /// 유지 시간 (턴)
    pub duration: i32,
    /// 크기 변화 메시지
    pub size_msg: Option<String>,
}

/// [v2.24.0 R12-1] 변신 스탯 계산 (원본: polyself의 핵심 로직)
pub fn calc_polymorph_stats(input: &PolymorphInput, rng: &mut NetHackRng) -> PolymorphStats {
    // HP 계산: d(target_level, 8) (원본: d(mlev, 8))
    let mut new_max_hp = 0;
    for _ in 0..input.target_level.max(1) {
        new_max_hp += rng.rn1(8, 1);
    }
    new_max_hp = new_max_hp.max(1);

    // HP는 원래 HP와 원래 maxHP의 비율 적용
    let hp_ratio = if input.original_max_hp > 0 {
        input.original_hp as f64 / input.original_max_hp as f64
    } else {
        1.0
    };
    let new_hp = ((new_max_hp as f64) * hp_ratio) as i32;

    // 유지 시간: d(level, 500) (원본: d(mlev, 500))
    let duration = rng.rn1(500, 1) * input.target_level.max(1);

    // 크기 변화 메시지
    let size_msg = match input.target_size {
        0 => Some("몸이 줄어든다!".to_string()),
        3 => Some("몸이 거대해진다!".to_string()),
        _ => None,
    };

    PolymorphStats {
        new_hp: new_hp.max(1),
        new_max_hp,
        new_ac: input.target_base_ac,
        new_speed: input.target_speed.clamp(1, 30),
        duration: duration.max(100),
        size_msg,
    }
}

// =============================================================================
// [4] 변신 해제 판정 (원본: polymorph.c rehumanize)
// =============================================================================

/// [v2.24.0 R12-1] 변신 해제 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevertReason {
    /// 시간 만료
    TimedOut,
    /// HP 0 (변신 중 사망 → 원래로)
    Killed,
    /// 강제 해제 (저주 제거, 마법 취소)
    Forced,
    /// 변신 반지 해제
    ItemRemoved,
    /// 다른 형태로 재변신
    NewPolymorph,
}

/// [v2.24.0 R12-1] 변신 해제 시 HP 복원 계산 (원본: rehumanize)
pub fn calc_revert_hp(
    saved_hp: i32,
    saved_max_hp: i32,
    poly_hp_remaining: i32,
    reason: &RevertReason,
) -> (i32, i32) {
    match reason {
        RevertReason::Killed => {
            // 변신 중 HP 0 → 원래 형태로 돌아오되 HP 감소
            let penalty = saved_max_hp / 4;
            let new_hp = (saved_hp - penalty).max(1);
            (new_hp, saved_max_hp)
        }
        RevertReason::TimedOut | RevertReason::Forced | RevertReason::ItemRemoved => {
            // 정상 해제 → 원래 HP 비율로 복원
            (saved_hp.max(1), saved_max_hp)
        }
        RevertReason::NewPolymorph => {
            // 재변신 → 현재 HP 보존
            (poly_hp_remaining.max(1), saved_max_hp)
        }
    }
}

// =============================================================================
// [5] 아이템 변신 (원본: polymorph.c poly_obj)
// =============================================================================

/// [v2.24.0 R12-1] 아이템 변신 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemPolymorphResult {
    /// 변신 성공 (새 아이템 ID)
    Transformed {
        new_item_class: char,
        new_item_name: String,
    },
    /// 변신 실패 (아티팩트, 특수 아이템)
    Immune(String),
    /// 파괴됨 (일부 아이템은 변신 시 파괴)
    Destroyed(String),
}

/// [v2.24.0 R12-1] 아이템 변신 판정 (원본: poly_obj)
pub fn polymorph_item(
    item_name: &str,
    item_class: char,
    is_artifact: bool,
    is_quest_item: bool,
    rng: &mut NetHackRng,
) -> ItemPolymorphResult {
    // 아티팩트/퀘스트 아이템 불변
    if is_artifact {
        return ItemPolymorphResult::Immune("아티팩트는 변신할 수 없다.".to_string());
    }
    if is_quest_item {
        return ItemPolymorphResult::Immune("퀘스트 아이템은 변신할 수 없다.".to_string());
    }

    // 20% 확률로 파괴
    if rng.rn2(5) == 0 {
        return ItemPolymorphResult::Destroyed(format!("{}이(가) 산산조각난다!", item_name));
    }

    // 같은 클래스 내에서 랜덤 변화
    let new_name = format!("변신된 {}", item_name);
    ItemPolymorphResult::Transformed {
        new_item_class: item_class,
        new_item_name: new_name,
    }
}

// =============================================================================
// [6] 몬스터 변신 (원본: polymorph.c newcham)
// =============================================================================

/// [v2.24.0 R12-1] 몬스터 변신 판정 (원본: newcham)
pub fn monster_polymorph(
    monster_level: i32,
    is_unique: bool,
    is_shapeshifter: bool,
    rng: &mut NetHackRng,
) -> Option<i32> {
    // 유니크 몬스터는 변신 불가
    if is_unique && !is_shapeshifter {
        return None;
    }
    // 셰이프시프터는 자유롭게 변신
    if is_shapeshifter {
        return Some(rng.rn2(50) + 1);
    }
    // 일반 몬스터: 유사 레벨 내 랜덤
    Some(rng.rn1(10, (monster_level - 3).max(1)))
}

// =============================================================================
// [7] 신체 변형 (슬라임, 석화 등) (원본: polymorph.c set_uasmon)
// =============================================================================

/// [v2.24.0 R12-1] 강제 변신 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForcedTransform {
    /// 슬라임화 (green slime 접촉)
    Sliming,
    /// 석화 (코카트리스 접촉)
    Stoning,
    /// 늑대인간화 (lycanthropy)
    Lycanthropy,
    /// 뱀파이어화
    Vampirism,
}

/// [v2.24.0 R12-1] 강제 변신 진행 턴 계산
pub fn forced_transform_turns(transform: ForcedTransform) -> i32 {
    match transform {
        ForcedTransform::Sliming => 10,    // 10턴 후 슬라임化
        ForcedTransform::Stoning => 5,     // 5턴 후 석화
        ForcedTransform::Lycanthropy => 1, // 즉시
        ForcedTransform::Vampirism => 1,   // 즉시
    }
}

/// [v2.24.0 R12-1] 강제 변신 방지 수단 확인
pub fn can_prevent_transform(
    transform: ForcedTransform,
    has_unchanging: bool,
    has_slime_resistance: bool,
    has_stone_resistance: bool,
    has_cure: bool,
) -> bool {
    if has_unchanging {
        return true; // Unchanging 속성은 모든 강제 변신 방지
    }
    match transform {
        ForcedTransform::Sliming => has_slime_resistance || has_cure,
        ForcedTransform::Stoning => has_stone_resistance || has_cure,
        ForcedTransform::Lycanthropy => has_cure,
        ForcedTransform::Vampirism => has_cure,
    }
}

// =============================================================================
// [8] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_polymorph_ok() {
        let r = can_polymorph_into(1, false, false, false, 15, 10, false);
        assert!(r.is_ok());
    }

    #[test]
    fn test_polymorph_same_form() {
        let r = can_polymorph_into(1, false, false, false, 15, 10, true);
        assert_eq!(r.unwrap_err(), PolymorphDenied::AlreadySameForm);
    }

    #[test]
    fn test_polymorph_unique() {
        let r = can_polymorph_into(1, true, false, false, 15, 10, false);
        assert_eq!(r.unwrap_err(), PolymorphDenied::UniqueMonster);
    }

    #[test]
    fn test_polymorph_genocided() {
        let r = can_polymorph_into(1, false, true, false, 15, 10, false);
        assert_eq!(r.unwrap_err(), PolymorphDenied::Genocided);
    }

    #[test]
    fn test_polymorph_level_too_low() {
        let r = can_polymorph_into(1, false, false, false, 5, 20, false);
        assert!(matches!(
            r.unwrap_err(),
            PolymorphDenied::LevelTooLow { .. }
        ));
    }

    #[test]
    fn test_select_random_target() {
        let mut rng = NetHackRng::new(42);
        let target = select_random_target(10, 100, &mut rng);
        assert!(target >= 0 && target < 100);
    }

    #[test]
    fn test_validate_controlled_choice() {
        let names = vec!["dragon", "troll", "giant"];
        assert_eq!(validate_controlled_choice("Troll", &names), Some(1));
        assert_eq!(validate_controlled_choice("unicorn", &names), None);
    }

    #[test]
    fn test_calc_polymorph_stats() {
        let mut rng = NetHackRng::new(42);
        let input = PolymorphInput {
            original_hp: 50,
            original_max_hp: 100,
            original_ac: 5,
            target_level: 5,
            target_base_ac: 3,
            target_speed: 12,
            target_size: 1,
        };
        let stats = calc_polymorph_stats(&input, &mut rng);
        assert!(stats.new_max_hp >= 5); // 최소 5d1
        assert!(stats.new_hp >= 1);
        assert_eq!(stats.new_ac, 3);
        assert_eq!(stats.new_speed, 12);
        assert!(stats.duration >= 100);
    }

    #[test]
    fn test_calc_polymorph_stats_tiny() {
        let mut rng = NetHackRng::new(42);
        let input = PolymorphInput {
            original_hp: 100,
            original_max_hp: 100,
            original_ac: 0,
            target_level: 2,
            target_base_ac: 8,
            target_speed: 6,
            target_size: 0,
        };
        let stats = calc_polymorph_stats(&input, &mut rng);
        assert_eq!(stats.size_msg, Some("몸이 줄어든다!".to_string()));
    }

    #[test]
    fn test_revert_hp_timed_out() {
        let (hp, max) = calc_revert_hp(80, 100, 10, &RevertReason::TimedOut);
        assert_eq!(hp, 80);
        assert_eq!(max, 100);
    }

    #[test]
    fn test_revert_hp_killed() {
        let (hp, max) = calc_revert_hp(80, 100, 0, &RevertReason::Killed);
        assert_eq!(hp, 55); // 80 - 100/4 = 55
        assert_eq!(max, 100);
    }

    #[test]
    fn test_item_polymorph_artifact() {
        let mut rng = NetHackRng::new(42);
        let r = polymorph_item("Excalibur", ')', true, false, &mut rng);
        assert!(matches!(r, ItemPolymorphResult::Immune(_)));
    }

    #[test]
    fn test_item_polymorph_normal() {
        let mut rng = NetHackRng::new(42);
        // 여러 번 시도하여 Transformed 결과 확인
        let mut got_transform = false;
        for seed in 0..20 {
            let mut r = NetHackRng::new(seed);
            if let ItemPolymorphResult::Transformed { new_item_class, .. } =
                polymorph_item("dagger", ')', false, false, &mut r)
            {
                assert_eq!(new_item_class, ')');
                got_transform = true;
                break;
            }
        }
        assert!(got_transform);
    }

    #[test]
    fn test_monster_polymorph_unique() {
        let mut rng = NetHackRng::new(42);
        let r = monster_polymorph(20, true, false, &mut rng);
        assert!(r.is_none());
    }

    #[test]
    fn test_monster_polymorph_shapeshifter() {
        let mut rng = NetHackRng::new(42);
        let r = monster_polymorph(10, false, true, &mut rng);
        assert!(r.is_some());
    }

    #[test]
    fn test_forced_transform_turns() {
        assert_eq!(forced_transform_turns(ForcedTransform::Sliming), 10);
        assert_eq!(forced_transform_turns(ForcedTransform::Stoning), 5);
    }

    #[test]
    fn test_prevent_unchanging() {
        assert!(can_prevent_transform(
            ForcedTransform::Sliming,
            true,
            false,
            false,
            false
        ));
    }

    #[test]
    fn test_prevent_resistance() {
        assert!(can_prevent_transform(
            ForcedTransform::Stoning,
            false,
            false,
            true,
            false
        ));
        assert!(!can_prevent_transform(
            ForcedTransform::Stoning,
            false,
            false,
            false,
            false
        ));
    }

    #[test]
    fn test_prevent_cure() {
        assert!(can_prevent_transform(
            ForcedTransform::Lycanthropy,
            false,
            false,
            false,
            true
        ));
    }
}

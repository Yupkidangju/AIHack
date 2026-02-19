// vault_ext.rs — vault.c 핵심 로직 순수 결과 패턴 이식
// [v2.18.0] 신규 생성: 금고 경비원 대화/반응, 임시 복도 벽 유형, 경비원 이동 등 10개 함수
// 원본: NetHack 3.6.7 src/vault.c (1,152줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 금고 경비원 등장 타이머
pub const VAULT_GUARD_TIME: i32 = 30;

/// 경비원 목격 비트 플래그
pub const GD_EATGOLD: i32 = 1;
pub const GD_DESTROYGOLD: i32 = 2;

/// 벽 유형 (levl[].typ 값)
pub const STONE: i32 = 0;
pub const VWALL: i32 = 1;
pub const HWALL: i32 = 2;
pub const TLCORNER: i32 = 3;
pub const TRCORNER: i32 = 4;
pub const BLCORNER: i32 = 5;
pub const BRCORNER: i32 = 6;
pub const CORR: i32 = 7;

// ============================================================
// 열거형
// ============================================================

/// 금고 경비원 인사 반응
/// 원본: vault.c invault() L452-520
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardReaction {
    /// 크로이소스 사칭 — 크로이소스가 살아있으면 따님
    CroesusAlive,
    /// 크로이소스 사칭 — 이미 죽었으면 분노
    CroesusDeadAngry,
    /// 모르는 사람 — 돈 없으면 따라오라고
    UnknownNoGold,
    /// 모르는 사람 — 숨긴 돈 있으면 내놓으라고
    UnknownHiddenGold,
    /// 모르는 사람 — 보이는 돈 있으면 내놓으라고
    UnknownVisibleGold,
}

/// 경비원 입장 전 상태 체크
/// 원본: vault.c invault() L416-437
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardEntryCheck {
    /// 미믹/숨김 — 경비원 당황 후 퇴장
    MimicOrHiding,
    /// 목 졸림/침묵/행동불능 — 나중에 오겠다고
    CannotSpeak,
    /// 삼킴 상태 — 무슨 일이냐?
    Swallowed,
    /// 정상 진행
    Normal,
}

/// 임시 복도 원래 벽 유형
/// 원본: vault.c invault() L526-545
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FakeCorridorWallType {
    pub wall_type: i32,
}

/// 금고 벽 복원 시 모서리 유형 결정
/// 원본: vault.c wallify_vault() L603-610
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallifyType {
    pub wall_type: i32,
}

// ============================================================
// 1. guard_reaction — 이름 답변에 따른 경비원 반응
// ============================================================

/// 플레이어 이름 답변에 따른 경비원 반응 결정
/// 원본: vault.c invault() L452-520
pub fn guard_reaction(
    is_croesus_name: bool,
    croesus_dead: bool,
    has_visible_gold: bool,
    has_hidden_gold: bool,
) -> GuardReaction {
    if is_croesus_name {
        if croesus_dead {
            GuardReaction::CroesusDeadAngry
        } else {
            GuardReaction::CroesusAlive
        }
    } else if !has_visible_gold && !has_hidden_gold {
        GuardReaction::UnknownNoGold
    } else if has_hidden_gold && !has_visible_gold {
        GuardReaction::UnknownHiddenGold
    } else {
        GuardReaction::UnknownVisibleGold
    }
}

// ============================================================
// 2. guard_entry_check — 경비원 진입 전 상태 확인
// ============================================================

/// 경비원이 금고에 입장했을 때 플레이어 상태 분기
/// 원본: vault.c invault() L407-437
pub fn guard_entry_check(
    is_swallowed: bool,
    is_mimic_or_hiding: bool,
    is_strangled: bool,
    is_silent_form: bool,
    is_paralyzed: bool,
) -> GuardEntryCheck {
    if is_swallowed {
        GuardEntryCheck::Swallowed
    } else if is_mimic_or_hiding {
        GuardEntryCheck::MimicOrHiding
    } else if is_strangled || is_silent_form || is_paralyzed {
        GuardEntryCheck::CannotSpeak
    } else {
        GuardEntryCheck::Normal
    }
}

// ============================================================
// 3. fake_corridor_wall — 임시 복도 원래 벽 유형 결정
// ============================================================

/// 금고 벽 위치에 따라 복구할 벽 유형 결정
/// 원본: vault.c invault() L526-545
pub fn fake_corridor_wall(x: i32, y: i32, lowx: i32, lowy: i32, hix: i32, hiy: i32) -> i32 {
    if x == lowx - 1 && y == lowy - 1 {
        TLCORNER
    } else if x == hix + 1 && y == lowy - 1 {
        TRCORNER
    } else if x == lowx - 1 && y == hiy + 1 {
        BLCORNER
    } else if x == hix + 1 && y == hiy + 1 {
        BRCORNER
    } else if y == lowy - 1 || y == hiy + 1 {
        HWALL
    } else if x == lowx - 1 || x == hix + 1 {
        VWALL
    } else {
        STONE
    }
}

// ============================================================
// 4. wallify_type — 금고 벽 복원 시 벽 유형
// ============================================================

/// 금고 경계 좌표에서 복원할 벽 유형 결정
/// 원본: vault.c wallify_vault() L603-610
pub fn wallify_type(x: i32, y: i32, lox: i32, loy: i32, hix: i32, hiy: i32) -> i32 {
    if x == lox {
        if y == loy {
            TLCORNER
        } else if y == hiy {
            BLCORNER
        } else {
            VWALL
        }
    } else if x == hix {
        if y == loy {
            TRCORNER
        } else if y == hiy {
            BRCORNER
        } else {
            VWALL
        }
    } else {
        HWALL
    }
}

// ============================================================
// 5. move_gold_position — 금고 내 금화 이동 위치
// ============================================================

/// 금화를 금고 내 랜덤 위치로 이동
/// 원본: vault.c move_gold() L563-564
pub fn move_gold_position(vault_lx: i32, vault_ly: i32, rng: &mut NetHackRng) -> (i32, i32) {
    let nx = vault_lx + rng.rn2(2);
    let ny = vault_ly + rng.rn2(2);
    (nx, ny)
}

// ============================================================
// 6. guard_alignment_penalty — 거짓 이름 성향 페널티
// ============================================================

/// 합법 성향의 플레이어가 거짓 이름을 대면 성향 -1
/// 원본: vault.c invault() L452-456
pub fn guard_alignment_penalty(is_lawful: bool, name_matches: bool) -> i32 {
    if is_lawful && !name_matches {
        -1
    } else {
        0
    }
}

// ============================================================
// 7. guard_warncnt_action — 경비원 경고 수에 따른 행동
// ============================================================

/// 경비원 경고 횟수에 따른 행동 결정
/// 원본: vault.c gd_move() L800-900 (warncnt 1→6 분기)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardWarning {
    /// 경고 0: 초기 접근
    Approach,
    /// 경고 1-4: "금 내려놓아!"
    DropGold,
    /// 경고 5: "마지막 경고!"
    FinalWarning,
    /// 경고 6+: 적대 전환
    GoHostile,
}

pub fn guard_warncnt_action(warncnt: i32) -> GuardWarning {
    match warncnt {
        0 => GuardWarning::Approach,
        1..=4 => GuardWarning::DropGold,
        5 => GuardWarning::FinalWarning,
        _ => GuardWarning::GoHostile,
    }
}

// ============================================================
// 8. guard_gold_witness — 경비원이 금 파괴/소비 목격
// ============================================================

/// 경비원이 플레이어의 금 파괴/소비를 목격했을 때 반응
/// 원본: vault.c gd_move() L791-798
pub fn guard_gold_witness(witness_flags: i32) -> &'static str {
    if witness_flags & GD_EATGOLD != 0 {
        "consume"
    } else {
        "destroy"
    }
}

// ============================================================
// 9. vault_guard_timer_check — 경비원 등장 타이밍
// ============================================================

/// 금고 내 체류 시간이 경비원 소환 조건을 만족하는지
/// 원본: vault.c invault() L322
pub fn vault_guard_timer_check(uinvault: i32) -> bool {
    (uinvault + 1) % VAULT_GUARD_TIME == 0
}

// ============================================================
// 10. is_on_boundary — 금고 경계 판정
// ============================================================

/// 좌표가 금고 방 경계에 있는지 판정
/// 원본: vault.c wallify_vault() L588-589
pub fn is_on_boundary(x: i32, y: i32, lox: i32, loy: i32, hix: i32, hiy: i32) -> bool {
    x == lox || x == hix || y == loy || y == hiy
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

    // --- guard_reaction ---
    #[test]
    fn test_croesus_alive() {
        assert_eq!(
            guard_reaction(true, false, false, false),
            GuardReaction::CroesusAlive
        );
    }

    #[test]
    fn test_croesus_dead() {
        assert_eq!(
            guard_reaction(true, true, false, false),
            GuardReaction::CroesusDeadAngry
        );
    }

    #[test]
    fn test_unknown_no_gold() {
        assert_eq!(
            guard_reaction(false, false, false, false),
            GuardReaction::UnknownNoGold
        );
    }

    #[test]
    fn test_unknown_hidden() {
        assert_eq!(
            guard_reaction(false, false, false, true),
            GuardReaction::UnknownHiddenGold
        );
    }

    #[test]
    fn test_unknown_visible() {
        assert_eq!(
            guard_reaction(false, false, true, false),
            GuardReaction::UnknownVisibleGold
        );
    }

    // --- guard_entry_check ---
    #[test]
    fn test_entry_swallowed() {
        assert_eq!(
            guard_entry_check(true, false, false, false, false),
            GuardEntryCheck::Swallowed
        );
    }

    #[test]
    fn test_entry_mimic() {
        assert_eq!(
            guard_entry_check(false, true, false, false, false),
            GuardEntryCheck::MimicOrHiding
        );
    }

    #[test]
    fn test_entry_strangled() {
        assert_eq!(
            guard_entry_check(false, false, true, false, false),
            GuardEntryCheck::CannotSpeak
        );
    }

    #[test]
    fn test_entry_normal() {
        assert_eq!(
            guard_entry_check(false, false, false, false, false),
            GuardEntryCheck::Normal
        );
    }

    // --- fake_corridor_wall ---
    #[test]
    fn test_wall_type_corners() {
        assert_eq!(fake_corridor_wall(4, 4, 5, 5, 10, 10), TLCORNER);
        assert_eq!(fake_corridor_wall(11, 4, 5, 5, 10, 10), TRCORNER);
        assert_eq!(fake_corridor_wall(4, 11, 5, 5, 10, 10), BLCORNER);
        assert_eq!(fake_corridor_wall(11, 11, 5, 5, 10, 10), BRCORNER);
    }

    #[test]
    fn test_wall_type_sides() {
        assert_eq!(fake_corridor_wall(7, 4, 5, 5, 10, 10), HWALL);
        assert_eq!(fake_corridor_wall(4, 7, 5, 5, 10, 10), VWALL);
    }

    // --- wallify_type ---
    #[test]
    fn test_wallify_corners() {
        assert_eq!(wallify_type(3, 3, 3, 3, 12, 12), TLCORNER);
        assert_eq!(wallify_type(12, 3, 3, 3, 12, 12), TRCORNER);
        assert_eq!(wallify_type(3, 12, 3, 3, 12, 12), BLCORNER);
        assert_eq!(wallify_type(12, 12, 3, 3, 12, 12), BRCORNER);
    }

    #[test]
    fn test_wallify_sides() {
        assert_eq!(wallify_type(7, 3, 3, 3, 12, 12), HWALL);
        assert_eq!(wallify_type(3, 7, 3, 3, 12, 12), VWALL);
    }

    // --- move_gold_position ---
    #[test]
    fn test_gold_pos() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let (nx, ny) = move_gold_position(5, 5, &mut rng);
            assert!(nx >= 5 && nx <= 6, "nx: {}", nx);
            assert!(ny >= 5 && ny <= 6, "ny: {}", ny);
        }
    }

    // --- guard_alignment_penalty ---
    #[test]
    fn test_alignment_liar() {
        assert_eq!(guard_alignment_penalty(true, false), -1);
    }

    #[test]
    fn test_alignment_truthful() {
        assert_eq!(guard_alignment_penalty(true, true), 0);
    }

    #[test]
    fn test_alignment_neutral() {
        assert_eq!(guard_alignment_penalty(false, false), 0);
    }

    // --- guard_warncnt_action ---
    #[test]
    fn test_guard_warnings() {
        assert_eq!(guard_warncnt_action(0), GuardWarning::Approach);
        assert_eq!(guard_warncnt_action(1), GuardWarning::DropGold);
        assert_eq!(guard_warncnt_action(4), GuardWarning::DropGold);
        assert_eq!(guard_warncnt_action(5), GuardWarning::FinalWarning);
        assert_eq!(guard_warncnt_action(6), GuardWarning::GoHostile);
    }

    // --- guard_gold_witness ---
    #[test]
    fn test_witness_eat() {
        assert_eq!(guard_gold_witness(GD_EATGOLD), "consume");
    }

    #[test]
    fn test_witness_destroy() {
        assert_eq!(guard_gold_witness(GD_DESTROYGOLD), "destroy");
    }

    // --- vault_guard_timer_check ---
    #[test]
    fn test_timer_spawn() {
        // uinvault=29 → (29+1)%30==0 → true
        assert!(vault_guard_timer_check(VAULT_GUARD_TIME - 1));
        assert!(!vault_guard_timer_check(10));
    }

    // --- is_on_boundary ---
    #[test]
    fn test_boundary() {
        assert!(is_on_boundary(3, 5, 3, 3, 12, 12));
        assert!(is_on_boundary(7, 12, 3, 3, 12, 12));
        assert!(!is_on_boundary(7, 7, 3, 3, 12, 12));
    }
}

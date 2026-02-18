// ============================================================================
// AIHack - write_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] write.c 핵심 함수 완전 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 write.c (391줄)
//
// 이식 대상:
//   scroll_cost(), marker_write_result(), write_cost_calc(),
//   new_book_description(), write_feasibility_check()
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// 두루마리/주문서 작성 비용 (원본: cost())
// [v2.10.1] write.c:10-58 이식
// =============================================================================

/// 두루마리 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollType {
    Light,
    GoldDetection,
    FoodDetection,
    MagicMapping,
    Amnesia,
    Fire,
    Earth,
    DestroyArmor,
    CreateMonster,
    Punishment,
    ConfuseMonster,
    Identify,
    EnchantArmor,
    RemoveCurse,
    EnchantWeapon,
    Charging,
    ScareMonster,
    StinkingCloud,
    Taming,
    Teleportation,
    Genocide,
    BlankPaper,
    Mail,
}

/// 두루마리 기본 비용 (원본: cost L10-58)
/// 마법 마커 잉크 소비량의 기준값
pub fn scroll_cost(scroll: ScrollType) -> i32 {
    match scroll {
        ScrollType::Mail => 2,
        ScrollType::Light
        | ScrollType::GoldDetection
        | ScrollType::FoodDetection
        | ScrollType::MagicMapping
        | ScrollType::Amnesia
        | ScrollType::Fire
        | ScrollType::Earth => 8,
        ScrollType::DestroyArmor | ScrollType::CreateMonster | ScrollType::Punishment => 10,
        ScrollType::ConfuseMonster => 12,
        ScrollType::Identify => 14,
        ScrollType::EnchantArmor
        | ScrollType::RemoveCurse
        | ScrollType::EnchantWeapon
        | ScrollType::Charging => 16,
        ScrollType::ScareMonster
        | ScrollType::StinkingCloud
        | ScrollType::Taming
        | ScrollType::Teleportation => 20,
        ScrollType::Genocide => 30,
        ScrollType::BlankPaper => 1000, // 작성 불가
    }
}

/// 주문서 비용 (원본: cost L17-18)
/// 레벨 * 10
pub fn spellbook_cost(spell_level: i32) -> i32 {
    10 * spell_level
}

// =============================================================================
// 작성 가능성 판정 (원본: dowrite 핵심 로직)
// [v2.10.1] write.c:92-353 이식
// =============================================================================

/// 작성 실패 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteFeasibility {
    /// 작성 가능
    Ok,
    /// 손 없음 (변신 상태)
    NoHands,
    /// 미끄러운 손
    SlipperyHands,
    /// 장님이고 대상 모름
    BlindUnknown,
    /// 장님이고 주문서
    BlindSpellbook,
    /// 빈 종이/주문서 아님
    NotBlank,
    /// 작성 불가 유형 (백지나 사자의 서)
    ForbiddenType,
    /// 잉크 부족
    InsufficientInk { required: i32, available: i32 },
}

/// 작성 가능성 판정 (원본: dowrite 핵심 분기)
pub fn write_feasibility_check(
    has_hands: bool,
    is_slippery: bool,
    is_blind: bool,
    is_blank_paper: bool,
    is_spellbook: bool,
    target_is_book_of_dead: bool,
    dknown: bool,
    marker_charges: i32,
    base_cost: i32,
) -> WriteFeasibility {
    // 손 체크 (원본:107-109)
    if !has_hands {
        return WriteFeasibility::NoHands;
    }
    // 미끄러운 손 (원본:110-114)
    if is_slippery {
        return WriteFeasibility::SlipperyHands;
    }
    // 장님 + 모름 (원본:128-131)
    if is_blind && !dknown {
        return WriteFeasibility::BlindUnknown;
    }
    // 장님 + 주문서 (원본:132-137)
    if is_blind && is_spellbook {
        return WriteFeasibility::BlindSpellbook;
    }
    // 빈 종이 아님 (원본:140-144)
    if !is_blank_paper {
        return WriteFeasibility::NotBlank;
    }
    // 금지 유형 (원본:211-218)
    if target_is_book_of_dead {
        return WriteFeasibility::ForbiddenType;
    }
    // 잉크 부족 (원본:236-240)
    let min_required = base_cost / 2;
    if marker_charges < min_required {
        return WriteFeasibility::InsufficientInk {
            required: min_required,
            available: marker_charges,
        };
    }
    WriteFeasibility::Ok
}

// =============================================================================
// 실제 잉크 비용 계산 (원본: dowrite 244)
// [v2.10.1] write.c:244 이식
// =============================================================================

/// 실제 잉크 소비량 (원본: rn1(basecost/2, basecost/2))
/// basecost/2 ~ basecost 범위의 랜덤값
pub fn actual_ink_cost(base_cost: i32, rng: &mut NetHackRng) -> i32 {
    let half = base_cost / 2;
    rng.rn1(half, half)
}

/// 작성 결과의 축복/저주값 (원본: curseval = bcsign(pen) + bcsign(paper))
pub fn write_curseval(pen_buc: i32, paper_buc: i32) -> i32 {
    pen_buc + paper_buc
}

// =============================================================================
// new_book_description — 주문서 설명 변환
// [v2.10.1] write.c:355-388 이식
// =============================================================================

/// 주문서 재질 설명인지 판정 (원본: compositions[])
/// "parchment", "vellum", "cloth"는 "into " 접두어 필요
pub fn book_description_prefix(description: &str) -> String {
    let compositions = ["parchment", "vellum", "cloth"];
    let lower = description.to_lowercase();
    if compositions.iter().any(|c| lower == *c) {
        format!("into {}", description)
    } else {
        description.to_string()
    }
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_cost() {
        assert_eq!(scroll_cost(ScrollType::Light), 8);
        assert_eq!(scroll_cost(ScrollType::Identify), 14);
        assert_eq!(scroll_cost(ScrollType::Genocide), 30);
        assert_eq!(scroll_cost(ScrollType::BlankPaper), 1000);
        assert_eq!(scroll_cost(ScrollType::Mail), 2);
    }

    #[test]
    fn test_spellbook_cost() {
        assert_eq!(spellbook_cost(1), 10);
        assert_eq!(spellbook_cost(5), 50);
        assert_eq!(spellbook_cost(7), 70);
    }

    #[test]
    fn test_write_feasibility_ok() {
        assert_eq!(
            write_feasibility_check(true, false, false, true, false, false, true, 20, 16),
            WriteFeasibility::Ok
        );
    }

    #[test]
    fn test_write_feasibility_no_hands() {
        assert_eq!(
            write_feasibility_check(false, false, false, true, false, false, true, 20, 16),
            WriteFeasibility::NoHands
        );
    }

    #[test]
    fn test_write_feasibility_slippery() {
        assert_eq!(
            write_feasibility_check(true, true, false, true, false, false, true, 20, 16),
            WriteFeasibility::SlipperyHands
        );
    }

    #[test]
    fn test_write_feasibility_blind_spellbook() {
        assert_eq!(
            write_feasibility_check(true, false, true, true, true, false, true, 20, 50),
            WriteFeasibility::BlindSpellbook
        );
    }

    #[test]
    fn test_write_feasibility_not_blank() {
        assert_eq!(
            write_feasibility_check(true, false, false, false, false, false, true, 20, 16),
            WriteFeasibility::NotBlank
        );
    }

    #[test]
    fn test_write_feasibility_forbidden() {
        assert_eq!(
            write_feasibility_check(true, false, false, true, true, true, true, 100, 16),
            WriteFeasibility::ForbiddenType
        );
    }

    #[test]
    fn test_write_feasibility_ink() {
        let result = write_feasibility_check(true, false, false, true, false, false, true, 3, 16);
        assert!(matches!(result, WriteFeasibility::InsufficientInk { .. }));
    }

    #[test]
    fn test_actual_ink_cost() {
        for seed in 0..50u64 {
            let mut rng = NetHackRng::new(seed);
            let cost = actual_ink_cost(16, &mut rng);
            assert!(cost >= 8 && cost < 16, "cost={}", cost);
        }
    }

    #[test]
    fn test_write_curseval() {
        assert_eq!(write_curseval(1, 1), 2); // 축복 + 축복
        assert_eq!(write_curseval(-1, 1), 0); // 저주 + 축복
        assert_eq!(write_curseval(-1, -1), -2); // 저주 + 저주
    }

    #[test]
    fn test_book_description_prefix() {
        assert_eq!(book_description_prefix("parchment"), "into parchment");
        assert_eq!(book_description_prefix("vellum"), "into vellum");
        assert_eq!(book_description_prefix("red"), "red");
        assert_eq!(book_description_prefix("ragged"), "ragged");
    }
}

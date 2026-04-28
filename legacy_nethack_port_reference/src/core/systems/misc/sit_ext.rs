// ============================================================================
// [v2.28.0 R16-3] 앉기/금고 (sit_ext.rs)
// 원본: NetHack 3.6.7 sit.c (290줄) + vault logic
// 왕좌 앉기 효과, 금고 경비원, 소원
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.28.0 R16-3] 왕좌 앉기 결과 (원본: dosit)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThroneResult {
    /// 소원 획득!
    Wish,
    /// 레벨 업
    LevelGain,
    /// 속성 증가
    StatBoost(String),
    /// 제노사이드 기회
    Genocide,
    /// 왕좌 사라짐
    ThroneVanishes,
    /// 전기 충격
    Shocked(i32),
    /// 저주 걸림
    Cursed,
    /// 몬스터 소환
    SummonMonster,
    /// 텔레포트
    Teleported,
    /// 아무 일 없음
    Nothing,
}

/// [v2.28.0 R16-3] 왕좌 앉기 판정 (원본: dosit throne logic)
pub fn sit_on_throne(luck: i32, rng: &mut NetHackRng) -> ThroneResult {
    let roll = rng.rn2(100);
    // 행운이 양수면 roll을 낮춤 → 좋은 결과 확률 증가
    let luck_adj = luck.clamp(-5, 5);
    let adjusted = (roll - luck_adj).clamp(0, 99);
    match adjusted {
        0..=1 => ThroneResult::Wish,                          // 2%
        2..=6 => ThroneResult::LevelGain,                     // 5%
        7..=11 => ThroneResult::StatBoost("STR".to_string()), // 5%
        12..=14 => ThroneResult::Genocide,                    // 3%
        15..=25 => ThroneResult::ThroneVanishes,              // 11%
        26..=35 => ThroneResult::Shocked(rng.rn1(10, 1)),     // 10%
        36..=45 => ThroneResult::Cursed,                      // 10%
        46..=55 => ThroneResult::SummonMonster,               // 10%
        56..=60 => ThroneResult::Teleported,                  // 5%
        _ => ThroneResult::Nothing,                           // 나머지
    }
}

/// [v2.28.0 R16-3] 금고 경비원 AI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultGuardAction {
    /// 이름 묻기
    AskName,
    /// 호위하여 내보내기
    EscortOut,
    /// 공격 (위조 이름 사용 시)
    Attack,
    /// 철수 (플레이어가 떠남)
    Leave,
}

/// [v2.28.0 R16-3] 금고 경비원 판정
pub fn vault_guard_action(
    player_in_vault: bool,
    player_gave_name: bool,
    name_is_fake: bool,
    player_has_gold: bool,
) -> VaultGuardAction {
    if !player_in_vault {
        return VaultGuardAction::Leave;
    }
    if !player_gave_name {
        return VaultGuardAction::AskName;
    }
    if name_is_fake {
        return VaultGuardAction::Attack;
    }
    if player_has_gold {
        return VaultGuardAction::EscortOut;
    }
    VaultGuardAction::EscortOut
}

/// [v2.28.0 R16-3] 위조 이름 감지 (원본: Croesus 등)
pub fn is_fake_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    let fakes = ["croesus", "wizard", "vlad", "medusa", "dark one"];
    fakes.iter().any(|f| lower.contains(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_throne_outcomes() {
        let mut found_wish = false;
        for seed in 0..500 {
            let mut rng = NetHackRng::new(seed);
            if sit_on_throne(13, &mut rng) == ThroneResult::Wish {
                found_wish = true;
                break;
            }
        }
        assert!(found_wish);
    }

    #[test]
    fn test_vault_ask_name() {
        assert_eq!(
            vault_guard_action(true, false, false, false),
            VaultGuardAction::AskName
        );
    }

    #[test]
    fn test_vault_fake_name() {
        assert_eq!(
            vault_guard_action(true, true, true, false),
            VaultGuardAction::Attack
        );
    }

    #[test]
    fn test_vault_escort() {
        assert_eq!(
            vault_guard_action(true, true, false, true),
            VaultGuardAction::EscortOut
        );
    }

    #[test]
    fn test_fake_name() {
        assert!(is_fake_name("Croesus"));
        assert!(is_fake_name("the wizard"));
        assert!(!is_fake_name("Alice"));
    }
}

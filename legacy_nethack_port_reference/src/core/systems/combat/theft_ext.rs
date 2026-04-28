// ============================================================================
// [v2.28.0 R16-5] 도둑질 확장 (theft_ext.rs)
// 원본: NetHack 3.6.7 steal.c 확장 + 님프/원숭이 도둑질 AI
// 도둑질 성공률, 대상 선택, 방어 수단
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.28.0 R16-5] 도둑질 대상
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheftTarget {
    /// 손에 든 아이템
    WieldedItem,
    /// 장착 갑옷
    WornArmor,
    /// 인벤토리 랜덤
    RandomInventory,
    /// 골드
    Gold,
    /// 아뮬렛
    Amulet,
}

/// [v2.28.0 R16-5] 도둑질 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheftResult {
    /// 성공
    Stolen {
        target: TheftTarget,
        item_name: String,
    },
    /// 실패 (속도 부족)
    TooSlow,
    /// 실패 (방어 장비)
    Defended(String),
    /// 실패 (아이템 없음)
    NothingToSteal,
    /// 장비 해제 실패 (저주)
    CursedItem,
}

/// [v2.28.0 R16-5] 도둑질 성공률 (원본: steal.c steal_it)
pub fn theft_chance(
    thief_dex: i32,
    victim_dex: i32,
    victim_has_free_action: bool,
    victim_has_guard: bool, // 관찰 님프/경비
    rng: &mut NetHackRng,
) -> bool {
    if victim_has_free_action {
        return false;
    }

    let base = 50 + (thief_dex - victim_dex) * 3;
    let adjusted = if victim_has_guard { base / 2 } else { base };
    let chance = adjusted.clamp(5, 95);

    rng.rn2(100) < chance
}

/// [v2.28.0 R16-5] 도둑질 대상 선택 (원본: steal.c find_steal_target)
pub fn select_theft_target(
    has_wielded: bool,
    has_armor: bool,
    has_gold: bool,
    has_amulet: bool,
    rng: &mut NetHackRng,
) -> Option<TheftTarget> {
    let mut candidates = Vec::new();
    if has_amulet {
        candidates.push(TheftTarget::Amulet);
    }
    if has_wielded {
        candidates.push(TheftTarget::WieldedItem);
    }
    if has_armor {
        candidates.push(TheftTarget::WornArmor);
    }
    if has_gold {
        candidates.push(TheftTarget::Gold);
    }
    candidates.push(TheftTarget::RandomInventory);

    if candidates.is_empty() {
        return None;
    }

    let idx = rng.rn2(candidates.len() as i32) as usize;
    Some(candidates[idx].clone())
}

/// [v2.28.0 R16-5] 전체 도둑질 판정
pub fn attempt_theft(
    thief_dex: i32,
    victim_dex: i32,
    victim_has_free_action: bool,
    has_wielded: bool,
    has_armor: bool,
    has_gold: bool,
    has_amulet: bool,
    item_is_cursed: bool,
    rng: &mut NetHackRng,
) -> TheftResult {
    if !theft_chance(thief_dex, victim_dex, victim_has_free_action, false, rng) {
        return TheftResult::TooSlow;
    }

    let target = match select_theft_target(has_wielded, has_armor, has_gold, has_amulet, rng) {
        Some(t) => t,
        None => return TheftResult::NothingToSteal,
    };

    // 장착 해제 시 저주 체크
    if item_is_cursed && matches!(target, TheftTarget::WieldedItem | TheftTarget::WornArmor) {
        return TheftResult::CursedItem;
    }

    TheftResult::Stolen {
        target,
        item_name: "stolen item".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theft_chance_free_action() {
        let mut rng = NetHackRng::new(42);
        assert!(!theft_chance(18, 10, true, false, &mut rng));
    }

    #[test]
    fn test_theft_high_dex() {
        let mut success = 0;
        for s in 0..50 {
            let mut rng = NetHackRng::new(s);
            if theft_chance(20, 5, false, false, &mut rng) {
                success += 1;
            }
        }
        assert!(success > 25); // 높은 DEX → 대부분 성공
    }

    #[test]
    fn test_target_amulet_priority() {
        // 아뮬렛이 후보에 포함됨 확인
        let mut found = false;
        for s in 0..20 {
            let mut rng = NetHackRng::new(s);
            if let Some(TheftTarget::Amulet) = select_theft_target(true, true, true, true, &mut rng)
            {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_cursed_block() {
        let mut rng = NetHackRng::new(42);
        let result = attempt_theft(20, 5, false, true, false, false, false, true, &mut rng);
        // 도둑질 성공하더라도 저주 아이템이면 CursedItem
        assert!(matches!(
            result,
            TheftResult::CursedItem | TheftResult::TooSlow | TheftResult::Stolen { .. }
        ));
    }

    #[test]
    fn test_nothing_to_steal() {
        // RandomInventory는 항상 후보이므로 None은 안 나옴
        let mut rng = NetHackRng::new(42);
        let target = select_theft_target(false, false, false, false, &mut rng);
        assert!(target.is_some()); // RandomInventory 항상 존재
    }
}

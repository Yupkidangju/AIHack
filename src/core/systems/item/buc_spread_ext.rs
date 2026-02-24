// ============================================================================
// [v2.34.0 R22-1] BUC 전파 (buc_spread_ext.rs)
// 원본: NetHack 3.6.7 pray.c/mkobj.c BUC 전파
// 성수/저주물 전파, 컨테이너 BUC, 축복 확률
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.34.0 R22-1] BUC 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Buc {
    Blessed,
    Uncursed,
    Cursed,
}

/// [v2.34.0 R22-1] 성수/저주물 딥 전파
pub fn water_dip(item_buc: Buc, water_buc: Buc) -> Buc {
    match water_buc {
        Buc::Blessed => match item_buc {
            Buc::Cursed => Buc::Uncursed,
            _ => Buc::Blessed,
        },
        Buc::Cursed => match item_buc {
            Buc::Blessed => Buc::Uncursed,
            _ => Buc::Cursed,
        },
        Buc::Uncursed => item_buc,
    }
}

/// [v2.34.0 R22-1] 컨테이너 BUC 전파 (bag of holding → 저주 시 파괴)
pub fn container_curse_risk(container_buc: Buc, item_count: i32) -> bool {
    matches!(container_buc, Buc::Cursed) && item_count > 0
}

/// [v2.34.0 R22-1] 축복 확률 (기도, 봉헌)
pub fn bless_chance(piety: i32, rng: &mut NetHackRng) -> bool {
    let chance = (piety * 5).clamp(5, 95);
    rng.rn2(100) < chance
}

/// [v2.34.0 R22-1] 컨테이너 내 아이템 BUC 일괄 전환
pub fn spread_buc_in_container(items: &mut Vec<Buc>, target: Buc) {
    for item in items.iter_mut() {
        *item = target;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_holy_water() {
        assert_eq!(water_dip(Buc::Cursed, Buc::Blessed), Buc::Uncursed);
        assert_eq!(water_dip(Buc::Uncursed, Buc::Blessed), Buc::Blessed);
    }

    #[test]
    fn test_unholy_water() {
        assert_eq!(water_dip(Buc::Blessed, Buc::Cursed), Buc::Uncursed);
        assert_eq!(water_dip(Buc::Uncursed, Buc::Cursed), Buc::Cursed);
    }

    #[test]
    fn test_container_risk() {
        assert!(container_curse_risk(Buc::Cursed, 5));
        assert!(!container_curse_risk(Buc::Blessed, 5));
    }

    #[test]
    fn test_bless_chance() {
        let mut success = 0;
        for s in 0..100 {
            let mut rng = NetHackRng::new(s);
            if bless_chance(15, &mut rng) {
                success += 1;
            }
        }
        assert!(success > 50);
    }

    #[test]
    fn test_spread() {
        let mut items = vec![Buc::Uncursed, Buc::Cursed, Buc::Blessed];
        spread_buc_in_container(&mut items, Buc::Blessed);
        assert!(items.iter().all(|b| *b == Buc::Blessed));
    }
}

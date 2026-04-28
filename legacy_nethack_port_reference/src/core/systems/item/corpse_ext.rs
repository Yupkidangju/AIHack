// ============================================================================
// [v2.31.0 R19-1] 시체 효과 (corpse_ext.rs)
// 원본: NetHack 3.6.7 eat.c cprefx/cpostfx (시체 섭취 효과)
// 시체별 특수 효과, 독성, 텔레파시, 저항 획득
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.31.0 R19-1] 시체 섭취 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpseEffect {
    /// 독 (HP 손실)
    Poison { damage: i32, fatal: bool },
    /// 텔레파시 획득
    Telepathy,
    /// 투명 감지 획득
    SeeInvisible,
    /// 독 저항 획득
    PoisonResistance,
    /// 냉기 저항 획득
    ColdResistance,
    /// 화염 저항 획득
    FireResistance,
    /// 수면 저항 획득
    SleepResistance,
    /// 전기 저항 획득
    ShockResistance,
    /// 순간이동 능력 획득 (텔레포트itis)
    Teleportitis,
    /// 순간이동 제어 획득
    TeleportControl,
    /// 석화 (코카트리스)
    Petrification,
    /// 슬라임화
    Sliming,
    /// 산 데미지
    AcidDamage(i32),
    /// 중독 (질병)
    Sickness,
    /// 환각
    Hallucination(i32),
    /// 효과 없음
    None,
}

/// [v2.31.0 R19-1] 몬스터 종류별 시체 효과 (원본: cpostfx)
pub fn corpse_effect(monster_class: char, monster_name: &str) -> Vec<CorpseEffect> {
    match monster_class {
        'e' => vec![CorpseEffect::Telepathy],         // floating eye
        'F' => vec![CorpseEffect::Hallucination(50)], // fungi
        'c' => {
            // cockatrice
            if monster_name.contains("cockatrice") {
                vec![CorpseEffect::Petrification]
            } else {
                vec![CorpseEffect::None]
            }
        }
        'P' => vec![CorpseEffect::Sliming], // green slime
        'a' => vec![CorpseEffect::PoisonResistance], // ants (일부)
        'D' => {
            // dragons
            if monster_name.contains("red") {
                vec![CorpseEffect::FireResistance]
            } else if monster_name.contains("white") {
                vec![CorpseEffect::ColdResistance]
            } else if monster_name.contains("blue") {
                vec![CorpseEffect::ShockResistance]
            } else {
                vec![CorpseEffect::None]
            }
        }
        'N' => vec![CorpseEffect::Teleportitis],    // nymph
        't' => vec![CorpseEffect::Teleportitis],    // tengu
        'L' => vec![CorpseEffect::TeleportControl], // leprechaun
        _ => vec![CorpseEffect::None],
    }
}

/// [v2.31.0 R19-1] 시체 부패 여부 (턴 기반)
pub fn is_rotten(age_turns: u64, blessed: bool) -> bool {
    let threshold = if blessed { 100 } else { 50 };
    age_turns > threshold
}

/// [v2.31.0 R19-1] 부패 시체 섭취 위험
pub fn rotten_effect(rng: &mut NetHackRng) -> CorpseEffect {
    let roll = rng.rn2(4);
    match roll {
        0 => CorpseEffect::Sickness,
        1 => CorpseEffect::Poison {
            damage: rng.rn1(10, 1),
            fatal: false,
        },
        2 => CorpseEffect::Hallucination(20),
        _ => CorpseEffect::None, // 운 좋음
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_eye() {
        let effects = corpse_effect('e', "floating eye");
        assert!(effects.contains(&CorpseEffect::Telepathy));
    }

    #[test]
    fn test_cockatrice() {
        let effects = corpse_effect('c', "cockatrice");
        assert!(effects.contains(&CorpseEffect::Petrification));
    }

    #[test]
    fn test_red_dragon() {
        let effects = corpse_effect('D', "red dragon");
        assert!(effects.contains(&CorpseEffect::FireResistance));
    }

    #[test]
    fn test_rotten() {
        assert!(!is_rotten(30, false));
        assert!(is_rotten(60, false));
        assert!(!is_rotten(60, true)); // 축복이면 100턴 까지
    }

    #[test]
    fn test_rotten_effect() {
        let mut rng = NetHackRng::new(42);
        let effect = rotten_effect(&mut rng);
        assert!(matches!(
            effect,
            CorpseEffect::Sickness
                | CorpseEffect::Poison { .. }
                | CorpseEffect::Hallucination(_)
                | CorpseEffect::None
        ));
    }
}

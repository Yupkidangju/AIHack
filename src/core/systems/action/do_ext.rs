// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-16] 행동/드랍 확장 모듈 (do_ext.rs)
// 원본: NetHack 3.6.7 do.c (드랍 가능 판정, 싱크 반지 효과 식별, 제단 BUC)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 드랍 가능 판정 (원본: do.c:561-598 canletgo)
// =============================================================================

/// [v2.22.0 R34-16] 드랍 불가 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropBlockReason {
    /// 착용 중 (갑옷/장신구)
    Wearing,
    /// 저주받은 로드스톤
    CursedLoadstone { quantity: i64 },
    /// 목줄로 연결됨
    LeashTied,
    /// 안장에 앉아있음
    SittingOnSaddle,
}

/// [v2.22.0 R34-16] 드랍 가능 판정 입력
#[derive(Debug, Clone)]
pub struct DropCheckInput {
    /// 착용 중인 갑옷/장신구인지 (owornmask & (W_ARMOR | W_ACCESSORY))
    pub is_worn_armor_or_accessory: bool,
    /// 로드스톤인지
    pub is_loadstone: bool,
    /// 저주받았는지
    pub is_cursed: bool,
    /// 목줄이 몬스터에 연결되어 있는지
    pub leash_attached: bool,
    /// 안장인지 (owornmask & W_SADDLE)
    pub is_saddle_worn: bool,
    /// 수량 (로드스톤 메시지용)
    pub quantity: i64,
}

/// [v2.22.0 R34-16] 드랍 가능 판정 (원본: canletgo)
pub fn check_can_drop(input: &DropCheckInput) -> Result<(), DropBlockReason> {
    if input.is_worn_armor_or_accessory {
        return Err(DropBlockReason::Wearing);
    }
    if input.is_loadstone && input.is_cursed {
        return Err(DropBlockReason::CursedLoadstone {
            quantity: input.quantity,
        });
    }
    if input.leash_attached {
        return Err(DropBlockReason::LeashTied);
    }
    if input.is_saddle_worn {
        return Err(DropBlockReason::SittingOnSaddle);
    }
    Ok(())
}

// =============================================================================
// [2] 제단 BUC 판정 (원본: do.c:269-296 doaltarobj)
// =============================================================================

/// [v2.22.0 R34-16] 제단에 아이템을 올렸을 때 BUC 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AltarResult {
    /// 호박색 빛 (축복)
    AmberFlash,
    /// 검은색 빛 (저주)
    BlackFlash,
    /// 빛 없음 (BUC 중립 → 그냥 놓임)
    NoFlash,
    /// 금화 (BUC 없음)
    Coin,
}

/// [v2.22.0 R34-16] 제단 BUC 판정 (원본: doaltarobj)
/// `is_blind`: 플레이어가 장님인지
pub fn check_altar_buc(
    is_coin: bool,
    is_blessed: bool,
    is_cursed: bool,
    is_blind: bool,
) -> AltarResult {
    if is_blind {
        // 장님이면 아무 것도 모름
        return AltarResult::NoFlash;
    }
    if is_coin {
        return AltarResult::Coin;
    }
    if is_blessed {
        return AltarResult::AmberFlash;
    }
    if is_cursed {
        return AltarResult::BlackFlash;
    }
    AltarResult::NoFlash
}

// =============================================================================
// [3] 싱크 반지 효과 식별 (원본: do.c:395-559 dosinkring의 switch)
// =============================================================================

/// [v2.22.0 R34-16] 반지 유형 (싱크 효과용)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingType {
    Searching,
    SlowDigestion,
    Levitation,
    PoisonResistance,
    AggravateMonster,
    ShockResistance,
    Conflict,
    SustainAbility,
    GainStrength,
    GainConstitution,
    IncreaseAccuracy,
    IncreaseDamage,
    Hunger,
    TeleportControl,
    Teleportation,
    Polymorph,
    PolymorphControl,
    Adornment,
    Regeneration,
    Invisibility,
    FreeAction,
    SeeInvisible,
    Stealth,
    FireResistance,
    ColdResistance,
    ProtectionFromShapeChange,
    Protection,
    Warning,
    MeatRing,
    Other,
}

/// [v2.22.0 R34-16] 싱크 반지 효과 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkRingEffect {
    /// 반지가 되돌아옴 (Searching, SlowDigestion)
    ReturnRing,
    /// 효과 식별됨 (눈 필요 없음)
    IdentifiedBlind { description: &'static str },
    /// 효과 식별됨 (눈 필요)
    IdentifiedSight { description: &'static str },
    /// 특수 효과 (텔레포트/변신)
    SpecialEffect { description: &'static str },
    /// 알 수 없음
    Unidentified,
}

/// [v2.22.0 R34-16] 싱크 반지 효과 판정 (원본: dosinkring의 switch)
pub fn calc_sink_ring_effect(ring: RingType, spe: i8, is_blind: bool) -> SinkRingEffect {
    match ring {
        RingType::Searching => SinkRingEffect::ReturnRing,
        RingType::SlowDigestion => SinkRingEffect::ReturnRing,
        RingType::Levitation => SinkRingEffect::IdentifiedBlind {
            description: "The sink quivers upward for a moment.",
        },
        RingType::PoisonResistance => SinkRingEffect::IdentifiedBlind {
            description: "You smell rotten fruit.",
        },
        RingType::AggravateMonster => SinkRingEffect::IdentifiedBlind {
            description: "Several flies buzz angrily around the sink.",
        },
        RingType::ShockResistance => SinkRingEffect::IdentifiedBlind {
            description: "Static electricity surrounds the sink.",
        },
        RingType::Conflict => SinkRingEffect::IdentifiedBlind {
            description: "You hear loud noises coming from the drain.",
        },
        RingType::SustainAbility => SinkRingEffect::IdentifiedBlind {
            description: "The water flow seems fixed.",
        },
        RingType::GainStrength => SinkRingEffect::IdentifiedBlind {
            description: if spe < 0 {
                "The water flow seems weaker now."
            } else {
                "The water flow seems stronger now."
            },
        },
        RingType::GainConstitution => SinkRingEffect::IdentifiedBlind {
            description: if spe < 0 {
                "The water flow seems lesser now."
            } else {
                "The water flow seems greater now."
            },
        },
        RingType::IncreaseAccuracy => SinkRingEffect::IdentifiedBlind {
            description: if spe < 0 {
                "The water flow misses the drain."
            } else {
                "The water flow hits the drain."
            },
        },
        RingType::IncreaseDamage => SinkRingEffect::IdentifiedBlind {
            description: if spe < 0 {
                "The water's force seems smaller now."
            } else {
                "The water's force seems greater now."
            },
        },
        RingType::Teleportation => SinkRingEffect::SpecialEffect {
            description: "The sink vanishes.",
        },
        RingType::Polymorph => SinkRingEffect::SpecialEffect {
            description: "The sink transforms!",
        },
        RingType::MeatRing => SinkRingEffect::IdentifiedBlind {
            description: "Several flies buzz around the sink.",
        },
        // 눈이 필요한 효과
        RingType::Adornment if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "The faucets flash brightly for a moment.",
        },
        RingType::Regeneration if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "The sink looks as good as new.",
        },
        RingType::Invisibility if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "You don't see anything happen to the sink.",
        },
        RingType::FreeAction if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "You see the ring slide right down the drain!",
        },
        RingType::SeeInvisible if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "You see some air in the sink.",
        },
        RingType::Stealth if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "The sink seems to blend into the floor for a moment.",
        },
        RingType::FireResistance if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "The hot water faucet flashes brightly for a moment.",
        },
        RingType::ColdResistance if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "The cold water faucet flashes brightly for a moment.",
        },
        RingType::ProtectionFromShapeChange if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "The sink looks nothing like a fountain.",
        },
        RingType::Protection if !is_blind => SinkRingEffect::IdentifiedSight {
            description: if spe < 0 {
                "The sink glows black for a moment."
            } else {
                "The sink glows silver for a moment."
            },
        },
        RingType::Warning if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "The sink glows white for a moment.",
        },
        RingType::TeleportControl if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "The sink looks like it is being beamed aboard somewhere.",
        },
        RingType::PolymorphControl if !is_blind => SinkRingEffect::IdentifiedSight {
            description: "The sink momentarily looks like a regularly erupting geyser.",
        },
        _ => SinkRingEffect::Unidentified,
    }
}

// =============================================================================
// [4] 크리스나이프 퇴화 (원본: do.c:763-794 obj_no_longer_held)
// =============================================================================

/// [v2.22.0 R34-16] 크리스나이프→웜 이빨 퇴화 판정
/// `is_crysknife`: 크리스나이프인지
/// `is_erodeproof`: 고정 크리스나이프인지
pub fn should_crysknife_revert(
    is_crysknife: bool,
    is_erodeproof: bool,
    rng: &mut NetHackRng,
) -> bool {
    if !is_crysknife {
        return false;
    }
    // 일반: 항상 퇴화. 고정: 1/10 확률
    !is_erodeproof || rng.rn2(10) == 0
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::rng::NetHackRng;

    #[test]
    fn test_can_drop_normal() {
        let input = DropCheckInput {
            is_worn_armor_or_accessory: false,
            is_loadstone: false,
            is_cursed: false,
            leash_attached: false,
            is_saddle_worn: false,
            quantity: 1,
        };
        assert!(check_can_drop(&input).is_ok());
    }

    #[test]
    fn test_cannot_drop_wearing() {
        let input = DropCheckInput {
            is_worn_armor_or_accessory: true,
            is_loadstone: false,
            is_cursed: false,
            leash_attached: false,
            is_saddle_worn: false,
            quantity: 1,
        };
        assert_eq!(check_can_drop(&input), Err(DropBlockReason::Wearing));
    }

    #[test]
    fn test_cannot_drop_cursed_loadstone() {
        let input = DropCheckInput {
            is_worn_armor_or_accessory: false,
            is_loadstone: true,
            is_cursed: true,
            leash_attached: false,
            is_saddle_worn: false,
            quantity: 3,
        };
        assert_eq!(
            check_can_drop(&input),
            Err(DropBlockReason::CursedLoadstone { quantity: 3 })
        );
    }

    #[test]
    fn test_altar_blessed() {
        assert_eq!(
            check_altar_buc(false, true, false, false),
            AltarResult::AmberFlash
        );
    }

    #[test]
    fn test_altar_cursed() {
        assert_eq!(
            check_altar_buc(false, false, true, false),
            AltarResult::BlackFlash
        );
    }

    #[test]
    fn test_altar_blind() {
        assert_eq!(
            check_altar_buc(false, true, false, true),
            AltarResult::NoFlash
        );
    }

    #[test]
    fn test_altar_coin() {
        assert_eq!(
            check_altar_buc(true, false, false, false),
            AltarResult::Coin
        );
    }

    #[test]
    fn test_sink_ring_searching() {
        let effect = calc_sink_ring_effect(RingType::Searching, 0, false);
        assert_eq!(effect, SinkRingEffect::ReturnRing);
    }

    #[test]
    fn test_sink_ring_levitation() {
        let effect = calc_sink_ring_effect(RingType::Levitation, 0, false);
        matches!(effect, SinkRingEffect::IdentifiedBlind { .. });
    }

    #[test]
    fn test_sink_ring_blind_blocks_sight_effect() {
        let effect = calc_sink_ring_effect(RingType::Adornment, 0, true);
        assert_eq!(effect, SinkRingEffect::Unidentified);
    }

    #[test]
    fn test_crysknife_normal_reverts() {
        let mut rng = NetHackRng::new(42);
        assert!(should_crysknife_revert(true, false, &mut rng));
    }

    #[test]
    fn test_crysknife_fixed_sometimes() {
        let mut rng = NetHackRng::new(42);
        let mut reverted = false;
        for _ in 0..100 {
            if should_crysknife_revert(true, true, &mut rng) {
                reverted = true;
                break;
            }
        }
        assert!(reverted); // 1/10 확률, 100번이면 거의 확실
    }
}

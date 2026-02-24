// ============================================================================
// [v2.36.0 R24-3] 장신구 효과 (accessory_ext.rs)
// 원본: NetHack 3.6.7 worn.c/do_wear.c 반지+아뮬렛
// 착용 효과, 해제 효과, 저주 제약
// ============================================================================

/// [v2.36.0 R24-3] 장신구 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessoryType {
    // 반지
    RingAdornment,
    RingProtection,
    RingFreeAction,
    RingTeleportControl,
    RingRegeneration,
    RingSeeInvisible,
    RingStealth,
    RingConflict,
    RingLevitation,
    RingPolyControl,
    RingSustainAbility,
    // 아뮬렛
    AmuletESP,
    AmuletLifeSaving,
    AmuletReflection,
    AmuletMagicBreathing,
    AmuletUnchanging,
    AmuletStrangulation,
    AmuletRestfulSleep,
    AmuletYendor,
}

/// [v2.36.0 R24-3] 장신구 착용 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WearEffect {
    GainProperty(String),
    GainProtection(i32),
    HPRegen(i32),
    Danger(String), // 목졸림 등
    QuestItem,      // 아뮬렛 of Yendor
    None,
}

pub fn wear_accessory(acc: AccessoryType) -> WearEffect {
    match acc {
        AccessoryType::RingProtection => WearEffect::GainProtection(1),
        AccessoryType::RingFreeAction => WearEffect::GainProperty("free action".into()),
        AccessoryType::RingTeleportControl => WearEffect::GainProperty("teleport control".into()),
        AccessoryType::RingRegeneration => WearEffect::HPRegen(1),
        AccessoryType::RingSeeInvisible => WearEffect::GainProperty("see invisible".into()),
        AccessoryType::RingStealth => WearEffect::GainProperty("stealth".into()),
        AccessoryType::RingConflict => WearEffect::GainProperty("conflict".into()),
        AccessoryType::RingLevitation => WearEffect::GainProperty("levitation".into()),
        AccessoryType::RingPolyControl => WearEffect::GainProperty("polymorph control".into()),
        AccessoryType::RingSustainAbility => WearEffect::GainProperty("sustain ability".into()),
        AccessoryType::AmuletESP => WearEffect::GainProperty("telepathy".into()),
        AccessoryType::AmuletLifeSaving => WearEffect::GainProperty("life saving".into()),
        AccessoryType::AmuletReflection => WearEffect::GainProperty("reflection".into()),
        AccessoryType::AmuletMagicBreathing => WearEffect::GainProperty("magical breathing".into()),
        AccessoryType::AmuletUnchanging => WearEffect::GainProperty("unchanging".into()),
        AccessoryType::AmuletStrangulation => WearEffect::Danger("목이 조여온다!".into()),
        AccessoryType::AmuletRestfulSleep => WearEffect::Danger("졸음이 쏟아진다...".into()),
        AccessoryType::AmuletYendor => WearEffect::QuestItem,
        AccessoryType::RingAdornment => WearEffect::None,
    }
}

/// [v2.36.0 R24-3] 저주된 장신구 해제 불가
pub fn can_remove(is_cursed: bool) -> bool {
    !is_cursed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protection_ring() {
        assert_eq!(
            wear_accessory(AccessoryType::RingProtection),
            WearEffect::GainProtection(1)
        );
    }

    #[test]
    fn test_esp() {
        assert_eq!(
            wear_accessory(AccessoryType::AmuletESP),
            WearEffect::GainProperty("telepathy".into())
        );
    }

    #[test]
    fn test_strangulation() {
        assert!(matches!(
            wear_accessory(AccessoryType::AmuletStrangulation),
            WearEffect::Danger(_)
        ));
    }

    #[test]
    fn test_yendor() {
        assert_eq!(
            wear_accessory(AccessoryType::AmuletYendor),
            WearEffect::QuestItem
        );
    }

    #[test]
    fn test_cursed_remove() {
        assert!(!can_remove(true));
        assert!(can_remove(false));
    }
}

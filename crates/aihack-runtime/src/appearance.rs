use aihack_core::domain::item::ItemKind;

const POTION_APPEARANCES: [&str; 10] = [
    "crimson potion",
    "azure potion",
    "amber potion",
    "jade potion",
    "silver potion",
    "violet potion",
    "emerald potion",
    "indigo potion",
    "scarlet potion",
    "smoky potion",
];

const SCROLL_APPEARANCES: [&str; 10] = [
    "scroll labeled ARDEN VEX",
    "scroll labeled KORIM SOL",
    "scroll labeled THALEN RUX",
    "scroll labeled MIRAE DOV",
    "scroll labeled SELVAN KYT",
    "scroll labeled ORIN ZEL",
    "scroll labeled VEXA NOR",
    "scroll labeled CALEN TIR",
    "scroll labeled DURAL FEN",
    "scroll labeled NYRA KOS",
];

const RING_APPEARANCES: [&str; 6] = [
    "copper ring",
    "onyx ring",
    "ivory ring",
    "opal ring",
    "bronze ring",
    "granite ring",
];

const AMULET_APPEARANCES: [&str; 4] = [
    "circular amulet",
    "triangular amulet",
    "oval amulet",
    "hexagonal amulet",
];

const POTION_KINDS: [ItemKind; 9] = [
    ItemKind::PotionHealing,
    ItemKind::PotionExtraHealing,
    ItemKind::PotionCure,
    ItemKind::PotionSpeed,
    ItemKind::PotionBlindness,
    ItemKind::PotionConfusion,
    ItemKind::PotionHallucination,
    ItemKind::PotionInvisibility,
    ItemKind::PotionLevitation,
];

const SCROLL_KINDS: [ItemKind; 7] = [
    ItemKind::ScrollReveal,
    ItemKind::ScrollIdentify,
    ItemKind::ScrollLevelTeleport,
    ItemKind::ScrollCharging,
    ItemKind::ScrollEnchantWeapon,
    ItemKind::ScrollEnchantArmor,
    ItemKind::ScrollRemoveCurse,
];

const RING_KINDS: [ItemKind; 3] = [
    ItemKind::RingPoisonResistance,
    ItemKind::RingFireResistance,
    ItemKind::RingProtection,
];

const AMULET_KINDS: [ItemKind; 1] = [ItemKind::AmuletEsp];

/// 시드 기반 독립 SplitMix64 셔플로 미식별 포션/두루마리/반지/아뮬렛 외형을 결정한다.
pub fn appearance_for_kind(seed: u64, kind: ItemKind) -> Option<&'static str> {
    if let Some(index) = POTION_KINDS.iter().position(|candidate| *candidate == kind) {
        let mut pool = POTION_APPEARANCES;
        shuffle(seed ^ 0x517c_c1b7_2722_0a95, &mut pool);
        return Some(pool[index]);
    }
    if let Some(index) = SCROLL_KINDS.iter().position(|candidate| *candidate == kind) {
        let mut pool = SCROLL_APPEARANCES;
        shuffle(seed ^ 0x94d0_49bb_1331_11eb, &mut pool);
        return Some(pool[index]);
    }
    if let Some(index) = RING_KINDS.iter().position(|candidate| *candidate == kind) {
        let mut pool = RING_APPEARANCES;
        shuffle(seed ^ 0xa076_1d64_78bd_642f, &mut pool);
        return Some(pool[index]);
    }
    if let Some(index) = AMULET_KINDS.iter().position(|candidate| *candidate == kind) {
        let mut pool = AMULET_APPEARANCES;
        shuffle(seed ^ 0xe703_7ed1_a0b4_28db, &mut pool);
        return Some(pool[index]);
    }
    None
}

fn shuffle<T>(seed: u64, slice: &mut [T]) {
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    for i in (1..slice.len()).rev() {
        let pick = (splitmix64(&mut state) % (i as u64 + 1)) as usize;
        slice.swap(i, pick);
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

use aihack_core::{domain::item::ItemKind, ids::EntityId};

const POTION_APPEARANCES: &[&str] = &[
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
    "golden potion",
    "effervescent potion",
    "murky potion",
    "swirling potion",
];

const SCROLL_APPEARANCES: &[&str] = &[
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
    "scroll labeled VERAN TUL",
    "scroll labeled ZELOR QIN",
];

const RING_APPEARANCES: &[&str] = &[
    "copper ring",
    "onyx ring",
    "ivory ring",
    "opal ring",
    "bronze ring",
    "granite ring",
    "silver ring",
    "gold ring",
    "sapphire ring",
    "ruby ring",
    "jade ring",
    "twisted ring",
    "wooden ring",
    "pearl ring",
];

const AMULET_APPEARANCES: &[&str] = &[
    "circular amulet",
    "triangular amulet",
    "oval amulet",
    "hexagonal amulet",
    "square amulet",
    "octagonal amulet",
    "pyramidal amulet",
    "concave amulet",
];

const WAND_APPEARANCES: &[&str] = &[
    "glass wand",
    "oak wand",
    "ebony wand",
    "brass wand",
    "copper wand",
    "silver wand",
    "runed wand",
    "maple wand",
    "marble wand",
    "crystal wand",
    "iron wand",
    "tin wand",
    "platinum wand",
    "bone wand",
    "bamboo wand",
    "hexagonal wand",
    "short wand",
    "jeweled wand",
];

const SPELLBOOK_APPEARANCES: &[&str] = &[
    "parchment spellbook",
    "vellum spellbook",
    "rag-paper spellbook",
    "dog-eared spellbook",
    "indigo spellbook",
    "crimson spellbook",
    "bronze spellbook",
    "azure spellbook",
    "jade spellbook",
    "violet spellbook",
    "silver-bound spellbook",
    "gilded spellbook",
    "stained spellbook",
    "mottled spellbook",
    "thick spellbook",
    "thin spellbook",
    "wide spellbook",
    "big spellbook",
    "small spellbook",
    "glossy spellbook",
    "dull spellbook",
    "tattered spellbook",
    "obsidian spellbook",
    "ash-gray spellbook",
    "scarlet spellbook",
    "emerald spellbook",
    "amber spellbook",
    "cyan spellbook",
    "ochre spellbook",
    "clothbound spellbook",
];

fn pool_appearance(
    seed: u64,
    salt: u64,
    pool_name: &str,
    kind: ItemKind,
    appearances: &[&'static str],
) -> Option<&'static str> {
    let mut index = 0usize;
    let mut found = None;
    for &candidate in ItemKind::ALL {
        if aihack_content::item_unidentified_pool(candidate) == Some(pool_name) {
            if candidate == kind {
                found = Some(index);
                break;
            }
            index += 1;
        }
    }
    let idx = found?;
    let mut pool = appearances.to_vec();
    shuffle(seed ^ salt, &mut pool);
    pool.get(idx).copied()
}

/// 시드 기반 독립 SplitMix64 셔플로 미식별 아이템 외형을 결정한다.
pub fn appearance_for_kind(seed: u64, kind: ItemKind) -> Option<&'static str> {
    let pool_name = aihack_content::item_unidentified_pool(kind)?;
    match pool_name {
        "potion" => pool_appearance(
            seed,
            0x517c_c1b7_2722_0a95,
            "potion",
            kind,
            POTION_APPEARANCES,
        ),
        "scroll" => pool_appearance(
            seed,
            0x94d0_49bb_1331_11eb,
            "scroll",
            kind,
            SCROLL_APPEARANCES,
        ),
        "ring" => pool_appearance(seed, 0xa076_1d64_78bd_642f, "ring", kind, RING_APPEARANCES),
        "amulet" => pool_appearance(
            seed,
            0xe703_7ed1_a0b4_28db,
            "amulet",
            kind,
            AMULET_APPEARANCES,
        ),
        "wand" => pool_appearance(seed, 0x369d_ea0f_31a5_3f85, "wand", kind, WAND_APPEARANCES),
        "spellbook" => pool_appearance(
            seed,
            0x7b93_1244_88c5_e211,
            "spellbook",
            kind,
            SPELLBOOK_APPEARANCES,
        ),
        _ => None,
    }
}

/// 시드 기반 외형 문자열에만 의존하는 표시 색상이다(실제 ItemKind 색상을 노출하지 않는다).
pub fn color_for_appearance(appearance: &str) -> &'static str {
    match appearance {
        "crimson potion" | "crimson spellbook" | "ruby ring" => "red",
        "azure potion" | "azure spellbook" | "sapphire ring" => "blue",
        "amber potion" | "amber spellbook" | "golden potion" | "gold ring" | "bronze ring"
        | "bronze spellbook" | "ochre spellbook" => "yellow",
        "jade potion" | "jade ring" | "jade spellbook" => "green",
        "silver potion"
        | "ivory ring"
        | "silver ring"
        | "pearl ring"
        | "silver wand"
        | "silver-bound spellbook"
        | "platinum wand"
        | "bone wand"
        | "marble wand" => "white",
        "violet potion" | "violet spellbook" => "magenta",
        "emerald potion" | "emerald spellbook" => "bright_green",
        "indigo potion" | "indigo spellbook" | "crystal wand" => "bright_blue",
        "scarlet potion" | "scarlet spellbook" => "bright_red",
        "smoky potion" | "granite ring" | "iron wand" | "tin wand" | "ash-gray spellbook"
        | "dull spellbook" => "gray",
        "copper ring"
        | "copper wand"
        | "oak wand"
        | "maple wand"
        | "wooden ring"
        | "bamboo wand"
        | "parchment spellbook"
        | "vellum spellbook"
        | "tattered spellbook" => "brown",
        "onyx ring" | "ebony wand" | "obsidian spellbook" => "black",
        "opal ring" | "cyan spellbook" | "glass wand" => "cyan",
        "effervescent potion" | "swirling potion" | "runed wand" => "bright_cyan",
        "gilded spellbook" | "circular amulet" | "triangular amulet" | "oval amulet"
        | "hexagonal amulet" | "square amulet" | "octagonal amulet" | "pyramidal amulet"
        | "concave amulet" | "brass wand" | "jeweled wand" => "bright_yellow",
        _ => "white",
    }
}

pub fn display_name_for_kind(seed: u64, world: &crate::world::GameWorld, kind: ItemKind) -> String {
    if world.campaign.is_some() && !world.is_item_identified(kind) {
        if let Some(app) = appearance_for_kind(seed, kind) {
            return app.to_owned();
        }
    }
    aihack_content::item_display_name(kind, world.content_registry())
}

pub fn display_name_for_world_item(
    seed: u64,
    world: &crate::world::GameWorld,
    item: EntityId,
    kind: ItemKind,
) -> String {
    if let Some(def) = crate::systems::artifacts::artifact_def_for_item(world, item) {
        return def.name.to_owned();
    }
    if let Some(mk) = world
        .entities
        .get(item)
        .and_then(|entity| entity.item_corpse_monster())
    {
        return format!(
            "{} corpse",
            aihack_content::monster_display_name(mk, world.content_registry())
        );
    }
    display_name_for_kind(seed, world, kind)
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

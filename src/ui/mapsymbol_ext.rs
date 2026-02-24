// ============================================================================
// [v2.34.0 R22-3] 맵 심볼 (mapsymbol_ext.rs)
// 원본: NetHack 3.6.7 drawing.c/display.c 심볼 매핑
// 타일→문자, 몬스터 심볼, 아이템 심볼
// ============================================================================

/// [v2.34.0 R22-3] 지형 심볼
pub fn terrain_symbol(terrain: &str) -> char {
    match terrain {
        "floor" => '.',
        "corridor" => '#',
        "wall_h" => '-',
        "wall_v" => '|',
        "door_open" => '|',
        "door_closed" => '+',
        "stairs_up" => '<',
        "stairs_down" => '>',
        "fountain" => '{',
        "sink" => '#',
        "altar" => '_',
        "throne" => '\\',
        "grave" => '|',
        "tree" => '#',
        "water" => '}',
        "lava" => '}',
        "ice" => '.',
        "air" => ' ',
        "cloud" => '#',
        _ => '?',
    }
}

/// [v2.34.0 R22-3] 아이템 클래스 심볼 (원본: def_oc_syms)
pub fn item_class_symbol(oclass: &str) -> char {
    match oclass {
        "weapon" => ')',
        "armor" => '[',
        "ring" => '=',
        "amulet" => '"',
        "tool" => '(',
        "food" => '%',
        "potion" => '!',
        "scroll" => '?',
        "spellbook" => '+',
        "wand" => '/',
        "coin" => '$',
        "gem" => '*',
        "rock" => '`',
        "ball" => '0',
        "chain" => '_',
        "venom" => '.',
        _ => '?',
    }
}

/// [v2.34.0 R22-3] 몬스터 클래스 (원본: def_monsyms)
pub fn monster_class_symbol(mclass: char) -> &'static str {
    match mclass {
        'a' => "ant/insect",
        'b' => "blob/jelly",
        'c' => "cockatrice",
        'd' => "dog/canine",
        'e' => "floating eye",
        'f' => "feline",
        'h' => "humanoid",
        'k' => "kobold",
        'o' => "orc",
        'A' => "angel",
        'D' => "dragon",
        'G' => "gnome",
        'H' => "giant",
        'L' => "lich",
        'N' => "naga",
        'T' => "troll",
        'V' => "vampire",
        'Z' => "zombie",
        '@' => "human",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain() {
        assert_eq!(terrain_symbol("floor"), '.');
        assert_eq!(terrain_symbol("stairs_up"), '<');
    }

    #[test]
    fn test_item() {
        assert_eq!(item_class_symbol("weapon"), ')');
        assert_eq!(item_class_symbol("potion"), '!');
    }

    #[test]
    fn test_monster() {
        assert_eq!(monster_class_symbol('D'), "dragon");
        assert_eq!(monster_class_symbol('@'), "human");
    }

    #[test]
    fn test_unknown() {
        assert_eq!(terrain_symbol("xyz"), '?');
        assert_eq!(monster_class_symbol('~'), "unknown");
    }
}

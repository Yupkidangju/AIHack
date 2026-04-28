// ============================================================================
// [v2.37.0 R25-4] 던전 특성 (dungeon_feature_ext.rs)
// 원본: NetHack 3.6.7 dungeon.c/mklev.c 특성 확장
// 방 유형, 특수 타일, 분기 속성
// ============================================================================

/// [v2.37.0 R25-4] 방 유형 (원본: mkroom.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    Ordinary,
    Shop,
    Vault,
    Morgue,
    Barracks,
    Zoo,
    Beehive,
    Swamp,
    Temple,
    Cocknest,
    Anthole,
    Leprehall,
}

/// [v2.37.0 R25-4] 방 몬스터 밀도
pub fn room_monster_density(room: RoomType) -> i32 {
    match room {
        RoomType::Ordinary => 1,
        RoomType::Shop => 0,
        RoomType::Vault => 0,
        RoomType::Morgue => 5,
        RoomType::Barracks => 8,
        RoomType::Zoo => 6,
        RoomType::Beehive => 10,
        RoomType::Swamp => 3,
        RoomType::Temple => 1,
        RoomType::Cocknest => 4,
        RoomType::Anthole => 7,
        RoomType::Leprehall => 5,
    }
}

/// [v2.37.0 R25-4] 방 아이템 밀도
pub fn room_item_density(room: RoomType) -> i32 {
    match room {
        RoomType::Shop => 15,
        RoomType::Vault => 10,
        RoomType::Leprehall => 8,
        RoomType::Morgue => 3,
        RoomType::Zoo => 5,
        _ => 1,
    }
}

/// [v2.37.0 R25-4] 분기 속성
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub min_depth: i32,
    pub max_depth: i32,
    pub is_hell: bool,
    pub no_bones: bool,
}

pub fn branches() -> Vec<BranchInfo> {
    vec![
        BranchInfo {
            name: "Dungeons of Doom".into(),
            min_depth: 1,
            max_depth: 25,
            is_hell: false,
            no_bones: false,
        },
        BranchInfo {
            name: "Gnomish Mines".into(),
            min_depth: 2,
            max_depth: 13,
            is_hell: false,
            no_bones: false,
        },
        BranchInfo {
            name: "Sokoban".into(),
            min_depth: 6,
            max_depth: 10,
            is_hell: false,
            no_bones: true,
        },
        BranchInfo {
            name: "Quest".into(),
            min_depth: 1,
            max_depth: 5,
            is_hell: false,
            no_bones: true,
        },
        BranchInfo {
            name: "Gehennom".into(),
            min_depth: 26,
            max_depth: 45,
            is_hell: true,
            no_bones: false,
        },
        BranchInfo {
            name: "Vlad's Tower".into(),
            min_depth: 1,
            max_depth: 3,
            is_hell: false,
            no_bones: true,
        },
        BranchInfo {
            name: "Wizard's Tower".into(),
            min_depth: 1,
            max_depth: 3,
            is_hell: true,
            no_bones: true,
        },
        BranchInfo {
            name: "Elemental Planes".into(),
            min_depth: 1,
            max_depth: 4,
            is_hell: false,
            no_bones: true,
        },
        BranchInfo {
            name: "Astral Plane".into(),
            min_depth: 0,
            max_depth: 0,
            is_hell: false,
            no_bones: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_density() {
        assert_eq!(room_monster_density(RoomType::Beehive), 10);
        assert_eq!(room_monster_density(RoomType::Shop), 0);
    }

    #[test]
    fn test_item_density() {
        assert_eq!(room_item_density(RoomType::Shop), 15);
    }

    #[test]
    fn test_branches() {
        assert_eq!(branches().len(), 9);
    }

    #[test]
    fn test_gehennom() {
        let g = branches()
            .into_iter()
            .find(|b| b.name == "Gehennom")
            .unwrap();
        assert!(g.is_hell);
    }

    #[test]
    fn test_sokoban_no_bones() {
        let s = branches()
            .into_iter()
            .find(|b| b.name == "Sokoban")
            .unwrap();
        assert!(s.no_bones);
    }
}

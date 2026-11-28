use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        map::{GameMap, MapLayout, PHASE2_STAIRS_DOWN, PHASE5_LEVEL2_STAIRS_UP},
        tile::TileKind,
    },
    error::ContentError,
    ids::{BranchId, LevelId},
    position::Pos,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialRoomKind {
    Shop,
    Temple,
    Vault,
    Barracks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngravingKind {
    Dust,
    Burned,
    Engraved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Engraving {
    pub pos: Pos,
    pub text: String,
    pub kind: EngravingKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BranchEntryKind {
    StairsUp,
    StairsDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BranchDef {
    pub id: BranchId,
    pub max_depth: i16,
    pub parent: Option<(LevelId, BranchEntryKind)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BranchGraph {
    pub branches: &'static [BranchDef],
}

impl BranchGraph {
    pub const fn campaign_v3() -> Self {
        const BRANCHES: &[BranchDef] = &[
            BranchDef {
                id: BranchId::Main,
                max_depth: 6,
                parent: None,
            },
            BranchDef {
                id: BranchId::Mines,
                max_depth: 2,
                parent: Some((LevelId::main(3), BranchEntryKind::StairsUp)),
            },
            BranchDef {
                id: BranchId::Sokoban,
                max_depth: 4,
                parent: Some((LevelId::main(4), BranchEntryKind::StairsDown)),
            },
            BranchDef {
                id: BranchId::Quest,
                max_depth: 3,
                parent: Some((LevelId::main(5), BranchEntryKind::StairsDown)),
            },
            BranchDef {
                id: BranchId::Gehennom,
                max_depth: 4,
                parent: Some((LevelId::main(6), BranchEntryKind::StairsDown)),
            },
            BranchDef {
                id: BranchId::Planes,
                max_depth: 5,
                parent: None,
            },
        ];
        Self { branches: BRANCHES }
    }

    pub fn branch_def(self, id: BranchId) -> Option<BranchDef> {
        self.branches.iter().copied().find(|b| b.id == id)
    }

    pub fn active_level_ids(self) -> Vec<LevelId> {
        let mut out = Vec::new();
        for branch in self.branches {
            for depth in 1..=branch.max_depth {
                out.push(LevelId::new(branch.id, depth));
            }
        }
        out
    }

    pub fn has_branch_entry(self, from: LevelId, entry_kind: BranchEntryKind) -> bool {
        self.branches.iter().any(|branch| {
            branch.max_depth > 0
                && branch
                    .parent
                    .is_some_and(|(parent, kind)| parent == from && kind == entry_kind)
        })
    }

    pub fn branch_entry_from(self, from: LevelId, tile: TileKind) -> Option<LevelId> {
        self.branches.iter().find_map(|branch| {
            if branch.max_depth <= 0 {
                return None;
            }
            let (parent_level, entry_kind) = branch.parent?;
            let matches_tile = matches!(
                (entry_kind, tile),
                (BranchEntryKind::StairsUp, TileKind::StairsUp)
                    | (BranchEntryKind::StairsDown, TileKind::StairsDown)
            );
            (parent_level == from && matches_tile).then(|| LevelId::new(branch.id, 1))
        })
    }

    pub fn portal_target(self, from: LevelId) -> Option<LevelId> {
        if from.branch != BranchId::Planes {
            return None;
        }
        let planes = self.branch_def(BranchId::Planes)?;
        (from.depth >= 1 && from.depth < planes.max_depth)
            .then(|| LevelId::new(BranchId::Planes, from.depth + 1))
    }

    pub fn descend_target(self, from: LevelId) -> Option<LevelId> {
        if from.branch == BranchId::Planes {
            return None;
        }
        let branch = self.branch_def(from.branch)?;
        (from.depth < branch.max_depth).then(|| LevelId::new(from.branch, from.depth + 1))
    }

    pub fn ascend_target(self, from: LevelId) -> Option<LevelId> {
        if from.branch == BranchId::Planes {
            return None;
        }
        let branch = self.branch_def(from.branch)?;
        if from.depth > 1 {
            Some(LevelId::new(from.branch, from.depth - 1))
        } else {
            branch.parent.map(|(parent_level, _)| parent_level)
        }
    }
}

/// level map과 식별자를 함께 보관한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameLevel {
    pub id: LevelId,
    pub map: GameMap,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub explored: Vec<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lit_tiles: Vec<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special_room: Option<SpecialRoomKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub engravings: Vec<Engraving>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub altar_alignments: Vec<(Pos, crate::campaign::Alignment)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special_level_id: Option<String>,
}

impl GameLevel {
    pub fn altar_alignment(&self, pos: Pos) -> Option<crate::campaign::Alignment> {
        self.altar_alignments
            .iter()
            .find_map(|(at, alignment)| (*at == pos).then_some(*alignment))
    }

    pub fn set_altar_alignment(&mut self, pos: Pos, alignment: crate::campaign::Alignment) {
        if let Some((_, existing)) = self.altar_alignments.iter_mut().find(|(at, _)| *at == pos) {
            *existing = alignment;
        } else {
            self.altar_alignments.push((pos, alignment));
        }
    }

    pub fn is_lit(&self, pos: Pos) -> bool {
        if !self.map.contains(pos) {
            return false;
        }
        if self.lit_tiles.len() != self.map.tile_count() {
            return true;
        }
        let idx = (pos.y as usize * self.map.width as usize) + pos.x as usize;
        self.lit_tiles.get(idx).copied().unwrap_or(true)
    }

    pub fn is_explored(&self, pos: Pos) -> bool {
        if !self.map.contains(pos) || self.explored.is_empty() {
            return false;
        }
        let idx = (pos.y as usize * self.map.width as usize) + pos.x as usize;
        self.explored.get(idx).copied().unwrap_or(false)
    }

    pub fn mark_explored(&mut self, positions: &[Pos]) {
        if positions.is_empty() {
            return;
        }
        let expected = self.map.tile_count();
        if self.explored.len() != expected {
            self.explored.resize(expected, false);
        }
        for &pos in positions {
            if self.map.contains(pos) {
                let idx = (pos.y as usize * self.map.width as usize) + pos.x as usize;
                self.explored[idx] = true;
            }
        }
    }

    pub fn explored_positions(&self) -> Vec<Pos> {
        let expected = self.map.tile_count();
        if self.explored.len() != expected {
            return Vec::new();
        }
        let mut out = Vec::new();
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let idx = (y as usize * self.map.width as usize) + x as usize;
                if self.explored[idx] {
                    out.push(Pos { x, y });
                }
            }
        }
        out
    }
}

/// level을 deterministic 순서로 보관한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelRegistry {
    pub levels: Vec<GameLevel>,
}

impl LevelRegistry {
    pub fn fixture_phase5() -> Self {
        Self {
            levels: vec![
                GameLevel {
                    id: PHASE5_LEVEL1_ID,
                    map: GameMap::fixture_phase2(),
                    explored: Vec::new(),
                    lit_tiles: Vec::new(),
                    special_room: None,
                    engravings: Vec::new(),
                    altar_alignments: Vec::new(),
                    special_level_id: None,
                },
                GameLevel {
                    id: PHASE5_LEVEL2_ID,
                    map: GameMap::fixture_phase5_level2(),
                    explored: Vec::new(),
                    lit_tiles: Vec::new(),
                    special_room: None,
                    engravings: Vec::new(),
                    altar_alignments: Vec::new(),
                    special_level_id: None,
                },
            ],
        }
    }

    pub fn from_layouts<'a, L: MapLayout + 'a>(
        layouts: impl Iterator<Item = &'a L>,
    ) -> Result<Self, ContentError> {
        let mut levels = layouts
            .map(|definition| {
                Ok(GameLevel {
                    id: LevelId::main(definition.depth()),
                    map: GameMap::from_level_data(definition)?,
                    explored: Vec::new(),
                    lit_tiles: Vec::new(),
                    special_room: None,
                    engravings: Vec::new(),
                    altar_alignments: Vec::new(),
                    special_level_id: None,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        levels.sort_by_key(|level| level.id);
        Ok(Self { levels })
    }

    pub fn len(&self) -> usize {
        self.levels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    pub fn get(&self, id: LevelId) -> Option<&GameLevel> {
        self.levels.iter().find(|level| level.id == id)
    }

    pub fn get_mut(&mut self, id: LevelId) -> Option<&mut GameLevel> {
        self.levels.iter_mut().find(|level| level.id == id)
    }

    pub fn map(&self, id: LevelId) -> Option<&GameMap> {
        self.get(id).map(|level| &level.map)
    }

    pub fn map_mut(&mut self, id: LevelId) -> Option<&mut GameMap> {
        self.get_mut(id).map(|level| &mut level.map)
    }

    pub fn contains(&self, id: LevelId) -> bool {
        self.get(id).is_some()
    }

    pub fn stairs_up_pos(&self, id: LevelId) -> Option<Pos> {
        self.find_tile(id, TileKind::StairsUp)
    }

    pub fn stairs_down_pos(&self, id: LevelId) -> Option<Pos> {
        self.find_tile(id, TileKind::StairsDown)
    }

    fn find_tile(&self, id: LevelId, needle: TileKind) -> Option<Pos> {
        let map = self.map(id)?;
        for y in 0..map.height {
            for x in 0..map.width {
                let pos = Pos { x, y };
                if map.tile(pos).ok()? == needle {
                    return Some(pos);
                }
            }
        }
        None
    }
}

pub const PHASE5_LEVEL1_ID: LevelId = LevelId::main(1);
pub const PHASE5_LEVEL2_ID: LevelId = LevelId::main(2);
pub const PHASE5_LEVEL1_STAIRS_DOWN: Pos = PHASE2_STAIRS_DOWN;
pub const PHASE5_LEVEL2_STAIRS_UP_POS: Pos = PHASE5_LEVEL2_STAIRS_UP;

use aihack_core::{
    domain::{
        level::GameLevel,
        map::{GameMap, MapLayout},
        tile::TileKind,
    },
    error::ContentError,
    ids::{BranchId, LevelId},
    position::Pos,
};

pub(crate) fn level_ids() -> Vec<LevelId> {
    (1..=6)
        .map(LevelId::main)
        .chain((1..=2).map(|depth| LevelId {
            branch: BranchId::Mines,
            depth,
        }))
        .collect()
}

struct Layout(Vec<(Pos, TileKind)>);
impl MapLayout for Layout {
    fn level_id(&self) -> &str {
        "campaign generated"
    }
    fn depth(&self) -> i16 {
        1
    }
    fn dimensions(&self) -> (i16, i16) {
        (40, 20)
    }
    fn tile_overrides(&self) -> Result<Vec<(Pos, TileKind)>, ContentError> {
        Ok(self.0.clone())
    }
}

/// Combat RNG와 독립인 generator v1. 각 방을 L자 통로로 연결한다.
pub(crate) fn generate(seed: u64, id: LevelId) -> Result<(GameLevel, [Pos; 4]), ContentError> {
    let mut state = seed
        ^ (id.depth as u64).wrapping_mul(0x9e3779b97f4a7c15)
        ^ if id.branch == BranchId::Mines {
            0xa0761d6478bd642f
        } else {
            0
        };
    let mut centers = [Pos { x: 0, y: 0 }; 4];
    for (center, x) in centers.iter_mut().zip([4, 14, 25, 35]) {
        state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        *center = Pos {
            x,
            y: 4 + ((z ^ (z >> 31)) % 12) as i16,
        };
    }
    let mut overrides = Vec::new();
    for y in 0..20 {
        for x in 0..40 {
            overrides.push((Pos { x, y }, TileKind::Wall));
        }
    }
    for center in centers {
        for y in center.y - 2..=center.y + 2 {
            for x in center.x - 2..=center.x + 2 {
                overrides.push((Pos { x, y }, TileKind::Floor));
            }
        }
    }
    for pair in centers.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        for x in a.x..=b.x {
            overrides.push((Pos { x, y: a.y }, TileKind::Floor));
        }
        for y in a.y.min(b.y)..=a.y.max(b.y) {
            overrides.push((Pos { x: b.x, y }, TileKind::Floor));
        }
    }
    overrides.push((centers[0], TileKind::StairsUp));
    if (id.branch == BranchId::Main && id.depth < 6)
        || (id.branch == BranchId::Mines && id.depth < 2)
    {
        overrides.push((centers[3], TileKind::StairsDown));
    }
    Ok((
        GameLevel {
            id,
            map: GameMap::from_level_data(&Layout(overrides))?,
        },
        centers,
    ))
}

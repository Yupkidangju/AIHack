use crate::{domain::map::GameMap, event::GameEvent, position::Pos};

pub fn reveal_tile(map: &mut GameMap, pos: Pos) -> Option<GameEvent> {
    let current = map.tile(pos).ok()?;
    if !current.is_hidden() {
        return None;
    }
    let revealed = current.revealed_equivalent();
    map.set_tile(pos, revealed).ok()?;
    Some(GameEvent::TileRevealed {
        pos,
        tile: revealed,
    })
}

pub fn reveal_all_hidden_tiles(map: &mut GameMap) -> Vec<GameEvent> {
    let (width, height) = (map.width, map.height);
    let mut events = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if let Some(event) = reveal_tile(map, Pos { x, y }) {
                events.push(event);
            }
        }
    }
    events
}

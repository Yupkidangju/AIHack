use crate::{
    core::{event::GameEvent, position::Pos, world::GameWorld},
    domain::tile::TileKind,
};

pub const PHASE7_PIT_DAMAGE: i16 = 3;

/// [v0.1.0] Phase 7 search는 주변 hidden tile을 deterministic하게 reveal한다.
pub fn search(world: &mut GameWorld) -> Vec<GameEvent> {
    let origin = world.player_pos();
    let mut events = Vec::new();
    for pos in adjacent_and_current(origin) {
        if let Some(event) = reveal_tile(world, pos) {
            events.push(event);
        }
    }
    events
}

/// [v0.1.0] hidden tile을 revealed counterpart로 바꾸고 event를 반환한다.
pub fn reveal_tile(world: &mut GameWorld, pos: Pos) -> Option<GameEvent> {
    let current = world.current_map().tile(pos).ok()?;
    if !current.is_hidden() {
        return None;
    }
    let revealed = current.revealed_equivalent();
    world.current_map_mut().set_tile(pos, revealed).ok()?;
    Some(GameEvent::TileRevealed {
        pos,
        tile: revealed,
    })
}

/// [v0.1.0] ScrollReveal은 current level의 모든 hidden tile을 reveal한다.
pub fn reveal_all_hidden_tiles(world: &mut GameWorld) -> Vec<GameEvent> {
    let map = world.current_map().clone();
    let mut events = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            if let Some(event) = reveal_tile(world, Pos { x, y }) {
                events.push(event);
            }
        }
    }
    events
}

/// [v0.1.0] movement entry 직후 player trap trigger를 검사한다.
pub fn trigger_player_trap(world: &mut GameWorld) -> Vec<GameEvent> {
    let pos = world.player_pos();
    let Some(tile) = world.current_map().tile(pos).ok() else {
        return Vec::new();
    };
    let trap = match tile {
        TileKind::Trap(kind) | TileKind::HiddenTrap(kind) => kind,
        _ => return Vec::new(),
    };

    let mut events = Vec::new();
    if matches!(tile, TileKind::HiddenTrap(_)) {
        if let Some(event) = reveal_tile(world, pos) {
            events.push(event);
        }
    }
    if let Some(stats) = world.entities.actor_stats_mut(world.player_id) {
        stats.hp -= PHASE7_PIT_DAMAGE;
    }
    events.push(GameEvent::TrapTriggered {
        entity: world.player_id,
        trap,
        pos,
        damage: PHASE7_PIT_DAMAGE,
    });
    events
}

fn adjacent_and_current(origin: Pos) -> [Pos; 9] {
    [
        Pos {
            x: origin.x - 1,
            y: origin.y - 1,
        },
        Pos {
            x: origin.x,
            y: origin.y - 1,
        },
        Pos {
            x: origin.x + 1,
            y: origin.y - 1,
        },
        Pos {
            x: origin.x - 1,
            y: origin.y,
        },
        origin,
        Pos {
            x: origin.x + 1,
            y: origin.y,
        },
        Pos {
            x: origin.x - 1,
            y: origin.y + 1,
        },
        Pos {
            x: origin.x,
            y: origin.y + 1,
        },
        Pos {
            x: origin.x + 1,
            y: origin.y + 1,
        },
    ]
}

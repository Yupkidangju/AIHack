use aihack_core::{domain::tile::TileKind, event::GameEvent, position::Pos};

use crate::world::GameWorld;

pub const PHASE7_PIT_DAMAGE: i16 = 3;

/// 주변의 숨겨진 타일을 결정적으로 공개한다.
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

/// 숨겨진 타일을 공개 상태로 바꾸고 이벤트를 반환한다.
pub fn reveal_tile(world: &mut GameWorld, pos: Pos) -> Option<GameEvent> {
    aihack_core::traps::reveal_tile(world.current_map_mut(), pos)
}

/// 현재 레벨의 모든 숨겨진 타일을 공개한다.
pub fn reveal_all_hidden_tiles(world: &mut GameWorld) -> Vec<GameEvent> {
    aihack_core::traps::reveal_all_hidden_tiles(world.current_map_mut())
}

/// 이동 직후 플레이어가 밟은 함정을 발동한다.
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
    let player_id = world.player_id;
    if let Some(stats) = world.entities.actor_stats_mut(player_id) {
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

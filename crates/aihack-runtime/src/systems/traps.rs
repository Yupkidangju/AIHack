use aihack_core::{
    domain::tile::{TileKind, TrapKind},
    event::GameEvent,
    position::Pos,
};

use crate::world::GameWorld;

pub const PHASE7_PIT_DAMAGE: i16 = 3;
pub const DART_TRAP_DAMAGE: i16 = 2;
pub const BEAR_TRAP_DAMAGE: i16 = 2;
/// 턴 종료 시 paralysis_turns가 1 감소하므로 다음 1턴 완전 마비를 위해 2로 시작한다.
pub const BEAR_TRAP_PARALYSIS_TURNS: u8 = 2;

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

/// 이동 직후 플레이어가 밟은 함정을 발동하고, 발동한 함정 종류를 함께 반환한다.
pub(crate) fn trigger_player_trap_with_kind(
    world: &mut GameWorld,
) -> (Vec<GameEvent>, Option<TrapKind>) {
    let pos = world.player_pos();
    let Some(tile) = world.current_map().tile(pos).ok() else {
        return (Vec::new(), None);
    };
    let trap = match tile {
        TileKind::Trap(kind) | TileKind::HiddenTrap(kind) => kind,
        _ => return (Vec::new(), None),
    };

    let mut events = Vec::new();
    if matches!(tile, TileKind::HiddenTrap(_)) {
        if let Some(event) = reveal_tile(world, pos) {
            events.push(event);
        }
    }
    let (damage, paralysis) = match trap {
        TrapKind::Pit => (PHASE7_PIT_DAMAGE, 0),
        TrapKind::Dart => (DART_TRAP_DAMAGE, 0),
        TrapKind::Bear => (BEAR_TRAP_DAMAGE, BEAR_TRAP_PARALYSIS_TURNS),
    };
    let player_id = world.player_id;
    if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
        stats.hp = stats.hp.saturating_sub(damage);
    }
    if paralysis > 0 {
        world.state_mut().paralysis_turns = world.paralysis_turns.max(paralysis);
    }
    events.push(GameEvent::TrapTriggered {
        entity: world.player_id,
        trap,
        pos,
        damage,
    });
    (events, Some(trap))
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

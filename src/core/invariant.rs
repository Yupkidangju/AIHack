use crate::{
    core::{
        ids::{EntityId, LevelId},
        position::Pos,
        world::GameWorld,
    },
    domain::entity::EntityKind,
};

pub const WORLD_INVARIANT_COUNT: u8 = 6;

/// 저장 데이터와 turn commit 전에 확인하는 GameWorld의 복구 불가능한 관계 오류다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldInvariantError {
    CurrentLevelMissing {
        level: LevelId,
    },
    PlayerMissing {
        player: EntityId,
    },
    PlayerIsNotPlayer {
        player: EntityId,
    },
    PlayerLevelMismatch {
        current_level: LevelId,
        player_level: LevelId,
    },
    PlayerOutOfBounds {
        level: LevelId,
        pos: Pos,
    },
    InventoryOwnerMismatch {
        player: EntityId,
        owner: EntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantReport {
    pub checked: u8,
    pub errors: Vec<WorldInvariantError>,
}

impl InvariantReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn validate(world: &GameWorld) -> InvariantReport {
    let mut errors = Vec::new();
    let current_level = world.current_level();
    let Some(map) = world.levels.map(current_level) else {
        errors.push(WorldInvariantError::CurrentLevelMissing {
            level: current_level,
        });
        return report(errors);
    };

    let player = world.player_id;
    let Some(entity) = world.entities.get(player) else {
        errors.push(WorldInvariantError::PlayerMissing { player });
        return report(errors);
    };
    if !matches!(entity.kind(), EntityKind::Player) {
        errors.push(WorldInvariantError::PlayerIsNotPlayer { player });
        return report(errors);
    }

    let Some((player_level, player_pos)) = world.entities.actor_location(player) else {
        errors.push(WorldInvariantError::PlayerMissing { player });
        return report(errors);
    };
    if player_level != current_level {
        errors.push(WorldInvariantError::PlayerLevelMismatch {
            current_level,
            player_level,
        });
        return report(errors);
    }
    if !map.contains(player_pos) {
        errors.push(WorldInvariantError::PlayerOutOfBounds {
            level: current_level,
            pos: player_pos,
        });
        return report(errors);
    }
    if world.inventory.owner != player {
        errors.push(WorldInvariantError::InventoryOwnerMismatch {
            player,
            owner: world.inventory.owner,
        });
    }

    report(errors)
}

fn report(errors: Vec<WorldInvariantError>) -> InvariantReport {
    InvariantReport {
        checked: WORLD_INVARIANT_COUNT,
        errors,
    }
}

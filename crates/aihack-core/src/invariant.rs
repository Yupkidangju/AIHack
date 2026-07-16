use crate::{
    domain::entity::EntityKind,
    ids::{EntityId, LevelId},
    position::Pos,
};

pub const WORLD_INVARIANT_COUNT: u8 = 6;

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

/// World 구현이 제공해야 하는 읽기 전용 관계만 고정한다.
pub trait WorldInvariantView {
    fn current_level_id(&self) -> LevelId;
    fn level_exists(&self, level: LevelId) -> bool;
    fn contains_position(&self, level: LevelId, pos: Pos) -> bool;
    fn player_entity_id(&self) -> EntityId;
    fn entity_kind(&self, entity: EntityId) -> Option<EntityKind>;
    fn actor_location(&self, entity: EntityId) -> Option<(LevelId, Pos)>;
    fn inventory_owner(&self) -> EntityId;
}

pub fn validate_world(world: &impl WorldInvariantView) -> InvariantReport {
    let mut errors = Vec::new();
    let current_level = world.current_level_id();
    let player = world.player_entity_id();

    if !world.level_exists(current_level) {
        return report(vec![WorldInvariantError::CurrentLevelMissing {
            level: current_level,
        }]);
    }

    let Some(kind) = world.entity_kind(player) else {
        return report(vec![WorldInvariantError::PlayerMissing { player }]);
    };
    if kind != EntityKind::Player {
        return report(vec![WorldInvariantError::PlayerIsNotPlayer { player }]);
    }
    let Some((player_level, player_pos)) = world.actor_location(player) else {
        return report(vec![WorldInvariantError::PlayerMissing { player }]);
    };
    if player_level != current_level {
        errors.push(WorldInvariantError::PlayerLevelMismatch {
            current_level,
            player_level,
        });
        return report(errors);
    }
    if !world.contains_position(current_level, player_pos) {
        errors.push(WorldInvariantError::PlayerOutOfBounds {
            level: current_level,
            pos: player_pos,
        });
        return report(errors);
    }
    let owner = world.inventory_owner();
    if owner != player {
        errors.push(WorldInvariantError::InventoryOwnerMismatch { player, owner });
    }
    report(errors)
}

fn report(errors: Vec<WorldInvariantError>) -> InvariantReport {
    InvariantReport {
        checked: WORLD_INVARIANT_COUNT,
        errors,
    }
}

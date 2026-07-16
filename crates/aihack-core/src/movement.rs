use crate::{
    domain::map::GameMap,
    error::GameError,
    ids::{EntityId, LevelId},
    position::{Direction, Pos},
};

/// 이동 legality가 요구하는 world 조회 경계다.
///
/// core는 저장·UI adapter를 알지 못하며, map·actor 위치·살아있는 actor 점유만 읽는다.
pub trait MovementWorld {
    fn map(&self, level: LevelId) -> &GameMap;
    fn actor_location(&self, actor: EntityId) -> Option<(LevelId, Pos)>;
    fn alive_actor_at(&self, level: LevelId, pos: Pos) -> Option<EntityId>;
}

/// 점유 상태와 무관한 map 목적지 통과 가능성을 판정한다.
pub fn require_passable(map: &GameMap, pos: Pos) -> Result<(), GameError> {
    let tile = map.tile(pos)?;
    if tile.is_movement_passable() {
        Ok(())
    } else {
        Err(GameError::BlockedMovement { pos, tile })
    }
}

/// actor의 map 통과·점유·대각선 corner-cut 규칙을 함께 검증한다.
pub fn validate_actor_destination(
    world: &impl MovementWorld,
    actor: EntityId,
    direction: Direction,
) -> Result<(), GameError> {
    let (level, from) = world.actor_location(actor).ok_or_else(|| {
        GameError::CommandRejected(format!("actor {actor:?} has no map position"))
    })?;
    let to = from.offset(direction.delta());
    validate_destination_on_level(world, actor, level, from, to, direction)
}

/// 이미 계산된 목적지를 검증할 때 쓰는 level-aware 경계다.
pub fn validate_destination_on_level(
    world: &impl MovementWorld,
    actor: EntityId,
    level: LevelId,
    from: Pos,
    to: Pos,
    direction: Direction,
) -> Result<(), GameError> {
    require_passable(world.map(level), to)?;
    require_unoccupied(world, actor, level, to)?;
    validate_path(world, actor, level, from, direction)
}

/// 목적지 공격이 허용되는 action도 공유할 수 있는 대각선 경로 검사다.
pub fn validate_path(
    world: &impl MovementWorld,
    actor: EntityId,
    level: LevelId,
    from: Pos,
    direction: Direction,
) -> Result<(), GameError> {
    if let Some((a, b)) = direction.orthogonal_components() {
        for component in [a, b] {
            let pos = from.offset(component.delta());
            require_passable(world.map(level), pos)?;
            require_unoccupied(world, actor, level, pos)?;
        }
    }
    Ok(())
}

/// action-space projection에서 사용하는 non-error 형태의 동일한 이동 legality다.
pub fn is_passable_for_actor(
    world: &impl MovementWorld,
    actor: EntityId,
    direction: Direction,
) -> bool {
    validate_actor_destination(world, actor, direction).is_ok()
}

fn require_unoccupied(
    world: &impl MovementWorld,
    actor: EntityId,
    level: LevelId,
    pos: Pos,
) -> Result<(), GameError> {
    if world
        .alive_actor_at(level, pos)
        .is_some_and(|occupant| occupant != actor)
    {
        return Err(GameError::CommandRejected(format!(
            "movement blocked by living entity at {pos:?}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_actor_destination, MovementWorld};
    use crate::{
        domain::{map::GameMap, tile::TileKind},
        ids::{EntityId, LevelId},
        position::{Direction, Pos},
    };

    struct TestWorld {
        map: GameMap,
        actor: EntityId,
        location: (LevelId, Pos),
        occupied: Option<(LevelId, Pos, EntityId)>,
    }

    impl MovementWorld for TestWorld {
        fn map(&self, _level: LevelId) -> &GameMap {
            &self.map
        }

        fn actor_location(&self, actor: EntityId) -> Option<(LevelId, Pos)> {
            (actor == self.actor).then_some(self.location)
        }

        fn alive_actor_at(&self, level: LevelId, pos: Pos) -> Option<EntityId> {
            self.occupied
                .filter(|(occupied_level, occupied_pos, _)| {
                    *occupied_level == level && *occupied_pos == pos
                })
                .map(|(_, _, actor)| actor)
        }
    }

    #[test]
    fn diagonal_move_rejects_a_blocked_intermediate_tile() {
        let mut map = GameMap::fixture_phase2();
        map.set_tile(Pos { x: 5, y: 4 }, TileKind::Wall).unwrap();
        let world = TestWorld {
            map,
            actor: EntityId(1),
            location: (LevelId::main(1), Pos { x: 5, y: 5 }),
            occupied: None,
        };

        assert!(validate_actor_destination(&world, EntityId(1), Direction::NorthEast).is_err());
    }

    #[test]
    fn diagonal_move_rejects_an_occupied_intermediate_tile() {
        let world = TestWorld {
            map: GameMap::fixture_phase2(),
            actor: EntityId(1),
            location: (LevelId::main(1), Pos { x: 5, y: 5 }),
            occupied: Some((LevelId::main(1), Pos { x: 6, y: 5 }, EntityId(2))),
        };

        assert!(validate_actor_destination(&world, EntityId(1), Direction::SouthEast).is_err());
    }
}

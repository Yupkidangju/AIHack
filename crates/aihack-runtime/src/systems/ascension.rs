use aihack_core::{
    domain::monster::MonsterKind,
    event::{GameEvent, MessagePriority},
    ids::{BranchId, LevelId},
    position::Pos,
};

use crate::{domain::monster::try_monster_template_from_registry, world::GameWorld};

pub fn on_level_entered(world: &mut GameWorld, to: LevelId) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() || !crate::campaign::has_amulet(world) {
        return Ok(Vec::new());
    }

    let (target_stage, monster_kind, announcement) =
        match (to.branch, to.depth, world.ascension_pursuit_stage) {
            (BranchId::Main, 4, stage) if stage < 1 => (
                1,
                MonsterKind::FireAnt,
                "The dungeon senses the Amulet! A pursuing fire ant appears!",
            ),
            (BranchId::Main, 2, stage) if stage < 2 => (
                2,
                MonsterKind::Troll,
                "The dungeon senses the Amulet! A pursuing troll appears!",
            ),
            (BranchId::Main, 1, stage) if stage < 3 => (
                3,
                MonsterKind::Minotaur,
                "The dungeon senses the Amulet! A pursuing minotaur appears!",
            ),
            (BranchId::Planes, 2, stage) if stage < 4 => (
                4,
                MonsterKind::WizardOfYendor,
                "The Wizard of Yendor materializes: 'The Amulet belongs to me!'",
            ),
            (BranchId::Planes, 4, stage) if stage < 5 => (
                5,
                MonsterKind::WizardOfYendor,
                "The Wizard of Yendor returns in fury across the planes!",
            ),
            _ => return Ok(Vec::new()),
        };

    let spawn_pos = find_pursuit_spawn_pos(world, to)
        .ok_or_else(|| "no valid spawn position for ascension pursuer".to_string())?;
    let template = try_monster_template_from_registry(monster_kind, world.content_registry())
        .map_err(|error| error.to_string())?;
    let monster = world
        .state_mut()
        .entities
        .spawn_monster_with_template(monster_kind, template, spawn_pos)
        .map_err(|error| error.to_string())?;
    world
        .state_mut()
        .entities
        .set_actor_location(monster, to, spawn_pos);
    world.state_mut().ascension_pursuit_stage = target_stage;

    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Warning,
        text: announcement.to_string(),
    }])
}

fn find_pursuit_spawn_pos(world: &GameWorld, level: LevelId) -> Option<Pos> {
    let map = world.map(level);
    let player_pos = world.player_pos();
    let mut best: Option<((bool, i16, i16, i16), Pos)> = None;
    for y in 0..map.height {
        for x in 0..map.width {
            let pos = Pos { x, y };
            if pos == player_pos {
                continue;
            }
            let Ok(tile) = map.tile(pos) else {
                continue;
            };
            if !tile.is_movement_passable()
                || matches!(
                    tile,
                    aihack_core::domain::tile::TileKind::Water
                        | aihack_core::domain::tile::TileKind::Lava
                )
                || world.entities.alive_actor_at(level, pos).is_some()
            {
                continue;
            }
            let too_close = pos.chebyshev_distance(player_pos) < 2;
            let key = (too_close, (pos.x - (map.width / 2)).abs(), pos.y, pos.x);
            if best.is_none_or(|(best_key, _)| key < best_key) {
                best = Some((key, pos));
            }
        }
    }
    best.map(|(_, pos)| pos)
}

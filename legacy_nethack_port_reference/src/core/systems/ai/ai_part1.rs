use crate::assets::AssetManager;
use crate::core::dungeon::Grid;
use crate::core::entity::status::StatusFlags;
use crate::core::entity::{
    monster::Faction, monster::MonsterState, status::Status, status::StatusBundle, CombatStats,
    Health, Level, Monster, MonsterTag, PlayerTag, Position, Swallowed,
};
use crate::core::systems::ai_helper::{AiHelper, MoveFlags};
use crate::core::systems::combat::CombatEngine;
use crate::core::systems::vision::VisionSystem;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::{EntityStore, SubWorld};
use legion::*;
use std::collections::{HashMap, HashSet};

///
#[system]
#[read_component(MonsterTag)]
#[write_component(crate::core::entity::monster::Pet)]
pub fn pet_hunger(
    world: &mut SubWorld,
    #[resource] rng: &mut NetHackRng,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
) {
    let mut query = <(&mut crate::core::entity::monster::Pet, &MonsterTag)>::query();
    for (pet, _) in query.iter_mut(world) {
        if *turn % 10 == 0 {
            pet.hunger += 1;
            if pet.hunger > 1000 && rng.rn2(20) == 0 {
                log.add("Your pet whines.", *turn);
            }
        }
    }
}

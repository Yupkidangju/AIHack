use aihack_core::{
    domain::{
        entity::EntityLocation,
        item::ItemKind,
        level::BranchGraph,
        status::Resistances,
        tile::{TileKind, TrapKind},
    },
    event::{GameEvent, MessagePriority},
    position::Pos,
    rng::GameRng,
};

use crate::{
    systems::{combat::roll_die, items},
    world::GameWorld,
};

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
    rng: &mut GameRng,
) -> Result<(Vec<GameEvent>, Option<TrapKind>), String> {
    let pos = world.player_pos();
    let Some(tile) = world.current_map().tile(pos).ok() else {
        return Ok((Vec::new(), None));
    };
    let trap = match tile {
        TileKind::Trap(kind) | TileKind::HiddenTrap(kind) => kind,
        _ => return Ok((Vec::new(), None)),
    };

    let mut events = Vec::new();
    if matches!(tile, TileKind::HiddenTrap(_)) {
        if let Some(event) = reveal_tile(world, pos) {
            events.push(event);
        }
    }
    if world.is_levitating() && matches!(trap, TrapKind::Pit | TrapKind::Bear | TrapKind::Hole) {
        events.push(GameEvent::Message {
            priority: MessagePriority::Info,
            text: "You float over the trap.".to_string(),
        });
        return Ok((events, Some(trap)));
    }
    let player_id = world.player_id;
    let dex_reduction = world
        .campaign
        .map(|c| c.attributes.trap_damage_reduction())
        .unwrap_or(0);
    let mut damage = 0_i16;
    match trap {
        TrapKind::Pit => {
            damage = (PHASE7_PIT_DAMAGE - dex_reduction).max(1);
            apply_trap_damage(world, player_id, damage);
        }
        TrapKind::Dart => {
            damage = (DART_TRAP_DAMAGE - dex_reduction).max(1);
            apply_trap_damage(world, player_id, damage);
        }
        TrapKind::Bear => {
            damage = (BEAR_TRAP_DAMAGE - dex_reduction).max(1);
            apply_trap_damage(world, player_id, damage);
            world.state_mut().paralysis_turns =
                world.paralysis_turns.max(BEAR_TRAP_PARALYSIS_TURNS);
        }
        TrapKind::Hole => {
            damage = (roll_die(rng, 6) - dex_reduction).max(1);
            apply_trap_damage(world, player_id, damage);
            events.push(GameEvent::TrapTriggered {
                entity: player_id,
                trap,
                pos,
                damage,
            });
            if world
                .entities
                .actor_stats(player_id)
                .is_none_or(|stats| stats.hp <= 0)
                || !world.player_alive()
            {
                return Ok((events, Some(trap)));
            }
            if let Some(next_level) =
                BranchGraph::campaign_v3().descend_target(world.current_level())
            {
                let landing = find_first_safe_floor(world, next_level);
                if let Some(dest) = landing {
                    let from_level = world.current_level();
                    world.set_player_location(next_level, dest);
                    events.push(GameEvent::LevelChanged {
                        entity: player_id,
                        from: from_level,
                        to: next_level,
                    });
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Warning,
                        text: "You fall through a hole to the level below!".to_string(),
                    });
                }
            }
            return Ok((events, Some(trap)));
        }
        TrapKind::Teleport => {
            let level = world.current_level();
            let candidates = safe_floor_positions(world, level);
            if !candidates.is_empty() {
                let pick = (rng.next_u64() as usize) % candidates.len();
                let dest = candidates[pick];
                world.set_player_location(level, dest);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "A teleportation trap whisks you away!".to_string(),
                });
            }
        }
        TrapKind::SleepingGas => {
            if world
                .effective_player_resistances()
                .contains(Resistances::SLEEP)
            {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "You resist the sleeping gas.".to_string(),
                });
            } else {
                world.state_mut().paralysis_turns = world.paralysis_turns.max(2);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "A cloud of sleeping gas puts you to sleep!".to_string(),
                });
            }
        }
        TrapKind::Rust => {
            for entry in world.inventory.entries.clone() {
                if let Some(e) = world.entities.get(entry.item) {
                    if e.item()
                        .is_some_and(|(k, _, _, _, _)| k == ItemKind::OilLamp)
                        && e.item_tool_active() == Some(true)
                    {
                        world
                            .state_mut()
                            .entities
                            .set_item_tool_active(entry.item, false);
                    }
                }
            }
            let rust_target = world
                .inventory
                .equipped_slots()
                .into_iter()
                .filter_map(|(_, opt)| opt)
                .find(|id| {
                    if world
                        .entities
                        .get(*id)
                        .and_then(|e| e.item_artifact_id())
                        .is_some()
                    {
                        return false;
                    }
                    world
                        .entities
                        .item_data(*id)
                        .and_then(|d| {
                            world
                                .content_registry()
                                .item(aihack_content::item_id(d.kind))
                                .and_then(|def| def.material.as_deref())
                        })
                        .is_some_and(|m| matches!(m, "iron" | "steel"))
                });
            if let Some(gear_id) = rust_target {
                let next = (world
                    .entities
                    .get(gear_id)
                    .and_then(|e| e.item_erosion())
                    .unwrap_or(0)
                    + 1)
                .min(3);
                world.state_mut().entities.set_item_erosion(gear_id, next);
                let _ = items::recompute_player_ac(world);
            }
            events.push(GameEvent::Message {
                priority: MessagePriority::Warning,
                text: "A jet of water splashes over you!".to_string(),
            });
        }
        TrapKind::Polymorph => {
            let state = world.state_mut();
            state.status_timers.hallucinating = state.status_timers.hallucinating.max(10);
            state.hallucinating = true;
            state.status_timers.confused = state.status_timers.confused.max(5);
            if let Some(mut campaign) = state.campaign {
                if campaign.attributes.str_ > 3 {
                    campaign.attributes.str_ -= 1;
                    let dmg_bonus = campaign.damage_bonus();
                    state.campaign = Some(campaign);
                    if let Some(stats) = state.entities.actor_stats_mut(player_id) {
                        stats.damage_bonus = dmg_bonus;
                    }
                }
            }
            events.push(GameEvent::Message {
                priority: MessagePriority::Warning,
                text: "A polymorph beam washes over you!".to_string(),
            });
        }
        TrapKind::AntiMagic => {
            if world.mp > 0 {
                let drain = world.mp.min(6);
                world.state_mut().mp -= drain;
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "You feel your magical energy drain away!".to_string(),
                });
            } else {
                damage = roll_die(rng, 4);
                apply_trap_damage(world, player_id, damage);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "An anti-magic shock jolts you!".to_string(),
                });
            }
        }
        TrapKind::FallingRock => {
            let raw = roll_die(rng, 4) + roll_die(rng, 4);
            damage = if world.inventory.equipped_helm.is_some() {
                (raw / 2).max(1)
            } else {
                raw
            };
            apply_trap_damage(world, player_id, damage);
            let rock_data = crate::domain::item::try_item_data_from_registry(
                ItemKind::Rock,
                world.content_registry(),
            )
            .map_err(|err| err.to_string())?;
            let level = world.current_level();
            world
                .state_mut()
                .entities
                .spawn_item_with_data(
                    ItemKind::Rock,
                    rock_data,
                    EntityLocation::OnMap { level, pos },
                )
                .map_err(|err| err.to_string())?;
            events.push(GameEvent::Message {
                priority: MessagePriority::Warning,
                text: "A rock falls from the ceiling onto your head!".to_string(),
            });
        }
        TrapKind::Magic => match rng.next_u64() % 4 {
            0 => {
                let heal = roll_die(rng, 4);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
                    stats.hp = stats.hp.saturating_add(heal).min(stats.max_hp);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "A magical flash heals you slightly.".to_string(),
                });
            }
            1 => {
                world.state_mut().luck = world
                    .luck
                    .saturating_add(1)
                    .min(aihack_core::campaign::MAX_LUCK);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "You feel unexpectedly lucky!".to_string(),
                });
            }
            2 => {
                events.extend(reveal_all_hidden_tiles(world));
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "A blinding flash reveals hidden secrets nearby!".to_string(),
                });
            }
            _ => {
                world.state_mut().status_timers.blind = world.status_timers.blind.max(5);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "A deafening flash temporarily blinds you!".to_string(),
                });
            }
        },
        TrapKind::Fire => {
            if items::has_fire_resistance(world) {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "A harmless warmth flickers around your feet.".to_string(),
                });
            } else {
                damage = roll_die(rng, 4) + roll_die(rng, 4);
                apply_trap_damage(world, player_id, damage);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Danger,
                    text: "A gout of flame erupts beneath you!".to_string(),
                });
            }
        }
    }
    events.push(GameEvent::TrapTriggered {
        entity: player_id,
        trap,
        pos,
        damage,
    });
    Ok((events, Some(trap)))
}

fn apply_trap_damage(world: &mut GameWorld, player_id: aihack_core::ids::EntityId, damage: i16) {
    if damage > 0 {
        if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
            stats.hp = stats.hp.saturating_sub(damage);
        }
    }
}

fn safe_floor_positions(world: &GameWorld, level: aihack_core::ids::LevelId) -> Vec<Pos> {
    let map = world.map(level);
    let mut out = Vec::new();
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let p = Pos { x, y };
            if map.tile(p).ok() == Some(TileKind::Floor)
                && world.entities.alive_actor_at(level, p).is_none()
            {
                out.push(p);
            }
        }
    }
    out
}

fn find_first_safe_floor(world: &GameWorld, level: aihack_core::ids::LevelId) -> Option<Pos> {
    let map = world.map(level);
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let p = Pos { x, y };
            if map.tile(p).ok() == Some(TileKind::StairsUp)
                && world.entities.alive_actor_at(level, p).is_none()
            {
                return Some(p);
            }
        }
    }
    safe_floor_positions(world, level).into_iter().next()
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

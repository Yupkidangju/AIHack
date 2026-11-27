use std::collections::{BTreeMap, BTreeSet};

use aihack_content::ContentRegistry;
use aihack_core::{
    domain::{
        inventory::Inventory,
        level::{LevelRegistry, PHASE5_LEVEL1_ID},
        status::{Resistances, StatusTimers},
    },
    error::ContentError,
    ids::EntityId,
    position::Pos,
    world::WorldState,
};

use crate::domain::{
    entity::{EntityLocation, EntityStore},
    item::{try_item_data_from_registry, ItemKind},
    monster::try_monster_template_from_registry,
};

fn allocation_content_error(error: aihack_core::error::EntityAllocationError) -> ContentError {
    ContentError::Parse {
        file: "world bootstrap".to_owned(),
        message: error.to_string(),
    }
}

fn resolve_safe_floor_pos(map: &aihack_core::domain::map::GameMap, preferred: Pos) -> Pos {
    use aihack_core::domain::tile::TileKind;
    if map.tile(preferred).ok() == Some(TileKind::Floor) {
        return preferred;
    }
    for radius in 1..=3_i16 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs().max(dy.abs()) != radius {
                    continue;
                }
                let candidate = Pos {
                    x: preferred.x.saturating_add(dx),
                    y: preferred.y.saturating_add(dy),
                };
                if map.contains(candidate) && map.tile(candidate).ok() == Some(TileKind::Floor) {
                    return candidate;
                }
            }
        }
    }
    preferred
}

pub(crate) fn campaign_world(
    registry: &ContentRegistry,
    seed: u64,
    role: aihack_core::campaign::Role,
) -> Result<WorldState<EntityStore>, ContentError> {
    campaign_world_with_build(
        registry,
        seed,
        role,
        role.default_race(),
        role.default_alignment(),
    )
}

pub(crate) fn campaign_world_with_build(
    registry: &ContentRegistry,
    seed: u64,
    role: aihack_core::campaign::Role,
    race: aihack_core::campaign::Race,
    alignment: aihack_core::campaign::Alignment,
) -> Result<WorldState<EntityStore>, ContentError> {
    use aihack_core::{
        campaign::{CampaignState, Role},
        domain::{item::BucStatus, monster::MonsterKind},
        ids::{BranchId, LevelId},
    };
    if !role.is_valid_combo(race, alignment) {
        return Err(ContentError::Parse {
            file: "campaign build".to_owned(),
            message: format!(
                "invalid build: {} cannot be {} {}",
                role.display_name(),
                alignment.as_str(),
                race.as_str()
            ),
        });
    }
    let mut state = initial_world(registry)?;
    state.entities = EntityStore::new();
    state.levels.levels.clear();
    let (_, start) = crate::campaign_map::generate(seed, LevelId::main(1))?;
    state.player_id = state
        .entities
        .spawn_player(start[0])
        .map_err(allocation_content_error)?;
    state.inventory = Inventory::new(state.player_id);
    let mut amulet = None;
    for id in crate::campaign_map::level_ids() {
        let (mut level, centers) = crate::campaign_map::generate(seed, id)?;
        if id == LevelId::main(3) {
            let positions: Vec<Pos> = (0..level.map.height)
                .flat_map(|y| (0..level.map.width).map(move |x| Pos { x, y }))
                .filter(|&p| {
                    level.map.tile(p).ok() == Some(aihack_core::domain::tile::TileKind::Altar)
                })
                .collect();
            for pos in positions {
                level.set_altar_alignment(pos, alignment);
            }
        } else if id
            == (LevelId {
                branch: BranchId::Mines,
                depth: 2,
            })
        {
            let cross = alignment.cross_alignment();
            let positions: Vec<Pos> = (0..level.map.height)
                .flat_map(|y| (0..level.map.width).map(move |x| Pos { x, y }))
                .filter(|&p| {
                    level.map.tile(p).ok() == Some(aihack_core::domain::tile::TileKind::Altar)
                })
                .collect();
            for pos in positions {
                level.set_altar_alignment(pos, cross);
            }
        }
        state.levels.levels.push(level);
        let mut monster_spawns = match (id.branch, id.depth) {
            (aihack_core::ids::BranchId::Main, 1 | 2) => {
                vec![(MonsterKind::Jackal, centers[2])]
            }
            (aihack_core::ids::BranchId::Main, 3) => vec![(MonsterKind::Goblin, centers[2])],
            (aihack_core::ids::BranchId::Main, 4) => vec![
                (MonsterKind::Orc, centers[2]),
                (
                    MonsterKind::FloatingEye,
                    Pos {
                        x: centers[2].x + 1,
                        y: centers[2].y - 1,
                    },
                ),
                (
                    MonsterKind::GnollArcher,
                    Pos {
                        x: centers[3].x - 1,
                        y: centers[3].y + 1,
                    },
                ),
            ],
            (aihack_core::ids::BranchId::Main, 5) => vec![
                (MonsterKind::Orc, centers[2]),
                (
                    MonsterKind::Troll,
                    Pos {
                        x: centers[2].x + 1,
                        y: centers[2].y + 1,
                    },
                ),
                (
                    MonsterKind::FireAnt,
                    Pos {
                        x: centers[3].x - 1,
                        y: centers[3].y + 1,
                    },
                ),
            ],
            (aihack_core::ids::BranchId::Main, 6) => vec![
                (MonsterKind::Ogre, centers[2]),
                (
                    MonsterKind::FireAnt,
                    Pos {
                        x: centers[2].x + 1,
                        y: centers[2].y + 1,
                    },
                ),
                (
                    MonsterKind::Dragon,
                    Pos {
                        x: centers[3].x - 1,
                        y: centers[3].y,
                    },
                ),
            ],
            (aihack_core::ids::BranchId::Mines, 1) => vec![
                (MonsterKind::Kobold, centers[2]),
                (
                    MonsterKind::FloatingEye,
                    Pos {
                        x: centers[2].x + 1,
                        y: centers[2].y - 1,
                    },
                ),
                (
                    MonsterKind::GiantSpider,
                    Pos {
                        x: centers[3].x - 1,
                        y: centers[3].y + 1,
                    },
                ),
                (MonsterKind::Bat, centers[4]),
            ],
            _ => vec![
                (MonsterKind::Orc, centers[2]),
                (
                    MonsterKind::Kobold,
                    Pos {
                        x: centers[2].x + 1,
                        y: centers[2].y + 1,
                    },
                ),
                (
                    MonsterKind::Minotaur,
                    Pos {
                        x: centers[3].x - 1,
                        y: centers[3].y,
                    },
                ),
            ],
        };
        if id.branch == aihack_core::ids::BranchId::Main && (2..=5).contains(&id.depth) {
            let weighted = aihack_content::weighted_monster_kinds_for_depth(id.depth, registry);
            let total: u32 = weighted.iter().map(|(_, w)| *w).sum();
            if total > 0 {
                let mut roll = (seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add((id.depth as u64).wrapping_mul(1442695040888963407))
                    % u64::from(total)) as u32;
                let mut picked = weighted[0].0;
                for (kind, weight) in weighted {
                    if roll < weight {
                        picked = kind;
                        break;
                    }
                    roll -= weight;
                }
                if let Some(level_ref) = state.levels.levels.last() {
                    let mut chosen = None;
                    for radius in 0..=2_i16 {
                        for dy in -radius..=radius {
                            for dx in -radius..=radius {
                                if radius > 0 && dx.abs().max(dy.abs()) != radius {
                                    continue;
                                }
                                let pos = Pos {
                                    x: centers[5].x.saturating_add(dx),
                                    y: centers[5].y.saturating_add(dy),
                                };
                                if level_ref.map.tile(pos).ok()
                                    == Some(aihack_core::domain::tile::TileKind::Floor)
                                    && pos != centers[0]
                                    && !monster_spawns.iter().any(|(_, p)| *p == pos)
                                {
                                    chosen = Some(pos);
                                    break;
                                }
                            }
                            if chosen.is_some() {
                                break;
                            }
                        }
                        if chosen.is_some() {
                            break;
                        }
                    }
                    if let Some(pos) = chosen {
                        monster_spawns.push((picked, pos));
                    }
                }
            }
        }
        for (kind, pos) in monster_spawns.drain(..) {
            let monster = state
                .entities
                .spawn_monster_with_template(
                    kind,
                    try_monster_template_from_registry(kind, registry)?,
                    pos,
                )
                .map_err(allocation_content_error)?;
            state.entities.set_actor_location(monster, id, pos);
        }
        let mut floor_items = vec![
            (ItemKind::FoodRation, centers[1], BucStatus::Uncursed, 0_i8),
            (
                ItemKind::PotionHealing,
                Pos {
                    x: centers[1].x + 1,
                    y: centers[1].y,
                },
                BucStatus::Uncursed,
                0,
            ),
        ];
        match (id.branch, id.depth) {
            (aihack_core::ids::BranchId::Main, 1) => {
                floor_items.push((
                    ItemKind::ScrollEnchantWeapon,
                    Pos {
                        x: centers[1].x,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::OilLamp,
                    Pos {
                        x: centers[4].x + 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::Sack,
                    Pos {
                        x: centers[4].x - 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::SkeletonKey,
                    Pos {
                        x: centers[5].x + 1,
                        y: centers[5].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::PotionWater,
                    Pos {
                        x: centers[5].x - 1,
                        y: centers[5].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
            }
            (aihack_core::ids::BranchId::Main, 2) => {
                floor_items.push((
                    ItemKind::ScrollCharging,
                    Pos {
                        x: centers[1].x,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::ScrollRemoveCurse,
                    Pos {
                        x: centers[1].x + 1,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((ItemKind::ArmorShield, centers[4], BucStatus::Uncursed, 0));
                floor_items.push((
                    ItemKind::PickAxe,
                    Pos {
                        x: centers[4].x + 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::TinWhistle,
                    Pos {
                        x: centers[4].x - 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::ScrollBlank,
                    Pos {
                        x: centers[5].x + 1,
                        y: centers[5].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
            }
            (aihack_core::ids::BranchId::Main, 3) => {
                floor_items.push((
                    ItemKind::ShortSword,
                    Pos {
                        x: centers[1].x,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Blessed,
                    1,
                ));
                floor_items.push((
                    ItemKind::ScrollEnchantArmor,
                    Pos {
                        x: centers[1].x + 1,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((ItemKind::ArmorGloves, centers[4], BucStatus::Uncursed, 0));
                floor_items.push((
                    ItemKind::PotionWater,
                    Pos {
                        x: centers[4].x + 1,
                        y: centers[4].y,
                    },
                    BucStatus::Blessed,
                    0,
                ));
                floor_items.push((
                    ItemKind::Blindfold,
                    Pos {
                        x: centers[4].x - 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::BagOfHolding,
                    Pos {
                        x: centers[5].x + 1,
                        y: centers[5].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
            }
            (aihack_core::ids::BranchId::Main, 4) => {
                floor_items.push((
                    ItemKind::PotionExtraHealing,
                    Pos {
                        x: centers[1].x,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::PotionCure,
                    Pos {
                        x: centers[1].x + 1,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::RingFireResistance,
                    Pos {
                        x: centers[1].x - 1,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((ItemKind::PotionSpeed, centers[4], BucStatus::Uncursed, 0));
                floor_items.push((
                    ItemKind::UnicornHorn,
                    Pos {
                        x: centers[4].x + 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::DwarvishMattock,
                    Pos {
                        x: centers[4].x - 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
            }
            (aihack_core::ids::BranchId::Main, 5) => {
                floor_items.push((
                    ItemKind::ScrollCharging,
                    Pos {
                        x: centers[1].x,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::WandFire,
                    Pos {
                        x: centers[1].x + 1,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((ItemKind::ArmorCloak, centers[5], BucStatus::Uncursed, 0));
                floor_items.push((
                    ItemKind::PotionWater,
                    Pos {
                        x: centers[4].x + 1,
                        y: centers[4].y,
                    },
                    BucStatus::Cursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::LockPick,
                    Pos {
                        x: centers[4].x - 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
            }
            (aihack_core::ids::BranchId::Main, 6) => {
                floor_items.push((
                    ItemKind::PotionExtraHealing,
                    Pos {
                        x: centers[1].x,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((ItemKind::ArmorBoots, centers[5], BucStatus::Uncursed, 0));
            }
            (aihack_core::ids::BranchId::Mines, 1) => {
                floor_items.push((
                    ItemKind::ShortSword,
                    Pos {
                        x: centers[1].x,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::PotionCure,
                    Pos {
                        x: centers[1].x + 1,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::Dagger,
                    Pos {
                        x: centers[1].x - 1,
                        y: centers[1].y,
                    },
                    BucStatus::Cursed,
                    -1,
                ));
                floor_items.push((
                    ItemKind::RingPoisonResistance,
                    Pos {
                        x: centers[1].x - 1,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((ItemKind::ArmorHelm, centers[5], BucStatus::Uncursed, 0));
                floor_items.push((
                    ItemKind::PickAxe,
                    Pos {
                        x: centers[4].x + 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::OilLamp,
                    Pos {
                        x: centers[4].x - 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::Sack,
                    Pos {
                        x: centers[5].x + 1,
                        y: centers[5].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
            }
            (aihack_core::ids::BranchId::Mines, 2) => {
                floor_items.push((
                    ItemKind::ArmorChainMail,
                    Pos {
                        x: centers[1].x,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
                floor_items.push((
                    ItemKind::Mace,
                    Pos {
                        x: centers[1].x + 1,
                        y: centers[1].y + 1,
                    },
                    BucStatus::Uncursed,
                    1,
                ));
                floor_items.push((
                    ItemKind::RingProtection,
                    Pos {
                        x: centers[1].x - 1,
                        y: centers[1].y,
                    },
                    BucStatus::Uncursed,
                    1,
                ));
                floor_items.push((ItemKind::AmuletEsp, centers[4], BucStatus::Uncursed, 0));
                floor_items.push((
                    ItemKind::UnicornHorn,
                    Pos {
                        x: centers[4].x + 1,
                        y: centers[4].y,
                    },
                    BucStatus::Uncursed,
                    0,
                ));
            }
            _ => {}
        }
        for (kind, raw_pos, buc, enchantment) in floor_items {
            let pos = state
                .levels
                .levels
                .last()
                .map(|lvl| resolve_safe_floor_pos(&lvl.map, raw_pos))
                .unwrap_or(raw_pos);
            let item_id = spawn_item(
                &mut state.entities,
                registry,
                kind,
                EntityLocation::OnMap { level: id, pos },
            )?;
            state.entities.set_item_buc(item_id, buc);
            state.entities.set_item_enchantment(item_id, enchantment);
        }
        if matches!(
            (id.branch, id.depth),
            (aihack_core::ids::BranchId::Main, 2) | (aihack_core::ids::BranchId::Mines, 2)
        ) {
            let chest_pos = Pos {
                x: centers[5].x - 1,
                y: centers[5].y,
            };
            let chest_id = spawn_item(
                &mut state.entities,
                registry,
                ItemKind::Chest,
                EntityLocation::OnMap {
                    level: id,
                    pos: chest_pos,
                },
            )?;
            state.entities.set_item_container_locked(chest_id, true);
            state.entities.set_item_container_trapped(chest_id, true);
            let contents: &[(ItemKind, BucStatus, i8)] = if id == LevelId::main(2) {
                &[
                    (ItemKind::PotionExtraHealing, BucStatus::Uncursed, 0),
                    (ItemKind::LockPick, BucStatus::Uncursed, 0),
                ]
            } else {
                &[
                    (ItemKind::RingProtection, BucStatus::Blessed, 1),
                    (ItemKind::PotionWater, BucStatus::Blessed, 0),
                ]
            };
            for &(child_kind, child_buc, child_enchant) in contents {
                let child_id = spawn_item(
                    &mut state.entities,
                    registry,
                    child_kind,
                    EntityLocation::InContainer {
                        container: chest_id,
                    },
                )?;
                state.entities.set_item_buc(child_id, child_buc);
                state.entities.set_item_enchantment(child_id, child_enchant);
                state.entities.push_contained_item(chest_id, child_id);
            }
        }
        if id == LevelId::main(6) {
            amulet = Some(spawn_item(
                &mut state.entities,
                registry,
                ItemKind::AmuletAscension,
                EntityLocation::OnMap {
                    level: id,
                    pos: centers[3],
                },
            )?);
        }
    }
    let extras = match role {
        Role::Knight => vec![ItemKind::ArmorLeather, ItemKind::PotionHealing],
        Role::Scout => vec![ItemKind::FoodRation, ItemKind::FoodRation],
        Role::Mage => vec![
            ItemKind::WandMagicMissile,
            ItemKind::PotionHealing,
            ItemKind::PotionHealing,
        ],
        Role::Cleric => vec![
            ItemKind::Mace,
            ItemKind::PotionExtraHealing,
            ItemKind::ScrollIdentify,
        ],
        Role::Valkyrie => vec![
            ItemKind::LongSword,
            ItemKind::ArmorShield,
            ItemKind::PotionHealing,
        ],
        Role::Wizard => vec![
            ItemKind::Quarterstaff,
            ItemKind::WandMagicMissile,
            ItemKind::PotionHealing,
        ],
        Role::Rogue => vec![
            ItemKind::ShortSword,
            ItemKind::LockPick,
            ItemKind::PotionInvisibility,
        ],
        Role::Priest => vec![
            ItemKind::Mace,
            ItemKind::PotionWater,
            ItemKind::PotionExtraHealing,
        ],
        Role::Samurai => vec![
            ItemKind::Katana,
            ItemKind::ArmorLeather,
            ItemKind::PotionSpeed,
        ],
        Role::Barbarian => vec![
            ItemKind::BattleAxe,
            ItemKind::FoodRation,
            ItemKind::PotionHealing,
        ],
        Role::Ranger => vec![ItemKind::Rock, ItemKind::Rock, ItemKind::ArmorCloak],
        Role::Monk => vec![
            ItemKind::FoodRation,
            ItemKind::PotionHealing,
            ItemKind::PotionSpeed,
        ],
        Role::Archeologist => vec![
            ItemKind::PickAxe,
            ItemKind::ArmorLeather,
            ItemKind::ScrollIdentify,
        ],
    };
    for kind in [
        ItemKind::Dagger,
        ItemKind::FoodRation,
        ItemKind::WandMagicMissile,
        ItemKind::ScrollReveal,
        ItemKind::Rock,
    ]
    .into_iter()
    .chain(extras)
    {
        let item = spawn_item(
            &mut state.entities,
            registry,
            kind,
            EntityLocation::Inventory {
                owner: state.player_id,
            },
        )?;
        let letter = state
            .inventory
            .add_existing_with_next_letter(item)
            .expect("campaign starts with fewer than 52 items");
        state.entities.set_item_letter(item, letter);
        state.entities.set_item_buc_known(item, true);
        state.entities.set_item_enchant_known(item, true);
    }
    let campaign =
        CampaignState::new_custom(role, race, alignment, amulet.expect("Main 6 was generated"))
            .map_err(|message| ContentError::Parse {
                file: "campaign build".to_owned(),
                message,
            })?;
    let stats = state
        .entities
        .actor_stats_mut(state.player_id)
        .expect("player was spawned");
    stats.hp = campaign.max_hp();
    stats.max_hp = campaign.max_hp();
    stats.hit_bonus = campaign.hit_bonus();
    stats.damage_bonus = campaign.damage_bonus();
    state.campaign = Some(campaign);
    state.intrinsic_resistances = race.intrinsic_resistances();
    crate::systems::skills::init_role_skills(&mut state, role);
    state.gold = 50;
    state.max_mp = campaign.max_mp();
    state.mp = campaign.max_mp();
    state.ascension_pursuit_stage = 0;
    state.shop_debt = 0;
    spawn_campaign_companions_and_npcs(&mut state, registry, role, race)?;
    mark_level_initial_explored(&mut state);
    Ok(state)
}

fn find_open_actor_tile(
    state: &WorldState<EntityStore>,
    level_id: aihack_core::ids::LevelId,
    center: Pos,
    avoid_pos: Option<Pos>,
) -> Option<Pos> {
    let map = state.levels.map(level_id)?;
    for radius in 1..=3_i16 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs().max(dy.abs()) != radius {
                    continue;
                }
                let candidate = Pos {
                    x: center.x.saturating_add(dx),
                    y: center.y.saturating_add(dy),
                };
                if avoid_pos == Some(candidate) || !map.contains(candidate) {
                    continue;
                }
                if map.tile(candidate).ok() == Some(aihack_core::domain::tile::TileKind::Floor)
                    && state.entities.alive_actor_at(level_id, candidate).is_none()
                {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn find_tile_on_level(
    state: &WorldState<EntityStore>,
    level_id: aihack_core::ids::LevelId,
    target: aihack_core::domain::tile::TileKind,
) -> Option<Pos> {
    let map = state.levels.map(level_id)?;
    for y in 0..map.height {
        for x in 0..map.width {
            let pos = Pos { x, y };
            if map.tile(pos).ok() == Some(target) {
                return Some(pos);
            }
        }
    }
    None
}

fn spawn_campaign_companions_and_npcs(
    state: &mut WorldState<EntityStore>,
    registry: &ContentRegistry,
    role: aihack_core::campaign::Role,
    race: aihack_core::campaign::Race,
) -> Result<(), ContentError> {
    use aihack_core::{
        domain::{entity::Faction, monster::MonsterKind, tile::TileKind},
        ids::{BranchId, LevelId},
    };
    // 1. Starting pet adjacent to player on Main 1
    let main1 = LevelId::main(1);
    if let Some((_, player_pos)) = state.entities.actor_location(state.player_id) {
        if let Some(pet_pos) = find_open_actor_tile(state, main1, player_pos, Some(player_pos)) {
            let pet_kind = crate::systems::pets::starting_pet_kind(role, race);
            let pet_id = state
                .entities
                .spawn_monster_with_faction(
                    pet_kind,
                    try_monster_template_from_registry(pet_kind, registry)?,
                    Faction::Pet,
                    pet_pos,
                )
                .map_err(allocation_content_error)?;
            state.entities.set_actor_location(pet_id, main1, pet_pos);
            crate::systems::pets::init_pet_state(&mut state.entities, pet_id, 5);
        }
    }

    // 2. Shopkeepers + displayed unpaid wares on Main 2 and Mines 1
    let shop_configs: [(LevelId, [ItemKind; 3]); 2] = [
        (
            LevelId::main(2),
            [
                ItemKind::ScrollIdentify,
                ItemKind::ArmorCloak,
                ItemKind::RingProtection,
            ],
        ),
        (
            LevelId {
                branch: BranchId::Mines,
                depth: 1,
            },
            [
                ItemKind::ScrollEnchantArmor,
                ItemKind::ArmorGloves,
                ItemKind::LockPick,
            ],
        ),
    ];
    for (shop_level, wares) in shop_configs {
        if let Some(shop_pos) = find_tile_on_level(state, shop_level, TileKind::Shop) {
            if let Some(sk_pos) = find_open_actor_tile(state, shop_level, shop_pos, Some(shop_pos))
            {
                let sk_id = state
                    .entities
                    .spawn_monster_with_faction(
                        MonsterKind::Shopkeeper,
                        try_monster_template_from_registry(MonsterKind::Shopkeeper, registry)?,
                        Faction::Shopkeeper,
                        sk_pos,
                    )
                    .map_err(allocation_content_error)?;
                state.entities.set_actor_location(sk_id, shop_level, sk_pos);
            }
            let offsets = [(-1_i16, 0_i16), (1, 0), (-1, 1), (1, 1), (0, 2), (-2, 1)];
            let mut placed = 0usize;
            for (dx, dy) in offsets {
                if placed >= wares.len() {
                    break;
                }
                let candidate = Pos {
                    x: shop_pos.x.saturating_add(dx),
                    y: shop_pos.y.saturating_add(dy),
                };
                let is_open_floor = state.levels.map(shop_level).is_some_and(|map| {
                    map.contains(candidate) && map.tile(candidate).ok() == Some(TileKind::Floor)
                }) && state.entities.items_at(shop_level, candidate).is_empty()
                    && state
                        .entities
                        .alive_actor_at(shop_level, candidate)
                        .is_none();
                if is_open_floor {
                    let item_id = spawn_item(
                        &mut state.entities,
                        registry,
                        wares[placed],
                        EntityLocation::OnMap {
                            level: shop_level,
                            pos: candidate,
                        },
                    )?;
                    state.entities.set_item_unpaid(item_id, true);
                    placed += 1;
                }
            }
        }
    }

    // 3. Temple Priest on Main 3 adjacent to Altar
    let main3 = LevelId::main(3);
    if let Some(altar_pos) = find_tile_on_level(state, main3, TileKind::Altar) {
        if let Some(priest_pos) = find_open_actor_tile(state, main3, altar_pos, Some(altar_pos)) {
            let priest_id = state
                .entities
                .spawn_monster_with_faction(
                    MonsterKind::TemplePriest,
                    try_monster_template_from_registry(MonsterKind::TemplePriest, registry)?,
                    Faction::Peaceful,
                    priest_pos,
                )
                .map_err(allocation_content_error)?;
            state
                .entities
                .set_actor_location(priest_id, main3, priest_pos);
        }
    }

    // 4. Oracle on Main 4
    let main4 = LevelId::main(4);
    if let Some(sink_pos) = find_tile_on_level(state, main4, TileKind::Sink) {
        if let Some(oracle_pos) = find_open_actor_tile(state, main4, sink_pos, Some(sink_pos)) {
            let oracle_id = state
                .entities
                .spawn_monster_with_faction(
                    MonsterKind::Oracle,
                    try_monster_template_from_registry(MonsterKind::Oracle, registry)?,
                    Faction::Peaceful,
                    oracle_pos,
                )
                .map_err(allocation_content_error)?;
            state
                .entities
                .set_actor_location(oracle_id, main4, oracle_pos);
        }
    }

    // 5. Watchman on Mines 1 near the shop
    let mines1 = LevelId {
        branch: BranchId::Mines,
        depth: 1,
    };
    if let Some(shop_pos) = find_tile_on_level(state, mines1, TileKind::Shop) {
        if let Some(watch_pos) = find_open_actor_tile(state, mines1, shop_pos, Some(shop_pos)) {
            let watch_id = state
                .entities
                .spawn_monster_with_faction(
                    MonsterKind::Watchman,
                    try_monster_template_from_registry(MonsterKind::Watchman, registry)?,
                    Faction::Peaceful,
                    watch_pos,
                )
                .map_err(allocation_content_error)?;
            state
                .entities
                .set_actor_location(watch_id, mines1, watch_pos);
        }
    }

    // 6. Medusa guardian in Main 6 Room 5 (optional deep vault chamber off the direct Room 0->3 corridor)
    let main6 = LevelId::main(6);
    if let Some(stairs_up) = state.levels.stairs_up_pos(main6) {
        let preferred = Pos { x: 10, y: 15 };
        if let Some(medusa_pos) = find_open_actor_tile(state, main6, preferred, Some(stairs_up)) {
            let medusa_id = state
                .entities
                .spawn_monster_with_faction(
                    MonsterKind::Medusa,
                    try_monster_template_from_registry(MonsterKind::Medusa, registry)?,
                    Faction::Hostile,
                    medusa_pos,
                )
                .map_err(allocation_content_error)?;
            state
                .entities
                .set_actor_location(medusa_id, main6, medusa_pos);
        }
    }
    Ok(())
}

fn mark_level_initial_explored(state: &mut WorldState<EntityStore>) {
    let Some((_, pos)) = state.entities.actor_location(state.player_id) else {
        return;
    };
    if let Some(level) = state.levels.get_mut(state.current_level) {
        let visible = aihack_core::vision::visible_positions(&level.map, pos);
        level.mark_explored(&visible);
    }
}

fn spawn_item(
    entities: &mut EntityStore,
    registry: &ContentRegistry,
    kind: ItemKind,
    location: EntityLocation,
) -> Result<EntityId, ContentError> {
    entities
        .spawn_item_with_data(kind, try_item_data_from_registry(kind, registry)?, location)
        .map_err(allocation_content_error)
}

/// Embedded content에서 초기 월드 상태를 만든다. session/UI는 이 경계에 관여하지 않는다.
pub fn initial_world(registry: &ContentRegistry) -> Result<WorldState<EntityStore>, ContentError> {
    let mut entities = EntityStore::new();
    let level = registry
        .level("main:1")
        .ok_or_else(|| ContentError::UnknownReference {
            owner: "world bootstrap".to_owned(),
            target: "main:1".to_owned(),
        })?;
    let player_id = entities
        .spawn_player(Pos {
            x: level.player_start[0],
            y: level.player_start[1],
        })
        .map_err(allocation_content_error)?;
    for spawn in aihack_content::level_spawns(level)? {
        match spawn {
            aihack_content::LevelSpawn::Monster { kind, pos } => {
                entities
                    .spawn_monster_with_template(
                        kind,
                        try_monster_template_from_registry(kind, registry)?,
                        pos,
                    )
                    .map_err(allocation_content_error)?;
            }
            aihack_content::LevelSpawn::Item { kind, pos } => {
                spawn_item(
                    &mut entities,
                    registry,
                    kind,
                    EntityLocation::OnMap {
                        level: PHASE5_LEVEL1_ID,
                        pos,
                    },
                )?;
            }
        }
    }
    let mut inventory = Inventory::new(player_id);
    for kind in [
        ItemKind::Dagger,
        ItemKind::FoodRation,
        ItemKind::WandMagicMissile,
        ItemKind::ScrollReveal,
        ItemKind::Rock,
    ] {
        let item = spawn_item(
            &mut entities,
            registry,
            kind,
            EntityLocation::Inventory { owner: player_id },
        )?;
        let letter = inventory
            .add_existing_with_next_letter(item)
            .ok_or_else(|| ContentError::UnknownReference {
                owner: "world bootstrap inventory".to_owned(),
                target: "item letter".to_owned(),
            })?;
        entities.set_item_letter(item, letter);
    }
    for (kind, pos) in [
        (ItemKind::ArmorLeather, Pos { x: 7, y: 5 }),
        (ItemKind::ScrollIdentify, Pos { x: 9, y: 5 }),
        (ItemKind::ScrollLevelTeleport, Pos { x: 11, y: 5 }),
    ] {
        spawn_item(
            &mut entities,
            registry,
            kind,
            EntityLocation::OnMap {
                level: PHASE5_LEVEL1_ID,
                pos,
            },
        )?;
    }
    let mut state = WorldState {
        campaign: None,
        levels: LevelRegistry::from_layouts(registry.levels())?,
        current_level: PHASE5_LEVEL1_ID,
        entities,
        player_id,
        inventory,
        nutrition: 900,
        luck: 0,
        prayer_cooldown: 0,
        paralysis_turns: 0,
        hallucinating: false,
        poison_turns: 0,
        kill_count: 0,
        gold: 0,
        identified_items: Vec::new(),
        mp: 0,
        max_mp: 0,
        ascension_pursuit_stage: 0,
        intrinsic_resistances: Resistances::NONE,
        status_timers: StatusTimers::default(),
        weapon_skills: BTreeMap::new(),
        spell_skill: BTreeMap::new(),
        skill_practice: BTreeMap::new(),
        learned_spells: Vec::new(),
        generated_artifacts: BTreeSet::new(),
        ascension_altars: Vec::new(),
        shop_debt: 0,
        last_death_cause: None,
    };
    mark_level_initial_explored(&mut state);
    Ok(state)
}

use aihack_content::ContentRegistry;
use aihack_core::{
    domain::{
        inventory::Inventory,
        level::{LevelRegistry, PHASE5_LEVEL1_ID},
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

pub(crate) fn campaign_world(
    registry: &ContentRegistry,
    seed: u64,
    role: aihack_core::campaign::Role,
) -> Result<WorldState<EntityStore>, ContentError> {
    use aihack_core::{
        campaign::{CampaignState, Role},
        domain::{item::BucStatus, monster::MonsterKind},
        ids::LevelId,
    };
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
        let (level, centers) = crate::campaign_map::generate(seed, id)?;
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
            ],
        };
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
            (aihack_core::ids::BranchId::Main, 1) => floor_items.push((
                ItemKind::ScrollEnchantWeapon,
                Pos {
                    x: centers[1].x,
                    y: centers[1].y + 1,
                },
                BucStatus::Uncursed,
                0,
            )),
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
            }
            (aihack_core::ids::BranchId::Main, 6) => floor_items.push((
                ItemKind::PotionExtraHealing,
                Pos {
                    x: centers[1].x,
                    y: centers[1].y + 1,
                },
                BucStatus::Uncursed,
                0,
            )),
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
                        x: centers[1].x,
                        y: centers[1].y - 1,
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
                        x: centers[1].x,
                        y: centers[1].y - 1,
                    },
                    BucStatus::Uncursed,
                    1,
                ));
            }
            _ => {}
        }
        for (kind, pos, buc, enchantment) in floor_items {
            let item_id = spawn_item(
                &mut state.entities,
                registry,
                kind,
                EntityLocation::OnMap { level: id, pos },
            )?;
            state.entities.set_item_buc(item_id, buc);
            state.entities.set_item_enchantment(item_id, enchantment);
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
    }
    let campaign = CampaignState {
        role,
        xp: 0,
        amulet: amulet.expect("Main 6 was generated"),
    };
    let stats = state
        .entities
        .actor_stats_mut(state.player_id)
        .expect("player was spawned");
    stats.hp = campaign.max_hp();
    stats.max_hp = campaign.max_hp();
    stats.hit_bonus = campaign.hit_bonus();
    stats.damage_bonus = campaign.damage_bonus();
    state.campaign = Some(campaign);
    state.gold = 50;
    Ok(state)
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
    Ok(WorldState {
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
        last_death_cause: None,
    })
}

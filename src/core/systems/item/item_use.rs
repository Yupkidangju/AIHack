// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::assets::AssetManager;
use crate::core::entity::object::ItemClass;
use crate::core::entity::{
    status::StatusBundle, status::StatusFlags, Equipment, Health, Inventory, Item, LearnedSpell,
    PlayerTag, Position, SpellKnowledge,
};
use crate::core::events::{EventQueue, GameEvent}; // [v2.0.0 R5] 상태 이벤트 발행
use crate::ui::input::Command;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{component, Entity, EntityStore, IntoQuery};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemAction {
    Drink(Entity),
    Eat(Entity),
    Read(Entity),
    Wield(Entity),
    Wear(Entity),
    TakeOff(Entity),
    Zap(Entity),
    Apply(Entity),
    TakeOut { container: Entity, item: Entity },
    PutIn { container: Entity, item: Entity },
    Offer(Entity),
    Name(Entity),
    Call(Entity),
    EquipOffhand(Entity),
    QuiverSelect(Entity),
    Drop(Entity),
    // [v2.2.0 M5] 장비 화면용 액션
    Unequip(Entity),
    WearPrompt,      // 착용 선택 모드 진입
    TakeOffPrompt,   // 해제 선택 모드 진입
    WieldPrompt,     // 무기 착용 선택 모드 진입
    PutOnPrompt,     // 반지/부적 착용 선택 모드 진입
    RemovePrompt,    // 반지/부적 해제 선택 모드 진입
}

///
#[legion::system]
#[read_component(Inventory)]
#[read_component(Equipment)]
#[read_component(Item)]
#[read_component(Position)]
#[read_component(PlayerTag)]
pub fn item_input(
    world: &mut SubWorld,
    #[resource] cmd: &Command,
    #[resource] assets: &AssetManager,
    #[resource] grid: &crate::core::dungeon::Grid,
    #[resource] action: &mut Option<ItemAction>,
    #[resource] log: &mut GameLog,
) {
    if action.is_some() {
        return;
    }

    //
    let mut player_query = <(&Inventory, &Equipment)>::query().filter(component::<PlayerTag>());
    let (inventory, equipment) = match player_query.iter(world).next() {
        Some((i, e)) => (i, e),
        None => return,
    };

    // 장비 해제 명령 처리
    if *cmd == Command::TakeOff {
        if let Some((_slot, entity)) = equipment.slots.iter().next() {
            *action = Some(ItemAction::TakeOff(*entity));
        } else {
            log.add("You are not wearing anything.", log.current_turn);
        }
        return;
    }

    let target_classes = match cmd {
        Command::Quaff => vec![ItemClass::Potion],
        Command::Read => vec![ItemClass::Scroll, ItemClass::Spellbook],
        Command::Wear => vec![ItemClass::Armor],
        Command::Wield => vec![ItemClass::Weapon],
        Command::Eat => vec![ItemClass::Food],
        Command::Apply => vec![
            ItemClass::Tool,
            ItemClass::Weapon,
            ItemClass::Wand,
            ItemClass::Potion,
            ItemClass::Gem,
        ],
        Command::Drop => vec![
            ItemClass::Weapon,
            ItemClass::Armor,
            ItemClass::Ring,
            ItemClass::Amulet,
            ItemClass::Tool,
            ItemClass::Food,
            ItemClass::Potion,
            ItemClass::Scroll,
            ItemClass::Spellbook,
            ItemClass::Wand,
            ItemClass::Gem,
            ItemClass::Rock,
            ItemClass::Coin, // Coin support?
        ],
        Command::Name | Command::Call => vec![
            ItemClass::Weapon,
            ItemClass::Armor,
            ItemClass::Ring,
            ItemClass::Amulet,
            ItemClass::Tool,
            ItemClass::Food,
            ItemClass::Potion,
            ItemClass::Scroll,
            ItemClass::Spellbook,
            ItemClass::Wand,
            ItemClass::Gem,
        ],
        _ => return,
    };

    //
    for item_ent in &inventory.items {
        if let Ok(entry) = world.entry_ref(*item_ent) {
            if let Ok(item) = entry.get_component::<Item>() {
                if let Some(template) = assets.items.get_by_kind(item.kind) {
                    if target_classes.contains(&template.class) {
                        // 액션 생성
                        *action = match cmd {
                            Command::Quaff => Some(ItemAction::Drink(*item_ent)),
                            Command::Read => Some(ItemAction::Read(*item_ent)),
                            Command::Wear => Some(ItemAction::Wear(*item_ent)),
                            Command::Wield => Some(ItemAction::Wield(*item_ent)),
                            Command::Eat => Some(ItemAction::Eat(*item_ent)),
                            Command::Apply => Some(ItemAction::Apply(*item_ent)),
                            Command::Name => Some(ItemAction::Name(*item_ent)),
                            Command::Call => Some(ItemAction::Call(*item_ent)),
                            Command::Drop => Some(ItemAction::Drop(*item_ent)),
                            _ => None,
                        };

                        if action.is_some() {
                            return;
                        }
                    }
                }
            }
        }
    }

    //
    if *cmd == Command::Quaff {
        let mut p_query = <&Position>::query().filter(component::<PlayerTag>());
        if let Some(pos) = p_query.iter(world).next() {
            if let Some(tile) = grid.get_tile(pos.x as usize, pos.y as usize) {
                if tile.typ == crate::core::dungeon::tile::TileType::Fountain {
                    log.add("You drink from the fountain.", log.current_turn);
                    log.add("It's cool and refreshing.", log.current_turn);
                    return;
                } else if tile.typ == crate::core::dungeon::tile::TileType::Sink {
                    log.add("You drink from the sink.", log.current_turn);
                    log.add("It tastes of metallic pipes.", log.current_turn);
                    return;
                }
            }
        }
    }

    //
    match cmd {
        Command::Quaff => log.add("You have nothing to drink.", log.current_turn),
        Command::Read => log.add("You have nothing to read.", log.current_turn),
        Command::Wear => log.add("You have nothing to wear.", log.current_turn),
        Command::Wield => log.add("You have no weapon to wield.", log.current_turn),
        Command::Eat => log.add("You have nothing to eat.", log.current_turn),
        _ => {}
    }
}

///
#[legion::system]
#[read_component(Item)]
#[write_component(Inventory)]
#[write_component(StatusBundle)]
#[write_component(Health)]
#[write_component(Position)]
#[write_component(SpellKnowledge)]
#[read_component(PlayerTag)]
pub fn item_use(
    world: &mut SubWorld,
    #[resource] _assets: &AssetManager,
    #[resource] grid: &mut crate::core::dungeon::Grid,
    #[resource] log: &mut GameLog,
    #[resource] ident_table: &crate::core::entity::identity::IdentityTable,
    #[resource] action: &Option<ItemAction>,
    #[resource] state: &mut crate::core::game_state::GameState,
    #[resource] altar_update: &mut Option<crate::core::systems::pray::PendingAltarUpdate>,
    #[resource] event_queue: &mut EventQueue, // [v2.0.0 R5] 상태 이벤트
    command_buffer: &mut CommandBuffer,
) {
    let action = match action {
        Some(a) => a,
        None => return,
    };

    let item_ent = match action {
        ItemAction::Drink(e)
        | ItemAction::Eat(e)
        | ItemAction::Read(e)
        | ItemAction::Zap(e)
        | ItemAction::Apply(e)
        | ItemAction::Name(e)
        | ItemAction::Call(e)
        | ItemAction::Offer(e)
        | ItemAction::Drop(e)
        | ItemAction::Unequip(e) => e,
        ItemAction::Wield(e)
        | ItemAction::Wear(e)
        | ItemAction::TakeOff(e)
        | ItemAction::EquipOffhand(e)
        | ItemAction::QuiverSelect(e) => e,
        ItemAction::TakeOut { item, .. } => item,
        ItemAction::PutIn { item, .. } => item,
        // [v2.2.0
        ItemAction::WearPrompt
        | ItemAction::TakeOffPrompt
        | ItemAction::WieldPrompt
        | ItemAction::PutOnPrompt
        | ItemAction::RemovePrompt => return,
    };

    // 장비 액션은 EquipmentSystem에서 처리
    match action {
        ItemAction::Wield(_)
        | ItemAction::Wear(_)
        | ItemAction::TakeOff(_)
        | ItemAction::Unequip(_)
        | ItemAction::EquipOffhand(_)
        | ItemAction::QuiverSelect(_) => return,
        ItemAction::TakeOut { container, item } => {
            //
            let mut container_inv = None;
            if let Ok(entry) = world.entry_ref(*container) {
                if let Ok(inv) = entry.get_component::<Inventory>() {
                    container_inv = Some(inv.clone());
                }
            }

            if let Some(mut inv) = container_inv {
                //
                if let Some(pos) = inv.items.iter().position(|&e| e == *item) {
                    inv.items.remove(pos);
                    command_buffer.add_component(*container, inv);
                    //
                    command_buffer.remove_component::<crate::core::entity::InContainerTag>(*item);

                    //
                    let mut player_query =
                        <&mut Inventory>::query().filter(component::<PlayerTag>());
                    if let Some(p_inv) = player_query.iter_mut(world).next() {
                        p_inv.items.push(*item);
                        log.add("You take the item out of the container.", log.current_turn);
                    }
                }
            }
            return;
        }
        ItemAction::PutIn { container, item } => {
            // 2. 마법 가방(Bag of Holding) 폭발 체크 (Phase 48.2 - 정규화)
            if crate::core::systems::inventory::InventorySystem::check_boh_explosion(
                world, *container, *item,
            ) {
                log.add_colored(
                    "Magical energies clash! The bag explodes!",
                    [255, 50, 50], // Red
                    log.current_turn,
                );

                // 재귀적 파괴: 가방 내부의 모든 아이템 소멸 (NetHack 100% 이식)
                if let Ok(c_entry) = world.entry_ref(*container) {
                    if let Ok(c_inv) = c_entry.get_component::<Inventory>() {
                        for &inner_item in &c_inv.items {
                            command_buffer.remove(inner_item);
                        }
                    }
                }

                //
                command_buffer.remove(*container);
                command_buffer.remove(*item);

                //
                let mut query = <&mut Inventory>::query().filter(component::<PlayerTag>());
                if let Some(inv) = query.iter_mut(world).next() {
                    inv.items.retain(|&e| e != *container && e != *item);
                }

                log.add(
                    "Everything inside is lost in another dimension!",
                    log.current_turn,
                );
                return;
            }

            //
            if let Ok(mut entry) = world.entry_mut(*container) {
                if let Ok(c_inv) = entry.get_component_mut::<Inventory>() {
                    c_inv.items.push(*item);
                    command_buffer.add_component(
                        *item,
                        crate::core::entity::InContainerTag {
                            container: *container,
                        },
                    );

                    //
                    let mut query = <&mut Inventory>::query().filter(component::<PlayerTag>());
                    if let Some(p_inv) = query.iter_mut(world).next() {
                        if let Some(pos) = p_inv.items.iter().position(|&e| e == *item) {
                            p_inv.items.remove(pos);
                        }
                    }

                    log.add("You put the item into the container.", log.current_turn);
                }
            }
            return;
        }
        ItemAction::Apply(e) => {
            crate::core::systems::apply::item_apply(
                *e,
                world,
                _assets,
                grid,
                log,
                log.current_turn,
                state,
                command_buffer,
            );
            return;
        }
        ItemAction::Name(e) => {
            *state = crate::core::game_state::GameState::Naming {
                entity: Some(*e),
                is_call: false,
            };
            return;
        }
        ItemAction::Call(e) => {
            *state = crate::core::game_state::GameState::Naming {
                entity: Some(*e),
                is_call: true,
            };
            return;
        }
        _ => {}
    }

    // 아이템 정보 수집 (Borrow 충돌 방지)
    use crate::core::systems::item_helper::ItemHelper;
    let (item_inst, display_name, item_class) = match world.entry_ref(*item_ent) {
        Ok(entry) => {
            let item = entry.get_component::<Item>().ok().cloned();
            let (d_name, class) = if let Some(i) = &item {
                if let Some(t) = _assets.items.get_by_kind(i.kind) {
                    (ItemHelper::get_name(i, t, Some(ident_table)), t.class)
                } else {
                    ("strange object".to_string(), ItemClass::IllObj)
                }
            } else {
                ("strange object".to_string(), ItemClass::IllObj)
            };
            (item, d_name, class)
        }
        Err(_) => (None, "strange object".to_string(), ItemClass::IllObj),
    };

    let mut item_inst = match item_inst {
        Some(i) => i,
        None => return,
    };
    let item_name = item_inst.kind.to_string();

    let mut player_query = <(
        &mut crate::core::entity::player::Player,
        &mut Inventory,
        &mut StatusBundle,
        &mut Health,
        &mut Position,
        &mut SpellKnowledge,
        &mut Equipment,
    )>::query()
    .filter(component::<PlayerTag>());

    let mut target_to_identify = None;
    if item_name == "Scroll of identify" {
        let mut inv_query = <&Inventory>::query().filter(component::<PlayerTag>());
        if let Some(p_inv) = inv_query.iter(world).next() {
            if !p_inv.items.is_empty() {
                let mut rng = NetHackRng::new(log.current_turn);
                let idx = rng.rn2(p_inv.items.len() as i32) as usize;
                target_to_identify = Some(p_inv.items[idx]);
            }
        }
    }

    let mut p_query = <Entity>::query().filter(component::<PlayerTag>());
    let player_ent = p_query.iter(world).next().cloned();

    let mut enchant_weapon_target = None;
    let mut enchant_armor_target = None;

    if let Some((p_stats, p_inv, p_status, p_health, p_pos, p_spell, p_equip)) =
        player_query.iter_mut(world).next()
    {
        match action {
            ItemAction::Drink(_) => {
                log.add(format!("You quaff a {}.", display_name), log.current_turn);

                match item_name.as_str() {
                    "Potion of healing" | "healing potion" => {
                        let amount = 10;
                        p_health.current = (p_health.current + amount).min(p_health.max);
                        log.add("You feel better.", log.current_turn);
                    }
                    "Potion of extra healing" => {
                        let amount = 25;
                        p_health.current = (p_health.current + amount).min(p_health.max);
                        log.add("You feel much better.", log.current_turn);
                    }
                    "Potion of confusion" | "confusion" => {
                        p_status.add(StatusFlags::CONFUSED, 20);
                        log.add("You feel somewhat confused.", log.current_turn);
                        // [v2.0.0 R5] 혼란 상태 적용 이벤트
                        event_queue.push(GameEvent::StatusApplied {
                            target: "player".to_string(),
                            status: StatusFlags::CONFUSED,
                            turns: 20,
                        });
                    }
                    "Potion of blindness" | "blindness" => {
                        p_status.add(StatusFlags::BLIND, 30);
                        log.add("You can't see anything!", log.current_turn);
                        // [v2.0.0 R5] 실명 상태 적용 이벤트
                        event_queue.push(GameEvent::StatusApplied {
                            target: "player".to_string(),
                            status: StatusFlags::BLIND,
                            turns: 30,
                        });
                    }
                    "Potion of speed" => {
                        p_status.add(StatusFlags::FAST, 50);
                        log.add("You feel yourself speeding up.", log.current_turn);
                        // [v2.0.0 R5] 속도 상태 적용 이벤트
                        event_queue.push(GameEvent::StatusApplied {
                            target: "player".to_string(),
                            status: StatusFlags::FAST,
                            turns: 50,
                        });
                    }
                    "Potion of gain level" => {
                        p_stats.experience += 1000;
                        log.add("You feel much more experienced.", log.current_turn);
                    }
                    "Potion of paralysis" | "paralysis" => {
                        p_status.add(StatusFlags::PARALYZED, 10);
                        log.add("You are paralyzed!", log.current_turn);
                        // [v2.0.0 R5] 마비 상태 적용 이벤트
                        event_queue.push(GameEvent::StatusApplied {
                            target: "player".to_string(),
                            status: StatusFlags::PARALYZED,
                            turns: 10,
                        });
                    }
                    "Potion of see invisible" => {
                        log.add("Your vision feels sharper.", log.current_turn);
                    }
                    _ => {
                        log.add("It tastes strange.", log.current_turn);
                    }
                }

                item_inst.known = true;
                item_inst.bknown = true;
                command_buffer.add_component(*item_ent, item_inst.clone());

                consume_item(p_inv, *item_ent, command_buffer);
            }
            ItemAction::Eat(_) => {
                log.add(format!("You eat a {}.", display_name), log.current_turn);

                // 1. 시체 판별: corpsenm 필드 or 이름에 "corpse" 포함
                // [v2.0.0] death.rs에서 생성한 시체는 "{name} corpse" 형식
                if item_inst.corpsenm.is_some() || item_name.ends_with("corpse") {
                    let age_diff = log.current_turn - item_inst.age;
                    if age_diff > 50 {
                        log.add("Ulch! That corpse was rotten!", log.current_turn);
                        p_status.add(StatusFlags::SICK, 50);
                        // [v2.0.0 R5] 질병 상태 적용 이벤트
                        event_queue.push(GameEvent::StatusApplied {
                            target: "player".to_string(),
                            status: StatusFlags::SICK,
                            turns: 50,
                        });
                    }

                    if let Some(monster_name) = &item_inst.corpsenm {
                        if let Some(m_template) = _assets.monsters.get_template(monster_name) {
                            // 시체 영양분: 몬스터 무게 / 2 (NetHack 방식)
                            let nutrition_gain = (m_template.weight as i32 / 2).max(10);
                            p_stats.nutrition += nutrition_gain;
                            log.add(
                                format!(
                                    "That {} corpse tasted better than you expected.",
                                    monster_name
                                ),
                                log.current_turn,
                            );

                            // 3.6.7: rn2(100) < chance %
                            let mut rng = NetHackRng::new(log.current_turn);

                            // 내성 전이 (conveys)
                            //
                            // NetHack: MR_POISON(0x20) -> StatusFlags::POISON_RES (0x04000000)
                            if m_template.conveys > 0 && rng.rn2(3) == 0 {
                                let mut gained = StatusFlags::empty();
                                if m_template.conveys & 0x0001 != 0 {
                                    gained |= StatusFlags::FIRE_RES;
                                }
                                if m_template.conveys & 0x0002 != 0 {
                                    gained |= StatusFlags::COLD_RES;
                                }
                                if m_template.conveys & 0x0004 != 0 {
                                    gained |= StatusFlags::SLEEP_RES;
                                }
                                if m_template.conveys & 0x0008 != 0 {
                                    gained |= StatusFlags::DISINT_RES;
                                }
                                if m_template.conveys & 0x0010 != 0 {
                                    gained |= StatusFlags::SHOCK_RES;
                                }
                                if m_template.conveys & 0x0020 != 0 {
                                    gained |= StatusFlags::POISON_RES;
                                }
                                if m_template.conveys & 0x0040 != 0 {
                                    gained |= StatusFlags::ACID_RES;
                                }

                                if !gained.is_empty() {
                                    p_status.permanent |= gained;
                                    log.add_colored(
                                        "You feel a strange sense of protection.",
                                        [255, 255, 100],
                                        log.current_turn,
                                    );
                                }
                            }
                        }
                    } else {
                        log.add("You eat a strange, unidentified corpse.", log.current_turn);
                    }
                } else if let Some(template) = _assets.items.get_template(&item_name) {
                    let mut nutrition_gain = template.nutrition as i32;

                    if item_inst.blessed {
                        nutrition_gain = (nutrition_gain * 5) / 4;
                    } else if item_inst.cursed {
                        nutrition_gain = (nutrition_gain * 3) / 4;
                    }

                    p_stats.nutrition += nutrition_gain;

                    if p_stats.hunger == crate::core::entity::player::HungerState::Satiated {
                        log.add(
                            "You're having a hard time getting it all down.",
                            log.current_turn,
                        );
                        if p_stats.nutrition > 2000 {
                            p_status.add(StatusFlags::CHOKING, 10);
                            log.add("You are choking!", log.current_turn);
                            // [v2.0.0 R5] 질식 상태 적용 이벤트
                            event_queue.push(GameEvent::StatusApplied {
                                target: "player".to_string(),
                                status: StatusFlags::CHOKING,
                                turns: 10,
                            });
                        }
                    } else {
                        log.add("That tasted good.", log.current_turn);
                    }
                }

                p_health.current = (p_health.current + 2).min(p_health.max);
                item_inst.known = true;
                item_inst.bknown = true;
                command_buffer.add_component(*item_ent, item_inst.clone());
                consume_item(p_inv, *item_ent, command_buffer);
            }
            ItemAction::Read(_) => {
                log.add(format!("You read a {}.", display_name), log.current_turn);
                match item_name.as_str() {
                    "Scroll of teleportation" => {
                        let mut rng = NetHackRng::new(log.current_turn);
                        if let Some(target) = player_ent {
                            command_buffer.add_component(
                                target,
                                crate::core::systems::teleport::TeleportAction {
                                    target,
                                    is_level_tele: p_status.has(StatusFlags::CONFUSED)
                                        || rng.rn2(10) == 0,
                                },
                            );
                        }
                    }
                    "Scroll of level teleportation" => {
                        if let Some(target) = player_ent {
                            command_buffer.add_component(
                                target,
                                crate::core::systems::teleport::TeleportAction {
                                    target,
                                    is_level_tele: true,
                                },
                            );
                        }
                    }
                    "Scroll of magic mapping" => {
                        log.add("A map coalesces in your mind!", log.current_turn);
                        command_buffer.add_component(
                            player_ent.unwrap(),
                            crate::core::entity::MagicMapRequest,
                        );
                    }
                    "Scroll of light" => {
                        log.add(
                            "You are surrounded by a shimmer of light!",
                            log.current_turn,
                        );
                        // 현재 방/복도 밝히기
                        grid.light_room_at(p_pos.x as usize, p_pos.y as usize);
                    }
                    "Scroll of confuse monster" => {
                        p_status.add(StatusFlags::CONFUSED, 20);
                        log.add("Your hands begin to glow handsomely.", log.current_turn);
                        // [v2.0.0
                        event_queue.push(GameEvent::StatusApplied {
                            target: "player".to_string(),
                            status: StatusFlags::CONFUSED,
                            turns: 20,
                        });
                    }
                    "Scroll of identify" => {
                        let mut count = 1;
                        if item_inst.blessed {
                            let mut rng = NetHackRng::new(log.current_turn);
                            if rng.rn2(4) == 0 {
                                count = 100; // Identify all
                            } else {
                                count = rng.rn1(3, 1) as u32;
                            }
                        } else if item_inst.cursed {
                            let mut rng = NetHackRng::new(log.current_turn);
                            if rng.rn2(2) == 0 {
                                count = 0;
                            }
                        }

                        if count > 0 {
                            *state = crate::core::game_state::GameState::IdentifySelect {
                                scroll: *item_ent,
                                count,
                            };
                            log.add("Select an item to identify.", log.current_turn);
                            return; // Consume after selection
                        } else {
                            log.add(
                                "You feel more knowledgeable, but nothing happens.",
                                log.current_turn,
                            );
                        }
                    }
                    "Scroll of enchant weapon" => {
                        if let Some(w_ent) = p_equip
                            .slots
                            .get(&crate::core::entity::EquipmentSlot::Melee)
                        {
                            enchant_weapon_target = Some(*w_ent);
                            log.add("Your weapon glows blue for a moment.", log.current_turn);
                        } else {
                            log.add("Your hands glow blue for a moment.", log.current_turn);
                        }
                    }
                    "Scroll of enchant armor" => {
                        if let Some(a_ent) =
                            p_equip.slots.get(&crate::core::entity::EquipmentSlot::Body)
                        {
                            enchant_armor_target = Some(*a_ent);
                            log.add("Your armor glows silver for a moment.", log.current_turn);
                        } else {
                            log.add("Your skin tingles for a moment.", log.current_turn);
                        }
                    }
                    _ => {
                        if item_class == ItemClass::Spellbook {
                            // 마법 주문 습득 확률 체크 (원본: Int 기반)
                            let mut rng = NetHackRng::new(log.current_turn);
                            let spell_level = 1; // TODO: 아이템 데이터 연동
                                                 // 성공 확률 공식 (단순화): (Int * 5) - (Spell Level * 10)
                            let success_chance = (p_stats.int.base * 5) - (spell_level * 10);

                            if rng.rn2(100) < success_chance && success_chance > 0 {
                                let mut assigned_key = None;
                                for c in 'a'..='z' {
                                    if !p_spell.spells.contains_key(&c) {
                                        assigned_key = Some(c);
                                        break;
                                    }
                                }

                                if let Some(key) = assigned_key {
                                    p_spell.spells.insert(
                                        key,
                                        LearnedSpell {
                                            name: item_name.clone(),
                                            level: 1,
                                            retention: 20000,
                                        },
                                    );
                                    log.add(
                                        format!("You learn the spell: {} ({})", item_name, key),
                                        log.current_turn,
                                    );
                                } else {
                                    log.add("Your head is full of spells!", log.current_turn);
                                }
                            } else {
                                // 학습 실패
                                log.add(
                                    format!("You fail to understand the {}.", item_name),
                                    log.current_turn,
                                );
                                // 실패 시 에너지 소모
                                p_stats.energy = (p_stats.energy - 5).max(0);
                                log.add("You feel a slight headache.", log.current_turn);
                            }
                        }
                    }
                }
                item_inst.known = true;
                item_inst.bknown = true;
                command_buffer.add_component(*item_ent, item_inst.clone());
                consume_item(p_inv, *item_ent, command_buffer);
            }
            ItemAction::Offer(item_ent) => {
                let mut rng = NetHackRng::new(log.current_turn);
                if let Some(new_align) = crate::core::systems::pray::try_offer(
                    *item_ent,
                    world,
                    grid,
                    _assets,
                    &mut rng,
                    log,
                    log.current_turn,
                    command_buffer,
                ) {
                    //
                    let mut p_query = <&Position>::query().filter(component::<PlayerTag>());
                    if let Some(pos) = p_query.iter(world).next() {
                        *altar_update = Some(crate::core::systems::pray::PendingAltarUpdate {
                            pos: (pos.x, pos.y),
                            new_align,
                        });
                    }
                }
            }
            ItemAction::Drop(item_ent) => {
                //
                let mut p_pos = None;
                if let Some(pos) = <&Position>::query()
                    .filter(component::<PlayerTag>())
                    .iter(world)
                    .next()
                {
                    p_pos = Some((pos.x, pos.y));
                }

                //
                let mut player_inv = None;
                let mut inv_query = <&mut Inventory>::query().filter(component::<PlayerTag>());
                if let Some(inv) = inv_query.iter_mut(world).next() {
                    if let Some(pos) = inv.items.iter().position(|&e| e == *item_ent) {
                        inv.items.remove(pos);
                        player_inv = Some(inv.clone());
                    }
                }

                if player_inv.is_some() {
                    //
                    let mut handled_by_sink = false;
                    if let Some((px, py)) = p_pos {
                        if let Some(tile) = grid.get_tile(px as usize, py as usize) {
                            if tile.typ == crate::core::dungeon::tile::TileType::Sink {
                                handled_by_sink = crate::core::systems::sink::try_drop_into_sink(
                                    *item_ent,
                                    world,
                                    log,
                                    log.current_turn,
                                );
                            }
                        }
                    }

                    // 4. 일반 드랍 처리 (바닥에 놓기)
                    if !handled_by_sink {
                        log.add("You drop the item.", log.current_turn);
                        if let Some((px, py)) = p_pos {
                            command_buffer.add_component(*item_ent, Position { x: px, y: py });
                        }
                    } else {
                        //
                        // SubWorld에서는 remove가 안 될 수 있음.
                        //
                        //
                        command_buffer.remove(*item_ent);
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(w_ent) = enchant_weapon_target {
        if let Ok(mut w_entry) = world.entry_mut(w_ent) {
            if let Ok(w_item) = w_entry.get_component_mut::<Item>() {
                w_item.spe += 1;
                w_item.cursed = false;
            }
        }
    }

    if let Some(a_ent) = enchant_armor_target {
        if let Ok(mut a_entry) = world.entry_mut(a_ent) {
            if let Ok(a_item) = a_entry.get_component_mut::<Item>() {
                a_item.spe += 1;
                a_item.cursed = false;
            }
        }
    }

    if let Some(target_ent) = target_to_identify {
        if let Ok(entry) = world.entry_ref(target_ent) {
            if let Ok(item) = entry.get_component::<Item>() {
                let mut identified = item.clone();
                identified.known = true;
                identified.bknown = true;
                command_buffer.add_component(target_ent, identified);
                log.add("You identify an item.", log.current_turn);
            }
        }
    }
}

fn consume_item(inv: &mut Inventory, item_ent: Entity, command_buffer: &mut CommandBuffer) {
    if let Some(pos) = inv.items.iter().position(|&e| e == item_ent) {
        inv.items.remove(pos);
    }
    command_buffer.remove(item_ent);
}

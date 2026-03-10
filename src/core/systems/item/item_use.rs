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
    WearPrompt,    // 착용 선택 모드 진입
    TakeOffPrompt, // 해제 선택 모드 진입
    WieldPrompt,   // 무기 착용 선택 모드 진입
    PutOnPrompt,   // 반지/부적 착용 선택 모드 진입
    RemovePrompt,  // 반지/부적 해제 선택 모드 진입

    // [v2.21.0 R9-2] Extended Commands
    Rub(Entity),
    Dip(Entity),
}

/// [v3.0.0] GameContext 기반 전환 완료
pub fn item_input_system(ctx: &mut crate::core::context::GameContext) {
    if ctx
        .action_queue
        .actions
        .iter()
        .any(|a| matches!(a, crate::core::action_queue::GameAction::Item(_)))
    {
        return;
    }

    // Gather: 플레이어 인벤토리/장비 정보 수집
    let mut player_items = Vec::new();
    let mut equipment_slots: Vec<(crate::core::entity::EquipmentSlot, Entity)> = Vec::new();
    {
        let mut player_query = <(&Inventory, &Equipment)>::query().filter(component::<PlayerTag>());
        if let Some((inventory, equipment)) = player_query.iter(ctx.world).next() {
            player_items = inventory.items.clone();
            equipment_slots = equipment.slots.iter().map(|(k, v)| (*k, *v)).collect();
        } else {
            return;
        }
    }

    // 장비 해제 명령 처리
    if ctx.cmd == Command::TakeOff {
        if let Some((_slot, entity)) = equipment_slots.first() {
            ctx.action_queue
                .push(crate::core::action_queue::GameAction::Item(
                    ItemAction::TakeOff(*entity),
                ));
        } else {
            ctx.log
                .add("You are not wearing anything.", ctx.log.current_turn);
        }
        return;
    }

    let target_classes = match ctx.cmd {
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
            ItemClass::Coin,
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

    // 인벤토리 아이템 매칭 (Gather된 items로 순회)
    for item_ent in &player_items {
        if let Ok(entry) = ctx.world.entry_ref(*item_ent) {
            if let Ok(item) = entry.get_component::<Item>() {
                if let Some(template) = ctx.assets.items.get_by_kind(item.kind) {
                    if target_classes.contains(&template.class) {
                        // 액션 생성
                        let action = match ctx.cmd {
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

                        if let Some(a) = action {
                            ctx.action_queue
                                .push(crate::core::action_queue::GameAction::Item(a));
                            return;
                        }
                    }
                }
            }
        }
    }

    //
    if ctx.cmd == Command::Quaff {
        let mut p_query = <&Position>::query().filter(component::<PlayerTag>());
        if let Some(pos) = p_query.iter(ctx.world).next() {
            if let Some(tile) = ctx.grid.get_tile(pos.x as usize, pos.y as usize) {
                if tile.typ == crate::core::dungeon::tile::TileType::Fountain {
                    ctx.log
                        .add("You drink from the fountain.", ctx.log.current_turn);
                    ctx.log
                        .add("It's cool and refreshing.", ctx.log.current_turn);
                    return;
                } else if tile.typ == crate::core::dungeon::tile::TileType::Sink {
                    ctx.log
                        .add("You drink from the sink.", ctx.log.current_turn);
                    ctx.log
                        .add("It tastes of metallic pipes.", ctx.log.current_turn);
                    return;
                }
            }
        }
    }

    //
    match ctx.cmd {
        Command::Quaff => ctx
            .log
            .add("You have nothing to drink.", ctx.log.current_turn),
        Command::Read => ctx
            .log
            .add("You have nothing to read.", ctx.log.current_turn),
        Command::Wear => ctx
            .log
            .add("You have nothing to wear.", ctx.log.current_turn),
        Command::Wield => ctx
            .log
            .add("You have no weapon to wield.", ctx.log.current_turn),
        Command::Eat => ctx
            .log
            .add("You have nothing to eat.", ctx.log.current_turn),
        _ => {}
    }
}

/// [v3.0.0] GameContext 기반 전환 완료 (아이템 사용 시스템)
pub fn item_use_system(ctx: &mut crate::core::context::GameContext) {
    let crate::core::context::GameContext {
        world,
        grid,
        log,
        action_queue,
        event_queue,
        game_state,
        assets,
        ..
    } = ctx;
    let world = &mut **world;
    let grid = &mut **grid;
    let log = &mut **log;
    let _assets = &**assets;
    let state = &mut **game_state;
    // [v3.0.0] CommandBuffer 제거 — world 직접 접근
    // [v3.0.0] deferred ops 변수 (쿼리 빌림 중 world.entry 호출 불가하므로)
    let mut deferred_add_item: Option<(Entity, Item)> = None;
    let mut deferred_consume: Option<Entity> = None;
    let mut deferred_teleport: Option<(Entity, bool)> = None;
    let mut deferred_magic_map: Option<Entity> = None;

    // [v3.0.0] ident_table — 빈 테이블 (이름 표시에만 사용, 기능 영향 없음)
    let ident_table = &crate::core::entity::identity::IdentityTable::new();
    // [v3.0.0] altar_update — 제단 제물 업데이트용
    let mut altar_update_local: Option<crate::core::systems::pray::PendingAltarUpdate> = None;
    let altar_update = &mut altar_update_local;

    let mut to_keep = Vec::new();
    let mut action_to_process = None;
    while let Some(game_action) = action_queue.pop() {
        if let crate::core::action_queue::GameAction::Item(a) = game_action {
            action_to_process = Some(a);
        } else {
            to_keep.push(game_action);
        }
    }
    for a in to_keep {
        action_queue.push(a);
    }

    let action = match action_to_process {
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
        | ItemAction::Rub(e)
        | ItemAction::Dip(e)
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
            let mut container_inv = None;
            if let Ok(entry) = world.entry_ref(container) {
                if let Ok(inv) = entry.get_component::<Inventory>() {
                    container_inv = Some(inv.clone());
                }
            }

            if let Some(mut inv) = container_inv {
                if inv.items.contains(&item) {
                    inv.remove_item(item);
                    if let Some(mut e) = world.entry(container) {
                        e.add_component(inv);
                    }
                    if let Some(mut e) = world.entry(item) {
                        e.remove_component::<crate::core::entity::InContainerTag>();
                    }

                    // [v2.21.0 R9-3] 플레이어 인벤토리 편입 시 Stacking 처리
                    let mut merge_target = None;
                    let mut qty_to_add = 0;
                    if let Ok(item_entry) = world.entry_ref(item) {
                        if let Ok(item_comp) = item_entry.get_component::<Item>() {
                            qty_to_add = item_comp.quantity;
                            let mut p_query =
                                <&Inventory>::query().filter(component::<PlayerTag>());
                            if let Some(p_inv) = p_query.iter(world).next() {
                                merge_target = crate::core::systems::item_helper::ItemHelper::try_find_merge_target(p_inv, item_comp, world);
                            }
                        }
                    }

                    if let Some(target) = merge_target {
                        if let Ok(target_entry) = world.entry_ref(target) {
                            if let Ok(exist_comp) = target_entry.get_component::<Item>() {
                                let mut merged = exist_comp.clone();
                                merged.quantity += qty_to_add;
                                if let Some(mut e) = world.entry(target) {
                                    e.add_component(merged);
                                }
                                world.remove(item);
                            }
                        }
                        log.add(
                            "You take the item out and it merges with your items.",
                            log.current_turn,
                        );
                    } else {
                        let mut player_query =
                            <&mut Inventory>::query().filter(component::<PlayerTag>());
                        if let Some(p_inv) = player_query.iter_mut(world).next() {
                            p_inv.items.push(item);
                            p_inv.assign_letter(item);
                            log.add("You take the item out of the container.", log.current_turn);
                        }
                    }
                }
            }
            return;
        }
        ItemAction::PutIn { container, item } => {
            // 2. 마법 가방(Bag of Holding) 폭발 체크 (Phase 48.2 - 정규화)
            if crate::core::systems::inventory::InventorySystem::check_boh_explosion(
                world, container, item,
            ) {
                log.add_colored(
                    "Magical energies clash! The bag explodes!",
                    [255, 50, 50], // Red
                    log.current_turn,
                );

                // [v3.0.0] 먼저 아이템 목록 수집 후 entry_ref drop 후 제거
                let items_to_remove: Vec<Entity> = if let Ok(c_entry) = world.entry_ref(container) {
                    if let Ok(c_inv) = c_entry.get_component::<Inventory>() {
                        c_inv.items.clone()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                for inner_item in items_to_remove {
                    world.remove(inner_item);
                }

                world.remove(container);
                world.remove(item);

                let mut query = <&mut Inventory>::query().filter(component::<PlayerTag>());
                if let Some(inv) = query.iter_mut(world).next() {
                    inv.items.retain(|&e| e != container && e != item);
                }

                log.add(
                    "Everything inside is lost in another dimension!",
                    log.current_turn,
                );
                return;
            }

            // 플레이어 인벤토리에서 아이템 제거
            let mut query = <&mut Inventory>::query().filter(component::<PlayerTag>());
            if let Some(p_inv) = query.iter_mut(world).next() {
                if p_inv.items.contains(&item) {
                    p_inv.remove_item(item);
                }
            }

            // [v2.21.0 R9-3] 컨테이너 인벤토리 안에서 Stacking
            let mut merge_target = None;
            let mut qty_to_add = 0;
            if let Ok(item_entry) = world.entry_ref(item) {
                if let Ok(item_comp) = item_entry.get_component::<Item>() {
                    qty_to_add = item_comp.quantity;
                    if let Ok(c_entry) = world.entry_ref(container) {
                        if let Ok(c_inv) = c_entry.get_component::<Inventory>() {
                            merge_target = crate::core::systems::item_helper::ItemHelper::try_find_merge_target(c_inv, item_comp, world);
                        }
                    }
                }
            }

            if let Some(target) = merge_target {
                if let Ok(target_entry) = world.entry_ref(target) {
                    if let Ok(exist_comp) = target_entry.get_component::<Item>() {
                        let mut merged = exist_comp.clone();
                        merged.quantity += qty_to_add;
                        if let Some(mut e) = world.entry(target) {
                            e.add_component(merged);
                        }
                        world.remove(item);
                        log.add(
                            "You put the item in and it merges with other items.",
                            log.current_turn,
                        );
                    }
                }
            } else {
                if let Some(mut entry) = world.entry(container) {
                    if let Ok(c_inv) = entry.get_component_mut::<Inventory>() {
                        c_inv.items.push(item);
                        // [v3.0.0] 직접 접근 불가 (entry 대출 중) — drop 후 접근
                        drop(entry);
                        if let Some(mut e2) = world.entry(item) {
                            e2.add_component(crate::core::entity::InContainerTag { container });
                        }
                        log.add("You put the item into the container.", log.current_turn);
                    }
                }
            }
            return;
        }
        ItemAction::Apply(e) => {
            crate::core::systems::apply::item_apply(
                e,
                world,
                _assets,
                grid,
                log,
                log.current_turn,
                state,
            );
            return;
        }
        ItemAction::Name(e) => {
            *state = crate::core::game_state::GameState::Naming {
                entity: Some(e),
                is_call: false,
            };
            return;
        }
        ItemAction::Call(e) => {
            *state = crate::core::game_state::GameState::Naming {
                entity: Some(e),
                is_call: true,
            };
            return;
        }
        _ => {}
    }

    // 아이템 정보 수집 (Borrow 충돌 방지)
    use crate::core::systems::item_helper::ItemHelper;
    let (item_inst, display_name, item_class) = match world.entry_ref(item_ent) {
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
                // [v3.0.0] deferred — 쿼리 빌림 종료 후 적용
                deferred_add_item = Some((item_ent, item_inst.clone()));
                deferred_consume = Some(item_ent);
                p_inv.remove_item(item_ent);
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
                // [v3.0.0] deferred — 쿼리 빌림 종료 후 적용
                deferred_add_item = Some((item_ent, item_inst.clone()));
                deferred_consume = Some(item_ent);
                p_inv.remove_item(item_ent);
            }
            ItemAction::Read(_) => {
                log.add(format!("You read a {}.", display_name), log.current_turn);
                match item_name.as_str() {
                    "Scroll of teleportation" => {
                        let mut rng = NetHackRng::new(log.current_turn);
                        if let Some(target) = player_ent {
                            // [v3.0.0] deferred — 쿼리 빌림 중 world.entry 불가
                            deferred_teleport = Some((target, p_status.has(StatusFlags::CONFUSED)
                                || rng.rn2(10) == 0));
                        }
                    }
                    "Scroll of level teleportation" => {
                        if let Some(target) = player_ent {
                            deferred_teleport = Some((target, true));
                        }
                    }
                    "Scroll of magic mapping" => {
                        log.add("A map coalesces in your mind!", log.current_turn);
                        if let Some(p_ent) = player_ent {
                            deferred_magic_map = Some(p_ent);
                        }
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
                                scroll: item_ent,
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
                // [v3.0.0] deferred — 쿼리 빌림 종료 후 적용
                deferred_add_item = Some((item_ent, item_inst.clone()));
                deferred_consume = Some(item_ent);
                p_inv.remove_item(item_ent);
            }
            ItemAction::Offer(item_ent) => {
                let mut rng = NetHackRng::new(log.current_turn);
                if let Some(new_align) = crate::core::systems::pray::try_offer(
                    item_ent,
                    world,
                    grid,
                    _assets,
                    &mut rng,
                    log,
                    log.current_turn,
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
                    if inv.items.contains(&item_ent) {
                        inv.remove_item(item_ent);
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
                                    item_ent,
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
                            if let Some(mut e) = world.entry(item_ent) {
                                e.add_component(Position { x: px, y: py });
                            }
                        }
                    } else {
                        //
                        // SubWorld에서는 remove가 안 될 수 있음.
                        //
                        //
                        world.remove(item_ent);
                    }
                }
            }
            ItemAction::Rub(_) => {
                log.add("You rub it.", log.current_turn);
            }
            ItemAction::Dip(_) => {
                log.add("You dip it.", log.current_turn);
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
                if let Some(mut e) = world.entry(target_ent) {
                    e.add_component(identified);
                }
                log.add("You identify an item.", log.current_turn);
            }
        }
    }

    // [v3.0.0] deferred ops 적용 (쿼리 빌림 해제 후)
    if let Some((ent, comp)) = deferred_add_item {
        if let Some(mut e) = world.entry(ent) {
            e.add_component(comp);
        }
    }
    if let Some(ent) = deferred_consume {
        world.remove(ent);
    }
    if let Some((target, is_level_tele)) = deferred_teleport {
        if let Some(mut e) = world.entry(target) {
            e.add_component(crate::core::systems::teleport::TeleportAction {
                target,
                is_level_tele,
            });
        }
    }
    if let Some(p_ent) = deferred_magic_map {
        if let Some(mut e) = world.entry(p_ent) {
            e.add_component(crate::core::entity::MagicMapRequest);
        }
    }
}

fn consume_item(inv: &mut Inventory, item_ent: Entity, world: &mut legion::world::World) {
    if inv.items.contains(&item_ent) {
        inv.remove_item(item_ent);
    }
    world.remove(item_ent);
}

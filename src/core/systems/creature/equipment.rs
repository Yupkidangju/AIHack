// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::assets::AssetManager;
use crate::core::entity::object::ItemClass;
use crate::core::entity::{Equipment, EquipmentSlot, Inventory, Item, PlayerTag};
use crate::core::events::{EventQueue, GameEvent}; // [v2.0.0 R5] 장비 이벤트 발행
use crate::core::systems::item_use::ItemAction;
use crate::ui::log::GameLog;
use legion::world::SubWorld;
use legion::{component, Entity, EntityStore, IntoQuery};

#[legion::system]
#[read_component(PlayerTag)]
#[read_component(Item)]
#[write_component(Inventory)]
#[write_component(Equipment)]
#[write_component(crate::core::entity::Health)]
#[write_component(crate::core::entity::player::Player)]
#[read_component(crate::core::entity::ItemTag)]
pub fn equipment(
    world: &mut SubWorld,
    #[resource] assets: &AssetManager,
    #[resource] action: &mut Option<ItemAction>,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] event_queue: &mut EventQueue, // [v2.0.0 R5] 장비 이벤트
) {
    let current_action = match *action {
        Some(a) => a,
        None => return,
    };

    let item_ent = match current_action {
        ItemAction::Wield(e)
        | ItemAction::Wear(e)
        | ItemAction::TakeOff(e)
        | ItemAction::EquipOffhand(e)
        | ItemAction::QuiverSelect(e) => e,
        _ => return,
    };

    // ... (item info collection) ...

    let item_info = if let Ok(entry) = world.entry_ref(item_ent) {
        entry.get_component::<Item>().ok().cloned()
    } else {
        None
    };

    let (item_template_name, artifact_id) = match &item_info {
        Some(i) => (Some(i.kind.to_string()), i.artifact.clone()),
        None => (None, None),
    };

    // ...

    let template = if let Some(name) = &item_template_name {
        assets.items.get_template(name)
    } else {
        None
    };

    let mut query = <(
        &mut Inventory,
        &mut Equipment,
        &mut crate::core::entity::player::Player,
        &mut crate::core::entity::Health,
    )>::query()
    .filter(component::<PlayerTag>());

    if let Some((inventory, equipment, player, health)) = query.iter_mut(world).next() {
        // ... (artifact check) ...
        if let Some(art_id) = artifact_id {
            if let Some(art_tmpl) = assets.artifacts.get_artifact(&art_id) {
                if art_tmpl.alignment != player.alignment {
                    log.add_colored(
                        format!("The {} blasts you!", art_tmpl.name),
                        [255, 100, 100],
                        *turn,
                    );
                    health.current -= 10;
                    if health.current <= 0 {
                        log.add("You died from the blast...", *turn);
                    }
                    *action = None;
                    return;
                }
            }
        }

        match current_action {
            ItemAction::Wield(_) => {
                // ...
                if let Some(t) = template {
                    if t.class == ItemClass::Weapon {
                        equip_item(
                            item_ent,
                            EquipmentSlot::Melee,
                            t.name.clone(),
                            inventory,
                            equipment,
                            log,
                            *turn,
                            event_queue,
                        );
                        *action = None;
                    } else {
                        log.add("You can only wield weapons.", *turn);
                        *action = None;
                    }
                }
            }
            ItemAction::EquipOffhand(_) => {
                if let Some(t) = template {
                    if t.class == ItemClass::Weapon {
                        // Check Shield slot
                        if equipment.slots.contains_key(&EquipmentSlot::Shield) {
                            log.add("You cannot dual-wield while wearing a shield.", *turn);
                            *action = None;
                            return;
                        }

                        equip_item(
                            item_ent,
                            EquipmentSlot::Offhand,
                            t.name.clone(),
                            inventory,
                            equipment,
                            log,
                            *turn,
                            event_queue,
                        );
                        player.two_weapon = true;
                        *action = None;
                    } else {
                        log.add("You can only wield weapons in your off-hand.", *turn);
                        *action = None;
                    }
                }
            }
            ItemAction::Wear(_) => {
                // ... (wear logic) ...
                if let Some(t) = template {
                    if t.class == ItemClass::Armor {
                        let slot = match t.subtype {
                            0 => Some(EquipmentSlot::Body),
                            1 => Some(EquipmentSlot::Shield),
                            2 => Some(EquipmentSlot::Head),
                            3 => Some(EquipmentSlot::Hands),
                            4 => Some(EquipmentSlot::Feet),
                            5 => Some(EquipmentSlot::Cloak),
                            _ => None,
                        };
                        if let Some(s) = slot {
                            // Check Offhand if trying to wear Shield
                            if s == EquipmentSlot::Shield
                                && equipment.slots.contains_key(&EquipmentSlot::Offhand)
                            {
                                log.add("You cannot wear a shield while dual-wielding.", *turn);
                                *action = None;
                                return;
                            }

                            equip_item(
                                item_ent,
                                s,
                                t.name.clone(),
                                inventory,
                                equipment,
                                log,
                                *turn,
                                event_queue,
                            );
                            *action = None;
                        } else {
                            log.add("You cannot wear that.", *turn);
                            *action = None;
                        }
                    } else {
                        log.add("You can only wear armor.", *turn);
                        *action = None;
                    }
                }
            }
            ItemAction::QuiverSelect(_) => {
                if let Some(t) = template {
                    // NetHack allows Quivering weapons, tools (wands), or food (sometimes)
                    // But primarily Weapons, Tools, Gem, or Food
                    if t.class == ItemClass::Weapon
                        || t.class == ItemClass::Tool
                        || t.class == ItemClass::Gem
                        || t.class == ItemClass::Food
                    {
                        equip_item(
                            item_ent,
                            EquipmentSlot::Quiver,
                            t.name.clone(),
                            inventory,
                            equipment,
                            log,
                            *turn,
                            event_queue,
                        );
                        *action = None;
                    } else {
                        log.add("You cannot quiver that.", *turn);
                        *action = None;
                    }
                }
            }
            ItemAction::TakeOff(_) => {
                unequip_item(item_ent, inventory, equipment, log, *turn, event_queue);
                *action = None;
            }
            _ => {}
        }
    }
}

fn equip_item(
    item: Entity,
    slot: EquipmentSlot,
    name: String,
    inventory: &mut Inventory,
    equipment: &mut Equipment,
    log: &mut GameLog,
    turn: u64,
    event_queue: &mut EventQueue, // [v2.0.0 R5] 이벤트 발행
) {
    if equipment.slots.contains_key(&slot) {
        log.add(
            format!(
                "You are already equipping something in the {:?} slot.",
                slot
            ),
            turn,
        );
        return;
    }

    if let Some(pos) = inventory.items.iter().position(|&e| e == item) {
        inventory.items.remove(pos);
        equipment.slots.insert(slot, item);
        log.add(format!("You equip {}.", name), turn);

        // [v2.0.0 R5] ItemEquipped + EquipmentChanged 이벤트 발행
        event_queue.push(GameEvent::ItemEquipped {
            item_name: name,
            slot: format!("{:?}", slot),
        });
        event_queue.push(GameEvent::EquipmentChanged);
    } else {
        log.add("You don't have that item.", turn);
    }
}

fn unequip_item(
    item: Entity,
    inventory: &mut Inventory,
    equipment: &mut Equipment,
    log: &mut GameLog,
    turn: u64,
    event_queue: &mut EventQueue, // [v2.0.0 R5] 이벤트 발행
) {
    let mut slot_to_remove = None;
    for (slot, entity) in &equipment.slots {
        if *entity == item {
            slot_to_remove = Some(*slot);
            break;
        }
    }

    if let Some(slot) = slot_to_remove {
        equipment.slots.remove(&slot);
        inventory.items.push(item);
        log.add("You take off the item.", turn);

        // [v2.0.0 R5] ItemUnequipped + EquipmentChanged 이벤트 발행
        event_queue.push(GameEvent::ItemUnequipped {
            item_name: "item".to_string(), // TODO: 실제 아이템 이름 전달
            slot: format!("{:?}", slot),
        });
        event_queue.push(GameEvent::EquipmentChanged);
    } else {
        log.add("You are not wearing that.", turn);
    }
}

///
#[legion::system]
#[read_component(PlayerTag)]
#[read_component(Equipment)]
#[read_component(Item)]
#[write_component(crate::core::entity::CombatStats)]
#[write_component(crate::core::entity::status::StatusBundle)]
#[write_component(crate::core::entity::player::Player)]
pub fn update_player_stats(world: &mut SubWorld, #[resource] assets: &AssetManager) {
    // 1. 장착된 아이템 ID들 수집 (Borrow 충돌 방지)
    let mut equipped_items = Vec::new();
    let mut p_query = <&Equipment>::query().filter(component::<PlayerTag>());
    for equipment in p_query.iter(world) {
        for item_ent in equipment.slots.values() {
            equipped_items.push(*item_ent);
        }
    }

    // 2. 아이템 효과 계산
    let mut total_ac_bonus = 0;
    let mut artifact_resists = crate::core::entity::status::StatusFlags::empty();
    // [v2.0.0] worn.rs 장비 효과 합산
    let mut total_effects = crate::core::systems::worn::EquipmentEffects::new();

    for item_ent in equipped_items {
        if let Ok(entry) = world.entry_ref(item_ent) {
            if let Ok(item) = entry.get_component::<Item>() {
                // 일반 아이템 속성 (AC)
                if let Some(template) = assets.items.get_by_kind(item.kind) {
                    if template.class == ItemClass::Armor {
                        let base_bonus = template.oc1 as i32;
                        let erosion = (item.oeroded as i32) + (item.oeroded2 as i32);
                        let actual_bonus = (base_bonus - erosion).max(0);
                        total_ac_bonus += actual_bonus;
                    }

                    // [v2.0.0] worn.rs 장비 효과 계산 (방어구/반지/부적)
                    let blessed = item.blessed;
                    let cursed = item.cursed;
                    let spe = item.spe as i32;
                    let fx = match template.class {
                        ItemClass::Armor => crate::core::systems::worn::armor_effects(
                            item.kind.as_str(),
                            spe,
                            blessed,
                            cursed,
                        ),
                        ItemClass::Ring => crate::core::systems::worn::ring_effects(
                            item.kind.as_str(),
                            spe,
                            blessed,
                            cursed,
                        ),
                        ItemClass::Amulet => crate::core::systems::worn::amulet_effects(
                            item.kind.as_str(),
                            blessed,
                            cursed,
                        ),
                        _ => crate::core::systems::worn::EquipmentEffects::new(),
                    };
                    total_effects.combine(&fx);
                }

                //
                if let Some(art_id) = &item.artifact {
                    if let Some(art) = assets.artifacts.get_artifact(art_id) {
                        for &res in &art.resists {
                            artifact_resists.insert(res);
                        }
                    }
                }
            }
        }
    }

    //
    let mut query = <(
        &mut crate::core::entity::CombatStats,
        &mut crate::core::entity::status::StatusBundle,
        &mut crate::core::entity::player::Player,
    )>::query()
    .filter(component::<PlayerTag>());
    for (stats, status, player) in query.iter_mut(world) {
        stats.ac = 10 - total_ac_bonus;

        //
        status.permanent = artifact_resists;

        // [v2.0.0] 장비 배고픔 보너스 계산 (원본 worn.c: update_mon_intrinsics)
        // hunger: +1 (Ring of Hunger 등)
        // regeneration: +1 (재생 장비)
        // slow_digestion: -1 (소화 감속)
        let mut hunger_mod = 0i32;
        if total_effects.hunger {
            hunger_mod += 1; // 배고픔 가속
        }
        if total_effects.regeneration {
            hunger_mod += 1; // 재생 시 추가 소모
        }
        if total_effects.slow_digestion {
            hunger_mod -= 1; // 소화 감속
        }
        player.equip_hunger_bonus = hunger_mod;
    }
}

// =============================================================================
// [v2.3.1] do_wear.c 확장 이식
// 원본: nethack-3.6.7/src/do_wear.c (2,663줄)
//
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorCategory {
    Body,
    Shield,
    Helmet,
    Gloves,
    Boots,
    Cloak,
}

/// [v2.3.1] 장비 착용 순서 제약 (원본: cantweararm)
pub fn wearing_blocks_removal(
    removing_slot: EquipmentSlot,
    equipment: &Equipment,
) -> Option<&'static str> {
    match removing_slot {
        EquipmentSlot::Body => {
            if equipment.slots.contains_key(&EquipmentSlot::Cloak) {
                Some("You can't take off your body armor while wearing a cloak.")
            } else {
                None
            }
        }
        _ => None,
    }
}

/// [v2.3.1] 저주된 장비 탈착 불가 (원본: cursed check)
pub fn is_cursed_equipped(item_cursed: bool) -> bool {
    item_cursed
}

/// [v2.3.1] 장비 착용 턴 수 (원본: delay_wearing)
pub fn wearing_delay(category: ArmorCategory) -> i32 {
    match category {
        ArmorCategory::Body => 5,
        ArmorCategory::Shield => 1,
        ArmorCategory::Helmet => 1,
        ArmorCategory::Gloves => 1,
        ArmorCategory::Boots => 1,
        ArmorCategory::Cloak => 1,
    }
}

/// [v2.3.1] 장비 부식 레벨 설명 (원본: describe_erosion)
pub fn erosion_description(erodeamt: i32) -> &'static str {
    match erodeamt {
        0 => "",
        1 => "corroded",
        2 => "very corroded",
        _ => "thoroughly corroded",
    }
}

/// [v2.3.1] 방어구 정보 조회용 명세
#[derive(Debug, Clone)]
pub struct ArmorInfo {
    pub name: &'static str,
    pub category: ArmorCategory,
    pub base_ac: i32,
    pub magic_cancellation: i32,
    pub weight: i32,
    pub special_property: Option<&'static str>,
}

/// [v2.3.1] 드래곤 비늘 갑옷 속성 (원본: dragon_scales)
pub fn dragon_scale_property(dragon_type: &str) -> Option<&'static str> {
    match dragon_type {
        s if s.contains("red") => Some("fire_resistance"),
        s if s.contains("white") => Some("cold_resistance"),
        s if s.contains("blue") => Some("shock_resistance"),
        s if s.contains("orange") => Some("sleep_resistance"),
        s if s.contains("black") => Some("disintegration_resistance"),
        s if s.contains("yellow") => Some("acid_resistance"),
        s if s.contains("green") => Some("poison_resistance"),
        s if s.contains("gray") | s.contains("grey") => Some("magic_resistance"),
        s if s.contains("silver") => Some("reflection"),
        _ => None,
    }
}

/// [v2.3.1] 변신 시 장비 탈락 판정 (원본: break_armor)
pub fn polymorph_drops_armor(
    new_form_is_small: bool,
    new_form_no_hands: bool,
    new_form_no_head: bool,
) -> Vec<EquipmentSlot> {
    let mut dropped = Vec::new();
    if new_form_is_small {
        dropped.push(EquipmentSlot::Body);
        dropped.push(EquipmentSlot::Cloak);
    }
    if new_form_no_hands {
        dropped.push(EquipmentSlot::Melee);
        dropped.push(EquipmentSlot::Hands);
        dropped.push(EquipmentSlot::Shield);
    }
    if new_form_no_head {
        dropped.push(EquipmentSlot::Head);
    }
    dropped
}

///
pub fn magic_cancellation_bonus(mc: i32) -> i32 {
    mc.clamp(0, 3)
}

///
pub fn armor_speed_penalty(total_armor_weight: i32) -> i32 {
    if total_armor_weight > 300 {
        -3
    } else if total_armor_weight > 200 {
        -2
    } else if total_armor_weight > 100 {
        -1
    } else {
        0
    }
}

/// [v2.3.1] 주요 방어구 데이터 테이블 (원본: objects[])
pub fn armor_data_table() -> Vec<ArmorInfo> {
    vec![
        ArmorInfo {
            name: "leather armor",
            category: ArmorCategory::Body,
            base_ac: 2,
            magic_cancellation: 0,
            weight: 150,
            special_property: None,
        },
        ArmorInfo {
            name: "ring mail",
            category: ArmorCategory::Body,
            base_ac: 3,
            magic_cancellation: 0,
            weight: 250,
            special_property: None,
        },
        ArmorInfo {
            name: "scale mail",
            category: ArmorCategory::Body,
            base_ac: 4,
            magic_cancellation: 0,
            weight: 250,
            special_property: None,
        },
        ArmorInfo {
            name: "chain mail",
            category: ArmorCategory::Body,
            base_ac: 5,
            magic_cancellation: 1,
            weight: 300,
            special_property: None,
        },
        ArmorInfo {
            name: "banded mail",
            category: ArmorCategory::Body,
            base_ac: 6,
            magic_cancellation: 0,
            weight: 350,
            special_property: None,
        },
        ArmorInfo {
            name: "splint mail",
            category: ArmorCategory::Body,
            base_ac: 6,
            magic_cancellation: 1,
            weight: 400,
            special_property: None,
        },
        ArmorInfo {
            name: "plate mail",
            category: ArmorCategory::Body,
            base_ac: 7,
            magic_cancellation: 2,
            weight: 450,
            special_property: None,
        },
        ArmorInfo {
            name: "crystal plate mail",
            category: ArmorCategory::Body,
            base_ac: 7,
            magic_cancellation: 2,
            weight: 450,
            special_property: None,
        },
        ArmorInfo {
            name: "mithril-coat",
            category: ArmorCategory::Body,
            base_ac: 5,
            magic_cancellation: 3,
            weight: 150,
            special_property: None,
        },
        ArmorInfo {
            name: "small shield",
            category: ArmorCategory::Shield,
            base_ac: 1,
            magic_cancellation: 0,
            weight: 30,
            special_property: None,
        },
        ArmorInfo {
            name: "large shield",
            category: ArmorCategory::Shield,
            base_ac: 2,
            magic_cancellation: 0,
            weight: 100,
            special_property: None,
        },
        ArmorInfo {
            name: "shield of reflection",
            category: ArmorCategory::Shield,
            base_ac: 2,
            magic_cancellation: 0,
            weight: 50,
            special_property: Some("reflection"),
        },
        ArmorInfo {
            name: "helm of brilliance",
            category: ArmorCategory::Helmet,
            base_ac: 1,
            magic_cancellation: 0,
            weight: 50,
            special_property: Some("int_wis_bonus"),
        },
        ArmorInfo {
            name: "helm of telepathy",
            category: ArmorCategory::Helmet,
            base_ac: 1,
            magic_cancellation: 0,
            weight: 50,
            special_property: Some("telepathy"),
        },
        ArmorInfo {
            name: "gauntlets of power",
            category: ArmorCategory::Gloves,
            base_ac: 1,
            magic_cancellation: 0,
            weight: 30,
            special_property: Some("str_25"),
        },
        ArmorInfo {
            name: "gauntlets of dexterity",
            category: ArmorCategory::Gloves,
            base_ac: 1,
            magic_cancellation: 0,
            weight: 10,
            special_property: Some("dex_bonus"),
        },
        ArmorInfo {
            name: "speed boots",
            category: ArmorCategory::Boots,
            base_ac: 1,
            magic_cancellation: 0,
            weight: 20,
            special_property: Some("speed"),
        },
        ArmorInfo {
            name: "levitation boots",
            category: ArmorCategory::Boots,
            base_ac: 1,
            magic_cancellation: 0,
            weight: 15,
            special_property: Some("levitation"),
        },
        ArmorInfo {
            name: "jumping boots",
            category: ArmorCategory::Boots,
            base_ac: 1,
            magic_cancellation: 0,
            weight: 20,
            special_property: Some("jumping"),
        },
        ArmorInfo {
            name: "elven cloak",
            category: ArmorCategory::Cloak,
            base_ac: 1,
            magic_cancellation: 3,
            weight: 10,
            special_property: Some("stealth"),
        },
        ArmorInfo {
            name: "cloak of magic resistance",
            category: ArmorCategory::Cloak,
            base_ac: 1,
            magic_cancellation: 3,
            weight: 10,
            special_property: Some("magic_resistance"),
        },
        ArmorInfo {
            name: "cloak of invisibility",
            category: ArmorCategory::Cloak,
            base_ac: 1,
            magic_cancellation: 2,
            weight: 10,
            special_property: Some("invisibility"),
        },
        ArmorInfo {
            name: "cloak of displacement",
            category: ArmorCategory::Cloak,
            base_ac: 1,
            magic_cancellation: 2,
            weight: 10,
            special_property: Some("displacement"),
        },
        ArmorInfo {
            name: "cloak of protection",
            category: ArmorCategory::Cloak,
            base_ac: 3,
            magic_cancellation: 3,
            weight: 10,
            special_property: None,
        },
    ]
}

///
pub fn get_armor_info(name: &str) -> Option<ArmorInfo> {
    let lower = name.to_lowercase();
    armor_data_table()
        .into_iter()
        .find(|a| lower.contains(a.name))
}

// =============================================================================
// [v2.3.5] 장비 확장 (원본 worn.c/objnam.c: advanced equipment)
// =============================================================================

/// 방어구 재료 (원본: objects.h material)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorMaterial {
    Leather,    // 가죽
    Iron,       // 철
    Steel,      // 강철
    Mithril,    // 미스릴
    DragonHide, // 드래곤 가죽
    Cloth,      // 천
    Wood,       // 나무
    Bone,       // 뼈
    Crystal,    // 크리스탈
    Silver,     // 은
}

/// 재료별 무게 보정 (원본: material weight factor)
pub fn material_weight_factor(material: ArmorMaterial) -> f32 {
    match material {
        ArmorMaterial::Cloth => 0.3,
        ArmorMaterial::Leather => 0.6,
        ArmorMaterial::Wood => 0.8,
        ArmorMaterial::Bone => 0.9,
        ArmorMaterial::Iron => 1.0,
        ArmorMaterial::Steel => 1.1,
        ArmorMaterial::Crystal => 1.3,
        ArmorMaterial::Silver => 1.2,
        ArmorMaterial::Mithril => 0.5,
        ArmorMaterial::DragonHide => 0.7,
    }
}

/// 재료별 부식 저항 (원본: material erodeproof check)
pub fn material_rust_resistant(material: ArmorMaterial) -> bool {
    matches!(
        material,
        ArmorMaterial::Mithril
            | ArmorMaterial::Crystal
            | ArmorMaterial::DragonHide
            | ArmorMaterial::Cloth
            | ArmorMaterial::Leather
            | ArmorMaterial::Wood
    )
}

/// 장비 강화 비용 (원본: enchant armor cost)
pub fn enchant_armor_cost(current_enchant: i32, material: ArmorMaterial) -> u32 {
    let base = ((current_enchant + 1) * 100) as u32;
    let material_mult = match material {
        ArmorMaterial::Mithril => 3,
        ArmorMaterial::DragonHide => 4,
        ArmorMaterial::Crystal => 5,
        _ => 2,
    };
    base * material_mult
}

/// 방어구 세트 효과 (원본: set bonus)
pub fn armor_set_bonus(pieces_worn: i32) -> i32 {
    match pieces_worn {
        0..=1 => 0,
        2..=3 => 1,
        4..=5 => 2,
        6..=7 => 4,
        _ => 5,
    }
}

/// 장비 수리 턴 수 (원본: repair armor)
pub fn repair_turns(erosion: i32, material: ArmorMaterial) -> i32 {
    let base = erosion * 3;
    let bonus = if material_rust_resistant(material) {
        -1
    } else {
        1
    };
    (base + bonus).max(1)
}

/// 최적 방어구 추천 (원본: best armor by AC)
pub fn best_armor_ac(armors: &[ArmorInfo]) -> Option<&ArmorInfo> {
    armors
        .iter()
        .max_by_key(|a| a.base_ac + a.magic_cancellation)
}

/// 장비 통계
#[derive(Debug, Clone, Default)]
pub struct EquipmentStatistics {
    pub items_worn: u32,
    pub items_removed: u32,
    pub items_enchanted: u32,
    pub items_eroded: u32,
    pub items_repaired: u32,
    pub total_ac_bonus: i32,
}

impl EquipmentStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_wear(&mut self) {
        self.items_worn += 1;
    }
    pub fn record_enchant(&mut self) {
        self.items_enchanted += 1;
    }
    pub fn record_erode(&mut self) {
        self.items_eroded += 1;
    }
}

// =============================================================================
// [v2.4.0] worn.c 핵심 이식 ? 몬스터 장비 시스템
// 원본: nethack-3.6.7/src/worn.c (1,031줄)
//
//
// 이 섹션은 몬스터 장비 AI, AC 계산, 속도 조정, 외재 속성, 변신 시 장비 파괴 등
//
// =============================================================================

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

///
pub const W_ARM: u32 = 0x00000001; // 갑옷 (body armor)
pub const W_ARMC: u32 = 0x00000002; // 망토 (cloak)
pub const W_ARMH: u32 = 0x00000004; // 투구 (helmet)
pub const W_ARMS: u32 = 0x00000008; // 방패 (shield)
pub const W_ARMG: u32 = 0x00000010; // 장갑 (gloves)
pub const W_ARMF: u32 = 0x00000020; // 장화 (boots)
pub const W_ARMU: u32 = 0x00000040; // 셔츠 (shirt)
pub const W_RINGL: u32 = 0x00000080; // 왼손 반지
pub const W_RINGR: u32 = 0x00000100; // 오른손 반지
pub const W_WEP: u32 = 0x00000200; // 주무기
pub const W_SWAPWEP: u32 = 0x00000400; // 교체 무기
pub const W_QUIVER: u32 = 0x00000800; // 화살통
pub const W_AMUL: u32 = 0x00001000; // 부적
pub const W_TOOL: u32 = 0x00002000; // 도구 (눈가리개 등)
pub const W_SADDLE: u32 = 0x00004000; // 안장
pub const W_BALL: u32 = 0x00008000; // 족쇄 공
pub const W_CHAIN: u32 = 0x00010000; // 족쇄 사슬

///
pub const W_ARMOR: u32 = W_ARM | W_ARMC | W_ARMH | W_ARMS | W_ARMG | W_ARMF | W_ARMU;

///
pub const W_RING: u32 = W_RINGL | W_RINGR;

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

///
///
///
/// is_mergeable: 탄약류 여부, tool_name: 도구 이름
pub fn wearslot(class: &str, subtype: u8, is_mergeable: bool, tool_name: &str) -> u32 {
    match class {
        // 부적 → 부적 슬롯
        "amulet" => W_AMUL,
        // 반지 → 양손
        "ring" => W_RING,
        //
        "armor" => match subtype {
            0 => W_ARM,  // ARM_SUIT (body)
            1 => W_ARMS, // ARM_SHIELD
            2 => W_ARMH, // ARM_HELM
            3 => W_ARMG, // ARM_GLOVES
            4 => W_ARMF, // ARM_BOOTS
            5 => W_ARMC, // ARM_CLOAK
            6 => W_ARMU, // ARM_SHIRT
            _ => 0,
        },
        // 무기 → 주무기 + 교체 무기, 탄약은 화살통 추가
        "weapon" => {
            let mut res = W_WEP | W_SWAPWEP;
            if is_mergeable {
                res |= W_QUIVER;
            }
            res
        }
        //
        "tool" => {
            let lower = tool_name.to_lowercase();
            if lower.contains("blindfold") || lower.contains("towel") || lower.contains("lenses") {
                W_TOOL
            } else if lower.contains("saddle") {
                W_SADDLE
            } else {
                //
                W_WEP | W_SWAPWEP
            }
        }
        // 보석 → 화살통 (투석기 탄환)
        "gem" => W_QUIVER,
        // 공 → 족쇄 공
        "ball" => W_BALL,
        // 사슬 → 족쇄 사슬
        "chain" => W_CHAIN,
        // 음식 중 "meat ring" → 반지 슬롯
        "food" => {
            if tool_name.to_lowercase().contains("meat ring") {
                W_RING
            } else {
                0
            }
        }
        _ => 0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

///
///
/// 실제 ARM_BONUS는 spe + oc_oprop의 AC 보정을 포함
///
pub fn find_mac(base_ac: i32, worn_items: &[(bool, i32)]) -> i32 {
    let mut ac = base_ac;
    for (is_worn, arm_bonus) in worn_items {
        if *is_worn {
            //
            ac -= arm_bonus;
        }
    }
    ac
}

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

/// 몬스터 속도 상수 (원본: MSLOW, MFAST)
pub const MSLOW: u8 = 1;
pub const MFAST: u8 = 2;

/// [v2.4.0] 몬스터 속도 조정 결과
#[derive(Debug, Clone)]
pub struct SpeedAdjustResult {
    /// 새 영구 속도 (0=보통, MSLOW=감속, MFAST=가속)
    pub new_permspeed: u8,
    /// 새 현재 속도 (speed boots 여부에 따라 달라짐)
    pub new_mspeed: u8,
    /// 메시지 (가시 범위에 있을 때)
    pub message: Option<String>,
}

///
/// adjust: +2(생성시 FAST), +1(가속), 0(부츠만 체크), -1(감속), -2(생성시 SLOW),
///         -3(석화), -4(녹색 슬라임)
///
/// mon_name: 몬스터 이름, base_mmove: 기본 이동력, is_frozen_or_sleeping: 이동 불가 상태
pub fn mon_adjust_speed(
    adjust: i32,
    current_permspeed: u8,
    has_speed_boots: bool,
    mon_name: &str,
    base_mmove: i32,
    is_frozen_or_sleeping: bool,
) -> SpeedAdjustResult {
    let mut permspeed = current_permspeed;
    let mut give_msg = true;
    let petrify;

    match adjust {
        2 => {
            // 생성 시 FAST 설정 (메시지 없음)
            permspeed = MFAST;
            give_msg = false;
            petrify = false;
        }
        1 => {
            // 가속: SLOW→보통, 보통→FAST
            if permspeed == MSLOW {
                permspeed = 0;
            } else {
                permspeed = MFAST;
            }
            petrify = false;
        }
        0 => {
            // 부츠 체크만 (속도 변경 없음)
            petrify = false;
        }
        -1 => {
            // 감속: FAST→보통, 보통→SLOW
            if permspeed == MFAST {
                permspeed = 0;
            } else {
                permspeed = MSLOW;
            }
            petrify = false;
        }
        -2 => {
            // 생성 시 SLOW (메시지 없음)
            permspeed = MSLOW;
            give_msg = false;
            petrify = false;
        }
        -3 => {
            // 석화: 고유 가속만 제거 (기본 속도는 유지)
            if permspeed == MFAST {
                permspeed = 0;
            }
            petrify = true;
        }
        -4 => {
            // 녹색 슬라임 (메시지 없음)
            if permspeed == MFAST {
                permspeed = 0;
            }
            give_msg = false;
            petrify = false;
        }
        _ => {
            petrify = false;
        }
    }

    // 속도 부츠가 있으면 현재 속도는 무조건 FAST
    let mspeed = if has_speed_boots { MFAST } else { permspeed };

    // 메시지 생성 (가시 범위, 이동 가능, 속도 변동 시)
    let message = if give_msg
        && (mspeed != current_permspeed || petrify)
        && base_mmove > 0
        && !is_frozen_or_sleeping
    {
        if petrify {
            Some(format!("{} is slowing down.", mon_name))
        } else {
            let howmuch =
                if (mspeed as i32 + current_permspeed as i32) == (MFAST as i32 + MSLOW as i32) {
                    "much "
                } else {
                    ""
                };
            if adjust > 0 || mspeed == MFAST {
                Some(format!(
                    "{} is suddenly moving {}faster.",
                    mon_name, howmuch
                ))
            } else {
                Some(format!(
                    "{} seems to be moving {}slower.",
                    mon_name, howmuch
                ))
            }
        }
    } else {
        None
    };

    SpeedAdjustResult {
        new_permspeed: permspeed,
        new_mspeed: mspeed,
        message,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.4.0] 장비의 외재 속성 종류 (원본: oc_oprop 값)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicProperty {
    None,
    FireRes,
    ColdRes,
    SleepRes,
    DisintRes,
    ShockRes,
    PoisonRes,
    AcidRes,
    StoneRes,
    Invis,
    Fast,
    Antimagic,
    Reflecting,
    Clairvoyant,
    Stealth,
    Telepathy,
    Levitation,
    WaterWalking,
    Displacement,
    Fumbling,
    Jumping,
    Protection,
}

/// [v2.4.0] 장비 속성 갱신 결과
#[derive(Debug, Clone)]
pub struct IntrinsicUpdateResult {
    ///
    pub resist_mask_change: i16,
    ///
    pub invisibility_change: Option<bool>,
    /// 속도 재계산 필요 여부
    pub need_speed_update: bool,
    /// 가시성 변경에 따른 UI 갱신 필요 여부
    pub need_display_update: bool,
}

///
pub fn property_from_index(idx: u8) -> IntrinsicProperty {
    match idx {
        1 => IntrinsicProperty::FireRes,
        2 => IntrinsicProperty::ColdRes,
        3 => IntrinsicProperty::SleepRes,
        4 => IntrinsicProperty::DisintRes,
        5 => IntrinsicProperty::ShockRes,
        6 => IntrinsicProperty::PoisonRes,
        7 => IntrinsicProperty::AcidRes,
        8 => IntrinsicProperty::StoneRes,
        9 => IntrinsicProperty::Invis,
        10 => IntrinsicProperty::Fast,
        11 => IntrinsicProperty::Antimagic,
        12 => IntrinsicProperty::Reflecting,
        13 => IntrinsicProperty::Clairvoyant,
        14 => IntrinsicProperty::Stealth,
        15 => IntrinsicProperty::Telepathy,
        16 => IntrinsicProperty::Levitation,
        17 => IntrinsicProperty::WaterWalking,
        18 => IntrinsicProperty::Displacement,
        19 => IntrinsicProperty::Fumbling,
        20 => IntrinsicProperty::Jumping,
        21 => IntrinsicProperty::Protection,
        _ => IntrinsicProperty::None,
    }
}

/// [v2.4.0] 장비 착/탈 시 몬스터 외재 속성 갱신 (원본: update_mon_intrinsics)
/// on=true: 장착, on=false: 해제
/// property: 아이템 속성, current_extrinsics: 현재 외재 저항 비트,
/// is_perminvis: 영구 투명 여부, has_other_with_same_property: 같은 속성 다른 아이템 착용 여부
pub fn update_mon_intrinsics(
    on: bool,
    property: IntrinsicProperty,
    current_extrinsics: u16,
    is_perminvis: bool,
    has_other_with_same_property: bool,
) -> IntrinsicUpdateResult {
    let mut result = IntrinsicUpdateResult {
        resist_mask_change: 0,
        invisibility_change: None,
        need_speed_update: false,
        need_display_update: false,
    };

    if property == IntrinsicProperty::None {
        return result;
    }

    if on {
        // 장착 시 속성 부여
        match property {
            IntrinsicProperty::Invis => {
                result.invisibility_change = Some(true);
                result.need_display_update = true;
            }
            IntrinsicProperty::Fast => {
                result.need_speed_update = true;
            }
            //
            IntrinsicProperty::FireRes
            | IntrinsicProperty::ColdRes
            | IntrinsicProperty::SleepRes
            | IntrinsicProperty::DisintRes
            | IntrinsicProperty::ShockRes
            | IntrinsicProperty::PoisonRes
            | IntrinsicProperty::AcidRes
            | IntrinsicProperty::StoneRes => {
                let bit = resist_property_to_bit(property);
                result.resist_mask_change = bit as i16;
            }
            //
            IntrinsicProperty::Antimagic
            | IntrinsicProperty::Reflecting
            | IntrinsicProperty::Clairvoyant
            | IntrinsicProperty::Stealth
            | IntrinsicProperty::Telepathy
            | IntrinsicProperty::Levitation
            | IntrinsicProperty::WaterWalking
            | IntrinsicProperty::Displacement
            | IntrinsicProperty::Fumbling
            | IntrinsicProperty::Jumping
            | IntrinsicProperty::Protection => {}
            IntrinsicProperty::None => {}
        }
    } else {
        // 해제 시 속성 제거
        match property {
            IntrinsicProperty::Invis => {
                //
                result.invisibility_change = Some(is_perminvis);
                result.need_display_update = true;
            }
            IntrinsicProperty::Fast => {
                result.need_speed_update = true;
            }
            IntrinsicProperty::FireRes
            | IntrinsicProperty::ColdRes
            | IntrinsicProperty::SleepRes
            | IntrinsicProperty::DisintRes
            | IntrinsicProperty::ShockRes
            | IntrinsicProperty::PoisonRes
            | IntrinsicProperty::AcidRes
            | IntrinsicProperty::StoneRes => {
                //
                if !has_other_with_same_property {
                    let bit = resist_property_to_bit(property);
                    result.resist_mask_change = -(bit as i16);
                }
            }
            _ => {}
        }
    }

    result
}

/// [v2.4.0] IntrinsicProperty → 저항 비트 변환
fn resist_property_to_bit(prop: IntrinsicProperty) -> u16 {
    match prop {
        IntrinsicProperty::FireRes => 0x0001,
        IntrinsicProperty::ColdRes => 0x0002,
        IntrinsicProperty::SleepRes => 0x0004,
        IntrinsicProperty::DisintRes => 0x0008,
        IntrinsicProperty::ShockRes => 0x0010,
        IntrinsicProperty::PoisonRes => 0x0020,
        IntrinsicProperty::AcidRes => 0x0040,
        IntrinsicProperty::StoneRes => 0x0080,
        _ => 0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

///
///
///
pub fn which_armor_index(items: &[(u32, usize)], flag: u32) -> Option<usize> {
    for (worn_mask, idx) in items {
        if *worn_mask & flag != 0 {
            return Some(*idx);
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.4.0] 몬스터 장비 착용 가능 여부 (원본: m_dowear 사전 조건)
/// very_small: 매우 작은 크기, no_hands: 손 없음, is_animal: 동물, is_mindless: 정신 없음
/// is_creation: 생성 시점 여부, is_mummy: 미라, is_skeleton: 해골
pub fn can_monster_wear(
    very_small: bool,
    no_hands: bool,
    is_animal: bool,
    is_mindless: bool,
    is_creation: bool,
    is_mummy: bool,
    is_skeleton: bool,
) -> bool {
    //
    if very_small || no_hands || is_animal {
        return false;
    }
    //
    if is_mindless && (!is_creation || (!is_mummy && !is_skeleton)) {
        return false;
    }
    true
}

///
///
///
pub fn mon_wear_priority(
    cant_wear_arm: bool,
    is_slithy: bool,
    is_centaur: bool,
    is_small: bool,
    has_bimanual_weapon: bool,
    current_worn: u32,
) -> Vec<u32> {
    let mut slots = Vec::new();

    // 1. 부적
    slots.push(W_AMUL);

    // 2. 셔츠 (방어구 착용 가능 + 갑옷 미착용 시)
    if !cant_wear_arm && (current_worn & W_ARM) == 0 {
        slots.push(W_ARMU);
    }

    // 3. 망토 (방어구 착용 가능 또는 작은 크기)
    if !cant_wear_arm || is_small {
        slots.push(W_ARMC);
    }

    // 4. 투구
    slots.push(W_ARMH);

    //
    if !has_bimanual_weapon {
        slots.push(W_ARMS);
    }

    // 6. 장갑
    slots.push(W_ARMG);

    //
    if !is_slithy && !is_centaur {
        slots.push(W_ARMF);
    }

    // 8. 갑옷
    if !cant_wear_arm {
        slots.push(W_ARM);
    }

    slots
}

///
///
///
pub fn is_better_armor(new_bonus: i32, old_bonus: i32) -> bool {
    new_bonus > old_bonus
}

///
///
pub fn should_autocurse(item_name: &str, is_already_cursed: bool) -> bool {
    if is_already_cursed {
        return false;
    }
    let lower = item_name.to_lowercase();
    lower.contains("helm of opposite alignment") || lower.contains("dunce cap")
}

///
/// slot: 착용 슬롯, old_item_delay: 기존 아이템 착용 지연, new_item_delay: 신규 아이템 착용 지연,
/// is_wearing_cloak: 망토 착용 여부, has_old_item: 기존 아이템 존재
pub fn mon_wear_delay(
    slot: u32,
    old_item_delay: i32,
    new_item_delay: i32,
    is_wearing_cloak: bool,
    has_old_item: bool,
) -> i32 {
    let mut delay = 0;

    // 갑옷/셔츠 착용 시 망토 벗고 다시 입는 시간
    if (slot == W_ARM || slot == W_ARMU) && is_wearing_cloak {
        delay += 2;
    }
    // 기존 아이템 벗는 시간
    if has_old_item {
        delay += old_item_delay;
    }
    // 신규 아이템 입는 시간
    delay += new_item_delay;

    delay
}

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

///
///
pub fn extra_pref(item_name: &str, current_permspeed: u8) -> i32 {
    let lower = item_name.to_lowercase();
    if lower.contains("speed boots") && current_permspeed != MFAST {
        return 20;
    }
    0
}

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.4.0] 종족별 방어구 착용 예외 (원본: racial_exception)
/// 반환: 0(기본 규칙), 1(허용 예외), -1(금지 예외)
pub fn racial_exception(race_name: &str, armor_name: &str) -> i32 {
    let race = race_name.to_lowercase();
    let armor = armor_name.to_lowercase();

    //
    if race.contains("hobbit") && armor.contains("elven") {
        return 1;
    }

    // 기본: 예외 없음
    0
}

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.4.0] 변신 시 장비 파괴/탈락 이벤트 종류
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArmorBreakEvent {
    ///
    BodyArmorBreaks(String),
    /// 망토 찢김
    CloakTears(String),
    /// 셔츠 찢김
    ShirtRips(String),
    ///
    BodyArmorFalls(String),
    /// 망토 떨어짐
    CloakFalls(String),
    ///
    ShirtSlipsOff(String),
    /// 장갑 떨어짐
    GlovesDrop(String),
    /// 방패 들 수 없음
    ShieldDrop(String),
    /// 투구 떨어짐
    HelmetFalls(String),
    /// 장화 떨어짐
    BootsFallOff(String),
    /// 안장 떨어짐
    SaddleFalls(String),
}

///
///
/// handless_or_tiny: 손 없거나 매우 작음, has_horns: 뿔 있음
///
///
pub fn mon_break_armor_events(
    break_arm: bool,
    slip_arm: bool,
    handless_or_tiny: bool,
    has_horns: bool,
    is_slithy: bool,
    is_centaur: bool,
    can_saddle: bool,
    current_worn: u32,
    mon_name: &str,
) -> Vec<ArmorBreakEvent> {
    let mut events = Vec::new();

    if break_arm {
        //
        if current_worn & W_ARM != 0 {
            events.push(ArmorBreakEvent::BodyArmorBreaks(format!(
                "{} breaks out of its armor!",
                mon_name
            )));
        }
        if current_worn & W_ARMC != 0 {
            events.push(ArmorBreakEvent::CloakTears(format!(
                "{}'s cloak tears apart!",
                mon_name
            )));
        }
        if current_worn & W_ARMU != 0 {
            events.push(ArmorBreakEvent::ShirtRips(format!(
                "{}'s shirt rips to shreds!",
                mon_name
            )));
        }
    } else if slip_arm {
        //
        if current_worn & W_ARM != 0 {
            events.push(ArmorBreakEvent::BodyArmorFalls(format!(
                "{}'s armor falls around it!",
                mon_name
            )));
        }
        if current_worn & W_ARMC != 0 {
            events.push(ArmorBreakEvent::CloakFalls(format!(
                "{}'s cloak falls, unsupported!",
                mon_name
            )));
        }
        if current_worn & W_ARMU != 0 {
            events.push(ArmorBreakEvent::ShirtSlipsOff(format!(
                "{} seeps right through its shirt!",
                mon_name
            )));
        }
    }

    // 손 없거나 매우 작으면 장갑/방패 탈락
    if handless_or_tiny {
        if current_worn & W_ARMG != 0 {
            events.push(ArmorBreakEvent::GlovesDrop(format!(
                "{} drops its gloves!",
                mon_name
            )));
        }
        if current_worn & W_ARMS != 0 {
            events.push(ArmorBreakEvent::ShieldDrop(format!(
                "{} can no longer hold its shield!",
                mon_name
            )));
        }
    }

    // 손 없거나 뿔 → 투구 탈락
    if handless_or_tiny || has_horns {
        if current_worn & W_ARMH != 0 {
            events.push(ArmorBreakEvent::HelmetFalls(format!(
                "{}'s helmet falls to the ground!",
                mon_name
            )));
        }
    }

    //
    if handless_or_tiny || is_slithy || is_centaur {
        if current_worn & W_ARMF != 0 {
            events.push(ArmorBreakEvent::BootsFallOff(format!(
                "{}'s boots fall off!",
                mon_name
            )));
        }
    }

    // 안장 탈락
    if !can_saddle {
        if current_worn & W_SADDLE != 0 {
            events.push(ArmorBreakEvent::SaddleFalls(format!(
                "{}'s saddle falls off.",
                mon_name
            )));
        }
    }

    events
}

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

///
/// 미라 붕대(mummy wrapping) 착용 시 투명을 차단,
///
pub fn w_blocks(item_name: &str, worn_mask: u32, is_wizard: bool) -> IntrinsicProperty {
    let lower = item_name.to_lowercase();
    if lower.contains("mummy wrapping") && (worn_mask & W_ARMC) != 0 {
        return IntrinsicProperty::Invis;
    }
    if lower.contains("cornuthaum") && (worn_mask & W_ARMH) != 0 && !is_wizard {
        return IntrinsicProperty::Clairvoyant;
    }
    IntrinsicProperty::None
}

// ─────────────────────────────────────────────────────────────────────────────
//
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.4.0] 몬스터 영구 투명 설정 결과
#[derive(Debug, Clone)]
pub struct SetMinvisResult {
    /// 영구 투명 설정됨
    pub perminvis: bool,
    /// 현재 투명 상태 (invis_blocked이면 false)
    pub minvis: bool,
    /// UI 갱신 필요 여부
    pub need_newsym: bool,
}

/// [v2.4.0] 몬스터 영구 투명 설정 (원본: mon_set_minvis)
pub fn mon_set_minvis(invis_blocked: bool) -> SetMinvisResult {
    SetMinvisResult {
        perminvis: true,
        minvis: !invis_blocked,
        need_newsym: !invis_blocked,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ErosionType 확장 (원본: do_wear.c / worn.c 부식 시스템)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.4.0] 부식/손상 유형 (원본: oeroded/oeroded2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErosionType {
    Rust,
    Rot,
    Corrode,
    Burn,
}

/// [v2.4.0] 부식 단계별 메시지 (원본: erode_obj)
pub fn erosion_message(erosion_type: ErosionType, level: i32, item_name: &str) -> &'static str {
    match erosion_type {
        ErosionType::Rust => match level {
            0 => "is not affected.",
            1 => "is rusted.",
            2 => "is very rusted.",
            _ => "is thoroughly rusted.",
        },
        ErosionType::Rot => match level {
            0 => "is not affected.",
            1 => "is rotted.",
            2 => "is very rotted.",
            _ => "is thoroughly rotted.",
        },
        ErosionType::Corrode => match level {
            0 => "is not affected.",
            1 => "is corroded.",
            2 => "is very corroded.",
            _ => "is thoroughly corroded.",
        },
        ErosionType::Burn => match level {
            0 => "is not affected.",
            1 => "is burnt.",
            2 => "is very burnt.",
            _ => "is thoroughly burnt.",
        },
    }
}

/// [v2.4.0] 부식 가능 여부 (재료 기반)
pub fn can_erode(material: ArmorMaterial, erosion_type: ErosionType) -> bool {
    match erosion_type {
        ErosionType::Rust => matches!(material, ArmorMaterial::Iron | ArmorMaterial::Steel),
        ErosionType::Rot => matches!(
            material,
            ArmorMaterial::Leather | ArmorMaterial::Wood | ArmorMaterial::Cloth
        ),
        ErosionType::Corrode => matches!(
            material,
            ArmorMaterial::Iron | ArmorMaterial::Steel | ArmorMaterial::Silver
        ),
        ErosionType::Burn => matches!(
            material,
            ArmorMaterial::Cloth | ArmorMaterial::Leather | ArmorMaterial::Wood
        ),
    }
}

/// [v2.4.0] 부식 방지 재료 목록 (erodeproof 상당)
pub fn is_erodeproof_material(material: ArmorMaterial) -> bool {
    matches!(
        material,
        ArmorMaterial::Mithril | ArmorMaterial::DragonHide | ArmorMaterial::Crystal
    )
}

#[cfg(test)]
mod equipment_extended_tests {
    use super::*;

    #[test]
    fn test_material_weight() {
        assert!(
            material_weight_factor(ArmorMaterial::Mithril)
                < material_weight_factor(ArmorMaterial::Iron)
        );
    }

    #[test]
    fn test_rust_resistant() {
        assert!(material_rust_resistant(ArmorMaterial::Mithril));
        assert!(!material_rust_resistant(ArmorMaterial::Iron));
    }

    #[test]
    fn test_enchant_cost() {
        assert!(
            enchant_armor_cost(3, ArmorMaterial::Mithril)
                > enchant_armor_cost(1, ArmorMaterial::Iron)
        );
    }

    #[test]
    fn test_set_bonus() {
        assert_eq!(armor_set_bonus(0), 0);
        assert!(armor_set_bonus(6) > armor_set_bonus(2));
    }

    #[test]
    fn test_equipment_stats() {
        let mut s = EquipmentStatistics::new();
        s.record_wear();
        s.record_enchant();
        assert_eq!(s.items_worn, 1);
        assert_eq!(s.items_enchanted, 1);
    }

    // ─────────────────────────────────────────────────────────────────
    // [v2.4.0] worn.c 이식 테스트
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_wearslot() {
        //
        assert_eq!(wearslot("armor", 0, false, ""), W_ARM);
        assert_eq!(wearslot("armor", 1, false, ""), W_ARMS);
        assert_eq!(wearslot("armor", 5, false, ""), W_ARMC);
        // 반지 → 양손
        assert_eq!(wearslot("ring", 0, false, ""), W_RING);
        // 부적
        assert_eq!(wearslot("amulet", 0, false, ""), W_AMUL);
        // 무기 (탄약)
        assert_ne!(wearslot("weapon", 0, true, ""), W_WEP);
        assert!(wearslot("weapon", 0, true, "") & W_QUIVER != 0);
        //
        assert_eq!(wearslot("tool", 0, false, "blindfold"), W_TOOL);
        // 보석 → 화살통
        assert_eq!(wearslot("gem", 0, false, ""), W_QUIVER);
    }

    #[test]
    fn test_find_mac() {
        // 기본 AC 10, 장비 없으면 10
        assert_eq!(find_mac(10, &[]), 10);
        // 갑옷 AC 보너스 5 → 10-5=5
        assert_eq!(find_mac(10, &[(true, 5)]), 5);
        //
        assert_eq!(find_mac(10, &[(false, 5), (true, 3)]), 7);
        // 여러 개 합산
        assert_eq!(find_mac(10, &[(true, 3), (true, 2), (true, 1)]), 4);
    }

    #[test]
    fn test_mon_adjust_speed() {
        // 가속: 보통→FAST
        let result = mon_adjust_speed(1, 0, false, "orc", 12, false);
        assert_eq!(result.new_permspeed, MFAST);

        // 감속: 보통→SLOW
        let result = mon_adjust_speed(-1, 0, false, "orc", 12, false);
        assert_eq!(result.new_permspeed, MSLOW);

        // 가속: SLOW→보통
        let result = mon_adjust_speed(1, MSLOW, false, "orc", 12, false);
        assert_eq!(result.new_permspeed, 0);

        // 속도 부츠면 현재 속도 무조건 FAST
        let result = mon_adjust_speed(-1, MFAST, true, "orc", 12, false);
        assert_eq!(result.new_mspeed, MFAST);

        // 석화: 메시지 있음
        let result = mon_adjust_speed(-3, MFAST, false, "orc", 12, false);
        assert_eq!(result.new_permspeed, 0);
        assert!(result.message.is_some());
        assert!(result.message.unwrap().contains("slowing down"));
    }

    #[test]
    fn test_update_mon_intrinsics_on() {
        // 화염 저항 장착
        let result = update_mon_intrinsics(true, IntrinsicProperty::FireRes, 0, false, false);
        assert_eq!(result.resist_mask_change, 0x0001);

        // 투명 장착
        let result = update_mon_intrinsics(true, IntrinsicProperty::Invis, 0, false, false);
        assert_eq!(result.invisibility_change, Some(true));
        assert!(result.need_display_update);

        // Fast 장착
        let result = update_mon_intrinsics(true, IntrinsicProperty::Fast, 0, false, false);
        assert!(result.need_speed_update);
    }

    #[test]
    fn test_update_mon_intrinsics_off() {
        // 화염 저항 해제 (다른 화염 저항 아이템 없음)
        let result = update_mon_intrinsics(false, IntrinsicProperty::FireRes, 0x0001, false, false);
        assert_eq!(result.resist_mask_change, -1);

        // 화염 저항 해제 (다른 아이템 있으면 유지)
        let result = update_mon_intrinsics(false, IntrinsicProperty::FireRes, 0x0001, false, true);
        assert_eq!(result.resist_mask_change, 0);

        //
        let result = update_mon_intrinsics(false, IntrinsicProperty::Invis, 0, true, false);
        assert_eq!(result.invisibility_change, Some(true));
    }

    #[test]
    fn test_which_armor_index() {
        let items = vec![(W_ARM, 0usize), (W_ARMH, 1usize), (W_ARMF, 2usize)];
        assert_eq!(which_armor_index(&items, W_ARM), Some(0));
        assert_eq!(which_armor_index(&items, W_ARMH), Some(1));
        assert_eq!(which_armor_index(&items, W_ARMC), None);
    }

    #[test]
    fn test_can_monster_wear() {
        // 일반 몬스터 → 착용 가능
        assert!(can_monster_wear(
            false, false, false, false, false, false, false
        ));
        // 매우 작은 → 불가
        assert!(!can_monster_wear(
            true, false, false, false, false, false, false
        ));
        // 손 없음 → 불가
        assert!(!can_monster_wear(
            false, true, false, false, false, false, false
        ));
        // 동물 → 불가
        assert!(!can_monster_wear(
            false, false, true, false, false, false, false
        ));
        // 정신 없는 미라 (생성 시) → 가능
        assert!(can_monster_wear(
            false, false, false, true, true, true, false
        ));
        // 정신 없는 미라 아닌 (생성 시) → 불가
        assert!(!can_monster_wear(
            false, false, false, true, true, false, false
        ));
    }

    #[test]
    fn test_mon_wear_priority() {
        // 기본: 모든 슬롯 포함
        let slots = mon_wear_priority(false, false, false, false, false, 0);
        assert!(slots.contains(&W_AMUL));
        assert!(slots.contains(&W_ARM));
        assert!(slots.contains(&W_ARMF));

        // 뱀형: 장화 제외
        let slots = mon_wear_priority(false, true, false, false, false, 0);
        assert!(!slots.contains(&W_ARMF));

        //
        let slots = mon_wear_priority(false, false, false, false, true, 0);
        assert!(!slots.contains(&W_ARMS));
    }

    #[test]
    fn test_extra_pref() {
        // 속도 부츠 + 느린 몬스터 → +20
        assert_eq!(extra_pref("speed boots", 0), 20);
        // 이미 빠른 몬스터 → 0
        assert_eq!(extra_pref("speed boots", MFAST), 0);
        // 일반 아이템 → 0
        assert_eq!(extra_pref("iron helm", 0), 0);
    }

    #[test]
    fn test_racial_exception() {
        assert_eq!(racial_exception("hobbit", "elven cloak"), 1);
        assert_eq!(racial_exception("orc", "elven cloak"), 0);
        assert_eq!(racial_exception("hobbit", "iron helm"), 0);
    }

    #[test]
    fn test_mon_break_armor_events() {
        // 대형화 → 갑옷/망토/셔츠 파괴
        let events = mon_break_armor_events(
            true,
            false,
            false,
            false,
            false,
            false,
            true,
            W_ARM | W_ARMC | W_ARMU,
            "orc",
        );
        assert_eq!(events.len(), 3);

        // 소형화 → 갑옷/망토 탈락
        let events = mon_break_armor_events(
            false,
            true,
            false,
            false,
            false,
            false,
            true,
            W_ARM | W_ARMC,
            "newt",
        );
        assert_eq!(events.len(), 2);

        // 손 없고 뿔 → 장갑+방패+투구+장화 탈락
        let events = mon_break_armor_events(
            false,
            false,
            true,
            true,
            false,
            false,
            true,
            W_ARMG | W_ARMS | W_ARMH | W_ARMF,
            "dragon",
        );
        assert!(events.len() >= 4);
    }

    #[test]
    fn test_w_blocks() {
        // 미라 붕대 → 투명 차단
        assert_eq!(
            w_blocks("mummy wrapping", W_ARMC, false),
            IntrinsicProperty::Invis
        );
        //
        assert_eq!(
            w_blocks("cornuthaum", W_ARMH, false),
            IntrinsicProperty::Clairvoyant
        );
        //
        assert_eq!(
            w_blocks("cornuthaum", W_ARMH, true),
            IntrinsicProperty::None
        );
        // 일반 아이템 → 차단 없음
        assert_eq!(
            w_blocks("iron helm", W_ARMH, false),
            IntrinsicProperty::None
        );
    }

    #[test]
    fn test_erosion_system() {
        // 철은 녹슬 수 있음
        assert!(can_erode(ArmorMaterial::Iron, ErosionType::Rust));
        //
        assert!(!can_erode(ArmorMaterial::Mithril, ErosionType::Rust));
        // 가죽은 부패함
        assert!(can_erode(ArmorMaterial::Leather, ErosionType::Rot));
        //
        assert!(!can_erode(ArmorMaterial::Iron, ErosionType::Rot));
        //
        assert!(is_erodeproof_material(ArmorMaterial::Mithril));
        assert!(is_erodeproof_material(ArmorMaterial::DragonHide));
        assert!(!is_erodeproof_material(ArmorMaterial::Iron));
    }

    #[test]
    fn test_should_autocurse() {
        assert!(should_autocurse("Helm of Opposite Alignment", false));
        assert!(should_autocurse("dunce cap", false));
        // 이미 저주됨 → false
        assert!(!should_autocurse("Helm of Opposite Alignment", true));
        assert!(!should_autocurse("iron helm", false));
    }

    #[test]
    fn test_mon_wear_delay() {
        // 갑옷 착용 + 망토 착용 → +2 추가
        let delay = mon_wear_delay(W_ARM, 0, 5, true, false);
        assert_eq!(delay, 7); // 2(망토) + 0(기존없음) + 5(신규)

        // 기존 아이템 교체
        let delay = mon_wear_delay(W_ARMH, 1, 1, false, true);
        assert_eq!(delay, 2); // 1(기존벗기) + 1(신규)

        // 셔츠 교체 + 망토 착용
        let delay = mon_wear_delay(W_ARMU, 1, 1, true, true);
        assert_eq!(delay, 4); // 2(망토) + 1(기존) + 1(신규)
    }
}

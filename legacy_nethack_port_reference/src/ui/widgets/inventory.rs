use crate::core::entity::{identity::IdentityTable, Inventory, Item};
use crate::core::systems::item_use::ItemAction;
use legion::*;

///
#[derive(Debug, Clone, PartialEq)]
pub enum InventoryViewAction {
    /// 아무 액션 없음
    None,
    ///
    SwitchToEquipment,
    /// 아이템 액션
    ItemAction(ItemAction),
}

///
pub fn show_inventory(
    ui: &mut eframe::egui::Ui,
    world: &World,
    inventory: &Inventory,
    equipment: &crate::core::entity::Equipment,
    item_manager: &crate::core::entity::object::ItemManager,
    ident_table: &IdentityTable,
    looting_container: Option<Entity>,
) -> Option<ItemAction> {
    let mut pending_action = None;

    ui.vertical(|ui| {
        // ================================================================
        // 헤더: Inventory 타이틀 + Equipment 전환 버튼
        // ================================================================
        ui.horizontal(|ui| {
            ui.label(
                eframe::egui::RichText::new("Inventory")
                    .color(eframe::egui::Color32::from_rgb(255, 220, 100))
                    .size(18.0)
                    .strong(),
            );
            ui.with_layout(
                eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                |ui| {
                    // Equipment 전환 버튼 (현재 show_inventory 반환 타입이 Option<ItemAction>이므로
                    // 전환 처리는 상위 game_ui.rs에서 별도 상태로 관리)
                    if ui.button("🛡 Equipment ⇄").clicked() {
                        //
                        //
                    }
                },
            );
        });

        if inventory.items.is_empty() {
            ui.add_space(8.0);
            ui.label(
                eframe::egui::RichText::new("Your bag is empty.")
                    .color(eframe::egui::Color32::from_rgb(120, 120, 140))
                    .italics()
                    .size(13.0),
            );
            return;
        }

        // ================================================================
        // 무게 합계 + 캐리 상태 표시
        // ================================================================
        use crate::core::systems::item_helper::ItemHelper;
        let mut total_weight: i32 = 0;
        for item_ent in &inventory.items {
            total_weight += ItemHelper::calculate_weight(*item_ent, world);
        }
        let item_count = inventory.items.len();
        //
        let max_weight: i32 = 520;
        let max_items: usize = 52;

        // 하중 레벨
        let (encumbrance_label, enc_color) = if total_weight > max_weight {
            ("Overloaded!", eframe::egui::Color32::from_rgb(255, 60, 60))
        } else if total_weight > max_weight * 4 / 5 {
            ("Overtaxed", eframe::egui::Color32::from_rgb(255, 140, 60))
        } else if total_weight > max_weight * 3 / 5 {
            ("Strained", eframe::egui::Color32::from_rgb(255, 200, 80))
        } else if total_weight > max_weight * 2 / 5 {
            ("Stressed", eframe::egui::Color32::from_rgb(220, 220, 100))
        } else if total_weight > max_weight / 5 {
            ("Burdened", eframe::egui::Color32::from_rgb(200, 200, 160))
        } else {
            ("Normal", eframe::egui::Color32::from_rgb(140, 200, 140))
        };

        ui.horizontal(|ui| {
            ui.label(
                eframe::egui::RichText::new(format!(
                    "Carrying: {}/{} lbs",
                    total_weight, max_weight
                ))
                .monospace()
                .size(11.0)
                .color(eframe::egui::Color32::from_rgb(160, 160, 180)),
            );
            ui.add_space(10.0);
            ui.label(
                eframe::egui::RichText::new(format!("Items: {}/{}", item_count, max_items))
                    .monospace()
                    .size(11.0)
                    .color(eframe::egui::Color32::from_rgb(160, 160, 180)),
            );
            ui.add_space(10.0);
            ui.label(
                eframe::egui::RichText::new(encumbrance_label)
                    .size(11.0)
                    .strong()
                    .color(enc_color),
            );
        });

        ui.separator();

        // ================================================================
        //
        // ================================================================
        use crate::core::entity::object::ItemClass;

        //
        let mut categorized: std::collections::BTreeMap<ItemClass, Vec<Entity>> =
            std::collections::BTreeMap::new();

        for item_ent in &inventory.items {
            if let Some(entry) = world.entry_ref(*item_ent).ok() {
                if let Ok(item) = entry.get_component::<Item>() {
                    if let Some(template) = item_manager.get_by_kind(item.kind) {
                        categorized
                            .entry(template.class)
                            .or_default()
                            .push(*item_ent);
                    }
                }
            }
        }

        //
        let class_display = |class: ItemClass| -> (&'static str, &'static str) {
            match class {
                ItemClass::Weapon => ("⚔", "Weapons"),
                ItemClass::Armor => ("🛡", "Armor"),
                ItemClass::Food => ("🍖", "Food"),
                ItemClass::Potion => ("🧪", "Potions"),
                ItemClass::Scroll => ("📜", "Scrolls"),
                ItemClass::Spellbook => ("📕", "Spellbooks"),
                ItemClass::Wand => ("🪄", "Wands"),
                ItemClass::Coin => ("💰", "Coins"),
                ItemClass::Tool => ("🔧", "Tools"),
                ItemClass::Ring => ("💍", "Rings"),
                _ => ("📦", "Other"),
            }
        };

        // 스크롤 영역
        eframe::egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for (class, item_entities) in &categorized {
                    let (icon, class_name) = class_display(*class);

                    ui.add_space(5.0);
                    ui.label(
                        eframe::egui::RichText::new(format!("{} {}", icon, class_name))
                            .size(13.0)
                            .underline()
                            .color(eframe::egui::Color32::from_rgb(130, 180, 255)),
                    );

                    //
                    let mut template_groups: std::collections::HashMap<String, Vec<Entity>> =
                        std::collections::HashMap::new();
                    for &ent in item_entities {
                        if let Some(entry) = world.entry_ref(ent).ok() {
                            if let Ok(item) = entry.get_component::<Item>() {
                                template_groups
                                    .entry(item.kind.to_string())
                                    .or_default()
                                    .push(ent);
                            }
                        }
                    }

                    //
                    let mut displayed_in_class = std::collections::HashSet::new();

                    for &item_ent in &inventory.items {
                        if !item_entities.contains(&item_ent) {
                            continue;
                        }

                        if let Some(entry) = world.entry_ref(item_ent).ok() {
                            if let Ok(item) = entry.get_component::<Item>() {
                                if displayed_in_class.contains(&item.kind.to_string()) {
                                    continue;
                                }
                                displayed_in_class.insert(item.kind.to_string());

                                let group = &template_groups[&item.kind.to_string()];
                                let letter = inventory.get_letter(item_ent);

                                // 장착 마커
                                let mut marker = "";
                                for (slot, &eq_ent) in &equipment.slots {
                                    if group.contains(&eq_ent) {
                                        marker = match slot {
                                            crate::core::entity::EquipmentSlot::Melee => {
                                                " (wielded)"
                                            }
                                            crate::core::entity::EquipmentSlot::Shield => {
                                                " (in hand)"
                                            }
                                            _ => " (being worn)",
                                        };
                                        break;
                                    }
                                }

                                // 아이템 표시 이름
                                let display_name =
                                    if let Some(template) = item_manager.get_by_kind(item.kind) {
                                        ItemHelper::get_name(item, template, Some(ident_table))
                                    } else {
                                        item.kind.to_string()
                                    };

                                // BUC 색상 결정
                                let item_color = if item.bknown {
                                    if item.blessed {
                                        eframe::egui::Color32::from_rgb(80, 200, 255)
                                    // 축복 = 청록
                                    } else if item.cursed {
                                        eframe::egui::Color32::from_rgb(255, 80, 80)
                                    // 저주 = 빨강
                                    } else {
                                        eframe::egui::Color32::from_rgb(200, 200, 220)
                                        // 일반 = 밝은 회색
                                    }
                                } else {
                                    eframe::egui::Color32::from_rgb(160, 160, 170)
                                    // 미감정 = 어두운 회색
                                };

                                // 무게 표시
                                let weight = ItemHelper::calculate_weight(item_ent, world);

                                ui.horizontal(|ui| {
                                    //
                                    ui.label(
                                        eframe::egui::RichText::new(format!(
                                            "{} - {}{}",
                                            letter, display_name, marker
                                        ))
                                        .size(12.0)
                                        .color(item_color),
                                    );

                                    // 우측 정렬: 무게 + 액션 버튼
                                    ui.with_layout(
                                        eframe::egui::Layout::right_to_left(
                                            eframe::egui::Align::Center,
                                        ),
                                        |ui| {
                                            // 무게 표시
                                            ui.label(
                                                eframe::egui::RichText::new(format!(
                                                    "{}lb",
                                                    weight
                                                ))
                                                .monospace()
                                                .size(10.0)
                                                .color(eframe::egui::Color32::from_rgb(
                                                    120, 120, 140,
                                                )),
                                            );

                                            //
                                            if let Some(container_ent) = looting_container {
                                                if item_ent != container_ent {
                                                    if ui.small_button("Put In").clicked() {
                                                        pending_action = Some(ItemAction::PutIn {
                                                            container: container_ent,
                                                            item: item_ent,
                                                        });
                                                    }
                                                }
                                            }

                                            // Drop 버튼
                                            if ui.small_button("Drop").clicked() {
                                                pending_action = Some(ItemAction::Drop(item_ent));
                                            }

                                            //
                                            match class {
                                                ItemClass::Weapon => {
                                                    // [v2.41.1] Wield 버튼
                                                    if ui.small_button("Wield").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Wield(item_ent));
                                                    }
                                                    // [S5c] Throw 버튼 — 무기는 투척 가능
                                                    if ui.small_button("Throw").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Throw(item_ent));
                                                    }
                                                }
                                                ItemClass::Armor => {
                                                    // [v2.41.1] Wear 버튼
                                                    if ui.small_button("Wear").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Wear(item_ent));
                                                    }
                                                }
                                                ItemClass::Potion => {
                                                    if ui.small_button("Drink").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Drink(item_ent));
                                                    }
                                                    // [S5c] Throw — 포션도 투척 가능
                                                    if ui.small_button("Throw").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Throw(item_ent));
                                                    }
                                                }
                                                ItemClass::Food => {
                                                    if ui.small_button("Eat").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Eat(item_ent));
                                                    }
                                                    // [S5c] Throw — 음식(크림 파이 등)도 투척 가능
                                                    if ui.small_button("Throw").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Throw(item_ent));
                                                    }
                                                }
                                                ItemClass::Scroll | ItemClass::Spellbook => {
                                                    if ui.small_button("Read").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Read(item_ent));
                                                    }
                                                }
                                                ItemClass::Wand => {
                                                    if ui.small_button("Zap").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Zap(item_ent));
                                                    }
                                                }
                                                ItemClass::Gem => {
                                                    if ui.small_button("Apply").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Apply(item_ent));
                                                    }
                                                    // [S5c] Throw — 보석/돌은 투척 가능
                                                    if ui.small_button("Throw").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Throw(item_ent));
                                                    }
                                                }
                                                ItemClass::Tool => {
                                                    if ui.small_button("Apply").clicked() {
                                                        pending_action =
                                                            Some(ItemAction::Apply(item_ent));
                                                    }
                                                }
                                                _ => {}
                                            }
                                        },
                                    );
                                });
                            }
                        }
                    }
                }
            });

        ui.add_space(4.0);
        ui.separator();

        // ================================================================
        // 하단 액션 버튼 바
        // ================================================================
        ui.horizontal(|ui| {
            if ui.button("Drop").clicked() { /* 대상 선택 모드 — 향후 */ }
            if ui.button("Apply").clicked() { /* 대상 선택 모드 — 향후 */ }
            if ui.button("Eat").clicked() { /* 대상 선택 모드 — 향후 */ }
            if ui.button("Drink").clicked() { /* 대상 선택 모드 — 향후 */ }
            if ui.button("Read").clicked() { /* 대상 선택 모드 — 향후 */ }
            if ui.button("Wear").clicked() { /* 대상 선택 모드 — 향후 */ }
            if ui.button("Wield").clicked() { /* 대상 선택 모드 — 향후 */ }
        });
    });

    pending_action
}

///
pub fn show_offhand_selector(
    ui: &mut eframe::egui::Ui,
    world: &World,
    inventory: &Inventory,
    item_manager: &crate::core::entity::object::ItemManager,
    ident_table: &IdentityTable,
) -> Option<ItemAction> {
    let mut pending_action = None;

    ui.vertical(|ui| {
        ui.heading("Select an off-hand weapon:");
        ui.separator();

        let mut weapons_found = false;
        use crate::core::entity::object::ItemClass;
        use crate::core::systems::item_helper::ItemHelper;

        for &item_ent in &inventory.items {
            if let Some(entry) = world.entry_ref(item_ent).ok() {
                if let Ok(item) = entry.get_component::<Item>() {
                    if let Some(template) = item_manager.get_by_kind(item.kind) {
                        if template.class == ItemClass::Weapon {
                            weapons_found = true;
                            let display_name =
                                ItemHelper::get_name(item, template, Some(ident_table));
                            let letter = inventory.get_letter(item_ent);

                            ui.horizontal(|ui| {
                                ui.label(format!("{} - {}", letter, display_name));
                                if ui.button("Select").clicked() {
                                    pending_action = Some(ItemAction::EquipOffhand(item_ent));
                                }
                            });
                        }
                    }
                }
            }
        }

        if !weapons_found {
            ui.label("No suitable off-hand weapons in inventory.");
        }
    });

    pending_action
}

///
pub fn show_quiver_selector(
    ui: &mut eframe::egui::Ui,
    world: &World,
    inventory: &Inventory,
    item_manager: &crate::core::entity::object::ItemManager,
    ident_table: &IdentityTable,
) -> Option<ItemAction> {
    let mut pending_action = None;

    ui.vertical(|ui| {
        ui.heading("Select an item to quiver:");
        ui.separator();

        let mut items_found = false;
        use crate::core::entity::object::ItemClass;
        use crate::core::systems::item_helper::ItemHelper;

        for &item_ent in &inventory.items {
            if let Some(entry) = world.entry_ref(item_ent).ok() {
                if let Ok(item) = entry.get_component::<Item>() {
                    if let Some(template) = item_manager.get_by_kind(item.kind) {
                        //
                        if template.class == ItemClass::Weapon
                            || template.class == ItemClass::Tool
                            || template.class == ItemClass::Gem
                            || template.class == ItemClass::Food
                        {
                            items_found = true;
                            let display_name =
                                ItemHelper::get_name(item, template, Some(ident_table));
                            let letter = inventory.get_letter(item_ent);

                            ui.horizontal(|ui| {
                                ui.label(format!("{} - {}", letter, display_name));
                                if ui.button("Select").clicked() {
                                    pending_action = Some(ItemAction::QuiverSelect(item_ent));
                                }
                            });
                        }
                    }
                }
            }
        }

        if !items_found {
            ui.label("No suitable items in inventory.");
        }
    });

    pending_action
}

///
pub fn show_naming_selector(
    ui: &mut eframe::egui::Ui,
    world: &World,
    inventory: &Inventory,
    item_manager: &crate::core::entity::object::ItemManager,
    ident_table: &IdentityTable,
) -> Option<Entity> {
    let mut selected_item = None;

    ui.vertical(|ui| {
        ui.heading("Name an item:");
        ui.separator();

        use crate::core::systems::item_helper::ItemHelper;

        if inventory.items.is_empty() {
            ui.label("Inventory is empty.");
        }

        for &item_ent in &inventory.items {
            if let Some(entry) = world.entry_ref(item_ent).ok() {
                if let Ok(item) = entry.get_component::<Item>() {
                    if let Some(template) = item_manager.get_by_kind(item.kind) {
                        let display_name = ItemHelper::get_name(item, template, Some(ident_table));
                        let letter = inventory.get_letter(item_ent);

                        ui.horizontal(|ui| {
                            ui.label(format!("{} - {}", letter, display_name));
                            if ui.button("Name").clicked() {
                                selected_item = Some(item_ent);
                            }
                        });
                    }
                }
            }
        }
    });

    selected_item
}

///
pub fn show_invoke_selector(
    ui: &mut eframe::egui::Ui,
    world: &World,
    inventory: &Inventory,
    item_manager: &crate::core::entity::object::ItemManager,
    ident_table: &IdentityTable,
) -> Option<Entity> {
    let mut selected_item = None;

    ui.vertical(|ui| {
        ui.heading("Invoke which artifact:");
        ui.separator();

        use crate::core::systems::item_helper::ItemHelper;
        let mut artifacts_found = false;

        for &item_ent in &inventory.items {
            if let Some(entry) = world.entry_ref(item_ent).ok() {
                if let Ok(item) = entry.get_component::<Item>() {
                    if item.artifact.is_some() {
                        artifacts_found = true;
                        if let Some(template) = item_manager.get_by_kind(item.kind) {
                            let display_name =
                                ItemHelper::get_name(item, template, Some(ident_table));
                            let letter = inventory.get_letter(item_ent);

                            ui.horizontal(|ui| {
                                ui.label(format!("{} - {}", letter, display_name));
                                if ui.button("Invoke").clicked() {
                                    selected_item = Some(item_ent);
                                }
                            });
                        }
                    }
                }
            }
        }

        if !artifacts_found {
            ui.label("You have no artifacts to invoke.");
        }
    });

    selected_item
}

///
pub fn show_offer_selector(
    ui: &mut eframe::egui::Ui,
    world: &World,
    inventory: &Inventory,
    item_manager: &crate::core::entity::object::ItemManager,
    ident_table: &IdentityTable,
) -> Option<ItemAction> {
    let mut pending_action = None;

    ui.vertical(|ui| {
        ui.heading("Sacrifice which corpse:");
        ui.separator();

        use crate::core::systems::item_helper::ItemHelper;
        let mut corpses_found = false;

        for &item_ent in &inventory.items {
            if let Some(entry) = world.entry_ref(item_ent).ok() {
                if let Ok(item) = entry.get_component::<Item>() {
                    if item.kind.is_corpse() {
                        corpses_found = true;
                        if let Some(template) = item_manager.get_by_kind(item.kind) {
                            let display_name =
                                ItemHelper::get_name(item, template, Some(ident_table));
                            let letter = inventory.get_letter(item_ent);

                            ui.horizontal(|ui| {
                                ui.label(format!("{} - {}", letter, display_name));
                                if ui.button("Offer").clicked() {
                                    pending_action = Some(ItemAction::Offer(item_ent));
                                }
                            });
                        }
                    }
                }
            }
        }

        if !corpses_found {
            ui.label("You have no corpses to sacrifice.");
        }
    });

    pending_action
}

/// [v2.21.0 R9-2] Extended Commands - Rub
pub fn show_rub_selector(
    ui: &mut eframe::egui::Ui,
    world: &World,
    inventory: &Inventory,
    item_manager: &crate::core::entity::object::ItemManager,
    ident_table: &IdentityTable,
) -> Option<ItemAction> {
    let mut pending_action = None;

    ui.vertical(|ui| {
        ui.heading("Rub which item:");
        ui.separator();

        use crate::core::systems::item_helper::ItemHelper;
        let mut items_found = false;

        for &item_ent in &inventory.items {
            if let Some(entry) = world.entry_ref(item_ent).ok() {
                if let Ok(item) = entry.get_component::<Item>() {
                    items_found = true;
                    if let Some(template) = item_manager.get_by_kind(item.kind) {
                        let display_name = ItemHelper::get_name(item, template, Some(ident_table));
                        let letter = inventory.get_letter(item_ent);

                        ui.horizontal(|ui| {
                            ui.label(format!("{} - {}", letter, display_name));
                            if ui.button("Rub").clicked() {
                                pending_action = Some(ItemAction::Rub(item_ent));
                            }
                        });
                    }
                }
            }
        }

        if !items_found {
            ui.label("You have nothing to rub.");
        }
    });

    pending_action
}

/// [v2.21.0 R9-2] Extended Commands - Dip
pub fn show_dip_selector(
    ui: &mut eframe::egui::Ui,
    world: &World,
    inventory: &Inventory,
    item_manager: &crate::core::entity::object::ItemManager,
    ident_table: &IdentityTable,
) -> Option<ItemAction> {
    let mut pending_action = None;

    ui.vertical(|ui| {
        ui.heading("Dip which item:");
        ui.separator();

        use crate::core::systems::item_helper::ItemHelper;
        let mut items_found = false;

        for &item_ent in &inventory.items {
            if let Some(entry) = world.entry_ref(item_ent).ok() {
                if let Ok(item) = entry.get_component::<Item>() {
                    items_found = true;
                    if let Some(template) = item_manager.get_by_kind(item.kind) {
                        let display_name = ItemHelper::get_name(item, template, Some(ident_table));
                        let letter = inventory.get_letter(item_ent);

                        ui.horizontal(|ui| {
                            ui.label(format!("{} - {}", letter, display_name));
                            if ui.button("Dip").clicked() {
                                pending_action = Some(ItemAction::Dip(item_ent));
                            }
                        });
                    }
                }
            }
        }

        if !items_found {
            ui.label("You have nothing to dip.");
        }
    });

    pending_action
}

///
pub fn show_engrave_tool_selector(
    ui: &mut eframe::egui::Ui,
    world: &World,
    inventory: &Inventory,
    item_manager: &crate::core::entity::object::ItemManager,
    ident_table: &IdentityTable,
) -> Option<Option<Entity>> {
    let mut selected_tool = None;

    ui.vertical(|ui| {
        ui.heading("What do you want to write with?");
        ui.separator();

        if ui.button("- Your finger").clicked() {
            selected_tool = Some(None);
        }

        ui.add_space(5.0);
        ui.label("Or choose an item from inventory:");

        use crate::core::systems::item_helper::ItemHelper;
        for &item_ent in &inventory.items {
            if let Some(entry) = world.entry_ref(item_ent).ok() {
                if let Ok(item) = entry.get_component::<Item>() {
                    if let Some(template) = item_manager.get_by_kind(item.kind) {
                        let display_name = ItemHelper::get_name(item, template, Some(ident_table));
                        let letter = inventory.get_letter(item_ent);

                        ui.horizontal(|ui| {
                            ui.label(format!("{} - {}", letter, display_name));
                            if ui.button("Use").clicked() {
                                selected_tool = Some(Some(item_ent));
                            }
                        });
                    }
                }
            }
        }
    });

    selected_tool
}

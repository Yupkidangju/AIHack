//!
//!
//!
//!
//!

use crate::core::entity::identity::IdentityTable;
use crate::core::entity::object::ItemManager;
use crate::core::entity::{Equipment, EquipmentSlot, Inventory, Item};
use crate::core::systems::item_use::ItemAction;
use eframe::egui;
use legion::*;

///
#[derive(Debug, Clone, PartialEq)]
pub enum EquipmentAction {
    ///
    None,
    ///
    SwitchToInventory,
    ///
    ItemAction(ItemAction),
}

///
fn slot_display(slot: EquipmentSlot) -> (&'static str, &'static str) {
    match slot {
        EquipmentSlot::Melee => ("[W] Weapon (R)", "Melee"),
        EquipmentSlot::Offhand => ("[W] Weapon (L)", "Offhand"),
        EquipmentSlot::Shield => ("[S] Shield", "Shield"),
        EquipmentSlot::Head => ("[H] Head", "Head"),
        EquipmentSlot::Cloak => ("[C] Cloak", "Cloak"),
        EquipmentSlot::Body => ("[B] Body", "Body"),
        EquipmentSlot::Hands => ("[G] Gloves", "Gloves"),
        EquipmentSlot::Feet => ("[B] Boots", "Boots"),
        EquipmentSlot::Boots => ("[B] Boots", "Boots"),
        EquipmentSlot::RingLeft => ("[R] Ring (L)", "Ring L"),
        EquipmentSlot::RingRight => ("[R] Ring (R)", "Ring R"),
        EquipmentSlot::Amulet => ("[A] Amulet", "Amulet"),
        EquipmentSlot::Swap => ("[S] Swap", "Swap"),
        EquipmentSlot::Quiver => ("[Q] Quiver", "Quiver"),
    }
}

///
///
pub fn show_equipment_screen(
    ui: &mut egui::Ui,
    world: &World,
    equipment: &Equipment,
    inventory: &Inventory,
    item_manager: &ItemManager,
    ident_table: &IdentityTable,
    player_ac: i32,
) -> EquipmentAction {
    let mut result = EquipmentAction::None;

    //
    let title_color = egui::Color32::from_rgb(255, 220, 100);
    let slot_label_color = egui::Color32::from_rgb(160, 160, 180);
    let equipped_color = egui::Color32::from_rgb(200, 255, 200);
    let vacant_color = egui::Color32::from_rgb(100, 100, 120);
    let buc_blessed_color = egui::Color32::from_rgb(80, 200, 255);
    let buc_cursed_color = egui::Color32::from_rgb(255, 80, 80);
    let ac_color = egui::Color32::from_rgb(180, 220, 255);

    //
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Equipment")
                .color(title_color)
                .size(18.0)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("[I] Inventory").clicked() {
                result = EquipmentAction::SwitchToInventory;
            }
        });
    });
    ui.separator();
    ui.add_space(4.0);

    // ================================================================
    //
    // ================================================================
    ui.horizontal(|ui| {
        //
        ui.vertical(|ui| {
            ui.add_space(8.0);
            let ascii_art =
                "   +--------+\n   |   @    |\n   |  /|\\   |\n   |  / \\   |\n   +--------+";
            ui.label(
                egui::RichText::new(ascii_art)
                    .monospace()
                    .size(14.0)
                    .color(egui::Color32::from_rgb(200, 200, 220)),
            );
        });

        ui.add_space(16.0);

        //
        ui.vertical(|ui| {
            //
            let display_slots = [
                EquipmentSlot::Head,
                EquipmentSlot::Cloak,
                EquipmentSlot::Body,
                EquipmentSlot::Shield,
                EquipmentSlot::Hands,
                EquipmentSlot::Feet,
                EquipmentSlot::Melee,
                EquipmentSlot::Offhand,
                EquipmentSlot::RingRight,
                EquipmentSlot::RingLeft,
                EquipmentSlot::Amulet,
                EquipmentSlot::Quiver,
            ];

            for &slot in &display_slots {
                let (icon_label, _short) = slot_display(slot);

                ui.horizontal(|ui| {
                    //
                    ui.label(
                        egui::RichText::new(format!("{:<16}", icon_label))
                            .monospace()
                            .size(12.0)
                            .color(slot_label_color),
                    );

                    //
                    if let Some(&item_ent) = equipment.slots.get(&slot) {
                        //
                        let item_text =
                            get_equipped_item_name(world, item_ent, item_manager, ident_table);
                        let buc_color = get_buc_color(
                            world,
                            item_ent,
                            equipped_color,
                            buc_blessed_color,
                            buc_cursed_color,
                        );
                        ui.label(egui::RichText::new(&item_text).size(12.0).color(buc_color));

                        //
                        if ui
                            .small_button("X")
                            .on_hover_text("Take off / Unwield")
                            .clicked()
                        {
                            result = EquipmentAction::ItemAction(ItemAction::Unequip(item_ent));
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("(vacant)")
                                .size(12.0)
                                .color(vacant_color)
                                .italics(),
                        );
                    }
                });
            }
        });
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    // ================================================================
    //
    // ================================================================
    {
        //
        let base_ac = 10;
        let mut armor_bonus = 0i32;
        let mut shield_bonus = 0i32;
        let mut other_bonus = 0i32;

        for (&slot, &item_ent) in &equipment.slots {
            if let Ok(entry) = world.entry_ref(item_ent) {
                if let Ok(item) = entry.get_component::<Item>() {
                    if let Some(template) = item_manager.get_by_kind(item.kind) {
                        //
                        let ac_val = template.oc1 as i32 + item.spe as i32;
                        match slot {
                            EquipmentSlot::Body
                            | EquipmentSlot::Head
                            | EquipmentSlot::Cloak
                            | EquipmentSlot::Hands
                            | EquipmentSlot::Feet
                            | EquipmentSlot::Boots => {
                                armor_bonus += ac_val;
                            }
                            EquipmentSlot::Shield => {
                                shield_bonus += ac_val;
                            }
                            _ => {
                                //
                                if template.oc1 != 0 {
                                    other_bonus += ac_val;
                                }
                            }
                        }
                    }
                }
            }
        }

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("AC: {}", player_ac))
                    .size(14.0)
                    .strong()
                    .color(ac_color),
            );
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(format!(
                    "(Base: {}, Armor: {}, Shield: {}, Other: {})",
                    base_ac,
                    if armor_bonus != 0 {
                        format!("{:+}", -armor_bonus)
                    } else {
                        "0".to_string()
                    },
                    if shield_bonus != 0 {
                        format!("{:+}", -shield_bonus)
                    } else {
                        "0".to_string()
                    },
                    if other_bonus != 0 {
                        format!("{:+}", -other_bonus)
                    } else {
                        "0".to_string()
                    },
                ))
                .size(11.0)
                .color(slot_label_color),
            );
        });
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    // ================================================================
    //
    // ================================================================
    ui.horizontal(|ui| {
        if ui.button("[W] Wear").clicked() {
            result = EquipmentAction::ItemAction(ItemAction::WearPrompt);
        }
        if ui.button("[T] Take Off").clicked() {
            result = EquipmentAction::ItemAction(ItemAction::TakeOffPrompt);
        }
        if ui.button("[W] Wield").clicked() {
            result = EquipmentAction::ItemAction(ItemAction::WieldPrompt);
        }
        if ui.button("[P] Put On").clicked() {
            result = EquipmentAction::ItemAction(ItemAction::PutOnPrompt);
        }
        if ui.button("[R] Remove").clicked() {
            result = EquipmentAction::ItemAction(ItemAction::RemovePrompt);
        }
    });

    result
}

///
fn get_equipped_item_name(
    world: &World,
    item_ent: Entity,
    item_manager: &ItemManager,
    ident_table: &IdentityTable,
) -> String {
    if let Ok(entry) = world.entry_ref(item_ent) {
        if let Ok(item) = entry.get_component::<Item>() {
            if let Some(template) = item_manager.get_by_kind(item.kind) {
                use crate::core::systems::item_helper::ItemHelper;
                let name = ItemHelper::get_name(item, template, Some(ident_table));
                //
                if item.spe != 0 {
                    return format!("{:+} {}", item.spe, name);
                }
                return name;
            }
        }
    }
    "(unknown)".to_string()
}

///
fn get_buc_color(
    world: &World,
    item_ent: Entity,
    default_color: egui::Color32,
    blessed_color: egui::Color32,
    cursed_color: egui::Color32,
) -> egui::Color32 {
    if let Ok(entry) = world.entry_ref(item_ent) {
        if let Ok(item) = entry.get_component::<Item>() {
            if item.bknown {
                if item.blessed {
                    return blessed_color;
                }
                if item.cursed {
                    return cursed_color;
                }
            }
        }
    }
    default_color
}

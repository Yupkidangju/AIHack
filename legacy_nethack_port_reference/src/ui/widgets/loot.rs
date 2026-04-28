use crate::core::entity::{Inventory, Item};
use crate::core::systems::item_use::ItemAction;
use legion::*;

/// egui 기반 루팅 위젯
pub fn show_loot_menu(
    ui: &mut eframe::egui::Ui,
    world: &World,
    container_ent: Entity,
    container_inv: &Inventory,
    item_manager: &crate::core::entity::object::ItemManager,
    ident_table: &crate::core::entity::identity::IdentityTable,
) -> Option<ItemAction> {
    let mut pending_action = None;

    ui.vertical(|ui| {
        if container_inv.items.is_empty() {
            ui.label("The container is empty.");
        } else {
            use crate::core::systems::item_helper::ItemHelper;
            let total_weight = ItemHelper::calculate_weight(container_ent, world);
            let base_weight = world
                .entry_ref(container_ent)
                .ok()
                .and_then(|e| e.get_component::<Item>().ok().map(|i| i.weight))
                .unwrap_or(0);

            ui.horizontal(|ui| {
                ui.label(eframe::egui::RichText::new("Looting").strong());
                ui.add_space(20.0);
                ui.label(format!(
                    "(Total Weight: {}, Base: {})",
                    total_weight, base_weight
                ));
            });
            ui.separator();

            //
            use crate::core::entity::object::ItemClass;
            let mut categorized: std::collections::BTreeMap<ItemClass, Vec<Entity>> =
                std::collections::BTreeMap::new();

            for item_ent in &container_inv.items {
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

            for (class, item_entities) in categorized {
                let class_name = match class {
                    ItemClass::Weapon => "Weapons",
                    ItemClass::Armor => "Armor",
                    ItemClass::Food => "Food",
                    ItemClass::Potion => "Potions",
                    ItemClass::Scroll => "Scrolls",
                    ItemClass::Spellbook => "Spellbooks",
                    ItemClass::Wand => "Wands",
                    ItemClass::Coin => "Coins",
                    ItemClass::Tool => "Tools",
                    ItemClass::Ring => "Rings",
                    _ => "Other",
                };

                ui.add_space(5.0);
                ui.label(
                    eframe::egui::RichText::new(class_name)
                        .underline()
                        .color(eframe::egui::Color32::LIGHT_BLUE),
                );

                for item_ent in item_entities {
                    if let Ok(entry) = world.entry_ref(item_ent) {
                        if let Ok(item) = entry.get_component::<Item>() {
                            if let Some(template) = item_manager.get_by_kind(item.kind) {
                                let display_name =
                                    ItemHelper::get_name(item, template, Some(ident_table));

                                ui.horizontal(|ui| {
                                    ui.label(display_name);

                                    ui.with_layout(
                                        eframe::egui::Layout::right_to_left(
                                            eframe::egui::Align::Center,
                                        ),
                                        |ui| {
                                            if ui.button("Take").clicked() {
                                                pending_action = Some(ItemAction::TakeOut {
                                                    container: container_ent,
                                                    item: item_ent,
                                                });
                                            }
                                        },
                                    );
                                });
                            } // if let Some(template)
                        } // if let Ok(item)
                    } // if let Ok(entry)
                }
            }
        }
    });

    pending_action
}

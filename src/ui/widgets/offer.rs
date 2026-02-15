use crate::core::entity::{Inventory, Item};
use crate::core::systems::item_use::ItemAction;
use legion::*;

/// 제물 선택 메뉴 (#offer)
pub fn show_offer_menu(
    ui: &mut eframe::egui::Ui,
    world: &World,
    inventory: &Inventory,
    _item_manager: &crate::core::entity::object::ItemManager,
) -> Option<ItemAction> {
    let mut pending_action = None;

    ui.vertical(|ui| {
        ui.label(
            eframe::egui::RichText::new("Sacrifice what?")
                .strong()
                .color(eframe::egui::Color32::YELLOW),
        );
        ui.separator();

        let mut found_corpse = false;

        //
        for &item_ent in &inventory.items {
            if let Some(entry) = world.entry_ref(item_ent).ok() {
                if let Ok(item) = entry.get_component::<Item>() {
                    //
                    if item.kind.is_corpse() {
                        found_corpse = true;
                        ui.horizontal(|ui| {
                            ui.label(item.kind.as_str());
                            ui.with_layout(
                                eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                                |ui| {
                                    if ui.button("Offer").clicked() {
                                        pending_action = Some(ItemAction::Offer(item_ent));
                                    }
                                },
                            );
                        });
                    }
                }
            }
        }

        if !found_corpse {
            ui.label("You have no corpses to sacrifice in your bag.");
            ui.add_space(10.0);
            ui.label("(To sacrifice corpses on the ground, pickup them first or stand over them)");
        }
    });

    pending_action
}

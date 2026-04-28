// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::entity::object::{ItemTemplate, Material};
use crate::core::entity::Item;
use crate::generated::ItemKind;
use crate::ui::log::GameLog;
use legion::EntityStore;

/// 아이템 손상 로직 (Original: NetHack 3.6.7 rust_dmg, corrode_obj, burn_obj 등)
pub struct ItemDamageSystem;

impl ItemDamageSystem {
    /// 철제 아이템 녹 발생 (oeroded)
    ///
    pub fn rust_item(
        item: &mut Item,
        template: &ItemTemplate,
        log: &mut GameLog,
        turn: u64,
        is_player: bool,
    ) -> bool {
        if template.material == Material::Iron || template.material == Material::Metal {
            if item.oeroded < 3 {
                item.oeroded += 1;
                if is_player {
                    let msg = format!(
                        "Your {} {}s!",
                        template.name,
                        if template.material == Material::Iron {
                            "rust"
                        } else {
                            "corrode"
                        }
                    );
                    log.add(msg, turn);
                }
                return true;
            }
        }
        false
    }

    /// 산성에 의한 부식 (oeroded2) - 무기/방어구 전반
    pub fn corrode_item(
        item: &mut Item,
        template: &ItemTemplate,
        log: &mut GameLog,
        turn: u64,
        is_player: bool,
    ) -> bool {
        //
        let susceptible = matches!(
            template.material,
            Material::Iron | Material::Metal | Material::Copper | Material::Silver | Material::Gold
        );

        if susceptible {
            if item.oeroded2 < 3 {
                item.oeroded2 += 1;
                if is_player {
                    log.add(format!("Your {} is corroded!", template.name), turn);
                }
                return true;
            }
        }
        false
    }

    /// 화염에 의한 손상 (oeroded2) - 가연성 재질
    pub fn burn_item(
        item: &mut Item,
        template: &ItemTemplate,
        log: &mut GameLog,
        turn: u64,
        is_player: bool,
    ) -> bool {
        let flammable = matches!(
            template.material,
            Material::Wood | Material::Leather | Material::Cloth | Material::Paper
        );

        if flammable {
            if item.oeroded2 < 3 {
                item.oeroded2 += 1;
                if is_player {
                    log.add(format!("Your {} burns!", template.name), turn);
                }
                return true;
            } else {
                // 완전히 타버림 (소멸)
                if is_player {
                    log.add(
                        format!("Your {} is destroyed by fire!", template.name),
                        turn,
                    );
                }
                return true; // 호출 측에서 소멸 처리 필요
            }
        }
        false
    }

    /// 유기물 부패 (oeroded2)
    pub fn rot_item(
        item: &mut Item,
        template: &ItemTemplate,
        log: &mut GameLog,
        turn: u64,
        is_player: bool,
    ) -> bool {
        if template.material == Material::Flesh || template.material == Material::Veggy {
            if item.oeroded2 < 3 {
                item.oeroded2 += 1;
                if is_player {
                    log.add(format!("Your {} starts to rot!", template.name), turn);
                }
                return true;
            }
        }
        false
    }

    /// 물 노출에 의한 아이템 손상 (재귀적 처리)
    ///
    pub fn water_exposure_recursive(
        world: &mut legion::world::SubWorld,
        inventory_items: &[legion::Entity],
        protection_chance: i32,
        log: &mut GameLog,
        turn: u64,
        is_player: bool,
        item_manager: &crate::core::entity::object::ItemManager,
        rng: &mut crate::util::rng::NetHackRng,
    ) {
        use crate::core::entity::object::ItemClass;
        use crate::core::entity::Inventory;

        for &item_ent in inventory_items {
            // 보호 확률 체크 (기술 등)
            if rng.rn2(100) < protection_chance {
                continue;
            }

            let mut is_oilskin = false;
            let mut sub_items = Vec::new();

            if let Ok(entry) = world.entry_ref(item_ent) {
                //
                if let Ok(item) = entry.get_component::<Item>() {
                    if item.kind == ItemKind::OilskinBag {
                        is_oilskin = true;
                    }
                }
                //
                if let Ok(sub_inv) = entry.get_component::<Inventory>() {
                    sub_items = sub_inv.items.clone();
                }
            }

            // Oilskin Bag이라면 내부 아이템 보호 (재귀 중단)
            if is_oilskin {
                continue;
            }

            // 개별 아이템 데미지 처리
            if let Ok(mut entry) = world.entry_mut(item_ent) {
                if let Ok(item) = entry.get_component_mut::<Item>() {
                    if let Some(it) = item_manager.get_by_kind(item.kind) {
                        if it.class == ItemClass::Potion && item.kind != ItemKind::Water {
                            item.kind = ItemKind::Water;
                            if is_player {
                                log.add(format!("One of your {}s dilutes.", it.name), turn);
                            }
                        } else if (it.class == ItemClass::Scroll
                            || it.class == ItemClass::Spellbook)
                            && item.kind != ItemKind::BlankPaper
                        {
                            item.kind = ItemKind::BlankPaper;
                            if is_player {
                                log.add(format!("One of your {}s becomes blank.", it.name), turn);
                            }
                        }
                    }
                }
            }

            //
            if !sub_items.is_empty() {
                Self::water_exposure_recursive(
                    world,
                    &sub_items,
                    protection_chance,
                    log,
                    turn,
                    is_player,
                    item_manager,
                    rng,
                );
            }
        }
    }
}

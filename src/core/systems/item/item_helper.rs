use crate::core::entity::object::ItemTemplate;
use crate::core::entity::{identity::IdentityTable, Item};
use crate::generated::ItemKind;
use legion::EntityStore;

pub struct ItemHelper;

impl ItemHelper {
    ///
    pub fn get_name(
        item: &Item,
        template: &ItemTemplate,
        ident_table: Option<&IdentityTable>,
    ) -> String {
        let mut name = String::new();

        // 1. 축복/저주 상태
        if item.bknown {
            if item.blessed {
                name.push_str("blessed ");
            } else if item.cursed {
                name.push_str("cursed ");
            } else {
                name.push_str("uncursed ");
            }
        }

        // 2. 강화 수치
        if item.known && item.spe != 0 {
            if item.spe > 0 {
                name.push_str(&format!("+{} ", item.spe));
            } else {
                name.push_str(&format!("{} ", item.spe));
            }
        }

        // 3. 수량
        if item.quantity > 1 {
            name.push_str(&format!("{} ", item.quantity));
        }

        // 4. 기반 이름 또는 외형
        let mut base_name = if item.known {
            template.name.clone()
        } else {
            if let Some(desc) = Self::get_description(item, template) {
                // Call된 이름이 있는지 확인
                if let Some(it) = ident_table {
                    if let Some(ident) = it.mapping.get(&desc) {
                        if let Some(call) = &ident.call_name {
                            format!("{} called {}", desc, call)
                        } else {
                            desc
                        }
                    } else {
                        desc
                    }
                } else {
                    desc
                }
            } else {
                template.name.clone()
            }
        };

        // 시체 이름 처리
        if item.kind == ItemKind::Corpse {
            if let Some(m_name) = &item.corpsenm {
                base_name = format!("{} {}", m_name, base_name);
            }
        }

        name.push_str(&base_name);

        //
        if let Some(user_name) = &item.user_name {
            name.push_str(&format!(" named {}", user_name));
        }

        name
    }

    pub fn get_description(item: &Item, template: &ItemTemplate) -> Option<String> {
        if item.dknown {
            template.description.clone()
        } else {
            None
        }
    }

    ///
    pub fn calculate_weight(item_ent: legion::Entity, world: &legion::World) -> i32 {
        if let Ok(entry) = world.entry_ref(item_ent) {
            if let Ok(item) = entry.get_component::<Item>() {
                let mut contents_weight: i32 = 0;

                // 용기일 경우 내용물 무게 합산
                if let Ok(inv) = entry.get_component::<crate::core::entity::Inventory>() {
                    for &sub_item in &inv.items {
                        contents_weight += Self::calculate_weight(sub_item, world);
                    }

                    // Bag of Holding 효과 (NetHack 3.6.x 기준)
                    if item.kind == ItemKind::BagOfHolding {
                        if item.blessed {
                            contents_weight = contents_weight / 4; // 25%
                        } else if item.cursed {
                            contents_weight = contents_weight * 2; // 200%
                        } else {
                            contents_weight = contents_weight / 2; // 50%
                        }
                    }
                }
                return (item.weight as i32) + contents_weight;
            }
        }
        0
    }
}

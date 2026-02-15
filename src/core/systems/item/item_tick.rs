use crate::core::entity::{Item, Position};
use crate::generated::ItemKind;
use crate::ui::log::GameLog;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{Entity, EntityStore, IntoQuery};

/// 아이템 상태 전산 시스템 (부패 등)
#[legion::system]
#[write_component(Item)]
#[write_component(crate::core::entity::LightSource)]
#[read_component(Position)]
#[read_component(crate::core::entity::InContainerTag)]
#[read_component(crate::core::entity::ContainerProperties)]
pub fn item_tick(
    world: &mut SubWorld,
    #[resource] turn: &u64,
    #[resource] log: &mut GameLog,
    command_buffer: &mut CommandBuffer,
) {
    //
    // 나머지(ContainerProperties)는 container_world에 남김
    let (mut item_world, container_world) = world.split::<(
        &mut Item,
        &mut crate::core::entity::LightSource,
        &Position,
        &crate::core::entity::InContainerTag,
    )>();

    let mut query = <(
        Entity,
        &mut Item,
        Option<&mut crate::core::entity::LightSource>,
        Option<&Position>,
        Option<&crate::core::entity::InContainerTag>,
    )>::query();

    for (entity, item, light, pos, in_container) in query.iter_mut(&mut item_world) {
        // IceBox Check (Phase 48)
        let mut is_preserved = false;
        if let Some(tag) = in_container {
            if let Ok(entry) = container_world.entry_ref(tag.container) {
                if let Ok(props) = entry.get_component::<crate::core::entity::ContainerProperties>()
                {
                    if props.typ == crate::core::entity::ContainerType::IceBox {
                        is_preserved = true;
                    }
                }
            }
        }

        if is_preserved {
            continue;
        }

        // 0. 등불 연료 소모 (Phase 43)
        if let Some(light) = light {
            if light.lit && item.age > 0 && item.kind != ItemKind::MagicLamp {
                item.age -= 1;
                if item.age == 0 {
                    light.lit = false;
                    if pos.is_some() {
                        log.add(format!("The {} goes out.", item.kind), *turn);
                    } else {
                        log.add(format!("Your {} goes out.", item.kind), *turn);
                    }
                } else if item.age == 50 {
                    if pos.is_none() {
                        log.add(format!("Your {} is flickering.", item.kind), *turn);
                    }
                }
            }
        }
        // 1. 시체 부패 (Rotting) — [v2.0.0] corpsenm 또는 이름에 "corpse" 포함 체크
        if item.corpsenm.is_some() || item.kind.is_corpse() {
            let age_diff = *turn - item.age;

            if age_diff == 50 {
                if pos.is_some() {
                    log.add("You smell something rotten.", *turn);
                } else {
                    log.add("Something in your bag smells rotten.", *turn);
                }
            } else if age_diff > 70 {
                if pos.is_some() {
                    log.add("A corpse rots away.", *turn);
                } else {
                    log.add("A corpse in your bag rots away.", *turn);
                }
                command_buffer.remove(*entity);
            }
        }
    }
}

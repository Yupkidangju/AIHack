use crate::core::entity::{Item, Position};
use crate::generated::ItemKind;
use legion::{Entity, EntityStore, IntoQuery};

/// [v3.0.0] 아이템 상태 전산 시스템 (부패/연료 소모 등)
/// Gather-Apply 패턴: 먼저 모든 아이템 상태를 수집한 뒤, 일괄 적용
pub fn item_tick_system(ctx: &mut crate::core::context::GameContext) {
    // === Phase 1: Gather (컨테이너 정보 수집) ===
    // IceBox에 들어있는 아이템 엔티티 목록 수집
    let mut icebox_items: Vec<Entity> = Vec::new();
    {
        let mut container_query = <(Entity, &crate::core::entity::InContainerTag)>::query();

        for (entity, tag) in container_query.iter(ctx.world) {
            if let Ok(entry) = ctx.world.entry_ref(tag.container) {
                if let Ok(props) = entry.get_component::<crate::core::entity::ContainerProperties>()
                {
                    if props.typ == crate::core::entity::ContainerType::IceBox {
                        icebox_items.push(*entity);
                    }
                }
            }
        }
    }

    // === Phase 2: Apply (아이템 상태 업데이트) ===
    // 삭제할 엔티티 수집
    let mut to_remove: Vec<Entity> = Vec::new();
    // 로그 메시지 수집
    let mut messages: Vec<(String, u64)> = Vec::new();

    {
        let mut query = <(
            Entity,
            &mut Item,
            Option<&mut crate::core::entity::LightSource>,
            Option<&Position>,
        )>::query();

        let turn = ctx.turn;

        for (entity, item, light, pos) in query.iter_mut(ctx.world) {
            // IceBox에 보존된 아이템은 건너뜀
            if icebox_items.contains(entity) {
                continue;
            }

            // 0. 등불 연료 소모 (Phase 43)
            if let Some(light) = light {
                if light.lit && item.age > 0 && item.kind != ItemKind::MagicLamp {
                    item.age -= 1;
                    if item.age == 0 {
                        light.lit = false;
                        if pos.is_some() {
                            messages.push((format!("The {} goes out.", item.kind), turn));
                        } else {
                            messages.push((format!("Your {} goes out.", item.kind), turn));
                        }
                    } else if item.age == 50 {
                        if pos.is_none() {
                            messages.push((format!("Your {} is flickering.", item.kind), turn));
                        }
                    }
                }
            }

            // 1. 시체 부패 (Rotting)
            if item.corpsenm.is_some() || item.kind.is_corpse() {
                let age_diff = turn - item.age;

                if age_diff == 50 {
                    if pos.is_some() {
                        messages.push(("You smell something rotten.".to_string(), turn));
                    } else {
                        messages.push(("Something in your bag smells rotten.".to_string(), turn));
                    }
                } else if age_diff > 70 {
                    if pos.is_some() {
                        messages.push(("A corpse rots away.".to_string(), turn));
                    } else {
                        messages.push(("A corpse in your bag rots away.".to_string(), turn));
                    }
                    to_remove.push(*entity);
                }
            }
        }
    }

    // === Phase 3: 로그 출력 + 엔티티 삭제 ===
    for (msg, turn) in messages {
        ctx.log.add(msg, turn);
    }
    for entity in to_remove {
        ctx.world.remove(entity);
    }
}

// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::entity::player::Player;
use crate::core::entity::{Inventory, Item};
use legion::world::SubWorld;
use legion::*;

/// 하중(Encumbrance) 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encumbrance {
    Unencumbered, // 정상
    Burdened,     // 부담 (Burdened)
    Stressed,     // 스트레스 (Stressed)
    Strained,     // 긴장 (Strained)
    Overtaxed,    // 과부하 (Overtaxed)
    Overloaded,   // 초과 (Overloaded)
}

///
/// 원본: invent.c: weight_cap 이식 기초
pub fn calculate_total_weight(world: &SubWorld, inv: &Inventory) -> u32 {
    let mut total_weight = 0;

    for &item_ent in &inv.items {
        if let Ok(entry) = world.entry_ref(item_ent) {
            if let Ok(item) = entry.get_component::<Item>() {
                let mut item_weight = item.weight * item.quantity.max(1);

                // 마법 가방(Bag of Holding) 효과 처리 (Phase 48.2)
                if item.kind == crate::generated::ItemKind::BagOfHolding {
                    if let Ok(sub_inv) = entry.get_component::<Inventory>() {
                        let contents_weight = calculate_total_weight(world, sub_inv);

                        //
                        //
                        //
                        //
                        let reduction_factor = if item.blessed {
                            0.25
                        } else if item.cursed {
                            2.0 // 저주받은 가방은 내용물 무게를 2배로 만듦 (악랄한 nethack 메커니즘)
                        } else {
                            0.5
                        };

                        item_weight += (contents_weight as f32 * reduction_factor) as u32;
                    }
                } else if let Ok(sub_inv) = entry.get_component::<Inventory>() {
                    // 일반 용기인 경우 내용물 무게 그대로 합산
                    item_weight += calculate_total_weight(world, sub_inv);
                }

                total_weight += item_weight;
            }
        }
    }

    total_weight
}

///
pub fn get_carrying_capacity(player: &Player) -> u32 {
    let s = player.str.base;
    let c = player.con.base;

    //
    //
    let mut cap = (s + c) * 25 + 50;

    // 강화 수치나 특수 장비에 따른 보정 가능 (예: 힘의 장갑)
    cap as u32
}

/// 현재 무게에 따른 하중 상태 판정
pub fn get_encumbrance(total_weight: u32, capacity: u32) -> Encumbrance {
    if total_weight <= capacity {
        Encumbrance::Unencumbered
    } else if total_weight <= (capacity * 3) / 2 {
        Encumbrance::Burdened
    } else if total_weight <= capacity * 2 {
        Encumbrance::Stressed
    } else if total_weight <= (capacity * 5) / 2 {
        Encumbrance::Strained
    } else if total_weight <= capacity * 3 {
        Encumbrance::Overtaxed
    } else {
        Encumbrance::Overloaded
    }
}

///
#[legion::system]
#[read_component(crate::core::entity::PlayerTag)]
#[read_component(crate::core::entity::player::Player)]
#[read_component(Inventory)]
#[write_component(crate::core::entity::status::StatusBundle)]
pub fn update_encumbrance(
    world: &mut SubWorld,
    #[resource] log: &mut crate::ui::log::GameLog,
    #[resource] turn: &u64,
) {
    use crate::core::entity::status::StatusFlags;

    //
    let mut p_query = <(&crate::core::entity::player::Player, &Inventory)>::query()
        .filter(legion::component::<crate::core::entity::PlayerTag>());
    let (p_stats, p_inv) = match p_query.iter(world).next() {
        Some((s, i)) => (s.clone(), i.clone()),
        None => return,
    };

    // 2. 무게 및 용량 계산
    let total_weight = calculate_total_weight(world, &p_inv);
    let capacity = get_carrying_capacity(&p_stats);
    let enc_state = get_encumbrance(total_weight, capacity);

    // 3. 상태 플래그 갱신
    let mut s_query = <&mut crate::core::entity::status::StatusBundle>::query()
        .filter(legion::component::<crate::core::entity::PlayerTag>());
    if let Some(status) = s_query.iter_mut(world).next() {
        // 기존 하중 플래그 제거
        status.permanent.remove(StatusFlags::BURDENED);
        status.permanent.remove(StatusFlags::STRESSED);
        status.permanent.remove(StatusFlags::STRAINED);
        status.permanent.remove(StatusFlags::OVERTAXED);
        status.permanent.remove(StatusFlags::OVERLOADED);

        // 새 하중 플래그 설정 및 로그 출력 (상태 변화 시)
        match enc_state {
            Encumbrance::Burdened => {
                status.permanent.insert(StatusFlags::BURDENED);
            }
            Encumbrance::Stressed => {
                status.permanent.insert(StatusFlags::STRESSED);
            }
            Encumbrance::Strained => {
                status.permanent.insert(StatusFlags::STRAINED);
            }
            Encumbrance::Overtaxed => {
                status.permanent.insert(StatusFlags::OVERTAXED);
            }
            Encumbrance::Overloaded => {
                status.permanent.insert(StatusFlags::OVERLOADED);
            }
            _ => {}
        }
    }
}

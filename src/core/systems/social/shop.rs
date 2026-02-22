// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use super::InteractionProvider; // [R8-3] trait 메서드 접근용
use crate::core::dungeon::Grid;
use crate::core::entity::{Inventory, Item, PlayerTag, Position, ShopkeeperTag};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

/// 상점(Shop) 시스템
/// [v2.20.0 R8-3] InteractionProvider 경유로 상점 대사 생성
pub fn try_pay(
    world: &mut SubWorld,
    log: &mut GameLog,
    provider: &dyn super::InteractionProvider,
) -> bool {
    //
    let (mut gold, inv_items, _cha) = {
        let mut p_query = <(&mut crate::core::entity::player::Player, &Inventory)>::query()
            .filter(component::<PlayerTag>());
        if let Some((p_stats, inventory)) = p_query.iter_mut(world).next() {
            (p_stats.gold, inventory.items.clone(), p_stats.cha.base)
        } else {
            return false;
        }
    };

    let mut total_due = 0;
    let mut items_to_pay = Vec::new();

    collect_unpaid_recursive(world, &inv_items, &mut total_due, &mut items_to_pay);

    if total_due == 0 {
        // [R8-3] provider 경유 대사
        log.add(
            provider.generate_shop_reaction("nothing_owed", "Shopkeeper", 0),
            log.current_turn,
        );
        return false;
    }

    if gold >= total_due as u64 {
        gold -= total_due as u64;
        // [R8-3] provider 경유 대사
        log.add(
            provider.generate_shop_reaction("paid", "Shopkeeper", total_due as i64),
            log.current_turn,
        );

        //
        let mut p_query =
            <&mut crate::core::entity::player::Player>::query().filter(component::<PlayerTag>());
        if let Some(p_stats) = p_query.iter_mut(world).next() {
            p_stats.gold = gold;
        }

        // 미지급 상태 해제
        for item_ent in items_to_pay {
            if let Ok(mut entry) = world.entry_mut(item_ent) {
                if let Ok(item) = entry.get_component_mut::<Item>() {
                    item.unpaid = false;
                }
            }
        }
        return true;
    } else {
        // [R8-3] provider 경유 대사
        log.add(
            provider.generate_shop_reaction("too_poor", "Shopkeeper", total_due as i64),
            log.current_turn,
        );
        return false;
    }
}

///
pub fn try_identify_service(
    world: &mut SubWorld,
    log: &mut GameLog,
    turn: u64,
    provider: &dyn super::InteractionProvider,
) -> bool {
    let mut player_pos = Position { x: 0, y: 0 };
    let mut gold = 0;

    //
    {
        let mut p_query = <(&Position, &crate::core::entity::player::Player)>::query()
            .filter(component::<PlayerTag>());
        if let Some((pos, p_stats)) = p_query.iter(world).next() {
            player_pos = *pos;
            gold = p_stats.gold;
        } else {
            return false;
        }
    }

    // 2. 주변에 상점 주인이 있는지 확인
    let mut shk_found = false;
    {
        let mut sk_query = <(&Position, &crate::core::entity::Monster)>::query()
            .filter(component::<ShopkeeperTag>());
        for (sk_pos, _sk_mon) in sk_query.iter(world) {
            let dx = (sk_pos.x - player_pos.x).abs();
            let dy = (sk_pos.y - player_pos.y).abs();
            if dx <= 1 && dy <= 1 {
                shk_found = true;
                break;
            }
        }
    }

    if !shk_found {
        log.add(
            provider.generate_shop_reaction("no_shopkeeper", "", 0),
            turn,
        );
        return false;
    }

    let id_cost = 500;
    if gold < id_cost {
        log.add(
            provider.generate_shop_reaction("too_poor", "Shopkeeper", id_cost as i64),
            turn,
        );
        return false;
    }

    //
    let mut identified = false;
    let mut p_inv_items = Vec::new();
    {
        let mut p_query = <&Inventory>::query().filter(component::<PlayerTag>());
        if let Some(inv) = p_query.iter(world).next() {
            p_inv_items = inv.items.clone();
        }
    }

    for item_ent in p_inv_items {
        if let Ok(mut entry) = world.entry_mut(item_ent) {
            if let Ok(item) = entry.get_component_mut::<Item>() {
                if !item.known {
                    item.known = true;
                    item.bknown = true;
                    identified = true;

                    log.add(
                        provider.generate_shop_reaction("identify", "Shopkeeper", 0),
                        turn,
                    );
                    break;
                }
            }
        }
    }

    if identified {
        // 골드 차감
        let mut p_query =
            <&mut crate::core::entity::player::Player>::query().filter(component::<PlayerTag>());
        if let Some(p_stats) = p_query.iter_mut(world).next() {
            p_stats.gold -= id_cost;
        }
        return true;
    } else {
        log.add("You have nothing unidentified.", turn);
        return false;
    }
}

/// 상점 주인 AI 및 도둑질 감시
#[legion::system]
#[read_component(Position)]
#[read_component(PlayerTag)]
#[read_component(ShopkeeperTag)]
#[read_component(Item)]
#[read_component(Inventory)]
#[read_component(crate::core::entity::Monster)]
#[write_component(Position)]
pub fn shopkeeper_update(
    world: &mut SubWorld,
    #[resource] grid: &Grid,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] rng: &mut NetHackRng,
    #[resource] provider: &super::DefaultInteractionProvider,
    command_buffer: &mut legion::systems::CommandBuffer,
) {
    //
    let mut p_query = <(&Position, &Inventory)>::query().filter(component::<PlayerTag>());
    let (p_pos, p_inv) = match p_query.iter(world).next() {
        Some(d) => (*d.0, d.1.clone()),
        None => return,
    };

    let p_tile = grid.get_tile(p_pos.x as usize, p_pos.y as usize);
    let u_in_shop = p_tile.map(|t| t.shop_type > 0).unwrap_or(false);
    let u_room_id = p_tile.map(|t| t.roomno as i32).unwrap_or(-1);

    //
    let has_unpaid_player = has_unpaid_recursive(world, &p_inv);

    // 2. 상점 주인 루프
    let mut sk_query = <(Entity, &mut Position, &crate::core::entity::Monster)>::query()
        .filter(component::<ShopkeeperTag>());
    for (sk_ent, sk_pos, sk_mon) in sk_query.iter_mut(world) {
        let sk_tile = grid.get_tile(sk_pos.x as usize, sk_pos.y as usize);
        let sk_room_id = sk_tile.map(|t| t.roomno as i32).unwrap_or(-1);

        //
        if u_room_id == sk_room_id && sk_room_id != -1 {
            // 입구 근처 대사 (Entry/Exit Dialogue)
            let dist_sq = (p_pos.x - sk_pos.x).pow(2) + (p_pos.y - sk_pos.y).pow(2);
            if dist_sq < 9 && rng.rn2(10) == 0 {
                if has_unpaid_player {
                    log.add(
                        provider.generate_shop_reaction("pay_reminder", sk_mon.kind.as_str(), 0),
                        *turn,
                    );
                } else {
                    log.add(
                        provider.generate_shop_reaction("welcome", sk_mon.kind.as_str(), 0),
                        *turn,
                    );
                }
            }

            if has_unpaid_player {
                //
                let mut exit_pos = None;
                for dx in -5..=5 {
                    for dy in -5..=5 {
                        let tx = sk_pos.x + dx;
                        let ty = sk_pos.y + dy;
                        if tx < 0 || tx >= 80 || ty < 0 || ty >= 21 {
                            continue;
                        }
                        if let Some(t) = grid.get_tile(tx as usize, ty as usize) {
                            //
                            if (t.typ == crate::core::dungeon::tile::TileType::OpenDoor
                                || t.typ == crate::core::dungeon::tile::TileType::Door)
                                && t.roomno as i32 == sk_room_id
                            {
                                exit_pos = Some((tx, ty));
                                break;
                            }
                        }
                    }
                    if exit_pos.is_some() {
                        break;
                    }
                }

                if let Some((ex, ey)) = exit_pos {
                    // 상점 주인이 문 앞에 서서 길을 막음 (shk.c:shk_move)
                    if sk_pos.x != ex || sk_pos.y != ey {
                        //
                        let nx = if sk_pos.x < ex {
                            sk_pos.x + 1
                        } else if sk_pos.x > ex {
                            sk_pos.x - 1
                        } else {
                            sk_pos.x
                        };
                        let ny = if sk_pos.y < ey {
                            sk_pos.y + 1
                        } else if sk_pos.y > ey {
                            sk_pos.y - 1
                        } else {
                            sk_pos.y
                        };

                        // 문으로 이동 시도
                        sk_pos.x = nx;
                        sk_pos.y = ny;
                    }
                }
            }
        }

        // 3. 도둑질 즉각 판정 (상점을 이미 벗어난 경우)
        if !u_in_shop && !sk_mon.hostile {
            if has_unpaid_player {
                stop_thief(*sk_ent, sk_mon, log, *turn, command_buffer, provider);
            }
        }
    }
}

///
fn has_unpaid_recursive(world: &SubWorld, inv: &Inventory) -> bool {
    for &item_ent in &inv.items {
        if let Ok(entry) = world.entry_ref(item_ent) {
            //
            if let Ok(item) = entry.get_component::<Item>() {
                if item.unpaid {
                    return true;
                }
            }
            //
            if let Ok(sub_inv) = entry.get_component::<Inventory>() {
                if has_unpaid_recursive(world, sub_inv) {
                    return true;
                }
            }
        }
    }
    false
}

/// 도둑질 발견 시 상점 주인 분노 처리
fn stop_thief(
    sk_ent: Entity,
    sk_mon: &crate::core::entity::Monster,
    log: &mut GameLog,
    turn: u64,
    command_buffer: &mut CommandBuffer,
    provider: &dyn super::InteractionProvider,
) {
    // [R8-3] provider 경유 대사
    log.add_colored(
        provider.generate_shop_reaction("thief", sk_mon.kind.as_str(), 0),
        [255, 50, 50],
        turn,
    );
    let mut new_mon = sk_mon.clone();
    new_mon.hostile = true;
    command_buffer.add_component(sk_ent, new_mon);

    //
}

///
fn collect_unpaid_recursive(
    world: &SubWorld,
    items: &[Entity],
    total_due: &mut u32,
    items_to_pay: &mut Vec<Entity>,
) {
    for &item_ent in items {
        if let Ok(entry) = world.entry_ref(item_ent) {
            //
            if let Ok(item) = entry.get_component::<Item>() {
                if item.unpaid {
                    *total_due += item.price;
                    items_to_pay.push(item_ent);
                }
            }
            //
            if let Ok(sub_inv) = entry.get_component::<Inventory>() {
                collect_unpaid_recursive(world, &sub_inv.items, total_due, items_to_pay);
            }
        }
    }
}

/// 구매 가격 산출 (shk.c: self_price 이식)
pub fn get_buy_price(
    item: &Item,
    template: &crate::core::entity::object::ItemTemplate,
    player: &crate::core::entity::player::Player,
    shop_type: u8,
) -> u32 {
    let mut price = item.price;
    if price == 0 {
        price = template.cost as u32;
    }

    // 1. 매력(Cha)에 따른 흥정 보정
    if player.cha.base < 6 {
        price = (price * 3) / 2; // 150%
    } else if player.cha.base < 8 {
        price = (price * 4) / 3; // 133%
    } else if player.cha.base > 15 {
        price = (price * 3) / 4; // 75%
    }

    // 2. 관광객(Tourist) 페널티 (shk.c: self_price)
    //
    if player.role == crate::core::entity::player::PlayerClass::Tourist && player.exp_level < 15 {
        price = (price * 4) / 3; // 133%
    }

    // 3. 상점 종류별 가중치 (Specialty Shops)
    price = (price as f32 * get_shop_multiplier(template, shop_type, true)) as u32;

    // 수량 반영
    if item.quantity > 0 {
        price *= item.quantity;
    }

    price.max(1)
}

/// 판매 가격 산출 (shk.c: sellobj 이식 기초)
pub fn get_sell_price(
    item: &Item,
    template: &crate::core::entity::object::ItemTemplate,
    player: &crate::core::entity::player::Player,
    shop_type: u8,
) -> u32 {
    let mut price = item.price;
    if price == 0 {
        price = template.cost as u32;
    }

    // NetHack: 보통 매입 가격의 1/3 ~ 1/2 수준
    price /= 3;

    // 1. 식별 여부 보정
    // 정체를 모르는 물건은 싸게 매입함
    if !item.known {
        price /= 2;
    }

    // 2. 저주 상태 보정
    if item.bknown && item.cursed {
        price /= 4;
    }

    // 3. 상점 종류별 가중치
    price = (price as f32 * get_shop_multiplier(template, shop_type, false)) as u32;

    // 매력 보너스 (판매 시에도 약간의 이득)
    if player.cha.base > 16 {
        price = (price * 11) / 10; // 110%
    }

    price *= item.quantity.max(1);
    price.max(1)
}

/// 상점 종류 및 아이템 일치 여부에 따른 가격 배수
fn get_shop_multiplier(
    template: &crate::core::entity::object::ItemTemplate,
    shop_type: u8,
    is_buying: bool,
) -> f32 {
    use crate::core::entity::object::ItemClass;

    // 0: 일반(General Store), 1: 무기, 2: 방어구, 3: 마법서/스크롤, 4: 포션, 5: 보석, 6: 도구, 7: 식료품
    let match_specialty = match shop_type {
        1 => template.class == ItemClass::Weapon,
        2 => template.class == ItemClass::Armor,
        3 => template.class == ItemClass::Spellbook || template.class == ItemClass::Scroll,
        4 => template.class == ItemClass::Potion,
        5 => template.class == ItemClass::Gem || template.class == ItemClass::Rock,
        6 => template.class == ItemClass::Tool,
        7 => template.class == ItemClass::Food,
        _ => false,
    };

    if match_specialty {
        if is_buying {
            1.0
        } else {
            1.2
        } // 전문점에서 팔 때 조금 더 쳐줌
    } else {
        if shop_type == 0 {
            1.0
        } else {
            0.5
        } // 전문점에서 엉뚱한 물건 팔면 헐값
    }
}

// =============================================================================
// [v2.3.1] shk.c 확장 이식
// 원본: nethack-3.6.7/src/shk.c (4,542줄)
//
// 상점 종류 데이터, 상점 주인 이름, 부채 관리, 도둑 경보, 가격 감정 등
// =============================================================================

/// [v2.3.1] 상점 종류 (원본: shoptype_t / shclasses[])
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopType {
    /// 잡화점
    General = 0,
    /// 무기상
    Weapon = 1,
    ///
    Armor = 2,
    ///
    Scroll = 3,
    /// 물약점
    Potion = 4,
    /// 보석상
    Gem = 5,
    /// 도구점
    Tool = 6,
    ///
    Food = 7,
    /// 촛불/조명점
    Candle = 8,
    /// 책방
    Book = 9,
}

/// [v2.3.1] 상점 종류 데이터 (원본: shclass[])
#[derive(Debug, Clone)]
pub struct ShopTypeInfo {
    pub typ: ShopType,
    pub name: &'static str,
    pub buy_markup: f32,  // 매입 마크업
    pub sell_markup: f32, // 매출 마크업
    pub probability: i32, // 생성 확률 가중치
}

pub fn shop_type_table() -> Vec<ShopTypeInfo> {
    vec![
        ShopTypeInfo {
            typ: ShopType::General,
            name: "general store",
            buy_markup: 1.0,
            sell_markup: 0.3,
            probability: 40,
        },
        ShopTypeInfo {
            typ: ShopType::Weapon,
            name: "weapon shop",
            buy_markup: 1.0,
            sell_markup: 0.35,
            probability: 10,
        },
        ShopTypeInfo {
            typ: ShopType::Armor,
            name: "armor shop",
            buy_markup: 1.0,
            sell_markup: 0.35,
            probability: 10,
        },
        ShopTypeInfo {
            typ: ShopType::Scroll,
            name: "scroll shop",
            buy_markup: 1.2,
            sell_markup: 0.4,
            probability: 8,
        },
        ShopTypeInfo {
            typ: ShopType::Potion,
            name: "potion shop",
            buy_markup: 1.2,
            sell_markup: 0.4,
            probability: 8,
        },
        ShopTypeInfo {
            typ: ShopType::Gem,
            name: "gem shop",
            buy_markup: 1.3,
            sell_markup: 0.5,
            probability: 5,
        },
        ShopTypeInfo {
            typ: ShopType::Tool,
            name: "tool shop",
            buy_markup: 1.0,
            sell_markup: 0.3,
            probability: 8,
        },
        ShopTypeInfo {
            typ: ShopType::Food,
            name: "delicatessen",
            buy_markup: 0.8,
            sell_markup: 0.25,
            probability: 8,
        },
        ShopTypeInfo {
            typ: ShopType::Candle,
            name: "lighting shop",
            buy_markup: 1.1,
            sell_markup: 0.3,
            probability: 2,
        },
        ShopTypeInfo {
            typ: ShopType::Book,
            name: "bookshop",
            buy_markup: 1.5,
            sell_markup: 0.5,
            probability: 1,
        },
    ]
}

///
pub fn shopkeeper_names() -> Vec<&'static str> {
    vec![
        "Asidonhopo",
        "Shkusky",
        "Astrstrstr",
        "Lansen",
        "Dansen",
        "Nansen",
        "Olansen",
        "Izchak",
        "Magenta",
        "Moodkee",
        "Yigal",
        "Flint",
        "Candice",
        "Rocky",
        "Moonchild",
        "Hawkins",
        "Delphi",
        "Rameses",
        "Kelp",
        "Wilamina",
        "Birdperson",
        "Snuffkin",
        "Grimjaw",
        "Kettlesworth",
        "Pax",
        "Humperdink",
        "Elwood",
        "Daikatana",
        "Merkin",
        "Sethbridge",
    ]
}

/// [v2.3.1] 랜덤 상점 주인 이름 생성
pub fn random_shopkeeper_name(rng: &mut NetHackRng) -> &'static str {
    let names = shopkeeper_names();
    let idx = rng.rn2(names.len() as i32) as usize;
    names[idx]
}

/// [v2.3.1] 랜덤 상점 유형 선택 (원본: shk.c pick_shop_type)
pub fn random_shop_type(rng: &mut NetHackRng) -> ShopType {
    let table = shop_type_table();
    let total_weight: i32 = table.iter().map(|s| s.probability).sum();
    let mut roll = rng.rn2(total_weight);
    for info in &table {
        roll -= info.probability;
        if roll < 0 {
            return info.typ;
        }
    }
    ShopType::General
}

/// [v2.3.1] 부채(Debt) 관리 구조체 (원본: bp/billing)
#[derive(Debug, Clone, Default)]
pub struct ShopDebt {
    /// 총 부채 금액
    pub total: u64,
    ///
    pub items: Vec<(String, u32)>,
    /// 도둑질 횟수
    pub theft_count: i32,
    /// 파손 수
    pub damage_count: i32,
}

impl ShopDebt {
    pub fn new() -> Self {
        Self::default()
    }

    /// 아이템 부채 추가
    pub fn add_item(&mut self, name: &str, price: u32) {
        self.total += price as u64;
        self.items.push((name.to_string(), price));
    }

    /// 부채 전액 상환
    pub fn pay_all(&mut self, gold: &mut u64) -> bool {
        if *gold >= self.total {
            *gold -= self.total;
            self.total = 0;
            self.items.clear();
            true
        } else {
            false
        }
    }

    /// 부채 일부 상환
    pub fn pay_partial(&mut self, amount: u64, gold: &mut u64) -> u64 {
        let paid = amount.min(*gold).min(self.total);
        *gold -= paid;
        self.total -= paid;
        paid
    }

    /// 파손 비용 추가 (원본: cost_damage)
    pub fn add_damage(&mut self, amount: u32) {
        self.total += amount as u64;
        self.damage_count += 1;
    }
}

/// [v2.3.1] 상점 인사말 (원본: shk_greet)
pub fn shopkeeper_greeting(name: &str, shop_type: ShopType, has_debt: bool) -> String {
    let shop_name = match shop_type {
        ShopType::General => "store",
        ShopType::Weapon => "weapon shop",
        ShopType::Armor => "armor shop",
        ShopType::Scroll => "scroll shop",
        ShopType::Potion => "potion shop",
        ShopType::Gem => "gem shop",
        ShopType::Tool => "tool shop",
        ShopType::Food => "delicatessen",
        ShopType::Candle => "lighting shop",
        ShopType::Book => "bookshop",
    };
    if has_debt {
        format!("{} says: \"You still owe me money! Pay up!\"", name)
    } else {
        format!(
            "{} says: \"Welcome to my {}! Feel free to browse.\"",
            name, shop_name
        )
    }
}

/// [v2.3.1] 값어치 감정 (원본: price_identification)
pub fn price_identify(base_cost: u32, is_blessed: bool, is_cursed: bool) -> String {
    let buc = if is_blessed {
        "blessed "
    } else if is_cursed {
        "cursed "
    } else {
        ""
    };
    format!("{}item worth {} zorkmids", buc, base_cost)
}

/// [v2.3.1] 도둑 경보 수준 (원본: rob_shop 후 결과)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheftSeverity {
    /// 경고만
    Warning,
    /// 상점 주인 적대화
    Hostile,
    /// Kops 소환 (심각)
    KopsAlert,
    /// 즉시 공격
    Immediate,
}

/// [v2.3.1] 도둑질 심각도 판정 (원본: rob_shop)
pub fn theft_severity(debt_amount: u64, theft_count: i32, player_level: i32) -> TheftSeverity {
    if theft_count == 0 && debt_amount < 100 {
        TheftSeverity::Warning
    } else if theft_count < 3 && debt_amount < 1000 {
        TheftSeverity::Hostile
    } else if player_level >= 20 || debt_amount >= 5000 {
        TheftSeverity::Immediate
    } else {
        TheftSeverity::KopsAlert
    }
}

/// [v2.3.1] 파손 비용 계산 (원본: cost_damage)
pub fn damage_cost(item_value: u32) -> u32 {
    // 원본: 파손된 아이템 가격의 2배
    item_value * 2
}

/// [v2.3.1] 상점 크기 안 아이템 수 결정 (원본: shkinit)
pub fn shop_inventory_count(shop_type: ShopType, depth: i32, rng: &mut NetHackRng) -> i32 {
    let base = match shop_type {
        ShopType::General => 20,
        ShopType::Gem => 15,
        ShopType::Book => 8,
        ShopType::Candle => 10,
        _ => 14,
    };
    let extra = rng.rn2(depth / 3 + 1).min(10);
    base + extra
}

// =============================================================================
// [v2.3.5] 상점 확장 (원본 shk.c: advanced shop mechanics)
// =============================================================================

/// 상점 유형별 마크업 (원본: shk.c markup)
pub fn shop_markup(shop_type: ShopType) -> f32 {
    match shop_type {
        ShopType::General => 1.0,
        ShopType::Weapon => 1.2,
        ShopType::Armor => 1.2,
        ShopType::Scroll => 1.5,
        ShopType::Potion => 1.5,
        ShopType::Gem => 2.0,
        ShopType::Tool => 1.1,
        ShopType::Food => 0.8,
        ShopType::Candle => 1.3,
        ShopType::Book => 1.8,
    }
}

///
pub fn charisma_discount(charisma: i32) -> f32 {
    if charisma >= 18 {
        0.67
    }
    // 33% 할인
    else if charisma >= 16 {
        0.75
    }
    // 25% 할인
    else if charisma >= 14 {
        0.85
    }
    // 15% 할인
    else if charisma >= 12 {
        0.90
    }
    // 10% 할인
    else if charisma >= 10 {
        1.0
    } else if charisma >= 8 {
        1.1
    }
    // 10% 추가
    else if charisma >= 6 {
        1.25
    }
    // 25% 추가
    else {
        1.5
    } // 50% 추가
}

/// 감정 비용 (원본: shk.c identify cost)
pub fn identify_cost(item_base_value: u32, shop_type: ShopType) -> u32 {
    let base = item_base_value / 5 + 10;
    let bonus = match shop_type {
        ShopType::Scroll | ShopType::Potion | ShopType::Book => 0,
        _ => 20, // 비전문 상점은 더 비쌈
    };
    base + bonus
}

/// 상점 보안 등급 (원본: shk.c security)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopSecurity {
    None,      // 보안 없음
    Bell,      // 종 경보
    Dog,       // 경비견
    MagicTrap, // 마법 함정
    Golem,     // 골렘 경비원
}

/// 상점 깊이별 보안 (원본: shk.c shop security)
pub fn shop_security_level(depth: i32) -> ShopSecurity {
    if depth >= 20 {
        ShopSecurity::Golem
    } else if depth >= 15 {
        ShopSecurity::MagicTrap
    } else if depth >= 10 {
        ShopSecurity::Dog
    } else if depth >= 5 {
        ShopSecurity::Bell
    } else {
        ShopSecurity::None
    }
}

///
pub fn barter_value_ratio(player_charisma: i32) -> f32 {
    charisma_discount(player_charisma) * 0.5 // 물물교환은 판매가의 50%
}

/// 가격 변동 (원본: shk.c price fluctuation)
pub fn price_fluctuation(base_price: u32, depth: i32, rng: &mut NetHackRng) -> u32 {
    let variation = rng.rn2((depth / 2 + 1).max(1)) as u32;
    let up = rng.rn2(2) == 0;
    if up {
        base_price + variation
    } else {
        base_price.saturating_sub(variation)
    }
}

/// 상점 통계
#[derive(Debug, Clone, Default)]
pub struct ShopStatistics {
    pub items_bought: u32,
    pub items_sold: u32,
    pub gold_spent: u64,
    pub gold_earned: u64,
    pub thefts: u32,
    pub identify_uses: u32,
}

impl ShopStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_buy(&mut self, cost: u64) {
        self.items_bought += 1;
        self.gold_spent += cost;
    }
    pub fn record_sell(&mut self, value: u64) {
        self.items_sold += 1;
        self.gold_earned += value;
    }
}

#[cfg(test)]
mod shop_extended_tests {
    use super::*;

    #[test]
    fn test_markup() {
        assert!(shop_markup(ShopType::Gem) > shop_markup(ShopType::General));
    }

    #[test]
    fn test_charisma_discount() {
        assert!(charisma_discount(18) < charisma_discount(6));
    }

    #[test]
    fn test_identify_cost() {
        let cost_scroll = identify_cost(100, ShopType::Scroll);
        let cost_weapon = identify_cost(100, ShopType::Weapon);
        assert!(cost_weapon > cost_scroll);
    }

    #[test]
    fn test_security() {
        assert_eq!(shop_security_level(3), ShopSecurity::None);
        assert_eq!(shop_security_level(25), ShopSecurity::Golem);
    }

    #[test]
    fn test_shop_stats() {
        let mut s = ShopStatistics::new();
        s.record_buy(100);
        s.record_sell(50);
        assert_eq!(s.items_bought, 1);
        assert_eq!(s.gold_earned, 50);
    }
}

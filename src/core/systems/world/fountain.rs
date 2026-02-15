// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::Grid;
use crate::core::entity::player::Player;
use crate::core::entity::{Health, PlayerTag, Position};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;

/// 분수 마시기 (Drink from Fountain)
/// 원본 NetHack의 fountain.c:dryfountain(), drinkfountain() 등 이식
pub fn try_drink_fountain(
    world: &mut SubWorld,
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
    item_mgr: &crate::core::entity::object::ItemManager,
) -> bool {
    //
    let mut p_query = <(
        &mut Player,
        &mut Health,
        &crate::core::entity::status::StatusBundle,
        &Position,
    )>::query()
    .filter(component::<PlayerTag>());
    let mut player_packet = None;

    for (p, h, s, pos) in p_query.iter_mut(world) {
        player_packet = Some((p, h, s, pos));
        break;
    }

    if let Some((p_stats, p_health, _p_status, p_pos)) = player_packet {
        if let Some(tile) = grid.get_tile(p_pos.x as usize, p_pos.y as usize) {
            if tile.typ != TileType::Fountain {
                log.add("There is no fountain here.", turn);
                return false;
            }

            log.add("You drink from the fountain.", turn);

            // 분수 마름 체크 (Dry up)
            if rng.rn2(100) < 20 {
                if let Some(tile_mut) = grid.get_tile_mut(p_pos.x as usize, p_pos.y as usize) {
                    tile_mut.typ = TileType::Room;
                    log.add("The fountain dries up!", turn);
                    return true;
                }
            }

            // 분수 효과 (Effect)
            let roll = rng.rn2(100);

            if roll < 5 {
                // 5% Level Gain
                log.add("You feel a strange power flow through you.", turn);
                p_stats.experience += 500;
            } else if roll < 15 {
                // 10% Restoration
                log.add("You feel better.", turn);
                p_health.current = p_health.max;
            } else if roll < 25 {
                // 10% Monster Summon (Water Demon)
                log.add_colored("A water demon appears!", [255, 0, 0], turn);
            } else if roll < 30 {
                // 5% Wish (Simple)
                log.add_colored("You feel lucky! (Wish chance)", [255, 215, 0], turn);
                p_stats.gold += 5000;
            } else if roll < 50 {
                // 20% Coin
                let gold = rng.rn1(100, 50) as u64;
                p_stats.gold += gold;
                log.add(
                    format!("You found {} gold pieces in the fountain.", gold),
                    turn,
                );
            } else if roll < 60 {
                // 10% Sickness/Poison
                log.add("The water tastes foul!", turn);
                p_health.current -= 5;
            } else if roll < 70 {
                // 10% A gush of water
                log.add("A gush of water hits you!", turn);
                // 1/3 chance to rust random equipment
                if rng.rn2(3) == 0 {
                    rust_random_equipment(world, item_mgr, log, turn, rng);
                }
            } else {
                // 30% Just Water
                log.add("It tastes like clear water.", turn);
                p_stats.nutrition += 50;
            }

            return true;
        }
    }
    false
}

fn rust_random_equipment(
    world: &mut SubWorld,
    item_mgr: &crate::core::entity::object::ItemManager,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
) {
    use crate::core::entity::{Equipment, Item, PlayerTag};
    use crate::core::systems::item_damage::ItemDamageSystem;

    let mut p_query = <(Entity, &Equipment)>::query().filter(component::<PlayerTag>());
    let mut item_to_rust = None;

    if let Some((_ent, equip)) = p_query.iter(world).next() {
        let slots: Vec<_> = equip.slots.values().cloned().collect();
        if !slots.is_empty() {
            item_to_rust = Some(slots[rng.rn2(slots.len() as i32) as usize]);
        }
    }

    if let Some(item_ent) = item_to_rust {
        if let Ok(mut entry) = world.entry_mut(item_ent) {
            if let Ok(item) = entry.get_component_mut::<Item>() {
                if let Some(template) = item_mgr.get_by_kind(item.kind) {
                    ItemDamageSystem::rust_item(item, template, log, turn, true);
                }
            }
        }
    }
}

// =============================================================================
// [v2.3.3] 분수 효과 확장 (원본 fountain.c: dip/wash/wish)
// =============================================================================

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FountainWishResult {
    NoWish,
    MinorWish,  // 소형 소원 (금화/포션)
    MajorWish,  // 대형 소원 (아이템)
    WaterDemon, // 물 악마 소환
}

///
pub fn fountain_wish_roll(luck: i32, rng: &mut NetHackRng) -> FountainWishResult {
    let roll = rng.rn2(100);
    let luck_bonus = luck.clamp(-5, 5);

    if roll + luck_bonus < 3 {
        FountainWishResult::MajorWish // 3% 미만: 대형 소원
    } else if roll + luck_bonus < 8 {
        FountainWishResult::MinorWish // 5%: 소형 소원
    } else if roll < 18 {
        FountainWishResult::WaterDemon // 10%: 물 악마
    } else {
        FountainWishResult::NoWish
    }
}

/// 분수 고갈 확률 (원본: fountain.c dryup)
pub fn fountain_dryup_chance(times_used: u32, rng: &mut NetHackRng) -> bool {
    //
    let base_chance = 20 + (times_used * 5).min(40);
    rng.rn2(100) < base_chance as i32
}

// =============================================================================
// [v2.3.3] 분수에 담그기 (원본 fountain.c: dipfountain)
// =============================================================================

/// 담그기 효과 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DipEffect {
    Nothing,      // 아무 일 없음
    Curse,        // 저주
    Bless,        // 축복
    Rust,         // 녹
    Dilute,       // 희석 (포션)
    Erode,        // 부식
    ExcalSuccess, // 엑스칼리버 생성 성공
    ExcalFail,    // 엑스칼리버 실패 (악마 소환)
}

/// 분수에 아이템 담그기 (원본: dipfountain)
pub fn dip_in_fountain(
    item_name: &str,
    item_is_long_sword: bool,
    player_is_lawful: bool,
    player_level: i32,
    rng: &mut NetHackRng,
) -> DipEffect {
    //
    if item_is_long_sword && player_is_lawful && player_level >= 5 {
        let excal_chance = rng.rn2(10);
        if excal_chance < 3 {
            return DipEffect::ExcalSuccess;
        } else if excal_chance < 5 {
            return DipEffect::ExcalFail; // 물 악마 소환
        }
    }

    let roll = rng.rn2(30);
    match roll {
        0..=4 => DipEffect::Nothing,
        5..=9 => DipEffect::Curse,
        10..=14 => DipEffect::Bless,
        15..=19 => {
            //
            if item_name.contains("sword")
                || item_name.contains("mace")
                || item_name.contains("armor")
                || item_name.contains("shield")
            {
                DipEffect::Rust
            } else {
                DipEffect::Nothing
            }
        }
        20..=24 => DipEffect::Dilute,
        _ => DipEffect::Erode,
    }
}

/// 담그기 결과 메시지 (원본: fountain.c messages)
pub fn dip_effect_message(effect: &DipEffect) -> &'static str {
    match effect {
        DipEffect::Nothing => "The water glistens for a moment.",
        DipEffect::Curse => "The water turns black. A curse falls upon your item!",
        DipEffect::Bless => "The water glows briefly. Your item feels blessed!",
        DipEffect::Rust => "The water causes rust! Your item corrodes!",
        DipEffect::Dilute => "The potion is diluted by the water.",
        DipEffect::Erode => "The water eats away at your item!",
        DipEffect::ExcalSuccess => "A brilliant light erupts from the water! Excalibur!",
        DipEffect::ExcalFail => "A water demon appears!",
    }
}

// =============================================================================
// [v2.3.3] 분수 씻기 (원본 fountain.c: washfountain)
// =============================================================================

/// 씻기 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WashEffect {
    Clean,       // 장갑/손 씻기
    RemoveBlind, // 눈 씻어 실명 해제
    RemoveSlime, // 슬라임 제거
    Nothing,     // 아무것도 아님
    WaterGush,   // 물줄기 맞음
}

///
pub fn wash_at_fountain(is_blinded: bool, is_slimed: bool, rng: &mut NetHackRng) -> WashEffect {
    let roll = rng.rn2(20);

    if is_slimed && roll < 5 {
        return WashEffect::RemoveSlime; // 25%: 슬라임 제거
    }

    if is_blinded && roll < 8 {
        return WashEffect::RemoveBlind; // 40%: 실명 해제
    }

    match roll {
        0..=5 => WashEffect::Clean,
        6..=10 => WashEffect::Nothing,
        _ => WashEffect::WaterGush,
    }
}

/// 씻기 메시지 (원본: fountain.c wash messages)
pub fn wash_effect_message(effect: &WashEffect) -> &'static str {
    match effect {
        WashEffect::Clean => "You wash your hands in the cool water.",
        WashEffect::RemoveBlind => "Your eyes tingle! You can see again!",
        WashEffect::RemoveSlime => "The slime dissolves in the water!",
        WashEffect::Nothing => "You splash some water around.",
        WashEffect::WaterGush => "A gush of water hits you in the face!",
    }
}

// =============================================================================
// [v2.3.3] 분수 통계
// =============================================================================

/// 분수 통계
#[derive(Debug, Clone, Default)]
pub struct FountainStatistics {
    pub drinks: u32,
    pub dips: u32,
    pub washes: u32,
    pub wishes_granted: u32,
    pub demons_summoned: u32,
    pub fountains_dried: u32,
    pub excaliburs_created: u32,
}

impl FountainStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_drink(&mut self) {
        self.drinks += 1;
    }
    pub fn record_dip(&mut self) {
        self.dips += 1;
    }
    pub fn record_wash(&mut self) {
        self.washes += 1;
    }
    pub fn record_wish(&mut self) {
        self.wishes_granted += 1;
    }
    pub fn record_demon(&mut self) {
        self.demons_summoned += 1;
    }
    pub fn record_dry(&mut self) {
        self.fountains_dried += 1;
    }
    pub fn record_excalibur(&mut self) {
        self.excaliburs_created += 1;
    }
}

// =============================================================================
// [v2.3.3] 테스트
// =============================================================================
#[cfg(test)]
mod fountain_extended_tests {
    use super::*;

    #[test]
    fn test_fountain_wish_roll() {
        let mut rng = NetHackRng::new(42);
        //
        let mut no_wish = 0;
        for _ in 0..100 {
            if fountain_wish_roll(0, &mut rng) == FountainWishResult::NoWish {
                no_wish += 1;
            }
        }
        assert!(no_wish > 70); // 70% 이상 NoWish여야 함
    }

    #[test]
    fn test_fountain_dryup() {
        let mut rng = NetHackRng::new(42);
        //
        let mut dried = 0;
        for _ in 0..100 {
            if fountain_dryup_chance(20, &mut rng) {
                dried += 1;
            }
        }
        assert!(dried > 30);
    }

    #[test]
    fn test_dip_excalibur() {
        let mut rng = NetHackRng::new(42);
        let mut excal = false;
        for _ in 0..100 {
            let r = dip_in_fountain("long sword", true, true, 10, &mut rng);
            if r == DipEffect::ExcalSuccess {
                excal = true;
                break;
            }
        }
        assert!(excal);
    }

    #[test]
    fn test_dip_effect_message() {
        assert!(dip_effect_message(&DipEffect::ExcalSuccess).contains("Excalibur"));
        assert!(dip_effect_message(&DipEffect::Curse).contains("curse"));
    }

    #[test]
    fn test_wash() {
        let mut rng = NetHackRng::new(42);
        let r = wash_at_fountain(false, false, &mut rng);
        // 결과는 Clean, Nothing, WaterGush 중 하나
        assert!(matches!(
            r,
            WashEffect::Clean | WashEffect::Nothing | WashEffect::WaterGush
        ));
    }

    #[test]
    fn test_fountain_stats() {
        let mut stats = FountainStatistics::new();
        stats.record_drink();
        stats.record_dip();
        stats.record_wish();
        assert_eq!(stats.drinks, 1);
        assert_eq!(stats.dips, 1);
        assert_eq!(stats.wishes_granted, 1);
    }
}

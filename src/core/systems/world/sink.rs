// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::dungeon::Grid;
use crate::core::entity::{Health, Item, PlayerTag, Position};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;

/// 싱크대 발차기 (Kick Sink)
/// 원본 NetHack sink.c: dosinkring() 등 참조
pub fn try_kick_sink(
    world: &mut World,
    grid: &Grid,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
    pos: (i32, i32), // 싱크대 위치
) {
    if let Some(tile) = grid.get_tile(pos.0 as usize, pos.1 as usize) {
        if tile.typ != crate::core::dungeon::tile::TileType::Sink {
            return;
        }
    } else {
        return;
    }

    log.add("Klunk! You kick the sink.", turn);

    // 1/3 확률로 효과 발생
    let roll = rng.rn2(3);
    match roll {
        0 => {
            // 반지 획득
            log.add("You hear a tinkling sound.", turn);
            // 랜덤 반지 생성
            crate::core::entity::spawn::Spawner::mkobj_of_class(
                crate::core::entity::object::ItemClass::Ring,
                &crate::core::entity::object::ItemManager::new(), // TODO: Need actual ItemManager from assets
                world,
                rng,
            )
            .map(|e| {
                if let Some(mut entry) = world.entry(e) {
                    //
                    //
                    entry.add_component(Position { x: pos.0, y: pos.1 });
                    log.add("A ring pops out!", turn);
                }
            });
        }
        1 => {
            // 블랙 푸딩 소환
            log.add("The pipes gurgle...", turn);
            // Black Pudding ('P') 소환
            // TODO: Need MonsterTemplates and ItemManager
            // For now, just a message.
            log.add_colored("A black goo oozes from the drain!", [100, 100, 100], turn);
        }
        2 => {
            // 다침 (Dex check needed)
            log.add("Ouch! That hurts your foot.", turn);
            // 데미지
            let mut p_query = <(&mut Health, &Position)>::query().filter(component::<PlayerTag>());
            for (h, _) in p_query.iter_mut(world) {
                h.current -= rng.rn1(3, 1); // 1d3 damage
            }
        }
        _ => {}
    }
}

///
pub fn try_drop_into_sink(
    item_ent: Entity,
    world: &mut SubWorld,
    log: &mut GameLog,
    turn: u64,
) -> bool {
    // 아이템 확인
    let mut is_ring = false;
    let mut item_name = String::new();

    if let Ok(entry) = world.entry_ref(item_ent) {
        if let Ok(item) = entry.get_component::<Item>() {
            //
            //
            //
            if item.kind.as_str().contains("ring") {
                is_ring = true;
                item_name = item.kind.to_string();
            }
        }
    }

    if is_ring {
        log.add(format!("You drop {} down the drain.", item_name), turn);
        log.add("You hear the ring bouncing down the pipes...", turn);
        // 아이템 삭제
        //
        true // 드랍 처리 완료 (사라짐)
    } else {
        log.add("You restart the dishwasher.", turn); // NetHack humor
        false // 일반 드랍 처리 (바닥에 남음)
    }
}

// =============================================================================
// [v2.3.3] 싱크대 시스템 확장 (원본 sink.c)
// =============================================================================

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrinkSinkEffect {
    Nothing,         // 아무 일 없음
    HotWater,        // 뜨거운 물 (데미지)
    SewageWater,     // 하수도 물 (메스꺼움)
    RefreshingWater, // 시원한 물 (HP 회복)
    PoisonedWater,   // 독물
    MagicWater,      // 마법 물 (능력치 변동)
    WormAppears,     // 지렁이 출현
    Hallucinate,     // 환각 시작
}

///
pub fn drink_from_sink(rng: &mut NetHackRng) -> DrinkSinkEffect {
    let roll = rng.rn2(20);
    match roll {
        0..=3 => DrinkSinkEffect::Nothing,
        4..=6 => DrinkSinkEffect::RefreshingWater,
        7..=9 => DrinkSinkEffect::HotWater,
        10..=12 => DrinkSinkEffect::SewageWater,
        13..=14 => DrinkSinkEffect::PoisonedWater,
        15..=16 => DrinkSinkEffect::MagicWater,
        17 => DrinkSinkEffect::WormAppears,
        18 => DrinkSinkEffect::Hallucinate,
        _ => DrinkSinkEffect::Nothing,
    }
}

///
pub fn drink_sink_message(effect: &DrinkSinkEffect) -> &'static str {
    match effect {
        DrinkSinkEffect::Nothing => "You take a sip of water.",
        DrinkSinkEffect::HotWater => "Ouch! That water was scalding!",
        DrinkSinkEffect::SewageWater => "Yuck! That water tasted like sewage!",
        DrinkSinkEffect::RefreshingWater => "That water was very refreshing!",
        DrinkSinkEffect::PoisonedWater => "The water is contaminated!",
        DrinkSinkEffect::MagicWater => "The water glows briefly...",
        DrinkSinkEffect::WormAppears => "A sewer rat swims up the pipe!",
        DrinkSinkEffect::Hallucinate => "The water tastes... psychedelic!",
    }
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkRingEffect {
    Identified, // 효과 식별됨
    Swallowed,  // 반지가 사라짐
    BounceBack, // 반지가 튀어나옴
    PipeClog,   // 파이프 막힘
}

///
pub fn sink_ring_identify(ring_name: &str, rng: &mut NetHackRng) -> SinkRingEffect {
    let roll = rng.rn2(10);

    // 반지 종류에 따른 특수 효과
    if ring_name.contains("fire") {
        return SinkRingEffect::Identified; // 물이 끓음 → 식별
    }
    if ring_name.contains("cold") {
        return SinkRingEffect::Identified; // 물이 얼음 → 식별
    }
    if ring_name.contains("poison") {
        return SinkRingEffect::Identified; // 물이 변색 → 식별
    }

    match roll {
        0..=4 => SinkRingEffect::Swallowed,
        5..=7 => SinkRingEffect::BounceBack,
        8 => SinkRingEffect::PipeClog,
        _ => SinkRingEffect::Identified,
    }
}

/// 반지 식별 메시지
pub fn sink_ring_message(effect: &SinkRingEffect) -> &'static str {
    match effect {
        SinkRingEffect::Identified => "The water reacts! You identify the ring's properties!",
        SinkRingEffect::Swallowed => "The ring slides down the drain. You hear it jangling.",
        SinkRingEffect::BounceBack => "The ring bounces back out of the drain!",
        SinkRingEffect::PipeClog => "You hear the drain clogging up!",
    }
}

/// 싱크대 파괴/사라짐 (원본: breaksink)
pub fn can_destroy_sink(player_str: i32, rng: &mut NetHackRng) -> bool {
    player_str >= 16 && rng.rn2(10) < 3
}

// =============================================================================
// [v2.3.3] 싱크대 통계
// =============================================================================

/// 싱크대 통계
#[derive(Debug, Clone, Default)]
pub struct SinkStatistics {
    pub kicks: u32,
    pub drinks: u32,
    pub rings_dropped: u32,
    pub rings_identified: u32,
    pub rings_lost: u32,
    pub sinks_destroyed: u32,
}

impl SinkStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_kick(&mut self) {
        self.kicks += 1;
    }
    pub fn record_drink(&mut self) {
        self.drinks += 1;
    }
    pub fn record_ring_drop(&mut self, identified: bool) {
        self.rings_dropped += 1;
        if identified {
            self.rings_identified += 1;
        } else {
            self.rings_lost += 1;
        }
    }
}

// =============================================================================
// [v2.3.3] 테스트
// =============================================================================
#[cfg(test)]
mod sink_extended_tests {
    use super::*;

    #[test]
    fn test_drink_sink() {
        let mut rng = NetHackRng::new(42);
        // 여러 번 마셔서 다양한 효과 확인
        let mut effects = std::collections::HashSet::new();
        for _ in 0..50 {
            let e = drink_from_sink(&mut rng);
            effects.insert(format!("{:?}", e));
        }
        assert!(effects.len() >= 3); // 최소 3가지 효과
    }

    #[test]
    fn test_drink_message() {
        assert!(drink_sink_message(&DrinkSinkEffect::HotWater).contains("scalding"));
    }

    #[test]
    fn test_sink_ring_fire() {
        let mut rng = NetHackRng::new(42);
        let r = sink_ring_identify("ring of fire resistance", &mut rng);
        assert_eq!(r, SinkRingEffect::Identified);
    }

    #[test]
    fn test_sink_ring_message() {
        assert!(sink_ring_message(&SinkRingEffect::Swallowed).contains("drain"));
    }

    #[test]
    fn test_sink_stats() {
        let mut stats = SinkStatistics::new();
        stats.record_kick();
        stats.record_drink();
        stats.record_ring_drop(true);
        assert_eq!(stats.kicks, 1);
        assert_eq!(stats.drinks, 1);
        assert_eq!(stats.rings_identified, 1);
    }
}

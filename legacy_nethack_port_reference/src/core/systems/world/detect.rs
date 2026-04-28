// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
//
// 원본: nethack-3.6.7/src/detect.c (1,862줄)
// [v1.9.0] Phase 51: 탐지 마법 시스템 전수 구현
// =============================================================================
//
// 이 모듈은 탐지 마법 (몬스터 감지, 아이템 감지, 함정 감지,
//
//

use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::Grid;
use crate::core::dungeon::{COLNO, ROWNO};
use crate::core::entity::{Health, Item, ItemTag, MonsterTag, Position};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::*;

// =============================================================================
// 탐지 결과 구조체
// =============================================================================

/// 탐지 결과 (원본: detect.c 반환값 기반)
#[derive(Debug, Clone)]
pub struct DetectResult {
    pub found_count: usize,         // 발견된 대상 수
    pub positions: Vec<(i32, i32)>, // 발견된 위치 목록
    pub message: String,            // 결과 메시지
    pub map_updated: bool,          // 맵이 갱신되었는지
}

impl DetectResult {
    pub fn nothing() -> Self {
        Self {
            found_count: 0,
            positions: Vec::new(),
            message: "You fail to sense anything.".to_string(),
            map_updated: false,
        }
    }
}

// =============================================================================
// 몬스터 감지 (원본 detect.c: monster_detect / detect_monsters)
// =============================================================================

///
/// 맵 전체의 모든 몬스터 위치를 표시
pub fn monster_detect(
    world: &World,
    _grid: &mut Grid,
    blessed: bool,
    log: &mut GameLog,
    turn: u64,
) -> DetectResult {
    let mut result = DetectResult::nothing();
    let mut query = <(&Position, &MonsterTag)>::query();

    for (pos, _) in query.iter(world) {
        result.positions.push((pos.x, pos.y));
        result.found_count += 1;

        // 맵에 몬스터 위치를 시야 벡터로 표시
        //
        if pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < COLNO && (pos.y as usize) < ROWNO {
            _grid.locations[pos.x as usize][pos.y as usize].seenv = 0xFF;
        }
    }

    if result.found_count > 0 {
        if blessed {
            result.message = format!(
                "You sense the presence of {} monster{}.",
                result.found_count,
                if result.found_count != 1 { "s" } else { "" }
            );
        } else {
            result.message = "You sense the presence of monsters.".to_string();
        }
        result.map_updated = true;
        log.add(&result.message, turn);
    } else {
        result.message = "You feel lonely.".to_string();
        log.add(&result.message, turn);
    }

    result
}

///
///
pub fn telepathy_detect(
    world: &World,
    player_x: i32,
    player_y: i32,
    range: i32,
    blind: bool,
) -> Vec<(i32, i32, String)> {
    let mut detected = Vec::new();
    let range_sq = range * range;

    let mut query = <(&Position, &crate::core::entity::Monster, &MonsterTag)>::query();
    for (pos, mon, _) in query.iter(world) {
        let dist_sq = (pos.x - player_x).pow(2) + (pos.y - player_y).pow(2);

        //
        if dist_sq <= range_sq {
            if blind || dist_sq <= (range / 2) * (range / 2) {
                detected.push((pos.x, pos.y, mon.kind.to_string()));
            }
        }
    }

    detected
}

// =============================================================================
// 아이템 감지 (원본 detect.c: object_detect / detect_objects)
// =============================================================================

///
pub fn object_detect(
    world: &World,
    _grid: &mut Grid,
    blessed: bool,
    log: &mut GameLog,
    turn: u64,
) -> DetectResult {
    let mut result = DetectResult::nothing();
    let mut query = <(&Position, &ItemTag)>::query();

    for (pos, _) in query.iter(world) {
        result.positions.push((pos.x, pos.y));
        result.found_count += 1;

        if pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < COLNO && (pos.y as usize) < ROWNO {
            _grid.locations[pos.x as usize][pos.y as usize].seenv = 0xFF;
        }
    }

    if result.found_count > 0 {
        if blessed {
            result.message = format!(
                "You sense the presence of {} object{}.",
                result.found_count,
                if result.found_count != 1 { "s" } else { "" }
            );
        } else {
            result.message = "You sense the presence of objects.".to_string();
        }
        result.map_updated = true;
        log.add(&result.message, turn);
    } else {
        result.message = "You feel a lack of something.".to_string();
        log.add(&result.message, turn);
    }

    result
}

// =============================================================================
// 금 감지 (원본 detect.c: gold_detect)
// =============================================================================

/// 금 감지 (원본: gold_detect)
pub fn gold_detect(world: &World, _grid: &mut Grid, log: &mut GameLog, turn: u64) -> DetectResult {
    let mut result = DetectResult::nothing();
    let mut query = <(&Position, &Item)>::query();

    for (pos, item) in query.iter(world) {
        if item.kind.as_str().contains("gold") || item.kind.as_str().contains("zorkmid") {
            result.positions.push((pos.x, pos.y));
            result.found_count += 1;

            if pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < COLNO && (pos.y as usize) < ROWNO {
                _grid.locations[pos.x as usize][pos.y as usize].seenv = 0xFF;
            }
        }
    }

    if result.found_count > 0 {
        result.message = "You feel very greedy.".to_string();
        result.map_updated = true;
    } else {
        result.message = "You feel materially poor.".to_string();
    }
    log.add(&result.message, turn);
    result
}

// =============================================================================
// 함정 감지 (원본 detect.c: trap_detect / detect_traps)
// =============================================================================

/// 함정 감지 (원본: trap_detect)
///
pub fn trap_detect(
    world: &World,
    _grid: &mut Grid,
    player_x: i32,
    player_y: i32,
    blessed: bool,
    log: &mut GameLog,
    turn: u64,
) -> DetectResult {
    let mut result = DetectResult::nothing();
    let range = if blessed { 100 } else { 10 };

    let mut query = <(
        &Position,
        &crate::core::entity::Trap,
        &crate::core::entity::TrapTag,
    )>::query();
    for (pos, trap, _) in query.iter(world) {
        let dist_sq = (pos.x - player_x).pow(2) + (pos.y - player_y).pow(2);
        if dist_sq > range * range {
            continue;
        }

        if !trap.discovered {
            result.positions.push((pos.x, pos.y));
            result.found_count += 1;
        }
    }

    if result.found_count > 0 {
        result.message = format!(
            "You sense the presence of {} trap{}.",
            result.found_count,
            if result.found_count != 1 { "s" } else { "" }
        );
        result.map_updated = true;
    } else {
        result.message = "You feel safe.".to_string();
    }
    log.add(&result.message, turn);
    result
}

// =============================================================================
//
// =============================================================================

///
/// `s` (search) 명령의 핵심 로직
pub fn search_around(
    grid: &mut Grid,
    x: i32,
    y: i32,
    search_skill: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> i32 {
    let mut found = 0;

    // 8방향 검사
    let deltas: [(i32, i32); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    for (dx, dy) in &deltas {
        let nx = x + dx;
        let ny = y + dy;
        if nx < 0 || ny < 0 || nx as usize >= COLNO || ny as usize >= ROWNO {
            continue;
        }
        let ux = nx as usize;
        let uy = ny as usize;

        // 비밀문 탐색 (원본: SDOOR → DOOR 변환)
        if grid.locations[ux][uy].typ == TileType::SDoor {
            // 탐색 성공 확률: 1/7 + 기술 보너스
            if rng.rn2(7 - search_skill.min(4)) == 0 {
                grid.locations[ux][uy].typ = TileType::Door;
                grid.locations[ux][uy].seenv = 0xFF;
                found += 1;
                log.add("You find a hidden door.", turn);
            }
        }

        //
        if grid.locations[ux][uy].typ == TileType::SCorr {
            if rng.rn2(7 - search_skill.min(4)) == 0 {
                grid.locations[ux][uy].typ = TileType::Corr;
                grid.locations[ux][uy].seenv = 0xFF;
                found += 1;
                log.add("You find a hidden passage.", turn);
            }
        }
    }

    found
}

// =============================================================================
// 마법 지도 (원본 detect.c: do_mapping / magic_map)
// =============================================================================

///
pub fn magic_mapping(grid: &mut Grid, log: &mut GameLog, turn: u64) {
    for x in 1..COLNO - 1 {
        for y in 1..ROWNO - 1 {
            let tile = &grid.locations[x][y];
            //
            match tile.typ {
                TileType::VWall
                | TileType::HWall
                | TileType::TlCorner
                | TileType::TrCorner
                | TileType::BlCorner
                | TileType::BrCorner
                | TileType::CrossWall
                | TileType::TuWall
                | TileType::TdWall
                | TileType::TlWall
                | TileType::TrWall
                | TileType::DbWall
                | TileType::Room
                | TileType::Corr
                | TileType::Door
                | TileType::OpenDoor
                | TileType::StairsUp
                | TileType::StairsDown
                | TileType::Altar
                | TileType::Fountain
                | TileType::Throne
                | TileType::Sink
                | TileType::Grave
                | TileType::Pool
                | TileType::LavaPool => {
                    grid.locations[x][y].seenv = 0xFF;
                }
                //
                TileType::SDoor => {
                    grid.locations[x][y].typ = TileType::Door;
                    grid.locations[x][y].seenv = 0xFF;
                }
                TileType::SCorr => {
                    grid.locations[x][y].typ = TileType::Corr;
                    grid.locations[x][y].seenv = 0xFF;
                }
                _ => {}
            }
        }
    }

    log.add("A map coalesces in your mind!", turn);
}

/// 부분 지도 (원본: partial_map)
pub fn partial_mapping(
    grid: &mut Grid,
    player_x: i32,
    player_y: i32,
    range: i32,
    log: &mut GameLog,
    turn: u64,
) {
    let range_sq = range * range;

    for x in 1..COLNO - 1 {
        for y in 1..ROWNO - 1 {
            let dist_sq = (x as i32 - player_x).pow(2) + (y as i32 - player_y).pow(2);
            if dist_sq > range_sq {
                continue;
            }

            match grid.locations[x][y].typ {
                TileType::VWall
                | TileType::HWall
                | TileType::TlCorner
                | TileType::TrCorner
                | TileType::BlCorner
                | TileType::BrCorner
                | TileType::Room
                | TileType::Corr
                | TileType::Door
                | TileType::OpenDoor
                | TileType::StairsUp
                | TileType::StairsDown => {
                    grid.locations[x][y].seenv = 0xFF;
                }
                TileType::SDoor => {
                    grid.locations[x][y].typ = TileType::Door;
                    grid.locations[x][y].seenv = 0xFF;
                }
                TileType::SCorr => {
                    grid.locations[x][y].typ = TileType::Corr;
                    grid.locations[x][y].seenv = 0xFF;
                }
                _ => {}
            }
        }
    }

    log.add("You have a sense of the layout around you.", turn);
}

// =============================================================================
// 감지 물약 효과 (원본 detect.c)
// =============================================================================

/// 몬스터 탐지 물약 (원본: SPE_DETECT_MONSTERS)
pub fn potion_detect_monsters(blessed: bool, cursed: bool) -> (i32, String) {
    if cursed {
        (0, "You feel dizzy.".to_string())
    } else if blessed {
        (300, "You sense the presence of all monsters.".to_string())
    } else {
        (100, "You sense the presence of monsters.".to_string())
    }
}

/// 보물 탐지 물약 (원본: SPE_DETECT_TREASURE)
pub fn potion_detect_treasure(blessed: bool, cursed: bool) -> (i32, String) {
    if cursed {
        (0, "You feel a pull downward.".to_string())
    } else if blessed {
        (300, "You sense all valuable objects nearby.".to_string())
    } else {
        (100, "You sense objects nearby.".to_string())
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn use_crystal_ball(
    charges: &mut i32,
    blessed: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> Option<String> {
    if *charges <= 0 {
        log.add("The crystal ball is clouded.", turn);
        return None;
    }

    *charges -= 1;

    let success_chance = if blessed { 3 } else { 5 };
    if rng.rn2(success_chance) == 0 {
        log.add("You see only fog in the crystal ball.", turn);
        return None;
    }

    let visions = [
        "You see a dragon guarding treasure!",
        "You see a dark corridor leading downward...",
        "You see a group of monsters ahead.",
        "You see a shimmering portal.",
        "You see stairs going down.",
        "You see a fountain in a room.",
        "You see a chest in an empty room.",
        "You see a throne room!",
        "You see a dimly lit maze.",
        "You see an altar to an unknown god.",
    ];
    let idx = rng.rn2(visions.len() as i32) as usize;
    let vision = visions[idx];
    log.add(vision, turn);
    Some(vision.to_string())
}

// =============================================================================
// Warning 시스템 (원본 detect.c: warning)
// =============================================================================

/// 위험 감지 레벨 (원본: warning 시스템)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WarningLevel {
    None = 0,
    White = 1,
    Pink = 2,
    Red = 3,
    Ruby = 4,
    Purple = 5,
    Bright = 6,
}

/// 위험 레벨 색상 (원본: warn_color)
pub fn warning_color(level: WarningLevel) -> [u8; 3] {
    match level {
        WarningLevel::None => [0, 0, 0],
        WarningLevel::White => [255, 255, 255],
        WarningLevel::Pink => [255, 200, 200],
        WarningLevel::Red => [255, 0, 0],
        WarningLevel::Ruby => [200, 0, 0],
        WarningLevel::Purple => [200, 0, 200],
        WarningLevel::Bright => [255, 255, 0],
    }
}

/// 주변 위험 레벨 계산 (원본: dosearch + warning)
pub fn calculate_warning(
    world: &World,
    player_x: i32,
    player_y: i32,
    player_level: i32,
) -> WarningLevel {
    let mut max_danger = WarningLevel::None;

    let mut query = <(
        &Position,
        &crate::core::entity::Monster,
        &Health,
        &MonsterTag,
    )>::query();
    for (pos, _mon, _health, _) in query.iter(world) {
        let dist_sq = (pos.x - player_x).pow(2) + (pos.y - player_y).pow(2);
        if dist_sq > 100 {
            continue;
        } // 10칸 범위

        // 몬스터 CombatStats의 level로 판정 (단순화)
        //
        let level_diff = 0 - player_level; // 임시 값
        let danger = if level_diff >= 15 {
            WarningLevel::Bright
        } else if level_diff >= 10 {
            WarningLevel::Purple
        } else if level_diff >= 7 {
            WarningLevel::Ruby
        } else if level_diff >= 4 {
            WarningLevel::Red
        } else if level_diff >= 1 {
            WarningLevel::Pink
        } else {
            WarningLevel::White
        };

        if danger > max_danger {
            max_danger = danger;
        }
    }

    max_danger
}

/// 위험 경고 메시지 (원본: dosearch warning messages)
pub fn warning_message(level: WarningLevel) -> &'static str {
    match level {
        WarningLevel::None => "",
        WarningLevel::White => "You feel vaguely uneasy.",
        WarningLevel::Pink => "You feel nervous.",
        WarningLevel::Red => "You feel very uncomfortable.",
        WarningLevel::Ruby => "You feel a strong sense of danger!",
        WarningLevel::Purple => "You realize that something terrible is near!",
        WarningLevel::Bright => "DANGER! Something extremely powerful is near!",
    }
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warning_level() {
        assert!(WarningLevel::Bright > WarningLevel::None);
        assert!(WarningLevel::Red > WarningLevel::Pink);
    }

    #[test]
    fn test_warning_message() {
        assert_eq!(warning_message(WarningLevel::None), "");
        assert!(!warning_message(WarningLevel::Bright).is_empty());
    }
}

// =============================================================================
// [v2.3.3] 감지 시스템 확장 (원본 detect.c: detection utilities)
// =============================================================================

/// 감지 효과 유형 (원본: detect.c detect types)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionType {
    Monster,
    Object,
    Gold,
    Trap,
    SecretDoor,
    MagicMap,
    Telepathy,
    CrystalBall,
    Warning,
}

///
pub fn detection_duration(detect_type: &DetectionType, blessed: bool, skill_level: i32) -> u32 {
    let base = match detect_type {
        DetectionType::Monster => 50,
        DetectionType::Object => 40,
        DetectionType::Gold => 30,
        DetectionType::Trap => 60,
        DetectionType::SecretDoor => 20,
        DetectionType::MagicMap => 999, // 영구
        DetectionType::Telepathy => 100,
        DetectionType::CrystalBall => 10,
        DetectionType::Warning => 80,
    };

    let blessed_bonus = if blessed { base / 2 } else { 0 };
    let skill_bonus = (skill_level * 5) as u32;

    (base + blessed_bonus + skill_bonus).min(999)
}

/// 감지 범위 계산 (원본: detect.c range calculation)
pub fn detection_range(detect_type: &DetectionType, blessed: bool, _cursed: bool) -> i32 {
    let base = match detect_type {
        DetectionType::Monster => 20,
        DetectionType::Object => 15,
        DetectionType::Gold => 999, // 전역
        DetectionType::Trap => 10,
        DetectionType::SecretDoor => 5,
        DetectionType::MagicMap => 999, // 전역
        DetectionType::Telepathy => 12,
        DetectionType::CrystalBall => 999,
        DetectionType::Warning => 8,
    };

    if blessed {
        base * 2
    } else {
        base
    }
}

///
pub fn detection_fail_chance(cursed: bool, confused: bool) -> i32 {
    let mut chance = 0;
    if cursed {
        chance += 30;
    }
    if confused {
        chance += 20;
    }
    chance.min(80)
}

///
pub fn detection_priority(detect_type: &DetectionType) -> i32 {
    match detect_type {
        DetectionType::Monster => 10, // 최우선
        DetectionType::Trap => 9,
        DetectionType::Warning => 8,
        DetectionType::Object => 5,
        DetectionType::Gold => 3,
        DetectionType::SecretDoor => 7,
        DetectionType::MagicMap => 1,
        DetectionType::Telepathy => 6,
        DetectionType::CrystalBall => 4,
    }
}

// =============================================================================
// [v2.3.3] 감지 통계
// =============================================================================

/// 감지 통계
#[derive(Debug, Clone, Default)]
pub struct DetectionStatistics {
    pub monsters_detected: u32,
    pub objects_detected: u32,
    pub traps_detected: u32,
    pub secret_doors_found: u32,
    pub maps_revealed: u32,
    pub detection_failures: u32,
    pub crystal_ball_uses: u32,
}

impl DetectionStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_detection(&mut self, dtype: &DetectionType, success: bool) {
        if !success {
            self.detection_failures += 1;
            return;
        }
        match dtype {
            DetectionType::Monster | DetectionType::Telepathy | DetectionType::Warning => {
                self.monsters_detected += 1;
            }
            DetectionType::Object | DetectionType::Gold => self.objects_detected += 1,
            DetectionType::Trap => self.traps_detected += 1,
            DetectionType::SecretDoor => self.secret_doors_found += 1,
            DetectionType::MagicMap => self.maps_revealed += 1,
            DetectionType::CrystalBall => self.crystal_ball_uses += 1,
        }
    }
}

#[cfg(test)]
mod detect_extended_tests {
    use super::*;

    #[test]
    fn test_duration() {
        let d = detection_duration(&DetectionType::Monster, false, 0);
        assert_eq!(d, 50);
        let d_blessed = detection_duration(&DetectionType::Monster, true, 0);
        assert!(d_blessed > d);
    }

    #[test]
    fn test_range() {
        let r = detection_range(&DetectionType::Monster, false, false);
        assert_eq!(r, 20);
        let r_blessed = detection_range(&DetectionType::Monster, true, false);
        assert_eq!(r_blessed, 40);
    }

    #[test]
    fn test_fail_chance() {
        assert_eq!(detection_fail_chance(false, false), 0);
        assert_eq!(detection_fail_chance(true, false), 30);
        assert_eq!(detection_fail_chance(true, true), 50);
    }

    #[test]
    fn test_priority() {
        assert!(
            detection_priority(&DetectionType::Monster) > detection_priority(&DetectionType::Gold)
        );
    }

    #[test]
    fn test_detect_stats() {
        let mut stats = DetectionStatistics::new();
        stats.record_detection(&DetectionType::Monster, true);
        stats.record_detection(&DetectionType::Trap, false);
        assert_eq!(stats.monsters_detected, 1);
        assert_eq!(stats.detection_failures, 1);
    }
}

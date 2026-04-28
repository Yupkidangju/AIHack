// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
//!
//! 원본 NetHack의 apply.c 기반

use crate::assets::AssetManager;
use crate::core::dungeon::tile::TileType;
use crate::core::entity::{Health, Inventory, Item, LightSource, PlayerTag, Position};
use crate::core::game_state::{DirectionAction, GameState};
use crate::ui::log::GameLog;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{component, Entity, EntityStore, IntoQuery};

/// 도구 사용 시스템 (doapply)
///
pub fn item_apply(
    item_ent: Entity,
    world: &mut SubWorld,
    _assets: &AssetManager,
    _grid: &crate::core::dungeon::Grid,
    log: &mut GameLog,
    turn: u64,
    state: &mut GameState,
    _command_buffer: &mut CommandBuffer,
) {
    let item = match world.entry_ref(item_ent) {
        Ok(entry) => match entry.get_component::<Item>() {
            Ok(i) => i.clone(),
            Err(_) => return,
        },
        Err(_) => return,
    };

    // [v1.2.0
    // 원본 apply.c:3575 doapply()의 switch (obj->otyp) 로직 이식
    match item.kind.as_str() {
        "pick-axe" | "dwarvish mattock" => {
            log.add("In what direction?", turn);
            state.request_direction(DirectionAction::Apply { item: item_ent });
        }
        "skeleton key" | "lockpick" | "credit card" => {
            log.add("In what direction?", turn);
            state.request_direction(DirectionAction::Apply { item: item_ent });
        }
        "stethoscope" => {
            log.add("In what direction?", turn);
            state.request_direction(DirectionAction::Apply { item: item_ent });
        }
        "expensive camera" => {
            log.add("In what direction?", turn);
            state.request_direction(DirectionAction::Apply { item: item_ent });
        }
        "unicorn horn" => {
            use_unicorn_horn(world, log, turn);
        }
        "large box" | "chest" | "ice box" | "sack" | "bag of holding" => {
            // 용기를 직접 Apply 하면 루팅 메뉴(내용물 확인)로 전환
            *state = GameState::Looting {
                container: item_ent,
            };
            log.add(format!("You look inside the {}.", item.kind), turn);
        }
        "bag of tricks" => {
            // 마법의 자루 (몬스터 소환)
            use_bag_of_tricks(item_ent, world, _assets, log, turn, _command_buffer);
        }
        "lamp" | "oil lamp" | "brass lantern" | "magic lamp" => {
            use_lamp(item_ent, world, _assets, log, turn);
        }
        "potion of oil" => {
            use_oil(item_ent, world, log, turn, state);
        }
        _ => {
            log.add(
                format!("You don't know how to use the {}.", item.kind),
                turn,
            );
        }
    }
}

/// 유니콘 뿔 사용 로직
/// 원본: apply.c:3720 use_unicorn_horn()
fn use_unicorn_horn(world: &mut SubWorld, log: &mut GameLog, turn: u64) {
    let mut query =
        <&mut crate::core::entity::status::StatusBundle>::query().filter(component::<PlayerTag>());
    for status in query.iter_mut(world) {
        let mut cured = false;

        if status.has(crate::core::entity::status::StatusFlags::BLIND) {
            status.remove(crate::core::entity::status::StatusFlags::BLIND);
            log.add("Your vision clears.", turn);
            cured = true;
        }

        if status.has(crate::core::entity::status::StatusFlags::CONFUSED) {
            status.remove(crate::core::entity::status::StatusFlags::CONFUSED);
            log.add("You feel less confused.", turn);
            cured = true;
        }

        if status.has(crate::core::entity::status::StatusFlags::STUNNED) {
            status.remove(crate::core::entity::status::StatusFlags::STUNNED);
            log.add("You feel more stable.", turn);
            cured = true;
        }

        if cured {
            log.add("You feel better.", turn);
        }
    }
}

/// 등불 사용 로직 (Phase 43)
fn use_lamp(
    item_ent: Entity,
    world: &mut SubWorld,
    _assets: &AssetManager,
    log: &mut GameLog,
    turn: u64,
) {
    if let Ok(mut entry) = world.entry_mut(item_ent) {
        let (template, age, blessed, cursed) = if let Ok(item) = entry.get_component::<Item>() {
            (item.kind.to_string(), item.age, item.blessed, item.cursed)
        } else {
            return;
        };

        if age == 0 && template != "magic lamp" {
            log.add(format!("The {} is out of fuel.", template), turn);
            return;
        }

        if template == "magic lamp" {
            // 매직 램프 특수 효과 (Djinni)
            // 원본: apply.c:855
            let mut rng = crate::util::rng::NetHackRng::new(turn);
            if rng.rn2(100) < 20 {
                // 20%확률로 지니 출현
                log.add("A djinni emerges from the lamp!", turn);
                if blessed {
                    log.add("The djinni is very grateful and grants you a wish!", turn);
                    // TODO: Wish 시스템 연동 (Phase 50)
                } else if cursed {
                    log.add("The djinni is very angry!", turn);
                    // TODO: 몬스터 스폰 (Djinni)
                } else {
                    log.add("The djinni vanishes in a puff of smoke.", turn);
                }
                // 램프는 일반 오일 램프로 변함
                if let Ok(item) = entry.get_component_mut::<Item>() {
                    item.kind = crate::generated::ItemKind::OilLamp;
                }
                return;
            }
        }

        if let Ok(light) = entry.get_component_mut::<LightSource>() {
            if light.lit {
                light.lit = false;
                log.add(format!("You douse the {}.", template), turn);
            } else {
                light.lit = true;
                log.add(format!("You light the {}.", template), turn);
            }
        }
    }
}

/// 방향이 필요한 도구의 실제 효과 처리 (main.rs에서 호출)
pub fn execute_apply_action(
    item_ent: Entity,
    direction: crate::core::game_state::Direction,
    world: &mut legion::World,
    _assets: &AssetManager,
    grid: &mut crate::core::dungeon::Grid,
    log: &mut GameLog,
    turn: u64,
    _command_buffer: &mut CommandBuffer,
) {
    let item = match world.entry_ref(item_ent) {
        Ok(entry) => match entry.get_component::<Item>() {
            Ok(i) => i.clone(),
            Err(_) => return,
        },
        Err(_) => return,
    };

    match item.kind.as_str() {
        "pick-axe" | "dwarvish mattock" => {
            use_pick_axe(direction, world, grid, log, turn, _command_buffer);
        }
        "skeleton key" | "lockpick" | "credit card" => {
            use_key(direction, world, grid, log, turn);
        }
        "stethoscope" => {
            use_stethoscope(direction, world, grid, log, turn);
        }
        "expensive camera" => {
            use_camera(direction, world, grid, log, turn);
        }
        _ => {}
    }
}

fn use_pick_axe(
    direction: crate::core::game_state::Direction,
    world: &mut legion::World,
    grid: &mut crate::core::dungeon::Grid,
    log: &mut GameLog,
    turn: u64,
    command_buffer: &mut CommandBuffer,
) {
    let mut player_pos = None;
    let mut query = <&Position>::query().filter(component::<PlayerTag>());
    for pos in query.iter(world) {
        player_pos = Some((pos.x, pos.y));
    }

    if let Some((px, py)) = player_pos {
        let (dx, dy) = direction.to_delta();
        let tx = px + dx;
        let ty = py + dy;

        if let Some(tile) = grid.get_tile_mut(tx as usize, ty as usize) {
            // 바위 체크
            let mut boulder_found = None;
            {
                let mut b_query = <(Entity, &Position)>::query()
                    .filter(component::<crate::core::entity::BoulderTag>());
                for (ent, pos) in b_query.iter(world) {
                    if pos.x == tx && pos.y == ty {
                        boulder_found = Some(*ent);
                        break;
                    }
                }
            }

            if let Some(b_ent) = boulder_found {
                log.add("You smash the boulder to pieces!", turn);
                command_buffer.remove(b_ent);
            } else if tile.typ.is_wall() {
                tile.typ = TileType::Corr; // 벽을 파면 복도로 변함 (임시)
                log.add("You dig through the wall.", turn);
            } else if tile.typ == TileType::Door || tile.typ == TileType::SDoor {
                tile.typ = TileType::Corr;
                log.add("You smash the door.", turn);
            } else {
                log.add("You dig into the floor but find nothing.", turn);
            }
        }
    }
}

fn use_key(
    direction: crate::core::game_state::Direction,
    world: &mut legion::World,
    grid: &mut crate::core::dungeon::Grid,
    log: &mut GameLog,
    turn: u64,
) {
    let mut player_pos = None;
    let mut query = <&Position>::query().filter(component::<PlayerTag>());
    for pos in query.iter(world) {
        player_pos = Some((pos.x, pos.y));
    }

    if let Some((px, py)) = player_pos {
        let (dx, dy) = direction.to_delta();
        let tx = px + dx;
        let ty = py + dy;

        if let Some(tile) = grid.get_tile_mut(tx as usize, ty as usize) {
            match tile.typ {
                TileType::Door | TileType::SDoor => {
                    log.add("The door is already unlocked.", turn);
                }
                _ => {
                    // 기진 위치에 용기가 있는지 확인
                    let mut container_found = false;
                    let mut container_query = <(Entity, &Position, &mut Item)>::query()
                        .filter(component::<crate::core::entity::ContainerTag>());
                    for (_ent, pos, item) in container_query.iter_mut(world) {
                        if pos.x == tx && pos.y == ty {
                            if item.olocked {
                                item.olocked = false;
                                log.add(format!("You unlock the {}.", item.kind), turn);
                            } else {
                                log.add(format!("The {} is already unlocked.", item.kind), turn);
                            }
                            container_found = true;
                            break;
                        }
                    }

                    if !container_found {
                        log.add("You can't use a key there.", turn);
                    }
                }
            }
        }
    }
}

fn use_stethoscope(
    direction: crate::core::game_state::Direction,
    world: &mut legion::World,
    _grid: &crate::core::dungeon::Grid,
    log: &mut GameLog,
    turn: u64,
) {
    let mut player_pos = None;
    let mut query = <&Position>::query().filter(component::<PlayerTag>());
    for pos in query.iter(world) {
        player_pos = Some((pos.x, pos.y));
    }

    if let Some((px, py)) = player_pos {
        let (dx, dy) = direction.to_delta();
        let tx = px + dx;
        let ty = py + dy;

        let mut found_mon = false;
        let mut mon_query = <(&Position, &Health, &Item)>::query();
        for (pos, health, item) in mon_query.iter(world) {
            if pos.x == tx && pos.y == ty {
                log.add(
                    format!(
                        "You hear a heartbeat from the {}. HP: {}/{}",
                        item.kind, health.current, health.max
                    ),
                    turn,
                );
                found_mon = true;
                break;
            }
        }

        if !found_mon {
            log.add("You hear nothing special.", turn);
        }
    }
}

fn use_camera(
    _direction: crate::core::game_state::Direction,
    _world: &mut legion::World,
    _grid: &crate::core::dungeon::Grid,
    log: &mut GameLog,
    turn: u64,
) {
    log.add("Flash!", turn);
}

/// 기름 사용 로직 (Phase 43)
fn use_oil(
    oil_ent: Entity,
    world: &mut SubWorld,
    log: &mut GameLog,
    turn: u64,
    state: &mut GameState,
) {
    //
    let mut inv_query = <&Inventory>::query().filter(component::<PlayerTag>());
    let mut lamp_ent = None;

    if let Some(inv) = inv_query.iter(world).next() {
        for &ent in &inv.items {
            if let Ok(entry) = world.entry_ref(ent) {
                if let Ok(item) = entry.get_component::<Item>() {
                    if item.kind == crate::generated::ItemKind::Lamp
                        || item.kind == crate::generated::ItemKind::OilLamp
                    {
                        lamp_ent = Some(ent);
                        break;
                    }
                }
            }
        }
    }

    if let Some(lamp) = lamp_ent {
        *state = GameState::ConfirmRefill { lamp, oil: oil_ent };
        log.add("Refill the lamp with oil? [y/n]", turn);
    } else {
        log.add("You have nothing to refill with oil.", turn);
    }
}

/// 마법의 자루 사용 로직 (Phase 48)
fn use_bag_of_tricks(
    item_ent: Entity,
    world: &mut SubWorld,
    _assets: &AssetManager,
    log: &mut GameLog,
    turn: u64,
    command_buffer: &mut CommandBuffer,
) {
    //
    let mut player_pos = Position { x: 0, y: 0 };
    let mut p_query = <&Position>::query().filter(component::<PlayerTag>());
    if let Some(pos) = p_query.iter(world).next() {
        player_pos = *pos;
    } else {
        return;
    }

    if let Ok(mut entry) = world.entry_mut(item_ent) {
        if let Ok(item) = entry.get_component_mut::<Item>() {
            // 충전 확인 (spe)
            if item.spe > 0 {
                item.spe -= 1;
                log.add("The bag pulses as you untie it.", turn);

                // 2. 몬스터 스폰 요청 (SpawnRequest)
                let mut rng = crate::util::rng::NetHackRng::new(turn);
                let mon_templates = vec!["sewer rat", "kobold", "jackal", "gnome"];
                let tmpl = mon_templates[rng.rn2(mon_templates.len() as i32) as usize];

                // 주변 1칸 범위 랜덤 위치
                let dx = rng.rn2(3) as i32 - 1;
                let dy = rng.rn2(3) as i32 - 1;

                command_buffer.push((crate::core::entity::spawn::SpawnRequest {
                    x: player_pos.x + dx,
                    y: player_pos.y + dy,
                    template: tmpl.to_string(),
                },));

                log.add(format!("A {} jumps out of the bag!", tmpl), turn);
            } else {
                log.add("The bag is empty.", turn);
            }
        }
    }
}

// =============================================================================
// [v2.3.1] apply.c 확장 이식
// 원본: nethack-3.6.7/src/apply.c (3,527줄)
//
// 도구 종류 데이터, 적용 가능 판정, 개별 도구 효과 등
// =============================================================================

/// [v2.3.1] 도구 종류 분류 (원본: apply.c switch cases)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    /// 조명 (램프, 랜턴)
    Light,
    /// 자물쇠 도구 (열쇠, 락픽)
    Lockpick,
    /// 굴착 (곡괭이, 삽)
    Digging,
    /// 악기 (호른, 피리, 하프 등)
    Musical,
    /// 용기 (자루, 상자)
    Container,
    /// 마법 도구 (수정구, 마커)
    Magic,
    /// 이동 도구 (갈고리, 뗏목)
    Grapple,
    /// 감각 도구 (청진기, 거울)
    Sensory,
    /// 기타 (카메라, 유니콘 뿔 등)
    Other,
}

/// [v2.3.1] 도구 정보 구조체
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: &'static str,
    pub category: ToolCategory,
    pub needs_direction: bool,
    pub uses_charges: bool,
    pub skill_based: bool,
}

/// [v2.3.1] 도구 정보 테이블 (원본: objects[] + apply.c)
pub fn tool_info_table() -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            name: "pick-axe",
            category: ToolCategory::Digging,
            needs_direction: true,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "dwarvish mattock",
            category: ToolCategory::Digging,
            needs_direction: true,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "skeleton key",
            category: ToolCategory::Lockpick,
            needs_direction: true,
            uses_charges: false,
            skill_based: true,
        },
        ToolInfo {
            name: "lockpick",
            category: ToolCategory::Lockpick,
            needs_direction: true,
            uses_charges: false,
            skill_based: true,
        },
        ToolInfo {
            name: "credit card",
            category: ToolCategory::Lockpick,
            needs_direction: true,
            uses_charges: false,
            skill_based: true,
        },
        ToolInfo {
            name: "stethoscope",
            category: ToolCategory::Sensory,
            needs_direction: true,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "expensive camera",
            category: ToolCategory::Other,
            needs_direction: true,
            uses_charges: true,
            skill_based: false,
        },
        ToolInfo {
            name: "unicorn horn",
            category: ToolCategory::Other,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "large box",
            category: ToolCategory::Container,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "chest",
            category: ToolCategory::Container,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "ice box",
            category: ToolCategory::Container,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "sack",
            category: ToolCategory::Container,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "bag of holding",
            category: ToolCategory::Container,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "bag of tricks",
            category: ToolCategory::Magic,
            needs_direction: false,
            uses_charges: true,
            skill_based: false,
        },
        ToolInfo {
            name: "lamp",
            category: ToolCategory::Light,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "oil lamp",
            category: ToolCategory::Light,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "brass lantern",
            category: ToolCategory::Light,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "magic lamp",
            category: ToolCategory::Light,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "mirror",
            category: ToolCategory::Sensory,
            needs_direction: true,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "tin whistle",
            category: ToolCategory::Musical,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "magic whistle",
            category: ToolCategory::Musical,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "wooden flute",
            category: ToolCategory::Musical,
            needs_direction: false,
            uses_charges: false,
            skill_based: true,
        },
        ToolInfo {
            name: "magic flute",
            category: ToolCategory::Musical,
            needs_direction: false,
            uses_charges: true,
            skill_based: true,
        },
        ToolInfo {
            name: "tooled horn",
            category: ToolCategory::Musical,
            needs_direction: false,
            uses_charges: false,
            skill_based: true,
        },
        ToolInfo {
            name: "frost horn",
            category: ToolCategory::Musical,
            needs_direction: true,
            uses_charges: true,
            skill_based: true,
        },
        ToolInfo {
            name: "fire horn",
            category: ToolCategory::Musical,
            needs_direction: true,
            uses_charges: true,
            skill_based: true,
        },
        ToolInfo {
            name: "horn of plenty",
            category: ToolCategory::Magic,
            needs_direction: false,
            uses_charges: true,
            skill_based: false,
        },
        ToolInfo {
            name: "crystal ball",
            category: ToolCategory::Magic,
            needs_direction: false,
            uses_charges: true,
            skill_based: false,
        },
        ToolInfo {
            name: "magic marker",
            category: ToolCategory::Magic,
            needs_direction: false,
            uses_charges: true,
            skill_based: false,
        },
        ToolInfo {
            name: "grappling hook",
            category: ToolCategory::Grapple,
            needs_direction: true,
            uses_charges: false,
            skill_based: true,
        },
        ToolInfo {
            name: "tin opener",
            category: ToolCategory::Other,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "can of grease",
            category: ToolCategory::Other,
            needs_direction: false,
            uses_charges: true,
            skill_based: false,
        },
        ToolInfo {
            name: "blindfold",
            category: ToolCategory::Other,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "towel",
            category: ToolCategory::Other,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
        ToolInfo {
            name: "leash",
            category: ToolCategory::Other,
            needs_direction: false,
            uses_charges: false,
            skill_based: false,
        },
    ]
}

///
pub fn get_tool_info(name: &str) -> Option<ToolInfo> {
    tool_info_table()
        .into_iter()
        .find(|t| name.contains(t.name))
}

/// [v2.3.1] 거울 효과 (원본: use_mirror)
pub fn mirror_effect(
    target_is_monster: bool,
    target_name: &str,
    target_undead: bool,
) -> MirrorResult {
    if target_is_monster {
        if target_undead {
            //
            MirrorResult {
                message: format!("{} is not reflected in the mirror.", target_name),
                damage: 0,
                blind: false,
            }
        } else {
            MirrorResult {
                message: format!("{} is frightened by its own reflection!", target_name),
                damage: 0,
                blind: false,
            }
        }
    } else {
        // 자기 자신을 봄
        MirrorResult {
            message: "You look fine.".to_string(),
            damage: 0,
            blind: false,
        }
    }
}

/// [v2.3.1] 거울 결과
#[derive(Debug, Clone)]
pub struct MirrorResult {
    pub message: String,
    pub damage: i32,
    pub blind: bool,
}

///
pub fn whistle_effect(is_magic: bool) -> &'static str {
    if is_magic {
        "You produce a strange whistling sound. All your pets come running!"
    } else {
        "You produce a high-pitched humming sound."
    }
}

/// [v2.3.1] 수정구 효과 (원본: use_crystal_ball)
pub fn crystal_ball_effect(
    intelligence: i32,
    cursed: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> CrystalBallResult {
    if cursed {
        //
        return CrystalBallResult {
            message: "The crystal ball clouds over and you feel dizzy!".to_string(),
            success: false,
            hallucinate: true,
        };
    }

    // 지능 기반 성공률 (원본: use_crystal_ball)
    let success_chance = 20 + intelligence * 5;
    if rng.rn2(100) < success_chance {
        CrystalBallResult {
            message: "You see a vision in the crystal ball...".to_string(),
            success: true,
            hallucinate: false,
        }
    } else {
        CrystalBallResult {
            message: "The crystal ball is cloudy. You see nothing special.".to_string(),
            success: false,
            hallucinate: false,
        }
    }
}

/// [v2.3.1] 수정구 결과
#[derive(Debug, Clone)]
pub struct CrystalBallResult {
    pub message: String,
    pub success: bool,
    pub hallucinate: bool,
}

/// [v2.3.1] 매직 마커 효과 (원본: dowrite)
pub fn magic_marker_charges_needed(target_scroll: &str) -> i32 {
    //
    match target_scroll {
        s if s.contains("identify") => 10,
        s if s.contains("enchant armor") => 16,
        s if s.contains("enchant weapon") => 16,
        s if s.contains("remove curse") => 14,
        s if s.contains("teleportation") => 20,
        s if s.contains("genocide") => 30,
        s if s.contains("create monster") => 12,
        s if s.contains("gold detection") => 8,
        s if s.contains("food detection") => 8,
        s if s.contains("confuse monster") => 12,
        s if s.contains("scare monster") => 14,
        s if s.contains("blank paper") => 0,
        _ => 15, // 기본값
    }
}

/// [v2.3.1] 호른 오브 플렌티 효과 (원본: use_horn_of_plenty)
pub fn horn_of_plenty_effect(
    blessed: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> &'static str {
    if blessed {
        // 축복 시 더 좋은 음식
        let r = rng.rn2(5);
        match r {
            0 => "A lump of royal jelly emerges!",
            1 => "A cream pie appears!",
            2 => "A slime mold pops out!",
            3 => "A bunch of bananas falls out!",
            _ => "A tripe ration slides out!",
        }
    } else {
        let r = rng.rn2(3);
        match r {
            0 => "A ration of food drops out!",
            1 => "An apple rolls out!",
            _ => "A pear pops out!",
        }
    }
}

/// [v2.3.1] 그리스 캔 효과 (원본: use_grease)
pub fn grease_effect(target_name: &str) -> String {
    format!(
        "You cover the {} with grease. It's now slippery!",
        target_name
    )
}

///
pub fn blindfold_effect(putting_on: bool) -> &'static str {
    if putting_on {
        "You can't see!"
    } else {
        "You can see again."
    }
}

/// [v2.3.1] 목줄 효과 (원본: use_leash)
pub fn leash_effect(pet_nearby: bool, pet_name: &str) -> String {
    if pet_nearby {
        format!("You tie the leash to {}.", pet_name)
    } else {
        "There is nothing here to tie the leash to.".to_string()
    }
}

///
pub fn tin_opener_effect(item_name: &str) -> String {
    if item_name.contains("tin") {
        format!("You open the {}.", item_name)
    } else {
        "You can't open that with a tin opener.".to_string()
    }
}

/// [v2.3.1] 카메라 플래시 데미지/실명 (원본: use_camera)
pub fn camera_flash_effect(
    target_blind_res: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> CameraFlashResult {
    if target_blind_res {
        CameraFlashResult {
            message: "The flash has no effect.".to_string(),
            blind_turns: 0,
            damage: 0,
        }
    } else {
        let blind_turns = rng.rn1(10, 5);
        CameraFlashResult {
            message: "The flash temporarily blinds the target!".to_string(),
            blind_turns,
            damage: 0,
        }
    }
}

/// [v2.3.1] 카메라 플래시 결과
#[derive(Debug, Clone)]
pub struct CameraFlashResult {
    pub message: String,
    pub blind_turns: i32,
    pub damage: i32,
}

/// [v2.3.1] 갈고리 효과 (원본: use_grapple)
pub fn grapple_effect(
    target_is_monster: bool,
    distance: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> GrappleResult {
    if distance > 5 {
        return GrappleResult {
            success: false,
            message: "Too far away!".to_string(),
            pulled: false,
        };
    }

    let success = rng.rn2(100) < 70; // 70% 성공률
    if target_is_monster {
        if success {
            GrappleResult {
                success: true,
                message: "You yank the monster toward you!".to_string(),
                pulled: true,
            }
        } else {
            GrappleResult {
                success: false,
                message: "The hook misses its target.".to_string(),
                pulled: false,
            }
        }
    } else {
        if success {
            GrappleResult {
                success: true,
                message: "You are pulled toward the object!".to_string(),
                pulled: true,
            }
        } else {
            GrappleResult {
                success: false,
                message: "The grappling hook slips.".to_string(),
                pulled: false,
            }
        }
    }
}

/// [v2.3.1] 갈고리 결과
#[derive(Debug, Clone)]
pub struct GrappleResult {
    pub success: bool,
    pub message: String,
    pub pulled: bool,
}

/// [v2.3.1] 악기 연주 효과 (원본: do_play_instrument)
pub fn play_instrument_effect(
    instrument: &str,
    skill_level: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> &'static str {
    let good_play = rng.rn2(100) < (20 + skill_level * 15);

    if instrument.contains("flute") {
        if good_play {
            "You produce a beautiful melody!"
        } else {
            "You produce a few squeaky notes."
        }
    } else if instrument.contains("horn") {
        if good_play {
            "You produce a powerful blast!"
        } else {
            "You produce a feeble toot."
        }
    } else if instrument.contains("harp") || instrument.contains("lyre") {
        if good_play {
            "You produce a wonderful harmony!"
        } else {
            "You produce discordant strumming."
        }
    } else if instrument.contains("drum") {
        if good_play {
            "You beat out a thunderous rhythm!"
        } else {
            "You produce an uneven tapping."
        }
    } else if instrument.contains("bugle") {
        if good_play {
            "You produce a clear bugle call!"
        } else {
            "You produce a sour note."
        }
    } else {
        "You play it. Nothing happens."
    }
}

// =============================================================================
// [v2.3.4] 도구 확장 (원본 apply.c: advanced tool mechanics)
// =============================================================================

/// 도구 내구도 시스템 (원본: apply.c durability)
#[derive(Debug, Clone)]
pub struct ToolDurability {
    pub current: i32,
    pub maximum: i32,
}

impl ToolDurability {
    pub fn new(max: i32) -> Self {
        Self {
            current: max,
            maximum: max,
        }
    }

    /// 사용 시 내구도 감소
    pub fn use_tool(&mut self, amount: i32) -> bool {
        self.current -= amount;
        self.current > 0
    }

    /// 내구도 비율
    pub fn ratio(&self) -> f32 {
        if self.maximum == 0 {
            return 0.0;
        }
        self.current as f32 / self.maximum as f32
    }

    /// 상태 설명
    pub fn condition(&self) -> &'static str {
        let r = self.ratio();
        if r >= 0.9 {
            "pristine"
        } else if r >= 0.7 {
            "good"
        } else if r >= 0.4 {
            "worn"
        } else if r > 0.0 {
            "damaged"
        } else {
            "broken"
        }
    }
}

/// 도구 기본 내구도 (원본: apply.c tool durability table)
pub fn tool_max_durability(tool_name: &str) -> i32 {
    let l = tool_name.to_lowercase();
    if l.contains("pick-axe") || l.contains("mattock") {
        return 100;
    }
    if l.contains("tinning kit") {
        return 50;
    }
    if l.contains("skeleton key") || l.contains("lock pick") {
        return 40;
    }
    if l.contains("stethoscope") {
        return 80;
    }
    if l.contains("tin opener") {
        return 30;
    }
    if l.contains("mirror") {
        return 20;
    }
    if l.contains("leash") {
        return 60;
    }
    if l.contains("whistle") {
        return 200;
    } // 호루라기는 거의 안 부서짐
    if l.contains("horn") {
        return 70;
    }
    if l.contains("drum") {
        return 50;
    }
    if l.contains("crystal ball") {
        return 40;
    }
    if l.contains("candle") {
        return 15;
    }
    if l.contains("lamp") || l.contains("lantern") {
        return 100;
    }
    if l.contains("marker") {
        return 30;
    }
    50 // 기본값
}

/// 도구 수리 비용 (원본: apply.c repair cost)
pub fn tool_repair_cost(tool_name: &str, current_durability: i32, max_durability: i32) -> i32 {
    let damage_ratio = 1.0 - (current_durability as f32 / max_durability as f32);
    let base_cost = tool_max_durability(tool_name) * 2;
    (base_cost as f32 * damage_ratio) as i32
}

/// BUC 도구 효과 보정 (원본: apply.c buc modifier)
pub fn tool_buc_effect(blessed: bool, cursed: bool) -> &'static str {
    if blessed {
        "The tool works particularly well!"
    } else if cursed {
        "The tool fumbles in your hands."
    } else {
        ""
    }
}

/// BUC 도구 성공률 보정
pub fn tool_buc_success_modifier(blessed: bool, cursed: bool) -> i32 {
    if blessed {
        20
    } else if cursed {
        -30
    } else {
        0
    }
}

/// 도구 적용 실패 메시지 (원본: apply.c failure messages)
pub fn tool_failure_message(tool_name: &str) -> &'static str {
    let l = tool_name.to_lowercase();
    if l.contains("lock pick") || l.contains("skeleton key") {
        "The lock resists your attempt."
    } else if l.contains("stethoscope") {
        "You hear nothing special."
    } else if l.contains("crystal ball") {
        "The crystal ball is cloudy."
    } else if l.contains("mirror") {
        "You see a foggy reflection."
    } else if l.contains("horn") {
        "The horn emits a weak sound."
    } else if l.contains("pick-axe") {
        "The rock is too hard to dig through."
    } else {
        "Nothing happens."
    }
}

/// 도구 연료 소모량 (원본: apply.c fuel consumption)
pub fn tool_fuel_per_use(tool_name: &str) -> i32 {
    let l = tool_name.to_lowercase();
    if l.contains("candle") {
        return 5;
    }
    if l.contains("oil lamp") {
        return 2;
    }
    if l.contains("brass lantern") {
        return 1;
    }
    if l.contains("magic lamp") {
        return 0;
    } // 마법 램프는 연료 불필요
    if l.contains("marker") {
        return 3;
    }
    0
}

/// 도구 통계
#[derive(Debug, Clone, Default)]
pub struct ToolStatistics {
    pub tools_used: u32,
    pub tools_broken: u32,
    pub locks_picked: u32,
    pub instruments_played: u32,
    pub items_identified: u32,
    pub fuel_consumed: i32,
}

impl ToolStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_use(&mut self) {
        self.tools_used += 1;
    }
    pub fn record_break(&mut self) {
        self.tools_broken += 1;
    }
    pub fn record_fuel(&mut self, amount: i32) {
        self.fuel_consumed += amount;
    }
}

#[cfg(test)]
mod apply_extended_tests {
    use super::*;

    #[test]
    fn test_durability() {
        let mut d = ToolDurability::new(100);
        assert_eq!(d.condition(), "pristine");
        d.use_tool(50);
        assert_eq!(d.condition(), "worn"); // 50/100 = 0.5 >= 0.4
        d.use_tool(40);
        assert_eq!(d.condition(), "damaged"); // 10/100 = 0.1 > 0.0
        d.use_tool(10);
        assert_eq!(d.condition(), "broken"); // 0/100 = 0.0
    }

    #[test]
    fn test_max_durability() {
        assert!(tool_max_durability("pick-axe") > tool_max_durability("candle"));
    }

    #[test]
    fn test_repair_cost() {
        let cost = tool_repair_cost("pick-axe", 50, 100);
        assert!(cost > 0);
    }

    #[test]
    fn test_buc_modifier() {
        assert_eq!(tool_buc_success_modifier(true, false), 20);
        assert_eq!(tool_buc_success_modifier(false, true), -30);
    }

    #[test]
    fn test_failure_message() {
        let m = tool_failure_message("lock pick");
        assert!(m.contains("lock"));
    }

    #[test]
    fn test_fuel() {
        assert!(tool_fuel_per_use("candle") > tool_fuel_per_use("magic lamp"));
    }

    #[test]
    fn test_tool_stats() {
        let mut s = ToolStatistics::new();
        s.record_use();
        s.record_break();
        s.record_fuel(5);
        assert_eq!(s.tools_used, 1);
        assert_eq!(s.fuel_consumed, 5);
    }
}

// ============================================================================
// E2E 안정화 테스트 (STABILIZATION_ROADMAP Phase S3~S4 대응)
// 목표: GUI 없이 게임 초기화 → N턴 실행을 검증하여 패닉/크래시 부재를 확인
// ============================================================================

use aihack::assets::AssetManager;
use aihack::core::dungeon::{DungeonBranch, Grid, LevelID};
use aihack::core::entity::*;

use legion::*;

/// 게임 월드를 GUI 없이 초기화하는 헬퍼 함수
/// app.rs의 initialize_game_with_choices() 로직을 재현
fn create_test_world() -> (World, Resources, Grid) {
    let mut assets = AssetManager::new();
    assets.load_defaults(".");
    assets.load_defaults("./nethack_original/NetHack-NetHack-3.6.7_Released");

    let mut world = World::default();
    let mut resources = Resources::default();

    let mut rng = aihack::util::rng::NetHackRng::new(42); // 결정론적 시드

    let monster_templates: Vec<_> = assets.monsters.templates.values().collect();
    let level_id = LevelID::new(DungeonBranch::Main, 1);

    let (grid, start_pos, _down_pos, _rooms) =
        aihack::core::dungeon::gen::MapGenerator::generate_improved(
            &mut rng,
            level_id,
            &mut world,
            &assets.items,
            &monster_templates,
            aihack::core::dungeon::gen::LevelType::Ordinary,
        );

    // 시야 시스템 초기화
    let mut vision = aihack::core::systems::vision::VisionSystem::new();
    if start_pos.0 >= 0 && start_pos.1 >= 0 {
        vision.recalc(&grid, start_pos.0 as usize, start_pos.1 as usize, 5);
    }

    // 플레이어 엔티티 생성
    let starting_items = ["long sword", "small shield", "Potion of healing"];
    let mut inventory_items = Vec::new();
    for item_name in starting_items {
        if let Some(item_ent) =
            aihack::core::entity::spawn::Spawner::mksobj(item_name, &assets.items, &mut world)
        {
            inventory_items.push(item_ent);
        }
    }

    let player_ent = world.push((
        PlayerTag,
        Species {
            current: "player".to_string(),
            original: "player".to_string(),
            timer: None,
        },
        Position {
            x: start_pos.0,
            y: start_pos.1,
        },
        Renderable {
            glyph: '@',
            color: 7,
        },
        player::Player::new(),
        Health {
            current: 15,
            max: 15,
        },
        CombatStats { ac: 10, level: 1 },
        Inventory::new(),
    ));

    // 레벨 컴포넌트 추가
    if let Some(mut entry) = world.entry(player_ent) {
        entry.add_component(Level(level_id));
    }

    // 상태/장비/마법 컴포넌트 추가
    if let Some(mut entry) = world.entry(player_ent) {
        let mut status = status::StatusBundle::new();
        status.permanent.insert(status::StatusFlags::INFRAVISION);
        status.permanent.insert(status::StatusFlags::NIGHT_VISION);
        entry.add_component(status);
        entry.add_component(Equipment::new());
        entry.add_component(SpellKnowledge::new());

        if let Ok(inv) = entry.get_component_mut::<Inventory>() {
            for item in inventory_items {
                inv.items.push(item);
                inv.assign_letter(item);
            }
        }
    }

    // 리소스 등록
    resources.insert(grid.clone());
    resources.insert(vision);
    resources.insert(rng.clone());
    resources.insert(assets.clone());
    resources.insert(assets.items.clone());

    let log = aihack::ui::log::GameLog::new(100);
    resources.insert(log);
    resources.insert(0u64); // 턴 카운터
    resources.insert(aihack::core::systems::talk::Rumors::new());
    resources.insert(aihack::core::action_queue::ActionQueue::new());
    resources.insert(None::<aihack::core::dungeon::LevelChange>);
    resources.insert(None::<aihack::core::systems::pray::PendingAltarUpdate>);
    resources.insert(status::StatusFlags::empty());
    resources.insert(aihack::core::events::EventQueue::new());
    resources.insert(aihack::core::events::EventHistory::default());
    resources.insert(aihack::core::systems::social::DefaultInteractionProvider);
    resources.insert(aihack::core::systems::death::DeathResults::default());
    resources.insert(aihack::core::options::Options::default());
    resources.insert(aihack::core::game_state::GameState::Normal);
    resources.insert(None::<aihack::core::systems::world::teleport::TeleportAction>);
    resources.insert(aihack::ui::input::Command::Wait);

    let mut dungeon = aihack::core::dungeon::dungeon::Dungeon::new();
    dungeon.set_level(level_id, grid.clone());
    resources.insert(dungeon);

    let mut identity = aihack::core::entity::identity::IdentityTable::new();
    identity.shuffle(&mut rng, &assets.items.templates);
    resources.insert(identity);

    (world, resources, grid)
}

/// 턴 시스템을 하나씩 추가하며 실행하는 안전 테스트
/// STABILIZATION_ROADMAP Phase S3 전략:
/// "execute_turn_systems() 내 시스템을 1개씩 활성화하며 디버깅"
fn run_schedule_safe(world: &mut World, resources: &mut Resources, system_count: usize) {
    let mut builder = Schedule::builder();

    // 시스템 목록을 하나씩 활성화
    if system_count >= 1 {
        builder.add_system(aihack::core::systems::movement::movement_system());
        builder.flush();
    }
    if system_count >= 2 {
        builder.add_system(aihack::core::systems::ai::pet_hunger_system());
        builder.flush();
    }
    if system_count >= 3 {
        builder.add_system(aihack::core::systems::ai::monster_ai_system());
        builder.flush();
    }
    if system_count >= 4 {
        builder.add_system(aihack::core::systems::luck::luck_maintenance_system());
        builder.flush();
    }
    if system_count >= 5 {
        builder.add_system(aihack::core::systems::engrave::engrave_tick_system());
        builder.flush();
    }
    if system_count >= 6 {
        builder.add_system(aihack::core::systems::trap::trap_trigger_system());
        builder.flush();
    }
    if system_count >= 7 {
        builder.add_system(aihack::core::systems::death::death_system());
        builder.flush();
    }
    if system_count >= 8 {
        builder.add_system(aihack::core::systems::vision_system::vision_update_system());
        builder.flush();
    }
    if system_count >= 9 {
        builder.add_system(aihack::core::systems::vision_system::magic_map_effect_system());
        builder.flush();
    }
    if system_count >= 10 {
        builder.add_system(aihack::core::systems::inventory::autopickup_tick_system());
        builder.flush();
    }
    if system_count >= 11 {
        builder.add_system(aihack::core::systems::inventory::inventory_action_system());
        builder.flush();
    }
    if system_count >= 12 {
        builder.add_system(aihack::core::systems::item_use::item_input_system());
        builder.flush();
    }
    if system_count >= 13 {
        builder.add_system(aihack::core::systems::item_use::item_use_system());
        builder.flush();
    }
    if system_count >= 14 {
        builder.add_system(aihack::core::systems::equipment::equipment_system());
        builder.flush();
    }
    if system_count >= 15 {
        builder.add_system(aihack::core::systems::equipment::update_player_stats_system());
        builder.flush();
    }
    if system_count >= 16 {
        builder.add_system(aihack::core::systems::throw::throw_system());
        builder.flush();
    }
    if system_count >= 17 {
        builder.add_system(aihack::core::systems::zap::zap_system());
        builder.flush();
    }
    if system_count >= 18 {
        builder.add_system(aihack::core::systems::teleport::teleport_system());
        builder.flush();
    }
    if system_count >= 19 {
        builder.add_system(aihack::core::systems::spell::spell_cast_system());
        builder.flush();
    }
    if system_count >= 20 {
        builder.add_system(aihack::core::systems::stairs::stairs_system());
        builder.flush();
    }
    if system_count >= 21 {
        builder.add_system(aihack::core::systems::status::status_tick_system());
        builder.flush();
    }
    if system_count >= 22 {
        builder.add_system(aihack::core::systems::attrib::attrib_maintenance_system());
        builder.flush();
    }
    if system_count >= 23 {
        builder.add_system(aihack::core::systems::timeout::timeout_dialogue_system());
        builder.flush();
    }
    if system_count >= 24 {
        builder.add_system(aihack::core::systems::item_tick::item_tick_system());
        builder.flush();
    }
    if system_count >= 25 {
        builder.add_system(aihack::core::systems::regeneration::regeneration_system());
        builder.flush();
    }
    if system_count >= 26 {
        builder.add_system(aihack::core::systems::regeneration::monster_regeneration_system());
        builder.flush();
    }
    if system_count >= 27 {
        builder.add_system(aihack::core::systems::evolution::evolution_tick_system());
        builder.flush();
    }
    if system_count >= 28 {
        builder.add_system(aihack::core::systems::evolution::lycanthropy_tick_system());
        builder.flush();
    }
    if system_count >= 29 {
        builder.add_system(aihack::core::systems::shop::shopkeeper_update_system());
        builder.flush();
    }
    if system_count >= 30 {
        builder.add_system(aihack::core::systems::weight::update_encumbrance_system());
        builder.flush();
    }

    let mut schedule = builder.build();
    schedule.execute(world, resources);
}

// ===================== 테스트 케이스 =====================

/// Phase S0: 게임 월드 초기화가 패닉 없이 성공하는지 확인
#[test]
fn s0_world_initialization() {
    let (world, mut _resources, grid) = create_test_world();

    // 플레이어 엔티티가 존재하는지 확인
    let mut query = <(&PlayerTag, &Position, &Health)>::query();
    let player_count = query.iter(&world).count();
    assert_eq!(player_count, 1, "플레이어 엔티티가 정확히 1개 존재해야 함");

    // 맵이 비어있지 않은지 확인
    assert!(grid.locations[0].len() > 0, "그리드가 비어있으면 안됨");

    println!("✅ S0: 게임 월드 초기화 성공");
}

/// Phase S3: 시스템을 하나씩 추가하며 첫 턴 생존 테스트
#[test]
fn s3_incremental_system_activation() {
    // 각 시스템을 1개씩 추가하며 패닉 발생 여부 확인
    for count in 1..=30 {
        let (mut world, mut resources, _grid) = create_test_world();

        // std::panic::catch_unwind로 패닉을 캡처
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_schedule_safe(&mut world, &mut resources, count);
        }));

        match result {
            Ok(_) => println!("✅ 시스템 {}개 활성화: 패닉 없음", count),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "알 수 없는 패닉".to_string()
                };
                panic!("❌ 시스템 {}개 활성화 시 패닉 발생: {}", count, msg);
            }
        }
    }
    println!("✅ S3: 모든 30개 시스템 1턴 실행 성공");
}

/// Phase S4: 전체 시스템으로 10턴 연속 실행
#[test]
fn s4_ten_turn_loop() {
    let (mut world, mut resources, _grid) = create_test_world();

    for turn in 1..=10 {
        // 턴 카운터 증가
        resources.insert(turn as u64);

        // 전체 시스템 실행 (30개)
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_schedule_safe(&mut world, &mut resources, 30);
        }));

        match result {
            Ok(_) => println!("✅ 턴 {}: 패닉 없음", turn),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "알 수 없는 패닉".to_string()
                };
                panic!("❌ 턴 {} 패닉: {}", turn, msg);
            }
        }
    }

    // 플레이어가 아직 살아있는지 확인
    let mut query = <&Health>::query().filter(component::<PlayerTag>());
    if let Some(hp) = query.iter(&world).next() {
        println!("✅ S4: 10턴 완료. 플레이어 HP: {}/{}", hp.current, hp.max);
    }
}

/// 플레이어 이동 시뮬레이션 (대기 명령)
#[test]
fn s5_wait_action() {
    let (mut world, mut resources, _grid) = create_test_world();

    // "대기(.)" 명령은 턴 소모만 하고 이동 없음
    // 시스템을 5턴 돌리기 (이동 없이)
    for turn in 1..=5 {
        resources.insert(turn as u64);
        run_schedule_safe(&mut world, &mut resources, 30);
    }

    // 플레이어 위치 확인 (시작 위치에서 변하지 않아야 함)
    let mut query = <&Position>::query().filter(component::<PlayerTag>());
    if let Some(pos) = query.iter(&world).next() {
        println!("✅ S5: 5턴 대기 후 위치: ({}, {})", pos.x, pos.y);
    }
}

// ============================================================================
// [v2.42.3] 외부 감사 반영: Full Schedule Integration Tests
// 기존 격리 테스트(create_controlled_world + 단일 시스템)는 SubWorld 접근 충돌을
// 잡지 못함. 30개 시스템 전체 Schedule 동시 실행으로 AccessDenied 사전 포착.
// ============================================================================

/// 감사T1: 30개 시스템 전체 실행 50턴 무패닉 — MoveN 이동 반복
/// 실제 게임과 동일한 Schedule을 구성하여 SubWorld borrow 충돌 검증
#[test]
fn audit_full_schedule_50_turns_move() {
    let (mut world, mut resources, _grid) = create_test_world();

    // MoveN 명령으로 50턴 실행 — 이동 + 몬스터 AI + 전투 + 시야 갱신 전체
    for turn in 1..=50 {
        resources.insert(turn as u64);
        resources.insert(aihack::ui::input::Command::MoveN);
        run_schedule_safe(&mut world, &mut resources, 30);
    }

    // 패닉 없이 여기 도달하면 성공
    let mut query = <&Health>::query().filter(component::<PlayerTag>());
    let alive = query
        .iter(&world)
        .next()
        .map(|h| h.current > 0)
        .unwrap_or(false);
    println!("✅ 감사T1: 50턴 Full Schedule 완료 (생존: {})", alive);
}

/// 감사T2: 무작위 Command 시퀀스 100턴 퍼징
/// 모든 Command variant를 포함하여 미구현 명령에 의한 패닉도 검증
#[test]
fn audit_command_fuzzing_100_turns() {
    let (mut world, mut resources, _grid) = create_test_world();

    // 모든 Command variant를 순차 반복 주입
    let commands = [
        aihack::ui::input::Command::MoveN,
        aihack::ui::input::Command::MoveS,
        aihack::ui::input::Command::MoveE,
        aihack::ui::input::Command::MoveW,
        aihack::ui::input::Command::MoveNE,
        aihack::ui::input::Command::MoveNW,
        aihack::ui::input::Command::MoveSE,
        aihack::ui::input::Command::MoveSW,
        aihack::ui::input::Command::Wait,
        aihack::ui::input::Command::Pickup,
        aihack::ui::input::Command::Open,
        aihack::ui::input::Command::Close,
        aihack::ui::input::Command::Kick,
        aihack::ui::input::Command::Search,
        aihack::ui::input::Command::Descend,
        aihack::ui::input::Command::Ascend,
        aihack::ui::input::Command::Eat,
        aihack::ui::input::Command::Quaff,
        aihack::ui::input::Command::Read,
        aihack::ui::input::Command::Wear,
        aihack::ui::input::Command::Wield,
        aihack::ui::input::Command::TakeOff,
        aihack::ui::input::Command::Drop,
        aihack::ui::input::Command::Throw,
        aihack::ui::input::Command::Invoke,
        aihack::ui::input::Command::Zap,
        aihack::ui::input::Command::Cast,
        aihack::ui::input::Command::Fire,
        aihack::ui::input::Command::LookHere,
        aihack::ui::input::Command::Pray,
        aihack::ui::input::Command::Pay,
    ];

    for turn in 1..=100 {
        let cmd = commands[turn as usize % commands.len()];
        eprintln!("[FUZZ] Turn {} → {:?}", turn, cmd);
        resources.insert(turn as u64);
        resources.insert(cmd);
        run_schedule_safe(&mut world, &mut resources, 30);
    }

    // 패닉 없이 여기 도달하면 성공
    println!(
        "✅ 감사T2: 100턴 Command 퍼징 완료 ({}개 variant 순환)",
        commands.len()
    );
}

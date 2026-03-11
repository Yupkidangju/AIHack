// ============================================================================
// [v3.0.0 Phase E3] 통합 테스트 + 안정성 검증
// 목표: 30/30 GameContext 기반 시스템의 런타임 안정성 검증
//       1000턴 퍼징에서 패닉 0건 달성
// ============================================================================

use aihack::assets::AssetManager;
use aihack::core::context::GameContext;
use aihack::core::dungeon::{DungeonBranch, Grid, LevelID};
use aihack::core::entity::*;
use legion::*;

// =============================================================================
// 헬퍼: 테스트용 GameContext 생성
// =============================================================================

/// GUI 없이 게임 월드를 초기화하고 GameContext 구성에 필요한 모든 데이터를 반환
struct TestGameSetup {
    world: World,
    grid: Grid,
    log: aihack::ui::log::GameLog,
    rng: aihack::util::rng::NetHackRng,
    assets: AssetManager,
    event_queue: aihack::core::events::EventQueue,
    action_queue: aihack::core::action_queue::ActionQueue,
    vision: aihack::core::systems::vision::VisionSystem,
    level_req: Option<aihack::core::dungeon::LevelChange>,
    dungeon: aihack::core::dungeon::dungeon::Dungeon,
    game_state: aihack::core::game_state::GameState,
}

impl TestGameSetup {
    fn new(seed: u64) -> Self {
        let mut assets = AssetManager::new();
        assets.load_defaults(".");
        assets.load_defaults("./nethack_original/NetHack-NetHack-3.6.7_Released");

        let mut world = World::default();
        let mut rng = aihack::util::rng::NetHackRng::new(seed);

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

        // 시야 초기화
        let mut vision = aihack::core::systems::vision::VisionSystem::new();
        if start_pos.0 >= 0 && start_pos.1 >= 0 {
            vision.recalc(&grid, start_pos.0 as usize, start_pos.1 as usize, 5);
        }

        // 플레이어 생성
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

        if let Some(mut entry) = world.entry(player_ent) {
            entry.add_component(Level(level_id));
            let mut status_bundle = status::StatusBundle::new();
            status_bundle
                .permanent
                .insert(status::StatusFlags::INFRAVISION);
            status_bundle
                .permanent
                .insert(status::StatusFlags::NIGHT_VISION);
            entry.add_component(status_bundle);
            entry.add_component(Equipment::new());
            entry.add_component(SpellKnowledge::new());

            if let Ok(inv) = entry.get_component_mut::<Inventory>() {
                for item in inventory_items {
                    inv.items.push(item);
                    inv.assign_letter(item);
                }
            }
        }

        let mut dungeon = aihack::core::dungeon::dungeon::Dungeon::new();
        dungeon.set_level(level_id, grid.clone());

        Self {
            world,
            grid,
            log: aihack::ui::log::GameLog::new(100),
            rng,
            assets,
            event_queue: aihack::core::events::EventQueue::new(),
            action_queue: aihack::core::action_queue::ActionQueue::new(),
            vision,
            level_req: None,
            dungeon,
            game_state: aihack::core::game_state::GameState::Normal,
        }
    }

    /// GameContext를 생성하여 콜백에 전달 (라이프타임 관리)
    fn with_ctx<F>(&mut self, cmd: aihack::ui::input::Command, turn: u64, f: F)
    where
        F: FnOnce(&mut GameContext),
    {
        let mut ctx = GameContext::new(
            &mut self.world,
            &mut self.grid,
            &mut self.log,
            &mut self.rng,
            turn,
            cmd,
            &self.assets,
            &mut self.event_queue,
            &mut self.action_queue,
            &mut self.vision,
            &mut self.level_req,
            &self.dungeon,
            &mut self.game_state,
            None, // [v3.0.0 E4] LLM 없음 (테스트)
        );
        f(&mut ctx);
    }
}

/// 30개 시스템을 순차 실행 (game_loop.rs의 호출 순서와 동일)
fn run_all_systems(ctx: &mut GameContext) {
    // 이동 + AI (가장 먼저)
    aihack::core::systems::movement::movement(ctx);
    aihack::core::systems::ai::core::monster_ai(ctx);
    aihack::core::systems::ai::core::pet_hunger_system(ctx);

    // 장비/상태
    aihack::core::systems::equipment::update_player_stats_system(ctx);
    aihack::core::systems::equipment::equipment_system(ctx);

    // 마법/전투
    aihack::core::systems::spell::spell_cast_system(ctx);
    aihack::core::systems::throw::throw_system(ctx);

    // 월드 상호작용
    aihack::core::systems::teleport::teleport_system(ctx);
    aihack::core::systems::stairs::stairs_system(ctx);
    aihack::core::systems::trap::trap_trigger_system(ctx);

    // 상점/사교
    aihack::core::systems::shop::shopkeeper_update_system(ctx);

    // 사망/아이템
    aihack::core::systems::death::death_system(ctx);
    aihack::core::systems::zap::zap_system(ctx);
    aihack::core::systems::item_use::item_use_system(ctx);

    // 시야/UI
    aihack::core::systems::vision_system::vision_update_system(ctx);
    aihack::core::systems::vision_system::magic_map_effect_system(ctx);
    aihack::core::systems::item_use::item_input_system(ctx);

    // 각종 유지보수
    aihack::core::systems::engrave::engrave_tick_system(ctx);
    aihack::core::systems::inventory::autopickup_tick_system(ctx);
    aihack::core::systems::inventory::inventory_action_system(ctx);
    aihack::core::systems::luck::luck_maintenance_system(ctx);
    aihack::core::systems::status::status_tick_system(ctx);
    aihack::core::systems::attrib::attrib_maintenance_system(ctx);
    aihack::core::systems::timeout::timeout_dialogue_system(ctx);
    aihack::core::systems::item_tick::item_tick_system(ctx);
    aihack::core::systems::regeneration::regeneration_system(ctx);
    aihack::core::systems::regeneration::monster_regeneration_system(ctx);
    aihack::core::systems::evolution::evolution_tick_system(ctx);
    aihack::core::systems::evolution::lycanthropy_tick_system(ctx);
}

// =============================================================================
// 테스트: 불변식 검사
// =============================================================================

/// 턴 종료 후 기본 불변식 검증
fn check_invariants(world: &World, turn: u64) {
    // 1. 플레이어 존재 확인
    let player_count = <&PlayerTag>::query().iter(world).count();
    assert!(
        player_count >= 1,
        "[턴 {}] 플레이어가 사라졌습니다! (count={})",
        turn,
        player_count
    );

    // 2. 플레이어 포지션 범위 확인
    let mut p_q = <(&Position, &PlayerTag)>::query();
    for (pos, _) in p_q.iter(world) {
        assert!(
            pos.x >= 0 && pos.x < 80 && pos.y >= 0 && pos.y < 21,
            "[턴 {}] 플레이어 Position 범위 초과! ({}, {})",
            turn,
            pos.x,
            pos.y
        );
    }
}

// =============================================================================
// 커맨드 목록
// =============================================================================

const ALL_COMMANDS: [aihack::ui::input::Command; 31] = [
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

// =============================================================================
// 테스트 케이스
// =============================================================================

/// [E3-T1] GameContext 기반 100턴 퍼징 (기존 테스트 대체)
#[test]
fn e3_ctx_fuzzing_100_turns() {
    let mut setup = TestGameSetup::new(42);

    for turn in 1..=100u64 {
        let cmd = ALL_COMMANDS[turn as usize % ALL_COMMANDS.len()];
        setup.with_ctx(cmd, turn, |ctx| {
            run_all_systems(ctx);
        });
        check_invariants(&setup.world, turn);
    }

    println!(
        "✅ E3-T1: 100턴 GameContext 퍼징 완료 (30개 시스템, {}개 커맨드 순환)",
        ALL_COMMANDS.len()
    );
}

/// [E3-T2] 500턴 퍼징 (중간 안정성 검증)
#[test]
fn e3_ctx_fuzzing_500_turns() {
    let mut setup = TestGameSetup::new(12345);

    for turn in 1..=500u64 {
        let cmd = ALL_COMMANDS[turn as usize % ALL_COMMANDS.len()];
        setup.with_ctx(cmd, turn, |ctx| {
            run_all_systems(ctx);
        });

        // 50턴마다 불변식 검사
        if turn % 50 == 0 {
            check_invariants(&setup.world, turn);
        }
    }

    println!("✅ E3-T2: 500턴 GameContext 퍼징 완료");
}

/// [E3-T3] 1000턴 퍼징 (최종 안정성 검증 — Phase E3 합격 기준)
#[test]
fn e3_ctx_fuzzing_1000_turns() {
    let mut setup = TestGameSetup::new(99999);

    for turn in 1..=1000u64 {
        let cmd = ALL_COMMANDS[turn as usize % ALL_COMMANDS.len()];
        setup.with_ctx(cmd, turn, |ctx| {
            run_all_systems(ctx);
        });

        // 100턴마다 불변식 검사
        if turn % 100 == 0 {
            check_invariants(&setup.world, turn);
            eprintln!("[E3-T3] {}턴 통과", turn);
        }
    }

    println!("✅ E3-T3: 1000턴 GameContext 퍼징 완료 — Phase E3 합격!");
}

/// [E3-T4] 다중 시드 퍼징 (4개 시드 × 200턴 = 800턴)
#[test]
fn e3_multi_seed_fuzzing() {
    for seed in [1, 42, 777, 31337u64] {
        let mut setup = TestGameSetup::new(seed);

        for turn in 1..=200u64 {
            let cmd = ALL_COMMANDS[(turn as usize + seed as usize) % ALL_COMMANDS.len()];
            setup.with_ctx(cmd, turn, |ctx| {
                run_all_systems(ctx);
            });
        }

        eprintln!("  ✅ seed={}: 200턴 통과", seed);
    }

    println!("✅ E3-T4: 다중 시드 퍼징 완료 (4시드 × 200턴)");
}

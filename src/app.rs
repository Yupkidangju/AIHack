// [v2.0.0 Phase R1] NetHackApp 구조체 정의 및 초기화 로직
// 앱 상태 머신, ECS World/Resources, 던전, 렌더러 등 핵심 상태 관리
//
//

use crate::assets::AssetManager;
use crate::core::dungeon::Grid;
use crate::core::entity::{Health, Inventory, PlayerTag, Position, Renderable};
use crate::core::game_state::GameState;
use legion::*;
use std::sync::{Arc, Mutex};

// ============================================================================
// [v2.20.0 Phase R7-1] 하위 구조체 정의
// NetHackApp의 22개 필드를 4개 논리 그룹으로 분해하기 위한 구조체.
// R7-1-A: 정의만 추가, R7-1-B에서 NetHackApp 필드를 이동.
// ============================================================================

/// 앱 흐름 제어 (타이틀 → 캐릭터 생성 → 플레이 → 게임 오버)
pub(crate) struct AppContext {
    /// 현재 앱 상태 (Title/CharCreation/Playing/GameOver)
    pub(crate) app_state: crate::core::role::AppState,
    /// 캐릭터 생성 단계
    pub(crate) char_creation_step: crate::core::role::CharCreationStep,
    /// 캐릭터 생성 선택 결과
    pub(crate) char_creation_choices: crate::core::role::CharCreationChoices,
    /// 캐릭터 이름 입력 버퍼
    pub(crate) char_name_buf: String,
    /// 게임 초기화 완료 여부
    pub(crate) game_initialized: bool,
}

/// ECS World + Resources + 게임 데이터
/// 주의: world와 resources를 동시에 &mut로 빌려야 할 경우,
/// 직접 필드 접근(self.game.world, self.game.resources)을 사용하면
/// Rust가 필드별 독립 차용을 허용합니다.
pub(crate) struct GameWorld {
    /// Legion ECS World — 모든 엔티티(플레이어, 몬스터, 아이템)의 컨테이너
    pub(crate) world: World,
    /// Legion ECS Resources — 싱글톤 데이터(RNG, GameLog, VisionSystem 등)
    pub(crate) resources: Resources,
    /// 현재 층의 던전 맵 정보
    pub(crate) grid: Grid,
    /// 전체 던전 관리자 (층 간 전환, 레벨 저장/로드)
    pub(crate) dungeon: crate::core::dungeon::dungeon::Dungeon,
    /// 심볼 및 데이터 에셋 (몬스터/아이템 템플릿)
    pub(crate) assets: AssetManager,
    /// 터미널 버퍼 (레거시, 현재 미사용)
    pub(crate) _terminal_buffer: Arc<Mutex<Vec<u8>>>,
}

/// UI 표시 상태 (렌더러, 레이아웃, 컨텍스트 메뉴 등)
pub(crate) struct UiState {
    /// Ratatui 하이브리드 렌더러
    pub(crate) renderer: crate::ui::renderer::HybridRenderer,
    /// 캐릭터 시트 표시 여부
    pub(crate) show_character: bool,
    /// 메시지 히스토리 표시 여부
    pub(crate) show_log_history: bool,
    /// 레이아웃 설정 (패널 토글 등)
    pub(crate) layout_settings: crate::ui::layout::menu_bar::LayoutSettings,
    /// 우클릭 컨텍스트 메뉴 상태
    pub(crate) context_menu_state: crate::ui::context_menu::ContextMenuState,
}

/// 입력 처리 및 게임 상태 머신
pub(crate) struct InputState {
    /// 게임 상태 머신 (Normal/WaitingForDirection/Inventory 등)
    pub(crate) game_state: crate::core::game_state::GameState,
    /// 마지막으로 처리된 커맨드
    pub(crate) last_cmd: crate::ui::input::Command,
    /// 마법 주문 단축키 입력 (a-z)
    pub(crate) spell_key_input: Option<char>,
    /// 확장 명령(#) 입력 모드 활성 여부
    pub(crate) ext_cmd_mode: bool,
    /// 확장 명령 입력 버퍼
    pub(crate) ext_cmd_input: String,
    /// Run(Shift+방향) 달리기 방향
    pub(crate) run_direction: Option<crate::core::game_state::Direction>,
    /// Travel 모드 경로 큐 (남은 이동 목록)
    pub(crate) travel_path: Vec<(i32, i32)>,
    /// 이름 입력을 위한 버퍼
    pub(crate) naming_input: String,
    /// 새기기 입력을 위한 버퍼
    pub(crate) engraving_input: String,
    /// 게임 옵션
    pub(crate) options: crate::core::options::Options,
}

/// NetHack-RS Main Application Structure
/// [v2.20.0 R7-1-B] 22개 개별 필드 → 4개 하위 구조체로 분해
pub(crate) struct NetHackApp {
    /// 앱 흐름 제어 (타이틀/캐릭터 생성/플레이/게임 오버)
    pub(crate) ctx: AppContext,
    /// ECS World + Resources + 게임 데이터
    pub(crate) game: GameWorld,
    /// UI 표시 상태
    pub(crate) ui: UiState,
    /// 입력 처리 및 게임 상태 머신
    pub(crate) input: InputState,
}

impl NetHackApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 1. 에셋 로드
        let mut assets = crate::assets::AssetManager::new();
        assets.load_defaults(".");

        // [v1.9.0

        // 2. 세이브 파일 확인
        let save_path = "save/player.sav";
        if std::path::Path::new(save_path).exists() {
            println!("세이브 파일을 발견하여 로드합니다: {}", save_path);
            match crate::core::save::SaveManager::load(save_path) {
                Ok((world, resources, dungeon)) => {
                    // 로드 성공 후 세이브 파일 삭제 (NetHack 전통: 죽으면 파일이 없거나, 로드 시 삭제)
                    let _ = std::fs::remove_file(save_path);

                    let _identity = resources
                        .get::<crate::core::entity::identity::IdentityTable>()
                        .map(|id| (*id).clone())
                        .unwrap_or_else(crate::core::entity::identity::IdentityTable::new);

                    return Self {
                        ctx: AppContext {
                            // 세이브 로드 시 바로 Playing 상태
                            app_state: crate::core::role::AppState::Playing,
                            char_creation_step: crate::core::role::CharCreationStep::SelectRole,
                            char_creation_choices: crate::core::role::CharCreationChoices::new(),
                            char_name_buf: String::new(),
                            game_initialized: true,
                        },
                        game: GameWorld {
                            grid: dungeon
                                .levels
                                .get(&dungeon.current_level)
                                .cloned()
                                .unwrap_or_else(|| crate::core::dungeon::Grid::new()),
                            assets,
                            _terminal_buffer: std::sync::Arc::new(
                                std::sync::Mutex::new(Vec::new()),
                            ),
                            world,
                            resources,
                            dungeon,
                        },
                        ui: UiState {
                            renderer: crate::ui::renderer::HybridRenderer::new(),
                            show_character: false,
                            show_log_history: false,
                            layout_settings: crate::ui::layout::menu_bar::LayoutSettings::default(),
                            context_menu_state: crate::ui::context_menu::ContextMenuState::default(
                            ),
                        },
                        input: InputState {
                            game_state: crate::core::game_state::GameState::default(),
                            options: crate::core::options::Options::load(),
                            naming_input: String::new(),
                            engraving_input: String::new(),
                            travel_path: Vec::new(),
                            ext_cmd_mode: false,
                            ext_cmd_input: String::new(),
                            run_direction: None,
                            last_cmd: crate::ui::input::Command::Unknown,
                            spell_key_input: None,
                        },
                    };
                }
                Err(e) => eprintln!("세이브 로드 실패: {}", e),
            }
        }

        //
        assets.load_defaults("./nethack_original/NetHack-NetHack-3.6.7_Released");

        let mut world = World::default();
        let mut resources = Resources::default();
        let options = crate::core::options::Options::load();

        // 심볼 세트 설정 동기화
        assets.symbols.current_set = options.current_symbol_set.clone();

        resources.insert(options.clone());
        resources.insert(assets.clone()); // AssetManager 등록

        // RNG 및 맵 생성
        let mut rng = crate::util::rng::NetHackRng::new(rand::random());

        // 몬스터 템플릿 준비 (초기 스폰 및 생성에 필요)
        let monster_templates: Vec<_> = assets.monsters.templates.values().collect();

        let (grid, start_pos, _down_pos, _rooms) =
            crate::core::dungeon::gen::MapGenerator::generate_improved(
                &mut rng,
                crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1),
                &mut world,
                &assets.items,
                &monster_templates,
                crate::core::dungeon::gen::LevelType::Ordinary,
            );
        let mut vision = crate::core::systems::vision::VisionSystem::new();
        // 초기 시야 계산 (Start Poistion 기반)
        if start_pos.0 >= 0 && start_pos.1 >= 0 {
            vision.recalc(&grid, start_pos.0 as usize, start_pos.1 as usize, 5);
            // 반경 5
        }

        // 5. 초기 아이템 지급 (Spawner::mksobj 사용)
        let starting_items = [
            "long sword",
            "small shield",
            "Potion of healing",
            "Scroll of teleportation",
            "lamp",
        ];

        let mut inventory_items = Vec::new();
        for item_name in starting_items {
            if let Some(item_ent) =
                crate::core::entity::spawn::Spawner::mksobj(item_name, &assets.items, &mut world)
            {
                inventory_items.push(item_ent);
            }
        }

        //
        let player_ent = world.push((
            PlayerTag,
            crate::core::entity::Species {
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
            crate::core::entity::player::Player::new(),
            crate::core::entity::Health {
                current: 15,
                max: 15,
            },
            crate::core::entity::CombatStats { ac: 10, level: 1 },
            crate::core::entity::Inventory::new(),
        ));

        if let Some(mut entry) = world.entry(player_ent) {
            entry.add_component(crate::core::entity::Level(
                crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1),
            ));
        }

        if let Some(mut entry) = world.entry(player_ent) {
            let mut status = crate::core::entity::status::StatusBundle::new();
            //
            status
                .permanent
                .insert(crate::core::entity::status::StatusFlags::INFRAVISION);
            status
                .permanent
                .insert(crate::core::entity::status::StatusFlags::NIGHT_VISION);
            entry.add_component(status);
            entry.add_component(crate::core::entity::Equipment::new());
            entry.add_component(crate::core::entity::SpellKnowledge::new());

            //
            if let Ok(inv) = entry.get_component_mut::<crate::core::entity::Inventory>() {
                for item in inventory_items {
                    inv.items.push(item);
                    inv.assign_letter(item);
                }
            }
        }

        resources.insert(grid.clone());
        resources.insert(vision);

        //
        resources.insert(rng.clone()); // Clone 필요 (Rng에 Clone 구현되어 있음)

        // 게임 로그 및 턴 카운터 초기화
        let log = crate::ui::log::GameLog::new(100);
        resources.insert(log);
        resources.insert(0u64); // Turn counter
        resources.insert(crate::core::systems::talk::Rumors::new()); // Rumors 시스템
        resources.insert(crate::core::action_queue::ActionQueue::new()); // 신규 ActionQueue 병행
        resources.insert(None::<crate::core::dungeon::LevelChange>); // Level change request
        resources.insert(None::<crate::core::systems::pray::PendingAltarUpdate>); // Altar conversion
                                                                                  // [v1.9.0
        resources.insert(crate::core::entity::status::StatusFlags::empty());
        resources.insert(crate::core::systems::death::DeathResults::default()); // [v2.0.0] 시체/드롭 요청
        resources.insert(crate::core::events::EventQueue::new()); // [v2.0.0 R5] 이벤트 큐
        resources.insert(crate::core::events::EventHistory::default()); // [v2.0.0 R5] 이벤트 히스토리
        resources.insert(crate::core::systems::social::DefaultInteractionProvider);

        // 던전 매니저 초기화 및 1층 등록
        let mut dungeon = crate::core::dungeon::dungeon::Dungeon::new();
        dungeon.set_level(
            crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1),
            grid.clone(),
        );
        resources.insert(dungeon.clone());

        // 아이템 감정 테이블 초기화 및 셔플 (rng 재사용)
        //
        //
        let mut identity = crate::core::entity::identity::IdentityTable::new();
        identity.shuffle(&mut rng, &assets.items.templates);
        resources.insert(identity.clone());

        // 초기 몬스터 스폰은 MapGenerator::generate_improved 내에서 처리됨 (mkroom.rs)
        /*
        crate::core::entity::spawn::Spawner::spawn_monsters(
            &mut world,
            &grid,
            &mut rng,
            &monster_templates,
            &assets.items,
            1,
            15,
        );
        */

        let renderer = crate::ui::renderer::HybridRenderer::new();

        Self {
            ctx: AppContext {
                // [v1.9.0
                app_state: crate::core::role::AppState::Title,
                char_creation_step: crate::core::role::CharCreationStep::SelectRole,
                char_creation_choices: crate::core::role::CharCreationChoices::new(),
                char_name_buf: String::new(),
                game_initialized: false,
            },
            game: GameWorld {
                grid,
                assets,
                _terminal_buffer: Arc::new(Mutex::new(Vec::new())),
                world,
                resources,
                dungeon,
            },
            ui: UiState {
                renderer,
                show_character: false,
                show_log_history: false,
                layout_settings: crate::ui::layout::menu_bar::LayoutSettings::default(),
                context_menu_state: crate::ui::context_menu::ContextMenuState::default(),
            },
            input: InputState {
                game_state: crate::core::game_state::GameState::default(),
                options,
                naming_input: String::new(),
                engraving_input: String::new(),
                travel_path: Vec::new(),
                ext_cmd_mode: false,
                ext_cmd_input: String::new(),
                run_direction: None,
                last_cmd: crate::ui::input::Command::Unknown,
                spell_key_input: None,
            },
        }
    }

    pub(crate) fn restart_game(&mut self) {
        let mut world = World::default();
        let mut resources = Resources::default();
        let assets = self.game.assets.clone();

        let mut rng = crate::util::rng::NetHackRng::new(rand::random());
        let (grid, _up_pos, _down_pos, _rooms) =
            crate::core::dungeon::gen::MapGenerator::generate_improved(
                &mut rng,
                crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1),
                &mut world,
                &assets.items,
                &assets.monsters.templates.values().collect(),
                crate::core::dungeon::gen::LevelType::Ordinary,
            );

        resources.insert(rng.clone());
        resources.insert(crate::ui::log::GameLog::new(100));
        resources.insert(0u64);
        resources.insert(crate::core::systems::talk::Rumors::new());
        resources.insert(crate::core::action_queue::ActionQueue::new()); // 신규 ActionQueue 병행
        resources.insert(None::<crate::core::dungeon::LevelChange>);
        resources.insert(crate::core::systems::death::DeathResults::default());
        resources.insert(crate::core::events::EventQueue::new()); // [v2.0.0 R5]
        resources.insert(crate::core::events::EventHistory::default()); // [v2.0.0 R5]

        let mut dungeon = crate::core::dungeon::dungeon::Dungeon::new();
        dungeon.set_level(
            crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1),
            grid.clone(),
        );
        resources.insert(dungeon.clone());

        let mut identity = crate::core::entity::identity::IdentityTable::new();
        identity.shuffle(&mut rng, &assets.items.templates);
        resources.insert(identity.clone());

        self.game.world = world;
        self.game.resources = resources;
        self.game.grid = grid;
        self.game.dungeon = dungeon;
        self.input.game_state = GameState::default();
        // [M3/M4] 리셋
        self.input.travel_path.clear();
        self.input.ext_cmd_mode = false;
        self.input.ext_cmd_input.clear();
        self.input.run_direction = None;
    }

    ///
    ///
    pub(crate) fn initialize_game_with_choices(
        &mut self,
        choices: &crate::core::role::CharCreationChoices,
    ) {
        use crate::core::role::{get_race_data, get_role_data};

        let role = match choices.role {
            Some(r) => r,
            None => return,
        };
        let race = match choices.race {
            Some(r) => r,
            None => return,
        };
        let _gender = match choices.gender {
            Some(r) => r,
            None => return,
        };
        let _alignment = match choices.alignment {
            Some(r) => r,
            None => return,
        };

        let role_data = get_role_data(role);
        let race_data = get_race_data(race);

        //
        let mut world = World::default();
        let mut resources = Resources::default();
        let assets = self.game.assets.clone();

        let mut rng = crate::util::rng::NetHackRng::new(rand::random());

        let monster_templates: Vec<_> = assets.monsters.templates.values().collect();
        let (grid, start_pos, _down_pos, _rooms) =
            crate::core::dungeon::gen::MapGenerator::generate_improved(
                &mut rng,
                crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1),
                &mut world,
                &assets.items,
                &monster_templates,
                crate::core::dungeon::gen::LevelType::Ordinary,
            );

        let mut vision = crate::core::systems::vision::VisionSystem::new();
        if start_pos.0 >= 0 && start_pos.1 >= 0 {
            vision.recalc(&grid, start_pos.0 as usize, start_pos.1 as usize, 5);
        }

        // 2. 직업별 초기 아이템 (role.c 기반, 향후 확장 필요)
        let starting_items = match role {
            crate::core::role::Role::Barbarian => vec!["long sword", "small shield"],
            crate::core::role::Role::Knight => {
                vec!["long sword", "large shield", "Potion of healing"]
            }
            crate::core::role::Role::Wizard => vec!["Scroll of teleportation", "lamp"],
            crate::core::role::Role::Rogue => vec!["short sword", "Potion of healing"],
            crate::core::role::Role::Ranger => vec!["short sword", "Potion of healing"],
            crate::core::role::Role::Valkyrie => vec!["long sword", "small shield"],
            crate::core::role::Role::Archeologist => vec!["Potion of healing", "lamp"],
            crate::core::role::Role::Caveman => vec!["Potion of healing"],
            crate::core::role::Role::Healer => vec!["Potion of healing", "Potion of healing"],
            crate::core::role::Role::Monk => vec!["Potion of healing"],
            crate::core::role::Role::Priest => vec!["Potion of healing"],
            crate::core::role::Role::Samurai => vec!["long sword", "small shield"],
            crate::core::role::Role::Tourist => {
                vec!["Potion of healing", "Scroll of teleportation"]
            }
        };

        let mut inventory_items = Vec::new();
        for item_name in &starting_items {
            if let Some(item_ent) =
                crate::core::entity::spawn::Spawner::mksobj(item_name, &assets.items, &mut world)
            {
                inventory_items.push(item_ent);
            }
        }

        //
        let base_hp = role_data.base_hp + race_data.hp_bonus;
        let _base_stats = &role_data.base_stats;

        let player_ent = world.push((
            PlayerTag,
            crate::core::entity::Species {
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
            crate::core::entity::player::Player::new(),
            Health {
                current: base_hp,
                max: base_hp,
            },
            crate::core::entity::CombatStats {
                ac: 10 + role_data.base_ac, // 직업별 AC 보정
                level: 1,
            },
            Inventory::new(),
        ));

        //
        if let Some(mut entry) = world.entry(player_ent) {
            entry.add_component(crate::core::entity::Level(
                crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1),
            ));
        }

        //
        if let Some(mut entry) = world.entry(player_ent) {
            let mut status = crate::core::entity::status::StatusBundle::new();
            // 종족 특성 (Elf/Orc: INFRAVISION)
            match race {
                crate::core::role::Race::Elf | crate::core::role::Race::Orc => {
                    status
                        .permanent
                        .insert(crate::core::entity::status::StatusFlags::INFRAVISION);
                }
                _ => {}
            }
            status
                .permanent
                .insert(crate::core::entity::status::StatusFlags::NIGHT_VISION);
            entry.add_component(status);
            entry.add_component(crate::core::entity::Equipment::new());
            entry.add_component(crate::core::entity::SpellKnowledge::new());

            //
            if let Ok(inv) = entry.get_component_mut::<Inventory>() {
                for item in inventory_items {
                    inv.items.push(item);
                    inv.assign_letter(item);
                }
            }
        }

        // 4. 리소스 등록
        resources.insert(grid.clone());
        resources.insert(vision);
        resources.insert(rng.clone());
        resources.insert(assets.clone());
        resources.insert(self.input.options.clone());

        let mut log = crate::ui::log::GameLog::new(100);
        // 환영 메시지 (직업/종족 반영)
        log.add(
            format!(
                "{} the {} {}, welcome to AIHack!",
                choices.name,
                race_data.adjective,
                choices.role_display_name(),
            ),
            0,
        );
        log.add(
            format!("You are a {} {}.", _alignment, choices.role_display_name()),
            0,
        );
        resources.insert(log);
        resources.insert(0u64);
        resources.insert(crate::core::systems::talk::Rumors::new());
        resources.insert(crate::core::action_queue::ActionQueue::new()); // 신규 ActionQueue 병행
        resources.insert(None::<crate::core::dungeon::LevelChange>);
        resources.insert(None::<crate::core::systems::pray::PendingAltarUpdate>);
        resources.insert(crate::core::entity::status::StatusFlags::empty());
        resources.insert(crate::core::systems::death::DeathResults::default());
        resources.insert(crate::core::events::EventQueue::new()); // [v2.0.0 R5]
        resources.insert(crate::core::events::EventHistory::default()); // [v2.0.0 R5]
        resources.insert(crate::core::systems::social::DefaultInteractionProvider);

        let mut dungeon = crate::core::dungeon::dungeon::Dungeon::new();
        dungeon.set_level(
            crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1),
            grid.clone(),
        );
        resources.insert(dungeon.clone());

        let mut identity = crate::core::entity::identity::IdentityTable::new();
        identity.shuffle(&mut rng, &assets.items.templates);
        resources.insert(identity.clone());

        // 5. 앱 상태 갱신
        self.game.world = world;
        self.game.resources = resources;
        self.game.grid = grid;
        self.game.dungeon = dungeon;
        self.input.game_state = crate::core::game_state::GameState::default();
        self.ctx.game_initialized = true;
    }
}

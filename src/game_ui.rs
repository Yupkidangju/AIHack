// [v2.0.0 Phase R1] main.rs에서 분리된 UI 렌더링 로직
//
//
//

use crate::core::entity::*;
use crate::core::game_state::GameState;
use crate::ui::input::Command;
use eframe::egui;
use legion::*;

impl super::NetHackApp {
    /// 게임 UI 렌더링  egui 패널 + ratatui 맵 + 팝업 윈도우
    /// main.rs의 update()에서 process_game_turn() 이후 호출됨
    pub(crate) fn render_game_ui(&mut self, ctx: &egui::Context) {
        //
        {
            let player_info: Option<(Entity, i32, i32)> = {
                let mut q = <(Entity, &Position)>::query().filter(component::<PlayerTag>());
                q.iter(&self.game.world).next().map(|(e, p)| (*e, p.x, p.y))
            };
            if let Some((ent, px, py)) = player_info {
                let has_inv = self.game.world
                    .entry_ref(ent)
                    .map(|e| e.get_component::<Inventory>().is_ok())
                    .unwrap_or(false);
                let has_health = self.game.world
                    .entry_ref(ent)
                    .map(|e| e.get_component::<Health>().is_ok())
                    .unwrap_or(false);
                println!(
                    "[Debug] Player Entity: {:?}, Pos: {},{}, Inv: {}, Health: {}",
                    ent, px, py, has_inv, has_health
                );
            }
        }

        ctx.request_repaint();

        // ======================================================================
        // [v1.9.0
        // ======================================================================

        // (A) 턴 정보 추출
        let turn_count = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);

        // (B) 메뉴 바 렌더링 (최상단)
        {
            use crate::ui::layout::menu_bar::{render_menu_bar, MenuAction};
            let menu_action = render_menu_bar(ctx, turn_count, &mut self.ui.layout_settings);
            match menu_action {
                MenuAction::Inventory => {
                    self.input.last_cmd = Command::Inventory;
                }
                MenuAction::MessageHistory => {
                    self.ui.show_log_history = !self.ui.show_log_history;
                }
                MenuAction::CharacterInfo => {
                    self.ui.show_character = !self.ui.show_character;
                }
                MenuAction::Save => {
                    self.input.last_cmd = Command::Save;
                }
                MenuAction::Quit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                _ => {}
            }
        }

        // (C) 커맨드 바 렌더링 (최하단)
        {
            use crate::ui::layout::command_bar::{render_command_bar, CommandBarAction};
            let cmd_action = render_command_bar(ctx, self.ui.layout_settings.command_advanced_mode);
            // 커맨드 바 클릭 → 기존 Command enum으로 변환
            match cmd_action {
                CommandBarAction::Pickup => {
                    self.input.last_cmd = Command::Pickup;
                }
                CommandBarAction::Inventory => {
                    self.input.last_cmd = Command::Inventory;
                }
                CommandBarAction::Eat => {
                    self.input.last_cmd = Command::Eat;
                }
                CommandBarAction::Quaff => {
                    self.input.last_cmd = Command::Quaff;
                }
                CommandBarAction::Read => {
                    self.input.last_cmd = Command::Read;
                }
                CommandBarAction::Apply => {
                    self.input.last_cmd = Command::Apply;
                }
                CommandBarAction::Zap => {
                    self.input.last_cmd = Command::Zap;
                }
                CommandBarAction::Cast => {
                    self.input.last_cmd = Command::Cast;
                }
                CommandBarAction::Pray => {
                    self.input.last_cmd = Command::Pray;
                }
                CommandBarAction::Search => {
                    self.input.last_cmd = Command::Search;
                }
                CommandBarAction::Help => {
                    self.input.last_cmd = Command::Help;
                }
                CommandBarAction::Throw => {
                    self.input.last_cmd = Command::Throw;
                }
                CommandBarAction::Kick => {
                    self.input.last_cmd = Command::Kick;
                }
                CommandBarAction::Open => {
                    self.input.last_cmd = Command::Open;
                }
                CommandBarAction::Close => {
                    self.input.last_cmd = Command::Close;
                }
                CommandBarAction::Wear => {
                    self.input.last_cmd = Command::Wear;
                }
                CommandBarAction::TakeOff => {
                    self.input.last_cmd = Command::TakeOff;
                }
                CommandBarAction::Wield => {
                    self.input.last_cmd = Command::Wield;
                }
                CommandBarAction::Engrave => {
                    self.input.last_cmd = Command::Engrave;
                }
                CommandBarAction::Name => {
                    self.input.last_cmd = Command::Name;
                }
                CommandBarAction::Save => {
                    self.input.last_cmd = Command::Save;
                }
                CommandBarAction::None => {}
            }
        }

        // (D) 상태 바 렌더링 (커맨드 바 바로 위)
        {
            use crate::ui::layout::status_bar::{
                render_status_bar, xp_for_level, StatusBarMode, StatusEffect, StatusInfo,
            };

            let mut status_info = StatusInfo::default();

            //
            let mut query = <(
                &crate::core::entity::player::Player,
                &crate::core::entity::Health,
                &crate::core::entity::CombatStats,
                &crate::core::entity::status::StatusBundle,
            )>::query()
            .filter(component::<PlayerTag>());

            if let Some((player, health, stats, status)) = query.iter(&self.game.world).next() {
                status_info.str_ = player.str.base;
                status_info.dex = player.dex.base;
                status_info.con = player.con.base;
                status_info.int = player.int.base;
                status_info.wis = player.wis.base;
                status_info.cha = player.cha.base;
                status_info.hp = health.current;
                status_info.hp_max = health.max;
                status_info.energy = player.energy;
                status_info.energy_max = player.energy_max;
                status_info.ac = stats.ac;
                status_info.level = stats.level;
                status_info.experience = player.experience;
                status_info.gold = player.gold;
                status_info.alignment = "Neutral".to_string();
                status_info.turn = turn_count;

                // [v2.3.0 M6] 다음 레벨 경험치 계산
                status_info.xp_for_next_level = xp_for_level(stats.level + 1);

                // 던전 깊이 문자열
                status_info.depth_str = match self.game.dungeon.current_level.branch {
                    crate::core::dungeon::DungeonBranch::Main => {
                        format!("Dlvl:{}", self.game.dungeon.current_level.depth)
                    }
                    crate::core::dungeon::DungeonBranch::Mines => {
                        format!("Mines:{}", self.game.dungeon.current_level.depth)
                    }
                    crate::core::dungeon::DungeonBranch::Sokoban => {
                        format!("Sokoban:{}", self.game.dungeon.current_level.depth)
                    }
                    _ => format!("Dlvl:{}", self.game.dungeon.current_level.depth),
                };

                // [v2.1.0 M2] 상태 이상 아이콘 목록 (확장)
                use crate::core::entity::status::StatusFlags;
                let flag_effects: Vec<(&str, &str, egui::Color32, StatusFlags)> = vec![
                    // 감각 이상
                    ("Blind", "??", egui::Color32::YELLOW, StatusFlags::BLIND),
                    (
                        "Conf",
                        "??",
                        egui::Color32::from_rgb(255, 0, 255),
                        StatusFlags::CONFUSED,
                    ),
                    (
                        "Stun",
                        "??",
                        egui::Color32::from_rgb(255, 200, 0),
                        StatusFlags::STUNNED,
                    ),
                    (
                        "Halluc",
                        "??",
                        egui::Color32::from_rgb(255, 100, 200),
                        StatusFlags::HALLUCINATING,
                    ),
                    // 질병/중독
                    (
                        "Sick",
                        "??",
                        egui::Color32::from_rgb(0, 200, 0),
                        StatusFlags::SICK,
                    ),
                    (
                        "FPois",
                        "?",
                        egui::Color32::from_rgb(180, 0, 220),
                        StatusFlags::FOOD_POISONING,
                    ),
                    // 이동
                    (
                        "Lev",
                        "??",
                        egui::Color32::from_rgb(200, 200, 255),
                        StatusFlags::LEVITATING,
                    ),
                    (
                        "Fly",
                        "??",
                        egui::Color32::from_rgb(150, 200, 255),
                        StatusFlags::FLYING,
                    ),
                    (
                        "Slow",
                        "??",
                        egui::Color32::from_rgb(100, 100, 200),
                        StatusFlags::SLOW,
                    ),
                    (
                        "Haste",
                        "?",
                        egui::Color32::from_rgb(100, 200, 255),
                        StatusFlags::FAST,
                    ),
                    // 치명
                    (
                        "Stone",
                        "??",
                        egui::Color32::from_rgb(160, 160, 170),
                        StatusFlags::STONING,
                    ),
                    (
                        "Slime",
                        "??",
                        egui::Color32::from_rgb(0, 180, 0),
                        StatusFlags::SLIMED,
                    ),
                    (
                        "Strngl",
                        "?",
                        egui::Color32::from_rgb(200, 50, 50),
                        StatusFlags::STRANGLED,
                    ),
                    // 하중
                    ("Burden", "??", egui::Color32::YELLOW, StatusFlags::BURDENED),
                    ("Stress", "?", egui::Color32::GOLD, StatusFlags::STRESSED),
                    (
                        "Strain",
                        "?",
                        egui::Color32::from_rgb(255, 165, 0),
                        StatusFlags::STRAINED,
                    ),
                    (
                        "OvrTax",
                        "??",
                        egui::Color32::LIGHT_RED,
                        StatusFlags::OVERTAXED,
                    ),
                    ("OvrLd", "?", egui::Color32::RED, StatusFlags::OVERLOADED),
                ];

                for (name, icon, color, flag) in flag_effects {
                    if status.has(flag) {
                        status_info
                            .status_effects
                            .push(StatusEffect { name, icon, color });
                    }
                }

                // [v2.1.0 M2] 배고픔 상태 아이콘 (Player.hunger에서 추출)
                use crate::core::entity::player::HungerState;
                match player.hunger {
                    HungerState::Satiated => {
                        status_info.status_effects.push(StatusEffect {
                            name: "Satiated",
                            icon: "??",
                            color: egui::Color32::from_rgb(0, 200, 0),
                        });
                    }
                    HungerState::Hungry => {
                        status_info.status_effects.push(StatusEffect {
                            name: "Hungry",
                            icon: "??",
                            color: egui::Color32::from_rgb(255, 165, 0),
                        });
                    }
                    HungerState::Weak => {
                        status_info.status_effects.push(StatusEffect {
                            name: "Weak",
                            icon: "??",
                            color: egui::Color32::RED,
                        });
                    }
                    HungerState::Fainting => {
                        status_info.status_effects.push(StatusEffect {
                            name: "Faint",
                            icon: "??",
                            color: egui::Color32::RED,
                        });
                    }
                    HungerState::Starved => {
                        status_info.status_effects.push(StatusEffect {
                            name: "Starved",
                            icon: "??",
                            color: egui::Color32::DARK_RED,
                        });
                    }
                    HungerState::NotHungry => {}
                }
            }

            render_status_bar(ctx, &status_info, StatusBarMode::Graphical);
        }

        // (E) 우측 스탯 패널 (조건부 표시)
        if self.ui.layout_settings.show_stats_panel {
            use crate::ui::layout::stats_panel::{
                render_stats_panel, EquipmentSummary, StatsPanelData,
            };
            let mut panel_data = StatsPanelData::default();

            let mut query = <(
                &crate::core::entity::player::Player,
                &crate::core::entity::Health,
                &crate::core::entity::CombatStats,
            )>::query()
            .filter(component::<PlayerTag>());

            if let Some((player, health, stats)) = query.iter(&self.game.world).next() {
                panel_data.name = if self.ctx.char_name_buf.is_empty() {
                    format!("{:?}", player.role)
                } else {
                    self.ctx.char_name_buf.clone()
                };
                panel_data.title = format!("the {:?}", player.role);
                panel_data.hp = health.current;
                panel_data.hp_max = health.max;
                panel_data.energy = player.energy;
                panel_data.energy_max = player.energy_max;
                panel_data.str_ = player.str.base;
                panel_data.dex = player.dex.base;
                panel_data.con = player.con.base;
                panel_data.int = player.int.base;
                panel_data.wis = player.wis.base;
                panel_data.cha = player.cha.base;
                panel_data.ac = stats.ac;
                panel_data.level = stats.level;
                panel_data.gold = player.gold;
                panel_data.depth = match self.game.dungeon.current_level.branch {
                    crate::core::dungeon::DungeonBranch::Main => {
                        format!("Dlvl:{}", self.game.dungeon.current_level.depth)
                    }
                    _ => format!(
                        "{:?}:{}",
                        self.game.dungeon.current_level.branch, self.game.dungeon.current_level.depth
                    ),
                };

                // [v2.1.0
                panel_data.equipment = EquipmentSummary::default();
                {
                    //
                    let mut eq_query = <(&crate::core::entity::Equipment,)>::query()
                        .filter(component::<PlayerTag>());

                    if let Some((equip,)) = eq_query.iter(&self.game.world).next() {
                        use crate::core::entity::EquipmentSlot;
                        // 각 슬롯에 대해 Entity → Item.kind.as_str() 조회
                        let slot_mapping: Vec<(
                            EquipmentSlot,
                            Box<dyn FnMut(&mut EquipmentSummary, String)>,
                        )> = vec![
                            (
                                EquipmentSlot::Melee,
                                Box::new(|eq: &mut EquipmentSummary, s| eq.weapon = s),
                            ),
                            (
                                EquipmentSlot::Shield,
                                Box::new(|eq: &mut EquipmentSummary, s| eq.shield = s),
                            ),
                            (
                                EquipmentSlot::Body,
                                Box::new(|eq: &mut EquipmentSummary, s| eq.armor = s),
                            ),
                            (
                                EquipmentSlot::Head,
                                Box::new(|eq: &mut EquipmentSummary, s| eq.helmet = s),
                            ),
                            (
                                EquipmentSlot::Cloak,
                                Box::new(|eq: &mut EquipmentSummary, s| eq.cloak = s),
                            ),
                            (
                                EquipmentSlot::Hands,
                                Box::new(|eq: &mut EquipmentSummary, s| eq.gloves = s),
                            ),
                            (
                                EquipmentSlot::Feet,
                                Box::new(|eq: &mut EquipmentSummary, s| eq.boots = s),
                            ),
                        ];

                        for (slot, mut setter) in slot_mapping {
                            if let Some(&item_ent) = equip.slots.get(&slot) {
                                if let Ok(entry) = self.game.world.entry_ref(item_ent) {
                                    if let Ok(item) =
                                        entry.get_component::<crate::core::entity::Item>()
                                    {
                                        let name = if item.spe != 0 {
                                            format!("{:+} {}", item.spe, item.kind)
                                        } else {
                                            format!("{}", item.kind)
                                        };
                                        setter(&mut panel_data.equipment, name);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            render_stats_panel(ctx, &panel_data);
        }

        //
        if self.ui.layout_settings.show_minimap || self.ui.layout_settings.show_message_panel {
            egui::SidePanel::left("left_panel")
                .default_width(200.0)
                .min_width(160.0)
                .max_width(280.0)
                .resizable(true)
                .frame(
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgb(16, 16, 22))
                        .inner_margin(egui::Margin::same(6.0))
                        .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(45, 45, 60))),
                )
                .show(ctx, |ui| {
                    // 미니맵
                    if self.ui.layout_settings.show_minimap {
                        //
                        let mut px = 0usize;
                        let mut py = 0usize;
                        {
                            let mut pq = <&Position>::query().filter(component::<PlayerTag>());
                            if let Some(pos) = pq.iter(&self.game.world).next() {
                                px = pos.x as usize;
                                py = pos.y as usize;
                            }
                        }

                        if let Some(vision) = self.game.resources
                            .get::<crate::core::systems::vision::VisionSystem>()
                        {
                            let minimap_data = crate::ui::layout::minimap::MinimapData {
                                grid: &self.game.grid,
                                vision: &vision,
                                player_x: px,
                                player_y: py,
                            };
                            crate::ui::layout::minimap::render_minimap(ui, &minimap_data);
                        }
                    }

                    // 메시지 로그
                    if self.ui.layout_settings.show_message_panel {
                        if let Some(log) = self.game.resources.get::<crate::ui::log::GameLog>() {
                            crate::ui::layout::message_panel::render_message_panel(ui, &log);
                        }
                    }
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // [v2.1.0 M2] 버전 문자열 최신화
                ui.heading("NetHack-RS (v2.1.0)");
                ui.separator();
                ui.label(format!(
                    "Map: {}x{}",
                    crate::core::dungeon::COLNO,
                    crate::core::dungeon::ROWNO
                ));
                let current_set = &self.game.assets.symbols.current_set;
                ui.label(format!("Symbols: {}", current_set));
            });

            // [v1.9.0 M3] Hybrid UI: Ratatui Canvas Area ? Sense::click으로 마우스 클릭 감지
            let (rect, response) = ui.allocate_at_least(ui.available_size(), egui::Sense::click());
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgb(10, 10, 10));

            // 프레임 렌더링 실행
            if let Some(vision) = self.game.resources
                .get::<crate::core::systems::vision::VisionSystem>()
            {
                self.ui.renderer
                    .render_frame(&self.game.grid, &self.game.world, &self.game.assets, &vision);
            }

            // Ratatui 버퍼 내용을 egui Painter로 복사
            let buffer = self.ui.renderer.terminal.backend().buffer();
            let font_id = egui::FontId::monospace(14.0);
            let char_width = 8.5; // 고정폭 폰트 너비 근사치
            let char_height = 14.0;

            for y in 0..crate::core::dungeon::ROWNO {
                for x in 0..crate::core::dungeon::COLNO {
                    let cell = buffer.get(x as u16, y as u16);
                    let pos = rect.min
                        + egui::vec2(x as f32 * char_width + 5.0, y as f32 * char_height + 30.0);

                    let color = match cell.fg {
                        ratatui::style::Color::Black => egui::Color32::BLACK,
                        ratatui::style::Color::Red | ratatui::style::Color::LightRed => {
                            egui::Color32::RED
                        }
                        ratatui::style::Color::Green | ratatui::style::Color::LightGreen => {
                            egui::Color32::GREEN
                        }
                        ratatui::style::Color::Yellow | ratatui::style::Color::LightYellow => {
                            egui::Color32::YELLOW
                        }
                        ratatui::style::Color::Blue | ratatui::style::Color::LightBlue => {
                            egui::Color32::BLUE
                        }
                        ratatui::style::Color::Magenta | ratatui::style::Color::LightMagenta => {
                            egui::Color32::from_rgb(255, 0, 255)
                        }
                        ratatui::style::Color::Cyan | ratatui::style::Color::LightCyan => {
                            egui::Color32::from_rgb(0, 255, 255)
                        }
                        ratatui::style::Color::White => egui::Color32::WHITE,
                        ratatui::style::Color::Gray | ratatui::style::Color::DarkGray => {
                            egui::Color32::GRAY
                        }
                        _ => egui::Color32::WHITE,
                    };

                    ui.painter().text(
                        pos,
                        egui::Align2::LEFT_TOP,
                        cell.symbol(),
                        font_id.clone(),
                        color,
                    );
                }
            }

            egui::Window::new("Message History (Ctrl+P)")
                .open(&mut self.ui.show_log_history)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            if let Some(log) = self.game.resources.get::<crate::ui::log::GameLog>() {
                                for msg in &log.history {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(150, 150, 150),
                                            format!("[{:>04}]", msg.turn),
                                        );
                                        let display_text = if msg.count > 1 {
                                            format!("{} (x{})", msg.text, msg.count)
                                        } else {
                                            msg.text.clone()
                                        };
                                        ui.colored_label(
                                            egui::Color32::from_rgb(
                                                msg.color[0],
                                                msg.color[1],
                                                msg.color[2],
                                            ),
                                            display_text,
                                        );
                                    });
                                }
                            }
                        });
                });

            // Mouse Hover Tooltip (Phase 3)
            if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
                if rect.contains(hover_pos) {
                    let gx = ((hover_pos.x - rect.min.x - 5.0) / char_width) as i32;
                    let gy = ((hover_pos.y - rect.min.y - 30.0) / char_height) as i32;

                    if gx >= 0
                        && gx < crate::core::dungeon::COLNO as i32
                        && gy >= 0
                        && gy < crate::core::dungeon::ROWNO as i32
                    {
                        // 시야 정보 확인
                        let mut is_visible = false;
                        let mut is_memorized = false;
                        if let Some(vision) = self.game.resources
                            .get::<crate::core::systems::vision::VisionSystem>()
                        {
                            let flags = vision.viz_array[gx as usize][gy as usize];
                            is_visible = (flags & crate::core::systems::vision::IN_SIGHT) != 0;
                            is_memorized = (flags & crate::core::systems::vision::MEMORIZED) != 0;
                        }

                        if is_visible || is_memorized {
                            egui::show_tooltip(ctx, egui::Id::new("map_tooltip"), |ui| {
                                ui.label(format!("Location: ({}, {})", gx, gy));

                                // 타일 정보
                                if let Some(tile) = self.game.grid.get_tile(gx as usize, gy as usize) {
                                    ui.label(format!("Terrain: {:?}", tile.typ));
                                }

                                // 엔티티 정보 (시야 내에 있을 때만)
                                if is_visible {
                                    let mut i_query =
                                        <(Entity, &Position, &crate::core::entity::Item)>::query();
                                    for (_ent, pos, item) in i_query.iter(&self.game.world) {
                                        if pos.x == gx && pos.y == gy {
                                            ui.colored_label(
                                                egui::Color32::YELLOW,
                                                format!("Item: {}", item.kind),
                                            );
                                        }
                                    }

                                    let mut m_query = <(
                                        Entity,
                                        &Position,
                                        &crate::core::entity::Monster,
                                    )>::query(
                                    );
                                    for (_ent, pos, monster) in m_query.iter(&self.game.world) {
                                        if pos.x == gx && pos.y == gy {
                                            ui.colored_label(
                                                egui::Color32::LIGHT_RED,
                                                format!("Monster: {}", monster.kind),
                                            );
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            }

            // ================================================================
            // [v1.9.0 M3] 마우스 클릭 처리 ? 좌클릭(이동/공격), 우클릭(검사)
            // ================================================================
            {
                use crate::ui::mouse;

                //
                let mut player_x = 0i32;
                let mut player_y = 0i32;
                {
                    let mut pq = <&Position>::query().filter(component::<PlayerTag>());
                    if let Some(pos) = pq.iter(&self.game.world).next() {
                        player_x = pos.x;
                        player_y = pos.y;
                    }
                }

                // 좌클릭 처리
                if response.clicked() {
                    if let Some(click_pos) = response.interact_pointer_pos() {
                        if let Some((gx, gy)) = mouse::screen_to_grid(
                            click_pos,
                            rect,
                            char_width,
                            char_height,
                            5.0,
                            30.0,
                        ) {
                            let action = mouse::handle_left_click(gx, gy, player_x, player_y);
                            match action {
                                mouse::MouseAction::AdjacentMove { dir, .. } => {
                                    // 인접 이동/공격 (1턴)
                                    self.input.last_cmd = mouse::direction_to_command(dir);
                                    // Travel 모드 해제
                                    self.input.travel_path.clear();
                                }
                                mouse::MouseAction::Travel { target_x, target_y } => {
                                    // A* 경로 탐색 후 Travel 큐에 저장
                                    let path = crate::util::path::PathFinder::find_path(
                                        &self.game.grid,
                                        (player_x, player_y),
                                        (target_x, target_y),
                                        |grid, x, y| {
                                            //
                                            if let Some(tile) =
                                                grid.get_tile(x as usize, y as usize)
                                            {
                                                use crate::core::dungeon::tile::TileType;
                                                matches!(
                                                    tile.typ,
                                                    TileType::Room
                                                        | TileType::OpenDoor
                                                        | TileType::Corr
                                                        | TileType::StairsUp
                                                        | TileType::StairsDown
                                                        | TileType::Altar
                                                        | TileType::Fountain
                                                        | TileType::Throne
                                                        | TileType::Sink
                                                        | TileType::Grave
                                                        | TileType::Door
                                                )
                                            } else {
                                                false
                                            }
                                        },
                                    );
                                    if let Some(mut p) = path {
                                        if p.len() > 1 {
                                            //
                                            self.input.travel_path = p.split_off(1);
                                        }
                                    }
                                }
                                mouse::MouseAction::SelfClick => {
                                    // 자기 위치 클릭 → 줍기 시도
                                    self.input.last_cmd = Command::Pickup;
                                    self.input.travel_path.clear();
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // 우클릭 처리 (secondary_clicked)
                if response.secondary_clicked() {
                    if let Some(click_pos) = response.interact_pointer_pos() {
                        if let Some((gx, gy)) = mouse::screen_to_grid(
                            click_pos,
                            rect,
                            char_width,
                            char_height,
                            5.0,
                            30.0,
                        ) {
                            self.ui.context_menu_state.visible = true;
                            self.ui.context_menu_state.grid_x = gx;
                            self.ui.context_menu_state.grid_y = gy;
                        }
                    }
                }
            }
        });

        // ================================================================
        // [v2.1.0 M2] Settings 윈도우 ? 게임 옵션 변경
        // ================================================================
        if self.ui.layout_settings.show_settings {
            let mut open = self.ui.layout_settings.show_settings;
            egui::Window::new("? Settings")
                .open(&mut open)
                .collapsible(true)
                .resizable(true)
                .default_width(320.0)
                .show(ctx, |ui| {
                    ui.heading("Game Options");
                    ui.add_space(6.0);

                    // 자동 줍기 설정
                    ui.checkbox(&mut self.input.options.autopickup, "Autopickup (자동 줍기)");
                    if self.input.options.autopickup {
                        ui.horizontal(|ui| {
                            ui.label("  Pickup types:");
                            ui.text_edit_singleline(&mut self.input.options.pickup_types);
                        });
                    }

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // 표시 옵션
                    ui.checkbox(&mut self.input.options.show_exp, "Show Experience (경험치 표시)");
                    ui.checkbox(&mut self.input.options.show_score, "Show Score (점수 표시)");
                    ui.checkbox(&mut self.input.options.color, "Use Colors (색상 사용)");
                    ui.checkbox(&mut self.input.options.hilite_pet, "Highlight Pets (펫 강조)");

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // 심볼 세트 선택
                    ui.label("Symbol Set (심볼 세트):");
                    let symbol_sets = ["Original", "IBMgraphics", "DECgraphics"];
                    for set in &symbol_sets {
                        let selected = self.input.options.current_symbol_set == *set;
                        if ui.selectable_label(selected, *set).clicked() {
                            self.input.options.current_symbol_set = set.to_string();
                            self.game.assets.symbols.current_set = set.to_string();
                        }
                    }

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    //
                    ui.horizontal(|ui| {
                        ui.label("Dog name:");
                        ui.text_edit_singleline(&mut self.input.options.dogname);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Cat name:");
                        ui.text_edit_singleline(&mut self.input.options.catname);
                    });

                    ui.add_space(8.0);

                    // 저장 버튼
                    if ui.button("?? Save Options").clicked() {
                        self.input.options.save();
                        if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                            let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                            log.add("Options saved.", turn);
                        }
                    }
                });
            self.ui.layout_settings.show_settings = open;
        }

        // ================================================================
        // [v1.9.0 M4] 확장 명령(#) 입력 팝업
        // ================================================================
        if self.input.ext_cmd_mode {
            let mut close_popup = false;
            let mut resolved_cmd = Command::Unknown;

            egui::Window::new("Extended Command")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("# 명령어를 입력하세요:");
                    let re = ui.text_edit_singleline(&mut self.input.ext_cmd_input);
                    re.request_focus();

                    // Enter 키로 확인
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Some(cmd) = Command::from_extended_str(&self.input.ext_cmd_input) {
                            resolved_cmd = cmd;
                        }
                        close_popup = true;
                    }
                    // ESC 키로 취소
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        close_popup = true;
                    }

                    // 자동 완성 후보 목록 표시
                    if !self.input.ext_cmd_input.is_empty() {
                        let input_lower = self.input.ext_cmd_input.to_lowercase();
                        let candidates = [
                            "adjust",
                            "chat",
                            "conduct",
                            "dip",
                            "force",
                            "jump",
                            "loot",
                            "monster",
                            "name",
                            "offer",
                            "overview",
                            "pray",
                            "ride",
                            "rub",
                            "score",
                            "sit",
                            "tip",
                            "travel",
                            "turn",
                            "turncount",
                            "twoweapon",
                            "untrap",
                            "version",
                            "wipe",
                        ];
                        let matches: Vec<&&str> = candidates
                            .iter()
                            .filter(|c| c.starts_with(&input_lower))
                            .collect();

                        if !matches.is_empty() {
                            ui.separator();
                            for m in &matches {
                                if ui.selectable_label(false, **m).clicked() {
                                    self.input.ext_cmd_input = m.to_string();
                                    if let Some(cmd) =
                                        Command::from_extended_str(&self.input.ext_cmd_input)
                                    {
                                        resolved_cmd = cmd;
                                    }
                                    close_popup = true;
                                }
                            }
                        }
                    }
                });

            if close_popup {
                self.input.ext_cmd_mode = false;
                if resolved_cmd != Command::Unknown {
                    self.input.last_cmd = resolved_cmd;
                }
                self.input.ext_cmd_input.clear();
            }
        }

        // [M4] 확장 명령 진입
        if self.input.last_cmd == Command::ExtendedCommand && !self.input.ext_cmd_mode {
            self.input.ext_cmd_mode = true;
            self.input.ext_cmd_input.clear();
            self.input.last_cmd = Command::Unknown; // 이번 프레임은 명령 없음
        }

        // ================================================================
        // [v1.9.0
        // ================================================================
        // Run 명령 진입: 방향 저장
        match self.input.last_cmd {
            Command::RunN => {
                self.input.run_direction = Some(crate::core::game_state::Direction::North);
                self.input.last_cmd = Command::MoveN;
            }
            Command::RunS => {
                self.input.run_direction = Some(crate::core::game_state::Direction::South);
                self.input.last_cmd = Command::MoveS;
            }
            Command::RunE => {
                self.input.run_direction = Some(crate::core::game_state::Direction::East);
                self.input.last_cmd = Command::MoveE;
            }
            Command::RunW => {
                self.input.run_direction = Some(crate::core::game_state::Direction::West);
                self.input.last_cmd = Command::MoveW;
            }
            Command::RunNE => {
                self.input.run_direction = Some(crate::core::game_state::Direction::NorthEast);
                self.input.last_cmd = Command::MoveNE;
            }
            Command::RunNW => {
                self.input.run_direction = Some(crate::core::game_state::Direction::NorthWest);
                self.input.last_cmd = Command::MoveNW;
            }
            Command::RunSE => {
                self.input.run_direction = Some(crate::core::game_state::Direction::SouthEast);
                self.input.last_cmd = Command::MoveSE;
            }
            Command::RunSW => {
                self.input.run_direction = Some(crate::core::game_state::Direction::SouthWest);
                self.input.last_cmd = Command::MoveSW;
            }
            _ => {}
        }

        //
        if self.input.last_cmd == Command::Unknown {
            if let Some(dir) = self.input.run_direction {
                // 다음 칸의 이동 가능 여부 확인
                let mut can_continue = false;
                let (dx, dy) = dir.to_delta();

                //
                let player_pos: Option<(i32, i32)> = {
                    let mut pq = <&Position>::query().filter(component::<PlayerTag>());
                    pq.iter(&self.game.world).next().map(|p| (p.x, p.y))
                };

                if let Some((px, py)) = player_pos {
                    let nx = px + dx;
                    let ny = py + dy;
                    // [v1.9.0] 음수 좌표 안전 가드
                    if nx >= 0 && ny >= 0 {
                        if let Some(tile) = self.game.grid.get_tile(nx as usize, ny as usize) {
                            use crate::core::dungeon::tile::TileType;
                            can_continue = matches!(
                                tile.typ,
                                TileType::Room
                                    | TileType::OpenDoor
                                    | TileType::Corr
                                    | TileType::StairsUp
                                    | TileType::StairsDown
                            );
                        }
                    }
                }

                //
                if can_continue {
                    if let Some(vision) = self.game.resources
                        .get::<crate::core::systems::vision::VisionSystem>()
                    {
                        let mut mq = <&Position>::query().filter(
                            !component::<PlayerTag>() & component::<crate::core::entity::Monster>(),
                        );
                        for mpos in mq.iter(&self.game.world) {
                            if mpos.x >= 0
                                && mpos.x < crate::core::dungeon::COLNO as i32
                                && mpos.y >= 0
                                && mpos.y < crate::core::dungeon::ROWNO as i32
                            {
                                let flags = vision.viz_array[mpos.x as usize][mpos.y as usize];
                                if (flags & crate::core::systems::vision::IN_SIGHT) != 0 {
                                    can_continue = false;
                                    break;
                                }
                            }
                        }
                    }
                }

                if can_continue {
                    self.input.last_cmd = crate::ui::mouse::direction_to_command(dir);
                } else {
                    self.input.run_direction = None; // 달리기 종료
                }
            }
        } else if self.input.last_cmd != Command::Unknown {
            //
            if self.input.run_direction.is_some()
                && !matches!(
                    self.input.last_cmd,
                    Command::MoveN
                        | Command::MoveS
                        | Command::MoveE
                        | Command::MoveW
                        | Command::MoveNE
                        | Command::MoveNW
                        | Command::MoveSE
                        | Command::MoveSW
                )
            {
                self.input.run_direction = None;
            }
        }

        // ================================================================
        // [v1.9.0 M4] 신규 명령 기본 처리 (메시지 로그로 피드백)
        // ================================================================
        {
            let msg: Option<&str> = match self.input.last_cmd {
                Command::WhatIs => Some("What is that symbol? (Click a tile to inspect)"),
                Command::LookHere => Some("You look around."),
                Command::LookAtFloor => Some("You look at the ground beneath you."),
                Command::ShowWeapon => Some("(Weapon info displayed in Stats Panel)"),
                Command::ShowArmor => Some("(Armor info displayed in Stats Panel)"),
                Command::ShowRings => Some("(Ring info displayed in Stats Panel)"),
                Command::ShowAmulet => Some("(Amulet info displayed in Stats Panel)"),
                Command::ShowTool => Some("(Tool info displayed in Stats Panel)"),
                Command::Discoveries => Some("You recall your discoveries..."),
                Command::InventoryClass => Some("(Inventory by class - use category tabs)"),
                Command::Version => Some("AIHack v2.1.0 ? NetHack 3.6.7 Rust Port"),
                Command::Conduct => Some("You have followed these conducts..."),
                Command::Score => Some("Your current score is displayed."),
                Command::Overview => Some("Overview of your explorations..."),
                Command::TurnCount => None, // 아래에서 턴 수 직접 표시
                Command::Travel => None,    // M3 Travel 시스템과 연동
                Command::Dip => Some("Dip which item into what?"),
                Command::Force => Some("Force the lock on what?"),
                Command::Jump => Some("Where do you want to jump?"),
                Command::Ride => Some("You look around for a mount..."),
                Command::Rub => Some("Rub what?"),
                Command::Tip => Some("Tip what container?"),
                Command::TurnUndead => Some("You attempt to turn undead..."),
                Command::Untrap => Some("You search for traps nearby."),
                Command::Wipe => Some("You wipe your face."),
                Command::Chat => Some("Who do you want to chat with?"),
                Command::Adjust => Some("Adjust which inventory letter?"),
                Command::Monster => Some("You have no special monster abilities."),
                _ => None,
            };

            if let Some(text) = msg {
                let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    log.add(text, turn);
                }
                // 검사/정보 명령은 턴 소비 안 함 → Unknown으로 리셋
                match self.input.last_cmd {
                    Command::WhatIs
                    | Command::LookHere
                    | Command::LookAtFloor
                    | Command::ShowWeapon
                    | Command::ShowArmor
                    | Command::ShowRings
                    | Command::ShowAmulet
                    | Command::ShowTool
                    | Command::Discoveries
                    | Command::InventoryClass
                    | Command::Version
                    | Command::Conduct
                    | Command::Score
                    | Command::Overview
                    | Command::Monster => {
                        self.input.last_cmd = Command::Unknown;
                    }
                    _ => {}
                }
            }

            // TurnCount 특별 처리
            if self.input.last_cmd == Command::TurnCount {
                let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                let msg = format!("Current turn: {}", turn);
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    log.add(msg, turn);
                }
                self.input.last_cmd = Command::Unknown;
            }
        }

        // [v1.9.0 M3] Travel 큐 처리 ? 매 프레임 1칸씩 소비
        if !self.input.travel_path.is_empty() && self.input.last_cmd == Command::Unknown {
            let (tx, ty) = self.input.travel_path[0];
            //
            let mut px = 0i32;
            let mut py = 0i32;
            {
                let mut pq = <&Position>::query().filter(component::<PlayerTag>());
                if let Some(pos) = pq.iter(&self.game.world).next() {
                    px = pos.x;
                    py = pos.y;
                }
            }
            let dx = tx - px;
            let dy = ty - py;

            // 방향 변환
            if let Some(dir) = crate::ui::mouse::delta_to_direction_pub(dx, dy) {
                // Travel 중단 조건: 시야 내 새 몬스터 감지
                let mut monster_nearby = false;
                {
                    let mut mq = <&Position>::query().filter(
                        !component::<PlayerTag>() & component::<crate::core::entity::Monster>(),
                    );
                    if let Some(vision) = self.game.resources
                        .get::<crate::core::systems::vision::VisionSystem>()
                    {
                        for mpos in mq.iter(&self.game.world) {
                            if mpos.x >= 0
                                && mpos.x < crate::core::dungeon::COLNO as i32
                                && mpos.y >= 0
                                && mpos.y < crate::core::dungeon::ROWNO as i32
                            {
                                let flags = vision.viz_array[mpos.x as usize][mpos.y as usize];
                                if (flags & crate::core::systems::vision::IN_SIGHT) != 0 {
                                    monster_nearby = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if monster_nearby {
                    // 몬스터 감지 → Travel 중단
                    self.input.travel_path.clear();
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                        log.add_colored(
                            "You stop traveling. A monster is nearby!",
                            [255, 200, 100],
                            turn,
                        );
                    }
                } else {
                    self.input.last_cmd = crate::ui::mouse::direction_to_command(dir);
                    self.input.travel_path.remove(0);
                }
            } else {
                // 방향 변환 실패 → Travel 중단
                self.input.travel_path.clear();
            }
        }

        // [v1.9.0
        crate::ui::context_menu::render_context_menu(
            ctx,
            &mut self.ui.context_menu_state,
            &self.game.world,
            &self.game.grid,
            &self.game.resources,
        );

        // Floating UI Example
        egui::Window::new("Character Status (C)")
            .open(&mut self.ui.show_character)
            .show(ctx, |ui| {
                let mut query = <(
                    &crate::core::entity::player::Player,
                    &crate::core::entity::Health,
                    &crate::core::entity::CombatStats,
                )>::query()
                .filter(component::<PlayerTag>());
                for (player, health, stats) in query.iter(&self.game.world) {
                    ui.horizontal(|ui| {
                        ui.label("Name: Hero");
                        ui.label(format!(
                            "St:{} Dx:{} Co:{} In:{} Wi:{} Ch:{}",
                            player.str.base,
                            player.dex.base,
                            player.con.base,
                            player.int.base,
                            player.wis.base,
                            player.cha.base
                        ));
                    });
                    let dlvl_str = match self.game.dungeon.current_level.branch {
                        crate::core::dungeon::DungeonBranch::Main => {
                            format!("Dlvl:{}", self.game.dungeon.current_level.depth)
                        }
                        crate::core::dungeon::DungeonBranch::Mines => {
                            format!("Mines:{}", self.game.dungeon.current_level.depth)
                        }
                        crate::core::dungeon::DungeonBranch::Sokoban => {
                            format!("Sokoban:{}", self.game.dungeon.current_level.depth)
                        }
                        _ => format!("Dlvl:{}", self.game.dungeon.current_level.depth),
                    };

                    ui.label(format!(
                        "{} $:{} HP:{}/{} AC:{} Exp:{} Luck:{} Align:{} ({:?})",
                        dlvl_str,
                        player.gold,
                        health.current,
                        health.max,
                        stats.ac,
                        player.level, // Exp Level
                        player.luck + player.luck_bonus,
                        player.alignment_record,
                        player.alignment
                    ));
                }
            });

        // Inventory Window (i)
        if self.input.game_state == GameState::Inventory
            || matches!(self.input.game_state, GameState::Looting { .. })
        {
            egui::Window::new("Inventory (i)").show(ctx, |ui| {
                let mut query = <(
                    &crate::core::entity::Inventory,
                    &crate::core::entity::Equipment,
                )>::query()
                .filter(component::<PlayerTag>());
                for (inv, equip) in query.iter(&self.game.world) {
                    let looting_container =
                        if let GameState::Looting { container } = self.input.game_state {
                            Some(container)
                        } else {
                            None
                        };

                    let action = {
                        let identity = self.game.resources
                            .get::<crate::core::entity::identity::IdentityTable>()
                            .unwrap();
                        crate::ui::widgets::inventory::show_inventory(
                            ui,
                            &self.game.world,
                            inv,
                            equip,
                            &self.game.assets.items,
                            &identity,
                            looting_container,
                        )
                    };

                    if let Some(action) = action {
                        self.game.resources.insert(Some(action));
                    }
                }
            });
        }

        // Looting Window
        if let GameState::Looting { container } = self.input.game_state {
            egui::Window::new("Looting").show(ctx, |ui| {
                if let Ok(entry) = self.game.world.entry_ref(container) {
                    if let Ok(inv) = entry.get_component::<crate::core::entity::Inventory>() {
                        let identity = self.game.resources
                            .get::<crate::core::entity::identity::IdentityTable>()
                            .map(|id| (*id).clone())
                            .unwrap();
                        if let Some(action) = crate::ui::widgets::loot::show_loot_menu(
                            ui,
                            &self.game.world,
                            container,
                            inv,
                            &self.game.assets.items,
                            &identity,
                        ) {
                            self.game.resources.insert(Some(action));
                        }
                    }
                }

                if ui.button("Close").clicked() {
                    self.input.game_state = GameState::Normal;
                }
            });
        }

        // Offer Selection Window (#offer)
        if self.input.game_state == GameState::OfferSelection {
            egui::Window::new("Sacrifice Item").show(ctx, |ui| {
                let mut query = <(&crate::core::entity::Inventory, &PlayerTag)>::query();
                for (inv, _) in query.iter(&self.game.world) {
                    if let Some(action) = crate::ui::widgets::offer::show_offer_menu(
                        ui,
                        &self.game.world,
                        inv,
                        &self.game.assets.items,
                    ) {
                        self.game.resources.insert(Some(action));
                        self.input.game_state = GameState::Normal; // 액션 선택 시 자동 닫힘
                    }
                }

                if ui.button("Cancel").clicked() {
                    self.input.game_state = GameState::Normal;
                }
            });
        }

        // Naming Window (#name, #call)
        if let GameState::Naming { entity, is_call } = self.input.game_state {
            if entity.is_none() {
                egui::Window::new("Select Object to Name").show(ctx, |ui| {
                    let mut query =
                        <&crate::core::entity::Inventory>::query().filter(component::<PlayerTag>());
                    let mut selected = None;
                    for inv in query.iter(&self.game.world) {
                        let identity = self.game.resources
                            .get::<crate::core::entity::identity::IdentityTable>()
                            .unwrap();
                        selected = crate::ui::widgets::inventory::show_naming_selector(
                            ui,
                            &self.game.world,
                            inv,
                            &self.game.assets.items,
                            &identity,
                        );
                    }
                    if let Some(target) = selected {
                        self.input.game_state = GameState::Naming {
                            entity: Some(target),
                            is_call,
                        };
                    }
                    if ui.button("Cancel").clicked() {
                        self.input.game_state = GameState::Normal;
                    }
                });
            } else {
                egui::Window::new(if is_call { "Call Item Class" } else { "Name Object" }).show(ctx, |ui| {
                    ui.label("Enter name:");
                    let edit = ui.text_edit_singleline(&mut self.input.naming_input);
                    if (edit.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Apply").clicked() {
                        let final_name = self.input.naming_input.trim().to_string();
                        let name_to_set = if final_name.is_empty() { None } else { Some(final_name.clone()) };

                        if let Some(target) = entity {
                            if is_call {
                                // Call logic
                                if let Ok(entry) = self.game.world.entry_ref(target) {
                                    if let Ok(item) = entry.get_component::<crate::core::entity::Item>() {
                                        if let Some(template) = self.game.assets.items.get_by_kind(item.kind) {
                                            if let Some(desc) = crate::core::systems::item_helper::ItemHelper::get_description(item, template) {
                                                let mut ident_table = self.game.resources.get_mut::<crate::core::entity::identity::IdentityTable>().unwrap();
                                                if let Some(ident) = ident_table.mapping.get_mut(&desc) {
                                                    ident.call_name = name_to_set;
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                // Name logic
                                if let Ok(mut entry) = self.game.world.entry_mut(target) {
                                    if let Ok(item) = entry.get_component_mut::<crate::core::entity::Item>() {
                                        item.user_name = name_to_set.clone();
                                    } else if let Ok(mon) = entry.get_component_mut::<crate::core::entity::Monster>() {
                                        mon.mon_name = name_to_set.clone();
                                    }
                                }

                                //
                                if let Some(n) = &name_to_set {
                                    let mut p_query = <&crate::core::entity::player::Player>::query().filter(component::<PlayerTag>());
                                    let player_opt = p_query.iter(&self.game.world).next().cloned();
                                    if let Some(player) = player_opt {
                                        let mut log = self.game.resources.get_mut::<crate::ui::log::GameLog>().unwrap();
                                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                                        crate::core::systems::artifact::ArtifactSystem::try_artifact_promotion(
                                            target,
                                            n,
                                            &mut self.game.world,
                                            &self.game.assets,
                                            &player,
                                            &mut log,
                                            turn,
                                        );
                                    }
                                }
                            }
                        }

                        self.input.naming_input.clear();
                        self.input.game_state = GameState::Normal;
                    }

                    if ui.button("Cancel").clicked() {
                        self.input.naming_input.clear();
                        self.input.game_state = GameState::Normal;
                    }
                });
            }
        }

        // Invoke Selection Window (#invoke)
        if self.input.game_state == GameState::SelectInvoke {
            egui::Window::new("Invoke Artifact").show(ctx, |ui| {
                let mut query =
                    <&crate::core::entity::Inventory>::query().filter(component::<PlayerTag>());
                let mut selected = None;
                for inv in query.iter(&self.game.world) {
                    let identity = self.game.resources
                        .get::<crate::core::entity::identity::IdentityTable>()
                        .unwrap();
                    selected = crate::ui::widgets::inventory::show_invoke_selector(
                        ui,
                        &self.game.world,
                        inv,
                        &self.game.assets.items,
                        &identity,
                    );
                }

                if let Some(target) = selected {
                    let mut p_query = <Entity>::query().filter(component::<PlayerTag>());
                    let player_ent = p_query.iter(&self.game.world).next().cloned();

                    if let Some(p_entity) = player_ent {
                        let mut log = self.game.resources.get_mut::<crate::ui::log::GameLog>().unwrap();
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        crate::core::systems::artifact::ArtifactSystem::invoke_artifact(
                            target,
                            &mut self.game.world,
                            &self.game.assets,
                            p_entity,
                            &mut log,
                            turn,
                        );
                    }
                    self.input.game_state = GameState::Normal;
                }

                if ui.button("Cancel").clicked() {
                    self.input.game_state = GameState::Normal;
                }
            });
        }

        // Enhance Window (#enhance)
        if self.input.game_state == GameState::Enhance {
            egui::Window::new("Enhance Skills").show(ctx, |ui| {
                let mut advanced = false;
                {
                    let mut query = <&mut crate::core::entity::player::Player>::query()
                        .filter(component::<PlayerTag>());
                    for player in query.iter_mut(&mut self.game.world) {
                        ui.label("Current Skills:");
                        ui.separator();

                        // Collect and sort skills for consistent display
                        let mut skills_vec: Vec<_> = player.skills.iter_mut().collect();
                        skills_vec.sort_by(|a, b| (*a.0 as u8).cmp(&(*b.0 as u8)));

                        for (skill, record) in skills_vec {
                            // NetHack shows all skills or only those with progress?
                            // We'll show all that are at least Unskilled or have potential
                            if record.level != crate::core::entity::skills::SkillLevel::Restricted {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{:?}", skill));
                                    ui.label(format!(" [{:?}]", record.level));

                                    if record.can_advance() {
                                        if ui.button("Advance").clicked() {
                                            if record.advance_level() {
                                                advanced = true;
                                            }
                                        }
                                    } else {
                                        // Show progress?
                                        ui.add_enabled(
                                            false,
                                            egui::Button::new(format!(
                                                "{}/{}",
                                                record.advance,
                                                record.practice_needed()
                                            )),
                                        );
                                    }
                                });
                            }
                        }
                    }
                }

                if advanced {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                        log.add("You feel more confident in your skills.", turn);
                    }
                }

                ui.separator();
                if ui.button("Close").clicked() {
                    self.input.game_state = GameState::Normal;
                }
            });
        }

        // Confirm Drink Fountain Window
        if self.input.game_state == GameState::ConfirmDrinkFountain {
            egui::Window::new("Drink").show(ctx, |ui| {
                ui.label("Drink from the fountain?");
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        // 분수 마시기 실행
                        let (mut subworld, _) = self.game.world.split_for_query(&<(
                            &mut crate::core::entity::player::Player,
                            &mut crate::core::entity::Health,
                            &mut crate::core::entity::status::StatusBundle,
                            &Position,
                        )>::query(
                        ));

                        if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                            let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                            if let Some(mut rng) =
                                self.game.resources.get_mut::<crate::util::rng::NetHackRng>()
                            {
                                crate::core::systems::fountain::try_drink_fountain(
                                    &mut subworld,
                                    &mut self.game.grid,
                                    &mut log,
                                    turn,
                                    &mut rng,
                                    &self.game.assets.items,
                                );
                            }
                        }
                        self.input.game_state = GameState::Normal;
                    }
                    if ui.button("No").clicked() {
                        //
                        self.input.game_state = GameState::Inventory;
                    }
                });
            });
        }

        // Off-hand Selection Window
        if self.input.game_state == GameState::SelectOffhand {
            let mut selected_action = None;
            egui::Window::new("Dual Wield").show(ctx, |ui| {
                let ident_table = self.game.resources
                    .get::<crate::core::entity::identity::IdentityTable>()
                    .unwrap();
                let mut query =
                    <&crate::core::entity::Inventory>::query().filter(component::<PlayerTag>());
                for inv in query.iter(&self.game.world) {
                    if let Some(action) = crate::ui::widgets::inventory::show_offhand_selector(
                        ui,
                        &self.game.world,
                        inv,
                        &self.game.assets.items,
                        &ident_table,
                    ) {
                        selected_action = Some(action);
                    }
                }

                if ui.button("Cancel").clicked() {
                    self.input.game_state = GameState::Normal;
                }
            });

            if let Some(action) = selected_action {
                self.game.resources.insert(Some(action));
                self.input.game_state = GameState::Normal;
            }
        }

        // Quiver Selection Window
        if self.input.game_state == GameState::SelectQuiver {
            let mut selected_action = None;
            egui::Window::new("Quiver Selection").show(ctx, |ui| {
                let ident_table = self.game.resources
                    .get::<crate::core::entity::identity::IdentityTable>()
                    .unwrap();
                let mut query =
                    <&crate::core::entity::Inventory>::query().filter(component::<PlayerTag>());
                for inv in query.iter(&self.game.world) {
                    if let Some(action) = crate::ui::widgets::inventory::show_quiver_selector(
                        ui,
                        &self.game.world,
                        inv,
                        &self.game.assets.items,
                        &ident_table,
                    ) {
                        selected_action = Some(action);
                    }
                }

                if ui.button("Cancel").clicked() {
                    self.input.game_state = GameState::Normal;
                }
            });

            if let Some(action) = selected_action {
                self.game.resources.insert(Some(action));
                self.input.game_state = GameState::Normal;
            }
        }

        // Offer Selection Window (#offer)
        if self.input.game_state == GameState::OfferSelection {
            let mut selected_action = None;
            egui::Window::new("Sacrifice Corpse").show(ctx, |ui| {
                let ident_table = self.game.resources
                    .get::<crate::core::entity::identity::IdentityTable>()
                    .unwrap();
                let mut query =
                    <&crate::core::entity::Inventory>::query().filter(component::<PlayerTag>());
                for inv in query.iter(&self.game.world) {
                    if let Some(action) = crate::ui::widgets::inventory::show_offer_selector(
                        ui,
                        &self.game.world,
                        inv,
                        &self.game.assets.items,
                        &ident_table,
                    ) {
                        selected_action = Some(action);
                    }
                }

                if ui.button("Cancel").clicked() {
                    self.input.game_state = GameState::Normal;
                }
            });

            if let Some(action) = selected_action {
                self.game.resources.insert(Some(action));
                self.input.game_state = GameState::Normal;
            }
        }

        // Engrave Tool Selection Window (#engrave)
        if self.input.game_state == GameState::SelectEngraveTool {
            let mut selected_tool = None;
            egui::Window::new("Engrave Tool").show(ctx, |ui| {
                let ident_table = self.game.resources
                    .get::<crate::core::entity::identity::IdentityTable>()
                    .unwrap();
                let mut query =
                    <&crate::core::entity::Inventory>::query().filter(component::<PlayerTag>());
                for inv in query.iter(&self.game.world) {
                    if let Some(tool) = crate::ui::widgets::inventory::show_engrave_tool_selector(
                        ui,
                        &self.game.world,
                        inv,
                        &self.game.assets.items,
                        &ident_table,
                    ) {
                        selected_tool = Some(tool);
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.input.game_state = GameState::Normal;
                }
            });

            if let Some(tool) = selected_tool {
                self.input.game_state = GameState::EngravingText { tool };
            }
        }

        // Engraving Text Window
        if let GameState::EngravingText { tool } = self.input.game_state {
            egui::Window::new("Engrave Text").show(ctx, |ui| {
                ui.label("What do you want to write in the dust/floor?");
                let edit = ui.text_edit_singleline(&mut self.input.engraving_input);
                if (edit.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)))
                    || ui.button("Write").clicked()
                {
                    let text = self.input.engraving_input.clone();
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    let mut log = self.game.resources.get_mut::<crate::ui::log::GameLog>().unwrap();
                    let mut p_pos = (0, 0);
                    let mut query = <&Position>::query().filter(component::<PlayerTag>());
                    for pos in query.iter(&self.game.world) {
                        p_pos = (pos.x, pos.y);
                    }

                    // World를 SubWorld로 사용
                    let eng_type = crate::core::systems::engrave::get_engrave_type(
                        tool,
                        &self.game.world,
                        &self.game.assets,
                    );
                    crate::core::systems::engrave::engrave_at(
                        &text,
                        eng_type,
                        p_pos,
                        &mut self.game.grid,
                        &mut log,
                        turn,
                    );

                    self.input.engraving_input.clear();
                    self.input.game_state = GameState::Normal;

                    //
                    if let Some(mut t) = self.game.resources.get_mut::<u64>() {
                        *t += 1;
                        log.current_turn = *t;
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.input.engraving_input.clear();
                    self.input.game_state = GameState::Normal;
                }
            });
        }

        // Help Window (?)
        if self.input.game_state == GameState::Help {
            egui::Window::new("Help (?)").show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("NetHack-RS Commands");
                    ui.separator();
                    ui.label("Move: hjklyubn / Arrow Keys");
                    ui.label("Wait: .");
                    ui.label("Pickup: , / g");
                    ui.label("Inventory: i");
                    ui.label("Open/Close Door: o / c");
                    ui.label("Kick: K (Shift+k)");
                    ui.label("Search: s");
                    ui.label("Zap Wand: z");
                    ui.label("Cast Spell: Z (Shift+z)");
                    ui.label("Throw: t");
                    ui.label("Pray: p (Prayers)");
                    ui.add_space(10.0);
                    ui.label("Press ESC to close.");
                });
            });
        }

        //
        if self.ui.layout_settings.show_settings {
            let mut open = self.ui.layout_settings.show_settings;
            egui::Window::new("Settings")
                .open(&mut open)
                .collapsible(true)
                .resizable(false)
                .default_pos([10.0, 40.0])
                .show(ctx, |ui| {
                    ui.label("심볼 세트 선택:");
                    for set_name in self.game.assets.symbols.sets.keys() {
                        if ui
                            .selectable_label(
                                &self.game.assets.symbols.current_set == set_name,
                                set_name,
                            )
                            .clicked()
                        {
                            self.game.assets.symbols.current_set = set_name.clone();
                            self.input.options.current_symbol_set = set_name.clone();
                            self.input.options.save();
                            self.game.resources.insert(self.input.options.clone());
                        }
                    }

                    ui.separator();
                    ui.checkbox(&mut self.input.options.autopickup, "Autopickup");
                    if ui.button("옵션 저장").clicked() {
                        self.input.options.save();
                        self.game.resources.insert(self.input.options.clone());
                    }
                });
            self.ui.layout_settings.show_settings = open;
        }

        // 5. Game Over Overlay
        let game_over_info = if let GameState::GameOver { message } = &self.input.game_state {
            Some(message.clone())
        } else {
            None
        };

        if let Some(message) = game_over_info {
            egui::Window::new("Game Over")
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(
                            eframe::egui::RichText::new("Rest in Peace")
                                .color(egui::Color32::RED)
                                .size(30.0),
                        );
                        ui.add_space(10.0);
                        ui.label(eframe::egui::RichText::new(message).size(18.0));
                        ui.add_space(20.0);

                        let mut xp = 0;
                        let gold = 0; // TODO: Fetch from Player
                        {
                            let mut q = <&crate::core::entity::player::Player>::query()
                                .filter(component::<PlayerTag>());
                            if let Some(p) = q.iter(&self.game.world).next() {
                                xp = p.experience;
                            }
                        }
                        ui.label(format!("Experience Points: {}", xp));
                        ui.label(format!("Gold Collected: {}", gold));
                        ui.add_space(30.0);

                        if ui
                            .button(eframe::egui::RichText::new(" QUIT TO DESKTOP ").size(20.0))
                            .clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        ui.add_space(10.0);
                        if ui
                            .button(eframe::egui::RichText::new(" RESTART GAME ").size(20.0))
                            .clicked()
                        {
                            self.restart_game();
                        }
                    });
                });
        }
    }
}

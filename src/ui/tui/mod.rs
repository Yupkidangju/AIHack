use std::{path::Path, time::Duration};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, MouseEventKind},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::{
    core::{save, GameSession, Observation},
    llm::{
        decision::{decision_log_lines, SuggestedAction},
        narrative::{narrative_log_lines, NarrativeResponse},
    },
};

pub mod config;
pub mod effects;
pub mod input;
pub mod labels;
pub mod layout;
pub mod render_map;
pub mod render_panels;
pub mod theme;
pub mod viewport;

pub use config::UiRuntimeConfig;
pub use effects::{project_event, UiEffectEvent, UiEffectKind};
pub use input::{
    key_to_candidate, keyboard_baseline, map_mouse_event, UiCommandCandidate, UiInputEvent, UiPanel,
};
pub use layout::{compute_layout, LayoutTier, TuiLayout};
pub use theme::UiTheme;
pub use viewport::Viewport;

/// [v0.2.0] Phase 18: TUI adapter 런타임 상태에 debug observation 토글 추가.
pub struct TuiApp {
    pub session: GameSession,
    pub config: UiRuntimeConfig,
    next_effect_id: u64,
    latest_narrative: Option<NarrativeResponse>,
    latest_decision: Option<(SuggestedAction, Option<bool>)>,
    hovered_pos: Option<crate::core::Pos>,
    focused_panel: UiPanel,
    /// [v0.2.0] Phase 18: F9 키로 토글하는 debug observation 패널 표시 상태.
    /// 이 상태는 UI-only이며 core나 snapshot hash에 영향을 주지 않는다.
    pub debug_observation_visible: bool,
    /// [v0.2.0] Phase 19: 현재 표시 중인 자동 라벨 목록.
    /// 이 상태는 UI-only이며 core나 snapshot hash에 영향을 주지 않는다.
    pub active_labels: Vec<labels::AutoLabel>,
    /// [v0.2.0] Phase 19: 마지막으로 라벨을 업데이트한 턴 번호.
    /// 턴이 진행될 때만 새 라벨을 수집한다.
    pub last_label_update_turn: u64,
}

impl TuiApp {
    pub fn new(session: GameSession, config: UiRuntimeConfig) -> Self {
        Self {
            session,
            config,
            next_effect_id: 1,
            latest_narrative: None,
            latest_decision: None,
            hovered_pos: None,
            focused_panel: UiPanel::Map,
            debug_observation_visible: false,
            active_labels: Vec::new(),
            last_label_update_turn: 0,
        }
    }

    pub fn observation(&self) -> Observation {
        self.session.observation()
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), crate::core::error::GameError> {
        save::save_session_to_path(&self.session, path)
    }

    pub fn load_from_path(&mut self, path: &Path) -> Result<(), crate::core::error::GameError> {
        self.session = save::load_session_from_path(path)?;
        Ok(())
    }

    pub fn project_effects(&mut self) -> Vec<UiEffectEvent> {
        let mut out = Vec::new();
        for event in self.session.event_log.iter().rev().take(8).rev() {
            if let Some(effect) =
                effects::project_event_with_config(event, self.next_effect_id, &self.config)
            {
                self.next_effect_id += 1;
                out.push(effect);
            }
        }
        out
    }

    pub fn set_narrative_response(&mut self, response: NarrativeResponse) {
        self.latest_narrative = Some(response);
    }

    pub fn set_decision_suggestion(&mut self, suggestion: SuggestedAction, accepted: Option<bool>) {
        self.latest_decision = Some((suggestion, accepted));
    }

    pub fn narrative_lines(&self) -> Vec<String> {
        self.latest_narrative
            .as_ref()
            .map(narrative_log_lines)
            .unwrap_or_else(|| {
                vec![
                    "Narrative(idle)".to_string(),
                    "narrative not requested".to_string(),
                ]
            })
    }

    pub fn decision_lines(&self) -> Vec<String> {
        self.latest_decision
            .as_ref()
            .map(|(suggestion, accepted)| decision_log_lines(suggestion, *accepted))
            .unwrap_or_else(|| {
                vec![
                    "Decision(idle)".to_string(),
                    "decision support not requested".to_string(),
                ]
            })
    }

    pub fn hovered_pos(&self) -> Option<crate::core::Pos> {
        self.hovered_pos
    }

    pub fn focused_panel(&self) -> UiPanel {
        self.focused_panel
    }

    pub fn theme(&self) -> UiTheme {
        UiTheme::from_high_contrast(self.config.high_contrast)
    }

    pub fn run_single_frame(&mut self, width: u16, height: u16) -> Result<TuiLayout, String> {
        let layout = compute_layout(width, height);
        layout.validate()?;
        Ok(layout)
    }

    pub fn viewport_for_observation(&self, layout: TuiLayout) -> Viewport {
        let observation = self.observation();
        let origin = crate::core::Pos {
            x: observation.player_pos.x - layout.map.width as i16 / 2,
            y: observation.player_pos.y - layout.map.height as i16 / 2,
        };
        Viewport::from_rect(origin, observation.player_pos, layout.map)
    }

    pub fn handle_candidate(
        &mut self,
        candidate: UiCommandCandidate,
        save_path: &Path,
        load_path: &Path,
    ) -> Result<bool, crate::core::error::GameError> {
        match candidate {
            UiCommandCandidate::Command(intent) => {
                let outcome = self.session.submit(intent);
                // [v0.2.0] Phase 19: 턴이 진행되면 새로운 자동 라벨을 수집한다.
                if outcome.turn_advanced {
                    let observation = self.observation();
                    let current_time_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let new_labels = labels::collect_auto_labels(&observation, current_time_ms);
                    // 만료된 라벨 제거 후 새 라벨 추가
                    labels::filter_expired_labels(&mut self.active_labels, current_time_ms);
                    self.active_labels.extend(new_labels);
                    // 우선순위 정렬 후 최대 3개 유지
                    self.active_labels.sort_by_key(|l| l.kind.priority());
                    self.active_labels.truncate(3);
                }
                Ok(false)
            }
            UiCommandCandidate::Inspect(pos) => {
                self.hovered_pos = Some(pos);
                self.focused_panel = UiPanel::Inspect;
                Ok(false)
            }
            UiCommandCandidate::Focus(panel) => {
                self.focused_panel = panel;
                Ok(false)
            }
            UiCommandCandidate::Save => {
                self.save_to_path(save_path)?;
                Ok(false)
            }
            UiCommandCandidate::Load => {
                self.load_from_path(load_path)?;
                Ok(false)
            }
            UiCommandCandidate::Quit => Ok(true),
            UiCommandCandidate::NewRun => {
                self.session = GameSession::new(self.session.meta.seed.wrapping_add(1));
                Ok(false)
            }
        }
    }
}

/// [v0.2.0] Phase 17: RunState에 따라 화면을 분기한다.
/// Title -> CharacterCreation -> Playing <-> GameOver 흐름을 지원한다.
pub fn run_tui(seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = TuiApp::new(GameSession::new(seed), UiRuntimeConfig::default());
    let save_path = std::env::temp_dir().join("aihack-tui-save.json");
    let load_path = save_path.clone();
    let mut idle_ticks = 0u8;
    loop {
        terminal.draw(|frame| {
            let size = frame.area();
            if size.width < 80 || size.height < 28 {
                frame.render_widget(
                    render_panels::TextPanel {
                        title: "TUI",
                        lines: vec!["terminal too small: need at least 80x28".to_string()],
                    },
                    size,
                );
                return;
            }
            match app.session.state {
                crate::core::session::RunState::Title => render_title_screen(frame, size),
                crate::core::session::RunState::CharacterCreation => {
                    render_character_creation_screen(frame, size)
                }
                crate::core::session::RunState::Playing
                | crate::core::session::RunState::AwaitingDirection { .. }
                | crate::core::session::RunState::AwaitingInventorySelection { .. }
                | crate::core::session::RunState::MorePrompt => {
                    render_play_screen(frame, size, &mut app)
                }
                crate::core::session::RunState::GameOver { cause, final_score } => {
                    render_game_over_screen(frame, size, &app, cause, final_score)
                }
            }
        })?;
        let size = terminal.size()?;
        if size.width < 80 || size.height < 28 {
            break;
        }
        if event::poll(Duration::from_millis(50))? {
            idle_ticks = 0;
            let candidate = match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char(ch) => {
                        key_to_candidate_for_state(ch, &app.session.state, &app.observation())
                    }
                    KeyCode::Esc => Some(UiCommandCandidate::Quit),
                    // [v0.2.0] Phase 18: F9 키로 debug observation 패널을 토글한다.
                    // 이 입력은 UI-only이며 core나 snapshot hash에 영향을 주지 않는다.
                    KeyCode::F(9) => {
                        app.debug_observation_visible = !app.debug_observation_visible;
                        None
                    }
                    _ => None,
                },
                Event::Mouse(mouse) => {
                    let layout = compute_layout(size.width, size.height);
                    let viewport = app.viewport_for_observation(layout);
                    let input = match mouse.kind {
                        MouseEventKind::Moved => UiInputEvent::MouseHover {
                            column: mouse.column,
                            row: mouse.row,
                        },
                        MouseEventKind::Down(_) => UiInputEvent::MouseClick {
                            column: mouse.column,
                            row: mouse.row,
                        },
                        _ => UiInputEvent::FocusPanel(UiPanel::Map),
                    };
                    map_mouse_event_for_state(input, layout, viewport, &app)
                }
                _ => None,
            };
            if let Some(candidate) = candidate {
                if app.handle_candidate(candidate, &save_path, &load_path)? {
                    break;
                }
            }
        } else {
            idle_ticks = idle_ticks.saturating_add(1);
            if idle_ticks >= 2 {
                break;
            }
        }
    }
    let backend = terminal.backend_mut();
    backend.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    backend.execute(LeaveAlternateScreen)?;
    Ok(())
}

fn render_title_screen(frame: &mut ratatui::Frame, size: Rect) {
    frame.render_widget(
        render_panels::TextPanel {
            title: "AIHack",
            lines: render_panels::title_lines(),
        },
        size,
    );
}

fn render_character_creation_screen(frame: &mut ratatui::Frame, size: Rect) {
    frame.render_widget(
        render_panels::TextPanel {
            title: "Character Creation",
            lines: render_panels::character_creation_lines(),
        },
        size,
    );
}

fn render_play_screen(frame: &mut ratatui::Frame, _size: Rect, app: &mut TuiApp) {
    let layout = compute_layout(_size.width, _size.height);
    let observation = app.observation();
    let viewport = app.viewport_for_observation(layout);

    // [v0.2.0] Phase 19: 만료된 라벨 필터링
    let current_time_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    labels::filter_expired_labels(&mut app.active_labels, current_time_ms);

    // AwaitingDirection, AwaitingInventorySelection, MorePrompt 상태일 때 상태 메시지 오버레이
    let state_overlay = match app.session.state {
        crate::core::session::RunState::AwaitingDirection { action } => {
            let action_name = match action {
                crate::core::action::DirectionalAction::Open => "open",
                crate::core::action::DirectionalAction::Close => "close",
                crate::core::action::DirectionalAction::Kick => "kick",
            };
            Some(render_panels::awaiting_direction_lines(action_name))
        }
        crate::core::session::RunState::AwaitingInventorySelection { action } => {
            let action_name = match action {
                crate::core::action::InventoryAction::Drop => "drop",
                crate::core::action::InventoryAction::Wield => "wield",
                crate::core::action::InventoryAction::Wear => "wear",
                crate::core::action::InventoryAction::Quaff => "quaff",
                crate::core::action::InventoryAction::Read => "read",
            };
            Some(render_panels::awaiting_inventory_lines(action_name))
        }
        crate::core::session::RunState::MorePrompt => Some(render_panels::more_prompt_lines()),
        _ => None,
    };

    frame.render_widget(
        render_map::MapWidget {
            observation: &observation,
            viewport,
            labels: &app.active_labels,
        },
        layout.map,
    );
    frame.render_widget(
        render_panels::TextPanel {
            title: "STATUS",
            lines: render_panels::status_lines(&observation),
        },
        layout.status,
    );
    frame.render_widget(
        render_panels::TextPanel {
            title: "COMMANDS",
            lines: render_panels::command_lines(&observation, app.focused_panel()),
        },
        layout.command,
    );
    frame.render_widget(
        render_panels::TextPanel {
            title: "LOG",
            lines: render_panels::log_lines(&observation, &app.narrative_lines()),
        },
        layout.log,
    );
    frame.render_widget(
        render_panels::TextPanel {
            title: "INSPECT",
            lines: render_panels::inspect_lines(
                &observation,
                app.hovered_pos(),
                app.focused_panel(),
                &app.decision_lines(),
            ),
        },
        layout.inspect,
    );
    // [v0.2.0] Phase 18: F9 토글 debug observation 패널.
    // 이 패널은 UI-only이며 snapshot hash에 영향을 주지 않는다.
    if app.debug_observation_visible {
        let debug_lines = render_panels::debug_observation_lines(&observation);
        let debug_height = debug_lines.len() as u16 + 2;
        // 80x28에서도 표시되도록 맵 우측 상단에 작게 배치
        let debug_area = Rect {
            x: layout.map.x + layout.map.width.saturating_sub(40),
            y: layout.map.y,
            width: 40,
            height: debug_height.min(layout.map.height),
        };
        frame.render_widget(
            render_panels::TextPanel {
                title: "DEBUG OBS",
                lines: debug_lines,
            },
            debug_area,
        );
    } else if let Some(debug) = layout.debug {
        // roomy layout(120x36+)에서 기본 debug 패널 표시
        frame.render_widget(
            render_panels::TextPanel {
                title: "DEBUG",
                lines: vec![format!("effects {}", app.project_effects().len())],
            },
            debug,
        );
    }

    // 상태 오버레이 표시 (하단 로그 영역 위에 작게)
    if let Some(lines) = state_overlay {
        let overlay_height = lines.len() as u16 + 2;
        let overlay_area = Rect {
            x: layout.log.x,
            y: layout.log.y + layout.log.height.saturating_sub(overlay_height),
            width: layout.log.width,
            height: overlay_height.min(layout.log.height),
        };
        frame.render_widget(
            render_panels::TextPanel {
                title: "STATE",
                lines,
            },
            overlay_area,
        );
    }
}

fn render_game_over_screen(
    frame: &mut ratatui::Frame,
    size: Rect,
    app: &TuiApp,
    cause: crate::domain::combat::DeathCause,
    final_score: i32,
) {
    let cause_text = match cause {
        crate::domain::combat::DeathCause::Combat { attacker } => {
            format!("Killed by entity {:?}", attacker.0)
        }
        crate::domain::combat::DeathCause::Trap { trap } => {
            format!("Killed by {:?}", trap)
        }
    };
    let observation = app.observation();
    let lines = render_panels::game_over_lines(
        &cause_text,
        app.session.turn,
        observation.current_level.depth,
        app.session.world.kill_count,
        final_score,
        app.session.meta.seed,
    );
    frame.render_widget(
        render_panels::TextPanel {
            title: "GAME OVER",
            lines,
        },
        size,
    );
}

/// [v0.2.0] Phase 17: RunState에 따라 키 입력을 다른 후보로 매핑한다.
fn key_to_candidate_for_state(
    ch: char,
    state: &crate::core::session::RunState,
    observation: &Observation,
) -> Option<UiCommandCandidate> {
    use crate::core::session::RunState;
    match state {
        RunState::Title => match ch {
            '\n' | '\r' => Some(UiCommandCandidate::Command(
                crate::core::action::CommandIntent::Wait,
            )),
            'q' | 'Q' => Some(UiCommandCandidate::Quit),
            _ => None,
        },
        RunState::CharacterCreation => match ch {
            '\n' | '\r' => Some(UiCommandCandidate::Command(
                crate::core::action::CommandIntent::Wait,
            )),
            'q' | 'Q' => Some(UiCommandCandidate::Quit),
            _ => None,
        },
        RunState::GameOver { .. } => match ch {
            'n' | 'N' => Some(UiCommandCandidate::NewRun),
            'q' | 'Q' => Some(UiCommandCandidate::Quit),
            _ => None,
        },
        RunState::AwaitingDirection { .. }
        | RunState::AwaitingInventorySelection { .. }
        | RunState::MorePrompt
        | RunState::Playing => key_to_candidate(ch, observation),
    }
}

/// [v0.2.0] Phase 17: RunState에 따라 마우스 입력을 처리한다.
fn map_mouse_event_for_state(
    event: UiInputEvent,
    layout: TuiLayout,
    viewport: Viewport,
    app: &TuiApp,
) -> Option<UiCommandCandidate> {
    use crate::core::session::RunState;
    match app.session.state {
        RunState::Title | RunState::CharacterCreation | RunState::GameOver { .. } => None,
        _ => map_mouse_event(event, layout, viewport, &app.observation()),
    }
}

pub fn runtime_smoke() -> Result<Rect, String> {
    let mut app = TuiApp::new(GameSession::new(42), UiRuntimeConfig::default());
    let layout = app.run_single_frame(100, 32)?;
    Ok(layout.map)
}

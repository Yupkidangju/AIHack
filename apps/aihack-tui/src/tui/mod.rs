use std::{
    path::Path,
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, MouseEventKind},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use aihack_ai_contract::{ClientRevision, Observation, RunState};
use aihack_runtime::{save, GameClient, GameError, GameSession};

use crate::llm::{
    config::{validate_user_text, LlmRequestKind, LocalLlmConfig},
    decision::{
        decision_log_lines, validate_decision_payload, DecisionSource, SuggestedAction,
        ValidatedDecision,
    },
    narrative::{narrative_log_lines, NarrativeResponse, NarrativeSource},
    service::{LlmPayload, LlmRequestInput, LlmResponseEnvelope, LocalLlmPort, LocalLlmService},
    soft_adjudication::{
        fallback_soft_adjudication, soft_adjudication_lines, SoftAdjudicationResponse,
        SoftAdjudicationSource,
    },
    transport::LlmResponseError,
};
use aihack_llm::worker::{LlmEnqueueError, RequestId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmUiStatus {
    Disabled,
    Connecting,
    Ready,
    Pending {
        kind: LlmRequestKind,
        request_id: String,
    },
    Busy,
    Timeout {
        kind: LlmRequestKind,
    },
    Unavailable,
    Invalid,
    Stale,
}

struct OutstandingLlmRequest {
    request_id: RequestId,
    revision: ClientRevision,
    kind: LlmRequestKind,
    ignored: bool,
}

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
    key_to_candidate, keyboard_baseline, llm_footer_click_candidate, map_mouse_event,
    UiCommandCandidate, UiInputEvent, UiPanel,
};
pub use layout::{compute_layout, LayoutTier, TuiLayout};
pub use theme::UiTheme;
pub use viewport::Viewport;

/// [v0.2.0] Phase 18: TUI adapter 런타임 상태에 debug observation 토글 추가.
pub trait TuiClient: GameClient {
    fn save_to_path(&self, path: &Path) -> Result<(), GameError>;
    fn load_from_path(&mut self, path: &Path) -> Result<(), GameError>;
    fn start_new_run(&mut self) -> Result<(), GameError>;
    fn kill_count(&self) -> u32;
}

impl TuiClient for GameSession {
    fn save_to_path(&self, path: &Path) -> Result<(), GameError> {
        save::save_session_to_path(self, path)
    }

    fn load_from_path(&mut self, path: &Path) -> Result<(), GameError> {
        *self = save::load_session_from_path(path)?;
        Ok(())
    }

    fn start_new_run(&mut self) -> Result<(), GameError> {
        *self = GameSession::try_new(self.observation().seed.wrapping_add(1))?;
        Ok(())
    }

    fn kill_count(&self) -> u32 {
        self.world().kill_count()
    }
}

pub struct TuiApp {
    client: Box<dyn TuiClient>,
    pub config: UiRuntimeConfig,
    next_effect_id: u64,
    latest_narrative: Option<NarrativeResponse>,
    latest_decision: Option<(SuggestedAction, Option<bool>)>,
    latest_soft_adjudication: Option<SoftAdjudicationResponse>,
    llm_enabled: bool,
    llm_status: LlmUiStatus,
    soft_input: Option<String>,
    queued_llm_request: Option<LlmRequestKind>,
    outstanding_llm_request: Option<OutstandingLlmRequest>,
    last_llm_request: Option<LlmRequestKind>,
    validated_decision: Option<ValidatedDecision>,
    last_llm_enqueue: [Option<Instant>; 3],
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
    pub fn new(client: impl TuiClient + 'static, config: UiRuntimeConfig) -> Self {
        Self::new_with_llm_enabled(client, config, false)
    }

    pub fn new_with_llm_enabled(
        client: impl TuiClient + 'static,
        config: UiRuntimeConfig,
        llm_enabled: bool,
    ) -> Self {
        Self {
            client: Box::new(client),
            config,
            next_effect_id: 1,
            latest_narrative: None,
            latest_decision: None,
            latest_soft_adjudication: None,
            llm_enabled,
            llm_status: if llm_enabled {
                LlmUiStatus::Ready
            } else {
                LlmUiStatus::Disabled
            },
            soft_input: None,
            queued_llm_request: None,
            outstanding_llm_request: None,
            last_llm_request: None,
            validated_decision: None,
            last_llm_enqueue: [None; 3],
            hovered_pos: None,
            focused_panel: UiPanel::Map,
            debug_observation_visible: false,
            active_labels: Vec::new(),
            last_label_update_turn: 0,
        }
    }

    pub fn llm_status(&self) -> &LlmUiStatus {
        &self.llm_status
    }

    pub fn soft_input(&self) -> Option<&str> {
        self.soft_input.as_deref()
    }

    pub fn take_llm_request(&mut self) -> Option<LlmRequestKind> {
        self.queued_llm_request.take()
    }

    pub fn dispatch_llm_request(&mut self, port: &dyn LocalLlmPort) {
        let Some(kind) = self.take_llm_request() else {
            return;
        };
        if self.outstanding_llm_request.is_some() {
            self.llm_status = LlmUiStatus::Busy;
            return;
        }
        let revision = self.revision();
        let enqueue_result = port.enqueue(LlmRequestInput {
            revision: revision.clone(),
            observation: self.observation(),
            kind: kind.clone(),
        });
        self.last_llm_enqueue[llm_kind_index(&kind)] = Some(Instant::now());
        self.last_llm_request = Some(kind.clone());
        match enqueue_result {
            Ok(request_id) => {
                self.llm_status = LlmUiStatus::Pending {
                    kind: kind.clone(),
                    request_id: request_id.as_str().to_string(),
                };
                self.outstanding_llm_request = Some(OutstandingLlmRequest {
                    request_id,
                    revision,
                    kind,
                    ignored: false,
                });
            }
            Err(LlmEnqueueError::Disabled) => self.llm_status = LlmUiStatus::Disabled,
            Err(LlmEnqueueError::Busy { .. }) => self.llm_status = LlmUiStatus::Busy,
            Err(LlmEnqueueError::InvalidInput { .. }) => self.llm_status = LlmUiStatus::Invalid,
            Err(LlmEnqueueError::WorkerStopped) => self.llm_status = LlmUiStatus::Unavailable,
            Err(_) => self.llm_status = LlmUiStatus::Invalid,
        }
    }

    pub fn poll_llm_response(&mut self, port: &dyn LocalLlmPort) {
        while let Some(envelope) = port.try_recv() {
            self.accept_llm_response(envelope);
        }
    }

    pub fn accept_llm_response(&mut self, envelope: LlmResponseEnvelope) {
        let Some(outstanding) = self.outstanding_llm_request.as_ref() else {
            self.llm_status = LlmUiStatus::Invalid;
            return;
        };
        if envelope.request_id != outstanding.request_id {
            self.llm_status = LlmUiStatus::Invalid;
            return;
        }
        let outstanding = self.outstanding_llm_request.take().expect("checked above");
        if outstanding.ignored {
            self.llm_status = LlmUiStatus::Ready;
            return;
        }
        let current_revision = self.revision();
        if envelope.revision != outstanding.revision || envelope.revision != current_revision {
            self.llm_status = LlmUiStatus::Stale;
            return;
        }
        match envelope.result {
            Ok(payload) => self.accept_llm_payload(outstanding.kind, payload, current_revision),
            Err(error) => self.accept_llm_error(&outstanding.kind, error),
        }
    }

    fn accept_llm_payload(
        &mut self,
        expected_kind: LlmRequestKind,
        payload: LlmPayload,
        current_revision: ClientRevision,
    ) {
        match (expected_kind, payload) {
            (LlmRequestKind::Narrative, LlmPayload::Narrative(payload)) => {
                self.set_narrative_response(NarrativeResponse {
                    text: payload.text,
                    source: NarrativeSource::Provider,
                    timed_out: false,
                });
                self.llm_status = LlmUiStatus::Ready;
            }
            (LlmRequestKind::Decision, LlmPayload::Decision(payload)) => {
                match validate_decision_payload(
                    current_revision.clone(),
                    payload,
                    &current_revision,
                    &self.observation().action_space,
                ) {
                    Ok(validated) => {
                        self.set_decision_suggestion(
                            SuggestedAction {
                                action: validated.action(),
                                rationale: validated.rationale().to_string(),
                                source: DecisionSource::Provider,
                            },
                            None,
                        );
                        self.validated_decision = Some(validated);
                        self.llm_status = LlmUiStatus::Ready;
                    }
                    Err(error) => self.accept_llm_error(&LlmRequestKind::Decision, error),
                }
            }
            (LlmRequestKind::SoftAdjudication { .. }, LlmPayload::SoftAdjudication(payload)) => {
                self.set_soft_adjudication(SoftAdjudicationResponse {
                    payload,
                    source: SoftAdjudicationSource::Provider,
                });
                self.llm_status = LlmUiStatus::Ready;
            }
            _ => self.llm_status = LlmUiStatus::Invalid,
        }
    }

    fn accept_llm_error(&mut self, kind: &LlmRequestKind, error: LlmResponseError) {
        let timed_out = matches!(error, LlmResponseError::Timeout);
        self.llm_status = match error {
            LlmResponseError::Disabled => LlmUiStatus::Disabled,
            LlmResponseError::Timeout => LlmUiStatus::Timeout { kind: kind.clone() },
            LlmResponseError::Unavailable | LlmResponseError::HttpStatus { .. } => {
                LlmUiStatus::Unavailable
            }
            LlmResponseError::Stale => LlmUiStatus::Stale,
            _ => LlmUiStatus::Invalid,
        };
        match kind {
            LlmRequestKind::Narrative => self.set_narrative_response(NarrativeResponse {
                text: "Local narrator unavailable.".to_string(),
                source: NarrativeSource::Fallback,
                timed_out,
            }),
            LlmRequestKind::SoftAdjudication { .. } => {
                self.set_soft_adjudication(fallback_soft_adjudication())
            }
            LlmRequestKind::Decision => {}
        }
    }

    fn queue_llm_request(&mut self, kind: LlmRequestKind) {
        if !self.llm_enabled {
            self.llm_status = LlmUiStatus::Disabled;
            return;
        }
        if self.last_llm_enqueue[llm_kind_index(&kind)]
            .is_some_and(|instant| instant.elapsed() < Duration::from_millis(250))
        {
            return;
        }
        if self.queued_llm_request.is_none() && self.outstanding_llm_request.is_none() {
            self.queued_llm_request = Some(kind);
            self.llm_status = LlmUiStatus::Connecting;
        }
    }

    pub fn observation(&self) -> Observation {
        self.client.observation()
    }

    pub fn revision(&self) -> ClientRevision {
        self.client.revision()
    }

    pub fn run_state(&self) -> RunState {
        self.client.run_state()
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), GameError> {
        self.client.save_to_path(path)
    }

    pub fn load_from_path(&mut self, path: &Path) -> Result<(), GameError> {
        self.client.load_from_path(path)
    }

    pub fn project_effects(&mut self) -> Vec<UiEffectEvent> {
        let mut out = Vec::new();
        for event in &self.observation().last_events {
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

    pub fn set_soft_adjudication(&mut self, response: SoftAdjudicationResponse) {
        self.latest_soft_adjudication = Some(response);
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

    pub fn soft_adjudication_lines(&self) -> Vec<String> {
        self.latest_soft_adjudication
            .as_ref()
            .map(soft_adjudication_lines)
            .unwrap_or_else(|| {
                vec![
                    "Soft judgment · idle".to_string(),
                    "soft adjudication not requested".to_string(),
                ]
            })
    }

    fn llm_result_lines(&self) -> Vec<String> {
        if let Some(response) = self.latest_soft_adjudication.as_ref() {
            soft_adjudication_lines(response)
        } else if let Some((suggestion, accepted)) = self.latest_decision.as_ref() {
            decision_log_lines(suggestion, *accepted)
        } else {
            Vec::new()
        }
    }

    fn has_llm_result(&self) -> bool {
        self.latest_narrative.is_some()
            || self.latest_decision.is_some()
            || self.latest_soft_adjudication.is_some()
            || self.outstanding_llm_request.is_some()
    }

    fn dismiss_llm_result(&mut self) {
        self.latest_narrative = None;
        self.latest_decision = None;
        self.latest_soft_adjudication = None;
        self.validated_decision = None;
        if let Some(outstanding) = self.outstanding_llm_request.as_mut() {
            outstanding.ignored = true;
        }
        self.llm_status = if self.llm_enabled {
            LlmUiStatus::Ready
        } else {
            LlmUiStatus::Disabled
        };
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
    ) -> Result<bool, GameError> {
        match candidate {
            UiCommandCandidate::Command(intent) => {
                let outcome = self.client.submit(intent);
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
                self.client.start_new_run()?;
                self.dismiss_llm_result();
                self.soft_input = None;
                self.queued_llm_request = None;
                Ok(false)
            }
            UiCommandCandidate::DismissLlmResult => {
                self.dismiss_llm_result();
                Ok(false)
            }
            UiCommandCandidate::LlmNarrative => {
                self.queue_llm_request(LlmRequestKind::Narrative);
                Ok(false)
            }
            UiCommandCandidate::LlmSuggest => {
                self.queue_llm_request(LlmRequestKind::Decision);
                Ok(false)
            }
            UiCommandCandidate::LlmJudge => {
                if self.llm_enabled {
                    self.soft_input = Some(String::new());
                } else {
                    self.llm_status = LlmUiStatus::Disabled;
                }
                Ok(false)
            }
            UiCommandCandidate::LlmInput(character) => {
                if let Some(input) = self.soft_input.as_mut() {
                    if !character.is_control() && input.chars().count() < 240 {
                        input.push(character);
                    }
                }
                Ok(false)
            }
            UiCommandCandidate::LlmBackspace => {
                if let Some(input) = self.soft_input.as_mut() {
                    input.pop();
                }
                Ok(false)
            }
            UiCommandCandidate::LlmSubmitInput => {
                if let Some(input) = self.soft_input.as_ref() {
                    match validate_user_text(input) {
                        Ok(user_text) => {
                            self.soft_input = None;
                            self.queue_llm_request(LlmRequestKind::SoftAdjudication { user_text });
                        }
                        Err(_) => self.llm_status = LlmUiStatus::Invalid,
                    }
                }
                Ok(false)
            }
            UiCommandCandidate::LlmCancelInput => {
                self.soft_input = None;
                self.llm_status = if self.llm_enabled {
                    LlmUiStatus::Ready
                } else {
                    LlmUiStatus::Disabled
                };
                Ok(false)
            }
            UiCommandCandidate::LlmApply => {
                let Some(validated) = self.validated_decision.take() else {
                    return Ok(false);
                };
                if validated.revision() != &self.revision()
                    || !self
                        .observation()
                        .action_space
                        .commands
                        .contains(&validated.action())
                {
                    self.llm_status = LlmUiStatus::Stale;
                    return Ok(false);
                }
                if let aihack_ai_contract::ActionIntent::Command(command) = validated.action() {
                    let outcome = self.client.submit(command);
                    if let Some((suggestion, accepted)) = self.latest_decision.as_mut() {
                        *accepted = Some(outcome.accepted);
                        let _ = suggestion;
                    }
                }
                Ok(false)
            }
            UiCommandCandidate::LlmRetry => {
                if self.outstanding_llm_request.is_none() {
                    if let Some(kind) = self.last_llm_request.clone() {
                        self.queue_llm_request(kind);
                    }
                }
                Ok(false)
            }
        }
    }
}

fn llm_kind_index(kind: &LlmRequestKind) -> usize {
    match kind {
        LlmRequestKind::Narrative => 0,
        LlmRequestKind::Decision => 1,
        LlmRequestKind::SoftAdjudication { .. } => 2,
    }
}

/// [v0.2.0] Phase 17: RunState에 따라 화면을 분기한다.
/// Title -> CharacterCreation -> Playing <-> GameOver 흐름을 지원한다.
pub fn run_tui(seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    let llm_config = LocalLlmConfig::from_env().map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid local LLM configuration: {error:?}"),
        )
    })?;
    let llm_enabled = llm_config.enabled();
    let service = LocalLlmService::from_config(llm_config)
        .map_err(|error| std::io::Error::other(format!("local LLM startup failed: {error:?}")))?;
    run_tui_with_service(seed, service, llm_enabled)
}

fn run_tui_with_service(
    seed: u64,
    mut llm_service: LocalLlmService,
    llm_enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = TuiApp::new_with_llm_enabled(
        GameSession::try_new(seed)?,
        UiRuntimeConfig::default(),
        llm_enabled,
    );
    let save_path = std::env::temp_dir().join("aihack-tui-save.json");
    let load_path = save_path.clone();
    let run_result = (|| -> Result<(), Box<dyn std::error::Error>> {
        loop {
            app.poll_llm_response(&llm_service);
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
                match app.run_state() {
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
                let candidate = match event::read()? {
                    Event::Key(key) if app.soft_input().is_some() => match key.code {
                        KeyCode::Enter => Some(UiCommandCandidate::LlmSubmitInput),
                        KeyCode::Backspace => Some(UiCommandCandidate::LlmBackspace),
                        KeyCode::Esc => Some(UiCommandCandidate::LlmCancelInput),
                        KeyCode::Char(character) => Some(UiCommandCandidate::LlmInput(character)),
                        _ => None,
                    },
                    Event::Key(key) => match key.code {
                        KeyCode::Char('N') if app.has_llm_result() => {
                            Some(UiCommandCandidate::DismissLlmResult)
                        }
                        KeyCode::Char(ch) => {
                            key_to_candidate_for_state(ch, &app.run_state(), &app.observation())
                        }
                        KeyCode::Esc if app.has_llm_result() => {
                            Some(UiCommandCandidate::DismissLlmResult)
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
                    app.dispatch_llm_request(&llm_service);
                }
            }
        }
        Ok(())
    })();

    // 외부 응답이 지연되어도 terminal은 먼저 정상 상태로 복원한다.
    let backend = terminal.backend_mut();
    let restore_result = (|| -> Result<(), Box<dyn std::error::Error>> {
        backend.execute(cursor::Show)?;
        terminal::disable_raw_mode()?;
        backend.execute(LeaveAlternateScreen)?;
        Ok(())
    })();
    let _worker_stopped = llm_service.shutdown_with_grace(Duration::from_millis(250));

    run_result?;
    restore_result?;
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
    let state_overlay = match app.run_state() {
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
    let mut status_lines = render_panels::status_lines(&observation);
    status_lines.extend(
        render_panels::llm_status_lines(app.llm_status())
            .into_iter()
            .take(1),
    );
    frame.render_widget(
        render_panels::TextPanel {
            title: "STATUS",
            lines: status_lines,
        },
        layout.status,
    );
    let mut command_lines = render_panels::command_lines(&observation, app.focused_panel());
    command_lines[1] = render_panels::llm_footer_line(
        app.llm_status(),
        app.validated_decision.is_some(),
        app.has_llm_result(),
    );
    frame.render_widget(
        render_panels::TextPanel {
            title: "COMMANDS",
            lines: command_lines,
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
                &app.llm_result_lines(),
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

    if let Some(input) = app.soft_input() {
        frame.render_widget(
            render_panels::TextPanel {
                title: "SOFT JUDGMENT INPUT",
                lines: render_panels::soft_input_lines(input),
            },
            layout.inspect,
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
        observation.turn,
        observation.current_level.depth,
        app.client.kill_count(),
        final_score,
        observation.seed,
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
    match app.run_state() {
        RunState::Title | RunState::CharacterCreation | RunState::GameOver { .. } => None,
        _ => {
            if let UiInputEvent::MouseClick { column, row } = event {
                let footer = render_panels::llm_footer_line(
                    app.llm_status(),
                    app.validated_decision.is_some(),
                    app.has_llm_result(),
                );
                if let Some(candidate) =
                    input::llm_footer_click_candidate(layout.command, column, row, &footer)
                {
                    return Some(candidate);
                }
            }
            map_mouse_event(event, layout, viewport, &app.observation())
        }
    }
}

pub fn runtime_smoke() -> Result<Rect, String> {
    let mut app = TuiApp::new(
        GameSession::try_new(42).map_err(|error| error.to_string())?,
        UiRuntimeConfig::default(),
    );
    let layout = app.run_single_frame(100, 32)?;
    Ok(layout.map)
}

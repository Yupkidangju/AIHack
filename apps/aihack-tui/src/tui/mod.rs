use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        MouseEventKind,
    },
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use aihack_ai_contract::{ClientRevision, Observation, RunState};
use aihack_runtime::{save::ArtifactStore, GameClient, GameError, GameSession};

use crate::llm::{
    config::{validate_user_text, LlmRequestKind, LocalLlmConfig},
    decision::{
        decision_log_lines, validate_decision_payload, DecisionSource, SuggestedAction,
        ValidatedDecision,
    },
    narrative::{narrative_log_lines, NarrativeResponse, NarrativeSource},
    service::{
        validate_response_schema, LlmPayload, LlmRequestInput, LlmResponseEnvelope, LocalLlmPort,
        LocalLlmService,
    },
    soft_adjudication::{
        fallback_soft_adjudication, soft_adjudication_lines, SoftAdjudicationResponse,
        SoftAdjudicationSource,
    },
    transport::LlmResponseError,
};
use aihack_llm::worker::{LlmEnqueueError, RequestId};
mod play_menu;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageOperation {
    Save,
    Load,
    NewRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiOverlay {
    None,
    Inventory,
    StorageError { operation: StorageOperation },
}

/// UI cooldown 검증을 wall clock과 분리하는 monotonic clock 경계다.
pub trait UiClock: Send + Sync {
    fn now(&self) -> Duration;
}

struct SystemUiClock {
    started: Instant,
}

impl Default for SystemUiClock {
    fn default() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl UiClock for SystemUiClock {
    fn now(&self) -> Duration {
        self.started.elapsed()
    }
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
    InspectPresentation, UiCommandCandidate, UiInputEvent, UiPanel,
};
pub use layout::{compute_layout, LayoutTier, TuiLayout};
pub use theme::UiTheme;
pub use viewport::Viewport;

pub const MIN_TERMINAL_WIDTH: u16 = 60;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;
const TRANSITION_GESTURE_QUIET_WINDOW: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyGesture {
    code: KeyCode,
}

impl KeyGesture {
    fn from_event(event: &KeyEvent) -> Self {
        Self { code: event.code }
    }

    fn matches(&self, event: &KeyEvent) -> bool {
        logical_key_codes_match(self.code, event.code)
    }
}

fn logical_key_codes_match(left: KeyCode, right: KeyCode) -> bool {
    if left == right {
        return true;
    }
    matches!(
        (left, right),
        (
            KeyCode::Enter | KeyCode::Char('\r' | '\n'),
            KeyCode::Enter | KeyCode::Char('\r' | '\n')
        ) | (
            KeyCode::Esc | KeyCode::Char('\u{1b}'),
            KeyCode::Esc | KeyCode::Char('\u{1b}')
        ) | (
            KeyCode::Backspace | KeyCode::Char('\u{7f}'),
            KeyCode::Backspace | KeyCode::Char('\u{7f}')
        )
    )
}

#[derive(Debug, Default)]
struct TransitionGestureGate {
    blocked: Option<KeyGesture>,
    consecutive_idle_polls: u8,
    blocked_until: Option<Duration>,
}

pub fn terminal_size_supported(width: u16, height: u16) -> bool {
    width >= MIN_TERMINAL_WIDTH && height >= MIN_TERMINAL_HEIGHT
}

/// TUI adapter가 저장·새 run과 runtime 조회에 사용하는 경계다.
pub trait TuiClient: GameClient {
    fn save_to_store(&self, store: &ArtifactStore, path: &Path) -> Result<(), GameError>;
    fn load_from_store(&mut self, store: &ArtifactStore, path: &Path) -> Result<(), GameError>;
    fn start_new_run(&mut self) -> Result<(), GameError>;
    fn back_to_title(&mut self) -> Result<(), GameError>;
    fn kill_count(&self) -> u32;
}

impl TuiClient for GameSession {
    fn save_to_store(&self, store: &ArtifactStore, path: &Path) -> Result<(), GameError> {
        store.save_session(self, path)
    }

    fn load_from_store(&mut self, store: &ArtifactStore, path: &Path) -> Result<(), GameError> {
        *self = store.load_session(path)?;
        Ok(())
    }

    fn start_new_run(&mut self) -> Result<(), GameError> {
        *self = GameSession::try_new(self.observation().seed.wrapping_add(1))?;
        Ok(())
    }

    fn back_to_title(&mut self) -> Result<(), GameError> {
        *self = GameSession::try_new(self.observation().seed)?;
        Ok(())
    }

    fn kill_count(&self) -> u32 {
        self.world().kill_count()
    }
}

struct TuiStorage {
    _directory: Option<tempfile::TempDir>,
    store: ArtifactStore,
    quick_save: PathBuf,
}

impl TuiStorage {
    fn ephemeral() -> Result<Self, GameError> {
        let directory = tempfile::tempdir().map_err(|error| GameError::Io(error.to_string()))?;
        let store = ArtifactStore::open(directory.path())?;
        Ok(Self {
            _directory: Some(directory),
            store,
            quick_save: PathBuf::from("quick-save.json"),
        })
    }
}

pub struct TuiApp {
    menu: Option<play_menu::PlayMenu>,
    client: Box<dyn TuiClient>,
    storage: Option<TuiStorage>,
    overlay: UiOverlay,
    clock: Arc<dyn UiClock>,
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
    ignored_request_ids: Vec<RequestId>,
    last_llm_request: Option<LlmRequestKind>,
    validated_decision: Option<ValidatedDecision>,
    last_llm_enqueue: [Option<Duration>; 3],
    hovered_pos: Option<crate::core::Pos>,
    focused_panel: UiPanel,
    /// F9 키로 토글하는 debug observation 패널 표시 상태다.
    /// 이 상태는 UI-only이며 core나 snapshot hash에 영향을 주지 않는다.
    pub debug_observation_visible: bool,
    /// 현재 표시 중인 자동 라벨 목록이다.
    /// 이 상태는 UI-only이며 core나 snapshot hash에 영향을 주지 않는다.
    pub active_labels: Vec<labels::AutoLabel>,
    /// 마지막으로 라벨을 업데이트한 턴 번호다.
    /// 턴이 진행될 때만 새 라벨을 수집한다.
    pub last_label_update_turn: u64,
    transition_gesture: TransitionGestureGate,
}

impl TuiApp {
    /// production과 별도 프로세스 재개가 공유하는 지속 저장 경계다.
    pub fn with_save_directory(mut self, directory: &Path) -> Result<Self, GameError> {
        self.storage = Some(TuiStorage {
            _directory: None,
            store: ArtifactStore::open(directory)?,
            quick_save: PathBuf::from("quick-save.json"),
        });
        self.overlay = UiOverlay::None;
        Ok(self)
    }
    pub fn new(client: impl TuiClient + 'static, config: UiRuntimeConfig) -> Self {
        Self::new_with_llm_enabled(client, config, false)
    }

    pub fn new_with_llm_enabled(
        client: impl TuiClient + 'static,
        config: UiRuntimeConfig,
        llm_enabled: bool,
    ) -> Self {
        Self::new_with_llm_enabled_and_clock(
            client,
            config,
            llm_enabled,
            Arc::new(SystemUiClock::default()),
        )
    }

    /// 결정적 cooldown test가 주입 clock을 사용할 수 있는 생성 경계다.
    pub fn new_with_llm_enabled_and_clock(
        client: impl TuiClient + 'static,
        config: UiRuntimeConfig,
        llm_enabled: bool,
        clock: Arc<dyn UiClock>,
    ) -> Self {
        let storage = TuiStorage::ephemeral().ok();
        let overlay = if storage.is_some() {
            UiOverlay::None
        } else {
            UiOverlay::StorageError {
                operation: StorageOperation::Save,
            }
        };
        Self {
            client: Box::new(client),
            menu: None,
            storage,
            overlay,
            clock,
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
            ignored_request_ids: Vec::new(),
            last_llm_request: None,
            validated_decision: None,
            last_llm_enqueue: [None; 3],
            hovered_pos: None,
            focused_panel: UiPanel::Map,
            debug_observation_visible: false,
            active_labels: Vec::new(),
            last_label_update_turn: 0,
            transition_gesture: TransitionGestureGate::default(),
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
        let observation = self.observation();
        let enqueue_result = port.enqueue(LlmRequestInput::from_observation(
            revision.clone(),
            &observation,
            kind.clone(),
        ));
        let Some(kind_index) = llm_kind_index(&kind) else {
            self.llm_status = LlmUiStatus::Invalid;
            return;
        };
        self.last_llm_enqueue[kind_index] = Some(self.clock.now());
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
        if let Some(index) = self
            .ignored_request_ids
            .iter()
            .position(|request_id| request_id == &envelope.request_id)
        {
            self.ignored_request_ids.remove(index);
            return;
        }
        if validate_response_schema(envelope.schema_version).is_err() {
            self.outstanding_llm_request.take();
            self.llm_status = LlmUiStatus::Invalid;
            return;
        }
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
            _ => {}
        }
    }

    fn queue_llm_request(&mut self, kind: LlmRequestKind) {
        if !self.llm_enabled {
            self.llm_status = LlmUiStatus::Disabled;
            return;
        }
        let Some(kind_index) = llm_kind_index(&kind) else {
            self.llm_status = LlmUiStatus::Invalid;
            return;
        };
        let now = self.clock.now();
        if self.last_llm_enqueue[kind_index]
            .is_some_and(|instant| now.saturating_sub(instant) < Duration::from_millis(250))
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

    pub fn quick_save(&self) -> Result<(), GameError> {
        let storage = self
            .storage
            .as_ref()
            .ok_or_else(|| GameError::Io("TUI quick-save storage unavailable".to_string()))?;
        self.client
            .save_to_store(&storage.store, &storage.quick_save)
    }

    pub fn quick_load(&mut self) -> Result<(), GameError> {
        let storage = self
            .storage
            .as_ref()
            .ok_or_else(|| GameError::Io("TUI quick-load storage unavailable".to_string()))?;
        self.client
            .load_from_store(&storage.store, &storage.quick_save)?;
        self.reset_transients();
        Ok(())
    }

    pub fn ui_overlay(&self) -> &UiOverlay {
        &self.overlay
    }

    pub fn storage_error(&self) -> Option<StorageOperation> {
        match self.overlay {
            UiOverlay::StorageError { operation } => Some(operation),
            UiOverlay::None | UiOverlay::Inventory => None,
        }
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

    fn reset_transients(&mut self) {
        self.menu = None;
        if let Some(outstanding) = self.outstanding_llm_request.take() {
            if self.ignored_request_ids.len() == 16 {
                self.ignored_request_ids.remove(0);
            }
            self.ignored_request_ids.push(outstanding.request_id);
        }
        self.next_effect_id = 1;
        self.latest_narrative = None;
        self.latest_decision = None;
        self.latest_soft_adjudication = None;
        self.soft_input = None;
        self.queued_llm_request = None;
        self.last_llm_request = None;
        self.validated_decision = None;
        self.last_llm_enqueue = [None; 3];
        self.hovered_pos = None;
        self.focused_panel = UiPanel::Map;
        self.overlay = UiOverlay::None;
        self.debug_observation_visible = false;
        self.active_labels.clear();
        self.last_label_update_turn = 0;
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

    pub fn supports_terminal_size(&self, width: u16, height: u16) -> bool {
        width >= self.config.min_terminal_width && height >= self.config.min_terminal_height
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

    fn inventory_intent_for_letter(
        &self,
        letter: char,
    ) -> Option<aihack_ai_contract::CommandIntent> {
        use aihack_ai_contract::{CommandIntent, InventoryAction, ItemKind};

        let observation = self.observation();
        let item = observation
            .inventory
            .iter()
            .find(|item| item.letter.0 == letter)?;
        let intent = match self.run_state() {
            RunState::AwaitingInventorySelection { action } => match action {
                InventoryAction::Drop => CommandIntent::Drop { item: item.item },
                InventoryAction::Wield => CommandIntent::Wield { item: item.item },
                InventoryAction::Wear => CommandIntent::Wear { item: item.item },
                InventoryAction::Quaff => CommandIntent::Quaff { item: item.item },
                InventoryAction::Eat => CommandIntent::Eat { item: item.item },
                InventoryAction::Read => CommandIntent::Read { item: item.item },
            },
            _ => match item.kind {
                ItemKind::Dagger => CommandIntent::Wield { item: item.item },
                ItemKind::ArmorLeather => CommandIntent::Wear { item: item.item },
                ItemKind::PotionHealing => CommandIntent::Quaff { item: item.item },
                ItemKind::FoodRation | ItemKind::CorpseJackal => {
                    CommandIntent::Eat { item: item.item }
                }
                ItemKind::ScrollReveal
                | ItemKind::ScrollIdentify
                | ItemKind::ScrollLevelTeleport => CommandIntent::Read { item: item.item },
                ItemKind::WandMagicMissile | ItemKind::Rock | ItemKind::AmuletAscension => {
                    return None
                }
            },
        };
        observation
            .legal_actions
            .contains(&intent)
            .then_some(intent)
    }

    fn cycle_focus(&mut self, backwards: bool) {
        const ORDER: [UiPanel; 6] = [
            UiPanel::Map,
            UiPanel::Status,
            UiPanel::Inventory,
            UiPanel::Inspect,
            UiPanel::Log,
            UiPanel::Command,
        ];
        let index = ORDER
            .iter()
            .position(|panel| *panel == self.focused_panel)
            .unwrap_or(0);
        let next = if backwards {
            (index + ORDER.len() - 1) % ORDER.len()
        } else {
            (index + 1) % ORDER.len()
        };
        self.focused_panel = ORDER[next];
    }

    pub fn handle_candidate_owned(
        &mut self,
        candidate: UiCommandCandidate,
    ) -> Result<bool, GameError> {
        if matches!(
            candidate,
            UiCommandCandidate::Save
                | UiCommandCandidate::Load
                | UiCommandCandidate::LlmJudge
                | UiCommandCandidate::LlmNarrative
                | UiCommandCandidate::LlmSuggest
                | UiCommandCandidate::ToggleDebug
        ) {
            self.menu = None;
            self.overlay = UiOverlay::None;
        }
        match candidate {
            UiCommandCandidate::BeginAction(key) => {
                self.menu = Some(play_menu::action_menu(&self.observation(), key, None));
                Ok(false)
            }
            UiCommandCandidate::ChooseItem { action, item } => {
                self.menu = Some(if action == 'i' {
                    play_menu::item_menu(&self.observation(), item)
                } else {
                    play_menu::action_menu(&self.observation(), action, Some(item))
                });
                Ok(false)
            }
            UiCommandCandidate::MenuPage(forward) => {
                if let Some(menu) = &mut self.menu {
                    let max = menu.entries.len().saturating_sub(1) / play_menu::PAGE_SIZE;
                    menu.page = if forward {
                        (menu.page + 1).min(max)
                    } else {
                        menu.page.saturating_sub(1)
                    };
                }
                Ok(false)
            }
            UiCommandCandidate::OpenCommands => {
                self.menu = Some(play_menu::commands_menu(&self.observation()));
                Ok(false)
            }
            UiCommandCandidate::ShowMessages => {
                let entries = self
                    .observation()
                    .last_events
                    .iter()
                    .rev()
                    .enumerate()
                    .map(|(i, event)| play_menu::Entry {
                        key: char::from(b'0' + i as u8),
                        label: format!("{event:?}"),
                        candidate: UiCommandCandidate::CloseOverlay,
                    })
                    .collect();
                self.menu = Some(play_menu::PlayMenu::new("Recent messages", entries));
                Ok(false)
            }
            UiCommandCandidate::ConfirmQuit => Ok(true),
            UiCommandCandidate::Command(intent) => {
                let outcome = self.client.submit(intent);
                if intent == aihack_ai_contract::CommandIntent::ShowInventory && outcome.accepted {
                    self.overlay = UiOverlay::Inventory;
                    self.menu = Some(play_menu::inventory_menu(&self.observation()));
                    self.focused_panel = UiPanel::Inventory;
                } else if outcome.accepted {
                    self.overlay = UiOverlay::None;
                    self.menu = None;
                }
                // 턴이 진행된 경우에만 새 자동 라벨을 수집한다.
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
                if self.quick_save().is_err() {
                    self.overlay = UiOverlay::StorageError {
                        operation: StorageOperation::Save,
                    };
                } else {
                    self.overlay = UiOverlay::None;
                }
                Ok(false)
            }
            UiCommandCandidate::Load => {
                if self.quick_load().is_err() {
                    self.overlay = UiOverlay::StorageError {
                        operation: StorageOperation::Load,
                    };
                }
                Ok(false)
            }
            UiCommandCandidate::Quit => {
                if matches!(
                    self.run_state(),
                    RunState::Title | RunState::GameOver { .. } | RunState::Victory { .. }
                ) {
                    return Ok(true);
                }
                self.menu = Some(play_menu::PlayMenu::new(
                    "Quit this run?",
                    vec![
                        play_menu::Entry {
                            key: 'y',
                            label: "Quit without saving".into(),
                            candidate: UiCommandCandidate::ConfirmQuit,
                        },
                        play_menu::Entry {
                            key: 'n',
                            label: "Keep playing".into(),
                            candidate: UiCommandCandidate::CloseOverlay,
                        },
                    ],
                ));
                Ok(false)
            }
            UiCommandCandidate::NewRun => {
                if self.client.start_new_run().is_err() {
                    self.overlay = UiOverlay::StorageError {
                        operation: StorageOperation::NewRun,
                    };
                    return Ok(false);
                }
                self.reset_transients();
                Ok(false)
            }
            UiCommandCandidate::CloseOverlay => {
                self.menu = None;
                self.overlay = UiOverlay::None;
                self.focused_panel = UiPanel::Map;
                Ok(false)
            }
            UiCommandCandidate::BackToTitle => {
                if self.client.back_to_title().is_err() {
                    self.overlay = UiOverlay::StorageError {
                        operation: StorageOperation::NewRun,
                    };
                } else {
                    self.reset_transients();
                }
                Ok(false)
            }
            UiCommandCandidate::InventoryLetter(letter) => {
                if let Some(intent) = self.inventory_intent_for_letter(letter) {
                    let outcome = self.client.submit(intent);
                    if outcome.accepted {
                        self.overlay = UiOverlay::None;
                        self.focused_panel = UiPanel::Map;
                    }
                }
                Ok(false)
            }
            UiCommandCandidate::FocusNext => {
                self.cycle_focus(false);
                Ok(false)
            }
            UiCommandCandidate::FocusPrevious => {
                self.cycle_focus(true);
                Ok(false)
            }
            UiCommandCandidate::ToggleDebug => {
                self.debug_observation_visible = !self.debug_observation_visible;
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

    /// 합성 Release를 독립 gesture로 신뢰할 수 없는 transport에서 quiet/drain을 종료 조건으로 쓴다.
    pub fn release_transition_gesture_on_idle(&mut self) {
        if self.transition_gesture.blocked.is_none() {
            self.transition_gesture.consecutive_idle_polls = 0;
            return;
        }
        self.transition_gesture.consecutive_idle_polls = self
            .transition_gesture
            .consecutive_idle_polls
            .saturating_add(1);
        let quiet_window_elapsed = self
            .transition_gesture
            .blocked_until
            .is_some_and(|deadline| self.clock.now() >= deadline);
        if self.transition_gesture.consecutive_idle_polls >= 2 && quiet_window_elapsed {
            self.transition_gesture.blocked = None;
            self.transition_gesture.consecutive_idle_polls = 0;
            self.transition_gesture.blocked_until = None;
        }
    }

    fn filter_keyboard_candidate(
        &mut self,
        key: &KeyEvent,
        candidate: Option<UiCommandCandidate>,
    ) -> Option<UiCommandCandidate> {
        if key.kind == KeyEventKind::Release {
            return None;
        }

        let candidate_repeat_safe = candidate
            .as_ref()
            .is_some_and(|candidate| self.candidate_is_repeat_safe(candidate));
        if let Some(blocked) = self.transition_gesture.blocked.as_ref() {
            if blocked.matches(key) || !candidate_repeat_safe {
                self.transition_gesture.consecutive_idle_polls = 0;
                self.transition_gesture.blocked_until =
                    Some(self.clock.now() + TRANSITION_GESTURE_QUIET_WINDOW);
                return None;
            }
            self.transition_gesture.blocked = None;
            self.transition_gesture.consecutive_idle_polls = 0;
            self.transition_gesture.blocked_until = None;
        }

        let candidate = candidate?;
        let repeat_safe = self.candidate_is_repeat_safe(&candidate);
        if key.kind == KeyEventKind::Repeat && !repeat_safe {
            return None;
        }
        if key.kind == KeyEventKind::Press && !repeat_safe {
            self.transition_gesture.blocked = Some(KeyGesture::from_event(key));
            self.transition_gesture.consecutive_idle_polls = 0;
            self.transition_gesture.blocked_until =
                Some(self.clock.now() + TRANSITION_GESTURE_QUIET_WINDOW);
        }
        Some(candidate)
    }

    fn candidate_is_repeat_safe(&self, candidate: &UiCommandCandidate) -> bool {
        match candidate {
            UiCommandCandidate::LlmInput(_) | UiCommandCandidate::LlmBackspace => {
                self.soft_input.is_some()
            }
            UiCommandCandidate::Focus(_)
            | UiCommandCandidate::FocusNext
            | UiCommandCandidate::FocusPrevious => true,
            UiCommandCandidate::Command(
                aihack_ai_contract::CommandIntent::Move(_)
                | aihack_ai_contract::CommandIntent::Wait,
            ) => {
                self.overlay == UiOverlay::None
                    && self.menu.is_none()
                    && self.soft_input.is_none()
                    && self.run_state() == RunState::Playing
            }
            _ => false,
        }
    }
}

fn llm_kind_index(kind: &LlmRequestKind) -> Option<usize> {
    match kind {
        LlmRequestKind::Narrative => Some(0),
        LlmRequestKind::Decision => Some(1),
        LlmRequestKind::SoftAdjudication { .. } => Some(2),
        _ => None,
    }
}

/// RunState에 따라 Title -> CharacterCreation -> Playing <-> GameOver 화면을 분기한다.
pub fn run_tui(seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    run_tui_with_config(seed, UiRuntimeConfig::default())
}

pub fn run_tui_with_config(
    seed: u64,
    ui_config: UiRuntimeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    run_tui_with_config_and_save_dir(seed, ui_config, Path::new("runtime/tui"))
}

pub fn run_tui_with_config_and_save_dir(
    seed: u64,
    ui_config: UiRuntimeConfig,
    save_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let llm_config = LocalLlmConfig::from_env().map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid local LLM configuration: {error:?}"),
        )
    })?;
    let llm_enabled = llm_config.enabled();
    let service = LocalLlmService::from_config(llm_config)
        .map_err(|error| std::io::Error::other(format!("local LLM startup failed: {error:?}")))?;
    run_tui_with_service(seed, service, llm_enabled, ui_config, save_dir)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TerminalRestoreState {
    cursor_hidden: bool,
    mouse_capture: bool,
    raw_mode: bool,
    alternate_screen: bool,
}

trait TerminalRestoreOps {
    fn show_cursor(&mut self) -> std::io::Result<()>;
    fn disable_mouse_capture(&mut self) -> std::io::Result<()>;
    fn disable_raw_mode(&mut self) -> std::io::Result<()>;
    fn leave_alternate_screen(&mut self) -> std::io::Result<()>;
}

trait TerminalSetupOps {
    fn enter_alternate_screen(&mut self) -> std::io::Result<()>;
    fn enable_raw_mode(&mut self) -> std::io::Result<()>;
    fn enable_mouse_capture(&mut self) -> std::io::Result<()>;
    fn hide_cursor(&mut self) -> std::io::Result<()>;
}

struct CrosstermSetupOps<'a> {
    stdout: &'a mut std::io::Stdout,
}

impl TerminalSetupOps for CrosstermSetupOps<'_> {
    fn enter_alternate_screen(&mut self) -> std::io::Result<()> {
        self.stdout.execute(EnterAlternateScreen).map(|_| ())
    }

    fn enable_raw_mode(&mut self) -> std::io::Result<()> {
        terminal::enable_raw_mode()
    }

    fn enable_mouse_capture(&mut self) -> std::io::Result<()> {
        self.stdout.execute(EnableMouseCapture).map(|_| ())
    }

    fn hide_cursor(&mut self) -> std::io::Result<()> {
        self.stdout.execute(cursor::Hide).map(|_| ())
    }
}

struct CrosstermRestoreOps;

impl TerminalRestoreOps for CrosstermRestoreOps {
    fn show_cursor(&mut self) -> std::io::Result<()> {
        std::io::stdout().execute(cursor::Show).map(|_| ())
    }

    fn disable_mouse_capture(&mut self) -> std::io::Result<()> {
        std::io::stdout().execute(DisableMouseCapture).map(|_| ())
    }

    fn disable_raw_mode(&mut self) -> std::io::Result<()> {
        terminal::disable_raw_mode()
    }

    fn leave_alternate_screen(&mut self) -> std::io::Result<()> {
        std::io::stdout().execute(LeaveAlternateScreen).map(|_| ())
    }
}

fn setup_terminal_state(
    state: &mut TerminalRestoreState,
    ops: &mut impl TerminalSetupOps,
    enable_mouse: bool,
) -> std::io::Result<()> {
    ops.enter_alternate_screen()?;
    state.alternate_screen = true;
    ops.enable_raw_mode()?;
    state.raw_mode = true;
    if enable_mouse {
        ops.enable_mouse_capture()?;
        state.mouse_capture = true;
    }
    ops.hide_cursor()?;
    state.cursor_hidden = true;
    Ok(())
}

fn restore_terminal_state(
    state: &mut TerminalRestoreState,
    ops: &mut impl TerminalRestoreOps,
) -> std::io::Result<()> {
    let mut first_error = None;
    let mut attempt = |result: std::io::Result<()>| {
        if let Err(error) = result {
            if first_error.is_none() {
                first_error = Some(error);
            }
        }
    };

    if state.cursor_hidden {
        attempt(ops.show_cursor());
        state.cursor_hidden = false;
    }
    if state.mouse_capture {
        attempt(ops.disable_mouse_capture());
        state.mouse_capture = false;
    }
    if state.raw_mode {
        attempt(ops.disable_raw_mode());
        state.raw_mode = false;
    }
    if state.alternate_screen {
        attempt(ops.leave_alternate_screen());
        state.alternate_screen = false;
    }
    first_error.map_or(Ok(()), Err)
}

fn run_with_terminal_restore<T, E>(
    state: &mut TerminalRestoreState,
    ops: &mut impl TerminalRestoreOps,
    run: impl FnOnce() -> Result<T, E>,
) -> Result<T, E>
where
    E: From<std::io::Error>,
{
    let run_result = run();
    let restore_result = restore_terminal_state(state, ops).map_err(E::from);
    match run_result {
        Ok(value) => restore_result.map(|()| value),
        Err(error) => {
            let _ = restore_result;
            Err(error)
        }
    }
}

#[derive(Default)]
struct TerminalSessionGuard {
    state: TerminalRestoreState,
}

impl TerminalSessionGuard {
    fn restore(&mut self) -> std::io::Result<()> {
        restore_terminal_state(&mut self.state, &mut CrosstermRestoreOps)
    }
}

impl Drop for TerminalSessionGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn run_tui_with_service(
    seed: u64,
    mut llm_service: LocalLlmService,
    llm_enabled: bool,
    ui_config: UiRuntimeConfig,
    save_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    let mut terminal_guard = TerminalSessionGuard::default();
    {
        let mut setup_ops = CrosstermSetupOps {
            stdout: &mut stdout,
        };
        setup_terminal_state(
            &mut terminal_guard.state,
            &mut setup_ops,
            ui_config.enable_mouse,
        )?;
    }
    let mut restore_ops = CrosstermRestoreOps;
    let run_result = run_with_terminal_restore(
        &mut terminal_guard.state,
        &mut restore_ops,
        || -> Result<(), Box<dyn std::error::Error>> {
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;
            let mut app =
                TuiApp::new_with_llm_enabled(GameSession::try_new(seed)?, ui_config, llm_enabled)
                    .with_save_directory(save_dir)?;
            loop {
                app.poll_llm_response(&llm_service);
                terminal.draw(|frame| {
                    let size = frame.area();
                    if !app.supports_terminal_size(size.width, size.height) {
                        frame.render_widget(
                            render_panels::ThemedTextPanel {
                                title: "TUI",
                                lines: vec![
                                    "terminal requires 60x24; resize or press Q/Esc to exit"
                                        .to_string(),
                                ],
                                theme: app.theme(),
                            },
                            size,
                        );
                        return;
                    }
                    render_frame(frame, &mut app);
                })?;
                let size = terminal.size()?;
                if event::poll(Duration::from_millis(50))? {
                    let input_event = event::read()?;
                    let candidate =
                        runtime_event_to_candidate(input_event, size.width, size.height, &mut app);
                    if let Some(candidate) = candidate {
                        if app.handle_candidate_owned(candidate)? {
                            break;
                        }
                        app.dispatch_llm_request(&llm_service);
                    }
                } else {
                    app.release_transition_gesture_on_idle();
                }
            }
            Ok(())
        },
    );

    // 외부 응답이 지연되어도 terminal cleanup을 마친 뒤 worker 종료를 기다린다.
    let _worker_stopped = llm_service.shutdown_with_grace(Duration::from_millis(250));

    run_result?;
    Ok(())
}

/// 실행 루프와 버퍼 회귀가 동일한 화면 그리기를 사용한다.
pub fn render_frame(frame: &mut ratatui::Frame, app: &mut TuiApp) {
    let size = frame.area();
    match app.run_state() {
        crate::core::session::RunState::Title => render_title_screen(frame, size, app.theme()),
        crate::core::session::RunState::CharacterCreation => {
            render_character_creation_screen(frame, size, app.theme())
        }
        crate::core::session::RunState::Playing
        | crate::core::session::RunState::AwaitingDirection { .. }
        | crate::core::session::RunState::AwaitingInventorySelection { .. }
        | crate::core::session::RunState::MorePrompt => render_play_screen(frame, size, app),
        crate::core::session::RunState::GameOver { cause, final_score } => {
            render_game_over_screen(frame, size, app, cause, final_score)
        }
        RunState::Victory { final_score } => {
            frame.render_widget(
                render_panels::ThemedTextPanel {
                    title: "ASCENDED",
                    lines: render_panels::victory_lines(final_score),
                    theme: app.theme(),
                },
                size,
            );
        }
    }
    render_global_overlay(frame, size, app);
}

fn render_title_screen(frame: &mut ratatui::Frame, size: Rect, theme: UiTheme) {
    frame.render_widget(
        render_panels::ThemedTextPanel {
            title: "AIHack",
            lines: render_panels::title_lines(),
            theme,
        },
        size,
    );
}

fn render_character_creation_screen(frame: &mut ratatui::Frame, size: Rect, theme: UiTheme) {
    frame.render_widget(
        render_panels::ThemedTextPanel {
            title: "Character Creation",
            lines: render_panels::character_creation_lines(),
            theme,
        },
        size,
    );
}

fn render_play_screen(frame: &mut ratatui::Frame, _size: Rect, app: &mut TuiApp) {
    let layout = compute_layout(_size.width, _size.height);
    let observation = app.observation();
    let viewport = app.viewport_for_observation(layout);

    // frame 직전에 만료된 라벨을 제거한다.
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
                crate::core::action::InventoryAction::Eat => "eat",
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
            theme: app.theme(),
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
        render_panels::ThemedTextPanel {
            title: "STATUS",
            lines: status_lines,
            theme: app.theme(),
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
        render_panels::ThemedTextPanel {
            title: "COMMANDS",
            lines: command_lines,
            theme: app.theme(),
        },
        layout.command,
    );
    frame.render_widget(
        render_panels::ThemedTextPanel {
            title: "LOG",
            lines: render_panels::log_lines(&observation, &app.narrative_lines()),
            theme: app.theme(),
        },
        layout.log,
    );
    frame.render_widget(
        render_panels::ThemedTextPanel {
            title: "INSPECT",
            lines: render_panels::inspect_lines(
                &observation,
                app.hovered_pos(),
                app.focused_panel(),
                &app.llm_result_lines(),
            ),
            theme: app.theme(),
        },
        layout.inspect,
    );
    // F9 토글 debug observation 패널이다.
    // 이 패널은 UI-only이며 snapshot hash에 영향을 주지 않는다.
    if app.debug_observation_visible {
        let debug_lines = render_panels::debug_observation_lines(&observation);
        let debug_area = debug_observation_area(layout, &observation);
        frame.render_widget(
            render_panels::ThemedTextPanel {
                title: "DEBUG OBS",
                lines: debug_lines,
                theme: app.theme(),
            },
            debug_area,
        );
    } else if let Some(debug) = layout.debug {
        // roomy layout(120x36+)에서 기본 debug 패널 표시
        frame.render_widget(
            render_panels::ThemedTextPanel {
                title: "DEBUG",
                lines: vec![format!("effects {}", app.project_effects().len())],
                theme: app.theme(),
            },
            debug,
        );
    }

    if let Some(input) = app.soft_input() {
        frame.render_widget(
            render_panels::ThemedTextPanel {
                title: "SOFT JUDGMENT INPUT",
                lines: render_panels::soft_input_lines(input),
                theme: app.theme(),
            },
            layout.inspect,
        );
    }

    // blocking prompt는 최소 화면의 1행 log에 자르지 않고 root 중앙 modal로 표시한다.
    if let Some(lines) = state_overlay {
        let overlay_height = (lines.len() as u16 + 1).min(_size.height);
        let overlay_width = _size.width.min(72);
        let overlay_area = Rect {
            x: _size.x + _size.width.saturating_sub(overlay_width) / 2,
            y: _size.y + _size.height.saturating_sub(overlay_height) / 2,
            width: overlay_width,
            height: overlay_height,
        };
        frame.render_widget(
            render_panels::ThemedTextPanel {
                title: "STATE",
                lines,
                theme: app.theme(),
            },
            overlay_area,
        );
    }
}

/// renderer와 mouse dispatcher가 공유하는 F9 observation panel 영역이다.
pub fn debug_observation_area(layout: TuiLayout, observation: &Observation) -> Rect {
    let debug_height = render_panels::debug_observation_lines(observation).len() as u16 + 2;
    Rect {
        x: layout.map.x + layout.map.width.saturating_sub(40),
        y: layout.map.y,
        width: 40.min(layout.map.width),
        height: debug_height.min(layout.map.height),
    }
}

fn rect_contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
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
        render_panels::ThemedTextPanel {
            title: "GAME OVER",
            lines,
            theme: app.theme(),
        },
        size,
    );
}

/// RunState에 따라 키 입력을 다른 후보로 매핑한다.
fn key_to_candidate_for_state(
    ch: char,
    state: &crate::core::session::RunState,
    observation: &Observation,
) -> Option<UiCommandCandidate> {
    use crate::core::session::RunState;
    match state {
        RunState::Title => match ch {
            '\n' | '\r' | 'n' | 'N' => Some(UiCommandCandidate::Command(
                crate::core::action::CommandIntent::Wait,
            )),
            'q' | 'Q' => Some(UiCommandCandidate::Quit),
            'l' | 'L' => Some(UiCommandCandidate::Load),
            _ => None,
        },
        RunState::CharacterCreation => match ch {
            '1' | '2' | '3' => Some(UiCommandCandidate::Command(
                aihack_ai_contract::CommandIntent::StartCampaign {
                    role: match ch {
                        '1' => aihack_ai_contract::Role::Knight,
                        '2' => aihack_ai_contract::Role::Scout,
                        _ => aihack_ai_contract::Role::Mage,
                    },
                },
            )),
            '\n' | '\r' => Some(UiCommandCandidate::Command(
                crate::core::action::CommandIntent::Wait,
            )),
            'q' | 'Q' => Some(UiCommandCandidate::Quit),
            _ => None,
        },
        RunState::GameOver { .. } | RunState::Victory { .. } => match ch {
            'n' | 'N' => Some(UiCommandCandidate::NewRun),
            'q' | 'Q' => Some(UiCommandCandidate::Quit),
            _ => None,
        },
        RunState::AwaitingDirection { .. } => direction_for_key(ch)
            .map(crate::core::action::CommandIntent::Move)
            .map(UiCommandCandidate::Command),
        RunState::AwaitingInventorySelection { .. } => observation
            .inventory
            .iter()
            .any(|item| item.letter.0 == ch)
            .then_some(UiCommandCandidate::InventoryLetter(ch)),
        RunState::MorePrompt => Some(UiCommandCandidate::Command(
            crate::core::action::CommandIntent::AcknowledgeMore,
        )),
        RunState::Playing => key_to_candidate(ch, observation),
    }
}

pub fn runtime_key_to_candidate(
    key_code: KeyCode,
    state: &crate::core::session::RunState,
    observation: &Observation,
) -> Option<UiCommandCandidate> {
    use crate::core::session::RunState;
    match state {
        RunState::MorePrompt => {
            return Some(UiCommandCandidate::Command(
                crate::core::action::CommandIntent::AcknowledgeMore,
            ));
        }
        RunState::AwaitingDirection { .. } => {
            return match key_code {
                KeyCode::Esc => Some(UiCommandCandidate::Command(
                    crate::core::action::CommandIntent::AcknowledgeMore,
                )),
                KeyCode::Char(character) => {
                    key_to_candidate_for_state(character, state, observation)
                }
                _ => None,
            };
        }
        RunState::AwaitingInventorySelection { .. } => {
            return match key_code {
                KeyCode::Esc => Some(UiCommandCandidate::Command(
                    crate::core::action::CommandIntent::AcknowledgeMore,
                )),
                KeyCode::Char(character) => {
                    key_to_candidate_for_state(character, state, observation)
                }
                _ => None,
            };
        }
        RunState::GameOver { .. } | RunState::Victory { .. } => {
            return match key_code {
                KeyCode::Esc => Some(UiCommandCandidate::Quit),
                KeyCode::Char(character) => {
                    key_to_candidate_for_state(character, state, observation)
                }
                _ => None,
            };
        }
        _ => {}
    }
    match key_code {
        KeyCode::Esc => match state {
            crate::core::session::RunState::CharacterCreation => {
                Some(UiCommandCandidate::BackToTitle)
            }
            crate::core::session::RunState::AwaitingDirection { .. }
            | crate::core::session::RunState::AwaitingInventorySelection { .. }
            | crate::core::session::RunState::MorePrompt => Some(UiCommandCandidate::Command(
                crate::core::action::CommandIntent::AcknowledgeMore,
            )),
            crate::core::session::RunState::Title
            | crate::core::session::RunState::Playing
            | crate::core::session::RunState::GameOver { .. }
            | RunState::Victory { .. } => Some(UiCommandCandidate::Quit),
        },
        KeyCode::Tab => Some(UiCommandCandidate::FocusNext),
        KeyCode::BackTab => Some(UiCommandCandidate::FocusPrevious),
        KeyCode::Enter => key_to_candidate_for_state('\n', state, observation),
        KeyCode::Char(character) => key_to_candidate_for_state(character, state, observation),
        _ => None,
    }
}

/// 실제 event loop와 회귀 테스트가 공유하는 단일 state-aware dispatcher다.
fn modal_mouse_candidate(
    input: Event,
    width: u16,
    height: u16,
    app: &TuiApp,
) -> Option<UiCommandCandidate> {
    let Event::Mouse(mouse) = input else {
        return None;
    };
    if mouse.kind != MouseEventKind::Down(event::MouseButton::Left) {
        return None;
    }
    if matches!(app.ui_overlay(), UiOverlay::StorageError { .. }) {
        return Some(UiCommandCandidate::CloseOverlay);
    }
    let (lines, candidates): (Vec<String>, Vec<(&str, UiCommandCandidate)>) = match app.run_state()
    {
        RunState::Title => (
            render_panels::title_lines(),
            vec![
                (
                    "Press Enter to Start",
                    UiCommandCandidate::Command(aihack_ai_contract::CommandIntent::Wait),
                ),
                ("L - Load Game", UiCommandCandidate::Load),
                ("Q - Quit", UiCommandCandidate::Quit),
            ],
        ),
        RunState::CharacterCreation => (
            render_panels::character_creation_lines(),
            vec![
                (
                    "[1] Knight",
                    UiCommandCandidate::Command(aihack_ai_contract::CommandIntent::StartCampaign {
                        role: aihack_ai_contract::Role::Knight,
                    }),
                ),
                (
                    "[2] Scout",
                    UiCommandCandidate::Command(aihack_ai_contract::CommandIntent::StartCampaign {
                        role: aihack_ai_contract::Role::Scout,
                    }),
                ),
                (
                    "[3] Mage",
                    UiCommandCandidate::Command(aihack_ai_contract::CommandIntent::StartCampaign {
                        role: aihack_ai_contract::Role::Mage,
                    }),
                ),
                (
                    "Press Enter to confirm",
                    UiCommandCandidate::Command(aihack_ai_contract::CommandIntent::Wait),
                ),
                ("Esc - Back to Title", UiCommandCandidate::BackToTitle),
            ],
        ),
        RunState::GameOver {
            cause: _,
            final_score: _,
        } => (
            render_panels::game_over_lines("", 0, 0, 0, 0, 0),
            vec![
                ("[N] New Run", UiCommandCandidate::NewRun),
                ("[Q] Quit", UiCommandCandidate::Quit),
            ],
        ),
        RunState::Victory { final_score } => (
            render_panels::victory_lines(final_score),
            vec![
                ("[N] New Run", UiCommandCandidate::NewRun),
                ("[Q] Quit", UiCommandCandidate::Quit),
            ],
        ),
        _ => {
            if let Some(input) = app.soft_input() {
                let area = compute_layout(width, height).inspect;
                let lines = render_panels::soft_input_lines(input);
                return text_cta_at(
                    &lines,
                    area,
                    mouse.column,
                    mouse.row,
                    &[
                        ("[Enter] Submit", UiCommandCandidate::LlmSubmitInput),
                        ("[Esc] Cancel", UiCommandCandidate::LlmCancelInput),
                    ],
                );
            }
            return None;
        }
    };
    text_cta_at(
        &lines,
        Rect::new(0, 0, width, height),
        mouse.column,
        mouse.row,
        &candidates,
    )
}

fn text_cta_at(
    lines: &[String],
    area: Rect,
    x: u16,
    y: u16,
    candidates: &[(&str, UiCommandCandidate)],
) -> Option<UiCommandCandidate> {
    if !rect_contains(area, x, y) {
        return None;
    }
    let line = lines.get(y.checked_sub(area.y + 1)? as usize)?;
    let offset = x.checked_sub(area.x)? as usize;
    candidates.iter().find_map(|(label, candidate)| {
        let start = line.find(label)?;
        (offset >= start && offset < start + label.len()).then_some(*candidate)
    })
}

pub fn runtime_event_to_candidate(
    input_event: Event,
    width: u16,
    height: u16,
    app: &mut TuiApp,
) -> Option<UiCommandCandidate> {
    if !app.supports_terminal_size(width, height) {
        return match input_event {
            Event::Key(key) => {
                let candidate = (key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q' | 'Q') | KeyCode::Esc))
                .then_some(UiCommandCandidate::ConfirmQuit);
                app.filter_keyboard_candidate(&key, candidate)
            }
            _ => None,
        };
    }
    if matches!(&input_event, Event::Mouse(_))
        && app.menu.is_none()
        && (app.ui_overlay() != &UiOverlay::None
            || app.soft_input().is_some()
            || matches!(
                app.run_state(),
                RunState::Title
                    | RunState::CharacterCreation
                    | RunState::AwaitingDirection { .. }
                    | RunState::AwaitingInventorySelection { .. }
                    | RunState::MorePrompt
                    | RunState::GameOver { .. }
                    | RunState::Victory { .. }
            ))
    {
        return modal_mouse_candidate(input_event, width, height, app);
    }
    let key = match input_event {
        Event::Key(key) => key,
        Event::Mouse(mouse) => {
            if let Some(menu) = &app.menu {
                return match mouse.kind {
                    MouseEventKind::Down(event::MouseButton::Left) => {
                        menu.click(width, height, mouse.column, mouse.row)
                    }
                    MouseEventKind::ScrollDown => Some(UiCommandCandidate::MenuPage(true)),
                    MouseEventKind::ScrollUp => Some(UiCommandCandidate::MenuPage(false)),
                    _ => None,
                };
            }
            let layout = compute_layout(width, height);
            if app.debug_observation_visible
                && rect_contains(
                    debug_observation_area(layout, &app.observation()),
                    mouse.column,
                    mouse.row,
                )
            {
                return None;
            }
            let viewport = app.viewport_for_observation(layout);
            let input = match mouse.kind {
                MouseEventKind::Moved => UiInputEvent::MouseHover {
                    column: mouse.column,
                    row: mouse.row,
                },
                MouseEventKind::Down(event::MouseButton::Left) => UiInputEvent::MouseClick {
                    column: mouse.column,
                    row: mouse.row,
                },
                _ => return None,
            };
            return map_mouse_event_for_state(input, layout, viewport, app);
        }
        _ => return None,
    };
    let candidate = if let Some(menu) = &app.menu {
        match key.code {
            KeyCode::Esc => Some(UiCommandCandidate::CloseOverlay),
            KeyCode::PageDown => Some(UiCommandCandidate::MenuPage(true)),
            KeyCode::PageUp => Some(UiCommandCandidate::MenuPage(false)),
            KeyCode::Char(ch) => menu.key(ch),
            _ => None,
        }
    } else if matches!(app.ui_overlay(), UiOverlay::StorageError { .. }) {
        Some(UiCommandCandidate::CloseOverlay)
    } else if app.ui_overlay() == &UiOverlay::Inventory {
        match key.code {
            KeyCode::Esc | KeyCode::Char('i' | 'I') => Some(UiCommandCandidate::CloseOverlay),
            KeyCode::Char(letter) => Some(UiCommandCandidate::InventoryLetter(letter)),
            KeyCode::Tab => Some(UiCommandCandidate::FocusNext),
            KeyCode::BackTab => Some(UiCommandCandidate::FocusPrevious),
            _ => None,
        }
    } else if app.soft_input().is_some() {
        match key.code {
            KeyCode::Enter => Some(UiCommandCandidate::LlmSubmitInput),
            KeyCode::Backspace => Some(UiCommandCandidate::LlmBackspace),
            KeyCode::Esc => Some(UiCommandCandidate::LlmCancelInput),
            KeyCode::Char(character) => Some(UiCommandCandidate::LlmInput(character)),
            _ => None,
        }
    } else {
        let state = app.run_state();
        if matches!(
            state,
            RunState::AwaitingDirection { .. }
                | RunState::AwaitingInventorySelection { .. }
                | RunState::MorePrompt
                | RunState::GameOver { .. }
                | RunState::Victory { .. }
        ) {
            runtime_key_to_candidate(key.code, &state, &app.observation())
        } else {
            match key.code {
                KeyCode::F(9) => Some(UiCommandCandidate::ToggleDebug),
                KeyCode::Char('N') | KeyCode::Esc if app.has_llm_result() => {
                    Some(UiCommandCandidate::DismissLlmResult)
                }
                key_code => runtime_key_to_candidate(key_code, &state, &app.observation()),
            }
        }
    };
    app.filter_keyboard_candidate(&key, candidate)
}

fn render_global_overlay(frame: &mut ratatui::Frame, size: Rect, app: &TuiApp) {
    if let Some(menu) = &app.menu {
        frame.render_widget(
            render_panels::ThemedTextPanel {
                title: &menu.title,
                lines: menu.lines(),
                theme: app.theme(),
            },
            menu.area(size.width, size.height),
        );
        return;
    }
    let (title, lines) = match app.ui_overlay() {
        UiOverlay::None => return,
        UiOverlay::Inventory => (
            "INVENTORY",
            render_panels::inventory_overlay_lines(&app.observation()),
        ),
        UiOverlay::StorageError { operation } => (
            "STORAGE ERROR",
            render_panels::storage_error_lines(*operation),
        ),
    };
    let height = (lines.len() as u16 + 2).min(size.height);
    let width = size.width.min(72);
    let area = Rect {
        x: size.x + size.width.saturating_sub(width) / 2,
        y: size.y + size.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(
        render_panels::ThemedTextPanel {
            title,
            lines,
            theme: app.theme(),
        },
        area,
    );
}

fn direction_for_key(ch: char) -> Option<crate::core::Direction> {
    match ch {
        'h' => Some(crate::core::Direction::West),
        'j' => Some(crate::core::Direction::South),
        'k' => Some(crate::core::Direction::North),
        'l' => Some(crate::core::Direction::East),
        'y' => Some(crate::core::Direction::NorthWest),
        'u' => Some(crate::core::Direction::NorthEast),
        'b' => Some(crate::core::Direction::SouthWest),
        'n' => Some(crate::core::Direction::SouthEast),
        _ => None,
    }
}

/// RunState에 따라 마우스 입력을 처리한다.
fn map_mouse_event_for_state(
    event: UiInputEvent,
    layout: TuiLayout,
    viewport: Viewport,
    app: &TuiApp,
) -> Option<UiCommandCandidate> {
    use crate::core::session::RunState;
    match app.run_state() {
        RunState::Title
        | RunState::CharacterCreation
        | RunState::GameOver { .. }
        | RunState::Victory { .. } => None,
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
            input::map_mouse_event(
                event,
                layout,
                viewport,
                &app.observation(),
                input::inspect_presentation(app.hovered_pos(), &app.llm_result_lines()),
            )
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

#[cfg(test)]
mod terminal_restore_tests {
    use super::{
        restore_terminal_state, run_with_terminal_restore, setup_terminal_state,
        TerminalRestoreOps, TerminalRestoreState, TerminalSetupOps,
    };
    use std::io;

    #[derive(Default)]
    struct FakeRestoreOps {
        calls: Vec<&'static str>,
        fail_at: Option<usize>,
    }

    impl FakeRestoreOps {
        fn step(&mut self, name: &'static str) -> io::Result<()> {
            let index = self.calls.len();
            self.calls.push(name);
            if self.fail_at == Some(index) {
                Err(io::Error::other(format!("injected {name} failure")))
            } else {
                Ok(())
            }
        }
    }

    impl TerminalRestoreOps for FakeRestoreOps {
        fn show_cursor(&mut self) -> io::Result<()> {
            self.step("show_cursor")
        }
        fn disable_mouse_capture(&mut self) -> io::Result<()> {
            self.step("disable_mouse")
        }
        fn disable_raw_mode(&mut self) -> io::Result<()> {
            self.step("disable_raw")
        }
        fn leave_alternate_screen(&mut self) -> io::Result<()> {
            self.step("leave_alternate")
        }
    }

    #[derive(Default)]
    struct FakeSetupOps {
        calls: Vec<&'static str>,
        fail_at: Option<usize>,
    }

    impl FakeSetupOps {
        fn step(&mut self, name: &'static str) -> io::Result<()> {
            let index = self.calls.len();
            self.calls.push(name);
            if self.fail_at == Some(index) {
                Err(io::Error::other(format!("injected {name} failure")))
            } else {
                Ok(())
            }
        }
    }

    impl TerminalSetupOps for FakeSetupOps {
        fn enter_alternate_screen(&mut self) -> io::Result<()> {
            self.step("enter_alternate")
        }
        fn enable_raw_mode(&mut self) -> io::Result<()> {
            self.step("enable_raw")
        }
        fn enable_mouse_capture(&mut self) -> io::Result<()> {
            self.step("enable_mouse")
        }
        fn hide_cursor(&mut self) -> io::Result<()> {
            self.step("hide_cursor")
        }
    }

    #[test]
    fn best_effort_restore_attempts_every_step_after_each_injected_failure() {
        for fail_at in 0..4 {
            let mut state = TerminalRestoreState {
                cursor_hidden: true,
                mouse_capture: true,
                raw_mode: true,
                alternate_screen: true,
            };
            let mut ops = FakeRestoreOps {
                fail_at: Some(fail_at),
                ..Default::default()
            };

            assert!(restore_terminal_state(&mut state, &mut ops).is_err());
            assert_eq!(
                ops.calls,
                [
                    "show_cursor",
                    "disable_mouse",
                    "disable_raw",
                    "leave_alternate"
                ]
            );
            assert_eq!(state, TerminalRestoreState::default());
        }
    }

    #[test]
    fn setup_failure_records_only_completed_states_and_restores_them() {
        for fail_at in 0..4 {
            let mut state = TerminalRestoreState::default();
            let mut setup = FakeSetupOps {
                fail_at: Some(fail_at),
                ..Default::default()
            };
            assert!(setup_terminal_state(&mut state, &mut setup, true).is_err());

            let mut restore = FakeRestoreOps::default();
            restore_terminal_state(&mut state, &mut restore).unwrap();
            let expected = match fail_at {
                0 => Vec::new(),
                1 => vec!["leave_alternate"],
                2 => vec!["disable_raw", "leave_alternate"],
                3 => vec!["disable_mouse", "disable_raw", "leave_alternate"],
                _ => unreachable!(),
            };
            assert_eq!(restore.calls, expected, "setup fail_at={fail_at}");
            assert_eq!(state, TerminalRestoreState::default());
        }
    }

    #[test]
    fn terminal_new_draw_and_read_failures_share_the_full_restore_boundary() {
        for stage in ["terminal_new", "draw", "read"] {
            let mut state = TerminalRestoreState {
                cursor_hidden: true,
                mouse_capture: true,
                raw_mode: true,
                alternate_screen: true,
            };
            let mut restore = FakeRestoreOps::default();
            let result: io::Result<()> =
                run_with_terminal_restore(&mut state, &mut restore, || {
                    Err(io::Error::other(format!("injected {stage} failure")))
                });
            assert!(result.is_err(), "stage={stage}");
            assert_eq!(
                restore.calls,
                [
                    "show_cursor",
                    "disable_mouse",
                    "disable_raw",
                    "leave_alternate"
                ],
                "stage={stage}"
            );
            assert_eq!(state, TerminalRestoreState::default());
        }
    }
}

#[cfg(test)]
mod blocking_prompt_render_tests {
    use super::{render_play_screen, TuiApp, UiRuntimeConfig};
    use aihack_ai_contract::RunState;
    use aihack_runtime::GameSession;
    use ratatui::{backend::TestBackend, Terminal};

    fn rendered_symbols(state: RunState, width: u16, height: u16) -> String {
        let mut save = GameSession::new_for_playing(42).to_save_data();
        save.run_state = state;
        let session = GameSession::from_save_data(save).unwrap();
        let mut app = TuiApp::new(session, UiRuntimeConfig::default());
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_play_screen(frame, frame.area(), &mut app))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let mut rendered = String::new();
        for row in 0..height {
            for column in 0..width {
                rendered.push_str(buffer[(column, row)].symbol());
            }
            rendered.push('\n');
        }
        rendered
    }

    #[test]
    fn minimum_supported_sizes_render_complete_blocking_prompt_content() {
        for (width, height) in [(60, 24), (80, 24)] {
            let more = rendered_symbols(RunState::MorePrompt, width, height);
            assert!(more.contains("--More--"), "size={width}x{height}");
            assert!(
                more.contains("Press any key to continue"),
                "size={width}x{height}"
            );
        }
    }
}

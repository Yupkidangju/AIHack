use crate::core::{
    observation::ItemObservation, ActionIntent, CommandIntent, Direction, Observation, Pos,
};
use ratatui::layout::Rect;

use super::{layout::TuiLayout, viewport::Viewport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiPanel {
    Map,
    Status,
    Inspect,
    Log,
    Command,
    Debug,
    Inventory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectPresentation {
    Inventory,
    Hover,
    Decision,
}

pub fn inspect_presentation(
    hovered: Option<Pos>,
    decision_lines: &[String],
) -> InspectPresentation {
    if hovered.is_some() {
        InspectPresentation::Hover
    } else if decision_lines.is_empty() {
        InspectPresentation::Inventory
    } else {
        InspectPresentation::Decision
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiInputEvent {
    Key(CommandIntent),
    MouseHover { column: u16, row: u16 },
    MouseClick { column: u16, row: u16 },
    FocusPanel(UiPanel),
    SaveRequest,
    LoadRequest,
    Quit,
}

/// TUI 화면 전환과 명령 후보다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCommandCandidate {
    BeginAction(char),
    ChooseItem {
        action: char,
        item: aihack_ai_contract::EntityId,
    },
    MenuPage(bool),
    ConfirmQuit,
    OpenCommands,
    ShowMessages,
    Command(CommandIntent),
    Inspect(Pos),
    Focus(UiPanel),
    Save,
    Load,
    Quit,
    /// Game Over에서 새 게임을 시작한다.
    NewRun,
    CloseOverlay,
    BackToTitle,
    InventoryLetter(char),
    FocusNext,
    FocusPrevious,
    ToggleDebug,
    /// 표시 전용 LLM 결과를 제거하며 core command는 생성하지 않는다.
    DismissLlmResult,
    LlmNarrative,
    LlmSuggest,
    LlmJudge,
    LlmApply,
    LlmRetry,
    LlmInput(char),
    LlmBackspace,
    LlmSubmitInput,
    LlmCancelInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelCta {
    pub label: String,
    pub candidate: Option<UiCommandCandidate>,
}

pub fn command_panel_ctas(observation: &Observation) -> Vec<PanelCta> {
    let open_candidate = key_to_candidate('o', observation);
    vec![
        PanelCta {
            label: "[i] Inventory".to_string(),
            candidate: key_to_candidate('i', observation),
        },
        PanelCta {
            label: "[. ] Wait".to_string(),
            candidate: observation
                .action_space
                .commands
                .contains(&ActionIntent::Command(CommandIntent::Wait))
                .then_some(UiCommandCandidate::Command(CommandIntent::Wait)),
        },
        PanelCta {
            label: if open_candidate.is_some() {
                "[o] Open".to_string()
            } else {
                "[o] Open*".to_string()
            },
            candidate: open_candidate,
        },
        PanelCta {
            label: "[?] All".to_string(),
            candidate: Some(UiCommandCandidate::OpenCommands),
        },
        PanelCta {
            label: "[hover] Inspect".to_string(),
            candidate: Some(UiCommandCandidate::Focus(UiPanel::Inspect)),
        },
    ]
}

pub fn inventory_panel_ctas(observation: &Observation) -> Vec<PanelCta> {
    observation
        .inventory
        .iter()
        .map(|item| {
            let slot = item
                .equipped_slot
                .map(|slot| format!(" {:?}", slot))
                .unwrap_or_default();
            let identified = if item.identified { "" } else { " ?" };
            let label = format!(
                "{} {}{}{}",
                item.letter.0,
                item_label(item.kind),
                slot,
                identified
            );
            let candidate = primary_inventory_command(item)
                .filter(|intent| {
                    observation
                        .action_space
                        .commands
                        .contains(&ActionIntent::Command(*intent))
                })
                .map(UiCommandCandidate::Command);
            PanelCta { label, candidate }
        })
        .collect()
}

pub fn keyboard_baseline() -> Vec<(char, UiInputEvent)> {
    vec![
        (
            'y',
            UiInputEvent::Key(CommandIntent::Move(crate::core::Direction::NorthWest)),
        ),
        ('h', UiInputEvent::Key(CommandIntent::Move(Direction::West))),
        (
            'j',
            UiInputEvent::Key(CommandIntent::Move(Direction::South)),
        ),
        (
            'k',
            UiInputEvent::Key(CommandIntent::Move(Direction::North)),
        ),
        ('l', UiInputEvent::Key(CommandIntent::Move(Direction::East))),
        (
            'u',
            UiInputEvent::Key(CommandIntent::Move(crate::core::Direction::NorthEast)),
        ),
        (
            'b',
            UiInputEvent::Key(CommandIntent::Move(crate::core::Direction::SouthWest)),
        ),
        (
            'n',
            UiInputEvent::Key(CommandIntent::Move(crate::core::Direction::SouthEast)),
        ),
        ('.', UiInputEvent::Key(CommandIntent::Wait)),
        ('s', UiInputEvent::Key(CommandIntent::Search)),
        (',', UiInputEvent::Key(CommandIntent::Pickup)),
        ('i', UiInputEvent::Key(CommandIntent::ShowInventory)),
        ('>', UiInputEvent::Key(CommandIntent::Descend)),
        ('<', UiInputEvent::Key(CommandIntent::Ascend)),
        ('p', UiInputEvent::Key(CommandIntent::Pray)),
        ('S', UiInputEvent::SaveRequest),
        ('L', UiInputEvent::LoadRequest),
        ('Q', UiInputEvent::Quit),
    ]
}

pub fn key_to_candidate(key: char, observation: &Observation) -> Option<UiCommandCandidate> {
    if key == 'B' {
        return observation
            .legal_actions
            .contains(&CommandIntent::EnterBranch)
            .then_some(UiCommandCandidate::Command(CommandIntent::EnterBranch));
    }
    if matches!(
        key,
        'o' | 'c' | 'K' | 'w' | 'e' | 'q' | 'f' | 'd' | 't' | 'z' | 'r'
    ) {
        return Some(UiCommandCandidate::BeginAction(key));
    }
    if key == 'Q' {
        return Some(UiCommandCandidate::Quit);
    }
    if key == '?' {
        return Some(UiCommandCandidate::OpenCommands);
    }
    if key == 'M' {
        return Some(UiCommandCandidate::ShowMessages);
    }
    let llm_candidate = match key {
        'G' => Some(UiCommandCandidate::LlmNarrative),
        'A' => Some(UiCommandCandidate::LlmSuggest),
        'J' => Some(UiCommandCandidate::LlmJudge),
        'Y' => Some(UiCommandCandidate::LlmApply),
        'N' => Some(UiCommandCandidate::DismissLlmResult),
        'R' => Some(UiCommandCandidate::LlmRetry),
        _ => None,
    };
    if llm_candidate.is_some() {
        return llm_candidate;
    }

    let command_candidate = |intent: CommandIntent| {
        observation
            .action_space
            .commands
            .contains(&ActionIntent::Command(intent))
            .then_some(UiCommandCandidate::Command(intent))
    };

    let base = keyboard_baseline()
        .into_iter()
        .find(|(candidate, _)| *candidate == key)
        .and_then(|(_, event)| match event {
            UiInputEvent::Key(intent) => command_candidate(intent),
            UiInputEvent::SaveRequest => Some(UiCommandCandidate::Save),
            UiInputEvent::LoadRequest => Some(UiCommandCandidate::Load),
            UiInputEvent::Quit => Some(UiCommandCandidate::Quit),
            _ => unreachable!("keyboard baseline only emits key/save/load/quit"),
        });
    if base.is_some() {
        return base;
    }

    None
}

pub fn map_mouse_event(
    event: UiInputEvent,
    layout: TuiLayout,
    viewport: Viewport,
    observation: &Observation,
    inspect_presentation: InspectPresentation,
) -> Option<UiCommandCandidate> {
    match event {
        UiInputEvent::MouseHover { column, row } => viewport
            .terminal_to_world(column, row, layout.map)
            .map(UiCommandCandidate::Inspect),
        UiInputEvent::MouseClick { column, row } => {
            if let Some(pos) = viewport.terminal_to_world(column, row, layout.map) {
                let player = viewport.player_pos;
                let dx = pos.x - player.x;
                let dy = pos.y - player.y;
                direction_from_delta(dx, dy)
                    .map(|direction| UiCommandCandidate::Command(CommandIntent::Move(direction)))
                    .or(Some(UiCommandCandidate::Inspect(pos)))
            } else if contains(layout.inspect, column, row) {
                inspect_panel_click_candidate(
                    layout.inspect,
                    column,
                    row,
                    observation,
                    inspect_presentation,
                )
                .or(Some(UiCommandCandidate::Focus(UiPanel::Inspect)))
            } else if contains(layout.status, column, row) {
                Some(UiCommandCandidate::Focus(UiPanel::Status))
            } else if contains(layout.command, column, row) {
                command_panel_click_candidate(layout.command, column, row, observation)
                    .or(Some(UiCommandCandidate::Focus(UiPanel::Command)))
            } else {
                None
            }
        }
        UiInputEvent::FocusPanel(panel) => Some(UiCommandCandidate::Focus(panel)),
        UiInputEvent::SaveRequest => Some(UiCommandCandidate::Save),
        UiInputEvent::LoadRequest => Some(UiCommandCandidate::Load),
        UiInputEvent::Quit => Some(UiCommandCandidate::Quit),
        UiInputEvent::Key(intent) => Some(UiCommandCandidate::Command(intent)),
    }
}

pub fn llm_footer_click_candidate(
    command: Rect,
    column: u16,
    row: u16,
    footer: &str,
) -> Option<UiCommandCandidate> {
    if row != command.y.saturating_add(2) || column < command.x {
        return None;
    }
    let offset = column.saturating_sub(command.x) as usize;
    [
        ("[G] Narrative", UiCommandCandidate::LlmNarrative),
        ("[A] Suggest", UiCommandCandidate::LlmSuggest),
        ("[J] Judge", UiCommandCandidate::LlmJudge),
        ("[Y] Apply", UiCommandCandidate::LlmApply),
        ("[N] Dismiss", UiCommandCandidate::DismissLlmResult),
        ("[R] Retry", UiCommandCandidate::LlmRetry),
    ]
    .into_iter()
    .find_map(|(label, candidate)| {
        footer.match_indices(label).find_map(|(start, _)| {
            (offset >= start && offset < start + label.len()).then_some(candidate)
        })
    })
}

fn contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x && column < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

fn inspect_panel_click_candidate(
    inspect: Rect,
    column: u16,
    row: u16,
    observation: &Observation,
    presentation: InspectPresentation,
) -> Option<UiCommandCandidate> {
    if presentation != InspectPresentation::Inventory {
        return None;
    }
    let row_index = row.checked_sub(inspect.y + 1)? as usize;
    let offset = column.checked_sub(inspect.x)? as usize;
    let cta = inventory_panel_ctas(observation).get(row_index)?.clone();
    (offset < cta.label.chars().count())
        .then_some(cta.candidate)
        .flatten()
}

fn command_panel_click_candidate(
    command: Rect,
    column: u16,
    row: u16,
    observation: &Observation,
) -> Option<UiCommandCandidate> {
    if row != command.y.saturating_add(1) {
        return None;
    }
    let offset = column.checked_sub(command.x)? as usize;
    let mut start = 0usize;
    for cta in command_panel_ctas(observation) {
        let end = start + cta.label.chars().count();
        if (start..end).contains(&offset) {
            return cta.candidate;
        }
        start = end + 1;
    }
    None
}

fn primary_inventory_command(item: &ItemObservation) -> Option<CommandIntent> {
    match item.kind {
        crate::domain::item::ItemKind::Dagger => Some(CommandIntent::Wield { item: item.item }),
        crate::domain::item::ItemKind::ArmorLeather => {
            Some(CommandIntent::Wear { item: item.item })
        }
        crate::domain::item::ItemKind::PotionHealing => {
            Some(CommandIntent::Quaff { item: item.item })
        }
        crate::domain::item::ItemKind::ScrollReveal
        | crate::domain::item::ItemKind::ScrollIdentify
        | crate::domain::item::ItemKind::ScrollLevelTeleport => {
            Some(CommandIntent::Read { item: item.item })
        }
        _ => None,
    }
}

pub(crate) fn item_label(kind: crate::domain::item::ItemKind) -> &'static str {
    match kind {
        crate::domain::item::ItemKind::Dagger => "dagger",
        crate::domain::item::ItemKind::FoodRation => "food ration",
        crate::domain::item::ItemKind::PotionHealing => "healing potion",
        crate::domain::item::ItemKind::WandMagicMissile => "wand",
        crate::domain::item::ItemKind::ScrollReveal => "reveal scroll",
        crate::domain::item::ItemKind::ScrollIdentify => "identify scroll",
        crate::domain::item::ItemKind::ScrollLevelTeleport => "teleport scroll",
        crate::domain::item::ItemKind::Rock => "rock",
        crate::domain::item::ItemKind::ArmorLeather => "leather armor",
        crate::domain::item::ItemKind::CorpseJackal => "jackal corpse",
        crate::domain::item::ItemKind::AmuletAscension => "Amulet of Ascension",
    }
}

fn direction_from_delta(dx: i16, dy: i16) -> Option<crate::core::Direction> {
    match (dx, dy) {
        (-1, -1) => Some(crate::core::Direction::NorthWest),
        (0, -1) => Some(crate::core::Direction::North),
        (1, -1) => Some(crate::core::Direction::NorthEast),
        (-1, 0) => Some(crate::core::Direction::West),
        (1, 0) => Some(crate::core::Direction::East),
        (-1, 1) => Some(crate::core::Direction::SouthWest),
        (0, 1) => Some(crate::core::Direction::South),
        (1, 1) => Some(crate::core::Direction::SouthEast),
        _ => None,
    }
}

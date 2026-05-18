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
pub enum UiInputEvent {
    Key(CommandIntent),
    MouseHover { column: u16, row: u16 },
    MouseClick { column: u16, row: u16 },
    FocusPanel(UiPanel),
    SaveRequest,
    LoadRequest,
    Quit,
}

/// [v0.2.0] Phase 17: TUI 화면 전환과 명령 후보다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCommandCandidate {
    Command(CommandIntent),
    Inspect(Pos),
    Focus(UiPanel),
    Save,
    Load,
    Quit,
    /// [v0.2.0] Phase 17: Game Over에서 새 게임 시작
    NewRun,
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
        ('s', UiInputEvent::Key(CommandIntent::Search)),
        ('o', UiInputEvent::Key(CommandIntent::Open(Direction::East))),
        (
            'c',
            UiInputEvent::Key(CommandIntent::Close(Direction::East)),
        ),
        ('K', UiInputEvent::Key(CommandIntent::Kick(Direction::East))),
        (',', UiInputEvent::Key(CommandIntent::Pickup)),
        ('i', UiInputEvent::Key(CommandIntent::ShowInventory)),
        ('>', UiInputEvent::Key(CommandIntent::Descend)),
        ('<', UiInputEvent::Key(CommandIntent::Ascend)),
        ('p', UiInputEvent::Key(CommandIntent::Pray)),
        ('S', UiInputEvent::SaveRequest),
        ('L', UiInputEvent::LoadRequest),
        ('q', UiInputEvent::Quit),
    ]
}

pub fn key_to_candidate(key: char, observation: &Observation) -> Option<UiCommandCandidate> {
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

    let first_by = |pred: fn(&ItemObservation) -> bool| {
        observation
            .inventory
            .iter()
            .find(|item| pred(item))
            .map(|item| item.item)
    };

    match key {
        'w' => first_by(|item| item.kind == crate::domain::item::ItemKind::Dagger)
            .and_then(|item| command_candidate(CommandIntent::Wield { item })),
        'e' => first_by(|item| item.kind == crate::domain::item::ItemKind::ArmorLeather)
            .and_then(|item| command_candidate(CommandIntent::Wear { item })),
        'q' => first_by(|item| matches!(item.kind, crate::domain::item::ItemKind::PotionHealing))
            .and_then(|item| command_candidate(CommandIntent::Quaff { item })),
        'd' => observation
            .inventory
            .first()
            .and_then(|item| command_candidate(CommandIntent::Drop { item: item.item })),
        't' => first_by(|item| {
            matches!(
                item.kind,
                crate::domain::item::ItemKind::Dagger | crate::domain::item::ItemKind::Rock
            )
        })
        .and_then(|item| {
            command_candidate(CommandIntent::Throw {
                item,
                direction: Direction::East,
            })
        }),
        'z' => first_by(|item| item.kind == crate::domain::item::ItemKind::WandMagicMissile)
            .and_then(|item| {
                command_candidate(CommandIntent::Zap {
                    item,
                    direction: Direction::East,
                })
            }),
        'r' => first_by(|item| {
            matches!(
                item.kind,
                crate::domain::item::ItemKind::ScrollReveal
                    | crate::domain::item::ItemKind::ScrollIdentify
                    | crate::domain::item::ItemKind::ScrollLevelTeleport
            )
        })
        .and_then(|item| command_candidate(CommandIntent::Read { item })),
        _ => None,
    }
}

pub fn map_mouse_event(
    event: UiInputEvent,
    layout: TuiLayout,
    viewport: Viewport,
    observation: &Observation,
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
                inspect_panel_click_candidate(layout.inspect, row, observation)
                    .or(Some(UiCommandCandidate::Focus(UiPanel::Inspect)))
            } else if contains(layout.status, column, row) {
                Some(UiCommandCandidate::Focus(UiPanel::Status))
            } else if contains(layout.command, column, row) {
                command_panel_click_candidate(layout.command, column, observation)
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

fn contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x && column < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

fn inspect_panel_click_candidate(
    inspect: Rect,
    row: u16,
    observation: &Observation,
) -> Option<UiCommandCandidate> {
    let row_index = row.checked_sub(inspect.y + 1)? as usize;
    inventory_primary_candidates(observation)
        .get(row_index)
        .copied()
        .map(UiCommandCandidate::Command)
}

fn command_panel_click_candidate(
    command: Rect,
    column: u16,
    observation: &Observation,
) -> Option<UiCommandCandidate> {
    let left = column.checked_sub(command.x)?;
    if left < 16 {
        key_to_candidate('i', observation)
    } else if left < 32 {
        Some(UiCommandCandidate::Command(CommandIntent::Wait))
    } else if left < 48 {
        key_to_candidate('o', observation)
    } else if left < 64 {
        Some(UiCommandCandidate::Focus(UiPanel::Inspect))
    } else {
        None
    }
}

fn inventory_primary_candidates(observation: &Observation) -> Vec<CommandIntent> {
    observation
        .inventory
        .iter()
        .filter_map(primary_inventory_command)
        .filter(|intent| {
            observation
                .action_space
                .commands
                .contains(&ActionIntent::Command(*intent))
        })
        .collect()
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

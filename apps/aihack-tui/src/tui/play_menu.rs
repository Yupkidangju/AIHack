use super::input::{item_label, UiCommandCandidate as U};
use aihack_ai_contract::{CommandIntent as C, EntityId, Observation};
use ratatui::layout::Rect;

pub(super) const PAGE_SIZE: usize = 10;

pub(super) struct Entry {
    pub key: char,
    pub label: String,
    pub candidate: U,
}

pub(super) struct PlayMenu {
    pub title: String,
    pub entries: Vec<Entry>,
    pub page: usize,
}

impl PlayMenu {
    pub fn new(title: &str, entries: Vec<Entry>) -> Self {
        Self {
            title: title.into(),
            entries,
            page: 0,
        }
    }

    pub fn visible(&self) -> &[Entry] {
        let start = (self.page * PAGE_SIZE).min(self.entries.len());
        &self.entries[start..(start + PAGE_SIZE).min(self.entries.len())]
    }

    pub fn area(&self, width: u16, height: u16) -> Rect {
        let w = width.min(72);
        let h = (self.visible().len() as u16 + 3).min(height);
        Rect::new((width - w) / 2, (height - h) / 2, w, h)
    }

    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<_> = self
            .visible()
            .iter()
            .map(|entry| format!("[{}] {}", entry.key, entry.label))
            .collect();
        lines.push(format!(
            "Page {}/{}  [<] Prev [>] Next",
            self.page + 1,
            self.entries.len().div_ceil(PAGE_SIZE).max(1)
        ));
        lines.push("[Esc] Cancel / Close".into());
        lines
    }

    pub fn key(&self, key: char) -> Option<U> {
        match key {
            '<' => Some(U::MenuPage(false)),
            '>' => Some(U::MenuPage(true)),
            _ => self
                .entries
                .iter()
                .find(|entry| entry.key == key)
                .map(|entry| entry.candidate),
        }
    }

    pub fn click(&self, width: u16, height: u16, x: u16, y: u16) -> Option<U> {
        let area = self.area(width, height);
        if !super::rect_contains(area, x, y) {
            return None;
        }
        let row = y.checked_sub(area.y + 1)? as usize;
        if let Some(entry) = self.visible().get(row) {
            return Some(entry.candidate);
        }
        if row == self.visible().len() {
            let line = &self.lines()[row];
            let column = usize::from(x - area.x);
            for (label, next) in [("[<] Prev", false), ("[>] Next", true)] {
                let start = line.find(label)?;
                if (start..start + label.len()).contains(&column) {
                    return Some(U::MenuPage(next));
                }
            }
            return None;
        }
        (row == self.visible().len() + 1).then_some(U::CloseOverlay)
    }
}

fn belongs(command: C, key: char) -> bool {
    matches!(
        (key, command),
        ('o', C::Open(_))
            | ('c', C::Close(_))
            | ('K', C::Kick(_))
            | ('w', C::Wield { .. })
            | ('e', C::Wear { .. })
            | ('q', C::Quaff { .. })
            | ('f', C::Eat { .. })
            | ('d', C::Drop { .. })
            | ('t', C::Throw { .. })
            | ('z', C::Zap { .. })
            | ('r', C::Read { .. })
    )
}

fn item_of(command: C) -> Option<EntityId> {
    match command {
        C::Wield { item }
        | C::Wear { item }
        | C::Quaff { item }
        | C::Eat { item }
        | C::Drop { item }
        | C::Read { item }
        | C::Throw { item, .. }
        | C::Zap { item, .. } => Some(item),
        _ => None,
    }
}

pub(super) fn action_menu(
    observation: &Observation,
    key: char,
    selected: Option<EntityId>,
) -> PlayMenu {
    let commands: Vec<_> = observation
        .legal_actions
        .iter()
        .copied()
        .filter(|command| belongs(*command, key))
        .collect();
    let mut entries = Vec::new();
    if matches!(key, 'o' | 'c' | 'K') || selected.is_some() {
        for command in commands {
            if selected.is_some() && item_of(command) != selected {
                continue;
            }
            let direction = match command {
                C::Open(d)
                | C::Close(d)
                | C::Kick(d)
                | C::Throw { direction: d, .. }
                | C::Zap { direction: d, .. } => d,
                _ => continue,
            };
            let key = "ykulnjbh"
                .chars()
                .find(|k| super::direction_for_key(*k) == Some(direction))
                .unwrap();
            entries.push(Entry {
                key,
                label: format!("{direction:?}"),
                candidate: U::Command(command),
            });
        }
        return PlayMenu::new("Choose direction", entries);
    }
    for item in &observation.inventory {
        if let Some(command) = commands.iter().find(|c| item_of(**c) == Some(item.item)) {
            entries.push(Entry {
                key: item.letter.0,
                label: item_label(item.kind).into(),
                candidate: if matches!(key, 't' | 'z') {
                    U::ChooseItem {
                        action: key,
                        item: item.item,
                    }
                } else {
                    U::Command(*command)
                },
            });
        }
    }
    PlayMenu::new(&format!("Choose item [{key}]"), entries)
}

pub(super) fn inventory_menu(observation: &Observation) -> PlayMenu {
    PlayMenu::new(
        "INVENTORY",
        observation
            .inventory
            .iter()
            .map(|item| Entry {
                key: item.letter.0,
                label: item_label(item.kind).into(),
                candidate: U::ChooseItem {
                    action: 'i',
                    item: item.item,
                },
            })
            .collect(),
    )
}

pub(super) fn item_menu(observation: &Observation, item: EntityId) -> PlayMenu {
    PlayMenu::new(
        "Item action",
        [
            ('w', "Wield"),
            ('e', "Wear"),
            ('q', "Quaff"),
            ('f', "Eat"),
            ('r', "Read"),
            ('d', "Drop"),
            ('t', "Throw"),
            ('z', "Zap"),
        ]
        .into_iter()
        .filter_map(|(key, label)| {
            let command = observation
                .legal_actions
                .iter()
                .copied()
                .find(|c| item_of(*c) == Some(item) && belongs(*c, key))?;
            Some(Entry {
                key,
                label: label.into(),
                candidate: if matches!(key, 't' | 'z') {
                    U::ChooseItem { action: key, item }
                } else {
                    U::Command(command)
                },
            })
        })
        .collect(),
    )
}

pub(super) fn commands_menu(observation: &Observation) -> PlayMenu {
    let mut entries: Vec<_> = [
        ('s', "Search", C::Search),
        (',', "Pickup", C::Pickup),
        ('p', "Pray", C::Pray),
        ('D', "Descend", C::Descend),
        ('U', "Ascend", C::Ascend),
        ('.', "Wait", C::Wait),
    ]
    .into_iter()
    .filter(|(_, _, c)| observation.legal_actions.contains(c))
    .map(|(key, label, c)| Entry {
        key,
        label: label.into(),
        candidate: U::Command(c),
    })
    .collect();
    if observation.legal_actions.contains(&C::EnterBranch) {
        entries.push(Entry {
            key: 'B',
            label: "Enter Mines".into(),
            candidate: U::Command(C::EnterBranch),
        });
    }
    for (key, label) in [
        ('o', "Open"),
        ('c', "Close"),
        ('K', "Kick"),
        ('q', "Quaff"),
        ('d', "Drop"),
        ('t', "Throw"),
        ('z', "Zap"),
        ('w', "Wield"),
        ('e', "Wear"),
        ('f', "Eat"),
        ('r', "Read"),
    ] {
        entries.push(Entry {
            key,
            label: label.into(),
            candidate: U::BeginAction(key),
        });
    }
    for (key, label, candidate) in [
        ('i', "Inventory", U::Command(C::ShowInventory)),
        ('S', "Save", U::Save),
        ('L', "Load", U::Load),
        ('Q', "Quit", U::Quit),
        ('G', "Narrative", U::LlmNarrative),
        ('A', "Suggestion", U::LlmSuggest),
        ('J', "Judge", U::LlmJudge),
        ('F', "Debug", U::ToggleDebug),
    ] {
        entries.push(Entry {
            key,
            label: label.into(),
            candidate,
        });
    }
    entries.push(Entry {
        key: 'M',
        label: "Messages".into(),
        candidate: U::ShowMessages,
    });
    PlayMenu::new("COMMANDS", entries)
}

use crate::{
    core::{EntityObservation, GameEvent, Observation, Pos},
    domain::{entity::EntityKind, item::ItemKind, tile::TileKind},
    ui::tui::UiPanel,
};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

pub struct TextPanel<'a> {
    pub title: &'a str,
    pub lines: Vec<String>,
}

impl Widget for TextPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_char('-');
        }
        let mut write_line = |row: u16, text: &str| {
            for (i, ch) in text.chars().take(area.width as usize).enumerate() {
                buf[(area.x + i as u16, row)].set_char(ch);
            }
        };
        write_line(area.y, self.title);
        for (idx, line) in self.lines.iter().enumerate() {
            let row = area.y + 1 + idx as u16;
            if row >= area.y + area.height {
                break;
            }
            write_line(row, line);
        }
    }
}

pub fn status_lines(observation: &Observation) -> Vec<String> {
    let hp_ratio = if observation.player.max_hp == 0 {
        0.0
    } else {
        observation.player.hp as f32 / observation.player.max_hp as f32
    };
    let danger = if hp_ratio <= 0.30 {
        "ALERT low hp"
    } else {
        "stable"
    };
    vec![
        format!("turn {}", observation.turn),
        format!(
            "hp {}/{} {}",
            observation.player.hp, observation.player.max_hp, danger
        ),
        format!(
            "level {}:{}",
            match observation.current_level.branch {
                crate::core::BranchId::Main => "Main",
            },
            observation.current_level.depth
        ),
        format!(
            "pos {},{}",
            observation.player_pos.x, observation.player_pos.y
        ),
    ]
}

pub fn command_lines(observation: &Observation, focused_panel: UiPanel) -> Vec<String> {
    let open_hint = if observation
        .legal_actions
        .iter()
        .any(|intent| matches!(intent, crate::core::CommandIntent::Open(_)))
    {
        "[o] Open"
    } else {
        "[o] Open*"
    };
    vec![
        format!(
            "[i] Inventory [. ] Wait {} [hover] Inspect [click] Focus",
            open_hint
        ),
        format!("focus {:?}", focused_panel),
    ]
}

pub fn log_lines(observation: &Observation, narrative_lines: &[String]) -> Vec<String> {
    let mut out = recent_priority_messages(observation);
    if out.len() < 4 {
        out.extend(narrative_lines.iter().take(4 - out.len()).cloned());
    }
    out
}

pub fn inspect_lines(
    observation: &Observation,
    hovered: Option<Pos>,
    focused_panel: UiPanel,
    decision_lines: &[String],
) -> Vec<String> {
    if let Some(pos) = hovered {
        return hovered_inspect_lines(observation, pos);
    }

    let mut lines = inventory_lines(observation);
    if lines.len() < 4 {
        lines.extend(decision_lines.iter().take(4 - lines.len()).cloned());
    }
    lines.push(format!("focus {:?}", focused_panel));
    lines
}

fn hovered_inspect_lines(observation: &Observation, pos: Pos) -> Vec<String> {
    let tile = observation
        .visible_tiles
        .iter()
        .find(|tile| tile.pos == pos)
        .map(|tile| tile_label(tile.tile))
        .unwrap_or("unknown");
    let entity = observation
        .visible_entities
        .iter()
        .find(|entity| entity.pos == pos)
        .map(entity_line)
        .unwrap_or_else(|| "entity none".to_string());
    vec![
        format!("hover {},{}", pos.x, pos.y),
        format!("tile {}", tile),
        entity,
        "read-only inspect".to_string(),
    ]
}

fn inventory_lines(observation: &Observation) -> Vec<String> {
    observation
        .inventory
        .iter()
        .take(4)
        .map(|item| {
            let slot = item
                .equipped_slot
                .map(|slot| format!(" {:?}", slot))
                .unwrap_or_default();
            let identified = if item.identified { "" } else { " ?" };
            format!(
                "{} {}{}{}",
                item.letter.0,
                item_label(item.kind),
                slot,
                identified
            )
        })
        .collect()
}

fn recent_priority_messages(observation: &Observation) -> Vec<String> {
    let mut lines = observation
        .last_events
        .iter()
        .rev()
        .take(3)
        .map(event_line)
        .collect::<Vec<_>>();
    if observation.player.max_hp > 0 && observation.player.hp * 10 <= observation.player.max_hp * 3
    {
        lines.insert(0, "! hp critical".to_string());
    }
    lines
}

fn event_line(event: &GameEvent) -> String {
    match event {
        GameEvent::AttackResolved {
            hit: true, damage, ..
        } => format!("! hit for {damage}"),
        GameEvent::AttackResolved { hit: false, .. } => "~ miss".to_string(),
        GameEvent::ItemPickedUp { letter, .. } => format!("+ pickup {}", letter.0),
        GameEvent::DoorChanged { to, .. } => format!("> door {to:?}"),
        GameEvent::CommandRejected { reason } => format!("x {reason}"),
        GameEvent::TrapTriggered { damage, .. } => format!("! trap {damage}"),
        _ => format!("{event:?}"),
    }
}

fn entity_line(entity: &EntityObservation) -> String {
    let hp = entity.hp.map(|hp| format!(" hp {hp}")).unwrap_or_default();
    format!("entity {}{}", entity_kind_label(entity.kind), hp)
}

fn entity_kind_label(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Player => "player",
        EntityKind::Monster(crate::domain::monster::MonsterKind::Jackal) => "jackal",
        EntityKind::Monster(crate::domain::monster::MonsterKind::Goblin) => "goblin",
        EntityKind::Monster(crate::domain::monster::MonsterKind::FloatingEye) => "floating eye",
        EntityKind::Item(kind) => item_label(kind),
    }
}

fn tile_label(tile: TileKind) -> &'static str {
    match tile {
        TileKind::Wall => "wall",
        TileKind::Floor => "floor",
        TileKind::Door(_) => "door",
        TileKind::StairsDown => "stairs down",
        TileKind::StairsUp => "stairs up",
        TileKind::Trap(_) => "trap",
        TileKind::HiddenDoor => "hidden door",
        TileKind::HiddenTrap(_) => "hidden trap",
    }
}

fn item_label(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Dagger => "dagger",
        ItemKind::FoodRation => "food ration",
        ItemKind::PotionHealing => "healing potion",
        ItemKind::WandMagicMissile => "wand",
        ItemKind::ScrollReveal => "reveal scroll",
        ItemKind::ScrollIdentify => "identify scroll",
        ItemKind::ScrollLevelTeleport => "teleport scroll",
        ItemKind::Rock => "rock",
        ItemKind::ArmorLeather => "leather armor",
        ItemKind::CorpseJackal => "jackal corpse",
    }
}

// [v0.2.0] Phase 17: 화면별 렌더링 함수들

/// Title 화면 텍스트 라인 생성
pub fn title_lines() -> Vec<String> {
    vec![
        "".to_string(),
        "".to_string(),
        "              AIHack".to_string(),
        "".to_string(),
        "".to_string(),
        "         Press Enter to Start".to_string(),
        "".to_string(),
        "              L - Load Game".to_string(),
        "              Q - Quit".to_string(),
    ]
}

/// Character Creation 화면 텍스트 라인 생성
pub fn character_creation_lines() -> Vec<String> {
    vec![
        "".to_string(),
        "".to_string(),
        "         Character Creation".to_string(),
        "".to_string(),
        "         Class: Adventurer".to_string(),
        "         HP: 16/16".to_string(),
        "         Strength: 10".to_string(),
        "         Dexterity: 10".to_string(),
        "         AC: 0".to_string(),
        "".to_string(),
        "         Press Enter to confirm".to_string(),
        "         Esc - Back to Title".to_string(),
    ]
}

/// Game Over 화면 텍스트 라인 생성
/// designs.md 11 기준: 사망 원인, turn, depth, defeated, score, seed, N/Q/E
pub fn game_over_lines(
    cause_text: &str,
    turn: u64,
    depth: i16,
    defeated: u32,
    score: i32,
    seed: u64,
) -> Vec<String> {
    vec![
        "".to_string(),
        "                   GAME OVER".to_string(),
        format!("              {}", cause_text),
        "".to_string(),
        format!("  Turn: {:>12}  Depth: {:>12}", turn, depth),
        format!("  Defeated: {:>8}  Score: {:>12}", defeated, score),
        format!("  Seed: {:>12}", seed),
        "".to_string(),
        "         [N] New Run    [Q] Quit".to_string(),
    ]
}

/// AwaitingDirection 상태 라인 생성
pub fn awaiting_direction_lines(action_name: &str) -> Vec<String> {
    vec![
        "".to_string(),
        format!("Choose direction for {}: [hjklyubn]", action_name),
        "Esc to cancel".to_string(),
    ]
}

/// AwaitingInventorySelection 상태 라인 생성
pub fn awaiting_inventory_lines(action_name: &str) -> Vec<String> {
    vec![
        "".to_string(),
        format!("Choose item to {}", action_name),
        "Enter letter or Esc to cancel".to_string(),
    ]
}

/// MorePrompt 상태 라인 생성
pub fn more_prompt_lines() -> Vec<String> {
    vec![
        "".to_string(),
        "--More--".to_string(),
        "Press any key to continue".to_string(),
    ]
}

// [v0.2.0] Phase 18: Debug Observation 패널

/// [v0.2.0] Phase 18: debug observation 패널 텍스트 라인 생성.
/// designs.md 10 기준: schema_version, seed, turn, snapshot_hash, legal_actions,
/// visible tile/entity 수, last_events 등을 표시한다.
/// [v0.2.0] Phase 18: debug observation 패널 텍스트 라인 생성.
/// designs.md 10 기준: schema_version, seed, turn, run_state, player 상태,
/// visible tile/entity 수, inventory 수, action_space 수, last_events, legal_actions를 표시한다.
pub fn debug_observation_lines(observation: &Observation) -> Vec<String> {
    let mut lines = vec![
        format!("schema_version: {}", observation.schema_version),
        format!("seed: {}", observation.seed),
        format!("turn: {}", observation.turn),
        format!(
            "level: {:?}:{}",
            observation.current_level.branch, observation.current_level.depth
        ),
        format!("run_state: {:?}", observation.run_state),
        format!(
            "player_pos: ({}, {})",
            observation.player_pos.x, observation.player_pos.y
        ),
        format!(
            "player_hp: {}/{}",
            observation.player.hp, observation.player.max_hp
        ),
        format!("hunger: {}", observation.player.hunger),
        format!("luck: {}", observation.player.luck),
        format!("prayer_cooldown: {}", observation.player.prayer_cooldown),
        format!("paralysis_turns: {}", observation.player.paralysis_turns),
        format!("hallucinating: {}", observation.player.hallucinating),
        format!("visible_tiles: {}", observation.visible_tiles.len()),
        format!("visible_entities: {}", observation.visible_entities.len()),
        format!("inventory: {} items", observation.inventory.len()),
        format!(
            "action_space: {} actions",
            observation.action_space.commands.len()
        ),
        format!("last_events: {} events", observation.last_events.len()),
    ];

    lines.push("legal_actions:".to_string());
    for action in observation.legal_actions.iter().take(20) {
        lines.push(format!("  {:?}", action));
    }

    lines.push("last 10 events:".to_string());
    for event in observation.last_events.iter().rev().take(10).rev() {
        lines.push(format!("  {:?}", event));
    }

    lines
}

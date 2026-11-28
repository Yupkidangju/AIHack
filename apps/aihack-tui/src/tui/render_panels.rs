use crate::{
    core::{EntityObservation, GameEvent, Observation, Pos},
    domain::{entity::EntityKind, tile::TileKind},
    ui::tui::UiPanel,
};
use aihack_llm::config::LlmRequestKind;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use super::{
    input::{command_panel_ctas, inventory_panel_ctas, item_label},
    LlmUiStatus, UiTheme,
};

pub struct TextPanel<'a> {
    pub title: &'a str,
    pub lines: Vec<String>,
}

impl Widget for TextPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_text_panel(area, buf, self.title, &self.lines, UiTheme::standard());
    }
}

pub struct ThemedTextPanel<'a> {
    pub title: &'a str,
    pub lines: Vec<String>,
    pub theme: UiTheme,
}

impl Widget for ThemedTextPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_text_panel(area, buf, self.title, &self.lines, self.theme);
    }
}

fn render_text_panel(area: Rect, buf: &mut Buffer, title: &str, lines: &[String], theme: UiTheme) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let body_style = Style::default().fg(theme.fg).bg(theme.bg);
    let title_style = Style::default()
        .fg(theme.accent)
        .bg(theme.bg)
        .add_modifier(Modifier::BOLD);
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            buf[(x, y)].reset();
            buf[(x, y)].set_style(body_style);
        }
    }
    for x in area.x..area.x + area.width {
        buf[(x, area.y)].set_char('-').set_style(title_style);
    }
    let mut write_line = |row: u16, text: &str| {
        for (i, ch) in text.chars().take(area.width as usize).enumerate() {
            buf[(area.x + i as u16, row)].set_char(ch);
        }
    };
    if area.height == 1 && title == "LOG" {
        write_line(
            area.y,
            lines.first().map(String::as_str).unwrap_or("No events."),
        );
        return;
    }
    write_line(area.y, title);
    for (idx, line) in lines.iter().enumerate() {
        let row = area.y + 1 + idx as u16;
        if row >= area.y + area.height {
            break;
        }
        write_line(row, line);
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
    let mut status_flags = Vec::new();
    if observation.player.blind_turns > 0 {
        status_flags.push("BLIND");
    }
    if observation.player.blindfolded {
        status_flags.push("BLINDFOLD");
    }
    if observation.player.confused_turns > 0 {
        status_flags.push("CONF");
    }
    if observation.player.stunned_turns > 0 {
        status_flags.push("STUN");
    }
    if observation.player.hallucinating {
        status_flags.push("HALL");
    }
    if observation.player.fast_turns > 0 {
        status_flags.push("FAST");
    }
    if observation.player.levitation_turns > 0 {
        status_flags.push("LEV");
    }
    if observation.player.invisible_turns > 0 {
        status_flags.push("INV");
    }
    let flags_suffix = if status_flags.is_empty() {
        String::new()
    } else {
        format!(" {}", status_flags.join(","))
    };
    let special_suffix = observation
        .campaign
        .as_ref()
        .map(|c| {
            let mut s = String::new();
            if let Some(id) = &c.special_level_id {
                s.push_str(&format!(" ({id})"));
            }
            if let Some(align) = c.on_high_altar {
                s.push_str(&format!(" HIGH ALTAR({})", align.short_label()));
            } else if c.on_magic_portal {
                s.push_str(" PORTAL");
            }
            s
        })
        .unwrap_or_default();
    let mut lines = vec![
        format!("turn {}{flags_suffix}", observation.turn),
        format!(
            "hp {}/{} {}{}{}",
            observation.player.hp,
            observation.player.max_hp,
            if observation.player.max_mp > 0 {
                format!(
                    "mp {}/{} ",
                    observation.player.mp, observation.player.max_mp
                )
            } else {
                String::new()
            },
            danger,
            if observation.player.poison_turns > 0 {
                format!(" POIS:{}", observation.player.poison_turns)
            } else {
                String::new()
            }
        ),
        format!(
            "level {}:{}{} AC:{}",
            observation.current_level.branch.display_name(),
            observation.current_level.depth,
            special_suffix,
            observation.player.ac
        ),
        format!(
            "food {}{}{}",
            observation.player.hunger,
            hunger_badge(observation.player.hunger),
            if observation.campaign.as_ref().is_some_and(|c| c.has_amulet) {
                " Amulet!"
            } else {
                ""
            }
        ),
    ];
    if let Some(campaign) = &observation.campaign {
        lines[3] = format!(
            "food{}{} w{}/{}{}",
            observation.player.hunger,
            hunger_badge(observation.player.hunger),
            campaign.carried_weight,
            campaign.carrying_capacity,
            if campaign.has_amulet { " A" } else { "" }
        );
        let badges = aihack_ai_contract::Resistances(observation.player.resistances).badge_string();
        let res = if !badges.is_empty() {
            format!(" RES:{badges}")
        } else {
            match (
                observation.player.fire_resistant,
                observation.player.poison_resistant,
            ) {
                (true, true) => " RES:FP".to_string(),
                (true, false) => " RES:F".to_string(),
                (false, true) => " RES:P".to_string(),
                (false, false) => String::new(),
            }
        };
        let ready_skills = campaign.skills.iter().filter(|s| s.ready).count();
        let align_code = campaign.alignment.map(|a| a.short_label()).unwrap_or("?");
        let race_name = campaign.race.map(|r| r.as_str()).unwrap_or("Human");
        let anger_badge = if campaign.god_anger > 0 {
            format!(" ANG:{}", campaign.god_anger)
        } else {
            String::new()
        };
        let invoke_badge = if campaign.artifact_invoke_cooldown > 0 {
            format!(" INV:{}", campaign.artifact_invoke_cooldown)
        } else {
            String::new()
        };
        let enhance_badge = if ready_skills > 0 {
            format!(" ENH:{ready_skills}")
        } else {
            String::new()
        };
        let debt_badge = if observation.player.shop_debt > 0 {
            format!(" DEBT:{}", observation.player.shop_debt)
        } else {
            String::new()
        };
        lines.push(format!(
            "[{}] {} {} L{} X{} G{}{}{}{}{}{}",
            align_code,
            race_name,
            campaign.role.display_name(),
            campaign.level,
            campaign.xp,
            observation.player.gold,
            res,
            anger_badge,
            invoke_badge,
            enhance_badge,
            debt_badge
        ));
        if let Some(attrs) = campaign.attributes {
            lines.push(format!(
                "St:{} Dx:{} Co:{} In:{} Wi:{} Ch:{}",
                attrs.str_, attrs.dex, attrs.con, attrs.int, attrs.wis, attrs.cha
            ));
        }
    }
    lines
}

pub fn command_lines(observation: &Observation, focused_panel: UiPanel) -> Vec<String> {
    vec![
        command_panel_ctas(observation)
            .into_iter()
            .map(|cta| cta.label)
            .collect::<Vec<_>>()
            .join(" "),
        format!("focus {:?}", focused_panel),
    ]
}

pub fn llm_status_lines(status: &LlmUiStatus) -> Vec<String> {
    let (badge, body) = match status {
        LlmUiStatus::Disabled => ("LLM: OFF", "Local LLM disabled; core play is available."),
        LlmUiStatus::Connecting => ("LLM: ...", "Connecting to loopback provider."),
        LlmUiStatus::Ready => ("LLM: READY", "[G] Narrative [A] Suggest [J] Judge"),
        LlmUiStatus::Pending { kind, request_id } => {
            return vec![
                "LLM: WAIT".to_string(),
                format!(
                    "{} request {} [N] Dismiss",
                    request_kind_label(kind),
                    request_id.chars().take(8).collect::<String>()
                ),
            ];
        }
        LlmUiStatus::Busy => (
            "LLM: BUSY",
            "Queue capacity 16 reached. [R] Retry [N] Dismiss",
        ),
        LlmUiStatus::Timeout { kind } => {
            return vec![
                "LLM: TIMEOUT".to_string(),
                format!(
                    "{} request timed out. [R] Retry [N] Dismiss",
                    request_kind_label(kind)
                ),
            ];
        }
        LlmUiStatus::Unavailable => (
            "LLM: DOWN",
            "Loopback provider unavailable. [R] Retry [N] Dismiss",
        ),
        LlmUiStatus::Invalid => ("LLM: INVALID", "Response or input rejected. [N] Dismiss"),
        LlmUiStatus::Stale => (
            "LLM: STALE",
            "Session changed; response discarded. [N] Dismiss",
        ),
    };
    vec![badge.to_string(), body.to_string()]
}

pub fn llm_footer_line(
    status: &LlmUiStatus,
    has_valid_suggestion: bool,
    has_result: bool,
) -> String {
    if has_valid_suggestion {
        "[Y] Apply [N] Dismiss".to_string()
    } else if !matches!(status, LlmUiStatus::Ready | LlmUiStatus::Disabled) {
        llm_status_lines(status).get(1).cloned().unwrap_or_default()
    } else if has_result {
        "[N] Dismiss".to_string()
    } else {
        llm_status_lines(status).get(1).cloned().unwrap_or_default()
    }
}

pub fn soft_input_lines(input: &str) -> Vec<String> {
    vec![
        "Describe an attempted action for a presentation-only judgment.".to_string(),
        input.to_string(),
        format!("{}/240 Unicode characters", input.chars().count()),
        "[Enter] Submit  [Esc] Cancel".to_string(),
    ]
}

fn request_kind_label(kind: &LlmRequestKind) -> &'static str {
    match kind {
        LlmRequestKind::Narrative => "Narrative",
        LlmRequestKind::Decision => "Suggestion",
        LlmRequestKind::SoftAdjudication { .. } => "Judgment",
        _ => "Request",
    }
}

fn hunger_badge(nutrition: i16) -> &'static str {
    match nutrition {
        i16::MIN..=0 => " FAINT",
        1..=50 => " WEAK",
        51..=150 => " HUNGRY",
        _ => "",
    }
}

pub fn log_lines(observation: &Observation, narrative_lines: &[String]) -> Vec<String> {
    let mut out = recent_priority_messages(observation);
    if out.len() < 4 {
        out.extend(narrative_lines.iter().take(4 - out.len()).cloned());
    }
    if observation.player.hallucinating {
        if out.is_empty() {
            out.push(aihack_runtime::systems::score::hallucination_message(
                "The dungeon",
                true,
            ));
        } else {
            out = out
                .into_iter()
                .map(|line| aihack_runtime::systems::score::hallucination_message(&line, true))
                .collect();
        }
    }
    out
}

pub fn inspect_lines(
    observation: &Observation,
    hovered: Option<Pos>,
    focused_panel: UiPanel,
    decision_lines: &[String],
) -> Vec<String> {
    let mut lines = match super::input::inspect_presentation(hovered, decision_lines) {
        super::input::InspectPresentation::Hover => hovered_inspect_lines(
            observation,
            hovered.expect("hover presentation has a position"),
        ),
        super::input::InspectPresentation::Decision => {
            decision_lines.iter().take(4).cloned().collect::<Vec<_>>()
        }
        super::input::InspectPresentation::Inventory => inventory_lines(observation),
    };
    lines.push(format!("focus {:?}", focused_panel));
    lines
}

fn hovered_inspect_lines(observation: &Observation, pos: Pos) -> Vec<String> {
    let tile_obs = observation
        .visible_tiles
        .iter()
        .find(|tile| tile.pos == pos);
    let tile = tile_obs
        .map(|tile| tile_label(tile.tile))
        .unwrap_or("unknown");
    let entity = observation
        .visible_entities
        .iter()
        .find(|entity| entity.pos == pos)
        .map(entity_line)
        .unwrap_or_else(|| "entity none".to_string());
    let mut lines = vec![
        format!("hover {},{}", pos.x, pos.y),
        format!("tile {}", tile),
        entity,
    ];
    if let Some(align) = tile_obs.and_then(|t| t.altar_alignment) {
        lines.push(format!("altar alignment: {}", align.as_str()));
    }
    if let Some(engraving) = tile_obs.and_then(|t| t.engraving.as_deref()) {
        lines.push(format!("engraving \"{engraving}\""));
    } else {
        lines.push("read-only inspect".to_string());
    }
    lines
}

fn inventory_lines(observation: &Observation) -> Vec<String> {
    inventory_panel_ctas(observation)
        .into_iter()
        .map(|cta| cta.label)
        .collect()
}

pub fn inventory_overlay_lines(observation: &Observation) -> Vec<String> {
    let mut lines = inventory_lines(observation);
    if lines.is_empty() {
        lines.push("Inventory is empty.".to_string());
    }
    lines.push("Select an item letter; Esc closes inventory.".to_string());
    lines
}

pub fn storage_error_lines(operation: super::StorageOperation) -> Vec<String> {
    let operation = match operation {
        super::StorageOperation::Save => "save",
        super::StorageOperation::Load => "load",
        super::StorageOperation::NewRun => "new run",
    };
    vec![
        format!("Unable to complete {operation}."),
        "The quick-save store rejected the operation; no path detail is shown.".to_string(),
        "Press Esc or any key to continue.".to_string(),
    ]
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

pub(super) fn event_line(event: &GameEvent) -> String {
    use aihack_ai_contract::MessagePriority;
    match event {
        GameEvent::TurnStarted { turn } => format!(". turn {turn}"),
        GameEvent::Waited { turn } => format!(". wait (turn {turn})"),
        GameEvent::EntityMoved { to, .. } => format!("> moved ({},{})", to.x, to.y),
        GameEvent::LevelChanged { to, .. } => format!("> level {:?}:{}", to.branch, to.depth),
        GameEvent::DoorChanged { to, .. } => format!("> door {to:?}"),
        GameEvent::TileRevealed { pos, tile } => {
            format!("? revealed {tile:?} ({},{})", pos.x, pos.y)
        }
        GameEvent::ItemPickedUp { letter, .. } => format!("+ pickup {}", letter.0),
        GameEvent::ItemDropped { pos, .. } => format!("- drop ({},{})", pos.x, pos.y),
        GameEvent::ItemEquipped { slot, .. } => format!("= equip {slot:?}"),
        GameEvent::ItemConsumed { .. } => "- consumed item".to_string(),
        GameEvent::EntityHealed {
            amount, hp_after, ..
        } => format!("+ healed {amount} (hp {hp_after})"),
        GameEvent::TrapTriggered { damage, .. } => format!("! trap {damage}"),
        GameEvent::ItemThrown { to, .. } => format!("/ threw ({},{})", to.x, to.y),
        GameEvent::WandZapped {
            direction,
            charges_after,
            ..
        } => format!("/ zap {direction:?} ({charges_after})"),
        GameEvent::ScrollRead { .. } => "* read scroll".to_string(),
        GameEvent::ItemIdentified { .. } => "* identified item".to_string(),
        GameEvent::DoorKicked { pos } => format!("! kick ({},{})", pos.x, pos.y),
        GameEvent::PassiveAttackTriggered { .. } => "! passive strike".to_string(),
        GameEvent::PrayerOffered { cooldown_after, .. } => {
            format!("* prayed (cd {cooldown_after})")
        }
        GameEvent::SpellCast {
            spell, mp_after, ..
        } => format!("* cast {spell:?} (mp {mp_after})"),
        GameEvent::AttackResolved {
            hit: true, damage, ..
        } => format!("! hit for {damage}"),
        GameEvent::AttackResolved { hit: false, .. } => "~ miss".to_string(),
        GameEvent::EntityDied { cause, .. } => format!("! died ({cause:?})"),
        GameEvent::CommandRejected { reason } => format!("x {reason}"),
        GameEvent::Message { priority, text } => match priority {
            MessagePriority::Danger | MessagePriority::Warning => format!("! {text}"),
            MessagePriority::Info | MessagePriority::Low => text.clone(),
        },
    }
}

fn entity_line(entity: &EntityObservation) -> String {
    let unpaid_suffix = if entity.unpaid { " (unpaid)" } else { "" };
    if let Some(appearance) = &entity.appearance {
        return format!("entity {appearance}{unpaid_suffix}");
    }
    let label = entity
        .name
        .clone()
        .unwrap_or_else(|| entity_kind_label(entity.kind));
    let faction_tag = match entity.faction {
        Some(aihack_ai_contract::Faction::Pet) => " [pet]",
        Some(aihack_ai_contract::Faction::Peaceful) => " [peaceful]",
        Some(aihack_ai_contract::Faction::Shopkeeper) => " [shopkeeper]",
        Some(aihack_ai_contract::Faction::Hostile) => " [hostile]",
        _ => "",
    };
    let hp = entity.hp.map(|hp| format!(" hp {hp}")).unwrap_or_default();
    format!("entity {label}{faction_tag}{hp}{unpaid_suffix}")
}

fn entity_kind_label(kind: EntityKind) -> String {
    match kind {
        EntityKind::Player => "player".to_string(),
        EntityKind::Monster(kind) => aihack_runtime::monster_display_name(kind),
        EntityKind::Item(kind) => item_label(kind),
    }
}

fn tile_label(tile: TileKind) -> &'static str {
    match tile {
        TileKind::Wall => "wall",
        TileKind::Floor => "floor",
        TileKind::Door(aihack_ai_contract::DoorState::Closed) => "closed door",
        TileKind::Door(aihack_ai_contract::DoorState::Open) => "open door",
        TileKind::Door(aihack_ai_contract::DoorState::Locked) => "locked door",
        TileKind::StairsDown => "stairs down",
        TileKind::StairsUp => "stairs up",
        TileKind::Trap(aihack_ai_contract::TrapKind::Pit) => "pit trap",
        TileKind::Trap(aihack_ai_contract::TrapKind::Dart) => "dart trap",
        TileKind::Trap(aihack_ai_contract::TrapKind::Bear) => "bear trap",
        TileKind::Trap(aihack_ai_contract::TrapKind::Hole) => "hole",
        TileKind::Trap(aihack_ai_contract::TrapKind::Teleport) => "teleport trap",
        TileKind::Trap(aihack_ai_contract::TrapKind::SleepingGas) => "sleeping gas trap",
        TileKind::Trap(aihack_ai_contract::TrapKind::Rust) => "rust trap",
        TileKind::Trap(aihack_ai_contract::TrapKind::Polymorph) => "polymorph trap",
        TileKind::Trap(aihack_ai_contract::TrapKind::AntiMagic) => "anti-magic trap",
        TileKind::Trap(aihack_ai_contract::TrapKind::FallingRock) => "falling rock trap",
        TileKind::Trap(aihack_ai_contract::TrapKind::Magic) => "magic trap",
        TileKind::Trap(aihack_ai_contract::TrapKind::Fire) => "fire trap",
        TileKind::HiddenDoor => "hidden door",
        TileKind::HiddenTrap(_) => "hidden trap",
        TileKind::Shop => "shop",
        TileKind::Altar => "altar",
        TileKind::Fountain => "fountain",
        TileKind::DryFountain => "dry fountain",
        TileKind::Sink => "sink",
        TileKind::Water => "water",
        TileKind::Lava => "lava",
        TileKind::Boulder => "boulder",
        TileKind::MagicPortal => "magic portal",
    }
}

// 화면별 presentation line 생성 함수다.

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
    use aihack_ai_contract::Role;
    character_creation_lines_with_selection(
        None,
        Role::Knight.default_race(),
        Role::Knight.default_alignment(),
    )
}

pub fn character_creation_lines_with_selection(
    selected_role: Option<aihack_ai_contract::Role>,
    race: aihack_ai_contract::Race,
    alignment: aihack_ai_contract::Alignment,
) -> Vec<String> {
    use aihack_ai_contract::Role;
    let k = Role::Knight.base_attributes();
    let s = Role::Scout.base_attributes();
    let m = Role::Mage.base_attributes();
    let c = Role::Cleric.base_attributes();
    let (class_line, combo_status, hp_line, ac_line, detail_line) = match selected_role {
        None => (
            "  Class: Adventurer (Enter = legacy 2-floor run)".to_string(),
            "  Pick role: [1]-[9],[b],[g],[m],[k] or [Tab]".to_string(),
            "  HP: 16/16".to_string(),
            "  AC: 10".to_string(),
            "  Campaign: 24 floors (6 branches -> Astral Plane)".to_string(),
        ),
        Some(role) => {
            let attrs = role.base_attributes().with_race_deltas(race);
            let hp = (role.base_hp() + race.starting_hp_bonus()).max(1);
            let mp = (role.base_mp() + race.starting_mp_bonus()).max(0);
            let tag = if role.is_valid_combo(race, alignment) {
                "Valid"
            } else {
                "INVALID"
            };
            (
                format!(
                    "  Class: {} {} {}",
                    alignment.as_str(),
                    race.as_str(),
                    role.display_name()
                ),
                format!(
                    "  Selected: {} {} {} ({tag})",
                    alignment.as_str(),
                    race.as_str(),
                    role.display_name()
                ),
                format!("  HP: {hp}/{hp}   MP: {mp}/{mp}"),
                "  AC: 10".to_string(),
                format!(
                    "  St:{} Dx:{} Co:{} In:{} Wi:{} Ch:{}",
                    attrs.str_, attrs.dex, attrs.con, attrs.int, attrs.wis, attrs.cha
                ),
            )
        }
    };
    vec![
        "  Character Creation".to_string(),
        class_line,
        format!(
            "  [1] Knight - HP{}, melee (St{} Co{})",
            Role::Knight.base_hp(),
            k.str_,
            k.con
        ),
        format!(
            "  [2] Scout - HP{}, accuracy (Dx{} St{})",
            Role::Scout.base_hp(),
            s.dex,
            s.str_
        ),
        format!(
            "  [3] Mage - HP{}, wands (In{} Wi{})",
            Role::Mage.base_hp(),
            m.int,
            m.wis
        ),
        format!(
            "  [4] Cleric - HP{}, mace/heal (Wi{} Co{})",
            Role::Cleric.base_hp(),
            c.wis,
            c.con
        ),
        "  [5] Valkyrie     [6] Wizard      [7] Rogue".to_string(),
        "  [8] Priest       [9] Samurai     [b] Barbarian".to_string(),
        "  [g] Ranger       [m] Monk        [k] Archeologist".to_string(),
        String::new(),
        format!(
            "  [R] Race: {:<5}  [A] Align: {:<7} [Tab] Next",
            race.as_str(),
            alignment.as_str()
        ),
        combo_status,
        hp_line,
        ac_line,
        detail_line,
        String::new(),
        "  Press Enter to confirm   Esc - Back to Title".to_string(),
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

pub fn victory_lines(score: i32) -> Vec<String> {
    vec![
        "".into(),
        "              ASCENDED".into(),
        "You offered the Amulet and ascended to demigodhood.".into(),
        format!("Score: {score}"),
        "[N] New Run    [Q] Quit".into(),
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

// Debug Observation 패널

/// debug observation 패널 텍스트 라인을 생성한다.
/// designs.md 기준으로 schema_version, seed, turn, run_state, player 상태,
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
        format!("player_ac: {}", observation.player.ac),
        format!("hunger: {}", observation.player.hunger),
        format!("luck: {}", observation.player.luck),
        format!("prayer_cooldown: {}", observation.player.prayer_cooldown),
        format!("paralysis_turns: {}", observation.player.paralysis_turns),
        format!("poison_turns: {}", observation.player.poison_turns),
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

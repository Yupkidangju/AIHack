use super::input::{item_observation_label, UiCommandCandidate as U};
use aihack_ai_contract::{
    CommandIntent as C, DipTarget, EngravingMethod, EngravingText, EntityId, ItemKind, Observation,
    SpellKind, WishChoice,
};
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
            | ('P', C::PutOn { .. })
            | ('q', C::Quaff { .. } | C::Drink)
            | ('f', C::Eat { .. })
            | ('d', C::Drop { .. })
            | ('t', C::Throw { .. })
            | ('z', C::Zap { .. } | C::Wish { .. })
            | ('r', C::Read { .. })
            | ('$', C::Buy { .. } | C::Sell { .. } | C::Pay)
            | ('O', C::Offer { .. })
            | ('Z', C::Cast { .. })
            | ('x', C::TakeOff { .. })
            | ('a', C::Apply { .. })
            | ('C', C::ContainerPut { .. } | C::ContainerTake { .. })
            | ('v', C::Dip { .. })
            | ('E', C::Engrave { .. })
            | ('V', C::Invoke { .. })
            | ('X', C::Enhance { .. })
            | ('m', C::Chat(_))
    )
}

fn item_of(command: C) -> Option<EntityId> {
    match command {
        C::Wield { item }
        | C::Wear { item }
        | C::PutOn { item }
        | C::Sell { item }
        | C::Offer { item }
        | C::Quaff { item }
        | C::Eat { item }
        | C::Drop { item }
        | C::Read { item }
        | C::Throw { item, .. }
        | C::Zap { item, .. }
        | C::Wish { item, .. }
        | C::Apply { item, .. }
        | C::Dip { item, .. }
        | C::Invoke { item } => Some(item),
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
    if key == 'z' {
        if let Some(wand) = selected {
            let is_wish_wand = observation
                .inventory
                .iter()
                .find(|i| i.item == wand)
                .is_some_and(|i| i.identified && i.kind == ItemKind::WandWishing);
            if is_wish_wand
                && commands
                    .iter()
                    .any(|c| matches!(c, C::Wish { item, .. } if *item == wand))
            {
                return wish_menu_from_obs(observation, wand);
            }
        }
    }
    if matches!(key, 'o' | 'c' | 'K' | 'm') || selected.is_some() {
        for command in commands {
            if selected.is_some() && item_of(command) != selected {
                continue;
            }
            if let C::Apply {
                direction: None, ..
            } = command
            {
                entries.push(Entry {
                    key: '.',
                    label: "Here (chest / lock)".into(),
                    candidate: U::Command(command),
                });
                continue;
            }
            let direction = match command {
                C::Open(d)
                | C::Close(d)
                | C::Kick(d)
                | C::Chat(d)
                | C::Throw { direction: d, .. }
                | C::Zap { direction: d, .. }
                | C::Apply {
                    direction: Some(d), ..
                } => d,
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
    if key == 'Z' {
        const MENU_KEYS: &[char] = &[
            '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
            'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
        ];
        let spells: Vec<SpellKind> = SpellKind::ALL
            .iter()
            .copied()
            .filter(|spell| {
                commands
                    .iter()
                    .any(|command| matches!(command, C::Cast { spell: s, .. } if s == spell))
            })
            .collect();
        for (index, spell) in spells.into_iter().enumerate() {
            let Some(&key_char) = MENU_KEYS.get(index) else {
                break;
            };
            let fail_pct = observation
                .campaign
                .as_ref()
                .and_then(|camp| {
                    camp.spell_failure_rates
                        .iter()
                        .find(|&&(s, _)| s == spell)
                        .map(|&(_, pct)| pct)
                })
                .unwrap_or(0);
            entries.push(Entry {
                key: key_char,
                label: spell_label(spell, spell.mana_cost(), fail_pct),
                candidate: if spell.is_directional() {
                    U::ChooseSpell(spell)
                } else {
                    U::Command(C::Cast {
                        spell,
                        direction: None,
                    })
                },
            });
        }
        return PlayMenu::new(
            &format!("Cast Spell (MP: {})", observation.player.mp),
            entries,
        );
    }
    if key == 'x' {
        let mut idx = 0u8;
        for command in &commands {
            if let C::TakeOff { slot } = *command {
                idx += 1;
                if idx <= 9 {
                    entries.push(Entry {
                        key: char::from(b'0' + idx),
                        label: format!("Take off {}", slot.label()),
                        candidate: U::Command(command.to_owned()),
                    });
                }
            }
        }
        return PlayMenu::new("Take off equipped", entries);
    }
    if key == '$' {
        if commands.contains(&C::Pay) {
            entries.push(Entry {
                key: 'P',
                label: if observation.player.shop_debt > 0 {
                    format!("Pay shop debt ({}g)", observation.player.shop_debt)
                } else {
                    "Pay / Donate".into()
                },
                candidate: U::Command(C::Pay),
            });
        }
        let mut buy_index = 0u8;
        for command in &commands {
            if let C::Buy { kind } = command {
                buy_index += 1;
                if buy_index <= 9 {
                    let identified = observation
                        .campaign
                        .as_ref()
                        .is_none_or(|campaign| campaign.identified_items.contains(kind));
                    let name = if identified {
                        super::input::item_label(*kind)
                    } else {
                        aihack_runtime::appearance::appearance_for_kind(observation.seed, *kind)
                            .map(str::to_owned)
                            .unwrap_or_else(|| super::input::item_label(*kind))
                    };
                    let price_suffix = aihack_runtime::shop_buy_price(*kind)
                        .map(|base| {
                            let price = observation
                                .campaign
                                .as_ref()
                                .and_then(|c| c.attributes)
                                .map(|attrs| attrs.cha_buy_price(base))
                                .unwrap_or(base);
                            format!(" ({price}g)")
                        })
                        .unwrap_or_default();
                    entries.push(Entry {
                        key: char::from(b'0' + buy_index),
                        label: format!("Buy {name}{price_suffix}"),
                        candidate: U::Command(*command),
                    });
                }
            }
        }
        for item in &observation.inventory {
            if let Some(command) = commands.iter().find(|c| item_of(**c) == Some(item.item)) {
                entries.push(Entry {
                    key: item.letter.0,
                    label: format!("Sell {}", item_observation_label(item)),
                    candidate: U::Command(*command),
                });
            }
        }
        return PlayMenu::new(
            &format!("Shop (Gold: {})", observation.player.gold),
            entries,
        );
    }
    if key == 'a' {
        for item in &observation.inventory {
            let has_dir = commands
                .iter()
                .any(|c| matches!(c, C::Apply { item: i, direction: Some(_) } if *i == item.item));
            if has_dir {
                entries.push(Entry {
                    key: item.letter.0,
                    label: item_observation_label(item),
                    candidate: U::ChooseItem {
                        action: 'a',
                        item: item.item,
                    },
                });
            } else if let Some(command) = commands
                .iter()
                .find(|c| matches!(c, C::Apply { item: i, direction: None } if *i == item.item))
            {
                entries.push(Entry {
                    key: item.letter.0,
                    label: item_observation_label(item),
                    candidate: U::Command(*command),
                });
            }
        }
        return PlayMenu::new("Apply tool", entries);
    }
    if key == 'C' {
        let mut containers = Vec::new();
        for command in &commands {
            let container = match *command {
                C::ContainerPut { container, .. } | C::ContainerTake { container, .. } => container,
                _ => continue,
            };
            if !containers.contains(&container) {
                containers.push(container);
            }
        }
        for (idx, container) in containers.into_iter().take(9).enumerate() {
            let label = observation
                .inventory
                .iter()
                .find(|item| item.item == container)
                .map(item_observation_label)
                .or_else(|| {
                    observation
                        .visible_entities
                        .iter()
                        .find(|e| e.entity == container)
                        .and_then(|e| e.name.clone())
                })
                .unwrap_or_else(|| format!("Container #{}", container.0));
            entries.push(Entry {
                key: char::from(b'1' + idx as u8),
                label,
                candidate: U::ChooseContainer(container),
            });
        }
        return PlayMenu::new("Loot container", entries);
    }
    if key == 'v' {
        for item in &observation.inventory {
            if commands
                .iter()
                .any(|c| matches!(c, C::Dip { item: i, .. } if *i == item.item))
            {
                entries.push(Entry {
                    key: item.letter.0,
                    label: item_observation_label(item),
                    candidate: U::ChooseDipSource(item.item),
                });
            }
        }
        return PlayMenu::new("Dip item", entries);
    }
    if key == 'E' {
        if commands.iter().any(|c| {
            matches!(
                c,
                C::Engrave {
                    method: EngravingMethod::Fingers,
                    ..
                }
            )
        }) {
            entries.push(Entry {
                key: '0',
                label: "Fingers (dust)".into(),
                candidate: U::ChooseEngraveMethod(EngravingMethod::Fingers),
            });
        }
        for item in &observation.inventory {
            if commands.iter().any(|c| {
                matches!(c, C::Engrave { method: EngravingMethod::Item(id), .. } if *id == item.item)
            }) {
                entries.push(Entry {
                    key: item.letter.0,
                    label: item_observation_label(item),
                    candidate: U::ChooseEngraveMethod(EngravingMethod::Item(item.item)),
                });
            }
        }
        return PlayMenu::new("Engrave with", entries);
    }
    if key == 'V' {
        for item in &observation.inventory {
            if let Some(command) = commands
                .iter()
                .find(|c| matches!(c, C::Invoke { item: id } if *id == item.item))
            {
                entries.push(Entry {
                    key: item.letter.0,
                    label: item_observation_label(item),
                    candidate: U::Command(*command),
                });
            }
        }
        let cd = observation
            .campaign
            .as_ref()
            .map(|c| c.artifact_invoke_cooldown)
            .unwrap_or(0);
        let title = if cd > 0 {
            format!("Invoke Artifact (Cooldown: {cd}t)")
        } else {
            "Invoke Artifact".to_string()
        };
        return PlayMenu::new(&title, entries);
    }
    if key == 'X' {
        if let Some(campaign) = &observation.campaign {
            for (idx, skill_obs) in campaign.skills.iter().enumerate() {
                let key_char = char::from(b'1' + idx as u8);
                let threshold = skill_obs.rank.practice_threshold();
                let next_name = skill_obs
                    .rank
                    .next()
                    .map(|r| r.display_name())
                    .unwrap_or("MAX");
                if skill_obs.ready
                    && observation.legal_actions.contains(&C::Enhance {
                        skill: skill_obs.category,
                    })
                {
                    entries.push(Entry {
                        key: key_char,
                        label: format!(
                            "{} [{} -> {}] READY!",
                            skill_obs.category.display_name(),
                            skill_obs.rank.display_name(),
                            next_name
                        ),
                        candidate: U::Command(C::Enhance {
                            skill: skill_obs.category,
                        }),
                    });
                } else {
                    entries.push(Entry {
                        key: key_char,
                        label: format!(
                            "{} [{} / cap {} | prac {}/{}]",
                            skill_obs.category.display_name(),
                            skill_obs.rank.display_name(),
                            skill_obs.cap.display_name(),
                            skill_obs.practice,
                            threshold
                        ),
                        candidate: U::CloseOverlay,
                    });
                }
            }
        }
        return PlayMenu::new("Skills (#enhance)", entries);
    }
    if key == 'q' && commands.contains(&C::Drink) {
        entries.push(Entry {
            key: '0',
            label: "Drink from fountain/sink [g]".into(),
            candidate: U::Command(C::Drink),
        });
    }
    for item in &observation.inventory {
        if let Some(command) = commands.iter().find(|c| item_of(**c) == Some(item.item)) {
            entries.push(Entry {
                key: item.letter.0,
                label: item_observation_label(item),
                candidate: if key == 'z' && item.identified && item.kind == ItemKind::WandWishing {
                    U::ChooseWish(item.item)
                } else if matches!(key, 't' | 'z') {
                    U::ChooseItem {
                        action: key,
                        item: item.item,
                    }
                } else if key == 'r' && can_prompt_identify(observation, item.item) {
                    U::ChooseIdentifyTarget(item.item)
                } else {
                    U::Command(*command)
                },
            });
        }
    }
    PlayMenu::new(&format!("Choose item [{key}]"), entries)
}

pub(super) fn spell_direction_menu(observation: &Observation, spell: SpellKind) -> PlayMenu {
    let entries = "ykulnjbh"
        .chars()
        .filter_map(|key| {
            let direction = super::direction_for_key(key)?;
            let command = C::Cast {
                spell,
                direction: Some(direction),
            };
            observation.legal_actions.contains(&command).then(|| Entry {
                key,
                label: format!("{direction:?}"),
                candidate: U::Command(command),
            })
        })
        .collect();
    PlayMenu::new("Cast direction", entries)
}

pub(super) fn container_menu(observation: &Observation, container: EntityId) -> PlayMenu {
    let mut entries = Vec::new();
    let take_items: Vec<_> = observation
        .container_items
        .iter()
        .filter(|ci| {
            ci.container == container
                && observation.legal_actions.contains(&C::ContainerTake {
                    container,
                    item: ci.item,
                })
        })
        .collect();
    for (idx, ci) in take_items.into_iter().take(9).enumerate() {
        let qty = if ci.quantity > 1 {
            format!("{}x ", ci.quantity)
        } else {
            String::new()
        };
        entries.push(Entry {
            key: char::from(b'1' + idx as u8),
            label: format!("Take {qty}{}", ci.name),
            candidate: U::Command(C::ContainerTake {
                container,
                item: ci.item,
            }),
        });
    }
    for item in &observation.inventory {
        let cmd = C::ContainerPut {
            container,
            item: item.item,
        };
        if observation.legal_actions.contains(&cmd) {
            entries.push(Entry {
                key: item.letter.0,
                label: format!("Put {}", item_observation_label(item)),
                candidate: U::Command(cmd),
            });
        }
    }
    PlayMenu::new("Container (#loot)", entries)
}

pub(super) fn dip_target_menu(observation: &Observation, source: EntityId) -> PlayMenu {
    let mut entries = Vec::new();
    let tile_cmd = C::Dip {
        item: source,
        target: DipTarget::Tile,
    };
    if observation.legal_actions.contains(&tile_cmd) {
        entries.push(Entry {
            key: '0',
            label: "Current tile (fountain/sink/water/altar)".into(),
            candidate: U::Command(tile_cmd),
        });
    }
    for item in &observation.inventory {
        let cmd = C::Dip {
            item: source,
            target: DipTarget::Item(item.item),
        };
        if observation.legal_actions.contains(&cmd) {
            entries.push(Entry {
                key: item.letter.0,
                label: format!("Into {}", item_observation_label(item)),
                candidate: U::Command(cmd),
            });
        }
    }
    PlayMenu::new("Dip into", entries)
}

pub(super) fn engrave_text_menu(observation: &Observation, method: EngravingMethod) -> PlayMenu {
    let entries = [
        ('e', "Elbereth", EngravingText::Elbereth),
        ('w', "Beware!", EngravingText::Warning),
        ('s', "Safe", EngravingText::Safe),
        ('x', "X", EngravingText::XMark),
    ]
    .into_iter()
    .filter_map(|(key, label, text)| {
        let cmd = C::Engrave { method, text };
        observation.legal_actions.contains(&cmd).then(|| Entry {
            key,
            label: label.into(),
            candidate: U::Command(cmd),
        })
    })
    .collect();
    PlayMenu::new("Engrave text", entries)
}

fn can_prompt_identify(observation: &Observation, scroll: EntityId) -> bool {
    let is_identified_scroll = observation.inventory.iter().any(|item| {
        item.item == scroll
            && item.kind == crate::domain::item::ItemKind::ScrollIdentify
            && item.identified
    });
    is_identified_scroll
        && observation
            .legal_actions
            .iter()
            .any(|c| matches!(c, C::Identify { scroll: s, .. } if *s == scroll))
}

pub(super) fn identify_target_menu(observation: &Observation, scroll: EntityId) -> PlayMenu {
    let entries = observation
        .inventory
        .iter()
        .filter_map(|item| {
            let command = C::Identify {
                scroll,
                target: item.item,
            };
            observation.legal_actions.contains(&command).then(|| Entry {
                key: item.letter.0,
                label: item_observation_label(item),
                candidate: U::Command(command),
            })
        })
        .collect();
    PlayMenu::new("Identify item", entries)
}

fn spell_label(spell: SpellKind, cost: i16, fail_pct: u8) -> String {
    format!("{} (MP {cost}, fail {fail_pct}%)", spell.display_name())
}

pub(super) fn wish_menu_from_obs(observation: &Observation, wand: EntityId) -> PlayMenu {
    let entries = observation
        .legal_actions
        .iter()
        .copied()
        .filter_map(|c| match c {
            C::Wish { item, wish } if item == wand => Some((c, wish)),
            _ => None,
        })
        .enumerate()
        .take(9)
        .map(|(idx, (command, wish))| Entry {
            key: char::from(b'1' + idx as u8),
            label: wish_label(wish),
            candidate: U::Command(command),
        })
        .collect();
    PlayMenu::new("Make a Wish", entries)
}

fn wish_label(wish: WishChoice) -> String {
    match wish {
        WishChoice::Item {
            kind,
            blessed,
            enchant,
        } => {
            let b_str = if blessed { "blessed " } else { "" };
            let e_str = if enchant != 0 {
                format!("{enchant:+} ")
            } else {
                String::new()
            };
            format!("{e_str}{b_str}{kind:?}")
        }
        WishChoice::ArtifactExcalibur => "Excalibur".to_string(),
        WishChoice::ArtifactSunsword => "Sunsword".to_string(),
        WishChoice::ArtifactMagicbane => "Magicbane".to_string(),
        WishChoice::ArtifactCleaver => "Cleaver".to_string(),
        WishChoice::ArtifactStormbringer => "Stormbringer".to_string(),
        WishChoice::ArtifactSting => "Sting".to_string(),
    }
}

pub(super) fn inventory_menu(observation: &Observation) -> PlayMenu {
    PlayMenu::new(
        "INVENTORY",
        observation
            .inventory
            .iter()
            .map(|item| Entry {
                key: item.letter.0,
                label: item_observation_label(item),
                candidate: U::ChooseItem {
                    action: 'i',
                    item: item.item,
                },
            })
            .collect(),
    )
}

pub(super) fn item_menu(observation: &Observation, item: EntityId) -> PlayMenu {
    let mut entries: Vec<_> = [
        ('w', "Wield"),
        ('e', "Wear"),
        ('P', "Put on"),
        ('q', "Quaff"),
        ('f', "Eat"),
        ('r', "Read"),
        ('d', "Drop"),
        ('t', "Throw"),
        ('z', "Zap"),
        ('a', "Apply"),
        ('v', "Dip"),
        ('V', "Invoke"),
        ('$', "Sell"),
        ('O', "Offer"),
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
            candidate: if key == 'z'
                && observation
                    .inventory
                    .iter()
                    .find(|i| i.item == item)
                    .is_some_and(|i| i.identified && i.kind == ItemKind::WandWishing)
            {
                U::ChooseWish(item)
            } else if matches!(key, 't' | 'z')
                || (key == 'a'
                    && observation.legal_actions.iter().any(
                        |c| matches!(c, C::Apply { item: i, direction: Some(_) } if *i == item),
                    ))
            {
                U::ChooseItem { action: key, item }
            } else if key == 'v' {
                U::ChooseDipSource(item)
            } else if key == 'r' && can_prompt_identify(observation, item) {
                U::ChooseIdentifyTarget(item)
            } else {
                U::Command(command)
            },
        })
    })
    .collect();
    if observation.legal_actions.iter().any(|c| {
        matches!(
            c,
            C::ContainerPut { container, .. } | C::ContainerTake { container, .. }
                if *container == item
        )
    }) {
        entries.push(Entry {
            key: 'C',
            label: "Loot container".into(),
            candidate: U::ChooseContainer(item),
        });
    }
    if let Some(inv_item) = observation.inventory.iter().find(|i| i.item == item) {
        if let Some(slot) = inv_item.equipped_slot {
            let take_off = C::TakeOff { slot };
            if slot != aihack_ai_contract::EquipmentSlot::Melee
                && observation.legal_actions.contains(&take_off)
            {
                entries.push(Entry {
                    key: 'x',
                    label: "Take off".into(),
                    candidate: U::Command(take_off),
                });
            }
        }
    }
    PlayMenu::new("Item action", entries)
}

pub(super) fn commands_menu(observation: &Observation) -> PlayMenu {
    let mut entries: Vec<_> = [
        ('s', "Search", C::Search),
        (',', "Pickup", C::Pickup),
        ('p', "Pray", C::Pray),
        ('T', "Remove ring", C::RemoveRing),
        ('D', "Descend", C::Descend),
        ('U', "Ascend", C::Ascend),
        ('g', "Drink", C::Drink),
        ('!', "Pay / Donate", C::Pay),
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
        let label = match (
            observation
                .campaign
                .as_ref()
                .is_some_and(|c| c.on_magic_portal),
            observation.current_level.branch,
            observation.current_level.depth,
        ) {
            (true, _, _) => "Enter Magic Portal",
            (_, aihack_ai_contract::BranchId::Main, 3) => "Enter Mines",
            (_, aihack_ai_contract::BranchId::Main, 4) => "Enter Sokoban",
            (_, aihack_ai_contract::BranchId::Main, 5) => "Enter Quest",
            (_, aihack_ai_contract::BranchId::Main, 6) => "Enter Gehennom",
            _ => "Enter Branch",
        };
        entries.push(Entry {
            key: 'B',
            label: label.into(),
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
        ('P', "Put on"),
        ('x', "Take off"),
        ('f', "Eat"),
        ('r', "Read"),
        ('$', "Shop"),
        ('Z', "Cast"),
        ('O', "Offer"),
        ('a', "Apply"),
        ('C', "Container"),
        ('v', "Dip"),
        ('E', "Engrave"),
        ('V', "Invoke"),
        ('X', "Enhance"),
        ('m', "Chat"),
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

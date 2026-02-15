// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::assets::AssetManager;
use crate::core::entity::player::{Alignment, Player};
use crate::core::entity::Item;
use crate::ui::log::GameLog;
use legion::*;

pub struct ArtifactSystem;

impl ArtifactSystem {
    ///
    pub fn try_artifact_promotion(
        item_ent: Entity,
        new_name: &str,
        world: &mut World,
        _assets: &AssetManager,
        player: &Player,
        log: &mut GameLog,
        turn: u64,
    ) {
        let item_info = if let Ok(entry) = world.entry_ref(item_ent) {
            entry.get_component::<Item>().ok().cloned()
        } else {
            return;
        };

        if let Some(item) = item_info {
            //
            if item.artifact.is_some() {
                return;
            }

            //
            let promos = [
                ("Excalibur", ("long sword", 5, Alignment::Lawful)),
                ("Mjollnir", ("war hammer", 1, Alignment::Neutral)),
                ("Stormbringer", ("runesword", 1, Alignment::Chaotic)),
                ("Cleaver", ("battle-axe", 1, Alignment::Neutral)),
                ("Grimtooth", ("orcish dagger", 1, Alignment::Chaotic)),
            ];

            for (art_name, (base, min_lvl, req_align)) in promos {
                if new_name == art_name
                    && item.kind.as_str() == base
                    && player.alignment == req_align
                    && player.exp_level >= min_lvl
                {
                    if let Ok(mut entry) = world.entry_mut(item_ent) {
                        if let Ok(item_mut) = entry.get_component_mut::<Item>() {
                            item_mut.artifact = Some(art_name.to_string());
                            log.add_colored(
                                format!("You hear a chime! Your {} becomes {}!", base, art_name),
                                [255, 215, 0],
                                turn,
                            );
                            if art_name == "Excalibur" {
                                item_mut.blessed = true; // Excalibur는 축복받음
                            }
                            return;
                        }
                    }
                }
            }
        }
    }

    ///
    pub fn gift_artifact(
        alignment: Alignment,
        world: &mut World, // &mut World is needed to check existing artifacts
        assets: &AssetManager,
        _player_ent: Entity,
        p_pos: (i32, i32),
        p_level: crate::core::dungeon::LevelID,
        log: &mut GameLog,
        turn: u64,
        command_buffer: &mut legion::systems::CommandBuffer,
    ) {
        //
        let mut existing = std::collections::HashSet::new();
        let mut query = <&Item>::query();
        for item in query.iter(world) {
            if let Some(ref art) = item.artifact {
                existing.insert(art.clone());
            }
        }

        let gifts = match alignment {
            Alignment::Lawful => vec![("Excalibur", "long sword"), ("Sunsword", "long sword")],
            Alignment::Neutral => vec![("Mjollnir", "war hammer"), ("Vorpal Blade", "long sword")],
            Alignment::Chaotic => vec![("Stormbringer", "runesword"), ("Sting", "elven dagger")],
        };

        for (art_name, base_name) in gifts {
            if !existing.contains(art_name) {
                // 기반 아이템 템플릿 획득
                if let Some(template) = assets.items.get_template(base_name) {
                    let mut item = Item {
                        kind: crate::generated::ItemKind::from_str(base_name),
                        price: template.cost as u32 * 10,
                        weight: template.weight as u32,
                        unpaid: false,
                        spe: 0,
                        blessed: true,
                        cursed: false,
                        bknown: true,
                        known: true,
                        dknown: true,
                        oeroded: 0,
                        oeroded2: 0,
                        quantity: 1,
                        corpsenm: None,
                        age: turn,
                        oeaten: 0,
                        olocked: false,
                        oopened: false,
                        user_name: None,
                        artifact: Some(art_name.to_string()),
                        owet: 0,
                    };

                    if art_name == "Excalibur" || art_name == "Sunsword" {
                        item.spe = 2;
                    }

                    let glyph = if template.class == crate::core::entity::object::ItemClass::Weapon
                    {
                        ')'
                    } else {
                        '['
                    };

                    log.add_colored(
                        format!("A {} appears on the altar!", art_name),
                        [255, 255, 0],
                        turn,
                    );

                    command_buffer.push((
                        item,
                        crate::core::entity::Position {
                            x: p_pos.0,
                            y: p_pos.1,
                        },
                        crate::core::entity::Level(p_level),
                        crate::core::entity::Renderable {
                            glyph,
                            color: 14, // HI_GOLD
                        },
                        crate::core::entity::ItemTag,
                    ));
                }
                return;
            }
        }

        log.add(
            "You feel a sense of divine approval, but nothing happens.",
            turn,
        );
    }

    ///
    pub fn invoke_artifact(
        item_ent: Entity,
        world: &mut World,
        _assets: &AssetManager,
        _player_ent: Entity,
        log: &mut GameLog,
        turn: u64,
    ) {
        let item = if let Ok(entry) = world.entry_ref(item_ent) {
            entry.get_component::<Item>().ok().cloned()
        } else {
            return;
        };

        if let Some(i) = item {
            if let Some(art_id) = i.artifact {
                match art_id.as_str() {
                    "Excalibur" => {
                        log.add("Excalibur glows with a cold blue light.", turn);
                        // 수색 효과 (Searching) 보너스 (임시: 즉시 주변 수색)
                    }
                    "Mjollnir" => {
                        log.add("Mjollnir crackles with lightning!", turn);
                        //
                    }
                    _ => {
                        log.add(
                            format!("The {} does not have an active power.", art_id),
                            turn,
                        );
                    }
                }
            } else {
                log.add("This item has no special power to invoke.", turn);
            }
        }
    }
}

// =============================================================================
// [v2.3.1] artifact.c 전투/데이터 로직 이식
// 원본: nethack-3.6.7/src/artifact.c (2,005줄)
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct ArtifactData {
    ///
    pub name: &'static str,
    /// 기반 아이템 종류
    pub base_item: &'static str,
    /// 필요 성향
    pub alignment: Alignment,
    /// 역할 전용 (없으면 None)
    pub role: Option<&'static str>,
    /// 특수 데미지 배수 (vs 대상)
    pub spec_damage: i32,
    /// 대상 타입 (undead, demon 등)
    pub spec_target: &'static str,
    /// 공격시 특수 효과
    pub attack_effect: &'static str,
    /// 소유시 보너스
    pub carry_effect: &'static str,
    /// invoke 능력
    pub invoke_ability: &'static str,
}

///
pub fn artifact_database() -> Vec<ArtifactData> {
    vec![
        ArtifactData {
            name: "Excalibur",
            base_item: "long sword",
            alignment: Alignment::Lawful,
            role: Some("Knight"),
            spec_damage: 10,
            spec_target: "undead",
            attack_effect: "drain_resistance",
            carry_effect: "searching",
            invoke_ability: "level_drain_resistance",
        },
        ArtifactData {
            name: "Mjollnir",
            base_item: "war hammer",
            alignment: Alignment::Neutral,
            role: Some("Valkyrie"),
            spec_damage: 24,
            spec_target: "shock",
            attack_effect: "lightning",
            carry_effect: "none",
            invoke_ability: "lightning_bolt",
        },
        ArtifactData {
            name: "Stormbringer",
            base_item: "runesword",
            alignment: Alignment::Chaotic,
            role: None,
            spec_damage: 8,
            spec_target: "all",
            attack_effect: "level_drain",
            carry_effect: "none",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Grayswandir",
            base_item: "silver saber",
            alignment: Alignment::Lawful,
            role: None,
            spec_damage: 8,
            spec_target: "all",
            attack_effect: "halved",
            carry_effect: "none",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Frost Brand",
            base_item: "long sword",
            alignment: Alignment::Neutral,
            role: None,
            spec_damage: 8,
            spec_target: "fire",
            attack_effect: "cold",
            carry_effect: "cold_resistance",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Fire Brand",
            base_item: "long sword",
            alignment: Alignment::Neutral,
            role: None,
            spec_damage: 8,
            spec_target: "cold",
            attack_effect: "fire",
            carry_effect: "fire_resistance",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Sting",
            base_item: "elven dagger",
            alignment: Alignment::Chaotic,
            role: None,
            spec_damage: 5,
            spec_target: "orc",
            attack_effect: "warning_orc",
            carry_effect: "orc_warning",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Orcrist",
            base_item: "elven broadsword",
            alignment: Alignment::Chaotic,
            role: None,
            spec_damage: 10,
            spec_target: "orc",
            attack_effect: "orc_bonus",
            carry_effect: "none",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Vorpal Blade",
            base_item: "long sword",
            alignment: Alignment::Neutral,
            role: None,
            spec_damage: 1,
            spec_target: "beheading",
            attack_effect: "vorpal",
            carry_effect: "none",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Sunsword",
            base_item: "long sword",
            alignment: Alignment::Lawful,
            role: None,
            spec_damage: 8,
            spec_target: "undead",
            attack_effect: "blind_undead",
            carry_effect: "light",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Magicbane",
            base_item: "athame",
            alignment: Alignment::Neutral,
            role: Some("Wizard"),
            spec_damage: 4,
            spec_target: "all",
            attack_effect: "magic_resist",
            carry_effect: "magic_resistance",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Cleaver",
            base_item: "battle-axe",
            alignment: Alignment::Neutral,
            role: Some("Barbarian"),
            spec_damage: 6,
            spec_target: "all",
            attack_effect: "cleave",
            carry_effect: "none",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Grimtooth",
            base_item: "orcish dagger",
            alignment: Alignment::Chaotic,
            role: None,
            spec_damage: 6,
            spec_target: "elf",
            attack_effect: "poison",
            carry_effect: "none",
            invoke_ability: "none",
        },
        ArtifactData {
            name: "Demonbane",
            base_item: "long sword",
            alignment: Alignment::Lawful,
            role: None,
            spec_damage: 14,
            spec_target: "demon",
            attack_effect: "scare_demon",
            carry_effect: "none",
            invoke_ability: "banish_demon",
        },
    ]
}

///
pub fn lookup_artifact(name: &str) -> Option<ArtifactData> {
    artifact_database().into_iter().find(|a| a.name == name)
}

///
pub fn artifact_damage_bonus(
    artifact_name: &str,
    target_symbol: char,
    target_is_undead: bool,
    target_is_demon: bool,
) -> i32 {
    if let Some(data) = lookup_artifact(artifact_name) {
        let applies = match data.spec_target {
            "undead" => target_is_undead,
            "demon" => target_is_demon,
            "orc" => target_symbol == 'o' || target_symbol == 'O',
            "elf" => target_symbol == 'e' || target_symbol == '@',
            "all" => true,
            "beheading" => true, // Vorpal: 별도 처리
            _ => false,
        };
        if applies {
            data.spec_damage
        } else {
            0
        }
    } else {
        0
    }
}

///
pub fn artifact_hit_bonus(artifact_name: &str) -> i32 {
    //
    match artifact_name {
        "Excalibur" => 5,
        "Mjollnir" => 5,
        "Stormbringer" => 5,
        "Grayswandir" => 5,
        "Magicbane" => 3,
        _ => 0,
    }
}

///
pub fn artifact_carry_ac_bonus(artifact_name: &str) -> i32 {
    match artifact_name {
        "Magicbane" => -3,
        "Sunsword" => -2,
        _ => 0,
    }
}

///
pub fn artifact_alignment_ok(artifact_name: &str, player_alignment: Alignment) -> bool {
    if let Some(data) = lookup_artifact(artifact_name) {
        data.alignment == player_alignment
    } else {
        true // 비아티팩트는 제한 없음
    }
}

/// [v2.3.1] Vorpal 참수 효과 (원본: spec_applies + vorpal)
pub fn vorpal_check(target_has_head: bool, rng: &mut crate::util::rng::NetHackRng) -> bool {
    // 원본: 1/20 확률로 참수 (머리가 있는 대상만)
    target_has_head && rng.rn2(20) == 0
}

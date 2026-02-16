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

// =============================================================================
// [v2.9.0] artifact.c 대량 이식  터치/저항/보호/점수/말하기
// 원본: nethack-3.6.7/src/artifact.c (2,005줄)
// =============================================================================

/// [v2.9.0] 아티팩트 터치 페널티 (원본: artifact.c:1455-1520 touch_artifact)
/// 성향 불일치 아티팩트를 잡으면 데미지를 받음
pub fn touch_artifact_penalty(
    artifact_name: &str,
    player_alignment: Alignment,
    is_worn: bool,
) -> Option<TouchPenalty> {
    let data = lookup_artifact(artifact_name)?;

    if data.alignment == player_alignment {
        return None; // 성향 일치  페널티 없음
    }

    // 착용 중이면 더 큰 페널티
    let damage = if is_worn { 8 } else { 4 };
    let msg = match data.alignment {
        Alignment::Lawful => "You feel a burning lawful aura!",
        Alignment::Neutral => "You feel a powerful neutral force!",
        Alignment::Chaotic => "You feel a chaotic energy surge!",
    };

    Some(TouchPenalty {
        damage,
        message: msg,
        drops_item: damage >= 6,
    })
}

/// [v2.9.0] 터치 페널티 결과
#[derive(Debug, Clone)]
pub struct TouchPenalty {
    /// 받는 데미지
    pub damage: i32,
    /// 표시 메시지
    pub message: &'static str,
    /// 아이템을 떨어뜨리는지
    pub drops_item: bool,
}

/// [v2.9.0] 재터치 (원본: artifact.c:1522-1570 retouch_object)
/// 저주받은 아티팩트가 BUC 상태 변경 시 재평가
pub fn retouch_check(
    artifact_name: &str,
    player_alignment: Alignment,
    item_cursed: bool,
    item_blessed: bool,
) -> RetouchResult {
    let data = match lookup_artifact(artifact_name) {
        Some(d) => d,
        None => return RetouchResult::Safe,
    };

    // 축복 + 성향 일치  안전
    if item_blessed && data.alignment == player_alignment {
        return RetouchResult::Safe;
    }

    // 저주 + 성향 불일치  위험
    if item_cursed && data.alignment != player_alignment {
        return RetouchResult::Dangerous("The artifact burns your hands!");
    }

    // 성향 불일치만  경고
    if data.alignment != player_alignment {
        return RetouchResult::Warning("The artifact feels uncomfortable.");
    }

    RetouchResult::Safe
}

/// [v2.9.0] 재터치 결과
#[derive(Debug, Clone, PartialEq)]
pub enum RetouchResult {
    Safe,
    Warning(&'static str),
    Dangerous(&'static str),
}

/// [v2.9.0] 아티팩트가 말하기 (원본: artifact.c:1690-1730 arti_speaks)
pub fn arti_speaks(artifact_name: &str) -> &'static str {
    match artifact_name {
        "Excalibur" => "I am the sword of justice.",
        "Mjollnir" => "Thunder and lightning!",
        "Stormbringer" => "I hunger for souls...",
        "Vorpal Blade" => "Snicker-snack!",
        "Sting" => "I glow blue in the presence of orcs!",
        "Orcrist" => "I am the goblin-cleaver!",
        "Magicbane" => "I resist enchantment!",
        "Sunsword" => "I shine with holy light!",
        "Grimtooth" => "I seek elven blood...",
        "Demonbane" => "I banish the forces of evil!",
        "Fire Brand" => "I burn with elemental fire!",
        "Frost Brand" => "I freeze with elemental cold!",
        "Grayswandir" => "I am the silver sabre of light.",
        "Cleaver" => "I cleave through all!",
        _ => "The artifact hums softly.",
    }
}

/// [v2.9.0] 아티팩트 저항 제공 (원본: artifact.c:1050-1120 protects)
pub fn artifact_provides_resistance(artifact_name: &str) -> Vec<&'static str> {
    match artifact_name {
        "Excalibur" => vec!["drain_resistance"],
        "Frost Brand" => vec!["cold_resistance"],
        "Fire Brand" => vec!["fire_resistance"],
        "Magicbane" => vec!["magic_resistance"],
        "Sunsword" => vec![],
        "Mjollnir" => vec!["shock_resistance"],
        "Stormbringer" => vec!["drain_resistance"],
        "Grayswandir" => vec![],
        _ => vec![],
    }
}

/// [v2.9.0] 아티팩트 방어 보너스 (원본: artifact.c:1130-1180 spec_abon)
pub fn artifact_defense_bonus(artifact_name: &str) -> ArtifactDefense {
    match artifact_name {
        "Grayswandir" => ArtifactDefense {
            halves_damage: true,
            blocks_type: "all",
            ac_bonus: 0,
        },
        "Magicbane" => ArtifactDefense {
            halves_damage: false,
            blocks_type: "magic",
            ac_bonus: -3,
        },
        "Excalibur" => ArtifactDefense {
            halves_damage: false,
            blocks_type: "none",
            ac_bonus: 0,
        },
        _ => ArtifactDefense {
            halves_damage: false,
            blocks_type: "none",
            ac_bonus: 0,
        },
    }
}

/// [v2.9.0] 아티팩트 방어 정보
#[derive(Debug, Clone)]
pub struct ArtifactDefense {
    /// 데미지 절반
    pub halves_damage: bool,
    /// 차단 대상 ("all", "magic", "none")
    pub blocks_type: &'static str,
    /// AC 보너스
    pub ac_bonus: i32,
}

/// [v2.9.0] 아티팩트 특수 데미지 보너스 확장 (원본: artifact.c:890-985 spec_dbon)
/// 크기별 데미지, 대상별 보정 포함
pub fn artifact_spec_dbon(
    artifact_name: &str,
    target_symbol: char,
    target_is_undead: bool,
    target_is_demon: bool,
    target_size: i32, // 0=small, 1=medium, 2=large
    rng: &mut crate::util::rng::NetHackRng,
) -> (i32, Option<&'static str>) {
    let data = match lookup_artifact(artifact_name) {
        Some(d) => d,
        None => return (0, None),
    };

    let applies = match data.spec_target {
        "undead" => target_is_undead,
        "demon" => target_is_demon,
        "orc" => target_symbol == 'o' || target_symbol == 'O',
        "elf" => target_symbol == 'e' || target_symbol == '@',
        "all" => true,
        "beheading" => true,
        "fire" => true,
        "cold" => true,
        "shock" => true,
        _ => false,
    };

    if !applies {
        return (0, None);
    }

    // Vorpal Blade 특수 처리
    if data.attack_effect == "vorpal" {
        if rng.rn2(20) == 0 {
            return (9999, Some("Snicker-snack!")); // 참수!
        } else {
            return (data.spec_damage, None);
        }
    }

    // 크기 보정: 큰 대상에게는 추가 피해
    let size_bonus = if target_size >= 2 {
        data.spec_damage / 3
    } else {
        0
    };

    // 효과 메시지
    let effect_msg = match data.attack_effect {
        "lightning" => Some("Lightning strikes!"),
        "fire" => Some("The weapon burns with fire!"),
        "cold" => Some("The weapon freezes!"),
        "level_drain" => Some("The weapon drains life force!"),
        "poison" => Some("The weapon is coated with poison!"),
        "drain_resistance" => None,
        _ => None,
    };

    (data.spec_damage + size_bonus, effect_msg)
}

/// [v2.9.0] 아티팩트 점수 계산 (원본: artifact.c artifact_score 관련)
pub fn calc_artifact_score(artifact_name: &str) -> i64 {
    let data = match lookup_artifact(artifact_name) {
        Some(d) => d,
        None => return 0,
    };

    // 기본 점수: spec_damage * 250
    let base = data.spec_damage as i64 * 250;

    // 역할 전용 아티팩트는 추가 보너스
    let role_bonus = if data.role.is_some() { 500 } else { 0 };

    // invoke 능력이 있으면 추가
    let invoke_bonus = if data.invoke_ability != "none" {
        1000
    } else {
        0
    };

    base + role_bonus + invoke_bonus
}

/// [v2.9.0] 모든 아티팩트의 점수 합계
pub fn total_artifact_score(owned_artifacts: &[&str]) -> i64 {
    owned_artifacts.iter().map(|a| calc_artifact_score(a)).sum()
}

/// [v2.9.0] 아티팩트 존재 여부 확인
pub fn artifact_exists(name: &str) -> bool {
    artifact_database().iter().any(|a| a.name == name)
}

/// [v2.9.0] 특정 성향의 아티팩트 목록
pub fn artifacts_for_alignment(alignment: Alignment) -> Vec<&'static str> {
    artifact_database()
        .into_iter()
        .filter(|a| a.alignment == alignment)
        .map(|a| a.name)
        .collect()
}

/// [v2.9.0] 역할 전용 아티팩트 찾기
pub fn quest_artifact_for_role(role: &str) -> Option<&'static str> {
    artifact_database()
        .into_iter()
        .find(|a| a.role == Some(role))
        .map(|a| a.name)
}

/// [v2.9.0] 아티팩트 invoke 쿨다운 (원본: artifact.c:1600-1650)
pub fn invoke_cooldown(artifact_name: &str) -> i32 {
    match artifact_name {
        "Excalibur" => 100,
        "Mjollnir" => 200,
        "Demonbane" => 150,
        _ => 0,
    }
}

/// [v2.9.0] 아티팩트가 대상에게 적용되는지 판정 (원본: spec_applies)
pub fn spec_applies(
    artifact_name: &str,
    target_symbol: char,
    target_is_undead: bool,
    target_is_demon: bool,
) -> bool {
    let data = match lookup_artifact(artifact_name) {
        Some(d) => d,
        None => return false,
    };
    match data.spec_target {
        "undead" => target_is_undead,
        "demon" => target_is_demon,
        "orc" => target_symbol == 'o' || target_symbol == 'O',
        "elf" => target_symbol == 'e' || target_symbol == '@',
        "all" | "beheading" => true,
        "fire" | "cold" | "shock" => true,
        _ => false,
    }
}

/// [v2.9.0] 아티팩트 명중 보너스 (대상 유형별)
pub fn artifact_to_hit_vs_target(
    artifact_name: &str,
    target_is_undead: bool,
    target_is_demon: bool,
) -> i32 {
    let base = artifact_hit_bonus(artifact_name);
    let applies = spec_applies(artifact_name, ' ', target_is_undead, target_is_demon);
    if applies {
        base + 4
    } else {
        base
    }
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_db() {
        let db = artifact_database();
        assert!(db.len() >= 14);
    }

    #[test]
    fn test_lookup() {
        let excal = lookup_artifact("Excalibur");
        assert!(excal.is_some());
        assert_eq!(excal.unwrap().base_item, "long sword");
    }

    #[test]
    fn test_damage_bonus() {
        let dmg = artifact_damage_bonus("Excalibur", 'Z', true, false);
        assert_eq!(dmg, 10); // undead 대상
        let dmg2 = artifact_damage_bonus("Excalibur", 'o', false, false);
        assert_eq!(dmg2, 0); // undead 아닌 대상
    }

    #[test]
    fn test_hit_bonus() {
        assert_eq!(artifact_hit_bonus("Excalibur"), 5);
        assert_eq!(artifact_hit_bonus("random"), 0);
    }

    #[test]
    fn test_alignment_ok() {
        assert!(artifact_alignment_ok("Excalibur", Alignment::Lawful));
        assert!(!artifact_alignment_ok("Excalibur", Alignment::Chaotic));
    }

    // [v2.9.0] 신규 테스트

    #[test]
    fn test_touch_penalty_mismatch() {
        let result = touch_artifact_penalty("Excalibur", Alignment::Chaotic, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().damage, 4);
    }

    #[test]
    fn test_touch_penalty_match() {
        let result = touch_artifact_penalty("Excalibur", Alignment::Lawful, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_retouch_safe() {
        let r = retouch_check("Excalibur", Alignment::Lawful, false, true);
        assert_eq!(r, RetouchResult::Safe);
    }

    #[test]
    fn test_retouch_dangerous() {
        let r = retouch_check("Excalibur", Alignment::Chaotic, true, false);
        assert!(matches!(r, RetouchResult::Dangerous(_)));
    }

    #[test]
    fn test_arti_speaks() {
        assert_eq!(arti_speaks("Excalibur"), "I am the sword of justice.");
        assert_eq!(arti_speaks("Vorpal Blade"), "Snicker-snack!");
    }

    #[test]
    fn test_resistance() {
        let r = artifact_provides_resistance("Frost Brand");
        assert!(r.contains(&"cold_resistance"));
        let r2 = artifact_provides_resistance("Sunsword");
        assert!(r2.is_empty());
    }

    #[test]
    fn test_defense() {
        let d = artifact_defense_bonus("Grayswandir");
        assert!(d.halves_damage);
        let d2 = artifact_defense_bonus("Magicbane");
        assert_eq!(d2.ac_bonus, -3);
    }

    #[test]
    fn test_calc_score() {
        let s = calc_artifact_score("Excalibur");
        assert!(s > 0);
        let s2 = calc_artifact_score("NonExistent");
        assert_eq!(s2, 0);
    }

    #[test]
    fn test_total_score() {
        let total = total_artifact_score(&["Excalibur", "Mjollnir"]);
        assert!(total > 0);
    }

    #[test]
    fn test_artifact_exists() {
        assert!(artifact_exists("Excalibur"));
        assert!(!artifact_exists("FakeArtifact"));
    }

    #[test]
    fn test_alignment_artifacts() {
        let lawful = artifacts_for_alignment(Alignment::Lawful);
        assert!(lawful.contains(&"Excalibur"));
        assert!(!lawful.contains(&"Stormbringer"));
    }

    #[test]
    fn test_quest_artifact() {
        assert_eq!(quest_artifact_for_role("Knight"), Some("Excalibur"));
        assert_eq!(quest_artifact_for_role("Valkyrie"), Some("Mjollnir"));
        assert!(quest_artifact_for_role("Tourist").is_none());
    }

    #[test]
    fn test_invoke_cooldown() {
        assert_eq!(invoke_cooldown("Excalibur"), 100);
        assert_eq!(invoke_cooldown("Sting"), 0);
    }

    #[test]
    fn test_spec_applies() {
        assert!(spec_applies("Excalibur", 'Z', true, false));
        assert!(!spec_applies("Excalibur", 'o', false, false));
        assert!(spec_applies("Stormbringer", 'a', false, false)); // "all"
    }

    #[test]
    fn test_to_hit_vs_target() {
        let h = artifact_to_hit_vs_target("Excalibur", true, false);
        assert_eq!(h, 9); // 5 + 4
        let h2 = artifact_to_hit_vs_target("Excalibur", false, false);
        assert_eq!(h2, 5); // base only
    }
}

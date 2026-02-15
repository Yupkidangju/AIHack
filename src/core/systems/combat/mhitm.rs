// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::assets::AssetManager;
use crate::core::dungeon::Grid;
use crate::core::entity::{
    monster::Attack, monster::AttackType, monster::DamageType, CombatStats, Health, Monster,
    Position,
};
use crate::core::systems::combat::CombatEngine;
use crate::core::systems::vision::VisionSystem;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::{EntityStore, SubWorld};
use legion::*;

/// Monster attacking another Monster (mhitm.c 이식)
pub struct MHitM;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MHitResult {
    Miss = 0,
    Hit = 1,
    DefDied = 2,
    AgrDied = 4,
}

impl MHitM {
    /// 몬스터 간 공격 핵심 루틴 (mattackm 이식)
    pub fn mattackm(
        world: &mut SubWorld,
        magr_ent: Entity,
        mdef_ent: Entity,
        rng: &mut NetHackRng,
        assets: &AssetManager,
        grid: &Grid,
        vision: &VisionSystem,
        log: &mut GameLog,
        turn: &u64,
        command_buffer: &mut CommandBuffer,
    ) -> u8 {
        let mut result = 0;

        // 1. 공격자 및 방어자 정보 추출
        let (magr_template_name, magr_level, magr_pos) = {
            if let Ok(entry) = world.entry_ref(magr_ent) {
                if let Ok(m) = entry.get_component::<Monster>() {
                    if let Ok(stats) = entry.get_component::<CombatStats>() {
                        if let Ok(pos) = entry.get_component::<Position>() {
                            (m.kind, stats.level as i32, *pos)
                        } else {
                            return 0;
                        }
                    } else {
                        return 0;
                    }
                } else {
                    return 0;
                }
            } else {
                return 0;
            }
        };

        let (mdef_template_name, mdef_ac, mdef_pos, mdef_status) = {
            if let Ok(entry) = world.entry_ref(mdef_ent) {
                if let Ok(m) = entry.get_component::<Monster>() {
                    if let Ok(stats) = entry.get_component::<CombatStats>() {
                        if let Ok(pos) = entry.get_component::<Position>() {
                            let flags = entry
                                .get_component::<crate::core::entity::status::StatusBundle>()
                                .map(|b| b.flags())
                                .unwrap_or(crate::core::entity::status::StatusFlags::empty());
                            (m.kind, stats.ac as i32, *pos, flags)
                        } else {
                            return 0;
                        }
                    } else {
                        return 0;
                    }
                } else {
                    return 0;
                }
            } else {
                return 0;
            }
        };

        let magr_template =
            if let Some(t) = assets.monsters.templates.get(magr_template_name.as_str()) {
                t
            } else {
                return 0;
            };
        let mdef_template =
            if let Some(t) = assets.monsters.templates.get(mdef_template_name.as_str()) {
                t
            } else {
                return 0;
            };

        let vis = Self::can_see(vision, &magr_pos) || Self::can_see(vision, &mdef_pos);

        // 2. 공격 루프
        for (i, attack) in magr_template.attacks.iter().enumerate() {
            if i >= 6 {
                break;
            }
            if attack.atype == AttackType::None {
                continue;
            }

            // 명중 체크
            let is_hit = CombatEngine::calculate_monster_hit(
                rng,
                magr_level,
                mdef_ac,
                i,
                &mdef_status,
                false,
            );

            if is_hit {
                result |= MHitResult::Hit as u8;

                if vis {
                    log.add(
                        format!(
                            "The {} hits the {}!",
                            magr_template.name, mdef_template.name
                        ),
                        *turn,
                    );
                }

                let hit_res = Self::hitmm(
                    world,
                    magr_ent,
                    mdef_ent,
                    attack,
                    rng,
                    assets,
                    grid,
                    vision,
                    log,
                    turn,
                    command_buffer,
                );
                result |= hit_res;

                if (result & MHitResult::DefDied as u8) != 0 {
                    break;
                }
                if (result & MHitResult::AgrDied as u8) != 0 {
                    break;
                }
            } else {
                if vis && rng.rn2(10) == 0 {
                    log.add(
                        format!(
                            "The {} misses the {}!",
                            magr_template.name, mdef_template.name
                        ),
                        *turn,
                    );
                }
            }
        }

        result
    }

    fn hitmm(
        world: &mut SubWorld,
        magr_ent: Entity,
        mdef_ent: Entity,
        attack: &Attack,
        rng: &mut NetHackRng,
        assets: &AssetManager,
        _grid: &Grid,
        vision: &VisionSystem,
        log: &mut GameLog,
        turn: &u64,
        command_buffer: &mut CommandBuffer,
    ) -> u8 {
        let mut result = MHitResult::Hit as u8;

        let mdef_status = if let Ok(entry) = world.entry_ref(mdef_ent) {
            entry
                .get_component::<crate::core::entity::status::StatusBundle>()
                .map(|b| b.flags())
                .unwrap_or(crate::core::entity::status::StatusFlags::empty())
        } else {
            crate::core::entity::status::StatusFlags::empty()
        };

        let mdef_ac = if let Ok(entry) = world.entry_ref(mdef_ent) {
            entry
                .get_component::<CombatStats>()
                .map_or(10, |s| s.ac as i32)
        } else {
            10
        };

        let dmg = CombatEngine::calculate_monster_damage(rng, attack, mdef_ac, &mdef_status);

        if dmg > 0 {
            if let Ok(mut entry) = world.entry_mut(mdef_ent) {
                if let Ok(health) = entry.get_component_mut::<Health>() {
                    health.current -= dmg;
                    if health.current <= 0 {
                        result |= MHitResult::DefDied as u8;
                    }
                }
            }
        }

        let mdef_pos = if let Ok(entry) = world.entry_ref(mdef_ent) {
            entry.get_component::<Position>().ok().cloned()
        } else {
            None
        };

        let vis = mdef_pos.map_or(false, |p| Self::can_see(vision, &p));

        match attack.adtype {
            DamageType::Fire => {
                if vis {
                    log.add_colored("The attack is burning!", [255, 100, 0], *turn);
                }
            }
            DamageType::Cold => {
                if vis {
                    log.add_colored("The attack is freezing!", [100, 100, 255], *turn);
                }
            }
            DamageType::Elec => {
                if vis {
                    log.add_colored("The attack is shocking!", [255, 255, 0], *turn);
                }
            }
            _ => {}
        }

        if (result & MHitResult::DefDied as u8) != 0 {
            if vis {
                if let Ok(entry) = world.entry_ref(mdef_ent) {
                    if let Ok(m) = entry.get_component::<Monster>() {
                        log.add(format!("The {} is killed!", m.kind), *turn);
                    }
                }
            }
            command_buffer.remove(mdef_ent);
        }

        if (result & MHitResult::DefDied as u8) == 0 {
            let pass_res = Self::passivemm(
                world,
                magr_ent,
                mdef_ent,
                true,
                (result & MHitResult::DefDied as u8) != 0,
                rng,
                assets,
                vision,
                log,
                turn,
                command_buffer,
            );
            result |= pass_res;
        }

        result
    }

    pub fn passivemm(
        world: &mut SubWorld,
        magr_ent: Entity,
        mdef_ent: Entity,
        mhit: bool,
        mdead: bool,
        rng: &mut NetHackRng,
        assets: &AssetManager,
        vision: &VisionSystem,
        log: &mut GameLog,
        turn: &u64,
        command_buffer: &mut CommandBuffer,
    ) -> u8 {
        let mut result = 0;
        if mdead {
            return 0;
        }

        let mdef_t_name = if let Ok(entry) = world.entry_ref(mdef_ent) {
            entry
                .get_component::<Monster>()
                .map(|m| m.kind.to_string())
                .ok()
        } else {
            None
        };
        let magr_t_name = if let Ok(entry) = world.entry_ref(magr_ent) {
            entry
                .get_component::<Monster>()
                .map(|m| m.kind.to_string())
                .ok()
        } else {
            None
        };

        if mdef_t_name.is_none() || magr_t_name.is_none() {
            return 0;
        }
        let mdef_t = if let Some(t) = assets
            .monsters
            .templates
            .get(mdef_t_name.as_ref().unwrap().as_str())
        {
            t
        } else {
            return 0;
        };
        let magr_t = if let Some(t) = assets
            .monsters
            .templates
            .get(magr_t_name.as_ref().unwrap().as_str())
        {
            t
        } else {
            return 0;
        };

        let vis = if let Ok(entry) = world.entry_ref(mdef_ent) {
            entry
                .get_component::<Position>()
                .map_or(false, |p| Self::can_see(vision, p))
        } else {
            false
        };

        for attack in &mdef_t.attacks {
            if attack.atype == AttackType::None {
                match attack.adtype {
                    DamageType::Acid => {
                        if mhit && rng.rn2(2) == 0 {
                            if vis {
                                log.add(
                                    format!("The {} is splashed with acid!", magr_t.name),
                                    *turn,
                                );
                            }
                            let dmg = rng.d(attack.dice as i32, attack.sides as i32);
                            if let Ok(mut entry) = world.entry_mut(magr_ent) {
                                if let Ok(health) = entry.get_component_mut::<Health>() {
                                    health.current -= dmg;
                                    if health.current <= 0 {
                                        result |= MHitResult::AgrDied as u8;
                                        command_buffer.remove(magr_ent);
                                    }
                                }
                            }
                        }
                    }
                    DamageType::Plys => {
                        if !mdead && rng.rn2(3) == 0 {
                            if vis {
                                log.add(
                                    format!(
                                        "The {} is frozen by the {}'s gaze!",
                                        magr_t.name, mdef_t.name
                                    ),
                                    *turn,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        result
    }

    fn can_see(vision: &VisionSystem, pos: &Position) -> bool {
        let ux = pos.x as usize;
        let uy = pos.y as usize;
        if ux < crate::core::dungeon::COLNO && uy < crate::core::dungeon::ROWNO {
            vision.viz_array[ux][uy] & crate::core::systems::vision::IN_SIGHT != 0
        } else {
            false
        }
    }
}

// =============================================================================
// [v2.3.3] 특수 공격 효과 (원본 mhitm.c: special attack effects)
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialAttackEffect {
    None,
    Poison,
    Petrify,
    Slime,
    Teleport,
    Acid,
    Drain,
    Disease,
    Stun,
    Paralyze,
    Steal,
    Seduce,
    Aging,
}

/// 특수 공격 결과
#[derive(Debug, Clone)]
pub struct SpecialAttackResult {
    pub effect: SpecialAttackEffect,
    pub damage_bonus: i32,
    pub message: String,
    pub defender_status_change: Option<String>,
    pub defender_killed: bool,
}

/// 특수 공격 해결 (원본: damageum / mhitm2)
pub fn resolve_special_attack(
    _attack_type: AttackType,
    damage_type: DamageType,
    base_damage: i32,
    defender_has_resistance: bool,
    rng: &mut NetHackRng,
) -> SpecialAttackResult {
    let effect = match damage_type {
        DamageType::Drst => SpecialAttackEffect::Poison,
        DamageType::Ston => SpecialAttackEffect::Petrify,
        DamageType::Acid => SpecialAttackEffect::Acid,
        DamageType::Drli => SpecialAttackEffect::Drain,
        DamageType::Stun => SpecialAttackEffect::Stun,
        DamageType::Plys => SpecialAttackEffect::Paralyze,
        _ => SpecialAttackEffect::None,
    };

    let (dmg_bonus, msg, status) = match effect {
        SpecialAttackEffect::Poison => {
            if defender_has_resistance {
                (0, "The poison doesn't seem to affect it.", None)
            } else {
                let extra = rng.rn2(6) + 1;
                (extra, "It is poisoned!", Some("Poisoned".to_string()))
            }
        }
        SpecialAttackEffect::Petrify => {
            if defender_has_resistance {
                (0, "It seems unaffected.", None)
            } else {
                (0, "It is turning to stone!", Some("Stoning".to_string()))
            }
        }
        SpecialAttackEffect::Acid => {
            if defender_has_resistance {
                (0, "The acid doesn't harm it.", None)
            } else {
                (rng.rn2(4) + 2, "It is splashed with acid!", None)
            }
        }
        SpecialAttackEffect::Drain => {
            if defender_has_resistance {
                (0, "It resists the drain.", None)
            } else {
                (
                    base_damage / 4,
                    "It feels weaker!",
                    Some("Drained".to_string()),
                )
            }
        }
        SpecialAttackEffect::Stun => (0, "It staggers!", Some("Stunned".to_string())),
        SpecialAttackEffect::Paralyze => {
            if defender_has_resistance {
                (0, "It shakes off the paralysis.", None)
            } else {
                (0, "It is paralyzed!", Some("Paralyzed".to_string()))
            }
        }
        _ => (0, "", None),
    };

    SpecialAttackResult {
        effect,
        damage_bonus: dmg_bonus,
        message: msg.to_string(),
        defender_status_change: status,
        defender_killed: false,
    }
}

// =============================================================================
// [v2.3.3] 데미지 타입별 보너스 (원본 mhitm.c)
// =============================================================================

/// 데미지 타입 보너스 (원본: damage_type switch)
pub fn damage_type_bonus(damage_type: DamageType, base: i32) -> i32 {
    match damage_type {
        DamageType::Phys => 0,
        DamageType::Fire => base / 3,
        DamageType::Cold => base / 4,
        DamageType::Elec => base / 3,
        DamageType::Acid => base / 2,
        DamageType::Magm => base / 5,
        _ => 0,
    }
}

/// 공격 유형 명중 보정 (원본: attack_type weight)
pub fn attack_type_hit_modifier(attack_type: AttackType) -> i32 {
    match attack_type {
        AttackType::Claw => 2,
        AttackType::Bite => 1,
        AttackType::Kick => 0,
        AttackType::Butt => -1,
        AttackType::Touch => 3,
        AttackType::Sting => 1,
        AttackType::Hugs => -2,
        AttackType::Breath => 5,
        AttackType::Spit => 0,
        AttackType::Gaze => 10,
        AttackType::Engulf => -3,
        AttackType::Weapon => 0,
        _ => 0,
    }
}

// =============================================================================
// [v2.3.3] 몬스터 전투 AI (원본 mhitm.c)
// =============================================================================

/// 공격 선택 AI (원본: 효과적 공격 우선)
pub fn select_best_attack<'a>(
    attacks: &'a [Attack],
    defender_resistances: u32,
) -> Option<&'a Attack> {
    let mut best: Option<&'a Attack> = None;
    let mut best_score = i32::MIN;

    for attack in attacks {
        let mut score = attack.dice as i32;
        score += damage_type_bonus(attack.adtype, attack.dice as i32);

        let has_resist = match attack.adtype {
            DamageType::Fire => defender_resistances & 0x01 != 0,
            DamageType::Cold => defender_resistances & 0x02 != 0,
            DamageType::Elec => defender_resistances & 0x04 != 0,
            DamageType::Drst => defender_resistances & 0x08 != 0,
            DamageType::Acid => defender_resistances & 0x10 != 0,
            _ => false,
        };
        if has_resist {
            score -= attack.dice as i32;
        }
        score += attack_type_hit_modifier(attack.atype);
        if score > best_score {
            best_score = score;
            best = Some(attack);
        }
    }
    best
}

/// 도주 판정 (원본: mflee)
pub fn should_monster_flee(current_hp: i32, max_hp: i32, fear: i32) -> bool {
    if max_hp <= 0 {
        return true;
    }
    let pct = (current_hp * 100) / max_hp;
    if pct <= 15 {
        return true;
    }
    if fear >= 3 {
        return true;
    }
    if pct <= 30 && fear >= 1 {
        return true;
    }
    false
}

// =============================================================================
// [v2.3.3] 전투 통계
// =============================================================================

/// 전투 통계
#[derive(Debug, Clone, Default)]
pub struct MonsterCombatStats {
    pub total_attacks: u32,
    pub total_hits: u32,
    pub total_misses: u32,
    pub total_kills: u32,
    pub total_damage_dealt: u64,
    pub special_attacks_used: u32,
}

impl MonsterCombatStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_attack(&mut self, hit: bool, damage: i32, special: bool) {
        self.total_attacks += 1;
        if hit {
            self.total_hits += 1;
            self.total_damage_dealt += damage as u64;
        } else {
            self.total_misses += 1;
        }
        if special {
            self.special_attacks_used += 1;
        }
    }

    pub fn record_kill(&mut self) {
        self.total_kills += 1;
    }

    pub fn hit_rate(&self) -> f64 {
        if self.total_attacks == 0 {
            0.0
        } else {
            (self.total_hits as f64) / (self.total_attacks as f64) * 100.0
        }
    }
}

// =============================================================================
// [v2.3.3] 패시브 효과 (원본 mhitm.c: passivemm)
// =============================================================================

/// 패시브 반격 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassiveEffect {
    None,
    AcidSplash,
    ShockTouch,
    ColdAura,
    FireAura,
    PoisonSpine,
    StoneGaze,
}

/// 패시브 반격 결과
#[derive(Debug, Clone)]
pub struct PassiveResult {
    pub effect: PassiveEffect,
    pub damage: i32,
    pub message: String,
}

/// 패시브 반격 해결
pub fn resolve_passive(
    passive: PassiveEffect,
    resist: bool,
    rng: &mut NetHackRng,
) -> PassiveResult {
    match passive {
        PassiveEffect::AcidSplash => PassiveResult {
            effect: passive,
            damage: if resist { 0 } else { rng.rn2(4) + 1 },
            message: if resist {
                "The acid doesn't harm it."
            } else {
                "Splashed by acid!"
            }
            .to_string(),
        },
        PassiveEffect::ShockTouch => PassiveResult {
            effect: passive,
            damage: if resist { 0 } else { rng.rn2(6) + 2 },
            message: if resist {
                "Unaffected by shock."
            } else {
                "Gets shocked!"
            }
            .to_string(),
        },
        PassiveEffect::ColdAura => PassiveResult {
            effect: passive,
            damage: if resist { 0 } else { rng.rn2(4) + 1 },
            message: if resist {
                "Unaffected by cold."
            } else {
                "Frozen!"
            }
            .to_string(),
        },
        PassiveEffect::FireAura => PassiveResult {
            effect: passive,
            damage: if resist { 0 } else { rng.rn2(6) + 2 },
            message: if resist {
                "Unaffected by fire."
            } else {
                "Burned!"
            }
            .to_string(),
        },
        PassiveEffect::PoisonSpine => PassiveResult {
            effect: passive,
            damage: if resist { 0 } else { rng.rn2(3) + 1 },
            message: if resist {
                "Poison doesn't affect it."
            } else {
                "Pricked by poison!"
            }
            .to_string(),
        },
        PassiveEffect::StoneGaze => PassiveResult {
            effect: passive,
            damage: 0,
            message: if resist {
                "Resists petrification."
            } else {
                "Turning to stone!"
            }
            .to_string(),
        },
        PassiveEffect::None => PassiveResult {
            effect: passive,
            damage: 0,
            message: String::new(),
        },
    }
}

// =============================================================================
// [v2.3.3] 테스트
// =============================================================================
#[cfg(test)]
mod mhitm_extended_tests {
    use super::*;

    #[test]
    fn test_special_attack_poison() {
        let mut rng = NetHackRng::new(42);
        let r = resolve_special_attack(AttackType::Bite, DamageType::Drst, 10, false, &mut rng);
        assert_eq!(r.effect, SpecialAttackEffect::Poison);
        assert!(r.damage_bonus > 0);
    }

    #[test]
    fn test_special_attack_resist() {
        let mut rng = NetHackRng::new(42);
        let r = resolve_special_attack(AttackType::Bite, DamageType::Drst, 10, true, &mut rng);
        assert_eq!(r.damage_bonus, 0);
    }

    #[test]
    fn test_damage_type_bonus() {
        assert_eq!(damage_type_bonus(DamageType::Phys, 12), 0);
        assert!(damage_type_bonus(DamageType::Fire, 12) > 0);
    }

    #[test]
    fn test_should_flee() {
        assert!(should_monster_flee(5, 100, 0));
        assert!(should_monster_flee(50, 100, 3));
        assert!(!should_monster_flee(80, 100, 0));
    }

    #[test]
    fn test_combat_stats() {
        let mut s = MonsterCombatStats::new();
        s.record_attack(true, 10, false);
        s.record_attack(false, 0, false);
        s.record_kill();
        assert_eq!(s.total_attacks, 2);
        assert_eq!(s.total_hits, 1);
        assert_eq!(s.total_kills, 1);
    }

    #[test]
    fn test_passive_acid() {
        let mut rng = NetHackRng::new(42);
        let r = resolve_passive(PassiveEffect::AcidSplash, false, &mut rng);
        assert!(r.damage > 0);
    }

    #[test]
    fn test_passive_resist() {
        let mut rng = NetHackRng::new(42);
        let r = resolve_passive(PassiveEffect::ShockTouch, true, &mut rng);
        assert_eq!(r.damage, 0);
    }
}

// =============================================================================
// [v2.3.4] 몬스터 전투 확장 (원본 mhitm.c: advanced monster combat)
// =============================================================================

///
pub fn battle_win_chance(
    attacker_level: i32,
    attacker_ac: i32,
    defender_level: i32,
    defender_ac: i32,
) -> f32 {
    // 공격력(레벨)과 방어력(AC)으로 추정 승률 계산
    let atk_power = (attacker_level * 2 + (10 - attacker_ac)) as f32;
    let def_power = (defender_level * 2 + (10 - defender_ac)) as f32;
    let total = atk_power + def_power;
    if total <= 0.0 {
        return 0.5;
    }
    (atk_power / total).max(0.05).min(0.95)
}

/// 공격 유형별 기대 데미지 (원본: mhitm.c expected damage)
pub fn attack_expected_damage(attack_type: &str, attacker_level: i32) -> (i32, i32) {
    // (최소, 최대) 데미지
    let l = attack_type.to_lowercase();
    if l.contains("claw") {
        return (1, attacker_level.max(4));
    }
    if l.contains("bite") {
        return (2, attacker_level.max(6));
    }
    if l.contains("kick") {
        return (1, attacker_level.max(5));
    }
    if l.contains("butt") {
        return (2, attacker_level.max(8));
    }
    if l.contains("touch") {
        return (0, attacker_level.max(3));
    }
    if l.contains("sting") {
        return (1, attacker_level.max(4));
    }
    if l.contains("engulf") {
        return (attacker_level / 2, attacker_level * 2);
    }
    if l.contains("breath") {
        return (attacker_level, attacker_level * 3);
    }
    if l.contains("gaze") {
        return (0, 0);
    } // 시선 공격은 특수 효과
      // 기본 물리 공격
    (1, attacker_level.max(4))
}

/// 몬스터 공포 판정 (원본: mhitm.c fear check)
pub fn monster_fears_opponent(
    monster_level: i32,
    monster_hp_ratio: f32,
    opponent_level: i32,
) -> bool {
    // HP 낮고 상대 레벨 높으면 공포
    if monster_hp_ratio > 0.5 {
        return false;
    }
    let fear_threshold = opponent_level - monster_level;
    fear_threshold >= 3 || (fear_threshold >= 1 && monster_hp_ratio < 0.2)
}

/// 도주 방향 판정 (원본: mhitm.c flee direction)
pub fn flee_priority(monster_x: i32, monster_y: i32, threat_x: i32, threat_y: i32) -> (i32, i32) {
    //
    let dx = if monster_x > threat_x {
        1
    } else if monster_x < threat_x {
        -1
    } else {
        0
    };
    let dy = if monster_y > threat_y {
        1
    } else if monster_y < threat_y {
        -1
    } else {
        0
    };
    (dx, dy)
}

/// 방어 유형 (원본: mhitm.c defense type)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenseType {
    None,
    Dodge,       // 회피
    Block,       // 방어
    Parry,       // 패링
    MagicShield, // 마법 실드
    Reflect,     // 반사
}

/// 방어 유형 판정
pub fn determine_defense(
    defender_ac: i32,
    has_shield: bool,
    has_reflection: bool,
    attack_is_magical: bool,
) -> DefenseType {
    if attack_is_magical && has_reflection {
        return DefenseType::Reflect;
    }
    if has_shield && defender_ac < 3 {
        return DefenseType::Block;
    }
    if defender_ac < 0 {
        return DefenseType::MagicShield;
    }
    if defender_ac < 5 {
        return DefenseType::Parry;
    }
    DefenseType::None
}

/// 전투 보고서 (원본: mhitm.c battle report)
pub fn battle_report_message(
    attacker: &str,
    defender: &str,
    hit: bool,
    damage: i32,
    killed: bool,
) -> String {
    if killed {
        format!("{} kills {}!", attacker, defender)
    } else if hit && damage > 0 {
        format!("{} hits {} for {} damage.", attacker, defender, damage)
    } else if hit {
        format!("{} hits {} but does no damage.", attacker, defender)
    } else {
        format!("{} misses {}.", attacker, defender)
    }
}

/// 몬스터 전투 통계 확장
#[derive(Debug, Clone, Default)]
pub struct MhitmExtendedStats {
    pub monster_battles: u32,
    pub monsters_fled: u32,
    pub passive_damages: u32,
    pub defenses_triggered: u32,
    pub battle_kills: u32,
}

impl MhitmExtendedStats {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_battle(&mut self, killed: bool) {
        self.monster_battles += 1;
        if killed {
            self.battle_kills += 1;
        }
    }
    pub fn record_flee(&mut self) {
        self.monsters_fled += 1;
    }
}

#[cfg(test)]
mod mhitm_advanced_tests {
    use super::*;

    #[test]
    fn test_battle_prediction() {
        let chance = battle_win_chance(10, 0, 5, 5);
        assert!(chance > 0.5);
    }

    #[test]
    fn test_expected_damage() {
        let (min, max) = attack_expected_damage("bite", 8);
        assert!(max >= min);
        assert!(max >= 6);
    }

    #[test]
    fn test_fear() {
        assert!(monster_fears_opponent(3, 0.1, 10));
        assert!(!monster_fears_opponent(10, 0.8, 3));
    }

    #[test]
    fn test_flee_direction() {
        let (dx, dy) = flee_priority(5, 5, 3, 3);
        assert_eq!((dx, dy), (1, 1));
    }

    #[test]
    fn test_defense() {
        assert_eq!(
            determine_defense(-2, false, false, false),
            DefenseType::MagicShield
        );
        assert_eq!(
            determine_defense(5, false, true, true),
            DefenseType::Reflect
        );
    }

    #[test]
    fn test_report() {
        let msg = battle_report_message("kobold", "rat", true, 3, false);
        assert!(msg.contains("hits"));
        let msg2 = battle_report_message("dragon", "goblin", true, 50, true);
        assert!(msg2.contains("kills"));
    }

    #[test]
    fn test_mhitm_stats() {
        let mut s = MhitmExtendedStats::new();
        s.record_battle(true);
        s.record_flee();
        assert_eq!(s.battle_kills, 1);
        assert_eq!(s.monsters_fled, 1);
    }
}

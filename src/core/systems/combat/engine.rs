use crate::core::entity::monster::{Attack, AttackType, DamageType, MonsterState, MonsterTemplate};
use crate::core::entity::object::ItemTemplate;
use crate::core::entity::player::{Player, PlayerClass};
use crate::core::entity::skills::{SkillLevel, WeaponSkill};
use crate::core::entity::status::StatusFlags;
use crate::core::entity::{Health, Position};
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::{EntityStore, SubWorld};
use legion::{Entity, IntoQuery};

/// 전투 결과 정보
#[derive(Debug)]
pub struct AttackResult {
    pub hit: bool,
    pub damage: i32,
    pub adtype: DamageType,
    pub atype: AttackType,
}

pub struct CombatEngine;

impl CombatEngine {
    /// 구조물 공격 처리 (Phase 50.1 RAID)
    pub fn attack_structure(
        rng: &mut NetHackRng,
        structure: &mut crate::core::entity::Structure,
    ) -> (i32, bool) {
        //
        let base_damage = rng.d(1, 8) + 2;
        structure.integrity -= base_damage;
        let destroyed = structure.integrity <= 0;
        (base_damage, destroyed)
    }

    /// 원본 NetHack의 명중률 보정 (weapon.c: abon() 이식)
    pub fn abon(player: &Player) -> i32 {
        let mut sbon: i32;
        let str_val = player.str.base; // 18/xx = 18+xx, 19 = 119...
        let dex_val = player.dex.base;

        // Strength bonus
        if str_val < 6 {
            sbon = -2;
        } else if str_val < 8 {
            sbon = -1;
        } else if str_val < 17 {
            sbon = 0;
        } else if str_val <= 68 {
            // STR18(50)
            sbon = 1;
        } else if str_val < 118 {
            // STR18(100)
            sbon = 2;
        } else {
            sbon = 3;
        }

        //
        if player.exp_level < 3 {
            sbon += 1;
        }

        // Dex bonus
        if dex_val < 4 {
            sbon - 3
        } else if dex_val < 6 {
            sbon - 2
        } else if dex_val < 8 {
            sbon - 1
        } else if dex_val < 14 {
            sbon
        } else {
            sbon + dex_val - 14
        }
    }

    /// 데미지 보정 (weapon.c: dbon() 이식)
    pub fn dbon(player: &Player) -> i32 {
        let str_val = player.str.base;
        if str_val < 6 {
            -1
        } else if str_val < 16 {
            0
        } else if str_val < 18 {
            1
        } else if str_val == 18 {
            2
        } else if str_val <= 93 {
            // STR18(75)
            3
        } else if str_val <= 108 {
            // STR18(90)
            4
        } else if str_val < 118 {
            // STR18(100)
            5
        } else {
            6
        }
    }

    /// NetHack AC 가치 계산 (hack.h: AC_VALUE 이식)
    ///
    pub fn ac_value(rng: &mut NetHackRng, ac: i32) -> i32 {
        if ac >= 0 {
            ac
        } else {
            //
            -rng.rnd(-ac)
        }
    }

    ///
    pub fn calculate_player_hit(
        rng: &mut NetHackRng,
        player: &Player,
        _status: &crate::core::entity::status::StatusBundle,
        level: i32,
        target: &MonsterTemplate,
        mstate: Option<&MonsterState>,
        target_stats: Option<&crate::core::entity::CombatStats>,
        weapon: Option<(&crate::core::entity::Item, &ItemTemplate)>,
        aatyp: AttackType,
        offhand_attack: bool,
    ) -> bool {
        // tmp = 1 + Luck + abon() + monster_ac + level
        // NetHack 3.6.7 uhitm.c:find_roll_to_hit()
        let monster_ac = if let Some(stats) = target_stats {
            stats.ac
        } else {
            target.ac as i32
        };

        let mut tmp = 1 + player.luck + Self::abon(player) + monster_ac + level;

        // 쌍수 공격 페널티 (NetHack 3.6.7 이식)
        if player.two_weapon {
            //
            tmp -= if offhand_attack { 6 } else { 2 };

            // 스킬에 따른 추가 페널티 (-20에서 위로 상쇄)
            let tw_skill = player
                .skills
                .get(&WeaponSkill::TwoWeapon)
                .map(|r| r.level)
                .unwrap_or(SkillLevel::Unskilled);
            let tw_penalty = match tw_skill {
                SkillLevel::Restricted | SkillLevel::Unskilled => -20,
                SkillLevel::Basic => -10,
                SkillLevel::Skilled => -5,
                SkillLevel::Expert | SkillLevel::Master | SkillLevel::GrandMaster => 0,
            };
            tmp += tw_penalty;
        }

        // 몬스터 상태 보정
        if let Some(ms) = mstate {
            if ms.mstun {
                tmp += 2;
            }
            if ms.mflee {
                tmp += 2;
            }
            if ms.msleeping {
                tmp += 2;
            }
            if !ms.mcanmove {
                tmp += 4;
            }
            if ms.mconf {
                tmp += 2;
            }
        }

        // 수도승(Monk) 보술 보너스
        if player.role == PlayerClass::Monk && weapon.is_none() {
            tmp += (level / 3) + 2;
        }

        if aatyp == AttackType::Weapon || aatyp == AttackType::Claw {
            if let Some((w_inst, w_tmpl)) = weapon {
                tmp += Self::hitval(w_inst, w_tmpl, target);
            }
            tmp += Self::weapon_hit_bonus(player, weapon.map(|(_, t)| t));
        }

        // 보이지 않는 적에 대한 패널티
        //

        // 다이스 롤 (1~20)
        let dieroll = rng.rnd(20);

        // NetHack: 1은 무조건 실패, 20은 무조건 성공 (NATURAL_20)
        if dieroll == 1 {
            return false;
        }
        if dieroll == 20 {
            return true;
        }

        tmp > dieroll
    }

    ///
    pub fn calculate_monster_hit(
        rng: &mut NetHackRng,
        monster_level: i32,
        player_ac: i32,
        attack_index: usize,
        player_status: &StatusFlags,
        is_invisible_to_monster: bool,
    ) -> bool {
        // tmp = AC_VALUE(u.uac) + 10 + m_lev
        let mut final_ac = player_ac;
        if player_status.contains(StatusFlags::PROTECTION) {
            final_ac -= 2; // 임시: 보호 상태일 때 AC -2 보너스
        }

        let mut tmp = Self::ac_value(rng, final_ac) + 10 + monster_level;

        if player_status.contains(StatusFlags::STUNNED) {
            tmp += 4;
        }
        if is_invisible_to_monster {
            tmp -= 2;
        }
        if player_status.contains(StatusFlags::DISPLACED) {
            tmp -= 2; // 분신 효과로 명중률 하락
        }

        if tmp <= 0 {
            tmp = 1;
        }

        let roll = rng.rnd(20 + attack_index as i32);
        tmp > roll
    }

    ///
    pub fn martial_bonus(player: &Player) -> bool {
        player.role == PlayerClass::Monk || player.role == PlayerClass::Samurai
    }

    /// 무기 고유 명중 보너스 (weapon.c: hitval 이식)
    pub fn hitval(
        wep: &crate::core::entity::Item,
        tmpl: &ItemTemplate,
        _target: &MonsterTemplate,
    ) -> i32 {
        let mut tmp = wep.spe as i32;

        // 특정 몬스터 타입에 대한 명중 보너스 (예: Spear vs Snake)
        // NetHack 3.6.7: objects.c의 oc_hitbon 및 hitval 로직
        tmp += tmpl.oc1 as i32; // oc_hitbon 이 oc1에 저장됨

        //
        //

        tmp
    }

    ///
    pub fn calculate_player_damage(
        rng: &mut NetHackRng,
        player: &Player,
        monster: &MonsterTemplate,
        mstate: &MonsterState,
        weapon: Option<(&crate::core::entity::Item, &ItemTemplate)>,
        thrown: bool,
        offhand_attack: bool,
        artifact_manager: &crate::core::entity::artifact::ArtifactManager,
    ) -> i32 {
        let mut tmp: i32;

        if weapon.is_none() {
            // 맨손 공격 (uhitm.c:704)
            if monster.name == "shade" {
                tmp = 0;
            } else if Self::martial_bonus(player) {
                //
                tmp = rng.rnz(1, 4);
            } else {
                tmp = rng.rnz(1, 2);
            }
            // 맨손 스킬 데미지 보너스 적용
            tmp += Self::weapon_dam_bonus(player, None);

            //
        } else {
            let (w_inst, w_tmpl) = weapon.unwrap();
            tmp = Self::dmgval(rng, player, w_inst, w_tmpl, monster, offhand_attack);
        }

        // 로그의 배후 공격 (uhitm.c:775)
        if mstate.mflee && player.role == PlayerClass::Rogue && !thrown {
            // "You strike from behind!"
            tmp += rng.rnd(player.exp_level as i32);
        }

        // 특별 보너스 데미지 (은, 축복 등)
        if let Some((w_inst, w_tmpl)) = weapon {
            tmp += Self::special_dmgval(rng, w_inst, w_tmpl, monster, artifact_manager);
        }

        // 스킬 데미지 보너스 (weapon_dam_bonus) 적용
        if let Some((_, tmpl)) = weapon {
            tmp += Self::weapon_dam_bonus(player, Some(tmpl));
        }

        if tmp < 1 {
            tmp = 1;
        }
        tmp
    }

    /// 무기 데미지 기본 계산 (weapon.c: dmgval 이식)
    pub fn dmgval(
        rng: &mut NetHackRng,
        player: &Player,
        wep: &crate::core::entity::Item,
        tmpl: &ItemTemplate,
        target: &MonsterTemplate,
        offhand_attack: bool,
    ) -> i32 {
        let mut tmp: i32;

        // 크기별 데미지 주사위 (S/L 구분)
        if target.msize >= 4 {
            // MZ_LARGE 이상
            if tmpl.wldam > 0 {
                tmp = rng.rnd(tmpl.wldam as i32);
            } else {
                tmp = 1;
            }
        } else {
            // MZ_SMALL, MZ_MEDIUM
            if tmpl.wsdam > 0 {
                tmp = rng.rnd(tmpl.wsdam as i32);
            } else {
                tmp = 1;
            }
        }

        // 강화(spe) 보너스
        if tmpl.class == crate::core::entity::object::ItemClass::Weapon
            || tmpl.class == crate::core::entity::object::ItemClass::Tool
        {
            tmp += wep.spe as i32;
        }

        // 힘(Str) 보너스 (dbon)
        //
        let tw_skill = player
            .skills
            .get(&WeaponSkill::TwoWeapon)
            .map(|r| r.level)
            .unwrap_or(SkillLevel::Unskilled);
        if !offhand_attack || tw_skill >= SkillLevel::Skilled {
            tmp += Self::dbon(player);
        }

        // 부식(oeroded) 패널티 (weapon.c: dmgval)
        if tmpl.material == crate::core::entity::object::Material::Iron
            || tmpl.material == crate::core::entity::object::Material::Metal
        {
            tmp -= wep.oeroded as i32;
        }

        if tmp < 1 {
            tmp = 1;
        }
        tmp
    }

    /// 특별 보너스 데미지 (weapon.c: special_dmgval 이식)
    pub fn special_dmgval(
        rng: &mut NetHackRng,
        wep: &crate::core::entity::Item,
        tmpl: &ItemTemplate,
        target: &MonsterTemplate,
        artifact_manager: &crate::core::entity::artifact::ArtifactManager,
    ) -> i32 {
        let mut tmp = 0;

        // 은제 무기 보너스
        if tmpl.material == crate::core::entity::object::Material::Silver {
            // NetHack 3.6.7: weapon.c:special_dmgval()
            if target.hates_silver() {
                tmp += rng.rnd(20);
            }
        }

        //
        if wep.blessed
            && (target.has_capability(crate::core::entity::capability::MonsterCapability::Undead)
                || target.has_capability(crate::core::entity::capability::MonsterCapability::Demon))
        {
            tmp += rng.rnz(1, 4);
        }

        //
        if let Some(art_id) = &wep.artifact {
            if let Some(art) = artifact_manager.get_artifact(art_id) {
                // 1. 보너스 주사위
                if let Some(sides) = art.bonus_dice {
                    tmp += rng.rnd(sides);
                }

                // 2. 종족 혐오 (배수 데미지 또는 추가 고정 보너스)
                if let Some(hate) = &art.hate_species {
                    //
                    if target.name.to_lowercase().contains(&hate.to_lowercase()) {
                        if art.double_damage {
                            //
                            //
                            tmp += 10; // 임시 고정 보너스
                        } else {
                            tmp += 5;
                        }
                    }
                }
            }
        }

        tmp
    }

    ///
    pub fn calculate_monster_damage(
        rng: &mut NetHackRng,
        attack: &Attack,
        player_ac: i32,
        player_status: &StatusFlags,
    ) -> i32 {
        if attack.dice <= 0 || attack.sides <= 0 {
            return 0;
        }

        let mut dmg = rng.d(attack.dice as i32, attack.sides as i32);

        // 음수 AC의 데미지 감소 효과 (dmg -= rnd(-u.uac))
        if dmg > 0 && player_ac < 0 {
            dmg -= rng.rnd(-player_ac);
        }

        // 절반 데미지 보정 (Half Damage)
        if player_status.contains(StatusFlags::HALF_DMG) && dmg > 1 {
            dmg = (dmg + 1) / 2;
        }

        if dmg < 1 {
            dmg = 1;
        }
        dmg
    }

    ///
    pub fn passive(
        rng: &mut NetHackRng,
        world: &mut SubWorld,
        p_ent: Entity,
        _player_clone: &Player,
        monster_tmpl: &MonsterTemplate,
        hit: bool,
        dead: bool,
        log: &mut crate::ui::log::GameLog,
        turn: &u64,
        _assets: &crate::assets::AssetManager,
        _command_buffer: &mut CommandBuffer,
        _p_level: crate::core::dungeon::LevelID,
    ) {
        if dead {
            return;
        }

        // 분열 체크 (Phase 17.2)
        if hit && Self::try_split(rng, monster_tmpl) {
            //
            //
            //
        }

        let mut has_gloves = false;
        if let Ok(entry) = world.entry_ref(p_ent) {
            if let Ok(eq) = entry.get_component::<crate::core::entity::Equipment>() {
                if eq
                    .slots
                    .contains_key(&crate::core::entity::EquipmentSlot::Hands)
                {
                    has_gloves = true;
                }
            }
        }

        for attack in &monster_tmpl.attacks {
            // 수동형 반격은 atype이 None인 것들 중에서 발생함
            if attack.atype == AttackType::None {
                match attack.adtype {
                    DamageType::Acid => {
                        if hit && rng.rn2(2) == 0 {
                            if has_gloves {
                                log.add("Your gloves protect you from the acid.", *turn);
                            } else {
                                log.add_colored(
                                    format!("The {}'s acid splashes you!", monster_tmpl.name),
                                    [100, 255, 100],
                                    *turn,
                                );
                                let dmg = rng.d(attack.dice as i32, attack.sides as i32);
                                if let Ok(mut entry) = world.entry_mut(p_ent) {
                                    if let Ok(health) = entry.get_component_mut::<Health>() {
                                        health.current -= dmg;
                                    }
                                }
                            }
                        }
                    }
                    DamageType::Fire => {
                        if hit && rng.rn2(2) == 0 {
                            log.add_colored(
                                format!("The {} is burning hot!", monster_tmpl.name),
                                [255, 100, 0],
                                *turn,
                            );
                            let dmg = rng.d(attack.dice as i32, attack.sides as i32);
                            if let Ok(mut entry) = world.entry_mut(p_ent) {
                                if let Ok(health) = entry.get_component_mut::<Health>() {
                                    health.current -= dmg;
                                }
                            }
                        }
                    }
                    DamageType::Cold => {
                        if hit && rng.rn2(2) == 0 {
                            log.add_colored(
                                format!("The {} is freezing cold!", monster_tmpl.name),
                                [100, 100, 255],
                                *turn,
                            );
                            let dmg = rng.d(attack.dice as i32, attack.sides as i32);
                            if let Ok(mut entry) = world.entry_mut(p_ent) {
                                if let Ok(health) = entry.get_component_mut::<Health>() {
                                    health.current -= dmg;
                                }
                            }
                        }
                    }
                    DamageType::Elec => {
                        if hit && rng.rn2(2) == 0 {
                            log.add_colored(
                                format!("The {} shocks you!", monster_tmpl.name),
                                [255, 255, 0],
                                *turn,
                            );
                            let dmg = rng.d(attack.dice as i32, attack.sides as i32);
                            if let Ok(mut entry) = world.entry_mut(p_ent) {
                                if let Ok(health) = entry.get_component_mut::<Health>() {
                                    health.current -= dmg;
                                }
                            }
                        }
                    }
                    DamageType::Plys => {
                        // Floating Eye 마비 등의 효과 (TODO: 수면 저항 체크 추가)
                        if hit && rng.rn2(3) == 0 {
                            log.add_colored(
                                format!("You are frozen by the {}'s gaze!", monster_tmpl.name),
                                [200, 200, 255],
                                *turn,
                            );
                            if let Ok(mut entry) = world.entry_mut(p_ent) {
                                if let Ok(status) = entry
                                    .get_component_mut::<crate::core::entity::status::StatusBundle>(
                                    )
                                {
                                    status.add(
                                        crate::core::entity::status::StatusFlags::PARALYZED,
                                        5,
                                    );
                                }
                            }
                        }
                    }
                    DamageType::Ston => {
                        if hit && !has_gloves {
                            log.add_colored(
                                format!("Touching the {} stones you!", monster_tmpl.name),
                                [200, 200, 200],
                                *turn,
                            );
                            if let Ok(mut entry) = world.entry_mut(p_ent) {
                                if let Ok(health) = entry.get_component_mut::<Health>() {
                                    health.current = 0; // 즉사
                                }
                            }
                        }
                    }
                    DamageType::Rust => {
                        if hit && rng.rn2(2) == 0 {
                            Self::damage_equipment(
                                rng,
                                world,
                                p_ent,
                                _assets,
                                DamageType::Rust,
                                log,
                                *turn,
                            );
                        }
                    }
                    DamageType::Corr => {
                        if hit && rng.rn2(2) == 0 {
                            Self::damage_equipment(
                                rng,
                                world,
                                p_ent,
                                _assets,
                                DamageType::Corr,
                                log,
                                *turn,
                            );
                        }
                    }
                    DamageType::Drli => {
                        // 레벨 드레인 (AD_DRLI)
                        if hit && rng.rn2(3) == 0 {
                            log.add_colored(format!("You feel weaker!"), [150, 0, 150], *turn);
                            if let Ok(mut entry) = world.entry_mut(p_ent) {
                                if let Ok(p) = entry.get_component_mut::<Player>() {
                                    if p.exp_level > 1 {
                                        p.exp_level -= 1;
                                        // HP 감소 및 경험치 손실 (원본 3.6.7 로직 간략화 이식)
                                        if let Ok(h) = entry.get_component_mut::<Health>() {
                                            h.max = (h.max - 8).max(1);
                                            h.current = h.current.min(h.max);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// 장비 손상 처리 (rust, corrosion 등)
    pub fn damage_equipment(
        rng: &mut NetHackRng,
        world: &mut SubWorld,
        p_ent: Entity,
        assets: &crate::assets::AssetManager,
        damage_type: DamageType,
        log: &mut crate::ui::log::GameLog,
        turn: u64,
    ) {
        let mut equipment_ent = None;
        if let Ok(entry) = world.entry_ref(p_ent) {
            if let Ok(eq) = entry.get_component::<crate::core::entity::Equipment>() {
                // 무작위 슬롯 선택
                let slots: Vec<_> = eq.slots.values().cloned().collect();
                if !slots.is_empty() {
                    equipment_ent = Some(slots[rng.rn2(slots.len() as i32) as usize]);
                }
            }
        }

        if let Some(item_ent) = equipment_ent {
            if let Ok(mut entry) = world.entry_mut(item_ent) {
                if let Ok(item) = entry.get_component_mut::<crate::core::entity::Item>() {
                    if let Some(tmpl) = assets.items.get_by_kind(item.kind) {
                        use crate::core::systems::item_damage::ItemDamageSystem;
                        match damage_type {
                            DamageType::Rust => {
                                ItemDamageSystem::rust_item(item, tmpl, log, turn, true);
                            }
                            DamageType::Corr | DamageType::Acid => {
                                ItemDamageSystem::corrode_item(item, tmpl, log, turn, true);
                            }
                            DamageType::Fire => {
                                ItemDamageSystem::burn_item(item, tmpl, log, turn, true);
                            }
                            DamageType::Cold => {
                                // Cold can rot organic matter or freeze liquids
                                ItemDamageSystem::rot_item(item, tmpl, log, turn, true);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    /// 몬스터 분열 처리 (Pudding, Ooze 등)
    pub fn try_split(rng: &mut NetHackRng, monster_tmpl: &MonsterTemplate) -> bool {
        //
        if monster_tmpl
            .has_capability(crate::core::entity::capability::MonsterCapability::Amorphous)
        {
            let name = monster_tmpl.name.to_lowercase();
            if name.contains("pudding") || name.contains("ooze") || name.contains("jelly") {
                // 1/2 확률로 분열
                return rng.rn2(2) == 0;
            }
        }
        false
    }

    /// 발사체 스킬 타입 반환
    pub fn get_launcher_skill(_player: &Player, launcher: Option<&ItemTemplate>) -> WeaponSkill {
        if let Some(tmpl) = launcher {
            // 런처의 경우 subtype이 스킬 ID와 일치함
            if let Ok(skill) = WeaponSkill::try_from(tmpl.subtype as u8) {
                return skill;
            }
        }
        WeaponSkill::None
    }

    /// 무기 스킬 타입 반환
    pub fn get_weapon_skill(_player: &Player, weapon: Option<&ItemTemplate>) -> WeaponSkill {
        if let Some(tmpl) = weapon {
            if let Ok(skill) = WeaponSkill::try_from(tmpl.subtype as u8) {
                return skill;
            }
        }
        WeaponSkill::BareHanded
    }

    /// 무기 스킬 명중 보너스 (weapon.c: weapon_hit_bonus 이식)
    pub fn weapon_hit_bonus(player: &Player, weapon: Option<&ItemTemplate>) -> i32 {
        let skill_type = Self::get_weapon_skill(player, weapon);

        let level = if let Some(record) = player.skills.get(&skill_type) {
            record.level
        } else {
            SkillLevel::Unskilled
        };

        let lvl_val = level as i32;

        if skill_type == WeaponSkill::BareHanded {
            // formula: ((bonus + 2) * mult) / 2
            // Unskilled(1) -> bonus=0.
            let bonus = lvl_val.max(1) - 1;
            let mult = if Self::martial_bonus(player) { 2 } else { 1 };
            return ((bonus + 2) * mult) / 2;
        }

        // Standard weapon
        match level {
            SkillLevel::Restricted | SkillLevel::Unskilled => -4,
            SkillLevel::Basic => 0,
            SkillLevel::Skilled => 2,
            SkillLevel::Expert | SkillLevel::Master | SkillLevel::GrandMaster => 3,
        }
    }

    /// 무기 스킬 데미지 보너스 (weapon.c: weapon_dam_bonus 이식)
    pub fn weapon_dam_bonus(player: &Player, weapon: Option<&ItemTemplate>) -> i32 {
        let skill_type = Self::get_weapon_skill(player, weapon);

        let level = if let Some(record) = player.skills.get(&skill_type) {
            record.level
        } else {
            SkillLevel::Unskilled
        };

        let lvl_val = level as i32;

        if skill_type == WeaponSkill::BareHanded {
            let bonus = lvl_val.max(1) - 1;
            let mult = if Self::martial_bonus(player) { 3 } else { 1 };
            return ((bonus + 1) * mult) / 2;
        }

        // NetHack 3.6.7 weapon.c: weapon_dam_bonus()
        match level {
            SkillLevel::Restricted | SkillLevel::Unskilled => -2,
            SkillLevel::Basic => 0,
            SkillLevel::Skilled => 1,
            SkillLevel::Expert => 2,
            SkillLevel::Master => 3,
            SkillLevel::GrandMaster => 4,
        }
    }

    /// 스킬 숙련도 연습 (weapon.c: use_skill 이식)
    pub fn practice_weapon_skill(
        player: &mut Player,
        skill_type: WeaponSkill,
        amount: u16,
        log: &mut crate::ui::log::GameLog,
        turn: u64,
    ) {
        if let Some(record) = player.skills.get_mut(&skill_type) {
            //
            if record.level >= record.max_level || record.level == SkillLevel::Restricted {
                return;
            }

            let was_ready = record.can_advance();
            record.advance = record.advance.saturating_add(amount);

            if !was_ready && record.can_advance() {
                // NetHack 3.6.7 weapon.c: give_may_advance_msg()
                let msg = match record.level {
                    SkillLevel::Unskilled => "You feel more confident in your weapon skills.",
                    SkillLevel::Basic => "You feel you could be more dangerous!",
                    SkillLevel::Skilled => {
                        "You feel you are becoming an expert in your weapon skills."
                    }
                    SkillLevel::Expert => {
                        "You feel you are becoming a master in your weapon skills."
                    }
                    SkillLevel::Master => {
                        "You feel you are becoming a grand master in your weapon skills."
                    }
                    SkillLevel::GrandMaster | SkillLevel::Restricted => "",
                };
                if !msg.is_empty() {
                    log.add(msg, turn);
                }
            }
        }
    }

    ///
    pub fn nearby_passive(
        rng: &mut NetHackRng,
        p_health: &mut Health,
        p_status: &StatusFlags,
        monster_tmpl: &MonsterTemplate,
        log: &mut crate::ui::log::GameLog,
        turn: u64,
    ) {
        //
        if monster_tmpl.name.contains("fire") && !p_status.contains(StatusFlags::FIRE_RES) {
            let dmg = rng.d(1, 4);
            p_health.current -= dmg;
            log.add_colored(
                "It's burning hot standing near the fire monster!",
                [255, 100, 0],
                turn,
            );
        } else if monster_tmpl.name.contains("cold") && !p_status.contains(StatusFlags::COLD_RES) {
            let dmg = rng.d(1, 4);
            p_health.current -= dmg;
            log.add_colored(
                "The air near the ice monster is freezing!",
                [100, 100, 255],
                turn,
            );
        }
    }

    /// 광역 폭발 처리 (AttackType::Explode, Boom 및 화염 마법 대응)
    pub fn execute_explosion(
        world: &mut SubWorld,
        pos: (i32, i32),
        dtype: DamageType,
        dice: (i32, i32),
        log: &mut crate::ui::log::GameLog,
        turn: u64,
        rng: &mut NetHackRng,
        assets: &crate::assets::AssetManager,
    ) {
        use crate::core::entity::status::StatusBundle;

        let (bx, by) = pos;

        // 3x3 영역 폭발
        let mut affected = Vec::new();
        {
            let mut query = <(Entity, &Position, Option<&StatusBundle>)>::query();
            for (ent, p, status) in query.iter(world) {
                let dx = (p.x - bx).abs();
                let dy = (p.y - by).abs();

                if dx <= 1 && dy <= 1 {
                    let is_player = world
                        .entry_ref(*ent)
                        .ok()
                        .map(|e| e.get_component::<crate::core::entity::PlayerTag>().is_ok())
                        .unwrap_or(false);

                    let mut dmg = rng.d(dice.0, dice.1);

                    // 저항 체크
                    let mut resists = false;
                    if let Some(s) = status {
                        let f = s.flags();
                        resists = match dtype {
                            DamageType::Fire => f.contains(StatusFlags::FIRE_RES),
                            DamageType::Cold => f.contains(StatusFlags::COLD_RES),
                            DamageType::Elec => f.contains(StatusFlags::SHOCK_RES),
                            DamageType::Acid => f.contains(StatusFlags::ACID_RES),
                            _ => false,
                        };
                    }

                    if resists {
                        dmg /= 2;
                    }

                    affected.push((*ent, dmg, is_player));
                }
            }
        }

        for (ent, dmg, is_player) in affected {
            if let Ok(mut entry) = world.entry_mut(ent) {
                if let Ok(h) = entry.get_component_mut::<Health>() {
                    if dmg > 0 {
                        h.current -= dmg;
                        if is_player {
                            log.add(
                                format!("You are hit by the explosion for {} damage!", dmg),
                                turn,
                            );
                        }
                    }
                }

                if dtype == DamageType::Blnd {
                    if let Ok(status) = entry.get_component_mut::<StatusBundle>() {
                        status.add(StatusFlags::BLIND, 20);
                        if is_player {
                            log.add_colored("You are blinded by the blast!", [255, 255, 100], turn);
                        }
                    }
                }

                if is_player
                    && matches!(
                        dtype,
                        DamageType::Fire | DamageType::Acid | DamageType::Rust | DamageType::Corr
                    )
                {
                    Self::damage_equipment(rng, world, ent, assets, dtype, log, turn);
                }
            }
        }

        let msg = match dtype {
            DamageType::Fire => "You hear an explosion! Booom!",
            DamageType::Cold => "You hear a crackling freeze!",
            _ => "You hear a loud blast!",
        };
        log.add(msg, turn);
    }
}

// ============================================================================
// AIHack - weapon_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// [v2.10.1] weapon.c 미이식 함수 대량 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 weapon.c
//   hitval(), dmgval(), special_dmgval(), abon(), dbon(),
//   weapon_hit_bonus(), weapon_dam_bonus(), slots_required(),
//   use_skill(), wet/dry_a_towel(), weapon_descr(),
//   monmightthrowwep(), select_hwep(), skill_level_name(),
//   skill_name(), unrestrict_weapon_skill()
// ============================================================================

use super::weapon::*;
use crate::util::rng::NetHackRng;

// =============================================================================
// 상수 및 테이블
// =============================================================================

/// 꼬치 가능한 몬스터 심볼 (원본 kebabable[])
/// [v2.10.1] weapon.c:65 — Xorn, Dragon, Jabberwock, Naga, Giant
const KEBABABLE: &[char] = &['X', 'D', 'J', 'N', 'H'];

/// 몬스터가 던질 줄 아는 무기 목록 (원본 rwep[])
/// [v2.10.1] weapon.c:488-494 이식
const THROWABLE_WEAPONS: &[&str] = &[
    "dwarvish spear",
    "silver spear",
    "elven spear",
    "spear",
    "orcish spear",
    "javelin",
    "shuriken",
    "ya",
    "silver arrow",
    "elven arrow",
    "arrow",
    "orcish arrow",
    "crossbow bolt",
    "silver dagger",
    "elven dagger",
    "dagger",
    "orcish dagger",
    "knife",
    "flint",
    "rock",
    "loadstone",
    "luckstone",
    "dart",
    "cream pie",
];

/// 몬스터 근접 무기 선호도 테이블 (원본 hwep[])
/// [v2.10.1] weapon.c:638-649 이식
const MELEE_WEAPON_PREFERENCE: &[&str] = &[
    "tsurugi",
    "runesword",
    "dwarvish mattock",
    "two-handed sword",
    "battle-axe",
    "katana",
    "unicorn horn",
    "crysknife",
    "trident",
    "long sword",
    "elven broadsword",
    "broadsword",
    "scimitar",
    "silver saber",
    "morning star",
    "short sword",
    "mace",
    "axe",
    "spear",
    "flail",
    "bullwhip",
    "quarterstaff",
    "javelin",
    "aklys",
    "club",
    "pick-axe",
    "war hammer",
    "dagger",
    "knife",
];

// =============================================================================
// hitval — 무기 vs 몬스터 명중 보너스
// [v2.10.1] weapon.c:134-180 이식
// =============================================================================

/// hitval 입력 구조체
#[derive(Debug, Clone)]
pub struct HitvalInput {
    pub weapon_name: String,
    pub enchantment: i32,
    pub is_weapon: bool,
    pub blessed: bool,
    pub is_spear: bool,
    pub is_pick: bool,
    pub artifact_hit_bonus: i32,
    pub target_symbol: char,
    pub target_is_undead: bool,
    pub target_is_demon: bool,
    pub target_is_vampshifter: bool,
    pub target_is_swimmer: bool,
    pub target_in_pool: bool,
    pub target_passes_walls_thick: bool,
}

/// 무기가 특정 몬스터에 대해 갖는 명중 보너스 계산
pub fn hitval_result(input: &HitvalInput) -> i32 {
    let mut tmp = 0i32;

    // 무기 클래스이면 강화치 적용
    if input.is_weapon {
        tmp += input.enchantment;
    }

    // 축복된 무기 vs 언데드/악마/뱀파이어시프터: +2
    if input.is_weapon
        && input.blessed
        && (input.target_is_demon || input.target_is_undead || input.target_is_vampshifter)
    {
        tmp += 2;
    }

    // 창 vs 꼬치 가능 몬스터: +2
    if input.is_spear && KEBABABLE.contains(&input.target_symbol) {
        tmp += 2;
    }

    // 삼지창 vs 수영 몬스터
    let l = input.weapon_name.to_lowercase();
    if l.contains("trident") && input.target_is_swimmer {
        if input.target_in_pool {
            tmp += 4;
        } else if input.target_symbol == ';' || input.target_symbol == 'S' {
            tmp += 2;
        }
    }

    // 곡괭이 vs 벽 통과+두꺼운 피부: +2
    if input.is_pick && input.target_passes_walls_thick {
        tmp += 2;
    }

    // 아티팩트 명중 보너스
    tmp += input.artifact_hit_bonus;

    tmp
}

// =============================================================================
// dmgval — 무기 vs 몬스터 데미지 보너스
// [v2.10.1] weapon.c:204-351 이식
// =============================================================================

/// dmgval 입력 구조체
#[derive(Debug, Clone)]
pub struct DmgvalInput {
    pub weapon_name: String,
    pub enchantment: i32,
    pub is_weapon: bool,
    pub blessed: bool,
    pub is_axe: bool,
    pub material_silver: bool,
    pub artifact_light_lit: bool,
    pub target_large: bool,
    pub target_undead: bool,
    pub target_demon: bool,
    pub target_vampshifter: bool,
    pub target_wooden: bool,
    pub target_hates_silver: bool,
    pub target_hates_light: bool,
    pub target_thick_skinned: bool,
    pub target_is_shade: bool,
    pub greatest_erosion: i32,
    pub artifact_double_damage: bool,
}

/// 무기가 특정 몬스터에 대해 갖는 데미지 계산
pub fn dmgval_result(input: &DmgvalInput, rng: &mut NetHackRng) -> i32 {
    let l = input.weapon_name.to_lowercase();

    // 크림파이는 0 데미지
    if l.contains("cream pie") {
        return 0;
    }

    // 기본 주사위 데미지 (대형/소형 분기)
    let (base_small, base_large) = weapon_base_damage(&input.weapon_name);
    let mut tmp = if input.target_large {
        if base_large > 0 {
            rng.rnd(base_large)
        } else {
            0
        }
    } else {
        if base_small > 0 {
            rng.rnd(base_small)
        } else {
            0
        }
    };

    // 무기 클래스면 강화치 적용, 음수 데미지 방지
    if input.is_weapon {
        tmp += input.enchantment;
        if tmp < 0 {
            tmp = 0;
        }
    }

    // 두꺼운 피부에 가죽/나무/뼈 재질 무효
    if input.target_thick_skinned {
        let mat = infer_material(&input.weapon_name);
        if matches!(
            mat,
            WeaponMaterial::Leather | WeaponMaterial::Wood | WeaponMaterial::Bone
        ) {
            tmp = 0;
        }
    }

    // Shade에게는 빛나는 무기만 유효
    if input.target_is_shade && !input.artifact_light_lit {
        tmp = 0;
    }

    // 추가 보너스 (축복/은/빛/도끼)
    let mut bonus = 0i32;
    if input.blessed && (input.target_undead || input.target_demon || input.target_vampshifter) {
        bonus += rng.rnd(4);
    }
    if input.is_axe && input.target_wooden {
        bonus += rng.rnd(4);
    }
    if input.material_silver && input.target_hates_silver {
        bonus += rng.rnd(20);
    }
    if input.artifact_light_lit && input.target_hates_light {
        bonus += rng.rnd(8);
    }

    // 아티팩트 더블 데미지 시 보너스 반감
    if bonus > 1 && input.artifact_double_damage {
        bonus = (bonus + 1) / 2;
    }
    tmp += bonus;

    // 침식도 감산 (최소 1)
    if tmp > 0 {
        tmp -= input.greatest_erosion;
        if tmp < 1 {
            tmp = 1;
        }
    }

    tmp
}

// =============================================================================
// special_dmgval — 비무기 타격(갑옷/반지) 은/축복 추가 데미지
// [v2.10.1] weapon.c:353-426 이식
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecialDmgvalResult {
    pub bonus: i32,
    pub silver_hit_mask: u32,
}

pub fn special_dmgval_result(
    armor_blessed: bool,
    armor_material_silver: bool,
    target_undead: bool,
    target_demon: bool,
    target_vampshifter: bool,
    target_hates_silver: bool,
    armor_slot_mask: u32,
    rng: &mut NetHackRng,
) -> SpecialDmgvalResult {
    let mut bonus = 0i32;
    let mut silver_hit_mask = 0u32;

    if armor_blessed && (target_undead || target_demon || target_vampshifter) {
        bonus += rng.rnd(4);
    }
    if armor_material_silver && target_hates_silver {
        bonus += rng.rnd(20);
        silver_hit_mask |= armor_slot_mask;
    }

    SpecialDmgvalResult {
        bonus,
        silver_hit_mask,
    }
}

// =============================================================================
// abon / dbon — 힘+민첩 공격/데미지 보너스
// [v2.10.1] weapon.c:889-952 이식
// =============================================================================

/// 힘+민첩 공격 보너스 (원본 abon())
pub fn abon_result(
    strength: i32,
    dexterity: i32,
    player_level: i32,
    is_polymorphed: bool,
    poly_adj_level: i32,
) -> i32 {
    if is_polymorphed {
        return poly_adj_level - 3;
    }

    let sbon = if strength < 6 {
        -2
    } else if strength < 8 {
        -1
    } else if strength < 17 {
        0
    } else if strength <= 18 {
        1
    } else if strength < 100 {
        2
    } else {
        3
    };

    // 저레벨 보정
    let level_adj = if player_level < 3 { 1 } else { 0 };
    let str_total = sbon + level_adj;

    if dexterity < 4 {
        str_total - 3
    } else if dexterity < 6 {
        str_total - 2
    } else if dexterity < 8 {
        str_total - 1
    } else if dexterity < 14 {
        str_total
    } else {
        str_total + dexterity - 14
    }
}

/// 힘 데미지 보너스 (원본 dbon())
pub fn dbon_result(strength: i32, is_polymorphed: bool) -> i32 {
    if is_polymorphed {
        return 0;
    }

    if strength < 6 {
        -1
    } else if strength < 16 {
        0
    } else if strength < 18 {
        1
    } else if strength == 18 {
        2
    } else if strength <= 93 {
        3
    } else if strength <= 108 {
        4
    } else if strength < 118 {
        5
    } else {
        6
    }
}

// =============================================================================
// weapon_hit_bonus / weapon_dam_bonus — 스킬 기반 명중/데미지 보너스
// [v2.10.1] weapon.c:1394-1579 이식
// =============================================================================

/// 스킬 기반 명중 보너스 (이중무기/맨손/기승 포함)
pub fn weapon_hit_bonus_result(
    skill_level: SkillLevel,
    is_two_weapon: bool,
    two_weapon_skill: SkillLevel,
    wep_type_skill: SkillLevel,
    is_bare_handed: bool,
    is_martial: bool,
    is_riding: bool,
    riding_skill: SkillLevel,
) -> i32 {
    let mut bonus;

    if is_bare_handed {
        let sv = skill_level as i32;
        let base = (sv.max(0)).saturating_sub(1).max(0);
        let mult = if is_martial { 2 } else { 1 };
        bonus = ((base + 2) * mult) / 2;
    } else if is_two_weapon {
        let eff = two_weapon_skill.min(wep_type_skill);
        bonus = match eff {
            SkillLevel::Unskilled => -9,
            SkillLevel::Basic => -7,
            SkillLevel::Skilled => -5,
            SkillLevel::Expert => -3,
            _ => -9,
        };
    } else {
        bonus = match skill_level {
            SkillLevel::Unskilled => -4,
            SkillLevel::Basic => 0,
            SkillLevel::Skilled => 2,
            SkillLevel::Expert => 3,
            _ => -4,
        };
    }

    // 기승 페널티
    if is_riding {
        match riding_skill {
            SkillLevel::Unskilled => bonus -= 2,
            SkillLevel::Basic => bonus -= 1,
            _ => {}
        }
        if is_two_weapon {
            bonus -= 2;
        }
    }

    bonus
}

/// 스킬 기반 데미지 보너스
pub fn weapon_dam_bonus_result(
    skill_level: SkillLevel,
    is_two_weapon: bool,
    two_weapon_skill: SkillLevel,
    wep_type_skill: SkillLevel,
    is_bare_handed: bool,
    is_martial: bool,
    is_riding: bool,
    riding_skill: SkillLevel,
) -> i32 {
    let mut bonus;

    if is_bare_handed {
        let sv = skill_level as i32;
        let base = (sv.max(0)).saturating_sub(1).max(0);
        let mult = if is_martial { 3 } else { 1 };
        bonus = ((base + 1) * mult) / 2;
    } else if is_two_weapon {
        let eff = two_weapon_skill.min(wep_type_skill);
        bonus = match eff {
            SkillLevel::Unskilled => -3,
            SkillLevel::Basic => -1,
            SkillLevel::Skilled => 0,
            SkillLevel::Expert => 1,
            _ => -3,
        };
    } else {
        bonus = match skill_level {
            SkillLevel::Unskilled => -2,
            SkillLevel::Basic => 0,
            SkillLevel::Skilled => 1,
            SkillLevel::Expert => 2,
            _ => -2,
        };
    }

    // 기승 돌진 보너스 (이중무기 제외)
    if is_riding && !is_two_weapon {
        match riding_skill {
            SkillLevel::Skilled => bonus += 1,
            SkillLevel::Expert | SkillLevel::Master | SkillLevel::GrandMaster => bonus += 2,
            _ => {}
        }
    }

    bonus
}

// =============================================================================
// 스킬 유틸리티 함수
// [v2.10.1] weapon.c:1058-1368 이식
// =============================================================================

/// 스킬 승급에 필요한 슬롯 수
pub fn slots_required(skill: WeaponSkill, current_level: SkillLevel) -> i32 {
    let lv = current_level as i32;
    match skill {
        WeaponSkill::BareHanded | WeaponSkill::MartialArts => (lv + 1) / 2,
        _ => lv,
    }
}

/// 레벨별 필요 경험치 (원본 practice_needed_to_advance 근사)
pub fn practice_needed(level: SkillLevel) -> i32 {
    match level {
        SkillLevel::Unskilled => 100,
        SkillLevel::Basic => 200,
        SkillLevel::Skilled => 400,
        SkillLevel::Expert => 800,
        SkillLevel::Master => 1600,
        SkillLevel::GrandMaster => i32::MAX,
    }
}

/// use_skill 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseSkillResult {
    pub newly_advanceable: bool,
    pub skill_category: &'static str,
}

pub fn use_skill_result(
    skill: WeaponSkill,
    degree: i32,
    current_advance: i32,
    current_level: SkillLevel,
    max_level: SkillLevel,
) -> UseSkillResult {
    let th = practice_needed(current_level);
    let was = current_level < max_level && current_advance >= th;
    let now = current_level < max_level && (current_advance + degree) >= th;
    let cat = match skill {
        WeaponSkill::AttackSpell
        | WeaponSkill::HealingSpell
        | WeaponSkill::DivinationSpell
        | WeaponSkill::EnchantmentSpell
        | WeaponSkill::ClericalSpell
        | WeaponSkill::EscapeSpell
        | WeaponSkill::MatterSpell => "spell casting",
        WeaponSkill::BareHanded
        | WeaponSkill::MartialArts
        | WeaponSkill::Riding
        | WeaponSkill::TwoWeaponCombat => "fighting",
        _ => "weapon",
    };
    UseSkillResult {
        newly_advanceable: !was && now,
        skill_category: cat,
    }
}

/// 스킬 레벨 이름 (원본 skill_level_name)
pub fn skill_level_name(level: SkillLevel) -> &'static str {
    match level {
        SkillLevel::Unskilled => "Unskilled",
        SkillLevel::Basic => "Basic",
        SkillLevel::Skilled => "Skilled",
        SkillLevel::Expert => "Expert",
        SkillLevel::Master => "Master",
        SkillLevel::GrandMaster => "Grand Master",
    }
}

/// 스킬 이름 (원본 P_NAME 매크로)
pub fn skill_name(skill: WeaponSkill) -> &'static str {
    match skill {
        WeaponSkill::Dagger => "dagger",
        WeaponSkill::Knife => "knife",
        WeaponSkill::Axe => "axe",
        WeaponSkill::ShortSword => "short sword",
        WeaponSkill::BroadSword => "broadsword",
        WeaponSkill::LongSword => "long sword",
        WeaponSkill::TwoHandedSword => "two-handed sword",
        WeaponSkill::Scimitar => "scimitar",
        WeaponSkill::Saber => "saber",
        WeaponSkill::Club => "club",
        WeaponSkill::Mace => "mace",
        WeaponSkill::MorningStar => "morning star",
        WeaponSkill::Flail => "flail",
        WeaponSkill::Hammer => "hammer",
        WeaponSkill::Quarterstaff => "quarterstaff",
        WeaponSkill::Polearm => "polearm",
        WeaponSkill::Spear => "spear",
        WeaponSkill::Javelin => "javelin",
        WeaponSkill::Trident => "trident",
        WeaponSkill::Lance => "lance",
        WeaponSkill::Bow => "bow",
        WeaponSkill::Sling => "sling",
        WeaponSkill::Crossbow => "crossbow",
        WeaponSkill::Dart => "dart",
        WeaponSkill::Shuriken => "shuriken",
        WeaponSkill::Boomerang => "boomerang",
        WeaponSkill::Whip => "whip",
        WeaponSkill::PickAxe => "pick-axe",
        WeaponSkill::Unicorn => "unicorn horn",
        WeaponSkill::BareHanded => "bare handed combat",
        WeaponSkill::MartialArts => "martial arts",
        WeaponSkill::AttackSpell => "attack spells",
        WeaponSkill::HealingSpell => "healing spells",
        WeaponSkill::DivinationSpell => "divination spells",
        WeaponSkill::EnchantmentSpell => "enchantment spells",
        WeaponSkill::ClericalSpell => "clerical spells",
        WeaponSkill::EscapeSpell => "escape spells",
        WeaponSkill::MatterSpell => "matter spells",
        WeaponSkill::Riding => "riding",
        WeaponSkill::TwoWeaponCombat => "two weapon combat",
    }
}

/// 제한→미숙련 해제 결과 (원본 unrestrict_weapon_skill)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrestrictResult {
    pub new_level: SkillLevel,
    pub new_max: SkillLevel,
}

pub fn unrestrict_weapon_skill_result(current_max: SkillLevel) -> Option<UnrestrictResult> {
    if current_max > SkillLevel::Unskilled {
        return None;
    }
    Some(UnrestrictResult {
        new_level: SkillLevel::Unskilled,
        new_max: SkillLevel::Basic,
    })
}

// =============================================================================
// 수건 젖음/마름
// [v2.10.1] weapon.c:954-1013 이식
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TowelWetResult {
    pub new_spe: i32,
    pub message: Option<&'static str>,
}

/// 수건 젖음 증가 (원본 wet_a_towel)
pub fn wet_a_towel_result(current_spe: i32, amt: i32) -> TowelWetResult {
    let newspe = if amt <= 0 { current_spe - amt } else { amt };
    let clamped = newspe.min(7);
    let message = if newspe > current_spe {
        if newspe < 3 {
            if current_spe == 0 {
                Some("damp")
            } else {
                Some("damper")
            }
        } else {
            if current_spe == 0 {
                Some("wet")
            } else {
                Some("wetter")
            }
        }
    } else {
        None
    };
    TowelWetResult {
        new_spe: clamped,
        message,
    }
}

/// 수건 젖음 감소 (원본 dry_a_towel)
pub fn dry_a_towel_result(current_spe: i32, amt: i32) -> TowelWetResult {
    let newspe = if amt <= 0 { current_spe + amt } else { amt };
    let clamped = newspe.min(7).max(0);
    let message = if newspe < current_spe {
        if clamped == 0 {
            Some("dries out")
        } else {
            Some("dries")
        }
    } else {
        None
    };
    TowelWetResult {
        new_spe: clamped,
        message,
    }
}

// =============================================================================
// weapon_descr — 무기 설명 카테고리명
// [v2.10.1] weapon.c:82-132 이식
// =============================================================================

pub fn weapon_descr(item_name: &str) -> &'static str {
    let skill = weapon_type(item_name);
    match skill {
        Some(WeaponSkill::Sling) => {
            if is_ammo(item_name) {
                "stone"
            } else {
                "sling"
            }
        }
        Some(WeaponSkill::Bow) => {
            if is_ammo(item_name) {
                "arrow"
            } else {
                "bow"
            }
        }
        Some(WeaponSkill::Crossbow) => {
            if is_ammo(item_name) {
                "bolt"
            } else {
                "crossbow"
            }
        }
        Some(WeaponSkill::Flail) => {
            if item_name.to_lowercase().contains("grappling") {
                "hook"
            } else {
                "flail"
            }
        }
        Some(WeaponSkill::PickAxe) => {
            if item_name.to_lowercase().contains("mattock") {
                "mattock"
            } else {
                "pick-axe"
            }
        }
        Some(WeaponSkill::Dagger) => "dagger",
        Some(WeaponSkill::Knife) => "knife",
        Some(WeaponSkill::Axe) => "axe",
        Some(WeaponSkill::LongSword) => "long sword",
        Some(WeaponSkill::ShortSword) => "short sword",
        Some(WeaponSkill::BroadSword) => "broadsword",
        Some(WeaponSkill::TwoHandedSword) => "two-handed sword",
        Some(WeaponSkill::Scimitar) => "scimitar",
        Some(WeaponSkill::Saber) => "saber",
        Some(WeaponSkill::Club) => "club",
        Some(WeaponSkill::Mace) => "mace",
        Some(WeaponSkill::MorningStar) => "morning star",
        Some(WeaponSkill::Hammer) => "hammer",
        Some(WeaponSkill::Quarterstaff) => "quarterstaff",
        Some(WeaponSkill::Polearm) => "polearm",
        Some(WeaponSkill::Spear) => "spear",
        Some(WeaponSkill::Javelin) => "javelin",
        Some(WeaponSkill::Trident) => "trident",
        Some(WeaponSkill::Lance) => "lance",
        Some(WeaponSkill::Dart) => "dart",
        Some(WeaponSkill::Shuriken) => "shuriken",
        Some(WeaponSkill::Boomerang) => "boomerang",
        Some(WeaponSkill::Whip) => "whip",
        Some(WeaponSkill::Unicorn) => "unicorn horn",
        Some(WeaponSkill::BareHanded) => "bare hands",
        Some(WeaponSkill::MartialArts) => "martial arts",
        _ => "weapon",
    }
}

// =============================================================================
// 몬스터 무기 선택 AI
// [v2.10.1] weapon.c:504-688 이식
// =============================================================================

/// 몬스터가 던질 줄 아는 무기인지 판별 (원본 monmightthrowwep)
pub fn mon_might_throw_wep(item_name: &str) -> bool {
    let l = item_name.to_lowercase();
    THROWABLE_WEAPONS.iter().any(|w| l.contains(w))
}

/// 몬스터 근접 무기 선택 AI (순수 결과 버전, 원본 select_hwep)
/// 보유 아이템 목록에서 선호도 순으로 최적 무기 인덱스 반환
pub fn select_hwep_result(
    inventory_names: &[String],
    is_strong: bool,
    wearing_shield: bool,
    hates_silver: bool,
) -> Option<usize> {
    for pref in MELEE_WEAPON_PREFERENCE {
        for (idx, item) in inventory_names.iter().enumerate() {
            let l = item.to_lowercase();
            if l.contains(pref) {
                // 양손 무기인데 약하거나 방패 착용 중이면 스킵
                if is_two_handed(&l) && (!is_strong || wearing_shield) {
                    continue;
                }
                // 은 무기인데 은 혐오 몬스터면 스킵
                if l.contains("silver") && hates_silver {
                    continue;
                }
                return Some(idx);
            }
        }
    }
    None
}

// =============================================================================
// select_rwep — 몬스터 원거리 무기 선택 AI
// [v2.11.0] weapon.c:504-623 이식
// =============================================================================

/// 몬스터 인벤토리 아이템 정보 (원거리 무기 선택용)
#[derive(Debug, Clone)]
pub struct MonInventoryItem {
    /// 아이템 이름 (소문자)
    pub name: String,
    /// 아티팩트인지 여부
    pub is_artifact: bool,
    /// 저주 여부
    pub cursed: bool,
    /// 현재 장착된 무기(mw_tmp)인지 여부
    pub is_wielded: bool,
    /// 용접(welded) 상태인지 여부
    pub is_welded: bool,
}

/// 원거리 무기 선택 결과
#[derive(Debug, Clone)]
pub struct SelectRwepResult {
    /// 선택된 투척/발사 무기 인벤토리 인덱스
    pub weapon_index: usize,
    /// 필요한 발사대 인덱스 (활/투석기 등, 없으면 None)
    pub propellor_index: Option<usize>,
}

/// 장대무기(polearm) 선호 목록 (원본 pwep[])
const POLEARM_PREFERENCE: &[&str] = &[
    "halberd",
    "bardiche",
    "spetum",
    "bill-guisarme",
    "voulge",
    "ranseur",
    "guisarme",
    "glaive",
    "lucern hammer",
    "bec de corbin",
    "fauchard",
    "partisan",
    "lance",
];

/// 몬스터 원거리 무기 선택 AI (순수 결과 패턴, 원본 select_rwep)
/// [v2.11.0] weapon.c:504-623 이식
///
/// 선택 우선순위:
/// 1. 코카트리스 알
/// 2. Kop이면 크림 파이
/// 3. 바위 던지기 가능하면 바위
/// 4. 사거리 내 장대무기 (강한 몬스터만)
/// 5. 투척 무기 (rwep[] 순서대로 + 발사대 매칭)
pub fn select_rwep_result(
    inventory: &[MonInventoryItem],
    is_kop: bool,
    throws_rocks: bool,
    is_strong: bool,
    wearing_shield: bool,
    hates_silver: bool,
    distance_sq: i32,
    can_see_target: bool,
    wielded_is_welded: bool,
) -> Option<SelectRwepResult> {
    // 코카트리스 알 → 최우선 (원본 Oselect(EGG) with touch_petrifies)
    for (idx, item) in inventory.iter().enumerate() {
        let l = &item.name;
        if l.contains("cockatrice egg") || l.contains("chickatrice egg") {
            return Some(SelectRwepResult {
                weapon_index: idx,
                propellor_index: None,
            });
        }
    }

    // Kop이면 크림 파이 우선
    if is_kop {
        for (idx, item) in inventory.iter().enumerate() {
            if item.name.contains("cream pie") {
                return Some(SelectRwepResult {
                    weapon_index: idx,
                    propellor_index: None,
                });
            }
        }
    }

    // 바위 던지기 가능하면 바위 우선
    if throws_rocks {
        for (idx, item) in inventory.iter().enumerate() {
            if item.name.contains("boulder") {
                return Some(SelectRwepResult {
                    weapon_index: idx,
                    propellor_index: None,
                });
            }
        }
    }

    // 장대무기 — 사거리 13 이내이고 시야 확보 시
    if distance_sq <= 13 && can_see_target {
        for pref in POLEARM_PREFERENCE {
            for (idx, item) in inventory.iter().enumerate() {
                if !item.name.contains(pref) {
                    continue;
                }
                // 양손 무기: 강해야 하고 방패 없어야 함
                if is_two_handed(&item.name) && (!is_strong || wearing_shield) {
                    continue;
                }
                // 은 무기: 은 혐오 몬스터 스킵
                if item.name.contains("silver") && hates_silver {
                    continue;
                }
                // 장착 무기가 용접되어 다른 무기를 못 드는 상태면 장착 무기만 가능
                if wielded_is_welded && !item.is_wielded {
                    continue;
                }
                return Some(SelectRwepResult {
                    weapon_index: idx,
                    propellor_index: Some(idx), // 장대무기는 직접 장착
                });
            }
        }
    }

    // 투척 무기 (rwep[] 순서대로)
    for throw_name in THROWABLE_WEAPONS {
        // 발사대 필요 무기 처리
        let propellor_idx = find_propellor(inventory, throw_name);

        // 발사대가 필요한데 없으면 스킵
        if needs_propellor(throw_name) && propellor_idx.is_none() {
            continue;
        }

        // 용접된 무기와 발사대가 다르면 스킵
        if wielded_is_welded {
            if let Some(prop_idx) = propellor_idx {
                if !inventory[prop_idx].is_wielded {
                    continue;
                }
            }
        }

        for (idx, item) in inventory.iter().enumerate() {
            if !item.name.contains(throw_name) {
                continue;
            }
            // 아티팩트는 던지지 않음
            if item.is_artifact {
                continue;
            }
            // 장착 무기가 용접되어 있으면 제외
            if item.is_wielded && item.is_welded {
                continue;
            }
            // 저주받은 load stone은 자동 발사 금지 (원본에서 !otmp->cursed)
            if item.name.contains("loadstone") && item.cursed {
                continue;
            }
            return Some(SelectRwepResult {
                weapon_index: idx,
                propellor_index: propellor_idx,
            });
        }
    }

    None
}

/// 발사대가 필요한 탄환인지 판별
fn needs_propellor(weapon_name: &str) -> bool {
    weapon_name.contains("arrow")
        || weapon_name == "ya"
        || weapon_name.contains("bolt")
        || weapon_name.contains("rock")
        || weapon_name.contains("loadstone")
        || weapon_name.contains("luckstone")
        || weapon_name.contains("flint")
}

/// 인벤토리에서 탄환에 맞는 발사대 찾기
fn find_propellor(inventory: &[MonInventoryItem], ammo_name: &str) -> Option<usize> {
    if ammo_name.contains("arrow") || ammo_name == "ya" {
        // 활 계열
        for name in &["yumi", "elven bow", "bow", "orcish bow"] {
            if let Some(idx) = inventory.iter().position(|i| i.name.contains(name)) {
                return Some(idx);
            }
        }
    } else if ammo_name.contains("bolt") {
        // 석궁
        if let Some(idx) = inventory.iter().position(|i| i.name.contains("crossbow")) {
            return Some(idx);
        }
    } else if ammo_name.contains("rock")
        || ammo_name.contains("loadstone")
        || ammo_name.contains("luckstone")
        || ammo_name.contains("flint")
    {
        // 투석기
        if let Some(idx) = inventory.iter().position(|i| i.name.contains("sling")) {
            return Some(idx);
        }
    }
    // 발사대 불필요 무기 (창, 단검, 수리검 등)는 None 반환이 정상
    if !needs_propellor(ammo_name) {
        return None; // 발사대 필요 없음 = 그냥 던짐
    }
    None
}

// =============================================================================
// 스킬 승급 판정 함수군
// [v2.11.0] weapon.c:1084-1368 이식
// =============================================================================

/// 스킬 승급 가능 판단 결과 (원본 can_advance)
/// [v2.11.0] weapon.c:1084-1100
pub fn can_advance_result(
    current_level: SkillLevel,
    max_level: SkillLevel,
    current_advance: i32,
    skills_advanced: i32,
    skill_limit: i32,
    available_slots: i32,
    skill: WeaponSkill,
) -> bool {
    // 제한 상태거나 이미 최대면 불가
    if current_level >= max_level {
        return false;
    }
    // 총 승급 회수 제한 도달
    if skills_advanced >= skill_limit {
        return false;
    }
    // 경험치가 충분하고 슬롯도 충분한지
    current_advance >= practice_needed(current_level)
        && available_slots >= slots_required(skill, current_level)
}

/// 슬롯만 있으면 승급 가능한지 (원본 could_advance)
/// [v2.11.0] weapon.c:1103-1114
pub fn could_advance_result(
    current_level: SkillLevel,
    max_level: SkillLevel,
    current_advance: i32,
    skills_advanced: i32,
    skill_limit: i32,
) -> bool {
    if current_level >= max_level {
        return false;
    }
    if skills_advanced >= skill_limit {
        return false;
    }
    current_advance >= practice_needed(current_level)
}

/// 스킬이 최대에 도달했고 경험치도 넘치는지 (원본 peaked_skill)
/// [v2.11.0] weapon.c:1118-1128
pub fn peaked_skill_result(
    current_level: SkillLevel,
    max_level: SkillLevel,
    current_advance: i32,
) -> bool {
    current_level >= max_level && current_advance >= practice_needed(current_level)
}

/// 스킬 승급 실행 결과 (원본 skill_advance)
/// [v2.11.0] weapon.c:1130-1141
#[derive(Debug, Clone)]
pub struct SkillAdvanceResult {
    /// 새 스킬 레벨
    pub new_level: SkillLevel,
    /// 소비된 슬롯 수
    pub slots_consumed: i32,
    /// "most skilled" vs "more skilled" 메시지 분기
    pub is_most_skilled: bool,
    /// 스킬 이름
    pub skill_display_name: &'static str,
}

pub fn skill_advance_result(
    skill: WeaponSkill,
    current_level: SkillLevel,
    max_level: SkillLevel,
) -> SkillAdvanceResult {
    let slots_consumed = slots_required(skill, current_level);
    let new_level = match current_level {
        SkillLevel::Unskilled => SkillLevel::Basic,
        SkillLevel::Basic => SkillLevel::Skilled,
        SkillLevel::Skilled => SkillLevel::Expert,
        SkillLevel::Expert => SkillLevel::Master,
        SkillLevel::Master => SkillLevel::GrandMaster,
        SkillLevel::GrandMaster => SkillLevel::GrandMaster,
    };
    let is_most_skilled = new_level >= max_level;
    SkillAdvanceResult {
        new_level,
        slots_consumed,
        is_most_skilled,
        skill_display_name: skill_name(skill),
    }
}

/// add_weapon_skill 결과 — 슬롯 추가 후 새로 승급 가능한 스킬이 있는지
/// [v2.11.0] weapon.c:1329-1344
pub fn add_weapon_skill_gained_advanceable(
    skills: &[(WeaponSkill, SkillLevel, SkillLevel, i32)], // (스킬, 현재, 최대, 경험치)
    old_slots: i32,
    new_slots: i32,
    skills_advanced: i32,
    skill_limit: i32,
) -> bool {
    // 추가 전 승급 가능 수
    let before = skills
        .iter()
        .filter(|(sk, lv, mx, adv)| {
            can_advance_result(*lv, *mx, *adv, skills_advanced, skill_limit, old_slots, *sk)
        })
        .count();
    // 추가 후 승급 가능 수
    let after = skills
        .iter()
        .filter(|(sk, lv, mx, adv)| {
            can_advance_result(*lv, *mx, *adv, skills_advanced, skill_limit, new_slots, *sk)
        })
        .count();
    before < after
}

/// lose_weapon_skill 결과 — 레벨 다운 시 슬롯 감소 처리
/// [v2.11.0] weapon.c:1346-1368
#[derive(Debug, Clone)]
pub struct LoseSkillResult {
    /// 처리 후 남은 슬롯 수
    pub remaining_slots: i32,
    /// 처리 후 총 승급 회수
    pub skills_advanced: i32,
    /// 강등된 스킬 목록 (스킬, 새 레벨)
    pub demoted: Vec<(WeaponSkill, SkillLevel)>,
}

pub fn lose_weapon_skill_result(
    mut n: i32,
    mut slots: i32,
    mut skills_advanced: i32,
    skill_record: &[(WeaponSkill, SkillLevel)], // 승급 기록 (역순으로 팝)
) -> LoseSkillResult {
    let mut demoted = Vec::new();
    let mut record_idx = skill_record.len();

    while n > 0 {
        n -= 1;
        if slots > 0 {
            // 미사용 슬롯에서 차감
            slots -= 1;
        } else if record_idx > 0 {
            // 마지막 승급된 스킬을 강등
            record_idx -= 1;
            let (skill, level) = skill_record[record_idx];
            skills_advanced -= 1;
            let new_level = match level {
                SkillLevel::GrandMaster => SkillLevel::Master,
                SkillLevel::Master => SkillLevel::Expert,
                SkillLevel::Expert => SkillLevel::Skilled,
                SkillLevel::Skilled => SkillLevel::Basic,
                SkillLevel::Basic => SkillLevel::Unskilled,
                SkillLevel::Unskilled => SkillLevel::Unskilled,
            };
            // 강등된 스킬의 원래 승급에 사용된 슬롯 중 남은 분 환급
            let refund = slots_required(skill, new_level) - 1;
            if refund > 0 {
                slots = refund;
            }
            demoted.push((skill, new_level));
        }
    }

    LoseSkillResult {
        remaining_slots: slots,
        skills_advanced,
        demoted,
    }
}

// =============================================================================
// skill_init — 게임 시작 시 스킬 초기화
// [v2.11.0] weapon.c:1587-1659 이식
// =============================================================================

/// 초기 스킬 상태
#[derive(Debug, Clone)]
pub struct SkillInitEntry {
    pub skill: WeaponSkill,
    pub level: SkillLevel,
    pub max_level: SkillLevel,
    pub advance: i32,
}

/// 역할, 인벤토리, 역할별 스킬 캡을 기반으로 초기 스킬 테이블 생성 (원본 skill_init)
/// [v2.11.0] weapon.c:1587-1659
pub fn skill_init_result(
    role: &str,
    inventory_weapons: &[String], // 초기 인벤토리 무기 이름
    pet_is_pony: bool,
) -> Vec<SkillInitEntry> {
    let all_skills = all_weapon_skills();
    let role_caps = crate::core::systems::combat::weapon::role_skill_caps(role);

    let mut entries: Vec<SkillInitEntry> = all_skills
        .iter()
        .map(|sk| {
            SkillInitEntry {
                skill: *sk,
                level: SkillLevel::Unskilled, // 기본: 제한됨 (구현상 Unskilled로 표현)
                max_level: SkillLevel::Unskilled, // 제한 상태
                advance: 0,
            }
        })
        .collect();

    // 인벤토리 무기의 스킬을 Basic으로 (탄환 제외)
    for wep_name in inventory_weapons {
        if is_ammo(wep_name) {
            continue;
        }
        if let Some(sk) = weapon_type(wep_name) {
            if let Some(entry) = entries.iter_mut().find(|e| e.skill == sk) {
                if entry.level < SkillLevel::Basic {
                    entry.level = SkillLevel::Basic;
                }
            }
        }
    }

    // 역할별 마법 스킬 초기 설정
    let lr = role.to_lowercase();
    match lr.as_str() {
        "healer" | "monk" => {
            set_skill_level(&mut entries, WeaponSkill::HealingSpell, SkillLevel::Basic);
        }
        "priest" | "priestess" => {
            set_skill_level(&mut entries, WeaponSkill::ClericalSpell, SkillLevel::Basic);
        }
        "wizard" => {
            set_skill_level(&mut entries, WeaponSkill::AttackSpell, SkillLevel::Basic);
            set_skill_level(
                &mut entries,
                WeaponSkill::EnchantmentSpell,
                SkillLevel::Basic,
            );
        }
        _ => {}
    }

    // 역할 스킬 캡 적용
    for (sk, max_lv) in &role_caps {
        if let Some(entry) = entries.iter_mut().find(|e| e.skill == *sk) {
            entry.max_level = *max_lv;
            // 제한(Unskilled인데 max도 Unskilled)이 아니게 됨 → 최소 Unskilled
            // (max_level > Unskilled이면 사용 가능 상태)
        }
    }

    // 격투 잠재력이 Expert 초과이면 맨손 Basic으로 시작
    if let Some(entry) = entries
        .iter_mut()
        .find(|e| e.skill == WeaponSkill::BareHanded)
    {
        if entry.max_level > SkillLevel::Expert && entry.level < SkillLevel::Basic {
            entry.level = SkillLevel::Basic;
        }
    }

    // 말 시작 역할은 기승 Basic
    if pet_is_pony {
        set_skill_level(&mut entries, WeaponSkill::Riding, SkillLevel::Basic);
    }

    // advance를 현재 레벨의 이전 레벨 경험치로 초기화
    for entry in entries.iter_mut() {
        if entry.max_level > SkillLevel::Unskilled {
            let prev_level = match entry.level {
                SkillLevel::Unskilled => SkillLevel::Unskilled,
                SkillLevel::Basic => SkillLevel::Unskilled,
                SkillLevel::Skilled => SkillLevel::Basic,
                SkillLevel::Expert => SkillLevel::Skilled,
                SkillLevel::Master => SkillLevel::Expert,
                SkillLevel::GrandMaster => SkillLevel::Master,
            };
            entry.advance = practice_needed(prev_level);
        }
    }

    entries
}

/// 스킬 레벨 설정 헬퍼
fn set_skill_level(entries: &mut [SkillInitEntry], skill: WeaponSkill, level: SkillLevel) {
    if let Some(entry) = entries.iter_mut().find(|e| e.skill == skill) {
        if entry.level < level {
            entry.level = level;
        }
    }
}

/// 전체 무기 스킬 목록
fn all_weapon_skills() -> Vec<WeaponSkill> {
    vec![
        WeaponSkill::Dagger,
        WeaponSkill::Knife,
        WeaponSkill::Axe,
        WeaponSkill::ShortSword,
        WeaponSkill::BroadSword,
        WeaponSkill::LongSword,
        WeaponSkill::TwoHandedSword,
        WeaponSkill::Scimitar,
        WeaponSkill::Saber,
        WeaponSkill::Club,
        WeaponSkill::Mace,
        WeaponSkill::MorningStar,
        WeaponSkill::Flail,
        WeaponSkill::Hammer,
        WeaponSkill::Quarterstaff,
        WeaponSkill::Polearm,
        WeaponSkill::Spear,
        WeaponSkill::Javelin,
        WeaponSkill::Trident,
        WeaponSkill::Lance,
        WeaponSkill::Bow,
        WeaponSkill::Sling,
        WeaponSkill::Crossbow,
        WeaponSkill::Dart,
        WeaponSkill::Shuriken,
        WeaponSkill::Boomerang,
        WeaponSkill::Whip,
        WeaponSkill::PickAxe,
        WeaponSkill::Unicorn,
        WeaponSkill::BareHanded,
        WeaponSkill::MartialArts,
        WeaponSkill::AttackSpell,
        WeaponSkill::HealingSpell,
        WeaponSkill::DivinationSpell,
        WeaponSkill::EnchantmentSpell,
        WeaponSkill::ClericalSpell,
        WeaponSkill::EscapeSpell,
        WeaponSkill::MatterSpell,
        WeaponSkill::Riding,
        WeaponSkill::TwoWeaponCombat,
    ]
}

// =============================================================================
// enhance_weapon_skill — #enhance 명령 UI 데이터 생성
// [v2.11.0] weapon.c:1160-1297 이식
// =============================================================================

/// #enhance 메뉴 스킬 항목
#[derive(Debug, Clone)]
pub struct EnhanceSkillEntry {
    /// 스킬
    pub skill: WeaponSkill,
    /// 현재 레벨
    pub level: SkillLevel,
    /// 최대 레벨
    pub max_level: SkillLevel,
    /// 승급 가능 여부
    pub can_advance: bool,
    /// 슬 만 있으면 가능
    pub could_advance: bool,
    /// 최대 도달 상태
    pub peaked: bool,
    /// 카테고리 ("Fighting Skills", "Weapon Skills", "Spellcasting Skills")
    pub category: &'static str,
}

/// #enhance 명령 결과 생성 (원본 enhance_weapon_skill의 순수 결과 버전)
/// [v2.11.0] weapon.c:1160-1297
pub fn enhance_weapon_skill_entries(
    skills: &[(WeaponSkill, SkillLevel, SkillLevel, i32)], // (스킬, 현재, 최대, 경험치)
    available_slots: i32,
    skills_advanced: i32,
    skill_limit: i32,
) -> Vec<EnhanceSkillEntry> {
    let mut result = Vec::new();

    for &(skill, level, max_level, advance) in skills {
        // 제한 상태면 표시 안 함
        if max_level <= SkillLevel::Unskilled && level <= SkillLevel::Unskilled {
            continue;
        }

        let ca = can_advance_result(
            level,
            max_level,
            advance,
            skills_advanced,
            skill_limit,
            available_slots,
            skill,
        );
        let coa = could_advance_result(level, max_level, advance, skills_advanced, skill_limit);
        let pk = peaked_skill_result(level, max_level, advance);

        let category = skill_category(skill);

        result.push(EnhanceSkillEntry {
            skill,
            level,
            max_level,
            can_advance: ca,
            could_advance: coa && !ca,
            peaked: pk,
            category,
        });
    }

    result
}

/// 스킬의 카테고리 분류 (원본 skill_ranges[])
fn skill_category(skill: WeaponSkill) -> &'static str {
    match skill {
        WeaponSkill::BareHanded
        | WeaponSkill::MartialArts
        | WeaponSkill::Riding
        | WeaponSkill::TwoWeaponCombat => "Fighting Skills",
        WeaponSkill::AttackSpell
        | WeaponSkill::HealingSpell
        | WeaponSkill::DivinationSpell
        | WeaponSkill::EnchantmentSpell
        | WeaponSkill::ClericalSpell
        | WeaponSkill::EscapeSpell
        | WeaponSkill::MatterSpell => "Spellcasting Skills",
        _ => "Weapon Skills",
    }
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hitval_blessed_vs_undead() {
        let input = HitvalInput {
            weapon_name: "long sword".into(),
            enchantment: 3,
            is_weapon: true,
            blessed: true,
            is_spear: false,
            is_pick: false,
            artifact_hit_bonus: 0,
            target_symbol: 'Z',
            target_is_undead: true,
            target_is_demon: false,
            target_is_vampshifter: false,
            target_is_swimmer: false,
            target_in_pool: false,
            target_passes_walls_thick: false,
        };
        // 강화치 3 + 축복 보너스 2 = 5
        assert_eq!(hitval_result(&input), 5);
    }

    #[test]
    fn test_hitval_spear_vs_dragon() {
        let input = HitvalInput {
            weapon_name: "spear".into(),
            enchantment: 0,
            is_weapon: true,
            blessed: false,
            is_spear: true,
            is_pick: false,
            artifact_hit_bonus: 0,
            target_symbol: 'D',
            target_is_undead: false,
            target_is_demon: false,
            target_is_vampshifter: false,
            target_is_swimmer: false,
            target_in_pool: false,
            target_passes_walls_thick: false,
        };
        assert_eq!(hitval_result(&input), 2);
    }

    #[test]
    fn test_hitval_trident_pool() {
        let input = HitvalInput {
            weapon_name: "trident".into(),
            enchantment: 0,
            is_weapon: true,
            blessed: false,
            is_spear: false,
            is_pick: false,
            artifact_hit_bonus: 0,
            target_symbol: ';',
            target_is_undead: false,
            target_is_demon: false,
            target_is_vampshifter: false,
            target_is_swimmer: true,
            target_in_pool: true,
            target_passes_walls_thick: false,
        };
        assert_eq!(hitval_result(&input), 4);
    }

    #[test]
    fn test_dmgval_cream_pie() {
        let mut rng = NetHackRng::new(42);
        let input = DmgvalInput {
            weapon_name: "cream pie".into(),
            enchantment: 0,
            is_weapon: false,
            blessed: false,
            is_axe: false,
            material_silver: false,
            artifact_light_lit: false,
            target_large: false,
            target_undead: false,
            target_demon: false,
            target_vampshifter: false,
            target_wooden: false,
            target_hates_silver: false,
            target_hates_light: false,
            target_thick_skinned: false,
            target_is_shade: false,
            greatest_erosion: 0,
            artifact_double_damage: false,
        };
        assert_eq!(dmgval_result(&input, &mut rng), 0);
    }

    #[test]
    fn test_dmgval_silver_vs_hater() {
        let mut rng = NetHackRng::new(42);
        let input = DmgvalInput {
            weapon_name: "silver dagger".into(),
            enchantment: 0,
            is_weapon: true,
            blessed: false,
            is_axe: false,
            material_silver: true,
            artifact_light_lit: false,
            target_large: false,
            target_undead: false,
            target_demon: false,
            target_vampshifter: false,
            target_wooden: false,
            target_hates_silver: true,
            target_hates_light: false,
            target_thick_skinned: false,
            target_is_shade: false,
            greatest_erosion: 0,
            artifact_double_damage: false,
        };
        assert!(dmgval_result(&input, &mut rng) >= 1);
    }

    #[test]
    fn test_abon() {
        assert_eq!(abon_result(18, 14, 5, false, 0), 1);
        assert_eq!(abon_result(5, 3, 5, false, 0), -5);
        assert_eq!(abon_result(10, 10, 2, false, 0), 1); // 저레벨 보정
    }

    #[test]
    fn test_dbon() {
        assert_eq!(dbon_result(5, false), -1);
        assert_eq!(dbon_result(10, false), 0);
        assert_eq!(dbon_result(18, false), 2);
        assert_eq!(dbon_result(120, false), 6);
        assert_eq!(dbon_result(18, true), 0);
    }

    #[test]
    fn test_weapon_hit_bonus_bare() {
        let b = weapon_hit_bonus_result(
            SkillLevel::Expert,
            false,
            SkillLevel::Unskilled,
            SkillLevel::Unskilled,
            true,
            false,
            false,
            SkillLevel::Unskilled,
        );
        assert_eq!(b, 2);
    }

    #[test]
    fn test_weapon_hit_bonus_martial() {
        let b = weapon_hit_bonus_result(
            SkillLevel::Expert,
            false,
            SkillLevel::Unskilled,
            SkillLevel::Unskilled,
            true,
            true,
            false,
            SkillLevel::Unskilled,
        );
        assert_eq!(b, 4);
    }

    #[test]
    fn test_weapon_dam_bonus_riding() {
        // Expert 무기(2) + Skilled 기승(+1) = 3
        let b = weapon_dam_bonus_result(
            SkillLevel::Expert,
            false,
            SkillLevel::Unskilled,
            SkillLevel::Unskilled,
            false,
            false,
            true,
            SkillLevel::Skilled,
        );
        assert_eq!(b, 3);
    }

    #[test]
    fn test_slots_required_fn() {
        assert_eq!(slots_required(WeaponSkill::LongSword, SkillLevel::Basic), 1);
        assert_eq!(
            slots_required(WeaponSkill::BareHanded, SkillLevel::Expert),
            2
        );
    }

    #[test]
    fn test_wet_towel() {
        let r = wet_a_towel_result(0, -1);
        assert_eq!(r.new_spe, 1);
        assert_eq!(r.message, Some("damp"));
        let r2 = wet_a_towel_result(2, -2);
        assert_eq!(r2.new_spe, 4);
        assert_eq!(r2.message, Some("wetter"));
    }

    #[test]
    fn test_dry_towel() {
        let r = dry_a_towel_result(1, -1);
        assert_eq!(r.new_spe, 0);
        assert_eq!(r.message, Some("dries out"));
    }

    #[test]
    fn test_weapon_descr_fn() {
        assert_eq!(weapon_descr("long sword"), "long sword");
        assert_eq!(weapon_descr("dwarvish mattock"), "mattock");
    }

    #[test]
    fn test_mon_might_throw() {
        assert!(mon_might_throw_wep("spear"));
        assert!(mon_might_throw_wep("silver dagger"));
        assert!(!mon_might_throw_wep("long sword"));
    }

    #[test]
    fn test_select_hwep() {
        let inv = vec![
            "dagger".to_string(),
            "long sword".to_string(),
            "katana".to_string(),
        ];
        let pick = select_hwep_result(&inv, true, false, false);
        assert!(pick.is_some());
    }

    #[test]
    fn test_skill_names() {
        assert_eq!(skill_level_name(SkillLevel::GrandMaster), "Grand Master");
        assert_eq!(skill_name(WeaponSkill::LongSword), "long sword");
        assert_eq!(skill_name(WeaponSkill::AttackSpell), "attack spells");
    }

    #[test]
    fn test_unrestrict() {
        let r = unrestrict_weapon_skill_result(SkillLevel::Unskilled);
        assert!(r.is_some());
        assert_eq!(r.unwrap().new_max, SkillLevel::Basic);
    }

    #[test]
    fn test_use_skill() {
        let r = use_skill_result(
            WeaponSkill::LongSword,
            60,
            50,
            SkillLevel::Unskilled,
            SkillLevel::Expert,
        );
        assert!(r.newly_advanceable);
        assert_eq!(r.skill_category, "weapon");
    }

    #[test]
    fn test_special_dmgval() {
        let mut rng = NetHackRng::new(42);
        let r = special_dmgval_result(true, true, true, false, false, true, 0x01, &mut rng);
        assert!(r.bonus >= 2);
        assert_eq!(r.silver_hit_mask, 0x01);
    }

    #[test]
    fn test_practice_needed() {
        assert_eq!(practice_needed(SkillLevel::Unskilled), 100);
        assert_eq!(practice_needed(SkillLevel::Basic), 200);
    }

    // =====================================================================
    // [v2.11.0] 신규 이식 함수 테스트
    // =====================================================================

    #[test]
    fn test_select_rwep_cockatrice_egg_priority() {
        // 코카트리스 알이 인벤토리에 있으면 최우선
        let inv = vec![
            MonInventoryItem {
                name: "long sword".into(),
                is_artifact: false,
                cursed: false,
                is_wielded: true,
                is_welded: false,
            },
            MonInventoryItem {
                name: "cockatrice egg".into(),
                is_artifact: false,
                cursed: false,
                is_wielded: false,
                is_welded: false,
            },
        ];
        let r = select_rwep_result(&inv, false, false, true, false, false, 25, true, false);
        assert!(r.is_some());
        assert_eq!(r.unwrap().weapon_index, 1); // 코카트리스 알
    }

    #[test]
    fn test_select_rwep_kop_cream_pie() {
        // Kop은 크림 파이 우선
        let inv = vec![
            MonInventoryItem {
                name: "spear".into(),
                is_artifact: false,
                cursed: false,
                is_wielded: false,
                is_welded: false,
            },
            MonInventoryItem {
                name: "cream pie".into(),
                is_artifact: false,
                cursed: false,
                is_wielded: false,
                is_welded: false,
            },
        ];
        let r = select_rwep_result(&inv, true, false, true, false, false, 25, true, false);
        assert!(r.is_some());
        assert_eq!(r.unwrap().weapon_index, 1);
    }

    #[test]
    fn test_select_rwep_arrow_with_bow() {
        // 활이 있어야 화살 선택 가능
        let inv = vec![
            MonInventoryItem {
                name: "arrow".into(),
                is_artifact: false,
                cursed: false,
                is_wielded: false,
                is_welded: false,
            },
            MonInventoryItem {
                name: "bow".into(),
                is_artifact: false,
                cursed: false,
                is_wielded: false,
                is_welded: false,
            },
        ];
        let r = select_rwep_result(&inv, false, false, true, false, false, 25, true, false);
        assert!(r.is_some());
        let res = r.unwrap();
        assert_eq!(res.weapon_index, 0); // 화살
        assert_eq!(res.propellor_index, Some(1)); // 활
    }

    #[test]
    fn test_select_rwep_no_bow_no_arrow() {
        // 활이 없으면 화살 선택 불가
        let inv = vec![MonInventoryItem {
            name: "arrow".into(),
            is_artifact: false,
            cursed: false,
            is_wielded: false,
            is_welded: false,
        }];
        let r = select_rwep_result(&inv, false, false, true, false, false, 25, true, false);
        // 화살은 발사대가 필요하므로 불가, 다른 투척 무기도 없으므로 None
        assert!(r.is_none());
    }

    #[test]
    fn test_select_rwep_throws_rocks() {
        // 바위 던지기 가능 몬스터 → 바위 우선
        let inv = vec![
            MonInventoryItem {
                name: "spear".into(),
                is_artifact: false,
                cursed: false,
                is_wielded: false,
                is_welded: false,
            },
            MonInventoryItem {
                name: "boulder".into(),
                is_artifact: false,
                cursed: false,
                is_wielded: false,
                is_welded: false,
            },
        ];
        let r = select_rwep_result(&inv, false, true, true, false, false, 25, true, false);
        assert!(r.is_some());
        assert_eq!(r.unwrap().weapon_index, 1); // 바위
    }

    #[test]
    fn test_select_rwep_skip_artifact() {
        // 아티팩트 무기는 던지지 않음
        let inv = vec![
            MonInventoryItem {
                name: "spear".into(),
                is_artifact: true,
                cursed: false,
                is_wielded: false,
                is_welded: false,
            },
            MonInventoryItem {
                name: "dagger".into(),
                is_artifact: false,
                cursed: false,
                is_wielded: false,
                is_welded: false,
            },
        ];
        let r = select_rwep_result(&inv, false, false, true, false, false, 25, true, false);
        // 아티팩트 창은 스킵, 단검이 선택될 수 있음 (THROWABLE에 silver dagger 있음)
        // 일반 dagger는 THROWABLE에 포함
        assert!(r.is_some());
        let res = r.unwrap();
        assert_eq!(res.weapon_index, 1); // 단검
    }

    #[test]
    fn test_can_advance_result_fn() {
        // 경험치 충분, 슬롯 충분 → 승급 가능
        assert!(can_advance_result(
            SkillLevel::Unskilled,
            SkillLevel::Expert,
            150,
            0,
            10,
            5,
            WeaponSkill::LongSword,
        ));
        // 이미 최대 → 불가
        assert!(!can_advance_result(
            SkillLevel::Expert,
            SkillLevel::Expert,
            999,
            0,
            10,
            5,
            WeaponSkill::LongSword,
        ));
        // 경험치 부족 → 불가
        assert!(!can_advance_result(
            SkillLevel::Unskilled,
            SkillLevel::Expert,
            50,
            0,
            10,
            5,
            WeaponSkill::LongSword,
        ));
    }

    #[test]
    fn test_could_advance_result_fn() {
        // 경험치만 충분하면 could는 true
        assert!(could_advance_result(
            SkillLevel::Unskilled,
            SkillLevel::Expert,
            150,
            0,
            10,
        ));
        // 이미 최대면 false
        assert!(!could_advance_result(
            SkillLevel::Expert,
            SkillLevel::Expert,
            999,
            0,
            10,
        ));
    }

    #[test]
    fn test_peaked_skill_result_fn() {
        // 최대 상태 + 경험치 넘침
        assert!(peaked_skill_result(
            SkillLevel::Expert,
            SkillLevel::Expert,
            999,
        ));
        // 아직 최대 미만
        assert!(!peaked_skill_result(
            SkillLevel::Basic,
            SkillLevel::Expert,
            999,
        ));
    }

    #[test]
    fn test_skill_advance_result_fn() {
        let r = skill_advance_result(
            WeaponSkill::LongSword,
            SkillLevel::Basic,
            SkillLevel::Expert,
        );
        assert_eq!(r.new_level, SkillLevel::Skilled);
        assert_eq!(r.slots_consumed, 1); // Basic 슬롯 = 1
        assert!(!r.is_most_skilled); // Expert가 max인데 Skilled로 올라감

        let r2 = skill_advance_result(
            WeaponSkill::LongSword,
            SkillLevel::Skilled,
            SkillLevel::Expert,
        );
        assert_eq!(r2.new_level, SkillLevel::Expert);
        assert!(r2.is_most_skilled); // 최대 도달
    }

    #[test]
    fn test_lose_weapon_skill_result_fn() {
        // 미사용 슬롯에서 차감
        let r = lose_weapon_skill_result(1, 3, 2, &[]);
        assert_eq!(r.remaining_slots, 2);
        assert_eq!(r.skills_advanced, 2);
        assert!(r.demoted.is_empty());

        // 슬롯 없으면 강등
        let record = vec![(WeaponSkill::LongSword, SkillLevel::Skilled)];
        let r = lose_weapon_skill_result(1, 0, 3, &record);
        assert_eq!(r.skills_advanced, 2);
        assert_eq!(r.demoted.len(), 1);
        assert_eq!(r.demoted[0], (WeaponSkill::LongSword, SkillLevel::Basic));
    }

    #[test]
    fn test_skill_init_barbarian() {
        // 바바리안은 맨손 Expert 이하이므로 BareHanded가 Basic이 되지 않는 게 정상
        // (Expert 캡이므로 > Expert 미만 → Basic 안 됨)
        let inv = vec!["two-handed sword".to_string()];
        let entries = skill_init_result("barbarian", &inv, false);

        // two-handed sword 스킬이 Basic인지 확인
        let ths = entries
            .iter()
            .find(|e| e.skill == WeaponSkill::TwoHandedSword)
            .unwrap();
        assert_eq!(ths.level, SkillLevel::Basic);
        assert_eq!(ths.max_level, SkillLevel::Expert);

        // BareHanded max가 Expert 이므로 > Expert가 아님 → Basic 안 됨
        let bh = entries
            .iter()
            .find(|e| e.skill == WeaponSkill::BareHanded)
            .unwrap();
        assert_eq!(bh.max_level, SkillLevel::Expert);
    }

    #[test]
    fn test_skill_init_monk() {
        let inv = vec!["quarterstaff".to_string()];
        let entries = skill_init_result("monk", &inv, false);

        // 수도승은 맨손 GrandMaster 캡 → Basic으로 시작
        let bh = entries
            .iter()
            .find(|e| e.skill == WeaponSkill::BareHanded)
            .unwrap();
        assert!(bh.max_level > SkillLevel::Expert);
        assert_eq!(bh.level, SkillLevel::Basic);

        // 치유 마법 스킬이 Basic 이상
        let hs = entries
            .iter()
            .find(|e| e.skill == WeaponSkill::HealingSpell)
            .unwrap();
        assert!(hs.level >= SkillLevel::Basic);
    }

    #[test]
    fn test_skill_init_knight_pony() {
        let inv = vec!["long sword".to_string()];
        let entries = skill_init_result("knight", &inv, true);

        // 말 시작 → 기승 Basic
        let ride = entries
            .iter()
            .find(|e| e.skill == WeaponSkill::Riding)
            .unwrap();
        assert_eq!(ride.level, SkillLevel::Basic);
    }

    #[test]
    fn test_enhance_weapon_skill_entries_fn() {
        let skills = vec![
            (
                WeaponSkill::LongSword,
                SkillLevel::Unskilled,
                SkillLevel::Expert,
                150,
            ),
            (
                WeaponSkill::Dagger,
                SkillLevel::Basic,
                SkillLevel::Basic,
                300,
            ),
            (
                WeaponSkill::Axe,
                SkillLevel::Unskilled,
                SkillLevel::Unskilled,
                0,
            ), // 제한
        ];
        let entries = enhance_weapon_skill_entries(&skills, 5, 0, 10);
        // Axe는 제한이므로 표시 안 됨
        assert_eq!(entries.len(), 2);
        // LongSword은 경험치 충분 → 승급 가능
        let ls = entries
            .iter()
            .find(|e| e.skill == WeaponSkill::LongSword)
            .unwrap();
        assert!(ls.can_advance);
        // Dagger는 이미 최대 → peaked
        let dg = entries
            .iter()
            .find(|e| e.skill == WeaponSkill::Dagger)
            .unwrap();
        assert!(dg.peaked);
        assert!(!dg.can_advance);
    }

    #[test]
    fn test_skill_category_fn() {
        assert_eq!(skill_category(WeaponSkill::BareHanded), "Fighting Skills");
        assert_eq!(
            skill_category(WeaponSkill::AttackSpell),
            "Spellcasting Skills"
        );
        assert_eq!(skill_category(WeaponSkill::LongSword), "Weapon Skills");
    }
}

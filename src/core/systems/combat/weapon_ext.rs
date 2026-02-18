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
}

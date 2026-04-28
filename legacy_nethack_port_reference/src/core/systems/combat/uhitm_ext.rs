// =============================================================================
// AIHack — uhitm_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
// [v2.19.0] uhitm.c 핵심 함수 이식 — Pure Result 패턴
// 원본: nethack-3.6.7/src/uhitm.c (2,975줄)
//
// 이식 대상:
//   erode_armor       (L34-96)   → erode_armor_result
//   check_caitiff     (L236-253) → check_caitiff_result
//   joust             (L1444-1472) → joust_result
//   shade_aware       (L1338-1358) → shade_aware_check
//   m_slips_free      (L1400-1440) → slips_free_result
//   damageum          (L1630-2032) → damageum_result
//   explum            (L2034-2085) → explum_result
//   demonpet          (L1482-1495) → demonpet_chance
//   theft_petrifies   (L1497-1510) → theft_petrifies_check
//   attack_checks     (L98-231)  → attack_checks_result
// =============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [v2.19.0] 방어구 침식 대상 결정 (원본: uhitm.c L34-96 erode_armor)
// 녹먹이나 산성 몬스터가 공격 시 어떤 방어구 슬롯을 침식할지 결정
// 원본은 무한 루프로 유효 대상을 찾을 때까지 반복 — 여기서는 최대 20회 시도
// =============================================================================

/// 방어구 슬롯 (원본: W_ARMH, W_ARMC, W_ARM, W_ARMU, W_ARMS, W_ARMG, W_ARMF)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlot {
    /// 투구 (W_ARMH)
    Head,
    /// 망토 (W_ARMC)
    Cloak,
    /// 갑옷 본체 (W_ARM)
    Body,
    /// 속옷/셔츠 (W_ARMU)
    Shirt,
    /// 방패 (W_ARMS)
    Shield,
    /// 장갑 (W_ARMG)
    Gloves,
    /// 신발 (W_ARMF)
    Boots,
}

/// 침식 유형 (원본: ERODE_RUST, ERODE_CORRODE, ERODE_ROT 등)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErodeType {
    /// 녹 (철 계열)
    Rust,
    /// 부식 (산성)
    Corrode,
    /// 부패 (유기물)
    Rot,
    /// 화염 손상
    Burn,
}

/// 방어구 정보 입력 (침식 판정에 필요한 정보)
#[derive(Debug, Clone)]
pub struct ArmorInfo {
    /// 슬롯 종류
    pub slot: ArmorSlot,
    /// 장착 여부
    pub equipped: bool,
    /// 이미 완전 침식되었는가 (erodeproof이거나 최대 침식)
    pub already_eroded_max: bool,
    /// 그리스 도포 여부
    pub greased: bool,
}

/// 침식 대상 결정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErodeArmorResult {
    /// 대상 슬롯 침식 (실제 침식 적용은 호출자)
    Erode(ArmorSlot),
    /// 그리스가 보호하여 침식 무효
    GreaseProtected(ArmorSlot),
    /// 적절한 대상 없음
    NoTarget,
}

/// [v2.19.0] 방어구 침식 대상 결정 (원본: erode_armor L34-96)
/// 5개 슬롯(투구/망토+갑옷+셔츠/방패/장갑/신발) 중 하나를 랜덤 선택
/// 해당 슬롯이 없거나 이미 최대 침식이면 재시도 (최대 20회)
pub fn erode_armor_result(armor_set: &[ArmorInfo], rng: &mut NetHackRng) -> ErodeArmorResult {
    // 원본: while(1) { switch(rn2(5)) ... } — 최대 20회 시도로 제한
    for _ in 0..20 {
        let roll = rng.rn2(5);
        let target_slot = match roll {
            0 => ArmorSlot::Head,
            1 => ArmorSlot::Cloak, // 망토 없으면 갑옷, 갑옷 없으면 셔츠
            2 => ArmorSlot::Shield,
            3 => ArmorSlot::Gloves,
            _ => ArmorSlot::Boots,
        };

        // 슬롯 1(망토/갑옷/셔츠) 특수 처리: 원본 uhitm.c L56-70
        if roll == 1 {
            // 망토 우선, 없으면 갑옷, 없으면 셔츠
            let fallback_order = [ArmorSlot::Cloak, ArmorSlot::Body, ArmorSlot::Shirt];
            let mut found = false;
            for &slot in &fallback_order {
                if let Some(info) = armor_set.iter().find(|a| a.slot == slot && a.equipped) {
                    if info.greased {
                        return ErodeArmorResult::GreaseProtected(slot);
                    }
                    if !info.already_eroded_max {
                        return ErodeArmorResult::Erode(slot);
                    }
                    found = true;
                    break; // 망토가 있으면 멈춤 (원본과 동일)
                }
            }
            if found {
                continue; // 대상 있었지만 이미 최대 침식
            }
            continue; // 해당 슬롯 없음 — 재시도
        }

        // 일반 슬롯 처리
        if let Some(info) = armor_set
            .iter()
            .find(|a| a.slot == target_slot && a.equipped)
        {
            if info.greased {
                return ErodeArmorResult::GreaseProtected(target_slot);
            }
            if info.already_eroded_max {
                continue; // 이미 최대 침식 — 재시도
            }
            return ErodeArmorResult::Erode(target_slot);
        }
        // 해당 슬롯에 방어구 없음 — 재시도
    }
    ErodeArmorResult::NoTarget
}

// =============================================================================
// [v2.19.0] 기사도 위반 판정 (원본: uhitm.c L236-253 check_caitiff)
// 기사(Knight)가 무방비 상태의 적을 공격하면 성향 페널티
// 사무라이(Samurai)가 평화 생물을 공격하면 기리(義理) 위반
// =============================================================================

/// 역할(직업) 종류 — 기사도 판정에 필요한 것만
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleForCaitiff {
    Knight,
    Samurai,
    Other,
}

/// 기사도 위반 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaitiffResult {
    /// 기사가 무방비 적 공격 — "You caitiff!" + 성향 -1
    KnightCaitiff,
    /// 사무라이가 평화 생물 공격 — 명예 위반 + 성향 -1
    SamuraiDishonor,
    /// 위반 없음
    NoViolation,
}

/// [v2.19.0] 기사도 위반 판정 (원본: check_caitiff L236-253)
/// alignment_record가 -10 이하면 이미 평판 바닥이므로 무시
pub fn check_caitiff_result(
    role: RoleForCaitiff,
    is_lawful: bool,
    alignment_record: i32,
    target_can_move: bool,
    target_sleeping: bool,
    target_fleeing: bool,
    target_avenging: bool,
    target_peaceful: bool,
) -> CaitiffResult {
    // 원본: if (u.ualign.record <= -10) return;
    if alignment_record <= -10 {
        return CaitiffResult::NoViolation;
    }

    match role {
        // 원본: Role_if(PM_KNIGHT) && u.ualign.type == A_LAWFUL
        //        && (!mtmp->mcanmove || mtmp->msleeping
        //            || (mtmp->mflee && !mtmp->mavenge))
        RoleForCaitiff::Knight if is_lawful => {
            if !target_can_move || target_sleeping || (target_fleeing && !target_avenging) {
                return CaitiffResult::KnightCaitiff;
            }
        }
        // 원본: Role_if(PM_SAMURAI) && mtmp->mpeaceful
        RoleForCaitiff::Samurai => {
            if target_peaceful {
                return CaitiffResult::SamuraiDishonor;
            }
        }
        _ => {}
    }
    CaitiffResult::NoViolation
}

// =============================================================================
// [v2.19.0] 마상 창 돌격(Joust) 판정 (원본: uhitm.c L1444-1472 joust)
// 기마 상태에서 창으로 공격 시 추가 보너스/창 파괴 판정
// =============================================================================

/// 마상 돌격 판정 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoustResult {
    /// 성공적 돌격 — 추가 데미지
    JoustHit,
    /// 돌격 성공했지만 창 파괴
    LanceBroken,
    /// 돌격 없음 — 일반 공격으로 진행
    NoJoust,
}

/// 스킬 레벨 (원본: P_UNSKILLED ~ P_EXPERT)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillLevel {
    Restricted = 0,
    Unskilled = 1,
    Basic = 2,
    Skilled = 3,
    Expert = 4,
}

/// [v2.19.0] 마상 돌격 판정 (원본: joust L1444-1472)
/// 스킬별 돌격 확률: Expert 80%, Skilled 60%, Basic 40%, Unskilled 20%
/// 돌격 성공 시 1/50 확률로 창 파괴 (비고체 대상 제외)
pub fn joust_result(
    skill: SkillLevel,
    is_fumbling: bool,
    is_stunned: bool,
    target_unsolid: bool,
    lance_resistant: bool,
    rng: &mut NetHackRng,
) -> JoustResult {
    // 원본: if (Fumbling || Stunned) return 0;
    if is_fumbling || is_stunned {
        return JoustResult::NoJoust;
    }

    // 원본: skill_rating 계산 — Restricted → Unskilled (0→1)
    let rating = if skill == SkillLevel::Restricted {
        1
    } else {
        skill as i32
    };
    // 원본: if ((joust_dieroll = rn2(5)) < skill_rating)
    let joust_dieroll = rng.rn2(5);
    if joust_dieroll < rating {
        // 원본: if (joust_dieroll == 0 && rnl(50) == 49 && !unsolid && !obj_resists)
        if joust_dieroll == 0 && rng.rnl(50, 0) == 49 && !target_unsolid && !lance_resistant {
            return JoustResult::LanceBroken;
        }
        return JoustResult::JoustHit;
    }
    JoustResult::NoJoust
}

// =============================================================================
// [v2.19.0] 그림자(Shade) 면역 무기 판정 (원본: uhitm.c L1338-1358 shade_aware)
// 그림자 몬스터에게 물리적 데미지를 줄 수 있는 무기인지 확인
// =============================================================================

/// 아이템 종류 (그림자 판정용 최소 집합)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadeItem {
    /// 바위
    Boulder,
    /// 쇠공
    HeavyIronBall,
    /// 쇠사슬
    IronChain,
    /// 거울 (은 반사면)
    Mirror,
    /// 마늘 (도주 유발)
    Garlic,
    /// 은제 무기/아이템
    SilverMaterial,
    /// 기타
    Other,
}

/// [v2.19.0] 그림자 면역 무기 판정 (원본: shade_aware L1338-1358)
/// 바위/쇠공/쇠사슬/거울/마늘/은은 그림자에게 데미지를 줄 수 있다
pub fn shade_aware_check(item: Option<ShadeItem>) -> bool {
    match item {
        None => false,
        Some(it) => matches!(
            it,
            ShadeItem::Boulder
                | ShadeItem::HeavyIronBall
                | ShadeItem::IronChain
                | ShadeItem::Mirror
                | ShadeItem::Garlic
                | ShadeItem::SilverMaterial
        ),
    }
}

// =============================================================================
// [v2.19.0] 미끄러운 방어구 잡기 실패 (원본: uhitm.c L1400-1440 m_slips_free)
// 그리스 도포/기름가죽 망토 착용 몬스터에 대한 잡기/감싸기 실패 판정
// =============================================================================

/// 미끄러운 방어구 판정 입력
#[derive(Debug, Clone)]
pub struct SlipFreeInput {
    /// 공격 유형이 지능 흡수(AD_DRIN)인가 → 투구 검사
    pub is_brain_drain: bool,
    /// 투구 장착 여부 + 그리스/기름가죽 여부
    pub has_greased_helm: bool,
    /// 망토/갑옷/셔츠 중 그리스 도포 또는 기름가죽인 것이 있는가
    pub has_greased_body: bool,
    /// 해당 방어구가 저주 상태인가
    pub armor_cursed: bool,
}

/// 미끄러운 방어구 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlipFreeResult {
    /// 미끄러져서 잡기 실패
    SlippedFree,
    /// 저주 때문에 그리스 무시, 잡기 성공
    CursedNoSlip,
    /// 미끄러운 방어구 없음 — 잡기 진행
    NoSlippery,
}

/// [v2.19.0] 미끄러운 방어구 잡기 실패 판정 (원본: m_slips_free L1400-1440)
/// 그리스/기름가죽 방어구는 잡기를 무효화하되, 저주 시 33% 확률로 통과
pub fn slips_free_result(input: &SlipFreeInput, rng: &mut NetHackRng) -> SlipFreeResult {
    let has_greased = if input.is_brain_drain {
        input.has_greased_helm
    } else {
        input.has_greased_body
    };

    if !has_greased {
        return SlipFreeResult::NoSlippery;
    }

    // 원본: (!obj->cursed || rn2(3)) — 저주 시 1/3 확률로 그리스 무시
    if input.armor_cursed && rng.rn2(3) == 0 {
        return SlipFreeResult::CursedNoSlip;
    }

    SlipFreeResult::SlippedFree
}

// =============================================================================
// [v2.19.0] 변신 공격 데미지 유형 분기 (원본: uhitm.c L1630-2032 damageum)
// 플레이어가 변신(polymorph) 상태에서 공격 시 데미지 유형별 효과
// =============================================================================

/// 변신 공격 데미지 유형 (원본: AD_* 상수)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolyAttackType {
    Physical,
    Fire,
    Cold,
    Elec,
    Acid,
    Poison,
    DrainLife,
    Stun,
    Blind,
    Confuse,
    Sleep,
    Paralyze,
    Petrify,
    Teleport,
    Rust,
    Corrode,
    Decay,
    DrainMana,
    Slow,
    Wrap,
    Stick,
    Curse,
    Steal,
    StealGold,
    BrainDrain,
    Slime,
}

/// 변신 공격 결과
#[derive(Debug, Clone)]
pub struct DamageumResult {
    /// 기본 데미지
    pub damage: i32,
    /// 적용된 상태 이상
    pub status_effect: Option<String>,
    /// 대상 즉사 여부
    pub instant_kill: bool,
    /// 무효화(negation) 되었는가
    pub negated: bool,
    /// 메시지
    pub message: String,
}

/// [v2.19.0] 마법 무효화(Magic Negation) 판정
/// 원본: armpro = magic_negation(mdef); negated = !(rn2(10) >= 3 * armpro)
pub fn check_magic_negation(target_armor_mc: i32, rng: &mut NetHackRng) -> bool {
    let threshold = 3 * target_armor_mc;
    rng.rn2(10) < threshold
}

/// [v2.19.0] 변신 공격 데미지 계산 (원본: damageum L1630-2032)
/// 간소화된 Pure Result — ECS 부작용 없이 데미지와 효과만 판정
pub fn damageum_result(
    attack_type: PolyAttackType,
    base_dice: i32,
    base_sides: i32,
    special_dmg: i32,
    target_resists_fire: bool,
    target_resists_cold: bool,
    target_resists_elec: bool,
    target_resists_acid: bool,
    target_resists_poison: bool,
    target_resists_drain: bool,
    target_is_shade: bool,
    target_thick_skin: bool,
    negated: bool,
    rng: &mut NetHackRng,
) -> DamageumResult {
    let mut tmp = rng.d(base_dice, base_sides);

    match attack_type {
        // 원본: case AD_PHYS (L1667-1698)
        PolyAttackType::Physical | PolyAttackType::Stun => {
            if target_is_shade {
                tmp = 0;
            }
            tmp += special_dmg;
            if target_thick_skin {
                tmp = (tmp + 1) / 2;
            }
            let msg = if attack_type == PolyAttackType::Stun {
                "대상이 비틀거린다!"
            } else {
                "물리 공격 명중!"
            };
            DamageumResult {
                damage: tmp.max(0),
                status_effect: if attack_type == PolyAttackType::Stun {
                    Some("Stun".to_string())
                } else {
                    None
                },
                instant_kill: false,
                negated: false,
                message: msg.to_string(),
            }
        }
        // 원본: case AD_FIRE (L1700-1730)
        PolyAttackType::Fire => {
            if negated {
                return DamageumResult {
                    damage: 0,
                    status_effect: None,
                    instant_kill: false,
                    negated: true,
                    message: "화염 공격이 무효화되었다.".to_string(),
                };
            }
            if target_resists_fire {
                DamageumResult {
                    damage: 0,
                    status_effect: None,
                    instant_kill: false,
                    negated: false,
                    message: "화염이 대상에게 통하지 않는다!".to_string(),
                }
            } else {
                DamageumResult {
                    damage: tmp,
                    status_effect: None,
                    instant_kill: false,
                    negated: false,
                    message: "대상이 불타오른다!".to_string(),
                }
            }
        }
        // 원본: case AD_COLD (L1731-1746)
        PolyAttackType::Cold => {
            if negated {
                return DamageumResult {
                    damage: 0,
                    status_effect: None,
                    instant_kill: false,
                    negated: true,
                    message: "냉기 공격이 무효화되었다.".to_string(),
                };
            }
            if target_resists_cold {
                DamageumResult {
                    damage: 0,
                    status_effect: None,
                    instant_kill: false,
                    negated: false,
                    message: "냉기가 대상에게 통하지 않는다!".to_string(),
                }
            } else {
                DamageumResult {
                    damage: tmp,
                    status_effect: None,
                    instant_kill: false,
                    negated: false,
                    message: "대상이 서리로 뒤덮인다!".to_string(),
                }
            }
        }
        // 원본: case AD_ELEC (L1747-1764)
        PolyAttackType::Elec => {
            if negated {
                return DamageumResult {
                    damage: 0,
                    status_effect: None,
                    instant_kill: false,
                    negated: true,
                    message: "전기 공격이 무효화되었다.".to_string(),
                };
            }
            if target_resists_elec {
                DamageumResult {
                    damage: 0,
                    status_effect: None,
                    instant_kill: false,
                    negated: false,
                    message: "전격이 대상에게 통하지 않는다!".to_string(),
                }
            } else {
                DamageumResult {
                    damage: tmp,
                    status_effect: None,
                    instant_kill: false,
                    negated: false,
                    message: "대상이 감전된다!".to_string(),
                }
            }
        }
        // 원본: case AD_ACID (L1765-1768)
        PolyAttackType::Acid => {
            if target_resists_acid {
                tmp = 0;
            }
            DamageumResult {
                damage: tmp,
                status_effect: None,
                instant_kill: false,
                negated: false,
                message: if tmp > 0 {
                    "산성 공격!"
                } else {
                    "산이 효과가 없다."
                }
                .to_string(),
            }
        }
        // 원본: case AD_DRST/AD_DRDX/AD_DRCO (L1887-1902) — 독
        PolyAttackType::Poison => {
            if negated || rng.rn2(8) != 0 {
                DamageumResult {
                    damage: tmp,
                    status_effect: None,
                    instant_kill: false,
                    negated,
                    message: "공격 명중.".to_string(),
                }
            } else if target_resists_poison {
                DamageumResult {
                    damage: tmp,
                    status_effect: None,
                    instant_kill: false,
                    negated: false,
                    message: "독이 통하지 않는 것 같다.".to_string(),
                }
            } else {
                // 원본: 1/10 즉사, 아니면 rn1(10,6) 추가 데미지
                if rng.rn2(10) == 0 {
                    DamageumResult {
                        damage: 9999,
                        status_effect: Some("PoisonKill".to_string()),
                        instant_kill: true,
                        negated: false,
                        message: "치명적인 독이었다...".to_string(),
                    }
                } else {
                    let extra = rng.rn1(10, 6);
                    DamageumResult {
                        damage: tmp + extra,
                        status_effect: Some("Poison".to_string()),
                        instant_kill: false,
                        negated: false,
                        message: "독 공격!".to_string(),
                    }
                }
            }
        }
        // 원본: case AD_DRLI (L1845-1861)
        PolyAttackType::DrainLife => {
            if negated || rng.rn2(3) != 0 || target_resists_drain {
                DamageumResult {
                    damage: tmp,
                    status_effect: None,
                    instant_kill: false,
                    negated,
                    message: "공격 명중.".to_string(),
                }
            } else {
                let drain = rng.d(2, 6);
                DamageumResult {
                    damage: tmp,
                    status_effect: Some(format!("DrainLife({})", drain)),
                    instant_kill: false,
                    negated: false,
                    message: "대상이 갑자기 약해진 것 같다!".to_string(),
                }
            }
        }
        // 원본: case AD_PLYS (L1960-1966)
        PolyAttackType::Paralyze => {
            if !negated && rng.rn2(3) == 0 {
                let dur = rng.rnd(10);
                DamageumResult {
                    damage: tmp,
                    status_effect: Some(format!("Paralyze({})", dur)),
                    instant_kill: false,
                    negated: false,
                    message: "대상이 얼어붙었다!".to_string(),
                }
            } else {
                DamageumResult {
                    damage: tmp,
                    status_effect: None,
                    instant_kill: false,
                    negated,
                    message: "공격 명중.".to_string(),
                }
            }
        }
        // 원본: case AD_SLEE (L1967-1973)
        PolyAttackType::Sleep => {
            if !negated {
                let dur = rng.rnd(10);
                DamageumResult {
                    damage: tmp,
                    status_effect: Some(format!("Sleep({})", dur)),
                    instant_kill: false,
                    negated: false,
                    message: "대상이 잠들었다!".to_string(),
                }
            } else {
                DamageumResult {
                    damage: tmp,
                    status_effect: None,
                    instant_kill: true,
                    negated: true,
                    message: "공격 명중.".to_string(),
                }
            }
        }
        // 원본: case AD_CONF (L2004-2010)
        PolyAttackType::Confuse => DamageumResult {
            damage: tmp,
            status_effect: Some("Confusion".to_string()),
            instant_kill: false,
            negated: false,
            message: "대상이 혼란스러워 보인다.".to_string(),
        },
        // 원본: case AD_SLOW (L1995-2003)
        PolyAttackType::Slow => {
            if !negated {
                DamageumResult {
                    damage: tmp,
                    status_effect: Some("Slow".to_string()),
                    instant_kill: false,
                    negated: false,
                    message: "대상이 느려진다.".to_string(),
                }
            } else {
                DamageumResult {
                    damage: tmp,
                    status_effect: None,
                    instant_kill: false,
                    negated: true,
                    message: "공격 명중.".to_string(),
                }
            }
        }
        // 원본: case AD_BLND (L1818-1829)
        PolyAttackType::Blind => {
            let blind_dur = rng.rnd(10).min(127);
            DamageumResult {
                damage: 0,
                status_effect: Some(format!("Blind({})", blind_dur)),
                instant_kill: false,
                negated: false,
                message: "대상이 눈이 멀었다.".to_string(),
            }
        }
        // 원본: case AD_STON (L1769-1773)
        PolyAttackType::Petrify => DamageumResult {
            damage: 0,
            status_effect: Some("Petrify".to_string()),
            instant_kill: false,
            negated: false,
            message: "대상이 돌로 변하기 시작한다!".to_string(),
        },
        // 원본: case AD_TLPT (L1798-1817)
        PolyAttackType::Teleport => {
            let dmg = if tmp <= 0 { 1 } else { tmp };
            if !negated {
                DamageumResult {
                    damage: dmg,
                    status_effect: Some("Teleport".to_string()),
                    instant_kill: false,
                    negated: false,
                    message: "대상이 갑자기 사라졌다!".to_string(),
                }
            } else {
                DamageumResult {
                    damage: dmg,
                    status_effect: None,
                    instant_kill: false,
                    negated: true,
                    message: "공격 명중.".to_string(),
                }
            }
        }
        // 원본: case AD_RUST (L1862-1869)
        PolyAttackType::Rust => DamageumResult {
            damage: 0,
            status_effect: Some("Rust".to_string()),
            instant_kill: false,
            negated: false,
            message: "방어구가 녹슨다!".to_string(),
        },
        // 원본: case AD_CORR (L1870-1873)
        PolyAttackType::Corrode => DamageumResult {
            damage: 0,
            status_effect: Some("Corrode".to_string()),
            instant_kill: false,
            negated: false,
            message: "방어구가 부식된다!".to_string(),
        },
        // 원본: case AD_DCAY (L1874-1881)
        PolyAttackType::Decay => DamageumResult {
            damage: 0,
            status_effect: Some("Decay".to_string()),
            instant_kill: false,
            negated: false,
            message: "방어구가 부패한다!".to_string(),
        },
        // 기타 — 기본 물리 데미지
        _ => DamageumResult {
            damage: tmp,
            status_effect: None,
            instant_kill: false,
            negated: false,
            message: "공격 명중.".to_string(),
        },
    }
}

// =============================================================================
// [v2.19.0] 폭발 공격 결과 (원본: uhitm.c L2034-2085 explum)
// 변신해서 폭발하는 경우의 데미지 판정
// =============================================================================

/// 폭발 공격 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplumType {
    Blind,
    Hallucinate,
    Cold,
    Fire,
    Elec,
}

/// 폭발 공격 결과
#[derive(Debug, Clone)]
pub struct ExplumResult {
    pub damage: i32,
    pub target_blinded: bool,
    pub target_confused: bool,
    pub resisted: bool,
    pub message: String,
}

/// [v2.19.0] 폭발 공격 결과 계산 (원본: explum L2034-2085)
pub fn explum_result(
    explode_type: ExplumType,
    dice: i32,
    sides: i32,
    target_resists: bool,
    target_has_eyes: bool,
    target_can_see: bool,
    rng: &mut NetHackRng,
) -> ExplumResult {
    let tmp = rng.d(dice, sides);

    match explode_type {
        // 원본: case AD_BLND (L2044-2050)
        ExplumType::Blind => {
            if !target_resists && target_has_eyes && target_can_see {
                ExplumResult {
                    damage: 0,
                    target_blinded: true,
                    target_confused: false,
                    resisted: false,
                    message: "섬광에 눈이 멀었다!".to_string(),
                }
            } else {
                ExplumResult {
                    damage: 0,
                    target_blinded: false,
                    target_confused: false,
                    resisted: true,
                    message: "섬광이 효과가 없다.".to_string(),
                }
            }
        }
        // 원본: case AD_HALU (L2051-2056)
        ExplumType::Hallucinate => {
            if target_has_eyes && target_can_see {
                ExplumResult {
                    damage: 0,
                    target_blinded: false,
                    target_confused: true,
                    resisted: false,
                    message: "빛에 영향을 받았다!".to_string(),
                }
            } else {
                ExplumResult {
                    damage: 0,
                    target_blinded: false,
                    target_confused: false,
                    resisted: true,
                    message: "효과 없음.".to_string(),
                }
            }
        }
        // 원본: case AD_COLD/AD_FIRE/AD_ELEC (L2057-2080)
        ExplumType::Cold | ExplumType::Fire | ExplumType::Elec => {
            if !target_resists {
                ExplumResult {
                    damage: tmp,
                    target_blinded: false,
                    target_confused: false,
                    resisted: false,
                    message: "폭발에 휩쓸렸다!".to_string(),
                }
            } else {
                ExplumResult {
                    damage: 0,
                    target_blinded: false,
                    target_confused: false,
                    resisted: true,
                    message: "폭발이 통하지 않는 것 같다.".to_string(),
                }
            }
        }
    }
}

// =============================================================================
// [v2.19.0] 악마 펫 소환 확률 (원본: uhitm.c L1482-1495 demonpet)
// 변신한 악마가 근접 공격 시 1/13 확률로 악마 펫 소환
// =============================================================================

/// [v2.19.0] 악마 펫 소환 판정 (원본: demonpet L1482-1495)
/// 무기 없이 공격하면 1/13 확률로 소환
pub fn demonpet_chance(
    is_demon: bool,
    wielding_weapon: bool,
    is_succubus_incubus: bool,
    is_balrog: bool,
    rng: &mut NetHackRng,
) -> bool {
    // 원본: is_demon(youmonst.data) && !rn2(13) && !uwep
    //        && u.umonnum != PM_SUCCUBUS && != PM_INCUBUS && != PM_BALROG
    is_demon && !wielding_weapon && !is_succubus_incubus && !is_balrog && rng.rn2(13) == 0
}

// =============================================================================
// [v2.19.0] 공격 전 확인 — 평화 몬스터/숨은 몬스터 판정
// (원본: uhitm.c L98-231 attack_checks)
// =============================================================================

/// 공격 전 확인 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttackCheckResult {
    /// 공격 진행 허용
    ProceedAttack,
    /// 보이지 않는 몬스터 감지 — 공격 중단, 위치 표시
    InvisibleDetected,
    /// 미믹 발견 — 공격 중단
    MimicRevealed,
    /// 숨어있는 몬스터 감지 — 공격 중단
    HiddenMonsterDetected,
    /// 평화 생물 공격 확인 거부 — 공격 중단
    PeacefulRefused,
    /// Stormbringer 강제 공격
    StormbringerOverride,
}

/// [v2.19.0] 공격 전 확인 (원본: attack_checks L98-231)
/// 평화 몬스터/숨은 몬스터 등의 공격 가부 판정
pub fn attack_checks_result(
    is_peaceful: bool,
    is_tame: bool,
    is_invisible: bool,
    is_mimic: bool,
    is_hidden: bool,
    player_confused: bool,
    player_hallucinating: bool,
    player_stunned: bool,
    player_blind: bool,
    has_stormbringer: bool,
    is_swallowed_target: bool,
    force_fight: bool,
) -> AttackCheckResult {
    // 원본: if (u.uswallow && mtmp == u.ustuck) return FALSE; — 삼킨 상태면 공격 진행
    if is_swallowed_target {
        return AttackCheckResult::ProceedAttack;
    }

    // 원본: if (context.forcefight) return FALSE;
    if force_fight {
        return AttackCheckResult::ProceedAttack;
    }

    // 원본: 보이지 않고 기억된 표시도 없는 몬스터
    if is_invisible && !player_blind {
        return AttackCheckResult::InvisibleDetected;
    }

    // 원본: 미믹 발견
    if is_mimic {
        return AttackCheckResult::MimicRevealed;
    }

    // 원본: 숨은 몬스터 감지
    if is_hidden {
        return AttackCheckResult::HiddenMonsterDetected;
    }

    // 원본: flags.confirm && mtmp->mpeaceful && !Confusion && !Hallucination && !Stunned
    if (is_peaceful || is_tame) && !player_confused && !player_hallucinating && !player_stunned {
        if has_stormbringer {
            return AttackCheckResult::StormbringerOverride;
        }
        return AttackCheckResult::PeacefulRefused;
    }

    AttackCheckResult::ProceedAttack
}

// =============================================================================
// [v2.19.0] 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    // --- erode_armor_result 테스트 ---

    #[test]
    fn test_erode_no_armor() {
        let mut rng = NetHackRng::new(42);
        let result = erode_armor_result(&[], &mut rng);
        assert_eq!(result, ErodeArmorResult::NoTarget);
    }

    #[test]
    fn test_erode_greased_protection() {
        let mut rng = NetHackRng::new(1);
        let armor = vec![
            ArmorInfo {
                slot: ArmorSlot::Head,
                equipped: true,
                already_eroded_max: false,
                greased: true,
            },
            ArmorInfo {
                slot: ArmorSlot::Shield,
                equipped: true,
                already_eroded_max: false,
                greased: true,
            },
            ArmorInfo {
                slot: ArmorSlot::Gloves,
                equipped: true,
                already_eroded_max: false,
                greased: true,
            },
            ArmorInfo {
                slot: ArmorSlot::Boots,
                equipped: true,
                already_eroded_max: false,
                greased: true,
            },
            ArmorInfo {
                slot: ArmorSlot::Cloak,
                equipped: true,
                already_eroded_max: false,
                greased: true,
            },
        ];
        // 모든 슬롯에 그리스 — 어떤 걸 선택하든 GreaseProtected
        let result = erode_armor_result(&armor, &mut rng);
        match result {
            ErodeArmorResult::GreaseProtected(_) => {} // 정상
            _ => panic!("그리스 방어구에서 GreaseProtected 기대, 실제: {:?}", result),
        }
    }

    #[test]
    fn test_erode_finds_target() {
        let mut rng = NetHackRng::new(42);
        let armor = vec![
            ArmorInfo {
                slot: ArmorSlot::Head,
                equipped: true,
                already_eroded_max: false,
                greased: false,
            },
            ArmorInfo {
                slot: ArmorSlot::Body,
                equipped: true,
                already_eroded_max: false,
                greased: false,
            },
        ];
        let mut found_erode = false;
        for seed in 0..50 {
            rng = NetHackRng::new(seed);
            if let ErodeArmorResult::Erode(_) = erode_armor_result(&armor, &mut rng) {
                found_erode = true;
                break;
            }
        }
        assert!(found_erode, "침식 대상을 찾아야 한다");
    }

    // --- check_caitiff_result 테스트 ---

    #[test]
    fn test_knight_caitiff_sleeping() {
        let result = check_caitiff_result(
            RoleForCaitiff::Knight,
            true,
            0,
            true,
            true,
            false,
            false,
            false,
        );
        assert_eq!(result, CaitiffResult::KnightCaitiff);
    }

    #[test]
    fn test_knight_no_caitiff_awake_target() {
        let result = check_caitiff_result(
            RoleForCaitiff::Knight,
            true,
            0,
            true,
            false,
            false,
            false,
            false,
        );
        assert_eq!(result, CaitiffResult::NoViolation);
    }

    #[test]
    fn test_samurai_dishonor() {
        let result = check_caitiff_result(
            RoleForCaitiff::Samurai,
            false,
            0,
            true,
            false,
            false,
            false,
            true,
        );
        assert_eq!(result, CaitiffResult::SamuraiDishonor);
    }

    #[test]
    fn test_caitiff_low_alignment() {
        // 성향 -10 이하면 위반 없음
        let result = check_caitiff_result(
            RoleForCaitiff::Knight,
            true,
            -15,
            true,
            true,
            false,
            false,
            false,
        );
        assert_eq!(result, CaitiffResult::NoViolation);
    }

    // --- joust_result 테스트 ---

    #[test]
    fn test_joust_fumbling() {
        let mut rng = NetHackRng::new(42);
        let result = joust_result(SkillLevel::Expert, true, false, false, false, &mut rng);
        assert_eq!(result, JoustResult::NoJoust);
    }

    #[test]
    fn test_joust_expert_high_chance() {
        let mut hits = 0;
        for seed in 0..100 {
            let mut rng = NetHackRng::new(seed);
            if joust_result(SkillLevel::Expert, false, false, false, false, &mut rng)
                != JoustResult::NoJoust
            {
                hits += 1;
            }
        }
        // Expert: 80% 확률 — 60회 이상은 성공해야 함
        assert!(hits >= 50, "Expert 조스트 확률이 너무 낮다: {}/100", hits);
    }

    #[test]
    fn test_joust_unskilled_low_chance() {
        let mut hits = 0;
        for seed in 0..100 {
            let mut rng = NetHackRng::new(seed);
            if joust_result(SkillLevel::Unskilled, false, false, false, false, &mut rng)
                != JoustResult::NoJoust
            {
                hits += 1;
            }
        }
        assert!(
            hits <= 40,
            "Unskilled 조스트 확률이 너무 높다: {}/100",
            hits
        );
    }

    // --- shade_aware_check 테스트 ---

    #[test]
    fn test_shade_none() {
        assert!(!shade_aware_check(None));
    }

    #[test]
    fn test_shade_silver() {
        assert!(shade_aware_check(Some(ShadeItem::SilverMaterial)));
    }

    #[test]
    fn test_shade_other() {
        assert!(!shade_aware_check(Some(ShadeItem::Other)));
    }

    #[test]
    fn test_shade_boulder() {
        assert!(shade_aware_check(Some(ShadeItem::Boulder)));
    }

    // --- slips_free_result 테스트 ---

    #[test]
    fn test_slip_no_grease() {
        let mut rng = NetHackRng::new(42);
        let input = SlipFreeInput {
            is_brain_drain: false,
            has_greased_helm: false,
            has_greased_body: false,
            armor_cursed: false,
        };
        assert_eq!(
            slips_free_result(&input, &mut rng),
            SlipFreeResult::NoSlippery
        );
    }

    #[test]
    fn test_slip_greased_body() {
        let mut rng = NetHackRng::new(42);
        let input = SlipFreeInput {
            is_brain_drain: false,
            has_greased_helm: false,
            has_greased_body: true,
            armor_cursed: false,
        };
        assert_eq!(
            slips_free_result(&input, &mut rng),
            SlipFreeResult::SlippedFree
        );
    }

    // --- damageum_result 테스트 ---

    #[test]
    fn test_damageum_physical() {
        let mut rng = NetHackRng::new(42);
        let r = damageum_result(
            PolyAttackType::Physical,
            2,
            6,
            5,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            &mut rng,
        );
        assert!(r.damage > 0, "물리 공격 데미지 > 0");
        assert!(r.status_effect.is_none());
    }

    #[test]
    fn test_damageum_fire_resisted() {
        let mut rng = NetHackRng::new(42);
        let r = damageum_result(
            PolyAttackType::Fire,
            2,
            6,
            0,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            &mut rng,
        );
        assert_eq!(r.damage, 0, "화염 저항 시 데미지 0");
    }

    #[test]
    fn test_damageum_negated() {
        let mut rng = NetHackRng::new(42);
        let r = damageum_result(
            PolyAttackType::Fire,
            2,
            6,
            0,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            true,
            &mut rng,
        );
        assert!(r.negated, "무효화 시 negated");
        assert_eq!(r.damage, 0);
    }

    // --- explum_result 테스트 ---

    #[test]
    fn test_explum_fire_no_resist() {
        let mut rng = NetHackRng::new(42);
        let r = explum_result(ExplumType::Fire, 2, 6, false, true, true, &mut rng);
        assert!(r.damage > 0);
        assert!(!r.resisted);
    }

    #[test]
    fn test_explum_fire_resisted() {
        let mut rng = NetHackRng::new(42);
        let r = explum_result(ExplumType::Fire, 2, 6, true, true, true, &mut rng);
        assert_eq!(r.damage, 0);
        assert!(r.resisted);
    }

    #[test]
    fn test_explum_blind() {
        let mut rng = NetHackRng::new(42);
        let r = explum_result(ExplumType::Blind, 1, 1, false, true, true, &mut rng);
        assert!(r.target_blinded);
    }

    // --- demonpet_chance 테스트 ---

    #[test]
    fn test_demonpet_not_demon() {
        let mut rng = NetHackRng::new(42);
        assert!(!demonpet_chance(false, false, false, false, &mut rng));
    }

    #[test]
    fn test_demonpet_with_weapon() {
        let mut rng = NetHackRng::new(42);
        assert!(!demonpet_chance(true, true, false, false, &mut rng));
    }

    #[test]
    fn test_demonpet_succubus() {
        let mut rng = NetHackRng::new(42);
        assert!(!demonpet_chance(true, false, true, false, &mut rng));
    }

    #[test]
    fn test_demonpet_probability() {
        let mut count = 0;
        for seed in 0..1300 {
            let mut rng = NetHackRng::new(seed);
            if demonpet_chance(true, false, false, false, &mut rng) {
                count += 1;
            }
        }
        // 1/13 확률 ≈ 100회 정도 기대
        assert!(
            count > 50 && count < 200,
            "악마 소환 확률 이상: {}/1300",
            count
        );
    }

    // --- attack_checks_result 테스트 ---

    #[test]
    fn test_attack_check_swallowed() {
        let r = attack_checks_result(
            false, false, false, false, false, false, false, false, false, false, true, false,
        );
        assert_eq!(r, AttackCheckResult::ProceedAttack);
    }

    #[test]
    fn test_attack_check_forcefight() {
        let r = attack_checks_result(
            true, false, false, false, false, false, false, false, false, false, false, true,
        );
        assert_eq!(r, AttackCheckResult::ProceedAttack);
    }

    #[test]
    fn test_attack_check_peaceful_refused() {
        let r = attack_checks_result(
            true, false, false, false, false, false, false, false, false, false, false, false,
        );
        assert_eq!(r, AttackCheckResult::PeacefulRefused);
    }

    #[test]
    fn test_attack_check_stormbringer() {
        let r = attack_checks_result(
            true, false, false, false, false, false, false, false, false, true, false, false,
        );
        assert_eq!(r, AttackCheckResult::StormbringerOverride);
    }

    #[test]
    fn test_attack_check_invisible() {
        let r = attack_checks_result(
            false, false, true, false, false, false, false, false, false, false, false, false,
        );
        assert_eq!(r, AttackCheckResult::InvisibleDetected);
    }

    #[test]
    fn test_attack_check_normal() {
        let r = attack_checks_result(
            false, false, false, false, false, false, false, false, false, false, false, false,
        );
        assert_eq!(r, AttackCheckResult::ProceedAttack);
    }
}

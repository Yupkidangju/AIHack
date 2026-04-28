// =============================================================================
// AIHack - wield_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// [v2.11.0] wield.c 핵심 함수 이식 (순수 결과 패턴)
// 원본: nethack-3.6.7/src/wield.c (893줄)
//
// 이 모듈은 무기 장비/교체/마법 강화에 관련된 원본 C 함수들을
// 순수 결과(pure result) 패턴으로 Rust에 이식합니다.
//
// 이식된 함수:
//   1. erodeable_wep — 부식 가능 무기 판정 (wield.c:60-62)
//   2. will_weld — 무기 용접 판정 (wield.c:65-66)
//   3. cant_wield_corpse_result — 코카트리스 시체 맨손 장비 (wield.c:117-134)
//   4. ready_weapon_result — 무기 장비 결과 계산 (wield.c:136-225)
//   5. can_twoweapon_result — 쌍수 전투 가능 판정 (wield.c:598-636)
//   6. chwepon_result — 마법 무기 강화/약화 (wield.c:722-855)
//   7. welded_result — 주 무기 용접 상태 확인 (wield.c:857-866)
//   8. mwelded_result — 몬스터 무기 용접 확인 (wield.c:882-890)
//   9. wield_tool_check — 도구 장비 사전 검증 (wield.c:518-596)
//  10. unweapon_check — 장비 시 비무기 판정 (wield.c:108-114)
//  11. wield_swap_result — 무기 교환 결과 (wield.c:309-350)
//  12. enchant_weapon_cap_check — 강화 상한 판정 (wield.c:805-817)
// =============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// 아이템 분류 상수 / 열거형
// =============================================================================

/// 아이템 클래스 (wield 판정용)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WieldItemClass {
    /// 무기류
    Weapon,
    /// 도구류 (곡괭이, 채찍 등 무기 도구 포함)
    Tool,
    /// 보석/돌
    Gem,
    /// 방어구
    Armor,
    /// 악세서리 (반지, 부적)
    Accessory,
    /// 안장
    Saddle,
    /// 음식 (시체 포함)
    Food,
    /// 기타
    Other,
}

/// 무기 장비 판정에 필요한 아이템 정보
#[derive(Debug, Clone)]
pub struct WieldItemInfo {
    /// 아이템 이름
    pub name: String,
    /// 아이템 클래스
    pub item_class: WieldItemClass,
    /// 저주 여부
    pub cursed: bool,
    /// 무기 도구 여부 (곡괭이, 유니콘 뿔 등)
    pub is_weptool: bool,
    /// 양손 무기 여부
    pub is_bimanual: bool,
    /// 발사대 여부 (활, 석궁, 투석구)
    pub is_launcher: bool,
    /// 탄약 여부 (화살, 볼트, 탄환)
    pub is_ammo: bool,
    /// 투척 무기 여부 (부메랑 등)
    pub is_missile: bool,
    /// 장대 무기 여부
    pub is_pole: bool,
    /// 젖은 수건 여부
    pub is_wet_towel: bool,
    /// 아티팩트 여부
    pub is_artifact: bool,
    /// 아티팩트 id (선택)
    pub artifact_id: Option<u32>,
    /// 코카트리스 시체 여부
    pub is_cockatrice_corpse: bool,
    /// 쇠구슬 (heavy iron ball)
    pub is_heavy_iron_ball: bool,
    /// 쇠사슬 (iron chain)
    pub is_iron_chain: bool,
    /// 깡통 따개 여부
    pub is_tin_opener: bool,
    /// 벌레 이빨 → 크리스 나이프 변환 대상
    pub is_worm_tooth: bool,
    /// 크리스 나이프 → 벌레 이빨 변환 대상
    pub is_crysknife: bool,
    /// 엘프 무기 여부
    pub is_elven_weapon: bool,
    /// 강화치 (spe)
    pub spe: i32,
    /// 수량
    pub quantity: i32,
    /// 부식 방지 여부
    pub erodeproof: bool,
    /// 미지불 여부 (상점)
    pub unpaid: bool,
    /// 착용 마스크 (방어구/악세서리로 착용 중인지)
    pub worn_as_armor_or_accessory: bool,
    /// 은 재질 여부
    pub is_silver: bool,
}

// =============================================================================
// erodeable_wep — 부식 가능 무기 판정
// [v2.11.0] wield.c:60-62 이식
// =============================================================================

/// 부식 가능 무기인지 판정 (원본 erodeable_wep 매크로)
/// 무기 클래스, 무기 도구, 쇠구슬, 쇠사슬이면 true
pub fn erodeable_wep(item: &WieldItemInfo) -> bool {
    item.item_class == WieldItemClass::Weapon
        || item.is_weptool
        || item.is_heavy_iron_ball
        || item.is_iron_chain
}

// =============================================================================
// will_weld — 무기 용접 여부 판정
// [v2.11.0] wield.c:65-66 이식
// =============================================================================

/// 저주된 무기가 손에 용접될지 판정 (원본 will_weld 매크로)
/// 저주 AND (부식 가능 무기 OR 깡통 따개)
pub fn will_weld(item: &WieldItemInfo) -> bool {
    item.cursed && (erodeable_wep(item) || item.is_tin_opener)
}

// =============================================================================
// welded_result — 주 무기 용접 상태 확인
// [v2.11.0] wield.c:857-866 이식
// =============================================================================

/// 현재 장비된 주 무기가 용접 상태인지 확인 (원본 welded)
/// is_wielded: 해당 아이템이 현재 주 무기인지
pub fn welded_result(item: &WieldItemInfo, is_wielded: bool) -> bool {
    is_wielded && will_weld(item)
}

// =============================================================================
// mwelded_result — 몬스터 무기 용접 확인
// [v2.11.0] wield.c:882-890 이식
// =============================================================================

/// 몬스터가 장비한 무기가 용접 상태인지 확인 (원본 mwelded)
pub fn mwelded_result(item: &WieldItemInfo, is_wielded_by_monster: bool) -> bool {
    is_wielded_by_monster && will_weld(item)
}

// =============================================================================
// unweapon_check — 장비 시 비무기 판정
// [v2.11.0] wield.c:108-114 이식
// =============================================================================

/// 장비된 아이템이 "비무기"인지 판정 (원본 unweapon 설정 로직)
/// 비무기면 "bashing" 메시지 표시 대상
/// has_steed: 탈것(말) 보유 여부
pub fn unweapon_check(item: &WieldItemInfo, has_steed: bool) -> bool {
    if item.item_class == WieldItemClass::Weapon {
        // 무기 클래스: 발사대, 탄약, 투척무기, 비탈것 장대 → 비무기
        item.is_launcher || item.is_ammo || item.is_missile || (item.is_pole && !has_steed)
    } else {
        // 비무기 클래스: 무기도구/젖은수건 아니면 비무기
        !item.is_weptool && !item.is_wet_towel
    }
}

// =============================================================================
// cant_wield_corpse_result — 코카트리스 시체 맨손 장비 판정
// [v2.11.0] wield.c:117-134 이식
// =============================================================================

/// 코카트리스 시체 맨손 장비 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpseWieldResult {
    /// 안전하게 장비 가능 (장갑 착용 or 석화 저항 or 비코카트리스)
    Safe,
    /// 석화 위험! (맨손 + 코카트리스 시체 + 석화 저항 없음)
    Petrification,
}

/// 코카트리스 시체를 맨손으로 장비할 때 석화 위험 판정 (원본 cant_wield_corpse)
/// wearing_gloves: 장갑 착용 여부
/// stone_resistance: 석화 저항 보유 여부
pub fn cant_wield_corpse_result(
    item: &WieldItemInfo,
    wearing_gloves: bool,
    stone_resistance: bool,
) -> CorpseWieldResult {
    // 장갑 착용, 비코카트리스, 석화 저항 → 안전
    if wearing_gloves || !item.is_cockatrice_corpse || stone_resistance {
        CorpseWieldResult::Safe
    } else {
        CorpseWieldResult::Petrification
    }
}

// =============================================================================
// can_twoweapon_result — 쌍수 전투 가능 판정
// [v2.11.0] wield.c:598-636 이식
// =============================================================================

/// 쌍수 전투 불가 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TwoWeaponFail {
    /// 변이 형태가 쌍수 불가
    PolymorphedCantUse,
    /// 역할이 쌍수 불가
    RoleCantUse,
    /// 주 무기 없음
    NoMainWeapon,
    /// 보조 무기 없음
    NoSecondWeapon,
    /// 주 무기가 무기가 아님
    MainNotWeapon,
    /// 보조 무기가 무기가 아님
    SecondNotWeapon,
    /// 주 무기가 양손 무기
    MainBimanual,
    /// 보조 무기가 양손 무기
    SecondBimanual,
    /// 방패 착용 중
    WearingShield,
    /// 보조 무기가 아티팩트 (저항)
    SecondIsArtifact,
    /// 보조 무기가 저주/미끄러움
    SecondCursedOrSlippery,
}

/// 쌍수 전투 가능 판정 (원본 can_twoweapon)
/// 성공 시 Ok(()), 실패 시 Err(사유)
pub fn can_twoweapon_result(
    main_weapon: Option<&WieldItemInfo>,
    swap_weapon: Option<&WieldItemInfo>,
    could_twoweapon_form: bool,
    is_polymorphed: bool,
    wearing_shield: bool,
    is_glib: bool,
) -> Result<(), TwoWeaponFail> {
    // 변이/역할 제한
    if !could_twoweapon_form {
        return Err(if is_polymorphed {
            TwoWeaponFail::PolymorphedCantUse
        } else {
            TwoWeaponFail::RoleCantUse
        });
    }

    // 무기 미장비
    let main = match main_weapon {
        Some(w) => w,
        None => return Err(TwoWeaponFail::NoMainWeapon),
    };
    let swap = match swap_weapon {
        Some(w) => w,
        None => return Err(TwoWeaponFail::NoSecondWeapon),
    };

    // 무기 클래스 확인 (무기 도구도 허용)
    let main_is_weapon = main.item_class == WieldItemClass::Weapon || main.is_weptool;
    let swap_is_weapon = swap.item_class == WieldItemClass::Weapon || swap.is_weptool;

    if !main_is_weapon {
        return Err(TwoWeaponFail::MainNotWeapon);
    }
    if !swap_is_weapon {
        return Err(TwoWeaponFail::SecondNotWeapon);
    }

    // 양손 무기 불가
    if main.is_bimanual {
        return Err(TwoWeaponFail::MainBimanual);
    }
    if swap.is_bimanual {
        return Err(TwoWeaponFail::SecondBimanual);
    }

    // 방패 착용 시 불가
    if wearing_shield {
        return Err(TwoWeaponFail::WearingShield);
    }

    // 보조 무기가 아티팩트면 저항
    if swap.is_artifact {
        return Err(TwoWeaponFail::SecondIsArtifact);
    }

    // 미끄러움 또는 저주
    if is_glib || swap.cursed {
        return Err(TwoWeaponFail::SecondCursedOrSlippery);
    }

    Ok(())
}

// =============================================================================
// ready_weapon_result — 무기 장비 결과 계산
// [v2.11.0] wield.c:136-225 이식
// =============================================================================

/// 무기 장비 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadyWeaponResult {
    /// 빈손으로 변경 (이전에 무기 있었으면 턴 소모)
    BareHands { used_turn: bool },
    /// 코카트리스 시체 석화 (턴 소모)
    Petrified,
    /// 방패 + 양손 무기 → 불가 (턴 미소모)
    ShieldBlocksBimanual,
    /// 용접됨 (저주+무기 → 손에 달라붙음, 턴 소모)
    Welded,
    /// 정상 장비 (턴 소모)
    Wielded {
        /// 장비된 아이템이 비무기인지
        is_unweapon: bool,
        /// 아티팩트 빛 시작 여부
        starts_artifact_light: bool,
        /// 상점 경고 메시지 필요 여부
        shopkeeper_warning: bool,
    },
}

/// 무기 장비 결과 계산 (원본 ready_weapon)
/// has_weapon_currently: 현재 무기 소지 여부
/// wearing_shield: 방패 착용 여부
/// wearing_gloves: 장갑 착용 여부
/// stone_resistance: 석화 저항
/// has_steed: 탈것 보유
/// in_shop: 상점 내 여부
pub fn ready_weapon_result(
    weapon: Option<&WieldItemInfo>,
    has_weapon_currently: bool,
    wearing_shield: bool,
    wearing_gloves: bool,
    stone_resistance: bool,
    has_steed: bool,
    in_shop: bool,
) -> ReadyWeaponResult {
    match weapon {
        None => {
            // 빈손으로 변경
            ReadyWeaponResult::BareHands {
                used_turn: has_weapon_currently,
            }
        }
        Some(wep) => {
            // 코카트리스 시체 체크
            if wep.is_cockatrice_corpse
                && cant_wield_corpse_result(wep, wearing_gloves, stone_resistance)
                    == CorpseWieldResult::Petrification
            {
                return ReadyWeaponResult::Petrified;
            }

            // 방패 + 양손 무기 불가
            if wearing_shield && wep.is_bimanual {
                return ReadyWeaponResult::ShieldBlocksBimanual;
            }

            // 용접 판정
            if will_weld(wep) {
                return ReadyWeaponResult::Welded;
            }

            // 정상 장비
            let is_unweapon = unweapon_check(wep, has_steed);
            let starts_artifact_light = wep.is_artifact; // 간소화: 아티팩트면 빛 시작 가능
            let shopkeeper_warning = wep.unpaid && in_shop;

            ReadyWeaponResult::Wielded {
                is_unweapon,
                starts_artifact_light,
                shopkeeper_warning,
            }
        }
    }
}

// =============================================================================
// wield_tool_check — 도구 장비 사전 검증
// [v2.11.0] wield.c:518-596 이식
// =============================================================================

/// 도구 장비 불가 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WieldToolFail {
    /// 이미 장비 중
    AlreadyWielded,
    /// 방어구/악세서리로 착용 중
    WornAsArmor,
    /// 현재 무기가 용접됨
    CurrentWeaponWelded,
    /// 현재 형태로 들 수 없음
    CantHold,
    /// 방패 + 양손 도구
    ShieldBlocksBimanual,
}

/// 도구를 무기로 장비할 수 있는지 사전 검증 (원본 wield_tool)
/// current_weapon_welded: 현재 주 무기가 용접 상태인지
/// cant_wield_form: 현재 형태가 무기를 들 수 없는지
pub fn wield_tool_check(
    tool: &WieldItemInfo,
    is_already_wielded: bool,
    current_weapon_welded: bool,
    cant_wield_form: bool,
    wearing_shield: bool,
) -> Result<(), WieldToolFail> {
    if is_already_wielded {
        return Err(WieldToolFail::AlreadyWielded);
    }
    if tool.worn_as_armor_or_accessory {
        return Err(WieldToolFail::WornAsArmor);
    }
    if current_weapon_welded {
        return Err(WieldToolFail::CurrentWeaponWelded);
    }
    if cant_wield_form {
        return Err(WieldToolFail::CantHold);
    }
    if wearing_shield && tool.is_bimanual {
        return Err(WieldToolFail::ShieldBlocksBimanual);
    }
    Ok(())
}

// =============================================================================
// chwepon_result — 마법 무기 강화/약화 효과
// [v2.11.0] wield.c:722-855 이식
// =============================================================================

/// 마법 무기 강화/약화 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChweponResult {
    /// 무기 미장비 또는 비무기 → 효과 없음 (손 떨림/간지러움)
    NoWeapon {
        /// 저주 해제 가능 (용접된 깡통 따개)
        uncurse_welded: bool,
    },
    /// 벌레 이빨 → 크리스 나이프 변환 (amount >= 0)
    WormToothToCrysknife {
        /// 여러 개가 융합되는지
        multiple_fuse: bool,
    },
    /// 크리스 나이프 → 벌레 이빨 변환 (amount < 0)
    CrysknifeToWormTooth {
        /// 여러 개가 융합되는지
        multiple_fuse: bool,
    },
    /// 아티팩트가 음의 강화 저항 (약간 빛남)
    ArtifactResists,
    /// 강화 상한/하한 초과 → 무기 증발
    Evaporates,
    /// 정상 강화/약화
    Enchanted {
        /// 새 강화치
        new_spe: i32,
        /// 저주 해제 여부 (양의 강화 시)
        uncursed: bool,
        /// Magicbane 특수 반응 (spe >= 0일 때)
        magicbane_reaction: bool,
        /// 엘프 무기 진동 경고
        elven_vibration: bool,
    },
}

/// 마법 무기 강화/약화 결과 계산 (원본 chwepon)
/// amount: 강화량 (양수=강화, 음수=약화)
/// ART_MAGICBANE: 매직베인 아티팩트 id (일반적으로 1)
pub fn chwepon_result(
    weapon: Option<&WieldItemInfo>,
    amount: i32,
    rng: &mut NetHackRng,
) -> ChweponResult {
    let wep = match weapon {
        Some(w) => w,
        None => {
            return ChweponResult::NoWeapon {
                uncurse_welded: false,
            };
        }
    };

    // 비무기 체크 (무기 클래스도, 무기 도구도 아님)
    if wep.item_class != WieldItemClass::Weapon && !wep.is_weptool {
        // 저주된 깡통 따개 → amount >= 0이면 저주 해제
        let uncurse = amount >= 0 && will_weld(wep);
        return ChweponResult::NoWeapon {
            uncurse_welded: uncurse,
        };
    }

    // 벌레 이빨 → 크리스 나이프 (amount >= 0)
    if wep.is_worm_tooth && amount >= 0 {
        return ChweponResult::WormToothToCrysknife {
            multiple_fuse: wep.quantity > 1,
        };
    }

    // 크리스 나이프 → 벌레 이빨 (amount < 0)
    if wep.is_crysknife && amount < 0 {
        return ChweponResult::CrysknifeToWormTooth {
            multiple_fuse: wep.quantity > 1,
        };
    }

    // 아티팩트 음의 강화 저항 (금지된 이름 가진 아티팩트)
    if amount < 0 && wep.is_artifact {
        // 원본: restrict_name() — 간소화하여 아티팩트는 저항
        return ChweponResult::ArtifactResists;
    }

    // 강화 상한/하한 체크 → 증발
    // spe > 5이고 양의 강화, 또는 spe < -5이고 음의 강화 → 2/3 확률로 증발
    if ((wep.spe > 5 && amount >= 0) || (wep.spe < -5 && amount < 0)) && rng.rn2(3) != 0 {
        return ChweponResult::Evaporates;
    }

    // 정상 강화/약화
    let new_spe = wep.spe + amount;
    let uncursed = amount > 0 && wep.cursed;

    // Magicbane 특수 반응 (spe >= 0일 때)
    // 원본: ART_MAGICBANE 아티팩트 id 비교
    let magicbane_reaction = wep.artifact_id == Some(1) && new_spe >= 0;

    // 엘프 무기 진동 경고 (spe > 5이고 엘프 무기 or 아티팩트 or 1/7 확률)
    let elven_vibration =
        new_spe > 5 && (wep.is_elven_weapon || wep.is_artifact || rng.rn2(7) == 0);

    ChweponResult::Enchanted {
        new_spe,
        uncursed,
        magicbane_reaction,
        elven_vibration,
    }
}

// =============================================================================
// enchant_weapon_cap_check — 강화 상한 판정
// [v2.11.0] wield.c:805-817 이식 (독립 유틸)
// =============================================================================

/// 무기 강화 상한 도달 시 증발 확률 판정 (원본 chwepon 내부 로직)
/// 강화치가 +-5를 넘으면 2/3 확률로 증발
pub fn enchant_weapon_cap_check(current_spe: i32, amount: i32, rng: &mut NetHackRng) -> bool {
    if (current_spe > 5 && amount >= 0) || (current_spe < -5 && amount < 0) {
        rng.rn2(3) != 0 // 2/3 확률로 증발
    } else {
        false
    }
}

// =============================================================================
// wield_swap_result — 무기 교환 결과
// [v2.11.0] wield.c:309-350 이식
// =============================================================================

/// 무기 교환 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwapWeaponResult {
    /// 현재 형태로 무기를 다룰 수 없음
    CantWield,
    /// 현재 무기가 용접됨
    Welded,
    /// 교환 성공 후 장비 결과
    Swapped {
        /// 보조 무기 장비 결과
        ready_result: ReadyWeaponResult,
    },
}

/// 무기 교환 사전 판정 (원본 doswapweapon 로직)
/// cant_wield_form: 현재 형태가 무기를 다룰 수 없는지
/// current_weapon_welded: 현재 주 무기가 용접 상태인지
pub fn wield_swap_check(
    cant_wield_form: bool,
    current_weapon_welded: bool,
) -> Result<(), SwapWeaponResult> {
    if cant_wield_form {
        return Err(SwapWeaponResult::CantWield);
    }
    if current_weapon_welded {
        return Err(SwapWeaponResult::Welded);
    }
    Ok(())
}

// =============================================================================
// 도우미 함수: WieldItemInfo 기본 생성
// =============================================================================

impl WieldItemInfo {
    /// 테스트용 기본 무기 생성
    pub fn test_weapon(name: &str) -> Self {
        Self {
            name: name.to_string(),
            item_class: WieldItemClass::Weapon,
            cursed: false,
            is_weptool: false,
            is_bimanual: false,
            is_launcher: false,
            is_ammo: false,
            is_missile: false,
            is_pole: false,
            is_wet_towel: false,
            is_artifact: false,
            artifact_id: None,
            is_cockatrice_corpse: false,
            is_heavy_iron_ball: false,
            is_iron_chain: false,
            is_tin_opener: false,
            is_worm_tooth: false,
            is_crysknife: false,
            is_elven_weapon: false,
            spe: 0,
            quantity: 1,
            erodeproof: false,
            unpaid: false,
            worn_as_armor_or_accessory: false,
            is_silver: false,
        }
    }

    /// 테스트용 도구 생성
    pub fn test_tool(name: &str) -> Self {
        let mut item = Self::test_weapon(name);
        item.item_class = WieldItemClass::Tool;
        item.is_weptool = true;
        item
    }
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erodeable_wep() {
        // 무기 클래스 → true
        let sword = WieldItemInfo::test_weapon("long sword");
        assert!(erodeable_wep(&sword));

        // 무기 도구 → true
        let pick = WieldItemInfo::test_tool("pick-axe");
        assert!(erodeable_wep(&pick));

        // 쇠구슬 → true
        let mut ball = WieldItemInfo::test_weapon("heavy iron ball");
        ball.item_class = WieldItemClass::Other;
        ball.is_heavy_iron_ball = true;
        assert!(erodeable_wep(&ball));

        // 일반 아이템 → false
        let mut food = WieldItemInfo::test_weapon("apple");
        food.item_class = WieldItemClass::Food;
        assert!(!erodeable_wep(&food));
    }

    #[test]
    fn test_will_weld() {
        // 저주 + 무기 → 용접
        let mut sword = WieldItemInfo::test_weapon("long sword");
        sword.cursed = true;
        assert!(will_weld(&sword));

        // 비저주 무기 → 용접 안 됨
        let sword2 = WieldItemInfo::test_weapon("long sword");
        assert!(!will_weld(&sword2));

        // 저주 + 깡통 따개 → 용접
        let mut opener = WieldItemInfo::test_weapon("tin opener");
        opener.item_class = WieldItemClass::Other;
        opener.is_tin_opener = true;
        opener.cursed = true;
        assert!(will_weld(&opener));

        // 저주 + 비무기/비도구 → 용접 안 됨
        let mut food = WieldItemInfo::test_weapon("tripe ration");
        food.item_class = WieldItemClass::Food;
        food.cursed = true;
        assert!(!will_weld(&food));
    }

    #[test]
    fn test_welded_result() {
        let mut sword = WieldItemInfo::test_weapon("long sword");
        sword.cursed = true;
        // 장비 중 + 용접 조건 → true
        assert!(welded_result(&sword, true));
        // 미장비 → false
        assert!(!welded_result(&sword, false));
    }

    #[test]
    fn test_mwelded_result() {
        let mut sword = WieldItemInfo::test_weapon("long sword");
        sword.cursed = true;
        assert!(mwelded_result(&sword, true));
        assert!(!mwelded_result(&sword, false));
    }

    #[test]
    fn test_unweapon_check() {
        // 일반 무기 → 비무기 아님
        let sword = WieldItemInfo::test_weapon("long sword");
        assert!(!unweapon_check(&sword, false));

        // 발사대 → 비무기
        let mut bow = WieldItemInfo::test_weapon("bow");
        bow.is_launcher = true;
        assert!(unweapon_check(&bow, false));

        // 탄약 → 비무기
        let mut arrow = WieldItemInfo::test_weapon("arrow");
        arrow.is_ammo = true;
        assert!(unweapon_check(&arrow, false));

        // 장대 + 탈것 → 비무기 아님
        let mut lance = WieldItemInfo::test_weapon("lance");
        lance.is_pole = true;
        assert!(!unweapon_check(&lance, true));

        // 장대 + 비탈것 → 비무기
        assert!(unweapon_check(&lance, false));
    }

    #[test]
    fn test_cant_wield_corpse() {
        let mut corpse = WieldItemInfo::test_weapon("cockatrice corpse");
        corpse.item_class = WieldItemClass::Food;
        corpse.is_cockatrice_corpse = true;

        // 맨손 + 코카트리스 → 석화
        assert_eq!(
            cant_wield_corpse_result(&corpse, false, false),
            CorpseWieldResult::Petrification
        );

        // 장갑 착용 → 안전
        assert_eq!(
            cant_wield_corpse_result(&corpse, true, false),
            CorpseWieldResult::Safe
        );

        // 석화 저항 → 안전
        assert_eq!(
            cant_wield_corpse_result(&corpse, false, true),
            CorpseWieldResult::Safe
        );
    }

    #[test]
    fn test_can_twoweapon_no_main() {
        let swap = WieldItemInfo::test_weapon("dagger");
        assert_eq!(
            can_twoweapon_result(None, Some(&swap), true, false, false, false),
            Err(TwoWeaponFail::NoMainWeapon)
        );
    }

    #[test]
    fn test_can_twoweapon_bimanual() {
        let mut main = WieldItemInfo::test_weapon("two-handed sword");
        main.is_bimanual = true;
        let swap = WieldItemInfo::test_weapon("dagger");
        assert_eq!(
            can_twoweapon_result(Some(&main), Some(&swap), true, false, false, false),
            Err(TwoWeaponFail::MainBimanual)
        );
    }

    #[test]
    fn test_can_twoweapon_shield() {
        let main = WieldItemInfo::test_weapon("long sword");
        let swap = WieldItemInfo::test_weapon("dagger");
        assert_eq!(
            can_twoweapon_result(Some(&main), Some(&swap), true, false, true, false),
            Err(TwoWeaponFail::WearingShield)
        );
    }

    #[test]
    fn test_can_twoweapon_success() {
        let main = WieldItemInfo::test_weapon("long sword");
        let swap = WieldItemInfo::test_weapon("dagger");
        assert_eq!(
            can_twoweapon_result(Some(&main), Some(&swap), true, false, false, false),
            Ok(())
        );
    }

    #[test]
    fn test_can_twoweapon_artifact_swap() {
        let main = WieldItemInfo::test_weapon("long sword");
        let mut swap = WieldItemInfo::test_weapon("Excalibur");
        swap.is_artifact = true;
        assert_eq!(
            can_twoweapon_result(Some(&main), Some(&swap), true, false, false, false),
            Err(TwoWeaponFail::SecondIsArtifact)
        );
    }

    #[test]
    fn test_ready_weapon_empty_hands() {
        // 현재 무기 있음 → 빈손으로 → 턴 소모
        assert_eq!(
            ready_weapon_result(None, true, false, false, false, false, false),
            ReadyWeaponResult::BareHands { used_turn: true }
        );
        // 현재 무기 없음 → 빈손 → 턴 미소모
        assert_eq!(
            ready_weapon_result(None, false, false, false, false, false, false),
            ReadyWeaponResult::BareHands { used_turn: false }
        );
    }

    #[test]
    fn test_ready_weapon_petrified() {
        let mut corpse = WieldItemInfo::test_weapon("cockatrice corpse");
        corpse.item_class = WieldItemClass::Food;
        corpse.is_cockatrice_corpse = true;
        assert_eq!(
            ready_weapon_result(Some(&corpse), false, false, false, false, false, false),
            ReadyWeaponResult::Petrified
        );
    }

    #[test]
    fn test_ready_weapon_shield_bimanual() {
        let mut sword = WieldItemInfo::test_weapon("two-handed sword");
        sword.is_bimanual = true;
        assert_eq!(
            ready_weapon_result(Some(&sword), false, true, false, false, false, false),
            ReadyWeaponResult::ShieldBlocksBimanual
        );
    }

    #[test]
    fn test_ready_weapon_welded() {
        let mut sword = WieldItemInfo::test_weapon("long sword");
        sword.cursed = true;
        assert_eq!(
            ready_weapon_result(Some(&sword), false, false, false, false, false, false),
            ReadyWeaponResult::Welded
        );
    }

    #[test]
    fn test_ready_weapon_normal() {
        let sword = WieldItemInfo::test_weapon("long sword");
        match ready_weapon_result(Some(&sword), false, false, false, false, false, false) {
            ReadyWeaponResult::Wielded {
                is_unweapon,
                starts_artifact_light: _,
                shopkeeper_warning: _,
            } => {
                assert!(!is_unweapon);
            }
            other => panic!("예상 외 결과: {:?}", other),
        }
    }

    #[test]
    fn test_chwepon_worm_tooth() {
        let mut rng = NetHackRng::new(42);
        let mut tooth = WieldItemInfo::test_weapon("worm tooth");
        tooth.is_worm_tooth = true;
        match chwepon_result(Some(&tooth), 1, &mut rng) {
            ChweponResult::WormToothToCrysknife { multiple_fuse } => {
                assert!(!multiple_fuse);
            }
            other => panic!("예상 외: {:?}", other),
        }
    }

    #[test]
    fn test_chwepon_normal_enchant() {
        let mut rng = NetHackRng::new(42);
        let sword = WieldItemInfo::test_weapon("long sword");
        match chwepon_result(Some(&sword), 1, &mut rng) {
            ChweponResult::Enchanted {
                new_spe,
                uncursed,
                magicbane_reaction: _,
                elven_vibration: _,
            } => {
                assert_eq!(new_spe, 1);
                assert!(!uncursed); // 비저주 무기이므로
            }
            other => panic!("예상 외: {:?}", other),
        }
    }

    #[test]
    fn test_chwepon_no_weapon() {
        let mut rng = NetHackRng::new(42);
        match chwepon_result(None, 1, &mut rng) {
            ChweponResult::NoWeapon {
                uncurse_welded: false,
            } => {}
            other => panic!("예상 외: {:?}", other),
        }
    }

    #[test]
    fn test_wield_tool_check_success() {
        let pick = WieldItemInfo::test_tool("pick-axe");
        assert!(wield_tool_check(&pick, false, false, false, false).is_ok());
    }

    #[test]
    fn test_wield_tool_check_welded() {
        let pick = WieldItemInfo::test_tool("pick-axe");
        assert_eq!(
            wield_tool_check(&pick, false, true, false, false),
            Err(WieldToolFail::CurrentWeaponWelded)
        );
    }

    #[test]
    fn test_swap_check() {
        // 정상
        assert!(wield_swap_check(false, false).is_ok());
        // 무기 다룰 수 없음
        assert_eq!(
            wield_swap_check(true, false),
            Err(SwapWeaponResult::CantWield)
        );
        // 용접됨
        assert_eq!(wield_swap_check(false, true), Err(SwapWeaponResult::Welded));
    }
}

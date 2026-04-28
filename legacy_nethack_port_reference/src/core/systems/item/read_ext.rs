// read_ext.rs — read.c 핵심 로직 순수 결과 패턴 이식
// [v2.12.0] 신규 생성: 스크롤/충전/강화/부식방지/혼합 등 12개 함수
// 원본: NetHack 3.6.7 src/read.c (2,653줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 충전 한계 (지팡이 7회 폭발)
const RECHARGE_LIMIT: i32 = 7;

// ============================================================
// 열거형
// ============================================================

/// 축복/저주 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcsStatus {
    Blessed,
    Uncursed,
    Cursed,
}

/// 지팡이 타입 (충전 한계에 영향)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WandType {
    /// 소원의 지팡이 — 충전 한계 3
    Wishing,
    /// 방향 지정 지팡이 — 한계 8
    Directional,
    /// 무방향 지팡이 — 한계 15
    NonDirectional,
}

/// 충전 가능 도구 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeableToolType {
    BellOfOpening,
    MagicMarker,
    TinningKit,
    ExpensiveCamera,
    OilLamp,
    BrassLantern,
    CrystalBall,
    HornOfPlenty,
    BagOfTricks,
    CanOfGrease,
    MagicFlute,
    MagicHarp,
    FrostHorn,
    FireHorn,
    DrumOfEarthquake,
    Other,
}

/// 충전 가능 아이템 클래스
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeableClass {
    Wand,
    Ring { oc_charged: bool },
    Tool { oc_charged: bool, is_weptool: bool },
    Other,
}

/// 지팡이 충전 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WandRechargeResult {
    /// 폭발! (과충전)
    Explode { damage_dice: i32 },
    /// 저주로 spe를 0으로 만듦
    StripSpe,
    /// 정상 충전 성공
    Charged { new_spe: i32, glow_blue: bool },
}

/// 반지 충전 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RingRechargeResult {
    /// 폭발하여 파괴
    Explode { damage: i32 },
    /// spe 변화 (시계/반시계 회전)
    Spin { delta: i32 },
}

/// 도구 충전 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolRechargeResult {
    /// 저주로 spe 제거
    StripSpe,
    /// 마커 이미 충전됨(건조)
    MarkerDriedOut,
    /// 정상 충전
    Charged { new_spe: i32, glow_blue: bool },
    /// 충전 불가
    NotChargeable,
}

/// 방어구 강화 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnchantArmorResult {
    /// 방어구 없음 — 이상한 느낌
    NoArmor,
    /// 혼란 상태 — 부식방지 전환
    ConfusedErodeproof { new_erodeproof: bool },
    /// 용 비늘 → 용 비늘 갑옷 변환
    DragonScaleUpgrade { spe_bonus: i32, do_bless: bool },
    /// 과강화 폭발(증발)
    OverenchantExplode,
    /// 정상 강화
    Enchanted { delta: i32 },
}

/// 무기 강화 결과 (read.c SCR_ENCHANT_WEAPON 분기)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnchantWeaponResult {
    /// 혼란 상태 — 부식방지 전환
    ConfusedErodeproof { new_erodeproof: bool },
    /// 정상 강화 (chwepon 결과에 위임)
    EnchantDelta(i32),
    /// 무기 없음 (sobj consumed)
    NoWeapon,
}

/// T셔츠/앞치마 텍스트 목록
pub const TSHIRT_MSGS: &[&str] = &[
    "I explored the Dungeons of Doom and all I got was this lousy T-shirt!",
    "Is that Mjollnir in your pocket or are you just happy to see me?",
    "It's not the size of your sword, it's how #enhance'd you are with it.",
    "Madame Elvira's House O' Succubi Lifetime Customer",
    "Madame Elvira's House O' Succubi Employee of the Month",
    "Ludios Vault Guards Do It In Small, Dark Rooms",
    "Yendor Military Soldiers Do It In Large Groups",
    "I survived Yendor Military Boot Camp",
    "Ludios Accounting School Intra-Mural Lacrosse Team",
    "Oracle(TM) Fountains 10th Annual Wet T-Shirt Contest",
    "Hey, black dragon!  Disintegrate THIS!",
    "I'm With Stupid -->",
    "Don't blame me, I voted for Izchak!",
    "Don't Panic",
    "Furinkan High School Athletic Dept.",
    "Hel-LOOO, Nurse!",
    "=^.^=",
    "100% goblin hair - do not wash",
    "Aberzombie and Fitch",
    "cK -- Cockatrice touches the Kop",
    "Don't ask me, I only adventure here",
    "Down with pants!",
    "d, your dog or a killer?",
    "FREE PUG AND NEWT!",
    "Go team ant!",
    "Got newt?",
    "Hello, my darlings!",
    "Hey!  Nymphs!  Steal This T-Shirt!",
    "I <3 Dungeon of Doom",
    "I <3 Maud",
    "I am a Valkyrie.  If you see me running, try to keep up.",
    "I am not a pack rat - I am a collector",
    "I bounced off a rubber tree",
    "Plunder Island Brimstone Beach Club",
    "If you can read this, I can hit you with my polearm",
    "I'm confused!",
    "I scored with the princess",
    "I want to live forever or die in the attempt.",
    "Lichen Park",
    "LOST IN THOUGHT - please send search party",
    "Meat is Mordor",
    "Minetown Better Business Bureau",
    "Minetown Watch",
    "Ms. Palm's House of Negotiable Affection -- A Very Reputable House Of Disrepute",
    "Protection Racketeer",
    "Real men love Crom",
    "Somebody stole my Mojo!",
    "The Hellhound Gang",
    "The Werewolves",
    "They Might Be Storm Giants",
    "Weapons don't kill people, I kill people",
    "White Zombie",
    "You're killing me!",
    "Anhur State University - Home of the Fighting Fire Ants!",
    "FREE HUGS",
    "Serial Ascender",
    "Real men are valkyries",
    "Young Men's Cavedigging Association",
    "Occupy Fort Ludios",
    "I couldn't afford this T-shirt so I stole it!",
    "Mind flayers suck",
    "I'm not wearing any pants",
    "Down with the living!",
    "Pudding farmer",
    "Vegetarian",
    "Hello, I'm War!",
    "It is better to light a candle than to curse the darkness",
    "It is easier to curse the darkness than to light a candle",
    "rock--paper--scissors--lizard--Spock!",
    "/Valar morghulis/ -- /Valar dohaeris/",
];

pub const APRON_MSGS: &[&str] = &[
    "Kiss the cook",
    "I'm making SCIENCE!",
    "Don't mess with the chef",
    "Don't make me poison you",
    "Gehennom's Kitchen",
    "Rat: The other white meat",
    "If you can't stand the heat, get out of Gehennom!",
    "If we weren't meant to eat animals, why are they made out of meat?",
    "If you don't like the food, I'll stab you",
];

pub const CARD_MSGS: &[&str] = &[
    "Leprechaun Gold Tru$t - Shamrock Card",
    "Magic Memory Vault Charge Card",
    "Larn National Bank",
    "First Bank of Omega",
    "Bank of Zork - Frobozz Magic Card",
    "Ankh-Morpork Merchant's Guild Barter Card",
    "Ankh-Morpork Thieves' Guild Unlimited Transaction Card",
    "Ransmannsby Moneylenders Association",
    "Bank of Gehennom - 99% Interest Card",
    "Yendorian Express - Copper Card",
    "Yendorian Express - Silver Card",
    "Yendorian Express - Gold Card",
    "Yendorian Express - Mithril Card",
    "Yendorian Express - Platinum Card",
];

// ============================================================
// 1. tshirt_text_index — T셔츠 텍스트 인덱스 결정
// ============================================================

/// T셔츠에 적힌 문구 인덱스를 o_id로 결정
/// 원본: read.c tshirt_text()
pub fn tshirt_text_index(o_id: u32) -> usize {
    (o_id as usize) % TSHIRT_MSGS.len()
}

/// 앞치마에 적힌 문구 인덱스를 o_id로 결정
/// 원본: read.c apron_text()
pub fn apron_text_index(o_id: u32) -> usize {
    (o_id as usize) % APRON_MSGS.len()
}

// ============================================================
// 2. credit_card_text — 신용카드 텍스트와 번호 생성
// ============================================================

/// 신용카드의 발급사 이름과 카드 번호를 생성
/// 원본: read.c doread() CREDIT_CARD 분기
pub fn credit_card_text(o_id: u32, is_artifact: bool) -> (&'static str, String) {
    let issuer = if is_artifact {
        CARD_MSGS[CARD_MSGS.len() - 1] // "Platinum Card"
    } else {
        CARD_MSGS[(o_id as usize) % (CARD_MSGS.len() - 1)]
    };

    // 카드 번호 생성 (원본 포맷 재현)
    let id = o_id as i64;
    let part1 = (id % 89) + 10;
    let part2 = id % 4;
    let part3 = ((id * 499) % 899999) + 100000;
    let part4 = id % 10;
    let part5 = if id % 3 == 0 { 1 } else { 0 };
    let part6 = (id * 7) % 10;
    let number = format!(
        "{}0{} {}{}1 0{}{}0",
        part1, part2, part3, part4, part5, part6
    );

    (issuer, number)
}

// ============================================================
// 3. is_chargeable — 충전 가능 여부 판정
// ============================================================

/// 아이템이 충전 가능한지 판정
/// 원본: read.c is_chargeable()
pub fn is_chargeable(class: ChargeableClass) -> bool {
    match class {
        ChargeableClass::Wand => true,
        ChargeableClass::Ring { oc_charged } => oc_charged,
        ChargeableClass::Tool {
            oc_charged,
            is_weptool,
        } => {
            if is_weptool {
                false // 무기 도구는 충전 불가
            } else {
                oc_charged
            }
        }
        ChargeableClass::Other => false,
    }
}

// ============================================================
// 4. recharge_wand_result — 지팡이 충전 결과 계산
// ============================================================

/// 지팡이 충전 결과 계산
/// 원본: read.c recharge() WAND_CLASS 분기
pub fn recharge_wand_result(
    wand_type: WandType,
    current_spe: i32,
    recharged_count: u32,
    bcs: BcsStatus,
    rng: &mut NetHackRng,
) -> WandRechargeResult {
    let lim = match wand_type {
        WandType::Wishing => 3,
        WandType::Directional => 8,
        WandType::NonDirectional => 15,
    };

    // 취소된 지팡이 복원 (-1 → 0)
    let effective_spe = if current_spe == -1 { 0 } else { current_spe };

    // 폭발 확률 검사 (이전 충전 횟수 기반)
    let n = recharged_count as i32;
    if n > 0 && (wand_type == WandType::Wishing || n * n * n > rng.rn2(343)) {
        return WandRechargeResult::Explode {
            damage_dice: rng.rnd(lim),
        };
    }

    // 저주 → spe 제거
    if bcs == BcsStatus::Cursed {
        return WandRechargeResult::StripSpe;
    }

    // 정상 충전
    let charge_target = if lim == 3 {
        3 // 소원의 지팡이
    } else {
        rng.rn1(5, lim + 1 - 5)
    };
    let charge_target = if bcs != BcsStatus::Blessed {
        rng.rnd(charge_target.max(1))
    } else {
        charge_target
    };

    let new_spe = if effective_spe < charge_target {
        charge_target
    } else {
        effective_spe + 1
    };

    // 소원의 지팡이 spe > 3이면 폭발
    if wand_type == WandType::Wishing && new_spe > 3 {
        return WandRechargeResult::Explode { damage_dice: 1 };
    }

    WandRechargeResult::Charged {
        new_spe,
        glow_blue: new_spe >= lim,
    }
}

// ============================================================
// 5. recharge_ring_result — 반지 충전 결과 계산
// ============================================================

/// 반지 충전 결과 계산
/// 원본: read.c recharge() RING_CLASS 분기
pub fn recharge_ring_result(
    current_spe: i32,
    bcs: BcsStatus,
    rng: &mut NetHackRng,
) -> RingRechargeResult {
    // 파괴 조건: spe > rn2(7) 또는 spe <= -5
    if current_spe > rng.rn2(7) || current_spe <= -5 {
        let damage = rng.rnd((3 * current_spe.abs()).max(1));
        return RingRechargeResult::Explode { damage };
    }

    let delta = match bcs {
        BcsStatus::Blessed => rng.rnd(3),
        BcsStatus::Cursed => -rng.rnd(2),
        BcsStatus::Uncursed => 1,
    };

    RingRechargeResult::Spin { delta }
}

// ============================================================
// 6. recharge_tool_result — 도구 충전 결과 계산
// ============================================================

/// 도구 충전 결과 계산
/// 원본: read.c recharge() TOOL_CLASS 분기
pub fn recharge_tool_result(
    tool_type: ChargeableToolType,
    current_spe: i32,
    recharged_count: u32,
    bcs: BcsStatus,
    rng: &mut NetHackRng,
) -> ToolRechargeResult {
    match tool_type {
        ChargeableToolType::BellOfOpening => {
            if bcs == BcsStatus::Cursed {
                ToolRechargeResult::StripSpe
            } else {
                let delta = if bcs == BcsStatus::Blessed {
                    rng.rnd(3)
                } else {
                    1
                };
                let new_spe = (current_spe + delta).min(5);
                ToolRechargeResult::Charged {
                    new_spe,
                    glow_blue: bcs == BcsStatus::Blessed,
                }
            }
        }
        ChargeableToolType::MagicMarker => {
            if bcs == BcsStatus::Cursed {
                ToolRechargeResult::StripSpe
            } else if recharged_count > 0 {
                // 이미 충전된 마커 — 영구 건조
                ToolRechargeResult::MarkerDriedOut
            } else if bcs == BcsStatus::Blessed {
                let n = rng.rn1(16, 15); // 15~30
                let new_spe = if current_spe + n <= 50 {
                    50
                } else if current_spe + n <= 75 {
                    75
                } else {
                    (current_spe + n).min(127)
                };
                ToolRechargeResult::Charged {
                    new_spe,
                    glow_blue: true,
                }
            } else {
                let n = rng.rn1(11, 10); // 10~20
                let new_spe = if current_spe + n <= 50 {
                    50
                } else {
                    (current_spe + n).min(127)
                };
                ToolRechargeResult::Charged {
                    new_spe,
                    glow_blue: false,
                }
            }
        }
        ChargeableToolType::TinningKit | ChargeableToolType::ExpensiveCamera => {
            if bcs == BcsStatus::Cursed {
                ToolRechargeResult::StripSpe
            } else if bcs == BcsStatus::Blessed {
                let n = rng.rn1(16, 15);
                let new_spe = if current_spe + n <= 50 {
                    50
                } else if current_spe + n <= 75 {
                    75
                } else {
                    (current_spe + n).min(127)
                };
                ToolRechargeResult::Charged {
                    new_spe,
                    glow_blue: true,
                }
            } else {
                let n = rng.rn1(11, 10);
                let new_spe = if current_spe + n <= 50 {
                    50
                } else {
                    (current_spe + n).min(127)
                };
                ToolRechargeResult::Charged {
                    new_spe,
                    glow_blue: false,
                }
            }
        }
        ChargeableToolType::CrystalBall => {
            if bcs == BcsStatus::Cursed {
                ToolRechargeResult::StripSpe
            } else if bcs == BcsStatus::Blessed {
                ToolRechargeResult::Charged {
                    new_spe: 6,
                    glow_blue: true,
                }
            } else if current_spe < 5 {
                ToolRechargeResult::Charged {
                    new_spe: current_spe + 1,
                    glow_blue: false,
                }
            } else {
                ToolRechargeResult::NotChargeable // "nothing happens"
            }
        }
        ChargeableToolType::HornOfPlenty
        | ChargeableToolType::BagOfTricks
        | ChargeableToolType::CanOfGrease => {
            if bcs == BcsStatus::Cursed {
                ToolRechargeResult::StripSpe
            } else if bcs == BcsStatus::Blessed {
                let delta = if current_spe <= 10 {
                    rng.rn1(10, 6)
                } else {
                    rng.rn1(5, 6)
                };
                let new_spe = (current_spe + delta).min(50);
                ToolRechargeResult::Charged {
                    new_spe,
                    glow_blue: true,
                }
            } else {
                let delta = rng.rnd(5);
                let new_spe = (current_spe + delta).min(50);
                ToolRechargeResult::Charged {
                    new_spe,
                    glow_blue: false,
                }
            }
        }
        ChargeableToolType::MagicFlute
        | ChargeableToolType::MagicHarp
        | ChargeableToolType::FrostHorn
        | ChargeableToolType::FireHorn
        | ChargeableToolType::DrumOfEarthquake => {
            if bcs == BcsStatus::Cursed {
                ToolRechargeResult::StripSpe
            } else if bcs == BcsStatus::Blessed {
                let delta = rng.d(2, 4);
                let new_spe = (current_spe + delta).min(20);
                ToolRechargeResult::Charged {
                    new_spe,
                    glow_blue: true,
                }
            } else {
                let delta = rng.rnd(4);
                let new_spe = (current_spe + delta).min(20);
                ToolRechargeResult::Charged {
                    new_spe,
                    glow_blue: false,
                }
            }
        }
        ChargeableToolType::OilLamp | ChargeableToolType::BrassLantern => {
            if bcs == BcsStatus::Cursed {
                ToolRechargeResult::StripSpe
            } else if bcs == BcsStatus::Blessed {
                ToolRechargeResult::Charged {
                    new_spe: 1,
                    glow_blue: true,
                }
            } else {
                ToolRechargeResult::Charged {
                    new_spe: 1,
                    glow_blue: false,
                }
            }
        }
        ChargeableToolType::Other => ToolRechargeResult::NotChargeable,
    }
}

// ============================================================
// 7. enchant_armor_calc — 방어구 강화 보너스 계산
// ============================================================

/// 방어구 강화 보너스 계산
/// 원본: seffects() SCR_ENCHANT_ARMOR 분기
pub fn enchant_armor_calc(
    current_spe: i32,
    is_dragon_scales: bool,
    is_special_armor: bool,
    bcs: BcsStatus,
    rng: &mut NetHackRng,
) -> EnchantArmorResult {
    let scursed = bcs == BcsStatus::Cursed;
    let sblessed = bcs == BcsStatus::Blessed;

    // 과강화 검사: s = scursed ? -spe : spe
    let s_check = if scursed { -current_spe } else { current_spe };
    let limit = if is_special_armor { 5 } else { 3 };
    if s_check > limit && rng.rn2(s_check) != 0 {
        return EnchantArmorResult::OverenchantExplode;
    }

    // 실제 강화량 계산
    let delta = if scursed {
        -1
    } else if current_spe >= 9 {
        if rng.rn2(current_spe) == 0 {
            1
        } else {
            0
        }
    } else if sblessed {
        rng.rnd((3 - current_spe / 3).max(1))
    } else {
        1
    };

    // 용 비늘 업그레이드
    if delta >= 0 && is_dragon_scales {
        return EnchantArmorResult::DragonScaleUpgrade {
            spe_bonus: if sblessed { 1 } else { 0 },
            do_bless: sblessed,
        };
    }

    EnchantArmorResult::Enchanted { delta }
}

// ============================================================
// 8. enchant_weapon_calc — 무기 강화 보너스 계산
// ============================================================

/// 무기 강화 보너스 계산
/// 원본: seffects() SCR_ENCHANT_WEAPON 분기 → chwepon 호출 인자
pub fn enchant_weapon_calc(
    current_spe: i32,
    has_weapon: bool,
    bcs: BcsStatus,
    rng: &mut NetHackRng,
) -> EnchantWeaponResult {
    if !has_weapon {
        return EnchantWeaponResult::EnchantDelta(1); // 비무장 시 +1
    }

    let delta = if bcs == BcsStatus::Cursed {
        -1
    } else if current_spe >= 9 {
        if rng.rn2(current_spe) == 0 {
            1
        } else {
            0
        }
    } else if bcs == BcsStatus::Blessed {
        rng.rnd((3 - current_spe / 3).max(1))
    } else {
        1
    };

    EnchantWeaponResult::EnchantDelta(delta)
}

// ============================================================
// 9. forget_percentage_calc — 망각 스크롤의 망각 비율 계산
// ============================================================

/// 망각 스크롤 효과의 레벨/아이템 망각 비율 결정
/// 원본: read.c forget() 내부 rn2(3) → rn2(25) 로직
pub fn forget_percentage_calc(rng: &mut NetHackRng) -> (Option<i32>, Option<i32>) {
    // 1/3 확률로 레벨 망각
    let forget_levels = if rng.rn2(3) == 0 {
        Some(rng.rn2(25))
    } else {
        None
    };
    // 1/3 확률로 아이템 망각
    let forget_objects = if rng.rn2(3) == 0 {
        Some(rng.rn2(25))
    } else {
        None
    };
    (forget_levels, forget_objects)
}

// ============================================================
// 10. erode_text_calc — 부식에 의한 텍스트 와이프아웃 계산
// ============================================================

/// 부식 정도에 따른 텍스트 와이프아웃 문자 수 결정
/// 원본: read.c erode_obj_text()
/// erosion: 0~3 (MAX_ERODE=3), text_len: 텍스트 길이
/// 반환: 지울 문자 수
pub fn erode_wipeout_count(erosion: i32, text_len: usize) -> usize {
    if erosion <= 0 {
        return 0;
    }
    let max_erode = 3; // MAX_ERODE
    (text_len * erosion as usize) / (2 * max_erode)
}

// ============================================================
// 11. create_monster_count — 몬스터 생성 스크롤 수량 계산
// ============================================================

/// 몬스터 생성 스크롤의 생성 수량 결정
/// 원본: seffects() SCR_CREATE_MONSTER
pub fn create_monster_count(
    confused: bool,
    cursed: bool,
    blessed: bool,
    rng: &mut NetHackRng,
) -> i32 {
    let base = 1 + if confused || cursed { 12 } else { 0 };
    let bonus = if blessed || rng.rn2(73) != 0 {
        0
    } else {
        rng.rnd(4)
    };
    base + bonus
}

// ============================================================
// 12. scare_monster_result — 공포 스크롤 효과 방향 결정
// ============================================================

/// 공포 스크롤/주문의 효과 방향: 공포 유발 vs 각성(저주/혼란)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScareResult {
    /// 몬스터를 겁에 질리게 함
    Frighten,
    /// 저주/혼란 — 몬스터를 깨우고 풀어줌
    Awaken,
}

/// 공포 스크롤 효과 방향 결정
/// 원본: seffects() SCR_SCARE_MONSTER 분기
pub fn scare_monster_result(confused: bool, cursed: bool) -> ScareResult {
    if confused || cursed {
        ScareResult::Awaken
    } else {
        ScareResult::Frighten
    }
}

// ============================================================
// 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::rng::NetHackRng;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    // --- tshirt_text_index ---
    #[test]
    fn test_tshirt_text_index() {
        assert_eq!(tshirt_text_index(0), 0);
        let idx = tshirt_text_index(12345);
        assert!(idx < TSHIRT_MSGS.len());
    }

    #[test]
    fn test_apron_text_index() {
        let idx = apron_text_index(7);
        assert!(idx < APRON_MSGS.len());
    }

    // --- credit_card_text ---
    #[test]
    fn test_credit_card_normal() {
        let (issuer, number) = credit_card_text(42, false);
        assert!(!issuer.is_empty());
        assert!(!number.is_empty());
        assert_ne!(issuer, CARD_MSGS[CARD_MSGS.len() - 1]); // Platinum이 아님
    }

    #[test]
    fn test_credit_card_artifact() {
        let (issuer, _) = credit_card_text(42, true);
        assert_eq!(issuer, "Yendorian Express - Platinum Card");
    }

    // --- is_chargeable ---
    #[test]
    fn test_chargeable_wand() {
        assert!(is_chargeable(ChargeableClass::Wand));
    }

    #[test]
    fn test_chargeable_ring() {
        assert!(is_chargeable(ChargeableClass::Ring { oc_charged: true }));
        assert!(!is_chargeable(ChargeableClass::Ring { oc_charged: false }));
    }

    #[test]
    fn test_chargeable_weptool() {
        assert!(!is_chargeable(ChargeableClass::Tool {
            oc_charged: true,
            is_weptool: true,
        }));
    }

    // --- recharge_wand_result ---
    #[test]
    fn test_wand_recharge_cursed() {
        let mut rng = test_rng();
        let result = recharge_wand_result(WandType::Directional, 5, 0, BcsStatus::Cursed, &mut rng);
        assert_eq!(result, WandRechargeResult::StripSpe);
    }

    #[test]
    fn test_wand_recharge_blessed() {
        let mut rng = test_rng();
        match recharge_wand_result(WandType::Directional, 3, 0, BcsStatus::Blessed, &mut rng) {
            WandRechargeResult::Charged { new_spe, .. } => {
                assert!(new_spe >= 3);
            }
            _ => panic!("축복 충전은 폭발 아님(첫 충전)"),
        }
    }

    #[test]
    fn test_wand_wishing_overcharge() {
        let mut rng = test_rng();
        // 소원의 지팡이 spe=3, 충전 시도 → spe>3이면 폭발
        // 첫 충전이므로 폭발 확률 검사 통과, 하지만 new_spe > 3이면 폭발
        // 이것은 seed에 따라 달라지므로 match로 양쪽 모두 검증
        let result = recharge_wand_result(WandType::Wishing, 3, 0, BcsStatus::Blessed, &mut rng);
        match result {
            WandRechargeResult::Explode { .. } | WandRechargeResult::Charged { .. } => {}
            _ => panic!("소원 충전 결과가 예상과 다름"),
        }
    }

    // --- recharge_ring_result ---
    #[test]
    fn test_ring_normal_charge() {
        let mut rng = test_rng();
        match recharge_ring_result(0, BcsStatus::Uncursed, &mut rng) {
            RingRechargeResult::Spin { delta } => assert_eq!(delta, 1),
            _ => panic!("일반 반지 충전은 회전"),
        }
    }

    // --- recharge_tool_result ---
    #[test]
    fn test_bell_blessed() {
        let mut rng = test_rng();
        match recharge_tool_result(
            ChargeableToolType::BellOfOpening,
            2,
            0,
            BcsStatus::Blessed,
            &mut rng,
        ) {
            ToolRechargeResult::Charged { new_spe, glow_blue } => {
                assert!(new_spe >= 3 && new_spe <= 5);
                assert!(glow_blue);
            }
            _ => panic!("축복 종 충전"),
        }
    }

    #[test]
    fn test_marker_already_recharged() {
        let mut rng = test_rng();
        let result = recharge_tool_result(
            ChargeableToolType::MagicMarker,
            10,
            1,
            BcsStatus::Blessed,
            &mut rng,
        );
        assert_eq!(result, ToolRechargeResult::MarkerDriedOut);
    }

    #[test]
    fn test_crystal_ball_max() {
        let mut rng = test_rng();
        let result = recharge_tool_result(
            ChargeableToolType::CrystalBall,
            5,
            0,
            BcsStatus::Uncursed,
            &mut rng,
        );
        assert_eq!(result, ToolRechargeResult::NotChargeable);
    }

    // --- enchant_armor_calc ---
    #[test]
    fn test_enchant_armor_normal() {
        let mut rng = test_rng();
        match enchant_armor_calc(3, false, false, BcsStatus::Uncursed, &mut rng) {
            EnchantArmorResult::Enchanted { delta } => {
                let _ = delta;
            }
            EnchantArmorResult::OverenchantExplode => {}
            _ => {}
        }
    }

    #[test]
    fn test_enchant_dragon_scales() {
        let mut rng = test_rng();
        match enchant_armor_calc(0, true, false, BcsStatus::Blessed, &mut rng) {
            EnchantArmorResult::DragonScaleUpgrade {
                spe_bonus,
                do_bless,
            } => {
                assert_eq!(spe_bonus, 1);
                assert!(do_bless);
            }
            _ => panic!("용 비늘은 업그레이드"),
        }
    }

    // --- enchant_weapon_calc ---
    #[test]
    fn test_enchant_weapon_cursed() {
        let mut rng = test_rng();
        match enchant_weapon_calc(5, true, BcsStatus::Cursed, &mut rng) {
            EnchantWeaponResult::EnchantDelta(d) => assert_eq!(d, -1),
            _ => panic!("저주 무기 강화는 -1"),
        }
    }

    #[test]
    fn test_enchant_weapon_no_weapon() {
        let mut rng = test_rng();
        match enchant_weapon_calc(0, false, BcsStatus::Uncursed, &mut rng) {
            EnchantWeaponResult::EnchantDelta(d) => assert_eq!(d, 1),
            _ => panic!("무장 없이 무기 강화는 +1"),
        }
    }

    // --- forget_percentage_calc ---
    #[test]
    fn test_forget_percentage() {
        let mut rng = test_rng();
        let (levels, objects) = forget_percentage_calc(&mut rng);
        if let Some(l) = levels {
            assert!(l >= 0 && l < 25);
        }
        if let Some(o) = objects {
            assert!(o >= 0 && o < 25);
        }
    }

    // --- erode_wipeout_count ---
    #[test]
    fn test_erode_no_erosion() {
        assert_eq!(erode_wipeout_count(0, 50), 0);
    }

    #[test]
    fn test_erode_max_erosion() {
        // erosion=3, len=60: 60*3/(2*3) = 30
        assert_eq!(erode_wipeout_count(3, 60), 30);
    }

    #[test]
    fn test_erode_partial() {
        // erosion=1, len=30: 30*1/6 = 5
        assert_eq!(erode_wipeout_count(1, 30), 5);
    }

    // --- create_monster_count ---
    #[test]
    fn test_create_monster_normal() {
        let mut rng = test_rng();
        let count = create_monster_count(false, false, false, &mut rng);
        assert!(count >= 1 && count <= 5);
    }

    #[test]
    fn test_create_monster_confused() {
        let mut rng = test_rng();
        let count = create_monster_count(true, false, false, &mut rng);
        assert!(count >= 13); // 1 + 12
    }

    // --- scare_monster_result ---
    #[test]
    fn test_scare_normal() {
        assert_eq!(scare_monster_result(false, false), ScareResult::Frighten);
    }

    #[test]
    fn test_scare_confused() {
        assert_eq!(scare_monster_result(true, false), ScareResult::Awaken);
    }

    #[test]
    fn test_scare_cursed() {
        assert_eq!(scare_monster_result(false, true), ScareResult::Awaken);
    }
}

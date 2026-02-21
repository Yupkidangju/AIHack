// =============================================================================
// AIHack — invent_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
// [v2.19.0] invent.c 핵심 함수 이식 — Pure Result 패턴
// 원본: nethack-3.6.7/src/invent.c (4,480줄)
//
// 이식 대상:
//   loot_classify  (L56-213)    → loot_classify_result
//   sortloot_cmp   (L303-436)   → sortloot_compare
//   merged         (L703-793)   → can_merge / merge_result
//   addinv_core1   (L806-859)   → addinv_special_check
//   inv_cnt        (L1109-1124) → inv_count
//   compactify     (L???)       → compactify_classes
//   encumber_msg   (L1182-1252) → encumber_change_message
//   carrying       (L1087-1108) → carrying_check
// =============================================================================

// =============================================================================
// [v2.19.0] 아이템 분류 (원본: invent.c L56-213 loot_classify)
// 전리품 목록 정렬 시 사용하는 분류 키 생성
// =============================================================================

/// 아이템 클래스 (원본: objclass.h COIN_CLASS ... VENOM_CLASS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjClass {
    Coin = 0,
    Amulet = 1,
    Ring = 2,
    Wand = 3,
    Potion = 4,
    Scroll = 5,
    Spellbook = 6,
    Gem = 7,
    Food = 8,
    Tool = 9,
    Weapon = 10,
    Armor = 11,
    Rock = 12,
    Ball = 13,
    Chain = 14,
    Venom = 15,
    Other = 16,
}

/// 방어구 하위분류 (원본: objclass.h ARM_* 상수)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArmorSubclass {
    Helm = 1,
    Gloves = 2,
    Boots = 3,
    Shield = 4,
    Cloak = 5,
    Shirt = 6,
    Suit = 7,
}

/// 전리품 분류 키 (원본: Loot 구조체의 orderclass/subclass/disco)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LootClassification {
    /// 클래스 순서 (1부터 시작, 낮을수록 먼저)
    pub order_class: i32,
    /// 하위분류 (클래스 내 세부 분류)
    pub subclass: i32,
    /// 발견 상태 (1=미확인, 2=확인미발견, 3=이름부여, 4=발견완료)
    pub disco: i32,
}

/// 전리품 분류용 아이템 정보
#[derive(Debug, Clone)]
pub struct LootItemInfo {
    /// 아이템 클래스
    pub obj_class: ObjClass,
    /// 방어구 하위분류 (방어구일 때만 유효)
    pub armor_subclass: Option<ArmorSubclass>,
    /// 무기 스킬 종류 (-값: 탄약 종류, +값: 무기 종류)
    pub weapon_skill: i32,
    /// 도구: 컨테이너인가
    pub is_container: bool,
    /// 도구: 악기인가
    pub is_instrument: bool,
    /// 음식: 글롭(glob) 형태인가
    pub is_glob: bool,
    /// 음식: 통조림인가
    pub is_tin: bool,
    /// 음식: 알인가
    pub is_egg: bool,
    /// 음식: 시체인가
    pub is_corpse: bool,
    /// 보석: 재질 (1=보석, 2=유리, 3=광물)
    pub gem_material: i32,
    /// 보석: 바위인가
    pub is_rock: bool,
    /// 물체가 dknown 상태인가 (외관 확인)
    pub seen: bool,
    /// 이름이 알려졌는가
    pub discovered: bool,
    /// 사용자 지정 이름이 있는가
    pub user_named: bool,
    /// 극봉(pole arm)인가
    pub is_pole: bool,
}

/// [v2.19.0] 전리품 분류 (원본: loot_classify L56-213)
/// 클래스/하위분류/발견상태로 정렬 키 생성
pub fn loot_classify_result(info: &LootItemInfo) -> LootClassification {
    // 원본 기본 분류 순서: COIN, AMULET, RING, WAND, POTION, SCROLL, SPBOOK, GEM,
    //                     FOOD, TOOL, WEAPON, ARMOR, ROCK, BALL, CHAIN
    let order = match info.obj_class {
        ObjClass::Coin => 1,
        ObjClass::Amulet => 2,
        ObjClass::Ring => 3,
        ObjClass::Wand => 4,
        ObjClass::Potion => 5,
        ObjClass::Scroll => 6,
        ObjClass::Spellbook => 7,
        ObjClass::Gem => 8,
        ObjClass::Food => 9,
        ObjClass::Tool => 10,
        ObjClass::Weapon => 11,
        ObjClass::Armor => 12,
        ObjClass::Rock => 13,
        ObjClass::Ball => 14,
        ObjClass::Chain => 15,
        _ => 16,
    };

    // 하위분류 (원본: switch(oclass) case별 처리)
    let subclass = match info.obj_class {
        // 원본: case ARMOR_CLASS (L94-113)
        ObjClass::Armor => match info.armor_subclass {
            Some(ArmorSubclass::Helm) => 1,
            Some(ArmorSubclass::Gloves) => 2,
            Some(ArmorSubclass::Boots) => 3,
            Some(ArmorSubclass::Shield) => 4,
            Some(ArmorSubclass::Cloak) => 5,
            Some(ArmorSubclass::Shirt) => 6,
            Some(ArmorSubclass::Suit) => 7,
            None => 8,
        },
        // 원본: case WEAPON_CLASS (L114-123) — 탄약/발사대/미사일/적재/기타/극봉
        ObjClass::Weapon => {
            let k = info.weapon_skill;
            if k < 0 {
                // 원본: (k >= -P_CROSSBOW && k <= -P_BOW) ? 1 : 3
                if k >= -6 && k <= -4 {
                    1
                } else {
                    3
                } // 탄약류
            } else if k >= 4 && k <= 6 {
                2 // 발사대
            } else if k == 1 || k == 2 || k == 3 {
                4 // 단검/나이프/창 (적재 가능)
            } else if !info.is_pole {
                5 // 일반 무기
            } else {
                6 // 극봉
            }
        }
        // 원본: case TOOL_CLASS (L124-149)
        ObjClass::Tool => {
            if info.is_container {
                1
            } else if info.is_instrument {
                3
            } else {
                4
            }
        }
        // 원본: case FOOD_CLASS (L150-170)
        ObjClass::Food => {
            if info.is_glob {
                6
            } else if info.is_tin {
                3
            } else if info.is_egg {
                4
            } else if info.is_corpse {
                5
            } else {
                2
            }
        }
        // 원본: case GEM_CLASS (L171-199) — 보석/유리/광물 × 발견상태
        ObjClass::Gem => match info.gem_material {
            1 => {
                if !info.seen {
                    1
                } else if !info.discovered {
                    2
                } else {
                    3
                }
            } // 보석
            2 => {
                if !info.seen {
                    1
                } else if !info.discovered {
                    2
                } else {
                    4
                }
            } // 유리
            _ => {
                // 광물/바위
                if !info.seen {
                    5
                } else if !info.is_rock {
                    if !info.discovered {
                        6
                    } else {
                        7
                    }
                } else {
                    8
                }
            }
        },
        _ => 1,
    };

    // 발견 상태 (원본: L207-212)
    let disco = if !info.seen {
        1
    } else if info.discovered {
        4
    } else if info.user_named {
        3
    } else {
        2
    };

    LootClassification {
        order_class: order,
        subclass,
        disco,
    }
}

// =============================================================================
// [v2.19.0] 전리품 정렬 비교 (원본: invent.c L303-436 sortloot_cmp)
// =============================================================================

/// 정렬된 전리품 항목 (비교에 필요한 정보)
#[derive(Debug, Clone)]
pub struct SortLootEntry {
    /// 분류 키
    pub classification: LootClassification,
    /// 인벤토리 레터 (정렬 모드에 따라 사용)
    pub invlet: char,
    /// 이름 (알파벳 정렬용)
    pub name: String,
    /// 축복 상태 (3=축복, 2=보통, 1=저주, 0=미확인)
    pub bucx: i32,
    /// 그리스 도포 여부
    pub greased: bool,
    /// 최대 침식 수준
    pub erosion: i32,
    /// 침식 방지 여부 (알려진 상태)
    pub erodeproof_known: bool,
    /// 강화 수치 (+/- spe)
    pub enchantment: Option<i32>,
    /// 원래 순서 인덱스 (안정 정렬용)
    pub original_index: i32,
}

/// 정렬 모드 플래그
#[derive(Debug, Clone, Copy)]
pub struct SortLootMode {
    /// 클래스별 분류 (sortpack)
    pub by_pack: bool,
    /// 인벤토리 레터 순서
    pub by_invlet: bool,
    /// 전리품 모드 (이름 정렬 활성)
    pub by_loot: bool,
}

/// [v2.19.0] 전리품 비교 (원본: sortloot_cmp L303-436)
/// 두 항목의 정렬 순서를 결정 — 음수면 a가 먼저, 양수면 b가 먼저
pub fn sortloot_compare(
    a: &SortLootEntry,
    b: &SortLootEntry,
    mode: SortLootMode,
) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    // 원본: 클래스별 정렬 (sortpack이면서 invlet 전용이 아닌 경우)
    if mode.by_pack || !mode.by_invlet {
        let class_cmp = a
            .classification
            .order_class
            .cmp(&b.classification.order_class);
        if class_cmp != Ordering::Equal {
            return class_cmp;
        }

        if !mode.by_invlet {
            let sub_cmp = a.classification.subclass.cmp(&b.classification.subclass);
            if sub_cmp != Ordering::Equal {
                return sub_cmp;
            }

            let disco_cmp = a.classification.disco.cmp(&b.classification.disco);
            if disco_cmp != Ordering::Equal {
                return disco_cmp;
            }
        }
    }

    // 원본: 인벤토리 레터 순서 (L357-372)
    if mode.by_invlet {
        let rank_a = invlet_rank(a.invlet);
        let rank_b = invlet_rank(b.invlet);
        let let_cmp = rank_a.cmp(&rank_b);
        if let_cmp != Ordering::Equal {
            return let_cmp;
        }
    }

    // 원본: 이름 알파벳 정렬 (L374-436)
    if mode.by_loot {
        let name_cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
        if name_cmp != Ordering::Equal {
            return name_cmp;
        }

        // BUCX: 높을수록 좋다 → 역순
        let bucx_cmp = b.bucx.cmp(&a.bucx);
        if bucx_cmp != Ordering::Equal {
            return bucx_cmp;
        }

        // 그리스: 있는 게 좋다 → 역순
        let grease_cmp = b.greased.cmp(&a.greased);
        if grease_cmp != Ordering::Equal {
            return grease_cmp;
        }

        // 침식: 낮을수록 좋다
        let erosion_cmp = a.erosion.cmp(&b.erosion);
        if erosion_cmp != Ordering::Equal {
            return erosion_cmp;
        }

        // 침식방지: 있는 게 좋다 → 역순
        let proof_cmp = b.erodeproof_known.cmp(&a.erodeproof_known);
        if proof_cmp != Ordering::Equal {
            return proof_cmp;
        }

        // 강화: 높을수록 좋다 → 역순 (미확인은 -1000)
        let ench_a = a.enchantment.unwrap_or(-1000);
        let ench_b = b.enchantment.unwrap_or(-1000);
        let ench_cmp = ench_b.cmp(&ench_a);
        if ench_cmp != Ordering::Equal {
            return ench_cmp;
        }
    }

    // 안정 정렬: 원래 순서 유지
    a.original_index.cmp(&b.original_index)
}

/// 인벤토리 레터 정렬 순위 (원본: L358-363)
/// $, a-z, A-Z, #, 기타 순서
fn invlet_rank(c: char) -> i32 {
    match c {
        '$' => 1,
        'a'..='z' => (c as i32) - ('a' as i32) + 2,
        'A'..='Z' => (c as i32) - ('A' as i32) + 2 + 26,
        '#' => 1 + 52 + 1,
        _ => 1 + 52 + 1 + 1,
    }
}

// =============================================================================
// [v2.19.0] 아이템 병합 판정 (원본: invent.c L703-793 merged)
// =============================================================================

/// 아이템 병합 가능 여부 판정 입력
#[derive(Debug, Clone)]
pub struct MergeCheckInput {
    /// 아이템 종류 ID (otyp)
    pub item_type_a: i32,
    pub item_type_b: i32,
    /// 아이템 클래스
    pub class_a: ObjClass,
    pub class_b: ObjClass,
    /// 축복/저주 상태
    pub blessed_a: bool,
    pub blessed_b: bool,
    pub cursed_a: bool,
    pub cursed_b: bool,
    /// 강화 수치
    pub spe_a: i32,
    pub spe_b: i32,
    /// 아이템 이름
    pub name_a: String,
    pub name_b: String,
    /// 불빛 여부 (램프/양초)
    pub lit_a: bool,
    pub lit_b: bool,
    /// 글롭 형태인가
    pub glob_a: bool,
    pub glob_b: bool,
    /// 가격이 알려졌는가
    pub known_a: bool,
    pub known_b: bool,
    /// 침식 수준
    pub erosion_a: i32,
    pub erosion_b: i32,
    /// 침식 방지
    pub erodeproof_a: bool,
    pub erodeproof_b: bool,
    /// 병합 금지 플래그
    pub no_merge_a: bool,
    pub no_merge_b: bool,
}

/// [v2.19.0] 아이템 병합 가능 여부 판정 (원본: mergable() 기반)
/// 타입/클래스/BUC/강화/이름/상태가 모두 일치해야 병합 가능
pub fn can_merge(input: &MergeCheckInput) -> bool {
    // 기본 조건: 같은 타입, 같은 클래스
    if input.item_type_a != input.item_type_b {
        return false;
    }
    if input.class_a != input.class_b {
        return false;
    }

    // 병합 금지 플래그
    if input.no_merge_a || input.no_merge_b {
        return false;
    }

    // BUC 상태 일치
    if input.blessed_a != input.blessed_b || input.cursed_a != input.cursed_b {
        return false;
    }

    // 강화 수치 일치
    if input.spe_a != input.spe_b {
        return false;
    }

    // 아이템 이름 일치
    if input.name_a != input.name_b {
        return false;
    }

    // 불빛 상태 일치
    if input.lit_a != input.lit_b {
        return false;
    }

    // 침식 상태 일치
    if input.erosion_a != input.erosion_b {
        return false;
    }
    if input.erodeproof_a != input.erodeproof_b {
        return false;
    }

    // 식별 상태 일치
    if input.known_a != input.known_b {
        return false;
    }

    true
}

/// 병합 결과
#[derive(Debug, Clone)]
pub struct MergeResult {
    /// 병합 후 수량 합산에 사용할 가중 평균 나이
    pub merged_age: u64,
    /// 병합 후 합산 무게
    pub merged_weight: i32,
}

/// [v2.19.0] 병합 결과 계산 (원본: merged L703-793)
/// 나이의 가중 평균과 무게 합산
pub fn merge_result(
    qty_a: i32,
    age_a: u64,
    weight_a: i32,
    qty_b: i32,
    age_b: u64,
    weight_b: i32,
    is_lit: bool,
    is_glob: bool,
    is_coin: bool,
) -> MergeResult {
    // 원본: 불빛이 켜져 있거나 글롭이면 나이 평균 안 함
    let merged_age = if is_lit || is_glob {
        age_a // 원본 유지
    } else {
        // 원본: otmp->age = ((otmp->age * otmp->quan) + (obj->age * obj->quan))
        //                   / (otmp->quan + obj->quan);
        let total_qty = (qty_a + qty_b) as u64;
        if total_qty > 0 {
            (age_a * qty_a as u64 + age_b * qty_b as u64) / total_qty
        } else {
            age_a
        }
    };

    // 원본: 금화는 weight() 재계산, 글롭은 흡수 처리, 기타는 합산
    let merged_weight = if is_coin || is_glob {
        weight_a // 호출자에서 재계산 필요
    } else {
        weight_a + weight_b
    };

    MergeResult {
        merged_age,
        merged_weight,
    }
}

// =============================================================================
// [v2.19.0] 특수 아이템 소지 판정 (원본: invent.c L806-859 addinv_core1)
// 옌더의 부적, 촛대, 종, 사자의 서 등 고유 아이템 소지 플래그
// =============================================================================

/// 특수 아이템 종류 (원본: 상수 비교)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialItem {
    AmuletOfYendor,
    CandelabrumOfInvocation,
    BellOfOpening,
    BookOfTheDead,
    QuestArtifact,
    MinesLuckstone,
    SokobanPrize,
    None,
}

/// 특수 아이템 소지 플래그 변동
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecialItemFlags {
    pub has_amulet: bool,
    pub has_menorah: bool,
    pub has_bell: bool,
    pub has_book: bool,
    pub has_quest_artifact: bool,
    pub achieved_mines: bool,
    pub achieved_sokoban: bool,
}

/// [v2.19.0] 특수 아이템 소지 판정 (원본: addinv_core1 L806-859)
/// 해당 아이템을 인벤토리에 추가할 때 어떤 플래그가 활성화되는지 판정
pub fn addinv_special_check(item: SpecialItem) -> SpecialItemFlags {
    let mut flags = SpecialItemFlags {
        has_amulet: false,
        has_menorah: false,
        has_bell: false,
        has_book: false,
        has_quest_artifact: false,
        achieved_mines: false,
        achieved_sokoban: false,
    };
    match item {
        SpecialItem::AmuletOfYendor => flags.has_amulet = true,
        SpecialItem::CandelabrumOfInvocation => flags.has_menorah = true,
        SpecialItem::BellOfOpening => flags.has_bell = true,
        SpecialItem::BookOfTheDead => flags.has_book = true,
        SpecialItem::QuestArtifact => flags.has_quest_artifact = true,
        SpecialItem::MinesLuckstone => flags.achieved_mines = true,
        SpecialItem::SokobanPrize => flags.achieved_sokoban = true,
        SpecialItem::None => {}
    }
    flags
}

// =============================================================================
// [v2.19.0] 인벤토리 슬롯 카운트 (원본: invent.c L1109-1124 inv_cnt)
// =============================================================================

/// [v2.19.0] 인벤토리 슬롯 사용 수 계산 (원본: inv_cnt L1109-1124)
/// incl_gold가 false이면 금화를 제외
pub fn inv_count(items: &[(ObjClass, char)], incl_gold: bool) -> i32 {
    let mut count = 0;
    for (class, _letter) in items {
        if !incl_gold && *class == ObjClass::Coin {
            continue;
        }
        count += 1;
    }
    count
}

// =============================================================================
// [v2.19.0] 인벤토리 레터 압축 (원본: invent.c compactify)
// 클래스 문자열에서 중복 제거
// =============================================================================

/// [v2.19.0] 클래스 필터 문자열 압축 (원본: compactify)
/// 중복된 클래스 문자를 제거하여 압축된 문자열 반환
pub fn compactify_classes(classes: &str) -> String {
    let mut result = String::new();
    for c in classes.chars() {
        if !result.contains(c) {
            result.push(c);
        }
    }
    result
}

// =============================================================================
// [v2.19.0] 짐 무게 변경 메시지 (원본: invent.c L1182-1252 encumber_msg)
// 짐 상태가 변경되었을 때 출력할 메시지 결정
// =============================================================================

/// 짐 무게 단계 (원본: 0=Unencumbered ~ 5=Overloaded)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EncumbranceLevel {
    Unencumbered = 0,
    Burdened = 1,
    Stressed = 2,
    Strained = 3,
    Overtaxed = 4,
    Overloaded = 5,
}

/// [v2.19.0] 짐 상태 변경 메시지 (원본: encumber_msg L1182-1252)
/// 이전 상태와 현재 상태를 비교하여 적절한 메시지 반환
pub fn encumber_change_message(
    old_level: EncumbranceLevel,
    new_level: EncumbranceLevel,
) -> Option<&'static str> {
    if old_level == new_level {
        return None;
    }

    // 원본: enc_stat[] 배열 기반 메시지
    match new_level {
        EncumbranceLevel::Unencumbered => Some("당신의 움직임이 더 이상 방해받지 않는다."),
        EncumbranceLevel::Burdened => {
            if old_level < new_level {
                Some("당신의 움직임이 약간 둔해졌다.")
            } else {
                Some("당신의 짐이 좀 더 가벼워 졌다.")
            }
        }
        EncumbranceLevel::Stressed => {
            if old_level < new_level {
                Some("당신은 짐에 눌려 버둥거린다.")
            } else {
                Some("당신은 좀 더 편하게 움직일 수 있다.")
            }
        }
        EncumbranceLevel::Strained => {
            if old_level < new_level {
                Some("당신은 겨우 움직일 수 있다.")
            } else {
                Some("당신은 다시 움직일 수 있게 되었다.")
            }
        }
        EncumbranceLevel::Overtaxed => Some("당신은 거의 움직일 수 없다!"),
        EncumbranceLevel::Overloaded => Some("당신은 너무 무거운 짐에 눌려 움직일 수 없다!"),
    }
}

// =============================================================================
// [v2.19.0] 특정 아이템 소지 여부 (원본: invent.c L1087-1108 carrying)
// =============================================================================

/// [v2.19.0] 특정 아이템 타입 소지 여부 (원본: carrying L1087-1108)
/// 인벤토리에서 지정된 타입의 아이템이 있는지 확인
pub fn carrying_check(inventory_types: &[i32], target_type: i32) -> bool {
    inventory_types.iter().any(|&t| t == target_type)
}

/// [v2.19.0] 도마뱀 시체 소지 여부 (원본: have_lizard)
/// 석화 치료에 사용되는 도마뱀 시체를 인벤토리에서 검색
pub fn have_lizard(inventory_types: &[i32], lizard_corpse_type: i32) -> bool {
    carrying_check(inventory_types, lizard_corpse_type)
}

// =============================================================================
// [v2.19.0] 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    // --- loot_classify_result 테스트 ---

    #[test]
    fn test_classify_coin() {
        let info = LootItemInfo {
            obj_class: ObjClass::Coin,
            armor_subclass: None,
            weapon_skill: 0,
            is_container: false,
            is_instrument: false,
            is_glob: false,
            is_tin: false,
            is_egg: false,
            is_corpse: false,
            gem_material: 0,
            is_rock: false,
            seen: true,
            discovered: true,
            user_named: false,
            is_pole: false,
        };
        let c = loot_classify_result(&info);
        assert_eq!(c.order_class, 1); // 금화는 최우선
    }

    #[test]
    fn test_classify_armor_helm() {
        let info = LootItemInfo {
            obj_class: ObjClass::Armor,
            armor_subclass: Some(ArmorSubclass::Helm),
            weapon_skill: 0,
            is_container: false,
            is_instrument: false,
            is_glob: false,
            is_tin: false,
            is_egg: false,
            is_corpse: false,
            gem_material: 0,
            is_rock: false,
            seen: true,
            discovered: true,
            user_named: false,
            is_pole: false,
        };
        let c = loot_classify_result(&info);
        assert_eq!(c.subclass, 1); // 투구 = 하위분류 1
    }

    #[test]
    fn test_classify_gem_unseen() {
        let info = LootItemInfo {
            obj_class: ObjClass::Gem,
            armor_subclass: None,
            weapon_skill: 0,
            is_container: false,
            is_instrument: false,
            is_glob: false,
            is_tin: false,
            is_egg: false,
            is_corpse: false,
            gem_material: 1,
            is_rock: false,
            seen: false,
            discovered: false,
            user_named: false,
            is_pole: false,
        };
        let c = loot_classify_result(&info);
        assert_eq!(c.subclass, 1); // 미확인 보석
        assert_eq!(c.disco, 1); // 미확인 상태
    }

    #[test]
    fn test_classify_food_corpse() {
        let info = LootItemInfo {
            obj_class: ObjClass::Food,
            armor_subclass: None,
            weapon_skill: 0,
            is_container: false,
            is_instrument: false,
            is_glob: false,
            is_tin: false,
            is_egg: false,
            is_corpse: true,
            gem_material: 0,
            is_rock: false,
            seen: true,
            discovered: true,
            user_named: false,
            is_pole: false,
        };
        let c = loot_classify_result(&info);
        assert_eq!(c.subclass, 5); // 시체 = 하위분류 5
    }

    // --- sortloot_compare 테스트 ---

    fn make_entry(class: i32, sub: i32, disco: i32, name: &str, idx: i32) -> SortLootEntry {
        SortLootEntry {
            classification: LootClassification {
                order_class: class,
                subclass: sub,
                disco,
            },
            invlet: 'a',
            name: name.to_string(),
            bucx: 2,
            greased: false,
            erosion: 0,
            erodeproof_known: false,
            enchantment: None,
            original_index: idx,
        }
    }

    #[test]
    fn test_sort_by_class() {
        let a = make_entry(1, 1, 1, "gold", 0);
        let b = make_entry(5, 1, 1, "potion", 1);
        let mode = SortLootMode {
            by_pack: true,
            by_invlet: false,
            by_loot: false,
        };
        assert_eq!(sortloot_compare(&a, &b, mode), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_sort_by_name() {
        let a = make_entry(5, 1, 4, "healing", 0);
        let b = make_entry(5, 1, 4, "speed", 1);
        let mode = SortLootMode {
            by_pack: true,
            by_invlet: false,
            by_loot: true,
        };
        assert_eq!(sortloot_compare(&a, &b, mode), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_sort_stable() {
        let a = make_entry(5, 1, 4, "potion", 0);
        let b = make_entry(5, 1, 4, "potion", 1);
        let mode = SortLootMode {
            by_pack: true,
            by_invlet: false,
            by_loot: true,
        };
        assert_eq!(sortloot_compare(&a, &b, mode), std::cmp::Ordering::Less); // 원래 순서
    }

    // --- can_merge 테스트 ---

    #[test]
    fn test_merge_identical() {
        let input = MergeCheckInput {
            item_type_a: 100,
            item_type_b: 100,
            class_a: ObjClass::Weapon,
            class_b: ObjClass::Weapon,
            blessed_a: false,
            blessed_b: false,
            cursed_a: false,
            cursed_b: false,
            spe_a: 0,
            spe_b: 0,
            name_a: "".to_string(),
            name_b: "".to_string(),
            lit_a: false,
            lit_b: false,
            glob_a: false,
            glob_b: false,
            known_a: true,
            known_b: true,
            erosion_a: 0,
            erosion_b: 0,
            erodeproof_a: false,
            erodeproof_b: false,
            no_merge_a: false,
            no_merge_b: false,
        };
        assert!(can_merge(&input));
    }

    #[test]
    fn test_merge_different_type() {
        let input = MergeCheckInput {
            item_type_a: 100,
            item_type_b: 101,
            class_a: ObjClass::Weapon,
            class_b: ObjClass::Weapon,
            blessed_a: false,
            blessed_b: false,
            cursed_a: false,
            cursed_b: false,
            spe_a: 0,
            spe_b: 0,
            name_a: "".to_string(),
            name_b: "".to_string(),
            lit_a: false,
            lit_b: false,
            glob_a: false,
            glob_b: false,
            known_a: true,
            known_b: true,
            erosion_a: 0,
            erosion_b: 0,
            erodeproof_a: false,
            erodeproof_b: false,
            no_merge_a: false,
            no_merge_b: false,
        };
        assert!(!can_merge(&input));
    }

    #[test]
    fn test_merge_different_buc() {
        let input = MergeCheckInput {
            item_type_a: 100,
            item_type_b: 100,
            class_a: ObjClass::Potion,
            class_b: ObjClass::Potion,
            blessed_a: true,
            blessed_b: false,
            cursed_a: false,
            cursed_b: false,
            spe_a: 0,
            spe_b: 0,
            name_a: "".to_string(),
            name_b: "".to_string(),
            lit_a: false,
            lit_b: false,
            glob_a: false,
            glob_b: false,
            known_a: true,
            known_b: true,
            erosion_a: 0,
            erosion_b: 0,
            erodeproof_a: false,
            erodeproof_b: false,
            no_merge_a: false,
            no_merge_b: false,
        };
        assert!(!can_merge(&input));
    }

    // --- merge_result 테스트 ---

    #[test]
    fn test_merge_age_average() {
        let r = merge_result(3, 100, 10, 2, 200, 8, false, false, false);
        assert_eq!(r.merged_age, (100 * 3 + 200 * 2) / 5); // = 140
        assert_eq!(r.merged_weight, 18);
    }

    #[test]
    fn test_merge_lit_no_average() {
        let r = merge_result(3, 100, 10, 2, 200, 8, true, false, false);
        assert_eq!(r.merged_age, 100); // 불빛 상태 → 나이 유지
    }

    // --- addinv_special_check 테스트 ---

    #[test]
    fn test_special_amulet() {
        let f = addinv_special_check(SpecialItem::AmuletOfYendor);
        assert!(f.has_amulet);
        assert!(!f.has_bell);
    }

    #[test]
    fn test_special_none() {
        let f = addinv_special_check(SpecialItem::None);
        assert!(!f.has_amulet && !f.has_bell && !f.has_book);
    }

    // --- inv_count 테스트 ---

    #[test]
    fn test_inv_count_with_gold() {
        let items = vec![
            (ObjClass::Coin, '$'),
            (ObjClass::Weapon, 'a'),
            (ObjClass::Armor, 'b'),
        ];
        assert_eq!(inv_count(&items, true), 3);
        assert_eq!(inv_count(&items, false), 2);
    }

    // --- compactify_classes 테스트 ---

    #[test]
    fn test_compactify() {
        assert_eq!(compactify_classes("aabbcc"), "abc");
        assert_eq!(compactify_classes("wfpw"), "wfp");
    }

    // --- encumber_change_message 테스트 ---

    #[test]
    fn test_encumber_same() {
        assert!(
            encumber_change_message(EncumbranceLevel::Burdened, EncumbranceLevel::Burdened)
                .is_none()
        );
    }

    #[test]
    fn test_encumber_increase() {
        let msg =
            encumber_change_message(EncumbranceLevel::Unencumbered, EncumbranceLevel::Burdened);
        assert!(msg.is_some());
    }

    #[test]
    fn test_encumber_decrease() {
        let msg =
            encumber_change_message(EncumbranceLevel::Stressed, EncumbranceLevel::Unencumbered);
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("방해"));
    }

    #[test]
    fn test_encumber_overloaded() {
        let msg = encumber_change_message(EncumbranceLevel::Burdened, EncumbranceLevel::Overloaded);
        assert!(msg.unwrap().contains("움직일 수 없다"));
    }

    // --- carrying_check 테스트 ---

    #[test]
    fn test_carrying() {
        let inv = vec![10, 20, 30, 40];
        assert!(carrying_check(&inv, 20));
        assert!(!carrying_check(&inv, 50));
    }

    #[test]
    fn test_have_lizard() {
        let inv = vec![10, 42, 30]; // 42 = 도마뱀 시체 가정
        assert!(have_lizard(&inv, 42));
        assert!(!have_lizard(&inv, 99));
    }

    // --- invlet_rank 테스트 ---

    #[test]
    fn test_invlet_rank() {
        assert!(invlet_rank('$') < invlet_rank('a'));
        assert!(invlet_rank('a') < invlet_rank('z'));
        assert!(invlet_rank('z') < invlet_rank('A'));
        assert!(invlet_rank('Z') < invlet_rank('#'));
    }
}

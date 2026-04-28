// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// [v2.7.0] mkobj.c 핵심 시스템 대량 이식 (424→1100줄)
// ============================================================================

use crate::core::entity::object::{ItemClass, ItemTemplate};
use crate::util::rng::NetHackRng;

// =============================================================================
// 아이템 확률 테이블 (mkobjprobs / boxiprobs / rogueprobs / hellprobs 이식)
// =============================================================================

/// 아이템 생성 확률 항목 (struct icp 이식)
#[derive(Debug, Clone, Copy)]
pub struct ItemProb {
    pub prob: i32,
    pub class: ItemClass,
}

/// 일반 던전 아이템 확률 (mkobjprobs 이식)
pub const MKOBJ_PROBS: &[ItemProb] = &[
    ItemProb {
        prob: 10,
        class: ItemClass::Weapon,
    },
    ItemProb {
        prob: 10,
        class: ItemClass::Armor,
    },
    ItemProb {
        prob: 20,
        class: ItemClass::Food,
    },
    ItemProb {
        prob: 8,
        class: ItemClass::Tool,
    },
    ItemProb {
        prob: 8,
        class: ItemClass::Gem,
    },
    ItemProb {
        prob: 16,
        class: ItemClass::Potion,
    },
    ItemProb {
        prob: 16,
        class: ItemClass::Scroll,
    },
    ItemProb {
        prob: 4,
        class: ItemClass::Spellbook,
    },
    ItemProb {
        prob: 4,
        class: ItemClass::Wand,
    },
    ItemProb {
        prob: 3,
        class: ItemClass::Ring,
    },
    ItemProb {
        prob: 1,
        class: ItemClass::Amulet,
    },
];

/// 상자 내용물 확률 (boxiprobs 이식)
pub const BOX_PROBS: &[ItemProb] = &[
    ItemProb {
        prob: 18,
        class: ItemClass::Gem,
    },
    ItemProb {
        prob: 15,
        class: ItemClass::Food,
    },
    ItemProb {
        prob: 18,
        class: ItemClass::Potion,
    },
    ItemProb {
        prob: 18,
        class: ItemClass::Scroll,
    },
    ItemProb {
        prob: 12,
        class: ItemClass::Spellbook,
    },
    ItemProb {
        prob: 7,
        class: ItemClass::Coin,
    },
    ItemProb {
        prob: 6,
        class: ItemClass::Wand,
    },
    ItemProb {
        prob: 5,
        class: ItemClass::Ring,
    },
    ItemProb {
        prob: 1,
        class: ItemClass::Amulet,
    },
];

/// 로그 레벨 확률 (rogueprobs 이식)
pub const ROGUE_PROBS: &[ItemProb] = &[
    ItemProb {
        prob: 12,
        class: ItemClass::Weapon,
    },
    ItemProb {
        prob: 12,
        class: ItemClass::Armor,
    },
    ItemProb {
        prob: 22,
        class: ItemClass::Food,
    },
    ItemProb {
        prob: 22,
        class: ItemClass::Potion,
    },
    ItemProb {
        prob: 22,
        class: ItemClass::Scroll,
    },
    ItemProb {
        prob: 5,
        class: ItemClass::Wand,
    },
    ItemProb {
        prob: 5,
        class: ItemClass::Ring,
    },
];

/// 지옥 레벨 확률 (hellprobs 이식)
pub const HELL_PROBS: &[ItemProb] = &[
    ItemProb {
        prob: 20,
        class: ItemClass::Weapon,
    },
    ItemProb {
        prob: 20,
        class: ItemClass::Armor,
    },
    ItemProb {
        prob: 16,
        class: ItemClass::Food,
    },
    ItemProb {
        prob: 12,
        class: ItemClass::Tool,
    },
    ItemProb {
        prob: 10,
        class: ItemClass::Gem,
    },
    ItemProb {
        prob: 1,
        class: ItemClass::Potion,
    },
    ItemProb {
        prob: 1,
        class: ItemClass::Scroll,
    },
    ItemProb {
        prob: 8,
        class: ItemClass::Wand,
    },
    ItemProb {
        prob: 8,
        class: ItemClass::Ring,
    },
    ItemProb {
        prob: 4,
        class: ItemClass::Amulet,
    },
];

/// 레벨 환경 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelContext {
    Normal,
    Rogue,
    Hell,
}

/// 확률 테이블 기반 아이템 클래스 선택 (mkobj 내부 확률 로직 이식)
pub fn select_class_from_probs(probs: &[ItemProb], roll: i32) -> ItemClass {
    let mut remaining = roll;
    for p in probs {
        remaining -= p.prob;
        if remaining < 0 {
            return p.class;
        }
    }
    // 안전 폴백: 마지막 항목 반환
    probs.last().map(|p| p.class).unwrap_or(ItemClass::Food)
}

/// 레벨 환경에 맞는 확률 테이블 반환
pub fn probs_for_context(ctx: LevelContext) -> &'static [ItemProb] {
    match ctx {
        LevelContext::Normal => MKOBJ_PROBS,
        LevelContext::Rogue => ROGUE_PROBS,
        LevelContext::Hell => HELL_PROBS,
    }
}

// =============================================================================
// 아이템 생성 요청/결과 구조체
// =============================================================================

/// 아이템 생성 요청 (mkobj 파라미터 이식)
#[derive(Debug, Clone)]
pub struct MkObjRequest {
    pub class: Option<ItemClass>,
    pub name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub blessed: Option<bool>,
    pub cursed: Option<bool>,
    pub spe: Option<i8>,
    pub quantity: Option<u32>,
    pub no_curse: bool,
}

impl Default for MkObjRequest {
    fn default() -> Self {
        Self {
            class: None,
            name: None,
            x: 0,
            y: 0,
            blessed: None,
            cursed: None,
            spe: None,
            quantity: None,
            no_curse: false,
        }
    }
}

/// 아이템 생성 결과 (mksobj 반환값 이식)
#[derive(Debug, Clone)]
pub struct MkObjResult {
    pub template_name: String,
    pub class: ItemClass,
    pub x: i32,
    pub y: i32,
    pub blessed: bool,
    pub cursed: bool,
    pub spe: i8,
    pub quantity: u32,
    pub weight: u32,
    pub price: u32,
    pub known: bool,
    pub bknown: bool,
    pub dknown: bool,
}

// =============================================================================
// 클래스/BUC/SPE 결정
// =============================================================================

/// 랜덤 아이템 클래스 (random_item_class — mkobj 확률 기반)
pub fn random_item_class(rng: &mut NetHackRng) -> ItemClass {
    let roll = rng.rn2(100);
    select_class_from_probs(MKOBJ_PROBS, roll)
}

/// 컨텍스트별 랜덤 아이템 클래스
pub fn random_item_class_ctx(ctx: LevelContext, rng: &mut NetHackRng) -> ItemClass {
    let probs = probs_for_context(ctx);
    let total: i32 = probs.iter().map(|p| p.prob).sum();
    let roll = rng.rn2(total.max(1));
    select_class_from_probs(probs, roll)
}

/// BUC 상태 결정 (blessorcurse 로직 이식)
pub fn determine_buc(rng: &mut NetHackRng, no_curse: bool) -> (bool, bool) {
    let roll = rng.rn2(100);
    if no_curse {
        if roll < 15 {
            (true, false)
        } else {
            (false, false)
        }
    } else {
        if roll < 10 {
            (true, false)
        } else if roll < 25 {
            (false, true)
        } else {
            (false, false)
        }
    }
}

/// SPE 결정 (determine_spe — 클래스별 강화치 이식)
pub fn determine_spe(class: ItemClass, blessed: bool, cursed: bool, rng: &mut NetHackRng) -> i8 {
    match class {
        ItemClass::Weapon | ItemClass::Armor => {
            if blessed {
                (rng.rn2(4) + 1) as i8
            } else if cursed {
                -(rng.rn2(4) + 1) as i8
            } else {
                rng.rn2(3) as i8
            }
        }
        ItemClass::Ring => {
            if cursed {
                -(rng.rn2(3) + 1) as i8
            } else {
                (rng.rn2(3) + 1) as i8
            }
        }
        ItemClass::Wand => (rng.rn2(5) + 3) as i8,
        _ => 0,
    }
}

// =============================================================================
// BUC 상태 관리 (bless/unbless/curse/uncurse/blessorcurse/bcsign 이식)
// =============================================================================

/// BUC 상태 구조체
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucState {
    pub blessed: bool,
    pub cursed: bool,
}

impl BucState {
    pub fn uncursed() -> Self {
        Self {
            blessed: false,
            cursed: false,
        }
    }
    pub fn blessed() -> Self {
        Self {
            blessed: true,
            cursed: false,
        }
    }
    pub fn cursed() -> Self {
        Self {
            blessed: false,
            cursed: true,
        }
    }
}

/// 축복 적용 (bless 이식) — 금화는 변경 불가
pub fn bless_item(buc: &mut BucState, is_coin: bool) {
    if is_coin {
        return;
    }
    buc.cursed = false;
    buc.blessed = true;
}

/// 축복 해제 (unbless 이식)
pub fn unbless_item(buc: &mut BucState) {
    buc.blessed = false;
}

/// 저주 적용 (curse 이식) — 금화는 변경 불가
pub fn curse_item(buc: &mut BucState, is_coin: bool) {
    if is_coin {
        return;
    }
    buc.blessed = false;
    buc.cursed = true;
}

/// 저주 해제 (uncurse 이식)
pub fn uncurse_item(buc: &mut BucState) {
    buc.cursed = false;
}

/// 랜덤 축복/저주 (blessorcurse 이식)
pub fn bless_or_curse(buc: &mut BucState, chance: i32, rng: &mut NetHackRng) {
    if buc.blessed || buc.cursed {
        return;
    }
    if rng.rn2(chance.max(1)) == 0 {
        if rng.rn2(2) == 0 {
            buc.cursed = true;
        } else {
            buc.blessed = true;
        }
    }
}

/// BUC 부호 (bcsign 이식): 축복=+1, 저주=-1, 무=0
pub fn bcsign(buc: &BucState) -> i32 {
    (buc.blessed as i32) - (buc.cursed as i32)
}

// =============================================================================
// 무게 계산 (weight 함수 이식)
// =============================================================================

/// 아이템 무게 계산 — 재귀적 컨테이너 지원 (weight 이식)
/// base_wt: 기본 무게, qty: 수량, is_boh: 마법 가방 여부
/// contents_wt: 내용물 총 무게, oeaten: 먹은 비율
pub fn calc_weight(
    base_wt: i32,
    qty: i32,
    is_container: bool,
    is_boh: bool,
    buc: &BucState,
    contents_wt: i32,
    is_coin: bool,
    is_glob: bool,
    glob_owt: i32,
) -> i32 {
    // 글럽은 무게 누적 방식 (glob_owt가 0이 아니면 사용)
    let wt = if is_glob && glob_owt > 0 {
        glob_owt
    } else {
        base_wt
    };

    if is_container {
        // 마법 가방(Bag of Holding) 무게 공식
        let cwt = if is_boh {
            if buc.cursed {
                contents_wt * 2
            } else if buc.blessed {
                (contents_wt + 3) / 4
            } else {
                (contents_wt + 1) / 2
            }
        } else {
            contents_wt
        };
        return wt + cwt;
    }
    if is_coin {
        // 금화: (수량+50)/100
        return ((qty as i64 + 50) / 100) as i32;
    }
    // 일반 아이템
    if wt > 0 {
        wt * qty
    } else {
        (qty + 1) >> 1
    }
}

/// 컨테이너 무게 재계산 (container_weight 이식)
pub fn recalc_container_weight(
    base_wt: i32,
    is_boh: bool,
    buc: &BucState,
    item_weights: &[i32],
) -> i32 {
    let cwt: i32 = item_weights.iter().sum();
    calc_weight(base_wt, 1, true, is_boh, buc, cwt, false, false, 0)
}

// =============================================================================
// 시체 부패/부활 타이머 (start_corpse_timeout 이식)
// =============================================================================

/// 시체 타이머 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpseTimerAction {
    RotAway,
    Revive,
}

/// 시체 타이머 정보
#[derive(Debug, Clone)]
pub struct CorpseTimer {
    pub action: CorpseTimerAction,
    pub when: u64,
}

/// 부패 상수 (TAINT_AGE, ROT_AGE 이식)
pub const TAINT_AGE: u64 = 50;
pub const ROT_AGE: u64 = 250;
pub const TROLL_REVIVE_CHANCE: u32 = 37;

/// 시체 부패 타이머 시작 (start_corpse_timeout 이식)
/// is_lizard/is_lichen: 부패 면제, is_rider: 라이더 부활, is_troll: 트롤 부활
pub fn start_corpse_timeout(
    corpse_age: u64,
    current_turn: u64,
    is_lizard: bool,
    is_lichen: bool,
    is_rider: bool,
    is_troll: bool,
    no_revive: bool,
    in_mklev: bool,
    rng: &mut NetHackRng,
) -> Option<CorpseTimer> {
    // 도마뱀/지의류는 부패/부활 없음
    if is_lizard || is_lichen {
        return None;
    }

    let rot_adjust: u64 = if in_mklev { 25 } else { 10 };
    let age = current_turn.saturating_sub(corpse_age);
    let base_when = if age > ROT_AGE {
        rot_adjust
    } else {
        ROT_AGE - age
    };
    let variation = (rng.rn2(rot_adjust as i32 * 2 + 1) as u64).max(1);
    let mut when = base_when + variation - rot_adjust;

    // 라이더: 항상 부활, 12~500턴
    if is_rider {
        when = 12;
        for t in 12..500u64 {
            if rng.rn2(3) == 0 {
                when = t;
                break;
            }
        }
        return Some(CorpseTimer {
            action: CorpseTimerAction::Revive,
            when,
        });
    }

    // 트롤: 확률적 부활
    if is_troll && !no_revive {
        for age_t in 2..=TAINT_AGE {
            if rng.rn2(TROLL_REVIVE_CHANCE as i32) == 0 {
                return Some(CorpseTimer {
                    action: CorpseTimerAction::Revive,
                    when: age_t,
                });
            }
        }
    }

    Some(CorpseTimer {
        action: CorpseTimerAction::RotAway,
        when,
    })
}

// =============================================================================
// 얼음 위 시체 (peek_at_iced_corpse_age / obj_ice_effects 이식)
// =============================================================================

/// 얼음 위 부패 보정 배수
pub const ROT_ICE_ADJUSTMENT: u64 = 2;

/// 얼음 위 시체 실효 나이 (peek_at_iced_corpse_age 이식)
pub fn peek_iced_corpse_age(obj_age: u64, current_turn: u64, on_ice: bool) -> u64 {
    if !on_ice {
        return obj_age;
    }
    let age = current_turn.saturating_sub(obj_age);
    obj_age + age * (ROT_ICE_ADJUSTMENT - 1) / ROT_ICE_ADJUSTMENT
}

/// 얼음 위 배치 시 나이 조정 (obj_timer_checks — 얼음 배치 이식)
pub fn adjust_age_onto_ice(obj_age: u64, current_turn: u64) -> u64 {
    let age = current_turn.saturating_sub(obj_age);
    current_turn.saturating_sub(age * ROT_ICE_ADJUSTMENT)
}

/// 얼음에서 제거 시 나이 조정 (obj_timer_checks — 얼음 제거 이식)
pub fn adjust_age_off_ice(obj_age: u64, current_turn: u64) -> u64 {
    let age = current_turn.saturating_sub(obj_age);
    obj_age + age * (ROT_ICE_ADJUSTMENT - 1) / ROT_ICE_ADJUSTMENT
}

// =============================================================================
// 재료 속성 (is_flammable / is_rottable 이식)
// =============================================================================

/// 재료 유형 열거 (oc_material 이식)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Material {
    Liquid = 0,
    Wax = 1,
    Veggy = 2,
    Flesh = 3,
    Paper = 4,
    Cloth = 5,
    Leather = 6,
    Wood = 7,
    Bone = 8,
    DragonHide = 9,
    Iron = 10,
    Metal = 11,
    Copper = 12,
    Silver = 13,
    Gold = 14,
    Platinum = 15,
    Mithril = 16,
    Plastic = 17,
    Glass = 18,
    Gemstone = 19,
    Mineral = 20,
}

/// 가연성 판정 (is_flammable 이식)
pub fn is_flammable(material: Material, is_candle: bool, has_fire_res: bool) -> bool {
    if is_candle || has_fire_res {
        return false;
    }
    (material <= Material::Wood && material != Material::Liquid) || material == Material::Plastic
}

/// 부패 가능 판정 (is_rottable 이식)
pub fn is_rottable(material: Material) -> bool {
    material <= Material::Wood && material != Material::Liquid
}

// =============================================================================
// 나무 열매 (rnd_treefruit_at 이식)
// =============================================================================

/// 나무 열매 목록 (treefruits 이식)
pub const TREE_FRUITS: &[&str] = &["apple", "orange", "pear", "banana", "eucalyptus leaf"];

/// 랜덤 나무 열매 이름 반환
pub fn random_treefruit(rng: &mut NetHackRng) -> &'static str {
    TREE_FRUITS[rng.rn2(TREE_FRUITS.len() as i32) as usize]
}

// =============================================================================
// 상자 내용물 결정 (mkbox_cnts 이식)
// =============================================================================

/// 상자 유형별 최대 내용물 수 (mkbox_cnts 초기 n값 이식)
pub fn box_max_contents(box_type: &str, locked: bool, is_initial_inv: bool) -> i32 {
    match box_type {
        "ice box" => 20,
        "chest" => {
            if locked {
                7
            } else {
                5
            }
        }
        "large box" => {
            if locked {
                5
            } else {
                3
            }
        }
        "sack" | "oilskin sack" => {
            if is_initial_inv {
                0
            } else {
                1
            }
        }
        "bag of holding" => 1,
        _ => 0,
    }
}

/// 상자 내용물 클래스 선택 (boxiprobs 확률 기반)
pub fn box_content_class(rng: &mut NetHackRng) -> ItemClass {
    let roll = rng.rn2(100);
    select_class_from_probs(BOX_PROBS, roll)
}

// =============================================================================
// 풍요의 뿔 (hornoplenty 이식)
// =============================================================================

/// 풍요의 뿔 생성 결과
#[derive(Debug, Clone)]
pub struct HornResult {
    pub produced: bool,
    pub is_potion: bool,
    pub class: ItemClass,
    pub message: String,
}

/// 풍요의 뿔 사용 (hornoplenty 이식)
pub fn horn_of_plenty(spe: i32, rng: &mut NetHackRng) -> HornResult {
    if spe < 1 {
        return HornResult {
            produced: false,
            is_potion: false,
            class: ItemClass::Food,
            message: "Nothing happens.".into(),
        };
    }
    let is_potion = rng.rn2(13) == 0;
    let class = if is_potion {
        ItemClass::Potion
    } else {
        ItemClass::Food
    };
    let what = if is_potion { "A potion" } else { "Some food" };
    HornResult {
        produced: true,
        is_potion,
        class,
        message: format!("{} spills out.", what),
    }
}

// =============================================================================
// 오브젝트 분할 (splitobj 로직 이식)
// =============================================================================

/// 스택 분할 결과
#[derive(Debug, Clone)]
pub struct SplitResult {
    pub original_qty: u32,
    pub split_qty: u32,
    pub original_weight: i32,
    pub split_weight: i32,
}

/// 스택 분할 (splitobj 로직 이식)
pub fn split_stack(total_qty: u32, split_num: u32, weight_per: i32) -> Option<SplitResult> {
    if split_num == 0 || split_num >= total_qty {
        return None;
    }
    let remain = total_qty - split_num;
    Some(SplitResult {
        original_qty: remain,
        split_qty: split_num,
        original_weight: weight_per * remain as i32,
        split_weight: weight_per * split_num as i32,
    })
}

// =============================================================================
// 글럽 합체 (obj_absorb / obj_meld 이식)
// =============================================================================

/// 글럽 합체 결과
#[derive(Debug, Clone)]
pub struct GlobMergeResult {
    pub merged_weight: i32,
    pub merged_age: u64,
    pub merged_eaten: i32,
}

/// 글럽 합체 (obj_absorb 이식)
pub fn glob_absorb(
    wt1: i32,
    age1: u64,
    eaten1: i32,
    wt2: i32,
    age2: u64,
    eaten2: i32,
    current_turn: u64,
) -> GlobMergeResult {
    let o1wt = if eaten1 > 0 { eaten1 } else { wt1 };
    let o2wt = if eaten2 > 0 { eaten2 } else { wt2 };
    let total = (o1wt + o2wt).max(1);
    // 상대 나이 가중 평균
    let rel1 = current_turn.saturating_sub(age1) as i64;
    let rel2 = current_turn.saturating_sub(age2) as i64;
    let avg_rel = (rel1 * o1wt as i64 + rel2 * o2wt as i64) / total as i64;
    let merged_age = current_turn.saturating_sub(avg_rel as u64);
    let merged_eaten = if eaten1 > 0 || eaten2 > 0 {
        o1wt + o2wt
    } else {
        0
    };
    GlobMergeResult {
        merged_weight: wt1 + o2wt,
        merged_age,
        merged_eaten,
    }
}

// =============================================================================
// 변경 동사 (alteration_verbs 이식 — costly_alteration에서 사용)
// =============================================================================

/// 변경 유형 열거 (COST_xxx 이식)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlterationType {
    Cancel,
    Drain,
    Uncharge,
    Unbless,
    Uncurse,
    Disenchant,
    Degrade,
    Dilute,
    Erase,
    Burn,
    Neutralize,
    Destroy,
    Splatter,
    Bite,
    Open,
    BreakLock,
    Rust,
    Rot,
    Tarnish,
}

/// 변경 동사 문자열 (alteration_verbs 이식)
pub fn alteration_verb(a: AlterationType) -> &'static str {
    match a {
        AlterationType::Cancel => "cancel",
        AlterationType::Drain => "drain",
        AlterationType::Uncharge => "uncharge",
        AlterationType::Unbless => "unbless",
        AlterationType::Uncurse => "uncurse",
        AlterationType::Disenchant => "disenchant",
        AlterationType::Degrade => "degrade",
        AlterationType::Dilute => "dilute",
        AlterationType::Erase => "erase",
        AlterationType::Burn => "burn",
        AlterationType::Neutralize => "neutralize",
        AlterationType::Destroy => "destroy",
        AlterationType::Splatter => "splatter",
        AlterationType::Bite => "bite",
        AlterationType::Open => "open",
        AlterationType::BreakLock => "break the lock on",
        AlterationType::Rust => "rust",
        AlterationType::Rot => "rot",
        AlterationType::Tarnish => "tarnish",
    }
}

// =============================================================================
// 기존 함수 (mkobj / mkgold / mkcorpse / mkstatue 등)
// =============================================================================

/// 아이템 생성 메인 함수 (mkobj / mksobj 이식)
pub fn mkobj(
    request: &MkObjRequest,
    templates: &std::collections::HashMap<String, ItemTemplate>,
    rng: &mut NetHackRng,
) -> Option<MkObjResult> {
    let class = request.class.unwrap_or_else(|| random_item_class(rng));

    // 이름 지정 생성
    if let Some(ref name) = request.name {
        if let Some(tmpl) = templates.get(name) {
            let (bl, cu) = match (request.blessed, request.cursed) {
                (Some(b), Some(c)) => (b, c),
                _ => determine_buc(rng, request.no_curse),
            };
            let spe = request
                .spe
                .unwrap_or_else(|| determine_spe(class, bl, cu, rng));
            let qty = request.quantity.unwrap_or(1);
            return Some(MkObjResult {
                template_name: name.clone(),
                class: tmpl.class,
                x: request.x,
                y: request.y,
                blessed: bl,
                cursed: cu,
                spe,
                quantity: qty,
                weight: tmpl.weight as u32,
                price: tmpl.cost as u32,
                known: false,
                bknown: false,
                dknown: false,
            });
        }
    }

    // 확률 기반 선택
    let class_items: Vec<(&String, &ItemTemplate)> =
        templates.iter().filter(|(_, t)| t.class == class).collect();
    if class_items.is_empty() {
        return None;
    }

    let total_prob: i32 = class_items.iter().map(|(_, t)| t.prob as i32).sum();
    let mut roll = if total_prob > 0 {
        rng.rn2(total_prob)
    } else {
        0
    };
    let mut selected = &class_items[0];
    for item in &class_items {
        roll -= item.1.prob as i32;
        if roll < 0 {
            selected = item;
            break;
        }
    }

    let (bl, cu) = match (request.blessed, request.cursed) {
        (Some(b), Some(c)) => (b, c),
        _ => determine_buc(rng, request.no_curse),
    };
    let spe = request
        .spe
        .unwrap_or_else(|| determine_spe(class, bl, cu, rng));
    let qty = request.quantity.unwrap_or_else(|| match class {
        ItemClass::Coin => (rng.rn2(100) + 1) as u32,
        ItemClass::Gem | ItemClass::Rock => (rng.rn2(3) + 1) as u32,
        _ => 1,
    });

    Some(MkObjResult {
        template_name: selected.0.clone(),
        class,
        x: request.x,
        y: request.y,
        blessed: bl,
        cursed: cu,
        spe,
        quantity: qty,
        weight: selected.1.weight as u32,
        price: selected.1.cost as u32,
        known: false,
        bknown: false,
        dknown: false,
    })
}

/// 금화 생성 (mkgold 이식)
pub fn mkgold(amount: i32, x: i32, y: i32) -> MkObjResult {
    MkObjResult {
        template_name: "gold piece".into(),
        class: ItemClass::Coin,
        x,
        y,
        blessed: false,
        cursed: false,
        spe: 0,
        quantity: amount.max(1) as u32,
        weight: 1,
        price: 1,
        known: true,
        bknown: true,
        dknown: true,
    }
}

/// 시체 생성 (mkcorpse 이식)
pub fn mkcorpse(monster_name: &str, x: i32, y: i32, _turn: u64) -> MkObjResult {
    MkObjResult {
        template_name: format!("{} corpse", monster_name),
        class: ItemClass::Food,
        x,
        y,
        blessed: false,
        cursed: false,
        spe: 0,
        quantity: 1,
        weight: 100,
        price: 0,
        known: true,
        bknown: false,
        dknown: true,
    }
}

/// 조각상 생성 (mkstatue 이식)
pub fn mkstatue(monster_name: &str, x: i32, y: i32) -> MkObjResult {
    MkObjResult {
        template_name: format!("{} statue", monster_name),
        class: ItemClass::Rock,
        x,
        y,
        blessed: false,
        cursed: false,
        spe: 0,
        quantity: 1,
        weight: 450,
        price: 0,
        known: true,
        bknown: false,
        dknown: true,
    }
}

/// 랜덤 방어구 생성
pub fn random_armor(
    templates: &std::collections::HashMap<String, ItemTemplate>,
    rng: &mut NetHackRng,
) -> Option<MkObjResult> {
    mkobj(
        &MkObjRequest {
            class: Some(ItemClass::Armor),
            ..Default::default()
        },
        templates,
        rng,
    )
}

/// 랜덤 무기 생성
pub fn random_weapon(
    templates: &std::collections::HashMap<String, ItemTemplate>,
    rng: &mut NetHackRng,
) -> Option<MkObjResult> {
    mkobj(
        &MkObjRequest {
            class: Some(ItemClass::Weapon),
            ..Default::default()
        },
        templates,
        rng,
    )
}

/// 난도 기반 아이템 필터 (filter_by_difficulty)
pub fn filter_by_difficulty(
    templates: &std::collections::HashMap<String, ItemTemplate>,
    class: ItemClass,
    dungeon_level: i32,
) -> Vec<String> {
    templates
        .iter()
        .filter(|(_, t)| t.class == class && (t.oc2 as i32) <= dungeon_level)
        .map(|(name, _)| name.clone())
        .collect()
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_item_class() {
        let mut rng = NetHackRng::new(42);
        let cls = random_item_class(&mut rng);
        assert!(matches!(
            cls,
            ItemClass::Weapon
                | ItemClass::Armor
                | ItemClass::Ring
                | ItemClass::Amulet
                | ItemClass::Tool
                | ItemClass::Food
                | ItemClass::Potion
                | ItemClass::Scroll
                | ItemClass::Spellbook
                | ItemClass::Wand
                | ItemClass::Coin
                | ItemClass::Gem
                | ItemClass::Rock
        ));
    }

    #[test]
    fn test_determine_buc() {
        let mut rng = NetHackRng::new(42);
        for _ in 0..1000 {
            let (_, cursed) = determine_buc(&mut rng, true);
            assert!(!cursed);
        }
    }

    #[test]
    fn test_mkgold() {
        let g = mkgold(100, 5, 5);
        assert_eq!(g.quantity, 100);
        assert_eq!(g.class, ItemClass::Coin);
    }
    #[test]
    fn test_mkcorpse() {
        let c = mkcorpse("kobold", 3, 3, 100);
        assert!(c.template_name.contains("kobold"));
        assert_eq!(c.class, ItemClass::Food);
    }

    // [v2.7.0] 확률 테이블 테스트
    #[test]
    fn test_select_class_low() {
        assert_eq!(select_class_from_probs(MKOBJ_PROBS, 0), ItemClass::Weapon);
    }
    #[test]
    fn test_select_class_mid() {
        assert_eq!(select_class_from_probs(MKOBJ_PROBS, 25), ItemClass::Food);
    }
    #[test]
    fn test_select_class_end() {
        assert_eq!(select_class_from_probs(MKOBJ_PROBS, 99), ItemClass::Amulet);
    }
    #[test]
    fn test_rogue_probs() {
        assert_eq!(select_class_from_probs(ROGUE_PROBS, 0), ItemClass::Weapon);
    }
    #[test]
    fn test_hell_probs() {
        assert_eq!(select_class_from_probs(HELL_PROBS, 0), ItemClass::Weapon);
    }
    #[test]
    fn test_box_probs() {
        assert_eq!(select_class_from_probs(BOX_PROBS, 0), ItemClass::Gem);
    }

    #[test]
    fn test_ctx_class() {
        let mut rng = NetHackRng::new(99);
        let _ = random_item_class_ctx(LevelContext::Hell, &mut rng);
        let _ = random_item_class_ctx(LevelContext::Rogue, &mut rng);
    }

    // BUC 관리 테스트
    #[test]
    fn test_bless() {
        let mut b = BucState::uncursed();
        bless_item(&mut b, false);
        assert!(b.blessed);
        assert!(!b.cursed);
    }
    #[test]
    fn test_curse() {
        let mut b = BucState::uncursed();
        curse_item(&mut b, false);
        assert!(b.cursed);
        assert!(!b.blessed);
    }
    #[test]
    fn test_bless_coin() {
        let mut b = BucState::uncursed();
        bless_item(&mut b, true);
        assert!(!b.blessed);
    }
    #[test]
    fn test_unbless() {
        let mut b = BucState::blessed();
        unbless_item(&mut b);
        assert!(!b.blessed);
    }
    #[test]
    fn test_uncurse() {
        let mut b = BucState::cursed();
        uncurse_item(&mut b);
        assert!(!b.cursed);
    }
    #[test]
    fn test_bcsign_blessed() {
        assert_eq!(bcsign(&BucState::blessed()), 1);
    }
    #[test]
    fn test_bcsign_cursed() {
        assert_eq!(bcsign(&BucState::cursed()), -1);
    }
    #[test]
    fn test_bcsign_uncursed() {
        assert_eq!(bcsign(&BucState::uncursed()), 0);
    }
    #[test]
    fn test_blesscurse_rng() {
        let mut b = BucState::uncursed();
        let mut rng = NetHackRng::new(0);
        bless_or_curse(&mut b, 1, &mut rng);
        assert!(b.blessed || b.cursed);
    }

    // 무게 계산 테스트
    #[test]
    fn test_weight_normal() {
        assert_eq!(
            calc_weight(
                10,
                3,
                false,
                false,
                &BucState::uncursed(),
                0,
                false,
                false,
                0
            ),
            30
        );
    }
    #[test]
    fn test_weight_coin() {
        assert_eq!(
            calc_weight(
                0,
                250,
                false,
                false,
                &BucState::uncursed(),
                0,
                true,
                false,
                0
            ),
            3
        );
    }
    #[test]
    fn test_weight_boh_blessed() {
        assert_eq!(
            calc_weight(
                15,
                1,
                true,
                true,
                &BucState::blessed(),
                100,
                false,
                false,
                0
            ),
            15 + 25
        );
    }
    #[test]
    fn test_weight_boh_cursed() {
        assert_eq!(
            calc_weight(15, 1, true, true, &BucState::cursed(), 100, false, false, 0),
            15 + 200
        );
    }
    #[test]
    fn test_weight_container() {
        assert_eq!(
            recalc_container_weight(15, false, &BucState::uncursed(), &[10, 20, 30]),
            75
        );
    }

    // 시체 타이머 테스트
    #[test]
    fn test_corpse_lizard() {
        let mut rng = NetHackRng::new(1);
        assert!(
            start_corpse_timeout(100, 200, true, false, false, false, false, false, &mut rng)
                .is_none()
        );
    }
    #[test]
    fn test_corpse_rider() {
        let mut rng = NetHackRng::new(1);
        let t = start_corpse_timeout(100, 200, false, false, true, false, false, false, &mut rng);
        assert_eq!(t.unwrap().action, CorpseTimerAction::Revive);
    }

    // 얼음 나이 테스트
    #[test]
    fn test_peek_iced() {
        let a = peek_iced_corpse_age(100, 200, true);
        assert!(a > 100);
    }
    #[test]
    fn test_adj_onto_ice() {
        let a = adjust_age_onto_ice(100, 200);
        assert!(a < 100);
    }

    // 재료 속성 테스트
    #[test]
    fn test_flammable_wood() {
        assert!(is_flammable(Material::Wood, false, false));
    }
    #[test]
    fn test_flammable_iron() {
        assert!(!is_flammable(Material::Iron, false, false));
    }
    #[test]
    fn test_rottable_wood() {
        assert!(is_rottable(Material::Wood));
    }
    #[test]
    fn test_rottable_iron() {
        assert!(!is_rottable(Material::Iron));
    }

    // 기타 테스트
    #[test]
    fn test_treefruit() {
        let mut rng = NetHackRng::new(7);
        let f = random_treefruit(&mut rng);
        assert!(TREE_FRUITS.contains(&f));
    }
    #[test]
    fn test_box_max() {
        assert_eq!(box_max_contents("ice box", false, false), 20);
        assert_eq!(box_max_contents("sack", false, true), 0);
    }
    #[test]
    fn test_horn_empty() {
        let mut rng = NetHackRng::new(1);
        let h = horn_of_plenty(0, &mut rng);
        assert!(!h.produced);
    }
    #[test]
    fn test_horn_use() {
        let mut rng = NetHackRng::new(1);
        let h = horn_of_plenty(5, &mut rng);
        assert!(h.produced);
    }
    #[test]
    fn test_split() {
        let s = split_stack(10, 3, 5).unwrap();
        assert_eq!(s.original_qty, 7);
        assert_eq!(s.split_weight, 15);
    }
    #[test]
    fn test_split_invalid() {
        assert!(split_stack(5, 5, 10).is_none());
        assert!(split_stack(5, 0, 10).is_none());
    }
    #[test]
    fn test_glob_absorb() {
        let g = glob_absorb(20, 100, 0, 30, 110, 0, 200);
        assert_eq!(g.merged_weight, 50);
        assert_eq!(g.merged_eaten, 0);
    }
    #[test]
    fn test_alteration_verb() {
        assert_eq!(alteration_verb(AlterationType::Cancel), "cancel");
        assert_eq!(alteration_verb(AlterationType::Rust), "rust");
    }
    #[test]
    fn test_mkstatue() {
        let s = mkstatue("orc", 1, 2);
        assert_eq!(s.class, ItemClass::Rock);
        assert_eq!(s.weight, 450);
    }
}

// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// 원본: nethack-3.6.7/src/muse.c (2,608줄)
// 몬스터 아이템 사용 AI — 방어/공격/기타 아이템 선택 및 사용
// ============================================================================

use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
// [v2.9.4] 몬스터 아이템 사용 타입 정의 (원본: muse.c L41-56)
// =============================================================================

/// [v2.9.4] 방어 아이템 사용 유형 (원본: muse.c L275-300 #define MUSE_*)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenseUse {
    /// 텔레포트 두루마리 읽기
    ScrollTeleport,
    /// 텔레포트 마법봉 (자신에게)
    WandTeleportSelf,
    /// 치유 물약
    PotionHealing,
    /// 고급 치유 물약
    PotionExtraHealing,
    /// 완전 치유 물약
    PotionFullHealing,
    /// 굴착 마법봉 (아래로 도주)
    WandDigging,
    /// 함정문 이용 도주
    Trapdoor,
    /// 텔레포트 함정 이용
    TeleportTrap,
    /// 위층 계단 이용
    Upstairs,
    /// 아래층 계단 이용
    Downstairs,
    /// 몬스터 생성 마법봉
    WandCreateMonster,
    /// 몬스터 생성 두루마리
    ScrollCreateMonster,
    /// 위층 사다리
    UpLadder,
    /// 아래층 사다리
    DownLadder,
    /// 특수 계단
    SecretStairs,
    /// 텔레포트 마법봉 (타인에게)
    WandTeleportOther,
    /// 나팔 (병사 기상)
    Bugle,
    /// 유니콘 뿔 (상태이상 해제)
    UnicornHorn,
    /// 도마뱀 시체 (석화 해제)
    LizardCorpse,
}

/// [v2.9.4] 공격 아이템 사용 유형 (원본: muse.c L1061-1077)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffenseUse {
    /// 죽음의 마법봉
    WandDeath,
    /// 수면 마법봉
    WandSleep,
    /// 불꽃 마법봉
    WandFire,
    /// 냉기 마법봉
    WandCold,
    /// 번개 마법봉
    WandLightning,
    /// 매직 미사일 마법봉
    WandMagicMissile,
    /// 타격 마법봉
    WandStriking,
    /// 불꽃 두루마리
    ScrollFire,
    /// 마비 물약 (투척)
    PotionParalysis,
    /// 실명 물약 (투척)
    PotionBlindness,
    /// 혼란 물약 (투척)
    PotionConfusion,
    /// 냉기의 뿔피리
    FrostHorn,
    /// 불꽃의 뿔피리
    FireHorn,
    /// 산성 물약 (투척)
    PotionAcid,
    /// 수면 물약 (투척)
    PotionSleeping,
    /// 대지 두루마리
    ScrollEarth,
}

/// [v2.9.4] 기타 아이템 사용 유형 (원본: muse.c L1600+)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiscUse {
    /// 변신 마법봉
    WandPolymorph,
    /// 변신 물약
    PotionPolymorph,
    /// 속도 물약
    PotionSpeed,
    /// 투명 물약
    PotionInvisibility,
    /// 레벨업 물약
    PotionGainLevel,
    /// 저주 해제 두루마리
    ScrollRemoveCurse,
}

/// [v2.9.4] 몬스터의 아이템 사용 결정 결과
#[derive(Debug, Clone)]
pub struct MuseDecision {
    /// 방어 행동 (위기 시)
    pub defense: Option<DefenseUse>,
    /// 공격 행동
    pub offense: Option<OffenseUse>,
    /// 기타 행동
    pub misc: Option<MiscUse>,
}

/// [v2.9.4] 아이템 사용 결과
#[derive(Debug, Clone)]
pub struct UseResult {
    /// 아이템이 소모되었는지
    pub item_consumed: bool,
    /// 몬스터가 사망했는지 (역폭발 등)
    pub monster_died: bool,
    /// 몬스터가 텔레포트했는지
    pub monster_teleported: bool,
    /// HP 회복량
    pub hp_healed: i32,
    /// 소환된 몬스터 수
    pub monsters_summoned: i32,
    /// 플레이어에게 가한 데미지
    pub damage_to_player: i32,
    /// 로그 메시지
    pub message: String,
}

// =============================================================================
// [v2.9.4] 몬스터 상태 정보 (아이템 사용 판단용)
// =============================================================================

/// [v2.9.4] 아이템 사용 판단에 필요한 몬스터 정보 (원본: 전역 변수 → 구조체)
#[derive(Debug, Clone)]
pub struct MonsterUseContext {
    /// 몬스터 이름
    pub name: String,
    /// 현재 HP
    pub hp: i32,
    /// 최대 HP
    pub hp_max: i32,
    /// 몬스터 레벨
    pub level: i32,
    /// 위치 (x, y)
    pub pos: (i32, i32),
    /// 플레이어까지 거리 제곱
    pub dist_sq_to_player: i32,
    /// 혼란 상태인지
    pub confused: bool,
    /// 기절 상태인지
    pub stunned: bool,
    /// 실명 상태인지
    pub blind: bool,
    /// 평화적인지
    pub peaceful: bool,
    /// 도주 중인지
    pub fleeing: bool,
    /// 동물인지 (아이템 사용 불가)
    pub is_animal: bool,
    /// 지능 없는 몬스터인지
    pub is_mindless: bool,
    /// 손이 없는지
    pub no_hands: bool,
    /// 비행 가능한지
    pub is_floater: bool,
    /// 상점 주인인지
    pub is_shopkeeper: bool,
    /// 경비원인지
    pub is_guard: bool,
    /// 사제인지
    pub is_priest: bool,
    /// 용병인지
    pub is_mercenary: bool,
    /// 유니콘인지
    pub is_unicorn: bool,
    /// 역병 기사인지 (Pestilence — 치유 역전)
    pub is_pestilence: bool,
}

/// [v2.9.4] 몬스터 인벤토리 아이템 정보
#[derive(Debug, Clone)]
pub struct MonsterItem {
    /// 아이템 종류 문자열
    pub kind: String,
    /// 충전 수 (마법봉)
    pub charges: i32,
    /// 저주 여부
    pub cursed: bool,
    /// 축복 여부
    pub blessed: bool,
    /// 시체인 경우 종류
    pub corpse_type: Option<String>,
}

// =============================================================================
// [v2.9.4] 핵심 함수: 방어 아이템 탐색 (원본: find_defensive, L327-622)
// =============================================================================

/// HP 임계값 상수 (원본 매직넘버 분리)
const HEAL_HP_THRESHOLD: i32 = 10;
/// 방어 탐색 거리 제한 (원본: dist2 > 25)
const DEFENSE_SEARCH_DIST_SQ: i32 = 25;

/// [v2.9.4] 방어 아이템/행동 탐색 (원본: find_defensive)
pub fn find_defensive(
    ctx: &MonsterUseContext,
    inventory: &[MonsterItem],
    on_stairs: bool,
    on_ladder: bool,
    near_trapdoor: bool,
    near_teleport_trap: bool,
    player_level: i32,
    _rng: &mut NetHackRng,
) -> Option<DefenseUse> {
    // 동물이나 지능 없는 몬스터는 아이템 사용 불가
    if ctx.is_animal || ctx.is_mindless {
        return None;
    }
    if ctx.dist_sq_to_player > DEFENSE_SEARCH_DIST_SQ {
        return None;
    }

    // --- 1단계: 상태이상 해제 (유니콘 뿔 / 도마뱀 시체) ---
    if ctx.confused || ctx.stunned || ctx.blind {
        if ctx.is_unicorn {
            return Some(DefenseUse::UnicornHorn);
        }
        if !ctx.no_hands {
            for item in inventory {
                if item.kind == "unicorn horn" && !item.cursed {
                    return Some(DefenseUse::UnicornHorn);
                }
            }
        }
    }
    if ctx.confused || ctx.stunned {
        for item in inventory {
            if item.kind == "corpse" {
                if let Some(ref ct) = item.corpse_type {
                    if ct == "lizard" {
                        return Some(DefenseUse::LizardCorpse);
                    }
                }
            }
        }
    }

    // --- 2단계: 실명 치유 ---
    if ctx.blind && !ctx.no_hands && !ctx.is_pestilence {
        if let Some(heal) = find_healing_potion(inventory) {
            return Some(heal);
        }
    }

    // --- 3단계: HP 위험도 판정 ---
    let fraction = if player_level < 10 {
        5
    } else if player_level < 14 {
        4
    } else {
        3
    };
    let hp_critical =
        ctx.hp < ctx.hp_max && (ctx.hp < HEAL_HP_THRESHOLD || ctx.hp * fraction < ctx.hp_max);
    if !hp_critical {
        return None;
    }

    // 평화적 몬스터: 치유만
    if ctx.peaceful {
        if !ctx.no_hands {
            return find_healing_potion(inventory);
        }
        return None;
    }

    // --- 4단계: 물리적 도주 ---
    if on_stairs {
        return Some(DefenseUse::Upstairs);
    }
    if on_ladder {
        return Some(DefenseUse::UpLadder);
    }
    if near_trapdoor && !ctx.is_floater && !ctx.is_shopkeeper && !ctx.is_guard && !ctx.is_priest {
        return Some(DefenseUse::Trapdoor);
    }
    if near_teleport_trap {
        return Some(DefenseUse::TeleportTrap);
    }

    // --- 5단계: 아이템 기반 도주/방어 ---
    if ctx.no_hands {
        return None;
    }

    for item in inventory {
        match item.kind.as_str() {
            "wand of teleportation" if item.charges > 0 => {
                return Some(DefenseUse::WandTeleportSelf);
            }
            "scroll of teleportation" if !ctx.blind => {
                return Some(DefenseUse::ScrollTeleport);
            }
            "wand of digging"
                if item.charges > 0
                    && !ctx.is_floater
                    && !ctx.is_shopkeeper
                    && !ctx.is_guard
                    && !ctx.is_priest =>
            {
                return Some(DefenseUse::WandDigging);
            }
            "wand of create monster" if item.charges > 0 && !ctx.is_pestilence => {
                return Some(DefenseUse::WandCreateMonster);
            }
            "scroll of create monster" => {
                return Some(DefenseUse::ScrollCreateMonster);
            }
            _ => {}
        }
    }

    if !ctx.is_pestilence {
        if let Some(heal) = find_healing_potion(inventory) {
            return Some(heal);
        }
    }
    None
}

/// [v2.9.4] 인벤토리에서 최상급 치유 물약 탐색 (원본: m_use_healing)
fn find_healing_potion(inventory: &[MonsterItem]) -> Option<DefenseUse> {
    for item in inventory {
        if item.kind == "potion of full healing" {
            return Some(DefenseUse::PotionFullHealing);
        }
    }
    for item in inventory {
        if item.kind == "potion of extra healing" {
            return Some(DefenseUse::PotionExtraHealing);
        }
    }
    for item in inventory {
        if item.kind == "potion of healing" {
            return Some(DefenseUse::PotionHealing);
        }
    }
    None
}

// =============================================================================
// [v2.9.4] 방어 아이템 사용 (원본: use_defensive, L628-1012)
// =============================================================================

/// [v2.9.4] 방어 행동 실행
pub fn use_defensive(
    defense: DefenseUse,
    ctx: &mut MonsterUseContext,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> UseResult {
    match defense {
        DefenseUse::UnicornHorn => {
            let msg = if ctx.is_unicorn {
                format!("{}의 뿔 끝이 빛난다!", ctx.name)
            } else {
                format!("{}이(가) 유니콘 뿔을 사용한다!", ctx.name)
            };
            ctx.confused = false;
            ctx.stunned = false;
            ctx.blind = false;
            log.add(&msg, turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
        DefenseUse::PotionHealing => {
            let heal = rng.d(6, 4);
            ctx.hp = (ctx.hp + heal).min(ctx.hp_max + 1);
            if ctx.hp > ctx.hp_max {
                ctx.hp_max = ctx.hp;
            }
            let msg = format!("{}이(가) 치유 물약을 마신다. (HP +{})", ctx.name, heal);
            log.add(&msg, turn);
            UseResult {
                item_consumed: true,
                monster_died: false,
                monster_teleported: false,
                hp_healed: heal,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
        DefenseUse::PotionExtraHealing => {
            let heal = rng.d(6, 8);
            ctx.hp = (ctx.hp + heal).min(ctx.hp_max + 2);
            if ctx.hp > ctx.hp_max {
                ctx.hp_max = ctx.hp;
            }
            let msg = format!("{}이(가) 고급 치유 물약을 마신다. (HP +{})", ctx.name, heal);
            log.add(&msg, turn);
            UseResult {
                item_consumed: true,
                monster_died: false,
                monster_teleported: false,
                hp_healed: heal,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
        DefenseUse::PotionFullHealing => {
            let bonus = 4;
            ctx.hp_max += bonus;
            ctx.hp = ctx.hp_max;
            let msg = format!(
                "{}이(가) 완전 치유 물약을 마신다! (최대HP +{}, 완전 회복)",
                ctx.name, bonus
            );
            log.add_colored(&msg, [0, 255, 100], turn);
            UseResult {
                item_consumed: true,
                monster_died: false,
                monster_teleported: false,
                hp_healed: ctx.hp_max,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
        DefenseUse::WandTeleportSelf | DefenseUse::ScrollTeleport | DefenseUse::TeleportTrap => {
            let msg = format!("{}이(가) 텔레포트했다!", ctx.name);
            log.add(&msg, turn);
            UseResult {
                item_consumed: matches!(defense, DefenseUse::ScrollTeleport),
                monster_died: false,
                monster_teleported: true,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
        DefenseUse::WandDigging => {
            let msg = format!(
                "{}이(가) 굴착 마법봉으로 바닥에 구멍을 뚫고 도주한다!",
                ctx.name
            );
            log.add(&msg, turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: true,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
        DefenseUse::Upstairs
        | DefenseUse::Downstairs
        | DefenseUse::UpLadder
        | DefenseUse::DownLadder
        | DefenseUse::SecretStairs
        | DefenseUse::Trapdoor => {
            let direction = match defense {
                DefenseUse::Upstairs | DefenseUse::UpLadder => "위",
                _ => "아래",
            };
            let msg = format!("{}이(가) {}층으로 도주한다!", ctx.name, direction);
            log.add(&msg, turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: true,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
        DefenseUse::WandCreateMonster | DefenseUse::ScrollCreateMonster => {
            let count = if defense == DefenseUse::ScrollCreateMonster {
                1 + rng.rn2(4)
            } else {
                1
            };
            let msg = format!("{}이(가) 몬스터를 소환한다! ({}마리)", ctx.name, count);
            log.add_colored(&msg, [255, 100, 100], turn);
            UseResult {
                item_consumed: matches!(defense, DefenseUse::ScrollCreateMonster),
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: count,
                damage_to_player: 0,
                message: msg,
            }
        }
        DefenseUse::Bugle => {
            let msg = format!("{}이(가) 나팔을 분다! 병사들이 깨어난다!", ctx.name);
            log.add_colored(&msg, [255, 200, 0], turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
        DefenseUse::LizardCorpse => {
            ctx.confused = false;
            ctx.stunned = false;
            let msg = format!("{}이(가) 도마뱀 시체를 먹는다. 상태가 안정된다.", ctx.name);
            log.add(&msg, turn);
            UseResult {
                item_consumed: true,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
        DefenseUse::WandTeleportOther => {
            let msg = format!("{}이(가) 텔레포트 마법봉을 당신에게 쏜다!", ctx.name);
            log.add_colored(&msg, [255, 100, 255], turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
    }
}

// =============================================================================
// [v2.9.4] 공격 아이템 탐색 (원본: find_offensive, L1082-1250)
// =============================================================================

/// [v2.9.4] 공격 아이템 탐색
pub fn find_offensive(
    ctx: &MonsterUseContext,
    inventory: &[MonsterItem],
    player_has_reflection: bool,
    _rng: &mut NetHackRng,
) -> Option<OffenseUse> {
    if ctx.peaceful || ctx.is_animal || ctx.is_mindless || ctx.no_hands {
        return None;
    }
    let skip_beams = player_has_reflection;

    for item in inventory {
        match item.kind.as_str() {
            "wand of death" if item.charges > 0 && !skip_beams => {
                return Some(OffenseUse::WandDeath);
            }
            "wand of sleep" if item.charges > 0 && !skip_beams => {
                return Some(OffenseUse::WandSleep);
            }
            "wand of fire" if item.charges > 0 && !skip_beams => {
                return Some(OffenseUse::WandFire);
            }
            "wand of cold" if item.charges > 0 && !skip_beams => {
                return Some(OffenseUse::WandCold);
            }
            "wand of lightning" if item.charges > 0 && !skip_beams => {
                return Some(OffenseUse::WandLightning);
            }
            "wand of magic missile" if item.charges > 0 && !skip_beams => {
                return Some(OffenseUse::WandMagicMissile);
            }
            "wand of striking" if item.charges > 0 => {
                return Some(OffenseUse::WandStriking);
            }
            "scroll of fire" if !ctx.blind => {
                return Some(OffenseUse::ScrollFire);
            }
            "fire horn" if item.charges > 0 && !skip_beams => {
                return Some(OffenseUse::FireHorn);
            }
            "frost horn" if item.charges > 0 && !skip_beams => {
                return Some(OffenseUse::FrostHorn);
            }
            "potion of paralysis" if ctx.dist_sq_to_player <= 16 => {
                return Some(OffenseUse::PotionParalysis);
            }
            "potion of blindness" if ctx.dist_sq_to_player <= 16 => {
                return Some(OffenseUse::PotionBlindness);
            }
            "potion of confusion" if ctx.dist_sq_to_player <= 16 => {
                return Some(OffenseUse::PotionConfusion);
            }
            "potion of sleeping" if ctx.dist_sq_to_player <= 16 => {
                return Some(OffenseUse::PotionSleeping);
            }
            "potion of acid" if ctx.dist_sq_to_player <= 16 => {
                return Some(OffenseUse::PotionAcid);
            }
            "scroll of earth" if !ctx.blind => {
                return Some(OffenseUse::ScrollEarth);
            }
            _ => {}
        }
    }
    None
}

// =============================================================================
// [v2.9.4] 공격 아이템 사용 (원본: use_offensive, L1250-1590)
// =============================================================================

/// [v2.9.4] 공격 행동 실행
pub fn use_offensive(
    offense: OffenseUse,
    ctx: &MonsterUseContext,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> UseResult {
    match offense {
        OffenseUse::WandDeath => {
            let msg = format!("{}이(가) 죽음의 마법봉을 발사한다!", ctx.name);
            log.add_colored(&msg, [255, 0, 0], turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: 999,
                message: msg,
            }
        }
        OffenseUse::WandFire | OffenseUse::FireHorn => {
            let dmg = rng.d(6, 6);
            let src = if matches!(offense, OffenseUse::FireHorn) {
                "불꽃의 뿔피리"
            } else {
                "불꽃 마법봉"
            };
            let msg = format!("{}이(가) {}을(를) 발사한다! ({}dmg)", ctx.name, src, dmg);
            log.add_colored(&msg, [255, 100, 0], turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: dmg,
                message: msg,
            }
        }
        OffenseUse::WandCold | OffenseUse::FrostHorn => {
            let dmg = rng.d(6, 6);
            let src = if matches!(offense, OffenseUse::FrostHorn) {
                "냉기의 뿔피리"
            } else {
                "냉기 마법봉"
            };
            let msg = format!("{}이(가) {}을(를) 발사한다! ({}dmg)", ctx.name, src, dmg);
            log.add_colored(&msg, [100, 150, 255], turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: dmg,
                message: msg,
            }
        }
        OffenseUse::WandLightning => {
            let dmg = rng.d(6, 6);
            let msg = format!("{}이(가) 번개 마법봉을 발사한다! ({}dmg)", ctx.name, dmg);
            log.add_colored(&msg, [255, 255, 0], turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: dmg,
                message: msg,
            }
        }
        OffenseUse::WandSleep => {
            let msg = format!("{}이(가) 수면 마법봉을 발사한다!", ctx.name);
            log.add_colored(&msg, [100, 100, 255], turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: 0,
                message: msg,
            }
        }
        OffenseUse::WandMagicMissile => {
            let dmg = rng.d(2, 6);
            let msg = format!(
                "{}이(가) 매직 미사일 마법봉을 발사한다! ({}dmg)",
                ctx.name, dmg
            );
            log.add_colored(&msg, [200, 0, 255], turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: dmg,
                message: msg,
            }
        }
        OffenseUse::WandStriking => {
            let dmg = rng.d(2, 12);
            let msg = format!("{}이(가) 타격 마법봉을 발사한다! ({}dmg)", ctx.name, dmg);
            log.add(&msg, turn);
            UseResult {
                item_consumed: false,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: dmg,
                message: msg,
            }
        }
        OffenseUse::ScrollFire => {
            let dmg = rng.d(4, 6) + ctx.level;
            let msg = format!("{}이(가) 불꽃 두루마리를 읽는다! ({}dmg)", ctx.name, dmg);
            log.add_colored(&msg, [255, 80, 0], turn);
            UseResult {
                item_consumed: true,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: dmg,
                message: msg,
            }
        }
        OffenseUse::ScrollEarth => {
            let dmg = rng.d(3, 6);
            let msg = format!(
                "{}이(가) 대지 두루마리를 읽는다! 바위가 떨어진다! ({}dmg)",
                ctx.name, dmg
            );
            log.add_colored(&msg, [150, 100, 50], turn);
            UseResult {
                item_consumed: true,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: dmg,
                message: msg,
            }
        }
        OffenseUse::PotionParalysis
        | OffenseUse::PotionBlindness
        | OffenseUse::PotionConfusion
        | OffenseUse::PotionSleeping
        | OffenseUse::PotionAcid => {
            let (effect, dmg) = match offense {
                OffenseUse::PotionAcid => ("산성", rng.d(2, 6)),
                OffenseUse::PotionParalysis => ("마비", 0),
                OffenseUse::PotionBlindness => ("실명", 0),
                OffenseUse::PotionConfusion => ("혼란", 0),
                _ => ("수면", 0),
            };
            let msg = format!("{}이(가) {} 물약을 당신에게 던진다!", ctx.name, effect);
            log.add_colored(&msg, [255, 150, 0], turn);
            UseResult {
                item_consumed: true,
                monster_died: false,
                monster_teleported: false,
                hp_healed: 0,
                monsters_summoned: 0,
                damage_to_player: dmg,
                message: msg,
            }
        }
    }
}

// =============================================================================
// [v2.9.4] 기타 아이템 탐색/사용 (원본: find_misc/use_misc, L1600-2100)
// =============================================================================

/// [v2.9.4] 기타 아이템 탐색
pub fn find_misc(
    ctx: &MonsterUseContext,
    inventory: &[MonsterItem],
    _rng: &mut NetHackRng,
) -> Option<MiscUse> {
    if ctx.is_animal || ctx.is_mindless || ctx.no_hands {
        return None;
    }
    for item in inventory {
        match item.kind.as_str() {
            "wand of polymorph" if item.charges > 0 => return Some(MiscUse::WandPolymorph),
            "potion of polymorph" => return Some(MiscUse::PotionPolymorph),
            "potion of speed" => return Some(MiscUse::PotionSpeed),
            "potion of invisibility" => return Some(MiscUse::PotionInvisibility),
            "potion of gain level" => return Some(MiscUse::PotionGainLevel),
            _ => {}
        }
    }
    None
}

/// [v2.9.4] 기타 아이템 사용
pub fn use_misc(
    misc: MiscUse,
    ctx: &MonsterUseContext,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> UseResult {
    let msg = match misc {
        MiscUse::WandPolymorph | MiscUse::PotionPolymorph => {
            format!("{}이(가) 변신한다!", ctx.name)
        }
        MiscUse::PotionSpeed => format!("{}이(가) 속도 물약을 마신다! 빨라진다!", ctx.name),
        MiscUse::PotionInvisibility => format!("{}이(가) 투명해졌다!", ctx.name),
        MiscUse::PotionGainLevel => format!("{}이(가) 레벨업 물약을 마신다!", ctx.name),
        MiscUse::ScrollRemoveCurse => format!("{}이(가) 저주 해제 두루마리를 읽는다!", ctx.name),
    };
    log.add(&msg, turn);
    UseResult {
        item_consumed: !matches!(misc, MiscUse::WandPolymorph),
        monster_died: false,
        monster_teleported: false,
        hp_healed: 0,
        monsters_summoned: 0,
        damage_to_player: 0,
        message: msg,
    }
}

// =============================================================================
// [v2.9.4] 마법봉 역폭발 판정 (원본: precheck, L62-165)
// =============================================================================

/// 저주받은 마법봉 역폭발 확률 (원본: WAND_BACKFIRE_CHANCE)
const WAND_BACKFIRE_CHANCE: i32 = 100;

/// [v2.9.4] 마법봉 사전 검사: 저주 시 역폭발 가능
pub fn precheck_wand(
    item: &MonsterItem,
    ctx: &mut MonsterUseContext,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> Option<UseResult> {
    if !item.cursed {
        return None;
    }
    if rng.rn2(WAND_BACKFIRE_CHANCE) != 0 {
        return None;
    }
    let dam = rng.d(item.charges + 2, 6);
    ctx.hp -= dam;
    let died = ctx.hp <= 0;
    let msg = format!(
        "{}이(가) 마법봉을 쏘지만, 폭발한다! ({}dmg{})",
        ctx.name,
        dam,
        if died { " - 사망!" } else { "" }
    );
    log.add_colored(&msg, [255, 0, 0], turn);
    Some(UseResult {
        item_consumed: true,
        monster_died: died,
        monster_teleported: false,
        hp_healed: 0,
        monsters_summoned: 0,
        damage_to_player: 0,
        message: msg,
    })
}

// =============================================================================
// [v2.9.4] 방어/공격 아이템 랜덤 생성 (원본: rnd_defensive_item/rnd_offensive_item)
// =============================================================================

/// [v2.9.4] 몬스터 생성 시 방어 아이템 결정
pub fn rnd_defensive_item(
    difficulty: i32,
    is_floater: bool,
    rng: &mut NetHackRng,
) -> Option<&'static str> {
    let extra = (difficulty > 3) as i32 + (difficulty > 6) as i32 + (difficulty > 8) as i32;
    match rng.rn2(8 + extra) {
        0 | 1 => Some("scroll of teleportation"),
        2 => Some("scroll of create monster"),
        3 => Some("potion of healing"),
        4 => Some("potion of extra healing"),
        5 => Some("potion of full healing"),
        6 | 9 => Some("wand of teleportation"),
        7 => {
            if is_floater {
                None
            } else {
                Some("wand of digging")
            }
        }
        8 | 10 => Some("wand of create monster"),
        _ => None,
    }
}

/// [v2.9.4] 몬스터 생성 시 공격 아이템 결정
pub fn rnd_offensive_item(difficulty: i32, rng: &mut NetHackRng) -> Option<&'static str> {
    let extra = (difficulty > 3) as i32 + (difficulty > 6) as i32 + (difficulty > 8) as i32;
    match rng.rn2(8 + extra) {
        0 => Some("potion of paralysis"),
        1 => Some("potion of blindness"),
        2 => Some("potion of confusion"),
        3 => Some("potion of sleeping"),
        4 => Some("potion of acid"),
        5 | 6 => Some("scroll of fire"),
        7 => Some("wand of magic missile"),
        8 => Some("wand of sleep"),
        9 => Some("wand of fire"),
        10 => Some("wand of cold"),
        _ => None,
    }
}

// =============================================================================
// [v2.9.4] 통계 추적
// =============================================================================

/// [v2.9.4] 몬스터 아이템 사용 통계
#[derive(Debug, Clone, Default)]
pub struct MuseStatistics {
    pub defensive_uses: u32,
    pub offensive_uses: u32,
    pub misc_uses: u32,
    pub wand_backfires: u32,
    pub total_hp_healed: i64,
    pub total_damage_dealt: i64,
}

impl MuseStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_defense(&mut self, result: &UseResult) {
        self.defensive_uses += 1;
        self.total_hp_healed += result.hp_healed as i64;
    }
    pub fn record_offense(&mut self, result: &UseResult) {
        self.offensive_uses += 1;
        self.total_damage_dealt += result.damage_to_player as i64;
    }
    pub fn record_misc(&mut self) {
        self.misc_uses += 1;
    }
    pub fn record_backfire(&mut self) {
        self.wand_backfires += 1;
    }
}

// =============================================================================
// [v2.9.4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx() -> MonsterUseContext {
        MonsterUseContext {
            name: "goblin".to_string(),
            hp: 5,
            hp_max: 20,
            level: 3,
            pos: (10, 10),
            dist_sq_to_player: 4,
            confused: false,
            stunned: false,
            blind: false,
            peaceful: false,
            fleeing: false,
            is_animal: false,
            is_mindless: false,
            no_hands: false,
            is_floater: false,
            is_shopkeeper: false,
            is_guard: false,
            is_priest: false,
            is_mercenary: false,
            is_unicorn: false,
            is_pestilence: false,
        }
    }

    fn make_item(kind: &str) -> MonsterItem {
        MonsterItem {
            kind: kind.to_string(),
            charges: 5,
            cursed: false,
            blessed: false,
            corpse_type: None,
        }
    }

    #[test]
    fn test_find_defensive_healing() {
        let ctx = make_ctx();
        let inv = vec![make_item("potion of healing")];
        let mut rng = NetHackRng::new(42);
        let result = find_defensive(&ctx, &inv, false, false, false, false, 10, &mut rng);
        assert_eq!(result, Some(DefenseUse::PotionHealing));
    }

    #[test]
    fn test_find_defensive_full_healing_priority() {
        let ctx = make_ctx();
        let inv = vec![
            make_item("potion of healing"),
            make_item("potion of full healing"),
        ];
        let mut rng = NetHackRng::new(42);
        let result = find_defensive(&ctx, &inv, false, false, false, false, 10, &mut rng);
        assert_eq!(result, Some(DefenseUse::PotionFullHealing));
    }

    #[test]
    fn test_find_defensive_unicorn_horn() {
        let mut ctx = make_ctx();
        ctx.confused = true;
        ctx.hp = 20;
        let inv = vec![make_item("unicorn horn")];
        let mut rng = NetHackRng::new(42);
        let result = find_defensive(&ctx, &inv, false, false, false, false, 10, &mut rng);
        assert_eq!(result, Some(DefenseUse::UnicornHorn));
    }

    #[test]
    fn test_find_defensive_animal_no_use() {
        let mut ctx = make_ctx();
        ctx.is_animal = true;
        let inv = vec![make_item("potion of healing")];
        let mut rng = NetHackRng::new(42);
        let result = find_defensive(&ctx, &inv, false, false, false, false, 10, &mut rng);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_defensive_peaceful_no_flee() {
        let mut ctx = make_ctx();
        ctx.peaceful = true;
        let inv = vec![make_item("potion of healing"), make_item("wand of digging")];
        let mut rng = NetHackRng::new(42);
        let result = find_defensive(&ctx, &inv, false, false, false, false, 10, &mut rng);
        assert_eq!(result, Some(DefenseUse::PotionHealing));
    }

    #[test]
    fn test_find_offensive_wand_death() {
        let ctx = make_ctx();
        let inv = vec![make_item("wand of death")];
        let mut rng = NetHackRng::new(42);
        let result = find_offensive(&ctx, &inv, false, &mut rng);
        assert_eq!(result, Some(OffenseUse::WandDeath));
    }

    #[test]
    fn test_find_offensive_reflection_skip() {
        let ctx = make_ctx();
        let inv = vec![make_item("wand of death")];
        let mut rng = NetHackRng::new(42);
        let result = find_offensive(&ctx, &inv, true, &mut rng);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_offensive_peaceful_no_attack() {
        let mut ctx = make_ctx();
        ctx.peaceful = true;
        let inv = vec![make_item("wand of fire")];
        let mut rng = NetHackRng::new(42);
        let result = find_offensive(&ctx, &inv, false, &mut rng);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_misc_polymorph() {
        let ctx = make_ctx();
        let inv = vec![make_item("wand of polymorph")];
        let mut rng = NetHackRng::new(42);
        let result = find_misc(&ctx, &inv, &mut rng);
        assert_eq!(result, Some(MiscUse::WandPolymorph));
    }

    #[test]
    fn test_use_defensive_healing() {
        let mut ctx = make_ctx();
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = use_defensive(DefenseUse::PotionHealing, &mut ctx, &mut rng, &mut log, 1);
        assert!(result.item_consumed);
        assert!(result.hp_healed > 0);
        assert!(ctx.hp > 5);
    }

    #[test]
    fn test_use_defensive_teleport() {
        let mut ctx = make_ctx();
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = use_defensive(
            DefenseUse::WandTeleportSelf,
            &mut ctx,
            &mut rng,
            &mut log,
            1,
        );
        assert!(result.monster_teleported);
        assert!(!result.item_consumed);
    }

    #[test]
    fn test_precheck_wand_normal() {
        let item = make_item("wand of fire");
        let mut ctx = make_ctx();
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = precheck_wand(&item, &mut ctx, &mut rng, &mut log, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_rnd_defensive_item() {
        let mut rng = NetHackRng::new(42);
        let item = rnd_defensive_item(5, false, &mut rng);
        assert!(item.is_some());
    }

    #[test]
    fn test_statistics() {
        let mut stats = MuseStatistics::new();
        let result = UseResult {
            item_consumed: true,
            monster_died: false,
            monster_teleported: false,
            hp_healed: 15,
            monsters_summoned: 0,
            damage_to_player: 0,
            message: String::new(),
        };
        stats.record_defense(&result);
        assert_eq!(stats.defensive_uses, 1);
        assert_eq!(stats.total_hp_healed, 15);
    }

    #[test]
    fn test_use_offensive_fire() {
        let ctx = make_ctx();
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = use_offensive(OffenseUse::WandFire, &ctx, &mut rng, &mut log, 1);
        assert!(result.damage_to_player > 0);
        assert!(!result.item_consumed);
    }

    #[test]
    fn test_lizard_corpse_defense() {
        let mut ctx = make_ctx();
        ctx.confused = true;
        ctx.stunned = true;
        ctx.hp = 20;
        let mut inv_item = make_item("corpse");
        inv_item.corpse_type = Some("lizard".to_string());
        let inv = vec![inv_item];
        let mut rng = NetHackRng::new(42);
        let result = find_defensive(&ctx, &inv, false, false, false, false, 10, &mut rng);
        assert_eq!(result, Some(DefenseUse::LizardCorpse));
    }
}

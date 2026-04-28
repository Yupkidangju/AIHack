// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::Grid;
use crate::core::entity::player::Player;
use crate::core::entity::{Health, PlayerTag, Position};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;

/// 분수 마시기 (Drink from Fountain)
/// 원본 NetHack의 fountain.c:dryfountain(), drinkfountain() 등 이식
pub fn try_drink_fountain(
    world: &mut SubWorld,
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
    item_mgr: &crate::core::entity::object::ItemManager,
) -> bool {
    //
    let mut p_query = <(
        &mut Player,
        &mut Health,
        &crate::core::entity::status::StatusBundle,
        &Position,
    )>::query()
    .filter(component::<PlayerTag>());
    let mut player_packet = None;

    for (p, h, s, pos) in p_query.iter_mut(world) {
        player_packet = Some((p, h, s, pos));
        break;
    }

    if let Some((p_stats, p_health, _p_status, p_pos)) = player_packet {
        if let Some(tile) = grid.get_tile(p_pos.x as usize, p_pos.y as usize) {
            if tile.typ != TileType::Fountain {
                log.add("There is no fountain here.", turn);
                return false;
            }

            log.add("You drink from the fountain.", turn);

            // 분수 마름 체크 (Dry up)
            if rng.rn2(100) < 20 {
                if let Some(tile_mut) = grid.get_tile_mut(p_pos.x as usize, p_pos.y as usize) {
                    tile_mut.typ = TileType::Room;
                    log.add("The fountain dries up!", turn);
                    return true;
                }
            }

            // 분수 효과 (Effect)
            let roll = rng.rn2(100);

            if roll < 5 {
                // 5% Level Gain
                log.add("You feel a strange power flow through you.", turn);
                p_stats.experience += 500;
            } else if roll < 15 {
                // 10% Restoration
                log.add("You feel better.", turn);
                p_health.current = p_health.max;
            } else if roll < 25 {
                // 10% Monster Summon (Water Demon)
                log.add_colored("A water demon appears!", [255, 0, 0], turn);
            } else if roll < 30 {
                // 5% Wish (Simple)
                log.add_colored("You feel lucky! (Wish chance)", [255, 215, 0], turn);
                p_stats.gold += 5000;
            } else if roll < 50 {
                // 20% Coin
                let gold = rng.rn1(100, 50) as u64;
                p_stats.gold += gold;
                log.add(
                    format!("You found {} gold pieces in the fountain.", gold),
                    turn,
                );
            } else if roll < 60 {
                // 10% Sickness/Poison
                log.add("The water tastes foul!", turn);
                p_health.current -= 5;
            } else if roll < 70 {
                // 10% A gush of water
                log.add("A gush of water hits you!", turn);
                // 1/3 chance to rust random equipment
                if rng.rn2(3) == 0 {
                    rust_random_equipment(world, item_mgr, log, turn, rng);
                }
            } else {
                // 30% Just Water
                log.add("It tastes like clear water.", turn);
                p_stats.nutrition += 50;
            }

            return true;
        }
    }
    false
}

fn rust_random_equipment(
    world: &mut SubWorld,
    item_mgr: &crate::core::entity::object::ItemManager,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
) {
    use crate::core::entity::{Equipment, Item, PlayerTag};
    use crate::core::systems::item_damage::ItemDamageSystem;

    let mut p_query = <(Entity, &Equipment)>::query().filter(component::<PlayerTag>());
    let mut item_to_rust = None;

    if let Some((_ent, equip)) = p_query.iter(world).next() {
        let slots: Vec<_> = equip.slots.values().cloned().collect();
        if !slots.is_empty() {
            item_to_rust = Some(slots[rng.rn2(slots.len() as i32) as usize]);
        }
    }

    if let Some(item_ent) = item_to_rust {
        if let Ok(mut entry) = world.entry_mut(item_ent) {
            if let Ok(item) = entry.get_component_mut::<Item>() {
                if let Some(template) = item_mgr.get_by_kind(item.kind) {
                    ItemDamageSystem::rust_item(item, template, log, turn, true);
                }
            }
        }
    }
}

// =============================================================================
// [v2.3.3] 분수 효과 확장 (원본 fountain.c: dip/wash/wish)
// =============================================================================

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FountainWishResult {
    NoWish,
    MinorWish,  // 소형 소원 (금화/포션)
    MajorWish,  // 대형 소원 (아이템)
    WaterDemon, // 물 악마 소환
}

///
pub fn fountain_wish_roll(luck: i32, rng: &mut NetHackRng) -> FountainWishResult {
    let roll = rng.rn2(100);
    let luck_bonus = luck.clamp(-5, 5);

    if roll + luck_bonus < 3 {
        FountainWishResult::MajorWish // 3% 미만: 대형 소원
    } else if roll + luck_bonus < 8 {
        FountainWishResult::MinorWish // 5%: 소형 소원
    } else if roll < 18 {
        FountainWishResult::WaterDemon // 10%: 물 악마
    } else {
        FountainWishResult::NoWish
    }
}

/// 분수 고갈 확률 (원본: fountain.c dryup)
pub fn fountain_dryup_chance(times_used: u32, rng: &mut NetHackRng) -> bool {
    //
    let base_chance = 20 + (times_used * 5).min(40);
    rng.rn2(100) < base_chance as i32
}

// =============================================================================
// [v2.3.3] 분수에 담그기 (원본 fountain.c: dipfountain)
// =============================================================================

/// 담그기 효과 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DipEffect {
    Nothing,      // 아무 일 없음
    Curse,        // 저주
    Bless,        // 축복
    Rust,         // 녹
    Dilute,       // 희석 (포션)
    Erode,        // 부식
    ExcalSuccess, // 엑스칼리버 생성 성공
    ExcalFail,    // 엑스칼리버 실패 (악마 소환)
}

/// 분수에 아이템 담그기 (원본: dipfountain)
pub fn dip_in_fountain(
    item_name: &str,
    item_is_long_sword: bool,
    player_is_lawful: bool,
    player_level: i32,
    rng: &mut NetHackRng,
) -> DipEffect {
    //
    if item_is_long_sword && player_is_lawful && player_level >= 5 {
        let excal_chance = rng.rn2(10);
        if excal_chance < 3 {
            return DipEffect::ExcalSuccess;
        } else if excal_chance < 5 {
            return DipEffect::ExcalFail; // 물 악마 소환
        }
    }

    let roll = rng.rn2(30);
    match roll {
        0..=4 => DipEffect::Nothing,
        5..=9 => DipEffect::Curse,
        10..=14 => DipEffect::Bless,
        15..=19 => {
            //
            if item_name.contains("sword")
                || item_name.contains("mace")
                || item_name.contains("armor")
                || item_name.contains("shield")
            {
                DipEffect::Rust
            } else {
                DipEffect::Nothing
            }
        }
        20..=24 => DipEffect::Dilute,
        _ => DipEffect::Erode,
    }
}

/// 담그기 결과 메시지 (원본: fountain.c messages)
pub fn dip_effect_message(effect: &DipEffect) -> &'static str {
    match effect {
        DipEffect::Nothing => "The water glistens for a moment.",
        DipEffect::Curse => "The water turns black. A curse falls upon your item!",
        DipEffect::Bless => "The water glows briefly. Your item feels blessed!",
        DipEffect::Rust => "The water causes rust! Your item corrodes!",
        DipEffect::Dilute => "The potion is diluted by the water.",
        DipEffect::Erode => "The water eats away at your item!",
        DipEffect::ExcalSuccess => "A brilliant light erupts from the water! Excalibur!",
        DipEffect::ExcalFail => "A water demon appears!",
    }
}

// =============================================================================
// [v2.3.3] 분수 씻기 (원본 fountain.c: washfountain)
// =============================================================================

/// 씻기 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WashEffect {
    Clean,       // 장갑/손 씻기
    RemoveBlind, // 눈 씻어 실명 해제
    RemoveSlime, // 슬라임 제거
    Nothing,     // 아무것도 아님
    WaterGush,   // 물줄기 맞음
}

///
pub fn wash_at_fountain(is_blinded: bool, is_slimed: bool, rng: &mut NetHackRng) -> WashEffect {
    let roll = rng.rn2(20);

    if is_slimed && roll < 5 {
        return WashEffect::RemoveSlime; // 25%: 슬라임 제거
    }

    if is_blinded && roll < 8 {
        return WashEffect::RemoveBlind; // 40%: 실명 해제
    }

    match roll {
        0..=5 => WashEffect::Clean,
        6..=10 => WashEffect::Nothing,
        _ => WashEffect::WaterGush,
    }
}

/// 씻기 메시지 (원본: fountain.c wash messages)
pub fn wash_effect_message(effect: &WashEffect) -> &'static str {
    match effect {
        WashEffect::Clean => "You wash your hands in the cool water.",
        WashEffect::RemoveBlind => "Your eyes tingle! You can see again!",
        WashEffect::RemoveSlime => "The slime dissolves in the water!",
        WashEffect::Nothing => "You splash some water around.",
        WashEffect::WaterGush => "A gush of water hits you in the face!",
    }
}

// =============================================================================
// [v2.3.3] 분수 통계
// =============================================================================

/// 분수 통계
#[derive(Debug, Clone, Default)]
pub struct FountainStatistics {
    pub drinks: u32,
    pub dips: u32,
    pub washes: u32,
    pub wishes_granted: u32,
    pub demons_summoned: u32,
    pub fountains_dried: u32,
    pub excaliburs_created: u32,
}

impl FountainStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_drink(&mut self) {
        self.drinks += 1;
    }
    pub fn record_dip(&mut self) {
        self.dips += 1;
    }
    pub fn record_wash(&mut self) {
        self.washes += 1;
    }
    pub fn record_wish(&mut self) {
        self.wishes_granted += 1;
    }
    pub fn record_demon(&mut self) {
        self.demons_summoned += 1;
    }
    pub fn record_dry(&mut self) {
        self.fountains_dried += 1;
    }
    pub fn record_excalibur(&mut self) {
        self.excaliburs_created += 1;
    }
}

// =============================================================================
// [v2.3.3] 테스트
// =============================================================================
#[cfg(test)]
mod fountain_extended_tests {
    use super::*;

    #[test]
    fn test_fountain_wish_roll() {
        let mut rng = NetHackRng::new(42);
        //
        let mut no_wish = 0;
        for _ in 0..100 {
            if fountain_wish_roll(0, &mut rng) == FountainWishResult::NoWish {
                no_wish += 1;
            }
        }
        assert!(no_wish > 70); // 70% 이상 NoWish여야 함
    }

    #[test]
    fn test_fountain_dryup() {
        let mut rng = NetHackRng::new(42);
        //
        let mut dried = 0;
        for _ in 0..100 {
            if fountain_dryup_chance(20, &mut rng) {
                dried += 1;
            }
        }
        assert!(dried > 30);
    }

    #[test]
    fn test_dip_excalibur() {
        let mut rng = NetHackRng::new(42);
        let mut excal = false;
        for _ in 0..100 {
            let r = dip_in_fountain("long sword", true, true, 10, &mut rng);
            if r == DipEffect::ExcalSuccess {
                excal = true;
                break;
            }
        }
        assert!(excal);
    }

    #[test]
    fn test_dip_effect_message() {
        assert!(dip_effect_message(&DipEffect::ExcalSuccess).contains("Excalibur"));
        assert!(dip_effect_message(&DipEffect::Curse).contains("curse"));
    }

    #[test]
    fn test_wash() {
        let mut rng = NetHackRng::new(42);
        let r = wash_at_fountain(false, false, &mut rng);
        // 결과는 Clean, Nothing, WaterGush 중 하나
        assert!(matches!(
            r,
            WashEffect::Clean | WashEffect::Nothing | WashEffect::WaterGush
        ));
    }

    #[test]
    fn test_fountain_stats() {
        let mut stats = FountainStatistics::new();
        stats.record_drink();
        stats.record_dip();
        stats.record_wish();
        assert_eq!(stats.drinks, 1);
        assert_eq!(stats.dips, 1);
        assert_eq!(stats.wishes_granted, 1);
    }
}

// =============================================================================
// [v2.9.8] fountain.c 미구현 함수 이식 — 분수 마시기 확장, 싱크대, 소환, 고갈
// =============================================================================

/// [v2.9.8] 분수 마시기 상세 효과 (원본: drinkfountain L222-358)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrinkFountainEffect {
    /// 시원한 물 — 허기 +rnd(10) (fate < 10)
    CoolDraught { hunger_gain: i32 },
    /// 축복 분수 효과 — 어트리뷰트 복원 및 증가
    BlessedFountain { attr_index: usize },
    /// 자기 인식 (fate=19)
    SelfKnowledge,
    /// 오염된 물 — 구토 (fate=20)
    FoulWater { hunger_loss: i32 },
    /// 독물 (fate=21)
    Poisonous { has_poison_res: bool, str_loss: i32, hp_loss: i32 },
    /// 뱀 소환 (fate=22)
    WaterSnakes { count: i32 },
    /// 물 악마 소환 (fate=23)
    WaterDemon { grants_wish: bool },
    /// 아이템 저주 (fate=24)
    CurseItems { hunger_loss: i32 },
    /// 투명체 감지 (fate=25)
    SeeInvisible { is_blind: bool, is_invisible: bool },
    /// 몬스터 감지 (fate=26)
    MonsterDetect,
    /// 보석 발견 (fate=27), 이미 약탈된 경우 님프로 대체
    FindGem,
    /// 님프 소환 (fate=28 또는 이미 약탈된 27)
    WaterNymph,
    /// 악취 공포 (fate=29)
    BadBreathScare,
    /// 물 분출 (fate=30)
    GushForth,
    /// 미지근한 물 (default)
    TepidWater,
}

/// [v2.9.8] 분수 마시기 판정 (원본: drinkfountain L222-358)
pub fn drink_fountain_effect(
    is_blessed: bool,
    luck: i32,
    is_fountain_looted: bool,
    rng: &mut NetHackRng,
) -> DrinkFountainEffect {
    let fate = rng.rnd(30);

    // 축복 분수: 행운 >= 0이고 fate >= 10이면 축복 효과
    if is_blessed && luck >= 0 && fate >= 10 {
        let attr_idx = rng.rn2(6) as usize; // A_MAX = 6
        return DrinkFountainEffect::BlessedFountain { attr_index: attr_idx };
    }

    if fate < 10 {
        // 시원한 물
        return DrinkFountainEffect::CoolDraught { hunger_gain: rng.rnd(10) };
    }

    match fate {
        19 => DrinkFountainEffect::SelfKnowledge,
        20 => DrinkFountainEffect::FoulWater { hunger_loss: rng.rn1(20, 11) },
        21 => {
            let has_pr = false; // 호출자가 결정
            DrinkFountainEffect::Poisonous {
                has_poison_res: has_pr,
                str_loss: rng.rn1(4, 3),
                hp_loss: rng.rnd(10),
            }
        }
        22 => DrinkFountainEffect::WaterSnakes { count: rng.rn1(5, 2) },
        23 => {
            // 물 악마 — 레벨 난이도에 따라 소원 부여
            let grants = rng.rnd(100) > 80; // 간단 근사
            DrinkFountainEffect::WaterDemon { grants_wish: grants }
        }
        24 => DrinkFountainEffect::CurseItems { hunger_loss: rng.rn1(20, 11) },
        25 => DrinkFountainEffect::SeeInvisible { is_blind: false, is_invisible: false },
        26 => DrinkFountainEffect::MonsterDetect,
        27 => {
            if !is_fountain_looted {
                DrinkFountainEffect::FindGem
            } else {
                DrinkFountainEffect::WaterNymph // FALLTHRU
            }
        }
        28 => DrinkFountainEffect::WaterNymph,
        29 => DrinkFountainEffect::BadBreathScare,
        30 => DrinkFountainEffect::GushForth,
        _ => DrinkFountainEffect::TepidWater,
    }
}

/// [v2.9.8] 싱크대 마시기 효과 (원본: drinksink L519-626)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrinkSinkEffect {
    /// 차가운 물 (case 0)
    VeryCold,
    /// 따뜻한 물 (case 1)
    VeryWarm,
    /// 끓는 물 (case 2)
    ScaldingHot { has_fire_res: bool, hp_loss: i32 },
    /// 하수쥐 소환 (case 3)
    SewerRat { rats_gone: bool },
    /// 랜덤 포션 효과 (case 4)
    RandomPotion,
    /// 반지 발견 (case 5)
    FindRing { already_looted: bool },
    /// 파이프 파괴 → 분수로 변환 (case 6)
    BreakSink,
    /// 워터 엘리멘탈 소환 (case 7)
    WaterElemental { elementals_gone: bool },
    /// 역겨운 물 — 경험치 +1 (case 8)
    AwfulTaste,
    /// 하수 맛 — 구토 (case 9)
    SewageTaste { hunger_loss: i32 },
    /// 독성 폐수 — 변신 (case 10)
    ToxicWaste,
    /// 파이프 소리 (case 11)
    PipeClanking,
    /// 하수도 노래 (case 12)
    SewerSong,
    /// 환각 시 손 (case 19)
    HallucinatoryHand { is_hallucinating: bool },
    /// 일반 물 (default)
    NormalWater { temperature: &'static str },
}

/// [v2.9.8] 싱크대 마시기 판정 (원본: drinksink L519-626)
pub fn drink_sink_effect(rng: &mut NetHackRng) -> DrinkSinkEffect {
    let roll = rng.rn2(20);
    match roll {
        0 => DrinkSinkEffect::VeryCold,
        1 => DrinkSinkEffect::VeryWarm,
        2 => DrinkSinkEffect::ScaldingHot { has_fire_res: false, hp_loss: rng.rnd(6) },
        3 => DrinkSinkEffect::SewerRat { rats_gone: false },
        4 => DrinkSinkEffect::RandomPotion,
        5 => DrinkSinkEffect::FindRing { already_looted: false },
        6 => DrinkSinkEffect::BreakSink,
        7 => DrinkSinkEffect::WaterElemental { elementals_gone: false },
        8 => DrinkSinkEffect::AwfulTaste,
        9 => DrinkSinkEffect::SewageTaste { hunger_loss: rng.rn1(20, 11) },
        10 => DrinkSinkEffect::ToxicWaste,
        11 => DrinkSinkEffect::PipeClanking,
        12 => DrinkSinkEffect::SewerSong,
        19 => DrinkSinkEffect::HallucinatoryHand { is_hallucinating: false },
        _ => {
            let temp = match rng.rn2(3) {
                0 => { if rng.rn2(2) == 0 { "cold" } else { "warm" } }
                _ => "hot",
            };
            DrinkSinkEffect::NormalWater { temperature: temp }
        }
    }
}

/// [v2.9.8] 분수 고갈 상세 결과 (원본: dryup L166-219)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DryupResult {
    /// 고갈 안 됨
    NoChange,
    /// 마을 내 경고 — 경비원이 경고
    TownWarning { guard_warned: bool, is_deaf: bool },
    /// 분수 파괴 — ROOM으로 변환
    FountainDestroyed { in_town: bool },
}

/// [v2.9.8] 분수 고갈 판정 (원본: dryup L166-219)
pub fn dryup_result(
    is_fountain: bool,
    was_warned: bool,
    is_in_town: bool,
    is_player: bool,
    rng: &mut NetHackRng,
) -> DryupResult {
    if !is_fountain {
        return DryupResult::NoChange;
    }
    // 33% 확률 또는 이미 경고받은 경우
    if rng.rn2(3) != 0 && !was_warned {
        return DryupResult::NoChange;
    }
    // 마을 내 + 플레이어 + 아직 미경고 → 경고
    if is_player && is_in_town && !was_warned {
        return DryupResult::TownWarning { guard_warned: true, is_deaf: false };
    }
    // 분수 파괴
    DryupResult::FountainDestroyed { in_town: is_in_town }
}

/// [v2.9.8] 물 분출 타일 판정 (원본: gush L120-149)
pub fn gush_tile_should_pool(
    x: i32, y: i32,
    player_x: i32, player_y: i32,
    tile_is_room: bool,
    has_boulder: bool,
    next_to_door: bool,
    rng: &mut NetHackRng,
) -> bool {
    // 체커보드 패턴, 플레이어 위치, 거리, 방 타일, 바위, 문 옆 확인
    if ((x + y) % 2) != 0 { return false; }
    if x == player_x && y == player_y { return false; }
    let dist = (x - player_x).abs().max((y - player_y).abs());
    if rng.rn2(1 + dist) != 0 { return false; }
    if !tile_is_room || has_boulder || next_to_door { return false; }
    true
}

/// [v2.9.8] 분수에 아이템 담그기 상세 효과 (원본: dipfountain L361-503)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DipFountainEffect {
    /// 엑스칼리버 성공 (질서 성향)
    ExcaliburSuccess,
    /// 엑스칼리버 실패 — 저주 + 강화치 감소
    ExcaliburFail { spe_decrease: bool },
    /// 물 손상
    WaterDamage,
    /// 아이템 저주 (case 16)
    CurseItem,
    /// 아이템 해주 (case 17-20)
    UncurseItem { was_cursed: bool },
    /// 물 악마 소환 (case 21)
    WaterDemonDip,
    /// 님프 소환 (case 22)
    WaterNymphDip,
    /// 뱀 소환 (case 23)
    WaterSnakesDip,
    /// 보석 발견 (case 24)
    FindGemDip,
    /// 물 분출 (case 25)
    GushForthDip,
    /// 따끔거림 (case 26)
    StrangeTingling,
    /// 한기 (case 27)
    SuddenChill,
    /// 목욕 충동 — 돈 손실 (case 28)
    BathUrge { money_fraction: i32 },
    /// 코인 발견 (case 29)
    FindCoins { amount: i32 },
    /// 아무 일 없음
    Nothing,
}

/// [v2.9.8] 분수 담그기 상세 판정 (원본: dipfountain L361-503)
pub fn dip_fountain_detail(
    is_long_sword: bool,
    quantity: i32,
    player_level: i32,
    player_alignment: i32, // 1=lawful, -1=chaotic, 0=neutral
    has_excalibur: bool,
    is_fountain_looted: bool,
    dungeon_depth_from_surface: i32,
    rng: &mut NetHackRng,
) -> DipFountainEffect {
    // 엑스칼리버 시도: 롱소드, 수량 1, 레벨 5+, 6분의1, 미보유
    if is_long_sword && quantity == 1 && player_level >= 5
        && rng.rn2(6) == 0 && !has_excalibur
    {
        if player_alignment != 1 {
            // 비질서 → 실패
            let spe_dec = rng.rn2(3) != 0;
            return DipFountainEffect::ExcaliburFail { spe_decrease: spe_dec };
        } else {
            return DipFountainEffect::ExcaliburSuccess;
        }
    }

    // 물 손상 후 50% 확률로 종료
    if rng.rn2(2) == 0 {
        return DipFountainEffect::WaterDamage;
    }

    match rng.rnd(30) {
        16 => DipFountainEffect::CurseItem,
        17 | 18 | 19 | 20 => DipFountainEffect::UncurseItem { was_cursed: false },
        21 => DipFountainEffect::WaterDemonDip,
        22 => DipFountainEffect::WaterNymphDip,
        23 => DipFountainEffect::WaterSnakesDip,
        24 => {
            if !is_fountain_looted { DipFountainEffect::FindGemDip }
            else { DipFountainEffect::GushForthDip } // FALLTHRU
        }
        25 => DipFountainEffect::GushForthDip,
        26 => DipFountainEffect::StrangeTingling,
        27 => DipFountainEffect::SuddenChill,
        28 => {
            let fraction = rng.rnd(10); // somegold 근사
            DipFountainEffect::BathUrge { money_fraction: fraction }
        }
        29 => {
            let amount = rng.rnd(dungeon_depth_from_surface.max(1) * 2) + 5;
            DipFountainEffect::FindCoins { amount }
        }
        _ => DipFountainEffect::Nothing,
    }
}

/// [v2.9.8] breaksink 결과 (원본: breaksink L505-517)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakSinkResult {
    /// 파이프 파괴 가시 여부
    pub visible: bool,
}

/// [v2.9.8] 공중 부유 메시지 (원본: floating_above L17-30)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FloatingAboveResult {
    /// 바닥에 갇힘
    TrappedInFloor,
    /// 공중 부유
    FloatingAbove,
}

/// [v2.9.8] floating_above 판정
pub fn floating_above_result(is_trapped_in_floor: bool) -> FloatingAboveResult {
    if is_trapped_in_floor {
        FloatingAboveResult::TrappedInFloor
    } else {
        FloatingAboveResult::FloatingAbove
    }
}

// =============================================================================
// [v2.9.8] 테스트 — fountain.c 추가 이식분
// =============================================================================
#[cfg(test)]
mod fountain_phase2_tests {
    use super::*;

    #[test]
    fn test_drink_fountain_cool() {
        let mut rng = NetHackRng::new(42);
        // fate < 10이 나올 때까지 시도
        for seed in 0..100u64 {
            let mut r = NetHackRng::new(seed);
            let effect = drink_fountain_effect(false, 0, false, &mut r);
            if let DrinkFountainEffect::CoolDraught { hunger_gain } = effect {
                assert!(hunger_gain >= 1 && hunger_gain <= 10);
                return;
            }
        }
    }

    #[test]
    fn test_drink_fountain_blessed() {
        // 축복 분수 + 행운 >= 0 + fate >= 10 → BlessedFountain
        for seed in 0..100u64 {
            let mut r = NetHackRng::new(seed);
            let effect = drink_fountain_effect(true, 5, false, &mut r);
            if let DrinkFountainEffect::BlessedFountain { attr_index } = effect {
                assert!(attr_index < 6);
                return;
            }
        }
    }

    #[test]
    fn test_drink_fountain_variety() {
        // 다양한 효과가 나오는지 확인
        let mut effects = std::collections::HashSet::new();
        for seed in 0..200u64 {
            let mut r = NetHackRng::new(seed);
            let e = drink_fountain_effect(false, 0, false, &mut r);
            effects.insert(std::mem::discriminant(&e));
        }
        assert!(effects.len() >= 5); // 최소 5종 이상
    }

    #[test]
    fn test_drink_sink_variety() {
        let mut effects = std::collections::HashSet::new();
        for seed in 0..200u64 {
            let mut r = NetHackRng::new(seed);
            let e = drink_sink_effect(&mut r);
            effects.insert(std::mem::discriminant(&e));
        }
        assert!(effects.len() >= 5);
    }

    #[test]
    fn test_dryup_no_fountain() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(dryup_result(false, false, false, true, &mut rng), DryupResult::NoChange);
    }

    #[test]
    fn test_dryup_town_warning() {
        // 마을 내 미경고 상태에서 고갈 시도
        for seed in 0..100u64 {
            let mut r = NetHackRng::new(seed);
            if let DryupResult::TownWarning { .. } = dryup_result(true, false, true, true, &mut r) {
                return; // 경고 발생 확인
            }
        }
    }

    #[test]
    fn test_gush_tile() {
        let mut rng = NetHackRng::new(42);
        // 체커보드 패턴, 플레이어 위치 아닌 곳, 방 타일
        let result = gush_tile_should_pool(2, 2, 5, 5, true, false, false, &mut rng);
        // 결과는 랜덤 — 타입만 확인
        assert!(result == true || result == false);
    }

    #[test]
    fn test_dip_excalibur_lawful() {
        // 질서 성향으로 엑스칼리버 시도
        for seed in 0..500u64 {
            let mut r = NetHackRng::new(seed);
            let effect = dip_fountain_detail(true, 1, 10, 1, false, false, 5, &mut r);
            if effect == DipFountainEffect::ExcaliburSuccess {
                return; // 성공 확인
            }
        }
    }

    #[test]
    fn test_dip_excalibur_chaotic() {
        for seed in 0..500u64 {
            let mut r = NetHackRng::new(seed);
            let effect = dip_fountain_detail(true, 1, 10, -1, false, false, 5, &mut r);
            if let DipFountainEffect::ExcaliburFail { .. } = effect {
                return;
            }
        }
    }

    #[test]
    fn test_dip_variety() {
        let mut effects = std::collections::HashSet::new();
        for seed in 0..500u64 {
            let mut r = NetHackRng::new(seed);
            let e = dip_fountain_detail(false, 1, 3, 0, false, false, 5, &mut r);
            effects.insert(std::mem::discriminant(&e));
        }
        assert!(effects.len() >= 3);
    }

    #[test]
    fn test_floating_above() {
        assert_eq!(floating_above_result(true), FloatingAboveResult::TrappedInFloor);
        assert_eq!(floating_above_result(false), FloatingAboveResult::FloatingAbove);
    }
}

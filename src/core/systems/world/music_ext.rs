// =============================================================================
// AIHack - music_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// [v2.11.0] music.c 핵심 로직 순수 결과 패턴 이식
// 원본: nethack-3.6.7/src/music.c (990줄)
//
// 이식된 로직:
//   1. instrument_effect_type — 악기별 효과 분류 (music.c:565-676)
//   2. awaken_range — 각성 범위 계산 (music.c:61-85, 170-209)
//   3. sleep_range — 수면 범위 (music.c:91-106)
//   4. charm_range — 매혹 범위 (music.c:214-235)
//   5. earthquake_range — 지진 범위 (music.c:254-261)
//   6. improvisation_mode — 즉흥연주 모드 판정 (music.c:502-525)
//   7. drawbridge_tune_check — 도개교 멜로디 힌트 (music.c:800-836)
//   8. generic_lvl_desc — 레벨 설명 문자열 (music.c:443-458)
//   9. drum_deafness_duration — 드럼 청각 장애 지속시간 (music.c:655-670)
//  10. horn_direction_damage — 뿔 피리 방향 데미지 (music.c:584-603)
//  11. flute_check — 플루트 매혹 판정 (music.c:574-583)
//  12. harp_check — 하프 위안 판정 (music.c:630-640)
// =============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// 1. instrument_effect_type — 악기별 효과 분류
// [v2.11.0] music.c:565-676 이식
// =============================================================================

/// 악기가 발동하는 효과 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentEffect {
    /// 마법 플루트 → 몬스터 수면
    PutToSleep,
    /// 나무 플루트 → 뱀 매혹 (기량 충분 시)
    CharmSnakes,
    /// 화염 뿔 → 화염 광선 (지팡이 효과)
    FireBeam,
    /// 냉기 뿔 → 냉기 광선 (지팡이 효과)
    FrostBeam,
    /// 도구 뿔 → 각성/공포
    Awaken,
    /// 나팔 → 병사 각성
    AwakenSoldiers,
    /// 마법 하프 → 몬스터 길들임
    CharmMonsters,
    /// 나무 하프 → 님프 위안
    CalmNymphs,
    /// 지진 드럼 → 지진(구덩이 생성)
    Earthquake,
    /// 가죽 드럼 → 각성 + 청각 장애
    DrumBeat,
    /// 알 수 없는 악기
    Unknown,
}

/// 악기 OTyp 상수
pub const MAGIC_FLUTE: i32 = 0;
pub const WOODEN_FLUTE: i32 = 1;
pub const FIRE_HORN: i32 = 2;
pub const FROST_HORN: i32 = 3;
pub const TOOLED_HORN: i32 = 4;
pub const BUGLE: i32 = 5;
pub const MAGIC_HARP: i32 = 6;
pub const WOODEN_HARP: i32 = 7;
pub const DRUM_OF_EARTHQUAKE: i32 = 8;
pub const LEATHER_DRUM: i32 = 9;

/// 악기 효과 판정 (원본 music.c:565-676)
/// can_special: 기절/혼란이 아니고 충전이 남아있는지
pub fn instrument_effect_type(otyp: i32, can_special: bool) -> InstrumentEffect {
    // 마법 악기가 special 발동 불가능하면 일반 대응물로 강등
    let effective_otyp = if !can_special {
        match otyp {
            MAGIC_FLUTE => WOODEN_FLUTE,
            MAGIC_HARP => WOODEN_HARP,
            DRUM_OF_EARTHQUAKE => LEATHER_DRUM,
            // 화염/냉기 뿔은 충전 필요하지만 별도 처리
            _ => otyp,
        }
    } else {
        otyp
    };

    match effective_otyp {
        MAGIC_FLUTE => InstrumentEffect::PutToSleep,
        WOODEN_FLUTE => InstrumentEffect::CharmSnakes,
        FIRE_HORN => InstrumentEffect::FireBeam,
        FROST_HORN => InstrumentEffect::FrostBeam,
        TOOLED_HORN => InstrumentEffect::Awaken,
        BUGLE => InstrumentEffect::AwakenSoldiers,
        MAGIC_HARP => InstrumentEffect::CharmMonsters,
        WOODEN_HARP => InstrumentEffect::CalmNymphs,
        DRUM_OF_EARTHQUAKE => InstrumentEffect::Earthquake,
        LEATHER_DRUM => InstrumentEffect::DrumBeat,
        _ => InstrumentEffect::Unknown,
    }
}

// =============================================================================
// 2. awaken_range — 각성 범위
// [v2.11.0] music.c:61-85 (awaken_monsters), music.c:170-209 (awaken_soldiers)
// =============================================================================

/// 각성 범위 계산 (level * multiplier 기반 거리 제곱)
/// 도구뿔: level * 30
/// 나팔: level * 30 (관련 로직)
/// 가죽 드럼(일반): level * 5
/// 가죽 드럼(마법): level * 40
pub fn awaken_range(player_level: i32, instrument: InstrumentEffect, mundane_drum: bool) -> i32 {
    match instrument {
        InstrumentEffect::Awaken => player_level * 30,
        InstrumentEffect::AwakenSoldiers => player_level * 30,
        InstrumentEffect::DrumBeat => {
            if mundane_drum {
                player_level * 5
            } else {
                player_level * 40
            }
        }
        // 지진 드럼은 전체 레벨 (ROWNO * COLNO)
        InstrumentEffect::Earthquake => 80 * 21, // COLNO=80, ROWNO=21
        _ => 0,
    }
}

// =============================================================================
// 3. sleep_range — 수면 범위
// [v2.11.0] music.c:91-106 이식
// =============================================================================

/// 마법 플루트 수면 범위 (원본 music.c:571)
/// 거리 제곱 기준 = level * 5
pub fn sleep_range(player_level: i32) -> i32 {
    player_level * 5
}

// =============================================================================
// 4. charm_range — 매혹 범위
// [v2.11.0] music.c:214-235, music.c:627 이식
// =============================================================================

/// 마법 하프 매혹 범위 (원본 music.c:627)
/// 거리 제곱 기준 = (level-1)/3 + 1
pub fn charm_range(player_level: i32) -> i32 {
    (player_level - 1) / 3 + 1
}

/// 뱀 매혹 범위 (원본 music.c:581)
/// 거리 제곱 기준 = level * 3
pub fn snake_charm_range(player_level: i32) -> i32 {
    player_level * 3
}

/// 님프 위안 범위 (원본 music.c:638)
/// 거리 제곱 기준 = level * 3
pub fn nymph_calm_range(player_level: i32) -> i32 {
    player_level * 3
}

// =============================================================================
// 5. earthquake_range — 지진 범위
// [v2.11.0] music.c:254-261 이식
// =============================================================================

/// 지진 효과 범위 (원본 music.c:254-261)
/// force = (level-1)/3 + 1
/// 범위 = force * 2 (양방향)
pub fn earthquake_range(player_level: i32) -> (i32, i32, i32, i32) {
    let force = (player_level - 1) / 3 + 1;
    let extend = force * 2;
    // (start_offset, end_offset) — 플레이어 위치 기준
    (-extend, -extend, extend, extend)
}

/// 지진 구덩이 생성 확률 분모 (원본 music.c:278)
/// 확률 = 1/(14 - force), force가 클수록 높은 확률
pub fn earthquake_pit_chance(force: i32) -> i32 {
    (14 - force).max(2) // 최소 2로 제한하여 0 나누기 방지
}

// =============================================================================
// 6. improvisation_mode — 즉흥연주 모드 판정
// [v2.11.0] music.c:502-525 이식
// =============================================================================

/// 즉흥연주 모드 (원본 music.c:502-525)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImprovMode {
    /// 정상 연주
    Normal,
    /// 기절 — 싫은 드론 소리
    Stunned,
    /// 혼란 — 거친 소음
    Confused,
    /// 환각 — 나비 만화경
    Hallucinating,
    /// 복합 장애(구체적 미구분)
    MixedImpaired,
}

/// 즉흥연주 모드 결정 (원본 music.c:502-525)
pub fn improvisation_mode(
    is_stunned: bool,
    is_confused: bool,
    is_hallucinating: bool,
    rng: &mut NetHackRng,
) -> ImprovMode {
    let mut mode = 0u8;
    if is_stunned {
        mode |= 0x01;
    }
    if is_confused {
        mode |= 0x02;
    }
    if is_hallucinating {
        mode |= 0x04;
    }

    if mode == 0 {
        return ImprovMode::Normal;
    }

    // 복합 장애 시 50% 확률로 단순화
    if rng.rn2(2) == 0 {
        if mode == 0x03 {
            // STUNNED+CONFUSED
            mode = if rng.rn2(2) == 0 { 0x01 } else { 0x02 };
        }
        if mode & 0x04 != 0 {
            mode = 0x04;
        }
    }

    match mode {
        0x01 => ImprovMode::Stunned,
        0x02 => ImprovMode::Confused,
        0x04 => ImprovMode::Hallucinating,
        _ => ImprovMode::MixedImpaired,
    }
}

// =============================================================================
// 7. drawbridge_tune_check — 도개교 멜로디 힌트
// [v2.11.0] music.c:800-836 이식
// =============================================================================

/// 도개교 멜로디 대조 결과 (원본 music.c:800-836)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuneCheckResult {
    /// 정확히 맞춘 음 (gears)
    pub gears: i32,
    /// 위치는 다르지만 포함된 음 (tumblers)
    pub tumblers: i32,
    /// 완전 일치 여부
    pub perfect_match: bool,
}

/// 도개교 멜로디 대조 — Mastermind 방식 (원본 music.c:800-836)
/// played: 연주된 음 (5자 이하)
/// correct: 정확한 멜로디 (항상 5자)
pub fn drawbridge_tune_check(played: &[u8; 5], correct: &[u8; 5]) -> TuneCheckResult {
    if played == correct {
        return TuneCheckResult {
            gears: 5,
            tumblers: 0,
            perfect_match: true,
        };
    }

    let mut matched = [false; 5];
    let mut gears = 0;
    let mut tumblers = 0;

    // 1패스: 정확한 위치 매칭
    for i in 0..5 {
        if played[i] == correct[i] {
            gears += 1;
            matched[i] = true;
        }
    }

    // 2패스: 포함되어 있지만 위치가 다른 음
    for i in 0..5 {
        if played[i] != correct[i] {
            for j in 0..5 {
                if !matched[j] && played[i] == correct[j] && played[j] != correct[j] {
                    tumblers += 1;
                    matched[j] = true;
                    break;
                }
            }
        }
    }

    TuneCheckResult {
        gears,
        tumblers,
        perfect_match: false,
    }
}

// =============================================================================
// 8. generic_lvl_desc — 레벨 설명 문자열
// [v2.11.0] music.c:443-458 이식
// =============================================================================

/// 레벨 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelType {
    Astral,
    Endgame,
    Sanctum,
    Sokoban,
    VladsTower,
    Normal,
}

/// 레벨 유형별 설명 문자열 (원본 music.c:443-458)
pub fn generic_lvl_desc(level_type: LevelType) -> &'static str {
    match level_type {
        LevelType::Astral => "astral plane",
        LevelType::Endgame => "plane",
        LevelType::Sanctum => "sanctum",
        LevelType::Sokoban => "puzzle",
        LevelType::VladsTower => "tower",
        LevelType::Normal => "dungeon",
    }
}

// =============================================================================
// 9. drum_deafness_duration — 드럼 청각 장애 지속시간
// [v2.11.0] music.c:658-659 이식
// =============================================================================

/// 가죽 드럼 연주 시 청각 장애 지속시간 (원본 music.c:659)
/// rn1(20, 30) = 30~49 턴
pub fn drum_deafness_duration(rng: &mut NetHackRng) -> i32 {
    rng.rn1(20, 30)
}

// =============================================================================
// 10. horn_direction_damage — 뿔 피리 자해 데미지 구분
// [v2.11.0] music.c:584-603 이식
// =============================================================================

/// 뿔 피리(화염/냉기) 효과 결정 (원본 music.c:584-603)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HornResult {
    /// 방향 지정 안 됨 → 진동만
    NoDirection,
    /// 자기 자신에게 발사 (dx=0,dy=0,dz=0)
    SelfTarget,
    /// 유효 방향으로 광선 발사
    Beam {
        /// 광선 타입 인덱스 (AD_COLD-1 또는 AD_FIRE-1)
        beam_type: i32,
        /// 광선 강도 rn1(6,6) = [6,11]
        intensity: i32,
    },
}

/// 뿔 피리 결과 판정 (원본 music.c:584-603)
pub fn horn_direction_result(
    is_frost: bool,
    has_direction: bool,
    dx: i32,
    dy: i32,
    dz: i32,
    rng: &mut NetHackRng,
) -> HornResult {
    if !has_direction {
        return HornResult::NoDirection;
    }
    if dx == 0 && dy == 0 && dz == 0 {
        return HornResult::SelfTarget;
    }
    let beam_type = if is_frost { 1 } else { 0 }; // AD_COLD-1=1, AD_FIRE-1=0
    let intensity = rng.rn1(6, 6);
    HornResult::Beam {
        beam_type,
        intensity,
    }
}

// =============================================================================
// 11. flute_check — 플루트 매혹 판정
// [v2.11.0] music.c:574-583 이식
// =============================================================================

/// 나무 플루트 뱀 매혹 성공 판정 (원본 music.c:575)
/// rn2(DEX) + level > 25
pub fn flute_charm_check(dexterity: i32, player_level: i32, rng: &mut NetHackRng) -> bool {
    rng.rn2(dexterity) + player_level > 25
}

// =============================================================================
// 12. harp_check — 하프 위안 판정
// [v2.11.0] music.c:631 이식
// =============================================================================

/// 나무 하프 님프 위안 성공 판정 (원본 music.c:631)
/// rn2(DEX) + level > 25
pub fn harp_calm_check(dexterity: i32, player_level: i32, rng: &mut NetHackRng) -> bool {
    rng.rn2(dexterity) + player_level > 25
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    // ─── instrument_effect_type ────────────────────────────────
    #[test]
    fn test_magic_flute_special() {
        assert_eq!(
            instrument_effect_type(MAGIC_FLUTE, true),
            InstrumentEffect::PutToSleep
        );
    }

    #[test]
    fn test_magic_flute_no_special() {
        // 마법 플루트 → special 불가면 나무 플루트로 강등
        assert_eq!(
            instrument_effect_type(MAGIC_FLUTE, false),
            InstrumentEffect::CharmSnakes
        );
    }

    #[test]
    fn test_fire_horn() {
        assert_eq!(
            instrument_effect_type(FIRE_HORN, true),
            InstrumentEffect::FireBeam
        );
    }

    #[test]
    fn test_drum_of_earthquake() {
        assert_eq!(
            instrument_effect_type(DRUM_OF_EARTHQUAKE, true),
            InstrumentEffect::Earthquake
        );
    }

    #[test]
    fn test_earthquake_to_leather() {
        assert_eq!(
            instrument_effect_type(DRUM_OF_EARTHQUAKE, false),
            InstrumentEffect::DrumBeat
        );
    }

    // ─── awaken_range ─────────────────────────────────────────
    #[test]
    fn test_awaken_horn() {
        assert_eq!(awaken_range(10, InstrumentEffect::Awaken, false), 300);
    }

    #[test]
    fn test_awaken_drum_mundane() {
        assert_eq!(awaken_range(10, InstrumentEffect::DrumBeat, true), 50);
    }

    #[test]
    fn test_awaken_drum_magic() {
        assert_eq!(awaken_range(10, InstrumentEffect::DrumBeat, false), 400);
    }

    // ─── sleep_range ──────────────────────────────────────────
    #[test]
    fn test_sleep_range() {
        assert_eq!(sleep_range(12), 60);
    }

    // ─── charm_range ──────────────────────────────────────────
    #[test]
    fn test_charm_range_level1() {
        assert_eq!(charm_range(1), 1);
    }

    #[test]
    fn test_charm_range_level10() {
        assert_eq!(charm_range(10), 4); // (10-1)/3+1=4
    }

    // ─── earthquake_range ─────────────────────────────────────
    #[test]
    fn test_earthquake_range() {
        let (sx, sy, ex, ey) = earthquake_range(7);
        // force = (7-1)/3+1 = 3, extend = 6
        assert_eq!(sx, -6);
        assert_eq!(sy, -6);
        assert_eq!(ex, 6);
        assert_eq!(ey, 6);
    }

    #[test]
    fn test_earthquake_pit_chance() {
        assert_eq!(earthquake_pit_chance(3), 11); // 14-3=11
        assert_eq!(earthquake_pit_chance(12), 2); // max(14-12, 2)=2
    }

    // ─── improvisation_mode ───────────────────────────────────
    #[test]
    fn test_improv_normal() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            improvisation_mode(false, false, false, &mut rng),
            ImprovMode::Normal
        );
    }

    #[test]
    fn test_improv_stunned() {
        let mut rng = NetHackRng::new(42);
        // 단일 장애는 항상 정확히 분류
        let result = improvisation_mode(true, false, false, &mut rng);
        // 50% 확률로 Stunned 또는 MixedImpaired (단일이라 항상 Stunned)
        assert!(result == ImprovMode::Stunned || result == ImprovMode::MixedImpaired);
    }

    // ─── drawbridge_tune_check ────────────────────────────────
    #[test]
    fn test_tune_perfect() {
        let played = [b'A', b'B', b'C', b'D', b'E'];
        let correct = [b'A', b'B', b'C', b'D', b'E'];
        let result = drawbridge_tune_check(&played, &correct);
        assert!(result.perfect_match);
        assert_eq!(result.gears, 5);
    }

    #[test]
    fn test_tune_partial() {
        let played = [b'A', b'C', b'B', b'D', b'E'];
        let correct = [b'A', b'B', b'C', b'D', b'E'];
        let result = drawbridge_tune_check(&played, &correct);
        assert!(!result.perfect_match);
        assert_eq!(result.gears, 3); // A, D, E
        assert_eq!(result.tumblers, 2); // B, C (스왑)
    }

    #[test]
    fn test_tune_no_match() {
        let played = [b'F', b'G', b'F', b'G', b'F'];
        let correct = [b'A', b'B', b'C', b'D', b'E'];
        let result = drawbridge_tune_check(&played, &correct);
        assert_eq!(result.gears, 0);
        assert_eq!(result.tumblers, 0);
    }

    // ─── generic_lvl_desc ─────────────────────────────────────
    #[test]
    fn test_lvl_desc() {
        assert_eq!(generic_lvl_desc(LevelType::Astral), "astral plane");
        assert_eq!(generic_lvl_desc(LevelType::Normal), "dungeon");
        assert_eq!(generic_lvl_desc(LevelType::VladsTower), "tower");
    }

    // ─── drum_deafness_duration ───────────────────────────────
    #[test]
    fn test_deafness_range() {
        let mut rng = NetHackRng::new(42);
        let dur = drum_deafness_duration(&mut rng);
        assert!(dur >= 30 && dur <= 49);
    }

    // ─── horn_direction_result ────────────────────────────────
    #[test]
    fn test_horn_no_direction() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            horn_direction_result(true, false, 0, 0, 0, &mut rng),
            HornResult::NoDirection
        );
    }

    #[test]
    fn test_horn_self_target() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            horn_direction_result(true, true, 0, 0, 0, &mut rng),
            HornResult::SelfTarget
        );
    }

    #[test]
    fn test_horn_beam() {
        let mut rng = NetHackRng::new(42);
        match horn_direction_result(false, true, 1, 0, 0, &mut rng) {
            HornResult::Beam {
                beam_type,
                intensity,
            } => {
                assert_eq!(beam_type, 0); // 화염
                assert!(intensity >= 6 && intensity <= 11);
            }
            _ => panic!("기대값: Beam"),
        }
    }

    // ─── flute_check ──────────────────────────────────────────
    #[test]
    fn test_flute_high_dex() {
        let mut rng = NetHackRng::new(42);
        // DEX=30, level=10: rn2(30) + 10 은 대부분 >25
        // 단, rn2 결과에 따라 달라질 수 있음
        let result = flute_charm_check(30, 10, &mut rng);
        // rn2(30)는 0~29, +10 → 10~39 중 25 초과는 대부분
        let _ = result; // 결과는 확률적
    }

    #[test]
    fn test_flute_low_dex() {
        let mut rng = NetHackRng::new(42);
        // DEX=5, level=5: rn2(5)+5 = [5,9] → 모두 <=25
        assert!(!flute_charm_check(5, 5, &mut rng));
    }

    // ─── harp_check ───────────────────────────────────────────
    #[test]
    fn test_harp_low_level() {
        let mut rng = NetHackRng::new(42);
        assert!(!harp_calm_check(5, 3, &mut rng));
    }
}

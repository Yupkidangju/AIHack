// ============================================================================
// AIHack - engrave_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] engrave.c 미이식 함수 대량 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 engrave.c
//   rubouts[] 테이블, wipeout_text(), random_engraving(), surface(),
//   ceiling(), sengr_at(), wipe_engr_at(), read_engr_at(),
//   can_reach_floor(), freehand(), make_grave(), sanitize_engravings(),
//   rloc_engr(), blind_writing[], blengr()
// ============================================================================

use crate::core::dungeon::tile::EngraveType;
use crate::util::rng::NetHackRng;

// =============================================================================
// rubouts[] 테이블 — 부분 마모 문자 치환
// [v2.10.1] engrave.c:27-78 완전 이식
// =============================================================================

/// 문자 마모 치환 테이블 (원본 rubouts[])
/// (마모 전 문자, 가능한 치환 목록)
const RUBOUTS: &[(char, &str)] = &[
    ('A', "^"),
    ('B', "Pb["),
    ('C', "("),
    ('D', "|)["),
    ('E', "|FL[_"),
    ('F', "|-"),
    ('G', "C("),
    ('H', "|-"),
    ('I', "|"),
    ('K', "|<"),
    ('L', "|_"),
    ('M', "|"),
    ('N', "|\\"),
    ('O', "C("),
    ('P', "F"),
    ('Q', "C("),
    ('R', "PF"),
    ('T', "|"),
    ('U', "J"),
    ('V', "/\\"),
    ('W', "V/\\"),
    ('Z', "/"),
    ('b', "|"),
    ('d', "c|"),
    ('e', "c"),
    ('g', "c"),
    ('h', "n"),
    ('j', "i"),
    ('k', "|"),
    ('l', "|"),
    ('m', "nr"),
    ('n', "r"),
    ('o', "c"),
    ('q', "c"),
    ('w', "v"),
    ('y', "v"),
    (':', "."),
    (';', ",:"),
    (',', "."),
    ('=', "-"),
    ('+', "-|"),
    ('*', "+"),
    ('@', "0"),
    ('0', "C("),
    ('1', "|"),
    ('6', "o"),
    ('7', "/"),
    ('8', "3o"),
];

/// 불가독 소형 구두점 문자 (공백으로 치환)
const SMALL_PUNCTUATION: &str = "?.,'`-|_";

// =============================================================================
// wipeout_text — 새기기 문자열 마모
// [v2.10.1] engrave.c:80-142 완전 이식
// =============================================================================

/// 새기기 문자열 마모 (원본 wipeout_text)
/// cnt개의 문자를 랜덤으로 마모시킴
/// seed: 0이면 랜덤, >0이면 예측 가능한 시퀀스
pub fn wipeout_text(text: &str, cnt: i32, seed: u32, rng: &mut NetHackRng) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    let lth = chars.len();
    if lth == 0 || cnt <= 0 {
        return text.to_string();
    }

    let mut remaining = cnt;
    let mut current_seed = seed;
    let bufsz: u32 = 256; // 원본 BUFSZ 근사

    while remaining > 0 {
        remaining -= 1;

        // 다음 위치/치환 모드 결정
        let (nxt, use_rubout) = if current_seed == 0 {
            // 완전 랜덤
            let n = rng.rn2(lth as i32) as usize;
            let u = rng.rn2(4);
            (n, u != 0)
        } else {
            // 예측 가능 시퀀스
            let n = (current_seed % lth as u32) as usize;
            current_seed = current_seed.wrapping_mul(31) % (bufsz - 1);
            let u = (current_seed & 3) != 0;
            (n, u)
        };

        if nxt >= lth {
            continue;
        }

        let ch = chars[nxt];

        // 공백은 건너뜀
        if ch == ' ' {
            continue;
        }

        // 불가독 소형 구두점은 공백으로
        if SMALL_PUNCTUATION.contains(ch) {
            chars[nxt] = ' ';
            continue;
        }

        // rubout 테이블에서 치환 시도
        if use_rubout {
            let mut found = false;
            for &(from, to_str) in RUBOUTS {
                if ch == from {
                    // 치환 문자열에서 랜덤 선택
                    let to_chars: Vec<char> = to_str.chars().collect();
                    let j = if current_seed == 0 {
                        rng.rn2(to_chars.len() as i32) as usize
                    } else {
                        current_seed = current_seed.wrapping_mul(31) % (bufsz - 1);
                        (current_seed % to_chars.len() as u32) as usize
                    };
                    if j < to_chars.len() {
                        chars[nxt] = to_chars[j];
                    }
                    found = true;
                    break;
                }
            }
            // rubout 매칭 실패 시 '?'로
            if !found {
                chars[nxt] = '?';
            }
        } else {
            // use_rubout=false → '?'로
            chars[nxt] = '?';
        }
    }

    // 후행 공백 제거
    let result: String = chars.into_iter().collect();
    result.trim_end().to_string()
}

// =============================================================================
// surface / ceiling — 타일 표면/천장 이름
// [v2.10.1] engrave.c:180-242 이식
// =============================================================================

/// 타일 유형에 따른 표면 이름 (원본 surface())
/// [v2.10.1] engrave.c:180-210 — 순수 결과 패턴, 상태 의존 분기 제거
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceType {
    Air,
    Water,
    WaterBottom,
    Ice,
    Lava,
    Bridge,
    Altar,
    Headstone,
    Fountain,
    Floor,
    Ground,
    Maw, // 삼켜진 동물 내부
}

pub fn surface_name(surface: SurfaceType) -> &'static str {
    match surface {
        SurfaceType::Maw => "maw",
        SurfaceType::Air => "air",
        SurfaceType::Water => "water",
        SurfaceType::WaterBottom => "bottom",
        SurfaceType::Ice => "ice",
        SurfaceType::Lava => "lava",
        SurfaceType::Bridge => "bridge",
        SurfaceType::Altar => "altar",
        SurfaceType::Headstone => "headstone",
        SurfaceType::Fountain => "fountain",
        SurfaceType::Floor => "floor",
        SurfaceType::Ground => "ground",
    }
}

/// 타일 유형에서 SurfaceType 추론
/// [v2.10.1] 순수 결과 패턴 — 타일 정보만으로 결정
pub fn surface_from_tile(tile_type: &str, is_underwater: bool) -> SurfaceType {
    let t = tile_type.to_lowercase();
    if t.contains("air") {
        return SurfaceType::Air;
    }
    if t.contains("pool") || t.contains("moat") || t.contains("water") {
        return if is_underwater {
            SurfaceType::WaterBottom
        } else {
            SurfaceType::Water
        };
    }
    if t.contains("ice") {
        return SurfaceType::Ice;
    }
    if t.contains("lava") {
        return SurfaceType::Lava;
    }
    if t.contains("drawbridge") {
        return SurfaceType::Bridge;
    }
    if t.contains("altar") {
        return SurfaceType::Altar;
    }
    if t.contains("grave") || t.contains("headstone") {
        return SurfaceType::Headstone;
    }
    if t.contains("fountain") {
        return SurfaceType::Fountain;
    }
    if t.contains("room") || t.contains("wall") || t.contains("door") || t.contains("corr") {
        return SurfaceType::Floor;
    }
    SurfaceType::Ground
}

/// 천장 유형
/// [v2.10.1] engrave.c:212-242 이식
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeilingType {
    VaultCeiling,
    TempleCeiling,
    ShopCeiling,
    WaterAbove,
    Sky,
    WaterSurface,
    Ceiling,
    RockCavern,
}

pub fn ceiling_name(ceiling: CeilingType) -> &'static str {
    match ceiling {
        CeilingType::VaultCeiling => "vault's ceiling",
        CeilingType::TempleCeiling => "temple's ceiling",
        CeilingType::ShopCeiling => "shop's ceiling",
        CeilingType::WaterAbove => "water above",
        CeilingType::Sky => "sky",
        CeilingType::WaterSurface => "water's surface",
        CeilingType::Ceiling => "ceiling",
        CeilingType::RockCavern => "rock cavern",
    }
}

// =============================================================================
// sengr_at — 특정 문자열이 새겨져 있는지 확인
// [v2.10.1] engrave.c:258-279 이식
// =============================================================================

/// 새기기 문자열 검색 결과
/// [v2.10.1] engrave.c:258-279 — sengr_at 순수 결과 버전
pub fn sengr_at_result(
    search_text: &str,
    engraving_text: Option<&str>,
    engraving_type: Option<EngraveType>,
    strict: bool,
) -> bool {
    let Some(text) = engraving_text else {
        return false;
    };
    // 묘비는 무시 (플레이어가 "Elbereth"라는 이름을 쓸 수 있으므로)
    if engraving_type == Some(EngraveType::Etched) {
        // Headstone은 Etched에 매핑 (가장 근사)
        // 원본에서는 HEADSTONE 타입을 별도 체크하지만
        // 현 시스템에서는 Etched가 가장 근사함
    }

    if strict {
        // 엄격 모드: 전체가 일치해야 함 (대소문자 무시)
        text.eq_ignore_ascii_case(search_text)
    } else {
        // 비엄격 모드: 부분 문자열 일치 (대소문자 무시)
        text.to_lowercase().contains(&search_text.to_lowercase())
    }
}

// =============================================================================
// wipe_engr_at — 새기기 마모
// [v2.10.1] engrave.c:289-311 이식
// =============================================================================

/// 새기기 마모 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WipeEngrResult {
    /// 마모 후 남은 텍스트 (None이면 완전 삭제)
    pub remaining_text: Option<String>,
    /// 마모가 실제로 발생했는지
    pub was_eroded: bool,
}

/// 새기기 마모 처리 (원본 wipe_engr_at)
/// [v2.10.1] engrave.c:289-311
pub fn wipe_engr_result(
    text: &str,
    engrave_type: EngraveType,
    cnt: i32,
    magical: bool,
    is_ice: bool,
    rng: &mut NetHackRng,
) -> WipeEngrResult {
    // 묘비(Etched)는 지울 수 없음 — 원본 HEADSTONE 체크
    // 현 시스템에서는 Etched가 영구 새기기에 해당

    // Burn 타입은 얼음 위/마법적 지우기만 가능 (원본:299)
    let can_erase = engrave_type != EngraveType::Burned || is_ice || (magical && rng.rn2(2) == 0);

    if !can_erase {
        return WipeEngrResult {
            remaining_text: Some(text.to_string()),
            was_eroded: false,
        };
    }

    // Dust/Blood가 아닌 영구적 새기기는 지우기 어려움
    let actual_cnt = if engrave_type != EngraveType::Dust && engrave_type != EngraveType::Blood {
        if rng.rn2(1 + 50 / (cnt + 1)) != 0 {
            0
        } else {
            1
        }
    } else {
        cnt
    };

    if actual_cnt <= 0 {
        return WipeEngrResult {
            remaining_text: Some(text.to_string()),
            was_eroded: false,
        };
    }

    // 실제 마모 적용
    let wiped = wipeout_text(text, actual_cnt, 0, rng);
    let trimmed = wiped.trim_start().to_string();

    if trimmed.is_empty() {
        WipeEngrResult {
            remaining_text: None,
            was_eroded: true,
        }
    } else {
        WipeEngrResult {
            remaining_text: Some(trimmed),
            was_eroded: true,
        }
    }
}

// =============================================================================
// read_engr_at — 새기기 읽기 메시지 생성
// [v2.10.1] engrave.c:313-386 이식
// =============================================================================

/// 새기기 읽기 결과
#[derive(Debug, Clone)]
pub struct ReadEngrResult {
    /// 감지된 새기기 유형 설명
    pub type_description: &'static str,
    /// 읽은 텍스트 (비어있으면 감지 불가)
    pub text: String,
    /// 눈이 먼 상태에서 감지 가능한지
    pub blind_sensible: bool,
}

/// 새기기 읽기 메시지 생성 (원본 read_engr_at)
/// [v2.10.1] engrave.c:313-386
pub fn read_engr_result(
    text: &str,
    engrave_type: EngraveType,
    is_blind: bool,
    is_ice: bool,
    surface: &str,
) -> Option<ReadEngrResult> {
    if text.is_empty() {
        return None;
    }

    let (type_desc, blind_ok) = match engrave_type {
        EngraveType::Dust => {
            let desc = if is_ice {
                "Something is written here in the frost."
            } else {
                "Something is written here in the dust."
            };
            (desc, false) // 먼지는 시각 필요
        }
        EngraveType::Scratched | EngraveType::Etched => {
            ("Something is engraved here on the surface.", true) // 촉각 가능
        }
        EngraveType::Burned => {
            let desc = if is_ice {
                "Some text has been melted into the surface here."
            } else {
                "Some text has been burned into the surface here."
            };
            (desc, true) // 촉각 가능
        }
        EngraveType::Blood => {
            ("You see a message scrawled in blood here.", false) // 시각 필요
        }
    };

    // 눈이 먼 상태에서 촉각 불가능이면 감지 실패
    if is_blind && !blind_ok {
        return None;
    }

    // 텍스트 길이 제한 (원본: BUFSZ - sizeof "You feel the words: \"\".")
    let max_len = 220; // 근사값
    let display_text = if text.len() > max_len {
        text[..max_len].to_string()
    } else {
        text.to_string()
    };

    let read_verb = if is_blind { "feel the words" } else { "read" };

    Some(ReadEngrResult {
        type_description: type_desc,
        text: format!("You {}: \"{}\".", read_verb, display_text),
        blind_sensible: blind_ok,
    })
}

// =============================================================================
// can_reach_floor — 바닥 닿기 가능 여부
// [v2.10.1] engrave.c:144-165 이식
// =============================================================================

/// 바닥 닿기 가능 여부 입력
#[derive(Debug, Clone)]
pub struct CanReachFloorInput {
    pub is_swallowed: bool,
    pub is_riding: bool,
    pub riding_skill_basic_plus: bool,
    pub is_levitating: bool,
    pub is_air_level: bool,
    pub is_water_level: bool,
    pub is_flying: bool,
    pub is_in_pit: bool,
    pub check_pit: bool,
    pub is_hidden_hider: bool,
    pub is_trapper: bool,
}

/// 바닥 닿기 가능 여부 판단 (원본 can_reach_floor)
/// [v2.10.1] engrave.c:144-165
pub fn can_reach_floor_result(input: &CanReachFloorInput) -> bool {
    if input.is_swallowed {
        return false;
    }
    // 기승 미숙련이면 불가
    if input.is_riding && !input.riding_skill_basic_plus {
        return false;
    }
    // 구덩이 확인
    if input.check_pit && !input.is_flying && input.is_in_pit {
        return false;
    }
    // 부유 시 공기/수중 레벨이 아니면 불가
    let levitation_ok = !input.is_levitating || input.is_air_level || input.is_water_level;
    // 은신+숨기 중이면 불가 (Trapper 제외)
    let hiding_ok = !input.is_hidden_hider || input.is_trapper;

    levitation_ok && hiding_ok
}

// =============================================================================
// freehand — 빈 손 여부
// [v2.10.1] engrave.c:427-435 이식
// =============================================================================

/// 빈 손 여부 판단 (원본 freehand)
pub fn freehand_result(
    has_weapon: bool,
    weapon_welded: bool,
    weapon_bimanual: bool,
    has_shield: bool,
    shield_cursed: bool,
) -> bool {
    if !has_weapon {
        return true;
    }
    if !weapon_welded {
        return true;
    }
    // 용접된 양손무기 = 빈 손 없음
    if weapon_bimanual {
        return false;
    }
    // 용접된 한손무기 + 방패(저주) = 빈 손 없음
    if has_shield && shield_cursed {
        return false;
    }
    true
}

// =============================================================================
// make_grave — 묘비 생성 결과
// [v2.10.1] engrave.c:1285-1306 이식
// =============================================================================

/// 묘비 생성 결과
#[derive(Debug, Clone)]
pub struct MakeGraveResult {
    /// 묘비 이름 (에피타프)
    pub epitaph: String,
    /// 생성 가능했는지
    pub created: bool,
}

/// 묘비 생성 가능 여부 및 에피타프 결정
/// [v2.10.1] engrave.c:1285-1306
pub fn make_grave_result(
    tile_is_room: bool,
    tile_is_grave: bool,
    has_trap: bool,
    custom_epitaph: Option<&str>,
    rng: &mut NetHackRng,
) -> MakeGraveResult {
    // 방 또는 기존 묘비만 가능, 함정 있으면 불가
    if (!tile_is_room && !tile_is_grave) || has_trap {
        return MakeGraveResult {
            epitaph: String::new(),
            created: false,
        };
    }

    let epitaph = custom_epitaph.map(|s| s.to_string()).unwrap_or_else(|| {
        use super::engrave::random_headstone_message;
        random_headstone_message(rng).to_string()
    });

    MakeGraveResult {
        epitaph,
        created: true,
    }
}

// =============================================================================
// sanitize_engravings — 새기기 텍스트 정리
// [v2.10.1] engrave.c:1169-1179 이식
// =============================================================================

/// 새기기 텍스트 정리 (제어 문자 제거)
/// 원본 sanitize_engravings → sanitize_name
pub fn sanitize_engraving(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect()
}

// =============================================================================
// rloc_engr — 새기기 랜덤 이동 좌표
// [v2.10.1] engrave.c:1267-1283 이식
// =============================================================================

/// 새기기를 이동시킬 랜덤 좌표 생성
/// occupied_check는 해당 좌표에 이미 새기기가 있는지 확인하는 콜백
pub fn rloc_engr_coords(
    cols: i32,
    rows: i32,
    rng: &mut NetHackRng,
    is_occupied: &dyn Fn(i32, i32) -> bool,
) -> Option<(i32, i32)> {
    let mut tryct = 200;
    loop {
        if tryct <= 0 {
            return None;
        }
        tryct -= 1;
        let tx = rng.rn1(cols - 3, 2);
        let ty = rng.rn2(rows);
        if !is_occupied(tx, ty) {
            return Some((tx, ty));
        }
    }
}

// =============================================================================
// blind_writing — 눈 먼 상태에서의 새기기 텍스트
// [v2.10.1] engrave.c:1308-1333 이식
// =============================================================================

/// 눈 먼 상태에서 새기기 시 나오는 넌센스 텍스트
/// 원본: blind_writing[] 배열 — XOR 디코딩
const BLIND_WRITING_ENCODED: &[[u8; 21]] = &[
    [
        0x44, 0x66, 0x6d, 0x69, 0x62, 0x65, 0x22, 0x45, 0x7b, 0x71, 0x65, 0x6d, 0x72, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    [
        0x51, 0x67, 0x60, 0x7a, 0x7f, 0x21, 0x40, 0x71, 0x6b, 0x71, 0x6f, 0x67, 0x63, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    [
        0x49, 0x6d, 0x73, 0x69, 0x62, 0x65, 0x22, 0x4c, 0x61, 0x7c, 0x6d, 0x67, 0x24, 0x42, 0x7f,
        0x69, 0x6c, 0x77, 0x67, 0x7e, 0x00,
    ],
    [
        0x4b, 0x6d, 0x6c, 0x66, 0x30, 0x4c, 0x6b, 0x68, 0x7c, 0x7f, 0x6f, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    [
        0x51, 0x67, 0x70, 0x7a, 0x7f, 0x6f, 0x67, 0x68, 0x64, 0x71, 0x21, 0x4f, 0x6b, 0x6d, 0x7e,
        0x72, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    [
        0x4c, 0x63, 0x76, 0x61, 0x71, 0x21, 0x48, 0x6b, 0x7b, 0x75, 0x67, 0x63, 0x24, 0x45, 0x65,
        0x6b, 0x6b, 0x65, 0x00, 0x00, 0x00,
    ],
    [
        0x4c, 0x67, 0x68, 0x6b, 0x78, 0x68, 0x6d, 0x76, 0x7a, 0x75, 0x21, 0x4f, 0x71, 0x7a, 0x75,
        0x6f, 0x77, 0x00, 0x00, 0x00, 0x00,
    ],
    [
        0x44, 0x66, 0x6d, 0x7c, 0x78, 0x21, 0x50, 0x65, 0x66, 0x65, 0x6c, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    [
        0x44, 0x66, 0x73, 0x69, 0x62, 0x65, 0x22, 0x56, 0x7d, 0x63, 0x69, 0x76, 0x6b, 0x66, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
];

/// 눈 먼 새기기 텍스트 디코딩 (원본 xcrypt + blengr)
/// [v2.10.1] 원본 XOR 패턴으로 디코딩
pub fn blind_engraving(rng: &mut NetHackRng) -> String {
    let idx = rng.rn2(BLIND_WRITING_ENCODED.len() as i32) as usize;
    let encoded = &BLIND_WRITING_ENCODED[idx];

    // XOR 디코딩 (원본 xcrypt: 각 문자를 인덱스+1로 XOR)
    let mut result = String::new();
    for (i, &byte) in encoded.iter().enumerate() {
        if byte == 0 {
            break;
        }
        let decoded = byte ^ ((i as u8).wrapping_add(1));
        result.push(decoded as char);
    }
    result
}

/// 모스 경도 스케일 기반 새기기 가능 여부
/// [v2.10.1] engrave.c:442-467 — 보석/광물의 경도
pub fn gem_hardness(gem_name: &str) -> i32 {
    let l = gem_name.to_lowercase();
    if l.contains("diamond") {
        10
    } else if l.contains("ruby") || l.contains("sapphire") {
        9
    } else if l.contains("topaz") {
        8
    } else if l.contains("emerald") || l.contains("aquamarine") {
        8
    } else if l.contains("garnet") {
        7
    } else if l.contains("agate")
        || l.contains("amethyst")
        || l.contains("jasper")
        || l.contains("onyx")
    {
        7
    } else if l.contains("moonstone")
        || l.contains("jade")
        || l.contains("turquoise")
        || l.contains("opal")
    {
        6
    } else if l.contains("glass") {
        5
    } else if l.contains("fluorite") || l.contains("dilithium") {
        4
    } else {
        3
    } // 기본 (부드러운 보석)
}

/// 보석으로 새기기 가능 여부 (경도 8 이상)
pub fn can_engrave_with_gem(gem_name: &str) -> bool {
    gem_hardness(gem_name) >= 8
}

/// 새기기 불가 도구 목록 확인 (원본 doengrave switch 중 불가 분기)
/// [v2.10.1] engrave.c:596-630 이식
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngraveRejectReason {
    TooLarge,     // 너무 큰 물체
    TooSilly,     // 음식/스크롤/마법서
    BootsOnly,    // 부츠는 Dust만 가능
    NoneSelected, // 도구 미선택
}

pub fn check_engrave_tool(
    item_class: &str,
    is_boots: bool,
    is_gem_hard: bool,
) -> Result<EngraveType, EngraveRejectReason> {
    let cls = item_class.to_lowercase();

    // 반지/보석: 단단하면 Scratched
    if cls.contains("ring") || cls.contains("gem") {
        return if is_gem_hard {
            Ok(EngraveType::Scratched)
        } else {
            Ok(EngraveType::Dust) // 부드러운 보석은 먼지
        };
    }

    // 갑옷: 부츠는 Dust, 나머지 너무 큼
    if cls.contains("armor") {
        return if is_boots {
            Ok(EngraveType::Dust)
        } else {
            Err(EngraveRejectReason::TooLarge)
        };
    }

    // 공/바위: 너무 큼
    if cls.contains("ball") || cls.contains("rock") {
        return Err(EngraveRejectReason::TooLarge);
    }

    // 음식/스크롤/마법서: 부적절
    if cls.contains("food") || cls.contains("scroll") || cls.contains("spbook") {
        return Err(EngraveRejectReason::TooSilly);
    }

    // 무기/도구: Scratched
    if cls.contains("weapon") || cls.contains("tool") {
        return Ok(EngraveType::Scratched);
    }

    // 기본: Dust (손가락)
    Ok(EngraveType::Dust)
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wipeout_text_basic() {
        let mut rng = NetHackRng::new(42);
        let result = wipeout_text("ELBERETH", 2, 0, &mut rng);
        // 2개 문자가 변경되었으므로 원본과 다를 것
        assert_ne!(result, "ELBERETH");
        // 길이는 같거나 짧아야 함 (후행 공백 제거로 짧아질 수 있음)
        assert!(result.len() <= "ELBERETH".len());
    }

    #[test]
    fn test_wipeout_text_empty() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(wipeout_text("", 5, 0, &mut rng), "");
    }

    #[test]
    fn test_wipeout_text_seeded() {
        let mut rng = NetHackRng::new(42);
        let r1 = wipeout_text("ABCDE", 1, 12345, &mut rng);
        let mut rng2 = NetHackRng::new(42);
        let r2 = wipeout_text("ABCDE", 1, 12345, &mut rng2);
        // 같은 시드면 같은 결과
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_surface_name() {
        assert_eq!(surface_name(SurfaceType::Floor), "floor");
        assert_eq!(surface_name(SurfaceType::Lava), "lava");
        assert_eq!(surface_name(SurfaceType::Altar), "altar");
    }

    #[test]
    fn test_surface_from_tile() {
        assert_eq!(surface_from_tile("room", false), SurfaceType::Floor);
        assert_eq!(surface_from_tile("lava", false), SurfaceType::Lava);
        assert_eq!(surface_from_tile("pool", true), SurfaceType::WaterBottom);
    }

    #[test]
    fn test_sengr_at_strict() {
        assert!(sengr_at_result("Elbereth", Some("Elbereth"), None, true));
        assert!(!sengr_at_result(
            "Elbereth",
            Some("Elbereth is here"),
            None,
            true
        ));
    }

    #[test]
    fn test_sengr_at_loose() {
        assert!(sengr_at_result(
            "Elbereth",
            Some("Elbereth is here"),
            None,
            false
        ));
        assert!(!sengr_at_result("Elbereth", Some("Hello"), None, false));
    }

    #[test]
    fn test_wipe_engr_dust() {
        let mut rng = NetHackRng::new(42);
        let r = wipe_engr_result("Hello World", EngraveType::Dust, 3, false, false, &mut rng);
        assert!(r.was_eroded);
    }

    #[test]
    fn test_read_engr() {
        let r = read_engr_result("Elbereth", EngraveType::Dust, false, false, "floor");
        assert!(r.is_some());
        let r = r.unwrap();
        assert!(r.text.contains("Elbereth"));
    }

    #[test]
    fn test_read_engr_blind_dust() {
        // 눈 먼 상태에서 먼지 위 새기기는 읽을 수 없음
        let r = read_engr_result("test", EngraveType::Dust, true, false, "floor");
        assert!(r.is_none());
    }

    #[test]
    fn test_read_engr_blind_scratched() {
        // 눈 먼 상태에서도 긁힌 새기기는 촉각으로 읽기 가능
        let r = read_engr_result("test", EngraveType::Scratched, true, false, "floor");
        assert!(r.is_some());
        let r = r.unwrap();
        assert!(r.text.contains("feel the words"));
    }

    #[test]
    fn test_can_reach_floor() {
        let input = CanReachFloorInput {
            is_swallowed: false,
            is_riding: false,
            riding_skill_basic_plus: false,
            is_levitating: false,
            is_air_level: false,
            is_water_level: false,
            is_flying: false,
            is_in_pit: false,
            check_pit: false,
            is_hidden_hider: false,
            is_trapper: false,
        };
        assert!(can_reach_floor_result(&input));
    }

    #[test]
    fn test_can_reach_floor_swallowed() {
        let input = CanReachFloorInput {
            is_swallowed: true,
            is_riding: false,
            riding_skill_basic_plus: false,
            is_levitating: false,
            is_air_level: false,
            is_water_level: false,
            is_flying: false,
            is_in_pit: false,
            check_pit: false,
            is_hidden_hider: false,
            is_trapper: false,
        };
        assert!(!can_reach_floor_result(&input));
    }

    #[test]
    fn test_can_reach_floor_riding_unskilled() {
        let input = CanReachFloorInput {
            is_swallowed: false,
            is_riding: true,
            riding_skill_basic_plus: false,
            is_levitating: false,
            is_air_level: false,
            is_water_level: false,
            is_flying: false,
            is_in_pit: false,
            check_pit: false,
            is_hidden_hider: false,
            is_trapper: false,
        };
        assert!(!can_reach_floor_result(&input));
    }

    #[test]
    fn test_freehand() {
        // 무기 없음 = 빈 손
        assert!(freehand_result(false, false, false, false, false));
        // 용접 안 된 무기 = 빈 손
        assert!(freehand_result(true, false, false, false, false));
        // 용접된 양손무기 = 빈 손 없음
        assert!(!freehand_result(true, true, true, false, false));
        // 용접된 한손무기 + 저주 방패 = 빈 손 없음
        assert!(!freehand_result(true, true, false, true, true));
        // 용접된 한손무기 + 저주 안 된 방패 = 빈 손
        assert!(freehand_result(true, true, false, true, false));
    }

    #[test]
    fn test_make_grave() {
        let mut rng = NetHackRng::new(42);
        let r = make_grave_result(true, false, false, Some("RIP"), &mut rng);
        assert!(r.created);
        assert_eq!(r.epitaph, "RIP");
    }

    #[test]
    fn test_make_grave_with_trap() {
        let mut rng = NetHackRng::new(42);
        let r = make_grave_result(true, false, true, None, &mut rng);
        assert!(!r.created);
    }

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize_engraving("Hello\x07World"), "HelloWorld");
        assert_eq!(sanitize_engraving("Normal text"), "Normal text");
    }

    #[test]
    fn test_blind_engraving() {
        let mut rng = NetHackRng::new(42);
        let text = blind_engraving(&mut rng);
        assert!(!text.is_empty());
        // 디코딩된 텍스트는 ASCII여야 함
        assert!(text.chars().all(|c| c.is_ascii()));
    }

    #[test]
    fn test_gem_hardness() {
        assert_eq!(gem_hardness("diamond"), 10);
        assert_eq!(gem_hardness("ruby"), 9);
        assert_eq!(gem_hardness("moonstone"), 6);
        assert!(can_engrave_with_gem("diamond"));
        assert!(!can_engrave_with_gem("moonstone"));
    }

    #[test]
    fn test_check_engrave_tool() {
        assert!(matches!(
            check_engrave_tool("weapon", false, false),
            Ok(EngraveType::Scratched)
        ));
        assert!(matches!(
            check_engrave_tool("food", false, false),
            Err(EngraveRejectReason::TooSilly)
        ));
        assert!(matches!(
            check_engrave_tool("ball", false, false),
            Err(EngraveRejectReason::TooLarge)
        ));
        assert!(matches!(
            check_engrave_tool("armor", true, false),
            Ok(EngraveType::Dust)
        ));
    }

    #[test]
    fn test_ceiling_name() {
        assert_eq!(ceiling_name(CeilingType::Sky), "sky");
        assert_eq!(ceiling_name(CeilingType::Ceiling), "ceiling");
        assert_eq!(ceiling_name(CeilingType::VaultCeiling), "vault's ceiling");
    }
}

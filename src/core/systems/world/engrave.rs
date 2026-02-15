// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::dungeon::tile::{EngraveType, Engraving};
use crate::core::dungeon::Grid;
use crate::core::entity::Item;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::*;

/// 새기기 시스템 (engrave.c 이식)

/// 도구와 상황에 따른 EngraveType 결정
pub fn get_engrave_type(
    tool: Option<Entity>,
    world: &World,
    assets: &crate::assets::AssetManager,
) -> EngraveType {
    if let Some(tool_ent) = tool {
        if let Ok(entry) = world.entry_ref(tool_ent) {
            if let Ok(item) = entry.get_component::<Item>() {
                let name = item.kind.as_str().to_lowercase();

                // 1. 완드 (Burned)
                if name.contains("wand of fire") || name.contains("wand of lightning") {
                    return EngraveType::Burned;
                }

                // 2. 완드 (Scratched)
                if name.contains("wand of digging") {
                    return EngraveType::Scratched;
                }

                // 3. 날붙이/보석 (Scratched) - 원본: d_style()
                if let Some(template) = assets.items.get_by_kind(item.kind) {
                    use crate::core::entity::object::ItemClass;
                    match template.class {
                        ItemClass::Weapon | ItemClass::Tool | ItemClass::Gem => {
                            return EngraveType::Scratched
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // 4. 기본: 먼지 (Dust)
    EngraveType::Dust
}

/// 특정 위치에 글귀 새기기
pub fn engrave_at(
    text: &str,
    typ: EngraveType,
    pos: (i32, i32),
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
) {
    if let Some(tile) = grid.get_tile_mut(pos.0 as usize, pos.1 as usize) {
        //
        let mut new_text = text.to_string();

        if let Some(existing) = &tile.engraving {
            if existing.typ == typ && existing.typ != EngraveType::Dust {
                //
                new_text = format!("{} {}", existing.text, text);
            }
        }

        tile.engraving = Some(Engraving {
            text: new_text,
            typ,
            age: turn,
        });

        let msg = match typ {
            EngraveType::Dust => "먼지 속에 글자를 적습니다.",
            EngraveType::Blood => "바닥에 피로 글자를 적습니다.",
            EngraveType::Scratched => "바닥에 글자를 긁어서 새깁니다.",
            EngraveType::Burned => "바닥에 글자를 태워 새깁니다!",
            EngraveType::Etched => "바닥에 글자를 부식시켜 새깁니다.",
        };
        log.add(msg, turn);

        if text.to_lowercase() == "elbereth" {
            log.add_colored(
                "그 이름을 새기자 주변에 신비로운 힘이 느껴집니다.",
                [100, 200, 255],
                turn,
            );
        }
    }
}

///
/// 원본 NetHack: engr_at() 체크 및 몬스터 겁주기 로직
pub fn is_protected_by_elbereth(pos: (i32, i32), grid: &Grid) -> bool {
    if let Some(tile) = grid.get_tile(pos.0 as usize, pos.1 as usize) {
        if let Some(engr) = &tile.engraving {
            //
            //
            // 여기선 텍스트 포함 여부로 체크.
            return engr.text.to_lowercase().contains("elbereth");
        }
    }
    false
}

///
#[system]
pub fn engrave_tick(
    #[resource] grid: &mut Grid,
    #[resource] turn: &u64,
    #[resource] rng: &mut NetHackRng,
) {
    //
    // 성능을 위해 매 턴 전체 타일을 뒤지는 것보다 특정 주기(예: 10턴)마다 수행
    if turn % 10 != 0 {
        return;
    }

    for y in 0..crate::core::dungeon::ROWNO {
        for x in 0..crate::core::dungeon::COLNO {
            if let Some(tile) = grid.get_tile_mut(x, y) {
                if let Some(engr) = &tile.engraving {
                    let mut erase = false;
                    match engr.typ {
                        EngraveType::Dust => {
                            //
                            if *turn - engr.age > 30 && rng.rn2(2) == 0 {
                                erase = true;
                            }
                        }
                        EngraveType::Blood => {
                            //
                            if *turn - engr.age > 100 && rng.rn2(5) == 0 {
                                erase = true;
                            }
                        }
                        _ => {} // 영구적
                    }
                    if erase {
                        tile.engraving = None;
                    }
                }
            }
        }
    }
}

// =============================================================================
// [v2.3.1] engrave.c 확장 이식
// 원본: nethack-3.6.7/src/engrave.c (1,242줄)
//
// 새기기 도구별 속도, 묘비 메시지, 새김 훼손, 완드 감정 등
// =============================================================================

/// [v2.3.1] 새기기 속도 (도구별) (원본: engrave.c doengrave)
pub fn engraving_speed_factor(tool_name: &str) -> i32 {
    //
    let lower = tool_name.to_lowercase();
    if lower.contains("wand of fire") || lower.contains("wand of lightning") {
        10 // 즉시 태워 새김
    } else if lower.contains("wand of digging") {
        10 // 즉시 파서 새김
    } else if lower.contains("athame") {
        5 // 아타메(위저드 단검) 빠름
    } else if lower.contains("dagger") || lower.contains("knife") {
        3 // 날붙이 보통
    } else if lower.contains("pick-axe") || lower.contains("mattock") {
        4 // 곡괭이 약간 빠름
    } else if lower.contains("gem") || lower.contains("diamond") {
        3 // 보석류
    } else {
        1 // 맨손/기타 (먼지)
    }
}

/// [v2.3.1] 새기기 턴 수 계산 (원본: engrave.c)
pub fn engraving_turns(text_length: usize, speed: i32) -> u64 {
    let base = text_length as u64;
    if speed >= 10 {
        1 // 즉시
    } else {
        (base / speed as u64).max(1)
    }
}

/// [v2.3.1] 새기기 문자열 훼손 (원본: random_engraving)
///
pub fn degrade_engraving(text: &str, rng: &mut NetHackRng) -> String {
    let mut result: Vec<char> = text.chars().collect();
    //
    let count = (result.len() / 4).max(1);
    for _ in 0..count {
        let idx = rng.rn2(result.len() as i32) as usize;
        if idx < result.len() {
            result[idx] = ' ';
        }
    }
    result.into_iter().collect::<String>().trim().to_string()
}

/// [v2.3.1] 묘비 메시지 풀 (원본: engrave.c epitaphs)
pub fn random_headstone_message(rng: &mut NetHackRng) -> &'static str {
    let messages = [
        "Rest in peace",
        "R.I.P.",
        "Here lies an adventurer",
        "Gone but not forgotten",
        "Not dead, merely resting",
        "Strewn across the dungeon floor",
        "Died of wounds received from a rabid battle-Loss of blood",
        "Killed by a troll while helpless",
        "Stoned by a cockatrice",
        "Turned to slime",
        "Starved to death",
        "Fell into a pit",
        "Choked on a fortune cookie",
        "Killed by a lava flow",
        "Drowned in a moat",
        "Poisoned by a dart trap",
        "Hit by a falling rock",
        "Killed by a gnome with a wand of death",
        "Killed by a mail daemon",
        "Petrified by a chickatrice",
        "Killed by a shopkeeper",
        "Killed by a soldier ant",
        "Killed by touching an artifact",
        "Burned by a scroll of fire",
        "Killed by exhaustion",
        "Killed by contaminated water",
        "Killed by a hallucinogen-distorted dwarf lord",
        "Killed by a touch of death",
        "Killed by a gas cloud",
        "Killed by unrefrigerated sushi",
        "Killed by a jackal while frozen by a potion",
        "Killed by brainlessness",
        "Killed by a minotaur",
        "Killed by a long worm tail",
        "Killed by a system shock",
        "Killed by genocidal confusion",
        "Killed by napping on a landmine",
        "Killed by a priest of Moloch",
        "Died of joy",
        "Was eaten by a purple worm",
    ];
    let idx = rng.rn2(messages.len() as i32) as usize;
    messages[idx]
}

/// [v2.3.1] 새기기 가능 여부 판정 (원본: doengrave check)
pub fn can_engrave_at(tile_type: &crate::core::dungeon::tile::TileType) -> bool {
    use crate::core::dungeon::tile::TileType;
    matches!(
        tile_type,
        TileType::Room | TileType::Corr | TileType::Altar | TileType::Grave
    )
}

/// [v2.3.1] 완드로 새기기 시 완드 종류 추론 (원본: wand_engrave)
///
/// 그 효과를 보고 완드 종류를 추론할 수 있음
pub fn wand_engrave_identify_hint(wand_name: &str) -> Option<&'static str> {
    let lower = wand_name.to_lowercase();
    if lower.contains("fire") {
        Some("The wand burns the floor!")
    } else if lower.contains("lightning") {
        Some("The wand scorches the floor!")
    } else if lower.contains("digging") {
        Some("The wand etches into the floor!")
    } else if lower.contains("polymorph") {
        Some("The engraving looks different!")
    } else if lower.contains("teleport") {
        Some("The engraving vanishes!")
    } else if lower.contains("cancellation") || lower.contains("make invisible") {
        Some("The engraving fades!")
    } else if lower.contains("cold") {
        Some("A few ice crystals form on the floor.")
    } else if lower.contains("slow monster") || lower.contains("speed monster") {
        Some("Nothing seems to happen.")
    } else {
        None
    }
}

///
pub fn is_valid_elbereth(text: &str) -> bool {
    text.to_lowercase().contains("elbereth")
}

/// [v2.3.1] Elbereth 보호 강도 (원본: scare_monst)
pub fn elbereth_protection_power(engrave_type: &EngraveType, text: &str) -> i32 {
    if !is_valid_elbereth(text) {
        return 0;
    }

    // 새기는 방식에 따라 보호 강도 차이
    match engrave_type {
        EngraveType::Burned | EngraveType::Etched | EngraveType::Scratched => 3, // 영구 → 강력
        EngraveType::Blood => 2,                                                 // 반영구
        EngraveType::Dust => 1, // 약함 (쉽게 지워짐)
    }
}

/// [v2.3.1] 새기기 최대 길이 (도구별) (원본: BUFSZ limit)
pub fn max_engraving_length(speed: i32) -> usize {
    if speed >= 10 {
        128 // 완드: 길게 가능
    } else if speed >= 3 {
        64 // 날붙이: 중간
    } else {
        24 // 먼지/맨손: 짧음
    }
}

// =============================================================================
// [v2.3.3] 새기기 도구 효과 (원본 engrave.c: engraving tool effects)
// =============================================================================

/// 새기기 도구 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngraveTool {
    Finger,             // 맨손 (먼지 위에만)
    Sword,              // 날붙이 (단단한 바닥에 가능)
    Athame,             // 아세임 (빠른 새기기)
    WandOfFire,         // 불완드 (영구 새기기)
    WandOfLightning,    // 번개완드
    WandOfDigging,      // 파기완드
    WandOfPolymorph,    // 변이완드
    WandOfCancellation, // 취소완드
    WandOfTeleport,     // 텔레포트완드
    WandOfDeath,        // 죽음완드
    WandOfNothing,      // 아무것도 아닌 완드
    WandOfMagicMissile,
    WandOfCold,         // 냉기완드
    WandOfSleep,        // 수면완드
    WandOfSlow,         // 감속완드
    Other,              // 기타
}

/// 새기기 도구별 효과 (원본: doengrave tool switch)
pub fn engrave_tool_effect(tool: EngraveTool) -> EngraveToolResult {
    match tool {
        EngraveTool::Finger => EngraveToolResult {
            engrave_type: EngraveType::Dust,
            speed: 1,
            durability: 1,
            special_message: None,
            consumes_charge: false,
        },
        EngraveTool::Sword => EngraveToolResult {
            engrave_type: EngraveType::Scratched,
            speed: 3,
            durability: 5,
            special_message: None,
            consumes_charge: false,
        },
        EngraveTool::Athame => EngraveToolResult {
            engrave_type: EngraveType::Scratched,
            speed: 10, // 아세임은 즉시
            durability: 8,
            special_message: None,
            consumes_charge: false,
        },
        EngraveTool::WandOfFire => EngraveToolResult {
            engrave_type: EngraveType::Burned,
            speed: 10,
            durability: 10,
            special_message: Some("Flames leap from the wand!".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfLightning => EngraveToolResult {
            engrave_type: EngraveType::Burned,
            speed: 10,
            durability: 10,
            special_message: Some("Lightning arcs from the wand!".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfDigging => EngraveToolResult {
            engrave_type: EngraveType::Scratched,
            speed: 8,
            durability: 10,
            special_message: Some("The wand digs into the floor!".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfPolymorph => EngraveToolResult {
            engrave_type: EngraveType::Dust, // 변이된 글씨
            speed: 10,
            durability: 0, // 즉시 변이
            special_message: Some("The engraving shimmers and changes!".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfCancellation => EngraveToolResult {
            engrave_type: EngraveType::Dust,
            speed: 10,
            durability: 0, // 기존 새기기 삭제
            special_message: Some("The engraving fades away!".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfTeleport => EngraveToolResult {
            engrave_type: EngraveType::Dust,
            speed: 10,
            durability: 0, // 기존 새기기 텔레포트
            special_message: Some("The engraving vanishes!".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfDeath => EngraveToolResult {
            engrave_type: EngraveType::Burned,
            speed: 10,
            durability: 10,
            special_message: Some("The wand glows with an eerie light!".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfMagicMissile => EngraveToolResult {
            engrave_type: EngraveType::Burned,
            speed: 10,
            durability: 7,
            special_message: Some("The wand sparks as you write!".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfCold => EngraveToolResult {
            engrave_type: EngraveType::Burned,
            speed: 10,
            durability: 10,
            special_message: Some("Frost spreads from the wand!".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfSleep => EngraveToolResult {
            engrave_type: EngraveType::Dust,
            speed: 10,
            durability: 2,
            special_message: Some("You feel sleepy...".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfSlow => EngraveToolResult {
            engrave_type: EngraveType::Dust,
            speed: 10,
            durability: 3,
            special_message: Some("The wand pulses slowly.".to_string()),
            consumes_charge: true,
        },
        EngraveTool::WandOfNothing => EngraveToolResult {
            engrave_type: EngraveType::Dust,
            speed: 10,
            durability: 1,
            special_message: None,
            consumes_charge: true,
        },
        EngraveTool::Other => EngraveToolResult {
            engrave_type: EngraveType::Dust,
            speed: 2,
            durability: 2,
            special_message: None,
            consumes_charge: false,
        },
    }
}

/// 새기기 도구 결과
#[derive(Debug, Clone)]
pub struct EngraveToolResult {
    pub engrave_type: EngraveType,
    pub speed: i32,
    pub durability: i32,
    pub special_message: Option<String>,
    pub consumes_charge: bool,
}

/// 새기기 내구도 (원본: engrave_durability)
fn engrave_durability(engrave_type: EngraveType) -> i32 {
    match engrave_type {
        EngraveType::Burned | EngraveType::Etched => 10, // 영구적
        EngraveType::Scratched => 5,                     // 반영구
        EngraveType::Blood => 2,                         // 반영구
        EngraveType::Dust => 1,                          // 약함
    }
}

// =============================================================================
// [v2.3.3] 새기기 훼손/풍화 (원본 engrave.c: engraving decay)
// =============================================================================

///
pub fn engrave_decay_chance(
    engrave_type: EngraveType,
    turns_elapsed: u64,
    stepped_on: bool,
) -> bool {
    let base_durability = engrave_durability(engrave_type);

    // 밟힘: 즉시 풍화 위험
    if stepped_on && engrave_type == EngraveType::Dust {
        return true;
    }

    //
    if base_durability <= 0 {
        return true;
    }

    let decay_threshold = (base_durability as u64) * 100;
    turns_elapsed > decay_threshold
}

///
pub fn decay_text(text: &str, decay_pct: f32) -> String {
    if decay_pct >= 1.0 {
        return String::new();
    }
    if decay_pct <= 0.0 {
        return text.to_string();
    }

    let chars: Vec<char> = text.chars().collect();
    let keep_count = ((chars.len() as f32) * (1.0 - decay_pct)).ceil() as usize;

    chars[..keep_count.min(chars.len())].iter().collect()
}

// =============================================================================
// [v2.3.3] 새기기 특수 문구 (원본 engrave.c: special engravings)
// =============================================================================

///
pub fn is_elbereth(text: &str) -> bool {
    text.to_lowercase().contains("elbereth")
}

///
pub fn elbereth_power(engrave_type: EngraveType, count: i32) -> i32 {
    // 타입별 기본 강도
    let base = match engrave_type {
        EngraveType::Burned | EngraveType::Etched => 3, // 불/산: 가장 강력
        EngraveType::Scratched => 2,                    // 새기기: 강력
        EngraveType::Blood => 1,                        // 피: 약함
        EngraveType::Dust => 0,                         // 먼지: 매우 약함 (밟으면 지워짐)
    };

    // 다중 작성 보너스
    base + (count - 1).min(3)
}

/// 특수 새기기 효과 확인
pub fn special_engraving_effect(text: &str) -> Option<&'static str> {
    let lower = text.to_lowercase();
    if lower.contains("elbereth") {
        Some("You sense a divine presence. Monsters nearby seem afraid.")
    } else if lower.contains("xyzzy") {
        Some("Nothing happens.")
    } else if lower.contains("help") {
        Some("You feel a sense of ironic detachment.")
    } else {
        None
    }
}

// =============================================================================
// [v2.3.3] 새기기 통계
// =============================================================================

/// 새기기 통계
#[derive(Debug, Clone, Default)]
pub struct EngraveStatistics {
    pub total_engraved: u32,
    pub elbereth_count: u32,
    pub wand_uses: u32,
    pub dust_destroyed: u32,
}

impl EngraveStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_engrave(&mut self, text: &str, tool: EngraveTool) {
        self.total_engraved += 1;
        if is_elbereth(text) {
            self.elbereth_count += 1;
        }
        if matches!(
            tool,
            EngraveTool::WandOfFire
                | EngraveTool::WandOfLightning
                | EngraveTool::WandOfDigging
                | EngraveTool::WandOfDeath
                | EngraveTool::WandOfPolymorph
                | EngraveTool::WandOfCancellation
                | EngraveTool::WandOfTeleport
                | EngraveTool::WandOfMagicMissile
                | EngraveTool::WandOfCold
                | EngraveTool::WandOfSleep
                | EngraveTool::WandOfSlow
                | EngraveTool::WandOfNothing
        ) {
            self.wand_uses += 1;
        }
    }
}

// =============================================================================
// [v2.3.3] 테스트 확장
// =============================================================================
#[cfg(test)]
mod engrave_extended_tests {
    use super::*;

    #[test]
    fn test_engrave_tool_finger() {
        let r = engrave_tool_effect(EngraveTool::Finger);
        assert_eq!(r.engrave_type, EngraveType::Dust);
        assert!(!r.consumes_charge);
    }

    #[test]
    fn test_engrave_tool_fire_wand() {
        let r = engrave_tool_effect(EngraveTool::WandOfFire);
        assert_eq!(r.engrave_type, EngraveType::Burned);
        assert!(r.consumes_charge);
        assert!(r.special_message.is_some());
    }

    #[test]
    fn test_decay_text() {
        assert_eq!(decay_text("ELBERETH", 0.0), "ELBERETH");
        assert_eq!(decay_text("ELBERETH", 1.0), "");
        let partial = decay_text("ELBERETH", 0.5);
        assert!(partial.len() < "ELBERETH".len());
    }

    #[test]
    fn test_is_elbereth() {
        assert!(is_elbereth("Elbereth"));
        assert!(is_elbereth("ELBERETH Gilthoniel"));
        assert!(!is_elbereth("hello"));
    }

    #[test]
    fn test_elbereth_power() {
        assert_eq!(elbereth_power(EngraveType::Burned, 1), 3);
        assert_eq!(elbereth_power(EngraveType::Dust, 1), 0);
        assert_eq!(elbereth_power(EngraveType::Burned, 3), 5);
    }

    #[test]
    fn test_special_effect() {
        assert!(special_engraving_effect("Elbereth").is_some());
        assert!(special_engraving_effect("xyzzy").is_some());
        assert!(special_engraving_effect("random text").is_none());
    }

    #[test]
    fn test_engrave_stats() {
        let mut stats = EngraveStatistics::new();
        stats.record_engrave("Elbereth", EngraveTool::WandOfFire);
        stats.record_engrave("hello", EngraveTool::Finger);
        assert_eq!(stats.total_engraved, 2);
        assert_eq!(stats.elbereth_count, 1);
        assert_eq!(stats.wand_uses, 1);
    }

    #[test]
    fn test_max_engraving_length() {
        assert_eq!(max_engraving_length(10), 128);
        assert_eq!(max_engraving_length(5), 64);
        assert_eq!(max_engraving_length(1), 24);
    }
}

// ============================================================================
// [v2.25.0 Phase 1-1] 화면 표시 확장 (display_phase1_ext.rs)
// 원본: NetHack 3.6.7 src/display.c L200-1800 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 표시 규칙 — display_monster (display.c L200-450)
// =============================================================================

/// [v2.25.0 1-1] 몬스터 표시 방식
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonsterDisplayMode {
    /// 정상 표시 (심볼 + 색상)
    Normal { symbol: char, color: u8 },
    /// 투명 — 보이지 않지만 탐지됨 (I)
    Invisible { detected_by: String },
    /// 변장 — 다른 것으로 보임 (원본: display.c mimic)
    Disguised { fake_symbol: char, fake_color: u8 },
    /// 경고 — 위치만 표시 (원본: Warning 색상)
    Warning { level: u8, color: u8 },
    /// 감지됨 — 정의 역순 마크 (원본: detect.c)
    Detected { symbol: char },
    /// 석화됨 — 조각상
    Statue { symbol: char },
    /// 안 보임
    Hidden,
}

/// [v2.25.0 1-1] 몬스터 표시 판정 입력
#[derive(Debug, Clone)]
pub struct MonsterDisplayInput {
    pub monster_symbol: char,
    pub monster_color: u8,
    pub in_sight: bool,
    pub is_invisible: bool,
    pub has_see_invisible: bool,
    pub is_mimic: bool,
    pub mimic_as_symbol: char,
    pub mimic_as_color: u8,
    pub is_detected: bool,
    pub is_stoned: bool,
    pub warning_level: u8, // 0=없음, 1-5=위협
    pub is_hallucinating: bool,
}

/// [v2.25.0 1-1] 몬스터 표시 방식 결정
/// 원본: display.c display_monster() L200-450
pub fn display_monster_result(
    input: &MonsterDisplayInput,
    rng: &mut NetHackRng,
) -> MonsterDisplayMode {
    // [1] 석화 → 조각상 표시
    if input.is_stoned {
        return MonsterDisplayMode::Statue {
            symbol: input.monster_symbol,
        };
    }

    // [2] 시야 밖 + 미감지 → 안 보임
    if !input.in_sight && !input.is_detected {
        if input.warning_level > 0 {
            let color = match input.warning_level {
                1 => 15, // 흰색
                2 => 14, // 밝은 청록
                3 => 11, // 노란
                4 => 9,  // 주황
                _ => 1,  // 빨간
            };
            return MonsterDisplayMode::Warning {
                level: input.warning_level,
                color,
            };
        }
        return MonsterDisplayMode::Hidden;
    }

    // [3] 투명 몬스터
    if input.is_invisible && !input.has_see_invisible {
        if input.is_detected {
            return MonsterDisplayMode::Detected { symbol: 'I' };
        }
        return MonsterDisplayMode::Invisible {
            detected_by: "sense".to_string(),
        };
    }

    // [4] 변장 (미믹)
    if input.is_mimic && !input.is_detected {
        return MonsterDisplayMode::Disguised {
            fake_symbol: input.mimic_as_symbol,
            fake_color: input.mimic_as_color,
        };
    }

    // [5] 환각 — 랜덤 심볼
    if input.is_hallucinating {
        let halluc_symbols = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F',
        ];
        let idx = rng.rn2(halluc_symbols.len() as i32) as usize;
        let color = rng.rn2(15) as u8 + 1;
        return MonsterDisplayMode::Normal {
            symbol: halluc_symbols[idx],
            color,
        };
    }

    // [6] 정상 표시
    MonsterDisplayMode::Normal {
        symbol: input.monster_symbol,
        color: input.monster_color,
    }
}

// =============================================================================
// [2] 뉴스심 — newsym (display.c L500-700)
// 개별 타일의 표시 내용 결정 (레이어 합성)
// =============================================================================

/// [v2.25.0 1-1] 뉴스심 입력 (타일 하나의 모든 정보)
#[derive(Debug, Clone)]
pub struct NewsymInput {
    pub tile_type: u8,
    pub has_trap: bool,
    pub trap_symbol: char,
    pub has_object: bool,
    pub object_symbol: char,
    pub object_color: u8,
    pub has_monster: bool,
    pub monster_display: MonsterDisplayMode,
    pub is_player_pos: bool,
    pub in_sight: bool,
    pub is_memorized: bool,
    pub is_dark: bool,
}

/// [v2.25.0 1-1] 뉴스심 출력
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewsymOutput {
    pub symbol: char,
    pub color: u8,
    pub attr: u8, // 0=일반, 1=밝게, 2=반전
}

/// [v2.25.0 1-1] 뉴스심 — 타일 표시 결정
/// 원본: display.c newsym() L500-700
pub fn newsym(input: &NewsymInput) -> NewsymOutput {
    // 우선순위: 플레이어 > 몬스터 > 아이템 > 함정 > 타일

    // [1] 플레이어 위치
    if input.is_player_pos {
        return NewsymOutput {
            symbol: '@',
            color: 15, // 흰색
            attr: 1,   // 밝게
        };
    }

    // [2] 몬스터
    if input.has_monster {
        match &input.monster_display {
            MonsterDisplayMode::Normal { symbol, color } => {
                return NewsymOutput {
                    symbol: *symbol,
                    color: *color,
                    attr: 0,
                };
            }
            MonsterDisplayMode::Warning { color, .. } => {
                return NewsymOutput {
                    symbol: '0',
                    color: *color,
                    attr: 1,
                };
            }
            MonsterDisplayMode::Disguised {
                fake_symbol,
                fake_color,
            } => {
                return NewsymOutput {
                    symbol: *fake_symbol,
                    color: *fake_color,
                    attr: 0,
                };
            }
            MonsterDisplayMode::Detected { symbol } => {
                return NewsymOutput {
                    symbol: *symbol,
                    color: 5, // 마젠타
                    attr: 1,
                };
            }
            MonsterDisplayMode::Statue { symbol } => {
                return NewsymOutput {
                    symbol: *symbol,
                    color: 8, // 회색
                    attr: 0,
                };
            }
            MonsterDisplayMode::Invisible { .. } => {
                return NewsymOutput {
                    symbol: 'I',
                    color: 4, // 파란
                    attr: 0,
                };
            }
            MonsterDisplayMode::Hidden => {} // 아래로 계속 진행
        }
    }

    // [3] 아이템
    if input.has_object && (input.in_sight || input.is_memorized) {
        return NewsymOutput {
            symbol: input.object_symbol,
            color: input.object_color,
            attr: 0,
        };
    }

    // [4] 함정
    if input.has_trap && (input.in_sight || input.is_memorized) {
        return NewsymOutput {
            symbol: input.trap_symbol,
            color: 5, // 마젠타
            attr: 0,
        };
    }

    // [5] 타일 (어둠이면 공백)
    if input.is_dark && !input.in_sight && !input.is_memorized {
        return NewsymOutput {
            symbol: ' ',
            color: 0,
            attr: 0,
        };
    }

    // 기본 타일 심볼
    let (sym, col) = tile_default_display(input.tile_type);
    NewsymOutput {
        symbol: sym,
        color: col,
        attr: 0,
    }
}

/// [v2.25.0 1-1] 기본 타일 심볼/색상 테이블
fn tile_default_display(tile_type: u8) -> (char, u8) {
    match tile_type {
        0 => (' ', 0),   // STONE
        1 => ('.', 7),   // ROOM
        2 => ('#', 3),   // CORR
        3 => ('-', 7),   // HWALL
        4 => ('|', 7),   // VWALL
        5 => ('+', 3),   // DOOR
        6 => ('<', 15),  // STAIRS_UP
        7 => ('>', 15),  // STAIRS_DOWN
        8 => ('{', 4),   // FOUNTAIN
        9 => ('_', 7),   // ALTAR
        10 => ('\\', 6), // THRONE
        11 => ('}', 4),  // POOL
        12 => ('}', 1),  // LAVA
        13 => ('#', 2),  // TREE
        14 => ('.', 6),  // ICE
        _ => ('?', 9),
    }
}

// =============================================================================
// [3] 화면 갱신 범위 — flush_screen (display.c L1500-1600)
// =============================================================================

/// [v2.25.0 1-1] 화면 갱신 범위 결정
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlushMode {
    /// 전체 화면 갱신
    Full,
    /// 변경된 타일만 갱신 (dirty 리스트)
    Partial { dirty_count: usize },
    /// 애니메이션 프레임 (빔/폭발)
    Animation { frame: u32 },
    /// 메시지 줄만 갱신
    MessageOnly,
}

/// [v2.25.0 1-1] 화면 갱신 범위 결정
/// 원본: display.c flush_screen()
pub fn determine_flush_mode(
    dirty_tiles: usize,
    total_visible: usize,
    has_animation: bool,
    animation_frame: u32,
    message_changed: bool,
) -> FlushMode {
    if has_animation {
        return FlushMode::Animation {
            frame: animation_frame,
        };
    }

    if dirty_tiles == 0 && message_changed {
        return FlushMode::MessageOnly;
    }

    // 변경 타일이 전체의 50% 초과면 전체 갱신이 효율적
    if total_visible > 0 && dirty_tiles * 2 > total_visible {
        return FlushMode::Full;
    }

    FlushMode::Partial {
        dirty_count: dirty_tiles,
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    fn base_monster_input() -> MonsterDisplayInput {
        MonsterDisplayInput {
            monster_symbol: 'D',
            monster_color: 1,
            in_sight: true,
            is_invisible: false,
            has_see_invisible: false,
            is_mimic: false,
            mimic_as_symbol: '.',
            mimic_as_color: 7,
            is_detected: false,
            is_stoned: false,
            warning_level: 0,
            is_hallucinating: false,
        }
    }

    // --- display_monster_result ---

    #[test]
    fn test_display_normal() {
        let mut rng = test_rng();
        let input = base_monster_input();
        let result = display_monster_result(&input, &mut rng);
        assert!(matches!(
            result,
            MonsterDisplayMode::Normal {
                symbol: 'D',
                color: 1
            }
        ));
    }

    #[test]
    fn test_display_stoned() {
        let mut rng = test_rng();
        let mut input = base_monster_input();
        input.is_stoned = true;
        let result = display_monster_result(&input, &mut rng);
        assert!(matches!(result, MonsterDisplayMode::Statue { .. }));
    }

    #[test]
    fn test_display_invisible_no_see() {
        let mut rng = test_rng();
        let mut input = base_monster_input();
        input.is_invisible = true;
        input.is_detected = true;
        let result = display_monster_result(&input, &mut rng);
        assert!(matches!(result, MonsterDisplayMode::Detected { .. }));
    }

    #[test]
    fn test_display_mimic() {
        let mut rng = test_rng();
        let mut input = base_monster_input();
        input.is_mimic = true;
        let result = display_monster_result(&input, &mut rng);
        assert!(matches!(result, MonsterDisplayMode::Disguised { .. }));
    }

    #[test]
    fn test_display_warning_out_of_sight() {
        let mut rng = test_rng();
        let mut input = base_monster_input();
        input.in_sight = false;
        input.warning_level = 3;
        let result = display_monster_result(&input, &mut rng);
        assert!(matches!(result, MonsterDisplayMode::Warning { .. }));
    }

    #[test]
    fn test_display_hidden() {
        let mut rng = test_rng();
        let mut input = base_monster_input();
        input.in_sight = false;
        let result = display_monster_result(&input, &mut rng);
        assert!(matches!(result, MonsterDisplayMode::Hidden));
    }

    #[test]
    fn test_display_hallucinating() {
        let mut rng = test_rng();
        let mut input = base_monster_input();
        input.is_hallucinating = true;
        let result = display_monster_result(&input, &mut rng);
        // 환각은 Normal이지만 심볼이 랜덤
        assert!(matches!(result, MonsterDisplayMode::Normal { .. }));
    }

    // --- newsym ---

    #[test]
    fn test_newsym_player() {
        let input = NewsymInput {
            tile_type: 1,
            has_trap: false,
            trap_symbol: '^',
            has_object: false,
            object_symbol: ')',
            object_color: 7,
            has_monster: false,
            monster_display: MonsterDisplayMode::Hidden,
            is_player_pos: true,
            in_sight: true,
            is_memorized: true,
            is_dark: false,
        };
        let output = newsym(&input);
        assert_eq!(output.symbol, '@');
    }

    #[test]
    fn test_newsym_monster() {
        let input = NewsymInput {
            tile_type: 1,
            has_trap: false,
            trap_symbol: '^',
            has_object: false,
            object_symbol: ')',
            object_color: 7,
            has_monster: true,
            monster_display: MonsterDisplayMode::Normal {
                symbol: 'D',
                color: 1,
            },
            is_player_pos: false,
            in_sight: true,
            is_memorized: true,
            is_dark: false,
        };
        let output = newsym(&input);
        assert_eq!(output.symbol, 'D');
    }

    #[test]
    fn test_newsym_dark() {
        let input = NewsymInput {
            tile_type: 1,
            has_trap: false,
            trap_symbol: '^',
            has_object: false,
            object_symbol: ')',
            object_color: 7,
            has_monster: false,
            monster_display: MonsterDisplayMode::Hidden,
            is_player_pos: false,
            in_sight: false,
            is_memorized: false,
            is_dark: true,
        };
        let output = newsym(&input);
        assert_eq!(output.symbol, ' ');
    }

    // --- flush_mode ---

    #[test]
    fn test_flush_animation() {
        let mode = determine_flush_mode(0, 100, true, 3, false);
        assert!(matches!(mode, FlushMode::Animation { frame: 3 }));
    }

    #[test]
    fn test_flush_message_only() {
        let mode = determine_flush_mode(0, 100, false, 0, true);
        assert!(matches!(mode, FlushMode::MessageOnly));
    }

    #[test]
    fn test_flush_full_when_many_dirty() {
        let mode = determine_flush_mode(60, 100, false, 0, false);
        assert!(matches!(mode, FlushMode::Full));
    }

    #[test]
    fn test_flush_partial() {
        let mode = determine_flush_mode(10, 100, false, 0, false);
        assert!(matches!(mode, FlushMode::Partial { dirty_count: 10 }));
    }
}

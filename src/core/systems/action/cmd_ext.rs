// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.22.0 R10-2] 확장 커맨드 시스템 (cmd_ext.rs)
//
// 원본 참조: NetHack 3.6.7 cmd.c L200-2500
//
// 구현 내용:
//   1. 확장 커맨드 메타데이터 (#enhance, #adjust, #chat, #ride 등 30+ 커맨드)
//   2. 숫자 접두사(Count) 파서 — "20s" = 20회 검색
//   3. 컨텍스트 기반 Y/N/Q 확인 프롬프트 시스템
//   4. 커맨드 카테고리 분류 및 도움말 생성
//   5. 방향 입력 파서 (do_direction 원본 매핑)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 확장 커맨드 메타데이터 (원본: cmd.c struct ext_func_tab)
// =============================================================================

/// [v2.22.0 R10-2] 확장 커맨드 메타데이터 (원본: ext_func_tab[])
#[derive(Debug, Clone)]
pub struct ExtCmdEntry {
    /// 커맨드 이름 (소문자, "#" 제외)
    pub name: &'static str,
    /// 커맨드 설명 (도움말 표시용)
    pub description: &'static str,
    /// 커맨드 카테고리
    pub category: ExtCmdCategory,
    /// 자동 완성 가능 여부
    pub autocomplete: bool,
    /// 위저드 모드 전용 여부
    pub wizard_only: bool,
}

/// [v2.22.0 R10-2] 확장 커맨드 카테고리 (원본: cmd.c 분류 기반)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtCmdCategory {
    /// 일반 행동 (이동/전투/탐색)
    Action,
    /// 인벤토리/장비 관리
    Inventory,
    /// 마법/기도/스킬
    Magic,
    /// 정보/상태 조회
    Info,
    /// 시스템 (저장/종료/설정)
    System,
    /// 디버그/위저드
    Debug,
}

/// [v2.22.0 R10-2] 전체 확장 커맨드 테이블 (원본: ext_func_tab[] 62항목 중 주요 40+ 이식)
pub fn ext_cmd_table() -> Vec<ExtCmdEntry> {
    vec![
        // === 행동 커맨드 ===
        ExtCmdEntry {
            name: "chat",
            description: "NPC와 대화",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "dip",
            description: "아이템을 액체에 담그기",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "force",
            description: "상자/문 강제로 열기",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "jump",
            description: "점프하기",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "kick",
            description: "발로 차기",
            category: ExtCmdCategory::Action,
            autocomplete: false,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "loot",
            description: "바닥 컨테이너 뒤지기",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "monster",
            description: "몬스터 특수 능력 사용",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "open",
            description: "문 열기",
            category: ExtCmdCategory::Action,
            autocomplete: false,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "ride",
            description: "탈것 타기/내리기",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "rub",
            description: "아이템 문지르기 (램프 등)",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "sit",
            description: "앉기",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "tip",
            description: "용기 뒤집기",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "turn",
            description: "언데드 퇴치",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "untrap",
            description: "함정 해제",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "wipe",
            description: "얼굴 닦기",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "travel",
            description: "자동 이동",
            category: ExtCmdCategory::Action,
            autocomplete: true,
            wizard_only: false,
        },
        // === 인벤토리/장비 ===
        ExtCmdEntry {
            name: "adjust",
            description: "인벤토리 글자 재배치",
            category: ExtCmdCategory::Inventory,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "name",
            description: "아이템/몬스터에 이름 붙이기",
            category: ExtCmdCategory::Inventory,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "call",
            description: "아이템 종류에 별명 붙이기",
            category: ExtCmdCategory::Inventory,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "twoweapon",
            description: "쌍수 전투 전환",
            category: ExtCmdCategory::Inventory,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "swap",
            description: "주무기/보조무기 교환",
            category: ExtCmdCategory::Inventory,
            autocomplete: false,
            wizard_only: false,
        },
        // === 마법/기도/스킬 ===
        ExtCmdEntry {
            name: "pray",
            description: "신에게 기도",
            category: ExtCmdCategory::Magic,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "offer",
            description: "제물 바치기",
            category: ExtCmdCategory::Magic,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "invoke",
            description: "아티팩트 능력 발동",
            category: ExtCmdCategory::Magic,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "enhance",
            description: "무기 스킬 강화",
            category: ExtCmdCategory::Magic,
            autocomplete: true,
            wizard_only: false,
        },
        // === 정보/조회 ===
        ExtCmdEntry {
            name: "conduct",
            description: "행동 규약 확인",
            category: ExtCmdCategory::Info,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "score",
            description: "현재 점수 확인",
            category: ExtCmdCategory::Info,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "overview",
            description: "던전 탐사 현황",
            category: ExtCmdCategory::Info,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "version",
            description: "버전 정보 표시",
            category: ExtCmdCategory::Info,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "turncount",
            description: "현재 턴 수 표시",
            category: ExtCmdCategory::Info,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "discoveries",
            description: "발견한 아이템 목록",
            category: ExtCmdCategory::Info,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "attributes",
            description: "캐릭터 속성 상세",
            category: ExtCmdCategory::Info,
            autocomplete: true,
            wizard_only: false,
        },
        // === 시스템 ===
        ExtCmdEntry {
            name: "save",
            description: "게임 저장",
            category: ExtCmdCategory::System,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "quit",
            description: "게임 종료",
            category: ExtCmdCategory::System,
            autocomplete: true,
            wizard_only: false,
        },
        ExtCmdEntry {
            name: "help",
            description: "도움말 표시",
            category: ExtCmdCategory::System,
            autocomplete: true,
            wizard_only: false,
        },
        // === 위저드 전용 ===
        ExtCmdEntry {
            name: "wish",
            description: "소원 빌기",
            category: ExtCmdCategory::Debug,
            autocomplete: true,
            wizard_only: true,
        },
        ExtCmdEntry {
            name: "identify",
            description: "아이템 식별",
            category: ExtCmdCategory::Debug,
            autocomplete: true,
            wizard_only: true,
        },
        ExtCmdEntry {
            name: "levelchange",
            description: "레벨 이동",
            category: ExtCmdCategory::Debug,
            autocomplete: true,
            wizard_only: true,
        },
        ExtCmdEntry {
            name: "genesis",
            description: "몬스터 소환",
            category: ExtCmdCategory::Debug,
            autocomplete: true,
            wizard_only: true,
        },
        ExtCmdEntry {
            name: "polyself",
            description: "변신",
            category: ExtCmdCategory::Debug,
            autocomplete: true,
            wizard_only: true,
        },
        ExtCmdEntry {
            name: "map",
            description: "전체 맵 공개",
            category: ExtCmdCategory::Debug,
            autocomplete: true,
            wizard_only: true,
        },
    ]
}

/// [v2.22.0 R10-2] 이름으로 확장 커맨드 조회
pub fn find_ext_cmd(name: &str) -> Option<ExtCmdEntry> {
    let lower = name.to_lowercase();
    ext_cmd_table().into_iter().find(|e| e.name == lower)
}

/// [v2.22.0 R10-2] 자동완성 후보 필터 (원본: cmd.c ext_cmd_complete)
pub fn autocomplete_ext_cmd(partial: &str) -> Vec<ExtCmdEntry> {
    let lower = partial.to_lowercase();
    ext_cmd_table()
        .into_iter()
        .filter(|e| e.autocomplete && e.name.starts_with(&lower))
        .collect()
}

/// [v2.22.0 R10-2] 카테고리별 분류 (도움말 출력용)
pub fn commands_by_category() -> Vec<(ExtCmdCategory, Vec<ExtCmdEntry>)> {
    let table = ext_cmd_table();
    let categories = [
        ExtCmdCategory::Action,
        ExtCmdCategory::Inventory,
        ExtCmdCategory::Magic,
        ExtCmdCategory::Info,
        ExtCmdCategory::System,
        ExtCmdCategory::Debug,
    ];

    categories
        .iter()
        .map(|cat| {
            let cmds: Vec<ExtCmdEntry> = table
                .iter()
                .filter(|e| e.category == *cat)
                .cloned()
                .collect();
            (*cat, cmds)
        })
        .filter(|(_, cmds)| !cmds.is_empty())
        .collect()
}

// =============================================================================
// [2] 숫자 접두사(Count) 파서 (원본: cmd.c parse_count, count_commands)
// =============================================================================

/// [v2.22.0 R10-2] 숫자 접두사 파싱 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountPrefix {
    /// 반복 횟수 (없으면 1)
    pub count: u32,
    /// 접두사 다음에 남은 커맨드 문자
    pub remaining: String,
}

/// [v2.22.0 R10-2] 입력 문자열에서 숫자 접두사 파싱 (원본: cmd.c)
/// 예: "20s" → CountPrefix { count: 20, remaining: "s" }
/// 예: "s" → CountPrefix { count: 1, remaining: "s" }
/// 예: "100." → CountPrefix { count: 100, remaining: "." }
pub fn parse_count_prefix(input: &str) -> CountPrefix {
    let mut digits = String::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    let remaining: String = chars.collect();

    if digits.is_empty() {
        CountPrefix {
            count: 1,
            remaining: input.to_string(),
        }
    } else {
        let count = digits.parse::<u32>().unwrap_or(1).max(1).min(9999);
        CountPrefix { count, remaining }
    }
}

// =============================================================================
// [3] Y/N/Q 확인 프롬프트 시스템 (원본: cmd.c yn_function)
// =============================================================================

/// [v2.22.0 R10-2] Y/N/Q 프롬프트 응답
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YnResponse {
    Yes,
    No,
    Quit,
    All,        // 'a' — 전체 적용
    None,       // 'n' — 건너뜀 (일부 컨텍스트에서)
    Count(u32), // 숫자 입력
}

/// [v2.22.0 R10-2] 프롬프트 입력 파싱 (원본: yn_function)
pub fn parse_yn_response(ch: char) -> YnResponse {
    match ch {
        'y' | 'Y' => YnResponse::Yes,
        'n' | 'N' => YnResponse::No,
        'q' | 'Q' | '\x1b' => YnResponse::Quit, // ESC = Quit
        'a' | 'A' => YnResponse::All,
        _ => YnResponse::No, // 기본값은 No
    }
}

/// [v2.22.0 R10-2] 프롬프트 응답이 긍정인지
pub fn is_affirmative(response: YnResponse) -> bool {
    matches!(response, YnResponse::Yes | YnResponse::All)
}

// =============================================================================
// [4] 방향 입력 파서 (원본: cmd.c getdir(), do_direction)
// =============================================================================

/// [v2.22.0 R10-2] 8방향 + 상/하/자기 (원본: cmd.c direction)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdDirection {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
    Up,    // '<'
    Down,  // '>'
    Self_, // '.' — 자기 자신 (발밑)
}

impl CmdDirection {
    /// 방향 → 좌표 델타 변환 (원본: xdir[], ydir[])
    pub fn to_delta(self) -> (i32, i32) {
        match self {
            CmdDirection::North => (0, -1),
            CmdDirection::South => (0, 1),
            CmdDirection::East => (1, 0),
            CmdDirection::West => (-1, 0),
            CmdDirection::NorthEast => (1, -1),
            CmdDirection::NorthWest => (-1, -1),
            CmdDirection::SouthEast => (1, 1),
            CmdDirection::SouthWest => (-1, 1),
            CmdDirection::Up | CmdDirection::Down | CmdDirection::Self_ => (0, 0),
        }
    }

    /// 역방향 (원본: cmd.c, 대칭 반사용)
    pub fn opposite(self) -> Self {
        match self {
            CmdDirection::North => CmdDirection::South,
            CmdDirection::South => CmdDirection::North,
            CmdDirection::East => CmdDirection::West,
            CmdDirection::West => CmdDirection::East,
            CmdDirection::NorthEast => CmdDirection::SouthWest,
            CmdDirection::NorthWest => CmdDirection::SouthEast,
            CmdDirection::SouthEast => CmdDirection::NorthWest,
            CmdDirection::SouthWest => CmdDirection::NorthEast,
            CmdDirection::Up => CmdDirection::Down,
            CmdDirection::Down => CmdDirection::Up,
            CmdDirection::Self_ => CmdDirection::Self_,
        }
    }

    /// 방향이 대각선인지
    pub fn is_diagonal(self) -> bool {
        matches!(
            self,
            CmdDirection::NorthEast
                | CmdDirection::NorthWest
                | CmdDirection::SouthEast
                | CmdDirection::SouthWest
        )
    }
}

/// [v2.22.0 R10-2] 키 문자 → 방향 변환 (원본: getdir(), cmd.c)
pub fn parse_direction(ch: char) -> Option<CmdDirection> {
    match ch {
        'k' | '8' => Some(CmdDirection::North),
        'j' | '2' => Some(CmdDirection::South),
        'l' | '6' => Some(CmdDirection::East),
        'h' | '4' => Some(CmdDirection::West),
        'u' | '9' => Some(CmdDirection::NorthEast),
        'y' | '7' => Some(CmdDirection::NorthWest),
        'n' | '3' => Some(CmdDirection::SouthEast),
        'b' | '1' => Some(CmdDirection::SouthWest),
        '<' => Some(CmdDirection::Up),
        '>' => Some(CmdDirection::Down),
        '.' | '5' => Some(CmdDirection::Self_),
        _ => None,
    }
}

// =============================================================================
// [5] 커맨드 사전조건 검사 (원본: cmd.c can_do_*)
// =============================================================================

/// [v2.22.0 R10-2] 커맨드 실행 사전조건 컨텍스트
#[derive(Debug, Clone)]
pub struct CmdPrecondition {
    pub is_paralyzed: bool,
    pub is_stunned: bool,
    pub is_confused: bool,
    pub is_blind: bool,
    pub is_levitating: bool,
    pub is_underwater: bool,
    pub is_swallowed: bool,
    pub is_riding: bool,
    pub is_polymorphed: bool,
    pub has_hands: bool,
    pub can_move: bool,
    pub hp_current: i32,
    pub hp_max: i32,
}

/// [v2.22.0 R10-2] 사전조건 검사 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreconditionResult {
    /// 실행 가능
    Ok,
    /// 실행 불가 + 사유 메시지
    Blocked(String),
}

/// [v2.22.0 R10-2] 범용 행동 사전조건 검사 (원본: check_can_act)
pub fn check_can_act(ctx: &CmdPrecondition) -> PreconditionResult {
    if ctx.is_paralyzed || !ctx.can_move {
        return PreconditionResult::Blocked("몸을 움직일 수 없습니다!".to_string());
    }
    PreconditionResult::Ok
}

/// [v2.22.0 R10-2] 점프 사전조건 (원본: cmd.c can_jump)
pub fn check_can_jump(ctx: &CmdPrecondition) -> PreconditionResult {
    if let PreconditionResult::Blocked(msg) = check_can_act(ctx) {
        return PreconditionResult::Blocked(msg);
    }
    if ctx.is_underwater {
        return PreconditionResult::Blocked("물속에서는 점프할 수 없습니다.".to_string());
    }
    if ctx.is_swallowed {
        return PreconditionResult::Blocked("삼켜진 상태에서는 점프할 수 없습니다.".to_string());
    }
    PreconditionResult::Ok
}

/// [v2.22.0 R10-2] 기마 사전조건 (원본: cmd.c can_ride)
pub fn check_can_ride(ctx: &CmdPrecondition) -> PreconditionResult {
    if let PreconditionResult::Blocked(msg) = check_can_act(ctx) {
        return PreconditionResult::Blocked(msg);
    }
    if ctx.is_levitating {
        return PreconditionResult::Blocked("공중에 떠 있어 탈 수 없습니다.".to_string());
    }
    if ctx.is_underwater {
        return PreconditionResult::Blocked("물속에서는 탈 수 없습니다.".to_string());
    }
    PreconditionResult::Ok
}

/// [v2.22.0 R10-2] 제물 바치기 사전조건 (원본: cmd.c can_offer)
pub fn check_can_offer(ctx: &CmdPrecondition) -> PreconditionResult {
    if let PreconditionResult::Blocked(msg) = check_can_act(ctx) {
        return PreconditionResult::Blocked(msg);
    }
    // 제물은 제단 위에서만 가능하지만 여기서는 기본 행동 검사만
    PreconditionResult::Ok
}

/// [v2.22.0 R10-2] 기도 사전조건 (원본: cmd.c can_pray)
pub fn check_can_pray(ctx: &CmdPrecondition) -> PreconditionResult {
    if let PreconditionResult::Blocked(msg) = check_can_act(ctx) {
        return PreconditionResult::Blocked(msg);
    }
    PreconditionResult::Ok
}

/// [v2.22.0 R10-2] 언데드 퇴치 사전조건 (원본: cmd.c can_turn)
pub fn check_can_turn_undead(ctx: &CmdPrecondition) -> PreconditionResult {
    if let PreconditionResult::Blocked(msg) = check_can_act(ctx) {
        return PreconditionResult::Blocked(msg);
    }
    if !ctx.has_hands {
        return PreconditionResult::Blocked("손이 없어 퇴치할 수 없습니다.".to_string());
    }
    PreconditionResult::Ok
}

/// [v2.22.0 R10-2] 함정 해제 사전조건 (원본: cmd.c can_untrap)
pub fn check_can_untrap(ctx: &CmdPrecondition) -> PreconditionResult {
    if let PreconditionResult::Blocked(msg) = check_can_act(ctx) {
        return PreconditionResult::Blocked(msg);
    }
    if !ctx.has_hands {
        return PreconditionResult::Blocked("손이 없어 함정을 해제할 수 없습니다.".to_string());
    }
    PreconditionResult::Ok
}

// =============================================================================
// [6] 스킬 강화 시스템 (원본: cmd.c enhance_weapon_skill)
// =============================================================================

/// [v2.22.0 R10-2] 무기 스킬 등급 (원본: objclass.h P_* 상수)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillLevel {
    Unskilled = 0,
    Basic = 1,
    Skilled = 2,
    Expert = 3,
    Master = 4,      // 무술 계열 전용
    GrandMaster = 5, // 무술 계열 전용
}

impl SkillLevel {
    /// 다음 등급
    pub fn next(self) -> Option<SkillLevel> {
        match self {
            SkillLevel::Unskilled => Some(SkillLevel::Basic),
            SkillLevel::Basic => Some(SkillLevel::Skilled),
            SkillLevel::Skilled => Some(SkillLevel::Expert),
            SkillLevel::Expert => Some(SkillLevel::Master),
            SkillLevel::Master => Some(SkillLevel::GrandMaster),
            SkillLevel::GrandMaster => None,
        }
    }

    /// 표시 이름
    pub fn display_name(self) -> &'static str {
        match self {
            SkillLevel::Unskilled => "Unskilled",
            SkillLevel::Basic => "Basic",
            SkillLevel::Skilled => "Skilled",
            SkillLevel::Expert => "Expert",
            SkillLevel::Master => "Master",
            SkillLevel::GrandMaster => "Grand Master",
        }
    }
}

/// [v2.22.0 R10-2] 스킬 강화 판정 결과 (원본: enhance_weapon_skill)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnhanceResult {
    /// 강화 성공
    Enhanced {
        skill_name: String,
        from: SkillLevel,
        to: SkillLevel,
    },
    /// 이미 최대 등급
    AlreadyMax { skill_name: String },
    /// 연습 부족
    NotEnoughPractice {
        skill_name: String,
        current: SkillLevel,
        needed: i32,
    },
    /// 강화 가능한 스킬 없음
    NoSkillsAvailable,
}

/// [v2.22.0 R10-2] 스킬 강화 가능 여부 판정 (원본: enhance_weapon_skill)
pub fn check_enhance(
    current_level: SkillLevel,
    max_allowed: SkillLevel,
    practice_points: i32,
    skill_name: &str,
) -> EnhanceResult {
    if current_level >= max_allowed {
        return EnhanceResult::AlreadyMax {
            skill_name: skill_name.to_string(),
        };
    }

    // 강화에 필요한 연습량 (원본: skill_advance() 기반)
    let needed = match current_level {
        SkillLevel::Unskilled => 20,
        SkillLevel::Basic => 80,
        SkillLevel::Skilled => 180,
        SkillLevel::Expert => 320,
        SkillLevel::Master => 500,
        SkillLevel::GrandMaster => i32::MAX,
    };

    if practice_points < needed {
        return EnhanceResult::NotEnoughPractice {
            skill_name: skill_name.to_string(),
            current: current_level,
            needed,
        };
    }

    if let Some(next) = current_level.next() {
        EnhanceResult::Enhanced {
            skill_name: skill_name.to_string(),
            from: current_level,
            to: next,
        }
    } else {
        EnhanceResult::AlreadyMax {
            skill_name: skill_name.to_string(),
        }
    }
}

// =============================================================================
// [7] 인벤토리 글자 재배치 (원본: cmd.c doorganize/adjust)
// =============================================================================

/// [v2.22.0 R10-2] 인벤토리 글자 교환 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjustResult {
    /// 교환 성공
    Swapped { from: char, to: char },
    /// 지정한 슬롯에 아이템 없음
    NoItemAtSlot(char),
    /// 같은 글자 지정
    SameSlot,
    /// 유효하지 않은 글자
    InvalidLetter(char),
}

/// [v2.22.0 R10-2] 인벤토리 글자 교환 유효성 검사 (원본: doorganize)
pub fn validate_adjust(from: char, to: char, occupied_slots: &[char]) -> AdjustResult {
    if from == to {
        return AdjustResult::SameSlot;
    }

    // 알파벳 유효성 검사
    let valid = |c: char| c.is_ascii_alphabetic();
    if !valid(from) {
        return AdjustResult::InvalidLetter(from);
    }
    if !valid(to) {
        return AdjustResult::InvalidLetter(to);
    }

    // from 슬롯에 아이템이 있어야 함
    if !occupied_slots.contains(&from) {
        return AdjustResult::NoItemAtSlot(from);
    }

    AdjustResult::Swapped { from, to }
}

// =============================================================================
// [8] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext_cmd_table_count() {
        let table = ext_cmd_table();
        // 최소 30개 이상의 확장 커맨드
        assert!(table.len() >= 30);
    }

    #[test]
    fn test_find_ext_cmd() {
        let cmd = find_ext_cmd("pray");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().category, ExtCmdCategory::Magic);

        let none = find_ext_cmd("nonexistent");
        assert!(none.is_none());
    }

    #[test]
    fn test_autocomplete() {
        let matches = autocomplete_ext_cmd("pr");
        // "pray" 매칭
        assert!(matches.iter().any(|e| e.name == "pray"));
    }

    #[test]
    fn test_commands_by_category() {
        let cats = commands_by_category();
        // 최소 5개 카테고리
        assert!(cats.len() >= 5);
        // Action 카테고리에 10개 이상
        let action = cats.iter().find(|(c, _)| *c == ExtCmdCategory::Action);
        assert!(action.is_some());
        assert!(action.unwrap().1.len() >= 10);
    }

    #[test]
    fn test_parse_count_prefix_with_number() {
        let r = parse_count_prefix("20s");
        assert_eq!(r.count, 20);
        assert_eq!(r.remaining, "s");
    }

    #[test]
    fn test_parse_count_prefix_no_number() {
        let r = parse_count_prefix("s");
        assert_eq!(r.count, 1);
        assert_eq!(r.remaining, "s");
    }

    #[test]
    fn test_parse_count_prefix_large() {
        let r = parse_count_prefix("99999x");
        assert_eq!(r.count, 9999); // 최대 9999 제한
    }

    #[test]
    fn test_yn_response() {
        assert_eq!(parse_yn_response('y'), YnResponse::Yes);
        assert_eq!(parse_yn_response('N'), YnResponse::No);
        assert_eq!(parse_yn_response('q'), YnResponse::Quit);
        assert_eq!(parse_yn_response('a'), YnResponse::All);
        assert_eq!(parse_yn_response('\x1b'), YnResponse::Quit);
    }

    #[test]
    fn test_parse_direction() {
        assert_eq!(parse_direction('k'), Some(CmdDirection::North));
        assert_eq!(parse_direction('l'), Some(CmdDirection::East));
        assert_eq!(parse_direction('.'), Some(CmdDirection::Self_));
        assert_eq!(parse_direction('<'), Some(CmdDirection::Up));
        assert_eq!(parse_direction('x'), None);
    }

    #[test]
    fn test_direction_delta() {
        assert_eq!(CmdDirection::North.to_delta(), (0, -1));
        assert_eq!(CmdDirection::SouthEast.to_delta(), (1, 1));
        assert_eq!(CmdDirection::Self_.to_delta(), (0, 0));
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(CmdDirection::North.opposite(), CmdDirection::South);
        assert_eq!(CmdDirection::NorthEast.opposite(), CmdDirection::SouthWest);
    }

    #[test]
    fn test_direction_is_diagonal() {
        assert!(CmdDirection::NorthEast.is_diagonal());
        assert!(!CmdDirection::North.is_diagonal());
    }

    #[test]
    fn test_precondition_paralyzed() {
        let ctx = CmdPrecondition {
            is_paralyzed: true,
            is_stunned: false,
            is_confused: false,
            is_blind: false,
            is_levitating: false,
            is_underwater: false,
            is_swallowed: false,
            is_riding: false,
            is_polymorphed: false,
            has_hands: true,
            can_move: true,
            hp_current: 10,
            hp_max: 20,
        };
        assert!(matches!(
            check_can_act(&ctx),
            PreconditionResult::Blocked(_)
        ));
    }

    #[test]
    fn test_precondition_jump_underwater() {
        let ctx = CmdPrecondition {
            is_paralyzed: false,
            is_stunned: false,
            is_confused: false,
            is_blind: false,
            is_levitating: false,
            is_underwater: true,
            is_swallowed: false,
            is_riding: false,
            is_polymorphed: false,
            has_hands: true,
            can_move: true,
            hp_current: 10,
            hp_max: 20,
        };
        assert!(matches!(
            check_can_jump(&ctx),
            PreconditionResult::Blocked(_)
        ));
    }

    #[test]
    fn test_enhance_success() {
        let r = check_enhance(SkillLevel::Basic, SkillLevel::Expert, 100, "long sword");
        assert!(matches!(r, EnhanceResult::Enhanced { .. }));
        if let EnhanceResult::Enhanced { from, to, .. } = r {
            assert_eq!(from, SkillLevel::Basic);
            assert_eq!(to, SkillLevel::Skilled);
        }
    }

    #[test]
    fn test_enhance_already_max() {
        let r = check_enhance(SkillLevel::Expert, SkillLevel::Expert, 999, "dagger");
        assert!(matches!(r, EnhanceResult::AlreadyMax { .. }));
    }

    #[test]
    fn test_enhance_not_enough() {
        let r = check_enhance(SkillLevel::Basic, SkillLevel::Expert, 10, "axe");
        assert!(matches!(r, EnhanceResult::NotEnoughPractice { .. }));
    }

    #[test]
    fn test_adjust_valid() {
        let r = validate_adjust('a', 'z', &['a', 'b', 'c']);
        assert_eq!(r, AdjustResult::Swapped { from: 'a', to: 'z' });
    }

    #[test]
    fn test_adjust_same_slot() {
        let r = validate_adjust('a', 'a', &['a']);
        assert_eq!(r, AdjustResult::SameSlot);
    }

    #[test]
    fn test_adjust_no_item() {
        let r = validate_adjust('z', 'a', &['a', 'b']);
        assert_eq!(r, AdjustResult::NoItemAtSlot('z'));
    }

    #[test]
    fn test_skill_level_display() {
        assert_eq!(SkillLevel::Expert.display_name(), "Expert");
        assert_eq!(SkillLevel::GrandMaster.display_name(), "Grand Master");
    }

    #[test]
    fn test_wizard_only_filter() {
        let table = ext_cmd_table();
        let wizard_cmds: Vec<_> = table.iter().filter(|e| e.wizard_only).collect();
        assert!(wizard_cmds.len() >= 4);
        assert!(wizard_cmds.iter().any(|e| e.name == "wish"));
    }
}

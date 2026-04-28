use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// =============================================================================
// [v2.3.1] options.c 옵션 이식
// 원본: nethack-3.6.7/src/options.c (6,473줄)
//
// NetHack의 방대한 옵션 시스템 포팅
//
// =============================================================================

/// [v2.3.1] 게임 옵션 (원본: opts[], boolopt[], compopt[])
/// options.c의 옵션 구조 — NetHack의 전체 옵션 필드
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    // === 기본 옵션 (원본: boolopt) ===
    /// 반려견 이름
    pub dogname: String,
    /// 반려묘 이름
    pub catname: String,
    ///
    pub fruit: String,
    /// 자동 줍기 활성
    pub autopickup: bool,
    ///
    pub pickup_types: String,
    /// 경험치 표시 활성
    pub show_exp: bool,
    /// 점수 표시 활성
    pub show_score: bool,
    /// 색상 사용 활성
    pub color: bool,
    ///
    pub hilite_pet: bool,
    /// 현재 심볼 세트 이름
    pub current_symbol_set: String,

    // === [v2.3.1] 게임 플레이 옵션 ===
    /// 자동 열기 (문 자동 열기)
    pub autoopen: bool,
    /// 안전 이동 확인 (dangerous move warning)
    pub safe_move: bool,
    /// 묻기 확인 (bury/confirm)
    pub confirm: bool,
    /// 조용한 모드 (메시지 최소화)
    pub quiet: bool,
    ///
    pub number_pad: bool,
    /// 뉴스 표시 (게임 시작 시)
    pub news: bool,
    ///
    pub rest_on_space: bool,
    /// 자동 줍기 무게 제한 (0이면 제한 없음)
    pub pickup_burden: i32,
    /// 달리기 중 메시지 표시
    pub run_message: bool,
    /// 밝은 복도 표시 색상
    pub lit_corridor: bool,
    ///
    pub hilite_monster: bool,
    /// 사망 시 무덤 표시
    pub tombstone: bool,
    /// 자동 열기 용기 (chest/box)
    pub lootabc: bool,
    ///
    pub travel: bool,
    /// 층간 이동 시 확인
    pub confirm_stairway: bool,
    ///
    pub showweight: bool,
    ///
    pub name: String,

    // === [v2.3.1] UI/표시 옵션 ===
    /// 메시지 윈도우 크기 (줄 수)
    pub msg_window: i32,
    /// 최대 메시지 수
    pub msghistory: i32,
    /// 상태 표시줄 모드 (0=기본, 1=확장, 2=커스텀)
    pub status_style: i32,
    /// 자동 스크롤 (메시지 창)
    pub autoscroll: bool,
    ///
    pub sortloot: bool,
    ///
    pub inventory_style: String,
    /// 맵 표시 크기 (1=작음, 2=보통, 3=큼)
    pub map_size: i32,
    /// 미니맵 활성화
    pub minimap: bool,
    /// 마우스 지원
    pub mouse_support: bool,

    //
    ///
    pub hilite_status: bool,
    /// HP 경고 기준값 (퍼센트)
    pub hpwarn_threshold: i32,
    /// 낮은 HP 색상 ([R, G, B])
    pub low_hp_color: [u8; 3],
    /// 위험 HP 색상
    pub critical_hp_color: [u8; 3],

    // === [v2.3.1] 고급 옵션 ===
    /// 디버그 모드 (위저드 모드)
    pub wizard: bool,
    /// 시드 고정
    pub fixed_seed: Option<u64>,
    /// 맵 디버그 표시
    pub debug_map: bool,
    /// FPS 표시
    pub show_fps: bool,
    ///
    pub performance_mode: bool,
}

impl Options {
    pub fn default() -> Self {
        Self {
            dogname: "Hachi".to_string(),
            catname: "Milo".to_string(),
            fruit: "slime mold".to_string(),
            autopickup: true,
            pickup_types: "$*?/".to_string(),
            show_exp: true,
            show_score: true,
            color: true,
            hilite_pet: true,
            current_symbol_set: "Original".to_string(),
            // [v2.3.1] 게임 플레이 옵션 기본값
            autoopen: true,
            safe_move: true,
            confirm: true,
            quiet: false,
            number_pad: false,
            news: true,
            rest_on_space: false,
            pickup_burden: 0,
            run_message: true,
            lit_corridor: true,
            hilite_monster: true,
            tombstone: true,
            lootabc: false,
            travel: true,
            confirm_stairway: false,
            showweight: true,
            name: String::new(),
            // [v2.3.1] UI 옵션 기본값
            msg_window: 5,
            msghistory: 50,
            status_style: 0,
            autoscroll: true,
            sortloot: true,
            inventory_style: "traditional".to_string(),
            map_size: 2,
            minimap: false,
            mouse_support: true,
            // [v2.3.1
            hilite_status: true,
            hpwarn_threshold: 25,
            low_hp_color: [255, 200, 0],
            critical_hp_color: [255, 50, 50],
            // [v2.3.1] 고급 옵션
            wizard: false,
            fixed_seed: None,
            debug_map: false,
            show_fps: false,
            performance_mode: false,
        }
    }

    pub fn load() -> Self {
        let config_path = "options.toml";
        if Path::new(config_path).exists() {
            if let Ok(content) = fs::read_to_string(config_path) {
                if let Ok(opts) = toml::from_str::<Options>(&content) {
                    return opts;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let config_path = "options.toml";
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = fs::write(config_path, content);
        }
    }

    /// [v2.3.1] 옵션 값 설정 (원본: doset)
    pub fn set_option(&mut self, key: &str, value: &str) -> bool {
        match key {
            "name" => {
                self.name = value.to_string();
                true
            }
            "dogname" => {
                self.dogname = value.to_string();
                true
            }
            "catname" => {
                self.catname = value.to_string();
                true
            }
            "fruit" => {
                self.fruit = value.to_string();
                true
            }
            "pickup_types" => {
                self.pickup_types = value.to_string();
                true
            }
            "autopickup" => {
                self.autopickup = parse_bool(value);
                true
            }
            "autoopen" => {
                self.autoopen = parse_bool(value);
                true
            }
            "safe_move" => {
                self.safe_move = parse_bool(value);
                true
            }
            "confirm" => {
                self.confirm = parse_bool(value);
                true
            }
            "quiet" => {
                self.quiet = parse_bool(value);
                true
            }
            "number_pad" => {
                self.number_pad = parse_bool(value);
                true
            }
            "color" => {
                self.color = parse_bool(value);
                true
            }
            "hilite_pet" => {
                self.hilite_pet = parse_bool(value);
                true
            }
            "show_exp" => {
                self.show_exp = parse_bool(value);
                true
            }
            "show_score" => {
                self.show_score = parse_bool(value);
                true
            }
            "hilite_status" => {
                self.hilite_status = parse_bool(value);
                true
            }
            "tombstone" => {
                self.tombstone = parse_bool(value);
                true
            }
            "travel" => {
                self.travel = parse_bool(value);
                true
            }
            "showweight" => {
                self.showweight = parse_bool(value);
                true
            }
            "minimap" => {
                self.minimap = parse_bool(value);
                true
            }
            "msg_window" => {
                if let Ok(v) = value.parse() {
                    self.msg_window = v;
                    true
                } else {
                    false
                }
            }
            "msghistory" => {
                if let Ok(v) = value.parse() {
                    self.msghistory = v;
                    true
                } else {
                    false
                }
            }
            "map_size" => {
                if let Ok(v) = value.parse() {
                    self.map_size = v;
                    true
                } else {
                    false
                }
            }
            "pickup_burden" => {
                if let Ok(v) = value.parse() {
                    self.pickup_burden = v;
                    true
                } else {
                    false
                }
            }
            "hpwarn_threshold" => {
                if let Ok(v) = value.parse() {
                    self.hpwarn_threshold = v;
                    true
                } else {
                    false
                }
            }
            "wizard" => {
                self.wizard = parse_bool(value);
                true
            }
            "debug_map" => {
                self.debug_map = parse_bool(value);
                true
            }
            "show_fps" => {
                self.show_fps = parse_bool(value);
                true
            }
            _ => false,
        }
    }

    /// [v2.3.1] 옵션 값 조회 (원본: doset)
    pub fn get_option(&self, key: &str) -> Option<String> {
        match key {
            "name" => Some(self.name.clone()),
            "dogname" => Some(self.dogname.clone()),
            "catname" => Some(self.catname.clone()),
            "fruit" => Some(self.fruit.clone()),
            "autopickup" => Some(self.autopickup.to_string()),
            "autoopen" => Some(self.autoopen.to_string()),
            "safe_move" => Some(self.safe_move.to_string()),
            "color" => Some(self.color.to_string()),
            "hilite_pet" => Some(self.hilite_pet.to_string()),
            "show_exp" => Some(self.show_exp.to_string()),
            "show_score" => Some(self.show_score.to_string()),
            "msg_window" => Some(self.msg_window.to_string()),
            "msghistory" => Some(self.msghistory.to_string()),
            "map_size" => Some(self.map_size.to_string()),
            "wizard" => Some(self.wizard.to_string()),
            _ => None,
        }
    }
}

///
fn parse_bool(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "true" | "1" | "yes" | "on" | "y")
}

/// [v2.3.1] 옵션 정보 테이블 (원본: optlist[])
pub fn option_help_table() -> Vec<OptionHelp> {
    vec![
        OptionHelp {
            name: "name",
            description: "Player name",
            option_type: "string",
            default: "",
        },
        OptionHelp {
            name: "dogname",
            description: "Pet dog name",
            option_type: "string",
            default: "Hachi",
        },
        OptionHelp {
            name: "catname",
            description: "Pet cat name",
            option_type: "string",
            default: "Milo",
        },
        OptionHelp {
            name: "fruit",
            description: "Preferred fruit",
            option_type: "string",
            default: "slime mold",
        },
        OptionHelp {
            name: "autopickup",
            description: "Auto-pickup",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "pickup_types",
            description: "Auto-pickup item types",
            option_type: "string",
            default: "$*?/",
        },
        OptionHelp {
            name: "autoopen",
            description: "Auto-open doors",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "safe_move",
            description: "Dangerous move warning",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "confirm",
            description: "Action confirm",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "color",
            description: "Color display",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "hilite_pet",
            description: "Pet highlight",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "hilite_status",
            description: "Status highlight",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "show_exp",
            description: "Show experience",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "show_score",
            description: "Show score",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "number_pad",
            description: "Numpad movement",
            option_type: "bool",
            default: "false",
        },
        OptionHelp {
            name: "tombstone",
            description: "Show tombstone",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "travel",
            description: "Auto-travel",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "showweight",
            description: "Show weight",
            option_type: "bool",
            default: "true",
        },
        OptionHelp {
            name: "msg_window",
            description: "Message window size (lines)",
            option_type: "int",
            default: "5",
        },
        OptionHelp {
            name: "msghistory",
            description: "Max message history",
            option_type: "int",
            default: "50",
        },
        OptionHelp {
            name: "map_size",
            description: "Map display size (1-3)",
            option_type: "int",
            default: "2",
        },
        OptionHelp {
            name: "minimap",
            description: "Minimap",
            option_type: "bool",
            default: "false",
        },
        OptionHelp {
            name: "hpwarn_threshold",
            description: "HP warning threshold (%)",
            option_type: "int",
            default: "25",
        },
        OptionHelp {
            name: "wizard",
            description: "Wizard (debug) mode",
            option_type: "bool",
            default: "false",
        },
    ]
}

/// [v2.3.1] 옵션 정보 구조체
#[derive(Debug, Clone)]
pub struct OptionHelp {
    pub name: &'static str,
    pub description: &'static str,
    pub option_type: &'static str,
    pub default: &'static str,
}

/// [v2.3.1] 옵션 검증 (원본: validate_option)
pub fn validate_option(key: &str, value: &str) -> Result<(), String> {
    match key {
        "msg_window" => {
            let v: i32 = value
                .parse()
                .map_err(|_| "Integer value required".to_string())?;
            if v < 1 || v > 20 {
                return Err("Message window size must be 1-20".to_string());
            }
            Ok(())
        }
        "msghistory" => {
            let v: i32 = value
                .parse()
                .map_err(|_| "Integer value required".to_string())?;
            if v < 10 || v > 500 {
                return Err("Message history must be 10-500".to_string());
            }
            Ok(())
        }
        "map_size" => {
            let v: i32 = value
                .parse()
                .map_err(|_| "Integer value required".to_string())?;
            if v < 1 || v > 3 {
                return Err("Map size must be 1-3".to_string());
            }
            Ok(())
        }
        "hpwarn_threshold" => {
            let v: i32 = value
                .parse()
                .map_err(|_| "Integer value required".to_string())?;
            if v < 0 || v > 100 {
                return Err("HP warning threshold must be 0-100".to_string());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

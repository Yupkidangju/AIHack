// [v2.0.0 Phase R4] 몬스터 AI 시스템 (monmove.c + dog.c + muse.c + mcastu.c + wizard.c)
//

pub mod ai_helper;
pub mod core;
// [v2.0.0 R3] ai_part1~3: 이전 분리 시도의 잔존 파일 (미사용)
//
// pub mod ai_part1;
// pub mod ai_part2;
// pub mod ai_part3;
pub mod dog;
pub mod mcastu;
pub mod monmove;
pub mod muse;
pub mod wizard;

// [v2.0.0 R3] 기존 경로 호환: ai::XXX 접근 유지
pub use self::core::*;

// ──────────────────────────────────────────────────────────────
// [v2.0.0
//
// 설계 근거:
//
// - 몬스터 유형별(일반, 펫, 마법사, 보스)로 다른 AI 전략을 trait impl로 구현
//
//
// ──────────────────────────────────────────────────────────────

use crate::core::entity::Position;
use serde::{Deserialize, Serialize};

///
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// 현재 턴 번호
    pub turn: u64,
    ///
    pub actor_pos: Position,
    ///
    pub actor_hp: (i32, i32),
    ///
    pub player_pos: Position,
    ///
    pub distance_sq: i32,
    ///
    pub can_see_player: bool,
    /// 인접한 적 수 (전투 판단용)
    pub adjacent_enemies: u8,
    ///
    pub adjacent_allies: u8,
    /// 현재 도주 중인지
    pub is_fleeing: bool,
    ///
    pub is_hostile: bool,
    /// 펫인지
    pub is_pet: bool,
}

/// AI가 선택할 수 있는 행동
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiAction {
    ///
    MoveTowards,
    ///
    Flee,
    /// 랜덤 이동 (배회)
    Wander,
    ///
    AttackPlayer,
    /// 아이템 줍기
    PickupItem,
    /// 마법 시전
    CastSpell,
    /// 대기 (행동 스킵)
    Wait,
    /// 소환 (wizard AI 전용)
    Summon,
    /// 대화 시도 (Phase R4.5 LLM 대상)
    Speak(String),
}

///
///
///
///
///
///
///
/// - `decide()` → 매 턴 × N마리 호출, **0ms 필수** (규칙 기반 전용)
///
///
/// # 사용 예시 (향후 구현)
/// ```ignore
/// struct BasicMonsterAi;
/// impl Behavior for BasicMonsterAi {
///     fn decide(&self, obs: &Observation) -> AiAction {
///         if obs.distance_sq <= 2 && obs.is_hostile {
///             AiAction::AttackPlayer
///         } else if obs.can_see_player && obs.distance_sq < 100 {
///             AiAction::MoveTowards
///         } else {
///             AiAction::Wander
///         }
///     }
/// }
/// ```
pub trait Behavior: Send + Sync {
    /// 행동 결정 (매 턴 호출 — 반드시 0ms 이내 반환)
    fn decide(&self, obs: &Observation) -> AiAction;

    ///
    fn priority(&self) -> i32 {
        0
    }

    /// 이 행동 패턴의 이름 (디버깅/로깅용)
    fn name(&self) -> &str {
        "basic"
    }
}

///
///
///
///
pub trait Conversable: Send + Sync {
    ///
    fn respond(&self, context: &str) -> String;

    /// 대화 가능 여부 (지능, 언어 능력 등 체크)
    fn can_converse(&self) -> bool {
        true
    }
}

// ──────────────────────────────────────────────────────────────
// 기본 구현: 규칙 기반 AI (현재 ai/core.rs 로직의 래핑)
// ──────────────────────────────────────────────────────────────

/// 기본 규칙 기반 AI (NetHack monmove.c 이식)
pub struct RuleBasedAi;

impl Behavior for RuleBasedAi {
    fn decide(&self, obs: &Observation) -> AiAction {
        // 도주 중이면 도주 계속
        if obs.is_fleeing {
            return AiAction::Flee;
        }

        //
        if obs.distance_sq <= 2 && obs.is_hostile {
            return AiAction::AttackPlayer;
        }

        //
        if obs.can_see_player && obs.distance_sq < 100 && obs.is_hostile {
            return AiAction::MoveTowards;
        }

        // 그 외 배회
        AiAction::Wander
    }

    fn name(&self) -> &str {
        "rule_based"
    }
}

/// 펫 AI (NetHack dogmove.c 이식)
pub struct PetAi;

impl Behavior for PetAi {
    fn decide(&self, obs: &Observation) -> AiAction {
        //
        if obs.distance_sq > 64 {
            return AiAction::MoveTowards;
        }

        // 인접한 적이 있으면 공격
        if obs.adjacent_enemies > 0 && obs.distance_sq <= 4 {
            return AiAction::AttackPlayer; // 향후 AttackEnemy로 세분화
        }

        // 가까이 있으면 배회
        AiAction::Wander
    }

    fn priority(&self) -> i32 {
        10 // 펫은 일반 AI보다 높은 우선순위
    }

    fn name(&self) -> &str {
        "pet"
    }
}

/// 고정 대사 기반 대화 (Phase R4.5 준비)
pub struct ScriptedDialogue {
    /// 고정 대사 목록
    pub lines: Vec<String>,
}

impl Conversable for ScriptedDialogue {
    fn respond(&self, _context: &str) -> String {
        if self.lines.is_empty() {
            "...".to_string()
        } else {
            //
            self.lines[0].clone()
        }
    }
}

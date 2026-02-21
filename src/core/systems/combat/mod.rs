// [v2.0.0 Phase R4] 전투 시스템 (uhitm.c + mhitu.c + mhitm.c + weapon.c + throw.c + explode.c + kick.c)
//

pub mod dokick_ext;
pub mod dothrow_ext;
pub mod engine;
pub mod explode;
pub mod explode_ext;
pub mod kick;
pub mod mhitm;
pub mod mhitu;
pub mod mhitu_ext;
pub mod mthrowu_ext;
pub mod throw;
pub mod uhitm;
pub mod uhitm_ext;
pub mod weapon;
pub mod weapon_ext;

// [v2.0.0 R3] 기존 경로 호환: combat::CombatEngine 등
pub use engine::*;

// ──────────────────────────────────────────────────────────────
// [v2.0.0 R4] 전투 결과 타입 — 시스템 간 통신에 사용
//
//
// 이벤트 큐 도입 후에는 GameEvent로 전환 예정.
// ──────────────────────────────────────────────────────────────

use crate::core::entity::status::StatusFlags;

/// 전투 한 번의 결과
#[derive(Debug, Clone)]
pub struct CombatResult {
    /// 공격자 이름
    pub attacker_name: String,
    /// 방어자 이름
    pub defender_name: String,
    /// 명중 여부
    pub hit: bool,
    /// 적용된 데미지 (빗나간 경우 0)
    pub damage: i32,
    /// 방어자 사망 여부
    pub killed: bool,
    /// 적용된 상태 이상 (있는 경우)
    pub status_applied: Option<StatusFlags>,
    /// 전투 메시지
    pub message: String,
}

impl CombatResult {
    /// 명중 결과 생성
    pub fn hit(
        attacker: impl Into<String>,
        defender: impl Into<String>,
        damage: i32,
        message: impl Into<String>,
    ) -> Self {
        Self {
            attacker_name: attacker.into(),
            defender_name: defender.into(),
            hit: true,
            damage,
            killed: false,
            status_applied: None,
            message: message.into(),
        }
    }

    /// 빗나감 결과 생성
    pub fn miss(
        attacker: impl Into<String>,
        defender: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            attacker_name: attacker.into(),
            defender_name: defender.into(),
            hit: false,
            damage: 0,
            killed: false,
            status_applied: None,
            message: message.into(),
        }
    }

    /// 치사 결과 표시
    pub fn with_kill(mut self) -> Self {
        self.killed = true;
        self
    }

    /// 상태 이상 적용 표시
    pub fn with_status(mut self, flag: StatusFlags) -> Self {
        self.status_applied = Some(flag);
        self
    }
}

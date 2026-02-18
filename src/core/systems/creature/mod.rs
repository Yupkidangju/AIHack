// [v2.0.0 Phase R4] 생물 공통 시스템 (status + attrib + equipment + movement)
//

pub mod attrib;
pub mod attrib_ext;
pub mod death;
pub mod do_wear;
pub mod end;
pub mod equipment;
pub mod evolution;
pub mod exper;
pub mod exper_ext;
pub mod movement;
pub mod regeneration;
pub mod rip_ext;
pub mod status;
pub mod were_ext;
pub mod worn;

// ──────────────────────────────────────────────────────────────
// [v2.0.0
//
//
//
// 설계 근거:
//
//
//
// - 향후 Phase R4.5 LLM 연동의 CreatureSnapshot/Observation과 연계
// ──────────────────────────────────────────────────────────────

use crate::core::entity::status::StatusFlags;
use serde::{Deserialize, Serialize};

///
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureSnapshot {
    /// 생물 이름 (표시용)
    pub name: String,
    /// 현재 체력
    pub hp: i32,
    /// 최대 체력
    pub hp_max: i32,
    /// 방어도 (AC)
    pub ac: i32,
    /// 레벨
    pub level: i32,
    /// 현재 위치
    pub x: i32,
    pub y: i32,
    /// 적대 여부
    pub hostile: bool,
    /// 활성 상태 플래그
    pub status_flags: StatusFlags,
    /// 저항 플래그
    pub resistances: StatusFlags,
    ///
    pub is_player: bool,
}

impl CreatureSnapshot {
    ///
    pub fn from_player(
        name: &str,
        hp: i32,
        hp_max: i32,
        ac: i32,
        level: i32,
        x: i32,
        y: i32,
        status_flags: StatusFlags,
        resistances: StatusFlags,
    ) -> Self {
        Self {
            name: name.to_string(),
            hp,
            hp_max,
            ac,
            level,
            x,
            y,
            hostile: false,
            status_flags,
            resistances,
            is_player: true,
        }
    }

    ///
    pub fn from_monster(
        name: &str,
        hp: i32,
        hp_max: i32,
        ac: i32,
        level: i32,
        x: i32,
        y: i32,
        hostile: bool,
        status_flags: StatusFlags,
        resistances: StatusFlags,
    ) -> Self {
        Self {
            name: name.to_string(),
            hp,
            hp_max,
            ac,
            level,
            x,
            y,
            hostile,
            status_flags,
            resistances,
            is_player: false,
        }
    }

    /// 생존 여부 확인
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// 특정 저항 보유 여부
    pub fn has_resistance(&self, flag: StatusFlags) -> bool {
        self.resistances.contains(flag)
    }

    /// 특정 상태 이상 보유 여부
    pub fn has_status(&self, flag: StatusFlags) -> bool {
        self.status_flags.contains(flag)
    }

    ///
    pub fn distance_sq(&self, other: &CreatureSnapshot) -> i32 {
        (self.x - other.x).pow(2) + (self.y - other.y).pow(2)
    }

    /// 인접 여부 (대각선 포함, dist_sq <= 2)
    pub fn is_adjacent(&self, other: &CreatureSnapshot) -> bool {
        self.distance_sq(other) <= 2
    }
}

///
///
///
///
///
pub trait Combatant {
    /// 전투용 스냅샷 생성
    fn snapshot(&self) -> CreatureSnapshot;

    /// 명중 보너스 (attack bonus)
    fn attack_bonus(&self) -> i32;

    /// 데미지 보너스 (damage bonus)
    fn damage_bonus(&self) -> i32;

    /// AC 유효값 (장비 + 상태 반영)
    fn effective_ac(&self) -> i32;

    ///
    fn display_name(&self) -> String;
}

///
///
pub trait DamageReceiver {
    /// 데미지 적용. 사망 시 true 반환
    fn apply_damage(&mut self, amount: i32) -> bool;

    /// 상태 이상 적용
    fn apply_status(&mut self, flag: StatusFlags, turns: u32);

    /// 사망 처리
    fn on_death(&mut self);
}

// [v2.0.0 Phase R4] 아이템 사용/관리 시스템 (eat.c + read.c + potion.c + apply.c + zap.c + mkobj.c)
//

pub mod apply;
pub mod artifact_ext;
pub mod eat;
pub mod eat_ext;
pub mod item_damage;
pub mod item_helper;
pub mod item_tick;
pub mod item_use;
pub mod loot;
pub mod mkobj;
pub mod objnam;
pub mod pickup;
pub mod potion;
pub mod potion_ext;
pub mod read;
pub mod read_ext;
pub mod weight;
pub mod write_ext;
pub mod zap;

// ──────────────────────────────────────────────────────────────
// [v2.0.0
//
// 설계 근거:
//
//
// - 새 아이템 효과 추가 = trait impl 1개 (기존: match 분기 추가)
//
// ──────────────────────────────────────────────────────────────

use crate::core::entity::status::StatusFlags;

/// 아이템 사용 결과
#[derive(Debug, Clone)]
pub enum UseResult {
    /// 효과 적용 성공
    Success {
        ///
        message: String,
        ///
        consumed: bool,
    },
    /// 효과 없음 (이미 적용됨, 저항됨 등)
    NoEffect { message: String },
    /// 사용 불가 (잘못된 대상 등)
    Failure { message: String },
}

impl UseResult {
    /// 성공 결과 생성 (아이템 소비)
    pub fn consumed(msg: impl Into<String>) -> Self {
        Self::Success {
            message: msg.into(),
            consumed: true,
        }
    }

    /// 성공 결과 생성 (아이템 유지)
    pub fn applied(msg: impl Into<String>) -> Self {
        Self::Success {
            message: msg.into(),
            consumed: false,
        }
    }

    /// 효과 없음 결과 생성
    pub fn no_effect(msg: impl Into<String>) -> Self {
        Self::NoEffect {
            message: msg.into(),
        }
    }

    /// 실패 결과 생성
    pub fn failed(msg: impl Into<String>) -> Self {
        Self::Failure {
            message: msg.into(),
        }
    }

    /// 성공 여부 확인
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// 아이템 소비 여부 확인
    pub fn should_consume(&self) -> bool {
        matches!(self, Self::Success { consumed: true, .. })
    }

    /// 메시지 참조
    pub fn message(&self) -> &str {
        match self {
            Self::Success { message, .. } => message,
            Self::NoEffect { message } => message,
            Self::Failure { message } => message,
        }
    }
}

///
///
#[derive(Debug)]
pub struct UseContext {
    /// 사용 시점의 턴 번호
    pub turn: u64,
    ///
    pub user_status: StatusFlags,
    /// 축복(blessed) 여부
    pub blessed: bool,
    /// 저주(cursed) 여부
    pub cursed: bool,
    /// 강화 수치 (spe)
    pub enchantment: i8,
}

///
///
///
///
///
/// # 사용 예시 (향후 구현)
/// ```ignore
/// struct PotionOfHealing;
/// impl UseEffect for PotionOfHealing {
///     fn item_class(&self) -> &str { "potion" }
///     fn apply(&self, ctx: &UseContext, rng: &mut NetHackRng) -> UseResult {
///         let heal = if ctx.blessed { rng.d(8, 4) + 8 } else { rng.d(4, 4) + 4 };
///         UseResult::consumed(format!("You feel better. (+{} HP)", heal))
///     }
/// }
/// ```
pub trait UseEffect: Send + Sync {
    /// 아이템 클래스 식별자 (\"potion\", \"scroll\", \"wand\" 등)
    fn item_class(&self) -> &str;

    /// 아이템 이름 (식별 전/후)
    fn name(&self) -> &str;

    /// 효과 적용
    fn apply(&self, ctx: &UseContext, rng: &mut crate::util::rng::NetHackRng) -> UseResult;

    /// 축복/저주에 따른 효과 변화 설명 (도감 표시용)
    fn buc_description(&self) -> Option<String> {
        None
    }
}

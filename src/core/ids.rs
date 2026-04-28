use serde::{Deserialize, Serialize};

/// [v0.1.0] Phase 1 최소 엔티티 식별자다.
/// 실제 엔티티 저장소는 Phase 2 이후에 붙이며, 0은 문서상 invalid sentinel로 예약한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u32);

/// [v0.1.0] Phase 1에서는 Main branch만 필요하지만, 후속 Phase의 레벨 확장을 위해 타입을 먼저 고정한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LevelId {
    pub branch: BranchId,
    pub depth: i16,
}

/// [v0.1.0] 첫 headless core는 메인 던전 분기만 사용한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BranchId {
    Main,
}

impl LevelId {
    /// [v0.1.0] Phase 5 고정 메인 던전 level id를 생성한다.
    pub const fn main(depth: i16) -> Self {
        Self {
            branch: BranchId::Main,
            depth,
        }
    }
}

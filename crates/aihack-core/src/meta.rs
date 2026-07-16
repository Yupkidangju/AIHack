use serde::{Deserialize, Serialize};

/// 실행 재현성에 필요한 세션 seed metadata다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMeta {
    pub seed: u64,
}

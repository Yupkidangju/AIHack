//

pub mod interaction;
pub mod minion_ext;
pub mod pray;
pub mod pray_ext;
pub mod priest_ext;
pub mod shk_ext;
pub mod shop;
pub mod steal;
pub mod steal_ext;
pub mod talk;
pub mod vault_ext;

/// [v2.20.0 R8-3] LLM 교체 포인트 — 게임 내 모든 동적 텍스트 생성 인터페이스
/// 현재 적용 영역: talk, pray, interaction, death, shop (7곳+)
pub trait InteractionProvider: Send + Sync {
    /// 컨텍스트에 기반한 대사/텍스트 생성 (R7 기존)
    fn generate_dialogue(&self, context: &str) -> String;

    /// [R8-3] 사망 에필로그 생성 (원본: end.c killer_format)
    fn generate_death_epitaph(&self, cause: &str, player_name: &str) -> String;

    /// [R8-3] 상점 주인 반응 대사 생성 (원본: shk.c shk_greet)
    /// reaction_type: "welcome", "pay_reminder", "thief", "identify", "too_poor" 등
    fn generate_shop_reaction(
        &self,
        reaction_type: &str,
        shopkeeper_name: &str,
        amount: i64,
    ) -> String;

    /// [R8-3] 묘비 텍스트 생성 (원본: end.c outrip)
    fn generate_tombstone_text(&self, player_name: &str, cause: &str, score: u64) -> String;
}

#[derive(Default, Clone)]
pub struct DefaultInteractionProvider;

/// [v2.20.0 R8-3] 기본 구현 — 원본 NetHack 스타일 하드코딩 텍스트
impl InteractionProvider for DefaultInteractionProvider {
    fn generate_dialogue(&self, context: &str) -> String {
        format!("Default Dialogue [{}]", context)
    }

    fn generate_death_epitaph(&self, cause: &str, player_name: &str) -> String {
        format!("{} died. Cause: {}", player_name, cause)
    }

    fn generate_shop_reaction(
        &self,
        reaction_type: &str,
        shopkeeper_name: &str,
        amount: i64,
    ) -> String {
        match reaction_type {
            "welcome" => format!("{}: \"Welcome to my shop!\"", shopkeeper_name),
            "pay_reminder" => format!("{}: \"Please pay before leaving!\"", shopkeeper_name),
            "thief" => format!("{} yells: \"Stop, thief!\"", shopkeeper_name),
            "identify" => format!("{} identifies an item for you.", shopkeeper_name),
            "too_poor" => format!(
                "{}: \"You don't have enough gold! (Need: {} zm)\"",
                shopkeeper_name, amount
            ),
            "paid" => format!("{}: \"Thank you for your patronage.\"", shopkeeper_name),
            "nothing_owed" => format!("{}: \"You don't owe anything.\"", shopkeeper_name),
            _ => format!("{}: \"...\"", shopkeeper_name),
        }
    }

    fn generate_tombstone_text(&self, player_name: &str, cause: &str, score: u64) -> String {
        format!("Here lies {}\nScore: {}\n{}", player_name, score, cause)
    }
}

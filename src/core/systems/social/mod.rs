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

pub trait InteractionProvider: Send + Sync {
    /// 컨텍스트에 기반한 대사/텍스트 생성
    fn generate_dialogue(&self, context: &str) -> String;
}

#[derive(Default, Clone)]
pub struct DefaultInteractionProvider;

impl InteractionProvider for DefaultInteractionProvider {
    fn generate_dialogue(&self, context: &str) -> String {
        format!("Default Dialogue [{}]", context)
    }
}

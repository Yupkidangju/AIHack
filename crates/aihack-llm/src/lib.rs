//! 결정론 core와 분리된 local LLM presentation adapter 경계다.

pub mod config;
pub mod decision;
pub mod narrative;
pub mod service;
pub mod soft_adjudication;
pub mod transport;
pub mod worker;

pub use aihack_ai_contract::ClientRevision;

pub(crate) fn is_forbidden_control(character: char) -> bool {
    let code = character as u32;
    code <= 0x1f || (0x7f..=0x9f).contains(&code)
}

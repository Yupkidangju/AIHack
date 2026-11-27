//! Content bootstrap과 게임 실행을 조합하고 adapter에는 안정된 client 계약만 제공한다.

pub mod appearance;
pub mod bootstrap;
pub mod causal;
pub mod client;
pub mod domain;
pub mod observation;
pub mod save;
pub mod session;
pub mod snapshot;
pub mod systems;
#[cfg(feature = "testing")]
#[doc(hidden)]
pub mod testing;
mod transaction;
pub mod world;

pub use aihack_core::error::{ContentError, GameError};
pub use client::GameClient;
pub use session::GameSession;
mod campaign;
mod campaign_map;

pub fn shop_buy_price(kind: aihack_core::domain::item::ItemKind) -> Option<u32> {
    let registry = aihack_content::registry().ok()?;
    systems::shop::buy_price(kind, registry).ok()
}

pub fn item_display_name(kind: aihack_core::domain::item::ItemKind) -> String {
    aihack_content::registry()
        .map(|reg| aihack_content::item_display_name(kind, reg))
        .unwrap_or_else(|_| format!("{kind:?}"))
}

pub fn monster_display_name(kind: aihack_core::domain::monster::MonsterKind) -> String {
    aihack_content::registry()
        .map(|reg| aihack_content::monster_display_name(kind, reg))
        .unwrap_or_else(|_| format!("{kind:?}"))
}

pub fn item_glyph(kind: aihack_core::domain::item::ItemKind) -> char {
    aihack_content::registry()
        .map(|reg| aihack_content::item_glyph(kind, reg))
        .unwrap_or('?')
}

pub fn monster_glyph(kind: aihack_core::domain::monster::MonsterKind) -> char {
    aihack_content::registry()
        .map(|reg| aihack_content::monster_glyph(kind, reg))
        .unwrap_or('?')
}

pub fn item_class(
    kind: aihack_core::domain::item::ItemKind,
) -> Option<aihack_core::domain::item::ItemClass> {
    let reg = aihack_content::registry().ok()?;
    domain::item::try_item_data_from_registry(kind, reg)
        .ok()
        .map(|data| data.class)
}

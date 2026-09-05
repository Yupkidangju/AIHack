//! UI와 외부 transport에 의존하지 않는 결정론적 게임 core다.
//!
//! R5는 순환 의존성을 만들지 않는 타입부터 이 crate로 이동한다.

pub mod action;
pub mod campaign;
pub mod death;
pub mod doors;
pub mod error;
pub mod event;
pub mod hash;
pub mod ids;
pub mod invariant;
pub mod meta;
pub mod movement;
pub mod position;
pub mod rng;
pub mod run_state;
pub mod save;
pub mod score;
pub mod session;
pub mod transaction;
pub mod traps;
pub mod turn;
pub mod vision;
pub mod world;

pub mod domain {
    pub mod combat;
    pub mod entity;
    pub mod inventory;
    pub mod item;
    pub mod level;
    pub mod map;
    pub mod monster;
    pub mod player;
    pub mod status;
    pub mod tile;
}

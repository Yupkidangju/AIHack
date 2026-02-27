pub mod combat_bridge;
pub mod hunger_bridge;
pub mod integration_r33;
pub mod integration_r34;
pub mod integration_tests;
pub mod luck_align_bridge;
pub mod monster_ai_bridge;
pub mod potion_bridge;
pub mod scroll_bridge;
pub mod social_bridge;
pub mod spell_bridge;
pub mod status_bridge;
pub mod terrain_bridge;
pub mod wand_bridge;
pub mod xp_bridge;
// [v2.31.0 Phase 95] 게임 턴/세이브 브릿지
pub mod game_bridge_phase95_ext;
// [v2.35.0 Phase 99] 세이브/로드 확장
pub mod save_phase99_ext;
// [v2.36.0 Phase 100] 🏆 최종 통합
pub mod event_phase100_ext;
pub mod gameloop_phase100_ext;
pub mod integration_phase100_ext;
// [v2.39.0 Phase 103] 시간/턴 관리
pub mod time_phase103_ext;
// [v2.40.0 Phase 104] 🏆 최종 마무리
pub mod finale_phase104_ext;
// [v2.41.0 Phase FINAL] 💯 100% 이식 완료 기념!
pub mod centennial_ext;

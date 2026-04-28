// 몬스터 생성 시스템 (makemon.c + spawn_manager)

pub mod makemon;
pub mod spawn_manager;

pub mod summon_ext;

pub mod spawn_rule_ext;

// [v2.21.0 R34-1] 몬스터 생성 확장 (원본: makemon.c 순수 함수 이식)
pub mod makemon_ext;
// [v2.22.0 R34-8] 플레이어 몬스터 장비/확률 (원본: mplayer.c + mkobj.c)
pub mod mplayer_ext;
// [v2.29.0 Phase 93] 몬스터 생성 확장
pub mod makemon_phase93_ext;
// [v2.34.0 Phase 98] 몬스터 생성 확장
pub mod mkmon_phase98_ext;

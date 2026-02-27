// 월드/환경 시스템 (stairs + teleport + trap + dig + fountain + vision)

pub mod ball_ext;
pub mod bones_ext;

pub mod altar_ext;
pub mod dbridge_ext;
pub mod detect;
pub mod dig;
pub mod dig_calc_ext;
pub mod dig_ext;
pub mod dig_phase50_ext;
pub mod door_ext;
pub mod engrave;
pub mod engrave_calc_ext;
pub mod engrave_ext;
pub mod fountain;
pub mod fountain_effect_ext;
pub mod fountain_ext;
pub mod light_ext;
pub mod light_source_ext;
pub mod lock;
pub mod lock_ext;
pub mod music_ext;
pub mod region_ext;
pub mod search;
pub mod sink;
pub mod sit;
pub mod stairs;
pub mod teleport;
pub mod timeout_ext;
pub mod trap;
pub mod trap_detect_ext;
pub mod trap_ext;
pub mod trap_ext2;
pub mod vision;
pub mod vision_ext;
pub mod vision_phase3_ext;
// [v2.38.0 Phase 102] 조명/시야 통합
pub mod lighting_phase102_ext;
pub mod vision_system;

pub mod world_time_ext;

pub mod weather_ext;
// [v2.22.0 R34-4] 탐지/탐색 확장 (원본: detect.c 순수 판정 함수)
pub mod detect_ext;
// [v2.24.0 Phase 50-2] 몬스터 터널/붕괴/매몰
pub mod terrain_destroy_ext;
// [v2.37.0 Phase 101] 지형/환경 통합
pub mod environ_phase101_ext;
// [v2.40.0 Phase 104] 함정/위험
pub mod hazard_phase104_ext;
// [v2.24.0 Phase 50-3] 액체 피해/환경 파괴
pub mod liquid_damage_ext;
// [v2.27.0 Phase 91] 함정/텔레포트/타이머 확장
pub mod detect_phase91_ext;
// [v2.30.0 Phase 94] 환경 효과/메시지 확장
pub mod region_phase94_ext;
// [v2.32.0 Phase 96] 함정/시야 확장
pub mod teleport_phase91_ext;
pub mod timeout_phase91_ext;
pub mod trap_phase91_ext;
pub mod trap_phase96_ext;
pub mod vision_phase96_ext;

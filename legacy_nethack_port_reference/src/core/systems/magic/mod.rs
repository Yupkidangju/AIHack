// magic 시스템 모듈
// [v2.13.0] 신규 생성: spell_ext, detect_ext, teleport_ext 하위 모듈 등록

/// 탐지/수정구/수색 등 핵심 로직 (detect.c 이식)
pub mod detect_ext;
pub mod detect_map_ext;
/// 주문 시전/학습/역화 등 핵심 로직 (spell.c 이식)
pub mod spell_ext;
pub mod spell_ext2;
/// 순간이동/위치 판정/레벨 이동 등 핵심 로직 (teleport.c 이식)
pub mod teleport_ext;

pub mod recharge_ext;

pub mod spell_school_ext;
pub mod wand_effect_ext;
// [v2.29.0 Phase 93] 마법 시스템 확장
pub mod spell_phase93_ext;
// [v2.38.0 Phase 102] 마법/주문 통합
pub mod spellbook_phase102_ext;
// [v2.33.0 Phase 97] 스크롤/완드 확장
pub mod read_phase97_ext;
pub mod wand_phase97_ext;

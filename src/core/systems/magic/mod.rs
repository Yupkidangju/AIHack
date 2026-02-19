// magic 시스템 모듈
// [v2.13.0] 신규 생성: spell_ext, detect_ext, teleport_ext 하위 모듈 등록

/// 탐지/수정구/수색 등 핵심 로직 (detect.c 이식)
pub mod detect_ext;
/// 주문 시전/학습/역화 등 핵심 로직 (spell.c 이식)
pub mod spell_ext;
/// 순간이동/위치 판정/레벨 이동 등 핵심 로직 (teleport.c 이식)
pub mod teleport_ext;

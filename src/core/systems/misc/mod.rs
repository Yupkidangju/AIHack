// 기타 시스템 (artifact + luck + timeout + spell + role)

pub mod artifact;
pub mod do_name_ext;
pub mod invent_ext;
pub mod invent_phase3_ext;
pub mod inventory;
pub mod luck;
pub mod luck_ext;
pub mod role;
pub mod role_ext;
pub mod rumors_ext;
pub mod score_calc_ext;
pub mod sit_ext;
pub mod spell;
pub mod timeout;
pub mod topten_ext;
// [v2.22.0 R34-6] 비트마스크 기반 역할 선택 엔진 (원본: role.c 검증 로직)
pub mod role_selection_ext;
// [v2.22.0 R34-7] 정보 표시 확장 (원본: pager.c 문자열 처리/판정)
pub mod pager_ext;
// [v2.22.0 R34-9] 초기 장비/종족 치환 (원본: u_init.c)
pub mod u_init_ext;
// [v2.27.0 Phase 91] 이름 지정 확장
pub mod do_name_phase91_ext;
// [v2.28.0 Phase 92] 직업/종족 확장
pub mod role_phase92_ext;
// [v2.29.0 Phase 93] 사망/종료 확장
pub mod end_phase93_ext;
// [v2.31.0 Phase 95] 퀘스트 시스템 확장
pub mod quest_phase95_ext;
// [v2.33.0 Phase 97] 인벤토리/옵션 확장
pub mod invent_phase97_ext;
pub mod options_phase97_ext;
// [v2.34.0 Phase 98] 점수/통계 확장
pub mod score_phase98_ext;
// [v2.35.0 Phase 99] 종료/사망/메시지 확장
pub mod death_phase99_ext;
pub mod message_phase99_ext;
// [v2.36.0 Phase 100] 난이도/밸런스 통합 🏆
pub mod difficulty_phase100_ext;
// [v2.38.0 Phase 102] 인벤토리/지식 통합
pub mod inventory_phase102_ext;
pub mod knowledge_phase102_ext;
// [v2.39.0 Phase 103] 고스트/본즈
pub mod ghost_phase103_ext;
// [v2.41.0 Phase FINAL] 🏆 최종 잔여 통합
pub mod final_misc_ext;

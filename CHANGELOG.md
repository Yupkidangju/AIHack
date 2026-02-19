# Changelog

이 프로젝트의 모든 주요 변경 사항은 이 파일에 기록됩니다.
형식은 [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)를 따르며, 이 프로젝트는 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)을 준수합니다.
## [2.18.0] - 2026-02-19
### Added
- **[이식] ball_ext.rs  ball.c + worm.c 핵심 함수 이식 (신규 ~380줄, 22테스트)**:
  - `ballfall_damage`  함정 문 쇠공 낙하 데미지 판정 (금속 투구 보정)
  - `drag_down_result`  계단 쇠공 끌림 결과 (앞/뒤, 충돌/끌림)
  - `ball_trap_escape`  쇠공에 의한 함정 탈출 (구덩이/거미줄/곰/용암)
  - `litter_drop_check`/`bc_felt_mask`/`chain_position_for_dist`
  - `drag_encumbrance_check`/`ball_smack_tohit`
  - `worm_grow_interval`/`worm_cut_chance`/`worm_split_check`/`worm_nomove_hp`/`worm_grow_hp`
- **[이식] vault_ext.rs  vault.c 핵심 함수 이식 (신규 ~350줄, 20테스트)**:
  - `guard_reaction`/`guard_entry_check`/`fake_corridor_wall`/`wallify_type`
  - `move_gold_position`/`guard_alignment_penalty`/`guard_warncnt_action`
  - `guard_gold_witness`/`vault_guard_timer_check`/`is_on_boundary`
- **[이식] worm_ext.rs  worm.c 핵심 함수 이식 (신규 ~350줄, 18테스트)**:
  - `get_wormno_check`/`worm_move_result`/`worm_nomove_result`
  - `worm_cut_check`/`worm_cut_result`/`worm_cross_check`
  - `wseg_index`/`worm_initial_segments`/`worm_attack_range`/`random_dir_calc`

### Changed
- 프로젝트 통계: 186 파일, 108,900줄, 1,996 테스트, 이식률 61.4%

### Fixed
- **[감사] ball_ext.rs**: `BallFallResult::MetalHelm` → `MetalHelm(3)`으로 변경 — 원본 ball.c L57의 데미지 3을 명시적으로 포함
- **[감사] worm_ext.rs**: `worm_cut_result` Split 경로에서 `old_hp` 계산 버그 수정 — 원본 worm.c L409 `if(mhpmax < mhp) mhp=mhpmax` 로직 반영 (기존: 새 주사위로 굴림 → 수정: current_hp.min(old_maxhp))

## [2.17.0] - 2026-02-19
### Added
- **[이식] dbridge_ext.rs — dbridge.c + region.c(가스) 핵심 함수 이식 (신규 ~340줄, 20테스트)**:
  - `db_under_type` — 도개교 아래 지형 판정 (얼음/용암/해자/석재)
  - `wall_for_db` — 도개교 벽 방향 오프셋
  - `e_missed_calc` — 낙하 도개교 회피 확률 (비행/부양/성문 보정)
  - `e_jumps_calc` — 도개교 점프 탈출 확률 (혼란/기절/성문 보정)
  - `destroy_db_debris` — 파괴 시 잔해 수 결정
  - `is_horizontal` — 방향의 수평 여부
  - `gas_cloud_ttl`/`gas_cloud_dissipate`/`gas_cloud_damage` — 가스 구름 관련
- **[이식] timeout_ext.rs — timeout.c 핵심 함수 이식 (신규 ~390줄, 24테스트)**:
  - `stoned_stage` — 석화 카운트다운 5단계
  - `vomiting_stage` — 구토 카운트다운 7단계
  - `choke_stage` — 질식 카운트다운 5단계
  - `luck_decay`/`luck_decay_interval`/`base_luck` — 행운 감쇠 시스템
  - `lamp_burn_stage`/`candle_burn_stage` — 연소 단계 판정
  - `lamp_next_turns`/`candle_next_turns` — 다음 이벤트까지 턴
  - `egg_hatch_time` — 알 부화 시간 확률 결정
  - `fig_transform_time` — 인형 변신 시간
  - `oil_diluted_turns`/`storm_strike_count` — 기타
- **[이식] region_ext.rs — region.c 핵심 함수 이식 (신규 ~320줄, 25테스트)**:
  - `inside_rect`/`inside_region` — 사각형/구역 포함 판정
  - `compute_bounding_box` — 바운딩 박스 계산
  - `gas_cloud_rects` — 가스 구름 다이아몬드 형태 생성
  - `gas_cloud_hero_effect`/`gas_cloud_monster_effect` — 가스 구름 효과
  - `gas_expire` — 가스 구름 소멸/약화
  - `region_danger_check` — 기도 시 위험도 판정
  - `region_ttl_adjust` — 세이브/로드 TTL 보정

### Changed
- 통계 갱신: Rust 107,556줄(183파일), 테스트 1,936개, 이식률 60.8%

## [2.16.0] - 2026-02-19
### Added
- **[이식] trap_ext.rs — trap.c 핵심 함수 이식 (신규 ~597줄, 20테스트)**:
  - `erode_check` — 아이템 부식 판정 (그리스/축복/내성/부식 단계)
  - `grease_wearoff` — 그리스 보호 후 벗겨짐 확률
  - `burnarmor_slot` — 화염 피해 시 연소 슬롯 결정 (5종)
  - `trap_escape_check` — 함정 탈출 판정 (소코반/부양/민첩)
  - `bear_trap_duration`/`bear_trap_damage` — 곰 함정 턴/데미지
  - `pit_damage`/`pit_escape_turns` — 구덩이 낙하 데미지/탈출 턴
  - `dart_poison_chance` — 다트 독 확률
  - `sleep_gas_duration` — 수면 가스 지속 턴
  - `rock_trap_damage` — 바위 함정 데미지 (헬멧 보정)
  - `fall_through_depth` — 함정 문/구멍 낙하 깊이
  - `arrow_trap_empty` — 화살 함정 소진 확률
  - `rust_trap_slot` — 녹 함정 피격 부위 (4종)
- **[이식] pray_ext.rs — pray.c 핵심 함수 이식 (신규 ~530줄, 12테스트)**:
  - `critically_low_hp` — HP 위험 판정 (레벨별 제수)
  - `angrygods_calc` — 신의 분노 효과 결정 (6종)
  - `prayer_action_calc` — 기도 보상 수준 결정 (6단계)
  - `pat_on_head_boon` — 특별 보상 결정 (무기 축복/골든 글로우/성채 힌트 등)
  - `prayer_timeout` — 기도 쿨다운 턴 (rnz 기반)
  - `fix_hit_hp_boost` — HP 위험 해결 시 최대HP 증가
  - `wall_phasing_duration` — 벽 통과 부여 턴
  - `satisfaction_level` — 신의 만족도 메시지 수준
- **[이식] eat_ext.rs — eat.c 핵심 함수 이식 (신규 ~530줄, 26테스트)**:
  - `tin_nutrition`/`spinach_nutrition` — 통조림 영양 계산 (15종)
  - `corpse_nutrition` — 시체 영양 (종족 보정: 엘프/오크/드워프)
  - `choke_check` — 과식 질식 판정
  - `intrinsic_chance` — 내성 획득 확률 (몬스터 레벨 기반)
  - `intrinsic_pick` — 다중 내성 후보 중 선택 (저수지 샘플링)
  - `newt_energy` — 뉴트 시체 마력 회복
  - `mimic_delay` — 미믹 변장 지속 턴
  - `quantum_speed` — 양자역학자 속도 토글
  - `cannibal_luck_penalty` — 식인 운 감소량
  - `hunger_state_from_nutrition` — 영양 기반 허기 상태 결정

### Changed
- 통계 갱신: Rust 106,157줄(180파일), 테스트 1,873개, 이식률 60.0%

## [2.15.0] - 2026-02-19
### Added
- **[이식] fountain_ext.rs — fountain.c + sit.c 핵심 함수 이식 (신규 ~750줄, 16테스트)**:
  - `drink_fountain_result` — 분수 음용 결과 결정 (마법 축복/구역질/뱀/악마/저주/투명 등 15종)
  - `dip_fountain_result` — 분수 담금 결과 결정 (엑스칼리버/저주 해제/보석/동전 등 12종)
  - `drink_sink_result` — 싱크대 음용 결과 결정 (끓는물/쥐/물약/반지/정령 등 15종)
  - `throne_result` — 왕좌 앉기 결과 결정 (능력치/전기/회복/소원/소환/혼란 등 13종)
  - `rndcurse_count` — 저주할 아이템 수 계산
  - `attrcurse_pick` — 제거할 고유 능력 선택 (FALLTHRU 패턴 시뮬레이션)
  - `fountain_dryup_chance` — 분수 말라붙기 확률
  - `water_snake_count`/`water_demon_wish_chance`/`dip_gold_amount` — 보조 계산
  - `throne_vanish_check`/`sink_scalding_damage`/`sink_sewer_hunger`/`sit_trap_additional_turns`
- **[이식] dig_ext.rs — dig.c 핵심 함수 이식 (신규 ~350줄, 16테스트)**:
  - `dig_effort_calc` — 채굴 노력치 계산 (능력치/강화/부식/드워프 2배)
  - `dig_complete_check` — 채굴 완료 판정 (아래 250/옆 100)
  - `holetime` — 구멍 완료 추정 시간
  - `fillholetyp` — 구멍 채울 액체 유형 결정 (용암/해자/풀/없음)
  - `dig_fumble_check` — 어질거림 실패 판정 (1/3 확률)
  - `dig_check_result` — 채굴 가능 여부 검사 (계단/왕좌/제단/공기/불가능)
  - `earth_elemental_spawn_check`/`earth_elemental_type` — 대지 정령 소환
  - `pit_trap_turns`/`tree_fruit_chance` — 보조 계산
  - `bear_trap_dig_self_hit`/`bear_trap_self_damage` — 곰 함정 속 채굴
- **[이식] artifact_ext.rs — artifact.c 핵심 함수 이식 (신규 ~380줄, 16테스트)**:
  - `spec_dbon_calc` — 아티팩트 특수 데미지 보너스 (다이스 또는 max(tmp,1))
  - `spec_abon_calc` — 아티팩트 특수 공격 보너스
  - `touch_artifact_damage` — 부적절한 터치 데미지 (반마법/자의/은 보너스)
  - `mb_hit_calc` — Magicbane 타격 효과 (Probe/Stun/Scare/Cancel + 추가 데미지)
  - `glow_strength` — 글로우 강도 4단계 (None/Faint/Moderate/Bright)
  - `artifact_fire_destroy_check`/`artifact_cold_destroy_check` — 아이템 파괴 확률
  - `vorpal_chance`/`bisect_chance` — 참수/이등분 확률
  - `fatal_damage` — 즉사 데미지 계산 (2×HP + 200)
  - `glow_intensity_from_count` — 주변 개체 수 기반 글로우 강도

### Changed
- **통계**: 104,610 Rust줄 / 177 파일 / 1,808 테스트 / 이식률 59.1%

## [2.14.0] - 2026-02-19
### Added
- **[이식] dokick_ext.rs — dokick.c 핵심 함수 이식 (신규 ~430줄, 24테스트)**:
  - `kick_damage_calc` — 킥 데미지 계산 (STR+DEX+CON/15, 부츠/서투름/피부)
  - `kick_range_calc` — 킥 오브젝트 사거리 (STR/2 - weight/40, 환경 보정)
  - `kick_clumsy_check` — 서투름 판정 (무게/어질증/방어구)
  - `kick_dodge_check` — 몬스터 회피 판정 (12개 조건)
  - `kick_block_chance` — 몬스터 손 방어 확률
  - `kick_recoil_range` — 부양 시 반동 거리
  - `kick_avrg_attrib` — 평균 능력치 계산
  - `kick_secret_door_check` — 비밀문 발견 확률
  - `kick_throne_result` — 왕좌 차기 4가지 결과
  - `kick_tree_result` — 나무 차기 3가지 결과
  - `mercenary_gold_required`/`mercenary_bribe_check` — 용병 매수
  - `box_kick_result` — 상자 차기 4가지 결과
- **[이식] steed_ext.rs — steed.c 핵심 함수 이식 (신규 ~375줄, 17테스트)**:
  - `can_saddle_check` — 안장 장착 가능 여부 (심볼/크기/형태)
  - `saddle_chance_calc` — 안장 장착 성공률 계산
  - `mount_slip_damage`/`mount_slip_check` — 탑승 미끄러짐
  - `dismount_damage`/`dismount_leg_damage_duration` — 낙마 데미지
  - `steed_gallop_duration` — 킥 후 질주 기간
  - `exercise_steed_check` — 기승 기술 100턴 연습 판정
  - `steed_wake_calc`/`maybewakesteed` — 수면/마비 탈것 깨우기
  - `steed_tame_decrease` — 킥 시 친밀도 감소 + 낙마 판정
- **[이식] sounds_ext.rs — sounds.c 핵심 함수 이식 (신규 ~365줄, 20테스트)**:
  - `growl_sound_type` — 으르렁 소리 동사 (11종 매칭)
  - `yelp_sound_type` — 학대 펫 비명 동사 (청각/농아 분기)
  - `whimper_sound_type` — 고통 펫 신음 동사
  - `halluc_sound` — 환각 소리 36종 랜덤 선택
  - `room_sound_chance` — 방 유형별 소리 발생 확률
  - `laugh_sound` — 웃음 4종 결정
  - `pet_sound_verb` — 펫 소리 통합 결정
  - `wake_radius_squared` — 소리 각성 반경² 계산

### Changed
- **통계**: 103,085 Rust줄 / 174 파일 / 1,756 테스트 / 이식률 58.2%

## [2.13.0] - 2026-02-19
### Added
- **[이식] spell_ext.rs — spell.c 핵심 함수 이식 (신규 ~1,011줄, 36테스트)**:
  - `percent_success` — 주문 시전 성공률 계산 (장비/스킬/레벨/지능 종합)
  - `spell_energy_cost` — 주문 에너지 비용 (레벨×5)
  - `study_delay` — 마법서 학습 소요 턴 수
  - `spell_hunger_cost` — 시전 시 배고픔 소비 (위저드 지능 보정)
  - `spell_backfire_calc` — 잊어버린 주문 역화 (혼란/기절 분기)
  - `cast_protection_calc` — 보호 마법 AC 보호량
  - `spell_retention_display` — 보유율 범위 표시 (Expert→2%, Unskilled→25%)
  - `cursed_book_effect` — 저주 마법서 8가지 부작용 결정
  - `spell_damage_bonus` — 주문 데미지 보너스
  - `spell_type_mnemonic` — 주문 유형 카테고리 문자열
  - `read_ability_check` — 마법서 읽기 능력 판정
  - `losespells_calc` — 기억상실 시 잃을 주문 수
- **[이식] detect_ext.rs — detect.c 핵심 함수 이식 (신규 ~694줄, 25테스트)**:
  - `level_distance_desc`/`level_distance_str` — 레벨 거리 설명 문자열
  - `crystal_ball_oops` — 수정구 실패 5가지 효과
  - `crystal_ball_halluc_msg` — 환각 수정구 6가지 메시지
  - `crystal_ball_oops_check` — 수정구 실패 판정 (지능/저주)
  - `crystal_ball_gaze_delay` — 응시 소요 턴
  - `search_chance`/`search_succeeds` — 비밀문/함정 탐색 확률
  - `obj_trap_combine` — 함정 위치 비트 결합
  - `food_detect_class` — 음식/물약 탐지 분기
  - `gold_detect_fail_message` — 금 탐지 실패 메시지
  - `monster_detect_wakeup` — 저주 몬스터 탐지 각성
- **[이식] teleport_ext.rs — teleport.c 핵심 함수 이식 (신규 ~664줄, 26테스트)**:
  - `tele_jump_ok` — 구역 간 장거리 이동 가능 판정
  - `goodpos_terrain_check` — 지형별 위치 유효성 판정
  - `is_valid_teleport_pos` — 좌표 맵 범위 확인
  - `random_teleport_level` — 비제어 레벨 순간이동 목적지
  - `level_tele_dest_calc` — 레벨 텔레포트 목적지 분류
  - `tele_trap_resist`/`level_tele_trap_resist` — 함정 저항
  - `distmin`/`distu` — 거리 계산 유틸리티
  - `safe_teleds_candidate`/`rloc_candidate` — 랜덤 좌표 생성
  - `confusion_tele_override`/`amulet_energy_drain` — 보조 판정
- **magic 모듈 시스템 등록**: `src/core/systems/magic/mod.rs` 신규 생성

## [2.12.0] - 2026-02-19
### Added
- **[이식] potion_ext.rs — potion.c 핵심 함수 이식 (신규 ~550줄, 29테스트→42테스트)**:
  - `itimeout`/`itimeout_incr` — 내재 능력 타임아웃 값 클램프 (MAX=100M)
  - `healup_calc` — 물약별 회복량 계산 (Healing/ExtraHealing/FullHealing, BCS 반영)
  - `potion_nutrition` — 물약 음용 영양 계산 (Water/Booze/FruitJuice, 희석)
  - `gain_energy_calc` — 에너지 물약 최대/현재 에너지 변화량 계산
  - `acid_damage_calc` — 산성 물약 데미지 (BCS+내성 반영)
  - `mixtype` — 물약 연금술 혼합 테이블 (12가지 조합)
  - `djinni_chance` — 진 병 개봉 결과 (blessed=80%소원, cursed=80%적대)
  - `bottlename` — 병 타입 무작위 선택(7종)
  - `oil_lamp_fill` — 기름 등잔 충전 수명 계산
  - `h2o_dip_result` — 성수/저주수 담금 결과 (Uncurse/Bless/Unbless/Curse)
  - `levitation_head_damage` — 저주 부양 물약 천장 충돌 데미지
  - `potion_explode_damage` — 혼합 폭발 데미지
- **[이식] read_ext.rs — read.c 핵심 함수 이식 (신규 ~985줄, 28테스트→34테스트)**:
  - `tshirt_text_index`/`apron_text_index` — T셔츠/앞치마 문구 인덱스(71+9종)
  - `credit_card_text` — 신용카드 발급사+번호 생성(14종)
  - `is_chargeable` — 충전 가능 여부 판정 (Wand/Ring/Tool 분기)
  - `recharge_wand_result` — 지팡이 충전 (폭발/StripSpe/정상, Wishing 특수)
  - `recharge_ring_result` — 반지 충전 (폭발/Spin, BCS 반영)
  - `recharge_tool_result` — 도구 충전 (Bell/Marker/Camera/CrystalBall 등 15종)
  - `enchant_armor_calc` — 방어구 강화 (과강화 폭발/용비늘 업그레이드/정상)
  - `enchant_weapon_calc` — 무기 강화 보너스
  - `forget_percentage_calc` — 망각 스크롤 비율 결정
  - `erode_wipeout_count` — 부식 텍스트 와이프아웃 문자 수
  - `create_monster_count` — 몬스터 생성 수량
  - `scare_monster_result` — 공포/각성 효과 방향
- **[이식] mthrowu_ext.rs — mthrowu.c 핵심 함수 이식 (신규 ~901줄, 37테스트)**:
  - `linedup_check` — 직선/대각선 정렬 판정 (BOLT_LIM 범위)
  - `distmin`/`dist2` — 체스 거리/유클리드 거리 제곱
  - `polearm_attack_calc` — 장대무기 명중 보너스+데미지 (POLE_LIM 사거리)
  - `mon_multishot_count` — 몬스터 연사 횟수 (레벨/탄환 기반)
  - `thitu_check` — 플레이어 피격 판정 (DEX/레벨/갑옷/d20)
  - `drop_throw_survival` — 투척 후 아이템 생존 판정
  - `hits_bars_check`/`bar_hit_sound` — 쇠창살 통과/소리 판정 (10여 클래스)
  - `breath_weapon_name` — 브레스 무기 이름(8종)
  - `breath_cooldown` — 브레스 재사용 대기 계산
  - `spit_venom_type` — 침 뱉기 독액 종류
  - `retreat_throw_check` — 후퇴 중 투척 판정
  - `random_breath_type` — 랜덤 브레스 유형 결정(8종)
- **총 통계**: 전체 테스트 1,608개 (이전 1,510 → +98), 신규 3 파일 추가

## [2.11.0] - 2026-02-19
### Added
- **[이식] weapon_ext.rs — weapon.c 잔여 함수 대량 이식 (1,063→1,902줄, +839줄, 16테스트 추가)**:
  - `select_rwep_result` — 몬스터 원거리 무기 선택 AI (코카트리스 알/Kop 크림파이/바위/장대무기/투척 무기 5단계 우선순위, 발사대 매칭)
  - `can_advance_result` — 스킬 승급 가능 판단 (경험치+슬롯+총 승급 회수 3중 검증)
  - `could_advance_result` — 슬롯만 있으면 승급 가능 판단
  - `peaked_skill_result` — 스킬 최대 도달 + 경험치 넘침 판정
  - `skill_advance_result` — 스킬 승급 실행 결과 (슬롯 소비, most/more skilled 메시지 분기)
  - `add_weapon_skill_gained_advanceable` — 레벨업 슬롯 추가 후 새로 승급 가능 스킬 감지
  - `lose_weapon_skill_result` — 레벨 다운 시 슬롯 감소/스킬 강등 처리
  - `skill_init_result` — 게임 시작 시 역할별 스킬 초기화 (인벤토리 무기 반영, 마법 스킬, 기승 등)
  - `enhance_weapon_skill_entries` — #enhance 명령 UI 데이터 생성 (카테고리 분류, 승급 가능/최대 도달 표시)
- **[이식] wield_ext.rs — wield.c 핵심 함수 이식 (신규 846줄, 22테스트)**:
  - `erodeable_wep` — 부식 가능 무기 판정 (WEAPON_CLASS + 무기도구 + 쇠구슬 + 쇠사슬)
  - `will_weld` — 저주 무기 손 용접 판정 (cursed AND erodeable/tin_opener)
  - `welded_result`/`mwelded_result` — 플레이어/몬스터 무기 용접 상태 확인
  - `cant_wield_corpse_result` — 코카트리스 시체 맨손 장비 석화 위험 판정
  - `ready_weapon_result` — 무기 장비 결과 계산 (빈손/석화/양손+방패/용접/정상)
  - `can_twoweapon_result` — 쌍수 전투 가능 판정 (역할/무기타입/양손/방패/아티팩트 11개 사유)
  - `chwepon_result` — 마법 무기 강화/약화 효과 (벌레이빨⇄크리스나이프, 증발, Magicbane 반응)
  - `wield_tool_check` — 도구 장비 사전 검증 (이미장비/방어구착용/용접/형태/양손+방패)
  - `unweapon_check` — 비무기 판정 (발사대/탄약/투척/장대+탈것)
  - `wield_swap_check` — 무기 교환 사전 판정
  - `enchant_weapon_cap_check` — 강화 상한(±5) 증발 확률 판정
- **[이식] explode_ext.rs — explode.c 미이식 핵심 로직 완전 이식 (신규 793줄, 32테스트)**:
  - `retributive_damage_reduction` — 역할별 보복 공격 감소 (Priest/Wizard:1/5, Knight:1/2)
  - `explosion_adtype`/`explosion_description` — 폭발 zap타입→공격타입/설명문자열 결정
  - `resistance_mask_result` — 3×3 저항 마스크 계산 (분해+지팡이→비생물/악마 특수)
  - `opposite_resistance_bonus` — 반대 속성 2배 (냉기저항+화염→2x, 화염저항+냉기→2x)
  - `grab_damage_bonus` — 잡기 2배 데미지
  - `half_physical_damage` — 물리/산성 절반 감소
  - `scatter_direction_result` — 파편 방향/범위 결정 (8방향, 무게 보정)
  - `scatter_fracture_chance`/`scatter_destroy_chance` — 바위 파쇄(9/10)/유리·달걀 파괴
  - `splatter_oil_damage` — 기름 폭발 데미지 (희석:3d4, 일반:4d4)
  - `wake_range` — 폭발 소음 범위 (삼킨 상태 1/4)
  - `monster_explosion_damage` — 종합 몬스터 폭발 데미지 계산
  - `engulf_explosion_adjective` — 삼킨 상태 폭발 형용사 (동물/비동물 분기)
- **[이식] dothrow_ext.rs — dothrow.c 핵심 함수 이식 (신규 885줄, 27테스트)**:
  - `multishot_count` — 연사 횟수 계산 (스킬/역할/종족 복합 보너스, 석궁 힘 제한)
  - `throwing_weapon_check` — 투척 무기 판정 (미사일/창/단검+관통/워해머/아클리스)
  - `autoquiver_priority` — 자동 화살집 우선순위 (4단계: 발사대매칭→미사일→보조→기타)
  - `throw_range` — 투척 사거리 계산 (무게/발사대/특수무기/수중/공중 반동)
  - `walk_path_bresenham` — Bresenham 직선 경로 생성 알고리즘
  - `hurtle_collision_damage` — 돌진 충돌 데미지 (잔여 범위 기반)
  - `slip_chance` — 저주/기름 미끄러짐 확률 판정 (1/7)
  - `toss_up_damage` — 위로 던진 물건 낙하 데미지 (무게기반, 금속헬멧, 물리절반)
  - `omon_adj` — 투척 명중 보정치 (크기/수면/고정/아이템별)
  - `gem_accept_result` — 유니콘 보석 선물 반응 (진짜→행운변화, 유리→적대, 회색→무시)
- **[이식] priest_ext.rs — priest.c 핵심 함수 이식 (신규 824줄, 29테스트)**:
  - `mon_aligntyp` — 몬스터 성향 판정 (사제→shrine, 미니언→min_align, 일반→maligntyp)
  - `align_str` — 성향 문자열 변환 (chaotic/neutral/lawful/unaligned)
  - `piousness_str` — 경건도 문자열 (20단계: piously~transgressed)
  - `p_coaligned` — 사제-플레이어 성향 일치 판정
  - `priestname_result` — 사제/미니언 이름 생성 (고위사제/여사제/수호천사/배교자)
  - `in_your_sanctuary_check` — 성역 판정 (동맹+성소+비죄과)
  - `priest_donation_result` — 기부 결과 (거부/인색/감사/경건/Protection/정화 6단계)
  - `ghod_direction` — 신벌 번개 방향 계산 (제단→문/벽→플레이어)
  - `move_special_pick` — 사제/상인 이동 최적 위치 선택
  - `temple_entry_message` — 사원 입장 메시지 판정 (성역/모독/경건/평온/무관리)
  - `should_convert_to_roamer` — 분노 사제→방랑자 변환 조건
  - `mstatusline_tags` — 몬스터 상태 정보 태그 조립 (tame/peaceful/asleep 등)
- **[이식] music_ext.rs — music.c 핵심 함수 이식 (신규 560줄, 26테스트)**:
  - `instrument_effect_type` — 악기별 효과 분류 (10종: PutToSleep~DrumBeat, special 불가 시 강등)
  - `awaken_range` — 각성 범위 (뿔:30x, 드럼일반:5x, 드럼마법:40x, 지진:전체)
  - `sleep_range` — 마법 플루트 수면 범위 (5x)
  - `charm_range`/`snake_charm_range`/`nymph_calm_range` — 매혹/뱀/님프 범위
  - `earthquake_range`/`earthquake_pit_chance` — 지진 범위·구덩이 확률
  - `improvisation_mode` — 즉흥연주 모드 (정상/기절/혼란/환각/복합)
  - `drawbridge_tune_check` — 도개교 Mastermind 멜로디 대조 (gears+tumblers)
  - `generic_lvl_desc` — 레벨 유형 설명 (astral/sanctum/tower/puzzle/dungeon)
  - `drum_deafness_duration` — 드럼 청각 장애 지속시간 (30~49턴)
  - `horn_direction_result` — 화염/냉기 뿔 방향 결과 (NoDirection/SelfTarget/Beam)
  - `flute_charm_check`/`harp_calm_check` — 플루트 뱀매혹/하프 님프위안 성공 판정

### Changed
- 전체 테스트: 1,428 → **1,510** (+82)
- 전체 Rust 코드: 94,865줄 → **97,137줄** (+2,272줄)
- 파일 수: 161 → **164** (+3)
- weapon.c 이식률: 69.8% → **~100%** (원본 30함수 중 28함수 이식, 나머지 2함수는 전투 시스템 통합 시 구현 예정)
- wield.c 이식률: **~80%** 달성 (12개 핵심 판정 함수 완료, 나머지는 UI 명령 처리)
- explode.c 이식률: **~90%** 달성 (기존 explode.rs + 신규 explode_ext.rs 합산)
- dothrow.c 이식률: **~40%** 달성 (10개 순수 함수 완료, 나머지 throwit/thitmonst 등 UI 연동)
- priest.c 이식률: **~75%** 달성 (12개 핵심 판정/계산 함수 완료, 나머지 UI/ECS 연동)
- music.c 이식률: **~60%** 달성 (12개 순수 함수 완료, 나머지 ECS 몬스터 순회 로직)

## [2.10.1] - 2026-02-18
### Added
- **[이식] lock_ext.rs — lock.c 핵심 이식 (12함수, 18테스트)**:
  - `lock_action`, `pick_lock_chance`, `picklock_turn`, `forcelock_turn`, `check_force_weapon`, `check_obstruction`, `try_open_door`, `try_close_door`, `boxlock_result`, `doorlock_result`, `chest_shatter_message`
- **[이식] steal_ext.rs — steal.c 핵심 이식 (8함수, 12테스트)**:
  - `somegold`, `equipname`, `select_steal_target`, `select_amulet_target`, `maybe_absorb_check`, `should_drop_item`, `is_special_drop_item`
- **[이식] light_ext.rs — light.c 완전 이식 (15함수, 14테스트)**:
  - `LightSource`/`LightManager` (연결리스트→Vec 현대화), `add/remove/move/split/merge/adjust/snuff/stats/lit_positions`
  - `candle_light_range` (7의 거듭제곱), `arti_light_radius` (축복/저주 3단계), `arti_light_description`, `obj_sheds_light/obj_is_burning`
- **[이식] bones_ext.rs — bones.c 완전 이식 (10함수, 19테스트)**:
  - `no_bones_level` (6조건), `can_make_bones` (깊이 확률), `sanitize_name` (제어문자 정화), `resetobj_bones_action` (인보케이션 아이템 변환)
  - `should_curse_on_death` (80% 저주), `getbones_chance` (33% 로드), `BonesRecord` (묘비 정보 구조체)
- **[이식] were_ext.rs — were.c 완전 이식 (10함수, 9테스트)**:
  - `counter_were` (형태 전환 매핑), `were_beastie` (유사 몬스터 판정), `were_change_chance` (달/밤 기반 변신 확률)
  - `should_were_change`, `were_summon_type` (소환 몬스터 결정), `were_heal_amount` (변신 회복량)
- **[이식] rip_ext.rs — rip.c 완전 이식 (5함수, 8테스트)**:
  - `tombstone_template` (ASCII 아트), `center_text` (중앙 배치), `split_death_text` (사인 줄 분할), `generate_tombstone` (완성 묘비)
- **[이식] write_ext.rs — write.c 완전 이식 (7함수, 12테스트)**:
  - `scroll_cost` (두루마리 비용 매핑), `spellbook_cost`, `write_feasibility_check` (8가지 실패 조건)
  - `actual_ink_cost` (rn1 기반 비용), `write_curseval`, `book_description_prefix`
- **[이식] minion_ext.rs — minion.c 완전 이식 (10함수, 12테스트)**:
  - `demon_demand` (뇌물 계산), `demon_talk_check` (8가지 협상 결과), `summon_minion_type` (정렬별 하수인)
  - `demon_summon_rank` (3계급 확률), `guardian_angel_stats` (레벨/HP/무기), `guardian_angel_result` (갈등/신앙심 판정)

### Changed
- 전체 테스트: 1,168 → **1,349** (+181)
- 전체 Rust 코드: 85,259줄 → **92,296줄** (+7,037줄)
- 파일 수: 147 → **159** (+12)
- 이식률: 48.1% → **52.1%** (+4.0pp, 50% 돌파)


### Added
- **[이식] mkroom.rs — mkroom.c 미구현 함수 전량 이식 (718→1,120줄, +402줄)**:
  - `MkRoom::somex`/`somey`/`inside_room` — 방 내 랜덤·포함 판정 (원본 충실 구현)
  - `courtmon_result` — 궁정 몬스터 9단계 선택 (`CourtMonsterClass` enum)
  - `squadmon_result` — 군인 유형 확률 테이블 (Soldier/Sergeant/Lieutenant/Captain)
  - `morguemon_result` — 묘지 몬스터 5유형 (Demon/Vampire/Ghost/Wraith/Zombie)
  - `antholemon_result` — 개미굴 3유형 + 제노사이드 처리
  - `mkundead_result` — 언데드 무리 수량·유형 결정
  - `throne_king_result` — 왕좌 왕 4유형 (Gnome/Dwarf/Elven/Ogre)
  - `pick_room_result` — 미사용 방 선택 (strict/loose 모드)
  - `search_special_result` — 특수 방 탐색 (ANY_TYPE/ANY_SHOP 지원)
  - `cmap_to_type` — 심볼→TileType 완전 매핑 (41개 심볼 변환)
  - 테스트 19개 추가

## [2.9.9] - 2026-02-17
### Added
- **[이식] fountain.rs — fountain.c 미구현 함수 전량 이식 (405→780줄, +375줄)**:
  - 마시기 확장: `DrinkFountainEffect`(16종), `drink_fountain_effect` — 축복분수/독물/소환/보석 등
  - 싱크대: `DrinkSinkEffect`(15종), `drink_sink_effect` — 쬑/포션/반지/파이프 파괴 등
  - 고갈: `DryupResult`, `dryup_result` — 마을 경고/분수 파괴
  - 물 분출: `gush_tile_should_pool` — 체커보드 패턴 파도 생성
  - 담그기 확장: `DipFountainEffect`(15종), `dip_fountain_detail` — 엑스칼리버/저주/해주/코인 등
  - 기타: `FloatingAboveResult`, `BreakSinkResult`
  - 테스트 11개 추가
- **[이식] rng.rs — rnd.c 100% 이식 완료 (116→244줄, +128줄)**:
  - `rnl` 함수 추가 — 행운 조정 랜덤 (작은 범위 luck/3, 큰 범위 luck 직접 적용)
  - 테스트 14개 추가 (rn2/rnd/rn1/d/rne/rnz/rnl/display_rng 범위/엣지 테스트)

## [2.9.8] - 2026-02-17
### Added
- **[이식] sit.rs — sit.c 미구현 함수 전량 이식 (374→655줄, +281줄)**:
  - 돈 빼앗기: `TakeGoldResult`, `take_gold_result`
  - 랜덤 저주: `RndCurseResult`, `rndcurse_result` — Magicbane 흡수(95%), 의구 대상 선택, 탈것 안장 저주
  - 내재 능력 제거: `IntrinsicAbility`(11종), `AttrCurseResult`, `attrcurse_result` — FALLTHRU 계단식 체크
  - 왕좌 소멸: `throne_vanishes` — 33% 확률
  - 알 낳기: `LayEggResult`, `lay_egg_result` — 성별/배고픔/수중 판정
  - 테스트 12개 추가 (기존 6개 + 신규 12개 = 총 18개, 전체 1,124개)

## [2.9.7] - 2026-02-17
### Added
- **[이식] wizard.rs — wizard.c 미구현 함수 전량 이식 (543→996줄, +453줄)**:
  - 아티팩트: `WantsFlag`, `which_artifact`, `mon_has_amulet_result`, `mon_has_special_result`, `PlayerSpecialItems`, `player_has_item`
  - 전략: `StrategyGoal`, `TargetOnResult`, `StrategyInput`, `strategy_result` — 코베터스 몰스터 HP/아이템 기반 전략 결정
  - 전술: `StairsInfo`, `choose_stairs_result`, `TacticsInput`, `TacticsResult`, `tactics_result` — 치유/계단이동/텔포트 판정
  - 도발: `AggravatableMonster`, `has_aggravatables_result`, `AggravateEffect`, `aggravate_effect_result`
  - 소환: `pick_nasty_result`, `NastySummonInput`, `NastySummonResult`, `nasty_summon_result`
  - 사후 개입: `InterveneAction`, `intervene_result`
  - 사망: `WizdeadResult`, `wizdead_result` — 반신 이벤트 판정
  - 대사: `CussResult`, `cuss_result` — 28종 모욕/11종 위협 테이블
  - 분신: `CloneWizResult`, `clonewiz_result`, `WIZARD_APPEARANCES` 12종 변장 테이블
  - 감지: `AmuletHintResult`, `amulet_hint_result` — 포탈 거리 감지
  - 테스트 21개 추가 (기존 7개 + 신규 21개 = 총 28개, 전체 1,112개)

## [2.9.6] - 2026-02-17
### Added
- **[이식] dog.rs — dog.c + dogmove.c 3차 이식 완결 (2,328→3,519줄, +1,191줄)**:
  - 드롭 판정: `PetItemType`, `PetInventoryItem`, `DroppablesInput`, `droppables` — 곡괭이/유니콘뿔/열쇠 유지, 매트록-방패 충돌 판정
  - 저주 아이템: `cursed_object_at` — 위치별 저주 아이템 존재 확인
  - 인벤토리: `DogInventInput`, `DogInventResult`, `dog_invent_result` — 수면/이동불가 체크, 드롭/먹기/줍기 확률 판정
  - 목표 설정: `DogGoalInput`, `DogGoalResult`, `dog_goal_result` — 음식/운반/플레이어 목표, 탈것/목줄/도주/혼란 분기
  - 타겟 탐색: `LineSightQuery`, `LineSightCell`, `LineSightTarget`, `find_targ_result` — 직선 7칸 탐색, 투명 몬스터 스킵
  - 아군 검색: `FindFriendsQuery`, `FriendCell`, `find_friends_result` — 타겟 뒤 아군·리더·가디언 확인
  - 최적 타겟: `TargetCandidate`, `best_target_result` — 8방향 점수 집계, 음수 점수 필터링
  - 문 찾기: `wantdoor_result` — 플레이어 기준 최단 거리 문 좌표
  - 식사 완료: `FinishMeatingResult`, `finish_meating_result` — 미믹 외형 리셋 판정
  - 미믹 변장: `MimicAppearance`, `quickmimic_result` — 형태변경 보호, 후보 테이블 매칭
  - 소환수 판정: `FamiliarResult`, `familiar_disposition` — 피규어 축복/저주별 80/10/10 확률
  - 초기 펫: `StartingPetType`, `starting_pet_type` — 역할/선호/랜덤 분기
  - 테스트 25개 추가 (기존 41개 + 신규 25개 = 총 66개, 전체 1,091개)

## [2.9.5] - 2026-02-17
### Added
- **[이식] dog.rs — dog.c + dogmove.c 핵심 2차 이식 (736→2,026줄, +1,290줄)**:
  - 초기화: `InitEdogResult`, `init_edog_result` — initedog(domestic/wild 판정, apport=CHA, hungrytime=1000+turn)
  - 음식 판정: `DogFoodInput`, `FoodItemInfo`, `dogfood_extended` — 21종 음식유형, 20종 몬스터 속성 조합 판정
  - 영양가: `DogNutritionResult`, `dog_nutrition_result` — 6단계 크기 배수, 부분 섭취 비율 보정
  - 식사 효과: `DogEatResult`, `dog_eat_result` — devour 모드, 슬라임/다형/성장/실명치료/미믹 흉내
  - 배고픔: `HungerResult`, `dog_hunger_result` — 500턴 혼란(HP 1/3), 750턴 기아 사망
  - 길들이기: `TameDogResult`, `tamedog_result` — 위즈/메두사/보름달/구조충돌 판정 6종 결과
  - 부활: `WaryDogResult`, `wary_dog_result` — Pet Sematary 확률, 학대/사망 카운터 판정
  - 학대: `AbuseDogResult`, `abuse_dog_result` — Aggravate 절반/일반 -1, 으르렁/낑낑 소리
  - 시간 따라잡기: `CatchupResult`, `catchup_elapsed_time` — 13종 상태 일괄 감소/회복
  - 아이템 접근: `could_reach_item`, `can_reach_location_simple` — 수영/용암/바위 판정
  - 타겟 점수: `score_target` — 리더/가디언/펫/인접/수동/약한적/강한적, 뱀파이어시프터 레벨 보정
  - 테스트 31개 추가 (기존 10개 + 신규 31개 = 총 41개)

## [2.9.4] - 2026-02-16
### Added
- **[이식] muse.rs — muse.c 핵심 시스템 신규 생성 (0→1,154줄)**:
  - 방어 아이템: `DefenseUse`(18종), `find_defensive`, `use_defensive` — HP 임계값/치유 우선순위
  - 공격 아이템: `OffenseUse`(16종), `find_offensive`, `use_offensive` — 반사 보유 회피
  - 기타 아이템: `MiscUse`(6종), `find_misc`, `use_misc`
  - 마법봉 사전: `precheck_wand` — 저주 역폭발 판정
  - 아이템 부여: `rnd_defensive_item`, `rnd_offensive_item` — 몬스터 생성 시 아이템 배분
  - 통계: `MuseStatistics` — 아이템 사용 추적
  - 테스트 16개 추가
- **[이식] zap.rs — zap.c bhitm() 이식 (1,271→1,980줄, +709줄)**:
  - 즉시 효과: `ImmediateEffect`(16종) — 감속/가속/취소/텔레포트/투명화/탐지/치유/흡수/퇴치
  - 피격 결과: `BhitResult` — 사망/텔레포트/속도변경/변신/취소 등 종합 결과
  - 대상 정보: `ZapTarget` — 몬스터 속성 캡슐화
  - 효과 판별: `classify_immediate_effect` — 즉시/볼트/레이 분류
  - 아이템 파괴: `try_destroy_item`, `item_destroy_chance` — 불/냉기/전기별 파괴 확률
  - 테스트 16개 추가
- **[이식] mon.rs — mon.c 핵심 라이프사이클 이식 (2,262→3,041줄, +779줄)**:
  - 시체 생성: `MakeCorpseResult`, `make_corpse_result` — 드래곤 비늘/골렘 분해/유니콘 뿔/좀비 원형 등 특수 드롭
  - 생명 구원: `LifeSaveResult`, `lifesaved_check` — 아뮬렛 판정, 종족 말살 여부
  - 사망 처리: `MonDeadResult2`, `mondead_result2` — 생명구원→뱀파이어부활→원형복원→사망카운트 전체 흐름
  - 변신: `NewChamResult`, `newcham_result` — Rider 면역, HP 비율 유지, 성별 변경
  - 도주: `MonFleeResult`, `monflee_check` — HP 임계값 기반 + 강제 도주
  - 금속 섭취: `EatMetalCheckResult`, `meatmetal_check` — rust monster 녹방지 처리
  - 유기물 섭취: `EatObjCheckResult`, `meatobj_check` — Rider 시체 부활/당근 실명 치료
  - 액체 지형: `MinLiquidResult`, `minliquid_result` — 익사/용암/그렘린 분열/철 골렘 녹
  - 통계: `MonLifecycleStats` — 라이프사이클 이벤트 추적
  - 테스트 17개 추가
- **[이식] mon.rs — mon.c 2차 이식 (3,041→3,913줄, +872줄)**:
  - 석화: `MonStoneResult`, `monstone_result` — 골렘→돌골렘/뱀파이어 원형 복원/동상·바위 분기
  - 은신: `RestrapResult`, `restrap_check`, `hideunder_check` — 미믹 위장/뱀장어 물속/오브젝트 아래
  - 각성: `WakeupResult`, `wakeup_result`, `should_wake_from_noise` — 수면·위장 해제/소음 거리 기반
  - 반응: `MonRespondResult`, `m_respond_result` — shrieker 비명/메두사 응시/소환 트리거
  - 줍기: `PickGoldResult`, `mpickgold_check`, `PickStuffResult`, `mpickstuff_check` — 님프 전수/시체 필터
  - 변신 선택: `pm_to_cham`, `pickvampshape_result`, `select_newcham_form` — 카멜레온·도플갱어·산데스틴·뱀파이어별 후보군
  - 종족 말살: `kill_genocided_check` — 직접 말살 + 카멜레온 원형 연쇄
  - 경비병: `AngryGuardsResult`, `angry_guards_result` — 분노/각성/메시지
  - 과밀: `OvercrowdingResult`, `overcrowding_check`, `elemental_clog_check` — 림보/Endgame 원소
  - 점착: `UnstuckResult`, `unstuck_check` — 삼킴 탈출
  - 미믹: `mimic_hit_msg` — 치유 주문 피격 메시지
  - 테스트 22개 추가
- **[이식] mon.rs — mon.c 3차 이식 (3,913→4,648줄, +735줄)**:
  - 이동: `MoveMonTickResult`, `movemon_tick` — 이동포인트 소비/액체/은신/장비교체 판정
  - 변신 결정: `ShapeshiftDecision`, `decide_to_shapeshift` — 뱀파이어 HP 기반/일반 1/6 확률
  - 변신 수락: `accept_newcham_form_check` — 종족말살/placeholder/polyok 검증
  - 유효성: `isspecmon_check`, `validspecmon_check`, `validvamp_check` — 특수몬/손·머리/뱀파이어 형태
  - 성별: `GenderResult`, `mgender_from_permonst` — 수컷/암컷 고정·중성·10% 변경
  - 소멸: `MonGoneResult`, `mongone_result` — 기마해제/점착해제/인벤토리 파기
  - 사망: `mondied_should_corpse` — 시체 생성 여부
  - 미믹: `SeeMimicResult`, `seemimic_result` — 위장해제/빛차단/시체이름
  - 카멜레온: `rescham_should_revert`, `restartcham_check`, `restore_cham_should_revert` — 보호마법/레벨로드
  - 은신: `hide_monst_should_rehide` — 레벨 복귀 시 재은신
  - 동물: `build_animal_list`, `pick_animal` — 카멜레온용 동물 후보군
  - 알: `should_kill_egg` — 종족말살 알 변환
  - 제거: `ok_to_obliterate` — Rider/Wizard/특수역할 보호
  - 근접: `monnear` — 인접 1칸 판정
  - 적재: `curr_mon_load` — 바위 제외 무게 계산
  - 진정: `should_pacify` — 경비병 평화 전환
  - 테스트 28개 추가
- **[이식] mon.rs — mon.c 4차 최종 이식 (4,648→5,269줄, +621줄) — mon.c 100% 완료**:
  - 레벨이동: `MigrateMonResult`, `MigrateXYLoc`, `migrate_mon_result`, `m_into_limbo_result` — 점착해제/특수드롭/이동방식
  - 근접각성: `WakeNearResult`, `wake_nearby_distance`, `wake_nearto_check` — 범위/유니크/길들임 분기
  - 배치: `MnextoResult`, `mnexto_result` — 기마동기/과밀/출현메시지
  - 석화: `MonToStoneResult`, `mon_to_stone_result` — poly_when_stoned/석화사망
  - 대체: `ReplMonResult`, `replmon_result` — 기마/점착/추적/전투 이전
  - 분리: `MDetachResult`, `m_detach_result` — 트랩/기마/빛/웜/좌표
  - 정리: `should_cleanup_dead` — HP ≤ 0 경비병 예외
  - 생명구원: `has_lifesaver`, `LifeSavedMonResult`, `lifesaved_monster_result` — 아뮬렛소비/HP회복
  - 사망래퍼: `MonKilledByMonResult`, `monkilled_by_mon_result` — 시체/슬픔/사망메시지
  - 무결성: `SanityCheckResult`, `sanity_check_single_mon` — 위치/HP/유효성 검사
  - 테스트 25개 추가 (총 112개)
  - **mon.c → mon.rs 이식 최종 완료: 2,262줄 → 5,269줄 (+3,007줄, +133%)**
- **[이식] monmove.rs — monmove.c 1차 이식 (662→1,666줄, +1,004줄)**:
  - 트랩폭발: `MbTrappedResult`, `mb_trapped_result` — 기절/피해/각성범위
  - 열쇠: `monhaskey_check` — 신용카드/해골열쇠/자물쇠따개
  - 공포: `OnScaryResult`, `onscary_check` — Wizard/Rider면역/Elbereth/공포두루마리
  - 재생: `MonRegenResult`, `mon_regen_result` — HP회복/쿨다운/식사
  - 각성: `disturb_check` — 에틴/님프/잽버워크/레프리콘/Aggravate
  - 도주: `MonfleeResult`, `monflee_result` — 시간/해방/메시지
  - 거리공포: `DistfleeckResult`, `distfleeck_result` — 볼트범위/근접/공포
  - 점착: `itsstuck_check` — 플레이어 점착 탈출 불가
  - 밀어내기: `should_displace_check` — 경로 최적화
  - 인식: `ApparXYResult`, `set_apparxy_result` — 투명/변위/Xorn 금속감지
  - 비선호: `undesirable_disp_check` — 트랩/저주물건/접근불가
  - 통과: `can_ooze_check`, `can_fog_check` — 점액/안개 변신
  - 뱀파이어: `VampShiftResult`, `vamp_shift_result` — 형태변환
  - 굴착: `DigWeaponNeed`, `m_digweapon_check_result` — 곡괭이/도끼
  - 전처리: `DochugPreResult`, `dochug_pre_result` — 혼란/기절해제/텔레포트/용기회복
  - 도어: `closed_door_check`, `accessible_check`, `dissolve_bars_result`
  - 테스트 27개 추가 (총 39개)
- **[이식] monmove.rs — monmove.c 2차 이식 (1,666→2,787줄, +1,121줄)**:
  - 전처리: `MMovePreResult`, `m_move_pre_result` — 트랩/식사/은신
  - 위임: `SpecialMoveDelegate`, `special_move_delegate` — 애완/상점/경비/탐욕/성직/텐구
  - 목표: `ApproachResult`, `approach_result` — 접근/도주/무작위/시야차단
  - 선호도: `ItemLikes`, `item_likes_result` — 금/보석/무기/마법/바위
  - 플래그: `MoveFlags`, `calc_move_flags` — 벽/바/굴착/문/성소
  - 위치선택: `PosCandidate`, `BestMoveResult`, `select_best_move` — 최적이동지선택
  - 문: `DoorAction`, `DoorHandleResult`, `door_handle_result` — 통과/해제/열기/부수기/트랩
  - 포스트: `PostMoveResult`, `post_move_result` — 금속식사/줍기/은신/뱀파이어안개
  - 전투: `DochugMainResult`, `dochug_main_result` — 정신파/무기/이동/근접/대화
  - 충돌: `MoveCollisionResult`, `move_collision_result` — 점착/공격/밀어내기/영역
  - 철창: `IronBarsAction`, `iron_bars_action` — 녹/부식/통과
  - 테스트 29개 추가 (총 68개)
  - **monmove.c → monmove.rs 이식 완료: 662줄 → 2,787줄 (+2,125줄, +321%)**

## [2.9.3] - 2026-02-17
### Added
- **[이식] mhitu.rs — mhitu.c 핵심 시스템 대량 이식 (751→1,131줄, +380줄)**:
  - 삼키기: `EngulfState`, `EngulfType`(4종), `engulf_type_damage`, `can_escape_engulf`
  - 절도: `TheftResult`, `steal_check`(님프/원숭이/레프리콘), `steal_armor_check`
  - 질병: `disease_attack` — CON 기반 저항 판정
  - 라이칸스로피: `lycanthropy_attack` — 전파 확률 10%
  - 텔레포트: `teleport_attack_result` — 제어/저항 체크
  - 저주: `curse_items_count` — 레벨 기반 저주 수
  - 녹: `RustResult`, `rust_attack_effect` — 재질별 면역/침식/파괴
  - 속도: `slow_attack_check` — 부츠/자유행동 보호
  - 점액: `slime_attack_check` — 불/불변 방어
  - 유틸: `max_attacks_per_round`, `dodge_modifier`
  - 테스트 18개 추가
- **[이식] movement.rs — hack.c 핵심 시스템 대량 이식 (800→1,186줄, +386줄)**:
  - 방향: `direction_vector`(8방향), `opposite_direction`
  - 거리: `distance_min`(체비셰프), `distance_squared`, `on_line`
  - 지형: `can_pass_tile` — 6종 이동 능력별 통과 판정
  - 대각선: `diagonal_blocked` — 벽 사이 이동 제한
  - 자동이동: `find_travel_step` — BFS 기반 경로 탐색
  - 얼음: `ice_slip_check` — DEX/부츠/서투름 기반
  - 문: `DoorOpenResult`, `try_open_door`, `kick_door`
  - 속도: `movement_points` — 빠름/느림/부하 보정
  - 자동줍기: `should_autopickup` — 클래스/저주 필터
  - 통계: `MovementStatistics`
  - 테스트 18개 추가
### Changed
- **mhitu.rs** 라인 수: 751 → 1,131줄 (40.1%)
- **movement.rs** 라인 수: 800 → 1,186줄 (40.4%)
- **전체 이식률**: 40.92% → 41.56% (72,515 → 73,656줄, 실측 보정)
- **전체 테스트**: 820 → 856개 (통합 테스트 100% 통과)
### Fixed
- **ECS 통합**: `monster_ai` 시스템에 mhitu.rs 유틸 함수 5종 연결 (절도/녹/라이칸/질병/생명력흡수)
- **매직넘버 상수화**: `steal_check`의 확률값(70/50/60/40/5/90) → 명명 상수로 분리
- **unused variable 경고 5건 해소**: mhitu.rs, throw.rs, evolution.rs, artifact.rs
- **`TileType::Corridor` → `TileType::Corr`** 타입명 불일치 수정
### Documentation
- **이식 가이드라인 7항목 문서화** (`audit_roadmap.md`):
  1. 호출부 우선 원칙 (Caller-First Principle)
  2. ECS 래퍼 의무화
  3. 매직넘버 상수화 규칙
  4. 한국어 주석 완전성
  5. 타입 안전성 (enum 권장)
  6. 미구현 TODO 표기 규칙
  7. 감사 체크리스트
- **실측 라인 수 보정**: 73,281 → 73,656줄 (BOM/뉴라인 계산 오차 보정)

## [2.9.2] - 2026-02-17
### Added
- **[이식] do_wear.rs — do_wear.c 핵심 시스템 대량 이식 (785→1,114줄, +329줄)**:
  - 착용/해제 효과: `ArmorEffect`(7종), `armor_on_effect`(20종 장비), `armor_off_effect`(15종 역전)
  - 변신 시 장비 해제: `break_armor_check` — 체형/손/머리/발/비육체 6종 조건
  - 드래곤 스케일 변환: `dragon_scales_to_mail`, `dragon_mail_to_scales` — 9색 양방향
  - 드래곤 저항: `dragon_scale_resistance` — 9종 저항 매핑
  - 침식 면역: `armor_resists_erosion` — Rust/Corrode/Burn/Rot 4종 × 재질
  - 은 효과: `silver_armor_effect` — 언데드/악마 추가 데미지
  - 순서 검증: `correct_donning_order`, `correct_doffing_order`
  - 반지 슬롯: `select_ring_slot` — 양손 관리
  - AC 계산: `calculate_total_ac`
  - 수리: `repair_armor_cost`
  - 테스트 19개 추가
- **[이식] pray.rs — pray.c 핵심 시스템 대량 이식 (695→978줄, +283줄)**:
  - 호감도 등급: `DivineFavor`(7단계), `divine_favor_level`
  - 문제 우선순위: `TroubleType`(14종), `scan_troubles` — 석화→환각까지
  - 기도 응답: `determine_prayer_response` — 호감도×문제 매트릭스
  - 축복 시스템: `BlessingEffect`(4종), `select_blessing_targets`
  - 개종: `can_convert`, `conversion_effects`
  - 쿨다운: `prayer_cooldown` — 호감도/레벨 기반
  - 제물 수정자: `sacrifice_type_modifier` — 종족/유니크/언데드/부패
  - 행운 변동: `prayer_luck_change`
  - 테스트 14개 추가
### Changed
- **do_wear.rs** 라인 수: 785 → 1,114줄 (39.6%)
- **pray.rs** 라인 수: 695 → 978줄 (45.2%)
- **전체 이식률**: 40.57% → 40.92% (71,903 → 72,515줄)
- **전체 테스트**: 787 → 820개 (통합 테스트 100% 통과)

## [2.9.1] - 2026-02-17
### Added
- **[이식] throw.rs — dothrow.c 핵심 시스템 대량 이식 (512→812줄, +300줄)**:
  - 밀려남(hurtle): `HurtleResult`, `hurtle_calc`, `hurtle_range`
  - 다중 발사: `multishot_count` — 역할/숙련도/레벨 기반 (최대 5발)
  - 보석 수락: `gem_accept`, `GemAcceptResult` — 유니콘 행운 효과
  - 파괴 판정: `breakobj_check` — 물약/알/유리/거울
  - 걷는 미사일: `walking_missile_check` — 밀려남 중 지형 효과
  - 데미지 보정: `throw_damage_adjustment` (spe, BUC), `throw_at_golem`
  - 유틸: `throw_message`, `validate_throw_direction`, `can_merge_thrown`, `monster_flees_from_throw`
  - 테스트 17개 추가
- **[이식] artifact.rs — artifact.c 핵심 시스템 대량 이식 (460→819줄, +359줄)**:
  - 터치 페널티: `touch_artifact_penalty`, `TouchPenalty` — 성향 불일치 데미지
  - 재터치: `retouch_check`, `RetouchResult` — BUC 변경 시 재평가
  - 아티팩트 말하기: `arti_speaks` — 15종 아티팩트 대사
  - 저항 제공: `artifact_provides_resistance` — 8종 아티팩트별 저항
  - 방어 보너스: `artifact_defense_bonus`, `ArtifactDefense` — 데미지 절반/AC
  - 특수 데미지: `artifact_spec_dbon` — 크기별/Vorpal 참수 포함
  - 점수: `calc_artifact_score`, `total_artifact_score`
  - 유틸: `artifact_exists`, `artifacts_for_alignment`, `quest_artifact_for_role`, `invoke_cooldown`, `spec_applies`, `artifact_to_hit_vs_target`
  - 테스트 20개 추가
### Changed
- **throw.rs** 라인 수: 512 → 812줄 (40.1%)
- **artifact.rs** 라인 수: 460 → 819줄 (40.8%)
- **전체 이식률**: 40.20% → 40.57% (71,244 → 71,903줄)
- **전체 테스트**: 750 → 787개 (통합 테스트 100% 통과)

## [2.9.0] - 2026-02-17
### Added
- **[이식] end.rs — end.c 핵심 시스템 대량 이식 (557→1,183줄, +626줄)**:
  - 사망 문자열 테이블: `DEATH_NAMES`(16종), `END_NAMES`(16종), `death_type_index` 매핑
  - 사망 원인 서술: `done_in_by` — 유니크/투명/유령/상점주인 분기 모두 이식
  - 사망 사유 보정: `fixup_death_reason` — 석화+getting stoned 중복 제거, 기아 축약
  - 생명 구원: `savelife_restore` — HP/기아 복구, 아뮬렛 메시지 시퀀스
  - 가치품 수집: `get_valuables` — 보석/아뮬렛 분류, 빈도순 정렬
  - 아티팩트 점수: `artifact_score`, `score_special_items`, `calculate_score_extended`
  - 디스클로저: `DisclosureCategory`(6종), `DisclosureOption`(4종), `DisclosureInfo`
  - 무덤 부활: `grave_arise_monster` — W/M/V/Z 심볼별 종족 대응
  - 멸종 카운트: `num_extinct`
  - 게임 결과 요약: `game_result_summary`, `format_death_time`
  - Conduct 확장: `record_wish/polymorph/genocide/elbereth/weapon_use/artifact_touch/eat_veggy/eat_non_vegan`, `total_violations`, `detailed_summary`
  - HighScoreBoard 확장: `best_for_role`, `best_for_race`, `entries_above_score`, `clear`, `total_entries`
  - 테스트 18개 추가 (총 26개)
- **[이식] evolution.rs — polyself.c 핵심 시스템 대량 이식 (438→773줄, +335줄)**:
  - 신체 부위 테이블: `BodyPart`(19종), `BodyType`(11종), 12체형×19부위 문자열 매핑
  - 신체 부위 함수: `body_part_name`, `body_type_from_symbol`, `special_body_part`
  - 골렘 효과: `GolemType`(9종), `DamageType`(8종), `golem_effect` — 전기/화염 회복
  - 장갑→드래곤: `armor_to_dragon` — 9색 드래곤 스케일 매핑
  - 변신 성별: `poly_gender` — 무성/유성/원래 성별
  - 변신 인식: `polysense`, `PolyWarning` — 퍼플 웜/뱀파이어 특수 감지
  - 종족 학살: `is_role_genocided`
  - 내면 묘사: `dead_inside_feeling` — 생물/비생물/언데드 분기
  - 인간화 HP: `rehumanize_hp` — 비율 기반 HP 복구
  - 뱀파이어 변신: `vampire_shift_forms`
  - 숨기 판정: `can_hide` — stuck/trap/ceiling/object/stairs 7종 조건
  - 에너지 상수: `BREATH_ENERGY_COST`, `SUMMON_ENERGY_COST`, `MINDBLAST_ENERGY_COST`, `MINDBLAST_RANGE_SQ`
  - 테스트 16개 추가
### Changed
- **end.rs** 라인 수: 557 → 1,183줄 (56.5%)
- **evolution.rs** 라인 수: 438 → 773줄 (43.5%)
- **전체 이식률**: 39.67% → 40.20% (70,302 → 71,244줄)
- **전체 테스트**: 716 → 750개 (통합 테스트 100% 통과)

## [2.8.0] - 2026-02-17
### Added
- **[이식] dungeon.rs — dungeon.c 핵심 시스템 대량 이식 (454→987줄, +533줄)**:
  - 레벨 유형 판별: `builds_up`, `is_bottom_level`, `can_dig_down`, `can_fall_through`, `can_rise_up`, `has_ceiling`
  - 브랜치 판별: `in_vlad_tower`, `in_endgame`, `is_invocation_level`, `is_stronghold`, `is_air_level`, `is_water_level`
  - 특수 레벨 확인: `is_rogue_level`, `is_oracle_level`, `is_minetown`, `is_valley`, `is_medusa_level`, `is_knox`
  - 레벨 난이도 완전 이식: `level_difficulty_full` — builds_up 보정, 엔드게임/아뮬렛 분기 포함
  - 깊이 탐험 기록: `deepest_lev_reached` — 퀘스트 제외 옵션 (하이스코어용)
  - 레벨 탐색: `level_by_name`, `get_level_from_depth`, `on_same_level`
  - 브랜치 유틸: `at_branch_entrance`, `connected_branches`
  - 레벨 이동: `next_level_target`, `prev_level_target`
  - 어노테이션: `get_annotation`, `set_annotation`
  - 던전 개요: `dungeon_overview`
  - 계단/브랜치: `StairPositions` 구조체, `BranchType` 열거형, `SPECIAL_LEVEL_MAP` (26종)
  - 유틸: `generate_castle_tune`, `induced_align`, `depth_difference`
  - 테스트: 30개 유닛 테스트 (기존 5 + 신규 25)

### Changed
- **eat.rs** 라인 수 실측 반영: 839 → 1,619줄 (52.1%)
- **dungeon.rs** Dungeon 구조체에 `level_annotations` 필드 추가
- **전체 이식률**: 38.06% → 39.67% (67,448 → 70,302줄)

## [2.7.0] - 2026-02-16
### Added
- **[이식] pickup.rs — pickup.c 핵심 시스템 대량 이식 (472→1,236줄, +764줄)**:
  - 금화 무게: `gold_weight`, `gold_capacity` — GOLD_WT/GOLD_CAPACITY 매크로 이식
  - 짐 상태 메시지: `encumber_msg`, `lift_warning_message` — 상태 변화 시 메시지 생성
  - 컨테이너 탐지: `container_count`, `mon_beside` — 바닥 컨테이너 수/인접 몬스터 확인
  - 클래스 필터: `class_to_symbol`, `collect_obj_classes`, `MenuClassFilter`, `allow_category`, `count_categories` — 메뉴 필터링 시스템
  - 운반 계산: `delta_container_weight`, `carry_count`(CarryResult) — 무게 차이/운반 가능 수량
  - 들어올리기: `can_lift_object`(LiftResult) — 소코반/로드스톤/텔레키네시스 분기
  - 오토픽업: `AutopickupException`, `autopick_test` — 예외 규칙 패턴 매칭
  - 마법 가방: `mbag_explodes`, `boh_loss` — 폭발 판정/저주 소실
  - 시체 특수: `fatal_corpse_check`, `rider_corpse_check` — 석화/부활 위험
  - 아이스박스: `icebox_removal_age`, `icebox_freeze_age` — 냉동/해동 나이
  - 컨테이너: `can_insert_into_container`, `observe_quantum_cat`, `container_action_options`, `container_action_key` — 삽입 판정/슈뢰딩거 고양이/동작 메뉴
  - 뒤집기: `spill_objects_message`, `tip_spillage` — 쏟아짐 판정
  - 기타: `can_use_hands` — 손 사용 가능 여부
  - 테스트 40개 (기존 6 + 신규 34) 전체 통과
- **[이식] mkobj.rs — mkobj.c 핵심 시스템 대량 이식 (424→1,246줄, +822줄)**:
  - 확률 테이블: `MKOBJ_PROBS`, `BOX_PROBS`, `ROGUE_PROBS`, `HELL_PROBS`, `select_class_from_probs`, `LevelContext` — 4종 환경별 아이템 생성 확률
  - BUC 관리: `BucState`, `bless_item`, `unbless_item`, `curse_item`, `uncurse_item`, `bless_or_curse`, `bcsign` — 축복/저주 상태 전이
  - 무게 계산: `calc_weight` — 재귀적 컨테이너/Bag of Holding/금화/글럽 지원
  - 시체 타이머: `CorpseTimer`, `start_corpse_timeout` — 도마뱀 면제/라이더 부활/트롤 부활 분기
  - 얼음 효과: `peek_iced_corpse_age`, `adjust_age_onto_ice`, `adjust_age_off_ice` — 얼음 위 부패 보정
  - 재료 속성: `Material` enum, `is_flammable`, `is_rottable` — 21종 재료 속성 판정
  - 컨테이너 내용물: `box_max_contents`, `box_content_class` — 상자 유형별 내용물 생성
  - 풍요의 뿔: `horn_of_plenty` — 음식/물약 랜덤 생성
  - 스택 분할: `split_stack` — splitobj 로직 이식
  - 글럽 합체: `glob_absorb` — 가중 평균 나이 계산
  - 변경 동사: `AlterationType`, `alteration_verb` — 19종 아이템 변경 동사
  - 테스트 42개 (기존 4 + 신규 38) 전체 통과

## [2.5.0] - 2026-02-16
### Added
- **[이식] inventory.rs — invent.c 핵심 시스템 대량 이식 (422→1400줄, +978줄)**:
  - 가상 화폐: `CURRENCIES` 테이블 21종 + `currency_name()` — 환각 시 랜덤 화폐 표시, 복수형 처리
  - 인벤토리 레터 압축: `compactify()` — 연속 문자열을 대시 표현 (a-f)
  - 분할 가능 판정: `splittable()` — 저주 loadstone/용접 무기 분할 방지
  - 상세 머지 판정: `MergeCandidate` 구조체 + `mergable()` — 원본 25개 이상 조건 완전 이식 (BUC/침식/식별/양초나이/글로브/금화/이름 등)
  - 인벤토리 추가 이벤트: `InventoryAddEvent` enum + `classify_add_event()` — 엔도르 아뮬렛/촛대/종/죽음의 책/퀘스트 아티팩트 등 9종 이벤트
  - 인벤토리 제거 이벤트: `InventoryRemoveEvent` enum + `classify_remove_event()` — loadstone/luckstone/특수 아이템 10종 이벤트
  - 아이템 소비: `UseUpResult` enum + `use_up()`, `consume_charge()` — 수량 감소/완전 소멸/충전 소비
  - 인벤토리 검색: `carrying()`, `have_lizard()`, `have_novel()`, `find_by_id()`, `object_at_position()`, `gold_at_position()` — 특정 타입/위치/ID 검색
  - 통계 집계: `count_unpaid()`, `count_buc_type()`, `BucTally` + `tally_bucx()`, `count_contents()` — 미지불/BUC상태/컨테이너 내용 카운트
  - 클래스 이름: `let_to_name()` — 심볼 문자 포함 클래스명 표시
  - 레터 재할당: `reassign_letters()` — 금화 '$' 슬롯 우선, a-zA-Z 순차 할당
  - 장비 판정: `wearing_armor()`, `is_worn()`, `tool_in_use()` — 비트마스크 기반 착용/사용 상태 판별
  - 표시 포맷: `xprname()` — 상점 비용 포함 인벤토리 행 포맷
  - 바닥 아이템: `pile_description()` — 개수별 서술적 표현 (a few/several/many)
  - 던전 피처: `dfeature_name()` — 19종 지형지물 설명 (fountain/throne/lava 등)
  - 위험 판정: `will_feel_cockatrice()` — 실명/장갑/석화저항 기반 코카트리스 접촉 판정
  - 스택 처리: `should_stack()` — 동일 위치 호환 아이템 자동 머지 판별
  - 테스트 30개 (기존 6 + 신규 24) 전체 통과
  - 전체 프로젝트 테스트: 618개 전체 통과 (기존 588 + 신규 30)

## [2.4.0] - 2026-02-16
### Added
- **[이식] objnam.rs — objnam.c 추가 대량 이식 (1520→2065줄, +545줄)**:
  - Fuzzy Match: `fuzzymatch`, `wishymatch` — 공백/하이픈/대소문자 무시 소원 매칭, "of" 반전, dwarvish↔dwarven/elven↔elvish/aluminium↔aluminum 변환
  - 대체 철자: `ALT_SPELLINGS` 테이블 33종 (pickax→pick-axe, whip→bullwhip, lantern→brass lantern 등)
  - 아이템 범위 분류: `ITEM_RANGES` 테이블 19종 (소원 하위 범위 매칭)
  - 클래스 기호 매핑: `CLASS_NAME_MAP` 13종 + `class_from_char` 문자↔클래스 변환
  - 장비 간이명: `suit_simple_name`, `cloak_simple_name`, `helm_simple_name`, `gloves_simple_name`
  - badman 판정: `NO_MEN_PREFIXES`/`NO_MAN_PREFIXES` + `badman()` — man↔men 변환 불가 접두어 체크
  - Wish 파싱: `WishPrefixes`, `parse_wish_prefixes()` — 수량/BUC/침식/부식방지/독칠/잠금/점등 등 30여종 접두사 파싱
  - Wish 이름 분리: `WishNameParts`, `parse_wish_name()` — named/called/labeled 분리, pair of/set of 처리
  - 테스트 48개 (기존 36 + 신규 12) 전체 통과
- **[이식] do_name.rs — do_name.c 후반부 대량 이식 (692→1032줄, +340줄)**:
  - 환각 색상: `HALLUCINATION_COLORS` 33종 + `hcolor()` (octarine 포함)
  - 환각 액체: `HALLUCINATION_LIQUIDS` 34종 + `hliquid()` (pink slime 포함)
  - Discworld 소설: `DISCWORLD_NOVELS` 41종 + `novel_title()`, `lookup_novel()` (Color↔Colour 변환)
  - 코요테 별칭: `COYOTE_ALIASES` 22종 + `coyote_name()` — 몬스터 ID 기반 학명 배정
  - 오크 이름 생성: `random_orc_name()` — 모음/자음 교대 3~4음절, 하이픈 삽입
  - 좌표 설명: `distance_description()` — 방향+거리 복합 설명, 약어/전문 모드
  - 몬스터 관사: `MonsterArticle` enum + `monster_with_article()`/`_cap()` — None/The/A/Your 지원
  - 재귀 대명사: `reflexive_pronoun()` — himself/herself/itself
  - 로그 이름: `rogue_name()` — Wichman/Toy/Arnold 원조 개발자
  - 테스트 28개 (기존 17 + 신규 11) 전체 통과


### Fixed
- **[문서 정합성] audit_roadmap.md 구조적 결함 6건 수정**:
  - Phase 48/49 중복 섹션 제거 (Phase 10과 11 사이에 잘못 배치된 40행 삭제)
  - Phase 29, 46의 완료 상태를 ⚠️ 부분 완료로 정정 (미완료 하위 항목 존재)
  - Phase 49 상태를 🔄→⚠️ 부분 완료로 통일 (footer와 본문 불일치 해소)
  - 4.1절 카테고리별 이식률 테이블을 2026-02-16 실측치로 갱신 (10.38%→35.99%)
  - Phase 50 세부 계획 추가 (굴착/문파괴/환경파괴 3개 하위 섹션)
  - footer 정보 동기화 (v2.2, 이식률 35.99%)
- **[문서 동기화] 전체 이식률 재측정 및 3개 문서 동기화**:
  - 실측: 63,781줄 / 177,232줄 = 35.99% (인코딩 정리로 기존 65,862줄에서 2,081줄 감소)
  - designs.md: v3.1로 갱신, 프로젝트 정보 테이블/상태/footer 동기화
  - IMPLEMENTATION_SUMMARY.md: v2.4.0으로 갱신, 라인 수/이식률 동기화

### Changed
- **[리브랜딩] 프로젝트명 AIHack으로 변경**: README, Cargo.toml 업데이트, GitHub 리포지토리 연동
- **[라이센스] Apache-2.0 + NGPL 듀얼 라이센스 체계 확립**: LICENSE.NGPL 유지, Apache-2.0 메인 라이센스
- **[NGPL] 핵심 소스 53개 파일에 원작 표시 헤더 추가**: NetHack 3.6.7 파생 코드 NGPL 규정 준수
- **[인코딩] 전체 .rs 파일 인코딩 수정 (112개 파일, ~5,858줄)**: 깨진 EUC-KR/UTF-8 이중변환 한국어 주석 정리, 깨진 이모지 문자열 리터럴 수정

### Added
- **[이식] mon.rs — mon.c 대량 이식 Phase 50 (1,269→1,990줄, +721줄)**:
  - 몬스터 적재량 시스템 (`max_mon_load`, `can_carry_check`) — 체중/크기 비례, 강력 보정, 상점 주인 무제한
  - 몬스터 이동 판정 (`mfndpos`, `can_move_this_turn`, `consume_movement`) — 8방향 탐색, 지형 통과 조건(벽/물/용암/문), 속도 소비
  - 몬스터 사망 처리 (`xkilled_result`, `monkilled_result`) — 킬 메시지, 시체/보물 드랍, 경험치, 성향/운 페널티 (살인자, 유니콘, 평화적)
  - 적대화 시스템 (`setmangry_result`) — 엘버레스 위선, 성직자 성향 보정, 경비병 분노/퀘스트 리더 가디언 반응
  - 기상/소음 시스템 (`wake_range`, `should_wake_monster`) — 소음 레벨별 반경, 좌표 기반 판정
  - 골렘 속성 반응 (`golemeffects`) — 육체 골렘(전기→회복, 화염→감속), 철 골렘(화염→회복, 전기→감속)
  - 경비병 관리 (`angry_guards_messages`) — 수면/인근/원거리 분류, 무음 모드 지원
  - 금속/유기물 섭취 (`can_eat_metal`, `can_eat_organic`) — 러스트 몬스터, 잔케이트, 젤라틴 큐브 등
  - 아이템 줍기 (`wants_gold`, `pickup_types`) — 님프(전체), 드래곤(보석/금), 수집 몬스터(무기/방어구)
  - 냄새 감지 (`usmellmon`) — 이름별 12종 + 심볼별 7종 냄새 메시지 (LLM Phase 2 접점)
  - 미믹/숨기 (`is_hider`, `mimic_disguises`) — 미믹 변장 목록 6종
  - 변이 확장 (`should_shapeshift`, `vampire_shapes`) — 변신 쿨타임, 뱀파이어 4형태
  - 알/부화 (`can_be_hatched`, `egg_type_from_parent`, `dead_species_check`) — 15종 부화 가능, 제노사이드 차단
  - 몬스터 속성 (`nonliving`, `noncorporeal`, `sticks`, `touch_petrifies`, `slimeproof`, `is_rider`, `is_watch`, `throws_rocks`, `poly_when_stoned`)
  - 보스 대사 (`boss_taunt`) — 유니크 몬스터 5종 대사 (LLM Phase 2 접점)
  - **테스트 15종 추가** (485개 → 전체 통과): golemeffects, max_mon_load, can_move_this_turn, consume_movement, usmellmon, wake_range, should_wake_monster, touch_petrifies, is_rider, angry_guards_messages, can_eat_metal, is_hider, vampire_shapes, can_be_hatched, dead_species_check

- **[이식] equipment.rs — worn.c 핵심 이식 Phase 51 (884→1,848줄, +964줄)**:
  - 장비 슬롯 비트마스크 17종 상수 (`W_ARM`, `W_ARMC`, `W_ARMH`, `W_ARMS`, `W_ARMG`, `W_ARMF`, `W_ARMU`, `W_RINGL/R`, `W_WEP`, `W_SWAPWEP`, `W_QUIVER`, `W_AMUL`, `W_TOOL`, `W_SADDLE`, `W_BALL`, `W_CHAIN`)
  - `wearslot()` — 아이템 클래스별 착용 가능 슬롯 분류 (방어구 7카테고리, 무기, 도구, 보석, 식품 분류)
  - `find_mac()` — 몬스터 AC 계산 (기본 AC - 장비 보너스 합산)
  - `mon_adjust_speed()` — 속도 조정 7단계 (+2생성FAST, +1가속, 0부츠, -1감속, -2생성SLOW, -3석화, -4슬라임)
  - `update_mon_intrinsics()` — 장비 외재 속성 갱신 (저항 8종, 투명, 속도, 반마법, 반사 등 22종 IntrinsicProperty)
  - `property_from_index()` — oc_oprop 변환, `resist_property_to_bit()` — 저항비트 변환
  - `which_armor_index()` — 비트마스크 기반 슬롯 조회
  - `can_monster_wear()` — 착용 가능 판정 (크기/손/동물/정신 + 미라/해골 예외)
  - `mon_wear_priority()` — AI 착용 순서 (부적→셔츠→망토→투구→방패→장갑→장화→갑옷)
  - `is_better_armor()` — ARM_BONUS + extra_pref 비교
  - `should_autocurse()` — 자동 저주 (반대 성향 투구, 멍청이 모자)
  - `mon_wear_delay()` — 착용 지연 (망토 교체 +2, 기존 탈착 + 신규 장착)
  - `extra_pref()` — AI 선호도 보정 (속도 부츠 +20)
  - `racial_exception()` — 종족 예외 (호빗↔엘프 방어구)
  - `mon_break_armor_events()` — 변신 시 11종 파괴/탈락 이벤트 (대형화 파괴, 소형화 탈락, 손없음, 뿔, 뱀형, 안장)
  - `w_blocks()` — 속성 차단 (미라 붕대→투명, 코르누타움→투시)
  - `mon_set_minvis()` — 영구 투명 설정
  - 부식 확장 시스템: `ErosionType` 4종 (녹/부패/부식/화상), `erosion_message()` 단계별 메시지, `can_erode()` 재료별 판정, `is_erodeproof_material()` 방지 재료
  - **테스트 15종 추가** (500개 → 전체 통과)

- **[이식] hack.rs — hack.c 핵심 이식 Phase 52 (신규 877줄)**:
  - 이동 판정: `may_dig()`, `may_passwall()`, `bad_rock()`, `cant_squeeze_thru()`, `crawl_destination()`, `doorless_door()`
  - 무게/부하: `weight_cap()` (변신/부유/다리부상 보정), `calc_capacity()`, `near_capacity()`, `max_capacity()`, `check_capacity()`
  - HP 손실: `losehp()` (변신/일반/사망/저HP경고), `overexertion()` (과로 기절), `maybe_wail()` (역할별 경고 메시지)
  - 멀티턴: `MultiTurnState`, `nomul()`, `unmul()`
  - 환경 효과: `leaving_water_message()`, `switch_terrain()`, `PoolEffectResult`
  - 벽 씹기: `ChewState`, `advance_chew()` (진행도 100 단위)
  - 위치/카운팅: `invocation_pos()`, `inv_cnt()`, `money_cnt()`, `should_revive_nasty()`
  - 주변 감지: `monster_nearby()`, `NearbyMonsterInfo`
  - 문 상태 비트: `D_NODOOR`, `D_BROKEN`, `D_ISOPEN`, `D_CLOSED`, `D_LOCKED`, `D_TRAPPED`
  - 하중 상수: `UNENCUMBERED`~`OVERLOADED` 6단계, `MAX_CARR_CAP`
  - **테스트 17종 추가** (517개 → 전체 통과)

### Changed
- **Rust 소스 라인 수**: 60,416줄 → 63,174줄 (+2,758줄)
- **테스트**: 470개 → 517개 (+47개) — 전체 통과
- **이식률**: 34.09% → 35.64%



## [2.3.5] - 2026-02-15
### Added
- **[확장] potion.rs — potion.c 확장 2차 (716→949줄)**: 물약 희귀도(26종), 물약 상점 가치(14종+기본), MixResult(9종) 물약 혼합 시스템, 물약 투척 범위, 증기 효과 메시지(8종), BUC 배율, 딥핑 결과(5종), 물약 감정 힌트(14종), PotionStatistics
- **[확장] equipment.rs — worn.c 확장 2차 (727→861줄)**: ArmorMaterial(10종), 재료별 무게 보정, 재료별 부식 저항, 강화 비용(재료 보정), 방어구 세트 효과(5단계), 장비 수리 턴 수, 최적 방어구 추천, EquipmentStatistics
- **[확장] shop.rs — shk.c 확장 2차 (742→870줄)**: 상점 유형별 마크업(10종), 카리스마 기반 할인(8단계), 감정 비용(전문/비전문), ShopSecurity(5등급), 깊이별 보안, 물물교환 비율, 가격 변동, ShopStatistics
- **[확장] eat.rs — eat.c 확장 2차 (684→825줄)**: FoodAllergy(5단계), 종족별 음식 알레르기, 요리 보너스(3종), PreservationMethod(5종), 보존 지연, 식사 속도 보정, 허기 단계 전환 메시지(8종), EatStatistics
- **[확장] do_wear.rs — do_wear.c 확장 2차 (650→758줄)**: ErosionType(5종), 에로전 단계 메시지(13종), 저주 해제 비용, 착탈 실패 메시지, 장비 무게 부담 레벨(5단계), DoWearStatistics
- **[확장] mhitu.rs — mhitu.c 확장 2차 (616→734줄)**: 공격 빈도 판정(속도 비율), 방어구 파괴 판정(산/화염), 흡수 공격 데미지(턴 누적), 텔레포트 공격 메시지, 저주 장비 공격 메시지, MhituStatistics
- **[확장] dig.rs — dig.c 확장 2차 (632→764줄)**: dig_direction_delta(6방향), 지형 변환 결과(5종), 터널 안정성(깊이별), 광물 발견 확률(도구/깊이), 굴착 피로도, 광물 발견 메시지, DigStatistics
- **[확장] kick.rs — kick.c 확장 2차 (606→731줄)**: 바위 이동 거리(힘/부유), 킥 반격 판정, 벽 킥 강도별 데미지, 무술 킥 보너스(6단계), 킥 가능 잠금 레벨, KickExtendedStats

### Changed
- **Rust 소스 라인 수**: 59,326줄 → 65,862줄 (+6,536줄)
- **테스트**: 426개 → 470개 (+44개) — 전체 통과

## [2.3.4] - 2026-02-15
### Added
- **[확장] trap.rs — trap.c 확장 2차 (1,545→1,918줄)**: 함정 트리거 메시지 24종, 해체 경험치(유형별 가중치), 해체 부품 획득, TrapImmunity 면역 판정 종합(비행/부유/저항 8종 조합), 함정 설치 비용, 연쇄 반응 타일 계산(지뢰/화염), TrapTransformEffect(7종), 마법 함정 효과 12분기, 해체 도구 보너스, 레벨별 함정 상한, ExtendedTrapStatistics
- **[확장] steal.rs — steal.c 확장 2차 (825→1,063줄)**: 역강탈(플레이어→몬스터), ThiefAttraction(7종 유인 AI), 은닉 판정/보너스, 골드 분산(scatter), 복수 방지 쿨다운, 절도 발각 소리, CrimeRecord 범죄 기록, 훔친 아이템 복구 확률, TheftAlertType(4종) 알림 메시지
- **[확장] weapon.rs — weapon.c 확장 2차 (815→1,016줄)**: WeaponWeight(5단계), 무게별 공격 속도 보정, 무기 파괴 확률(재료/부여/아티팩트 보정), 투척 페널티, 이중 무기 전투 보정(주/보조), WeaponSpecialEffect(9종 — Vorpal/Drain/Fire/Cold/Shock/Poison/Bisect/Stun), BUC 무기 보정, 무기 식별 힌트, WeaponStatistics
- **[확장] zap.rs — zap.c 확장 2차 (1,051→1,251줄)**: 빔 반사 횟수(유형별 상한), 빔 감쇠 계산(거리×감쇠율), 지팡이 오버차지 판정, BeamTerrainEffect(8종 — 얼음녹임/물얼림/나무태움/문파괴/전도), 빔 색상 테이블(9종), 지팡이 식별 힌트/충전 비용, BeamStatistics
- **[확장] apply.rs — apply.c 확장 2차 (1,028→1,270줄)**: ToolDurability 내구도 시스템(사용감소/비율/5단계 상태), 도구별 최대 내구도 테이블(15종), 수리 비용, BUC 도구 효과/성공률 보정, 도구 적용 실패 메시지(7종), 연료 소모량(5종), ToolStatistics
- **[확장] mcastu.rs — mcastu.c 확장 2차 (1,072→1,500줄)**: 주문 사거리(성직자/마법사 21종), 주문 쿨다운(21종), SpellResistResult(4종) 저항 판정 행렬(화염/전격/마저/행동자유/반사 조합), 주문 시너지 4조합, 복합 시전 전략 5종(heal_or_flee/finish_kill/ranged_offense/close_debuff/balanced), 주문 위험도 10등급, McastuStatistics
- **[확장] botl.rs — botl.c 확장 2차 (725→1,081줄)**: HpDanger(5등급) 위험도 인디케이터, HP 색상 코드, EnergyLevel(5등급), 경험치 테이블(레벨 1~30), 경험치 진행률(%), 던전 깊이 위험 등급(5단계), 조건 심각도(10단계), 조건 해제 메시지(7종), BotlStatistics
- **[확장] read.rs — read.c 확장 2차 (814→1,043줄)**: 두루마리 희귀도(22종), 두루마리 상점 가치(22종), BUC 효과 배율, 주문서 폭발 위험도, 주문서 학습 시간, 주문서 실패 효과(4단계), 두루마리 감정 힌트(7종), ReadStatistics
- **[확장] mhitm.rs — mhitm.c 확장 2차 (728→1,018줄)**: 몬스터간 전투 승률 예측, 공격 유형별 기대 데미지(9종), 몬스터 공포 판정, 도주 방향 계산, DefenseType(6종 — Dodge/Block/Parry/MagicShield/Reflect), 전투 보고서 생성, MhitmExtendedStats
- **[확장] spell.rs — spell.c 확장 2차 (730→977줄)**: 주문 망각 턴 수(지능 보정), 망각 경고 메시지(3단계), 에너지 회복률(지혜/레벨/역할 보정), SpellBackfire(7종) 실패 부작용, 주문 레벨업 효과(7학파), 학파별 방어 보너스, 마나 폭주 경계/메시지, 주문 충돌 판정, SpellStatistics

### Changed
- **Rust 소스 라인 수**: 56,920줄 → 59,326줄 (+2,406줄)
- **테스트**: 351개 → 426개 (+75개) — 전체 통과

## [2.3.3] - 2026-02-15
### Added
- **[확장] steal.rs — steal.c 대폭 이식 (400→825줄)**: 금화 절도량(레벨 기반), 레프러콘 특수 절도, 장비 탈취(님프/원숭이 대상 선택), 절도 저항(dex/장갑/보유 보정), 님프 매혹 4단계, 절도 발각 확률, 가치 기반 대상 선택, 저주 장비 차단, 절도 통계, 메시지 생성
- **[확장] monmove.rs — monmove.c 대폭 이식 (328→651줄)**: 특수 이동 7종(터널/위상/수영/비행/땅속/천정), 은신/매복 AI, 아이템 선호도(용/레프러콘/님프), 텔레포트 도주, 행동 우선순위 9단계(dochugw), 편향 배회, 추적 퍼시스턴스
- **[확장] do_name.rs — do_name.c 대폭 이식 (326→680줄)**: 랜덤 펫 이름(남/여 20종), 몬스터 칭호 7종, BUC 형용사, 마법 부여 표시, 완전한 아이템 이름, 복수형 처리(makeplural 10종 특수형+규칙), 단수형 역변환, 소유격, 8방향 이름, 부정관사(a/an)
- **[확장] exper.rs — exper.c 대폭 이식 (313→512줄)**: 보스/고유 몬스터 경험치(×3/×5), 킬 스트릭 감소(5단계), 엘리트 몬스터 경험치, 경험치 벌금 5종(우호/제노사이드/기도/아티팩트/간접), 레벨 통계, 레벨업 보상 테이블, 최대 HP 기대값
- **[확장] attrib.rs — attrib.c 대폭 이식 (274→510줄)**: 속성 설명(9단계), STR 18/xx 포맷, STR 데미지/명중 보너스, DEX AC 보너스, CON HP 보너스, WIS 에너지 보너스, CHA 상점 할인, 종족별 속성 상한(5종족×6속성), 속성 포션 효과, 속성 요약 문자열, 적재량
- **[확장] engrave.rs — engrave.c 대폭 이식 (314→667줄)**: 새기기 도구 16종(맨손/날붙이/아세임/완드 13종), 도구별 효과(타입/속도/내구도/메시지/충전소모), 풍화 시스템, 엘버레스 효과 강도, 특수 문구(xyzzy/help), 새기기 통계
- **[확장] teleport.rs — teleport.c 대폭 이식 (369→661줄)**: 안전 착지 검증, 텔레포트 금지 구역(성소/퀘스트), 몬스터 텔레포트, 텔레포트 저항 3단계, 함정 효과 3종, 근접 텔레포트, 쿨다운 체크, 텔레포트 통계
- **[확장] kick.rs — dokick.c 대폭 이식 (304→605줄)**: 밀림 효과(hurtle), 잠긴 상자 차기, 함정 위 발차기, 분수/싱크대 차기 효과, 부츠 타입 8종별 보너스/보호, 발차기 통계
- **[확장] explode.rs — explode.c 대폭 이식 (283→501줄)**: 5×5 대형 폭발, 연쇄 폭발(화염/마법), 폭발 방어 5종(저항/절반/구체/반사), 지형 변경 6종(소각/동결/녹임/증발/파기), 폭발 음향 메시지, 폭발 통계
- **[확장] fountain.rs — fountain.c 대폭 이식 (132→402줄)**: 분수 소원(대형/소형/물 악마), 고갈 확률, 담그기 8종(축복/저주/녹/희석/부식/엑스칼리버), 씻기 5종(세척/실명 해제/슬라임 제거), 분수 통계
- **[확장] timeout.rs — timeout.c 대폭 이식 (144→384줄)**: 타이머 카테고리 5종(치명/변환/디버프/버프/환경), 우선순위, 경고 메시지 다단계(석화6/슬라임6/질식6/디버프5), 해제 조건 6종(기도/유니콘/석화치료/불치료/포션/시간), 타이머 통계
- **[확장] vision.rs — vision.c 대폭 이식 (337→586줄)**: 몬스터 감지 종합(시야/투시/적외선/텔레파시/진동/경고), 시야 차단 분류(Full/Partial/None), 동적 광원 7종(양초/램프/랜턴/마법램프/태양검/완드/주문), 광원 지속시간/연료 경고, 시야 통계
- **[확장] luck.rs — luck.c 대폭 이식 (45→288줄)**: 운 범위(-13~13), 유효 운 계산(럭스톤 보호), 럭스톤 4종, 행위 기반 운 변동 12종(기도/살인/제물/절도/자선/거울/검은고양이/유니콘/분수/신벌/선물), 운 기반 주사위 보정(rnl), 성공 확률 보정, 운 통계
- **[확장] mhitm.rs — mhitm.c variant 수정 (392→791줄)**: DamageType/AttackType 실제 enum에 맞게 수정(Phys/Magm/Drst/Ston/Elec/Plys/Drli/Hugs), 공격 선택 로직 수정
- **[확장] search.rs — detect.c/cmd.c 대폭 이식 (73→302줄)**: 수색 성공률(운/지혜/스킬/반복 보정), 함정 종류별 감지 난이도(18종), 비밀통로 감지 난이도, SearchResult(8종), 수색 범위 확장, 자동 수색, 미믹 감지, 수색 통계
- **[확장] sink.rs — sink.c 대폭 이식 (105→275줄)**: 물마시기 효과 8종(뜨거운물/하수/독/마법/지렁이/환각), 반지 식별(dosinkring — 원소반지 자동식별), SinkRingEffect 4종, 싱크대 파괴 판정, 싱크대 통계
- **[확장] sit.rs — sit.c 대폭 이식 (122→360줄)**: 왕좌 효과 16종(소원/금화/제노사이드/식별/회복/능력치±/텔레포트/소환/감전/실명/혼란/독/변신/성향), 지형별 앉기 결과 9종(바닥/제단/분수/싱크대/함정/용암/물/무덤), 제단 앉기 효과, 왕좌 금화량, 앉기 통계
- **[확장] pray.rs — pray.c 대폭 이식 (434→686줄)**: 기도 성공 판정(신앙도/성향/운/제단 보정), 죄 7종(살인/펫학대/식인/무덤훼손/절도/규범위반/평화공격), 신의 분노 7종(번개/지진/벌레/부여해제/저주/소환/실명), 분노 데미지, 제물 가치(유니콘/고레벨 보너스), 기도 통계
- **[확장] wizard.rs — wizard.c 추가 유틸리티 (435→532줄)**: 아뮬렛 감지(거리/레벨), 재소환 간격(처치 횟수 비례 감소), 분신 수(3회 이상 처치 후), 저주 강도(레벨/운 보정), WizardStatistics
- **[확장] pager.rs — pager.c 대폭 이식 (491→671줄)**: MonsterDetail(공격/저항/특성 포함), 위험도 평가 7단계(DEADLY~Trivial), 아이템 상세 설명(BUC/부여), LookAtResult(몬스터/아이템/타일/함정/각인), 백과사전 검색(7종 몬스터), PagerStatistics
- **[확장] detect.rs — detect.c 대폭 이식 (583→744줄)**: DetectionType(9종), 감지 효과 지속시간(축복/스킬 보정), 감지 범위(축복 2배), 감지 실패 확률(저주/혼란), 감지 우선순위, DetectionStatistics
- **[확장] dog.rs — dog.c/dogmove.c 대폭 이식 (538→732줄)**: 먹이 거부 판정, 전투 능력 평가, 전투 성공 판정, 성장 체크/레벨업 효과, PetDeathCause(6종), 텔레포트 추적 거리, 서식지 적합성, PetStatistics

### Changed
- **Rust 소스 라인 수**: 60,446줄 → 56,920줄 (재측정 기준) — 추가 1,528줄
- **테스트**: 304개 → 351개 (+47개) — 전체 통과

## [2.3.2] - 2026-02-15
### Added
- **[확장] trap.rs — trap.c 대폭 이식 (898→1,544줄)**: 함정 기본 데미지 계산(24종 전체 커버), 회피 판정(dex/luck/비행/공중부양), 저항 판정(Immune/PartialResist/Vulnerable), 연쇄 반응(지뢰), 마법 함정 효과 9종, 레벨 텔레포트 목적지, TrapStatistics 통계, 깊이별 생성 가중치, 몬스터 함정 피해, 감지/해제 난이도
- **[확장] worn.rs — worn.c 대폭 이식 (755→1,071줄)**: 몬스터 장비 효과(update_mon_intrinsics), EquipmentEvent(착용/해제/저주/파괴/변환), 장비 슬롯 표시 이름, 착용/해제 동사, WornItem + 전체 효과 합산, 착용/벗기 지연 턴, BUC 식별, 장비 무게
- **[확장] mcastu.rs — mcastu.c 대폭 이식 (518→1,136줄)**: BeamType(9종 광선 주문), cast_beam_spell(반사/흡수/저항), 시전 빈도, SummonClass(10종), SpellProperties(성직자10+마법사11 속성), 특수 저항 조합
- **[확장] botl.rs — botl.c 대폭 이식 (436→793줄)**: HP 상태 텍스트(8단계), HP 경고 레벨(6단계), 경험치 테이블(1~30레벨), 상태 변경 감지(10종), FullStatusSnapshot, 점수 추정, AC 설명/색상
- **[확장] objnam.rs — objnam.c 대폭 이식 (458→866줄)**: 물약/스크롤/반지/완드 외관 테이블(113종), 골드 포맷, 수량 라벨, BUC 라벨, full_display_name, 아이템 병합 판정, 인벤토리 글자 할당

### Changed
- **Rust 소스 라인 수**: 49,745줄 → 56,337줄 (+6,592줄) — 이식률 31.8%

## [2.3.1] - 2026-02-15
### Added
- **[이식] status.rs 상태이상 적용 함수 10종**: potion.c의 `make_confused`, `make_stunned`, `make_sick`, `make_blinded`, `make_hallucinated`, `make_stoned`, `make_slimed`, `make_vomiting`, `make_deaf`, `make_glib` + `itimeout`/`itimeout_incr` 타이머 유틸리티
- **[이식] mon.rs 몬스터 유틸리티 17종**: `undead_to_corpse`, `genus`, `is_clinger`, `likes_lava`, `is_floater`, `amphibious`, `is_reviver`, `unique_corpstat`, `is_lava_resistant`, `is_dragon`, `MonsterSpeed` enum, `mcalcmove_with_speed`, `monflee_turns`, `canseemon`, `stagger`, `minliquid_check`, `mm_aggression`, `mcalcdistress_tick`
- **[이식] potion.rs 물약 시스템 확장 4종**: `potionhit_damage` (투척 데미지), `potionbreathe` (증기 흡입), `mixtype` (연금술 혼합), `ghost_from_bottle` (유령 병)
- **[신규] eat.rs — eat.c 완전 이식**: `FoodType` enum, `EatingState` 구조체, 영양 계산, 시체 효과(저항 부여/석화/슬라임화/중독/텔레포트), 과식/공복 시스템, 시체 부패, 금속 먹기, 통조림, 달걀 부화 등 50+ 함수
- **[신규] do_actions.rs — do.c 완전 이식**: 아이템 드롭(물/용암/제단), 계단 이동(업/다운/부유체크), 탐색(비밀문/함정), 앉기(왕좌 효과), 차기(문/몬스터), 턴 언데드, 제단 BUC 감정, 우물 드롭 등
- **[신규] weapon.rs — weapon.c 완전 이식**: WeaponSkill/SkillLevel/SkillData 구조, 무기 타입 판정, dmgval/hitval 보정, 역할별 스킬 초기화(13종), 은무기/부식/맨손격투/승려 등
- **[확장] dungeon.rs — dungeon.c 핵심 이식**: BranchInfo 구조체, 7개 브랜치(Main/Mines/Sokoban/Gehennom/Quest/VladTower/Astral), 깊이/난이도 계산, 특수 레벨 판정, 브랜치 네비게이션
- **[확장] inventory.rs — invent.c 핵심 이식**: InventoryLetters(52슬롯), ItemClass(16종), BurdenStatus(6단계), 짐 상태/속도 감소, 적재량 계산, 아이템 스택/정렬/표시
- **[확장] evolution.rs — polyself.c 핵심 이식**: PolyForm 구조체, 변신 가능 판정(polyok), HP/기간 계산, 장비 탈락, 능력 변환, 변신 함정, System Shock 체크
- **[확장] artifact.rs — artifact.c 핵심 이식**: ArtifactData(14종 아티팩트 DB), 전투 데미지/명중 보정, AC 보정, 성향 체크, Vorpal 참수
- **[확장] trap.rs — trap.c 대폭 이식 (377→898줄)**: TrapInfo 데이터 테이블(23종), 트랩 감지/해제(DisarmResult), 트랩 저항 체크, 랜덤 트랩 생성(rndtrap), 낙하 데미지, 곰 트랩/거미줄 탈출, 트랩 메시지 체계, 레벨 트랩 배치
- **[확장] teleport.rs — teleport.c 대폭 이식 (109→342줄)**: TeleportControl(3단계), TeleportCause(6종), 텔레포트 가능/불가 판정, 안전 착지점 탐색, 레벨 텔레포트 범위 계산, 아뮬렛 차단, 몬스터 텔레포트 AI, 쿨다운, 브랜치 텔레포트
- **[확장] shop.rs — shk.c 대폭 이식 (393→677줄)**: ShopType(10종), ShopTypeInfo 데이터, 상점 주인 이름 풀(30개), ShopDebt 부채 관리, 도둑 경보 수준(TheftSeverity), 상점 인사말, 값어치 감정, 파손 비용, 상점 인벤토리 수
- **[확장] death.rs — end.c 대폭 이식 (271→446줄)**: DeathCause(18종), 사망 원인 설명, 점수 계산(7팩터), Tombstone 묘비 생성(ASCII art), DeathStats 통계, DYWYPI, 게임 오버 요약
- **[확장] save.rs — save.c+restore.c 대폭 이식 (132→292줄)**: SaveMetadata(세이브 요약), 세이브 파일 목록/존재확인/삭제, 자동 세이브, bones 파일(보너), 무결성 검증
- **[확장] options.rs — options.c 대폭 이식 (49→544줄)**: 40+ 옵션 필드(게임플레이/UI/하이라이트/고급), set_option/get_option 동적 설정, OptionHelp 도움말 테이블(24종), 옵션 검증(validate_option)
- **[확장] zap.rs — zap.c 대폭 이식 (577→996줄)**: WandType 데이터 테이블(20종), 지팡이 충전(recharge), 자가 대상(self_zap), 저항 데미지 보정, 빔 시각화, 폭발 범위, 지팡이 꺾기(break_wand)
- **[확장] apply.rs — apply.c 대폭 이식 (400→967줄)**: ToolType(30종), 도구 적용 판정, 거울/호루라기/관악기/나팔/크리스탈볼/마커/갈고리/틴오프너/깡통따개 등 핵심 도구 효과
- **[확장] spell.rs — spell.c 대폭 이식 (259→730줄)**: SpellSchool(7분야), SpellData 테이블(37종), 주문 기억 감쇠, 시전 실패 확률, 역할별 주문 보너스, 실패 부작용, 학파별 색상
- **[확장] throw.rs — dothrow.c 대폭 이식 (222→469줄)**: ThrowableType(9종), 투척 데미지/거리/파괴 테이블, 물약 투척 효과(10종), 알/크림파이 효과, 부메랑 궤적, 역할별 보너스, 착탄 판정
- **[확장] kick.rs — dokick.c 대폭 이식 (106→273줄)**: KickTarget/KickResult, 데미지 계산(무술/장화/레벨 보너스), 문 부수기 확률, 벽 차기 자해, 바위 밀기, 도구 발차기, 아이템 밀기 거리
- **[확장] engrave.rs — engrave.c 대폭 이식 (134→289줄)**: 새기기 속도(도구별), 묘비 메시지 풀(40개), 문자열 훼손, 완드 감정 힌트(10종), Elbereth 보호 강도, 최대 길이 제한
- **[확장] equipment.rs — do_wear.c 대폭 이식 (304→687줄)**: ArmorCategory(6종), 장비 순서 제약, 저주 장비, 부식 설명, 드래곤 비늘 갑옷(9종 속성), 변신 장비 탈락, 방어구 데이터 테이블(24종)
- **[확장] vision.rs — vision.c 대폭 이식 (264→305줄)**: 일시 조명(TEMP_LIT), 블라인드 시야, 적외선 시야, 텔레파시 감지, 투시, 시야 반경 계산, 클레어보이언스, 광원 반경

### Changed
- **DungeonBranch enum 확장**: Gehennom, Quest, VladTower, FortKnox, Astral 5종 추가
- **Rust 소스 라인 수**: 42,182줄 → 49,745줄 (+7,563줄) — 이식률 28.07%


## [2.3.0] - 2026-02-15
### Added
- **[M6] 상태 바 Classic/Graphical 모드 전환**: `StatusBarMode` enum으로 전통 2줄 텍스트(Classic) ↔ GnollHack 스타일 그래피컬(Graphical) 전환 가능. Graphical 모드는 좌측 `@` 아이콘 + 세로 능력치 + 우측 HP/MP/XP 프로그레스 바 + AC/XL/Gold + 상태 아이콘 레이아웃
- **[M6] XP 프로그레스 바**: NetHack 원본 exper.c 경험치 테이블(1~30레벨) 기반으로 다음 레벨까지의 진행률을 시각적으로 표시
- **[M6] AC 색상 세분화**: AC ≤0 밝은 초록(매우 좋음) → AC 1~3 초록(양호) → AC 4~6 노랑(보통) → AC 7~9 주황(취약) → AC 10+ 빨강(위험)으로 5단계 색상 분류
- **[M6] 능력치 색상 분류**: 18 이상=초록, 7 미만=빨강, 14 이상=하이라이트, 나머지=기본색
- **[M7] 메시지 자동 색상 분류 시스템**: 6 카테고리(Normal/Danger/Loot/System/Achievement/Movement) × 패턴 매칭으로 메시지 텍스트 자동 분류. 카테고리별 아이콘(⚔/✦/ℹ/★) 자동 부착
- **[M7] --More-- 프롬프트**: 같은 턴에 8개 이상 메시지 발생 시 `--More--` 프롬프트 표시 (기존 `needs_more` 플래그 활용)
- **[M7] 턴 구분선**: 새로운 턴이 시작될 때 메시지 사이에 세퍼레이터 삽입
- **[M7] 메시지 카운터**: 패널 헤더 우측에 현재 메시지 수 표시

### Changed
- **[M8] main.rs 최소화**: 199줄 → 93줄로 축소. AppState 분기 핸들러를 `app_update.rs`로 추출 (apply_global_style/handle_title_screen/handle_char_creation/check_game_over/handle_game_over_screen 5개 메서드)
- **[M8] 공통 상수 정의**: `APP_VERSION` ("2.3.0"), `APP_TITLE` ("AIHack") 상수로 중앙 관리

## [2.2.0] - 2026-02-15
### Added
- **[M5] 인벤토리 화면 GnollHack 스타일 리뉴얼**: 카테고리별 아이콘(⚔🛡🍖🧪📜 등) + BUC 색상(축복=청록, 저주=빨강, 일반=밝은 회색, 미감정=어두운 회색) + 개별 무게 표시 + Carrying/Items 캐리 요약 + Encumbrance 5단계(Normal→Burdened→Stressed→Strained→Overtaxed→Overloaded) 경고 + 스크롤 영역 + 하단 액션 버튼 바(Drop/Apply/Eat/Drink/Read/Wear/Wield)
- **[M5] 장비 화면 신규 생성** (`equipment.rs`): ASCII @ 캐릭터 실루엣 + 12슬롯(Head/Cloak/Body/Shield/Gloves/Boots/Weapon R&L/Ring R&L/Amulet/Quiver) 장착 상태 표시 + BUC 색상 + ✕ 해제 버튼 + AC 분해(Base/Armor/Shield/Other) + 하단 장비 관리 버튼(Wear/Take Off/Wield/Put On/Remove)
- **[M5] EquipmentSlot 확장**: `RingLeft`, `RingRight`, `Amulet`, `Boots` 4개 슬롯 추가 (NetHack uleft/uright/uamul/uarmf 대응)
- **[M5] ItemAction 확장**: `Unequip(Entity)` + Prompt 계열 5종(`WearPrompt`/`TakeOffPrompt`/`WieldPrompt`/`PutOnPrompt`/`RemovePrompt`)

## [2.1.0] - 2026-02-15
### Added
- **[M1] AppState::GameOver + 전용 사망 화면**: `AppState`에 `GameOver` variant 추가 (message, score, turns, max_depth, epitaph). `game_over.rs` 풀스크린 사망 화면 — ASCII 묘비 + 통계 + New Game/Quit 버튼. `Playing` 상태에서 `GameState::GameOver` 감지 → `AppState::GameOver` 자동 전환. Restart → 타이틀 화면 복귀. 테스트 144개 전부 통과.
- **[M2] 상태 아이콘 확장 (7→21종)**: 감각 이상(Blind/Conf/Stun/Halluc), 질병(Sick/FPois), 이동(Lev/Fly/Slow/Haste), 치명(Stone/Slime/Strngl), 하중(Burden~OvrLd 5단계) + HungerState(Satiated/Hungry/Weak/Faint/Starved) 별도 처리. 상태 바에 아이콘+색상으로 표시.
- **[M2] 장비 요약 ECS 연동**: 우측 Stats Panel의 Equipment 섹션이 `Equipment` 컴포넌트에서 실제 장착 아이템 이름을 조회하여 표시 (Melee/Shield/Body/Head/Cloak/Hands/Feet 7슬롯). `item.spe` 강화치 포함 표시.
- **[M2] Settings 윈도우**: View > Settings 체크박스로 토글. Autopickup, 심볼 세트(Original/IBM/DEC), 색상, 펫 강조, 반려동물 이름 변경 가능. `options.toml` 저장 연동.
- **[M3] 마우스 클릭 이동 시스템 확인 완료**: `mouse.rs` 좌클릭(인접/Travel/SelfClick), `context_menu.rs` 우클릭 검사, A* 경로 탐색 Travel 큐 + 몬스터 감지 중단. v1.9.0에서 구현, v2.1.0에서 완료 확인.
- **[M4] 전체 키바인딩 완성 확인 완료**: 87개 Command enum, Run(Shift+이동) 8방향 달리기, `#` 확장명령 27개 매핑(`from_extended_str`), 검사 명령 10종, 도움말용 `all_commands_categorized()` 7카테고리. v1.9.0에서 구현, v2.1.0에서 완료 확인.
- **[ARCH-1] 이식 아키텍처 원칙 명문화**: C 구조 직역 금지 + 현 Rust/ECS 아키텍처 준수 원칙을 `spec.md`, `DESIGN_DECISIONS.md`, `designs.md`, `audit_roadmap.md` 4개 문서에 공식 등재.
### Fixed
- 루트 디버깅/테스트 잔여 txt 파일 61개 삭제 + `.gitignore`에 `*.txt` 추가 (assets/dat 보존)
- `dungeon/mod.rs` roomno (u8) >= 0 불필요 비교 → > 0으로 수정
- `game_ui.rs` Version 명령 피드백 버전 문자열 v1.9.0 → v2.1.0 업데이트

## [2.0.0] - 2026-02-14
### Changed
- **[Phase R6 완료] 비트플래그 래퍼 + God Object 뷰 타입 + 100% 전환**: `MonsterCapability` enum(60+ ability). **`has_flag` → `has_capability()` 100% 전환** (89개). **`WornSlot` enum** (12슬롯). **`StatusEffect` enum** (44효과, 6카테고리 컴파일 타임 분류). **Player 뷰 타입 4종 완전 구현** — `PlayerCombatView`/`SurvivalView`/`ProgressView`/`AttributeView` + `apply_to()` 역방향 메서드 + `as_*_view()` 팩토리. 테스트 8개. 기존 코드/데이터 무변경.
- **[Phase R2-5] `kind.as_str() == "리터럴"` → enum 패턴 매칭 100% 전환**: item_helper(2), item_damage(3), item_tick(2), inventory(1), apply(2), talk(1) — 총 11개소. 동적 비교 1개(artifact.rs) 잔존(설계상 의도). 테스트 144개 전부 통과.
- **[Phase R2-6] ItemManager/MonsterManager enum 기반 인덱스 + get_by_kind() API**: `HashMap<Kind, String>` 병렬 인덱스 추가. `build_kind_index()` 자동 구축. **전체 31개소** `get_template(as_str())`/`templates.get(as_str())` → `get_by_kind(kind)` 전환 (18개 파일). 추가 전환: weight(BagOfHolding), pray(Water, is_corpse), offer/inventory_ui(is_corpse). 잔존 10개소는 표시용/미등록enum/설계상 불가. 테스트 144개 전부 통과.
- **[Phase R5 확장] 전투/사망/장비/상태 이벤트 + 소비자 구현**: `death.rs`(MonsterDied, ExperienceGained, PlayerDied). `movement.rs`(DamageDealt, AttackMissed). `ai/core.rs`(DamageDealt, StatusApplied). `equipment.rs`(ItemEquipped, ItemUnequipped, EquipmentChanged). `status.rs`(StatusExpired). `item_use.rs`(StatusApplied ×7). `game_loop.rs`에 이벤트 소비자: EventQueue→EventHistory 기록 + clear (라이프사이클 완성). item_use에서 미사용 _level_req 리소스 제거(Legion 8-tuple 한계).
- **[Phase R5] GameEvent 이벤트 큐 시스템**: `GameEvent` enum(20+ variant) + `EventQueue` + `EventHistory` 인프라 정의. 카테고리 필터링, 자연어 요약(`to_narrative()`), 링 버퍼 히스토리(200개). Legion 리소스 등록. LLM 문맥 피딩 대비.
- **[Phase R4] Creature/UseEffect/Behavior 트레이트 도입**: 3개 도메인에 통합 트레이트 인터페이스 정의. `CreatureSnapshot` + `Combatant` (전투), `UseEffect` + `UseResult` (아이템), `Behavior` + `Conversable` (AI). `RuleBasedAi`, `PetAi`, `ScriptedDialogue` 기본 구현. LLM 연동 대비 레이턴시 경계 설계.
- **[Phase R3] 시스템 모듈 트리 재구성**: `systems/` 70개 플랫 파일 → 9개 도메인별 서브디렉토리로 재구성 (combat, ai, item, creature, world, social, spawn, identity, misc). 기존 import 경로 100% 호환 유지.
- **[Phase R2] Item.template: String → Item.kind: ItemKind 열거형 전환**:
  - `Item` 구조체의 `template` 필드를 `kind: ItemKind`로 변경 (타입 안전성 확보)
  - `ItemKind` enum은 `build.rs`에서 `items.toml`로부터 자동 생성
  - ~30개 파일에서 `item.template` → `item.kind` 변환 수행
  - 문자열 비교: `item.kind.as_str() == "xxx"` 패턴으로 단계적 전환
  - HashMap 조회: `get_template(item.kind.as_str())` 호환성 유지
  - 시체(corpse) 아이템: `ItemKind::from_str()` → `UnknownItem` + `corpsenm` 필드로 구분
- **[Phase R1] main.rs 해체 완료**: 174KB → 5.7KB (97% 축소)
- **던전 생성 시스템 대폭 개선 (원본 NetHack 3.6.7 makelevel() 충실 이식)**:
  - `split_rects()` 매개변수 순서를 원본과 일치하도록 수정 (방 중첩 방지)
  - 방 조명 확률을 깊이 기반으로 변경 (깊은 층일수록 어두운 방 증가)
  - `fill_ordinary_room()`: 방 면적 기반 몬스터 밀도 적용 (큰 방 1~3마리, 작은 방 0~1마리)
  - `fill_ordinary_room()`: 방에 아이템 배치 추가 (큰 방 0~2개, 작은 방 0~1개)
  - 특수 지형(Fountain/Sink/Grave) 깊이 조건 추가 (원본 대응)
  - `fill_level_monsters()`: 모든 레벨 타입에서 방 채우기 이후 추가 몬스터/아이템/금화 레벨 전체 배치
  - Bigroom: 몬스터 10~20마리 + 아이템 5~10개 + 분수 배치 추가
  - Maze: 몬스터 8~16마리 + 아이템 3~8개 배치 추가
  - Maze: 시작 방에도 apply_room 호출하여 올바른 벽/바닥 생성
  - 금화 스택: 깊이에 비례하는 양으로 개선

### Fixed
- `split_rects(used_rect, &rect)` → `split_rects(rect, &used_rect)`: 원본 NetHack 순서 복원
- 일반 방(Ordinary)에서 몬스터가 33% 방에만 1마리 생성되던 문제 → 면적 기반 복수 생성으로 수정
- Bigroom/Maze 레벨에 몬스터/아이템이 전혀 배치되지 않던 문제 수정
- `spawn_monsters()` 주석 처리 상태와 무관하게 `generate_improved()` 내에서 올바른 스폰 보장

### Added
- **몬스터 사망 처리 시스템 완전 이식 (원본 mon.c:mondead/xkilled)**:
  - 사망 시 시체 드롭 (corpse_chance 기반, `%` 글리프)
  - 사망 시 인벤토리 아이템 바닥에 드롭 (ItemDropRequest 패턴)
  - 경험치 시스템 연동 (exper.c:experience + adjusted_experience)
  - 레벨업 자동 트리거 (gain_experience → pluslvl)
  - 인간형 몬스터 금화 보너스 XP
  - 폭발형 몬스터 사망 시 execute_explosion 호출
  - DeathResults 리소스 기반 SubWorld↔World 브릿지 패턴
- **플레이어 사망 → 게임 오버 상태 전환 (원본 end.c:done)**:
  - HP ≤ 0 시 GameState::GameOver 자동 전환
  - 묘비명 출력 (epitaph)
- **턴 기반 몬스터 리스폰 시스템 (원본 allmain.c)**:
  - 매 턴 1/50 확률로 플레이어 시야 밖(15칸+)에 랜덤 몬스터 1마리 생성
  - 현재 층 몬스터 40마리 상한선
  - Sokoban 제외
- **트랩 배치 시스템 (원본 mktrap.c)**:
  - 던전 생성 시 깊이 기반 트랩 자동 배치 (depth/4 + 2 ~ depth/4 + 6 개)
  - 4단계 깊이별 트랩 종류 테이블 (초반: Arrow/Dart, 심층: LevelTeleport/Polymorph)
  - Sokoban 퍼즐 보호 (트랩 배치 금지)
- **시체 부패/먹기 시스템 정합성 수정**:
  - item_tick.rs: 시체 판별을 `corpsenm.is_some()` 기반으로 확장
  - item_use.rs: 먹기 시 시체 판별을 `ends_with("corpse")` 기반으로 확장
- **[Phase A 감사] 배고픔 시스템 원본 이식 (eat.c:morehungry, newuhs, starvation)**:
  - 매 턴 기본 영양소 소모 1 → Satiated 상태 시 추가 소모(+1) 적용
  - 기아 사망: nutrition < -200 도달 시 HP=0 (원본 "You die from starvation")
  - 기절: nutrition ≤ 0일 때 rn2(20) 확률로 SLEEPING 상태 부여
  - Satiated 상태 메시지 추가 ("You feel stuffed.")
- **[Phase A 감사] AI 시스템 대폭 보강 (monmove.c, mhitu.c)**:
  - Level 필터 적용: 같은 층 몬스터만 AI 처리 (다른 층 충돌 판정 방지)
  - 속도 시스템 연결: `monmove::can_act_this_turn(speed, turn)` — 빠른/느린 몬스터 구분
  - Peaceful 체크: `monster.hostile == false`인 몬스터는 플레이어 공격 안 함
  - 공격 메시지 출력: "The X attacks you!" 로그 추가
  - 점유맵(occupancy) 구축 시 Level 필터 적용
- **[Phase B 감사] 장비 효과 ↔ 배고픔 시스템 연결 (worn.rs ↔ status.rs)**:
  - `equipment.rs`: `update_player_stats`에 `worn.rs`의 `EquipmentEffects` 합산 로직 추가
  - Ring of Hunger → 배고픔 가속(+1), Regeneration → 추가 소모(+1), Slow Digestion → 감속(-1)
  - `Player.equip_hunger_bonus` 필드 추가 → `status.rs`에서 매 턴 반영
- **[Phase B 감사] AI 이동 일관성 개선**:
  - `move_random`: 비동물 몬스터 `OPENDOOR` 플래그 추가 (`move_towards`와 동일)
  - `can_see_pos`: `#[allow(dead_code)]` 추가 (향후 AI 고도화 시 사용 예정)

## [Unreleased] - 2026-02-13
### Added
- **M2: 현대적 게임 레이아웃 구현 (v1.9.0)**:
  - **Menu Bar** (`layout/menu_bar.rs`): File/View/Commands/Help 메뉴 + 턴 카운터. View 메뉴에서 미니맵/스탯패널/메시지패널 토글 가능
  - **Command Bar** (`layout/command_bar.rs`): Simple(1줄)/Advanced(2줄) 모드 전환 가능한 하단 액션 버튼 바. 아이콘+텍스트 조합
  - **Status Bar** (`layout/status_bar.rs`): 능력치 2줄 + 그래피컬 HP/MP 프로그레스 바. HP 색상은 체력 비율에 따라 동적 변경 (초록→노랑→주황→빨강)
  - **Minimap** (`layout/minimap.rs`): 좌측 패널에 탐험 지역 축소 맵 표시. 타일별 색상, 플레이어 위치 흰색 점, 미탐험 영역 투명 처리
  - **Message Panel** (`layout/message_panel.rs`): 좌측 패널 하단에 최근 50개 메시지 스크롤 표시. 색상+턴 번호
  - **Stats Panel** (`layout/stats_panel.rs`): 우측 사이드패널에 캐릭터 이름/HP바/MP바/능력치 6종/AC/XL/골드/덩전깊이/장비 10슬롯 요약
  - **LayoutSettings**: 각 패널의 가시성을 관리하는 설정 구조체. View 메뉴와 동기화
- **M3: 마우스 클릭 이동 시스템 (v1.9.0)**:
  - **좌클릭 이동** (`ui/mouse.rs`): 인접 칸 클릭 → 즉시 이동/공격 (1턴), 자기 위치 클릭 → 줍기
  - **원거리 Travel** : 먼 칸 클릭 → A* 경로 탐색 후 자동 이동 큐. PathFinder::find_path 연동
  - **Travel 중단 조건**: 시야 내 몬스터 감지 시 자동 중단 + 메시지 로그 알림
  - **우클릭 컨텍스트 메뉴** (`ui/context_menu.rs`): 타일/몬스터/아이템 정보 팝업 (HP, AC, Level 표시)
  - **좌표 변환 엔진** : 스크린 좌표 → 그리드 좌표 변환 (char_width/char_height 기반)
  - **Direction ↔ Command 매핑**: delta_to_direction → direction_to_command 체인으로 기존 이동 처리 경로 100% 재활용
- **M4: 전체 키바인딩 완성 (v1.9.0)**:
  - **Command enum 확장**: 45개 → 85개 명령어 (cmd.c 기반 완전 이식)
  - **달리기(Run)**: Shift+이동키로 8방향 연속 이동. 벽/몬스터 감지 시 자동 중단
  - **확장 명령(#) 입력 시스템**: `#` 키 → 팝업 텍스트 입력 + 자동 완성 후보 목록. dip/force/jump/ride/rub/tip/turn/untrap/wipe/chat/adjust/monster 지원
  - **아이템 검사 명령**: `/` WhatIs, `;` LookHere, `:` LookAtFloor, `)` ShowWeapon, `[` ShowArmor, `=` ShowRings, `"` ShowAmulet, `(` ShowTool, `\` Discoveries
  - **정보 유틸리티**: `v` Version, Ctrl+O Overview, `#conduct`, `#score`, `#turncount`
  - **display_name()/keybinding() 메서드**: 모든 85개 명령에 표시 이름과 키바인딩 문자열 제공 (도움말 화면용)
  - **all_commands_categorized()**: 이동/아이템사용/검사/상호작용/확장명령/정보/시스템 7개 카테고리로 분류
  - **from_extended_str()**: 문자열 → Command 변환 (26개 확장 명령 지원)
- **UI 현대화 계획 수립 (MODERNIZATION_PLAN.md)**:
  - GnollHack 인터페이스 참조 기반의 8단계 현대화 마일스톤 설계
  - M1: 타이틀 화면 + 캐릭터 생성 (13개 직업, 5개 종족, 성별, 성향)
  - M2: 현대적 레이아웃 (메뉴 바, 커맨드 바, Stats/Equipment 패널, 미니맵)
  - M3: 마우스 클릭 이동 (A* Travel, 우클릭 컨텍스트 메뉴)
  - M4: 전체 키바인딩 완성 (cmd.c 기반 ~100+ 명령어)
  - M5: 인벤토리/장비 화면 고도화 (카테고리별, BUC 색상, 장비 슬롯)
  - M6: 상태 바 고도화 (그래피컬 HP/MP 바, 상태 아이콘)
  - M7: 메시지 시스템 고도화
  - M8: main.rs 분리 리팩토링
### Changed
- **전체 문서 정비**: spec.md, audit_roadmap.md, designs.md, IMPLEMENTATION_SUMMARY.md, BUILD_GUIDE.md, DESIGN_DECISIONS.md, LESSONS_LEARNED.md 실측치 기반 갱신
- **이식률 정확도 보정**: 과대 보고된 이식률을 실측치(10.38%)로 교정
- **기존 상태줄/메시지로그 교체**: TopBottomPanel 기반의 단순 2줄 상태줄 → M2 현대적 레이아웃으로 전면 교체

## [1.8.0] - 2026-02-14
### Added
- **몬스터 AI 및 전투 시스템 고도화 (Phase 49) ✅**:
  - **Multi-Attack Engine**: 몬스터의 다중 공격 슬롯(최대 6개)을 순차적으로 처리하는 `mhitu.c` 정밀 이식. (Marilith 등)
  - **Resistance & Status**: 플레이어의 화염(`FIRE_RES`), 냉기(`COLD_RES`), 전기(`SHOCK_RES`) 등 저항력에 따른 데미지 감쇄 및 면역 로직.
  - **Deadly Effects**:
    - **Drain Life (`Drli`)**: 최대 HP 및 경험치 레벨 영구 감소.
    - **Paralysis (`Plys`)**: 수 턴간 행동 불가 상태(`PARALYZED`) 부여.
    - **Energy Drain (`Dren`)**: 마나(Energy) 흡수 및 고갈.
    - **Poison (`Drst`)**: 힘(Str) 감소 및 주기적 독 데미지.
  - **Social FSM**: `Fleeing`(도주), `Sleeping`(수면), `Hunting`(추격) 상태 머신 및 종족(`Faction`) 간 우호 관계 적용.

- **컨테이너 및 인벤토리 관리 (Phase 48) ✅**:
  - **Recursive Containers**: 가방(`Sack`)이나 상자(`Chest`) 안에 아이템을 보관하는 재귀적 인벤토리 구조(`inventory.rs`).
  - **Deep Weight**: 용기 자체 무게 + 내부 아이템 무게의 합산 계산 로직.
  - **Bag of Holding**: 축복받으면 무게 1/2, 1/4 감소, 저주받으면 2배 증가하는 마법 가방 효과 완벽 이식.
  - **Bag of Tricks**: 사용(`a`) 시 몬스터가 튀어나오거나 잡동사니를 발사하는 `bag_of_tricks` 시스템.
  - **Loot Command**: `#loot` (`Alt+l`) 명령으로 잠긴 상자를 열쇠(`key`)로 열거나(`lockpick`), 함정(`trap`)을 해제하고 내용물을 관리.

## [1.7.7] - 2026-02-11
### Added
- **트랩 시스템 고도화 (Phase 44) ✅**:
  - **Advanced Trap Engine**: NetHack 3.6.7의 모든 트랩 타입(Dart, Rock, Landmine, Web, Statue, Magic, AntiMagic, Polymorph 등)을 포팅 완료.
  - **Trap Interaction**: `trap.c`의 `dotrap` 로직을 ECS 환경에 맞게 재설계하여 독(Poison), 폭발, 상태 이상 효과 통합.
  - **Level Transition Fix**: 층간 이동 함정(Hole, Trapdoor)의 정밀한 레벨 전이 로직 구현 및 아이템 사용 시스템의 중복 대여 버그 수정.

## [1.7.6] - 2026-02-23
### Added
- **등불 및 조명 시스템 (Phase 43) ✅**:
  - **Light Source Engine**: `oil lamp`, `brass lantern`, `magic lamp` 등 광원 아이템의 연료 소비 및 점등 시스템 구현 (`apply.rs`, `item_tick.rs`).
  - **Magic Lamp Specials**: 매직 램프 사용 시 일정 확률로 Djinni가 출현하는 특수 이벤트 로직 이식. 다국어 메시지 지원 및 향후 Wish 시스템 연동 대비 스텁 추가.
  - **Illumination Items**: `Scroll of light`를 통한 방/복도 전체 조명 및 `Wand of light`의 직선 궤적 조명 발산 로직 구현 (`grid.light_room_at`, `zap.rs`).
  - **New Monster: Yellow Light**: 자폭 공격(`AD_BLND`)을 통해 광역 실명을 유발하는 `yellow light` 몬스터 템플릿 추가 및 폭발 엔진(`execute_explosion`) 고도화.
  - **Status Effect: Blindness**: 눈부신 빛에 의해 플레이어 및 몬스터가 눈이 머는 실명 상태 이상 구현 (`combat.rs`).

### Fixed
- **컴파일 오류 교정**: `dungeon/mod.rs` 내 `tile.room_id` 필드 참조를 `tile.roomno`로 올바르게 수정.
- **임포트 정합성**: `combat.rs` 내 `PlayerTag`, `Position`, `Health` 등 공통 컴포넌트의 중복/누락 임포트 정리 및 스코프 문제 해결.

## [1.7.5] - 2026-02-21
### Added
- **위험 지형 및 액체 상호작용 (Phase 42) ✅**:
  - **Dangerous Terrain Interactions**: 물(`Pool`, `Moat`)에서의 익사(Drowning) 및 용암(`LavaPool`)에서의 즉사 메커니즘 구현. 비행, 부양, 수상보행 상태에 따른 면제 로직 포함.
  - **Wetness & Item Damage**: 물에 젖었을 때 인벤토리의 포션이 희석(`water`로 변함)되거나 주문서/마법서가 백지가 되는(`blank paper`) 로직 구현.
  - **Swimming Skill**: 수영 숙련도(`WeaponSkill::Swimming`)에 따라 수중에서 아이템이 젖지 않고 보호될 확률(최대 90%) 적용.
  - **Terrain Transformation**: 냉기 공격에 의한 물의 결빙(`Ice`) 및 화염 공격에 의한 얼음의 해빙 상호작용 구현 (`zap.rs`).
  - **Ice Mechanics**: 얼음 위 이동 시 일정 확률로 미끄러지는(`Ice slipping`) 물리 효과 추가.
  - **New Items**: 희석된 포션인 `water`와 백지화된 주문서인 `blank paper` 아이템 템플릿 추가.

### Fixed
- **중복 매치 암 제거**: `main.rs` 내에서 `Command::Offer`가 중복 처리되어 발생하던 `unreachable pattern` 경고 해결.
- **임포트 정합성 개선**: `movement.rs` 내에서 `Inventory`, `Player`, `StatusFlags` 등 핵심 타입의 임포트 누락 및 스코프 문제 해결.

## [1.7.0] - 2026-02-12
### Added
- **운 및 신성 유지 시스템 (Phase 39) ✅**:
  - **Luck Maintenance Engine**: 원본 `luck.c`의 `u.uluckcnt` 로직을 이식하여 매 600턴마다 운(`u.uluck`)이 0을 향해 자연적으로 수렴하는 타이머 시스템 구현.
  - **Enhanced Divine Interaction**: 직업(`PlayerClass`) 및 성향(`Alignment`)에 따른 고유 신 이름(role.c) DB 구축. 기도 시 신의 호칭이 상황에 맞게 동적으로 표시됨.
  - **Offering UI (#offer)**: 제물로 바칠 수 있는 아이템(시체)만 필터링하여 보여주는 전용 인벤토리 위젯 추가. 플레이어 명령(`Command::Offer`) 처리 로직 통합.
  - **고도화된 상태 회복 로직**: 기도(`try_pray`) 시 `fix_trouble` 로직을 정밀화하여 석화(Stoning), 슬라임화(Slimed), 실명(Blind), 혼란(Confusion), 강직(Stunned), 식중독(Food Poisoning) 등의 해결 우선순위를 원본과 동일하게 배치.

### Changed
- **StatusFlags 확장**: 더 많은 상태 이상 상태를 수용하기 위해 `StatusFlags` 기반 타입을 `u32`에서 `u64`로 확장 및 `FOOD_POISONING` 플래그 추가.
- **주석 한국어화 및 D3D 준수**: `luck.rs`, `pray.rs` 등 핵심 시스템 파일의 모든 주석을 한국어로 전수 보정하여 프로젝트 전역 행동 지침(D3D Protocol) 준수.
### Added
- **몬스터 특수 공격 및 마법 시스템 (Phase 36, 37)**:
  - **광역 및 속성 공격**: 몬스터의 브레스(Breath), 시선(Gaze), 자폭(Explosion) 공격 구현. 속성 저항 및 반사(Reflection) 로직 적용.
  - **몬스터 주문 시전 (`cast_monster_spell`)**: Mage(Magic Missile, Haste, Teleport) 및 Cleric(Cure, Cause Wound, Blindness) 주문 엔진 이식.
  - **상세 공격 효과**: 에너지 흡수(`AD_DREN`), 아이템 절도(`AD_SITM`), 골드 절도(`AD_SGLD`), 휘감기/마비(`AD_STCK`) 등 특수 타격 효과 대거 추가.
- **특수 레벨 생성 시스템 (Phase 38)**:
  - **오라클 레벨 (Oracle)**: 델포이의 신탁 방 생성 로직 및 고유 NPC `Oracle` 스폰.
  - **광산 마을 (Minetown)**: 수영장, 성소, 상점가, 경비소가 포함된 광산 마을 레벨 레이아웃 완벽 이식.
  - **고유 NPC**: `Priest`, `Watchman` 등 마을 전용 몬스터 템플릿 및 스폰 로직 추가.

### Fixed
- **ECS 대여 충돌 및 린트 오류**: `movement.rs`와 `ai.rs` 내에서 발생하던 가변 대여 중첩 및 미사용 임포트 경고 해결.
- **몬스터 스폰 안정화**: 하드코딩된 이름 대신 템플릿 검색 기반의 동적 스폰 방식으로 정합성 개선.

## [1.5.0] - 2026-02-12
### Added
- **기도, 제단 및 운 시스템 (Phase 24)**:
  - **운(Luck) 및 성향(Alignment) 시스템**: `luck.c` 기반의 무작위 행운 및 보정 로직을 이식. `alignment_record` 필드를 추가하여 플레이어의 선행/악행에 따른 성향 변화를 정밀하게 관리.
  - **기도 명령 (`P`)**: `pray.c`의 `dopray()`를 이식하여 신에게 기도하는 기능을 구현. 기도 쿨다운(`prayer_cooldown`), 신의 분노, 그리고 위기 시 기적(HP 완치, 허기 해소, 상태 이상 치료) 로직 완성.
  - **제물 바치기 (`#offer`)**: 제단(Altar) 위에서 시체를 바쳐 성향 점수를 올리거나 Luck을 획득하는 상호작용 구현. `Shift+O` 키를 통해 전용 제물 선택 메뉴를 호출 가능.
  - **상태 이상 치료 (Troubleshooting)**: 기도를 통해 석화(Stoning), 슬라임화(Slimed), 실명(Blind), 질병(Sick) 등 치명적인 상태 이상을 즉시 치료하는 로직 통합.
  - **UI 업데이트**: 캐릭터 정보창(`Shift+C`)에 현재 행운 수치와 성향 레코드, 그리고 성향 유형(Lawful/Neutral/Chaotic)을 실시간으로 표시.

## [1.4.0] - 2026-02-12
### Added
- **용기 및 보관 시스템 (Phase 23)**:
  - **용기 인프라**: `large box`, `chest`, `sack` 등 다양한 보관용 아이템 추가 및 `ContainerTag` 도입.
  - **재귀적 인벤토리**: 용기 내부에 아이템을 보관할 수 있도록 `Inventory` 컴포넌트 확장 적용.
  - **루트 명령 (`#loot`)**: `Shift+L` 키를 통해 바닥에 있는 용기를 열어 내용물을 확인하는 루팅 파이프라인 구축.
  - **잠금 및 해제 메커니즘**: 상자류의 `olocked` 상태를 적용하고, 열쇠(`key`)나 락픽(`lockpick`) 사용 시 이를 해제하는 상호작용 구현.
  - **용기 내용물 UI**: 계단식 인벤토리 UI를 통해 용기 내부 아이템을 확인하고 꺼내거나(`Take Out`), 플레이어 아이템을 넣을 수 있는(`Put In`) 기능 구현.
  - **재귀적 무게 계산**: 용기 자체 무게에 내용물 무게가 합산되는 시스템 이식. `Bag of Holding`의 축복 상태에 따른 무게 경감(1/2, 1/4) 공식 완벽 재현.
  - **아이템 심볼 매핑**: 클래스별(무기, 갑옷, 도구 등) 고전적인 NetHack 심볼(`)`, `[`, `(`, `%` 등) 렌더링 시스템 완성.

## [1.3.0] - 2026-02-12
### Added
- **도구 및 상호작용 시스템 (Phase 22)**:
  - **도구 사용 엔진 (`doapply`)**: 인벤토리에서 도구를 선택하고 방향이나 대상을 지정하는 디스패처 이식.
  - **핵심 도구 구현**:
    - **곡괭이/맷독**: 벽을 파괴하거나 문을 부수어 지형을 변경하는 로직 구현.
    - **유니콘 뿔**: 실명, 혼란, 스턴 등 상태 이상을 치료하는 정화 효과 이식.
    - **청진기/카메라**: 몬스터의 상태(HP)를 청취하거나 카메라 플래시로 효과를 주는 기초 로직 구축.
    - **열쇠/락픽**: 문 잠금 해제를 위한 기초 상호작용 프레임워크 구축.
- **UI 및 UX 개선**:
  - **인벤토리 Apply 버튼**: 도구, 무기, 보석 등 '사용'이 필요한 아이템 옆에 'Apply' 버튼을 추가하여 접근성 개선.
  - **GameState 동기화**: legion ECS 시스템과 eframe UI 간의 게임 상태 동기화 인프라 강화.

## [1.2.0] - 2026-02-09
### Added
- **식사 및 영양 시스템 (Phase 21)**:
  - **식사 엔진 (`doeat`)**: 음식의 영양가 반영, 배부름 상태에서의 질식 메커니즘 이식.
  - **시체 상호작용**: 시체 식사 시 몬스터 속성에 따른 내성(Resistance) 획득 및 능력치 변화 구현.
  - **부패 시스템 (`rotting`)**: 시간이 지남에 따라 바닥이나 인벤토리의 시체가 부패하는 로직 구현.
- **가변 능력치 및 복합 상태 이상 (Phase 20)**:
  - **능력치 변동 엔진 (`adjattrib`)**: 독이나 장착물에 의한 능력치 증감 및 훈련(`exercise`) 시스템 이식.
  - **타이머 기반 상태 효과 (`timeout`)**: 석화, 슬라임화 등 지연 발동되는 복합 상태 이상의 카운트다운 및 즉사 로직 완성.

## [1.1.6] - 2026-02-09
### Added
- **텔레포트 엔진 (Phase 19.1)**:
  - **무작위 텔레포트**: 현재 층 내의 안전한 위치로 순간이동하는 시스템 구현.
  - **층간 텔레포트 (Level Teleport)**: 특정 확률이나 아이템을 통해 다른 층으로 강제 이동하는 메커니즘 이식.
- **층간 이동 및 난이도 시스템 (Phase 19.2)**:
  - **몬스터 레벨 스케일링**: 던전 깊이에 비례한 몬스터 능력치 보정 엔진 이식.
  - **계단 및 사다리 파이프라인**: 층 이동 시 플레이어 위치 보정 및 층 로드/생성 서비스 완성.

## [1.1.5] - 2026-02-11
### Added
- **수동형 반격 시스템 (Phase 17.1)**:
  - **몬스터 반격 (`passive`)**: 플레이어가 몬스터 타격 시 `AD_ACID`, `AD_FIRE`, `AD_COLD`, `AD_ELEC`, `AD_PLYS` 등 몬스터가 가진 수동형 공격이 발동되도록 구현.
  - **장갑 보호 메커니즘**: `Hands` 슬롯에 장착물이 있을 경우 산성(`AD_ACID`) 피해 및 석화(`AD_STON`) 위험으로부터 플레이어를 보호하는 로직 추가.
  - **Floating Eye 마비**: 타격 시 일정 확률로 플레이어를 마비(`PARALYZED`)시키는 특수 효과 구현.
- **특수 공격 효과 확장 (Phase 17.2)**:
  - **Cockatrice 석화**: 장갑 없이 코카트리스를 타격할 경우 즉시 석화되어 사망하는 처절한 NetHack 메커니즘 이식.

### Fixed
- **ECS 대여 충돌 해결**: `movement` 시스템과 `passive` 시스템 간의 `SubWorld` 대여 충돌을 피하기 위해 `CombatEngine::passive` 시그니처를 엔티티 및 `SubWorld` 기반으로 리팩토링하여 Borrow Checker 이슈 해결.
- **컴파일 오류 및 경고 수정**: `Health` 컴포넌트 임포트 누락 및 불필요한 `mut` 키워드 사용으로 인한 경고들을 해결하여 빌드 정합성 확보.

## [1.1.4] - 2026-02-10
### Added
- **세이브/로드 시스템 (Phase 16.1)**:
  - **World 직렬화**: Legion ECS의 `Registry`와 `Canon`을 사용하여 엔티티와 컴포넌트를 JSON으로 완벽하게 직렬화/역직렬화함.
  - **던전 상태 보존**: `DungeonManager`와 모든 층의 `Grid` 맵 데이터를 저장하여 층 이동 상태를 유지함.
  - **리소스 복구**: `turn`, `game_log`, `rng`, `identity` 등 주요 게임 엔진 리소스를 세이브 파일로부터 복제하여 게임 중단 시점부터 즉시 재개 가능.
  - **전통적 세이브 처리**: 성공적으로 로드된 후 세이브 파일을 자동 삭제하여 NetHack의 전통적인 퍼마데스(Permadeath) 메커니즘을 준수함.
- **옵션 시스템 (Phase 16.2)**:
  - **options.toml 기반 설정**: 게임 옵션을 로컬 파일에 영구 저장하고 로드하는 인프라 구축.
  - **실시간 심볼 세트 전환**: UI에서 심볼 세트를 변경하면 즉시 `options.toml`에 기록되어 다음 실행 시에도 유지됨.
  - **자동 줍기 (Autopickup)**: 설정에서 활성화 시 이동 중 선호하는 아이템(`$`, `*`, `?`, `/` 등)을 자동으로 습득하는 로직 구현.
- **UI 설정 창**: 게임 내 설정 창을 통해 옵션을 실시간으로 제어하고 저장할 수 있는 기능 추가.

### Fixed
- **명령 중복 처리**: main loop 내에서 `Command::LogHistory` 처리가 중복되어 발생하던 논리적 오류를 제거함.
- **AI 시스템 최적화**: 몬스터 AI 공격 로직 내에서 미사용 변수(`weapon_ent`)와 관련된 경고를 해결함.
- **소유권 충돌 해결**: 인벤토리 줍기 로직에서 발생하던 Borrow Checker 충돌(가변/불변 대여 간섭)을 데이터 선행 수집 방식으로 최적화하여 해결함.

## [1.1.3] - 2026-02-10
### Added
- **메시지 시스템 고도화**:
  - **중복 메시지 압축**: 연속되는 동일 메시지를 " (x5)" 형태로 압축하여 로그 가독성을 획기적으로 개선함 (`log.rs`).
  - **--More-- 프롬프트**: 한 턴에 너무 많은 메시지(8개 이상)가 발생할 경우 `--More--`를 출력하고 사용자 승인을 기다려 메시지 오버플로우를 방지함.
  - **로그 히스토리 창**: `Ctrl+P` 키로 과거의 모든 메시지 내역을 별도의 플로팅 창에서 확인할 수 있는 히스토리 기능을 추가함.
- **인벤토리 UI 혁신**:
  - **카테고리별 그룹화**: 무기, 방어구, 음식, 물약 등 아이템 클래스별로 그룹화하여 표시함 (`widgets/inventory.rs`).
  - **아이템 중복 스택**: 동일 템플릿의 아이템을 "2 rations"와 같이 하나로 합쳐서 표시하여 인벤토리 공간을 효율적으로 사용함.
  - **장착 마커 개선**: 인벤토리 내에서 현재 장착 중인 아이템에 `(wielded)`, `(in hand)`, `(being worn)` 등의 상세 마커를 추가함.
  - **무게 합계 표시**: 플레이어가 소지한 모든 아이템의 총 무게를 인벤토리 상단에 표시함.

### Fixed
- **UI 토글 정합성**: 캐릭터 상태창(`C`) 토글 버튼이 UI 닫기 버튼과 연동되지 않던 이슈 및 `Ctrl+P` 히스토리 창의 일관성 문제를 해결함.
- **아이템 클래스 정합성**: `BTreeMap` 정렬을 위해 `ItemClass` 열거형에 `Ord`, `PartialOrd` 트레이트를 구현함.

## [1.1.2] - 2026-02-09
### Added
- **아이템 데이터베이스 확장**: `items.toml`에 scimitar, elven weapons, potions, scrolls 등 20+종의 아이템을 추가하여 던전의 생동감과 몬스터 장착 다양성을 확보함.
- **상점 시스템 고도화**: `mkshop`에서 상점 타입별 특수 물품 배치 기능을 강화하고, 상점 주인(Shopkeeper) 스폰 및 타일별 상점 구역 정보를 기록하는 로직을 구현함.
- **특수 방 고도화**: Barracks(Soldiers), Morgue(Undead), Beehive(Bees), Court(Throne holders) 등 원본 NetHack 고유의 특별 방 스폰 및 환경 구성 로직을 개선함.
- **도둑질 감시 시스템**: 상점에서 미지급 물건(`unpaid`)을 들고 나갈 시 상점 주인이 즉시 적대적으로 변하여 추격하는 감시 시스템을 `shop.rs`에 추가함.

### Fixed
- **ECS 트레이트 및 접근 오류**: `mkroom.rs`에서 `entry_mut` 사용 시 필요한 `EntityStore` 트레이트 누락 및 `shop.rs` 내 가변 월드 접근 권한 이슈를 수정함.

## [1.1.1] - 2026-02-08
### Added
- **A* 경로 탐색 알고리즘 도입**: 단순 직선 추격을 넘어 장애물을 우회하여 플레이어를 추적하거나 원하는 지점으로 이동하는 지능형 경로 탄색 시스템을 `util/path.rs`에 구현함.
- **몬스터 탐욕(Greedy) AI 확장**: `COLLECT`, `GREEDY`, `JEWELS` 플래그를 가진 몬스터가 플레이어를 발견하지 못했을 때 주변의 아이템을 탐색하고 해당 위치로 이동하는 로직을 추가함.
- **몬스터 아이템 습득 시스템**: 몬스터가 아이템 위에 올라섰을 때 `NOTAKE` 플래그가 없는 경우 아이템을 자신의 인벤토리로 줍는 기능을 구현함 (Gather-Apply 패턴 사용).
- **MUSE AI 공격 확장**: 지능이 있는 몬스터가 원거리에서 지팡이(Wand of Striking, Magic Missile)를 사용하여 플레이어를 공격하는 MUSE(Monster Use of Items) 로직을 정교화함.

### Fixed
- **경로 탐색 효율 최적화**: 매 턴 모든 몬스터가 경로를 재계산하는 부하를 방지하기 위해 거리 및 상태 기반 재계산 트리거를 적용함.
- **ECS 쿼리 충돌 해결**: 몬스터 AI 루프 중 인벤토리 컴포넌트 접근 시 발생하는 Legion ECS의 동시성 대여 오류를 `pickups_to_apply` 벡터를 이용한 지연 처리 방식으로 해결함.

## [1.1.0] - 2026-02-07
### Added
- **도움말 시스템 도입**: `?` (Shift+/) 키를 눌러 상호작용 가능한 전체 명령 목록을 확인할 수 있는 도움말 전용 창을 추가함.
- **게임 상태 기반 UI 제어**: 인벤토리(`i`)와 도움말(`?`) 창이 해당 게임 상태에서만 나타나도록 변경하여 화면을 더 깔끔하게 관리할 수 있도록 함.
- **캐릭터 정보 토글**: `C` (Shift+c) 키를 통해 캐릭터 능력치 창을 토글할 수 있는 기능을 추가함.
- **몬스터 스폰 시스템 복구**: 에셋 로드 경로 오류를 수정하여 던전 내에 몬스터가 정상적으로 생성되도록 함. (Spawner 시스템 활성화)

### Fixed
- **맵 생성기 로직 강화**: 방 생성 시도 횟수를 늘리고(20->40), 방의 크기 다양성을 확보하여 보다 풍부한 던전 레이아웃을 생성하도록 개선함.
- **복도 생성 알고리즘 개선**: L-자형 복도 생성 시 무작위성을 부여하여 맵의 단조로움을 해소함.
- **에셋 로드 경로 분리**: 원본 NetHack 심볼 데이터와 프로젝트 전용 TOML 데이터(몬스터, 아이템)의 로드 경로를 분리하여 데이터가 누락되는 문제를 해결함.

### Changed
- UI 명령(Help, Inventory 등)은 게임 턴을 소모하지 않으며, 전용 창 내에서 상호작용 후 ESC 또는 해당 키를 다시 눌러 닫을 수 있음.

## [1.0.2] - 2026-02-07
### Added
- **명령 시스템 고도화 (Advanced Command System)**:
  - **Zap Action**: `z` 키로 지팡이를 발사하는 기능 추가 (방향 선택 인터페이스 포함).
  - **Help Command**: `?` 키 입력 시 주요 단축키 로그 출력.
  - **Quit Command**: `Shift+Q` 키로 게임 종료(GameOver) 처리 기능 추가.
- **턴 시스템 정교화 (Refined Turn System)**:
  - 행동 명령(이동, 공격, 아이템 사용, 수색 등) 시에만 게임 턴이 증가하도록 개선.
  - UI 명령(인벤토리 열기, 도움말, 명령 취소 등)은 턴을 소모하지 않음.

### Changed
- **맵 생성기 개선 (Improved Map Generator)**:
  - `MapGenerator`: 복도 생성 중 방의 벽과 교차하는 지점에 자동으로 '닫힌 문(Door)'을 생성하는 지능형 로직 적용.
  - `Grid`: 하드코딩된 테스트용 방 생성을 제거하고 순수 무작위 생성을 지원하도록 초기화 로직 변경.
  - 메모리 안전성 향상: `unsafe assume_init` 기반의 격자 생성을 안전한 초기화 방식으로 교체.
- **단축키 바인딩 수정 (Hotkeys Calibration)**:
  - `z`(Zap)와 `Z`(Cast) 단축키가 스왑되어 있던 것을 원본 NetHack 명세에 맞게 바로잡음.
  - `Shift+S` (Save), `Shift+Q` (Quit) 등 메타 명령 키 설정 최적화.

## [1.0.1] - 2026-02-07
### Added
- **원본 데이터 이식 (Original Data Porting)**:
  - `assets/dat/`: `rumors.tru`, `rumors.fal`, `oracles.txt`, `epitaph.txt`, `engrave.txt` 원본 파일 통합.
  - **Epitaph System**: 플레이어 사망 시 무작위 묘비명을 출력하는 기능 구현.
  - **Engraving System**: `Search` 명령으로 바닥의 무작위 글귀를 발견하는 기능 구현.
- **데이터 템플릿 확장 (Data Expansion)**:
  - **Monsters**: giant ant, killer bee, soldier ant, acid blob, little dog 등 10여 종 추가.
  - **Items**: dagger, mace, leather armor, plate mail, elven cloak 등 주요 장비 10여 종 추가.

### Changed
- `AssetManager`: 초기화 시 텍스트 데이터 파일을 자동으로 로드하도록 개선.
- `Rumors`: 하드코딩된 데이터를 제거하고 파일 기반 동적 로딩으로 전환.

## [1.0.0] - 2026-02-07
### Added
- **대화 및 내러티브 시스템 (Lore & Narrative)** (`core/systems/talk.rs`)
  - **Dialogue System**: NPC 고유 대사 보유 및 방향 선택을 통한 대화(`Talk`, `C` 키) 기능 구현.
  - **Oracle's Rumors**: 오라클 전용 소문 데이터베이스 통합 및 무작위 던전 팁/비밀 제공 로직.
  - **Quest Leader Logic**: 플레이어의 레벨과 직업(Role)을 인지하여 퀘스트 수락 여부를 결정하는 대화 분기 구현.
- **유물 시스템 (Artifact System)**
  - 유물 고유 성향(`Alignment`) 인지 로직 추가.
  - 플레이어 성향과 상충하는 유물 장착 시 데미지를 입히는 거부 반응(`Blasting`) 구현.
- **플레이어 직업(Role) 추가**: Valkyrie, Wizard 등 NetHack의 전통적인 직업군 데이터 구조 통합.

### Changed
- `audit_roadmap.md`: 모든 Phase (1~11) 완료 처리 및 정합성 감사 종료.
- **Project Status**: v1.0.0 정식 릴리즈 상태로 전환.

## [0.5.18] - 2026-02-07
### Added
- **생명 진화 시스템 (Life DNA & Evolution)** (`core/systems/evolution.rs`)
  - **Polymorph**: `Species` 컴포넌트를 통한 엔티티 형태 변환 및 타이머 기반 자동 해제 로직 구현
  - **Level Drain**: 언데드나 특수 공격에 의한 레벨 하락 시 최대 HP 및 경험치 손실 공식 이식
  - **Lycanthropy**: 인랑증(`LYCANTHROPY`) 상태 플래그 추가 및 턴당 1/80 확률로 늑대인간 변신/복구 로직 구현
- **Species Component**: 모든 플레이어 및 몬스터 엔티티에 종(Species) 정보 부여 및 스폰 시스템 통합

### Fixed
- `main.rs`: 플레이어 초기 생성 및 몬스터 스폰 시 `Species` 컴포넌트 누락 문제 해결
- 시스템 스케줄에 `evolution_tick_system` 및 `lycanthropy_tick_system` 등록

## [0.5.17] - 2026-02-07
### Added
- **트랩 시스템 (Trap System)** (`core/systems/trap.rs`)
  - Arrow, Pit, Teleport, Fire, Bear trap 등 다양한 트랩의 발동 로직 및 데미지 시스템 구현
  - `Trap` 컴포넌트를 통한 엔티티 기반 트랩 관리 및 발견 상태 제어
- **지형지물 상호작용 확장 (Interaction)**
  - **제단(Altar) & 기도(Pray)**: `#pray` 명령(`P`)을 통한 기도 시스템 및 성향(Piety) 관리, 신의 보상/응징 로직 구현
  - **왕좌(Throne) & 앉기(Sit)**: `#sit` 명령(`v`) 및 왕좌 무작위 이벤트(경험치, 마나 회복 등) 구현
  - **우물(Fountain) & 씽크(Sink)**: `q` 명령으로 바닥의 우물이나 씽크에서 물을 마시는(Drink) 상호작용 추가
- **수색 시스템 (Search System)** (`core/systems/search.rs`)
  - `s` 키를 통한 주변 수색 시 숨겨진 트랩 및 비밀문 발견 로직 통합
- **상점 시스템 기초 (Commercial System)** (`core/systems/shop.rs`)
  - 상점 구역(`shop_type`) 인식 및 상점 주인(`Shopkeeper`) 감시 AI 구현
  - 미지급 물건(`unpaid`) 관리 및 지불(`#pay`, `p` 키) 명령 파이프라인 구축
- **아이템 속성 확장**: `unpaid` 및 `price` 필드 추가를 통한 경제 시스템 기반 마련

### Changed
- **입력 맵핑**: 'p'를 Pay로, 'P'를 Pray로 변경하여 NetHack 원본 단축키 정합성 확보
- **시스템 스케줄**: `shopkeeper_update_system` 및 `trap_trigger_system` 등록

## [0.5.8] - 2026-02-06
### Added
- **방향성 마법 시전 (Directional Beam)** (`core/systems/spell.rs`)
  - `Force Bolt` 주문을 직선 궤적(Beam) 물리 판정 방식으로 변경하여 벽/몬스터 충돌 감지 구현
- **지능 기반 학습 확률 (Int-based Learning)** (`core/systems/item_use.rs`)
  - 마법서 탐독 시 플레이어의 지능(Int)에 따른 성공/실패 확률 및 패널티 시스템 이식
- **자동 회복 시스템 (Regeneration)** (`core/systems/regeneration.rs`)
  - 턴당 에너지(Pw) 및 체력(HP) 자동 회복 로직 구현 및 스케줄 등록
- **UI 데이터 동기화**: 하단 상태바의 Int/Wis/Cha 및 Pw(만나) 정보를 실제 플레이어 데이터와 연동 표시

## [0.5.0] - 2026-02-07
### Added
- **마법 시전 시스템 (Spellcasting)** (`core/systems/spell.rs`)
  - `z` 키 (Cast): 마법 주문 선택 및 시전 흐름 구현 (`WaitingForSpell` 상태)
  - 에너지(Energy/Mana) 소모 로직: 주문 레벨에 따른 에너지 감소 및 부족 시 시전 실패 처리
  - 초기 주문 효과: `Force Bolt` (주변 5칸 내 몬스터에게 데미지 적용) 구현
- **주문 습득 시스템 (Learning Spells)** (`core/systems/item_use.rs`)
  - 마법서(Spellbook) 읽기(`Read`) 기능 구현
  - 주문 습득 시 단축키(`a-z`) 자동 할당 및 `SpellKnowledge` 컴포넌트 저장
- **명령 체계 및 키 매핑 확장** (`ui/input.rs`, `main.rs`)
  - `z` (Cast)와 `Z` (Zap) 명령 명시적 분리 (NetHack 전통 준수)
  - `WaitingForSpell` 상태에서의 알파벳 단축키 입력 처리 로직 구현
  - 플레이어 초기 능력치 조정 (Int/Wis 14, Energy 10) 으로 마법 테스트 지원

### Fixed
- `core/systems/item_use.rs`, `core/systems/spell.rs`: Legion SubWorld 대여 충돌(Borrow Conflict) 해결을 위한 리팩토링 및 Gather-Apply 패턴 적용
- `main.rs`: `egui` 이벤트 루프 내 텍스트 입력 캡처 및 중괄호 구문 오류 수정


## [0.4.0] - 2026-02-07
### Added
- **NetHack 스타일 상태 표시줄 (Bottom Line)** (`main.rs`)
  - 하단 2줄 방식의 고전적인 상태창 구현 (Str/Dex/Con 및 HP/AC/Exp/Turn 표시)
  - 체력 저하 시 HP 색상 변경 (Red) 및 상태 이상 아이콘 통합
- **상단 메시지 로그 패널** (`main.rs`)
  - 로그 위치를 상단(Top)으로 이동하여 가독성 개선
  - 자동 스크롤(Stick to Bottom) 및 턴 번호 표시 디자인 적용
- **마우스 호버 툴팁 시스템** (`main.rs`)
  - 마우스 커서 위치의 타일 종류, 아이템 이름, 몬스터 정보를 실시간 툴팁으로 제공
  - 시야(FOV) 및 기억(Memorize) 여부에 따른 정보 노출 제한 로직 포함
- **장비 기반 능력치 갱신 시스템** (`core/systems/equipment.rs`)
  - 장착된 방어구의 AC 보정치를 계산하여 플레이어의 최종 AC를 갱신하는 `update_player_stats` 시스템 구현
  - Legion ECS의 Borrow 충돌을 방지하기 위한 Gather-Apply 패턴 적용

### Changed
- `main.rs`: 사이드 패널을 제거하고 레이아웃을 상단 로그 - 중앙 맵 - 하단 상태줄로 재편

## [0.3.0] - 2026-02-07
### Added
- **아이템 사용 시스템** (`core/systems/item_use.rs`)
  - `item_input` 시스템: `Quaff`, `Read`, `Wear`, `Wield`, `TakeOff`, `Eat` 명령을 처리하여 `ItemAction` 생성
  - 물약(Potion) 효과: `healing`, `extra healing` (체력 회복), `confusion` (혼란), `blindness` (실명) 구현
  - 두루마리(Scroll) 효과: `teleportation` (랜덤 위치 순간이동) 구현
  - 음식(Food) 효과: 기본 영양분 섭취(체력 2 회복) 구현
- **장비 시스템 기초** (`core/systems/equipment.rs`, `core/entity/mod.rs`)
  - `Equipment` 컴포넌트 추가: 슬롯별 장착 아이템 관리 (`HashMap<EquipmentSlot, Entity>`)
  - `EquipmentSlot` 정의: `Melee`, `Shield`, `Head`, `Body`, `Feet`, `Hands`, `Cloak`
  - `Wield`, `Wear`, `TakeOff` 명령을 통한 장착/해제 로직 구현
- **몬스터 AI 시야(LOS) 적용** (`core/systems/ai.rs`)
  - `VisionSystem::has_line_of_sight`를 활용한 시야 체크 추가
  - 몬스터가 플레이어를 볼 수 없을 경우 추격을 멈추고 배회하도록 개선
- **원거리 공격 시스템 (Throw)** (`core/systems/throw.rs`)
  - `t` 키 (Throw) 명령 및 방향 선택 흐름 구현
  - 투사체 궤적 계산 및 벽/몬스터 충돌 감지 로직 구현
  - 투사체 피격 시 데미지 적용 및 아이템 위치 이동(낙하) 처리
- **키 바인딩 및 초기 아이템** (`main.rs`)
  - `Q`, `W`, `E`, `R`, `T`, `A`, `Z`, `S` 등 주요 상호작용 키 매핑 확장
  - 플레이어 생성 시 테스트용 아이템(Long sword, Small shield, Healing Potion, Teleport Scroll) 지급

### Fixed
- `main.rs`: 플레이어 엔티티 생성 시 튜플 크기 제한 우회를 위한 컴포넌트 추가 방식 수정 (`add_component`)
- `main.rs`: `egui` 키 이벤트 처리에서 중복된 `Comma` 매핑 제거

## [0.2.0] - 2026-02-06
### Added
- **GameState 상태 머신 시스템** (`core/game_state.rs`)
  - `Normal`, `WaitingForDirection`, `Targeting`, `Inventory`, `GameOver` 상태 정의
  - `Direction` enum (8방향 + Here) 및 `DirectionAction` enum 추가
- **문 열기/닫기 시스템** (`core/systems/interaction.rs`)
  - `o` 키: 문 열기 (방향 입력 필요)
  - `c` 키: 문 닫기 (방향 입력 필요)
  - 원본 NetHack `lock.c`의 `doopen()`, `doclose()` 로직 이식
- **발차기 시스템** (`core/systems/kick.rs`)
  - `Shift+K`: 발차기 (방향 입력 필요)
  - 닫힌 문 차기로 부수기 (50% 확률)
  - 벽 차기 시 자해 메시지
  - 원본 NetHack `dokick.c`의 `dokick()` 로직 일부 이식
- **계단 시스템** (`core/systems/stairs.rs`)
  - `>` (Shift+.): 아래층으로 이동 (StairsDown/Ladder 타일 필요)
  - `<` (Shift+,): 위층으로 이동 (StairsUp 타일 필요)
  - 레벨 이동 요청 메시지 및 로그 출력 (실제 레벨 변경은 임시 구현)
- **시야 시스템 개선** (`core/systems/vision.rs`)
  - `LIT` 플래그가 있는 방(Room)에 플레이어가 위치하면 방 전체가 보임 (Lit Room)
  - `gen.rs`: 방 생성 시 `LIT` 플래그 및 방 ID(`roomno`) 할당
- **키 바인딩 확장** (`ui/input.rs`)
  - `Close` 명령 추가
  - `Cancel` (ESC) 명령 추가
- **상세 구현 계획서** (`IMPLEMENTATION_PLAN.md`)
  - NetHack 3.6.7 원본 소스 분석 결과
  - 100+ 명령어 키 바인딩 전체 목록
  - Phase 1-3 구현 로드맵
- **키 바인딩 참조 문서** (`KEYBINDINGS.md`)

### Changed
- `main.rs`: GameState 기반 명령 처리 로직 추가
- `main.rs`: `Shift+.` 및 `Shift+,` 키 바인딩 추가
- 방향 입력이 필요한 명령(o, c, K) 시 "In what direction?" 메시지 표시
- ESC로 방향 선택 취소 가능

## [0.1.0] - 2026-02-06
### Added
- 프로젝트 초기화 및 D3D 프로토콜 기반 문서 체계 구축.
- `spec.md`, `designs.md`를 바탕으로 한 기본 설계 수립.
- 다국어 지원 `README.md` 및 가이드 문서들 생성.
- 원본 NetHack 3.6.7 C 소스 코드 배치 (`nethack_original`).


# AIHack 감사 보고서 35 (`Stage 1 P7-Fix` ~ `Stage 4 P10` 중간 감사 및 시정 핸드오프)

- **문서 상태**: Active Audit Report & Remediation Handoff (`P10-Audit-Fix` -> `P11` Gate)
- **작성일**: 2026-11-27 (Asia/Seoul)
- **감사 판정**: **HOLD (조건부 보류 — `Stage 5 / P11` 착수 전 `CRIT-01` ~ `CRIT-03` 및 `MAJ-01` ~ `MAJ-06` 시정 필수)**
- **감사 기준선**:
  - 커밋 완료: `HEAD` `8766ca3` (`c00c1ea` + `8766ca3`: `Stage 1 / P7-Fix`, `Stage 2 / P8`, `Stage 3 / P9` 반영)
  - 미커밋 워킹트리: `Stage 4 / P10` (총 55개 파일 — 추적 파일 수정 51 + 신규 미추적 4)
- **적용 기준**: [AGENTS.md](../../AGENTS.md), [AI_AUDIT_DOC_STANDARD.md](../../AI_AUDIT_DOC_STANDARD.md)
- **연계 기준 문서**: [AGENTS.md](../../AGENTS.md), [CODER_HANDOFF_P7_TO_P13.md](../CODER_HANDOFF_P7_TO_P13.md), [spec.md](../../spec.md), [campaign_spec.md](../campaign_spec.md), [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md), [PROVENANCE.md](../../PROVENANCE.md)

> [!IMPORTANT]
> **작업 경계 및 D3D 운영 규칙**
> 1. 현재 워킹트리의 **미커밋 `Stage 4 (P10)` 변경분 55개 파일**을 유실·되돌림 없이 보존한 상태에서 본 시정 작업을 진행한다.
> 2. **D3D Document-First**: 규칙·계약·저장 검증·로드맵/`ADR` 변경 시 [spec.md](../../spec.md), [campaign_spec.md](../campaign_spec.md), [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md), [tasks/plan.md](../../tasks/plan.md), [tasks/todo.md](../../tasks/todo.md)를 먼저 갱신한 뒤 코드를 수정한다.
> 3. **Git 커밋·푸시·태그·외부 게시**는 사용자의 명시적 승인 전까지 수행하지 않는다.
> 4. [legacy_nethack_port_reference/](../../legacy_nethack_port_reference/AIHACK_REWRITE_VS_REPAIR_REPORT.md)는 [PROVENANCE.md](../../PROVENANCE.md)상 `Blocked` 격리 자산이므로 코드·데이터·문자열을 직접 복사하거나 `Cargo.toml`/`crates`/`apps`/`src`에서 참조·import하지 않는다.

---

## 1. 독립 감사 검증 요약 (`Verified` vs `Unverified`)

### 1.1 독립 실행 완료 게이트 (2026-11-27 실행 기준)

| 게이트 | 실행 명령 | 결과 | 비고 |
| --- | --- | --- | --- |
| 포맷 | `cargo fmt --all -- --check` | **PASS** | `exit 0` |
| 정적 분석 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **PASS** | `exit 0` |
| 전체 테스트 | `cargo test -j 2 --workspace --all-targets --locked --no-fail-fast` | **PASS** | **99개 타깃 / 544 passed, 0 failed, 0 ignored** (`stage1_fixes` 10, `stage2_p8` 7, `stage3_p9` 5, `stage4_p10` 6, `p10_cli_build` 1 포함) |
| 취약점 점검 | `cargo audit` | **PASS (경고 2)** | `exit 0`; 업스트림 unsound 경고 2건(`rustix 1.1.4` `RUSTSEC-2026-0486`, `tokio 1.52.4` `RUSTSEC-2026-0386`) |
| 의존성/라이선스 | `cargo deny check licenses bans sources` | **PASS** | `bans ok, licenses ok, sources ok` |
| 출처 게이트 R7 | `bash scripts/r7_checkpoint.sh` | **PASS** | `R7 CHECKPOINT: PASS` |
| 출처 게이트 R8 | `bash scripts/r8_checkpoint.sh` | **PASS** | `R8 CHECKPOINT: PASS` |
| 레거시 참조 격리 | `rg -n "legacy_nethack_port_reference" Cargo.toml src crates apps` | **PASS** | 0건 검출 |

### 1.2 미검증 및 제외 범위 (시정 완료 후 재감사 확인 대상)

- `cargo build --workspace --all-targets --release --locked` 및 `build.bat` / `build.sh` 패키징 검증은 본 읽기 전용 감사 및 미커밋 `Stage 4` dirty tree 상태 때문에 미실행 상태다.
- [designs.md](../../designs.md), [BUILD_GUIDE.md](../../BUILD_GUIDE.md), [IMPLEMENTATION_SUMMARY.md](../../IMPLEMENTATION_SUMMARY.md), [audit_roadmap.md](../../audit_roadmap.md), [LESSONS_LEARNED.md](../../LESSONS_LEARNED.md), `.github/workflows`, `build.bat`/`build.sh` 패키징 검증 및 `--release` 빌드는 본 회차 범위에서 제외·미실행되었으며, 향후 재감사에서 [AI_AUDIT_DOC_STANDARD.md](../../AI_AUDIT_DOC_STANDARD.md) 3-Pass 기준에 따라 확인한다.

---

## 2. 단계별 준수 현황 (`CODER_HANDOFF_P7_TO_P13.md` 대비)

| 단계 | 핸드오프 목표 | 감사 판정 | 핵심 사유 및 결함 ID |
| --- | --- | --- | --- |
| **Stage 1 (`P7-Fix`)** | `DEF-01` ~ `DEF-11` 시정 & P7 마감 | **부분 충족** | `DEF-01`~`05`, `07`, `08`은 대체로 반영되었으나, `DEF-06`(미식별 정보 누출)이 P8/P9/맵/LLM/Headless 경로에서 재발(`MAJ-03`)하고, `DEF-08`(트랜잭션 원자성)이 `reject()` 경로에서 훼손됨(`CRIT-02`). |
| **Stage 2 (`P8`)** | 데이터 주도 스키마(`P8-A`), 6능력치·상태이상·함정·속도(`P8-B`), `SaveV3`·던전 생성기 v3(`P8-C`) | **부분 충족 (구조 부채)** | `80x21` 생성기, 10슬롯 장비, 6능력치, 상태 타이머, 에너지 스케줄러, `SaveV3`는 동작한다. 단 `P8-A`의 `ItemKind`/`MonsterKind` enum 하드코딩 병목이 해소되지 않았고, `monster_resistances`·`hp_dice`·`attacks` 등 TOML 메타데이터 다수가 런타임에 연결되지 않음(`MAJ-05`, `GAP-01`). |
| **Stage 3 (`P9`)** | 도구, 컨테이너, `#dip`/연금술, `#engrave`, 분수·싱크·물·용암, 함정 확장 | **결함 존재** | P9 지형 위 몬스터 근접 공격 불가(`CRIT-03`), `ActionSpace` 64개 절단으로 이동 마비(`CRIT-01`), 컨테이너/Dip/각인 트랜잭션 누수(`CRIT-02`), 컨테이너/Dip 미식별 실명 노출(`MAJ-03`), 물/용암 정지·추락 피해 누락(`MAJ-06`). |
| **Stage 4 (`P10`)** | 13역할·5종족·3성향, 신앙/제물, 아티팩트 6종, `#enhance` 숙련도, CLI/TUI 빌드 선택 | **결함 존재 (미커밋)** | 핵심 도메인과 CLI(`--role`/`--race`/`--alignment`)는 동작한다. 단 TUI 캐릭터 생성 화면 키 충돌·미바인딩(`a/b/c/d`)·폭 클리핑(`MAJ-04`), 유니콘 뿔 종족 보정 무시(`MAJ-01`), P9 행운 `.min(3)` 회귀(`MAJ-02`), `SaveV3` 신규 필드 검증 누락(`MAJ-06`), 미커밋 상태. |

---

## 3. P0 치명적 결함 시정 명세 (`CRIT-01` ~ `CRIT-03`)

### `CRIT-01` `ActionSpace` 64개 절단으로 인한 TUI 인간 플레이어 이동 마비 및 LLM 내비게이션 소프트락

- **근거**:
  - [crates/aihack-runtime/src/observation.rs](../../crates/aihack-runtime/src/observation.rs#L140-L148): `action_space.commands`가 `legal_actions`의 앞 64개만 `.take(64)`로 절단한다.
  - [crates/aihack-runtime/src/observation.rs](../../crates/aihack-runtime/src/observation.rs#L701-L845): `playing_actions()`가 인벤토리 루프 안에서 아이템별 `Throw`(8방향), `Zap`(8방향), `Apply`(최대 8방향) 등 조합 폭발 액션을 먼저 추가한 뒤, 훨씬 뒤쪽인 `#L826-L831`에서 `Move(direction)`·`Open`/`Close`/`Kick`, `#L846` 이후 `Drop`/`Container`/`Dip`/`Identify`를 추가한다.
  - [apps/aihack-tui/src/tui/input.rs](../../apps/aihack-tui/src/tui/input.rs#L151-L158), [#L272-L292](../../apps/aihack-tui/src/tui/input.rs#L272-L292): `key_to_candidate`와 `inventory_panel_ctas`가 전체 `observation.legal_actions`가 아니라 64개로 절단된 `observation.action_space.commands`를 검사한다.
- **영향**:
  - 예컨대 `Ranger`는 1턴 시작 시점부터 이미 41개 커맨드를 소비한다. 초반에 투척 무기·완드·곡괭이 등 3~4개만 더 주워도 `Move`가 인덱스 64 밖으로 밀려나 **인간 TUI 키보드 이동(`h/j/k/l/y/u/b/n`)과 LLM 이동 제안이 모두 차단되는 완전 소프트락**이 발생한다.
- **수정 명세**:
  1. `playing_actions()`의 액션 추가 순서를 재정렬한다: 핵심 공간·생존 명령(`Move`, `Open`, `Close`, `Kick`, `Pickup`, `Descend`, `Ascend`, `EnterBranch`, `Wait`, `Drink`, `Pray`, `Search`, 기본 장비/소비 `Quaff`/`Eat`/`Read`/`Wield`/`Wear`/`PutOn`)을 앞쪽에 배치하고, 8방향 조합 폭발 명령(`Throw`, `Zap`, 방향성 `Apply`, `Cast`, `ContainerPut`/`Take`, `Dip`, `Engrave`, `Drop`)을 후순위로 보낸다.
  2. [apps/aihack-tui/src/tui/input.rs](../../apps/aihack-tui/src/tui/input.rs#L151-L292)의 `key_to_candidate`, `inventory_panel_ctas`, `command_panel_ctas`는 [play_menu.rs](../../apps/aihack-tui/src/tui/play_menu.rs)와 동일하게 `observation.legal_actions`를 기준으로 동작하도록 수정한다(`action_space.commands` 64개 상한은 LLM 의사결정 계약 용도로 분리 유지).
- **필수 회귀 테스트**:
  - 투척 아이템/완드 8개 이상(총 `legal_actions > 100`)을 소지한 캠페인 상태에서 8방향 `Move`가 `observation.action_space.commands`에 포함되고 TUI `key_to_candidate('l', &obs)`가 `Some(UiCommandCandidate::Command(CommandIntent::Move(Direction::East)))`를 반환함을 검증한다.

### `CRIT-02` 트랜잭션 원자성 훼손: `GameSession::submit`이 거부(`!outcome.accepted`) 시 부분 변이를 커밋

- **근거**:
  - [crates/aihack-runtime/src/session.rs](../../crates/aihack-runtime/src/session.rs#L143-L166): `submit()`이 `transaction.was_aborted()` 또는 `!report.is_valid()`일 때만 롤백하고, 일반 `self.reject(...)`가 반환되어 `outcome.accepted == false`인 경우에도 `*self = transaction.commit()`을 실행한다.
  - **확인된 4개 실제 손상·악용 경로**:
    1. [crates/aihack-runtime/src/systems/items.rs](../../crates/aihack-runtime/src/systems/items.rs#L607-L631), [#L981-L986](../../crates/aihack-runtime/src/systems/items.rs#L981-L986) (`items::read`): `Heal | CureStatus`만 사전 거부하고 P8 `ApplyStatus` 물약 및 P9 `PotionWater`를 누락했다. 그 결과 `consume_one_from_inventory`([L622](../../crates/aihack-runtime/src/systems/items.rs#L622))와 `world.identify_item_kind`([L624](../../crates/aihack-runtime/src/systems/items.rs#L624))를 먼저 실행한 뒤 `Err("item is not a scroll")`을 반환한다. 미식별 상태 물약에 `Read`를 호출하면 `accepted: false, turn_advanced: false`인데도 **물약이 소멸하고 무료 식별 상태가 커밋**된다.
    2. [crates/aihack-runtime/src/systems/containers.rs](../../crates/aihack-runtime/src/systems/containers.rs#L329-L366), [#L413-L417](../../crates/aihack-runtime/src/systems/containers.rs#L413-L417) (`container_take`): 아이템이 상자 내부에 있는지(`#L364`)와 인벤토리 여유 슬롯(`#L416`)을 검증하기 전에 상자 함정을 발동(함정 해제 + 2d4 HP 피해 + 독)시켜, 거부된 명령에서 피해 이벤트 없이 HP 감소·독·함정 해제가 커밋된다.
    3. [crates/aihack-runtime/src/systems/dip.rs](../../crates/aihack-runtime/src/systems/dip.rs#L102-L112), [#L180-L202](../../crates/aihack-runtime/src/systems/dip.rs#L180-L202), [#L214-L236](../../crates/aihack-runtime/src/systems/dip.rs#L214-L236): 제단 성수 축복(`3 MP` 차감)이나 물약 담그기(`consume_one_from_inventory`)를 먼저 실행한 뒤 `spawn_or_replace_single`에서 인벤토리 52슬롯 가득 참으로 실패하면, 거부된 명령에서 **MP·성수·물약이 영구 증발**한다.
    4. [crates/aihack-runtime/src/systems/engrave.rs](../../crates/aihack-runtime/src/systems/engrave.rs#L52-L56), [#L86-L88](../../crates/aihack-runtime/src/systems/engrave.rs#L86-L88): 층당 각인 한도(`MAX_ENGRAVINGS_PER_LEVEL`) 검사 전에 `WandFire` 충전량을 차감하여 거부된 명령에서 완드 충전이 소모된다.
- **수정 명세**:
  1. [crates/aihack-runtime/src/session.rs](../../crates/aihack-runtime/src/session.rs#L143-L166)의 `GameSession::submit`에서 `!outcome.accepted`이면 working copy를 커밋하지 않고 롤백하도록 기본 보장한다(저주받은 안대/반지 벗기 실패 시 `buc_known` 공개처럼 상태 변경이 의도된 동작은 턴 소비 수락으로 처리하거나 명시적 예외 경로로 분리하고, 그 외 모든 거부(`accepted == false`)는 반드시 롤백한다).
  2. 위 4개 시스템 함수를 **선검증 후변이(validate-before-mutate)** 로 재정렬한다:
     - `items::read`: 함수 진입부에서 `data.class != ItemClass::Scroll`이면 즉시 거부.
     - `containers::container_take`: `contained.contains(&item)` 확인과 인벤토리 용량(스택 가능 여부 포함) 사전 확인 후 함정/추출 실행(단, 실제 상자를 열어 함정이 터지는 것 자체가 정상적인 1턴 시도라면 `Ok`+턴 진행으로 명확히 모델링하고, 잘못된 `item` ID/인벤토리 초과 에러는 함정 이전에 차단).
     - `dip.rs`: `spawn_or_replace_single`이 새 슬롯을 요구할 때 `has_free_slot()` 여부를 MP 차감/`consume_one_from_inventory` 전에 검증.
     - `engrave.rs`: `MAX_ENGRAVINGS_PER_LEVEL` 확인 후 `WandFire` 충전 차감.
- **필수 회귀 테스트**:
  - 위 4개 경로 각각에서 거부 응답(`accepted == false`) 시 실행 전/후 `snapshot_hash()`가 완전히 동일함을 검증한다.

### `CRIT-03` P9 지형(`Fountain`, `DryFountain`, `Sink`, `Water`, `Lava`) 위 몬스터 근접 공격 불가 및 추격 몬스터 위험 타일 스폰

- **근거**:
  - [crates/aihack-runtime/src/systems/movement.rs](../../crates/aihack-runtime/src/systems/movement.rs#L103-L121): `is_walkable_or_attackable_destination`이 `Floor | Door(Open) | Trap | HiddenTrap | StairsDown | StairsUp | Shop | Altar`만 허용하고 P9 통과 가능 타일 5종(`Fountain`, `DryFountain`, `Sink`, `Water`, `Lava`, [tile.rs](../../crates/aihack-core/src/domain/tile.rs#L46-L63))을 누락했다.
  - [crates/aihack-runtime/src/session.rs](../../crates/aihack-runtime/src/session.rs#L464-L473): `submit_bump_attack`이 위 함수에 의존하므로, 분수·싱크대 위 몬스터나 물·용암 위 비행 몬스터(`Bat`, `Dragon`)는 **플레이어 근접 공격에 완전 무적**인 반면, 몬스터 AI([monster_ai.rs](../../crates/aihack-runtime/src/systems/monster_ai.rs#L320))는 `chebyshev_distance <= 1`만 검사해 플레이어를 반격한다.
  - [crates/aihack-runtime/src/systems/ascension.rs](../../crates/aihack-runtime/src/systems/ascension.rs#L72): `find_pursuit_spawn_pos`가 `tile.is_movement_passable()`만 확인하여 상승 추격 몬스터(`FireAnt`/`Troll`/`Minotaur`)를 `Water`/`Lava` 타일에 직접 스폰할 수 있다.
- **수정 명세**:
  1. `is_walkable_or_attackable_destination`이 `TileKind::is_movement_passable()`을 사용하도록 통일한다(인접 `Water`/`Lava` 타일 위 몬스터를 근접 공격할 때는 플레이어가 해당 위험 타일로 이동하지 않고 제자리에서 공격 판정만 수행한다).
  2. `find_pursuit_spawn_pos`에서 `TileKind::Water | TileKind::Lava`를 제외하고 안전한 통과 가능 타일에만 추격 몬스터를 스폰한다.
- **필수 회귀 테스트**:
  - `Fountain`, `DryFountain`, `Sink`, `Water`, `Lava` 타일에 서 있는 몬스터를 인접 타일에서 근접 범프 공격할 때 `legal_actions`에 포함되고 `submit`이 수락되며 피해를 주는지 검증한다.
  - `ascension.rs` 추격 스폰이 `Water`/`Lava` 타일을 선택하지 않음을 검증한다.

---

## 4. P1 주요 결함 시정 명세 (`MAJ-01` ~ `MAJ-06`)

### `MAJ-01` `UnicornHorn`이 P10 종족(`Race`) 능력치 보정을 무시

- **근거**: [crates/aihack-runtime/src/systems/tools.rs](../../crates/aihack-runtime/src/systems/tools.rs#L424)가 `let base = campaign.role.base_attributes();`만 계산하고 `campaign.race` 보정(`with_race_deltas`)을 누락했다.
- **영향**: 음수 종족 보정 조합(`Orc Wizard` `INT 16/WIS 13/CHA 9` 등)은 유니콘 뿔 사용 시 인간 기본치까지 공짜 영구 능력치 버프를 받고, 양수 보정 조합(`Dwarf Valkyrie` `STR 19` 등)은 드레인된 능력치를 종족 기준치로 복구하지 못한다.
- **수정·검증**: `campaign.role.base_attributes().with_race_deltas(campaign.race)`로 교체하고, 음수/양수 종족 보정 캐릭터 양쪽에서 유니콘 뿔 복구 회귀 테스트를 추가한다.

### `MAJ-02` P9 행운 상한 `.min(3)` 회귀로 P10 행운(`-13..=+13`)이 3으로 강제 삭감됨

- **근거**: [crates/aihack-runtime/src/systems/terrain.rs](../../crates/aihack-runtime/src/systems/terrain.rs#L49) (`Fountain` 행운 분기) 및 [crates/aihack-runtime/src/systems/traps.rs](../../crates/aihack-runtime/src/systems/traps.rs#L284) (`TrapKind::Magic` 행운 분기)가 `.min(3)`을 사용한다.
- **영향**: `luck = 4..=13`인 플레이어가 행운 상승 분수/마법 함정을 밟으면 행운이 `3`으로 떨어져 최대 `-10` 손실이 발생한다.
- **수정·검증**: 두 곳 모두 `aihack_core::campaign::MAX_LUCK` (`13`)으로 통일하고, `luck = 8` 상태에서 분수/마법 함정 행운 증가 시 `9`가 되는 회귀 테스트를 추가한다.

### `MAJ-03` 미식별 아이템 정보 유출(`DEF-06` 재발 — 맵 색상, 컨테이너/Dip 로그, LLM/Headless 계약)

- **근거**:
  1. **맵/관측 색상 오라클**: [crates/aihack-runtime/src/observation.rs](../../crates/aihack-runtime/src/observation.rs#L285), [#L366](../../crates/aihack-runtime/src/observation.rs#L366) 및 [apps/aihack-tui/src/tui/render_map.rs](../../apps/aihack-tui/src/tui/render_map.rs#L80)가 미식별 아이템에도 [items.toml](../../crates/aihack-content/src/data/items.toml)의 실제 `color`를 내보낸다. 현재 미식별 물약 9종과 반지 3종이 모두 서로 다른 고유 색상을 가지므로 색상만으로 실제 아이템을 1:1 역추적할 수 있다.
  2. **컨테이너 로그 실명 누출**: [crates/aihack-runtime/src/systems/containers.rs](../../crates/aihack-runtime/src/systems/containers.rs#L297-L302) (`ContainerPut`) 및 [#L429-L435](../../crates/aihack-runtime/src/systems/containers.rs#L429-L435) (`ContainerTake`)가 `aihack_content::item_display_name`을 직접 호출해 미식별 아이템을 가방에 넣다 빼는 것만으로 실제 식별명을 출력한다.
  3. **Dip/연금술 실명 누출**: [crates/aihack-runtime/src/systems/dip.rs](../../crates/aihack-runtime/src/systems/dip.rs#L295-L299)는 연금술 결과 물약을 식별 처리 없이 실명으로 출력하고, [#L378-L382](../../crates/aihack-runtime/src/systems/dip.rs#L378-L382)는 미식별 철제 반지(`RingProtection`)가 녹슬 때 실명을 출력한다.
  4. **LLM / Headless 정책의 실제 `ItemKind` 직접 열람**: [crates/aihack-ai-contract/src/observation.rs](../../crates/aihack-ai-contract/src/observation.rs#L128), [#L228](../../crates/aihack-ai-contract/src/observation.rs#L228)이 미식별 엔티티·인벤토리에 실제 `ItemKind`를 포함하고, [crates/aihack-llm/src/service.rs](../../crates/aihack-llm/src/service.rs#L55-L56)의 `LlmObservationView`와 [apps/aihack-headless/src/lib.rs](../../apps/aihack-headless/src/lib.rs#L346-L384), [#L439-L446](../../apps/aihack-headless/src/lib.rs#L439-L446)의 `SurvivalV1`이 이를 그대로 읽어 미식별 회복 물약 등을 오라클처럼 식별·사용한다.
- **수정 명세**:
  1. `appearance::appearance_for_kind(seed, kind)`에 외형별 표시 색상 매핑을 도입해, 캠페인 모드에서 미식별 아이템의 `EntityObservation.color` 및 `ItemObservation.color`는 실제 종류 색상이 아닌 **시드 외형 기반 색상**만 출력한다.
  2. `containers.rs`와 `dip.rs`의 사용자 메시지는 `appearance::display_name_for_item(...)`(또는 식별 정책과 일관된 외형 표시 함수)을 사용한다. 연금술로 새 물약이 만들어졌을 때는 외형명을 출력하거나, 설계상 합성이 식별을 동반한다면 `identify_item_kind`와 함께 명시적으로 문서화한다.
  3. `LlmObservationView` 구성 시 미식별 아이템의 `kind` 오라클을 마스킹하거나 `appearance`/클래스 정보만 직렬화하도록 계약(`observation_does_not_expose_unidentified_kinds`)을 강화하고, `HeadlessPolicy::SurvivalV1`은 `item.identified == true`인 경우에만 해당 종류 효과를 전제로 사용하도록 제한한다(`ai_api_schema` / `headless_contract` 회귀 영향 확인 및 필요 시 픽스처/테스트 동기화).
- **필수 회귀 테스트**: 미식별 물약/반지/두루마리에 대해 맵·관측 색상, 컨테이너 넣기/빼기 로그, 물 Dip 녹슬기 로그, LLM 요청 JSON에 실제 `ItemKind`/식별 이름이 누출되지 않음을 검증한다.

### `MAJ-04` TUI 캐릭터 생성 화면(`P10`) 키 충돌·미바인딩·폭 클리핑·하드코딩 스탯

- **근거**:
  - [apps/aihack-tui/src/tui/render_panels.rs](../../apps/aihack-tui/src/tui/render_panels.rs#L566-L590): `[a] Barbarian`, `[b] Ranger`, `[c] Monk`, `[d] Archeologist`와 `[A] Alignment`를 안내하지만, 각 줄 길이가 최대 **88컬럼**에 달해 `MIN_TERMINAL_WIDTH = 60` 및 `80x24` 터미널에서 우측 역할들이 잘린다.
  - [apps/aihack-tui/src/tui/mod.rs](../../apps/aihack-tui/src/tui/mod.rs#L2081-L2143): `KeyCode::Char('a' | 'A')`가 Alignment 순환으로 가로채져 `[a] Barbarian` 선택이 불가능하고 `'b'`, `'c'`, `'d'`는 아예 바인딩되어 있지 않다. 또한 `creation_role.is_some()` 상태에서 `'1'..='9'`를 누르면(`#L2131-L2140`) `Tab`처럼 종족/성향 호환성을 자동 보정하지 않고 그대로 `StartCampaignWithBuild`를 제출해 조용히 거부될 수 있다.
  - [apps/aihack-tui/src/tui/render_panels.rs](../../apps/aihack-tui/src/tui/render_panels.rs#L598-L599): 선택된 역할/종족과 무관하게 `"HP: 16/16"`, `"AC: 10"`을 고정 출력한다.
  - [apps/aihack-tui/src/tui/mod.rs](../../apps/aihack-tui/src/tui/mod.rs#L1873-L1906): 마우스 CTA가 구형 `[1]..[4]`만 처리한다.
- **수정 명세**:
  1. `character_creation_lines()`가 `60x24` 최소 해상도 내부(모든 줄 `<= 58` 문자, 전체 높이 `<= 22`줄)에 완전히 들어가도록 역할 목록을 2열 또는 컴팩트 다단 레이아웃으로 재구성한다.
  2. 키 바인딩 충돌을 제거한다: 예컨대 역할 선택 단축키(`1..=9` + `v/w/g/...` 또는 `Tab`/`Shift+Tab`)와 종족(`R`)/성향(`L`/`A` 대소문자 구분 또는 전용 키)이 절대 겹치지 않게 정리하고, 역할 변경 시 허용되지 않는 종족/성향은 즉시 유효 기본값으로 자동 정렬한다.
  3. 선택된 `(role, race, alignment)` 기준의 실제 시작 HP, AC, 6대 능력치를 동적으로 표시하고, 마우스 클릭 CTA가 표시된 모든 역할·종족·성향·확인·취소 라벨과 1:1 일치하도록 동기화한다.
- **필수 회귀 테스트**: `60x24` 버퍼에서 모든 13역할/5종족/3성향 선택 라벨과 동적 HP/AC가 잘림 없이 렌더링되고, 키보드 및 마우스 CTA로 13개 역할 전부를 선택해 게임을 시작할 수 있음을 검증한다.

### `MAJ-05` `P8-A` 데이터 주도 아키텍처 미완 및 TOML 메타데이터 런타임 사문화

- **근거**:
  - [CODER_HANDOFF_P7_TO_P13.md](../CODER_HANDOFF_P7_TO_P13.md) §5.1의 핵심 목적은 `P9~P13` 대량 콘텐츠 추가 전 Rust enum 병목을 제거하는 것이었으나, [crates/aihack-core/src/domain/item.rs](../../crates/aihack-core/src/domain/item.rs)와 [crates/aihack-core/src/domain/monster.rs](../../crates/aihack-core/src/domain/monster.rs)는 여전히 하드코딩 enum이며 [schema.rs](../../crates/aihack-content/src/schema.rs), [crates/aihack-content/src/lib.rs](../../crates/aihack-content/src/lib.rs), [entity.rs](../../crates/aihack-core/src/domain/entity.rs), [save.rs](../../crates/aihack-runtime/src/save.rs)에 수동 매핑 테이블이 중복 존재한다.
  - `rg` 검증 기준, TOML에 추가된 다수 필드가 런타임에 전혀 소비되지 않는다:
    - `aihack_content::monster_resistances()`는 [tests/stage2_p8.rs](../../tests/stage2_p8.rs#L198-L206)에서만 호출되며 런타임 호출자가 **0건**이다. [entity.rs](../../crates/aihack-core/src/domain/entity.rs#L572)에서 몬스터 생성 시 `resistances: Resistances::NONE`으로 고정되고, [projectiles.rs](../../crates/aihack-runtime/src/systems/projectiles.rs#L181-L200)의 `WandFire`도 몬스터 화염 저항을 검사하지 않아 **`Dragon`과 `FireAnt`가 화염 완드에 100% 피해**를 입는다.
    - [session.rs](../../crates/aihack-runtime/src/session.rs#L997)의 HP 자연 회복 주기 계산이 `campaign.attributes.con_regen_reduction()` 대신 `con_hp_per_level_bonus()`를 잘못 호출한다.
    - `damage_small`/`damage_large`, `hp_dice`, `attacks[]`, `unidentified_pool`, `spawn_depth_range`, `spawn_weight`, `ai_flags` 역시 런타임 미소비 상태다.
- **수정 명세 (2단계 구분)**:
  1. **즉시 시정 (`P11` 진입 전 필수)**:
     - `spawn_monster_with_template` 및 부트스트랩/몬스터 생성 경로에서 `monster_resistances`를 `stats.resistances`에 주입하고, `WandFire`·원소/독 피해 경로가 대상 몬스터의 `Resistances`를 검사·경감(또는 면역 처리)하도록 연결한다.
     - `session.rs#L997`을 `con_regen_reduction()`으로 수정한다.
     - 캠페인 동적/고정 스폰 또는 향후 스폰 테이블이 `spawn_depth_range`·`spawn_weight`를 소비하는 경로를 정리한다.
  2. **`P12` 스케일업 대비 아키텍처 확정 (`DESIGN_DECISIONS.md` `ADR` 필수)**:
     - `P12`(몬스터 80~150+, 아이템 120~200+, 주문 25~40) 진입 시 하드코딩 enum 매치 테이블 수동 추가를 막기 위해, (a) `ContentId` / 문자열·인덱스 ID 기반 런타임 데이터 주도 모델로 전환할 것인지, 아니면 (b) TOML → `build.rs` 코드 생성 + 데이터 기반 디스패치로 enum 수동 복제를 없앨 것인지 설계를 `ADR`로 확정하고 사문화 필드(`damage_small`/`damage_large`, `hp_dice`, `attacks[]`, `unidentified_pool`, `ai_flags`) 소비 단계를 명시한다.

### `MAJ-06` `SaveV3` 무결성 검증 누락 및 런타임 규칙 엣지 케이스 3건

- **근거**:
  1. **SaveV3 P10 신규 필드 검증 누락** ([crates/aihack-runtime/src/campaign.rs](../../crates/aihack-runtime/src/campaign.rs#L73-L162), [crates/aihack-runtime/src/save.rs](../../crates/aihack-runtime/src/save.rs#L623-L780)):
     - `campaign.god_anger`(`i16` 비정상 값 미검증), `campaign.prayer_timeout` 상한, `campaign.artifact_invoke_cooldown`(최대 25 초과 허용), `world.learned_spells` 중복/길이 상한, `world.ascension_altars` 길이·`LevelId`·좌표 범위, 캠페인 모드 `actor.monster_inventory` 내 존재/중복/아이템 타입 검증, 비시체 아이템의 `corpse_monster` 존재 차단, 아티팩트 `erosion > 0` 차단 검증이 빠져 있다.
  2. **치명적 `TrapKind::Hole` 사망 시 층 이동 후 사망**: [crates/aihack-runtime/src/systems/traps.rs](../../crates/aihack-runtime/src/systems/traps.rs#L97-L125)가 피해 후 `player_alive()` 확인 없이 `set_player_location`과 `LevelChanged`를 실행하여 HP가 0인 플레이어가 아래층으로 떨어진 뒤 사망 처리된다.
  3. **고속 몬스터 에너지 과축적**: [crates/aihack-runtime/src/systems/monster_ai.rs](../../crates/aihack-runtime/src/systems/monster_ai.rs#L72-L102)에서 `MonsterIntent::Wait` 시 `ENERGY_THRESHOLD(12)`만 차감하고 `break`하기 때문에 `speed > 12` 몬스터(예: `Bat = 24`)가 대기·차단 상태에서 에너지를 최대 `72`까지 축적했다가 시야 확보/인접 시 한 턴에 최대 4회 연속 공격(알파 스트라이크)할 수 있다.
  4. **물/용암 정지 및 부유 만료 시 피해 누락**: [crates/aihack-runtime/src/systems/terrain.rs](../../crates/aihack-runtime/src/systems/terrain.rs#L232-L377)의 `Water`/`Lava` 위험 판정이 `submit_move`([session.rs](../../crates/aihack-runtime/src/session.rs#L505))에서만 호출되어, 용암 위에서 제자리 대기(`.`)하거나 물/용암 위에서 `Levitation`이 만료되어 추락해도 0 피해로 통과된다.
- **수정 명세 & 회귀 테스트**:
  - 위 `SaveV3` 필드 전부의 유효 범위·유일성·참조 무결성을 fail-closed 검증에 추가하고 각 항목별 부정(tamper) 저장 테스트 작성.
  - `TrapKind::Hole` 피해 후 `!world.player_alive()`이면 아래층 이동·`LevelChanged` 없이 즉시 종료.
  - `MonsterIntent::Wait` 시 잔여 에너지를 `ENERGY_THRESHOLD`(또는 `effective_gain` 이내)로 상한 클램프해 대기 중 무한 에너지 비축 차단.
  - 턴 진행(`Wait` 등 포함) 및 `Levitation` 만료 직후 현재 타일이 `Water`/`Lava`이면 지형 위험 판정 호출.

---

## 5. P2 경미 결함 시정 명세 (`MIN-01` ~ `MIN-07`)

1. **`MIN-01` (`PutOn` 프롬프트 목걸이 누락)**: [crates/aihack-runtime/src/observation.rs](../../crates/aihack-runtime/src/observation.rs#L566)의 `InventoryAction::PutOn` 분기가 `ItemClass::Ring`만 허용하고 `ItemClass::Amulet`을 누락했다(`#L709`와 불일치). `ItemClass::Ring | ItemClass::Amulet`으로 통일한다.
2. **`MIN-02` (TUI 색상 테이블 누락)**: [apps/aihack-tui/src/tui/render_map.rs](../../apps/aihack-tui/src/tui/render_map.rs#L125-L140)의 `entity_color()`에 [items.toml](../../crates/aihack-content/src/data/items.toml)이 사용하는 `bright_yellow`, `bright_cyan`, `bright_magenta`, `orange`, `black` 매핑을 추가한다(`AmuletAscension`, `Katana`, 물약 3종 등).
3. **`MIN-03` (TUI 메뉴 엣지 케이스)**:
   - [apps/aihack-tui/src/tui/play_menu.rs](../../apps/aihack-tui/src/tui/play_menu.rs#L274-L300): `Apply`(`a`) 메뉴에서 `SkeletonKey`/`LockPick` 소지 시 인접 문(`direction: Some`)과 상자(`direction: None`)가 동시에 있으면 `if has_dir` 분기가 상자 해제 선택을 가린다. 둘 다 선택 가능하도록 항목을 분리한다.
   - [apps/aihack-tui/src/tui/play_menu.rs](../../apps/aihack-tui/src/tui/play_menu.rs#L648-L662): `item_menu`에 `'V'`(`Invoke`) 항목을 추가해 아티팩트 발동을 아이템 메뉴에서도 실행 가능하게 한다.
4. **`MIN-04` (아티팩트 규칙 일관성)**:
   - [crates/aihack-runtime/src/systems/artifacts.rs](../../crates/aihack-runtime/src/systems/artifacts.rs#L269-L292), [#L333-L353](../../crates/aihack-runtime/src/systems/artifacts.rs#L333-L353): `Stormbringer` `#invoke` 대상 탐색에 시야선(LOS) 검사를 추가해 벽 너머 적 공격을 차단한다.
   - [crates/aihack-runtime/src/systems/containers.rs](../../crates/aihack-runtime/src/systems/containers.rs#L306): `container_take`로 적대 성향 아티팩트를 꺼낼 때도 `Pickup`/`Wield`/`Invoke`와 일관되게 `ArtifactBlast`를 적용한다.
5. **`MIN-05` (투척 무기 보너스 미반영)**: [crates/aihack-runtime/src/systems/projectiles.rs](../../crates/aihack-runtime/src/systems/projectiles.rs#L106-L131)의 `Throw` 계산에 무기 `enchantment`, `erosion`, `buc`, `dex_hit_bonus`, `luck`, 아티팩트 보너스를 반영한다.
6. **`MIN-06` (데이터·UI·에러 처리 소결함 일괄 정리)**:
   - `FloatingEye` 난이도 일치: [monster.rs](../../crates/aihack-core/src/domain/monster.rs#L75) (`5`) vs [monsters.toml](../../crates/aihack-content/src/data/monsters.toml#L49) (`3`) 동기화.
   - [crates/aihack-runtime/src/systems/shop.rs](../../crates/aihack-runtime/src/systems/shop.rs#L94-L108): 상점 로그에 Rust `Debug` 이름(`PotionHealing`)이나 `AmuletEsp`의 `"unidentified item"` 대신 `appearance`/표시명 출력.
   - [apps/aihack-tui/src/tui/render_panels.rs](../../apps/aihack-tui/src/tui/render_panels.rs#L435-L450): `event_line`이 `GameEvent::Message` 등을 `format!("{event:?}")`로 원시 Debug 출력하지 않도록 명시 매치 추가.
   - [crates/aihack-runtime/src/systems/traps.rs](../../crates/aihack-runtime/src/systems/traps.rs#L261) & [terrain.rs](../../crates/aihack-runtime/src/systems/terrain.rs#L124): `EntityAllocationError`를 `let _ =` / `if let Ok(...)`로 삼키지 말고 상위로 전파해 트랜잭션을 안전하게 중단.
   - [crates/aihack-runtime/src/bootstrap.rs](../../crates/aihack-runtime/src/bootstrap.rs#L484), [#L550](../../crates/aihack-runtime/src/bootstrap.rs#L550): Mines 1 `Dagger`가 `Shop` 타일과 겹치거나 Mines 2 `RingProtection`이 `Altar` 타일과 겹치지 않도록 인접 안전 `Floor` 좌표로 이동(또는 `Shop`/`Altar` 비충돌 검증 추가).
7. **`MIN-07` (문서·데이터 소불일치 동기화)**:
   - [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md#L20): `ADR-P10-01` 본문의 `god_anger: u8` 표기를 실제 구현인 `god_anger: i16`([campaign.rs](../../crates/aihack-core/src/campaign.rs#L997))과 일치시킨다.
   - [crates/aihack-content/src/data/items.toml](../../crates/aihack-content/src/data/items.toml#L174): `ARTIFACTS` 테이블에 없는 미사용 `artifact_id = "amulet_of_ascension"` 속성을 정리하거나 용도를 명확히 일치시킨다.
   - [campaign_implementation.md](../campaign_implementation.md#L171): `10슬롯` 설명 문장에 11개 슬롯 이름이 나열된 표현을 실제 `EquipmentSlot` 구성과 정확히 일치하도록 수정한다.

---

## 6. 의도적 범위 축소 및 이월 항목 (`GAP-01`)

아래 항목은 구현 결함이라기보다 단계 간 범위 이월 또는 설계 판단 사항이다. 문서와 로드맵에 현재 상태와 후속 이월 단계를 명시적으로 동기화한다.

1. **던전 깊이(`Main 1..6` + `Mines 1..2`, 8층) 유지**: [CODER_HANDOFF_P7_TO_P13.md](../CODER_HANDOFF_P7_TO_P13.md) §5.3의 `Main 1..20+` / `Mines 1..5+` 확장은 현재 8층 활성 + `Sokoban`/`Quest`/`Gehennom` `max_depth = 0` 예약 상태다. [campaign_spec.md](../campaign_spec.md) 및 [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md)에 최종 확장 목표 단계(`P11`/`P13`)를 명시한다.
2. **레벨 상한(`MAX_XP = 180`, Lv 10) 및 `TrapKind::Polymorph` 간소화**: 현재 Lv 10 상한과 변신 대신 상태이상(`hallucination`/`confused`/`-1 STR`) 적용 상태가 향후 어느 단계(`P12` 등)에서 확장되는지 문서에 명시한다.
3. **`SaveV2` 캠페인 세이브 명시적 비호환 (`ADR-P8-01`)**: `0.3.0` 미릴리스 상태의 의도적 스키마 전환(`expected: 3, actual: 2`)이므로 유지하되, 문서상 비호환 정책이 일관됨을 재확인한다.
4. **워킹트리 커밋 마감 대기**: 본 시정 완료 후 사용자 승인을 받아 `Stage 4 (P10)` + `P10-Audit-Fix`를 하나의 깔끔한 기준선으로 커밋한 뒤 `Stage 5 (P11)`에 진입하는 것을 권장한다.

---

## 7. 권장 실행 순서 (`W1` ~ `W4`) 및 `Stage 5 (P11)` 진입 완료 게이트

### 7.1 단계별 작업 순서

- **`W1` (P0 차단 결함 우선 해소)**:
  - `CRIT-01` (`playing_actions` 정렬 + TUI `input.rs` `legal_actions` 전환)
  - `CRIT-02` (`GameSession::submit` `!outcome.accepted` 롤백 보장 + `items::read`/`containers`/`dip`/`engrave` 선검증 후변이)
  - `CRIT-03` (`is_walkable_or_attackable_destination` + `ascension::find_pursuit_spawn_pos`)
  - `MAJ-01` (`UnicornHorn` 종족 보정), `MAJ-02` (행운 상한 `MAX_LUCK`)
- **`W2` (P1 계약·보안·UX·무결성)**:
  - `MAJ-03` (미식별 색상/컨테이너/Dip/LLM/Headless 정보 유출 차단)
  - `MAJ-04` (TUI 캐릭터 생성 `60x24` 레이아웃, 키 충돌/미바인딩, 동적 스탯, 마우스 CTA)
  - `MAJ-06` (`SaveV3` P10 필드 검증 + `Hole` 사망 순서 + 고속 몬스터 에너지 클램프 + `Water`/`Lava` 대기/부유 해제 피해)
- **`W3` (P1.5 데이터 주도 부채 정리)**:
  - `MAJ-05` 중 즉시 항목(`monster_resistances` 스폰/화염 저항 연결, `con_regen_reduction` 연결, `spawn_depth_range`/`spawn_weight` 소비) 구현 및 `P12` 대규모 도감 전 데이터 주도 전환 `ADR` 확정.
- **`W4` (P2 마감 및 문서/릴리스 검증)**:
  - `MIN-01` ~ `MIN-07`, `GAP-01` 문서 동기화 반영.
  - **TOML/콘텐츠 해시 동기화 규칙**: `items.toml` 또는 `monsters.toml`이 1바이트라도 변경되면 반드시 [docs/provenance/r7-content.sha256](../provenance/r7-content.sha256) (LF 줄바꿈 유지), [PROVENANCE.md](../../PROVENANCE.md), [crates/aihack-content/tests/registry.rs](../../crates/aihack-content/tests/registry.rs) golden content hash를 함께 갱신한다.

### 7.2 `Stage 5 (P11)` 진입 전 필수 통과 게이트

아래 명령이 모두 **0 실패 / 0 경고**로 통과해야 `Stage 5 (P11)` 진입 가능 상태(`PASS`)로 판정한다:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -j 2 --workspace --all-targets --locked --no-fail-fast
cargo build --workspace --all-targets --release --locked
cargo audit
cargo deny check licenses bans sources
```

추가 셸/격리 게이트 (Git Bash):

```bash
bash scripts/r7_checkpoint.sh
bash scripts/r8_checkpoint.sh
rg -n "legacy_nethack_port_reference" Cargo.toml src crates apps   # 반드시 0건
```

모든 게이트 통과 후 [tasks/todo.md](../../tasks/todo.md), [campaign_implementation.md](../campaign_implementation.md), [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md) 마감 상태를 업데이트하고 감사자(또는 사용자) 확인을 거쳐 [CODER_HANDOFF_P7_TO_P13.md](../CODER_HANDOFF_P7_TO_P13.md)의 `Stage 5 (P11)`로 진입한다.

---

## 8. Coder Handoff

```text
Read the latest audit findings in `docs/audit/audit_report_35.md`.
Compare each finding with the relevant project documents and actual code, then make the necessary corrections.
If a contract must change, update its documents first. After correcting the implementation, run relevant tests and verification and record the results.
```

# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: 보안·파일/경로·저장·네트워크 신뢰 경계(file_security)
- User Goal: 현재 프로젝트의 모든 문서 및 구현내용을 파악 후 프로젝트의 문제점을 진단하여 모순 및 문제점을 해결할 수 있게 감사 진행
- Audit Basis: Standard-backed
- Standard Path: `C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md`
- Report Contract: `C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md`
- 감사 시점: 2026-08-23 (Asia/Seoul)

## 2. Assigned Scope

다음 실행 경로와 경계를 독립적으로 대조했다.

- `ArtifactStore`의 root capability 초기화, 상대 경로 검증, final symlink 및 hard-link 처리, nested directory, TOCTOU, 임시 파일, 권한, atomic replace
- headless CLI의 `--save`, `--load`, `--replay-in`, `--replay-out`, `--report` 경로와 실패 report
- TUI의 quick-save 생성·저장·load 호출 경로와 공개 `TuiClient`/`TuiApp` path API
- SaveDataV1 load와 ReplayLineV1 JSONL parsing, replay 검증 및 state/hash/revision 연계
- local LLM 설정·endpoint DNS/loopback 제한·redirect/proxy·request/response body limit·control character·revision/action gate
- runtime/app 호출 경로, 관련 테스트, 보안 문서·ADR·BUILD 문서, shell/Python 검증 스크립트의 path/shell 경계

## 3. Excluded and Uninspected Scope

- 일반 게임 규칙·밸런스·AI 정책·UI 미관은 조사하지 않았다.
- `target/`, `.git/`, `legacy_nethack_port_reference/`, generated output은 실행·출처 인벤토리의 참고 대상에서 제외했다.
- 다른 `docs/multi_audit/` 하위 감사 보고서는 독립 판정을 위해 읽지 않았다.
- 외부 네트워크의 실제 LLM provider는 호출하지 않았다. 저장소가 제공하는 loopback fixture와 local mock만 사용했다.
- 별도 OS 계정 간 ACL 공격과 전원 장애 중 파일시스템 복구는 현재 환경에서 재현하지 못했다.

## 4. Evidence Examined

### 문서 및 설정

- `spec.md` §9.4–9.5, §14, §16
- `BUILD_GUIDE.md` §7 CLI 표와 path/permission 경계
- `DESIGN_DECISIONS.md` ADR-0032(capability save/replay) 및 ADR-0026(local LLM)
- `designs.md` §7, §10, §12 (LLM 및 TUI quick-save 경계)
- `audit_roadmap.md` R5/R6 실패 matrix 및 security gate
- `Cargo.toml`, 각 workspace manifest, `Cargo.lock`

### 구현

- `crates/aihack-runtime/src/save.rs`
- `crates/aihack-runtime/src/world.rs`, `src/session.rs`, `src/observation.rs`
- `crates/aihack-core/src/save.rs`, `src/event.rs`, `src/invariant.rs`
- `apps/aihack-headless/src/main.rs`, `src/lib.rs`
- `apps/aihack-tui/src/tui/mod.rs`, `src/tui/render_panels.rs`
- `crates/aihack-llm/src/config.rs`, `transport.rs`, `service.rs`, `decision.rs`, `soft_adjudication.rs`
- `scripts/r6_loopback_fixture.py`, `scripts/r6_pty_matrix.sh`, `scripts/r6_pending_exit_smoke.sh`, `scripts/r7_checkpoint.sh`, `scripts/r8_checkpoint.sh`, `scripts/verify_release_bundle.sh`, `build.sh`, `build.bat`

### 테스트 및 명령

- `cargo test --workspace --locked --test headless_paths --test llm_transport --test llm_revision_gate --test llm_soft_adjudication --test llm_tui_integration`
  - `headless_paths`: 6 passed
  - `llm_revision_gate`: 9 passed
  - `llm_soft_adjudication`: 5 passed
  - `llm_transport`: 22 passed
  - `llm_tui_integration`: 10 passed
- `tests/headless_paths.rs`, `tests/world_invariants.rs`, `tests/ui_runtime_smoke.rs`, `tests/replay.rs`, `tests/save_load.rs`, `tests/llm_transport.rs`, `tests/llm_revision_gate.rs`, `tests/llm_soft_adjudication.rs`
- `rustc -vV`, `cargo -V`, `cargo tree -p aihack-llm --locked`
- `rg` 기반 production `unsafe`, shell execution, path helper, no-follow 및 response gate 정적 검색
- 안전한 임시 디렉터리에서 headless 실행 파일을 사용한 root junction, malformed SaveDataV1, forged ReplayLineV1 음성 검증

## 5. Verified Security Controls

아래 항목은 표적 테스트와 코드 대조에서 현재 통과했다. 이 결과는 아래 finding을 상쇄하지 않는다.

| 경계 | 확인 결과 |
| --- | --- |
| 상대 path | `validate_relative_path`가 absolute/parent/prefix를 거부하고 `tests/headless_paths.rs`의 3개 assertion이 통과했다. |
| final symlink 및 사전 배치 hard link | final symlink, destination hard link, predictable temp hard link, replay hard link가 외부 victim을 변경하지 않는 테스트를 통과했다. |
| nested parent junction | Windows 임시 root 안의 `saves` junction을 외부 디렉터리로 연결한 실행은 `io error: a path led outside of the filesystem`, exit 2가 되었고 외부 파일은 생성되지 않았다. root 자체 junction은 별도 finding이다. |
| save replacement 및 파일 mode | regular destination replacement와 Unix/Windows permission contract 테스트가 통과했다. |
| LLM endpoint | `http`·명시 port·loopback host·userinfo/query/fragment 차단, DNS resolve 결과 loopback 확인, `Policy::none()`, `no_proxy()`, resolved address pinning이 코드와 22개 transport 테스트에서 확인됐다. |
| LLM body/control/revision | request 32,768 bytes, response 65,536 bytes, C0/C1/ANSI, strict wire schema, action-space 및 stale revision gate가 표적 테스트에서 통과했다. |
| production shell/unsafe | runtime/app production Rust에서 `unsafe`와 shell command 실행을 찾지 못했다. CI/build script의 외부 명령은 고정 경로·고정 인자 중심이며, 사용자 입력을 shell fragment로 조합하는 production path는 확인하지 못했다. |

## 6. Findings

### [A03-F001] `ArtifactStore::open`이 root junction/symlink를 거부하지 않아 CLI capability root가 외부로 이동함

- Area: 파일 root trust, CLI save/replay/report 경계
- Pass: Security
- Pattern: SEC-003 / SEC-007 (비루프백·외부 root 노출 및 local artifact 경계)
- Severity: Major
- Status: Confirmed
- Summary: 최종 파일 path의 symlink는 검사하지만 `ArtifactStore` 자체의 root path는 `create_dir_all`과 ambient open에 그대로 넘긴다. 따라서 `cwd/runtime`가 Windows junction(또는 Unix root symlink)으로 바뀌어 있으면 capability가 외부 대상 디렉터리를 root로 열고 모든 산출물을 그곳에 성공적으로 기록한다.
- Evidence:
  - `crates/aihack-runtime/src/save.rs:27-30`은 `fs::create_dir_all(root)` 후 `Dir::open_ambient_dir(root, ambient_authority())`를 호출하며 root 자체의 `symlink_metadata`/nofollow 검사가 없다.
  - `apps/aihack-headless/src/main.rs:38-56`은 `current_dir()/runtime`을 만들고 `ArtifactStore::open`에 전달한다. `:147-166`은 같은 store로 replay/save/report를 기록한다.
  - 재현 명령: 안전한 임시 cwd에서 `runtime`을 `outside`로 가리키는 Windows junction으로 만든 뒤 `target/debug/aihack-headless.exe --seed 42 --turns 0 --save saves/escape.json` 실행. 결과는 `exit=0`, `outside/saves/escape.json` 존재였다.
  - 반대로 nested `runtime/saves` junction은 cap-std가 `a path led outside of the filesystem`으로 거부했다. 즉 문제는 root 초기화 지점에 국한된다.
  - 기존 `tests/headless_paths.rs:202-218`은 root 내부 final file symlink만 검증하고 root symlink/junction case는 검증하지 않는다.
- Expected Basis: `spec.md:727-729`, `BUILD_GUIDE.md:251`, `DESIGN_DECISIONS.md:56-59`의 “열린 runtime root capability”, “root 밖 symbolic link 거부”, “모든 open/rename을 root handle 기준”이라는 hard boundary와 사용자 요청의 root/symlink 경계 질문.
- Actual: `ArtifactStore::open`의 ambient root open이 junction target을 승인한다. 이후 상대 path 검증은 이미 잘못 열린 root 안에서만 수행되므로 외부 대상이 정상적인 root처럼 보인다.
- Impact: runtime 디렉터리를 교체할 수 있는 공격자 또는 오용된 실행 위치가 save/replay/report 전체를 임의 외부 디렉터리로 redirect할 수 있다. elevated/공유 workspace에서 victim 파일 덮어쓰기와 artifact 유출로 확대될 수 있다. 문서의 “root 밖 symlink 거부” 주장은 현재 구현보다 강하다.
- Suggested Action: root 자체를 no-follow로 여는 API/플랫폼 경계를 사용하고, Windows reparse-point/junction 및 Unix symlink root를 open 전에 fail-closed로 거부한다. 가능하면 caller가 신뢰된 directory handle을 제공하도록 `ArtifactStore` 생성 계약을 바꾼다. Windows junction, Unix root symlink, root 교체 race를 실제 negative test로 추가하고 외부 파일 미생성을 확인한다.
- Re-audit Method: 동일한 temporary `runtime -> outside` junction/symlink fixture로 `ArtifactStore::open`이 오류를 반환하는지 확인하고, CLI의 save/report/replay 각 경로를 별도로 실행한다. root handle 획득 후 path 교체가 일어나도 기존 handle 안에만 기록되는지 함께 확인한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: 파일 내부 final symlink와 nested parent escape는 통과했으므로 cap-std 사용 자체가 무효라는 finding이 아니다. root trust bootstrap이 닫히지 않은 국소 결함이다.

### [A03-F002] SaveDataV1 load가 schema version만 검사하고 world invariant를 검증하지 않아 malformed save가 panic을 유발함

- Area: save load, persisted state integrity, fail-closed parsing
- Pass: Security
- Pattern: SEC-005 (persisted-state hard boundary 과대주장)
- Severity: Major
- Status: Confirmed
- Summary: `load_session`은 JSON deserialize와 schema version만 확인한 뒤 invalid world를 그대로 `GameSession`에 넣는다. 잘못된 `current_level`, player entity, actor location 등의 save가 typed error로 거부되지 않으며 이후 runtime의 `expect`/debug assertion에서 프로세스가 종료된다.
- Evidence:
  - `crates/aihack-runtime/src/save.rs:81-87`은 payload를 무제한 문자열로 읽고 `serde_json::from_str` 후 `GameSession::from_save_data`를 호출한다.
  - `:193-209`의 `from_save_data`는 `schema_version`만 검사하고 `GameWorld::from_saved_world(save.world)`를 직접 저장한다.
  - `crates/aihack-runtime/src/world.rs:223-227`의 `from_saved_world`는 validation 없이 `saved.into()`만 수행한다. `:148-151`의 `player_pos`는 actor/map invariant를 전제로 하며 `debug_assert_eq`를 포함한다.
  - `tests/world_invariants.rs:19-92`는 같은 종류의 invalid world를 만들 수 있음을 증명하지만, load 경계에서 reject되는지 검증하지 않는다.
  - 실행 재현: 정상 save의 `world.current_level.depth`를 99로 바꾼 뒤 `--load saves/bad.json --turns 1` 실행. 결과: `thread 'main' panicked at crates\aihack-runtime\src\world.rs:150:9`, `assertion left == right failed`, exit 101.
- Expected Basis: 사용자 요청의 “load parsing ... fail-closed”, `spec.md:690-701`의 save/load hash·wire 계약, `spec.md:727-728`의 안전한 persisted artifact 경계, `crates/aihack-core/src/invariant.rs:57-95`의 persisted world invariant validator.
- Actual: schema 1이면 구조적으로 깨진 world도 `Ok(GameSession)`이 된다. Debug에서는 panic, release에서는 invariant이 깨진 상태가 계속될 가능성이 있다. CLI가 약속한 typed load error/exit 2 경계를 통과하지 않는다.
- Impact: 공격자 또는 손상된 artifact 하나로 headless/TUI 실행을 crash시키고, release에서 잘못된 위치·엔티티·inventory 참조를 통해 상태/결과를 오염시킬 수 있다. save를 신뢰할 수 없는 입력으로 취급하는 hard boundary가 없다.
- Suggested Action: `GameWorld::from_saved_world` 또는 전용 SaveDecoder를 `Result` 경계로 바꾸고, session 생성 전에 world invariant와 persisted collection/reference/range를 모두 검증한다. invalid save는 session을 만들지 않고 typed `GameError`로 반환해야 한다. debug/release 모두 panic이 없어야 하며 malformed save에 failure report와 exit 2를 고정한다.
- Re-audit Method: current-level missing, player missing/non-player, player-level mismatch, out-of-bounds, inventory owner mismatch, invalid entity/item references, inconsistent event/turn cases를 JSON fixture로 각각 load하고 모두 typed error인지 확인한다. 같은 fixture를 debug/release headless에서 실행해 panic/성공 report가 없는지 확인한다.
- Owner: Coder
- Confidence: High
- Notes: `validate_world`가 존재한다는 사실은 load가 자동으로 검증된다는 뜻이 아니다. 현재 테스트는 validator의 분류 능력만 잠근다.

### [A03-F003] ReplayLineV1의 turn/outcome/hash 필드를 읽고도 검증하지 않아 forged replay가 성공 report를 만듦

- Area: replay integrity, deterministic replay, headless runner
- Pass: Security
- Pattern: SEC-005 (replay integrity hard boundary 미강제)
- Severity: Major
- Status: Confirmed
- Summary: replay runner는 각 line의 `command`만 제출하고 `turn_before`, `outcome`, `snapshot_hash_after`를 무시한다. 따라서 잘못된/다른 seed의 trace가 실제 실행 결과와 달라도 replay가 성공한 것처럼 보고된다.
- Evidence:
  - `apps/aihack-headless/src/lib.rs:64-84`는 target turn이 되기 전 `session.submit(line.command)`만 호출한다. `line.turn_before`, `line.outcome`, `line.snapshot_hash_after` 비교가 없다.
  - `apps/aihack-headless/src/main.rs:102-110`은 JSONL을 deserialize한 뒤 곧바로 해당 runner를 호출한다.
  - 정상 1-turn trace의 `turn_before`를 9999, `snapshot_hash_after`와 `outcome.snapshot_hash`를 `forged`, `outcome.accepted/turn_advanced`를 false로 바꿔 `--policy replay-file --replay-in replays/bad.jsonl --turns 1` 실행했다. 결과는 exit 0과 `accepted_turns=1`, 실제 hash `54e43384cefa2590`인 성공 stdout였다.
  - `spec.md:691,701`, `BUILD_GUIDE.md:245-246,251`은 line metadata와 같은 검증 상대 경로를 계약에 포함하지만 현재 runner는 metadata를 신뢰하지도 거부하지도 않는다.
- Expected Basis: `ReplayLineV1`의 네 필드가 replay truth를 구성한다는 `spec.md:690-701`, 같은 seed/command sequence의 결정론 계약과 사용자 요청의 replay parsing fail-closed 요구.
- Actual: JSON shape만 맞으면 forged metadata가 있는 trace도 실제 command sequence로 재실행되고 report에 성공 hash/turn이 기록된다.
- Impact: replay artifact의 무결성·재현성 검증이 무력화된다. 손상된 trace, 다른 session에서 이어 붙인 trace, 결과를 위조한 trace를 성공 기준선으로 오인할 수 있다. `--replay-in`/`--replay-out` 동일 경로 textual equality 검사만으로는 이 문제를 막지 못한다.
- Suggested Action: 각 line 처리 전에 `line.turn_before == session.revision().turn`을 확인하고, command 제출 후 실제 `TurnOutcome`의 accepted/turn_advanced/events/next_state/snapshot hash를 line과 비교한다. mismatch는 typed `ReplayIntegrity` 오류로 즉시 중단하고 성공 report를 만들지 않는다. 초기 seed/content/session revision을 trace header 또는 첫 line metadata로 묶는 방안도 명세에서 결정한다.
- Re-audit Method: turn_before, command, outcome.accepted, turn_advanced, events, outcome hash, snapshot_hash_after를 각각 한 번씩 변조한 fixture와 early/extra line fixture를 실행해 모두 reject되는지 확인한다. 정상 trace는 load/resume 후 final hash가 direct run과 같아야 한다.
- Owner: Coder
- Confidence: High
- Notes: JSONL 문법 오류는 현재 parser가 거부하지만, 의미 무결성 오류는 모두 통과한다.

### [A03-F004] replay append의 single-link 검사가 write 전에 끝나 hard-link race가 남음

- Area: hard-link TOCTOU, replay append, file integrity
- Pass: Security
- Pattern: SEC-004 (path/file-mode/hard-link 제어군의 TOCTOU)
- Severity: Major
- Status: Probable
- Summary: replay append는 file descriptor를 연 뒤 한 번 `nlink == 1`을 확인하지만, 그 이후 handle에 append하는 동안 다른 process가 같은 inode에 hard link를 추가하는 것을 막지 않는다. no-follow와 단일 검사만으로는 검사와 write 사이의 link-count race를 닫지 못한다.
- Evidence:
  - `crates/aihack-runtime/src/save.rs:43-59`는 `open_with` → `validate_open_file` → `BufWriter` write 순서다.
  - `:245-263`의 `validate_metadata`는 `metadata.nlink() == 1`을 한 시점에만 검사한다. write 직전 exclusive lock, atomic replacement, descriptor-level immutable link guarantee가 없다.
  - `tests/headless_paths.rs:166-198`은 사전 배치 hard link만 검증한다. open 이후 link 생성 경쟁을 주입하는 regression은 없다.
- Expected Basis: `spec.md:698`, `spec.md:727-728`, `DESIGN_DECISIONS.md:57-59`의 “쓰기 대상 hard-link count 1이면 fail-closed” 및 TOCTOU를 닫는 capability decision.
- Actual: POSIX에서는 공격자가 validation 후 기존 replay inode에 외부 hard link를 만들 수 있고, 열린 descriptor는 같은 inode를 계속 가리킨다. 그 뒤 append가 수행되면 외부 hard-link victim도 함께 변경된다. Windows의 share/ACL 동작은 별도 검증이 필요하다.
- Impact: preplaced hard-link negative test를 통과하면서도 concurrent attacker가 root 밖 파일을 수정할 수 있다. replay artifact와 외부 victim의 데이터 무결성이 깨진다.
- Suggested Action: replay append를 lock/atomic rewrite 경계로 재설계하거나, 플랫폼별로 hard-link 생성과 write를 함께 배제하는 강한 descriptor/ACL 계약을 도입한다. 단순히 nlink을 한 번 더 읽는 것은 race를 제거하지 못하므로, attacker가 link를 만들 수 있는 조건 자체를 차단하는지 문서화한다.
- Re-audit Method: open 직후 write 전에 동기화 barrier를 제공하는 test seam을 두고 별도 thread/process가 hard link를 만든 뒤 append를 시도하는 Unix regression을 추가한다. victim inode가 변하지 않고 append가 typed error/rollback으로 끝나는지 확인하고 Windows equivalent를 별도로 실행한다.
- Owner: Architect / Coder
- Confidence: High (정적 TOCTOU 분석), 실행 재현 미완료
- Notes: save atomic replace는 신규 임시 inode를 rename하므로 같은 race가 destination payload에 직접 write되는 구조는 아니다. 이 finding은 replay append에 한정된다.

### [A03-F005] save/replay 입력 크기와 record 수에 상한이 없어 local artifact가 memory DoS 입력이 됨

- Area: untrusted artifact parsing, resource exhaustion
- Pass: Security
- Pattern: SEC-004 (untrusted file input resource boundary)
- Severity: Major
- Status: Confirmed
- Summary: save는 파일 전체를 `String`으로 읽고 replay는 모든 line을 `Vec`에 누적한다. 파일 크기, line 크기, line 수, event/entity collection 수에 대한 pre-parse 상한이 없어 손상되거나 악의적으로 큰 runtime artifact가 메모리와 CPU를 고갈시킬 수 있다.
- Evidence:
  - `crates/aihack-runtime/src/save.rs:62-72`는 각 line을 제한 없이 읽은 뒤 전체 결과를 `collect()`한다.
  - `:81-87`은 `read_to_string`으로 save 전체를 상한 없이 할당한 뒤 JSON을 parse한다.
  - `SaveDataV1`의 `event_log`, `SavedWorldV1`의 levels/entities/inventory 및 `ReplayLineV1`의 events는 serde `Vec`로 바로 materialize된다(`crates/aihack-core/src/save.rs:17-33,76-95`).
  - 32,768/65,536 byte 제한은 `crates/aihack-llm/src/transport.rs`의 HTTP body에만 적용되고 save/replay에는 적용되지 않는다.
- Expected Basis: 사용자 요청의 load/replay parsing fail-closed, 일반적인 untrusted file resource bound 불변조건. 현재 `spec.md`에는 save/replay maximum이 정의되지 않았으므로 정확한 수치에는 명세 보완이 필요하다.
- Actual: valid JSON이면 크기와 collection cardinality에 관계없이 parser가 할당·materialize를 시도한다. malformed JSON도 parse 전 큰 allocation을 일으킬 수 있다.
- Impact: headless/TUI가 artifact 하나만으로 OOM 또는 긴 parse를 겪고, 자동화된 replay/report gate가 거부 대신 process-level DoS를 맞을 수 있다.
- Suggested Action: spec에서 save byte limit, replay total/line/count limit, event/entity limits를 정하고, `metadata.len`/bounded reader와 line/count budget을 parse 전에 적용한다. 초과는 typed `ArtifactTooLarge`로 fail-closed 처리하고 partial `Vec`를 반환하지 않는다. JSON unknown/duplicate semantic field policy도 함께 명시한다.
- Re-audit Method: limit 바로 아래/초과의 save·replay fixture, 거대한 단일 line, 많은 작은 line, 큰 event string을 실행해 bounded error와 기존 artifact 보존을 확인한다. peak allocation을 관찰 가능한 test budget으로 고정한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: 제품이 “신뢰된 local-only artifact만 입력”이라고 결정한다면 그 trust boundary와 운영상 파일 크기 제한을 문서에 명시해야 하며, 현재 문서는 이를 정의하지 않는다.

### [A03-F006] persisted event 문자열이 TUI raw buffer와 LLM prompt로 재유입되기 전에 control/scope 검사를 받지 않음

- Area: save-originated text, terminal control injection, prompt boundary
- Pass: Security
- Pattern: SEC-005 / SEC-008 (renderer 및 prompt 보호 경계 불일치)
- Severity: Major
- Status: Confirmed
- Summary: SaveDataV1의 `event_log` 문자열은 load 시 그대로 복원된다. 그 값은 observation의 최근 event로 TUI에 전달되고, `CommandRejected.reason`은 raw text로 렌더되며, 전체 최근 event가 LLM canonical request에도 포함된다. LLM의 user text/provider output sanitization은 이 save-originated event를 보호하지 않는다.
- Evidence:
  - `crates/aihack-core/src/event.rs:114-120`의 `CommandRejected.reason`와 `Message.text`는 arbitrary `String`이고 serde derive로 읽힌다.
  - `crates/aihack-runtime/src/save.rs:189,200-208`은 event log를 저장·복원하며 text validation이 없다.
  - `crates/aihack-runtime/src/observation.rs:63-71`은 최근 event 8개를 그대로 observation에 넣는다.
  - `apps/aihack-tui/src/tui/render_panels.rs:257-267`은 `CommandRejected { reason }`를 직접 format하고, `:29-32`의 `TextPanel`은 각 char를 `Buffer::set_char`에 넣어 control char 제거를 하지 않는다.
  - `crates/aihack-llm/src/service.rs:285-300`은 observation/last_events를 canonical JSON으로 직렬화한다. `:267-280`은 `LlmRequestKind::SoftAdjudication.user_text`만 validate하고 observation event 문자열은 검사하지 않는다.
  - 현재 LLM 테스트의 C0/C1/ANSI cases는 `validate_user_text`와 provider payload에만 적용된다(`tests/llm_transport.rs:144-157`, `tests/llm_soft_adjudication.rs:34-79`). malformed SaveDataV1 event fixture는 없다.
- Expected Basis: `designs.md:185,295`, `spec.md:723-725,727-730`의 prompt/response control 및 외부 입력 격리 경계, 사용자 요청의 load/control fail-closed 질문.
- Actual: crafted save가 `\u{1b}` 등 C0/C1을 포함한 `CommandRejected.reason`/`Message.text`를 넣어도 load가 허용한다. 그 문자열은 TUI 화면과 local LLM user message로 재사용된다. 현재 provider가 loopback이라는 사실은 terminal injection과 future adapter prompt boundary를 제거하지 않는다.
- Impact: TUI 출력에 ANSI/control sequence가 주입되어 화면 위조·터미널 상태 변경이 가능하고, local model에 prompt injection/오염된 상황 설명이 전달된다. remote provider가 명세상 비목표여도 malformed artifact가 presentation trust boundary를 넘는다.
- Suggested Action: persisted event text를 typed allowlist와 길이/C0/C1 검사로 decode 단계에서 검증하고, invalid event가 있으면 load를 거부한다. 최소한 TUI renderer는 모든 외부/저장 문자열을 printable text로 sanitize하고 LLM projection에서는 persisted free text를 제외하거나 별도 bounded field로 정제한다. save/load negative fixture에서 control 및 prompt payload가 표시·전송되지 않는지 잠근다.
- Re-audit Method: ESC, C1, 개행, 240자 초과, prompt-like event text를 포함한 malformed save를 TUI load와 loopback LLM mock에 넣고, terminal output bytes와 request body를 검사한다. reject 또는 명시된 sanitize 결과만 허용한다.
- Owner: Coder
- Confidence: High
- Notes: 정상 게임이 생성하는 고정 메시지는 현재 안전해 보인다. 문제는 저장 artifact를 신뢰 경계로 취급하지 않는 load path다.

### [A03-F007] “테스트 전용” path helper가 public으로 남아 있고 TUI production adapter가 unrestricted parent root를 사용함

- Area: TUI quick-save/API boundary, compatibility helper, path authority
- Pass: Security
- Pattern: SEC-004 / SEC-005 (호출자 path authority drift)
- Severity: Minor
- Status: Confirmed
- Summary: 실제 `run_tui` quick-save는 `tempfile::tempdir()` 아래의 random `quick-save.json`을 사용하므로 현재 기본 실행에서 외부 path 탈출은 관찰되지 않았다. 그러나 TUI의 public `TuiClient`/`TuiApp` save/load API는 absolute path를 받아 `store_for_path`가 그 path의 parent를 새 `ArtifactStore` root로 삼는 compatibility helper를 호출한다. ADR은 이 helper를 trusted test path에서만 유지한다고 했지만 production adapter에도 연결되어 있다.
- Evidence:
  - `crates/aihack-runtime/src/save.rs:159-177,228-235`의 free helper와 `store_for_path`는 입력 path의 parent를 root로 연다. file name만 relative validation을 받으므로 helper 호출 자체는 repository runtime root에 고정되지 않는다.
  - `apps/aihack-tui/src/tui/mod.rs:89-102,376-381`의 public TUI API가 `save_session_to_path`/`load_session_from_path`를 직접 호출한다.
  - `:692-695`의 실제 event loop는 random tempfile directory를 생성하므로 현재 quick-save 호출 자체는 안전한 root를 갖는다.
  - `tests/ui_runtime_smoke.rs:16-23,43-51`은 이 unrestricted absolute path bridge를 정상 계약처럼 사용한다.
  - `DESIGN_DECISIONS.md:71`은 path 기반 compatibility helper를 trusted test path에만 유지하고 production CLI는 `ArtifactStore`를 사용한다고 선언한다.
  - `resolve_path_in_root`(`save.rs:134-156`)도 check 후 bare `PathBuf`를 반환하는 check-then-use API이며 production CLI에서 사용되지 않는다.
- Expected Basis: `DESIGN_DECISIONS.md:56-71`, `spec.md:695,727-729`, `BUILD_GUIDE.md:251`의 단일 capability root·production 경계.
- Actual: headless production CLI는 직접 `ArtifactStore`를 사용하지만 TUI production adapter는 helper를 통해 root를 호출자 path에서 정한다. 현재 호출자가 random tempfile을 만들기 때문에 즉시 exploit은 아니지만 public contract가 hard root boundary를 강제하지 않는다.
- Impact: 향후 TUI load/import, plugin, adapter가 사용자 path를 전달하면 arbitrary parent directory가 artifact root가 된다. `resolve_path_in_root`를 보안 resolver로 재사용하면 검사와 실제 open 사이 TOCTOU가 다시 도입된다.
- Suggested Action: TUI가 `ArtifactStore`와 relative `quick-save.json`을 소유하도록 API를 바꾸고, path compatibility helper는 private/`cfg(test)`로 격리하거나 “security boundary 아님”으로 명확히 이름을 바꾼다. `resolve_path_in_root`는 삭제하거나 handle-based operation으로 대체한다. production call graph에서 unrestricted helper가 사라지는 정적 테스트를 추가한다.
- Re-audit Method: `rg`로 production source의 free path helper 사용이 0인지 확인하고, TUI quick-save가 고정된 tempfile root와 relative name만 소비하는지 검증한다. 절대 path·parent traversal·symlink/junction fixture를 TUI API에 직접 넣어 typed reject를 확인한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: 현재 기본 TUI quick-save가 직접 boundary를 우회한다고 단정하지 않는다. 이 finding은 public API/ADR 간의 hard-boundary enforcement drift다.

### [A03-F008] 파일 payload는 atomic replace지만 root 권한·rename durability 계약은 완전히 닫혀 있지 않음

- Area: file permissions, directory metadata, crash durability
- Pass: Security
- Pattern: SEC-005 (실제 file-mode/durability 경계와 문서 표현)
- Severity: Minor
- Status: Confirmed
- Summary: 파일 payload에는 Unix 0600과 Windows ACL caveat가 있지만 `ArtifactStore::open`은 root directory mode/ACL을 설정하지 않고, `write_atomic`은 임시 파일 `sync_all` 후 parent directory를 sync하지 않는다. 따라서 내용 기밀성과 정상 동작은 일부 보장되지만 directory metadata 및 전원 장애 후 rename durability는 별도 계약으로 닫혀 있지 않다.
- Evidence:
  - `crates/aihack-runtime/src/save.rs:27-30`은 root directory를 기본 `create_dir_all`로 만들고 mode/ACL을 고정하지 않는다.
  - `:95-106`은 temp file write/flush/file `sync_all` 후 rename하지만 parent directory `sync_all` 단계가 없다.
  - `:275-285`은 file mode 0600(Unix)와 Windows read-only 해제만 처리한다.
  - `BUILD_GUIDE.md:251`, `spec.md:697`, `tests/headless_paths.rs:115-159`는 파일 mode와 Windows parent DACL은 설명하지만 root directory mode 또는 crash durability는 명시하지 않는다.
- Expected Basis: 문서가 주장하는 save permission/atomic replace를 실제 보호 수준과 일치시켜야 한다는 `SEC-005`/`SEC-007` 기준. “atomic visibility”와 “crash durable”은 구분되어야 한다.
- Actual: 일반적인 Unix umask/Windows inherited ACL에 의존하며, 파일 내용은 0600일 수 있어도 filename/metadata 노출과 directory ACL은 caller 환경에 달린다. 정상 rename의 원자성은 있으나 power loss 후 rename 보존은 보장되지 않는다.
- Impact: 공유/느슨한 runtime root에서 artifact 이름과 directory metadata가 노출될 수 있고, 장애 직후 새 save가 사라지거나 이전 save가 남는 durability 결과를 제품이 예측하지 못한다. 이는 root junction·load corruption보다 낮은 위험이다.
- Suggested Action: 제품 threat model에 root directory ownership/mode 요구와 crash durability 목표를 명시한다. 필요한 경우 root 생성·검증 시 Unix 0700 및 Windows 사용자 ACL을 검사하고, 지원 플랫폼에서 rename 후 parent directory sync를 수행한다. 보장하지 않을 경우 문서에서 atomic visibility만 명시하고 durable save로 과대주장하지 않는다.
- Re-audit Method: umask 000/공유 ACL fixture와 restrictive ACL fixture에서 root/file metadata를 확인하고, 가능한 Unix filesystem에서 rename 직후 parent fsync 유무를 검증한다. BUILD/spec/ADR 표현을 실제 보장 수준과 맞춘다.
- Owner: Architect / Coder
- Confidence: Medium
- Notes: Windows owner-only를 보장하지 않는다는 현재 문서 표현은 정직하다. 이 finding은 그 caveat를 뒤집지 않고 Unix root metadata와 crash durability의 미문서화를 지적한다.

## 7. Uncertainties and Clarifications Needed

1. Save/replay artifact의 최대 byte 수, replay line 수/길이, event/entity cardinality가 `spec.md`에 없다. F005의 정확한 limit은 제품 owner가 결정해야 하며, 수치 결정 전에는 `Accepted Risk`로 닫을 수 없다.
2. Hard boundary의 공격자 모델이 “같은 user가 writable cwd/runtime을 바꿀 수 있는 경우”까지 포함하는지 문서가 명시하지 않는다. 문서가 root 밖 symlink를 구조적으로 거부한다고 선언한 이상 F001은 현재 문구 기준으로 유지한다.
3. TUI `TuiApp::save_to_path`/`load_from_path`가 외부 adapter용 public contract인지 테스트 전용 bridge인지 authority가 충돌한다. production API라면 F007 수정이 필요하고, 테스트 전용이면 visibility/문서를 좁혀야 한다.
4. 파일 저장에서 요구하는 것은 crash 시에도 보존되는 durable commit인지, 정상 실행 중의 atomic visibility인지 명세가 구분하지 않는다. F008의 최종 severity는 이 결정을 반영해 재평가한다.
5. SaveDataV1을 사용자가 의도적으로 편집 가능한 local sandbox fixture로 취급하는지, 외부에서 전달될 수 있는 untrusted artifact로 취급하는지 명세가 없다. 후자이면 F002/F005/F006은 배포 gate 차단 수준으로 유지해야 한다.

## 8. Perspective Decision

`HOLD`.

LLM endpoint/redirect/proxy/body/revision 경계와 final-file capability 테스트는 양호했지만, root 자체 junction escape(F001), malformed save의 panic(F002), forged replay 성공(F003)이 확인되었다. replay hard-link TOCTOU(F004)와 저장 입력 resource/control boundary(F005/F006)도 현재 hard-boundary 문서와 맞지 않는다. 이 finding들이 해소되고, F007의 TUI API authority 및 F005/F008의 수치·durability가 명세로 닫힌 뒤 보안 Pass 3 재감사가 필요하다.

## 9. Coder Handoff

`C:\LocalDev\rust\AIHack\docs\multi_audit\1\sub_audit_03_file_security.md`를 먼저 읽고, 각 finding을 현재 `spec.md`/ADR/BUILD 문서와 실제 코드·테스트에 대조하여 우선순위대로 수정하세요. 계약 변경이 필요하면 관련 문서를 먼저 갱신하고, 수정 후 root symlink/junction, malformed save, replay metadata/hard-link race, bounded parsing, persisted-event control, TUI path authority 회귀 테스트와 표적 검증 결과를 기록하세요.

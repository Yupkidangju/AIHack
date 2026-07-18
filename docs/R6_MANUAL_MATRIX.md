# R6 Local LLM PTY Matrix

작성일: 2026-07-18
기준 커밋: `bc3363d` 이후 R6 terminal 보정 working tree
환경: Linux, tmux PTY, 저장소 보존 OpenAI-compatible deterministic fixture

## 범위와 제한

이 문서는 실제 TUI binary와 실제 loopback HTTP transport를 함께 실행한 증거다. 최초 수동 matrix에 사용한 일회성 fixture는 `audit_report_10.md`의 DBG-F004에서 재현 불가로 판정됐다. 시정 후 `scripts/r6_loopback_fixture.py`가 동일 wire와 timing을 결정적으로 제공하며 `scripts/r6_pty_matrix.sh`와 `scripts/r6_pending_exit_smoke.sh`가 안정된 semantic assertion을 재실행한다. 실제 언어 모델 추론은 SC-LLM-01..03의 필수 조건이 아니다.

## 재현 명령

```bash
scripts/r6_pty_matrix.sh
scripts/r6_pending_exit_smoke.sh
```

두 script는 고유 tmux session과 `mktemp` 디렉터리를 만들고 종료 trap에서 자신이 만든 자원만 정리한다. fixture는 `127.0.0.1`에만 bind하고 API key나 외부 network를 사용하지 않는다.

## 실행 결과

| terminal | 설정 | provider | 입력 | 관찰 결과 | 판정 |
| --- | --- | --- | --- | --- | --- |
| 120x36 | default/normal | disabled | Enter, Enter, G/A/J | Playing 진입, `LLM: OFF`, turn 0 유지 | PASS |
| 80x24 | high-contrast/reduced-motion | success fixture | G, N, A, Y, N, J | CTA 판독, suggestion은 Y 전 turn 0, Y 뒤 turn 1, Judge modal 0/240 표시 | PASS |
| 60x24 | default/reduced-motion | 300ms fixture + narrative timeout 100ms | G | `LLM: TIMEOUT`, `[R] Retry [N] Dismiss`, turn 0 | PASS |
| 59x23 | default/normal | disabled | h, Q | `terminal requires 60x24`, gameplay 입력 무시, Q clean exit | PASS |
| 120x36 | high-contrast/reduced-motion | 300ms fixture | A 직후 `.` | Wait로 turn 1, delayed response는 `LLM: STALE`, Y CTA 없음 | PASS |
| 60x24 | default/normal | connection refused | G | `LLM: DOWN`, `[R] Retry [N] Dismiss`, turn 0 | PASS |

저장소 재현 script의 2026-07-18 결과:

- success: WAIT 이후 `[N] Dismiss` PASS
- timeout: `LLM: TIMEOUT` PASS
- stale: delayed decision + `.` 이후 `LLM: STALE` PASS
- connection refused: `LLM: DOWN` PASS
- pending exit: TUI restore가 worker grace wait보다 먼저 관찰되고 process exit 291ms PASS

## 수동 실행 중 발견하고 보정한 회귀

- `KeyCode::Enter`가 Title/CharacterCreation 상태 매퍼로 전달되지 않던 runtime 경계를 연결했다.
- 승인 설계와 달리 80x28 미만에서 즉시 종료하던 gate를 60x24 지원 및 59x23 안전 입력 loop로 수정했다.
- failure fallback 결과가 TIMEOUT/DOWN의 Retry CTA를 가리던 footer 우선순위를 수정했다.
- footer가 안내하는 `.` Wait 키가 keyboard baseline에 없던 불일치를 수정했다.
- Judge modal의 빈 입력 행에 기존 inventory 내용이 비치던 panel clear 문제를 수정했다.
- `--high-contrast`, `--reduced-motion` 실행 플래그를 추가해 접근성 matrix를 실제 binary에서 선택할 수 있게 했다.

## 자동 증거

```bash
cargo test -p aihack-tui --locked --bin aihack --test tui_contract
cargo test -p aihack --locked --test ui_layout
cargo test -p aihack --locked --test ui_input_mapping
cargo test -p aihack --locked --test llm_tui_integration
scripts/r6_pty_matrix.sh
scripts/r6_pending_exit_smoke.sh
```

## 남은 게이트와 선택 검증

- 완료: `audit_report_11.md`가 IMP-F009/010/011, DBG-F004와 XPF-F007을 Verified하고 R6 checkpoint를 PASS로 종결했다.
- 고려 대상: 최종 통합에서 추가 호환성 증거가 반드시 필요할 때만 실제 provider smoke를 수행한다.
- 선택 검증을 수행하면 AIHack의 loopback 제한은 유지하고, 재사용 가능한 localhost OpenAI-compatible 임시 adapter가 Google AI Studio Gemini 같은 원격 API를 대리 호출한다. API key는 adapter 환경변수에만 주입하고 실제 model ID는 실행 시점에 확인한다.

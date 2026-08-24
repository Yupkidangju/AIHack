# AIHack

> Current code: Cargo 0.3.0 · Release audit target: 0.3.0 · Updated: 2026-08-24

## 한국어

AIHack은 NetHack 3.6.7의 관찰 가능한 규칙을 시나리오와 테스트로 재구현하는 Rust 로그라이크다. 줄 단위 C-to-Rust 번역이 아니라 결정론적 core, 출처가 추적되는 호환 규칙, 격리된 local LLM adapter를 목표로 한다.

### 현재 상태

- report 26의 최종 verifier successor SHA `1e84a94`는 [Actions `32660514315`](https://github.com/Yupkidangju/AIHack/actions/runs/32660514315)에서 Ubuntu/Windows 각 19개 step과 actual bundle을 모두 PASS했다. report 27은 그 기준선에서 allocator/registry, field-only causal, archive/calendar, TUI input과 local action 경계를 재개방했다.
- Rust 1.94.1과 ratatui 0.30/crossterm 0.29 계열은 repository에 고정됐다. report 24 implementation `2519bc8e0ede81c39f46b5778e62a41d4ca66901`은 [GitHub Actions run `32107862171`](https://github.com/Yupkidangju/AIHack/actions/runs/32107862171)에서 Ubuntu/Windows quality gate를 모두 PASS했다.
- R1 build 재현성과 R2 private state/transaction은 local gate를 통과했다.
- R3의 fallible `ContentRegistry` bootstrap과 R4의 policy 기반 headless runner는 local 검증을 통과했다.
- `survival-v1`은 seed 42, 7, 1234에서 각각 1000 accepted turn과 3회 동일 hash를 검증한다. R5 workspace·문서 시정은 `audit_report_9.md` 재감사를 통과했다.
- `audit_report_11.md`는 보고서 10의 public/schema contract와 evidence 재현성 시정을 모두 Verified하고 R6 checkpoint를 PASS로 종결했다. 실제 model provider smoke는 비차단 고려 대상이다.
- R7의 공식 checksum inventory, legacy 격리 gate, NH367-C001..C010 record/test와 독립 재감사는 완료되어 `PASS WITH KNOWN RISKS`다. 2026-07-20 프로젝트 소유자는 AIHack을 NetHack 3.6.7의 AI-assisted semantic rewrite 파생물로 분류하고 전체 배포에 NGPL을 승인했다.
- 실제 외부 게시에는 R8 기술 감사 `PASS`, `LICENSE`, `NOTICE`, 해당 바이너리를 만든 커밋의 complete corresponding source archive가 필요하다.
- project-owner 결정은 `AIHACK-OWNER-2026-07-20-NGPL-01`로 추적하며, release bundle은 `PROJECT_OWNER_LICENSE_APPROVAL.md`, `MODIFICATIONS.md`, commit-bound `RELEASE-METADATA`와 `SHA256SUMS`를 포함한다. metadata key는 정확히 한 번 존재하고 owner/modification ID의 전체 값이 bundled record와 일치해야 하며 qualified legal opinion은 별도로 주장하지 않는다.
- `audit_report_21.md`는 report 20의 `IMP-F016`/`DBG-F008` 시정을 PASS로 종결했다. report 20의 재감사 대기는 더 이상 현재 상태가 아니다.
- `docs/audit/audit_report_24.md`와 implementation SHA `2519bc8e`의 양 OS CI는 report 23/24 시정을 역사적으로 종결했다. report 23 재감사 대기는 현재 gate가 아니다.
- 현재 권위는 `docs/audit/audit_report_28.md`다. ADR-0038 시정 SHA `9725c378`의 [Actions `32694375654`](https://github.com/Yupkidangju/AIHack/actions/runs/32694375654)는 전체 local gate와 clean same-SHA Ubuntu/Windows actual bundle을 PASS했다. 새 독립 PASS와 별도 게시 승인 전에는 program/외부 게시 HOLD다.
- Windows all-target test는 dev-only `portable-pty 0.9.0` ConPTY에서 실제 TUI의 one-key state, mouse click, Inventory/Esc와 terminal restore를 검증한다. release binary dependency에는 포함되지 않는다.
- GitHub YAML action pin gate는 dev-only `saphyr 0.0.12`로 모든 workflow/composite node와 repository-root local action을 재귀 검사하며 release binary dependency에는 포함되지 않는다.
- R9 콘텐츠 인과 폐쇄는 음식·시체 섭취, 콘텐츠 기반 armor/monster behavior, 가격·난이도·gold·score, 기도·luck 전이를 실제 월드 상태에 연결한다. seed 42/7/1234 장기 테스트는 9종 semantic witness와 1000 accepted turn, 반복 hash를 함께 검증한다.

### v0.3.0 목표

- Rust 1.94.1, locked dependency, Linux/Windows CI
- private state와 transaction/invariant 기반 turn commit
- embedded TOML `ContentRegistry` 실연결
- accepted turn 1000개의 multi-seed deterministic 검증
- core/content/AI contract/LLM/TUI/headless workspace 분리
- loopback local LLM narrative, legal-action suggestion, presentation-only soft judgment
- NetHack 3.6.7 출처와 NH367 compatibility scenario 추적

### 현재 실행

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI는 `default-run = "aihack"`로 선택된다. headless 실행에는 package와 binary를 `-p aihack-headless --bin aihack-headless`로 명시한다.

### 구현 순서

1. `docs/audit/audit_report_26.md` 시정의 새 독립 재감사
2. 독립 PASS 뒤 별도 승인에 따른 외부 게시

### 문서

1. [Master specification](spec.md)
2. [Implementation tasks](IMPLEMENTATION_SUMMARY.md)
3. [Gap register](GAP_CLOSURE_ROADMAP.md)
4. [Runtime and UI design](designs.md)
5. [Architecture decisions](DESIGN_DECISIONS.md)
6. [Build guide](BUILD_GUIDE.md)
7. [Audit gates](audit_roadmap.md)
8. [Provenance policy](PROVENANCE.md)
9. [Compatibility template](docs/compatibility/README.md)
10. [Change history](CHANGELOG.md)
11. [R0 documentation audit](DOCUMENTATION_AUDIT_REPORT.md)
12. [R8 remediation closure](docs/audit/audit_report_21.md)
13. [Current re-audit](docs/audit/audit_report_26.md), [current remediation](docs/audit/audit_report_26_remediation.md), [historical final audit](docs/multi_audit/1/final_audit_report_1.md), [latest historical closed re-audit](docs/audit/audit_report_24.md)
14. [NetHack General Public License](LICENSE)와 [derivative-work notice](NOTICE)
15. [Project-owner license decision](PROJECT_OWNER_LICENSE_APPROVAL.md)과 [modification manifest](MODIFICATIONS.md)

과거 Phase 계획은 `.archive/`의 불변 snapshot으로 보존한다.

## English

AIHack is a Rust roguelike that reimplements observable NetHack 3.6.7 behavior through traced scenarios and tests. It targets a deterministic core, provenance-backed compatibility rules, and an isolated local-LLM adapter—not a line-by-line C-to-Rust translation.

### Current status

- Report 26's final verifier successor SHA `1e84a94` passed all 19 steps and the actual bundle on both Ubuntu and Windows in [Actions `32660514315`](https://github.com/Yupkidangju/AIHack/actions/runs/32660514315). Report 27 reopened allocator/registry, field-only causal, archive/calendar, TUI input, and local-action boundaries on that baseline.
- Rust 1.94.1 and the ratatui 0.30/crossterm 0.29 family are pinned. Report 24 implementation `2519bc8e0ede81c39f46b5778e62a41d4ca66901` passed both OS quality gates in [GitHub Actions run `32107862171`](https://github.com/Yupkidangju/AIHack/actions/runs/32107862171).
- R1 build reproducibility and R2 private-state transaction gates pass locally.
- R3's fallible `ContentRegistry` bootstrap and R4's policy-driven headless runner pass local verification.
- `survival-v1` verifies 1,000 accepted turns and three matching hashes for seeds 42, 7, and 1234. The R5 workspace and documentation remediation passed the `audit_report_9.md` re-audit.
- `audit_report_11.md` verified the public/schema-contract and reproducibility remediation from report 10 and closed the R6 checkpoint with PASS. A real-provider smoke remains non-blocking.
- R7 passes its engineering scope with known risks after verified checksums, a fail-closed legacy gate, and ten strengthened NH367 traces. On 2026-07-20 the project owner classified AIHack as an AI-assisted semantic-rewrite derivative of NetHack 3.6.7 and approved NGPL for the whole distribution.
- External publication still requires an R8 technical-audit PASS plus LICENSE, NOTICE, and the complete corresponding source for the released binaries.
- The project-owner decision is traceable as `AIHACK-OWNER-2026-07-20-NGPL-01`; release bundles carry PROJECT_OWNER_LICENSE_APPROVAL.md, MODIFICATIONS.md, commit-bound RELEASE-METADATA, and SHA256SUMS. Metadata IDs must resolve to the bundled records. No qualified legal opinion is claimed.
- `audit_report_21.md` closed report 20's documentation remediation with PASS; report 20 is no longer the current pending authority.
- Report 24 and the two-OS CI for implementation SHA `2519bc8e` historically closed report 23/24 remediation; report 23 is no longer the current pending gate.
- The current authority is `docs/audit/audit_report_28.md`. ADR-0038 remediation SHA `9725c378` passed the full local gate and clean same-SHA Ubuntu/Windows actual bundles in [Actions `32694375654`](https://github.com/Yupkidangju/AIHack/actions/runs/32694375654). Program PASS and publication remain blocked pending a new independent PASS and separate publication approval.
- Windows all-target tests use dev-only `portable-pty 0.9.0` ConPTY to exercise real TUI one-key states, mouse input, Inventory/Esc, and terminal restoration; it is not a release-binary dependency.
- The action-pin gate structurally scans every workflow/composite YAML node and recursively resolves repository-root local actions with dev-only `saphyr 0.0.12`; it is not a release-binary dependency.
- R9 causal closure connects food/corpse consumption, content-driven armor and monster behavior, price/difficulty/gold/score, and prayer/luck to observable world-state changes. Long runs for seeds 42/7/1234 now require all nine semantic witnesses as well as 1,000 accepted turns and repeatable hashes.

### v0.3.0 target

- Rust 1.94.1, locked dependencies, Linux/Windows CI
- Private state and transaction/invariant turn commits
- Runtime-backed embedded TOML `ContentRegistry`
- Deterministic 1,000-accepted-turn multi-seed verification
- Core/content/AI-contract/LLM/TUI/headless workspace boundaries
- Loopback local-LLM narrative, legal-action suggestions, and presentation-only soft judgments
- Traceable NetHack 3.6.7 sources and NH367 compatibility scenarios

### Run the current code

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

The TUI is selected through `default-run = "aihack"`; select the headless package and binary with `-p aihack-headless --bin aihack-headless`. See the Korean section for the ordered document set.

## 日本語

AIHack は、NetHack 3.6.7 の観察可能な挙動を、出典付きシナリオとテストで再実装する Rust ローグライクです。C ソースの行単位変換ではなく、決定論的 core、追跡可能な互換ルール、隔離された local LLM adapter を目標にします。

### 現在の状態

- report 26 の最終 verifier successor SHA `1e84a94` は Actions `32660514315` で Ubuntu/Windows 各 19 step と実 bundle をすべて PASS しました。report 27 はその基準から allocator/registry、field-only causal、archive/calendar、TUI input、local action 境界を再開しました。
- Rust 1.94.1 と ratatui 0.30/crossterm 0.29 は固定済みです。R5 workspace と文書是正は `audit_report_9.md` の再監査を通過しました。
- `audit_report_11.md` は、report 10 の public/schema contract と証拠再現性の是正をすべて Verified とし、R6 checkpoint を PASS で完了しました。実 provider smoke は非 blocking の検討事項です。
- AIHack は NetHack 3.6.7 の AI-assisted semantic rewrite による派生物としてプロジェクト所有者に承認され、配布物全体に NGPL を適用します。外部公開には R8 技術監査 PASS と LICENSE、NOTICE、対応する完全なソースが必要です。
- R9 の因果閉包では、食料・死体、armor、monster behavior、価格・難易度・gold・score、祈り・luck を実際の world state 変化へ接続し、seed 42/7/1234 の長期 test で semantic delta を検証します。
- report 24 と implementation SHA `2519bc8e` の両 OS CI により report 23/24 の修正は履歴上完了し、report 23 は現在の保留 gate ではありません。
- 現在の権威は `docs/audit/audit_report_28.md` です。ADR-0038 修正 SHA `9725c378` の Actions `32694375654` は全 local gate と clean same-SHA Ubuntu/Windows 実 bundle を PASS しました。新しい独立 PASS と別途公開承認までは program PASS と外部公開を保留します。
- Windows の all-target test は dev-only `portable-pty 0.9.0` ConPTY で実 TUI の一キー遷移、mouse、Inventory/Esc、terminal 復元を検証し、release binary には含めません。
- action pin gate は dev-only `saphyr 0.0.12` で全 workflow/composite YAML node と repository-root local action を再帰検査し、release binary には含めません。

### 現在の実行

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI は `default-run = "aihack"` で選択されます。headless は `-p aihack-headless --bin aihack-headless` で package と binary を指定します。実装順序と文書一覧は韓国語セクションを参照してください。

## 繁體中文

AIHack 是以具來源追蹤的情境與測試，重新實作 NetHack 3.6.7 可觀察行為的 Rust roguelike。目標不是逐行翻譯 C，而是 deterministic core、可追蹤的相容規則，以及隔離的 local LLM adapter。

### 目前狀態

- report 26 的最終 verifier successor SHA `1e84a94` 已在 Actions `32660514315` 通過 Ubuntu/Windows 各 19 個 step 與實際 bundle。report 27 在此基準上重新開啟 allocator/registry、field-only causal、archive/calendar、TUI input 與 local action 邊界。
- Rust 1.94.1 與 ratatui 0.30/crossterm 0.29 已固定；R5 workspace 與文件修正已通過 `audit_report_9.md` 複審。
- `audit_report_11.md` 已驗證 report 10 的 public/schema contract 與證據可重現性修正，並以 PASS 完成 R6 checkpoint；實際 provider smoke 仍為非阻斷考量。
- 專案所有者已將 AIHack 核准為 NetHack 3.6.7 的 AI-assisted semantic rewrite 衍生作品，整體散布適用 NGPL。對外發布仍需 R8 技術稽核 PASS、LICENSE、NOTICE 與對應完整原始碼。
- R9 因果閉合把食物／屍體、armor、monster behavior、價格／難度／gold／score、祈禱／luck 連到實際 world state 變化，並以 seed 42/7/1234 長期測試驗證 semantic delta。
- report 24 與 implementation SHA `2519bc8e` 的雙作業系統 CI 已在歷史上結束 report 23/24 修正；report 23 不再是目前待處理 gate。
- 目前權威是 `docs/audit/audit_report_28.md`。ADR-0038 修正 SHA `9725c378` 的 Actions `32694375654` 已通過完整 local gate 與 clean same-SHA Ubuntu/Windows 實際 bundle。新的獨立 PASS 與另行發布核准前，program PASS 與對外發布維持暫停。
- Windows all-target 測試使用僅供開發的 `portable-pty 0.9.0` ConPTY 驗證實際 TUI 的單鍵狀態、滑鼠、Inventory/Esc 與終端復原，不納入 release binary 相依性。
- action pin gate 使用僅供開發的 `saphyr 0.0.12` 結構化掃描全部 workflow/composite YAML node，並遞迴解析 repository-root local action，不納入 release binary 相依性。

### 目前執行

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI 由 `default-run = "aihack"` 選取；headless 以 `-p aihack-headless --bin aihack-headless` 指定 package 與 binary。實作順序與文件列表請參閱韓文段落。

## 简体中文

AIHack 是通过带来源追踪的场景和测试，重新实现 NetHack 3.6.7 可观察行为的 Rust roguelike。目标不是逐行翻译 C，而是 deterministic core、可追踪的兼容规则，以及隔离的 local LLM adapter。

### 当前状态

- report 26 的最终 verifier successor SHA `1e84a94` 已在 Actions `32660514315` 通过 Ubuntu/Windows 各 19 个 step 与实际 bundle。report 27 在此基线上重新打开 allocator/registry、field-only causal、archive/calendar、TUI input 与 local action 边界。
- Rust 1.94.1 和 ratatui 0.30/crossterm 0.29 已固定；R5 workspace 与文档修正已通过 `audit_report_9.md` 复审。
- `audit_report_11.md` 已验证 report 10 的 public/schema contract 与证据可复现性修正，并以 PASS 完成 R6 checkpoint；实际 provider smoke 仍为非阻断考虑项。
- 项目所有者已将 AIHack 批准为 NetHack 3.6.7 的 AI-assisted semantic rewrite 衍生作品，整体分发适用 NGPL。对外发布仍需 R8 技术审计 PASS、LICENSE、NOTICE 与对应完整源代码。
- R9 因果闭合将食物／尸体、armor、monster behavior、价格／难度／gold／score、祈祷／luck 连接到实际 world state 变化，并用 seed 42/7/1234 长期测试验证 semantic delta。
- report 24 与 implementation SHA `2519bc8e` 的双操作系统 CI 已在历史上结束 report 23/24 修正；report 23 不再是当前待处理 gate。
- 当前权威是 `docs/audit/audit_report_28.md`。ADR-0038 修正 SHA `9725c378` 的 Actions `32694375654` 已通过完整 local gate 与 clean same-SHA Ubuntu/Windows 实际 bundle。新的独立 PASS 与另行发布批准前，program PASS 和对外发布保持暂停。
- Windows all-target 测试使用仅供开发的 `portable-pty 0.9.0` ConPTY 验证实际 TUI 的单键状态、鼠标、Inventory/Esc 与终端恢复，不纳入 release binary 依赖。
- action pin gate 使用仅供开发的 `saphyr 0.0.12` 结构化扫描全部 workflow/composite YAML node，并递归解析 repository-root local action，不纳入 release binary 依赖。

### 当前运行

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI 由 `default-run = "aihack"` 选择；headless 使用 `-p aihack-headless --bin aihack-headless` 指定 package 和 binary。实现顺序和文档列表请参阅韩文部分。

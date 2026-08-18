# AIHack

> Current code: Cargo 0.3.0 · Release audit target: 0.3.0 · Updated: 2026-08-18

## 한국어

AIHack은 NetHack 3.6.7의 관찰 가능한 규칙을 시나리오와 테스트로 재구현하는 Rust 로그라이크다. 줄 단위 C-to-Rust 번역이 아니라 결정론적 core, 출처가 추적되는 호환 규칙, 격리된 local LLM adapter를 목표로 한다.

### 현재 상태

- fmt, clippy, 전체 test, release build는 현재 환경에서 통과한 기준선이다.
- Rust 1.94.1과 ratatui 0.30/crossterm 0.29 계열은 repository에 고정됐다. R9 기준 commit `41a1b63f11a57a671b0f705883431dab24298b5a`는 [GitHub Actions run `32034295607`](https://github.com/Yupkidangju/AIHack/actions/runs/32034295607)에서 `ubuntu-latest quality gate`와 `windows-latest quality gate` 모두 PASS했다.
- R1 build 재현성과 R2 private state/transaction은 local gate를 통과했다.
- R3의 fallible `ContentRegistry` bootstrap과 R4의 policy 기반 headless runner는 local 검증을 통과했다.
- `survival-v1`은 seed 42, 7, 1234에서 각각 1000 accepted turn과 3회 동일 hash를 검증한다. R5 workspace·문서 시정은 `audit_report_9.md` 재감사를 통과했다.
- `audit_report_11.md`는 보고서 10의 public/schema contract와 evidence 재현성 시정을 모두 Verified하고 R6 checkpoint를 PASS로 종결했다. 실제 model provider smoke는 비차단 고려 대상이다.
- R7의 공식 checksum inventory, legacy 격리 gate, NH367-C001..C010 record/test와 독립 재감사는 완료되어 `PASS WITH KNOWN RISKS`다. 2026-07-20 프로젝트 소유자는 AIHack을 NetHack 3.6.7의 AI-assisted semantic rewrite 파생물로 분류하고 전체 배포에 NGPL을 승인했다.
- 실제 외부 게시에는 R8 기술 감사 `PASS`, `LICENSE`, `NOTICE`, 해당 바이너리를 만든 커밋의 complete corresponding source archive가 필요하다.
- project-owner 결정은 `AIHACK-OWNER-2026-07-20-NGPL-01`로 추적하며, release bundle은 `PROJECT_OWNER_LICENSE_APPROVAL.md`, `MODIFICATIONS.md`, commit-bound `RELEASE-METADATA`와 `SHA256SUMS`를 포함한다. metadata key는 정확히 한 번 존재하고 owner/modification ID의 전체 값이 bundled record와 일치해야 하며 qualified legal opinion은 별도로 주장하지 않는다.
- `audit_report_21.md`는 report 20의 `IMP-F016`/`DBG-F008` 시정을 PASS로 종결했다. report 20의 재감사 대기는 더 이상 현재 상태가 아니다.
- `docs/audit/audit_report_23.md`는 R9 장기 witness, Windows checkpoint와 save/replay 파일 경계의 새 blocker를 HOLD했다. 2026-08-18 coder remediation은 capability 기반 artifact I/O, 9종 causal witness와 CRLF checkpoint 회귀를 구현해 로컬 검증을 통과했으며 독립 재감사를 기다린다. 외부 게시는 독립 PASS와 별도 사용자 승인 전까지 HOLD다.
- `docs/audit/audit_report_24.md`의 후속 finding은 로컬에서 시정됐다. cargo-deny 0.19.4가 winx 전용 license exception으로 PASS하고, Windows save 권한은 parent DACL 상속으로 정확히 표현되며, SC-CAUSE-01..07은 코드와 테스트에 개별 매핑된다. clean same-SHA 양 OS CI는 push 후 확인한다.
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

1. `docs/audit/audit_report_23.md` 시정의 독립 재감사
2. 최종 독립 PASS 뒤 별도 승인에 따른 외부 게시

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
12. [R8 remediation closure](audit_report_21.md)
13. [Latest re-audit](docs/audit/audit_report_24.md), [report 23 remediation record](docs/audit/audit_report_23_remediation.md), [R9 causal audit](docs/audit/audit_report_22.md)
14. [NetHack General Public License](LICENSE)와 [derivative-work notice](NOTICE)
15. [Project-owner license decision](PROJECT_OWNER_LICENSE_APPROVAL.md)과 [modification manifest](MODIFICATIONS.md)

과거 Phase 계획은 `.archive/`의 불변 snapshot으로 보존한다.

## English

AIHack is a Rust roguelike that reimplements observable NetHack 3.6.7 behavior through traced scenarios and tests. It targets a deterministic core, provenance-backed compatibility rules, and an isolated local-LLM adapter—not a line-by-line C-to-Rust translation.

### Current status

- The current baseline passes formatting, clippy, the full test suite, and a release build in the audited environment.
- Rust 1.94.1 and the ratatui 0.30/crossterm 0.29 family are pinned. R9 commit `41a1b63f11a57a671b0f705883431dab24298b5a` passed both OS quality gates in [GitHub Actions run `32034295607`](https://github.com/Yupkidangju/AIHack/actions/runs/32034295607).
- R1 build reproducibility and R2 private-state transaction gates pass locally.
- R3's fallible `ContentRegistry` bootstrap and R4's policy-driven headless runner pass local verification.
- `survival-v1` verifies 1,000 accepted turns and three matching hashes for seeds 42, 7, and 1234. The R5 workspace and documentation remediation passed the `audit_report_9.md` re-audit.
- `audit_report_11.md` verified the public/schema-contract and reproducibility remediation from report 10 and closed the R6 checkpoint with PASS. A real-provider smoke remains non-blocking.
- R7 passes its engineering scope with known risks after verified checksums, a fail-closed legacy gate, and ten strengthened NH367 traces. On 2026-07-20 the project owner classified AIHack as an AI-assisted semantic-rewrite derivative of NetHack 3.6.7 and approved NGPL for the whole distribution.
- External publication still requires an R8 technical-audit PASS plus LICENSE, NOTICE, and the complete corresponding source for the released binaries.
- The project-owner decision is traceable as `AIHACK-OWNER-2026-07-20-NGPL-01`; release bundles carry PROJECT_OWNER_LICENSE_APPROVAL.md, MODIFICATIONS.md, commit-bound RELEASE-METADATA, and SHA256SUMS. Metadata IDs must resolve to the bundled records. No qualified legal opinion is claimed.
- `audit_report_21.md` closed report 20's documentation remediation with PASS; report 20 is no longer the current pending authority.
- `docs/audit/audit_report_23.md` placed new R9-witness, Windows-checkpoint, and save/replay filesystem findings on HOLD. The 2026-08-18 coder remediation adds capability-based artifact I/O, nine typed causal witnesses, and CRLF checkpoint regressions; independent re-audit is still pending, so external publication remains on hold.
- The follow-up findings in `docs/audit/audit_report_24.md` are locally remediated: cargo-deny 0.19.4 passes with a winx-only license exception, Windows save permissions are described as inherited from the parent DACL, and SC-CAUSE-01..07 map individually to code and tests. Clean same-SHA CI remains to be confirmed after push.
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

- 現在の基準では format、clippy、全 test、release build が通過しています。
- Rust 1.94.1 と ratatui 0.30/crossterm 0.29 は固定済みです。R5 workspace と文書是正は `audit_report_9.md` の再監査を通過しました。
- `audit_report_11.md` は、report 10 の public/schema contract と証拠再現性の是正をすべて Verified とし、R6 checkpoint を PASS で完了しました。実 provider smoke は非 blocking の検討事項です。
- AIHack は NetHack 3.6.7 の AI-assisted semantic rewrite による派生物としてプロジェクト所有者に承認され、配布物全体に NGPL を適用します。外部公開には R8 技術監査 PASS と LICENSE、NOTICE、対応する完全なソースが必要です。
- R9 の因果閉包では、食料・死体、armor、monster behavior、価格・難易度・gold・score、祈り・luck を実際の world state 変化へ接続し、seed 42/7/1234 の長期 test で semantic delta を検証します。
- report 21 は report 20 の文書修正を PASS で完了しました。report 23 の新しい filesystem・因果 witness・Windows checkpoint 修正はローカルで完了していますが、独立再監査と外部公開承認はまだ保留です。
- report 24 の追加 finding もローカルで修正済みです。cargo-deny 0.19.4、Windows の parent DACL 境界、SC-CAUSE-01..07 の個別 code/test 対応を検証し、push 後の同一 SHA 両 OS CI を待ちます。

### 現在の実行

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI は `default-run = "aihack"` で選択されます。headless は `-p aihack-headless --bin aihack-headless` で package と binary を指定します。実装順序と文書一覧は韓国語セクションを参照してください。

## 繁體中文

AIHack 是以具來源追蹤的情境與測試，重新實作 NetHack 3.6.7 可觀察行為的 Rust roguelike。目標不是逐行翻譯 C，而是 deterministic core、可追蹤的相容規則，以及隔離的 local LLM adapter。

### 目前狀態

- 目前基準已通過 format、clippy、全部 test 與 release build。
- Rust 1.94.1 與 ratatui 0.30/crossterm 0.29 已固定；R5 workspace 與文件修正已通過 `audit_report_9.md` 複審。
- `audit_report_11.md` 已驗證 report 10 的 public/schema contract 與證據可重現性修正，並以 PASS 完成 R6 checkpoint；實際 provider smoke 仍為非阻斷考量。
- 專案所有者已將 AIHack 核准為 NetHack 3.6.7 的 AI-assisted semantic rewrite 衍生作品，整體散布適用 NGPL。對外發布仍需 R8 技術稽核 PASS、LICENSE、NOTICE 與對應完整原始碼。
- R9 因果閉合把食物／屍體、armor、monster behavior、價格／難度／gold／score、祈禱／luck 連到實際 world state 變化，並以 seed 42/7/1234 長期測試驗證 semantic delta。
- report 21 已以 PASS 結束 report 20 的文件修正。report 23 的 filesystem、因果 witness 與 Windows checkpoint 修正已通過本機驗證，但仍待獨立複審與對外發布核准。
- report 24 的後續 finding 也已在本機修正：cargo-deny 0.19.4、Windows parent DACL 邊界，以及 SC-CAUSE-01..07 的個別程式碼／測試映射皆已驗證，尚待 push 後同一 SHA 的雙作業系統 CI。

### 目前執行

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI 由 `default-run = "aihack"` 選取；headless 以 `-p aihack-headless --bin aihack-headless` 指定 package 與 binary。實作順序與文件列表請參閱韓文段落。

## 简体中文

AIHack 是通过带来源追踪的场景和测试，重新实现 NetHack 3.6.7 可观察行为的 Rust roguelike。目标不是逐行翻译 C，而是 deterministic core、可追踪的兼容规则，以及隔离的 local LLM adapter。

### 当前状态

- 当前基线已通过 format、clippy、全部 test 和 release build。
- Rust 1.94.1 和 ratatui 0.30/crossterm 0.29 已固定；R5 workspace 与文档修正已通过 `audit_report_9.md` 复审。
- `audit_report_11.md` 已验证 report 10 的 public/schema contract 与证据可复现性修正，并以 PASS 完成 R6 checkpoint；实际 provider smoke 仍为非阻断考虑项。
- 项目所有者已将 AIHack 批准为 NetHack 3.6.7 的 AI-assisted semantic rewrite 衍生作品，整体分发适用 NGPL。对外发布仍需 R8 技术审计 PASS、LICENSE、NOTICE 与对应完整源代码。
- R9 因果闭合将食物／尸体、armor、monster behavior、价格／难度／gold／score、祈祷／luck 连接到实际 world state 变化，并用 seed 42/7/1234 长期测试验证 semantic delta。
- report 21 已以 PASS 结束 report 20 的文档修正。report 23 的 filesystem、因果 witness 与 Windows checkpoint 修正已通过本地验证，但仍待独立复审和对外发布批准。
- report 24 的后续 finding 也已在本地修正：cargo-deny 0.19.4、Windows parent DACL 边界，以及 SC-CAUSE-01..07 的逐项代码／测试映射均已验证，尚待 push 后同一 SHA 的双操作系统 CI。

### 当前运行

```bash
cargo run --locked -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

TUI 由 `default-run = "aihack"` 选择；headless 使用 `-p aihack-headless --bin aihack-headless` 指定 package 和 binary。实现顺序和文档列表请参阅韩文部分。

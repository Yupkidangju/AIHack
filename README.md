# AIHack

## 한국어

AIHack은 NetHack의 핵심 재미를 참고하되, Rust-native 턴 엔진과 AI 연결성을 우선으로 다시 설계하는 로그라이크 프로젝트다.

이 저장소의 이전 NetHack Rust 포트는 `legacy_nethack_port_reference/`로 격리되었다. 새 개발은 루트 문서세트를 기준으로 진행한다.

### 현재 상태

- 이전 포트: `legacy_nethack_port_reference/`
- 전역 작업 규칙: `AGENTS.md`
- AI 구현 문서 표준: `AI_IMPLEMENTATION_DOC_STANDARD.md`
- 새 마스터 스펙: `spec.md`
- 새 구현 요약: `implementation_summary.md`
- 새 감사 로드맵: `audit_roadmap.md`
- Phase 5 실행 대상: `cargo run --bin aihack-headless -- --seed 42 --turns 100`

### 기능

- Rust-native `GameSession` 단일 상태 원천
- seed 기반 deterministic `GameRng`
- `CommandIntent::Wait` 기반 Phase 1 headless 턴 진행
- stable FNV-1a snapshot hash
- `aihack-headless` runner의 seed/turns/final hash 출력
- 레거시 코드 직접 import 금지 경계
- 40x20 Phase 2 fixture map, 이동/문/시야/visible tiles
- Phase 3 `EntityStore`, bump attack, jackal/goblin 전투, 사망 event, player `GameOver`
- Phase 4 item entity, inventory letter, pickup, dagger wield, healing potion quaff
- Phase 5 fixed 2-level registry, level-aware entity location, descend/ascend stairs, level snapshot hash

### 핵심 방향

- Headless 테스트와 replay가 가능한 결정론적 런타임
- AI가 읽을 수 있는 닫힌 `Observation`
- AI가 실행할 수 있는 제한된 `ActionSpace`
- UI와 엔진의 강한 분리
- NetHack 호환 규칙의 선별적, 검증 가능한 흡수

### 문서 읽는 순서

1. `spec.md`
2. `implementation_summary.md`
3. `designs.md`
4. `audit_roadmap.md`
5. `BUILD_GUIDE.md`
6. `DESIGN_DECISIONS.md`
7. `legacy_nethack_port_reference/REFERENCE_INDEX.md`

### 개발 원칙

- 기존 레거시 코드는 참조만 한다.
- 새 런타임은 `legacy_nethack_port_reference/src`를 직접 import하지 않는다.
- 모든 기능은 headless 테스트가 먼저 통과해야 UI에 연결된다.
- LLM은 게임 상태를 직접 수정하지 않는다.
- AI 행동은 `ActionSpace`에 정의된 명령으로만 들어온다.

## English

AIHack is a roguelike project inspired by NetHack, rebuilt around a Rust-native turn engine and AI-friendly integration boundaries.

The previous Rust NetHack port is isolated under `legacy_nethack_port_reference/`. New development follows the root documentation set.

### Current Status

- Legacy port: `legacy_nethack_port_reference/`
- Global working rules: `AGENTS.md`
- AI documentation standard: `AI_IMPLEMENTATION_DOC_STANDARD.md`
- Master spec: `spec.md`
- Implementation summary: `implementation_summary.md`
- Audit roadmap: `audit_roadmap.md`
- Phase 5 runner: `cargo run --bin aihack-headless -- --seed 42 --turns 100`

### Features

- Rust-native single source of truth through `GameSession`
- Seed-based deterministic `GameRng`
- Phase 1 headless turn progression via `CommandIntent::Wait`
- Stable FNV-1a snapshot hash
- `aihack-headless` runner outputting seed, turns, and final hash
- Explicit boundary against direct legacy source imports
- 40x20 Phase 2 fixture map with movement, doors, vision, and visible tiles
- Phase 3 `EntityStore`, bump attacks, jackal/goblin combat, death events, and player `GameOver`
- Phase 4 item entities, inventory letters, pickup, dagger wielding, and healing potion quaffing
- Phase 5 fixed two-level registry, level-aware entity locations, descend/ascend stairs, and level snapshot hash

### Direction

- Deterministic headless runtime with replay-oriented validation
- Closed `Observation` contract for AI reads
- Restricted `ActionSpace` for AI actions
- Strong separation between UI and engine
- Selective, testable absorption of NetHack-compatible rules

## 日本語

AIHack は NetHack の中核的な面白さを参考にしつつ、Rust ネイティブのターンエンジンと AI 連携境界を優先して再設計するローグライクプロジェクトです。

以前の Rust 版 NetHack ポートは `legacy_nethack_port_reference/` に隔離されています。新規開発はルートの文書セットを基準に進めます。

### 現在の状態

- レガシーポート: `legacy_nethack_port_reference/`
- 全体作業ルール: `AGENTS.md`
- AI 実装文書標準: `AI_IMPLEMENTATION_DOC_STANDARD.md`
- マスタースペック: `spec.md`
- 実装サマリー: `implementation_summary.md`
- 監査ロードマップ: `audit_roadmap.md`
- Phase 5 ランナー: `cargo run --bin aihack-headless -- --seed 42 --turns 100`

### 機能

- `GameSession` による Rust ネイティブな単一状態源
- seed ベースの deterministic `GameRng`
- `CommandIntent::Wait` による Phase 1 headless ターン進行
- 安定した FNV-1a snapshot hash
- seed、turns、final hash を出力する `aihack-headless` ランナー
- レガシーソースの直接 import を禁止する明示的境界
- 40x20 Phase 2 fixture map、移動、扉、視界、visible tiles
- Phase 3 `EntityStore`、bump attack、jackal/goblin 戦闘、死亡 event、player `GameOver`
- Phase 4 item entity、inventory letter、pickup、dagger wield、healing potion quaff
- Phase 5 fixed two-level registry、level-aware entity location、descend/ascend stairs、level snapshot hash

### 方向性

- replay 検証に適した deterministic headless runtime
- AI 読み取り用の閉じた `Observation` 契約
- AI 行動用の制限された `ActionSpace`
- UI とエンジンの強い分離
- NetHack 互換ルールの選択的かつ検証可能な吸収

## 繁體中文

AIHack 是參考 NetHack 核心樂趣，並以 Rust-native 回合引擎與 AI 整合邊界為優先重新設計的 roguelike 專案。

先前的 Rust NetHack port 已隔離在 `legacy_nethack_port_reference/`。新的開發以根目錄文件集為準。

### 目前狀態

- 舊版 port：`legacy_nethack_port_reference/`
- 全域工作規則：`AGENTS.md`
- AI 實作文檔標準：`AI_IMPLEMENTATION_DOC_STANDARD.md`
- 主規格：`spec.md`
- 實作摘要：`implementation_summary.md`
- 稽核路線圖：`audit_roadmap.md`
- Phase 5 runner：`cargo run --bin aihack-headless -- --seed 42 --turns 100`

### 功能

- 透過 `GameSession` 建立 Rust-native 單一狀態來源
- 以 seed 為基礎的 deterministic `GameRng`
- 透過 `CommandIntent::Wait` 進行 Phase 1 headless 回合推進
- 穩定的 FNV-1a snapshot hash
- 輸出 seed、turns、final hash 的 `aihack-headless` runner
- 明確禁止直接 import 舊版 source 的邊界
- 40x20 Phase 2 fixture map、移動、門、視野、visible tiles
- Phase 3 `EntityStore`、bump attack、jackal/goblin 戰鬥、死亡 event、player `GameOver`
- Phase 4 item entity、inventory letter、pickup、dagger wield、healing potion quaff
- Phase 5 fixed two-level registry、level-aware entity location、descend/ascend stairs、level snapshot hash

### 方向

- 可用於 replay 驗證的 deterministic headless runtime
- AI 讀取用的封閉 `Observation` 契約
- AI 行動用的受限 `ActionSpace`
- UI 與引擎的強分離
- 選擇性且可驗證地吸收 NetHack 相容規則

## 简体中文

AIHack 是参考 NetHack 核心乐趣，并以 Rust-native 回合引擎和 AI 集成边界为优先重新设计的 roguelike 项目。

之前的 Rust NetHack port 已隔离在 `legacy_nethack_port_reference/`。新的开发以根目录文档集为准。

### 当前状态

- 旧版 port：`legacy_nethack_port_reference/`
- 全局工作规则：`AGENTS.md`
- AI 实现文档标准：`AI_IMPLEMENTATION_DOC_STANDARD.md`
- 主规格：`spec.md`
- 实现摘要：`implementation_summary.md`
- 审计路线图：`audit_roadmap.md`
- Phase 5 runner：`cargo run --bin aihack-headless -- --seed 42 --turns 100`

### 功能

- 通过 `GameSession` 建立 Rust-native 单一状态来源
- 基于 seed 的 deterministic `GameRng`
- 通过 `CommandIntent::Wait` 进行 Phase 1 headless 回合推进
- 稳定的 FNV-1a snapshot hash
- 输出 seed、turns、final hash 的 `aihack-headless` runner
- 明确禁止直接 import 旧版 source 的边界
- 40x20 Phase 2 fixture map、移动、门、视野、visible tiles
- Phase 3 `EntityStore`、bump attack、jackal/goblin 战斗、死亡 event、player `GameOver`
- Phase 4 item entity、inventory letter、pickup、dagger wield、healing potion quaff
- Phase 5 fixed two-level registry、level-aware entity location、descend/ascend stairs、level snapshot hash

### 方向

- 可用于 replay 验证的 deterministic headless runtime
- AI 读取用的封闭 `Observation` 契约
- AI 行动用的受限 `ActionSpace`
- UI 与引擎的强分离
- 选择性且可验证地吸收 NetHack 兼容规则

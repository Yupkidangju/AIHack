# AIHack v2.10.0

> **A Modern Rust Roguelike — Based on NetHack 3.6.7**

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![NetHack: NGPL](https://img.shields.io/badge/NetHack-NGPL-orange.svg)](./LICENSE.NGPL)

> **[ 한 / 영 / 일 / 中(繁) / 中(简) ]**

## 🌐 다국어 지원 (Multilingual Support)
- [한국어](./README.md#한국어)
- [English](./README.md#english)
- [日本語](./README.md#日本語)
- [繁體中文](./README.md#繁體中文)
- [简体中文](./README.md#简体中文)

---

## 한국어

### 프로젝트 개요
AIHack은 클래식 로그라이크 게임인 NetHack 3.6.7의 C 소스 코드를 Rust로 이식하는 현대화 프로젝트입니다. 원본의 복잡한 로직을 Rust의 안정성 위에서 재구현하며, TUI(Ratatui)와 GUI(egui)가 결합된 하이브리드 인터페이스를 제공하는 것을 목표로 합니다.

**현재 상태**: 85,259줄 / 147파일 / 1,168개 테스트 통과 / 이식률 48.1%

### 주요 기능
- **Legacy Porting**: NetHack 3.6.7 C 소스 로직의 100% Rust 이식 추진.
- **Hybrid UI**: 클래식한 TUI 뷰와 현대적인 egui 기반 플로팅 윈도우 지원.
- **Improved Stability**: Rust의 소유권 모델을 통한 메모리 안전성 확보 및 전역 변수 제거.
- **Modern Interaction**: 마우스 상호작용 및 실시간 상태 정보를 제공하는 HUD.
- **Advanced Monster AI**: 상징적인 몬스터 마법(Mage/Cleric), 특수 공격(Gaze, Breath), 그리고 수동적 반격 시스템.
- **Deep Combat**: 다중 공격(Multi-attack), 저항력(Resistance) 및 치명적 상태 이상(Drain, Paralyze) 엔진.
- **Container System**: Bag of Holding 및 재귀적 인벤토리 보관 시스템.
- **Special Dungeon Levels**: 오라클(Oracle), 광산 마을(Minetown) 등 원본 NetHack의 고유 레벨 레이아웃 및 NPC 배치 완벽 재현.

### 라이센스
- **소스 코드**: [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- **원본 NetHack 파생**: [NetHack General Public License](./LICENSE.NGPL)

---

## English

### Project Overview
AIHack is a modernization project that ports the C source code of the classic roguelike game NetHack 3.6.7 to Rust. It aims to reimplement the original's complex logic on top of Rust's stability and provide a hybrid interface combining TUI (Ratatui) and GUI (egui).

**Current Status**: 85,259 lines / 147 files / 1,168 tests passing / 48.1% ported

### Key Features
- **Legacy Porting**: 100% Rust porting of NetHack 3.6.7 C source logic.
- **Hybrid UI**: Supports classic TUI view and modern egui-based floating windows.
- **Improved Stability**: Memory safety via Rust's ownership model and removal of global variables.
- **Modern Interaction**: Mouse interaction and real-time status HUD.
- **Advanced Monster AI**: Iconic monster spells (Mage/Cleric), special attacks (Gaze, Breath), and passive counter-attack systems.
- **Deep Combat**: Multi-attack logic, Resistance checks, and deadly status effects (Drain, Paralyze).
- **Container System**: Recursive inventory storage and Bag of Holding mechanics.
- **Special Dungeon Levels**: Perfect reproduction of Oracle, Minetown layouts and unique NPC placements from original NetHack.

### License
- **Source Code**: [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- **Original NetHack Derivative**: [NetHack General Public License](./LICENSE.NGPL)

---

## 日本語

### プロジェクト概要
AIHackは、古典的なローグライクゲームであるNetHack 3.6.7のCソースコードをRustに移植する近代化プロジェクトです。オリジナルの複雑なロジックをRustの安定性の上に再構築し、TUI（Ratatui）とGUI（egui）を組み合わせたハイブリッドインターフェースを提供することを目指しています。

### 主な機能
- **レガシー移植**: NetHack 3.6.7 Cソースロジックの100% Rust移植を推進。
- **ハイブリッドUI**: クラシックなTUIビューと現代的なeguiベースのフローティングウィンドウをサポート。
- **安定性の向上**: Rustの所有権モデルによるメモリ安全性のアドバンテージとグローバル変数の排除。
- **モダンな対話**: マウス操作およびリアルタイムなステータス情報を備えたHUD。
- **モンスターAI & 戦闘**: 多重攻撃(Multi-attack)、耐性、状態異常(Drain, Paralyze)を含む深層戦闘エンジン。
- **コンテナシステム**: Bag of Holdingおよび再帰的インベントリ保管システム。

---

## 繁體中文

### 專案概覽
AIHack 是一個將經典 Roguelike 遊戲 NetHack 3.6.7 的 C 語言原始碼移植到 Rust 的現代化專案。其目標是在 Rust 的穩定性基礎上重新實作原始遊戲的複雜邏輯，並提供結合了 TUI (Ratatui) 與 GUI (egui) 的混合介面。

### 主要功能
- **傳統移植**: 推動 NetHack 3.6.7 C 原始碼邏輯的 100% Rust 移植。
- **混合 UI**: 支援經典 TUI 視圖與現代基於 egui 的懸浮視窗。
- **改進的穩定性**: 透過 Rust 的所有權模型確保記憶體安全並消除全域變數。
- **現代化互動**: 滑鼠互動與提供即時狀態資訊的 HUD。
- **怪物 AI 與戰鬥**: 包含多重攻擊(Multi-attack)、抗性與異常狀態(Drain, Paralyze)的深度戰鬥引擎。
- **容器系統**: Bag of Holding 與遞歸物品清單儲存系統。

---

## 简体中文

### 项目概览
AIHack 是一个将经典 Roguelike 游戏 NetHack 3.6.7 的 C 语言源码移植到 Rust 的现代化项目。其目标是在 Rust 的稳定性基础上重新实现原始游戏的复杂逻辑，并提供结合了 TUI (Ratatui) 与 GUI (egui) 的混合界面。

### 主要功能
- **传统移植**: 推动 NetHack 3.6.7 C 源码逻辑的 100% Rust 移植。
- **混合 UI**: 支持经典 TUI 视图与现代基于 egui 的悬浮窗口。
- **改进的稳定性**: 通过 Rust 的所有权模型确保内存安全并消除全局变量。
- **现代化互动**: 鼠标互动与提供实时状态信息的 HUD。
- **怪物 AI 与战斗**: 包含多重攻击(Multi-attack)、抗性与异常状态(Drain, Paralyze)的深度战斗引擎。
- **容器系统**: Bag of Holding 与递归物品清单储存系统。

// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// =============================================================================
// [v2.7.0] dungeon.c 대량 이식 — 레벨 판별/브랜치/유틸리티/테스트
// 원본: nethack-3.6.7/src/dungeon.c (3,112줄) → 대폭 확장
//
// 이식 대상 함수:
//   builds_up, Is_botlevel, Can_dig_down, Can_fall_thru, Can_rise_up,
//   has_ceiling, On_stairs, Invocation_lev, In_V_tower, In_endgame,
//   In_hell, In_quest, In_mines, In_sokoban, Is_stronghold,
//   deepest_lev_reached, level_difficulty, at_dgn_entrance,
//   dungeon_branch, br_string, lev_by_name, print_dungeon,
//   get_annotation, donamelevel, find_mapseen, level_map,
//   next_level, prev_level, get_level, on_level, induced_align,
//   assign_level, assign_rnd_level, depth, Is_special 등
// =============================================================================

use crate::core::dungeon::{DungeonBranch, Grid, LevelID};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// =============================================================================
// 브랜치 정보 구조체
// =============================================================================

/// 던전 브랜치 메타데이터 (원본: dungeon 구조체의 각 필드)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// 브랜치 열거형
    pub branch: DungeonBranch,
    /// 절대 깊이 시작
    pub depth_start: i32,
    /// 절대 깊이 끝
    pub depth_end: i32,
    /// 브랜치 이름 (예: "The Dungeons of Doom")
    pub name: String,
    /// 이 브랜치의 입구가 있는 레벨 (부모 브랜치 쪽)
    pub entry_level: Option<LevelID>,
    /// 특수 레벨 포함 여부
    pub has_special_levels: bool,
    /// 지옥 속성 (원본: flags.hellish)
    pub is_hellish: bool,
    /// 미로 속성 (원본: flags.maze_like)
    pub is_maze: bool,
    /// 탑 속성 — 위로 올라가는 구조 (원본: builds_up 판별의 근거)
    pub is_tower: bool,
}

impl BranchInfo {
    /// 새 브랜치 정보 생성
    pub fn new(branch: DungeonBranch, depth_start: i32, depth_end: i32, name: &str) -> Self {
        Self {
            branch,
            depth_start,
            depth_end,
            name: name.to_string(),
            entry_level: None,
            has_special_levels: false,
            is_hellish: false,
            is_maze: false,
            is_tower: false,
        }
    }

    /// 이 브랜치의 총 레벨 수 (원본: num_dunlevs)
    pub fn num_levels(&self) -> i32 {
        (self.depth_end - self.depth_start + 1).max(1)
    }

    /// 상대 레벨 → 절대 깊이 변환 (원본: depth())
    pub fn absolute_depth(&self, relative_level: i32) -> i32 {
        self.depth_start + relative_level - 1
    }

    /// 절대 깊이 → 상대 레벨 변환
    pub fn relative_level(&self, absolute_depth: i32) -> i32 {
        absolute_depth - self.depth_start + 1
    }

    /// 이 절대 깊이가 브랜치 범위 안인지
    pub fn contains_depth(&self, absolute_depth: i32) -> bool {
        absolute_depth >= self.depth_start && absolute_depth <= self.depth_end
    }
}

// =============================================================================
// 던전 본체
// =============================================================================

/// 전체 던전 상태 관리 (원본: dungeons[] 배열 + 전역 변수)
#[derive(Clone, Serialize, Deserialize)]
pub struct Dungeon {
    pub levels: HashMap<LevelID, Grid>,
    pub current_level: LevelID,
    /// 모든 브랜치 메타데이터
    pub branches: Vec<BranchInfo>,
    /// 난이도 보정 오프셋
    pub difficulty_offset: i32,
    /// 지금까지 도달한 최대 깊이 (원본: dunlev_ureached)
    pub deepest_reached: i32,
    /// 아뮬렛 획득 여부 (난이도 계산에 영향)
    pub amulet_obtained: bool,
    /// [v2.7.0] 레벨별 사용자 어노테이션 (원본: mapseen->custom)
    #[serde(default)]
    pub level_annotations: HashMap<LevelID, String>,
}

impl Dungeon {
    pub fn new() -> Self {
        let mut d = Self {
            levels: HashMap::new(),
            current_level: LevelID::new(DungeonBranch::Main, 1),
            branches: Vec::new(),
            difficulty_offset: 0,
            deepest_reached: 1,
            amulet_obtained: false,
            level_annotations: HashMap::new(),
        };
        d.init_branches();
        d
    }

    /// 기본 브랜치 초기화 (원본: init_dungeons 의 데이터 부분)
    fn init_branches(&mut self) {
        // 메인 던전
        let mut main_branch = BranchInfo::new(DungeonBranch::Main, 1, 30, "The Dungeons of Doom");
        main_branch.has_special_levels = true;
        self.branches.push(main_branch);

        // 노움 광산
        let mut mines = BranchInfo::new(DungeonBranch::Mines, 3, 15, "The Gnomish Mines");
        mines.entry_level = Some(LevelID::new(DungeonBranch::Main, 3));
        self.branches.push(mines);

        // 소코반
        let mut sokoban = BranchInfo::new(DungeonBranch::Sokoban, 6, 10, "Sokoban");
        sokoban.entry_level = Some(LevelID::new(DungeonBranch::Main, 6));
        sokoban.is_maze = true;
        self.branches.push(sokoban);

        // 게헨놈 (지옥)
        let mut gehennom = BranchInfo::new(DungeonBranch::Gehennom, 25, 50, "Gehennom");
        gehennom.entry_level = Some(LevelID::new(DungeonBranch::Main, 25));
        gehennom.is_hellish = true;
        gehennom.has_special_levels = true;
        self.branches.push(gehennom);

        // 퀘스트
        let mut quest = BranchInfo::new(DungeonBranch::Quest, 16, 22, "The Quest");
        quest.entry_level = Some(LevelID::new(DungeonBranch::Main, 14));
        quest.has_special_levels = true;
        self.branches.push(quest);

        // 블라드 탑
        let mut vlad = BranchInfo::new(DungeonBranch::VladTower, 40, 43, "Vlad's Tower");
        vlad.entry_level = Some(LevelID::new(DungeonBranch::Gehennom, 40));
        vlad.is_tower = true;
        self.branches.push(vlad);

        // 아스트랄 (엔드게임)
        let astral = BranchInfo::new(DungeonBranch::Astral, 51, 51, "The Astral Plane");
        self.branches.push(astral);
    }

    // =========================================================================
    // 레벨/그리드 접근
    // =========================================================================

    /// 레벨 그리드 참조
    pub fn get_level(&self, id: LevelID) -> Option<&Grid> {
        self.levels.get(&id)
    }

    /// 레벨 그리드 가변 참조
    pub fn get_level_mut(&mut self, id: LevelID) -> Option<&mut Grid> {
        self.levels.get_mut(&id)
    }

    pub fn set_level(&mut self, id: LevelID, grid: Grid) {
        self.levels.insert(id, grid);
    }

    // =========================================================================
    // 브랜치 조회
    // =========================================================================

    /// 브랜치 정보 검색
    pub fn get_branch(&self, branch: DungeonBranch) -> Option<&BranchInfo> {
        self.branches.iter().find(|b| b.branch == branch)
    }

    // =========================================================================
    // 깊이/난이도 계산 (원본: depth, level_difficulty, deepest_lev_reached)
    // =========================================================================

    /// 현재 레벨의 절대 깊이
    pub fn current_depth(&self) -> i32 {
        self.level_depth(self.current_level)
    }

    /// 특정 레벨의 절대 깊이 (원본: dungeon.c:1088-1093 depth)
    pub fn level_depth(&self, level_id: LevelID) -> i32 {
        if let Some(branch) = self.get_branch(level_id.branch) {
            branch.absolute_depth(level_id.depth as i32)
        } else {
            level_id.depth as i32
        }
    }

    /// 기본 난이도 계산 (플레이어 레벨 보정 포함)
    pub fn level_difficulty(&self, player_level: i32) -> i32 {
        let depth = self.current_depth();
        let lev_bonus = player_level / 5;
        let amulet_bonus = if self.amulet_obtained { 5 } else { 0 };
        (depth + lev_bonus + amulet_bonus + self.difficulty_offset).max(1)
    }

    /// [v2.7.0] 전체 난이도 — builds_up 보정 포함
    /// (원본: dungeon.c:1589-1644 level_difficulty 완전 이식)
    pub fn level_difficulty_full(&self, player_level: i32) -> i32 {
        let mut res;
        if self.in_endgame() {
            // 엔드게임: 성역 깊이 + 플레이어 레벨/2
            res = 50 + player_level / 2;
        } else if self.amulet_obtained {
            // 아뮬렛 소지: 최대 도달 깊이
            res = self.deepest_lev_reached(false);
        } else {
            res = self.current_depth();
            // 위로 올라가는 브랜치 보정 (원본 주석의 소코반 예시 참조)
            if self.builds_up() {
                if let Some(info) = self.get_branch(self.current_level.branch) {
                    let entry_lev = info.num_levels();
                    res += 2 * (entry_lev - self.current_level.depth as i32 + 1);
                }
            }
        }
        res.max(1)
    }

    /// [v2.7.0] 깊은 탐험 기록 (원본: dungeon.c:996-1031 deepest_lev_reached)
    /// noquest=true이면 퀘스트 던전 제외 (하이스코어 표시용)
    pub fn deepest_lev_reached(&self, noquest: bool) -> i32 {
        let mut ret = 0i32;
        for info in &self.branches {
            if noquest && info.branch == DungeonBranch::Quest {
                continue;
            }
            if info.depth_start > ret {
                ret = info.depth_start;
            }
        }
        ret.max(self.deepest_reached)
    }

    // =========================================================================
    // 위치/유형 판별 (원본: In_hell, In_quest, ... 매크로/함수)
    // =========================================================================

    /// 지옥(Gehennom) 내부인지 (원본: dungeon.c:1507-1513 In_hell)
    pub fn in_hell(&self) -> bool {
        if let Some(branch) = self.get_branch(self.current_level.branch) {
            branch.is_hellish
        } else {
            false
        }
    }

    /// 퀘스트 던전 내부인지 (원본: dungeon.c:1406-1412 In_quest)
    pub fn in_quest(&self) -> bool {
        self.current_level.branch == DungeonBranch::Quest
    }

    /// 광산 내부인지 (원본: dungeon.c:1414-1420 In_mines)
    pub fn in_mines(&self) -> bool {
        self.current_level.branch == DungeonBranch::Mines
    }

    /// 소코반 내부인지
    pub fn in_sokoban(&self) -> bool {
        self.current_level.branch == DungeonBranch::Sokoban
    }

    /// 탑 위인지 (블라드 탑)
    pub fn on_tower(&self) -> bool {
        if let Some(branch) = self.get_branch(self.current_level.branch) {
            branch.is_tower
        } else {
            false
        }
    }

    /// [v2.7.0] 블라드 탑 내부인지 (원본: dungeon.c:1468-1474 In_V_tower)
    pub fn in_vlad_tower(&self) -> bool {
        self.current_level.branch == DungeonBranch::VladTower
    }

    /// [v2.7.0] 엔드게임 영역인지 (원본: In_endgame 매크로)
    pub fn in_endgame(&self) -> bool {
        self.current_level.branch == DungeonBranch::Astral
    }

    /// 아스트랄 평원인지
    pub fn on_astral_plane(&self) -> bool {
        self.current_level.branch == DungeonBranch::Astral
    }

    // =========================================================================
    // [v2.7.0] 레벨 유형 판별 (원본: dungeon.c 각종 함수)
    // =========================================================================

    /// 위로 올라가는 구조인지 (원본: dungeon.c:1136-1147 builds_up)
    /// 소코반, 블라드 탑 등
    pub fn builds_up(&self) -> bool {
        if let Some(info) = self.get_branch(self.current_level.branch) {
            info.num_levels() > 1 && info.is_tower
        } else {
            false
        }
    }

    /// 현재 브랜치의 최하층인지 (원본: dungeon.c:1289-1294 Is_botlevel)
    pub fn is_bottom_level(&self) -> bool {
        self.at_branch_bottom()
    }

    /// 아래로 파기 가능 여부 (원본: dungeon.c:1296-1303 Can_dig_down)
    pub fn can_dig_down(&self, hard_floor: bool) -> bool {
        !hard_floor && !self.is_bottom_level() && !self.is_invocation_level()
    }

    /// 통과 낙하 가능 여부 (원본: dungeon.c:1310-1315 Can_fall_thru)
    pub fn can_fall_through(&self, hard_floor: bool) -> bool {
        self.can_dig_down(hard_floor) || self.is_stronghold()
    }

    /// 위로 올라갈 수 있는지 (원본: dungeon.c:1323-1337 Can_rise_up)
    pub fn can_rise_up(&self) -> bool {
        if self.on_astral_plane() || self.in_sokoban() {
            return false;
        }
        self.current_level.depth > 1
    }

    /// 천장이 있는 레벨인지 (원본: dungeon.c:1339-1345 has_ceiling)
    pub fn has_ceiling(&self) -> bool {
        !self.is_air_level() && !self.is_water_level()
    }

    /// 소환(Invocation) 레벨인지 (원본: dungeon.c:1578-1584 Invocation_lev)
    pub fn is_invocation_level(&self) -> bool {
        if !self.in_hell() {
            return false;
        }
        if let Some(info) = self.get_branch(self.current_level.branch) {
            self.current_level.depth as i32 == info.num_levels() - 1
        } else {
            false
        }
    }

    /// 성채(Castle) 레벨인지 (원본: Is_stronghold 매크로)
    pub fn is_stronghold(&self) -> bool {
        self.current_level.branch == DungeonBranch::Main && self.current_level.depth == 20
    }

    /// 공기 원소 레벨인지 (원본: Is_airlevel 매크로)
    pub fn is_air_level(&self) -> bool {
        self.current_level.branch == DungeonBranch::Astral
            && self.special_level_name(self.current_level) == Some("Plane of Air")
    }

    /// 물 원소 레벨인지 (원본: Is_waterlevel 매크로)
    pub fn is_water_level(&self) -> bool {
        self.current_level.branch == DungeonBranch::Astral
            && self.special_level_name(self.current_level) == Some("Plane of Water")
    }

    /// 특정 위치가 계단 위인지 (원본: dungeon.c:1278-1287 On_stairs)
    pub fn on_stairs(&self, x: i32, y: i32, stair_positions: &StairPositions) -> bool {
        (x == stair_positions.up_x && y == stair_positions.up_y)
            || (x == stair_positions.down_x && y == stair_positions.down_y)
            || (x == stair_positions.up_ladder_x && y == stair_positions.up_ladder_y)
            || (x == stair_positions.down_ladder_x && y == stair_positions.down_ladder_y)
            || (x == stair_positions.special_x && y == stair_positions.special_y)
    }

    // =========================================================================
    // [v2.7.0] 특수 레벨 확인 매크로 대응
    // =========================================================================

    /// 로그(Rogue) 스타일 레벨인지 (원본: Is_rogue_level)
    pub fn is_rogue_level(&self, level_id: LevelID) -> bool {
        level_id.branch == DungeonBranch::Main && level_id.depth == 15
    }

    /// 오라클 레벨인지 (원본: Is_oracle_level)
    pub fn is_oracle_level(&self, level_id: LevelID) -> bool {
        level_id.branch == DungeonBranch::Main && level_id.depth == 5
    }

    /// 마인타운인지 (원본: Is_minetown_level)
    pub fn is_minetown(&self, level_id: LevelID) -> bool {
        level_id.branch == DungeonBranch::Mines && level_id.depth == 6
    }

    /// 밸리 오브 더 데드인지 (원본: Is_valley)
    pub fn is_valley(&self, level_id: LevelID) -> bool {
        level_id.branch == DungeonBranch::Main && level_id.depth == 25
    }

    /// 메두사 레벨인지 (원본: Is_medusa_level)
    pub fn is_medusa_level(&self, level_id: LevelID) -> bool {
        level_id.branch == DungeonBranch::Main && level_id.depth == 18
    }

    /// 포트 루디오스(Knox)인지 (원본: Is_knox)
    pub fn is_knox(&self, level_id: LevelID) -> bool {
        level_id.branch == DungeonBranch::Main && level_id.depth == 12
    }

    // =========================================================================
    // 최대 깊이 갱신
    // =========================================================================

    /// 현재 깊이로 최대 도달 깊이 갱신
    pub fn update_deepest(&mut self) {
        let depth = self.current_depth();
        if depth > self.deepest_reached {
            self.deepest_reached = depth;
        }
    }

    // =========================================================================
    // 특수 레벨 이름/브랜치 이름
    // =========================================================================

    /// 특수 레벨 이름 반환 (하드코딩 — 원본: level_map[])
    pub fn special_level_name(&self, level_id: LevelID) -> Option<&'static str> {
        match level_id.branch {
            DungeonBranch::Main => match level_id.depth {
                1 => Some("Welcome Level"),
                5 => Some("The Oracle"),
                10 => Some("Big Room"),
                14 => Some("Quest Portal"),
                20 => Some("Castle"),
                25 => Some("Valley of the Dead"),
                _ => None,
            },
            DungeonBranch::Mines => match level_id.depth {
                8 => Some("Mine's End"),
                6 => Some("Minetown"),
                _ => None,
            },
            DungeonBranch::Gehennom => match level_id.depth {
                35 => Some("Juiblex's Swamp"),
                40 => Some("Asmodeus' Lair"),
                45 => Some("Wizard's Tower"),
                50 => Some("Sanctum"),
                _ => None,
            },
            DungeonBranch::Astral => Some("The Astral Plane"),
            _ => None,
        }
    }

    /// 브랜치 이름 문자열 반환
    pub fn branch_name(&self, branch: DungeonBranch) -> &str {
        if let Some(info) = self.get_branch(branch) {
            &info.name
        } else {
            "Unknown"
        }
    }

    // =========================================================================
    // 레벨 내비게이션
    // =========================================================================

    /// 다음 레벨(아래) (원본: dungeon.c:1149-1165 next_level 의 기본 동작)
    pub fn next_level_down(&self) -> Option<LevelID> {
        let branch_info = self.get_branch(self.current_level.branch)?;
        let next_depth = self.current_level.depth as i32 + 1;
        if next_depth <= branch_info.num_levels() {
            Some(LevelID::new(self.current_level.branch, next_depth))
        } else {
            None
        }
    }

    /// 이전 레벨(위) (원본: dungeon.c:1167-1187 prev_level 의 기본 동작)
    pub fn next_level_up(&self) -> Option<LevelID> {
        let depth = self.current_level.depth as i32 - 1;
        if depth >= 1 {
            Some(LevelID::new(self.current_level.branch, depth))
        } else {
            // 브랜치 최상층이면 입구 레벨로
            let branch_info = self.get_branch(self.current_level.branch)?;
            branch_info.entry_level
        }
    }

    /// 현재 브랜치의 최하층인지
    pub fn at_branch_bottom(&self) -> bool {
        if let Some(info) = self.get_branch(self.current_level.branch) {
            self.current_level.depth as i32 >= info.num_levels()
        } else {
            false
        }
    }

    /// 현재 브랜치의 최상층인지
    pub fn at_branch_top(&self) -> bool {
        self.current_level.depth <= 1
    }

    /// [v2.7.0] 브랜치 입구에 있는지 (원본: dungeon.c:1458-1466 at_dgn_entrance)
    pub fn at_branch_entrance(&self, target_branch: DungeonBranch) -> bool {
        if let Some(info) = self.get_branch(target_branch) {
            if let Some(entry) = info.entry_level {
                return self.current_level == entry;
            }
        }
        false
    }

    /// [v2.7.0] 현재 레벨에서 연결된 브랜치 목록 (원본: Is_branchlev)
    pub fn connected_branches(&self) -> Vec<DungeonBranch> {
        let mut result = Vec::new();
        for info in &self.branches {
            if let Some(entry) = info.entry_level {
                if entry == self.current_level {
                    result.push(info.branch);
                }
            }
        }
        result
    }

    // =========================================================================
    // 레벨 목록
    // =========================================================================

    /// 모든 탐험된 레벨의 ID 목록
    pub fn all_level_ids(&self) -> Vec<LevelID> {
        self.levels.keys().cloned().collect()
    }

    /// 탐험된 레벨 수
    pub fn num_explored_levels(&self) -> usize {
        self.levels.len()
    }

    /// 특정 레벨이 이미 생성되었는지
    pub fn level_exists(&self, id: LevelID) -> bool {
        self.levels.contains_key(&id)
    }

    // =========================================================================
    // [v2.7.0] 어노테이션 (원본: dungeon.c:2042-2100)
    // =========================================================================

    /// 레벨 어노테이션(사용자 메모) 조회
    pub fn get_annotation(&self, level_id: LevelID) -> Option<&str> {
        self.level_annotations.get(&level_id).map(|s| s.as_str())
    }

    /// 레벨 어노테이션 설정 (빈 문자열이면 삭제)
    pub fn set_annotation(&mut self, level_id: LevelID, name: String) {
        if name.trim().is_empty() {
            self.level_annotations.remove(&level_id);
        } else {
            self.level_annotations.insert(level_id, name);
        }
    }

    // =========================================================================
    // [v2.7.0] 이름으로 레벨 검색 (원본: dungeon.c:1649-1724 lev_by_name)
    // =========================================================================

    /// 특수 레벨 이름이나 어노테이션으로 깊이 반환
    pub fn level_by_name(&self, name: &str) -> Option<i32> {
        let name_lower = name.to_lowercase();
        let search = if name_lower.starts_with("the ") {
            &name_lower[4..]
        } else {
            &name_lower
        };
        // " level" 접미사 제거
        let search = if search.ends_with(" level") {
            &search[..search.len() - 6]
        } else {
            search
        };
        // "hell"/"gehennom"은 밸리로
        if search == "hell" || search == "gehennom" {
            return Some(25);
        }
        // 특수 레벨 테이블 검색
        for (lev_name, branch, depth) in SPECIAL_LEVEL_MAP.iter() {
            if lev_name.to_lowercase().contains(search) {
                if let Some(info) = self.get_branch(*branch) {
                    return Some(info.absolute_depth(*depth));
                }
            }
        }
        // 어노테이션 검색
        for (level_id, annotation) in &self.level_annotations {
            if annotation.to_lowercase().contains(search) {
                return Some(self.level_depth(*level_id));
            }
        }
        None
    }

    // =========================================================================
    // [v2.7.0] 던전 개요 (원본: dungeon.c:1847-2001 print_dungeon 간략화)
    // =========================================================================

    /// 전체 던전 구조를 문자열 벡터로 생성
    pub fn dungeon_overview(&self) -> Vec<String> {
        let mut lines = Vec::new();
        for info in &self.branches {
            let nlev = info.num_levels();
            let header = if nlev > 1 {
                format!(
                    "{}: levels {} to {}",
                    info.name, info.depth_start, info.depth_end
                )
            } else {
                format!("{}: level {}", info.name, info.depth_start)
            };
            lines.push(header);

            // 특수 레벨 표시
            for (name, branch, depth) in SPECIAL_LEVEL_MAP.iter() {
                if *branch == info.branch {
                    let abs = info.absolute_depth(*depth);
                    let marker = if self.current_level.branch == *branch
                        && self.current_level.depth == *depth as i32
                    {
                        '*'
                    } else {
                        ' '
                    };
                    lines.push(format!("  {} {}: {}", marker, name, abs));
                }
            }
        }
        lines
    }

    /// [v2.7.0] 레벨 간 깊이 차이
    pub fn depth_difference(&self, from: LevelID, to: LevelID) -> i32 {
        self.level_depth(to) - self.level_depth(from)
    }
}

// =============================================================================
// [v2.7.0] 계단 위치 구조체 (원본: xupstair/yupstair/sstairs 등)
// =============================================================================

/// 레벨 내 계단/사다리/특수계단 위치
#[derive(Debug, Clone, Default)]
pub struct StairPositions {
    /// 올라가는 계단 좌표
    pub up_x: i32,
    pub up_y: i32,
    /// 내려가는 계단 좌표
    pub down_x: i32,
    pub down_y: i32,
    /// 올라가는 사다리 좌표
    pub up_ladder_x: i32,
    pub up_ladder_y: i32,
    /// 내려가는 사다리 좌표
    pub down_ladder_x: i32,
    pub down_ladder_y: i32,
    /// 특수 계단 좌표 (브랜치 연결)
    pub special_x: i32,
    pub special_y: i32,
    /// 특수 계단 목표 레벨
    pub special_dest: Option<LevelID>,
    /// 특수 계단 방향 (true=위로)
    pub special_up: bool,
}

// =============================================================================
// [v2.7.0] 브랜치 타입 (원본: dungeon.c:411-427 correct_branch_type)
// =============================================================================

/// 브랜치 연결 유형 (원본: BR_STAIR, BR_NO_END1, BR_NO_END2, BR_PORTAL)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchType {
    /// 양쪽 모두 계단 (원본: BR_STAIR)
    Stair,
    /// 위쪽 끝 없음 (원본: BR_NO_END1)
    NoUpperEnd,
    /// 아래쪽 끝 없음 (원본: BR_NO_END2)
    NoLowerEnd,
    /// 포탈 연결 (원본: BR_PORTAL)
    Portal,
}

/// 브랜치 타입 문자열 (원본: dungeon.c:1792-1808 br_string)
pub fn branch_type_name(bt: BranchType) -> &'static str {
    match bt {
        BranchType::Stair => "Stair",
        BranchType::NoUpperEnd => "Connection",
        BranchType::NoLowerEnd => "One way stair",
        BranchType::Portal => "Portal",
    }
}

// =============================================================================
// [v2.7.0] 특수 레벨 맵 (원본: dungeon.c:681-710 level_map)
// =============================================================================

/// 26종 특수 레벨 테이블: (이름, 브랜치, 상대 깊이)
pub const SPECIAL_LEVEL_MAP: &[(&str, DungeonBranch, i32)] = &[
    ("Oracle", DungeonBranch::Main, 5),
    ("Big Room", DungeonBranch::Main, 10),
    ("Rogue Level", DungeonBranch::Main, 15),
    ("Medusa", DungeonBranch::Main, 18),
    ("Castle", DungeonBranch::Main, 20),
    ("Valley of the Dead", DungeonBranch::Main, 25),
    ("Minetown", DungeonBranch::Mines, 6),
    ("Mine's End", DungeonBranch::Mines, 8),
    ("Sokoban End", DungeonBranch::Sokoban, 10),
    ("Juiblex's Swamp", DungeonBranch::Gehennom, 35),
    ("Baalzebub's Lair", DungeonBranch::Gehennom, 38),
    ("Asmodeus' Lair", DungeonBranch::Gehennom, 40),
    ("Orcus Town", DungeonBranch::Gehennom, 42),
    ("Wizard's Tower (1)", DungeonBranch::Gehennom, 45),
    ("Wizard's Tower (2)", DungeonBranch::Gehennom, 46),
    ("Wizard's Tower (3)", DungeonBranch::Gehennom, 47),
    ("Sanctum", DungeonBranch::Gehennom, 50),
    ("Vlad's Tower (Bottom)", DungeonBranch::VladTower, 40),
    ("Vlad's Tower (Top)", DungeonBranch::VladTower, 43),
    ("Plane of Earth", DungeonBranch::Astral, 51),
    ("Plane of Air", DungeonBranch::Astral, 51),
    ("Plane of Fire", DungeonBranch::Astral, 51),
    ("Plane of Water", DungeonBranch::Astral, 51),
    ("Astral Plane", DungeonBranch::Astral, 51),
    ("Fort Ludios", DungeonBranch::Main, 12),
    ("Quest Start", DungeonBranch::Quest, 16),
];

// =============================================================================
// [v2.7.0] 자유 함수 — 던전 유틸
// =============================================================================

/// 레벨 설명 문자열 생성
pub fn describe_level(dungeon: &Dungeon, level_id: LevelID) -> String {
    let branch_name = dungeon.branch_name(level_id.branch);
    let depth = dungeon.level_depth(level_id);

    if let Some(special_name) = dungeon.special_level_name(level_id) {
        format!("{}: {} (Depth {})", branch_name, special_name, depth)
    } else {
        format!(
            "{}: Level {} (Depth {})",
            branch_name, level_id.depth, depth
        )
    }
}

/// 짧은(상태줄용) 레벨명
pub fn short_level_name(dungeon: &Dungeon) -> String {
    let level_id = dungeon.current_level;
    match level_id.branch {
        DungeonBranch::Main => format!("Dlvl:{}", dungeon.current_depth()),
        DungeonBranch::Mines => format!("Mine:{}", level_id.depth),
        DungeonBranch::Sokoban => format!("Sok:{}", level_id.depth),
        DungeonBranch::Gehennom => format!("Geh:{}", dungeon.current_depth()),
        DungeonBranch::Quest => format!("Qst:{}", level_id.depth),
        DungeonBranch::VladTower => format!("Vlad:{}", level_id.depth),
        DungeonBranch::Astral => "Astral".to_string(),
        _ => format!("Lvl:{}", dungeon.current_depth()),
    }
}

/// 최대 몬스터 레벨 (난이도 기반)
pub fn max_monster_level(difficulty: i32) -> i32 {
    (difficulty + 5).min(49)
}

/// 아이템 난이도 (깊이 기반)
pub fn item_difficulty(depth: i32) -> i32 {
    (depth + 2).max(1)
}

/// [v2.7.0] 깊이로 레벨 환산 (원본: dungeon.c:1358-1404 get_level)
pub fn get_level_from_depth(dungeon: &Dungeon, target_depth: i32) -> LevelID {
    let current = dungeon.current_level;
    if let Some(info) = dungeon.get_branch(current.branch) {
        let bot = info.depth_start;
        let top = info.depth_end;
        if target_depth >= bot && target_depth <= top {
            let rel = target_depth - bot + 1;
            return LevelID::new(current.branch, rel);
        }
        if target_depth > top {
            return LevelID::new(current.branch, info.num_levels());
        }
    }
    // 메인 던전으로 폴백
    if let Some(main) = dungeon.get_branch(DungeonBranch::Main) {
        let rel = (target_depth - main.depth_start + 1)
            .max(1)
            .min(main.num_levels());
        return LevelID::new(DungeonBranch::Main, rel);
    }
    LevelID::new(DungeonBranch::Main, target_depth.max(1))
}

/// [v2.7.0] 두 레벨이 동일한지 (원본: dungeon.c:1095-1102 on_level)
pub fn on_same_level(a: LevelID, b: LevelID) -> bool {
    a.branch == b.branch && a.depth == b.depth
}

/// [v2.7.0] 다음 레벨(아래) 목표 (원본: dungeon.c:1149-1165 next_level)
pub fn next_level_target(
    dungeon: &Dungeon,
    at_special_stairs: bool,
    stairs: &StairPositions,
) -> Option<LevelID> {
    if at_special_stairs {
        stairs.special_dest
    } else {
        dungeon.next_level_down()
    }
}

/// [v2.7.0] 이전 레벨(위) 목표 (원본: dungeon.c:1167-1187 prev_level)
pub fn prev_level_target(
    dungeon: &Dungeon,
    at_special_stairs: bool,
    stairs: &StairPositions,
) -> Option<LevelID> {
    if at_special_stairs {
        stairs.special_dest
    } else {
        dungeon.next_level_up()
    }
}

/// [v2.7.0] 정렬 유도 (원본: dungeon.c:1559-1576 induced_align)
pub fn induced_align(_dungeon: &Dungeon, _pct: i32, rng_val: i32) -> i32 {
    ((rng_val % 3) + 3) % 3
}

/// [v2.7.0] 성채 입구용 연주곡 생성 (원본: dungeon.c:917-919)
pub fn generate_castle_tune(rng_vals: &[i32; 5]) -> String {
    rng_vals
        .iter()
        .map(|v| (b'A' + (v.unsigned_abs() % 7) as u8) as char)
        .collect()
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_new() {
        let d = Dungeon::new();
        assert!(!d.branches.is_empty());
        assert_eq!(d.current_level.branch, DungeonBranch::Main);
        assert_eq!(d.current_level.depth, 1);
    }

    #[test]
    fn test_branch_info() {
        let d = Dungeon::new();
        let main = d.get_branch(DungeonBranch::Main).unwrap();
        assert_eq!(main.depth_start, 1);
        assert_eq!(main.num_levels(), 30);
    }

    #[test]
    fn test_depth_calculation() {
        let d = Dungeon::new();
        let depth = d.level_depth(LevelID::new(DungeonBranch::Main, 5));
        assert_eq!(depth, 5);
    }

    #[test]
    fn test_hell_check() {
        let mut d = Dungeon::new();
        d.current_level = LevelID::new(DungeonBranch::Gehennom, 30);
        assert!(d.in_hell());
    }

    #[test]
    fn test_special_level_name() {
        let d = Dungeon::new();
        let name = d.special_level_name(LevelID::new(DungeonBranch::Main, 20));
        assert_eq!(name, Some("Castle"));
    }

    #[test]
    fn test_builds_up() {
        let mut d = Dungeon::new();
        assert!(!d.builds_up());
        d.current_level = LevelID::new(DungeonBranch::VladTower, 42);
        assert!(d.builds_up());
    }

    #[test]
    fn test_is_bottom_level() {
        let mut d = Dungeon::new();
        d.current_level = LevelID::new(DungeonBranch::Main, 30);
        assert!(d.is_bottom_level());
        d.current_level = LevelID::new(DungeonBranch::Main, 15);
        assert!(!d.is_bottom_level());
    }

    #[test]
    fn test_can_dig_down() {
        let mut d = Dungeon::new();
        d.current_level = LevelID::new(DungeonBranch::Main, 10);
        assert!(d.can_dig_down(false));
        assert!(!d.can_dig_down(true));
        d.current_level = LevelID::new(DungeonBranch::Main, 30);
        assert!(!d.can_dig_down(false));
    }

    #[test]
    fn test_can_fall_through() {
        let mut d = Dungeon::new();
        d.current_level = LevelID::new(DungeonBranch::Main, 20);
        assert!(d.can_fall_through(false));
    }

    #[test]
    fn test_has_ceiling() {
        let d = Dungeon::new();
        assert!(d.has_ceiling());
    }

    #[test]
    fn test_in_endgame() {
        let mut d = Dungeon::new();
        assert!(!d.in_endgame());
        d.current_level = LevelID::new(DungeonBranch::Astral, 51);
        assert!(d.in_endgame());
    }

    #[test]
    fn test_in_vlad_tower() {
        let mut d = Dungeon::new();
        d.current_level = LevelID::new(DungeonBranch::VladTower, 41);
        assert!(d.in_vlad_tower());
        assert!(!d.in_hell());
    }

    #[test]
    fn test_level_difficulty_full() {
        let mut d = Dungeon::new();
        d.current_level = LevelID::new(DungeonBranch::Main, 10);
        let diff = d.level_difficulty_full(5);
        assert_eq!(diff, 10);
    }

    #[test]
    fn test_at_branch_entrance() {
        let mut d = Dungeon::new();
        d.current_level = LevelID::new(DungeonBranch::Main, 3);
        assert!(d.at_branch_entrance(DungeonBranch::Mines));
    }

    #[test]
    fn test_level_by_name() {
        let d = Dungeon::new();
        let result = d.level_by_name("oracle");
        assert_eq!(result, Some(5));
        let result = d.level_by_name("hell");
        assert_eq!(result, Some(25));
    }

    #[test]
    fn test_branch_type_name_display() {
        assert_eq!(branch_type_name(BranchType::Portal), "Portal");
        assert_eq!(branch_type_name(BranchType::Stair), "Stair");
    }

    #[test]
    fn test_on_same_level() {
        let a = LevelID::new(DungeonBranch::Main, 5);
        let b = LevelID::new(DungeonBranch::Main, 5);
        let c = LevelID::new(DungeonBranch::Main, 6);
        assert!(on_same_level(a, b));
        assert!(!on_same_level(a, c));
    }

    #[test]
    fn test_get_level_from_depth() {
        let d = Dungeon::new();
        let lev = get_level_from_depth(&d, 10);
        assert_eq!(lev.branch, DungeonBranch::Main);
        assert_eq!(lev.depth, 10);
    }

    #[test]
    fn test_castle_tune() {
        let tune = generate_castle_tune(&[0, 1, 2, 3, 4]);
        assert_eq!(tune.len(), 5);
        for c in tune.chars() {
            assert!(c >= 'A' && c <= 'G');
        }
    }

    #[test]
    fn test_special_level_map_count() {
        assert_eq!(SPECIAL_LEVEL_MAP.len(), 26);
    }

    #[test]
    fn test_dungeon_overview() {
        let d = Dungeon::new();
        let overview = d.dungeon_overview();
        assert!(!overview.is_empty());
        assert!(overview[0].contains("Dungeons of Doom"));
    }

    #[test]
    fn test_connected_branches() {
        let mut d = Dungeon::new();
        d.current_level = LevelID::new(DungeonBranch::Main, 3);
        let connected = d.connected_branches();
        assert!(connected.contains(&DungeonBranch::Mines));
    }

    #[test]
    fn test_annotation() {
        let mut d = Dungeon::new();
        let lev = LevelID::new(DungeonBranch::Main, 5);
        assert!(d.get_annotation(lev).is_none());
        d.set_annotation(lev, "오라클 근처 상점".to_string());
        assert_eq!(d.get_annotation(lev), Some("오라클 근처 상점"));
        d.set_annotation(lev, "".to_string());
        assert!(d.get_annotation(lev).is_none());
    }

    #[test]
    fn test_is_special_levels() {
        let d = Dungeon::new();
        let oracle = LevelID::new(DungeonBranch::Main, 5);
        assert!(d.is_oracle_level(oracle));
        let minetown = LevelID::new(DungeonBranch::Mines, 6);
        assert!(d.is_minetown(minetown));
    }

    #[test]
    fn test_rogue_level() {
        let d = Dungeon::new();
        assert!(d.is_rogue_level(LevelID::new(DungeonBranch::Main, 15)));
        assert!(!d.is_rogue_level(LevelID::new(DungeonBranch::Main, 10)));
    }

    #[test]
    fn test_depth_difference() {
        let d = Dungeon::new();
        let a = LevelID::new(DungeonBranch::Main, 5);
        let b = LevelID::new(DungeonBranch::Main, 15);
        assert_eq!(d.depth_difference(a, b), 10);
    }

    #[test]
    fn test_next_prev_target() {
        let d = Dungeon::new();
        let stairs = StairPositions::default();
        let next = next_level_target(&d, false, &stairs);
        assert!(next.is_some());
        assert_eq!(next.unwrap().depth, 2);
    }

    #[test]
    fn test_can_rise_up() {
        let mut d = Dungeon::new();
        // 1층에서는 올라갈 수 없음
        d.current_level = LevelID::new(DungeonBranch::Main, 1);
        assert!(!d.can_rise_up());
        // 5층에서는 올라갈 수 있음
        d.current_level = LevelID::new(DungeonBranch::Main, 5);
        assert!(d.can_rise_up());
        // 소코반에서는 올라갈 수 없음
        d.current_level = LevelID::new(DungeonBranch::Sokoban, 3);
        assert!(!d.can_rise_up());
    }

    #[test]
    fn test_invocation_level() {
        let mut d = Dungeon::new();
        // 게헨놈 최하층-1이 소환 레벨
        // 게헨놈: 25-50, 26레벨, num_levels=26, 소환=25
        d.current_level = LevelID::new(DungeonBranch::Gehennom, 25);
        assert!(d.is_invocation_level());
        d.current_level = LevelID::new(DungeonBranch::Main, 10);
        assert!(!d.is_invocation_level());
    }

    #[test]
    fn test_deepest_lev_reached() {
        let mut d = Dungeon::new();
        d.deepest_reached = 20;
        assert_eq!(d.deepest_lev_reached(false), 51); // 아스트랄 depth_start=51
        assert!(d.deepest_lev_reached(true) >= 20);
    }
}

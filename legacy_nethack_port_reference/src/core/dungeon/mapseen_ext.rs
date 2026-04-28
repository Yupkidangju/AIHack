// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-1] 던전 맵 탐험 기록 (Mapseen) 확장 모듈 (mapseen_ext.rs)
// 원본: NetHack 3.6.7 dungeon.c (mapseen 구조체 및 #overview 관련 로직)
// ============================================================================

use crate::core::dungeon::LevelID;
use serde::{Deserialize, Serialize};

// =============================================================================
// [1] 데이터 구조 (원본: mapseen, mapseen_feat, mapseen_flags, msrooms)
// =============================================================================

/// [v2.22.0 R34-1] 레벨 내 발견된 특징 개수 (원본: mapseen_feat)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapseenFeat {
    pub nfount: u8,
    pub nsink: u8,
    pub nthrone: u8,
    pub naltar: u8,
    pub ngrave: u8,
    pub ntree: u8,
    pub nshop: u8,
    pub ntemple: u8,
    // (물, 얼음, 용암 등은 원본 3.6에서도 #if 0 처리되어 활성화되지 않음)
    /// 상점 종류 (rooms[i].rtype)
    pub shoptype: i32,
    /// 제단 정렬 정보 (MSA_NONE 등)
    pub msalign: i32,
}

/// [v2.22.0 R34-1] 레벨 관련 탐험 지식 플래그 (원본: mapseen_flags)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapseenFlags {
    /// 접근 불가 상태
    pub unreachable: bool,
    /// 건망증/기억상실로 잊혀짐
    pub forgot: bool,
    /// 이 레벨에 알려진 뼈대(죽은 모험가) 존재
    pub knownbones: bool,
    /// 소코반 레벨 클리어 여부
    pub sokosolved: bool,
    /// 매우 큰 방(Big room) 레벨
    pub bigroom: bool,
    /// 로그(Rogue) 스타일 레벨
    pub roguelevel: bool,
    /// 오라클 오브 델파이 발견
    pub oracle: bool,
    /// 캐슬 비밀 문/성벽 발견
    pub castle: bool,
    /// 죽음의 계곡(Valley of the Dead) 진입
    pub valley: bool,
    /// 몰록의 지성소(Moloch's Sanctum) 진입
    pub msanctum: bool,
    /// 퀘스트 리더의 부름을 받음
    pub quest_summons: bool,
    /// 퀘스트 수락됨
    pub questing: bool,
    /// 포트 루디오스(Fort Ludios) 진입/발견
    pub ludios: bool,
    /// 캐슬 멜로디 연주 정보 기억
    pub castletune: bool,
}

/// [v2.22.0 R34-1] 방 탐색 정보
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapseenRoom {
    /// 이 방을 본 적이 있는지
    pub seen: bool,
    /// 상점/신전 등에 npc가 없는 상태인지
    pub untended: bool,
}

/// [v2.22.0 R34-1] 던전 레벨별 맵 탐색 일지 (원본: mapseen)
/// 플레이어가 각 층에 대해 아는 정보를 담는다 (#overview 명령어에서 사용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapseen {
    /// 해당 레벨의 ID
    pub level_id: LevelID,
    /// 지형/특징물 통계
    pub feat: MapseenFeat,
    /// 탐험 플래그
    pub flags: MapseenFlags,
    /// 사용자가 남긴 주석 (#annotate)
    pub custom_annotation: Option<String>,
    /// 각 방(최대 MAXNROFROOMS)에 대한 식별 정보
    pub rooms: Vec<MapseenRoom>,
    /// 분기 연결점 정보 (연결된 타 분기의 BranchID)
    pub branch_connection: Option<String>,
}

impl Mapseen {
    /// 새로운 Mapseen 기록 생성
    pub fn new(level_id: LevelID) -> Self {
        Self {
            level_id,
            feat: MapseenFeat::default(),
            flags: MapseenFlags::default(),
            custom_annotation: None,
            rooms: vec![MapseenRoom::default(); crate::core::dungeon::rect::MAXNROFROOMS], // [S6d-2] 상수 참조
            branch_connection: None,
        }
    }

    /// [v2.22.0 R34-1] 레벨이 overview 대상인지(흥미로운지) 판정 (원본: dungeon.c:2366 interest_mapseen)
    /// `is_current_level`: 플레이어가 현재 위치한 레벨이면 무조건 관심 대상
    /// `in_sokoban_branch`: 현재 속해있는 브랜치가 소코반인지
    /// `in_endgame_branch`: 현재 속해있는 브랜치가 엔드게임인지
    /// `is_furthest_reached`: 이 레벨이 해당 브랜치에서 가장 깊이 도달한 층인지
    pub fn is_interesting(
        &self,
        is_current_level: bool,
        in_sokoban_branch: bool,
        in_endgame_branch: bool,
        is_furthest_reached: bool,
    ) -> bool {
        if is_current_level {
            return true;
        }
        if self.flags.unreachable || self.flags.forgot {
            return false;
        }
        // 자동 주석이 달리는 레벨인지
        if self.flags.oracle
            || self.flags.bigroom
            || self.flags.roguelevel
            || self.flags.castle
            || self.flags.valley
            || self.flags.msanctum
            || self.flags.quest_summons
            || self.flags.questing
        {
            return true;
        }

        // 소코반에 있거나, 소코반을 아직 풀지 않은 층이라면
        // (단, 원본에서는 In_sokoban(&mptr->lev) 조건이 필요함 - 여기선 호출자가 필터링하여 넘김)
        if in_sokoban_branch && !self.flags.sokosolved {
            return true;
        }

        // 엔드게임 브랜치인 경우 모든 방문 레벨을 노출
        if in_endgame_branch {
            return true;
        }

        // 발견된 장식이 하나라도 있거나, 유저 주석이 있거나, 브랜치 연결점이 있거나, 가장 먼 레벨인 경우
        let feat_interesting = self.feat.nfount > 0
            || self.feat.nsink > 0
            || self.feat.nthrone > 0
            || self.feat.naltar > 0
            || self.feat.ngrave > 0
            || self.feat.ntree > 0
            || self.feat.nshop > 0
            || self.feat.ntemple > 0;

        feat_interesting
            || self.flags.knownbones // (bones는 현재 지원 범위 외라 단순 플래그로만 확인)
            || self.custom_annotation.is_some()
            || self.branch_connection.is_some()
            || is_furthest_reached
    }

    /// [v2.22.0 R34-1] 특징물 갱신 추가 (원본의 3개 제한 적용)
    pub fn add_feature_count(count: &mut u8) {
        if *count < 3 {
            *count += 1;
        }
    }
}

// =============================================================================
// [2] 텍스트 반환 유틸 (원본: seen_string, shop_string)
// =============================================================================

/// 모음 문자인지 판별
fn is_vowel(c: char) -> bool {
    let check = c.to_ascii_lowercase();
    check == 'a' || check == 'e' || check == 'i' || check == 'o' || check == 'u'
}

/// [v2.22.0 R34-1] 특징물 개수를 문자열로 변환 (원본: dungeon.c:2744 seen_string)
/// 0: "no", 1: "a/an", 2: "some", 3(이상): "many"
pub fn format_seen_count(count: u8, object_name: &str) -> String {
    match count {
        0 => "no".to_string(),
        1 => {
            let first_char = object_name.chars().next().unwrap_or('x');
            if is_vowel(first_char) {
                "an".to_string()
            } else {
                "a".to_string()
            }
        }
        2 => "some".to_string(),
        3..=255 => "many".to_string(),
    }
}

/// [v2.22.0 R34-1] 상점 타입별 문자열 반환 (원본: dungeon.c:2821 shop_string)
/// 원본의 SHOPBASE 매크로 기반 하드코딩된 이름 대신, 범용 ID 기반으로 치환 (임시)
pub fn shop_string(shop_type_id: i32) -> &'static str {
    match shop_type_id {
        // -1 = 관리자 없는 상점
        -1 => "untended shop",
        1 => "general store",
        2 => "armor shop",
        3 => "scroll shop",
        4 => "potion shop",
        5 => "weapon shop",
        6 => "delicatessen",
        7 => "jewelers",
        8 => "wand shop",
        9 => "bookstore",
        10 => "health food store",
        11 => "lighting shop",
        _ => "shop",
    }
}

// =============================================================================
// [3] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seen_string_formatting() {
        assert_eq!(format_seen_count(0, "altar"), "no");
        assert_eq!(format_seen_count(1, "altar"), "an");
        assert_eq!(format_seen_count(1, "fountain"), "a");
        assert_eq!(format_seen_count(2, "grave"), "some");
        assert_eq!(format_seen_count(3, "tree"), "many");
        assert_eq!(format_seen_count(10, "shop"), "many");
    }

    #[test]
    fn test_interest_mapseen() {
        use crate::core::dungeon::DungeonBranch;
        let level_id = LevelID::new(DungeonBranch::Main, 5);
        let mut m = Mapseen::new(level_id);

        // 아무 특징 없음 -> 관심 대상 아님
        assert!(!m.is_interesting(false, false, false, false));

        // 현재 레벨이면 무조건 관심 대상
        assert!(m.is_interesting(true, false, false, false));

        // 특징물이 있으면 관심 대상
        m.feat.nshop = 1;
        assert!(m.is_interesting(false, false, false, false));
        m.feat.nshop = 0;

        // 속성이 있으면 관심 대상
        m.flags.bigroom = true;
        assert!(m.is_interesting(false, false, false, false));
    }

    #[test]
    fn test_feature_count_cap() {
        let mut f = MapseenFeat::default();
        Mapseen::add_feature_count(&mut f.nfount);
        assert_eq!(f.nfount, 1);
        Mapseen::add_feature_count(&mut f.nfount);
        assert_eq!(f.nfount, 2);
        Mapseen::add_feature_count(&mut f.nfount);
        assert_eq!(f.nfount, 3);
        Mapseen::add_feature_count(&mut f.nfount);
        assert_eq!(f.nfount, 3); // 3 이상 안 올라감
    }
}

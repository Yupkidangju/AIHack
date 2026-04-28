// ============================================================================
// AIHack - light_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] light.c 핵심 함수 완전 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 light.c (815줄)
//
// 이식 대상:
//   LightSource 구조체, new/del/move/split/merge/adjust,
//   candle_light_range(), arti_light_radius(), arti_light_description(),
//   obj_sheds_light(), obj_is_burning(), snuff_light_source(),
//   do_light_sources() 핵심 알고리즘
// ============================================================================

// =============================================================================
// 상수 정의
// =============================================================================

/// 최대 광원 반경
pub const MAX_RADIUS: i32 = 15;

/// 광원 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightSourceType {
    /// 오브젝트에 부착된 광원
    Object,
    /// 몬스터에 부착된 광원
    Monster,
}

/// 광원 플래그
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightFlags {
    /// 표시 여부
    pub show: bool,
    /// 복구 시 ID 재연결 필요
    pub needs_fixup: bool,
}

impl Default for LightFlags {
    fn default() -> Self {
        Self {
            show: false,
            needs_fixup: false,
        }
    }
}

// =============================================================================
// LightSource — 이동식 광원
// [v2.10.1] light.c 전체 구조 이식
// =============================================================================

/// 이동식 광원 (원본: light_source 구조체)
#[derive(Debug, Clone)]
pub struct LightSource {
    pub x: i32,
    pub y: i32,
    pub range: i32,
    pub source_type: LightSourceType,
    pub source_id: u64,
    pub flags: LightFlags,
}

impl LightSource {
    /// 새 광원 생성 (원본: new_light_source L54-80)
    pub fn new(
        x: i32,
        y: i32,
        range: i32,
        source_type: LightSourceType,
        source_id: u64,
    ) -> Option<Self> {
        if range < 1 || range > MAX_RADIUS {
            return None; // 불법 범위
        }
        Some(Self {
            x,
            y,
            range,
            source_type,
            source_id,
            flags: LightFlags::default(),
        })
    }
}

/// 광원 관리자 (원본: light_base 연결 리스트 → Vec)
#[derive(Debug, Clone, Default)]
pub struct LightManager {
    pub sources: Vec<LightSource>,
    pub vision_recalc_needed: bool,
}

impl LightManager {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            vision_recalc_needed: false,
        }
    }

    /// 광원 추가 (원본: new_light_source L54-80)
    pub fn add(
        &mut self,
        x: i32,
        y: i32,
        range: i32,
        source_type: LightSourceType,
        source_id: u64,
    ) -> bool {
        if let Some(ls) = LightSource::new(x, y, range, source_type, source_id) {
            self.sources.push(ls);
            self.vision_recalc_needed = true;
            true
        } else {
            false
        }
    }

    /// 광원 제거 (원본: del_light_source L82-127)
    pub fn remove(&mut self, source_type: LightSourceType, source_id: u64) -> bool {
        let before = self.sources.len();
        self.sources
            .retain(|ls| !(ls.source_type == source_type && ls.source_id == source_id));
        if self.sources.len() < before {
            self.vision_recalc_needed = true;
            true
        } else {
            false
        }
    }

    /// 광원 ID 교체 (원본: obj_move_light_source L545-557)
    pub fn move_source(&mut self, source_type: LightSourceType, old_id: u64, new_id: u64) {
        for ls in &mut self.sources {
            if ls.source_type == source_type && ls.source_id == old_id {
                ls.source_id = new_id;
            }
        }
    }

    /// 광원 분할 (원본: obj_split_light_source L623-650)
    /// 양초 그룹 분할 시 사용
    pub fn split_source(
        &mut self,
        source_type: LightSourceType,
        src_id: u64,
        dest_id: u64,
        src_new_range: Option<i32>,
        dest_range: Option<i32>,
    ) {
        let mut new_sources = Vec::new();
        for ls in &mut self.sources {
            if ls.source_type == source_type && ls.source_id == src_id {
                let mut new_ls = ls.clone();
                new_ls.source_id = dest_id;
                if let Some(dr) = dest_range {
                    new_ls.range = dr;
                }
                if let Some(sr) = src_new_range {
                    ls.range = sr;
                    self.vision_recalc_needed = true;
                }
                new_sources.push(new_ls);
            }
        }
        self.sources.extend(new_sources);
    }

    /// 광원 병합 (원본: obj_merge_light_sources L652-670)
    pub fn merge_sources(&mut self, src_id: u64, dest_id: u64, new_range: i32) {
        // src 제거
        if src_id != dest_id {
            self.sources.retain(|ls| {
                !(ls.source_type == LightSourceType::Object && ls.source_id == src_id)
            });
        }
        // dest 범위 업데이트
        for ls in &mut self.sources {
            if ls.source_type == LightSourceType::Object && ls.source_id == dest_id {
                if ls.range != new_range {
                    ls.range = new_range;
                    self.vision_recalc_needed = true;
                }
                break;
            }
        }
    }

    /// 광원 밝기 조정 (원본: obj_adjust_light_radius L672-688)
    pub fn adjust_radius(&mut self, source_type: LightSourceType, source_id: u64, new_radius: i32) {
        for ls in &mut self.sources {
            if ls.source_type == source_type && ls.source_id == source_id {
                if ls.range != new_radius {
                    ls.range = new_radius;
                    self.vision_recalc_needed = true;
                }
                return;
            }
        }
    }

    /// 특정 위치 광원 소화 (원본: snuff_light_source L566-602)
    pub fn snuff_at(
        &mut self,
        x: i32,
        y: i32,
        is_artifact_fn: &dyn Fn(u64) -> bool,
    ) -> Option<u64> {
        let idx = self.sources.iter().position(|ls| {
            ls.source_type == LightSourceType::Object
                && ls.x == x
                && ls.y == y
                && !is_artifact_fn(ls.source_id)
        });
        if let Some(i) = idx {
            let id = self.sources[i].source_id;
            self.sources.remove(i);
            self.vision_recalc_needed = true;
            Some(id)
        } else {
            None
        }
    }

    /// 광원 존재 여부 (원본: any_light_source L559-564)
    pub fn any_exist(&self) -> bool {
        !self.sources.is_empty()
    }

    /// 특정 위치 주변 밝기 확인 (원본: do_light_sources 핵심)
    /// 반환: 밝혀진 좌표 목록
    pub fn lit_positions(&self, grid_width: i32, grid_height: i32) -> Vec<(i32, i32)> {
        let mut lit = Vec::new();
        for ls in &self.sources {
            if !ls.flags.show && ls.range > 0 {
                // 원형 범위 내 모든 위치
                let r = ls.range;
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dy * dy <= r * r {
                            let nx = ls.x + dx;
                            let ny = ls.y + dy;
                            if nx >= 0 && nx < grid_width && ny >= 0 && ny < grid_height {
                                lit.push((nx, ny));
                            }
                        }
                    }
                }
            }
        }
        lit.sort();
        lit.dedup();
        lit
    }

    /// 통계 (원본: light_stats L383-398)
    pub fn stats(&self) -> (usize, usize) {
        let count = self.sources.len();
        let size = count * std::mem::size_of::<LightSource>();
        (count, size)
    }
}

// =============================================================================
// candle_light_range — 양초 광원 범위
// [v2.10.1] light.c:690-729 이식
// =============================================================================

/// 양초 광원 범위 계산 (원본 candle_light_range)
/// [v2.10.1] light.c:690-729
pub fn candle_light_range(quantity: i64, is_candelabrum: bool) -> i32 {
    if is_candelabrum {
        // 촛대별 특수 범위 (원본:698-706)
        if quantity < 4 {
            2
        } else if quantity < 7 {
            3
        } else {
            4
        }
    } else {
        // 일반 양초: 7의 거듭제곱 단위 (원본:707-722)
        let mut n = quantity;
        let mut radius = 1;
        loop {
            radius += 1;
            n /= 7;
            if n <= 0 {
                break;
            }
        }
        radius
    }
}

// =============================================================================
// arti_light_radius — 아티팩트 광원 범위
// [v2.10.1] light.c:731-751 이식
// =============================================================================

/// 아티팩트 광원 범위 (원본 arti_light_radius)
/// [v2.10.1] light.c:731-751
/// 축복: 3, 일반: 2, 저주: 1
pub fn arti_light_radius(is_lit: bool, is_artifact: bool, blessed: bool, cursed: bool) -> i32 {
    if !is_lit || !is_artifact {
        return 0;
    }
    if blessed {
        3
    } else if !cursed {
        2
    } else {
        1
    }
}

// =============================================================================
// arti_light_description — 아티팩트 광원 설명
// [v2.10.1] light.c:753-769 이식
// =============================================================================

/// 아티팩트 광원 설명 (원본 arti_light_description)
pub fn arti_light_description(radius: i32) -> &'static str {
    match radius {
        3 => "brilliantly",
        2 => "brightly",
        1 => "dimly",
        _ => "strangely",
    }
}

// =============================================================================
// obj_sheds_light / obj_is_burning
// [v2.10.1] light.c:604-621 이식
// =============================================================================

/// 오브젝트가 빛을 내는지 (원본: obj_sheds_light)
pub fn obj_sheds_light(is_lit: bool) -> bool {
    is_lit
}

/// 오브젝트가 타는 중인지 (원본: obj_is_burning)
pub fn obj_is_burning(
    is_lit: bool,
    is_magic_lamp: bool,
    is_ignitable: bool,
    is_artifact_light: bool,
) -> bool {
    is_lit && (is_magic_lamp || is_ignitable || is_artifact_light)
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_source_new() {
        assert!(LightSource::new(5, 5, 3, LightSourceType::Object, 1).is_some());
        assert!(LightSource::new(5, 5, 0, LightSourceType::Object, 1).is_none());
        assert!(LightSource::new(5, 5, 16, LightSourceType::Object, 1).is_none());
    }

    #[test]
    fn test_light_manager_add_remove() {
        let mut mgr = LightManager::new();
        assert!(mgr.add(5, 5, 3, LightSourceType::Object, 100));
        assert!(mgr.add(10, 10, 2, LightSourceType::Monster, 200));
        assert_eq!(mgr.sources.len(), 2);

        assert!(mgr.remove(LightSourceType::Object, 100));
        assert_eq!(mgr.sources.len(), 1);
        assert!(!mgr.remove(LightSourceType::Object, 999)); // 없는 항목
    }

    #[test]
    fn test_light_manager_move() {
        let mut mgr = LightManager::new();
        mgr.add(5, 5, 3, LightSourceType::Object, 100);
        mgr.move_source(LightSourceType::Object, 100, 200);
        assert_eq!(mgr.sources[0].source_id, 200);
    }

    #[test]
    fn test_light_manager_split() {
        let mut mgr = LightManager::new();
        mgr.add(5, 5, 3, LightSourceType::Object, 100);
        mgr.split_source(LightSourceType::Object, 100, 200, Some(2), Some(2));
        assert_eq!(mgr.sources.len(), 2);
    }

    #[test]
    fn test_light_manager_merge() {
        let mut mgr = LightManager::new();
        mgr.add(5, 5, 2, LightSourceType::Object, 100);
        mgr.add(5, 5, 2, LightSourceType::Object, 200);
        mgr.merge_sources(100, 200, 3);
        assert_eq!(mgr.sources.len(), 1);
        assert_eq!(mgr.sources[0].range, 3);
    }

    #[test]
    fn test_light_manager_adjust() {
        let mut mgr = LightManager::new();
        mgr.add(5, 5, 2, LightSourceType::Object, 100);
        mgr.adjust_radius(LightSourceType::Object, 100, 4);
        assert_eq!(mgr.sources[0].range, 4);
        assert!(mgr.vision_recalc_needed);
    }

    #[test]
    fn test_light_manager_snuff() {
        let mut mgr = LightManager::new();
        mgr.add(5, 5, 3, LightSourceType::Object, 100);
        let result = mgr.snuff_at(5, 5, &|_| false);
        assert_eq!(result, Some(100));
        assert!(mgr.sources.is_empty());
    }

    #[test]
    fn test_light_manager_snuff_artifact() {
        let mut mgr = LightManager::new();
        mgr.add(5, 5, 3, LightSourceType::Object, 100);
        // 아티팩트는 소화 안 됨
        let result = mgr.snuff_at(5, 5, &|_| true);
        assert_eq!(result, None);
        assert_eq!(mgr.sources.len(), 1);
    }

    #[test]
    fn test_light_manager_stats() {
        let mut mgr = LightManager::new();
        mgr.add(5, 5, 3, LightSourceType::Object, 100);
        mgr.add(10, 10, 2, LightSourceType::Monster, 200);
        let (count, _size) = mgr.stats();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_lit_positions() {
        let mut mgr = LightManager::new();
        mgr.add(5, 5, 2, LightSourceType::Object, 100);
        let positions = mgr.lit_positions(20, 20);
        // 반경 2 원 → 중심 포함 약 13개 (4+8+1)
        assert!(positions.len() >= 9);
        assert!(positions.contains(&(5, 5)));
        assert!(positions.contains(&(5, 3))); // 위 2칸
    }

    #[test]
    fn test_candle_light_range() {
        // 촛대
        assert_eq!(candle_light_range(3, true), 2);
        assert_eq!(candle_light_range(5, true), 3);
        assert_eq!(candle_light_range(7, true), 4);

        // 일반 양초
        assert_eq!(candle_light_range(1, false), 2);
        assert_eq!(candle_light_range(6, false), 2);
        assert_eq!(candle_light_range(7, false), 3);
        assert_eq!(candle_light_range(48, false), 3);
        assert_eq!(candle_light_range(49, false), 4);
    }

    #[test]
    fn test_arti_light_radius() {
        assert_eq!(arti_light_radius(true, true, true, false), 3);
        assert_eq!(arti_light_radius(true, true, false, false), 2);
        assert_eq!(arti_light_radius(true, true, false, true), 1);
        assert_eq!(arti_light_radius(false, true, true, false), 0);
        assert_eq!(arti_light_radius(true, false, true, false), 0);
    }

    #[test]
    fn test_arti_light_description() {
        assert_eq!(arti_light_description(3), "brilliantly");
        assert_eq!(arti_light_description(2), "brightly");
        assert_eq!(arti_light_description(1), "dimly");
        assert_eq!(arti_light_description(0), "strangely");
    }

    #[test]
    fn test_obj_is_burning() {
        assert!(obj_is_burning(true, true, false, false));
        assert!(obj_is_burning(true, false, true, false));
        assert!(obj_is_burning(true, false, false, true));
        assert!(!obj_is_burning(false, true, true, true));
        assert!(!obj_is_burning(true, false, false, false));
    }
}

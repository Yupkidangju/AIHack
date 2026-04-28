// ============================================================================
// [v2.27.0 R15-3] 아이템 초기화 (o_init_ext.rs)
// 원본: NetHack 3.6.7 o_init.c (880줄)
// 외관(description) 셔플, 아이템 식별 상태, 인식(discovery) 관리
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 외관 셔플 (원본: o_init.c shuffle_all)
// =============================================================================

/// [v2.27.0 R15-3] 외관 셔플 결과 (인덱스→외관 이름 매핑)
pub fn shuffle_descriptions(descriptions: &[&str], rng: &mut NetHackRng) -> Vec<String> {
    let mut shuffled: Vec<String> = descriptions.iter().map(|s| s.to_string()).collect();
    // Fisher-Yates 셔플
    let len = shuffled.len();
    for i in (1..len).rev() {
        let j = rng.rn2((i + 1) as i32) as usize;
        shuffled.swap(i, j);
    }
    shuffled
}

// =============================================================================
// [2] 식별 상태 관리 (원본: o_init.c objects[], discovery)
// =============================================================================

/// [v2.27.0 R15-3] 아이템 식별 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKnowledge {
    /// 완전 미식별
    Unknown,
    /// 이름 부여됨 (call)
    Named,
    /// 효과 경험함 (사용 후)
    Tried,
    /// 완전 식별
    Known,
}

/// [v2.27.0 R15-3] 아이템 클래스 식별 테이블
#[derive(Debug, Clone, Default)]
pub struct DiscoveryTable {
    /// 아이템 ID → 식별 상태
    pub knowledge: std::collections::HashMap<i32, ItemKnowledge>,
    /// 아이템 ID → 사용자 부여 이름 (call)
    pub user_names: std::collections::HashMap<i32, String>,
}

impl DiscoveryTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// 식별 상태 조회
    pub fn get_knowledge(&self, item_id: i32) -> ItemKnowledge {
        self.knowledge
            .get(&item_id)
            .copied()
            .unwrap_or(ItemKnowledge::Unknown)
    }

    /// 식별 (원본: makeknown)
    pub fn make_known(&mut self, item_id: i32) {
        self.knowledge.insert(item_id, ItemKnowledge::Known);
    }

    /// 사용 경험 (원본: setnotworn → already_tried)
    pub fn mark_tried(&mut self, item_id: i32) {
        if self.get_knowledge(item_id) == ItemKnowledge::Unknown {
            self.knowledge.insert(item_id, ItemKnowledge::Tried);
        }
    }

    /// 사용자 이름 부여 (원본: docall)
    pub fn set_user_name(&mut self, item_id: i32, name: &str) {
        self.user_names.insert(item_id, name.to_string());
        if self.get_knowledge(item_id) == ItemKnowledge::Unknown {
            self.knowledge.insert(item_id, ItemKnowledge::Named);
        }
    }

    /// 식별 카운트
    pub fn known_count(&self) -> usize {
        self.knowledge
            .values()
            .filter(|k| **k == ItemKnowledge::Known)
            .count()
    }
}

// =============================================================================
// [3] 초기 아이템 가격 (원본: o_init.c init_objects_price)
// =============================================================================

/// [v2.27.0 R15-3] 아이템 가격 테이블 미식별 시 가치 추정 (원본: price identification)
pub fn price_id_guess(observed_price: i32) -> Vec<&'static str> {
    match observed_price {
        0..=49 => vec!["worthless glass", "uncursed scroll"],
        50..=99 => vec!["potion of booze", "scroll of light"],
        100..=199 => vec!["potion of healing", "scroll of identify"],
        200..=299 => vec!["potion of extra healing", "scroll of enchant weapon"],
        300..=499 => vec!["potion of full healing", "scroll of teleportation"],
        _ => vec!["rare item"],
    }
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuffle() {
        let mut rng = NetHackRng::new(42);
        let descs = vec!["ruby", "pink", "cyan", "golden"];
        let shuffled = shuffle_descriptions(&descs, &mut rng);
        assert_eq!(shuffled.len(), 4);
        // 원본과 같은 요소 보유
        for d in &descs {
            assert!(shuffled.contains(&d.to_string()));
        }
    }

    #[test]
    fn test_shuffle_deterministic() {
        let descs = vec!["a", "b", "c", "d"];
        let s1 = shuffle_descriptions(&descs, &mut NetHackRng::new(42));
        let s2 = shuffle_descriptions(&descs, &mut NetHackRng::new(42));
        assert_eq!(s1, s2); // 같은 시드 → 같은 결과
    }

    #[test]
    fn test_discovery_unknown() {
        let table = DiscoveryTable::new();
        assert_eq!(table.get_knowledge(1), ItemKnowledge::Unknown);
    }

    #[test]
    fn test_discovery_flow() {
        let mut table = DiscoveryTable::new();
        table.mark_tried(1);
        assert_eq!(table.get_knowledge(1), ItemKnowledge::Tried);
        table.make_known(1);
        assert_eq!(table.get_knowledge(1), ItemKnowledge::Known);
    }

    #[test]
    fn test_user_name() {
        let mut table = DiscoveryTable::new();
        table.set_user_name(5, "teleport?");
        assert_eq!(table.get_knowledge(5), ItemKnowledge::Named);
        assert_eq!(table.user_names.get(&5), Some(&"teleport?".to_string()));
    }

    #[test]
    fn test_known_count() {
        let mut table = DiscoveryTable::new();
        table.make_known(1);
        table.make_known(2);
        table.mark_tried(3);
        assert_eq!(table.known_count(), 2);
    }

    #[test]
    fn test_price_id() {
        let guesses = price_id_guess(100);
        assert!(!guesses.is_empty());
    }
}

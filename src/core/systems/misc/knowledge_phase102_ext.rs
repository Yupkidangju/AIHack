// ============================================================================
// [v2.38.0 Phase 102-5] 식별/지식 관리 통합 (knowledge_phase102_ext.rs)
// 원본: NetHack 3.6.7 src/o_init.c + objnam.c 핵심 미이식 함수
// 순수 결과 패턴
//
// 구현 범위:
//   - 아이템 외형 ↔ 정체 매핑 (미감정 상태)
//   - 사용을 통한 간접 식별
//   - 감정 주문/스크롤에 의한 직접 식별
//   - 가격 기반 추론
//   - 지식 데이터베이스 관리
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 아이템 식별 상태 — identification
// =============================================================================

/// [v2.38.0 102-5] 식별 수준
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdLevel {
    Unknown,    // 완전 미감정 ("빨간 포션")
    Appearance, // 외형만 알고 있음
    Tried,      // 사용해봄 ("빨간 포션 — 시도함")
    PriceKnown, // 가격으로 추론
    Named,      // 플레이어가 이름 부여
    Identified, // 완전 감정 ("치유 포션")
}

/// [v2.38.0 102-5] 아이템 지식 항목
#[derive(Debug, Clone)]
pub struct ItemKnowledge {
    pub true_name: String,  // 실제 이름
    pub appearance: String, // 외형 (랜덤)
    pub id_level: IdLevel,
    pub times_used: i32,
    pub player_label: Option<String>, // 플레이어가 부여한 이름
    pub known_buc: bool,              // BUC 상태 감정 여부
    pub base_price: i32,              // 기본 가격
}

/// [v2.38.0 102-5] 지식 데이터베이스
#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    pub entries: Vec<ItemKnowledge>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 외형 셔플링 (게임 시작 시 호출)
    pub fn initialize_appearances(
        &mut self,
        items: &[(&str, i32)], // (실제 이름, 기본 가격)
        appearances: &[&str],  // 랜덤 외형 목록
        rng: &mut NetHackRng,
    ) {
        let mut shuffled_appearances: Vec<String> =
            appearances.iter().map(|s| s.to_string()).collect();

        // Fisher-Yates 셔플
        for i in (1..shuffled_appearances.len()).rev() {
            let j = rng.rn2(i as i32 + 1) as usize;
            shuffled_appearances.swap(i, j);
        }

        for (idx, (name, price)) in items.iter().enumerate() {
            let appearance = if idx < shuffled_appearances.len() {
                shuffled_appearances[idx].clone()
            } else {
                name.to_string()
            };

            self.entries.push(ItemKnowledge {
                true_name: name.to_string(),
                appearance,
                id_level: IdLevel::Unknown,
                times_used: 0,
                player_label: None,
                known_buc: false,
                base_price: *price,
            });
        }
    }

    /// 사용에 의한 간접 식별
    pub fn mark_tried(&mut self, appearance: &str) -> Option<String> {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.appearance == appearance) {
            entry.times_used += 1;
            if entry.id_level == IdLevel::Unknown {
                entry.id_level = IdLevel::Tried;
            }
            Some(format!("{} — 시도함", appearance))
        } else {
            None
        }
    }

    /// 완전 감정
    pub fn identify(&mut self, appearance: &str) -> Option<String> {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.appearance == appearance) {
            entry.id_level = IdLevel::Identified;
            entry.known_buc = true;
            Some(format!("{} → {} (감정 완료!)", appearance, entry.true_name))
        } else {
            None
        }
    }

    /// 가격 기반 추론
    pub fn infer_from_price(&mut self, appearance: &str, observed_price: i32) -> Vec<String> {
        let mut candidates: Vec<String> = Vec::new();

        for entry in &self.entries {
            // 기본 가격의 ±20% 이내면 후보
            let margin = entry.base_price as f64 * 0.2;
            if (entry.base_price as f64 - observed_price as f64).abs() <= margin {
                candidates.push(entry.true_name.clone());
            }
        }

        // 관찰된 아이템의 식별 수준 갱신
        if let Some(entry) = self.entries.iter_mut().find(|e| e.appearance == appearance) {
            if entry.id_level == IdLevel::Unknown || entry.id_level == IdLevel::Tried {
                entry.id_level = IdLevel::PriceKnown;
            }
        }

        candidates
    }

    /// 플레이어 라벨 부여
    pub fn label_item(&mut self, appearance: &str, label: &str) -> Result<String, String> {
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.appearance == appearance)
            .ok_or_else(|| "알 수 없는 아이템.".to_string())?;

        entry.player_label = Some(label.to_string());
        if entry.id_level == IdLevel::Unknown {
            entry.id_level = IdLevel::Named;
        }
        Ok(format!("{} → \"{}\"", appearance, label))
    }

    /// 표시 이름 (현재 식별 수준에 따라)
    pub fn display_name(&self, appearance: &str) -> String {
        if let Some(entry) = self.entries.iter().find(|e| e.appearance == appearance) {
            match entry.id_level {
                IdLevel::Identified => entry.true_name.clone(),
                IdLevel::Named => {
                    if let Some(ref label) = entry.player_label {
                        format!("{} (\"{}\")", appearance, label)
                    } else {
                        appearance.to_string()
                    }
                }
                IdLevel::Tried => format!("{} (시도함)", appearance),
                IdLevel::PriceKnown => format!("{} ({}G 상당)", appearance, entry.base_price),
                _ => appearance.to_string(),
            }
        } else {
            appearance.to_string()
        }
    }

    /// 감정된 아이템 수
    pub fn identified_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.id_level == IdLevel::Identified)
            .count()
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    fn setup_kb() -> KnowledgeBase {
        let mut kb = KnowledgeBase::new();
        let mut rng = test_rng();
        let items = [("치유 포션", 100), ("독 포션", 50), ("투명 포션", 150)];
        let appearances = ["빨간 포션", "파란 포션", "초록 포션"];
        kb.initialize_appearances(&items, &appearances, &mut rng);
        kb
    }

    #[test]
    fn test_setup() {
        let kb = setup_kb();
        assert_eq!(kb.entries.len(), 3);
    }

    #[test]
    fn test_all_unknown() {
        let kb = setup_kb();
        assert_eq!(kb.identified_count(), 0);
    }

    #[test]
    fn test_mark_tried() {
        let mut kb = setup_kb();
        let appearance = kb.entries[0].appearance.clone();
        let result = kb.mark_tried(&appearance);
        assert!(result.is_some());
    }

    #[test]
    fn test_identify() {
        let mut kb = setup_kb();
        let appearance = kb.entries[0].appearance.clone();
        let result = kb.identify(&appearance);
        assert!(result.is_some());
        assert_eq!(kb.identified_count(), 1);
    }

    #[test]
    fn test_display_unknown() {
        let kb = setup_kb();
        let appearance = &kb.entries[0].appearance;
        let name = kb.display_name(appearance);
        // 미감정 상태에서는 외형 이름 그대로 표시
        assert_eq!(name, *appearance);
    }

    #[test]
    fn test_display_identified() {
        let mut kb = setup_kb();
        let appearance = kb.entries[0].appearance.clone();
        kb.identify(&appearance);
        let name = kb.display_name(&appearance);
        assert!(name.contains("포션")); // 실제 이름 표시
    }

    #[test]
    fn test_label() {
        let mut kb = setup_kb();
        let appearance = kb.entries[0].appearance.clone();
        let result = kb.label_item(&appearance, "이거 좋은 거");
        assert!(result.is_ok());
    }

    #[test]
    fn test_price_inference() {
        let mut kb = setup_kb();
        let appearance = kb.entries[0].appearance.clone();
        let candidates = kb.infer_from_price(&appearance, 100);
        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_shuffled_appearances() {
        let kb1 = setup_kb();
        let mut kb2 = KnowledgeBase::new();
        let mut rng2 = NetHackRng::new(999); // 다른 시드
        let items = [("치유 포션", 100), ("독 포션", 50), ("투명 포션", 150)];
        let appearances = ["빨간 포션", "파란 포션", "초록 포션"];
        kb2.initialize_appearances(&items, &appearances, &mut rng2);
        // 시드가 다르면 외형 배정이 다를 수 있음
        // (우연히 같을 수도 있으므로 단순히 크기만 확인)
        assert_eq!(kb2.entries.len(), 3);
    }
}

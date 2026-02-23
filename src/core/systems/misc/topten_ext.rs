// ============================================================================
// [v2.26.0 R14-4] 하이스코어 (topten_ext.rs)
// 원본: NetHack 3.6.7 topten.c (855줄)
// 스코어 파일 관리, 랭킹 삽입, 포매팅
// ============================================================================

// =============================================================================
// [1] 스코어 엔트리 (원본: topten.c toptenentry)
// =============================================================================

/// [v2.26.0 R14-4] 스코어 엔트리
#[derive(Debug, Clone)]
pub struct TopTenEntry {
    pub rank: u32,
    pub score: i64,
    pub name: String,
    pub role: String,
    pub race: String,
    pub death_cause: String,
    pub max_depth: i32,
    pub max_level: i32,
    pub turns: u64,
    pub timestamp: u64,
}

/// [v2.26.0 R14-4] 스코어 테이블
#[derive(Debug, Clone, Default)]
pub struct ScoreTable {
    pub entries: Vec<TopTenEntry>,
    pub max_entries: usize,
}

impl ScoreTable {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// 새 스코어 삽입, 순위 반환 (원본: topten)
    pub fn insert(&mut self, entry: TopTenEntry) -> Option<u32> {
        let position = self
            .entries
            .iter()
            .position(|e| entry.score > e.score)
            .unwrap_or(self.entries.len());

        if position >= self.max_entries {
            return None;
        }

        self.entries.insert(position, entry);
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }

        // 순위 재계산
        for (i, e) in self.entries.iter_mut().enumerate() {
            e.rank = (i + 1) as u32;
        }

        Some(position as u32 + 1)
    }

    /// 역할별 최고 스코어
    pub fn best_for_role(&self, role: &str) -> Option<&TopTenEntry> {
        self.entries.iter().find(|e| e.role == role)
    }
}

// =============================================================================
// [2] 스코어 포매팅 (원본: topten.c outheader, outentry)
// =============================================================================

/// [v2.26.0 R14-4] 스코어 테이블 헤더
pub fn format_header() -> String {
    format!(
        "{:<4} {:<12} {:>8}  {:<12} {:<8} {}",
        "Rank", "Name", "Score", "Role", "Depth", "Death"
    )
}

/// [v2.26.0 R14-4] 스코어 엔트리 포매팅
pub fn format_entry(entry: &TopTenEntry) -> String {
    format!(
        "{:<4} {:<12} {:>8}  {:<12} Dl:{:<4} {}",
        entry.rank, entry.name, entry.score, entry.role, entry.max_depth, entry.death_cause
    )
}

// =============================================================================
// [3] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(name: &str, score: i64) -> TopTenEntry {
        TopTenEntry {
            rank: 0,
            score,
            name: name.to_string(),
            role: "Valkyrie".to_string(),
            race: "Human".to_string(),
            death_cause: "killed by dragon".to_string(),
            max_depth: 20,
            max_level: 15,
            turns: 5000,
            timestamp: 0,
        }
    }

    #[test]
    fn test_insert_first() {
        let mut table = ScoreTable::new(10);
        let rank = table.insert(make_entry("Alice", 50000));
        assert_eq!(rank, Some(1));
        assert_eq!(table.entries.len(), 1);
    }

    #[test]
    fn test_insert_ordering() {
        let mut table = ScoreTable::new(10);
        table.insert(make_entry("Alice", 50000));
        table.insert(make_entry("Bob", 30000));
        table.insert(make_entry("Charlie", 70000));

        assert_eq!(table.entries[0].name, "Charlie");
        assert_eq!(table.entries[1].name, "Alice");
        assert_eq!(table.entries[2].name, "Bob");
    }

    #[test]
    fn test_insert_overflow() {
        let mut table = ScoreTable::new(3);
        table.insert(make_entry("A", 50000));
        table.insert(make_entry("B", 40000));
        table.insert(make_entry("C", 30000));
        let rank = table.insert(make_entry("D", 20000));
        assert!(rank.is_none());
        assert_eq!(table.entries.len(), 3);
    }

    #[test]
    fn test_best_for_role() {
        let mut table = ScoreTable::new(10);
        table.insert(make_entry("A", 50000));
        let best = table.best_for_role("Valkyrie");
        assert!(best.is_some());
        assert_eq!(best.unwrap().name, "A");
    }

    #[test]
    fn test_format_header() {
        let h = format_header();
        assert!(h.contains("Rank"));
        assert!(h.contains("Score"));
    }

    #[test]
    fn test_format_entry() {
        let entry = make_entry("Alice", 50000);
        let line = format_entry(&TopTenEntry { rank: 1, ..entry });
        assert!(line.contains("Alice"));
        assert!(line.contains("50000"));
    }
}

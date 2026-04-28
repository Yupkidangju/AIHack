// ============================================================================
// [v2.35.0 Phase 99-4] 메시지 시스템 확장 (message_phase99_ext.rs)
// 원본: NetHack 3.6.7 src/pline.c + display.c 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 메시지 시스템 — message_system (pline.c 핵심)
// =============================================================================

/// [v2.35.0 99-4] 메시지 우선순위
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Debug,
    Info,
    GameAction,
    Combat,
    Warning,
    Critical,
    Death,
}

/// [v2.35.0 99-4] 메시지 항목
#[derive(Debug, Clone)]
pub struct GameMessage {
    pub text: String,
    pub priority: MessagePriority,
    pub turn: i32,
    pub color: Option<String>,
    pub is_more_prompt: bool,
}

/// [v2.35.0 99-4] 메시지 버퍼
#[derive(Debug, Clone)]
pub struct MessageBuffer {
    pub messages: Vec<GameMessage>,
    pub max_size: usize,
    pub unread_count: i32,
}

impl MessageBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_size,
            unread_count: 0,
        }
    }

    /// 메시지 추가
    pub fn push(&mut self, text: &str, priority: MessagePriority, turn: i32) {
        let color = match priority {
            MessagePriority::Combat => Some("빨강".to_string()),
            MessagePriority::Warning => Some("노랑".to_string()),
            MessagePriority::Critical | MessagePriority::Death => Some("밝은빨강".to_string()),
            _ => None,
        };

        let is_more = priority >= MessagePriority::Warning;

        self.messages.push(GameMessage {
            text: text.to_string(),
            priority,
            turn,
            color,
            is_more_prompt: is_more,
        });

        if self.messages.len() > self.max_size {
            self.messages.remove(0);
        }

        self.unread_count += 1;
    }

    /// 최근 N개 메시지
    pub fn recent(&self, count: usize) -> Vec<&GameMessage> {
        self.messages.iter().rev().take(count).collect()
    }

    /// 우선순위 필터
    pub fn filter_by_priority(&self, min_priority: MessagePriority) -> Vec<&GameMessage> {
        self.messages
            .iter()
            .filter(|m| m.priority >= min_priority)
            .collect()
    }

    /// 턴별 필터
    pub fn messages_for_turn(&self, turn: i32) -> Vec<&GameMessage> {
        self.messages.iter().filter(|m| m.turn == turn).collect()
    }

    /// 읽음 처리
    pub fn mark_read(&mut self) {
        self.unread_count = 0;
    }
}

// =============================================================================
// [2] 환각 메시지 — hallucination_messages (pline.c 환각 분기)
// =============================================================================

/// [v2.35.0 99-4] 환각 이름 생성
pub fn hallucinated_monster_name(real_name: &str, seed: i32) -> String {
    let names = [
        "스핑크스",
        "보라색 소",
        "날아다니는 돼지",
        "유니크한 감자",
        "무지개 닭",
        "자전거 타는 고블린",
        "춤추는 양배추",
        "황금 오리너구리",
        "불꽃 두꺼비",
        "크리스탈 햄스터",
        "시간여행 토끼",
        "우주 고양이",
        "레이저 펭귄",
        "테크노 좀비",
        "사이버 나비",
        "양자 거미",
    ];
    let idx = (seed.unsigned_abs() as usize + real_name.len()) % names.len();
    names[idx].to_string()
}

/// [v2.35.0 99-4] 환각 아이템 이름
pub fn hallucinated_item_name(real_name: &str, seed: i32) -> String {
    let names = [
        "빛나는 무언가",
        "이상한 물건",
        "존재하지 않는 것",
        "노래하는 칫솔",
        "미래의 유물",
        "차원의 열쇠",
        "시간의 모래시계",
        "우주의 리모컨",
        "확률의 주사위",
        "운명의 종이컵",
        "영원의 사탕",
        "무한의 반지",
    ];
    let idx = (seed.unsigned_abs() as usize + real_name.len()) % names.len();
    names[idx].to_string()
}

// =============================================================================
// [3] 디스플레이 포맷 — display_format (display.c 핵심)
// =============================================================================

/// [v2.35.0 99-4] 상태줄 포맷
pub fn format_statusline(
    name: &str,
    race: &str,
    role: &str,
    level: i32,
    hp: i32,
    max_hp: i32,
    mp: i32,
    max_mp: i32,
    gold: i64,
    dlvl: i32,
    turn: i32,
    ac: i32,
    str_val: i32,
    dex: i32,
    con: i32,
    int_val: i32,
    wis: i32,
    cha: i32,
) -> (String, String) {
    let line1 = format!(
        "{} the {} {} | St:{} Dx:{} Co:{} In:{} Wi:{} Ch:{}",
        name, race, role, str_val, dex, con, int_val, wis, cha,
    );
    let line2 = format!(
        "Dlvl:{} $:{} HP:{}/{} Pw:{}/{} AC:{} Lv:{} T:{}",
        dlvl, gold, hp, max_hp, mp, max_mp, ac, level, turn,
    );
    (line1, line2)
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_buffer() {
        let mut buf = MessageBuffer::new(100);
        buf.push("테스트 메시지", MessagePriority::Info, 1);
        assert_eq!(buf.messages.len(), 1);
        assert_eq!(buf.unread_count, 1);
    }

    #[test]
    fn test_buffer_overflow() {
        let mut buf = MessageBuffer::new(3);
        buf.push("A", MessagePriority::Info, 1);
        buf.push("B", MessagePriority::Info, 2);
        buf.push("C", MessagePriority::Info, 3);
        buf.push("D", MessagePriority::Info, 4);
        assert_eq!(buf.messages.len(), 3);
        assert_eq!(buf.messages[0].text, "B");
    }

    #[test]
    fn test_recent() {
        let mut buf = MessageBuffer::new(100);
        buf.push("A", MessagePriority::Info, 1);
        buf.push("B", MessagePriority::Combat, 2);
        let recent = buf.recent(1);
        assert_eq!(recent[0].text, "B");
    }

    #[test]
    fn test_priority_filter() {
        let mut buf = MessageBuffer::new(100);
        buf.push("일반", MessagePriority::Info, 1);
        buf.push("경고", MessagePriority::Warning, 1);
        buf.push("치명", MessagePriority::Critical, 1);
        let warnings = buf.filter_by_priority(MessagePriority::Warning);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn test_combat_color() {
        let mut buf = MessageBuffer::new(100);
        buf.push("전투!", MessagePriority::Combat, 1);
        assert_eq!(buf.messages[0].color, Some("빨강".to_string()));
    }

    #[test]
    fn test_hallucinated_name() {
        let name = hallucinated_monster_name("드래곤", 42);
        assert!(!name.is_empty());
        assert_ne!(name, "드래곤");
    }

    #[test]
    fn test_statusline() {
        let (l1, l2) = format_statusline(
            "용사", "인간", "전사", 15, 80, 100, 30, 50, 5000, 10, 20000, 5, 18, 12, 14, 10, 10, 12,
        );
        assert!(l1.contains("용사"));
        assert!(l2.contains("HP:80/100"));
    }

    #[test]
    fn test_mark_read() {
        let mut buf = MessageBuffer::new(100);
        buf.push("A", MessagePriority::Info, 1);
        buf.push("B", MessagePriority::Info, 2);
        assert_eq!(buf.unread_count, 2);
        buf.mark_read();
        assert_eq!(buf.unread_count, 0);
    }
}

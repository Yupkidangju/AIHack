// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.24.0 R12-3] 메시지 시스템 (pline_ext.rs)
//
// 원본 참조: NetHack 3.6.7 pline.c (636줄) + hacklib.c (904줄)
//
// 구현 내용:
//   1. 메시지 큐 (중복 억제)
//   2. 영문법 헬퍼 (관사, 복수형, 소유격)
//   3. 몬스터/아이템 이름 유틸
//   4. 숫자 서수 변환
// ============================================================================

// =============================================================================
// [1] 메시지 큐 (원본: pline.c pline, Norep)
// =============================================================================

/// [v2.24.0 R12-3] 메시지 엔트리
#[derive(Debug, Clone)]
pub struct MessageEntry {
    /// 메시지 텍스트
    pub text: String,
    /// 연속 중복 횟수
    pub count: u32,
    /// 발생 턴
    pub turn: u64,
}

/// [v2.24.0 R12-3] 메시지 큐
#[derive(Debug, Clone, Default)]
pub struct MessageQueue {
    /// 메시지 목록
    pub messages: Vec<MessageEntry>,
    /// 직전 메시지 (중복 체크용)
    pub last_message: Option<String>,
    /// 최대 보관 수
    pub max_history: usize,
}

impl MessageQueue {
    /// 새 메시지 큐 생성
    pub fn new(max_history: usize) -> Self {
        Self {
            messages: Vec::new(),
            last_message: None,
            max_history,
        }
    }

    /// 메시지 추가 (중복 시 카운트 증가) (원본: pline)
    pub fn pline(&mut self, text: &str, turn: u64) {
        if text.is_empty() {
            return;
        }
        // 직전과 동일하면 카운트만 증가
        if let Some(last) = self.messages.last_mut() {
            if last.text == text && last.turn == turn {
                last.count += 1;
                return;
            }
        }
        self.messages.push(MessageEntry {
            text: text.to_string(),
            count: 1,
            turn,
        });
        self.last_message = Some(text.to_string());
        // 히스토리 초과 시 제거
        if self.messages.len() > self.max_history {
            self.messages.remove(0);
        }
    }

    /// 현재 턴 메시지 목록
    pub fn current_turn_messages(&self, turn: u64) -> Vec<&MessageEntry> {
        self.messages.iter().filter(|m| m.turn == turn).collect()
    }

    /// 포맷된 메시지 (x2, x3 등)
    pub fn format_message(entry: &MessageEntry) -> String {
        if entry.count > 1 {
            format!("{} (x{})", entry.text, entry.count)
        } else {
            entry.text.clone()
        }
    }
}

// =============================================================================
// [2] 영문법 헬퍼 (원본: hacklib.c an, s_suffix, etc.)
// =============================================================================

/// [v2.24.0 R12-3] 부정관사 (a/an) (원본: an)
pub fn an(word: &str) -> String {
    if word.is_empty() {
        return "a".to_string();
    }
    let first = word
        .chars()
        .next()
        .unwrap_or(' ')
        .to_lowercase()
        .next()
        .unwrap_or(' ');
    if "aeiou".contains(first) {
        format!("an {}", word)
    } else {
        format!("a {}", word)
    }
}

/// [v2.24.0 R12-3] 정관사 (the) (원본: the)
pub fn the(word: &str) -> String {
    format!("the {}", word)
}

/// [v2.24.0 R12-3] 복수형 (원본: makeplural)
pub fn makeplural(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }
    let lower = word.to_lowercase();

    // 불규칙 복수형
    match lower.as_str() {
        "mouse" => return "mice".to_string(),
        "foot" => return "feet".to_string(),
        "goose" => return "geese".to_string(),
        "tooth" => return "teeth".to_string(),
        "child" => return "children".to_string(),
        "ox" => return "oxen".to_string(),
        _ => {}
    }

    // 규칙 복수형
    if lower.ends_with("ife") {
        return format!("{}ives", &word[..word.len() - 3]);
    }
    if lower.ends_with("olf") || lower.ends_with("alf") {
        return format!("{}ves", &word[..word.len() - 1]);
    }
    if lower.ends_with("ss")
        || lower.ends_with("sh")
        || lower.ends_with("ch")
        || lower.ends_with('x')
    {
        return format!("{}es", word);
    }
    if lower.ends_with('y')
        && !lower.ends_with("ey")
        && !lower.ends_with("ay")
        && !lower.ends_with("oy")
    {
        return format!("{}ies", &word[..word.len() - 1]);
    }
    if lower.ends_with('s') {
        return format!("{}es", word);
    }

    format!("{}s", word)
}

/// [v2.24.0 R12-3] 소유격 (원본: s_suffix)
pub fn s_suffix(word: &str) -> String {
    if word.ends_with('s') || word.ends_with("ss") {
        format!("{}'", word)
    } else {
        format!("{}'s", word)
    }
}

// =============================================================================
// [3] 숫자 유틸 (원본: hacklib.c)
// =============================================================================

/// [v2.24.0 R12-3] 숫자→서수 (원본: ordin)
pub fn ordinal(n: i32) -> String {
    let suffix = match n % 100 {
        11..=13 => "th",
        _ => match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{}{}", n, suffix)
}

/// [v2.24.0 R12-3] 숫자→영어 (1~10) (원본: hacklib.c)
pub fn number_word(n: i32) -> &'static str {
    match n {
        1 => "one",
        2 => "two",
        3 => "three",
        4 => "four",
        5 => "five",
        6 => "six",
        7 => "seven",
        8 => "eight",
        9 => "nine",
        10 => "ten",
        _ => "many",
    }
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pline_basic() {
        let mut q = MessageQueue::new(100);
        q.pline("Hello!", 1);
        assert_eq!(q.messages.len(), 1);
        assert_eq!(q.messages[0].count, 1);
    }

    #[test]
    fn test_pline_duplicate() {
        let mut q = MessageQueue::new(100);
        q.pline("Hit!", 1);
        q.pline("Hit!", 1);
        assert_eq!(q.messages.len(), 1);
        assert_eq!(q.messages[0].count, 2);
    }

    #[test]
    fn test_pline_different_turn() {
        let mut q = MessageQueue::new(100);
        q.pline("Hit!", 1);
        q.pline("Hit!", 2);
        assert_eq!(q.messages.len(), 2);
    }

    #[test]
    fn test_pline_overflow() {
        let mut q = MessageQueue::new(3);
        q.pline("a", 1);
        q.pline("b", 2);
        q.pline("c", 3);
        q.pline("d", 4);
        assert_eq!(q.messages.len(), 3);
        assert_eq!(q.messages[0].text, "b");
    }

    #[test]
    fn test_format_message() {
        let entry = MessageEntry {
            text: "Hit!".to_string(),
            count: 3,
            turn: 1,
        };
        assert_eq!(MessageQueue::format_message(&entry), "Hit! (x3)");
    }

    #[test]
    fn test_an_vowel() {
        assert_eq!(an("apple"), "an apple");
    }

    #[test]
    fn test_an_consonant() {
        assert_eq!(an("sword"), "a sword");
    }

    #[test]
    fn test_the() {
        assert_eq!(the("dragon"), "the dragon");
    }

    #[test]
    fn test_plural_regular() {
        assert_eq!(makeplural("sword"), "swords");
    }

    #[test]
    fn test_plural_es() {
        assert_eq!(makeplural("box"), "boxes");
    }

    #[test]
    fn test_plural_ies() {
        assert_eq!(makeplural("ruby"), "rubies");
    }

    #[test]
    fn test_plural_irregular() {
        assert_eq!(makeplural("mouse"), "mice");
        assert_eq!(makeplural("foot"), "feet");
    }

    #[test]
    fn test_s_suffix_normal() {
        assert_eq!(s_suffix("dragon"), "dragon's");
    }

    #[test]
    fn test_s_suffix_s() {
        assert_eq!(s_suffix("moss"), "moss'");
    }

    #[test]
    fn test_ordinal() {
        assert_eq!(ordinal(1), "1st");
        assert_eq!(ordinal(2), "2nd");
        assert_eq!(ordinal(3), "3rd");
        assert_eq!(ordinal(11), "11th");
        assert_eq!(ordinal(21), "21st");
    }

    #[test]
    fn test_number_word() {
        assert_eq!(number_word(1), "one");
        assert_eq!(number_word(10), "ten");
        assert_eq!(number_word(99), "many");
    }
}

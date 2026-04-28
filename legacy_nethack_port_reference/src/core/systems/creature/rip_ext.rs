// ============================================================================
// AIHack - rip_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
//
// [v2.10.1] rip.c 핵심 함수 완전 이식 (순수 결과 패턴)
// 원본: NetHack 3.6.7 rip.c (170줄)
//
// 이식 대상:
//   묘비 ASCII 아트 생성, center() 텍스트 배치,
//   genl_outrip() 묘비 렌더링 핵심 로직
// ============================================================================

// =============================================================================
// 묘비 상수
// [v2.10.1] rip.c:23-70
// =============================================================================

/// 묘비 한 줄에 들어갈 최대 문자 수 (원본: STONE_LINE_LEN)
pub const STONE_LINE_LEN: usize = 16;
/// 이름 줄 인덱스 (원본: NAME_LINE)
pub const NAME_LINE: usize = 6;
/// 골드 줄 인덱스 (원본: GOLD_LINE)
pub const GOLD_LINE: usize = 7;
/// 사인 줄 시작 인덱스 (원본: DEATH_LINE)
pub const DEATH_LINE: usize = 8;
/// 연도 줄 인덱스 (원본: YEAR_LINE)
pub const YEAR_LINE: usize = 12;

// =============================================================================
// 묘비 ASCII 아트 템플릿
// [v2.10.1] rip.c:23-39 이식
// =============================================================================

/// 묘비 ASCII 아트 (원본: rip_txt[])
pub fn tombstone_template() -> Vec<String> {
    vec![
        "                       ----------".to_string(),
        "                      /          \\".to_string(),
        "                     /    REST    \\".to_string(),
        "                    /      IN      \\".to_string(),
        "                   /     PEACE      \\".to_string(),
        "                  /                  \\".to_string(),
        "                  |                  |".to_string(), // 이름
        "                  |                  |".to_string(), // 골드
        "                  |                  |".to_string(), // 사인 (1)
        "                  |                  |".to_string(), // 사인 (2)
        "                  |                  |".to_string(), // 사인 (3)
        "                  |                  |".to_string(), // 사인 (4)
        "                  |       1001       |".to_string(), // 연도
        "                 *|     *  *  *      | *".to_string(),
        "        _________)/\\_//(\\/(/\\)/\\//\\/|_)_______".to_string(),
    ]
}

/// 묘비 중앙 열 위치 (원본: STONE_LINE_CENT)
pub const STONE_LINE_CENT: usize = 28;

// =============================================================================
// center — 텍스트 중앙 배치
// [v2.10.1] rip.c:74-84 이식
// =============================================================================

/// 묘비 줄에 텍스트 중앙 배치 (원본: center())
/// 줄의 중앙 위치에 텍스트를 삽입하여 새 줄 반환
pub fn center_text(line: &str, text: &str) -> String {
    let mut chars: Vec<char> = line.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    let start = STONE_LINE_CENT.saturating_sub((text_chars.len() + 1) / 2);

    for (i, &ch) in text_chars.iter().enumerate() {
        let pos = start + i;
        if pos < chars.len() {
            chars[pos] = ch;
        }
    }

    chars.into_iter().collect()
}

// =============================================================================
// 사인 줄 분할
// [v2.10.1] rip.c:118-137 이식
// =============================================================================

/// 사인 텍스트를 STONE_LINE_LEN 기준으로 분할 (원본: genl_outrip 118-137)
/// 단어 경계에서 끊어, 최대 4줄까지 반환
pub fn split_death_text(death_desc: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut remaining = death_desc;
    let max_lines = YEAR_LINE - DEATH_LINE; // 4줄

    for _ in 0..max_lines {
        if remaining.is_empty() {
            break;
        }
        if remaining.len() <= STONE_LINE_LEN {
            lines.push(remaining.to_string());
            remaining = "";
        } else {
            // 단어 경계 찾기 (원본:123-125)
            let mut split_at = STONE_LINE_LEN;
            for i in (0..=STONE_LINE_LEN).rev() {
                if remaining.as_bytes().get(i) == Some(&b' ') {
                    split_at = i;
                    break;
                }
            }
            if split_at == STONE_LINE_LEN {
                // 공백 없으면 강제 분할
                lines.push(remaining[..STONE_LINE_LEN].to_string());
                remaining = &remaining[STONE_LINE_LEN..];
            } else {
                lines.push(remaining[..split_at].to_string());
                remaining = &remaining[split_at + 1..]; // 공백 건너뜀
            }
        }
    }

    lines
}

// =============================================================================
// generate_tombstone — 완성 묘비 생성
// [v2.10.1] rip.c:86-165 핵심 이식
// =============================================================================

/// 묘비 입력 정보
#[derive(Debug, Clone)]
pub struct TombstoneInput {
    /// 플레이어 이름
    pub name: String,
    /// 보유 골드
    pub gold: i64,
    /// 사인 설명
    pub death_description: String,
    /// 사망 연도
    pub year: i64,
}

/// 완성된 묘비 ASCII 아트 생성 (원본: genl_outrip)
pub fn generate_tombstone(input: &TombstoneInput) -> Vec<String> {
    let mut rip = tombstone_template();

    // 이름 배치 (원본:105-107)
    let name = if input.name.len() > STONE_LINE_LEN {
        &input.name[..STONE_LINE_LEN]
    } else {
        &input.name
    };
    rip[NAME_LINE] = center_text(&rip[NAME_LINE], name);

    // 골드 배치 (원본:110-112)
    let gold_text = format!("{} Au", input.gold);
    let gold = if gold_text.len() > STONE_LINE_LEN {
        &gold_text[..STONE_LINE_LEN]
    } else {
        &gold_text
    };
    rip[GOLD_LINE] = center_text(&rip[GOLD_LINE], gold);

    // 사인 배치 (원본:118-137)
    let death_lines = split_death_text(&input.death_description);
    for (i, line) in death_lines.iter().enumerate() {
        let row = DEATH_LINE + i;
        if row < YEAR_LINE {
            let text = if line.len() > STONE_LINE_LEN {
                &line[..STONE_LINE_LEN]
            } else {
                line.as_str()
            };
            rip[row] = center_text(&rip[row], text);
        }
    }

    // 연도 배치 (원본:140-142)
    let year_text = format!("{:4}", input.year);
    rip[YEAR_LINE] = center_text(&rip[YEAR_LINE], &year_text);

    rip
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tombstone_template() {
        let t = tombstone_template();
        assert_eq!(t.len(), 15);
        assert!(t[0].contains("----------"));
    }

    #[test]
    fn test_center_text() {
        let line = "                  |                  |";
        let result = center_text(line, "Hero");
        assert!(result.contains("Hero"));
        // 텍스트가 중앙에 위치해야 함
        let hero_pos = result.find("Hero").unwrap();
        assert!(hero_pos > 18 && hero_pos < 30, "pos={}", hero_pos);
    }

    #[test]
    fn test_split_death_text_short() {
        let lines = split_death_text("killed by a gnome");
        assert_eq!(lines.len(), 2); // "killed by a" + "gnome"
    }

    #[test]
    fn test_split_death_text_single() {
        let lines = split_death_text("slain");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "slain");
    }

    #[test]
    fn test_split_death_text_long() {
        let lines = split_death_text("killed by a very dangerous ancient red dragon");
        assert!(lines.len() >= 2 && lines.len() <= 4);
    }

    #[test]
    fn test_generate_tombstone() {
        let input = TombstoneInput {
            name: "Artius".to_string(),
            gold: 12345,
            death_description: "killed by a gnome lord".to_string(),
            year: 2026,
        };
        let rip = generate_tombstone(&input);
        assert_eq!(rip.len(), 15);

        // 이름 확인
        assert!(rip[NAME_LINE].contains("Artius"));
        // 골드 확인
        assert!(rip[GOLD_LINE].contains("12345 Au"));
        // 연도 확인
        assert!(rip[YEAR_LINE].contains("2026"));
    }

    #[test]
    fn test_generate_tombstone_long_name() {
        let input = TombstoneInput {
            name: "AbcdefghijklmnopqrstuvwxyzAbcdef".to_string(),
            gold: 0,
            death_description: "ascended".to_string(),
            year: 1001,
        };
        let rip = generate_tombstone(&input);
        // 이름이 16글자로 잘려야 함
        assert!(!rip[NAME_LINE].contains("qrstuvwxyz"));
    }

    #[test]
    fn test_split_no_whitespace() {
        let lines = split_death_text("aaaaaaaaaaaaaaaaabbbbbbbbbbbbbbbbb");
        // 공백 없으면 강제 16글자 분할
        assert!(lines.len() >= 2);
    }
}

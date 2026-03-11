// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-17] 핵심 유틸리티 확장 모듈 (hacklib_ext.rs)
// 원본: NetHack 3.6.7 hacklib.c (문자열 유틸, 수학, 날짜, 패턴 매칭)
// ============================================================================

// =============================================================================
// [1] 서수 접미사 (원본: hacklib.c:538-547 ordin)
// =============================================================================

/// [v2.22.0 R34-17] 숫자의 서수 접미사 반환 (원본: ordin)
/// 예: 1 → "st", 2 → "nd", 3 → "rd", 4 → "th", 11 → "th"
pub fn ordinal_suffix(n: i32) -> &'static str {
    let dd = n % 10;
    if dd == 0 || dd > 3 || (n % 100) / 10 == 1 {
        "th"
    } else if dd == 1 {
        "st"
    } else if dd == 2 {
        "nd"
    } else {
        "rd"
    }
}

/// [v2.22.0 R34-17] 숫자를 부호 포함 문자열로 변환 (원본: sitoa)
pub fn signed_number(n: i32) -> String {
    if n < 0 {
        format!("{}", n)
    } else {
        format!("+{}", n)
    }
}

// =============================================================================
// [2] 수학 유틸리티 (원본: hacklib.c:560-655)
// =============================================================================

/// [v2.22.0 R34-17] 부호 함수 (원본: sgn)
pub fn sgn(n: i32) -> i32 {
    if n < 0 {
        -1
    } else if n > 0 {
        1
    } else {
        0
    }
}

/// [v2.22.0 R34-17] 반올림 나눗셈 (원본: rounddiv)
pub fn rounddiv(x: i64, y: i32) -> i32 {
    if y == 0 {
        panic!("division by zero in rounddiv");
    }
    let mut divsgn = 1;
    let mut abs_y = y as i64;
    let mut abs_x = x;

    if abs_y < 0 {
        divsgn = -divsgn;
        abs_y = -abs_y;
    }
    if abs_x < 0 {
        divsgn = -divsgn;
        abs_x = -abs_x;
    }

    let r = abs_x / abs_y;
    let m = abs_x % abs_y;
    let result = if 2 * m >= abs_y { r + 1 } else { r };

    (divsgn as i64 * result) as i32
}

/// [v2.22.0 R34-17] 정수 제곱근 (부동소수점 미사용, 원본: isqrt)
pub fn isqrt(val: i32) -> i32 {
    let mut v = val;
    let mut rt = 0;
    let mut odd = 1;
    while v >= odd {
        v -= odd;
        odd += 2;
        rt += 1;
    }
    rt
}

/// [v2.22.0 R34-17] 두 점이 일직선(직교/대각선)인지 (원본: online2)
pub fn online2(x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
    let dx = x0 - x1;
    let dy = y0 - y1;
    dy == 0 || dx == 0 || dy == dx || dy == -dx
}

// =============================================================================
// [3] 소유격 변환 (원본: hacklib.c:304-321 s_suffix)
// =============================================================================

/// [v2.22.0 R34-17] 영어 이름을 소유격으로 변환 (원본: s_suffix)
pub fn possessive(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower == "it" {
        format!("{}s", name)
    } else if lower == "you" {
        format!("{}r", name)
    } else if name.ends_with('s') {
        format!("{}'", name)
    } else {
        format!("{}'s", name)
    }
}

// =============================================================================
// [4] 달의 위상 (원본: hacklib.c:1105-1118 phase_of_the_moon)
// =============================================================================

/// [v2.22.0 R34-17] 달의 위상 계산 (0-7, 0: 신월, 4: 보름)
/// `year`: 연도 (기준: 1900 빼기 전 값, 예: 2026)
/// `day_of_year`: 1월 1일부터 경과 일수 (0-364)
pub fn phase_of_the_moon(year: i32, day_of_year: i32) -> i32 {
    let yr = year - 1900; // tm_year와 동일하게
    let goldn = (yr % 19) + 1;
    let mut epact = (11 * goldn + 18) % 30;
    if (epact == 25 && goldn > 11) || epact == 24 {
        epact += 1;
    }
    ((((day_of_year + epact) * 6 + 11) % 177) / 22) & 7
}

/// [v2.22.0 R34-17] 밤인지 판정 (원본: night)
pub fn is_night(hour: i32) -> bool {
    hour < 6 || hour > 21
}

/// [v2.22.0 R34-17] 자정인지 판정 (원본: midnight)
pub fn is_midnight(hour: i32) -> bool {
    hour == 0
}

/// [v2.22.0 R34-17] 13일의 금요일인지 판정 (원본: friday_13th)
/// `day_of_week`: 0(일)~6(토)
/// `day_of_month`: 1~31
pub fn is_friday_13th(day_of_week: i32, day_of_month: i32) -> bool {
    day_of_week == 5 && day_of_month == 13
}

// =============================================================================
// [5] 와일드카드 패턴 매칭 (원본: hacklib.c:657-723)
// =============================================================================

/// [v2.22.0 R34-17] 와일드카드 패턴 매칭 (원본: pmatch_internal)
/// `*`: 0개 이상 문자, `?`: 단일 문자
/// `case_insensitive`: 대소문자 무시
/// `skip_chars`: 무시할 문자 집합 (퍼지 매칭용)
pub fn pattern_match(
    pattern: &str,
    string: &str,
    case_insensitive: bool,
    skip_chars: &str,
) -> bool {
    // 스킵 문자 필터링 헬퍼
    fn next_char(bytes: &[u8], pos: &mut usize, skip: &[u8]) -> Option<u8> {
        while *pos < bytes.len() {
            let c = bytes[*pos];
            *pos += 1;
            if !skip.contains(&c) {
                return Some(c);
            }
        }
        None
    }

    fn match_internal(pat: &[u8], pi: usize, str: &[u8], si: usize, ci: bool, skip: &[u8]) -> bool {
        let mut pi = pi;
        let mut si = si;

        loop {
            let p = next_char(pat, &mut pi, skip);
            let s = next_char(str, &mut si, skip);

            match p {
                None => return s.is_none(),
                Some(b'*') => {
                    // '*' = 0개 이상 매칭
                    if pi >= pat.len()
                        || match_internal(
                            pat,
                            pi,
                            str,
                            si - if s.is_some() { 1 } else { 0 },
                            ci,
                            skip,
                        )
                    {
                        return true;
                    }
                    return s.is_some() && match_internal(pat, pi - 1, str, si, ci, skip);
                }
                Some(pc) => {
                    let sc = match s {
                        Some(c) => c,
                        None => return false,
                    };
                    let matches = if ci {
                        pc.to_ascii_lowercase() == sc.to_ascii_lowercase()
                    } else {
                        pc == sc
                    };
                    if !matches && pc != b'?' {
                        return false;
                    }
                    // 계속 다음 문자 비교
                }
            }
        }
    }

    let skip_bytes = skip_chars.as_bytes();
    match_internal(
        pattern.as_bytes(),
        0,
        string.as_bytes(),
        0,
        case_insensitive,
        skip_bytes,
    )
}

/// [v2.22.0 R34-17] 대소문자 구분 와일드카드 매칭 (원본: pmatch)
pub fn pmatch(pattern: &str, string: &str) -> bool {
    pattern_match(pattern, string, false, "")
}

/// [v2.22.0 R34-17] 대소문자 무시 와일드카드 매칭 (원본: pmatchi)
pub fn pmatchi(pattern: &str, string: &str) -> bool {
    pattern_match(pattern, string, true, "")
}

/// [v2.22.0 R34-17] 퍼지 와일드카드 매칭 (공백/대시/밑줄 무시, 원본: pmatchz)
pub fn pmatchz(pattern: &str, string: &str) -> bool {
    pattern_match(pattern, string, true, " \t-_")
}

// =============================================================================
// [6] 퍼지 문자열 비교 (원본: hacklib.c:797-821 fuzzymatch)
// =============================================================================

/// [v2.22.0 R34-17] 퍼지 문자열 비교 (원본: fuzzymatch)
pub fn fuzzymatch(s1: &str, s2: &str, ignore_chars: &str, caseblind: bool) -> bool {
    let b1 = s1.as_bytes();
    let b2 = s2.as_bytes();
    let ign = ignore_chars.as_bytes();

    let mut i1 = 0usize;
    let mut i2 = 0usize;

    loop {
        // 무시 문자 건너뛰기
        while i1 < b1.len() && ign.contains(&b1[i1]) {
            i1 += 1;
        }
        while i2 < b2.len() && ign.contains(&b2[i2]) {
            i2 += 1;
        }

        let c1 = if i1 < b1.len() { b1[i1] } else { 0 };
        let c2 = if i2 < b2.len() { b2[i2] } else { 0 };

        if c1 == 0 || c2 == 0 {
            return c1 == 0 && c2 == 0;
        }

        let (cmp1, cmp2) = if caseblind {
            (c1.to_ascii_lowercase(), c2.to_ascii_lowercase())
        } else {
            (c1, c2)
        };

        if cmp1 != cmp2 {
            return false;
        }

        i1 += 1;
        i2 += 1;
    }
}

/// [v2.22.0 R34-17] XOR 텍스트 암호화/복호화 (원본: xcrypt)
pub fn xcrypt(input: &str) -> String {
    let mut result = Vec::with_capacity(input.len());
    let mut bitmask: u8 = 1;
    for &b in input.as_bytes() {
        let mut c = b;
        if c & (32 | 64) != 0 {
            c ^= bitmask;
        }
        result.push(c);
        bitmask <<= 1;
        if bitmask >= 32 {
            bitmask = 1;
        }
    }
    String::from_utf8_lossy(&result).to_string()
}

// =============================================================================
// [7] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordinal_suffix() {
        assert_eq!(ordinal_suffix(1), "st");
        assert_eq!(ordinal_suffix(2), "nd");
        assert_eq!(ordinal_suffix(3), "rd");
        assert_eq!(ordinal_suffix(4), "th");
        assert_eq!(ordinal_suffix(11), "th");
        assert_eq!(ordinal_suffix(12), "th");
        assert_eq!(ordinal_suffix(13), "th");
        assert_eq!(ordinal_suffix(21), "st");
        assert_eq!(ordinal_suffix(22), "nd");
        assert_eq!(ordinal_suffix(100), "th");
    }

    #[test]
    fn test_signed_number() {
        assert_eq!(signed_number(5), "+5");
        assert_eq!(signed_number(-3), "-3");
        assert_eq!(signed_number(0), "+0");
    }

    #[test]
    fn test_sgn() {
        assert_eq!(sgn(-10), -1);
        assert_eq!(sgn(0), 0);
        assert_eq!(sgn(42), 1);
    }

    #[test]
    fn test_rounddiv() {
        assert_eq!(rounddiv(7, 2), 4); // 3.5 → 4
        assert_eq!(rounddiv(6, 4), 2); // 1.5 → 2
        assert_eq!(rounddiv(5, 3), 2); // 1.67 → 2
        assert_eq!(rounddiv(-7, 2), -4);
    }

    #[test]
    fn test_isqrt() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(9), 3);
        assert_eq!(isqrt(10), 3);
        assert_eq!(isqrt(100), 10);
    }

    #[test]
    fn test_online2() {
        assert!(online2(0, 0, 5, 0)); // 수평
        assert!(online2(0, 0, 0, 5)); // 수직
        assert!(online2(0, 0, 3, 3)); // 대각선
        assert!(online2(0, 0, 3, -3)); // 반대 대각선
        assert!(!online2(0, 0, 3, 2)); // 일직선 아님
    }

    #[test]
    fn test_possessive() {
        assert_eq!(possessive("goblin"), "goblin's");
        assert_eq!(possessive("it"), "its");
        assert_eq!(possessive("you"), "your");
        assert_eq!(possessive("Orcus"), "Orcus'");
    }

    #[test]
    fn test_moon_phase_range() {
        let phase = phase_of_the_moon(2026, 57); // 2월 26일
        assert!(phase >= 0 && phase <= 7);
    }

    #[test]
    fn test_is_night() {
        assert!(is_night(3));
        assert!(is_night(22));
        assert!(!is_night(12));
    }

    #[test]
    fn test_is_friday_13th() {
        assert!(is_friday_13th(5, 13));
        assert!(!is_friday_13th(4, 13));
        assert!(!is_friday_13th(5, 14));
    }

    #[test]
    fn test_pmatch_exact() {
        assert!(pmatch("hello", "hello"));
        assert!(!pmatch("hello", "Hello"));
    }

    #[test]
    fn test_pmatch_wildcard() {
        assert!(pmatch("h*o", "hello"));
        assert!(pmatch("h?llo", "hello"));
        assert!(pmatch("*", "anything"));
        assert!(pmatch("test*", "testing"));
    }

    #[test]
    fn test_pmatchi() {
        assert!(pmatchi("hello", "HELLO"));
        assert!(pmatchi("H*o", "hello"));
    }

    #[test]
    fn test_pmatchz() {
        assert!(pmatchz("long sword", "long_sword"));
        assert!(pmatchz("long sword", "long-sword"));
    }

    #[test]
    fn test_fuzzymatch() {
        assert!(fuzzymatch("hello", "hello", "", false));
        assert!(fuzzymatch("hello", "HELLO", "", true));
        assert!(fuzzymatch("h e l l o", "hello", " ", false));
        assert!(!fuzzymatch("hello", "world", "", false));
    }

    #[test]
    fn test_xcrypt_roundtrip() {
        let original = "This is a test message";
        let encrypted = xcrypt(original);
        let decrypted = xcrypt(&encrypted);
        assert_eq!(decrypted, original);
    }
}

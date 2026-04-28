// ============================================================================
// [v2.35.0 R23-3] 변신 제약 (polymorph_rule_ext.rs)
// 원본: NetHack 3.6.7 polyself.c 제약 확장
// 변신 가능성, 제한, 보존 속성, 역변신
// ============================================================================

/// [v2.35.0 R23-3] 변신 허용 판정
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolyAllowed {
    Allowed,
    BlockedByAmulet,   // 불변의 아뮬렛
    BlockedByArtifact, // 퀘스트 아티팩트
    BlockedByStoning,  // 석화 중
    AlreadySameForm,
}

pub fn can_polymorph(
    has_unchanging: bool,
    is_stoning: bool,
    current_form: &str,
    target_form: &str,
) -> PolyAllowed {
    if has_unchanging {
        return PolyAllowed::BlockedByAmulet;
    }
    if is_stoning {
        return PolyAllowed::BlockedByStoning;
    }
    if current_form == target_form {
        return PolyAllowed::AlreadySameForm;
    }
    PolyAllowed::Allowed
}

/// [v2.35.0 R23-3] HP 변환 (변신 시)
pub fn poly_hp(base_hp: i32, target_level: i32) -> i32 {
    let target_hp = target_level * 8;
    target_hp.min(base_hp * 2).max(1)
}

/// [v2.35.0 R23-3] 시스템 메시지
pub fn poly_message(target: &str, is_self: bool) -> String {
    if is_self {
        format!("{}(으)로 변신했다!", target)
    } else {
        format!("{}(이)가 모습을 바꿨다!", target)
    }
}

/// [v2.35.0 R23-3] 역변신 (타이머 기반)
pub fn revert_check(turns_remaining: i32) -> bool {
    turns_remaining <= 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed() {
        assert_eq!(
            can_polymorph(false, false, "human", "dragon"),
            PolyAllowed::Allowed
        );
    }

    #[test]
    fn test_unchanging() {
        assert_eq!(
            can_polymorph(true, false, "human", "dragon"),
            PolyAllowed::BlockedByAmulet
        );
    }

    #[test]
    fn test_same_form() {
        assert_eq!(
            can_polymorph(false, false, "dragon", "dragon"),
            PolyAllowed::AlreadySameForm
        );
    }

    #[test]
    fn test_hp() {
        assert_eq!(poly_hp(100, 10), 80); // 10*8=80, min(100*2, 80)=80
        assert_eq!(poly_hp(30, 10), 60); // min(60, 80)=60
    }

    #[test]
    fn test_revert() {
        assert!(!revert_check(5));
        assert!(revert_check(0));
    }
}

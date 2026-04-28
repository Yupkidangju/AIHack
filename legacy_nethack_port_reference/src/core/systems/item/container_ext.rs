// ============================================================================
// [v2.39.0 R27-3] 컨테이너 관리 (container_ext.rs)
// 원본: NetHack 3.6.7 invent.c/pickup.c 컨테이너
// Bag of Holding, 무게 경감, 저주 파괴, 중첩 방지
// ============================================================================

/// [v2.39.0 R27-3] 컨테이너 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType {
    Sack,
    LargeBag,
    BagOfHolding,
    OilskinSack,
    IceBox,
    Chest,
    LargeBox,
}

/// [v2.39.0 R27-3] 실효 무게 계산
pub fn effective_weight(container: ContainerType, contents_weight: i32, cursed: bool) -> i32 {
    match container {
        ContainerType::BagOfHolding if cursed => contents_weight * 4, // 저주 = 4배
        ContainerType::BagOfHolding => contents_weight / 4,           // 75% 감소
        ContainerType::OilskinSack => contents_weight,                // 방수만
        _ => contents_weight,
    }
}

/// [v2.39.0 R27-3] Bag of Holding 중첩 위험 (BoH in BoH)
pub fn boh_nesting_risk(item_is_boh: bool, container_is_boh: bool) -> bool {
    item_is_boh && container_is_boh
}

/// [v2.39.0 R27-3] 아이스박스 보존
pub fn icebox_preserves(container: ContainerType) -> bool {
    matches!(container, ContainerType::IceBox)
}

/// [v2.39.0 R27-3] 컨테이너 용량
pub fn container_capacity(container: ContainerType) -> i32 {
    match container {
        ContainerType::Sack => 300,
        ContainerType::LargeBag => 600,
        ContainerType::BagOfHolding => 2000,
        ContainerType::OilskinSack => 400,
        ContainerType::IceBox => 500,
        ContainerType::Chest => 800,
        ContainerType::LargeBox => 600,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boh_weight() {
        assert_eq!(
            effective_weight(ContainerType::BagOfHolding, 400, false),
            100
        );
    }

    #[test]
    fn test_boh_cursed() {
        assert_eq!(
            effective_weight(ContainerType::BagOfHolding, 100, true),
            400
        );
    }

    #[test]
    fn test_nesting() {
        assert!(boh_nesting_risk(true, true));
        assert!(!boh_nesting_risk(false, true));
    }

    #[test]
    fn test_icebox() {
        assert!(icebox_preserves(ContainerType::IceBox));
        assert!(!icebox_preserves(ContainerType::Sack));
    }

    #[test]
    fn test_capacity() {
        assert_eq!(container_capacity(ContainerType::BagOfHolding), 2000);
    }
}

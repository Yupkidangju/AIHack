// ============================================================================
// [v2.32.0 R20-5] 시야/조명 확장 (light_source_ext.rs)
// 원본: NetHack 3.6.7 light.c + vision.c 확장
// 광원 관리, 조명 범위, 야간 시야, 적외선
// ============================================================================

/// [v2.32.0 R20-5] 광원 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightSource {
    Candle,
    OilLamp,
    BrassLantern,
    MagicLamp,
    Sunsword,
    WandOfLight,
    SpellLight,
}

pub fn light_radius(source: LightSource) -> i32 {
    match source {
        LightSource::Candle => 2,
        LightSource::OilLamp => 3,
        LightSource::BrassLantern => 4,
        LightSource::MagicLamp => 5,
        LightSource::Sunsword => 3,
        LightSource::WandOfLight => 8,
        LightSource::SpellLight => 6,
    }
}

pub fn light_duration(source: LightSource) -> Option<i32> {
    match source {
        LightSource::Candle => Some(400),
        LightSource::OilLamp => Some(1500),
        LightSource::BrassLantern => Some(3000),
        LightSource::MagicLamp | LightSource::Sunsword => None,
        LightSource::WandOfLight => Some(50),
        LightSource::SpellLight => Some(100),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionType {
    Normal,
    Infravision,
    Telepathy,
    Blind,
}

pub fn can_see_at(
    vx: i32,
    vy: i32,
    tx: i32,
    ty: i32,
    radius: i32,
    vision: VisionType,
    warm: bool,
) -> bool {
    let dist = (vx - tx).abs().max((vy - ty).abs());
    match vision {
        VisionType::Blind => false,
        VisionType::Telepathy => true,
        VisionType::Infravision => warm || dist <= radius,
        VisionType::Normal => dist <= radius,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radius() {
        assert_eq!(light_radius(LightSource::MagicLamp), 5);
    }

    #[test]
    fn test_duration() {
        assert_eq!(light_duration(LightSource::MagicLamp), None);
        assert_eq!(light_duration(LightSource::OilLamp), Some(1500));
    }

    #[test]
    fn test_normal_see() {
        assert!(can_see_at(5, 5, 7, 7, 4, VisionType::Normal, false));
        assert!(!can_see_at(5, 5, 15, 15, 4, VisionType::Normal, false));
    }

    #[test]
    fn test_infra() {
        assert!(can_see_at(5, 5, 50, 50, 4, VisionType::Infravision, true));
    }

    #[test]
    fn test_blind() {
        assert!(!can_see_at(5, 5, 5, 5, 10, VisionType::Blind, true));
    }
}

// ============================================================================
// [v2.30.0 Phase 94-3] 환경 효과 확장 (region_phase94_ext.rs)
// 원본: NetHack 3.6.7 src/region.c + pline.c L200-800 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 영역 효과 — region_effects (region.c L200-500)
// =============================================================================

/// [v2.30.0 94-3] 영역 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    PoisonGas,
    StinkingCloud,
    Fog,
    Fire,
    Ice,
    Silence,
    AntiMagic,
    Sanctuary,
    Darkness,
    Light,
}

/// [v2.30.0 94-3] 영역 정보
#[derive(Debug, Clone)]
pub struct Region {
    pub region_type: RegionType,
    pub x: i32,
    pub y: i32,
    pub radius: i32,
    pub turns_remaining: i32,
    pub source: String,
}

/// [v2.30.0 94-3] 영역 효과 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionEffect {
    Damage { amount: i32, element: String },
    StatusInflict { status: String, turns: i32 },
    SpellBlock,
    Protection { ac_bonus: i32 },
    VisionChange { blind: bool },
    NoEffect,
}

/// [v2.30.0 94-3] 영역 효과 적용
/// 원본: region.c inside_region()
pub fn apply_region_effect(
    region: &Region,
    target_x: i32,
    target_y: i32,
    has_resistance: &dyn Fn(&str) -> bool,
    rng: &mut NetHackRng,
) -> RegionEffect {
    // 범위 확인
    let dist = (region.x - target_x).abs().max((region.y - target_y).abs());
    if dist > region.radius {
        return RegionEffect::NoEffect;
    }

    match region.region_type {
        RegionType::PoisonGas => {
            if has_resistance("독") {
                RegionEffect::NoEffect
            } else {
                RegionEffect::Damage {
                    amount: rng.rn2(4) + 2,
                    element: "독".to_string(),
                }
            }
        }
        RegionType::StinkingCloud => {
            if has_resistance("독") {
                RegionEffect::NoEffect
            } else {
                RegionEffect::StatusInflict {
                    status: "혼란".to_string(),
                    turns: rng.rn2(10) + 5,
                }
            }
        }
        RegionType::Fog => RegionEffect::VisionChange { blind: false },
        RegionType::Fire => {
            if has_resistance("화염") {
                RegionEffect::NoEffect
            } else {
                RegionEffect::Damage {
                    amount: rng.rn2(8) + 4,
                    element: "화염".to_string(),
                }
            }
        }
        RegionType::Ice => {
            if has_resistance("냉기") {
                RegionEffect::NoEffect
            } else {
                RegionEffect::Damage {
                    amount: rng.rn2(6) + 3,
                    element: "냉기".to_string(),
                }
            }
        }
        RegionType::Silence => RegionEffect::SpellBlock,
        RegionType::AntiMagic => RegionEffect::SpellBlock,
        RegionType::Sanctuary => RegionEffect::Protection { ac_bonus: 5 },
        RegionType::Darkness => RegionEffect::VisionChange { blind: true },
        RegionType::Light => RegionEffect::VisionChange { blind: false },
    }
}

// =============================================================================
// [2] 영역 업데이트 — region_tick (region.c L500-700)
// =============================================================================

/// [v2.30.0 94-3] 영역 틱 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionTickResult {
    Active { turns_left: i32 },
    Fading { turns_left: i32 },
    Expired,
}

/// [v2.30.0 94-3] 영역 턴 업데이트
pub fn region_tick(region: &Region) -> RegionTickResult {
    let remaining = region.turns_remaining - 1;
    if remaining <= 0 {
        RegionTickResult::Expired
    } else if remaining <= 3 {
        RegionTickResult::Fading {
            turns_left: remaining,
        }
    } else {
        RegionTickResult::Active {
            turns_left: remaining,
        }
    }
}

// =============================================================================
// [3] 메시지 시스템 — message_format (pline.c L200-400)
// =============================================================================

/// [v2.30.0 94-3] 메시지 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Normal,
    Warning,
    Danger,
    StatusChange,
    ItemAction,
    MonsterAction,
    MapAction,
    Quest,
    Achievement,
    System,
}

/// [v2.30.0 94-3] 게임 메시지
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameMessage {
    pub msg_type: MessageType,
    pub text: String,
    pub turn: i32,
    pub priority: i32,
}

/// [v2.30.0 94-3] 메시지 생성
pub fn create_message(msg_type: MessageType, text: &str, turn: i32) -> GameMessage {
    let priority = match msg_type {
        MessageType::Danger => 10,
        MessageType::Warning => 8,
        MessageType::Quest | MessageType::Achievement => 7,
        MessageType::StatusChange => 5,
        MessageType::MonsterAction | MessageType::ItemAction => 4,
        MessageType::Normal => 3,
        MessageType::MapAction => 2,
        MessageType::System => 1,
    };

    GameMessage {
        msg_type,
        text: text.to_string(),
        turn,
        priority,
    }
}

/// [v2.30.0 94-3] 메시지 로그 관리
#[derive(Debug, Clone)]
pub struct MessageLog {
    pub messages: Vec<GameMessage>,
    pub max_size: usize,
}

impl MessageLog {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_size,
        }
    }

    pub fn add(&mut self, msg: GameMessage) {
        self.messages.push(msg);
        if self.messages.len() > self.max_size {
            self.messages.remove(0);
        }
    }

    pub fn recent(&self, count: usize) -> &[GameMessage] {
        let start = if self.messages.len() > count {
            self.messages.len() - count
        } else {
            0
        };
        &self.messages[start..]
    }

    pub fn filter_by_type(&self, msg_type: MessageType) -> Vec<&GameMessage> {
        self.messages
            .iter()
            .filter(|m| m.msg_type == msg_type)
            .collect()
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

    fn test_region() -> Region {
        Region {
            region_type: RegionType::PoisonGas,
            x: 10,
            y: 10,
            radius: 3,
            turns_remaining: 10,
            source: "함정".to_string(),
        }
    }

    #[test]
    fn test_poison_gas_damage() {
        let mut rng = test_rng();
        let region = test_region();
        let result = apply_region_effect(&region, 11, 10, &|_| false, &mut rng);
        assert!(matches!(result, RegionEffect::Damage { .. }));
    }

    #[test]
    fn test_poison_resistant() {
        let mut rng = test_rng();
        let region = test_region();
        let result = apply_region_effect(&region, 11, 10, &|r| r == "독", &mut rng);
        assert!(matches!(result, RegionEffect::NoEffect));
    }

    #[test]
    fn test_out_of_range() {
        let mut rng = test_rng();
        let region = test_region();
        let result = apply_region_effect(&region, 50, 50, &|_| false, &mut rng);
        assert!(matches!(result, RegionEffect::NoEffect));
    }

    #[test]
    fn test_fire_damage() {
        let mut rng = test_rng();
        let mut region = test_region();
        region.region_type = RegionType::Fire;
        let result = apply_region_effect(&region, 10, 10, &|_| false, &mut rng);
        assert!(matches!(result, RegionEffect::Damage { .. }));
    }

    #[test]
    fn test_anti_magic() {
        let mut rng = test_rng();
        let mut region = test_region();
        region.region_type = RegionType::AntiMagic;
        let result = apply_region_effect(&region, 10, 10, &|_| false, &mut rng);
        assert!(matches!(result, RegionEffect::SpellBlock));
    }

    #[test]
    fn test_region_tick_active() {
        let region = test_region();
        let result = region_tick(&region);
        assert!(matches!(result, RegionTickResult::Active { .. }));
    }

    #[test]
    fn test_region_tick_expired() {
        let mut region = test_region();
        region.turns_remaining = 1;
        let result = region_tick(&region);
        assert!(matches!(result, RegionTickResult::Expired));
    }

    #[test]
    fn test_message_log() {
        let mut log = MessageLog::new(100);
        log.add(create_message(MessageType::Normal, "테스트 1", 1));
        log.add(create_message(MessageType::Danger, "위험!", 2));
        assert_eq!(log.messages.len(), 2);
        assert_eq!(log.recent(1).len(), 1);
    }

    #[test]
    fn test_message_filter() {
        let mut log = MessageLog::new(100);
        log.add(create_message(MessageType::Normal, "일반", 1));
        log.add(create_message(MessageType::Danger, "위험1", 2));
        log.add(create_message(MessageType::Danger, "위험2", 3));
        let dangers = log.filter_by_type(MessageType::Danger);
        assert_eq!(dangers.len(), 2);
    }

    #[test]
    fn test_message_priority() {
        let danger = create_message(MessageType::Danger, "위험", 1);
        let normal = create_message(MessageType::Normal, "일반", 1);
        assert!(danger.priority > normal.priority);
    }
}

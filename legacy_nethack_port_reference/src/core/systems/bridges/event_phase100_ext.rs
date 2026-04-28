// ============================================================================
// [v2.36.0 Phase 100-3] 이벤트 시스템 통합 (event_phase100_ext.rs)
// 원본: NetHack 3.6.7 전체 이벤트 처리 통합
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 게임 이벤트 — game_events
// =============================================================================

/// [v2.36.0 100-3] 게임 이벤트 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEventType {
    // 전투 이벤트
    PlayerAttack { target: String, damage: i32 },
    MonsterAttack { attacker: String, damage: i32 },
    MonsterDeath { name: String, xp: i32 },
    PlayerDeath { cause: String },

    // 아이템 이벤트
    ItemPickup { name: String, quantity: i32 },
    ItemDrop { name: String, quantity: i32 },
    ItemUse { name: String, effect: String },
    ItemEquip { name: String, slot: String },
    ItemIdentify { name: String },

    // 이동 이벤트
    Move { dx: i32, dy: i32 },
    LevelChange { new_level: i32, direction: String },
    TrapTriggered { trap_type: String },
    DoorOpen,
    DoorClose,

    // 마법 이벤트
    SpellCast { spell: String, target: String },
    WandZap { wand: String },
    ScrollRead { scroll: String },
    PotionDrink { potion: String },

    // 사회 이벤트
    ShopBuy { item: String, cost: i32 },
    ShopSell { item: String, earned: i32 },
    PrayerAnswer { god: String, result: String },
    NPCDialog { npc: String, topic: String },

    // 시스템 이벤트
    TurnStart { turn: i32 },
    TurnEnd { turn: i32 },
    GameSave,
    GameLoad,
    LevelGenerate { depth: i32 },
    Achievement { name: String },
}

/// [v2.36.0 100-3] 이벤트 항목
#[derive(Debug, Clone)]
pub struct GameEvent {
    pub event_type: GameEventType,
    pub turn: i32,
    pub timestamp_ms: u64, // 게임 내 시각
    pub priority: i32,
}

/// [v2.36.0 100-3] 이벤트 큐
#[derive(Debug, Clone)]
pub struct EventQueue {
    pub events: Vec<GameEvent>,
    pub max_history: usize,
    pub listeners: Vec<String>, // 리스너 이름
}

impl EventQueue {
    pub fn new(max_history: usize) -> Self {
        Self {
            events: Vec::new(),
            max_history,
            listeners: Vec::new(),
        }
    }

    /// 이벤트 발행
    pub fn emit(&mut self, event_type: GameEventType, turn: i32, priority: i32) {
        self.events.push(GameEvent {
            event_type,
            turn,
            timestamp_ms: turn as u64 * 1000,
            priority,
        });

        // 이력 제한
        if self.events.len() > self.max_history {
            self.events.remove(0);
        }
    }

    /// 특정 턴의 이벤트
    pub fn events_for_turn(&self, turn: i32) -> Vec<&GameEvent> {
        self.events.iter().filter(|e| e.turn == turn).collect()
    }

    /// 전투 이벤트만
    pub fn combat_events(&self) -> Vec<&GameEvent> {
        self.events
            .iter()
            .filter(|e| {
                matches!(
                    e.event_type,
                    GameEventType::PlayerAttack { .. }
                        | GameEventType::MonsterAttack { .. }
                        | GameEventType::MonsterDeath { .. }
                        | GameEventType::PlayerDeath { .. }
                )
            })
            .collect()
    }

    /// 최근 N개
    pub fn recent(&self, count: usize) -> Vec<&GameEvent> {
        self.events.iter().rev().take(count).collect()
    }

    /// 리스너 등록
    pub fn add_listener(&mut self, name: &str) {
        self.listeners.push(name.to_string());
    }

    /// 이벤트 수
    pub fn count(&self) -> usize {
        self.events.len()
    }
}

// =============================================================================
// [2] 이벤트 직렬화 — event_log
// =============================================================================

/// [v2.36.0 100-3] 이벤트 로그 포맷
pub fn format_event_log(event: &GameEvent) -> String {
    let desc = match &event.event_type {
        GameEventType::PlayerAttack { target, damage } => {
            format!("[전투] {}에게 {}데미지!", target, damage)
        }
        GameEventType::MonsterAttack { attacker, damage } => {
            format!("[피격] {}이(가) {}데미지!", attacker, damage)
        }
        GameEventType::MonsterDeath { name, xp } => format!("[처치] {} (+{}xp)", name, xp),
        GameEventType::ItemPickup { name, quantity } => format!("[획득] {} x{}", name, quantity),
        GameEventType::SpellCast { spell, target } => format!("[마법] {} → {}", spell, target),
        GameEventType::LevelChange {
            new_level,
            direction,
        } => format!("[이동] {}층으로 {}", new_level, direction),
        GameEventType::Achievement { name } => format!("[달성] 🏆 {}", name),
        GameEventType::TurnStart { turn } => format!("[턴 {}] 시작", turn),
        _ => format!("[이벤트] {:?}", event.event_type),
    };
    format!("T{:05} | {}", event.turn, desc)
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_emit() {
        let mut queue = EventQueue::new(100);
        queue.emit(GameEventType::TurnStart { turn: 1 }, 1, 0);
        assert_eq!(queue.count(), 1);
    }

    #[test]
    fn test_event_overflow() {
        let mut queue = EventQueue::new(3);
        queue.emit(GameEventType::TurnStart { turn: 1 }, 1, 0);
        queue.emit(GameEventType::TurnStart { turn: 2 }, 2, 0);
        queue.emit(GameEventType::TurnStart { turn: 3 }, 3, 0);
        queue.emit(GameEventType::TurnStart { turn: 4 }, 4, 0);
        assert_eq!(queue.count(), 3);
    }

    #[test]
    fn test_combat_filter() {
        let mut queue = EventQueue::new(100);
        queue.emit(
            GameEventType::PlayerAttack {
                target: "오크".to_string(),
                damage: 10,
            },
            1,
            5,
        );
        queue.emit(GameEventType::TurnStart { turn: 1 }, 1, 0);
        queue.emit(
            GameEventType::MonsterDeath {
                name: "오크".to_string(),
                xp: 50,
            },
            1,
            5,
        );
        assert_eq!(queue.combat_events().len(), 2);
    }

    #[test]
    fn test_turn_filter() {
        let mut queue = EventQueue::new(100);
        queue.emit(GameEventType::TurnStart { turn: 1 }, 1, 0);
        queue.emit(GameEventType::TurnStart { turn: 2 }, 2, 0);
        assert_eq!(queue.events_for_turn(1).len(), 1);
    }

    #[test]
    fn test_listener() {
        let mut queue = EventQueue::new(100);
        queue.add_listener("전투 시스템");
        queue.add_listener("로그 시스템");
        assert_eq!(queue.listeners.len(), 2);
    }

    #[test]
    fn test_format_combat() {
        let event = GameEvent {
            event_type: GameEventType::PlayerAttack {
                target: "드래곤".to_string(),
                damage: 25,
            },
            turn: 100,
            timestamp_ms: 100000,
            priority: 5,
        };
        let log = format_event_log(&event);
        assert!(log.contains("드래곤"));
        assert!(log.contains("25"));
    }

    #[test]
    fn test_format_achievement() {
        let event = GameEvent {
            event_type: GameEventType::Achievement {
                name: "첫 번째 처치".to_string(),
            },
            turn: 5,
            timestamp_ms: 5000,
            priority: 10,
        };
        let log = format_event_log(&event);
        assert!(log.contains("🏆"));
    }

    #[test]
    fn test_recent() {
        let mut queue = EventQueue::new(100);
        for i in 1..=10 {
            queue.emit(GameEventType::TurnStart { turn: i }, i, 0);
        }
        let recent = queue.recent(3);
        assert_eq!(recent.len(), 3);
    }
}

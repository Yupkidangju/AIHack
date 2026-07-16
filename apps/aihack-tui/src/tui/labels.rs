use crate::core::{observation::Observation, position::Pos};

/// [v0.2.0] Phase 19: 자동 라벨의 종류다.
/// 우선순위는 숫자가 낮을수록 높다 (1이 가장 높음).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelKind {
    /// [v0.2.0] Phase 19: 인접한 적대 몬스터. 가장 높은 우선순위.
    HostileAdjacent,
    /// [v0.2.0] Phase 19: 플레이어 HP 30% 이하.
    LowHpWarning,
    /// [v0.2.0] Phase 19: 계단.
    Stairs,
    /// [v0.2.0] Phase 19: 미식별 아이템.
    UnidentifiedItem,
    /// [v0.2.0] Phase 19: 비적대 몬스터.
    PassiveMonster,
}

impl LabelKind {
    /// [v0.2.0] Phase 19: 라벨 우선순위 값을 반환한다.
    /// 숫자가 작을수록 높은 우선순위다.
    pub fn priority(self) -> u8 {
        match self {
            LabelKind::HostileAdjacent => 1,
            LabelKind::LowHpWarning => 2,
            LabelKind::Stairs => 3,
            LabelKind::UnidentifiedItem => 4,
            LabelKind::PassiveMonster => 5,
        }
    }
}

/// [v0.2.0] Phase 19: 자동 라벨 데이터다.
/// 맵 위에 일정 시간 동안 표시되는 텍스트 라벨이다.
#[derive(Debug, Clone)]
pub struct AutoLabel {
    pub kind: LabelKind,
    pub pos: Pos,
    pub text: String,
    pub created_at_ms: u64,
    pub duration_ms: u16,
}

/// [v0.2.0] Phase 19: Observation 기준으로 자동 라벨을 수집한다.
/// 우선순위: hostile adjacent > low HP > stairs > unidentified item > passive monster
/// 최대 3개까지 반환하며, 각 라벨은 1200ms 동안 표시된다.
pub fn collect_auto_labels(observation: &Observation, current_time_ms: u64) -> Vec<AutoLabel> {
    let mut candidates = Vec::new();

    // visible_entities 순회
    for entity in &observation.visible_entities {
        if !entity.alive {
            continue;
        }

        let dx = (entity.pos.x - observation.player_pos.x).abs();
        let dy = (entity.pos.y - observation.player_pos.y).abs();
        let is_adjacent = dx <= 1 && dy <= 1 && (dx + dy) > 0;

        if let crate::domain::entity::EntityKind::Monster(monster_kind) = entity.kind {
            let is_hostile = matches!(
                monster_kind,
                crate::domain::monster::MonsterKind::Jackal
                    | crate::domain::monster::MonsterKind::Goblin
            );
            if is_hostile && is_adjacent {
                candidates.push(AutoLabel {
                    kind: LabelKind::HostileAdjacent,
                    pos: entity.pos,
                    text: format!(
                        "[{}]",
                        match monster_kind {
                            crate::domain::monster::MonsterKind::Jackal => "d",
                            crate::domain::monster::MonsterKind::Goblin => "g",
                            _ => "?",
                        }
                    ),
                    created_at_ms: current_time_ms,
                    duration_ms: 1200,
                });
            } else if !is_hostile && is_adjacent {
                // PassiveMonster
                candidates.push(AutoLabel {
                    kind: LabelKind::PassiveMonster,
                    pos: entity.pos,
                    text: "[floating eye]".to_string(),
                    created_at_ms: current_time_ms,
                    duration_ms: 1200,
                });
            }
        }
    }

    // HP 30% 이하 체크
    let hp_ratio = if observation.player.max_hp > 0 {
        observation.player.hp as f32 / observation.player.max_hp as f32
    } else {
        1.0
    };
    if hp_ratio <= 0.30 {
        candidates.push(AutoLabel {
            kind: LabelKind::LowHpWarning,
            pos: observation.player_pos,
            text: "[LOW HP]".to_string(),
            created_at_ms: current_time_ms,
            duration_ms: 1600, // spec 15.6 danger_label_ms
        });
    }

    // 계단 체크
    for tile in &observation.visible_tiles {
        use crate::domain::tile::TileKind;
        if matches!(tile.tile, TileKind::StairsDown | TileKind::StairsUp) {
            candidates.push(AutoLabel {
                kind: LabelKind::Stairs,
                pos: tile.pos,
                text: "[>]".to_string(),
                created_at_ms: current_time_ms,
                duration_ms: 1200,
            });
        }
    }

    // 미식별 아이템 체크
    let has_unidentified = observation.inventory.iter().any(|item| !item.identified);
    if has_unidentified {
        candidates.push(AutoLabel {
            kind: LabelKind::UnidentifiedItem,
            pos: observation.player_pos,
            text: "[? item]".to_string(),
            created_at_ms: current_time_ms,
            duration_ms: 1200,
        });
    }

    // 우선순위 정렬 후 상위 3개 선택
    candidates.sort_by_key(|l| l.kind.priority());
    candidates.into_iter().take(3).collect()
}

/// [v0.2.0] Phase 19: 만료된 라벨을 필터링한다.
pub fn filter_expired_labels(labels: &mut Vec<AutoLabel>, current_time_ms: u64) {
    labels.retain(|label| {
        let elapsed = current_time_ms.saturating_sub(label.created_at_ms);
        elapsed < label.duration_ms as u64
    });
}

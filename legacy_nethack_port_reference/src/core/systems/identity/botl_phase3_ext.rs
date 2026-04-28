// ============================================================================
// [v2.24.0 Phase 3-3] 상태 바 확장 (botl_phase3_ext.rs)
// 원본: NetHack 3.6.7 src/botl.c L800-1600 핵심 미이식 함수 이식
// 순수 결과 패턴: ECS 의존 없이 독립 테스트 가능
// ============================================================================

// =============================================================================
// [1] 상태 필드 변경 감지 — status_update (botl.c L800-950)
// 개별 필드 변경 감지 + 콜백 트리거 결정
// =============================================================================

/// [v2.24.0 3-3] 상태 필드 ID (원본: BL_xxx 상수)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusFieldId {
    Title,
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
    Align,
    Score,
    CarryCap,
    Gold,
    EnergyMax,
    Energy,
    HpMax,
    Hp,
    Ac,
    Level,
    Exp,
    Hunger,
    DungeonLevel,
    Condition,
    Time,
}

/// [v2.24.0 3-3] 필드 변경 정보
#[derive(Debug, Clone, PartialEq)]
pub struct FieldChange {
    pub field: StatusFieldId,
    pub old_value: i64,
    pub new_value: i64,
    /// 색상 변경이 필요한지
    pub color_changed: bool,
    /// 화면 갱신이 필요한 필드인지
    pub needs_redraw: bool,
}

/// [v2.24.0 3-3] 상태 업데이트 배치 결과
#[derive(Debug, Clone)]
pub struct StatusUpdateBatch {
    /// 변경된 필드 목록
    pub changes: Vec<FieldChange>,
    /// 전체 리드로우(full redraw) 필요 여부
    pub full_redraw: bool,
    /// 위험 경고 메시지
    pub warnings: Vec<String>,
}

/// [v2.24.0 3-3] 상태 값 스냅샷 (정수로 통일)
#[derive(Debug, Clone, PartialEq)]
pub struct StatusSnapshot {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub energy: i32,
    pub energy_max: i32,
    pub ac: i32,
    pub level: i32,
    pub exp: u64,
    pub gold: u64,
    pub hunger: i32, // HungerState 인덱스
    pub condition_mask: u32,
    pub dungeon_depth: i32,
    pub time: u64,
}

/// [v2.24.0 3-3] 두 스냅샷 비교 → 변경 필드 목록 생성
/// 원본: botl.c status_update() L800-950
pub fn detect_field_changes(old: &StatusSnapshot, new: &StatusSnapshot) -> StatusUpdateBatch {
    let mut changes = Vec::new();
    let mut warnings = Vec::new();
    let mut full_redraw = false;

    // 매크로로 필드 비교를 줄임
    macro_rules! check_field {
        ($field:ident, $id:expr, $old_val:expr, $new_val:expr) => {
            if $old_val != $new_val {
                let color_changed = match $id {
                    StatusFieldId::Hp | StatusFieldId::Energy | StatusFieldId::Ac => true,
                    _ => false,
                };
                changes.push(FieldChange {
                    field: $id,
                    old_value: $old_val as i64,
                    new_value: $new_val as i64,
                    color_changed,
                    needs_redraw: true,
                });
            }
        };
    }

    check_field!(
        strength,
        StatusFieldId::Strength,
        old.strength,
        new.strength
    );
    check_field!(
        dexterity,
        StatusFieldId::Dexterity,
        old.dexterity,
        new.dexterity
    );
    check_field!(
        constitution,
        StatusFieldId::Constitution,
        old.constitution,
        new.constitution
    );
    check_field!(
        intelligence,
        StatusFieldId::Intelligence,
        old.intelligence,
        new.intelligence
    );
    check_field!(wisdom, StatusFieldId::Wisdom, old.wisdom, new.wisdom);
    check_field!(
        charisma,
        StatusFieldId::Charisma,
        old.charisma,
        new.charisma
    );
    check_field!(hp, StatusFieldId::Hp, old.hp, new.hp);
    check_field!(hp_max, StatusFieldId::HpMax, old.hp_max, new.hp_max);
    check_field!(energy, StatusFieldId::Energy, old.energy, new.energy);
    check_field!(
        energy_max,
        StatusFieldId::EnergyMax,
        old.energy_max,
        new.energy_max
    );
    check_field!(ac, StatusFieldId::Ac, old.ac, new.ac);
    check_field!(level, StatusFieldId::Level, old.level, new.level);
    check_field!(
        dungeon_depth,
        StatusFieldId::DungeonLevel,
        old.dungeon_depth,
        new.dungeon_depth
    );

    // exp는 u64 → i64 변환 주의
    if old.exp != new.exp {
        changes.push(FieldChange {
            field: StatusFieldId::Exp,
            old_value: old.exp as i64,
            new_value: new.exp as i64,
            color_changed: false,
            needs_redraw: true,
        });
    }

    // gold
    if old.gold != new.gold {
        changes.push(FieldChange {
            field: StatusFieldId::Gold,
            old_value: old.gold as i64,
            new_value: new.gold as i64,
            color_changed: false,
            needs_redraw: true,
        });
    }

    // hunger
    if old.hunger != new.hunger {
        changes.push(FieldChange {
            field: StatusFieldId::Hunger,
            old_value: old.hunger as i64,
            new_value: new.hunger as i64,
            color_changed: true,
            needs_redraw: true,
        });
    }

    // condition_mask
    if old.condition_mask != new.condition_mask {
        changes.push(FieldChange {
            field: StatusFieldId::Condition,
            old_value: old.condition_mask as i64,
            new_value: new.condition_mask as i64,
            color_changed: true,
            needs_redraw: true,
        });
        full_redraw = true; // 상태이상 변경은 전체 리드로우
    }

    // time
    if old.time != new.time {
        changes.push(FieldChange {
            field: StatusFieldId::Time,
            old_value: old.time as i64,
            new_value: new.time as i64,
            color_changed: false,
            needs_redraw: false, // 시간은 자동 갱신
        });
    }

    // [2] 위험 경고 생성
    if new.hp > 0 && new.hp_max > 0 {
        let hp_pct = new.hp * 100 / new.hp_max;
        if hp_pct <= 10 && old.hp * 100 / old.hp_max.max(1) > 10 {
            warnings.push("*** 위험: HP가 극도로 낮다! ***".to_string());
        } else if hp_pct <= 25 && old.hp * 100 / old.hp_max.max(1) > 25 {
            warnings.push("** 경고: HP가 낮다 **".to_string());
        }
    }

    if new.hunger >= 4 && old.hunger < 4 {
        // 허기 → 요약
        warnings.push("* 배가 매우 고프다 *".to_string());
    }

    StatusUpdateBatch {
        changes,
        full_redraw,
        warnings,
    }
}

// =============================================================================
// [2] 상태 바 임계값 색상 — status_threshold (botl.c L1200-1400)
// =============================================================================

/// [v2.24.0 3-3] 상태 임계값 규칙
#[derive(Debug, Clone)]
pub struct ThresholdRule {
    pub field: StatusFieldId,
    pub behavior: ThresholdBehavior,
    pub threshold_value: i64,
    pub color: [u8; 3],
}

/// [v2.24.0 3-3] 임계값 동작 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdBehavior {
    /// 값이 임계값 이하일 때 활성
    AtOrBelow,
    /// 값이 임계값 이상일 때 활성
    AtOrAbove,
    /// 값이 변경되었을 때 활성
    Changed,
    /// 특정 퍼센트 이하일 때 활성 (분자, 분모)
    PercentBelow,
}

/// [v2.24.0 3-3] 임계값 체크 결과
pub fn check_threshold(
    rule: &ThresholdRule,
    current_value: i64,
    max_value: i64,
    changed: bool,
) -> Option<[u8; 3]> {
    match rule.behavior {
        ThresholdBehavior::AtOrBelow => {
            if current_value <= rule.threshold_value {
                Some(rule.color)
            } else {
                None
            }
        }
        ThresholdBehavior::AtOrAbove => {
            if current_value >= rule.threshold_value {
                Some(rule.color)
            } else {
                None
            }
        }
        ThresholdBehavior::Changed => {
            if changed {
                Some(rule.color)
            } else {
                None
            }
        }
        ThresholdBehavior::PercentBelow => {
            if max_value > 0 {
                let pct = current_value * 100 / max_value;
                if pct <= rule.threshold_value {
                    Some(rule.color)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

/// [v2.24.0 3-3] 기본 HP 임계값 규칙 생성 (원본: botl.c 기본 설정)
pub fn default_hp_thresholds() -> Vec<ThresholdRule> {
    vec![
        ThresholdRule {
            field: StatusFieldId::Hp,
            behavior: ThresholdBehavior::PercentBelow,
            threshold_value: 10,
            color: [255, 50, 50], // 빨간색 — 위험
        },
        ThresholdRule {
            field: StatusFieldId::Hp,
            behavior: ThresholdBehavior::PercentBelow,
            threshold_value: 25,
            color: [255, 165, 0], // 주황색 — 경고
        },
        ThresholdRule {
            field: StatusFieldId::Hp,
            behavior: ThresholdBehavior::PercentBelow,
            threshold_value: 50,
            color: [255, 255, 0], // 노란색 — 주의
        },
    ]
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_snapshot() -> StatusSnapshot {
        StatusSnapshot {
            strength: 16,
            dexterity: 14,
            constitution: 16,
            intelligence: 12,
            wisdom: 10,
            charisma: 10,
            hp: 50,
            hp_max: 50,
            energy: 30,
            energy_max: 30,
            ac: 5,
            level: 10,
            exp: 50000,
            gold: 1000,
            hunger: 1,
            condition_mask: 0,
            dungeon_depth: 5,
            time: 100,
        }
    }

    #[test]
    fn test_no_changes() {
        let s = default_snapshot();
        let result = detect_field_changes(&s, &s);
        assert!(result.changes.is_empty());
        assert!(!result.full_redraw);
    }

    #[test]
    fn test_hp_change() {
        let old = default_snapshot();
        let mut new = old.clone();
        new.hp = 30;
        let result = detect_field_changes(&old, &new);
        assert!(result.changes.iter().any(|c| c.field == StatusFieldId::Hp));
        let hp_change = result
            .changes
            .iter()
            .find(|c| c.field == StatusFieldId::Hp)
            .unwrap();
        assert!(hp_change.color_changed);
    }

    #[test]
    fn test_hp_low_warning() {
        let old = default_snapshot();
        let mut new = old.clone();
        new.hp = 5; // 10%
        let result = detect_field_changes(&old, &new);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_condition_change_triggers_redraw() {
        let old = default_snapshot();
        let mut new = old.clone();
        new.condition_mask = 0x0001; // BLIND
        let result = detect_field_changes(&old, &new);
        assert!(result.full_redraw);
    }

    #[test]
    fn test_gold_change() {
        let old = default_snapshot();
        let mut new = old.clone();
        new.gold = 5000;
        let result = detect_field_changes(&old, &new);
        assert!(result
            .changes
            .iter()
            .any(|c| c.field == StatusFieldId::Gold));
    }

    #[test]
    fn test_multiple_changes() {
        let old = default_snapshot();
        let mut new = old.clone();
        new.hp = 40;
        new.gold = 2000;
        new.level = 11;
        let result = detect_field_changes(&old, &new);
        assert_eq!(result.changes.len(), 3);
    }

    // --- threshold ---

    #[test]
    fn test_threshold_below() {
        let rule = ThresholdRule {
            field: StatusFieldId::Hp,
            behavior: ThresholdBehavior::AtOrBelow,
            threshold_value: 10,
            color: [255, 0, 0],
        };
        assert!(check_threshold(&rule, 5, 50, false).is_some());
        assert!(check_threshold(&rule, 15, 50, false).is_none());
    }

    #[test]
    fn test_threshold_percent() {
        let rule = ThresholdRule {
            field: StatusFieldId::Hp,
            behavior: ThresholdBehavior::PercentBelow,
            threshold_value: 25,
            color: [255, 165, 0],
        };
        // 10/50 = 20% <= 25%
        assert!(check_threshold(&rule, 10, 50, false).is_some());
        // 30/50 = 60% > 25%
        assert!(check_threshold(&rule, 30, 50, false).is_none());
    }

    #[test]
    fn test_default_hp_thresholds() {
        let rules = default_hp_thresholds();
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].threshold_value, 10);
    }

    #[test]
    fn test_hunger_warning() {
        let old = default_snapshot();
        let mut new = old.clone();
        new.hunger = 4; // 심각한 허기
        let result = detect_field_changes(&old, &new);
        assert!(!result.warnings.is_empty());
    }
}

// ============================================================================
// [v2.30.0 R18-5] 몬스터 AI 통합 (ai_brain_ext.rs)
// AI 결정 통합 계층 — 이동/전투/마법/아이템/추적을 하나로
// 이것이 LLM 교체 지점의 핵심 인터페이스
// ============================================================================

/// [v2.30.0 R18-5] AI 행동 요청 (순수 입력)
#[derive(Debug, Clone)]
pub struct AiPerceptionInput {
    pub hp_ratio: f64,
    pub intelligence: i32,
    pub is_peaceful: bool,
    pub is_fleeing: bool,
    pub can_see_player: bool,
    pub distance_to_player: i32,
    pub has_ranged_attack: bool,
    pub has_items: bool,
    pub has_spells: bool,
    pub ally_count_nearby: i32,
}

/// [v2.30.0 R18-5] AI 행동 결정 (순수 출력)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiDecision {
    /// 플레이어에게 접근
    Approach,
    /// 근접 공격
    MeleeAttack,
    /// 원거리 공격 (투척/마법)
    RangedAttack,
    /// 마법 시전
    CastSpell,
    /// 아이템 사용
    UseItem,
    /// 도주
    Flee,
    /// 대기 (패시브)
    Wait,
    /// 랜덤 이동
    Wander,
    /// 소환
    Summon,
}

/// [v2.30.0 R18-5] 통합 AI 결정 엔진
/// 이 함수가 나중에 LLM으로 교체 가능한 핵심 인터페이스
pub fn decide_action(input: &AiPerceptionInput) -> AiDecision {
    // 평화적 몬스터
    if input.is_peaceful {
        return AiDecision::Wander;
    }

    // 도주 상태
    if input.is_fleeing || input.hp_ratio < 0.15 {
        if input.has_items {
            return AiDecision::UseItem;
        } // 치유 시도
        return AiDecision::Flee;
    }

    // HP 위험 시 방어 우선
    if input.hp_ratio < 0.3 {
        if input.has_spells && input.intelligence >= 3 {
            return AiDecision::CastSpell;
        }
        if input.has_items {
            return AiDecision::UseItem;
        }
        return AiDecision::Flee;
    }

    // 공격 단계
    if !input.can_see_player {
        return AiDecision::Wander;
    }

    // 근접 범위
    if input.distance_to_player <= 1 {
        return AiDecision::MeleeAttack;
    }

    // 원거리 범위 (2+)
    if input.has_ranged_attack {
        return AiDecision::RangedAttack;
    }
    if input.has_spells && input.intelligence >= 2 {
        return AiDecision::CastSpell;
    }

    // 소환 (지능형 + 아군 부족)
    if input.has_spells && input.intelligence >= 4 && input.ally_count_nearby < 2 {
        return AiDecision::Summon;
    }

    AiDecision::Approach
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> AiPerceptionInput {
        AiPerceptionInput {
            hp_ratio: 0.8,
            intelligence: 3,
            is_peaceful: false,
            is_fleeing: false,
            can_see_player: true,
            distance_to_player: 5,
            has_ranged_attack: false,
            has_items: false,
            has_spells: false,
            ally_count_nearby: 0,
        }
    }

    #[test]
    fn test_peaceful_wander() {
        let input = AiPerceptionInput {
            is_peaceful: true,
            ..base_input()
        };
        assert_eq!(decide_action(&input), AiDecision::Wander);
    }

    #[test]
    fn test_melee() {
        let input = AiPerceptionInput {
            distance_to_player: 1,
            ..base_input()
        };
        assert_eq!(decide_action(&input), AiDecision::MeleeAttack);
    }

    #[test]
    fn test_ranged() {
        let input = AiPerceptionInput {
            has_ranged_attack: true,
            ..base_input()
        };
        assert_eq!(decide_action(&input), AiDecision::RangedAttack);
    }

    #[test]
    fn test_flee_low_hp() {
        let input = AiPerceptionInput {
            hp_ratio: 0.1,
            ..base_input()
        };
        assert_eq!(decide_action(&input), AiDecision::Flee);
    }

    #[test]
    fn test_approach() {
        assert_eq!(decide_action(&base_input()), AiDecision::Approach);
    }

    #[test]
    fn test_cast_when_hurt() {
        let input = AiPerceptionInput {
            hp_ratio: 0.2,
            has_spells: true,
            ..base_input()
        };
        assert_eq!(decide_action(&input), AiDecision::CastSpell);
    }

    #[test]
    fn test_blind_wander() {
        let input = AiPerceptionInput {
            can_see_player: false,
            ..base_input()
        };
        assert_eq!(decide_action(&input), AiDecision::Wander);
    }
}

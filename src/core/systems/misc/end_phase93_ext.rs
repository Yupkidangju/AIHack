// ============================================================================
// [v2.29.0 Phase 93-6] 사망/종료 시스템 확장 (end_phase93_ext.rs)
// 원본: NetHack 3.6.7 src/end.c L200-1200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 사망 원인 — death_cause (end.c L200-500)
// =============================================================================

/// [v2.29.0 93-6] 사망 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeathCause {
    /// 몬스터에 의한 사망
    KilledBy {
        monster_name: String,
        attack_type: String,
    },
    /// 함정 사망
    TrappedBy { trap_name: String },
    /// 낙하 사망
    FellDown { from_level: i32 },
    /// 독사
    Poisoned { source: String },
    /// 석화
    Petrified { source: String },
    /// 질병 사망
    Disease,
    /// 기아 사망
    Starvation,
    /// 익사
    Drowned,
    /// 용암 사망
    BurnedInLava,
    /// 자살 (퇴장)
    Quit,
    /// 에스케이프 (도주)
    Escaped { with_amulet: bool },
    /// 승천 (클리어)
    Ascended,
    /// 시스템/미분류
    Misc { message: String },
}

/// [v2.29.0 93-6] 사망 메시지 생성
/// 원본: end.c killer_format()
pub fn format_death_message(cause: &DeathCause, player_name: &str, player_level: i32) -> String {
    match cause {
        DeathCause::KilledBy {
            monster_name,
            attack_type,
        } => {
            format!(
                "{} (Lv.{})은(는) {}의 {}(으)로 사망했다.",
                player_name, player_level, monster_name, attack_type
            )
        }
        DeathCause::TrappedBy { trap_name } => {
            format!(
                "{} (Lv.{})은(는) {}에 걸려 사망했다.",
                player_name, player_level, trap_name
            )
        }
        DeathCause::FellDown { from_level } => {
            format!(
                "{} (Lv.{})은(는) {}층에서 떨어져 사망했다.",
                player_name, player_level, from_level
            )
        }
        DeathCause::Poisoned { source } => {
            format!(
                "{} (Lv.{})은(는) {}의 독으로 사망했다.",
                player_name, player_level, source
            )
        }
        DeathCause::Petrified { source } => {
            format!(
                "{} (Lv.{})은(는) {}에 의해 석화되었다.",
                player_name, player_level, source
            )
        }
        DeathCause::Disease => {
            format!(
                "{} (Lv.{})은(는) 질병으로 사망했다.",
                player_name, player_level
            )
        }
        DeathCause::Starvation => {
            format!(
                "{} (Lv.{})은(는) 굶주려 사망했다.",
                player_name, player_level
            )
        }
        DeathCause::Drowned => {
            format!("{} (Lv.{})은(는) 익사했다.", player_name, player_level)
        }
        DeathCause::BurnedInLava => {
            format!(
                "{} (Lv.{})은(는) 용암에 타 죽었다.",
                player_name, player_level
            )
        }
        DeathCause::Quit => {
            format!(
                "{} (Lv.{})은(는) 게임을 포기했다.",
                player_name, player_level
            )
        }
        DeathCause::Escaped { with_amulet } => {
            if *with_amulet {
                format!(
                    "{} (Lv.{})은(는) 아뮬렛과 함께 도주했다!",
                    player_name, player_level
                )
            } else {
                format!(
                    "{} (Lv.{})은(는) 던전에서 도주했다.",
                    player_name, player_level
                )
            }
        }
        DeathCause::Ascended => {
            format!(
                "{} (Lv.{})은(는) 승천하여 반신이 되었다!!!",
                player_name, player_level
            )
        }
        DeathCause::Misc { message } => {
            format!("{} (Lv.{}): {}", player_name, player_level, message)
        }
    }
}

// =============================================================================
// [2] 점수 계산 — score_calc (end.c L600-900)
// =============================================================================

/// [v2.29.0 93-6] 점수 계산 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreResult {
    pub base_score: i64,
    pub depth_bonus: i64,
    pub gold_bonus: i64,
    pub artifact_bonus: i64,
    pub ascension_bonus: i64,
    pub death_penalty: i64,
    pub total_score: i64,
}

/// [v2.29.0 93-6] 점수 계산
/// 원본: end.c calc_score()
pub fn calculate_score(
    player_level: i32,
    max_depth: i32,
    gold_earned: i64,
    monsters_killed: i32,
    artifacts_found: i32,
    ascended: bool,
    turns_used: i32,
    cause: &DeathCause,
) -> ScoreResult {
    let base = (player_level as i64) * 1000 + (monsters_killed as i64) * 10;
    let depth = (max_depth as i64) * 500;
    let gold = gold_earned;
    let artifacts = (artifacts_found as i64) * 5000;
    let ascension = if ascended { 100000 } else { 0 };
    let penalty = match cause {
        DeathCause::Quit => -5000,
        DeathCause::Starvation => -2000,
        _ => 0,
    };

    // 턴 보정 (효율성)
    let efficiency = if turns_used > 0 {
        (base + depth + gold + artifacts + ascension) / (turns_used as i64 / 100 + 1)
    } else {
        0
    };

    let total = base + depth + gold + artifacts + ascension + penalty + efficiency;

    ScoreResult {
        base_score: base,
        depth_bonus: depth,
        gold_bonus: gold,
        artifact_bonus: artifacts,
        ascension_bonus: ascension,
        death_penalty: penalty,
        total_score: total.max(0),
    }
}

// =============================================================================
// [3] 묘비 생성 — epitaph (end.c L1000-1200)
// =============================================================================

/// [v2.29.0 93-6] 묘비 텍스트 생성
pub fn generate_epitaph(
    player_name: &str,
    player_role: &str,
    player_race: &str,
    player_level: i32,
    cause: &DeathCause,
    score: i64,
    turns: i32,
) -> String {
    let mut lines = Vec::new();
    lines.push("╔══════════════════════════════╗".to_string());
    lines.push("║          R.I.P.              ║".to_string());
    lines.push(format!("║  {:^26}  ║", player_name));
    lines.push(format!(
        "║  {} {} Lv.{:>2}           ║",
        player_race, player_role, player_level
    ));
    lines.push("║                              ║".to_string());

    let death_text = match cause {
        DeathCause::Ascended => "승천하여 반신이 되다",
        DeathCause::KilledBy { .. } => "전투 중 사망",
        DeathCause::Starvation => "굶주림으로 사망",
        DeathCause::Drowned => "익사",
        DeathCause::Quit => "포기",
        _ => "사망",
    };
    lines.push(format!("║  {:^26}  ║", death_text));
    lines.push(format!("║  점수: {:>8}             ║", score));
    lines.push(format!("║  턴:   {:>8}             ║", turns));
    lines.push("╚══════════════════════════════╝".to_string());

    lines.join("\n")
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_death_killed_by() {
        let cause = DeathCause::KilledBy {
            monster_name: "드래곤".to_string(),
            attack_type: "화염 숨결".to_string(),
        };
        let msg = format_death_message(&cause, "용사", 15);
        assert!(msg.contains("드래곤"));
        assert!(msg.contains("화염 숨결"));
    }

    #[test]
    fn test_death_ascended() {
        let cause = DeathCause::Ascended;
        let msg = format_death_message(&cause, "용사", 30);
        assert!(msg.contains("승천"));
    }

    #[test]
    fn test_death_starvation() {
        let cause = DeathCause::Starvation;
        let msg = format_death_message(&cause, "용사", 5);
        assert!(msg.contains("굶주"));
    }

    #[test]
    fn test_score_normal() {
        let cause = DeathCause::KilledBy {
            monster_name: "고블린".to_string(),
            attack_type: "근접".to_string(),
        };
        let score = calculate_score(10, 15, 5000, 100, 2, false, 10000, &cause);
        assert!(score.total_score > 0);
    }

    #[test]
    fn test_score_ascended() {
        let cause = DeathCause::Ascended;
        let score = calculate_score(30, 50, 100000, 500, 5, true, 50000, &cause);
        assert!(score.ascension_bonus > 0);
        assert!(score.total_score > 100000);
    }

    #[test]
    fn test_score_penalty() {
        let cause = DeathCause::Quit;
        let score = calculate_score(5, 3, 100, 10, 0, false, 500, &cause);
        assert!(score.death_penalty < 0);
    }

    #[test]
    fn test_epitaph_format() {
        let cause = DeathCause::Ascended;
        let text = generate_epitaph("용사", "전사", "인간", 30, &cause, 500000, 30000);
        assert!(text.contains("R.I.P."));
        assert!(text.contains("용사"));
        assert!(text.contains("승천"));
    }

    #[test]
    fn test_all_death_types() {
        let causes = vec![
            DeathCause::KilledBy {
                monster_name: "M".to_string(),
                attack_type: "A".to_string(),
            },
            DeathCause::TrappedBy {
                trap_name: "T".to_string(),
            },
            DeathCause::FellDown { from_level: 5 },
            DeathCause::Poisoned {
                source: "P".to_string(),
            },
            DeathCause::Petrified {
                source: "S".to_string(),
            },
            DeathCause::Disease,
            DeathCause::Starvation,
            DeathCause::Drowned,
            DeathCause::BurnedInLava,
            DeathCause::Quit,
            DeathCause::Escaped { with_amulet: true },
            DeathCause::Ascended,
            DeathCause::Misc {
                message: "기타".to_string(),
            },
        ];
        for cause in causes {
            let msg = format_death_message(&cause, "테스트", 10);
            assert!(!msg.is_empty());
        }
    }
}

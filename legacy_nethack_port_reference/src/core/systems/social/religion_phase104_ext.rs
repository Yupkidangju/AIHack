// ============================================================================
// [v2.40.0 Phase 104-1] 신/종교 통합 (religion_phase104_ext.rs)
// 원본: NetHack 3.6.7 src/pray.c + src/priest.c 종교 시스템 통합
// 순수 결과 패턴
//
// 구현 범위:
//   - 신 체계 (혼돈/중립/질서 + 역할별 수호신)
//   - 기도 시스템 (기도 효과, 쿨다운, 분노)
//   - 제물 바치기 (아이템/시체/금화)
//   - 성향(alignment) 관리
//   - 신의 분노/축복/무관심 상태
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 신 체계 — pantheon
// =============================================================================

/// [v2.40.0 104-1] 성향 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Lawful,  // 질서
    Neutral, // 중립
    Chaotic, // 혼돈
}

/// [v2.40.0 104-1] 신의 태도
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivineFavor {
    Wrathful,    // 분노 (-100 이하)
    Displeased,  // 불쾌 (-50 ~ -1)
    Indifferent, // 무관심 (0 ~ 49)
    Pleased,     // 기쁨 (50 ~ 199)
    Devout,      // 독실 (200+)
}

/// [v2.40.0 104-1] 종교 상태
#[derive(Debug, Clone)]
pub struct ReligionState {
    pub alignment: Alignment,
    pub god_name: String,
    pub favor: i32,           // 신의 호감도
    pub prayer_cooldown: i32, // 기도 쿨다운 (턴)
    pub sins: i32,            // 죄악 수치
    pub gifts_received: i32,  // 받은 선물 수
    pub last_prayer_turn: i32,
}

/// [v2.40.0 104-1] 역할별 수호신
pub fn get_patron_god(role: &str, alignment: Alignment) -> String {
    match (role, alignment) {
        ("전사", Alignment::Chaotic) => "미트라".to_string(),
        ("마법사", Alignment::Neutral) => "타나토스".to_string(),
        ("기사", Alignment::Lawful) => "로그".to_string(),
        ("도적", Alignment::Chaotic) => "카르카".to_string(),
        ("성직자", Alignment::Neutral) => "블인드 아이오".to_string(),
        ("발키리", Alignment::Neutral) => "오딘".to_string(),
        _ => match alignment {
            Alignment::Lawful => "아누".to_string(),
            Alignment::Neutral => "아누비스".to_string(),
            Alignment::Chaotic => "세트".to_string(),
        },
    }
}

/// [v2.40.0 104-1] 신의 태도 판단
pub fn judge_favor(favor: i32) -> DivineFavor {
    match favor {
        f if f <= -100 => DivineFavor::Wrathful,
        f if f < 0 => DivineFavor::Displeased,
        f if f < 50 => DivineFavor::Indifferent,
        f if f < 200 => DivineFavor::Pleased,
        _ => DivineFavor::Devout,
    }
}

/// [v2.40.0 104-1] 기도
pub fn pray(
    state: &mut ReligionState,
    current_turn: i32,
    player_hp_ratio: f64,
    rng: &mut NetHackRng,
) -> String {
    // 쿨다운 체크
    let turns_since = current_turn - state.last_prayer_turn;
    if turns_since < state.prayer_cooldown {
        state.favor -= 10;
        return format!(
            "{}이(가) 분노한다! 너무 자주 기도하지 마라!",
            state.god_name
        );
    }

    state.last_prayer_turn = current_turn;
    state.prayer_cooldown = 300 + rng.rn2(200);

    let divine_favor = judge_favor(state.favor);

    match divine_favor {
        DivineFavor::Wrathful => {
            let punishment = rng.rn2(3);
            match punishment {
                0 => {
                    state.favor -= 20;
                    "번개가 내리친다!".to_string()
                }
                1 => "저주받은 아이템이 떨어진다!".to_string(),
                _ => "아무 응답도 없다... 아니, 불길한 기운이 감돈다.".to_string(),
            }
        }
        DivineFavor::Displeased => {
            state.favor += 5;
            "조용하다. 신이 아직 관심을 갖지 않는다.".to_string()
        }
        DivineFavor::Indifferent | DivineFavor::Pleased => {
            state.favor += 10;
            if player_hp_ratio < 0.2 {
                format!("{}의 자비로운 손길이 닿는다! HP 완전 회복!", state.god_name)
            } else {
                format!("{}의 축복이 느껴진다.", state.god_name)
            }
        }
        DivineFavor::Devout => {
            state.gifts_received += 1;
            state.favor += 15;
            format!(
                "{}이(가) 선물을 내려주었다! (선물 #{})",
                state.god_name, state.gifts_received
            )
        }
    }
}

/// [v2.40.0 104-1] 제물 바치기
pub fn make_offering(
    state: &mut ReligionState,
    item_value: i32,
    on_correct_altar: bool,
    rng: &mut NetHackRng,
) -> String {
    if !on_correct_altar {
        state.favor -= 5;
        return "잘못된 제단이다! 신이 불쾌해한다.".to_string();
    }

    let favor_gain = item_value / 10 + rng.rn2(5);
    state.favor += favor_gain;
    state.sins = (state.sins - 1).max(0);

    format!("제물이 빛 속으로 사라진다. 호감도 +{}", favor_gain)
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

    #[test]
    fn test_patron_god() {
        let god = get_patron_god("전사", Alignment::Chaotic);
        assert!(!god.is_empty());
    }

    #[test]
    fn test_favor_judgment() {
        assert_eq!(judge_favor(-150), DivineFavor::Wrathful);
        assert_eq!(judge_favor(0), DivineFavor::Indifferent);
        assert_eq!(judge_favor(100), DivineFavor::Pleased);
        assert_eq!(judge_favor(300), DivineFavor::Devout);
    }

    #[test]
    fn test_prayer_cooldown() {
        let mut state = ReligionState {
            alignment: Alignment::Lawful,
            god_name: "아누".to_string(),
            favor: 50,
            prayer_cooldown: 300,
            sins: 0,
            gifts_received: 0,
            last_prayer_turn: 0,
        };
        let mut rng = test_rng();
        pray(&mut state, 100, 1.0, &mut rng); // 쿨다운 위반
        assert!(state.favor < 50);
    }

    #[test]
    fn test_prayer_success() {
        let mut state = ReligionState {
            alignment: Alignment::Lawful,
            god_name: "아누".to_string(),
            favor: 100,
            prayer_cooldown: 300,
            sins: 0,
            gifts_received: 0,
            last_prayer_turn: 0,
        };
        let mut rng = test_rng();
        let msg = pray(&mut state, 500, 0.1, &mut rng);
        assert!(msg.contains("회복") || msg.contains("축복"));
    }

    #[test]
    fn test_offering() {
        let mut state = ReligionState {
            alignment: Alignment::Neutral,
            god_name: "아누비스".to_string(),
            favor: 50,
            prayer_cooldown: 300,
            sins: 2,
            gifts_received: 0,
            last_prayer_turn: 0,
        };
        let mut rng = test_rng();
        let msg = make_offering(&mut state, 500, true, &mut rng);
        assert!(msg.contains("호감도"));
        assert!(state.sins < 2);
    }

    #[test]
    fn test_wrong_altar() {
        let mut state = ReligionState {
            alignment: Alignment::Chaotic,
            god_name: "세트".to_string(),
            favor: 50,
            prayer_cooldown: 300,
            sins: 0,
            gifts_received: 0,
            last_prayer_turn: 0,
        };
        let mut rng = test_rng();
        let msg = make_offering(&mut state, 100, false, &mut rng);
        assert!(msg.contains("잘못된"));
    }

    #[test]
    fn test_devout_gift() {
        let mut state = ReligionState {
            alignment: Alignment::Lawful,
            god_name: "아누".to_string(),
            favor: 250,
            prayer_cooldown: 300,
            sins: 0,
            gifts_received: 0,
            last_prayer_turn: 0,
        };
        let mut rng = test_rng();
        let msg = pray(&mut state, 500, 1.0, &mut rng);
        assert!(msg.contains("선물"));
    }
}

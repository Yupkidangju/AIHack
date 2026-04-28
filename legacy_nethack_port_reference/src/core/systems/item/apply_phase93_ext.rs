// ============================================================================
// [v2.29.0 Phase 93-4] 아이템 사용 확장 (apply_phase93_ext.rs)
// 원본: NetHack 3.6.7 src/apply.c L400-1500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 도구 사용 — use_tool (apply.c L400-800)
// =============================================================================

/// [v2.29.0 93-4] 도구 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    Pickaxe,
    Lamp,
    MagicLamp,
    Horn,
    Drum,
    Whistle,
    MagicWhistle,
    Mirror,
    Camera,
    Tinning,
    Leash,
    Saddle,
    StethoscopeItem,
    CrystalBall,
    FigurinItem,
    Candelabrum,
    Bell,
    BagOfHolding,
    BagOfTricks,
    SkeletonKey,
    LockPick,
    CreditCard,
    TinOpener,
    Blindfold,
    Towel,
    Unicorn,
}

/// [v2.29.0 93-4] 도구 사용 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolUseResult {
    /// 채굴 시작
    Dig { direction: String },
    /// 빛 생성
    Light { radius: i32, turns: i32 },
    /// 소환/제니 (마법 램프)
    GenieAppears { wishes: i32 },
    /// 소리 효과 (혼/북)
    SoundEffect { effect: String, radius: i32 },
    /// 반사 (거울)
    Reflect { target: String },
    /// 사진 (카메라)
    Photo { blinding: bool, target: String },
    /// 상태 확인 (청진기)
    Stethoscope { info: String },
    /// 점술 (수정구)
    CrystalBall { vision: String },
    /// 가방 저장/꺼내기
    BagAction { item_count: i32 },
    /// 트릭 가방
    BagOfTricksSpawn { monster: String },
    /// 자물쇠 따기
    LockPicked,
    /// 장비 착용
    Equip { slot: String },
    /// 휘파람 (길들이기)
    Whistle { tamed: i32 },
    /// 효과 없음
    NoEffect { message: String },
    /// 충전 소진
    Exhausted,
}

/// [v2.29.0 93-4] 도구 사용 판정
/// 원본: apply.c doapply()
pub fn use_tool(
    tool: ToolType,
    charges: i32,
    is_blessed: bool,
    is_cursed: bool,
    player_level: i32,
    rng: &mut NetHackRng,
) -> ToolUseResult {
    // 충전 확인 (해당시)
    match tool {
        ToolType::Lamp
        | ToolType::MagicLamp
        | ToolType::Horn
        | ToolType::Drum
        | ToolType::Camera
        | ToolType::CrystalBall
        | ToolType::BagOfTricks => {
            if charges <= 0 {
                return ToolUseResult::Exhausted;
            }
        }
        _ => {}
    }

    match tool {
        ToolType::Pickaxe => ToolUseResult::Dig {
            direction: "앞".to_string(),
        },
        ToolType::Lamp => {
            let turns = if is_blessed { charges * 2 } else { charges };
            ToolUseResult::Light { radius: 5, turns }
        }
        ToolType::MagicLamp => {
            if rng.rn2(3) == 0 && !is_cursed {
                ToolUseResult::GenieAppears {
                    wishes: if is_blessed { 3 } else { 1 },
                }
            } else {
                ToolUseResult::Light {
                    radius: 7,
                    turns: -1,
                }
            }
        }
        ToolType::Horn => {
            let effect = if is_blessed {
                "공포 해제".to_string()
            } else {
                "소음".to_string()
            };
            ToolUseResult::SoundEffect { effect, radius: 10 }
        }
        ToolType::Drum => ToolUseResult::SoundEffect {
            effect: "전투 북소리".to_string(),
            radius: if is_blessed { 15 } else { 10 },
        },
        ToolType::Whistle => ToolUseResult::SoundEffect {
            effect: "날카로운 소리".to_string(),
            radius: 5,
        },
        ToolType::MagicWhistle => {
            ToolUseResult::Whistle {
                tamed: if is_blessed { -1 } else { rng.rn2(3) + 1 }, // -1 = 전체
            }
        }
        ToolType::Mirror => ToolUseResult::Reflect {
            target: "몬스터".to_string(),
        },
        ToolType::Camera => ToolUseResult::Photo {
            blinding: rng.rn2(2) == 0,
            target: "대상".to_string(),
        },
        ToolType::StethoscopeItem => ToolUseResult::Stethoscope {
            info: "HP/AC 정보 표시".to_string(),
        },
        ToolType::CrystalBall => {
            if rng.rn2(10) < player_level / 2 {
                ToolUseResult::CrystalBall {
                    vision: "미래의 단편이 보인다...".to_string(),
                }
            } else {
                ToolUseResult::NoEffect {
                    message: "구슬이 뿌옇게 흐려졌다.".to_string(),
                }
            }
        }
        ToolType::BagOfHolding => ToolUseResult::BagAction { item_count: 0 },
        ToolType::BagOfTricks => {
            let monsters = ["코볼트", "고블린", "개", "뱀", "쥐"];
            let idx = rng.rn2(monsters.len() as i32) as usize;
            ToolUseResult::BagOfTricksSpawn {
                monster: monsters[idx].to_string(),
            }
        }
        ToolType::SkeletonKey | ToolType::LockPick | ToolType::CreditCard => {
            ToolUseResult::LockPicked
        }
        ToolType::Saddle | ToolType::Leash => ToolUseResult::Equip {
            slot: "탈것".to_string(),
        },
        ToolType::Blindfold | ToolType::Towel => ToolUseResult::Equip {
            slot: "눈".to_string(),
        },
        _ => ToolUseResult::NoEffect {
            message: "특별한 효과가 없다.".to_string(),
        },
    }
}

// =============================================================================
// [2] 아이템 충전 — recharge (apply.c L800-1000)
// =============================================================================

/// [v2.29.0 93-4] 충전 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RechargeResult {
    /// 충전 성공
    Recharged { new_charges: i32 },
    /// 과충전 폭발
    Exploded { damage: i32 },
    /// 축복 충전 (2배)
    BlessedRecharge { new_charges: i32 },
    /// 충전 불가
    CannotRecharge,
}

/// [v2.29.0 93-4] 아이템 충전 판정
pub fn recharge_item(
    current_charges: i32,
    max_charges: i32,
    recharge_count: i32, // 이전 충전 횟수
    is_blessed: bool,
    rng: &mut NetHackRng,
) -> RechargeResult {
    // 과충전 위험: 충전 횟수에 비례
    if recharge_count > 0 && rng.rn2(recharge_count + 1) > 0 {
        return RechargeResult::Exploded {
            damage: rng.rn2(current_charges * 2 + 5) + 1,
        };
    }

    let add = if is_blessed {
        rng.rn2(max_charges / 2 + 1) + max_charges / 2
    } else {
        rng.rn2(max_charges / 2 + 1) + 1
    };

    let new_charges = (current_charges + add).min(max_charges);

    if is_blessed {
        RechargeResult::BlessedRecharge { new_charges }
    } else {
        RechargeResult::Recharged { new_charges }
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

    #[test]
    fn test_pickaxe() {
        let mut rng = test_rng();
        let result = use_tool(ToolType::Pickaxe, 0, false, false, 10, &mut rng);
        assert!(matches!(result, ToolUseResult::Dig { .. }));
    }

    #[test]
    fn test_lamp_blessed() {
        let mut rng = test_rng();
        let result = use_tool(ToolType::Lamp, 100, true, false, 10, &mut rng);
        if let ToolUseResult::Light { turns, .. } = result {
            assert_eq!(turns, 200);
        }
    }

    #[test]
    fn test_lamp_exhausted() {
        let mut rng = test_rng();
        let result = use_tool(ToolType::Lamp, 0, false, false, 10, &mut rng);
        assert!(matches!(result, ToolUseResult::Exhausted));
    }

    #[test]
    fn test_magic_lamp_genie() {
        let mut got_genie = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = use_tool(ToolType::MagicLamp, 1, true, false, 10, &mut rng);
            if matches!(result, ToolUseResult::GenieAppears { .. }) {
                got_genie = true;
                break;
            }
        }
        assert!(got_genie);
    }

    #[test]
    fn test_bag_of_tricks() {
        let mut rng = test_rng();
        let result = use_tool(ToolType::BagOfTricks, 5, false, false, 10, &mut rng);
        assert!(matches!(result, ToolUseResult::BagOfTricksSpawn { .. }));
    }

    #[test]
    fn test_lockpick() {
        let mut rng = test_rng();
        let result = use_tool(ToolType::LockPick, 0, false, false, 10, &mut rng);
        assert!(matches!(result, ToolUseResult::LockPicked));
    }

    #[test]
    fn test_recharge_first() {
        let mut rng = test_rng();
        let result = recharge_item(3, 15, 0, false, &mut rng);
        assert!(matches!(result, RechargeResult::Recharged { .. }));
    }

    #[test]
    fn test_recharge_blessed() {
        let mut rng = test_rng();
        let result = recharge_item(3, 15, 0, true, &mut rng);
        assert!(matches!(result, RechargeResult::BlessedRecharge { .. }));
    }
}

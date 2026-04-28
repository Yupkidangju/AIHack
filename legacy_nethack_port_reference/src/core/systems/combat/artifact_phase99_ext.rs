// ============================================================================
// [v2.35.0 Phase 99-5] 아티팩트 통합 확장 (artifact_phase99_ext.rs)
// 원본: NetHack 3.6.7 src/artifact.c L500-1500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 아티팩트 데이터베이스 — artifact_db (artifact.c L500-1000)
// =============================================================================

/// [v2.35.0 99-5] 아티팩트 정보
#[derive(Debug, Clone)]
pub struct ArtifactInfo {
    pub name: String,
    pub base_item: String,
    pub alignment: String,
    pub role_restriction: Option<String>,
    pub attack_bonus: i32,
    pub damage_bonus: i32,
    pub special_attack: Option<String>,
    pub special_defense: Option<String>,
    pub granted_abilities: Vec<String>,
    pub invoke_effect: Option<String>,
    pub is_quest_artifact: bool,
}

/// [v2.35.0 99-5] 전체 아티팩트 목록
pub fn get_artifact_list() -> Vec<ArtifactInfo> {
    vec![
        ArtifactInfo {
            name: "Excalibur".to_string(),
            base_item: "장검".to_string(),
            alignment: "합법".to_string(),
            role_restriction: Some("기사".to_string()),
            attack_bonus: 5,
            damage_bonus: 10,
            special_attack: Some("소멸 (언데드/악마)".to_string()),
            special_defense: Some("마법 저항".to_string()),
            granted_abilities: vec!["자동 탐색".to_string(), "레벨업 텔레파시".to_string()],
            invoke_effect: None,
            is_quest_artifact: false,
        },
        ArtifactInfo {
            name: "Mjollnir".to_string(),
            base_item: "전쟁 해머".to_string(),
            alignment: "중립".to_string(),
            role_restriction: Some("발키리".to_string()),
            attack_bonus: 5,
            damage_bonus: 24,
            special_attack: Some("전기 (d24)".to_string()),
            special_defense: None,
            granted_abilities: vec!["원거리 회수".to_string()],
            invoke_effect: None,
            is_quest_artifact: false,
        },
        ArtifactInfo {
            name: "Stormbringer".to_string(),
            base_item: "넓은 검".to_string(),
            alignment: "혼돈".to_string(),
            role_restriction: None,
            attack_bonus: 5,
            damage_bonus: 8,
            special_attack: Some("흡혈".to_string()),
            special_defense: None,
            granted_abilities: vec!["레벨 흡수".to_string()],
            invoke_effect: None,
            is_quest_artifact: false,
        },
        ArtifactInfo {
            name: "Grayswandir".to_string(),
            base_item: "은검".to_string(),
            alignment: "합법".to_string(),
            role_restriction: None,
            attack_bonus: 5,
            damage_bonus: 8,
            special_attack: Some("은 (2배 데미지)".to_string()),
            special_defense: Some("환각 면역".to_string()),
            granted_abilities: vec![],
            invoke_effect: None,
            is_quest_artifact: false,
        },
        ArtifactInfo {
            name: "Magicbane".to_string(),
            base_item: "아셈".to_string(),
            alignment: "중립".to_string(),
            role_restriction: Some("마법사".to_string()),
            attack_bonus: 3,
            damage_bonus: 6,
            special_attack: Some("취소/저주/비틀거림/탐색".to_string()),
            special_defense: Some("마법 저항".to_string()),
            granted_abilities: vec!["마법 흡수".to_string()],
            invoke_effect: None,
            is_quest_artifact: false,
        },
        ArtifactInfo {
            name: "Sting".to_string(),
            base_item: "엘프 단검".to_string(),
            alignment: "혼돈".to_string(),
            role_restriction: None,
            attack_bonus: 5,
            damage_bonus: 5,
            special_attack: Some("오크 특효 (2배)".to_string()),
            special_defense: None,
            granted_abilities: vec!["오크 감지 (빛남)".to_string()],
            invoke_effect: None,
            is_quest_artifact: false,
        },
        ArtifactInfo {
            name: "Frost Brand".to_string(),
            base_item: "장검".to_string(),
            alignment: "중립".to_string(),
            role_restriction: None,
            attack_bonus: 3,
            damage_bonus: 6,
            special_attack: Some("냉기 추가 데미지".to_string()),
            special_defense: Some("냉기 저항".to_string()),
            granted_abilities: vec![],
            invoke_effect: None,
            is_quest_artifact: false,
        },
        ArtifactInfo {
            name: "Fire Brand".to_string(),
            base_item: "장검".to_string(),
            alignment: "중립".to_string(),
            role_restriction: None,
            attack_bonus: 3,
            damage_bonus: 6,
            special_attack: Some("화염 추가 데미지".to_string()),
            special_defense: Some("화염 저항".to_string()),
            granted_abilities: vec![],
            invoke_effect: None,
            is_quest_artifact: false,
        },
    ]
}

// =============================================================================
// [2] 아티팩트 발동 — artifact_trigger (artifact.c L1000-1500)
// =============================================================================

/// [v2.35.0 99-5] 아티팩트 공격 발동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactAttackResult {
    SpecialDamage { bonus: i32, effect: String },
    Drain { hp_drained: i32 },
    Cancel,
    Stagger { turns: i32 },
    NoSpecial,
}

/// [v2.35.0 99-5] 아티팩트 특수 공격 판정
pub fn artifact_attack_check(
    artifact_name: &str,
    target_type: &str, // "언데드", "악마", "오크" 등
    rng: &mut NetHackRng,
) -> ArtifactAttackResult {
    match artifact_name {
        "Excalibur" => {
            if target_type == "언데드" || target_type == "악마" {
                ArtifactAttackResult::SpecialDamage {
                    bonus: rng.rn2(10) + 10,
                    effect: "소멸의 빛".to_string(),
                }
            } else {
                ArtifactAttackResult::SpecialDamage {
                    bonus: 5,
                    effect: "성스러운 일격".to_string(),
                }
            }
        }
        "Mjollnir" => ArtifactAttackResult::SpecialDamage {
            bonus: rng.rn2(24) + 1,
            effect: "전기 폭풍".to_string(),
        },
        "Stormbringer" => ArtifactAttackResult::Drain {
            hp_drained: rng.rn2(8) + 1,
        },
        "Magicbane" => {
            let roll = rng.rn2(4);
            match roll {
                0 => ArtifactAttackResult::Cancel,
                1 => ArtifactAttackResult::Stagger {
                    turns: rng.rn2(3) + 1,
                },
                _ => ArtifactAttackResult::SpecialDamage {
                    bonus: rng.rn2(6) + 1,
                    effect: "마법 에너지".to_string(),
                },
            }
        }
        "Sting" => {
            if target_type == "오크" {
                ArtifactAttackResult::SpecialDamage {
                    bonus: rng.rn2(10) + 5,
                    effect: "오크 특효".to_string(),
                }
            } else {
                ArtifactAttackResult::NoSpecial
            }
        }
        _ => ArtifactAttackResult::NoSpecial,
    }
}

/// [v2.35.0 99-5] 아티팩트 소원 가능 여부
pub fn can_wish_for_artifact(
    artifact_name: &str,
    player_role: &str,
    already_exists: &[String],
) -> bool {
    let artifacts = get_artifact_list();
    let info = artifacts.iter().find(|a| a.name == artifact_name);
    match info {
        None => false,
        Some(a) => {
            // 이미 존재하면 불가
            if already_exists.iter().any(|e| e == artifact_name) {
                return false;
            }
            // 역할 제한 확인
            if let Some(ref role) = a.role_restriction {
                if role != player_role {
                    return false;
                }
            }
            true
        }
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
    fn test_artifact_list() {
        let list = get_artifact_list();
        assert!(list.len() >= 8);
        assert!(list.iter().any(|a| a.name == "Excalibur"));
    }

    #[test]
    fn test_excalibur_undead() {
        let mut rng = test_rng();
        let result = artifact_attack_check("Excalibur", "언데드", &mut rng);
        if let ArtifactAttackResult::SpecialDamage { bonus, .. } = result {
            assert!(bonus >= 10);
        }
    }

    #[test]
    fn test_mjollnir() {
        let mut rng = test_rng();
        let result = artifact_attack_check("Mjollnir", "아무거나", &mut rng);
        assert!(matches!(result, ArtifactAttackResult::SpecialDamage { .. }));
    }

    #[test]
    fn test_stormbringer_drain() {
        let mut rng = test_rng();
        let result = artifact_attack_check("Stormbringer", "아무거나", &mut rng);
        assert!(matches!(result, ArtifactAttackResult::Drain { .. }));
    }

    #[test]
    fn test_sting_orc() {
        let mut rng = test_rng();
        let result = artifact_attack_check("Sting", "오크", &mut rng);
        assert!(matches!(result, ArtifactAttackResult::SpecialDamage { .. }));
    }

    #[test]
    fn test_sting_non_orc() {
        let mut rng = test_rng();
        let result = artifact_attack_check("Sting", "인간", &mut rng);
        assert!(matches!(result, ArtifactAttackResult::NoSpecial));
    }

    #[test]
    fn test_wish_ok() {
        let ok = can_wish_for_artifact("Frost Brand", "전사", &[]);
        assert!(ok);
    }

    #[test]
    fn test_wish_role_restrict() {
        let ok = can_wish_for_artifact("Excalibur", "마법사", &[]);
        assert!(!ok);
    }

    #[test]
    fn test_wish_already_exists() {
        let ok = can_wish_for_artifact("Frost Brand", "전사", &["Frost Brand".to_string()]);
        assert!(!ok);
    }
}

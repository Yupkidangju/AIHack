// ============================================================================
// [v2.28.0 R16-4] 아티팩트 전투 확장 (artifact_combat_ext.rs)
// 원본: NetHack 3.6.7 artifact.c 전투 로직 (touch_artifact, spec_dbon)
// 아티팩트 특수 공격, 저주/축복 효과, 자격 검증
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.28.0 R16-4] 아티팩트 특수 공격 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactAttack {
    /// 추가 데미지 (특정 대상)
    BonusDamage { amount: i32, target_type: String },
    /// 레벨 드레인
    LevelDrain,
    /// 이중 데미지
    DoubleDamage,
    /// 석화
    Petrify,
    /// 독
    Poison { damage: i32 },
    /// 슬로우
    Slow,
    /// 쳐들기 금지 (자살 방지)
    SelfDestruct(String),
    /// 효과 없음 (대상 면역)
    NoEffect,
}

/// [v2.28.0 R16-4] 아티팩트 데이터
#[derive(Debug, Clone)]
pub struct ArtifactData {
    pub name: String,
    pub base_damage_bonus: i32,
    pub vs_undead: bool,
    pub vs_demon: bool,
    pub vs_dragon: bool,
    pub alignment_required: Option<i32>, // -1 chaotic, 0 neutral, 1 lawful
    pub role_required: Option<String>,
    pub intelligent: bool,    // 자아 있는 무기
    pub blast_on_touch: bool, // 자격 없으면 피해
}

/// [v2.28.0 R16-4] 아티팩트 터치 판정 (원본: touch_artifact)
pub fn touch_artifact(
    artifact: &ArtifactData,
    player_alignment: i32,
    player_role: &str,
    rng: &mut NetHackRng,
) -> Result<(), i32> {
    // 정렬 불일치
    if let Some(req) = artifact.alignment_required {
        if req != player_alignment && artifact.blast_on_touch {
            let damage = rng.rn1(20, 1);
            return Err(damage);
        }
    }
    // 역할 불일치 (지능형 무기)
    if let Some(ref role) = artifact.role_required {
        if role != player_role && artifact.intelligent {
            let damage = rng.rn1(10, 1);
            return Err(damage);
        }
    }
    Ok(())
}

/// [v2.28.0 R16-4] 아티팩트 특수 데미지 (원본: spec_dbon)
pub fn artifact_bonus_damage(
    artifact: &ArtifactData,
    target_is_undead: bool,
    target_is_demon: bool,
    target_is_dragon: bool,
    rng: &mut NetHackRng,
) -> ArtifactAttack {
    let mut bonus = artifact.base_damage_bonus;

    if artifact.vs_undead && target_is_undead {
        bonus += rng.rn1(20, 1);
        return ArtifactAttack::BonusDamage {
            amount: bonus,
            target_type: "undead".to_string(),
        };
    }
    if artifact.vs_demon && target_is_demon {
        bonus += rng.rn1(20, 1);
        return ArtifactAttack::BonusDamage {
            amount: bonus,
            target_type: "demon".to_string(),
        };
    }
    if artifact.vs_dragon && target_is_dragon {
        return ArtifactAttack::DoubleDamage;
    }

    if bonus > 0 {
        ArtifactAttack::BonusDamage {
            amount: bonus,
            target_type: "any".to_string(),
        }
    } else {
        ArtifactAttack::NoEffect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_artifact() -> ArtifactData {
        ArtifactData {
            name: "Excalibur".to_string(),
            base_damage_bonus: 5,
            vs_undead: true,
            vs_demon: true,
            vs_dragon: false,
            alignment_required: Some(1), // lawful
            role_required: Some("Knight".to_string()),
            intelligent: true,
            blast_on_touch: true,
        }
    }

    #[test]
    fn test_touch_ok() {
        let mut rng = NetHackRng::new(42);
        assert!(touch_artifact(&test_artifact(), 1, "Knight", &mut rng).is_ok());
    }

    #[test]
    fn test_touch_wrong_alignment() {
        let mut rng = NetHackRng::new(42);
        let result = touch_artifact(&test_artifact(), -1, "Knight", &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_bonus_undead() {
        let mut rng = NetHackRng::new(42);
        let result = artifact_bonus_damage(&test_artifact(), true, false, false, &mut rng);
        assert!(
            matches!(result, ArtifactAttack::BonusDamage { target_type, .. } if target_type == "undead")
        );
    }

    #[test]
    fn test_bonus_demon() {
        let mut rng = NetHackRng::new(42);
        let result = artifact_bonus_damage(&test_artifact(), false, true, false, &mut rng);
        assert!(
            matches!(result, ArtifactAttack::BonusDamage { target_type, .. } if target_type == "demon")
        );
    }

    #[test]
    fn test_no_bonus() {
        let mut rng = NetHackRng::new(42);
        let art = ArtifactData {
            base_damage_bonus: 0,
            vs_undead: false,
            vs_demon: false,
            vs_dragon: false,
            ..test_artifact()
        };
        let result = artifact_bonus_damage(&art, false, false, false, &mut rng);
        assert_eq!(result, ArtifactAttack::NoEffect);
    }
}

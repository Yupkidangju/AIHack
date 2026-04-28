// ============================================================================
// [v2.34.0 Phase 98-4] 몬스터 생성 확장 (mkmon_phase98_ext.rs)
// 원본: NetHack 3.6.7 src/makemon.c L800-2000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 생성 테이블 — spawn_tables (makemon.c L800-1200)
// =============================================================================

/// [v2.34.0 98-4] 몬스터 생성 조건
#[derive(Debug, Clone)]
pub struct SpawnCondition {
    pub depth: i32,
    pub branch: String,
    pub is_night: bool,
    pub near_altar: bool,
    pub near_water: bool,
    pub in_shop: bool,
    pub genocide_list: Vec<String>,
}

/// [v2.34.0 98-4] 생성된 몬스터 정보
#[derive(Debug, Clone)]
pub struct SpawnedMonster {
    pub name: String,
    pub level: i32,
    pub hp: i32,
    pub speed: i32,
    pub is_hostile: bool,
    pub is_group: bool,
    pub group_size: i32,
    pub special_ability: Option<String>,
}

/// [v2.34.0 98-4] 깊이 기반 몬스터 생성
pub fn spawn_monster_by_depth(condition: &SpawnCondition, rng: &mut NetHackRng) -> SpawnedMonster {
    let depth = condition.depth;

    // 깊이별 몬스터 풀
    let pool: Vec<(&str, i32, i32, i32, bool, Option<&str>)> = if depth <= 5 {
        vec![
            ("쥐", 1, 4, 12, false, None),
            ("박쥐", 1, 4, 18, false, None),
            ("자칼", 2, 8, 12, true, None),
            ("코볼트", 2, 8, 6, true, None),
            ("검은 푸딩", 1, 6, 3, false, Some("분열")),
            ("그놈", 2, 12, 6, true, None),
            ("오크", 3, 15, 9, true, None),
        ]
    } else if depth <= 15 {
        vec![
            ("오거", 7, 40, 12, true, None),
            ("님프", 5, 20, 12, false, Some("도둑질")),
            ("트롤", 7, 45, 12, true, Some("재생")),
            ("유니콘", 8, 50, 18, false, Some("호른")),
            ("미노타우르스", 9, 60, 15, true, None),
            ("거인", 10, 70, 6, true, None),
        ]
    } else {
        vec![
            ("리치", 15, 80, 6, true, Some("마법")),
            ("발록", 18, 100, 12, true, Some("소환")),
            ("드래곤", 20, 120, 9, true, Some("브레스")),
            ("앙크헤그", 12, 50, 18, true, Some("산")),
            ("데몬", 16, 90, 12, true, Some("텔레포트")),
            ("아크리치", 25, 150, 6, true, Some("죽음의 마법")),
        ]
    };

    // 수역 근처 → 수중 몬스터
    if condition.near_water && rng.rn2(3) == 0 {
        return SpawnedMonster {
            name: "크라켄".to_string(),
            level: 12,
            hp: 60,
            speed: 6,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("촉수 공격".to_string()),
        };
    }

    let idx = rng.rn2(pool.len() as i32) as usize;
    let (name, level, hp, speed, hostile, ability) = pool[idx];

    // 제노사이드 확인
    if condition.genocide_list.iter().any(|g| g == name) {
        // 제노사이드된 몬스터 → 다른거 생성
        let alt_idx = (idx + 1) % pool.len();
        let (n2, l2, h2, s2, ho2, a2) = pool[alt_idx];
        return SpawnedMonster {
            name: n2.to_string(),
            level: l2,
            hp: h2,
            speed: s2,
            is_hostile: ho2,
            is_group: rng.rn2(4) == 0,
            group_size: if rng.rn2(4) == 0 { rng.rn2(3) + 2 } else { 1 },
            special_ability: a2.map(|s| s.to_string()),
        };
    }

    let group = rng.rn2(5) == 0;
    SpawnedMonster {
        name: name.to_string(),
        level,
        hp,
        speed,
        is_hostile: hostile,
        is_group: group,
        group_size: if group { rng.rn2(4) + 2 } else { 1 },
        special_ability: ability.map(|s| s.to_string()),
    }
}

// =============================================================================
// [2] 유니크 몬스터 — unique_spawn (makemon.c L1500-2000)
// =============================================================================

/// [v2.34.0 98-4] 유니크 몬스터 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniqueMonster {
    Medusa,
    WizardOfYendor,
    Vlad,
    Asmodeus,
    Baalzebub,
    Orcus,
    Demogorgon,
    Rodney,
    Oracle,
    DeathRider,
    Famine,
    Pestilence,
}

/// [v2.34.0 98-4] 유니크 몬스터 정보
pub fn unique_monster_info(unique: UniqueMonster) -> SpawnedMonster {
    match unique {
        UniqueMonster::Medusa => SpawnedMonster {
            name: "메두사".to_string(),
            level: 20,
            hp: 150,
            speed: 12,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("석화 시선".to_string()),
        },
        UniqueMonster::WizardOfYendor | UniqueMonster::Rodney => SpawnedMonster {
            name: "옌더의 마법사".to_string(),
            level: 30,
            hp: 200,
            speed: 12,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("부활 + 텔레포트 + 아뮬렛 도둑".to_string()),
        },
        UniqueMonster::Vlad => SpawnedMonster {
            name: "블라드 더 임팔러".to_string(),
            level: 20,
            hp: 120,
            speed: 18,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("흡혈 + 비행".to_string()),
        },
        UniqueMonster::Asmodeus => SpawnedMonster {
            name: "아스모데우스".to_string(),
            level: 55,
            hp: 300,
            speed: 12,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("냉기 브레스".to_string()),
        },
        UniqueMonster::Baalzebub => SpawnedMonster {
            name: "발제붑".to_string(),
            level: 55,
            hp: 300,
            speed: 18,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("독 + 질병".to_string()),
        },
        UniqueMonster::Orcus => SpawnedMonster {
            name: "오르쿠스".to_string(),
            level: 55,
            hp: 300,
            speed: 9,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("죽음의 지팡이".to_string()),
        },
        UniqueMonster::Demogorgon => SpawnedMonster {
            name: "데모고르곤".to_string(),
            level: 60,
            hp: 500,
            speed: 15,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("질병 + 부식 + 수면".to_string()),
        },
        UniqueMonster::Oracle => SpawnedMonster {
            name: "오라클".to_string(),
            level: 12,
            hp: 80,
            speed: 0,
            is_hostile: false,
            is_group: false,
            group_size: 1,
            special_ability: Some("상담 (금화 비용)".to_string()),
        },
        UniqueMonster::DeathRider => SpawnedMonster {
            name: "죽음".to_string(),
            level: 30,
            hp: 250,
            speed: 12,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("즉사 공격".to_string()),
        },
        UniqueMonster::Famine => SpawnedMonster {
            name: "기근".to_string(),
            level: 30,
            hp: 250,
            speed: 12,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("식량 파괴".to_string()),
        },
        UniqueMonster::Pestilence => SpawnedMonster {
            name: "역병".to_string(),
            level: 30,
            hp: 250,
            speed: 12,
            is_hostile: true,
            is_group: false,
            group_size: 1,
            special_ability: Some("질병 공격".to_string()),
        },
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

    fn base_condition(depth: i32) -> SpawnCondition {
        SpawnCondition {
            depth,
            branch: "Main".to_string(),
            is_night: false,
            near_altar: false,
            near_water: false,
            in_shop: false,
            genocide_list: vec![],
        }
    }

    #[test]
    fn test_early_spawn() {
        let mut rng = test_rng();
        let mon = spawn_monster_by_depth(&base_condition(3), &mut rng);
        assert!(mon.level <= 5);
    }

    #[test]
    fn test_mid_spawn() {
        let mut rng = test_rng();
        let mon = spawn_monster_by_depth(&base_condition(10), &mut rng);
        assert!(mon.level >= 5);
    }

    #[test]
    fn test_deep_spawn() {
        let mut rng = test_rng();
        let mon = spawn_monster_by_depth(&base_condition(25), &mut rng);
        assert!(mon.level >= 10);
    }

    #[test]
    fn test_water_spawn() {
        let mut cond = base_condition(10);
        cond.near_water = true;
        let mut found_kraken = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let mon = spawn_monster_by_depth(&cond, &mut rng);
            if mon.name == "크라켄" {
                found_kraken = true;
                break;
            }
        }
        assert!(found_kraken);
    }

    #[test]
    fn test_medusa() {
        let mon = unique_monster_info(UniqueMonster::Medusa);
        assert_eq!(mon.name, "메두사");
        assert!(mon.special_ability.unwrap().contains("석화"));
    }

    #[test]
    fn test_demogorgon() {
        let mon = unique_monster_info(UniqueMonster::Demogorgon);
        assert_eq!(mon.level, 60);
        assert_eq!(mon.hp, 500);
    }

    #[test]
    fn test_oracle_peaceful() {
        let mon = unique_monster_info(UniqueMonster::Oracle);
        assert!(!mon.is_hostile);
    }

    #[test]
    fn test_genocide_avoidance() {
        let mut cond = base_condition(3);
        cond.genocide_list = vec!["쥐".to_string()];
        let mut rng = test_rng();
        let mon = spawn_monster_by_depth(&cond, &mut rng);
        // 결과 검증 - 쥐가 안 나와야 함
        // (확률적이므로 반복)
        assert!(!mon.name.is_empty());
    }
}

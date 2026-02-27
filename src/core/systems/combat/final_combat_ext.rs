// ============================================================================
// [v2.41.0 Phase FINAL-2] 잔여 완전 통합 (final_combat_ext.rs)
// 원본: NetHack 3.6.7 잔여 전투/아이템/UI 로직 최종 마감
// 순수 결과 패턴
//
// 구현 범위:
//   - 원거리 전투 세부 계산 (투사체 궤적)
//   - 방어구 부위별 방어 (8부위)
//   - 치명타/급소 시스템
//   - 무기 내구도/파손
//   - 전투 로그 포매터
//   - 아이템 강화/인챈트 세부
//   - UI 전투 애니메이션 이벤트
//   - 최종 전투 통계 집계
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 투사체 궤적 — projectile_trajectory
// =============================================================================

/// [v2.41.0 FINAL-2] 투사체 궤적 포인트
#[derive(Debug, Clone)]
pub struct TrajectoryPoint {
    pub x: i32,
    pub y: i32,
    pub turn_fraction: f64, // 0.0~1.0 이동 진행도
}

/// [v2.41.0 FINAL-2] 투사체 궤적 계산 (Bresenham 기반)
pub fn calculate_trajectory(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    max_range: i32,
) -> Vec<TrajectoryPoint> {
    let mut points = Vec::new();
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut cx = x1;
    let mut cy = y1;
    let total_dist = ((dx * dx + dy * dy) as f64).sqrt().max(1.0);
    let mut steps = 0;

    loop {
        let dist = (((cx - x1) * (cx - x1) + (cy - y1) * (cy - y1)) as f64).sqrt();
        points.push(TrajectoryPoint {
            x: cx,
            y: cy,
            turn_fraction: dist / total_dist,
        });

        if (cx == x2 && cy == y2) || steps >= max_range {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
        steps += 1;
    }

    points
}

// =============================================================================
// [2] 방어구 부위별 방어 — armor_coverage
// =============================================================================

/// [v2.41.0 FINAL-2] 방어구 부위
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlot {
    Head,   // 투구
    Body,   // 갑옷
    Cloak,  // 망토
    Shield, // 방패
    Gloves, // 장갑
    Boots,  // 부츠
    Shirt,  // 셔츠
    Amulet, // 아뮬렛
}

/// [v2.41.0 FINAL-2] 부위별 방어력
pub fn armor_coverage(slot: ArmorSlot, base_ac: i32, enchantment: i32) -> (i32, f64) {
    let coverage = match slot {
        ArmorSlot::Body => 0.40,   // 40% 확률로 피격 부위
        ArmorSlot::Shield => 0.25, // 25%
        ArmorSlot::Cloak => 0.15,  // 15%
        ArmorSlot::Head => 0.10,   // 10%
        ArmorSlot::Boots => 0.05,  // 5%
        ArmorSlot::Gloves => 0.03, // 3%
        ArmorSlot::Shirt => 0.01,  // 1%
        ArmorSlot::Amulet => 0.01, // 1%
    };

    let total_ac = base_ac + enchantment;
    (total_ac, coverage)
}

// =============================================================================
// [3] 치명타/급소 — critical_hit
// =============================================================================

/// [v2.41.0 FINAL-2] 치명타 판정
pub fn check_critical_hit(
    attacker_level: i32,
    weapon_skill: i32, // 0~5
    rng: &mut NetHackRng,
) -> (bool, f64) {
    // 기본 5% + 레벨당 0.5% + 스킬당 2%
    let crit_chance = 5.0 + attacker_level as f64 * 0.5 + weapon_skill as f64 * 2.0;
    let crit_chance = crit_chance.min(25.0); // 최대 25%
    let roll = rng.rn2(100);

    if (roll as f64) < crit_chance {
        let multiplier = 1.5 + weapon_skill as f64 * 0.1;
        (true, multiplier)
    } else {
        (false, 1.0)
    }
}

// =============================================================================
// [4] 무기 내구도 — weapon_durability
// =============================================================================

/// [v2.41.0 FINAL-2] 무기 내구도
#[derive(Debug, Clone)]
pub struct WeaponDurability {
    pub name: String,
    pub durability: i32,     // 현재
    pub max_durability: i32, // 최대
    pub enchantment: i32,    // 강화 수치
}

impl WeaponDurability {
    pub fn new(name: &str, max_dur: i32, enchant: i32) -> Self {
        Self {
            name: name.to_string(),
            durability: max_dur,
            max_durability: max_dur,
            enchantment: enchant,
        }
    }

    /// 사용으로 인한 마모
    pub fn wear(&mut self, rng: &mut NetHackRng) -> Option<String> {
        // 강화 수치가 높을수록 마모 확률 낮음
        let wear_chance = 5 - self.enchantment.min(4);
        if rng.rn2(100) < wear_chance {
            self.durability -= 1;
            if self.durability <= 0 {
                return Some(format!("{}이(가) 부서졌다!", self.name));
            } else if self.durability <= self.max_durability / 5 {
                return Some(format!("{}이(가) 금이 가고 있다!", self.name));
            }
        }
        None
    }

    /// 수리
    pub fn repair(&mut self, amount: i32) -> String {
        let old = self.durability;
        self.durability = (self.durability + amount).min(self.max_durability);
        format!("{} 수리됨 ({} → {})", self.name, old, self.durability)
    }
}

// =============================================================================
// [5] 전투 로그 포매터 — combat_log
// =============================================================================

/// [v2.41.0 FINAL-2] 전투 로그 엔트리
#[derive(Debug, Clone)]
pub struct CombatLogEntry {
    pub turn: i32,
    pub attacker: String,
    pub defender: String,
    pub damage: i32,
    pub is_critical: bool,
    pub weapon_used: String,
    pub result: String, // "명중", "빗나감", "치명타", "사망"
}

/// [v2.41.0 FINAL-2] 전투 로그 포맷
pub fn format_combat_log(entry: &CombatLogEntry) -> String {
    let crit_marker = if entry.is_critical { "💥" } else { "" };
    format!(
        "[T{}] {} → {} ({}로) : {} {}{} dmg",
        entry.turn,
        entry.attacker,
        entry.defender,
        entry.weapon_used,
        entry.result,
        crit_marker,
        entry.damage
    )
}

// =============================================================================
// [6] 아이템 강화 — enchantment
// =============================================================================

/// [v2.41.0 FINAL-2] 강화 시도
pub fn attempt_enchant(
    current_enchant: i32,
    item_type: &str,
    rng: &mut NetHackRng,
) -> (i32, String) {
    let max_safe = match item_type {
        "무기" | "weapon" => 5,
        "갑옷" | "armor" => 3,
        _ => 5,
    };

    if current_enchant >= max_safe {
        // 초과 강화 시 파괴 위험
        let risk = (current_enchant - max_safe + 1) * 20;
        if rng.rn2(100) < risk {
            return (-1, "강화 실패! 장비가 증발했다!".to_string());
        }
    }

    let new_val = current_enchant + 1;
    (
        new_val,
        format!("강화 성공! +{} → +{}", current_enchant, new_val),
    )
}

// =============================================================================
// [7] 전투 통계 집계 — combat_stats
// =============================================================================

/// [v2.41.0 FINAL-2] 전투 통계
#[derive(Debug, Clone, Default)]
pub struct CombatStats {
    pub total_attacks: i32,
    pub hits: i32,
    pub misses: i32,
    pub criticals: i32,
    pub total_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub kills: i32,
    pub deaths: i32,
}

impl CombatStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_attacks == 0 {
            return 0.0;
        }
        self.hits as f64 / self.total_attacks as f64 * 100.0
    }

    pub fn crit_rate(&self) -> f64 {
        if self.hits == 0 {
            return 0.0;
        }
        self.criticals as f64 / self.hits as f64 * 100.0
    }

    pub fn summary(&self) -> String {
        format!(
            "전투 통계: {}회 공격 (명중률 {:.1}%, 치명타율 {:.1}%) | {} 킬 | 총 피해 {} / 피격 {}",
            self.total_attacks,
            self.hit_rate(),
            self.crit_rate(),
            self.kills,
            self.total_damage_dealt,
            self.total_damage_taken
        )
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
    fn test_trajectory() {
        let pts = calculate_trajectory(0, 0, 5, 5, 20);
        assert!(!pts.is_empty());
        assert_eq!(pts[0].x, 0);
        assert_eq!(pts[0].y, 0);
    }

    #[test]
    fn test_armor_coverage() {
        let (ac, cov) = armor_coverage(ArmorSlot::Body, 5, 2);
        assert_eq!(ac, 7);
        assert!(cov > 0.3);
    }

    #[test]
    fn test_critical_hit() {
        let mut rng = test_rng();
        // 여러번 시도하면 적어도 한번은 치명타
        let mut found_crit = false;
        for _ in 0..100 {
            let (crit, mult) = check_critical_hit(20, 5, &mut rng);
            if crit {
                found_crit = true;
                assert!(mult > 1.0);
            }
        }
        assert!(found_crit);
    }

    #[test]
    fn test_weapon_durability() {
        let mut w = WeaponDurability::new("장검", 100, 3);
        assert_eq!(w.durability, 100);
        let msg = w.repair(0);
        assert!(msg.contains("수리"));
    }

    #[test]
    fn test_combat_log() {
        let entry = CombatLogEntry {
            turn: 500,
            attacker: "용사".to_string(),
            defender: "오크".to_string(),
            damage: 15,
            is_critical: true,
            weapon_used: "장검 +3".to_string(),
            result: "치명타".to_string(),
        };
        let log = format_combat_log(&entry);
        assert!(log.contains("💥"));
        assert!(log.contains("용사"));
    }

    #[test]
    fn test_enchant_safe() {
        let mut rng = test_rng();
        let (val, msg) = attempt_enchant(2, "무기", &mut rng);
        assert_eq!(val, 3);
        assert!(msg.contains("성공"));
    }

    #[test]
    fn test_combat_stats() {
        let mut stats = CombatStats::default();
        stats.total_attacks = 100;
        stats.hits = 75;
        stats.criticals = 10;
        stats.kills = 20;
        assert!(stats.hit_rate() > 70.0);
        assert!(stats.crit_rate() > 10.0);
    }

    #[test]
    fn test_stats_summary() {
        let stats = CombatStats {
            total_attacks: 50,
            hits: 40,
            misses: 10,
            criticals: 5,
            total_damage_dealt: 1000,
            total_damage_taken: 200,
            kills: 15,
            deaths: 1,
        };
        let s = stats.summary();
        assert!(s.contains("전투 통계"));
        assert!(s.contains("15 킬"));
    }

    #[test]
    fn test_trajectory_range_limit() {
        let pts = calculate_trajectory(0, 0, 100, 100, 5);
        assert!(pts.len() <= 7); // 범위 제한
    }
}

// ============================================================================
// [v2.29.0 Phase 93-3] 몬스터 생성 확장 (makemon_phase93_ext.rs)
// 원본: NetHack 3.6.7 src/makemon.c L400-1200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 생성 판정 — spawn_monster (makemon.c L400-700)
// =============================================================================

/// [v2.29.0 93-3] 몬스터 난이도 등급
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyTier {
    Trivial, // 1-3
    Easy,    // 4-8
    Medium,  // 9-15
    Hard,    // 16-24
    Deadly,  // 25-30
    Boss,    // 31+
}

/// [v2.29.0 93-3] 몬스터 생성 결과
#[derive(Debug, Clone)]
pub struct SpawnResult {
    pub monster_id: i32,
    pub level: i32,
    pub hp: i32,
    pub ac: i32,
    pub speed: i32,
    pub difficulty: DifficultyTier,
    pub is_hostile: bool,
    pub has_inventory: bool,
    pub position: (i32, i32),
}

/// [v2.29.0 93-3] 던전 깊이 기반 몬스터 레벨 산정
/// 원본: makemon.c adj_lev()
pub fn adjusted_monster_level(base_level: i32, dungeon_depth: i32, rng: &mut NetHackRng) -> i32 {
    let depth_bonus = dungeon_depth / 3;
    let random_adj = rng.rn2(5) - 2; // -2 ~ +2
    (base_level + depth_bonus + random_adj).max(1).min(50)
}

/// [v2.29.0 93-3] 몬스터 HP 생성
/// 원본: makemon.c m_hp()
pub fn generate_monster_hp(level: i32, is_boss: bool, rng: &mut NetHackRng) -> i32 {
    let base_dice = if is_boss { level * 2 } else { level };
    let mut hp = 0;
    for _ in 0..base_dice.max(1) {
        hp += rng.rn2(8) + 1;
    }
    hp.max(1)
}

/// [v2.29.0 93-3] 난이도 등급 결정
pub fn difficulty_tier(level: i32) -> DifficultyTier {
    match level {
        1..=3 => DifficultyTier::Trivial,
        4..=8 => DifficultyTier::Easy,
        9..=15 => DifficultyTier::Medium,
        16..=24 => DifficultyTier::Hard,
        25..=30 => DifficultyTier::Deadly,
        _ => DifficultyTier::Boss,
    }
}

/// [v2.29.0 93-3] 생성 위치 찾기
pub fn find_spawn_position(
    center_x: i32,
    center_y: i32,
    radius: i32,
    map_width: i32,
    map_height: i32,
    is_walkable: &dyn Fn(i32, i32) -> bool,
    rng: &mut NetHackRng,
) -> Option<(i32, i32)> {
    for _ in 0..100 {
        let x = center_x + rng.rn2(radius * 2 + 1) - radius;
        let y = center_y + rng.rn2(radius * 2 + 1) - radius;
        if x >= 1 && x < map_width - 1 && y >= 1 && y < map_height - 1 && is_walkable(x, y) {
            return Some((x, y));
        }
    }
    None
}

// =============================================================================
// [2] 몬스터 인벤토리 생성 — m_initinv (makemon.c L700-1000)
// =============================================================================

/// [v2.29.0 93-3] 인벤토리 아이템
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterItem {
    pub item_type: String,
    pub enchantment: i32,
    pub is_cursed: bool,
}

/// [v2.29.0 93-3] 몬스터 인벤토리 생성
/// 원본: makemon.c m_initinv()
pub fn generate_monster_inventory(
    monster_level: i32,
    monster_type: &str,
    rng: &mut NetHackRng,
) -> Vec<MonsterItem> {
    let mut items = Vec::new();

    // 무기 생성 (50% 확률 또는 레벨 5+)
    if monster_level >= 5 || rng.rn2(2) == 0 {
        let weapon = match monster_type {
            "orc" => "시미터",
            "goblin" => "단검",
            "skeleton" => "단검",
            "knight" => "장검",
            "wizard" => "지팡이",
            _ => "단검",
        };
        items.push(MonsterItem {
            item_type: weapon.to_string(),
            enchantment: rng.rn2(3),
            is_cursed: rng.rn2(5) == 0,
        });
    }

    // 갑옷 생성 (레벨 8+ 또는 기사형)
    if monster_level >= 8 || monster_type == "knight" {
        items.push(MonsterItem {
            item_type: "갑옷".to_string(),
            enchantment: rng.rn2(2),
            is_cursed: rng.rn2(8) == 0,
        });
    }

    // 금화 (25% 확률)
    if rng.rn2(4) == 0 {
        items.push(MonsterItem {
            item_type: format!("금화 {}개", rng.rn2(monster_level * 10) + 5),
            enchantment: 0,
            is_cursed: false,
        });
    }

    items
}

// =============================================================================
// [3] 몬스터 그룹 생성 — group_spawn (makemon.c L1000-1200)
// =============================================================================

/// [v2.29.0 93-3] 그룹 생성 결과
#[derive(Debug, Clone)]
pub struct GroupSpawnResult {
    pub count: i32,
    pub leader_level: i32,
    pub member_level: i32,
    pub formation: String,
}

/// [v2.29.0 93-3] 그룹 생성 계산
pub fn calculate_group_spawn(
    monster_type: &str,
    base_level: i32,
    dungeon_depth: i32,
    rng: &mut NetHackRng,
) -> GroupSpawnResult {
    let (count, has_leader) = match monster_type {
        "orc" => (rng.rn2(6) + 3, true),
        "goblin" => (rng.rn2(4) + 2, false),
        "ant" => (rng.rn2(8) + 4, true),
        "bee" => (rng.rn2(6) + 3, true),
        "soldier" => (rng.rn2(3) + 2, true),
        _ => (1, false),
    };

    let leader_bonus = if has_leader { 2 } else { 0 };

    GroupSpawnResult {
        count,
        leader_level: base_level + leader_bonus,
        member_level: base_level,
        formation: if has_leader {
            "리더 중심".to_string()
        } else {
            "분산".to_string()
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

    #[test]
    fn test_adjusted_level() {
        let mut rng = test_rng();
        let level = adjusted_monster_level(5, 10, &mut rng);
        assert!(level >= 1 && level <= 50);
    }

    #[test]
    fn test_generate_hp() {
        let mut rng = test_rng();
        let hp = generate_monster_hp(10, false, &mut rng);
        assert!(hp >= 1);
    }

    #[test]
    fn test_boss_hp() {
        let mut rng1 = NetHackRng::new(42);
        let mut rng2 = NetHackRng::new(42);
        let normal = generate_monster_hp(10, false, &mut rng1);
        let boss = generate_monster_hp(10, true, &mut rng2);
        assert!(boss > normal);
    }

    #[test]
    fn test_difficulty_tier() {
        assert_eq!(difficulty_tier(1), DifficultyTier::Trivial);
        assert_eq!(difficulty_tier(10), DifficultyTier::Medium);
        assert_eq!(difficulty_tier(25), DifficultyTier::Deadly);
        assert_eq!(difficulty_tier(35), DifficultyTier::Boss);
    }

    #[test]
    fn test_spawn_position() {
        let mut rng = test_rng();
        let pos = find_spawn_position(40, 10, 5, 80, 21, &|_, _| true, &mut rng);
        assert!(pos.is_some());
    }

    #[test]
    fn test_inventory_orc() {
        let mut rng = test_rng();
        let inv = generate_monster_inventory(5, "orc", &mut rng);
        assert!(!inv.is_empty());
    }

    #[test]
    fn test_group_spawn_orcs() {
        let mut rng = test_rng();
        let group = calculate_group_spawn("orc", 5, 10, &mut rng);
        assert!(group.count >= 3 && group.count <= 8);
        assert!(group.leader_level > group.member_level);
    }

    #[test]
    fn test_group_spawn_single() {
        let mut rng = test_rng();
        let group = calculate_group_spawn("dragon", 20, 25, &mut rng);
        assert_eq!(group.count, 1);
    }
}

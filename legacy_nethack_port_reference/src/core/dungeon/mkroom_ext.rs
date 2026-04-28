// ============================================================================
// [v2.26.0 R14-2] 방 채우기 (mkroom_ext.rs)
// 원본: NetHack 3.6.7 mkroom.c (815줄)
// 특수 방 몬스터/아이템 배치
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 방 타입별 채우기 정책 (원본: mkroom.c mkzoo, mkshop 등)
// =============================================================================

/// [v2.26.0 R14-2] 방 채우기 정책
#[derive(Debug, Clone)]
pub struct RoomFillPolicy {
    /// 몬스터 클래스 제한 (없으면 전체)
    pub monster_classes: Vec<char>,
    /// 밀도 (방 면적 대비 비율)
    pub density: f64,
    /// 아이템 드롭 확률
    pub item_chance: f64,
    /// 골드 배치 여부
    pub has_gold: bool,
    /// 몬스터 수면 여부
    pub sleeping: bool,
    /// 최소 몬스터
    pub min_monsters: i32,
}

/// [v2.26.0 R14-2] 방 유형별 채우기 정책 (원본: mkzoo 분기)
pub fn fill_policy(room_type: &str) -> RoomFillPolicy {
    match room_type {
        "zoo" => RoomFillPolicy {
            monster_classes: vec!['a', 'b', 'c', 'd', 'f', 'h', 'j', 'k', 'r', 's'],
            density: 0.6,
            item_chance: 0.1,
            has_gold: true,
            sleeping: true,
            min_monsters: 4,
        },
        "barracks" => RoomFillPolicy {
            monster_classes: vec!['@'],
            density: 0.5,
            item_chance: 0.3,
            has_gold: false,
            sleeping: false,
            min_monsters: 6,
        },
        "beehive" => RoomFillPolicy {
            monster_classes: vec!['a'],
            density: 0.7,
            item_chance: 0.0,
            has_gold: false,
            sleeping: false,
            min_monsters: 8,
        },
        "morgue" => RoomFillPolicy {
            monster_classes: vec!['Z', 'M', 'V', 'W'],
            density: 0.5,
            item_chance: 0.0,
            has_gold: false,
            sleeping: true,
            min_monsters: 5,
        },
        "treasury" => RoomFillPolicy {
            monster_classes: vec![],
            density: 0.0,
            item_chance: 0.0,
            has_gold: true,
            sleeping: false,
            min_monsters: 0,
        },
        "throne" => RoomFillPolicy {
            monster_classes: vec!['@', 'o'],
            density: 0.3,
            item_chance: 0.2,
            has_gold: true,
            sleeping: false,
            min_monsters: 2,
        },
        _ => RoomFillPolicy {
            monster_classes: vec![],
            density: 0.0,
            item_chance: 0.0,
            has_gold: false,
            sleeping: false,
            min_monsters: 0,
        },
    }
}

// =============================================================================
// [2] 채우기 계획 (원본: mkroom.c fill_zoo)
// =============================================================================

/// [v2.26.0 R14-2] 몬스터 배치 계획
#[derive(Debug, Clone)]
pub struct MonsterPlacement {
    pub x: i32,
    pub y: i32,
    pub monster_class: char,
    pub sleeping: bool,
}

/// [v2.26.0 R14-2] 골드 배치 계획
#[derive(Debug, Clone)]
pub struct GoldPlacement {
    pub x: i32,
    pub y: i32,
    pub amount: i32,
}

/// [v2.26.0 R14-2] 방 채우기 결과
#[derive(Debug, Clone)]
pub struct FillResult {
    pub monsters: Vec<MonsterPlacement>,
    pub gold: Vec<GoldPlacement>,
}

/// [v2.26.0 R14-2] 방 채우기 계획 생성
pub fn plan_room_fill(
    room_lx: i32,
    room_ly: i32,
    room_hx: i32,
    room_hy: i32,
    policy: &RoomFillPolicy,
    depth: i32,
    rng: &mut NetHackRng,
) -> FillResult {
    let area = (room_hx - room_lx + 1) * (room_hy - room_ly + 1);
    let num_monsters = ((area as f64 * policy.density) as i32).max(policy.min_monsters);

    let mut monsters = Vec::new();
    for _ in 0..num_monsters {
        if policy.monster_classes.is_empty() {
            break;
        }
        let x = rng.rn1(room_hx - room_lx + 1, room_lx);
        let y = rng.rn1(room_hy - room_ly + 1, room_ly);
        let cls_idx = rng.rn2(policy.monster_classes.len() as i32) as usize;
        monsters.push(MonsterPlacement {
            x,
            y,
            monster_class: policy.monster_classes[cls_idx],
            sleeping: policy.sleeping,
        });
    }

    let mut gold = Vec::new();
    if policy.has_gold {
        let gold_piles = rng.rn1(3, 1);
        for _ in 0..gold_piles {
            let x = rng.rn1(room_hx - room_lx + 1, room_lx);
            let y = rng.rn1(room_hy - room_ly + 1, room_ly);
            let amount = rng.rn1(depth * 50, 10);
            gold.push(GoldPlacement { x, y, amount });
        }
    }

    FillResult { monsters, gold }
}

// =============================================================================
// [3] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoo_policy() {
        let p = fill_policy("zoo");
        assert!(p.density > 0.5);
        assert!(p.sleeping);
        assert!(p.has_gold);
    }

    #[test]
    fn test_barracks_policy() {
        let p = fill_policy("barracks");
        assert!(p.monster_classes.contains(&'@'));
        assert!(!p.sleeping);
    }

    #[test]
    fn test_treasury_no_monsters() {
        let p = fill_policy("treasury");
        assert_eq!(p.min_monsters, 0);
        assert!(p.has_gold);
    }

    #[test]
    fn test_plan_zoo_fill() {
        let mut rng = NetHackRng::new(42);
        let policy = fill_policy("zoo");
        let result = plan_room_fill(5, 3, 12, 8, &policy, 10, &mut rng);
        assert!(result.monsters.len() >= 4);
        assert!(!result.gold.is_empty());
    }

    #[test]
    fn test_plan_treasury_fill() {
        let mut rng = NetHackRng::new(42);
        let policy = fill_policy("treasury");
        let result = plan_room_fill(5, 3, 8, 6, &policy, 15, &mut rng);
        assert!(result.monsters.is_empty());
        assert!(!result.gold.is_empty());
    }

    #[test]
    fn test_ordinary_empty() {
        let mut rng = NetHackRng::new(42);
        let policy = fill_policy("ordinary");
        let result = plan_room_fill(0, 0, 5, 5, &policy, 1, &mut rng);
        assert!(result.monsters.is_empty());
        assert!(result.gold.is_empty());
    }
}

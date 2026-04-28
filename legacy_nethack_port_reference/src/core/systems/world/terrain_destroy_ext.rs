// ============================================================================
// [v2.24.0 Phase 50-2] 몬스터 터널/붕괴/매몰 (terrain_destroy_ext.rs)
// 원본: NetHack 3.6.7 src/dig.c L1270-1582 — 몬스터 터널, 붕괴, 매몰/발굴
// 순수 결과 패턴: ECS 의존 없이 독립 테스트 가능
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 터널 파기 — mdig_tunnel (dig.c L1270-1365)
// =============================================================================

/// [v2.24.0 50-2] 터널 파기 가능 몬스터 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelDigger {
    /// Umber Hulk — 벽/바위 양쪽 파기 가능
    UmberHulk,
    /// Xorn — 바위 통과 (파기가 아닌 관통)
    Xorn,
    /// Earth Elemental — 지면 파기
    EarthElemental,
    /// 기타 채굴 능력 몬스터
    Generic { dig_power: i32 },
}

/// [v2.24.0 50-2] 몬스터 터널 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdigTunnelResult {
    /// 벽 파괴 → 통로 생성
    WallDestroyed {
        new_tile: &'static str,
        message: String,
    },
    /// 바위 통과 (Xorn 등)
    PassedThrough { message: String },
    /// 파괴 불가 (nondiggable 벽)
    CannotDig { message: String },
    /// 몬스터가 채굴 도중 지형 물질 발견
    MineralFound {
        mineral_type: MineralType,
        message: String,
    },
}

/// [v2.24.0 50-2] 발견 가능 광물 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineralType {
    Gold,
    Gem,
    Rock,
}

/// [v2.24.0 50-2] 몬스터 터널 파기 판정
/// 원본: dig.c mdig_tunnel() L1270-1365
pub fn mdig_tunnel_result(
    digger: TunnelDigger,
    tile_is_wall: bool,
    tile_is_rock: bool,
    tile_is_nondiggable: bool,
    depth: i32,
    rng: &mut NetHackRng,
) -> MdigTunnelResult {
    // [1] Xorn은 관통
    if digger == TunnelDigger::Xorn {
        return MdigTunnelResult::PassedThrough {
            message: "바위를 관통했다.".to_string(),
        };
    }

    // [2] 비파괴 벽 검사
    if tile_is_nondiggable {
        return MdigTunnelResult::CannotDig {
            message: "이 벽은 파괴할 수 없다.".to_string(),
        };
    }

    // [3] 벽이나 바위가 아니면 파기 불가
    if !tile_is_wall && !tile_is_rock {
        return MdigTunnelResult::CannotDig {
            message: "파괴할 대상이 없다.".to_string(),
        };
    }

    // [4] 파기 능력 판정
    let dig_power = match digger {
        TunnelDigger::UmberHulk => 20,
        TunnelDigger::EarthElemental => 15,
        TunnelDigger::Generic { dig_power } => dig_power,
        TunnelDigger::Xorn => unreachable!(),
    };

    // [5] 깊이가 깊을수록 광물 발견 확률 증가
    // 원본: dig.c mdig_tunnel() L1320-1340
    let mineral_chance = (depth / 5).max(1);
    if rng.rn2(50) < mineral_chance {
        let mineral_roll = rng.rn2(20);
        let mineral_type = if mineral_roll == 0 {
            MineralType::Gem
        } else if mineral_roll < 5 {
            MineralType::Gold
        } else {
            MineralType::Rock
        };

        let msg = match mineral_type {
            MineralType::Gem => "보석을 발견했다!",
            MineralType::Gold => "금을 발견했다!",
            MineralType::Rock => "돌 조각을 발견했다.",
        };

        return MdigTunnelResult::MineralFound {
            mineral_type,
            message: msg.to_string(),
        };
    }

    // [6] 벽 파괴
    MdigTunnelResult::WallDestroyed {
        new_tile: "corridor",
        message: "벽을 부수고 통로를 만들었다.".to_string(),
    }
}

// =============================================================================
// [2] 터널 붕괴 — collapse_tunnel (dig.c L1521-1582)
// =============================================================================

/// [v2.24.0 50-2] 붕괴 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollapseTunnelResult {
    /// 안정적 — 붕괴 없음
    Stable,
    /// 부분 붕괴 — 몬스터/플레이어에게 피해
    PartialCollapse {
        damage: i32,
        tiles_collapsed: i32,
        message: String,
    },
    /// 완전 붕괴 — 통로가 바위로 메워짐
    FullCollapse {
        damage: i32,
        tiles_collapsed: i32,
        trapped: bool,
        message: String,
    },
}

/// [v2.24.0 50-2] 터널 붕괴 판정
/// 원본: dig.c collapse_tunnel() L1521-1582
/// adjacent_walls가 너무 적으면 불안정 → 붕괴 확률 증가
pub fn collapse_tunnel_result(
    depth: i32,
    adjacent_walls: i32,
    total_adjacent: i32,
    is_reinforced: bool,
    rng: &mut NetHackRng,
) -> CollapseTunnelResult {
    // [1] 보강된 통로는 절대 붕괴하지 않음
    if is_reinforced {
        return CollapseTunnelResult::Stable;
    }

    // [2] 안정성 계산
    // 원본: tunnel_stability 기반 — 인접 벽 수가 충분하면 안정
    let stability_threshold = if depth >= 20 { 3 } else { 2 };
    if adjacent_walls >= stability_threshold {
        return CollapseTunnelResult::Stable;
    }

    // [3] 붕괴 확률 계산
    // 지지대(인접 벽)가 적을수록 + 깊을수록 붕괴 확률 증가
    let instability = (stability_threshold - adjacent_walls).max(0);
    let collapse_chance = instability * 5 + depth / 3;

    if rng.rn2(100) >= collapse_chance {
        return CollapseTunnelResult::Stable;
    }

    // [4] 붕괴 규모 결정
    let tiles_collapsed = 1 + rng.rn2(instability.max(1) * 2);
    let damage = tiles_collapsed * (2 + rng.rn2(4));

    // [5] 완전 붕괴 vs 부분 붕괴
    if tiles_collapsed >= total_adjacent / 2 {
        let trapped = rng.rn2(3) == 0; // 1/3 확률로 매몰
        CollapseTunnelResult::FullCollapse {
            damage,
            tiles_collapsed,
            trapped,
            message: format!(
                "터널이 완전히 무너졌다! {} 타일이 붕괴. (피해: {})",
                tiles_collapsed, damage
            ),
        }
    } else {
        CollapseTunnelResult::PartialCollapse {
            damage,
            tiles_collapsed,
            message: format!(
                "터널의 일부가 무너졌다! {} 타일 붕괴. (피해: {})",
                tiles_collapsed, damage
            ),
        }
    }
}

// =============================================================================
// [3] 아이템 매몰 — bury_objs (dig.c L1368-1410)
// =============================================================================

/// [v2.24.0 50-2] 매몰 아이템 정보
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuriedItem {
    /// 아이템 인덱스 (인벤토리 또는 바닥 아이템 참조)
    pub item_index: usize,
    /// 아이템 이름 (로그용)
    pub name: String,
    /// 매몰된 깊이 (발굴 난이도에 영향)
    pub burial_depth: i32,
}

/// [v2.24.0 50-2] 아이템 매몰 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuryObjsResult {
    /// 매몰된 아이템 목록
    pub buried_items: Vec<BuriedItem>,
    /// 메시지
    pub message: String,
}

/// [v2.24.0 50-2] 바닥 아이템 매몰 판정
/// 원본: dig.c bury_objs() L1368-1410
pub fn bury_objs_result(
    items_on_floor: &[(usize, String)],
    collapse_severity: i32,
    rng: &mut NetHackRng,
) -> BuryObjsResult {
    let mut buried = Vec::new();

    for (idx, name) in items_on_floor {
        // 붕괴 강도에 비례하여 매몰 확률 결정
        // 원본: dig.c L1375 — rn2(2) => 50% 기본 확률
        let bury_chance = 50 + collapse_severity * 10;
        if rng.rn2(100) < bury_chance.min(90) {
            let burial_depth = 1 + rng.rn2(3);
            buried.push(BuriedItem {
                item_index: *idx,
                name: name.clone(),
                burial_depth,
            });
        }
    }

    let msg = if buried.is_empty() {
        "잔해 속에서 아이템을 찾을 수 없다.".to_string()
    } else {
        format!("{}개의 아이템이 잔해 아래에 묻혔다!", buried.len())
    };

    BuryObjsResult {
        buried_items: buried,
        message: msg,
    }
}

// =============================================================================
// [4] 매몰 아이템 발굴 — unearth_objs (dig.c L1413-1475)
// =============================================================================

/// [v2.24.0 50-2] 발굴 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnearthResult {
    /// 발굴 성공한 아이템 인덱스 목록
    pub unearthed_indices: Vec<usize>,
    /// 발굴 실패 (여전히 매몰) 아이템 수
    pub still_buried: i32,
    /// 메시지
    pub message: String,
}

/// [v2.24.0 50-2] 매몰 아이템 발굴 시도
/// 원본: dig.c unearth_objs() L1413-1475
pub fn unearth_objs_result(
    buried_items: &[BuriedItem],
    dig_power: i32,
    rng: &mut NetHackRng,
) -> UnearthResult {
    let mut unearthed = Vec::new();
    let mut still_buried = 0;

    for item in buried_items {
        // dig_power가 burial_depth 이상이면 성공 확률 높음
        // 원본: dig.c L1430 — rn2(burial_depth) < dig_power
        let chance = if dig_power >= item.burial_depth * 3 {
            // 충분한 파워 → 높은 성공률
            80
        } else if dig_power >= item.burial_depth {
            50
        } else {
            20
        };

        if rng.rn2(100) < chance {
            unearthed.push(item.item_index);
        } else {
            still_buried += 1;
        }
    }

    let msg = if unearthed.is_empty() {
        "아무것도 발굴하지 못했다.".to_string()
    } else if still_buried > 0 {
        format!(
            "{}개의 아이템을 발굴했다! ({}개는 아직 묻혀 있다)",
            unearthed.len(),
            still_buried
        )
    } else {
        format!("{}개의 아이템을 모두 발굴했다!", unearthed.len())
    };

    UnearthResult {
        unearthed_indices: unearthed,
        still_buried,
        message: msg,
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

    // --- mdig_tunnel_result ---

    #[test]
    fn test_xorn_passes_through() {
        let mut rng = test_rng();
        let result = mdig_tunnel_result(TunnelDigger::Xorn, true, false, false, 5, &mut rng);
        assert!(matches!(result, MdigTunnelResult::PassedThrough { .. }));
    }

    #[test]
    fn test_nondiggable_wall() {
        let mut rng = test_rng();
        let result = mdig_tunnel_result(TunnelDigger::UmberHulk, true, false, true, 5, &mut rng);
        assert!(matches!(result, MdigTunnelResult::CannotDig { .. }));
    }

    #[test]
    fn test_umber_hulk_digs_wall() {
        let mut rng = test_rng();
        let result = mdig_tunnel_result(TunnelDigger::UmberHulk, true, false, false, 1, &mut rng);
        // 깊이 1이면 광물 확률 매우 낮음 → 벽 파괴가 대부분
        assert!(
            matches!(result, MdigTunnelResult::WallDestroyed { .. })
                || matches!(result, MdigTunnelResult::MineralFound { .. })
        );
    }

    #[test]
    fn test_no_wall_no_rock() {
        let mut rng = test_rng();
        let result = mdig_tunnel_result(TunnelDigger::UmberHulk, false, false, false, 5, &mut rng);
        assert!(matches!(result, MdigTunnelResult::CannotDig { .. }));
    }

    #[test]
    fn test_deep_mineral_chance() {
        // 깊이 50이면 mineral_chance = 10, rn2(50) < 10 → 20% 확률
        // 50회 시도 중 최소 1회는 광물 발견되어야 함
        let mut found = false;
        for seed in 0..50 {
            let mut rng = NetHackRng::new(seed);
            let result = mdig_tunnel_result(
                TunnelDigger::EarthElemental,
                true,
                false,
                false,
                50,
                &mut rng,
            );
            if matches!(result, MdigTunnelResult::MineralFound { .. }) {
                found = true;
                break;
            }
        }
        assert!(found, "깊은 층에서 50회 시도 중 광물을 발견해야 함");
    }

    // --- collapse_tunnel_result ---

    #[test]
    fn test_reinforced_stable() {
        let mut rng = test_rng();
        let result = collapse_tunnel_result(10, 0, 8, true, &mut rng);
        assert!(matches!(result, CollapseTunnelResult::Stable));
    }

    #[test]
    fn test_enough_walls_stable() {
        let mut rng = test_rng();
        // 깊이 10, 인접 벽 3개 → 안정 (threshold = 2)
        let result = collapse_tunnel_result(10, 3, 8, false, &mut rng);
        assert!(matches!(result, CollapseTunnelResult::Stable));
    }

    #[test]
    fn test_unstable_may_collapse() {
        // 깊이 30, 인접 벽 0개 → 매우 불안정
        // collapse_chance = 3*5 + 30/3 = 25
        // 여러 시도 중 붕괴가 발생해야 함
        let mut collapsed = false;
        for seed in 0..50 {
            let mut rng = NetHackRng::new(seed);
            let result = collapse_tunnel_result(30, 0, 8, false, &mut rng);
            if !matches!(result, CollapseTunnelResult::Stable) {
                collapsed = true;
                break;
            }
        }
        assert!(collapsed, "불안정한 터널은 50회 중 1회 이상 붕괴해야 함");
    }

    // --- bury_objs_result ---

    #[test]
    fn test_bury_empty_floor() {
        let mut rng = test_rng();
        let result = bury_objs_result(&[], 3, &mut rng);
        assert!(result.buried_items.is_empty());
    }

    #[test]
    fn test_bury_some_items() {
        let mut rng = test_rng();
        let items = vec![
            (0, "short sword".to_string()),
            (1, "gold piece".to_string()),
            (2, "scroll".to_string()),
        ];
        let result = bury_objs_result(&items, 3, &mut rng);
        // 기본 확률 50% + severity 30% = 80%, 최대 90%
        // 3개 중 대부분 묻힐 것
        assert!(
            !result.buried_items.is_empty(),
            "일부 아이템이 매몰되어야 함"
        );
    }

    // --- unearth_objs_result ---

    #[test]
    fn test_unearth_high_power() {
        let mut rng = test_rng();
        let buried = vec![
            BuriedItem {
                item_index: 0,
                name: "sword".to_string(),
                burial_depth: 1,
            },
            BuriedItem {
                item_index: 1,
                name: "gold".to_string(),
                burial_depth: 1,
            },
        ];
        let result = unearth_objs_result(&buried, 10, &mut rng);
        // dig_power 10 >= burial_depth*3=3 → 80% 확률
        assert!(
            !result.unearthed_indices.is_empty(),
            "높은 파워로 최소 1개 발굴해야 함"
        );
    }

    #[test]
    fn test_unearth_low_power() {
        // dig_power 0 < burial_depth 3 → 20% 확률
        let mut unearthed_count = 0;
        for seed in 0..20 {
            let mut rng = NetHackRng::new(seed);
            let buried = vec![BuriedItem {
                item_index: 0,
                name: "sword".to_string(),
                burial_depth: 3,
            }];
            let result = unearth_objs_result(&buried, 0, &mut rng);
            unearthed_count += result.unearthed_indices.len();
        }
        // 20회 * 20% = 약 4개 발굴 기대
        assert!(unearthed_count > 0, "낮은 파워로도 가끔은 발굴 성공해야 함");
        assert!(unearthed_count < 20, "낮은 파워로 전부 발굴은 불가능");
    }

    #[test]
    fn test_unearth_empty() {
        let mut rng = test_rng();
        let result = unearth_objs_result(&[], 10, &mut rng);
        assert!(result.unearthed_indices.is_empty());
        assert_eq!(result.still_buried, 0);
    }
}

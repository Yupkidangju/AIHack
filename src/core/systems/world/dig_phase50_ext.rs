// ============================================================================
// [v2.24.0 Phase 50-1] 채굴 시스템 완성 (dig_phase50_ext.rs)
// 원본: NetHack 3.6.7 src/dig.c L260-1192 핵심 미이식 함수 이식
// 순수 결과 패턴: TurnContext/Grid 등 ECS 의존 없이 독립 테스트 가능
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 곡괭이 사용 커맨드 — use_pick_axe (dig.c L907-1005)
// =============================================================================

/// [v2.24.0 50-1] 곡괭이 사용 방향
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickAxeDirection {
    /// 수평 방향 (동/서/남/북)
    Horizontal { dx: i32, dy: i32 },
    /// 아래로 파기
    Down,
    /// 위로 파기
    Up,
}

/// [v2.24.0 50-1] 곡괭이 사용 전 검증 실패 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickAxeCheckFailure {
    /// 손에 곡괭이가 없음
    NotWielding,
    /// 저주받은 곡괭이를 내려놓을 수 없음
    CursedWeapon,
    /// 수중에서 사용 불가
    Underwater,
    /// 공중 레벨에서 사용 불가
    InAir,
    /// 유효하지 않은 방향
    InvalidDirection,
    /// 목표 타일이 채굴 불가
    NonDiggable { reason: String },
    /// 탈것에 탄 상태에서 아래 파기 불가
    MountedDigDown,
    /// 부양 상태에서 아래 파기 불가
    LevitatingDigDown,
}

/// [v2.24.0 50-1] 곡괭이 사용 검증 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickAxeCheckResult {
    /// 사용 가능 — 채굴 시작
    Ok {
        direction: PickAxeDirection,
        target_tile: String,
    },
    /// 사용 불가
    Failed(PickAxeCheckFailure),
}

/// [v2.24.0 50-1] 곡괭이 사용 전 검증
/// 원본: dig.c use_pick_axe() / use_pick_axe2()
pub fn pick_axe_check(
    is_wielding_pick: bool,
    is_cursed_weapon: bool,
    is_underwater: bool,
    is_air_level: bool,
    is_mounted: bool,
    is_levitating: bool,
    direction: PickAxeDirection,
    target_tile_name: &str,
    target_is_diggable: bool,
    target_is_nondiggable_wall: bool,
) -> PickAxeCheckResult {
    // [1] 기본 검증
    if !is_wielding_pick {
        return PickAxeCheckResult::Failed(PickAxeCheckFailure::NotWielding);
    }
    if is_cursed_weapon {
        return PickAxeCheckResult::Failed(PickAxeCheckFailure::CursedWeapon);
    }
    if is_underwater {
        return PickAxeCheckResult::Failed(PickAxeCheckFailure::Underwater);
    }
    if is_air_level {
        return PickAxeCheckResult::Failed(PickAxeCheckFailure::InAir);
    }

    // [2] 방향별 검증
    match direction {
        PickAxeDirection::Down => {
            if is_mounted {
                return PickAxeCheckResult::Failed(PickAxeCheckFailure::MountedDigDown);
            }
            if is_levitating {
                return PickAxeCheckResult::Failed(PickAxeCheckFailure::LevitatingDigDown);
            }
        }
        PickAxeDirection::Horizontal { .. } => {
            // 수평 방향은 추가 제약 없음
        }
        PickAxeDirection::Up => {
            // 위로 파기는 추가 제약 없음 (is_top_level은 dig_up에서 검사)
        }
    }

    // [3] 목표 타일 검증
    if target_is_nondiggable_wall {
        return PickAxeCheckResult::Failed(PickAxeCheckFailure::NonDiggable {
            reason: "이 벽은 파괴할 수 없다.".to_string(),
        });
    }
    if !target_is_diggable {
        return PickAxeCheckResult::Failed(PickAxeCheckFailure::NonDiggable {
            reason: format!("{}은(는) 채굴할 수 없다.", target_tile_name),
        });
    }

    PickAxeCheckResult::Ok {
        direction,
        target_tile: target_tile_name.to_string(),
    }
}

// =============================================================================
// [2] 1턴 채굴 진행 — dig 전체판 (dig.c L260-476)
// =============================================================================

/// [v2.24.0 50-1] 1턴 채굴 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigTurnResult {
    /// 채굴 진행 중 — 진행도 갱신
    InProgress {
        new_effort: i32,
        message: String,
    },
    /// 채굴 완료 — 지형 변경 필요
    Completed {
        new_effort: i32,
        result_type: DigCompletionType,
        message: String,
    },
    /// 어질거림으로 채굴 실패 — 진행도 감소
    Fumbled {
        new_effort: i32,
        message: String,
    },
    /// 곰 함정 자해 — 피해 발생
    BearTrapSelfHit {
        damage: i32,
        message: String,
    },
    /// 곰 함정 파괴 — 함정 해제
    BearTrapBroken {
        message: String,
    },
    /// 채굴 중단 (방해, 이동 등)
    Interrupted {
        message: String,
    },
}

/// [v2.24.0 50-1] 채굴 완료 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigCompletionType {
    /// 벽 파괴 → 통로 생성
    WallToCorr,
    /// 문(숨겨진 문 포함) 파괴 → 열린 문
    DoorOpened,
    /// 바닥 파기 → 구멍/구덩이 생성
    FloorHole,
    /// 천장 파기 → 위층 이동
    CeilingBreached,
    /// 나무 벌목 → 바닥 생성
    TreeFelled,
    /// 비밀 통로 발견
    SecretCorrRevealed,
}

/// [v2.24.0 50-1] 1턴 채굴 진행 상태 입력
#[derive(Debug, Clone)]
pub struct DigTurnInput {
    /// 현재 누적 노력치
    pub current_effort: i32,
    /// 아래로 파는 중인지
    pub digging_down: bool,
    /// 위로 파는 중인지
    pub digging_up: bool,
    /// 플레이어 힘 보정치 (str_bonus_calc에서 산출)
    pub str_bonus: i32,
    /// 무기 인챈트
    pub weapon_spe: i32,
    /// 무기 침식도 (erosion)
    pub weapon_erosion: i32,
    /// 데미지 증가치 (udaminc)
    pub damage_inc: i32,
    /// 난쟁이인지 (채굴 보너스)
    pub is_dwarf: bool,
    /// 어질거림 상태인지
    pub is_fumbling: bool,
    /// 곰 함정에 갇힌 상태인지
    pub in_bear_trap: bool,
    /// 럭 수치 (곰 함정 판정 시 사용)
    pub luck: i32,
    /// 목표 타일 종류
    pub target_tile: DigTargetTile,
    /// 최하단/최상단 레벨인지
    pub is_boundary_level: bool,
}

/// [v2.24.0 50-1] 채굴 대상 타일 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigTargetTile {
    Wall,
    Door,
    SecretDoor,
    SecretCorr,
    Floor,
    Tree,
    DbWall,
}

/// [v2.24.0 50-1] 채굴 완료에 필요한 목표 노력치
/// 원본: dig.c dig() L306-307, L363
fn dig_threshold(tile: DigTargetTile, digging_down: bool) -> i32 {
    if digging_down {
        // 아래로 파기는 더 많은 노력 필요
        // 원본: dig.c L363 — (effort >= 250)
        250
    } else {
        match tile {
            DigTargetTile::Wall => 100,
            DigTargetTile::Door | DigTargetTile::SecretDoor => 60,
            DigTargetTile::SecretCorr => 80,
            DigTargetTile::DbWall => 200,
            DigTargetTile::Tree => 150,
            DigTargetTile::Floor => 200,
        }
    }
}

/// [v2.24.0 50-1] 1턴 채굴 진행 (전체판)
/// 원본: dig.c dig() L260-476 통합
pub fn dig_turn(input: &DigTurnInput, rng: &mut NetHackRng) -> DigTurnResult {
    // [1] 곰 함정 처리 — 원본 dig.c L327-340
    if input.in_bear_trap {
        // rnl(7) > (Fumbling ? 1 : 4) 이면 자해
        let threshold = if input.is_fumbling { 1 } else { 4 };
        let roll = rnl(7, input.luck, rng);
        if roll > threshold {
            // 자해 — 원본 dig.c L329-334
            let base_dmg = 2 + rng.rn2(4);
            let damage = if input.weapon_spe > 0 {
                base_dmg + input.weapon_spe
            } else {
                base_dmg
            };
            return DigTurnResult::BearTrapSelfHit {
                damage,
                message: format!("곡괭이가 빗나가 자신을 찍었다! (피해: {})", damage),
            };
        } else {
            // 함정 파괴
            return DigTurnResult::BearTrapBroken {
                message: "곡괭이로 곰 함정을 부수었다!".to_string(),
            };
        }
    }

    // [2] 어질거림 처리 — 원본 dig.c L273-297
    if input.is_fumbling {
        let fumble_roll = rng.rn2(3);
        if fumble_roll == 0 {
            // 진행도 감소
            let loss = rng.rn2(10).max(1);
            let new_effort = (input.current_effort - loss).max(0);
            return DigTurnResult::Fumbled {
                new_effort,
                message: format!("어질거려서 채굴 진행이 후퇴했다. (노력치 -{})", loss),
            };
        }
        // 어질거려도 2/3 확률로 정상 진행
    }

    // [3] 노력치 계산 — 원본 dig.c L300-303
    // 기본: 10 + rn2(5) + abon + spe - erosion + udaminc
    let base = 10 + rng.rn2(5);
    let dwarf_bonus = if input.is_dwarf { 5 } else { 0 };
    let effort_gain = (base + input.str_bonus + input.weapon_spe
        - input.weapon_erosion
        + input.damage_inc
        + dwarf_bonus)
        .max(1);

    let new_effort = input.current_effort + effort_gain;

    // [4] 완료 판정
    let threshold = if input.digging_up {
        // 위로 파기는 아래보다 더 많은 노력 필요
        300
    } else {
        dig_threshold(input.target_tile, input.digging_down)
    };

    if new_effort >= threshold {
        // 채굴 완료!
        let result_type = if input.digging_down {
            DigCompletionType::FloorHole
        } else if input.digging_up {
            DigCompletionType::CeilingBreached
        } else {
            match input.target_tile {
                DigTargetTile::Wall | DigTargetTile::DbWall => DigCompletionType::WallToCorr,
                DigTargetTile::Door | DigTargetTile::SecretDoor => DigCompletionType::DoorOpened,
                DigTargetTile::SecretCorr => DigCompletionType::SecretCorrRevealed,
                DigTargetTile::Tree => DigCompletionType::TreeFelled,
                DigTargetTile::Floor => DigCompletionType::FloorHole,
            }
        };

        let message = match result_type {
            DigCompletionType::WallToCorr => "벽을 관통하는 데 성공했다!".to_string(),
            DigCompletionType::DoorOpened => "문을 부수었다!".to_string(),
            DigCompletionType::FloorHole => "바닥에 구멍을 뚫었다!".to_string(),
            DigCompletionType::CeilingBreached => "천장을 뚫었다!".to_string(),
            DigCompletionType::TreeFelled => "나무를 베어냈다!".to_string(),
            DigCompletionType::SecretCorrRevealed => "숨겨진 통로를 발견했다!".to_string(),
        };

        return DigTurnResult::Completed {
            new_effort,
            result_type,
            message,
        };
    }

    // [5] 진행 중
    let pct = new_effort * 100 / threshold;
    let progress_msg = if pct < 25 {
        "채굴이 시작되었다."
    } else if pct < 50 {
        "계속 파고 있다."
    } else if pct < 75 {
        "벽이 약해지고 있다."
    } else {
        "거의 다 뚫렸다!"
    };

    DigTurnResult::InProgress {
        new_effort,
        message: format!("{} ({}/{})", progress_msg, new_effort, threshold),
    }
}

// =============================================================================
// [3] 구멍 실제 생성 — digactualhole (dig.c L555-694)
// =============================================================================

/// [v2.24.0 50-1] 구멍 생성 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigHoleResult {
    /// 구덩이 생성 (같은 층에 머묾)
    PitCreated {
        trap_turns: i32,
        items_buried: i32,
        message: String,
    },
    /// 구멍 생성 (아래층으로 떨어짐)
    HoleCreated {
        fall_damage: i32,
        items_buried: i32,
        message: String,
    },
    /// 최하단 레벨이라 구멍 대신 구덩이
    AtBottom {
        trap_turns: i32,
        message: String,
    },
    /// 액체 유입 (용암/물이 차오름)
    LiquidFilled {
        liquid_type: LiquidType,
        message: String,
    },
}

/// [v2.24.0 50-1] 액체 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidType {
    Water,
    Lava,
    Moat,
}

/// [v2.24.0 50-1] 구멍 생성 입력
#[derive(Debug, Clone)]
pub struct DigHoleInput {
    /// 최하단 레벨인지
    pub is_bottom_level: bool,
    /// 주변 액체 상황
    pub adjacent_pools: i32,
    pub adjacent_moats: i32,
    pub adjacent_lava: i32,
    /// 현재 위치의 아이템 수
    pub items_on_floor: i32,
    /// 플레이어 비행/부양 여부
    pub is_flying: bool,
    pub is_levitating: bool,
    /// 던전 깊이 (떨어짐 피해 계산용)
    pub depth: i32,
}

/// [v2.24.0 50-1] 구멍 생성 판정
/// 원본: dig.c digactualhole() L555-694
pub fn dig_actual_hole(input: &DigHoleInput, rng: &mut NetHackRng) -> DigHoleResult {
    // [1] 액체 유입 판정 — 원본 dig.c L697-749
    let total_liquid = input.adjacent_pools + input.adjacent_moats + input.adjacent_lava;
    if total_liquid > 0 {
        let liquid_type = if input.adjacent_lava > 0
            && input.adjacent_lava >= input.adjacent_pools
            && input.adjacent_lava >= input.adjacent_moats
        {
            LiquidType::Lava
        } else if input.adjacent_moats > 0
            && input.adjacent_moats >= input.adjacent_pools
        {
            LiquidType::Moat
        } else {
            LiquidType::Water
        };

        let message = match liquid_type {
            LiquidType::Lava => "용암이 구멍으로 흘러들어온다!".to_string(),
            LiquidType::Moat => "해자의 물이 흘러들어온다!".to_string(),
            LiquidType::Water => "물이 구멍으로 흘러들어온다!".to_string(),
        };

        return DigHoleResult::LiquidFilled {
            liquid_type,
            message,
        };
    }

    // [2] 최하단 레벨 → 구덩이
    if input.is_bottom_level {
        let trap_turns = 2 + rng.rn2(4); // rn1(4, 2)
        return DigHoleResult::AtBottom {
            trap_turns,
            message: "바닥이 너무 단단해 구덩이만 파졌다.".to_string(),
        };
    }

    // [3] 비행/부양 → 떨어지지 않음 → 구멍이지만 떨어짐 피해 0
    if input.is_flying || input.is_levitating {
        return DigHoleResult::HoleCreated {
            fall_damage: 0,
            items_buried: input.items_on_floor.min(5), // 최대 5개까지 매몰
            message: "바닥에 구멍을 뚫었다! 공중에 떠 있어 떨어지지 않았다.".to_string(),
        };
    }

    // [4] 일반 구멍 생성 + 떨어짐
    // 원본: dig.c L628 — 떨어짐 피해 = d(depth, 6)
    let fall_damage = {
        let dice = input.depth.max(1).min(10); // 최대 10d6
        let mut total = 0;
        for _ in 0..dice {
            total += 1 + rng.rn2(6);
        }
        total
    };

    DigHoleResult::HoleCreated {
        fall_damage,
        items_buried: input.items_on_floor.min(5),
        message: format!(
            "바닥에 구멍이 뚫려 아래층으로 떨어졌다! (피해: {})",
            fall_damage
        ),
    }
}

// =============================================================================
// [4] 위로 파기 — dig_up (dig.c L778-906)
// =============================================================================

/// [v2.24.0 50-1] 위로 파기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigUpResult {
    /// 천장 파괴 성공 → 위층 이동
    Breached {
        debris_damage: i32,
        message: String,
    },
    /// 최상단 레벨이라 불가
    AtTop {
        message: String,
    },
    /// 특수 레벨이라 불가
    SpecialLevel {
        message: String,
    },
}

/// [v2.24.0 50-1] 위로 파기 판정
/// 원본: dig.c dig_up() L778-906
pub fn dig_up_result(
    is_top_level: bool,
    is_special_level: bool,
    depth: i32,
    rng: &mut NetHackRng,
) -> DigUpResult {
    if is_top_level {
        return DigUpResult::AtTop {
            message: "여기서 더 위로 파낼 수 없다.".to_string(),
        };
    }

    if is_special_level {
        return DigUpResult::SpecialLevel {
            message: "이 레벨의 천장은 너무 단단해서 파낼 수 없다.".to_string(),
        };
    }

    // 파편 피해 — 원본 dig.c L830 — rnd(depth+10)
    let debris_damage = 1 + rng.rn2((depth + 10).max(1));

    DigUpResult::Breached {
        debris_damage,
        message: format!(
            "천장을 뚫었다! 파편이 떨어진다. (피해: {})",
            debris_damage
        ),
    }
}

// =============================================================================
// [5] 상점 주인 감시 — watch_dig (dig.c L1135-1192)
// =============================================================================

/// [v2.24.0 50-1] 상점 주인 감시 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchDigResult {
    /// 상점 내 채굴 — 부채 부과
    Debt {
        amount: i64,
        message: String,
    },
    /// 상점 밖이므로 무관
    NoShop,
    /// 상점 주인이 없음 (이미 죽었거나 부재)
    NoShopkeeper,
}

/// [v2.24.0 50-1] 상점 주인 채굴 감시
/// 원본: dig.c watch_dig() L1135-1192
pub fn watch_dig_result(
    in_shop: bool,
    shopkeeper_alive: bool,
    is_wall: bool,
    wall_repair_cost: i64,
) -> WatchDigResult {
    if !in_shop {
        return WatchDigResult::NoShop;
    }
    if !shopkeeper_alive {
        return WatchDigResult::NoShopkeeper;
    }

    // 벽 수리 비용 — 원본 dig.c L1162
    // 기본 비용 + 추가 피해 보상
    let amount = if is_wall {
        wall_repair_cost.max(50) // 최소 50골드
    } else {
        // 바닥 채굴은 절반 비용
        (wall_repair_cost / 2).max(25)
    };

    WatchDigResult::Debt {
        amount,
        message: format!(
            "\"이봐! 내 가게를 파괴하다니!\" (부채 +{}금)",
            amount
        ),
    }
}

// =============================================================================
// [6] 지팡이 빔 → 바닥 관통 — zap_over_floor (dig.c L1195-1261)
// =============================================================================

/// [v2.24.0 50-1] 지팡이 채굴 빔 바닥 관통 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZapFloorResult {
    /// 구멍 생성
    HoleMade {
        message: String,
    },
    /// 웅덩이/용암 위 — 빔 소멸
    AbsorbedByLiquid {
        liquid: String,
        message: String,
    },
    /// 특수 바닥 — 빔 반사 또는 소멸
    Resisted {
        message: String,
    },
}

/// [v2.24.0 50-1] 지팡이 빔 바닥 관통 판정
/// 원본: dig.c zap_over_floor() L1195-1261
pub fn zap_floor_result(
    tile_is_room_or_corr: bool,
    is_pool: bool,
    is_lava: bool,
    is_bottom_level: bool,
    is_nondiggable: bool,
) -> ZapFloorResult {
    // [1] 특수 지형 검사
    if is_pool {
        return ZapFloorResult::AbsorbedByLiquid {
            liquid: "water".to_string(),
            message: "채굴 빔이 물에 흡수되었다.".to_string(),
        };
    }
    if is_lava {
        return ZapFloorResult::AbsorbedByLiquid {
            liquid: "lava".to_string(),
            message: "채굴 빔이 용암에 흡수되었다.".to_string(),
        };
    }
    if is_nondiggable || is_bottom_level {
        return ZapFloorResult::Resisted {
            message: "바닥이 너무 단단해서 빔이 반사되었다.".to_string(),
        };
    }

    // [2] 일반 바닥 — 구멍 생성
    if tile_is_room_or_corr {
        return ZapFloorResult::HoleMade {
            message: "채굴 빔이 바닥을 관통했다!".to_string(),
        };
    }

    ZapFloorResult::Resisted {
        message: "이 바닥에는 채굴 빔이 효과가 없다.".to_string(),
    }
}

// =============================================================================
// [보조] rnl 구현 — 럭 보정 난수
// =============================================================================

/// [v2.24.0 50-1] 럭 보정 난수 (rnl)
/// 원본: rnd.c rnl() — 행운 보정 적용
/// rnl(x) = rn2(x) - luck 보정
fn rnl(x: i32, luck: i32, rng: &mut NetHackRng) -> i32 {
    if x <= 0 {
        return 0;
    }
    let raw = rng.rn2(x);
    // 럭이 양수면 결과 감소 (유리), 음수면 증가 (불리)
    let adjusted = raw - (luck / 3);
    adjusted.clamp(0, x - 1)
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::rng::NetHackRng;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    // --- pick_axe_check ---

    #[test]
    fn test_pick_axe_not_wielding() {
        let result = pick_axe_check(
            false, false, false, false, false, false,
            PickAxeDirection::Down, "floor", true, false,
        );
        assert_eq!(result, PickAxeCheckResult::Failed(PickAxeCheckFailure::NotWielding));
    }

    #[test]
    fn test_pick_axe_cursed() {
        let result = pick_axe_check(
            true, true, false, false, false, false,
            PickAxeDirection::Down, "floor", true, false,
        );
        assert_eq!(result, PickAxeCheckResult::Failed(PickAxeCheckFailure::CursedWeapon));
    }

    #[test]
    fn test_pick_axe_underwater() {
        let result = pick_axe_check(
            true, false, true, false, false, false,
            PickAxeDirection::Horizontal { dx: 1, dy: 0 }, "wall", true, false,
        );
        assert_eq!(result, PickAxeCheckResult::Failed(PickAxeCheckFailure::Underwater));
    }

    #[test]
    fn test_pick_axe_mounted_down() {
        let result = pick_axe_check(
            true, false, false, false, true, false,
            PickAxeDirection::Down, "floor", true, false,
        );
        assert_eq!(result, PickAxeCheckResult::Failed(PickAxeCheckFailure::MountedDigDown));
    }

    #[test]
    fn test_pick_axe_levitating_down() {
        let result = pick_axe_check(
            true, false, false, false, false, true,
            PickAxeDirection::Down, "floor", true, false,
        );
        assert_eq!(result, PickAxeCheckResult::Failed(PickAxeCheckFailure::LevitatingDigDown));
    }

    #[test]
    fn test_pick_axe_ok() {
        let result = pick_axe_check(
            true, false, false, false, false, false,
            PickAxeDirection::Horizontal { dx: 1, dy: 0 }, "wall", true, false,
        );
        assert!(matches!(result, PickAxeCheckResult::Ok { .. }));
    }

    #[test]
    fn test_pick_axe_nondiggable() {
        let result = pick_axe_check(
            true, false, false, false, false, false,
            PickAxeDirection::Horizontal { dx: 1, dy: 0 }, "wall", true, true,
        );
        assert!(matches!(result, PickAxeCheckResult::Failed(PickAxeCheckFailure::NonDiggable { .. })));
    }

    // --- dig_turn ---

    fn default_dig_input() -> DigTurnInput {
        DigTurnInput {
            current_effort: 0,
            digging_down: false,
            digging_up: false,
            str_bonus: 2,
            weapon_spe: 0,
            weapon_erosion: 0,
            damage_inc: 0,
            is_dwarf: false,
            is_fumbling: false,
            in_bear_trap: false,
            luck: 0,
            target_tile: DigTargetTile::Wall,
            is_boundary_level: false,
        }
    }

    #[test]
    fn test_dig_turn_progress() {
        let mut rng = test_rng();
        let input = default_dig_input();
        let result = dig_turn(&input, &mut rng);
        assert!(matches!(result, DigTurnResult::InProgress { .. }));
    }

    #[test]
    fn test_dig_turn_complete() {
        let mut rng = test_rng();
        let mut input = default_dig_input();
        // 충분한 노력치를 미리 채워서 완료 유도
        input.current_effort = 95;
        input.str_bonus = 5;
        input.weapon_spe = 3;
        let result = dig_turn(&input, &mut rng);
        assert!(matches!(result, DigTurnResult::Completed { .. }));
    }

    #[test]
    fn test_dig_turn_bear_trap() {
        let mut rng = test_rng();
        let mut input = default_dig_input();
        input.in_bear_trap = true;
        input.luck = -5; // 불운 → 자해 확률 증가
        let result = dig_turn(&input, &mut rng);
        // 곰 함정 결과는 자해 또는 파괴 중 하나
        assert!(matches!(
            result,
            DigTurnResult::BearTrapSelfHit { .. } | DigTurnResult::BearTrapBroken { .. }
        ));
    }

    #[test]
    fn test_dig_turn_fumble() {
        let mut rng = NetHackRng::new(0); // 시드 0이면 첫 rn2(3)==0으로 fumble 확률 높음
        let mut input = default_dig_input();
        input.is_fumbling = true;
        input.current_effort = 50;
        // 여러 번 시도하여 fumble이 한 번은 나오는지 확인
        let mut fumbled = false;
        for seed in 0..20 {
            rng = NetHackRng::new(seed);
            let r = dig_turn(&input, &mut rng);
            if matches!(r, DigTurnResult::Fumbled { .. }) {
                fumbled = true;
                break;
            }
        }
        assert!(fumbled, "어질거림 상태에서 20회 시도 중 1회 이상 후퇴해야 함");
    }

    #[test]
    fn test_dig_turn_down_complete() {
        let mut rng = test_rng();
        let mut input = default_dig_input();
        input.digging_down = true;
        input.target_tile = DigTargetTile::Floor;
        input.current_effort = 245;
        input.str_bonus = 5;
        let result = dig_turn(&input, &mut rng);
        match result {
            DigTurnResult::Completed { result_type, .. } => {
                assert_eq!(result_type, DigCompletionType::FloorHole);
            }
            _ => panic!("바닥 파기가 완료되어야 함"),
        }
    }

    #[test]
    fn test_dig_turn_dwarf_bonus() {
        let mut rng = test_rng();
        let mut input = default_dig_input();
        input.is_dwarf = true;
        let result_dwarf = dig_turn(&input, &mut rng);

        rng = test_rng();
        input.is_dwarf = false;
        let result_normal = dig_turn(&input, &mut rng);

        // 난쟁이의 진행이 더 빨라야 함
        if let (DigTurnResult::InProgress { new_effort: e1, .. }, DigTurnResult::InProgress { new_effort: e2, .. })
            = (&result_dwarf, &result_normal)
        {
            assert!(e1 > e2, "난쟁이 보너스가 적용되어야 함");
        }
    }

    // --- dig_actual_hole ---

    #[test]
    fn test_hole_at_bottom() {
        let mut rng = test_rng();
        let input = DigHoleInput {
            is_bottom_level: true,
            adjacent_pools: 0,
            adjacent_moats: 0,
            adjacent_lava: 0,
            items_on_floor: 0,
            is_flying: false,
            is_levitating: false,
            depth: 5,
        };
        let result = dig_actual_hole(&input, &mut rng);
        assert!(matches!(result, DigHoleResult::AtBottom { .. }));
    }

    #[test]
    fn test_hole_liquid_fill() {
        let mut rng = test_rng();
        let input = DigHoleInput {
            is_bottom_level: false,
            adjacent_pools: 0,
            adjacent_moats: 0,
            adjacent_lava: 2,
            items_on_floor: 0,
            is_flying: false,
            is_levitating: false,
            depth: 5,
        };
        let result = dig_actual_hole(&input, &mut rng);
        match result {
            DigHoleResult::LiquidFilled { liquid_type, .. } => {
                assert_eq!(liquid_type, LiquidType::Lava);
            }
            _ => panic!("용암 유입이 발생해야 함"),
        }
    }

    #[test]
    fn test_hole_normal_fall() {
        let mut rng = test_rng();
        let input = DigHoleInput {
            is_bottom_level: false,
            adjacent_pools: 0,
            adjacent_moats: 0,
            adjacent_lava: 0,
            items_on_floor: 3,
            is_flying: false,
            is_levitating: false,
            depth: 5,
        };
        let result = dig_actual_hole(&input, &mut rng);
        match result {
            DigHoleResult::HoleCreated { fall_damage, items_buried, .. } => {
                assert!(fall_damage > 0, "떨어짐 피해가 발생해야 함");
                assert_eq!(items_buried, 3, "바닥 아이템이 매몰되어야 함");
            }
            _ => panic!("구멍이 생성되어야 함"),
        }
    }

    #[test]
    fn test_hole_flying_no_fall() {
        let mut rng = test_rng();
        let input = DigHoleInput {
            is_bottom_level: false,
            adjacent_pools: 0,
            adjacent_moats: 0,
            adjacent_lava: 0,
            items_on_floor: 1,
            is_flying: true,
            is_levitating: false,
            depth: 5,
        };
        let result = dig_actual_hole(&input, &mut rng);
        match result {
            DigHoleResult::HoleCreated { fall_damage, .. } => {
                assert_eq!(fall_damage, 0, "비행 중이면 떨어짐 피해 없음");
            }
            _ => panic!("구멍이 생성되어야 함"),
        }
    }

    // --- dig_up_result ---

    #[test]
    fn test_dig_up_at_top() {
        let mut rng = test_rng();
        let result = dig_up_result(true, false, 5, &mut rng);
        assert!(matches!(result, DigUpResult::AtTop { .. }));
    }

    #[test]
    fn test_dig_up_special() {
        let mut rng = test_rng();
        let result = dig_up_result(false, true, 5, &mut rng);
        assert!(matches!(result, DigUpResult::SpecialLevel { .. }));
    }

    #[test]
    fn test_dig_up_success() {
        let mut rng = test_rng();
        let result = dig_up_result(false, false, 5, &mut rng);
        match result {
            DigUpResult::Breached { debris_damage, .. } => {
                assert!(debris_damage > 0, "파편 피해가 발생해야 함");
            }
            _ => panic!("천장 돌파 성공이어야 함"),
        }
    }

    // --- watch_dig_result ---

    #[test]
    fn test_watch_dig_no_shop() {
        let result = watch_dig_result(false, true, true, 100);
        assert_eq!(result, WatchDigResult::NoShop);
    }

    #[test]
    fn test_watch_dig_no_keeper() {
        let result = watch_dig_result(true, false, true, 100);
        assert_eq!(result, WatchDigResult::NoShopkeeper);
    }

    #[test]
    fn test_watch_dig_wall_debt() {
        let result = watch_dig_result(true, true, true, 200);
        match result {
            WatchDigResult::Debt { amount, .. } => {
                assert_eq!(amount, 200);
            }
            _ => panic!("부채가 부과되어야 함"),
        }
    }

    #[test]
    fn test_watch_dig_floor_half() {
        let result = watch_dig_result(true, true, false, 200);
        match result {
            WatchDigResult::Debt { amount, .. } => {
                assert_eq!(amount, 100, "바닥 채굴은 절반 비용");
            }
            _ => panic!("부채가 부과되어야 함"),
        }
    }

    #[test]
    fn test_watch_dig_min_cost() {
        let result = watch_dig_result(true, true, true, 10);
        match result {
            WatchDigResult::Debt { amount, .. } => {
                assert_eq!(amount, 50, "최소 50골드");
            }
            _ => panic!("부채가 부과되어야 함"),
        }
    }

    // --- zap_floor_result ---

    #[test]
    fn test_zap_floor_pool() {
        let result = zap_floor_result(false, true, false, false, false);
        assert!(matches!(result, ZapFloorResult::AbsorbedByLiquid { .. }));
    }

    #[test]
    fn test_zap_floor_lava() {
        let result = zap_floor_result(false, false, true, false, false);
        assert!(matches!(result, ZapFloorResult::AbsorbedByLiquid { .. }));
    }

    #[test]
    fn test_zap_floor_nondiggable() {
        let result = zap_floor_result(true, false, false, false, true);
        assert!(matches!(result, ZapFloorResult::Resisted { .. }));
    }

    #[test]
    fn test_zap_floor_bottom() {
        let result = zap_floor_result(true, false, false, true, false);
        assert!(matches!(result, ZapFloorResult::Resisted { .. }));
    }

    #[test]
    fn test_zap_floor_success() {
        let result = zap_floor_result(true, false, false, false, false);
        assert!(matches!(result, ZapFloorResult::HoleMade { .. }));
    }

    // --- rnl ---

    #[test]
    fn test_rnl_positive_luck() {
        let mut rng = test_rng();
        // 행운이 높으면 결과가 더 낮아야 함
        let high_luck = rnl(10, 9, &mut rng);
        let mut rng2 = test_rng();
        let low_luck = rnl(10, -9, &mut rng2);
        // 같은 seed이므로 raw 값은 같지만, luck 보정으로 차이가 남
        assert!(high_luck <= low_luck, "높은 럭은 결과를 낮춰야 함");
    }
}

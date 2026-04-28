// dbridge_ext.rs — dbridge.c 핵심 로직 순수 결과 패턴 이식
// [v2.17.0] 신규 생성: 도개교 회피/점프/파편/잔해/방향/지형 판정 등 10개 함수
// 원본: NetHack 3.6.7 src/dbridge.c (1,005줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 도개교 방향 비트마스크
/// 원본: dbridge.c DB_NORTH/SOUTH/EAST/WEST
pub const DB_NORTH: i32 = 0;
pub const DB_SOUTH: i32 = 1;
pub const DB_EAST: i32 = 2;
pub const DB_WEST: i32 = 3;

/// 도개교 아래 지형 비트마스크
/// 원본: dbridge.c DB_UNDER / DB_ICE / DB_LAVA / DB_MOAT
pub const DB_UNDER: i32 = 0x1C; // 비트 2,3,4
pub const DB_ICE: i32 = 0x04;
pub const DB_LAVA: i32 = 0x08;
pub const DB_MOAT: i32 = 0x10;

// ============================================================
// 열거형
// ============================================================

/// 도개교 아래 지형 유형
/// 원본: dbridge.c db_under_typ() L111-125
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbUnderType {
    Ice,
    Lava,
    Moat,
    Stone,
}

/// 도개교 회피 결과
/// 원본: dbridge.c e_missed() L497-529
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbDodgeResult {
    /// 비실체/벽통과로 자동 회피
    AutoMiss,
    /// 비행/부양으로 회피
    Dodged,
    /// 맞음
    Hit,
}

/// 도개교 점프 결과
/// 원본: dbridge.c e_jumps() L534-556
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbJumpResult {
    /// 행동 불능으로 점프 불가
    CannotJump,
    /// 점프 성공
    Jumped,
    /// 점프 실패 — 압사
    Failed,
}

/// 도개교 파괴 시 파편 수
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebrisResult {
    pub chain_count: i32,
}

/// 도개교 벽 위치 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallOffset {
    pub dx: i32,
    pub dy: i32,
}

// ============================================================
// 1. db_under_type — 도개교 아래 지형 판정
// ============================================================

/// 도개교 마스크에서 아래 지형 유형 결정
/// 원본: dbridge.c db_under_typ() L111-125
pub fn db_under_type(mask: i32) -> DbUnderType {
    match mask & DB_UNDER {
        x if x == DB_ICE => DbUnderType::Ice,
        x if x == DB_LAVA => DbUnderType::Lava,
        x if x == DB_MOAT => DbUnderType::Moat,
        _ => DbUnderType::Stone,
    }
}

// ============================================================
// 2. wall_for_db — 도개교에 연결된 벽 방향 오프셋
// ============================================================

/// 도개교 방향에서 벽 위치 오프셋 계산
/// 원본: dbridge.c get_wall_for_db() L208-226
pub fn wall_for_db(direction: i32) -> WallOffset {
    match direction {
        DB_NORTH => WallOffset { dx: 0, dy: -1 },
        DB_SOUTH => WallOffset { dx: 0, dy: 1 },
        DB_EAST => WallOffset { dx: 1, dy: 0 },
        DB_WEST => WallOffset { dx: -1, dy: 0 },
        _ => WallOffset { dx: 0, dy: 0 },
    }
}

// ============================================================
// 3. e_missed_calc — 낙하 도개교 회피 확률 계산
// ============================================================

/// 낙하하는 도개교/성문이 대상을 빗나갈 확률 판정
/// 원본: dbridge.c e_missed() L497-529
pub fn e_missed_calc(
    passes_walls: bool,
    is_noncorporeal: bool,
    is_flyer: bool,
    is_alert: bool, // !Unaware && mcanmove && !msleeping
    is_floater: bool,
    is_levitating: bool,
    is_in_water: bool,
    is_at_portcullis: bool,
    is_chunks: bool,
    rng: &mut NetHackRng,
) -> DbDodgeResult {
    // 자동 회피: 벽 통과 또는 비실체
    if passes_walls || is_noncorporeal {
        return DbDodgeResult::AutoMiss;
    }

    // 회피 확률 계산 (8분의 N)
    let mut misses: i32 = if is_flyer && is_alert {
        5 // 비행 + 기민
    } else if is_floater || is_levitating {
        3 // 부양
    } else if is_chunks && is_in_water {
        2 // 물에 빠진 상태
    } else {
        0
    };

    // 성문 앞이면 공간 부족으로-3
    if is_at_portcullis {
        misses -= 3;
    }

    if misses >= rng.rnd(8) {
        DbDodgeResult::Dodged
    } else {
        DbDodgeResult::Hit
    }
}

// ============================================================
// 4. e_jumps_calc — 도개교 점프 탈출 확률
// ============================================================

/// 도개교 압사를 피해 점프할 수 있는지 판정
/// 원본: dbridge.c e_jumps() L534-556
pub fn e_jumps_calc(
    can_move: bool, // !Unaware && !Fumbling && mcanmove && !msleeping && mmove>0
    is_confused: bool,
    is_stunned: bool,
    is_at_portcullis: bool,
    rng: &mut NetHackRng,
) -> DbJumpResult {
    if !can_move {
        return DbJumpResult::CannotJump;
    }

    let mut tmp: i32 = 4; // 10분의 4 기본 확률

    if is_confused {
        tmp -= 2;
    }
    if is_stunned {
        tmp -= 3;
    }
    if is_at_portcullis {
        tmp -= 2;
    } // 좁은 공간

    if tmp >= rng.rnd(10) {
        DbJumpResult::Jumped
    } else {
        DbJumpResult::Failed
    }
}

// ============================================================
// 5. destroy_db_debris — 파괴 시 잔해 수
// ============================================================

/// 도개교 파괴 시 쇠사슬 잔해 수 결정
/// 원본: dbridge.c destroy_drawbridge() L936 — rn2(6)
pub fn destroy_db_debris(rng: &mut NetHackRng) -> DebrisResult {
    DebrisResult {
        chain_count: rng.rn2(6),
    }
}

// ============================================================
// 6. is_horizontal — 도개교 방향의 수평 여부
// ============================================================

/// 도개교 방향이 수평인지 판정
/// 원본: dbridge.c create_drawbridge() L246-264
pub fn is_horizontal(direction: i32) -> bool {
    matches!(direction, DB_NORTH | DB_SOUTH)
}

// ============================================================
// 7. bridge_misses_chunks — 파편 회피 (e_missed + chunks=TRUE)
// ============================================================

/// 폭발 파편이 대상을 빗나가는지 (destroy_drawbridge)
/// 원본: dbridge.c destroy_drawbridge() L970 — e_missed(etmp1, TRUE)
pub fn bridge_misses_chunks(
    passes_walls: bool,
    is_noncorporeal: bool,
    is_flyer: bool,
    is_alert: bool,
    is_floater: bool,
    is_levitating: bool,
    is_in_water: bool,
    is_at_portcullis: bool,
    rng: &mut NetHackRng,
) -> bool {
    let result = e_missed_calc(
        passes_walls,
        is_noncorporeal,
        is_flyer,
        is_alert,
        is_floater,
        is_levitating,
        is_in_water,
        is_at_portcullis,
        true, // chunks = true
        rng,
    );
    result != DbDodgeResult::Hit
}

// ============================================================
// 8. gas_cloud_ttl — 도개교 파괴 후 가스 구름 TTL (region 연계)
// ============================================================

/// 가스 구름 생존 시간 (region.c의 gas cloud와 호환)
/// 원본: region.c create_gas_cloud() L1054 — rn1(3,4) → 4~6
pub fn gas_cloud_ttl(rng: &mut NetHackRng) -> i32 {
    rng.rn1(3, 4)
}

// ============================================================
// 9. gas_cloud_dissipate — 가스 구름 약화 판정
// ============================================================

/// 가스 구름이 소멸하는지 또는 약화되는지 판정
/// 원본: region.c expire_gas_cloud() L933-953
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasCloudExpireResult {
    /// 약화 — 데미지 절반 + TTL 갱신
    Dissipate { new_damage: i32 },
    /// 완전 소멸
    Expired,
}

pub fn gas_cloud_dissipate(damage: i32) -> GasCloudExpireResult {
    if damage >= 5 {
        GasCloudExpireResult::Dissipate {
            new_damage: damage / 2,
        }
    } else {
        GasCloudExpireResult::Expired
    }
}

// ============================================================
// 10. gas_cloud_damage — 가스 구름 데미지 계산
// ============================================================

/// 가스 구름 내에서의 데미지 계산
/// 원본: region.c inside_gas_cloud() L983 — rnd(dam) + 5
pub fn gas_cloud_damage(base_damage: i32, rng: &mut NetHackRng) -> i32 {
    rng.rnd(base_damage) + 5
}

// ============================================================
// 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::rng::NetHackRng;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    // --- db_under_type ---
    #[test]
    fn test_db_under_ice() {
        assert_eq!(db_under_type(DB_ICE), DbUnderType::Ice);
    }

    #[test]
    fn test_db_under_lava() {
        assert_eq!(db_under_type(DB_LAVA), DbUnderType::Lava);
    }

    #[test]
    fn test_db_under_moat() {
        assert_eq!(db_under_type(DB_MOAT), DbUnderType::Moat);
    }

    #[test]
    fn test_db_under_stone() {
        assert_eq!(db_under_type(0), DbUnderType::Stone);
    }

    // --- wall_for_db ---
    #[test]
    fn test_wall_north() {
        let off = wall_for_db(DB_NORTH);
        assert_eq!(off.dx, 0);
        assert_eq!(off.dy, -1);
    }

    #[test]
    fn test_wall_east() {
        let off = wall_for_db(DB_EAST);
        assert_eq!(off.dx, 1);
        assert_eq!(off.dy, 0);
    }

    // --- e_missed_calc ---
    #[test]
    fn test_automiss_noncorporeal() {
        let mut rng = test_rng();
        let result = e_missed_calc(
            false, true, false, false, false, false, false, false, false, &mut rng,
        );
        assert_eq!(result, DbDodgeResult::AutoMiss);
    }

    #[test]
    fn test_automiss_passes_walls() {
        let mut rng = test_rng();
        let result = e_missed_calc(
            true, false, false, false, false, false, false, false, false, &mut rng,
        );
        assert_eq!(result, DbDodgeResult::AutoMiss);
    }

    #[test]
    fn test_missed_flyer_probability() {
        let mut rng = test_rng();
        let mut dodged = 0;
        for _ in 0..200 {
            if e_missed_calc(
                false, false, true, true, false, false, false, false, false, &mut rng,
            ) == DbDodgeResult::Dodged
            {
                dodged += 1;
            }
        }
        // 5/8 = 62.5%
        assert!(dodged > 90 && dodged < 170, "비행 회피: {}", dodged);
    }

    #[test]
    fn test_missed_portcullis_penalty() {
        let mut rng = test_rng();
        let mut dodged_open = 0;
        let mut dodged_port = 0;
        for _ in 0..200 {
            if e_missed_calc(
                false, false, true, true, false, false, false, false, false, &mut rng,
            ) == DbDodgeResult::Dodged
            {
                dodged_open += 1;
            }
            if e_missed_calc(
                false, false, true, true, false, false, false, true, false, &mut rng,
            ) == DbDodgeResult::Dodged
            {
                dodged_port += 1;
            }
        }
        // 성문 앞이면 더 적게 회피
        assert!(
            dodged_open > dodged_port,
            "성문<개방: {}>{}",
            dodged_port,
            dodged_open
        );
    }

    // --- e_jumps_calc ---
    #[test]
    fn test_jump_cannot() {
        let mut rng = test_rng();
        assert_eq!(
            e_jumps_calc(false, false, false, false, &mut rng),
            DbJumpResult::CannotJump
        );
    }

    #[test]
    fn test_jump_probability() {
        let mut rng = test_rng();
        let mut jumped = 0;
        for _ in 0..200 {
            if e_jumps_calc(true, false, false, false, &mut rng) == DbJumpResult::Jumped {
                jumped += 1;
            }
        }
        // 4/10 = 40%
        assert!(jumped > 50 && jumped < 120, "점프: {}", jumped);
    }

    #[test]
    fn test_jump_confused_stunned() {
        let mut rng = test_rng();
        let mut jumped = 0;
        for _ in 0..500 {
            if e_jumps_calc(true, true, true, false, &mut rng) == DbJumpResult::Jumped {
                jumped += 1;
            }
        }
        // (4-2-3)=-1 → 거의 0%
        assert!(jumped == 0, "혼란+기절 점프: {}", jumped);
    }

    // --- destroy_db_debris ---
    #[test]
    fn test_debris_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let d = destroy_db_debris(&mut rng);
            assert!(
                d.chain_count >= 0 && d.chain_count < 6,
                "잔해: {}",
                d.chain_count
            );
        }
    }

    // --- is_horizontal ---
    #[test]
    fn test_horizontal() {
        assert!(is_horizontal(DB_NORTH));
        assert!(is_horizontal(DB_SOUTH));
        assert!(!is_horizontal(DB_EAST));
        assert!(!is_horizontal(DB_WEST));
    }

    // --- gas_cloud ---
    #[test]
    fn test_gas_ttl() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let ttl = gas_cloud_ttl(&mut rng);
            assert!(ttl >= 4 && ttl <= 6, "TTL: {}", ttl);
        }
    }

    #[test]
    fn test_gas_dissipate() {
        assert!(matches!(
            gas_cloud_dissipate(10),
            GasCloudExpireResult::Dissipate { new_damage: 5 }
        ));
        assert!(matches!(
            gas_cloud_dissipate(5),
            GasCloudExpireResult::Dissipate { new_damage: 2 }
        ));
        assert_eq!(gas_cloud_dissipate(4), GasCloudExpireResult::Expired);
    }

    #[test]
    fn test_gas_damage() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dmg = gas_cloud_damage(10, &mut rng);
            assert!(dmg >= 6 && dmg <= 15, "가스 데미지: {}", dmg);
        }
    }
}

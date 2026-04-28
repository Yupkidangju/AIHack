// dokick_ext.rs — dokick.c 핵심 로직 순수 결과 패턴 이식
// [v2.13.0] 신규 생성: 킥 데미지/사거리/서투름/회피/왕좌 등 12개 함수
// 원본: NetHack 3.6.7 src/dokick.c (1,812줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 용병 매수 기본 금액
const SOLDIER_GOLD: i64 = 100;
const SERGEANT_GOLD: i64 = 250;
const LIEUTENANT_GOLD: i64 = 500;
const CAPTAIN_GOLD: i64 = 750;

// ============================================================
// 열거형
// ============================================================

/// 왕좌 차기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThroneKickResult {
    /// 파괴 → 금 드랍
    Destroyed { gold_amount: i32 },
    /// 보석/동전 드랍 (행운 좋을 때)
    Loot { gold_amount: i32, gem_count: i32 },
    /// 낙하 (아래 레벨 존재 시)
    FallThrough,
    /// 부상 (ouch)
    Ouch,
    /// 부양 중 → 헛발질
    Dumb,
}

/// 나무 차기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeKickResult {
    /// 과일 낙과
    Fruit { count: i32 },
    /// 벌 떼 소환
    BeeSwarm { count: i32 },
    /// 부상/아무것도 없음
    Ouch,
}

/// 용병 등급
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MercenaryRank {
    Soldier,
    Sergeant,
    Lieutenant,
    Captain,
    Other,
}

/// 상자 차기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoxKickResult {
    /// 잠금 해제
    LockBroken,
    /// 뚜껑 열림/닫힘
    LidSlammed,
    /// 너무 무거움 — 퉁 소리
    Thump,
    /// 사거리 충분 → 날아감
    Kicked,
}

// ============================================================
// 1. kick_damage_calc — 킥 데미지 계산
// ============================================================

/// 킥 기본 데미지 계산 (변신하지 않은 상태)
/// 원본: dokick.c kickdmg() L32-123
/// 반환: (기본 데미지, martial 추가 데미지용 DEX/2)
pub fn kick_damage_calc(
    strength: i32,
    dexterity: i32,
    constitution: i32,
    has_kicking_boots: bool,
    is_clumsy: bool,
    target_thick_skinned: bool,
    target_is_shade: bool,
    boot_spe: i32,
    damage_inc: i32,  // 반지 추가 데미지
    special_dmg: i32, // 특수 데미지 (축복 부츠 등)
    rng: &mut NetHackRng,
) -> i32 {
    // 기본 데미지: (STR + DEX + CON) / 15
    let mut dmg = (strength + dexterity + constitution) / 15;

    // 킥 부츠 보너스
    if has_kicking_boots {
        dmg += 5;
    }

    // 서투름 → 절반
    if is_clumsy {
        dmg /= 2;
    }

    // 두꺼운 피부/그림자 → 0
    if target_thick_skinned || target_is_shade {
        dmg = 0;
    }

    // 랜덤화
    if dmg > 0 {
        dmg = rng.rnd(dmg);
    }

    // 특수 데미지 + 부츠 강화 + 반지 추가
    dmg += special_dmg;
    dmg += boot_spe;
    dmg += damage_inc;

    dmg
}

// ============================================================
// 2. kick_range_calc — 킥 오브젝트 사거리
// ============================================================

/// 일반 킥 오브젝트 사거리 계산
/// 원본: dokick.c really_kick_object() L549
/// 반환: 사거리 (2 미만이면 오브젝트가 움직이지 않음)
pub fn kick_range_calc(
    strength: i32,
    object_weight: i32,
    is_martial: bool,
    in_water: bool,
    in_air: bool,
    on_ice: bool,
    is_greased: bool,
    is_mjollnir: bool,
    rng: &mut NetHackRng,
) -> i32 {
    // 기본: STR/2 - weight/40
    let mut range = strength / 2 - object_weight / 40;

    // martial bonus
    if is_martial {
        range += rng.rnd(3);
    }

    // 환경 보정
    if in_water {
        range = range / 3 + 1;
    } else if in_air {
        range += rng.rnd(3);
    } else {
        if on_ice {
            range += rng.rnd(3);
        }
        if is_greased {
            range += rng.rnd(3);
        }
    }

    // 묠니르는 마법적으로 무거움
    if is_mjollnir {
        range = 1;
    }

    range
}

// ============================================================
// 3. kick_clumsy_check — 서투름 판정
// ============================================================

/// 킥 서투름 판정
/// 원본: dokick.c kick_monster() L227-248
/// 반환: 서투름 여부
pub fn kick_clumsy_check(
    inv_weight_neg: i32, // -inv_weight() → 여유 무게
    weight_cap: i32,
    is_fumbling: bool,
    is_martial: bool,
    has_bulky_armor: bool,
    dexterity: i32,
    rng: &mut NetHackRng,
) -> bool {
    // 짐이 너무 무거움
    let ratio_30 = (weight_cap * 3) / 10;
    let ratio_10 = weight_cap / 10;
    let ratio_20 = weight_cap / 5;

    if inv_weight_neg < ratio_30 {
        // 무게 과중
        let fail_chance = if inv_weight_neg < ratio_10 {
            2
        } else if inv_weight_neg < ratio_20 {
            3
        } else {
            4
        };

        if rng.rn2(fail_chance) == 0 {
            // martial이면 50% 확률로 구제
            if is_martial && rng.rn2(2) == 0 {
                return false; // goto doit
            }
            return true; // 데미지 없음
        }

        // 무게에 따른 서투름
        if inv_weight_neg < ratio_10 {
            return true; // 확정 서투름
        } else {
            let clumsy_chance = if inv_weight_neg < ratio_20 { 2 } else { 3 };
            if rng.rn2(clumsy_chance) == 0 {
                return true;
            }
        }
    }

    // 어질거림
    if is_fumbling {
        return true;
    }

    // 무거운 방어구 + 낮은 DEX
    if has_bulky_armor && dexterity < rng.rnd(25) {
        return true;
    }

    false
}

// ============================================================
// 4. kick_dodge_check — 몬스터 회피 판정
// ============================================================

/// 몬스터가 킥을 회피할 수 있는지 판정
/// 원본: dokick.c kick_monster() L251-280
/// 반환: true면 회피 성공
pub fn kick_dodge_check(
    is_clumsy: bool,
    is_big_monster: bool,
    monster_can_see: bool,
    monster_trapped: bool,
    monster_thick_skin: bool,
    monster_is_eel: bool,
    monster_has_eyes: bool,
    monster_can_move: bool,
    monster_stunned: bool,
    monster_confused: bool,
    monster_sleeping: bool,
    monster_speed: i32, // mmove
    rng: &mut NetHackRng,
) -> bool {
    // 기본 회피 확률: 서투르면 1/3, 아니면 1/4
    let dodge_chance = if is_clumsy { 3 } else { 4 };

    if rng.rn2(dodge_chance) != 0 {
        return false; // 회피 시도 안 함
    }

    // 서투르거나 작은 몬스터여야 회피 가능
    if !is_clumsy && is_big_monster {
        return false;
    }

    // 몬스터 상태 확인
    if !monster_can_see
        || monster_trapped
        || monster_thick_skin
        || monster_is_eel
        || !monster_has_eyes
        || !monster_can_move
        || monster_stunned
        || monster_confused
        || monster_sleeping
        || monster_speed < 12
    {
        return false;
    }

    true
}

// ============================================================
// 5. kick_block_chance — 몬스터가 손으로 방어하는 확률
// ============================================================

/// 몬스터가 손으로 킥을 방어하는 확률
/// 원본: dokick.c L256-260
pub fn kick_block_chance(is_martial: bool, has_hands: bool, rng: &mut NetHackRng) -> bool {
    if !has_hands {
        return false;
    }
    let chance = if is_martial { 5 } else { 3 };
    rng.rn2(chance) == 0
}

// ============================================================
// 6. kick_recoil_range — 부양 시 반동 거리
// ============================================================

/// 부양/공중 상태에서 킥 시 반동 거리
/// 원본: dokick.c dokick() L949-957
pub fn kick_recoil_range(
    self_weight: i32,   // youmonst.data.cwt
    carry_total: i32,   // weight_cap() + inv_weight()
    target_weight: i32, // mdat.cwt
) -> i32 {
    let denominator = (self_weight + carry_total).max(1);
    let range = (3 * target_weight) / denominator;
    range.max(1)
}

// ============================================================
// 7. kick_avrg_attrib — 평균 능력치 계산
// ============================================================

/// 킥 부츠 없을 때의 평균 능력치
/// 원본: dokick.c dokick() L851
pub fn kick_avrg_attrib(strength: i32, dexterity: i32, constitution: i32) -> i32 {
    (strength + dexterity + constitution) / 3
}

// ============================================================
// 8. kick_secret_door_check — 비밀문 발견 확률
// ============================================================

/// 비밀문을 킥으로 발견하는 확률
/// 원본: dokick.c dokick() L980
/// 반환: 발견 성공 여부
pub fn kick_secret_door_check(avrg_attrib: i32, is_levitating: bool, rng: &mut NetHackRng) -> bool {
    !is_levitating && rng.rn2(30) < avrg_attrib
}

// ============================================================
// 9. kick_throne_result — 왕좌 차기 결과
// ============================================================

/// 왕좌를 차면 발생하는 결과 결정
/// 원본: dokick.c dokick() L1014-1056
pub fn kick_throne_result(
    luck: i32,
    is_levitating: bool,
    throne_looted: bool,
    has_lower_level: bool,
    rng: &mut NetHackRng,
) -> ThroneKickResult {
    if is_levitating {
        return ThroneKickResult::Dumb;
    }

    // 행운 나쁘거나 이미 약탈됨 → 1/3 확률로 파괴
    if (luck < 0 || throne_looted) && rng.rn2(3) == 0 {
        return ThroneKickResult::Destroyed {
            gold_amount: rng.rnd(200),
        };
    }

    // 행운 좋고 약탈 안 됨 → 1/3 확률로 보석 드랍
    if luck > 0 && !throne_looted && rng.rn2(3) == 0 {
        let gem_count = (luck + 1).min(6);
        return ThroneKickResult::Loot {
            gold_amount: rng.rn1(201, 300),
            gem_count,
        };
    }

    // 1/4 확률로 낙하 (하위 레벨 존재 시)
    if rng.rn2(4) == 0 {
        if has_lower_level {
            return ThroneKickResult::FallThrough;
        }
    }

    ThroneKickResult::Ouch
}

// ============================================================
// 10. kick_tree_result — 나무 차기 결과
// ============================================================

/// 나무를 차면 발생하는 결과 결정
/// 원본: dokick.c dokick() L1106-1161
pub fn kick_tree_result(
    tree_looted_fruit: bool,
    tree_looted_swarm: bool,
    bees_extinct: bool,
    luck: i32,
    rng: &mut NetHackRng,
) -> TreeKickResult {
    // 75% ouch
    if rng.rn2(3) != 0 {
        return TreeKickResult::Ouch;
    }

    // 23.5% 과일 (약탈 안 됨)
    if !tree_looted_fruit && rng.rn2(15) != 0 {
        let nfruit = 8 - rng.rnl(7, luck);
        return TreeKickResult::Fruit {
            count: nfruit.max(1) as i32,
        };
    }

    // 1.5% 벌 떼 (아직 발생 안 한 경우)
    if !tree_looted_swarm && !bees_extinct {
        let cnt = rng.rnl(4, luck) + 2;
        return TreeKickResult::BeeSwarm { count: cnt as i32 };
    }

    TreeKickResult::Ouch
}

// ============================================================
// 11. mercenary_gold_required — 용병 매수 금액
// ============================================================

/// 용병의 최소 매수 금액
/// 원본: dokick.c ghitm() L360-368
pub fn mercenary_gold_required(rank: MercenaryRank) -> i64 {
    match rank {
        MercenaryRank::Soldier => SOLDIER_GOLD,
        MercenaryRank::Sergeant => SERGEANT_GOLD,
        MercenaryRank::Lieutenant => LIEUTENANT_GOLD,
        MercenaryRank::Captain => CAPTAIN_GOLD,
        MercenaryRank::Other => 0,
    }
}

/// 용병 매수 성공 여부 판정
/// 원본: dokick.c ghitm() L372-376
pub fn mercenary_bribe_check(
    gold_value: i64,
    gold_required: i64,
    player_gold: i64,
    player_level: i32,
    charisma: i32,
    rng: &mut NetHackRng,
) -> bool {
    if gold_required == 0 || charisma <= 0 {
        return false;
    }
    let threshold = gold_required
        + (player_gold + (player_level as i64) * (rng.rn2(5) as i64)) / (charisma as i64);
    gold_value > threshold
}

// ============================================================
// 12. box_kick_result — 상자 차기 결과
// ============================================================

/// 상자 차기 결과 결정
/// 원본: dokick.c really_kick_object() L611-637
pub fn box_kick_result(
    is_locked: bool,
    is_martial: bool,
    range: i32,
    rng: &mut NetHackRng,
) -> BoxKickResult {
    // 사거리 부족 → 퉁 (단, 잠금 해제 시도는 가능)
    if is_locked {
        // 1/5 또는 martial 1/2로 잠금 해제
        let chance = if is_martial { 2 } else { 5 };
        if rng.rn2(chance) == 0 {
            return BoxKickResult::LockBroken;
        }
    } else {
        // 1/3 또는 martial 1/2로 뚜껑 열림
        let chance = if is_martial { 2 } else { 3 };
        if rng.rn2(chance) == 0 {
            return BoxKickResult::LidSlammed;
        }
    }

    if range < 2 {
        BoxKickResult::Thump
    } else {
        BoxKickResult::Kicked
    }
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

    // --- kick_damage_calc ---
    #[test]
    fn test_kick_damage_normal() {
        let mut rng = test_rng();
        let dmg = kick_damage_calc(18, 14, 16, false, false, false, false, 0, 0, 0, &mut rng);
        // (18+14+16)/15 = 3 → rnd(3) = 1~3
        assert!(dmg >= 1 && dmg <= 3, "데미지: {}", dmg);
    }

    #[test]
    fn test_kick_damage_boots() {
        let mut rng = test_rng();
        let dmg = kick_damage_calc(18, 14, 16, true, false, false, false, 2, 1, 3, &mut rng);
        // base = (48/15=3)+5=8, rnd(8)=1~8, +2+1+3 → 7~14
        assert!(dmg >= 7 && dmg <= 14, "부츠 데미지: {}", dmg);
    }

    #[test]
    fn test_kick_damage_thick_skin() {
        let mut rng = test_rng();
        let dmg = kick_damage_calc(18, 14, 16, true, false, true, false, 0, 0, 0, &mut rng);
        // 두꺼운 피부 → dmg=0, special/boot/inc도 0 → 0
        assert_eq!(dmg, 0);
    }

    #[test]
    fn test_kick_damage_shade_with_special() {
        let mut rng = test_rng();
        let dmg = kick_damage_calc(18, 14, 16, true, false, false, true, 0, 0, 5, &mut rng);
        // 그림자 → base=0, 하지만 special_dmg=5
        assert_eq!(dmg, 5);
    }

    // --- kick_range_calc ---
    #[test]
    fn test_range_light_object() {
        let mut rng = test_rng();
        let range = kick_range_calc(18, 20, false, false, false, false, false, false, &mut rng);
        // 18/2 - 20/40 = 9 - 0 = 9
        assert_eq!(range, 9);
    }

    #[test]
    fn test_range_heavy_object() {
        let mut rng = test_rng();
        let range = kick_range_calc(14, 400, false, false, false, false, false, false, &mut rng);
        // 14/2 - 400/40 = 7 - 10 = -3
        assert_eq!(range, -3);
    }

    #[test]
    fn test_range_in_water() {
        let mut rng = test_rng();
        let range = kick_range_calc(18, 40, false, true, false, false, false, false, &mut rng);
        // base = 9-1 = 8, /3+1 = 3
        assert_eq!(range, 3);
    }

    #[test]
    fn test_range_mjollnir() {
        let mut rng = test_rng();
        let range = kick_range_calc(18, 40, true, false, false, false, false, true, &mut rng);
        assert_eq!(range, 1);
    }

    // --- kick_clumsy_check ---
    #[test]
    fn test_clumsy_fumbling() {
        let mut rng = test_rng();
        assert!(kick_clumsy_check(
            100, 200, true, false, false, 18, &mut rng
        ));
    }

    // --- kick_dodge_check ---
    #[test]
    fn test_dodge_stunned_fails() {
        let mut rng = test_rng();
        // 기절한 몬스터는 회피 불가
        let mut dodge_count = 0;
        for _ in 0..100 {
            if kick_dodge_check(
                true, false, true, false, false, false, true, true, true, // 기절
                false, false, 15, &mut rng,
            ) {
                dodge_count += 1;
            }
        }
        assert_eq!(dodge_count, 0, "기절한 몬스터 회피 불가");
    }

    // --- kick_block_chance ---
    #[test]
    fn test_block_no_hands() {
        let mut rng = test_rng();
        assert!(!kick_block_chance(false, false, &mut rng));
    }

    #[test]
    fn test_block_with_hands() {
        let mut rng = test_rng();
        let mut blocks = 0;
        for _ in 0..100 {
            if kick_block_chance(false, true, &mut rng) {
                blocks += 1;
            }
        }
        // ~33% 확률 (1/3)
        assert!(blocks > 15 && blocks < 55, "방어 횟수: {}", blocks);
    }

    // --- kick_recoil_range ---
    #[test]
    fn test_recoil() {
        // 가벼운 플레이어 vs 무거운 대상
        assert_eq!(kick_recoil_range(100, 200, 1000), 10);
        assert_eq!(kick_recoil_range(100, 200, 100), 1);
    }

    // --- kick_avrg_attrib ---
    #[test]
    fn test_avrg_attrib() {
        assert_eq!(kick_avrg_attrib(18, 15, 12), 15);
        assert_eq!(kick_avrg_attrib(10, 10, 10), 10);
    }

    // --- kick_secret_door_check ---
    #[test]
    fn test_secret_door_levitating() {
        let mut rng = test_rng();
        assert!(!kick_secret_door_check(99, true, &mut rng));
    }

    #[test]
    fn test_secret_door_high_attrib() {
        let mut rng = test_rng();
        let mut found = 0;
        for _ in 0..100 {
            if kick_secret_door_check(25, false, &mut rng) {
                found += 1;
            }
        }
        // attrib 25 → rn2(30)<25 = 5/6 확률
        assert!(found > 60, "높은 능력치 발견 횟수: {}", found);
    }

    // --- kick_throne_result ---
    #[test]
    fn test_throne_levitating() {
        let mut rng = test_rng();
        assert_eq!(
            kick_throne_result(0, true, false, true, &mut rng),
            ThroneKickResult::Dumb
        );
    }

    #[test]
    fn test_throne_varied() {
        let mut rng = test_rng();
        let mut destroyed = 0;
        let mut loot = 0;
        let mut fall = 0;
        let mut ouch = 0;
        for _ in 0..300 {
            match kick_throne_result(-3, false, false, true, &mut rng) {
                ThroneKickResult::Destroyed { .. } => destroyed += 1,
                ThroneKickResult::Loot { .. } => loot += 1,
                ThroneKickResult::FallThrough => fall += 1,
                ThroneKickResult::Ouch => ouch += 1,
                _ => {}
            }
        }
        // 행운 < 0이므로 파괴가 나와야 함, loot은 0
        assert!(destroyed > 0, "파괴: {}", destroyed);
        assert_eq!(loot, 0, "행운 <0이면 보석 없음");
    }

    // --- kick_tree_result ---
    #[test]
    fn test_tree_fruit() {
        let mut rng = test_rng();
        let mut fruit_seen = false;
        for _ in 0..100 {
            if let TreeKickResult::Fruit { count } =
                kick_tree_result(false, false, true, 0, &mut rng)
            {
                assert!(count >= 1, "과일 수: {}", count);
                fruit_seen = true;
            }
        }
        assert!(fruit_seen, "과일이 한 번은 나와야 함");
    }

    // --- mercenary_gold_required ---
    #[test]
    fn test_merc_gold() {
        assert_eq!(mercenary_gold_required(MercenaryRank::Soldier), 100);
        assert_eq!(mercenary_gold_required(MercenaryRank::Captain), 750);
        assert_eq!(mercenary_gold_required(MercenaryRank::Other), 0);
    }

    // --- mercenary_bribe_check ---
    #[test]
    fn test_bribe_success() {
        let mut rng = test_rng();
        // 대량의 금 → 성공
        let success = mercenary_bribe_check(5000, 100, 1000, 10, 18, &mut rng);
        assert!(success, "충분한 금으로 매수 성공");
    }

    #[test]
    fn test_bribe_fail() {
        let mut rng = test_rng();
        // 금이 부족
        let success = mercenary_bribe_check(50, 100, 100, 1, 10, &mut rng);
        assert!(!success, "금 부족으로 매수 실패");
    }

    // --- box_kick_result ---
    #[test]
    fn test_box_locked_martial() {
        let mut rng = test_rng();
        let mut broken = 0;
        for _ in 0..100 {
            if let BoxKickResult::LockBroken = box_kick_result(true, true, 5, &mut rng) {
                broken += 1;
            }
        }
        // martial=1/2 확률로 잠금 해제
        assert!(broken > 30 && broken < 70, "잠금 해제: {}", broken);
    }

    #[test]
    fn test_box_unlocked_short_range() {
        let mut rng = test_rng();
        let mut thump = 0;
        let mut slammed = 0;
        for _ in 0..100 {
            match box_kick_result(false, false, 1, &mut rng) {
                BoxKickResult::Thump => thump += 1,
                BoxKickResult::LidSlammed => slammed += 1,
                _ => {}
            }
        }
        assert!(thump > 0 || slammed > 0, "짧은 사거리에서 결과 존재");
    }
}

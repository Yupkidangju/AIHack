// mthrowu_ext.rs — mthrowu.c 핵심 로직 순수 결과 패턴 이식
// [v2.12.0] 신규 생성: 몬스터 투척/발사/브레스/직선정렬/쇠창살 판정 등 12개 함수
// 원본: NetHack 3.6.7 src/mthrowu.c (1,216줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 볼트 한계 거리 (BOLT_LIM = 8)
const BOLT_LIM: i32 = 8;

/// 장대무기 공격 사거리 제곱 (POLE_LIM = 6, 3x3 근접)
const POLE_LIM: i32 = 6;

// ============================================================
// 열거형
// ============================================================

/// 몬스터 크기
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MonsterSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gigantic,
}

/// 공격 유형 분류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackDamageType {
    /// 마법 미사일 (AD_MAGM)
    MagicMissile,
    /// 화염 (AD_FIRE)
    Fire,
    /// 냉기 (AD_COLD)
    Cold,
    /// 수면 (AD_SLEE)
    Sleep,
    /// 분해 (AD_DISN)
    Disintegration,
    /// 전기 (AD_ELEC)
    Electricity,
    /// 독 (AD_DRST)
    Poison,
    /// 산성 (AD_ACID)
    Acid,
    /// 랜덤 브레스 (AD_RBRE)
    RandomBreath,
    /// 실명 (AD_BLND)
    Blinding,
    /// 기타
    Other(i32),
}

/// 투사체 오브젝트 클래스
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectClass {
    Weapon,
    Armor,
    Tool,
    Rock,
    Food,
    Spellbook,
    Wand,
    Ball,
    Chain,
    Coin,
    Gem,
    Other,
}

/// 무기 스킬 종류 (쇠창살 통과 판정용)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponSkill {
    Bow,
    Crossbow,
    Dart,
    Shuriken,
    Spear,
    Knife,
    Dagger,
    Other,
}

/// 쇠창살 충돌 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarHitResult {
    /// 통과 (막히지 않음)
    PassThrough,
    /// 쇠창살에 걸림 (파괴 가능)
    HitBars,
}

/// 쇠창살 충돌 시 소리
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarHitSound {
    /// 바위/쇠구슬
    Whang,
    /// 금/은 재질
    Clink,
    /// 기타
    Clonk,
}

/// 몬스터 연사 횟수 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultishotResult {
    /// 발사 횟수
    pub count: i32,
    /// 스킬 보너스 적용 여부
    pub skill_bonus: bool,
}

/// 장대무기 공격 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolearmAttackResult {
    /// 명중 판정 보너스 (hitv)
    pub hit_bonus: i32,
    /// 기본 데미지
    pub base_damage: i32,
}

/// 브레스 무기 냉각 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreathCooldown {
    /// 재사용 대기 턴
    pub mspec_used: i32,
    /// 수면 브레스 추가 쿨다운
    pub sleep_bonus: i32,
}

/// 침 뱉기 독액 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpitVenomType {
    /// 실명 독 (BLINDING_VENOM)
    BlindingVenom,
    /// 산성 독 (ACID_VENOM)
    AcidVenom,
}

/// 직선 정렬 판정에 필요한 변위값
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineupDisplacement {
    pub tbx: i32,
    pub tby: i32,
}

// ============================================================
// 1. linedup_check — 직선/대각선 정렬 판정
// ============================================================

/// 두 좌표가 직선(수평/수직/대각선)으로 정렬되어 있는지 판정
/// 원본: mthrowu.c linedup()
/// boulder_handling: 0=차단, 1=무시, 2=조건부
/// 반환: (정렬 여부, 변위값)
pub fn linedup_check(ax: i32, ay: i32, bx: i32, by: i32) -> Option<LineupDisplacement> {
    let tbx = ax - bx;
    let tby = ay - by;

    // 같은 위치 — 변위 없음 (투척/발사 불가)
    if tbx == 0 && tby == 0 {
        return None;
    }

    // 직선 또는 대각선 (|dx|==|dy| 또는 dx==0 또는 dy==0)
    let is_lined = tbx == 0 || tby == 0 || tbx.abs() == tby.abs();

    // BOLT_LIM 범위 내
    let dist = distmin(tbx, tby, 0, 0);
    if is_lined && dist < BOLT_LIM {
        Some(LineupDisplacement { tbx, tby })
    } else {
        None
    }
}

// ============================================================
// 2. distmin — 최소 거리 (체스 거리, Chebyshev)
// ============================================================

/// 최소 거리 계산 (체스 거리)
/// 원본: hacklib.c distmin()
pub fn distmin(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();
    dx.max(dy)
}

/// 유클리드 거리 제곱 (distu 대용)
pub fn dist2(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let dx = x0 - x1;
    let dy = y0 - y1;
    dx * dx + dy * dy
}

// ============================================================
// 3. polearm_range_check — 장대무기 공격 범위/명중 판정
// ============================================================

/// 장대무기 공격의 명중 보너스와 기본 데미지 계산
/// 원본: thrwmu() is_pole 분기
pub fn polearm_attack_calc(
    mon_x: i32,
    mon_y: i32,
    target_x: i32,
    target_y: i32,
    weapon_spe: i32,
    weapon_dmgval: i32,
    target_is_big: bool,
) -> Option<PolearmAttackResult> {
    // 사거리 검사 (POLE_LIM = dist2 기준 6)
    if dist2(mon_x, mon_y, target_x, target_y) > POLE_LIM {
        return None;
    }

    // 명중 보너스 = 3 - distmin + (큰 대상 보너스) + 8 + spe
    let dist = distmin(target_x, target_y, mon_x, mon_y);
    let mut hitv = 3 - dist;
    if hitv < -4 {
        hitv = -4;
    }
    if target_is_big {
        hitv += 1;
    }
    hitv += 8 + weapon_spe;

    let damage = weapon_dmgval.max(1);

    Some(PolearmAttackResult {
        hit_bonus: hitv,
        base_damage: damage,
    })
}

// ============================================================
// 4. mon_multishot_count — 몬스터 연사 횟수 계산
// ============================================================

/// 몬스터의 연사 횟수 계산 (화살/표창/투석 등)
/// 원본: monshoot() → m_shot.s 계산 로직
/// mon_level: 몬스터 레벨
/// is_ammo: 화살/볼트 등 탄환 여부
/// has_launcher: 발사대 소지 여부
pub fn mon_multishot_count(
    mon_level: i32,
    is_ammo: bool,
    has_launcher: bool,
    rng: &mut NetHackRng,
) -> MultishotResult {
    if !is_ammo || !has_launcher {
        return MultishotResult {
            count: 1,
            skill_bonus: false,
        };
    }
    // 기본 연사: 1 + 레벨/12, 최대 3
    let base = 1 + mon_level / 12;
    let count = base.min(3);
    // 랜덤 수정: rnd(count)
    let count = rng.rnd(count.max(1));

    MultishotResult {
        count: count.max(1),
        skill_bonus: mon_level >= 12,
    }
}

// ============================================================
// 5. thitu_result — 플레이어 피격 판정
// ============================================================

/// 플레이어 피격 판정 (명중 여부)
/// 원본: mthrowu.c thitu()
/// hitv: 명중 보너스, ac: 플레이어 AC, large_mon: 대형 여부
pub fn thitu_check(
    hitv: i32,
    player_ac: i32,
    player_dex: i32,
    player_level: i32,
    wearing_body_armor: bool,
    rng: &mut NetHackRng,
) -> bool {
    // 원본: (tmp > 0) → tmp > dieroll 로 명중 판정
    // tmp = hitv + player_ac (낮은 AC가 유리하므로 더함)
    let mut tmp = hitv + player_ac;

    // 몸통 갑옷 미착용 시 DEX 보너스 적용
    if !wearing_body_armor {
        // DEX 보정: (dex - 10) / 2, 최대 5
        let dex_bonus = ((player_dex - 10) / 2).min(5).max(-5);
        tmp -= dex_bonus; // 높은 DEX는 회피에 유리 → tmp 감소
    }

    // 레벨 보정
    tmp -= player_level / 3;

    // d20 굴림으로 판정
    let dieroll = rng.rnd(20);
    // 원본: 1은 항상 빗나감, 20은 항상 명중
    if dieroll == 1 {
        return false;
    }
    if dieroll == 20 {
        return true;
    }
    tmp > dieroll
}

// ============================================================
// 6. drop_throw_survival — 투척 후 아이템 생존 판정
// ============================================================

/// 투척/발사 후 아이템이 파괴되지 않고 생존하는지 판정
/// 원본: m_throw() → ohit 후 아이템 파괴 확률
/// breakable: 깨지는 물약/약병 여부
/// spe: 아이템 spe (마법 아이템은 더 내구적)
pub fn drop_throw_survival(
    breakable: bool,
    spe: i32,
    hit_something: bool,
    rng: &mut NetHackRng,
) -> bool {
    if breakable && hit_something {
        return false; // 물약 등은 명중 시 항상 파괴
    }
    // 원본: 명중 시 rn2(3) 확률로 파괴 (마법 아이템은 약간 내구)
    if hit_something {
        if spe > 0 {
            // 마법 아이템은 1/(3+spe) 확률로 파괴
            rng.rn2(3 + spe) != 0
        } else {
            rng.rn2(3) != 0
        }
    } else {
        true // 빗나간 아이템은 보통 생존
    }
}

// ============================================================
// 7. hits_bars_check — 쇠창살 통과 판정
// ============================================================

/// 투척/발사 물체가 쇠창살에 걸리는지 판정
/// 원본: mthrowu.c hits_bars()
pub fn hits_bars_check(
    obj_class: ObjectClass,
    weapon_skill: WeaponSkill,
    is_statue: bool,
    statue_size: MonsterSize,
    is_corpse: bool,
    corpse_size: MonsterSize,
    food_type: Option<i32>, // MEAT_STICK=0, HUGE_CHUNK_OF_MEAT=1로 간소화
    always_hit: bool,
) -> BarHitResult {
    if always_hit {
        return BarHitResult::HitBars;
    }

    let hits = match obj_class {
        ObjectClass::Weapon => {
            // 작은 투사체(활, 석궁, 다트, 슈리켄, 창, 칼)는 통과
            match weapon_skill {
                WeaponSkill::Bow
                | WeaponSkill::Crossbow
                | WeaponSkill::Dart
                | WeaponSkill::Shuriken
                | WeaponSkill::Spear
                | WeaponSkill::Knife => false,
                _ => true,
            }
        }
        ObjectClass::Armor => true, // 장갑(ARM_GLOVES)만 예외이나 단순화
        ObjectClass::Tool => true,  // 대부분 걸림 (열쇠/자물쇠 따개 등 예외 있지만 단순화)
        ObjectClass::Rock => {
            // 조각상은 크기가 Tiny 이하면 통과
            if is_statue {
                statue_size > MonsterSize::Tiny
            } else {
                true // 바위 등은 걸림
            }
        }
        ObjectClass::Food => {
            // 시체는 크기 Tiny 이하면 통과, 고기봉/거대 고기 덩어리는 걸림
            if is_corpse {
                corpse_size > MonsterSize::Tiny
            } else {
                food_type == Some(0) || food_type == Some(1) // MEAT_STICK, HUGE_CHUNK
            }
        }
        ObjectClass::Spellbook | ObjectClass::Wand | ObjectClass::Ball | ObjectClass::Chain => true,
        _ => false, // 동전, 보석 등은 통과
    };

    if hits {
        BarHitResult::HitBars
    } else {
        BarHitResult::PassThrough
    }
}

/// 쇠창살 충돌 시 소리 결정
/// 원본: hit_bars() 내 소리 분기
pub fn bar_hit_sound(
    obj_class: ObjectClass,
    is_boulder: bool,
    is_heavy_iron_ball: bool,
    is_gold_material: bool,
    is_silver_material: bool,
) -> BarHitSound {
    if is_boulder || is_heavy_iron_ball {
        BarHitSound::Whang
    } else if matches!(obj_class, ObjectClass::Coin) || is_gold_material || is_silver_material {
        BarHitSound::Clink
    } else {
        BarHitSound::Clonk
    }
}

// ============================================================
// 8. breath_weapon_name — 브레스 무기 이름
// ============================================================

/// 브레스 무기 유형에 대한 이름 문자열
/// 원본: breathwep[] 배열
const BREATH_NAMES: &[&str] = &[
    "magical energy",           // AD_MAGM
    "a blast of fire",          // AD_FIRE
    "a blast of cold",          // AD_COLD
    "a blast of sleep gas",     // AD_SLEE
    "a bolt of disintegration", // AD_DISN
    "a bolt of lightning",      // AD_ELEC
    "a blast of poison gas",    // AD_DRST
    "a splash of acid",         // AD_ACID
];

/// 브레스 무기 인덱스에서 이름 반환
pub fn breath_weapon_name(adtyp: AttackDamageType) -> &'static str {
    let idx = match adtyp {
        AttackDamageType::MagicMissile => 0,
        AttackDamageType::Fire => 1,
        AttackDamageType::Cold => 2,
        AttackDamageType::Sleep => 3,
        AttackDamageType::Disintegration => 4,
        AttackDamageType::Electricity => 5,
        AttackDamageType::Poison => 6,
        AttackDamageType::Acid => 7,
        _ => return "unknown breath",
    };
    BREATH_NAMES[idx]
}

// ============================================================
// 9. breath_cooldown — 브레스 무기 쿨다운 계산
// ============================================================

/// 몬스터 브레스 사용 후 재사용 대기(mspec_used) 계산
/// 원본: breamu() / breamm() 내 mspec_used 로직
pub fn breath_cooldown(
    adtyp: AttackDamageType,
    is_vs_player: bool,
    target_has_sleep_res: bool,
    rng: &mut NetHackRng,
) -> BreathCooldown {
    if is_vs_player {
        // 대 플레이어: 1/3 확률로 쿨다운, 수면 브레스 추가
        let base = if rng.rn2(3) == 0 { 10 + rng.rn2(20) } else { 0 };
        let sleep_bonus = if adtyp == AttackDamageType::Sleep && !target_has_sleep_res {
            rng.rnd(20)
        } else {
            0
        };
        BreathCooldown {
            mspec_used: base,
            sleep_bonus,
        }
    } else {
        // 대 몬스터: 항상 쿨다운 적용
        BreathCooldown {
            mspec_used: 6 + rng.rn2(18),
            sleep_bonus: 0,
        }
    }
}

// ============================================================
// 10. spit_venom_type — 침 뱉기 독액 종류 결정
// ============================================================

/// 침 뱉기 몬스터의 독액 종류 결정
/// 원본: spitmu() switch(mattk->adtyp)
pub fn spit_venom_type(adtyp: AttackDamageType) -> SpitVenomType {
    match adtyp {
        AttackDamageType::Blinding | AttackDamageType::Poison => SpitVenomType::BlindingVenom,
        AttackDamageType::Acid => SpitVenomType::AcidVenom,
        _ => SpitVenomType::AcidVenom, // 기본값 (impossible case)
    }
}

// ============================================================
// 11. retreat_throw_check — 후퇴 중 투척 판정
// ============================================================

/// 후퇴하는 대상에게 투척할지 결정
/// 원본: thrwmu() URETREATING 분기
/// dist_to_target: 몬스터와 대상 사이 거리(distmin)
pub fn retreat_throw_check(is_retreating: bool, dist_to_target: i32, rng: &mut NetHackRng) -> bool {
    if !is_retreating {
        return true; // 후퇴 중이 아니면 투척
    }
    // 원본: rn2(BOLT_LIM - distmin) → 0이면 투척
    let range = BOLT_LIM - dist_to_target;
    if range <= 0 {
        return true; // 충분히 멀면 투척
    }
    rng.rn2(range) == 0
}

// ============================================================
// 12. random_breath_type — 랜덤 브레스 유형 결정
// ============================================================

/// AD_RBRE(랜덤 브레스) 타입을 실제 브레스 유형으로 결정
/// 원본: breamu()/breamm()의 rnd(AD_ACID) 로직
/// AD_MAGM=1 ~ AD_ACID=8 범위에서 랜덤
pub fn random_breath_type(rng: &mut NetHackRng) -> AttackDamageType {
    match rng.rnd(8) {
        1 => AttackDamageType::MagicMissile,
        2 => AttackDamageType::Fire,
        3 => AttackDamageType::Cold,
        4 => AttackDamageType::Sleep,
        5 => AttackDamageType::Disintegration,
        6 => AttackDamageType::Electricity,
        7 => AttackDamageType::Poison,
        8 => AttackDamageType::Acid,
        _ => AttackDamageType::Fire, // 불가능하지만 안전장치
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

    // --- linedup_check ---
    #[test]
    fn test_linedup_same_pos() {
        // 같은 위치 → None
        assert!(linedup_check(5, 5, 5, 5).is_none());
    }

    #[test]
    fn test_linedup_horizontal() {
        let result = linedup_check(8, 5, 3, 5);
        assert!(result.is_some());
        let disp = result.unwrap();
        assert_eq!(disp.tbx, 5);
        assert_eq!(disp.tby, 0);
    }

    #[test]
    fn test_linedup_diagonal() {
        let result = linedup_check(7, 7, 3, 3);
        assert!(result.is_some());
        let disp = result.unwrap();
        assert_eq!(disp.tbx, 4);
        assert_eq!(disp.tby, 4);
    }

    #[test]
    fn test_linedup_too_far() {
        // 거리 > BOLT_LIM = 8
        assert!(linedup_check(20, 5, 3, 5).is_none());
    }

    #[test]
    fn test_linedup_not_aligned() {
        // 대각선도 직선도 아닌 위치
        assert!(linedup_check(5, 5, 3, 4).is_none());
    }

    // --- distmin ---
    #[test]
    fn test_distmin_zero() {
        assert_eq!(distmin(5, 5, 5, 5), 0);
    }

    #[test]
    fn test_distmin_horizontal() {
        assert_eq!(distmin(0, 0, 5, 0), 5);
    }

    #[test]
    fn test_distmin_diagonal() {
        assert_eq!(distmin(0, 0, 3, 4), 4); // max(3,4)=4
    }

    // --- dist2 ---
    #[test]
    fn test_dist2() {
        assert_eq!(dist2(0, 0, 3, 4), 25); // 9+16=25
    }

    // --- polearm_attack_calc ---
    #[test]
    fn test_polearm_in_range() {
        let result = polearm_attack_calc(5, 5, 7, 5, 3, 10, false);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.hit_bonus > 0);
        assert_eq!(r.base_damage, 10);
    }

    #[test]
    fn test_polearm_out_of_range() {
        // dist2(0,0,5,5) = 50 > POLE_LIM=6
        let result = polearm_attack_calc(0, 0, 5, 5, 3, 10, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_polearm_big_target() {
        let r1 = polearm_attack_calc(5, 5, 6, 5, 0, 5, false).unwrap();
        let r2 = polearm_attack_calc(5, 5, 6, 5, 0, 5, true).unwrap();
        assert_eq!(r2.hit_bonus, r1.hit_bonus + 1);
    }

    // --- mon_multishot_count ---
    #[test]
    fn test_multishot_no_ammo() {
        let mut rng = test_rng();
        let result = mon_multishot_count(20, false, true, &mut rng);
        assert_eq!(result.count, 1);
        assert!(!result.skill_bonus);
    }

    #[test]
    fn test_multishot_with_launcher() {
        let mut rng = test_rng();
        let result = mon_multishot_count(24, true, true, &mut rng);
        assert!(result.count >= 1 && result.count <= 3);
        assert!(result.skill_bonus);
    }

    // --- thitu_check ---
    #[test]
    fn test_thitu_auto_miss() {
        // dieroll=1이면 항상 빗나감 — seed를 조절하여 검증하기 어려우므로
        // 높은 AC(명중 어려운 조건)로 여러 번 굴려 일부 빗나감 확인
        let mut rng = test_rng();
        let mut miss_count = 0;
        for _ in 0..100 {
            if !thitu_check(0, 10, 10, 1, true, &mut rng) {
                miss_count += 1;
            }
        }
        assert!(miss_count > 0, "100번 중 최소 1회 빗나감 필요");
    }

    #[test]
    fn test_thitu_high_hitv() {
        // 매우 높은 명중 보너스 → 대부분 명중
        let mut rng = test_rng();
        let mut hit_count = 0;
        for _ in 0..100 {
            if thitu_check(30, -10, 10, 1, true, &mut rng) {
                hit_count += 1;
            }
        }
        assert!(hit_count >= 90, "높은 hitv는 대부분 명중: {}", hit_count);
    }

    // --- drop_throw_survival ---
    #[test]
    fn test_breakable_hit() {
        let mut rng = test_rng();
        assert!(!drop_throw_survival(true, 0, true, &mut rng));
    }

    #[test]
    fn test_normal_miss() {
        let mut rng = test_rng();
        assert!(drop_throw_survival(false, 0, false, &mut rng));
    }

    // --- hits_bars_check ---
    #[test]
    fn test_bars_arrow_passes() {
        assert_eq!(
            hits_bars_check(
                ObjectClass::Weapon,
                WeaponSkill::Bow,
                false,
                MonsterSize::Medium,
                false,
                MonsterSize::Medium,
                None,
                false,
            ),
            BarHitResult::PassThrough
        );
    }

    #[test]
    fn test_bars_sword_hits() {
        assert_eq!(
            hits_bars_check(
                ObjectClass::Weapon,
                WeaponSkill::Other,
                false,
                MonsterSize::Medium,
                false,
                MonsterSize::Medium,
                None,
                false,
            ),
            BarHitResult::HitBars
        );
    }

    #[test]
    fn test_bars_tiny_statue() {
        assert_eq!(
            hits_bars_check(
                ObjectClass::Rock,
                WeaponSkill::Other,
                true,
                MonsterSize::Tiny,
                false,
                MonsterSize::Medium,
                None,
                false,
            ),
            BarHitResult::PassThrough
        );
    }

    #[test]
    fn test_bars_always_hit() {
        assert_eq!(
            hits_bars_check(
                ObjectClass::Coin,
                WeaponSkill::Other,
                false,
                MonsterSize::Medium,
                false,
                MonsterSize::Medium,
                None,
                true,
            ),
            BarHitResult::HitBars
        );
    }

    // --- bar_hit_sound ---
    #[test]
    fn test_sound_boulder() {
        assert_eq!(
            bar_hit_sound(ObjectClass::Rock, true, false, false, false),
            BarHitSound::Whang
        );
    }

    #[test]
    fn test_sound_gold() {
        assert_eq!(
            bar_hit_sound(ObjectClass::Coin, false, false, true, false),
            BarHitSound::Clink
        );
    }

    #[test]
    fn test_sound_other() {
        assert_eq!(
            bar_hit_sound(ObjectClass::Weapon, false, false, false, false),
            BarHitSound::Clonk
        );
    }

    // --- breath_weapon_name ---
    #[test]
    fn test_breath_fire() {
        assert_eq!(
            breath_weapon_name(AttackDamageType::Fire),
            "a blast of fire"
        );
    }

    #[test]
    fn test_breath_acid() {
        assert_eq!(
            breath_weapon_name(AttackDamageType::Acid),
            "a splash of acid"
        );
    }

    #[test]
    fn test_breath_unknown() {
        assert_eq!(
            breath_weapon_name(AttackDamageType::Other(99)),
            "unknown breath"
        );
    }

    // --- breath_cooldown ---
    #[test]
    fn test_cooldown_vs_player() {
        let mut rng = test_rng();
        let cd = breath_cooldown(AttackDamageType::Fire, true, false, &mut rng);
        // 0 또는 10~29
        assert!(cd.mspec_used == 0 || (cd.mspec_used >= 10 && cd.mspec_used < 30));
        assert_eq!(cd.sleep_bonus, 0); // 화염은 수면 보너스 없음
    }

    #[test]
    fn test_cooldown_sleep_vs_player() {
        let mut rng = test_rng();
        let cd = breath_cooldown(AttackDamageType::Sleep, true, false, &mut rng);
        // 수면 저항 없으면 추가 쿨다운
        assert!(cd.sleep_bonus >= 1 && cd.sleep_bonus <= 20);
    }

    #[test]
    fn test_cooldown_vs_monster() {
        let mut rng = test_rng();
        let cd = breath_cooldown(AttackDamageType::Cold, false, false, &mut rng);
        // 항상 6~23
        assert!(cd.mspec_used >= 6 && cd.mspec_used < 24);
    }

    // --- spit_venom_type ---
    #[test]
    fn test_spit_blinding() {
        assert_eq!(
            spit_venom_type(AttackDamageType::Blinding),
            SpitVenomType::BlindingVenom
        );
    }

    #[test]
    fn test_spit_acid() {
        assert_eq!(
            spit_venom_type(AttackDamageType::Acid),
            SpitVenomType::AcidVenom
        );
    }

    // --- retreat_throw_check ---
    #[test]
    fn test_retreat_not_retreating() {
        let mut rng = test_rng();
        assert!(retreat_throw_check(false, 3, &mut rng));
    }

    #[test]
    fn test_retreat_far_away() {
        let mut rng = test_rng();
        // 거리 >= BOLT_LIM → 항상 투척
        assert!(retreat_throw_check(true, BOLT_LIM, &mut rng));
    }

    // --- random_breath_type ---
    #[test]
    fn test_random_breath_range() {
        let mut rng = test_rng();
        for _ in 0..50 {
            let result = random_breath_type(&mut rng);
            // Other(99) 같은 값은 나오면 안 됨
            assert_ne!(result, AttackDamageType::Other(99));
        }
    }

    #[test]
    fn test_random_breath_distribution() {
        let mut rng = test_rng();
        let mut fire_count = 0;
        for _ in 0..200 {
            if random_breath_type(&mut rng) == AttackDamageType::Fire {
                fire_count += 1;
            }
        }
        // 1/8 = 12.5% → 200 중 약 25, 최소 5는 나와야 함
        assert!(fire_count >= 5, "화염 브레스 {}번", fire_count);
    }
}

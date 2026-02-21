// trap_ext.rs — trap.c 핵심 로직 순수 결과 패턴 이식
// [v2.16.0] 신규 생성: 함정 데미지/부식/탈출/낙하/독 등 12개 함수
// 원본: NetHack 3.6.7 src/trap.c (5,477줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 최대 부식 수치 (MAX_ERODE)
const MAX_ERODE: i32 = 3;

// ============================================================
// 열거형
// ============================================================

/// 부식 유형
/// 원본: trap.c erode_obj() L139-190
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErodeType {
    Burn,
    Rust,
    Rot,
    Corrode,
}

/// 부식 결과
/// 원본: trap.c erode_obj() 반환값 (ER_*)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErodeResult {
    /// 그리스로 보호됨
    Greased,
    /// 아무 일 없음 (내성, 이미 최대 부식 등)
    Nothing,
    /// 부식 진행됨 (erosion 증가)
    Damaged,
    /// 완전 파괴됨
    Destroyed,
}

/// 화염 방어구 연소 슬롯
/// 원본: trap.c burnarmor() L55-128
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurnArmorSlot {
    Helmet,
    CloakOrBodyOrShirt,
    Shield,
    Gloves,
    Boots,
}

/// 함정 유형
/// 원본: trap.c dotrap() 주요 switch 분기
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    Arrow,
    Dart,
    Rock,
    SqueakyBoard,
    BearTrap,
    SleepGas,
    RustTrap,
    FireTrap,
    Pit,
    SpikedPit,
    Hole,
    Trapdoor,
    Teleporter,
    LevelTeleporter,
    Web,
    Landmine,
    MagicPortal,
    AntiMagic,
    PolyTrap,
    VibratingSquare,
    RollingBoulder,
    StatueTrap,
}

/// 함정 탈출 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapEscapeResult {
    /// 부양/비행으로 회피
    FloatOver,
    /// 민첩하게 탈출 (rn2(5) 성공)
    Escaped,
    /// 함정에 걸림
    Triggered,
}

// ============================================================
// 1. erode_check — 부식 판정 (순수)
// ============================================================

/// 아이템 부식 판정
/// 원본: trap.c erode_obj() L139-280
pub fn erode_check(
    erode_type: ErodeType,
    is_greased: bool,
    check_grease: bool,
    is_vulnerable: bool,
    is_erodeproof: bool,
    is_blessed: bool,
    current_erosion: i32,
    allow_destroy: bool,
    luck: i32,
    rng: &mut NetHackRng,
) -> ErodeResult {
    // 그리스 보호 (화염/부패에는 미적용)
    if check_grease && is_greased {
        return ErodeResult::Greased;
    }

    // 침식 대상 아님
    if !is_vulnerable {
        return ErodeResult::Nothing;
    }

    // 부식 방지 또는 이미 알려진 내성
    if is_erodeproof {
        return ErodeResult::Nothing;
    }

    // 축복 아이템: rnl(4) == 0이면 보호됨
    if is_blessed && rng.rnl(4, luck) == 0 {
        return ErodeResult::Nothing;
    }

    // 부식 진행 가능
    if current_erosion < MAX_ERODE {
        return ErodeResult::Damaged;
    }

    // 최대 부식 → 파괴 가능
    if allow_destroy {
        return ErodeResult::Destroyed;
    }

    ErodeResult::Nothing
}

// ============================================================
// 2. grease_wearoff — 그리스 벗겨짐 확률
// ============================================================

/// 그리스로 보호 후 벗겨질 확률
/// 원본: trap.c grease_protect() L303 — !rn2(2)
pub fn grease_wearoff(rng: &mut NetHackRng) -> bool {
    rng.rn2(2) == 0
}

// ============================================================
// 3. burnarmor_slot — 화염 방어구 연소 슬롯 결정
// ============================================================

/// 화염 피해 시 어떤 방어구 슬롯이 연소 대상인지 결정
/// 원본: trap.c burnarmor() L82 — rn2(5)
pub fn burnarmor_slot(rng: &mut NetHackRng) -> BurnArmorSlot {
    match rng.rn2(5) {
        0 => BurnArmorSlot::Helmet,
        1 => BurnArmorSlot::CloakOrBodyOrShirt,
        2 => BurnArmorSlot::Shield,
        3 => BurnArmorSlot::Gloves,
        4 => BurnArmorSlot::Boots,
        _ => unreachable!(),
    }
}

// ============================================================
// 4. trap_escape_check — 함정 탈출 판정
// ============================================================

/// 이미 확인된 함정에서 플레이어가 탈출 가능한지 판정
/// 원본: trap.c dotrap() L931-949
pub fn trap_escape_check(
    is_levitating: bool,
    is_flying: bool,
    is_fumbling: bool,
    is_plunging: bool,
    is_pit_type: bool,
    is_clinger: bool,
    is_sokoban: bool,
    rng: &mut NetHackRng,
) -> TrapEscapeResult {
    // 소코반에서는 탈출 불가
    if is_sokoban && is_pit_type {
        return TrapEscapeResult::Triggered;
    }

    // 부양/비행 + 구덩이/곰 함정이면 회피
    if (is_levitating || (is_flying && !is_plunging)) && is_pit_type {
        return TrapEscapeResult::FloatOver;
    }

    // 어질거림이면 탈출 불가
    if is_fumbling {
        return TrapEscapeResult::Triggered;
    }

    // 1/5 확률로 탈출 (클링어는 구덩이에서 자동 탈출)
    if rng.rn2(5) == 0 || (is_pit_type && is_clinger) {
        return TrapEscapeResult::Escaped;
    }

    TrapEscapeResult::Triggered
}

// ============================================================
// 5. bear_trap_duration — 곰 함정 지속 턴 계산
// ============================================================

/// 곰 함정에 잡히는 턴 수
/// 원본: trap.c dotrap() L1089 — rn1(4,4) → 4~7턴
pub fn bear_trap_duration(rng: &mut NetHackRng) -> i32 {
    rng.rn1(4, 4)
}

/// 곰 함정 데미지
/// 원본: trap.c dotrap() L1073 — d(2,4)
pub fn bear_trap_damage(rng: &mut NetHackRng) -> i32 {
    rng.d(2, 4)
}

// ============================================================
// 6. pit_damage — 구덩이 데미지 계산
// ============================================================

/// 구덩이 낙하 데미지
/// 원본: trap.c dotrap() L1227-1311
pub fn pit_damage(is_spiked: bool, rng: &mut NetHackRng) -> i32 {
    let base = rng.rnd(6); // 기본 낙하 데미지
    if is_spiked {
        base + rng.rnd(10) // 가시 추가 데미지
    } else {
        base
    }
}

/// 구덩이 탈출 턴 수
/// 원본: trap.c dotrap() L1286 — rn1(6,2) → 2~7턴
pub fn pit_escape_turns(rng: &mut NetHackRng) -> i32 {
    rng.rn1(6, 2)
}

// ============================================================
// 7. dart_poison_chance — 다트 독 부여 확률
// ============================================================

/// 다트 함정의 다트에 독이 있을 확률
/// 원본: trap.c dotrap() L997-998 — !rn2(6)
pub fn dart_poison_chance(rng: &mut NetHackRng) -> bool {
    rng.rn2(6) == 0
}

// ============================================================
// 8. sleep_gas_duration — 수면 가스 지속 턴
// ============================================================

/// 수면 가스 함정 지속 턴 수
/// 원본: trap.c dotrap() L1113 — rnd(25)
pub fn sleep_gas_duration(rng: &mut NetHackRng) -> i32 {
    rng.rnd(25)
}

// ============================================================
// 9. rock_trap_damage — 바위 함정 데미지
// ============================================================

/// 바위 함정 데미지 (헬멧에 따라 감소)
/// 원본: trap.c dotrap() L1027-1049
pub fn rock_trap_damage(has_metal_helmet: bool, rng: &mut NetHackRng) -> i32 {
    let base = rng.d(2, 6);
    if has_metal_helmet {
        2 // 금속 헬멧 장착 시 고정 2 데미지
    } else {
        base
    }
}

// ============================================================
// 10. fall_through_depth — 함정 문/구멍 낙하 깊이
// ============================================================

/// 함정 문이나 구멍으로 낙하 시 떨어지는 깊이 계산
/// 원본: trap.c fall_through() L472-474
/// 현재 레벨에서 25% 확률로 추가 층을 건너뜀
pub fn fall_through_depth(current_level: i32, bottom_level: i32, rng: &mut NetHackRng) -> i32 {
    let mut new_level = current_level;
    loop {
        new_level += 1;
        if rng.rn2(4) != 0 || new_level >= bottom_level {
            break;
        }
    }
    new_level.min(bottom_level)
}

// ============================================================
// 11. arrow_trap_empty — 화살 함정 소진 확률
// ============================================================

/// 한번 발사된 화살 함정이 빈 것(소진)으로 판정
/// 원본: trap.c dotrap() L962 — trap->once && !rn2(15)
pub fn arrow_trap_empty(already_fired: bool, rng: &mut NetHackRng) -> bool {
    already_fired && rng.rn2(15) == 0
}

// ============================================================
// 12. rust_trap_slot — 녹 함정 피격 부위
// ============================================================

/// 녹 함정 발동 시 물이 맞는 부위 결정
/// 원본: trap.c dotrap() L1126 — rn2(5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustTrapSlot {
    Head,     // 0
    LeftArm,  // 1
    RightArm, // 2
    Body,     // 3, 4
}

pub fn rust_trap_slot(rng: &mut NetHackRng) -> RustTrapSlot {
    match rng.rn2(5) {
        0 => RustTrapSlot::Head,
        1 => RustTrapSlot::LeftArm,
        2 => RustTrapSlot::RightArm,
        _ => RustTrapSlot::Body,
    }
}

// ============================================================
// [v2.19.0] 13. magic_trap_effect — 마법 함정 효과
// 원본: trap.c domagictrap() L1455-1567
// ============================================================

/// 마법 함정 효과 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagicTrapEffect {
    /// 구덩이로 변환 (함정 소멸)
    TurnIntoPit,
    /// 아이템 저주
    CurseItems,
    /// 아이템 축복 (행운)
    BlessItems,
    /// 몬스터 소환
    SummonMonster,
    /// 셔플 (텔레포트 등)
    TeleportShuffle,
    /// 불꽃 기둥
    TowerOfFlame,
    /// 마법 미사일 피격
    MagicMissile,
    /// 아무 일 없음
    Nothing,
}

/// [v2.19.0] 마법 함정 효과 결정 (원본: domagictrap L1455-1567)
/// 여러 갈래 중 확률적으로 하나를 선택
pub fn magic_trap_effect(rng: &mut NetHackRng) -> MagicTrapEffect {
    // 원본: rn2(30) + rn2(5) 조합
    let roll = rng.rn2(30);
    match roll {
        0..=4 => MagicTrapEffect::TurnIntoPit,
        5..=9 => MagicTrapEffect::CurseItems,
        10..=11 => MagicTrapEffect::BlessItems,
        12..=15 => MagicTrapEffect::SummonMonster,
        16..=19 => MagicTrapEffect::TeleportShuffle,
        20..=23 => MagicTrapEffect::TowerOfFlame,
        24..=27 => MagicTrapEffect::MagicMissile,
        _ => MagicTrapEffect::Nothing,
    }
}

// ============================================================
// [v2.19.0] 14. fire_trap_damage — 화염 함정 데미지
// 원본: trap.c dofiretrap() L1569-1620
// ============================================================

/// [v2.19.0] 화염 함정 데미지 (원본: dofiretrap L1569-1620)
/// 화염 저항 시 0, 아니면 d(2,4)
pub fn fire_trap_damage(has_fire_resistance: bool, rng: &mut NetHackRng) -> i32 {
    if has_fire_resistance {
        0
    } else {
        rng.d(2, 4)
    }
}

// ============================================================
// [v2.19.0] 15. web_escape_check — 거미줄 탈출 판정
// 원본: trap.c dotrap() L1340-1380
// ============================================================

/// [v2.19.0] 거미줄 탈출 판정 (원본: dotrap ~L1340)
/// 거미(amorphous), 절단 무기, 불 → 탈출 가능
pub fn web_escape_check(
    is_amorphous: bool,
    is_whirly: bool,
    has_fire_resistance: bool,
    is_spider: bool,
    has_edged_weapon: bool,
    strength: i32,
    rng: &mut NetHackRng,
) -> bool {
    // 거미는 무조건 통과
    if is_spider {
        return true;
    }
    // 비정형체/회오리는 통과
    if is_amorphous || is_whirly {
        return true;
    }
    // 날카로운 무기로 자름
    if has_edged_weapon {
        return true;
    }
    // 힘으로 탈출: str >= 18이면 1/3 확률, 아니면 1/6
    if strength >= 18 {
        rng.rn2(3) == 0
    } else {
        rng.rn2(6) == 0
    }
}

// ============================================================
// [v2.19.0] 16. landmine_damage — 지뢰 데미지
// 원본: trap.c dotrap() L1392-1454
// ============================================================

/// [v2.19.0] 지뢰 데미지 (원본: dotrap ~L1392)
/// 기본 d(16,1) = 16 데미지, 부양 시 절반
pub fn landmine_damage(is_levitating: bool, is_flying: bool, rng: &mut NetHackRng) -> i32 {
    let base = rng.rnd(16);
    if is_levitating || is_flying {
        base / 2 // 폭풍 약화
    } else {
        base
    }
}

/// [v2.19.0] 지뢰 폭발 시 주변 구덩이 생성 여부
pub fn landmine_creates_pit(is_levitating: bool, is_flying: bool) -> bool {
    !is_levitating && !is_flying
}

// ============================================================
// [v2.19.0] 17. rolling_boulder_damage — 구르는 바위 데미지
// 원본: trap.c dotrap() ~L880-920
// ============================================================

/// [v2.19.0] 구르는 바위 데미지 (원본: dotrap ~L880)
/// d(2,10) = 2~20
pub fn rolling_boulder_damage(rng: &mut NetHackRng) -> i32 {
    rng.d(2, 10)
}

/// [v2.19.0] 구르는 바위 회피 (원본: dotrap ~L880 — rn2(10))
/// 10% 확률로 회피
pub fn rolling_boulder_dodge(rng: &mut NetHackRng) -> bool {
    rng.rn2(10) == 0
}

// ============================================================
// [v2.19.0] 18. untrap_prob — 함정 해제 확률
// 원본: trap.c untrap_prob() ~L4600
// ============================================================

/// [v2.19.0] 함정 해제 난이도 (원본: untrap_prob ~L4600)
/// 민첩성(dex), 레벨, 도적 직업 여부로 성공률 결정
/// 반환: 1~100 범위의 성공 확률 (%)
pub fn untrap_prob(dexterity: i32, player_level: i32, is_rogue: bool, trap_type: TrapType) -> i32 {
    // 기본 확률: (dex + level) / 2
    let mut prob = (dexterity + player_level) / 2;

    // 도적 보너스 (+25%)
    if is_rogue {
        prob += 25;
    }

    // 함정 종류별 난이도 보정
    let difficulty = match trap_type {
        TrapType::Arrow | TrapType::Dart => 5,
        TrapType::BearTrap => 10,
        TrapType::SqueakyBoard => 0,
        TrapType::Landmine => 20,
        TrapType::Web => 5,
        TrapType::FireTrap => 15,
        TrapType::SleepGas => 10,
        TrapType::RustTrap => 5,
        TrapType::Pit | TrapType::SpikedPit => 5,
        TrapType::RollingBoulder => 15,
        TrapType::PolyTrap => 20,
        _ => 10,
    };

    prob -= difficulty;
    prob.clamp(5, 95) // 최소 5%, 최대 95%
}

// ============================================================
// [v2.19.0] 19. arrow_trap_hit — 화살/다트 함정 명중
// 원본: trap.c thitm() ~L5250
// ============================================================

/// [v2.19.0] 화살/다트 함정 명중 판정 (원본: thitm ~L5250)
/// AC 기반 명중 판정
pub fn arrow_trap_hit(target_ac: i32, rng: &mut NetHackRng) -> bool {
    // 원본: rnd(20) + 4 > AC
    // 명중 기준: d20 + 4 vs AC (낮을수록 방어 좋음)
    let roll = rng.rnd(20) + 4;
    roll > target_ac
}

/// [v2.19.0] 화살 함정 데미지 (원본: dotrap ~L950)
pub fn arrow_trap_damage(rng: &mut NetHackRng) -> i32 {
    rng.d(1, 6) // 1d6
}

/// [v2.19.0] 다트 함정 데미지 (원본: dotrap ~L980)
pub fn dart_trap_damage(rng: &mut NetHackRng) -> i32 {
    rng.d(1, 3) // 1d3
}

// ============================================================
// [v2.19.0] 20. poly_trap_result — 변이 함정 결과
// 원본: trap.c dotrap() L1624-1650
// ============================================================

/// 변이 함정 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolyTrapResult {
    /// 뉴 폼 (변칙적 변신)
    Polymorph,
    /// 마법 저항으로 무효화
    Resisted,
    /// 시스템 쇼크 (4% 확률 사망)
    SystemShock,
}

/// [v2.19.0] 변이 함정 결과 (원본: dotrap ~L1624)
pub fn poly_trap_result(
    has_magic_resistance: bool,
    has_unchanging: bool,
    rng: &mut NetHackRng,
) -> PolyTrapResult {
    if has_magic_resistance || has_unchanging {
        return PolyTrapResult::Resisted;
    }
    // 원본: 1/25 확률로 시스템 쇼크 (사망 가능)
    if rng.rn2(25) == 0 {
        PolyTrapResult::SystemShock
    } else {
        PolyTrapResult::Polymorph
    }
}

// ============================================================
// [v2.19.0] 21. level_tele_trap — 레벨 텔레포트 함정
// 원본: trap.c dotrap() L1195-1220
// ============================================================

/// [v2.19.0] 레벨 텔레포트 함정 목적지 (원본: dotrap ~L1195)
/// 현재 레벨에서 무작위 레벨로 이동
pub fn level_tele_destination(
    current_level: i32,
    max_level: i32,
    has_tele_control: bool,
    rng: &mut NetHackRng,
) -> i32 {
    if has_tele_control {
        // 컨트롤이 있으면 현재 레벨로 (호출자가 입력 요청)
        current_level
    } else {
        // 무작위 레벨 (원본: rnd(max) 후 보정)
        let dest = rng.rnd(max_level);
        if dest == current_level {
            // 같은 레벨이면 재도전
            (current_level + 1).min(max_level)
        } else {
            dest
        }
    }
}

// ============================================================
// [v2.19.0] 22. steed_trap_check — 탈 것 함정 판정
// 원본: trap.c steedintrap() ~L5100
// ============================================================

/// [v2.19.0] 탈 것 함정 판정 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteedTrapResult {
    /// 탈 것이 함정에 걸림
    SteedTrapped,
    /// 기수가 낙마
    RiderUnseated,
    /// 함정 통과
    Passed,
}

/// [v2.19.0] 탈 것 함정 판정 (원본: steedintrap ~L5100)
pub fn steed_trap_check(
    trap_type: TrapType,
    steed_flying: bool,
    steed_levitating: bool,
    rng: &mut NetHackRng,
) -> SteedTrapResult {
    // 비행/부양 탈 것은 구덩이 유형 회피
    if (steed_flying || steed_levitating)
        && matches!(
            trap_type,
            TrapType::Pit | TrapType::SpikedPit | TrapType::Hole | TrapType::Trapdoor
        )
    {
        return SteedTrapResult::Passed;
    }

    match trap_type {
        TrapType::BearTrap => SteedTrapResult::SteedTrapped,
        TrapType::Pit | TrapType::SpikedPit => {
            // 50% 확률로 낙마
            if rng.rn2(2) == 0 {
                SteedTrapResult::RiderUnseated
            } else {
                SteedTrapResult::SteedTrapped
            }
        }
        TrapType::Web => {
            // 큰 탈 것은 걸림
            SteedTrapResult::SteedTrapped
        }
        _ => SteedTrapResult::Passed,
    }
}

// ============================================================
// [v2.19.0] 23. squeaky_board_note — 삐걱이는 판자 노트
// 원본: trap.c trapnote() ~L350
// ============================================================

/// [v2.19.0] 삐걱이는 판자 음계 노트 (원본: trapnote ~L350)
/// 12개 노트 중 하나 반환 (C3~B3)
pub fn squeaky_board_note(note_index: i32) -> &'static str {
    const NOTES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let idx = (note_index.abs() % 12) as usize;
    NOTES[idx]
}

/// [v2.19.0] 삐걱이는 판자 음계 노트 랜덤 선택
pub fn random_squeaky_note(rng: &mut NetHackRng) -> &'static str {
    squeaky_board_note(rng.rn2(12))
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

    // --- erode_check ---
    #[test]
    fn test_erode_greased() {
        let mut rng = test_rng();
        let result = erode_check(
            ErodeType::Rust,
            true,
            true,
            true,
            false,
            false,
            0,
            false,
            0,
            &mut rng,
        );
        assert_eq!(result, ErodeResult::Greased);
    }

    #[test]
    fn test_erode_not_vulnerable() {
        let mut rng = test_rng();
        let result = erode_check(
            ErodeType::Rust,
            false,
            false,
            false,
            false,
            false,
            0,
            false,
            0,
            &mut rng,
        );
        assert_eq!(result, ErodeResult::Nothing);
    }

    #[test]
    fn test_erode_erodeproof() {
        let mut rng = test_rng();
        let result = erode_check(
            ErodeType::Rust,
            false,
            false,
            true,
            true,
            false,
            0,
            false,
            0,
            &mut rng,
        );
        assert_eq!(result, ErodeResult::Nothing);
    }

    #[test]
    fn test_erode_damaged() {
        let mut rng = test_rng();
        let result = erode_check(
            ErodeType::Rust,
            false,
            false,
            true,
            false,
            false,
            0,
            false,
            0,
            &mut rng,
        );
        assert_eq!(result, ErodeResult::Damaged);
    }

    #[test]
    fn test_erode_max_destroy() {
        let mut rng = test_rng();
        let result = erode_check(
            ErodeType::Burn,
            false,
            false,
            true,
            false,
            false,
            MAX_ERODE,
            true,
            0,
            &mut rng,
        );
        assert_eq!(result, ErodeResult::Destroyed);
    }

    // --- grease_wearoff ---
    #[test]
    fn test_grease_50_50() {
        let mut rng = test_rng();
        let mut worn = 0;
        for _ in 0..200 {
            if grease_wearoff(&mut rng) {
                worn += 1;
            }
        }
        assert!(worn > 60 && worn < 140, "그리스 벗겨짐: {}", worn);
    }

    // --- burnarmor_slot ---
    #[test]
    fn test_burnarmor_all_slots() {
        let mut rng = test_rng();
        let mut slots = std::collections::HashSet::new();
        for _ in 0..500 {
            slots.insert(format!("{:?}", burnarmor_slot(&mut rng)));
        }
        assert_eq!(slots.len(), 5, "5가지 슬롯 모두 발생");
    }

    // --- trap_escape_check ---
    #[test]
    fn test_escape_levitating() {
        let mut rng = test_rng();
        let result = trap_escape_check(true, false, false, false, true, false, false, &mut rng);
        assert_eq!(result, TrapEscapeResult::FloatOver);
    }

    #[test]
    fn test_escape_sokoban() {
        let mut rng = test_rng();
        let result = trap_escape_check(false, false, false, false, true, false, true, &mut rng);
        assert_eq!(result, TrapEscapeResult::Triggered);
    }

    // --- bear_trap ---
    #[test]
    fn test_bear_trap_duration() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dur = bear_trap_duration(&mut rng);
            assert!(dur >= 4 && dur < 8, "곰 턴: {}", dur);
        }
    }

    #[test]
    fn test_bear_trap_dmg() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dmg = bear_trap_damage(&mut rng);
            assert!(dmg >= 2 && dmg <= 8, "곰 데미지: {}", dmg);
        }
    }

    // --- pit_damage ---
    #[test]
    fn test_pit_spiked() {
        let mut rng = test_rng();
        let mut spiked_total = 0;
        let mut normal_total = 0;
        for _ in 0..100 {
            spiked_total += pit_damage(true, &mut rng);
            normal_total += pit_damage(false, &mut rng);
        }
        // 가시 구덩이가 평균적으로 더 높은 데미지
        assert!(spiked_total > normal_total, "가시 > 일반");
    }

    // --- dart_poison ---
    #[test]
    fn test_dart_poison() {
        let mut rng = test_rng();
        let mut poisoned = 0;
        for _ in 0..600 {
            if dart_poison_chance(&mut rng) {
                poisoned += 1;
            }
        }
        // ~16.7% 확률
        assert!(poisoned > 60 && poisoned < 140, "독 다트: {}", poisoned);
    }

    // --- sleep_gas ---
    #[test]
    fn test_sleep_gas() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dur = sleep_gas_duration(&mut rng);
            assert!(dur >= 1 && dur <= 25, "수면 턴: {}", dur);
        }
    }

    // --- rock_trap ---
    #[test]
    fn test_rock_helmet() {
        let mut rng = test_rng();
        assert_eq!(rock_trap_damage(true, &mut rng), 2);
    }

    #[test]
    fn test_rock_no_helmet() {
        let mut rng = test_rng();
        let dmg = rock_trap_damage(false, &mut rng);
        assert!(dmg >= 2 && dmg <= 12, "바위 데미지: {}", dmg);
    }

    // --- fall_through_depth ---
    #[test]
    fn test_fall_depth() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let depth = fall_through_depth(5, 25, &mut rng);
            assert!(depth >= 6 && depth <= 25, "낙하: {}", depth);
        }
    }

    #[test]
    fn test_fall_at_bottom() {
        let mut rng = test_rng();
        let depth = fall_through_depth(24, 25, &mut rng);
        assert_eq!(depth, 25);
    }

    // --- arrow_trap_empty ---
    #[test]
    fn test_arrow_not_fired() {
        let mut rng = test_rng();
        assert!(!arrow_trap_empty(false, &mut rng));
    }

    #[test]
    fn test_arrow_fired_rare_empty() {
        let mut rng = test_rng();
        let mut empties = 0;
        for _ in 0..1500 {
            if arrow_trap_empty(true, &mut rng) {
                empties += 1;
            }
        }
        // ~6.7% 확률
        assert!(empties > 50 && empties < 170, "빈 함정: {}", empties);
    }

    // --- rust_trap_slot ---
    #[test]
    fn test_rust_slots() {
        let mut rng = test_rng();
        let mut body_count = 0;
        for _ in 0..500 {
            if rust_trap_slot(&mut rng) == RustTrapSlot::Body {
                body_count += 1;
            }
        }
        // Body = 2/5 = 40%
        assert!(body_count > 140 && body_count < 260, "몸: {}", body_count);
    }

    // ============================================================
    // [v2.19.0] 추가 함수 테스트
    // ============================================================

    // --- magic_trap_effect ---
    #[test]
    fn test_magic_trap_various() {
        let mut rng = test_rng();
        let mut effects = std::collections::HashSet::new();
        for _ in 0..500 {
            effects.insert(format!("{:?}", magic_trap_effect(&mut rng)));
        }
        // 다양한 효과 발생 확인
        assert!(effects.len() >= 5, "최소 5가지 효과: {:?}", effects);
    }

    // --- fire_trap_damage ---
    #[test]
    fn test_fire_trap_resist() {
        let mut rng = test_rng();
        assert_eq!(fire_trap_damage(true, &mut rng), 0);
    }

    #[test]
    fn test_fire_trap_no_resist() {
        let mut rng = test_rng();
        let dmg = fire_trap_damage(false, &mut rng);
        assert!(dmg >= 2 && dmg <= 8, "화염 데미지: {}", dmg);
    }

    // --- web_escape_check ---
    #[test]
    fn test_web_spider_escapes() {
        let mut rng = test_rng();
        assert!(web_escape_check(
            false, false, false, true, false, 10, &mut rng
        ));
    }

    #[test]
    fn test_web_amorphous_escapes() {
        let mut rng = test_rng();
        assert!(web_escape_check(
            true, false, false, false, false, 10, &mut rng
        ));
    }

    #[test]
    fn test_web_edged_weapon_escapes() {
        let mut rng = test_rng();
        assert!(web_escape_check(
            false, false, false, false, true, 10, &mut rng
        ));
    }

    // --- landmine ---
    #[test]
    fn test_landmine_full_damage() {
        let mut rng = test_rng();
        let dmg = landmine_damage(false, false, &mut rng);
        assert!(dmg >= 1 && dmg <= 16, "지뢰: {}", dmg);
    }

    #[test]
    fn test_landmine_flying_reduced() {
        let mut rng = test_rng();
        let mut flying_total = 0;
        let mut ground_total = 0;
        for _ in 0..200 {
            flying_total += landmine_damage(false, true, &mut rng);
            ground_total += landmine_damage(false, false, &mut rng);
        }
        assert!(flying_total < ground_total, "비행 < 지상");
    }

    #[test]
    fn test_landmine_pit_creation() {
        assert!(landmine_creates_pit(false, false));
        assert!(!landmine_creates_pit(true, false));
        assert!(!landmine_creates_pit(false, true));
    }

    // --- rolling_boulder ---
    #[test]
    fn test_boulder_damage() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dmg = rolling_boulder_damage(&mut rng);
            assert!(dmg >= 2 && dmg <= 20, "바위: {}", dmg);
        }
    }

    #[test]
    fn test_boulder_dodge_rare() {
        let mut rng = test_rng();
        let mut dodges = 0;
        for _ in 0..1000 {
            if rolling_boulder_dodge(&mut rng) {
                dodges += 1;
            }
        }
        // ~10% 확률
        assert!(dodges > 50 && dodges < 150, "회피: {}", dodges);
    }

    // --- untrap_prob ---
    #[test]
    fn test_untrap_rogue_bonus() {
        let prob_norm = untrap_prob(15, 10, false, TrapType::Arrow);
        let prob_rogue = untrap_prob(15, 10, true, TrapType::Arrow);
        assert!(prob_rogue > prob_norm, "도적 보너스");
    }

    #[test]
    fn test_untrap_min_max() {
        let prob_low = untrap_prob(3, 1, false, TrapType::Landmine);
        assert!(prob_low >= 5);
        let prob_high = untrap_prob(25, 30, true, TrapType::SqueakyBoard);
        assert!(prob_high <= 95);
    }

    // --- arrow_trap_hit ---
    #[test]
    fn test_arrow_hit_low_ac() {
        let mut rng = test_rng();
        let mut hits = 0;
        for _ in 0..200 {
            if arrow_trap_hit(-5, &mut rng) {
                hits += 1;
            }
        }
        // AC -5는 매우 좋은 방어, 거의 모든 화살이 명중
        assert!(hits > 150, "AC-5 명중: {}", hits);
    }

    #[test]
    fn test_arrow_damage_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dmg = arrow_trap_damage(&mut rng);
            assert!(dmg >= 1 && dmg <= 6, "화살 데미지: {}", dmg);
        }
    }

    #[test]
    fn test_dart_damage_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let dmg = dart_trap_damage(&mut rng);
            assert!(dmg >= 1 && dmg <= 3, "다트 데미지: {}", dmg);
        }
    }

    // --- poly_trap ---
    #[test]
    fn test_poly_trap_resist() {
        let mut rng = test_rng();
        assert_eq!(
            poly_trap_result(true, false, &mut rng),
            PolyTrapResult::Resisted
        );
        assert_eq!(
            poly_trap_result(false, true, &mut rng),
            PolyTrapResult::Resisted
        );
    }

    #[test]
    fn test_poly_trap_mostly_polymorph() {
        let mut rng = test_rng();
        let mut poly_count = 0;
        let mut shock_count = 0;
        for _ in 0..500 {
            match poly_trap_result(false, false, &mut rng) {
                PolyTrapResult::Polymorph => poly_count += 1,
                PolyTrapResult::SystemShock => shock_count += 1,
                _ => {}
            }
        }
        assert!(poly_count > shock_count * 10, "변이 >> 쇼크");
    }

    // --- level_tele ---
    #[test]
    fn test_level_tele_control() {
        let mut rng = test_rng();
        assert_eq!(level_tele_destination(5, 25, true, &mut rng), 5);
    }

    #[test]
    fn test_level_tele_random() {
        let mut rng = test_rng();
        let dest = level_tele_destination(5, 25, false, &mut rng);
        assert!(dest >= 1 && dest <= 25, "목적지: {}", dest);
    }

    // --- steed_trap ---
    #[test]
    fn test_steed_flying_passes_pit() {
        let mut rng = test_rng();
        assert_eq!(
            steed_trap_check(TrapType::Pit, true, false, &mut rng),
            SteedTrapResult::Passed
        );
    }

    #[test]
    fn test_steed_bear_trap() {
        let mut rng = test_rng();
        assert_eq!(
            steed_trap_check(TrapType::BearTrap, false, false, &mut rng),
            SteedTrapResult::SteedTrapped
        );
    }

    #[test]
    fn test_steed_web() {
        let mut rng = test_rng();
        assert_eq!(
            steed_trap_check(TrapType::Web, false, false, &mut rng),
            SteedTrapResult::SteedTrapped
        );
    }

    // --- squeaky_board ---
    #[test]
    fn test_squeaky_notes() {
        assert_eq!(squeaky_board_note(0), "C");
        assert_eq!(squeaky_board_note(4), "E");
        assert_eq!(squeaky_board_note(11), "B");
        assert_eq!(squeaky_board_note(12), "C"); // 순환
    }

    #[test]
    fn test_random_squeaky() {
        let mut rng = test_rng();
        let note = random_squeaky_note(&mut rng);
        assert!(!note.is_empty());
    }
}

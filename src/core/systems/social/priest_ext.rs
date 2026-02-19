// =============================================================================
// AIHack - priest_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// [v2.11.0] priest.c 핵심 로직 순수 결과 패턴 이식
// 원본: nethack-3.6.7/src/priest.c (1,100줄)
//
// 이식된 로직:
//   1. mon_aligntyp — 몬스터 성향 판정 (priest.c:274-285)
//   2. align_str — 성향 문자열 (priest.c:874-889)
//   3. piousness_str — 경건도 문자열 (priest.c:891-931)
//   4. p_coaligned — 성향 일치 판정 (priest.c:347-352)
//   5. priestname_result — 사제 이름 생성 (priest.c:297-345)
//   6. in_your_sanctuary_check — 성역 판정 (priest.c:697-719)
//   7. priest_donation_result — 기부 결과 판정 (priest.c:578-643)
//   8. ghod_direction — 신벌 방향 계산 (priest.c:721-792)
//   9. move_special_pick — 사제/상인 이동 목표 선택 (priest.c:95-111)
//  10. temple_entry_message — 사원 입장 메시지 판정 (priest.c:387-516)
//  11. angry_priest_convert — 분노 사제 변환 판정 (priest.c:795-829)
//  12. mstatusline_info — 몬스터 상태 정보 문자열 (priest.c:933-1029)
// =============================================================================

// =============================================================================
// 1. mon_aligntyp — 몬스터 성향 판정
// [v2.11.0] priest.c:274-285 이식
// =============================================================================

/// 성향 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Lawful,
    Neutral,
    Chaotic,
    None, // 몰로크(Moloch) 등
}

/// 몬스터 성향 판정 (원본 priest.c:274-285)
/// 사제 → shrine 성향, 미니언 → min_align, 일반 → maligntyp
pub fn mon_aligntyp(
    is_priest: bool,
    is_minion: bool,
    shrine_align: i32,
    min_align: i32,
    maligntyp: i32,
) -> Alignment {
    let algn = if is_priest {
        shrine_align
    } else if is_minion {
        min_align
    } else {
        maligntyp
    };

    // A_NONE = -128 (원본), 여기서는 관례적으로 -128 사용
    if algn == -128 {
        return Alignment::None;
    }

    if algn > 0 {
        Alignment::Lawful
    } else if algn < 0 {
        Alignment::Chaotic
    } else {
        Alignment::Neutral
    }
}

// =============================================================================
// 2. align_str — 성향 문자열
// [v2.11.0] priest.c:874-889 이식
// =============================================================================

/// 성향을 문자열로 변환 (원본 priest.c:874-889)
pub fn align_str(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::Chaotic => "chaotic",
        Alignment::Neutral => "neutral",
        Alignment::Lawful => "lawful",
        Alignment::None => "unaligned",
    }
}

// =============================================================================
// 3. piousness_str — 경건도 문자열
// [v2.11.0] priest.c:891-931 이식
// =============================================================================

/// 경건도(alignment record)값에 따른 설명 문자열 (원본 priest.c:891-931)
/// show_negative: 음수 값을 구체적으로 표시할지 여부
pub fn piousness_str(record: i32, show_negative: bool) -> &'static str {
    if record >= 20 {
        "piously"
    } else if record > 13 {
        "devoutly"
    } else if record > 8 {
        "fervently"
    } else if record > 3 {
        "stridently"
    } else if record == 3 {
        ""
    } else if record > 0 {
        "haltingly"
    } else if record == 0 {
        "nominally"
    } else if !show_negative {
        "insufficiently"
    } else if record >= -3 {
        "strayed"
    } else if record >= -8 {
        "sinned"
    } else {
        "transgressed"
    }
}

// =============================================================================
// 4. p_coaligned — 성향 일치 판정
// [v2.11.0] priest.c:347-352 이식
// =============================================================================

/// 사제와 플레이어 성향 일치 여부 (원본 priest.c:347-352)
pub fn p_coaligned(player_align: Alignment, priest_align: Alignment) -> bool {
    player_align == priest_align
}

// =============================================================================
// 5. priestname_result — 사제 이름 생성
// [v2.11.0] priest.c:297-345 이식
// =============================================================================

/// 사제 이름 생성 결과
#[derive(Debug, Clone)]
pub struct PriestNameResult {
    pub name: String,
}

/// 사제/미니언 이름 생성 (원본 priest.c:297-345)
/// hallucination: 환각 여부 (환각 시 bogon 이름)
pub fn priestname_result(
    is_priest: bool,
    is_minion: bool,
    is_high_priest: bool,
    is_aligned_priest: bool,
    is_invisible: bool,
    is_female: bool,
    is_tame: bool,
    is_renegade: bool,
    hallucination: bool,
    monster_name: &str,
    god_name: &str,
    is_astral_level: bool,
    distance_to_player: i32,
    game_over: bool,
) -> PriestNameResult {
    if !is_priest && !is_minion {
        return PriestNameResult {
            name: monster_name.to_string(),
        };
    }

    let mut name = String::new();

    // "the " 접두사 (환각 시 일부 bogon은 자체 관사 있음 → 여기선 단순화)
    if !hallucination {
        name.push_str("the ");
    }

    if is_invisible {
        name.push_str("invisible ");
    }

    if is_minion && is_renegade {
        name.push_str("renegade ");
    }

    // 사제 계열
    if is_priest || is_aligned_priest {
        if !is_aligned_priest && !is_high_priest {
            // 변신한 사제: 원래 몬스터 이름 사용
            name.push_str(monster_name);
        } else {
            if is_high_priest {
                name.push_str("high ");
            }
            if hallucination {
                name.push_str("poohbah");
            } else if is_female {
                name.push_str("priestess");
            } else {
                name.push_str("priest");
            }
        }
    } else {
        // 미니언 (천사 등)
        if is_tame && monster_name.to_lowercase() == "angel" {
            name.push_str("guardian ");
        }
        name.push_str(monster_name);
    }

    // " of <신이름>" 접미사 (대부분의 경우)
    let show_god = hallucination
        || !is_high_priest
        || !is_astral_level
        || distance_to_player <= 2
        || game_over;

    if show_god {
        name.push_str(" of ");
        name.push_str(god_name);
    }

    PriestNameResult { name }
}

// =============================================================================
// 6. in_your_sanctuary_check — 성역 판정
// [v2.11.0] priest.c:697-719 이식
// =============================================================================

/// 성역 판정 (원본 priest.c:697-719)
/// 사원 안에 있고, 사제가 같은 성향이고, 성소가 온전하며, 플레이어가 죄를 짓지 않았을 때
pub fn in_your_sanctuary_check(
    is_minion_or_rider: bool,
    player_align_record: i32,
    player_in_temple: bool,
    target_in_same_temple: bool,
    has_shrine: bool,
    priest_coaligned: bool,
    priest_peaceful: bool,
) -> bool {
    // 미니언/기수는 성역 무효
    if is_minion_or_rider {
        return false;
    }
    // 죄를 지었으면 (-4 이하) 성역 무효
    if player_align_record <= -4 {
        return false;
    }
    // 사원에 있지 않으면 무효
    if !player_in_temple || !target_in_same_temple {
        return false;
    }

    has_shrine && priest_coaligned && priest_peaceful
}

// =============================================================================
// 7. priest_donation_result — 기부 결과 판정
// [v2.11.0] priest.c:578-643 이식
// =============================================================================

/// 기부 결과 타입
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DonationResult {
    /// 기부 거부 (regret 경고)
    Refused,
    /// 인색한 기부 ("Cheapskate")
    Cheapskate,
    /// 약간의 감사 (exercise wisdom)
    ThankYou,
    /// 경건한 기부 (약간의 보상 가능)
    Pious { can_bless: bool },
    /// 보호 보상 (Protection 증가)
    Protection { new_blessed: i32 },
    /// 매우 관대한 기부 (성향 정화 또는 +2)
    Generous { cleansed: bool },
}

/// 기부 결과 판정 (원본 priest.c:578-643)
pub fn priest_donation_result(
    offer: i64,
    player_level: i32,
    player_gold: i64,
    coaligned: bool,
    strayed: bool,
    align_record: i32,
    protection_active: bool,
    ublessed: i32,
    turns_since_cleansed: i64,
    rng_blessed: i32,
) -> DonationResult {
    if offer == 0 {
        return DonationResult::Refused;
    }

    let threshold_200 = player_level as i64 * 200;
    let threshold_400 = player_level as i64 * 400;
    let threshold_600 = player_level as i64 * 600;

    if offer < threshold_200 {
        if player_gold > offer * 2 {
            DonationResult::Cheapskate
        } else {
            DonationResult::ThankYou
        }
    } else if offer < threshold_400 {
        let can_bless = player_gold < offer * 2 && coaligned && align_record <= -4;
        DonationResult::Pious { can_bless }
    } else if offer < threshold_600 {
        // Protection 보상 조건
        if !protection_active || (ublessed < 20 && (ublessed < 9 || rng_blessed == 0)) {
            let new_blessed = if !protection_active {
                rng_blessed.max(2) // rn1(3,2) 범위 [2,4]
            } else {
                ublessed + 1
            };
            DonationResult::Protection { new_blessed }
        } else {
            // Protection 조건 미충족이면 Generous로 fall through
            DonationResult::Generous { cleansed: false }
        }
    } else {
        // 매우 관대한 기부
        let cleansed =
            player_gold < offer * 2 && coaligned && strayed && turns_since_cleansed > 5000;
        DonationResult::Generous { cleansed }
    }
}

// =============================================================================
// 8. ghod_direction — 신벌 방향 계산
// [v2.11.0] priest.c:721-792에서 방향 판정 로직 이식
// =============================================================================

/// 신벌 번개 시작 위치와 방향 판정 결과
#[derive(Debug, Clone)]
pub struct GhodDirection {
    /// 번개 시작 X
    pub start_x: i32,
    /// 번개 시작 Y
    pub start_y: i32,
    /// X 방향 (-1/0/1)
    pub dx: i32,
    /// Y 방향 (-1/0/1)
    pub dy: i32,
    /// 유효한 공격인지
    pub valid: bool,
}

/// 신벌 방향 계산 (원본 priest.c:736-773)
/// 사원 제단에서 플레이어를 향한 번개 방향
pub fn ghod_direction(
    altar_x: i32,
    altar_y: i32,
    player_x: i32,
    player_y: i32,
    room_lx: i32,
    room_ly: i32,
    room_hx: i32,
    room_hy: i32,
    player_on_altar: bool,
    player_lined_up: bool,
    player_at_door: bool,
    door_side: i32,   // 0=left, 1=right, 2=top, 3=bottom
    random_side: i32, // rn2(4) 결과
) -> GhodDirection {
    let mut x = altar_x;
    let mut y = altar_y;

    if player_on_altar || !player_lined_up {
        if player_at_door {
            // 문에서의 방향 결정
            match door_side {
                0 => {
                    x = room_hx;
                    y = player_y;
                }
                1 => {
                    x = room_lx;
                    y = player_y;
                }
                2 => {
                    x = player_x;
                    y = room_hy;
                }
                3 => {
                    x = player_x;
                    y = room_ly;
                }
                _ => {}
            }
        } else {
            // 무작위 벽에서
            match random_side {
                0 => {
                    x = player_x;
                    y = room_ly;
                }
                1 => {
                    x = player_x;
                    y = room_hy;
                }
                2 => {
                    x = room_lx;
                    y = player_y;
                }
                _ => {
                    x = room_hx;
                    y = player_y;
                }
            }
        }
    }

    // 방향 계산 (시작점 → 플레이어)
    let dx = (player_x - x).signum();
    let dy = (player_y - y).signum();

    // 직선 여부 재확인 (원래는 linedup 체크)
    let valid = dx != 0 || dy != 0;

    GhodDirection {
        start_x: x,
        start_y: y,
        dx,
        dy,
        valid,
    }
}

// =============================================================================
// 9. move_special_pick — 사제/상인 이동 목표 선택 (최적 위치)
// [v2.11.0] priest.c:95-111 이식 (순수 위치 비교 로직만)
// =============================================================================

/// 이동 후보 중 목표에 가장 가까운 위치 선택 (원본 priest.c:95-111)
/// positions: (x, y, 방에 있는지, notonl 여부) 후보 목록
/// goal_x, goal_y: 목표 위치
/// avoid: 플레이어와의 직선을 피할지
/// approach: 목표에 접근할지 (true=접근, false=무작위)
pub fn move_special_pick(
    positions: &[(i32, i32, bool, bool)],
    goal_x: i32,
    goal_y: i32,
    avoid: bool,
    approach: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> Option<(i32, i32)> {
    let mut best: Option<(i32, i32)> = None;
    let mut best_dist = i32::MAX;
    let mut count = 0;

    for &(nx, ny, in_room, notonl) in positions {
        if !in_room {
            continue;
        }
        if avoid && notonl {
            continue;
        }

        let dist = (nx - goal_x) * (nx - goal_x) + (ny - goal_y) * (ny - goal_y);

        if !approach {
            // 무작위 선택
            count += 1;
            if rng.rn2(count) == 0 {
                best = Some((nx, ny));
            }
        } else if dist < best_dist {
            best_dist = dist;
            best = Some((nx, ny));
        }
    }

    best
}

// =============================================================================
// 10. temple_entry_message — 사원 입장 메시지 판정
// [v2.11.0] priest.c:387-516에서 메시지 결정 로직 이식
// =============================================================================

/// 사원 입장 메시지 타입
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TempleEntryMessage {
    /// 몰로크의 성역 — 불신앙자 경고
    SanctumInfidel,
    /// 성역 재방문 — 불경한 존재
    SanctumDesecrate,
    /// 신성한 / 모독된 장소
    PilgrimEntry { sacred: bool },
    /// 적대적 느낌 (모독 or 비동맹)
    Forbidding { strange: bool },
    /// 평온한 장소 (동맹+ 경건)
    Peaceful { unusual: bool },
    /// 관리인 없는 사원 (으스스)
    UntendedEerie,
    /// 메시지 없음
    NoMessage,
}

/// 사원 입장 메시지 결정 (원본 priest.c:387-516)
pub fn temple_entry_message(
    has_priest: bool,
    is_sanctum: bool,
    priest_peaceful: bool,
    has_shrine: bool,
    coaligned: bool,
    align_record: i32,
) -> TempleEntryMessage {
    if has_priest {
        if is_sanctum {
            if priest_peaceful {
                TempleEntryMessage::SanctumInfidel
            } else {
                TempleEntryMessage::SanctumDesecrate
            }
        } else if !has_shrine || !coaligned || align_record <= -4 {
            let strange = has_shrine && coaligned;
            TempleEntryMessage::Forbidding { strange }
        } else if align_record >= 14 {
            TempleEntryMessage::Peaceful { unusual: false }
        } else {
            TempleEntryMessage::Peaceful { unusual: true }
        }
    } else {
        TempleEntryMessage::UntendedEerie
    }
}

// =============================================================================
// 11. angry_priest_convert — 분노 사제 변환 판정
// [v2.11.0] priest.c:795-829 이식
// =============================================================================

/// 사제가 방랑자(roamer/minion)로 변환되는지 판정 (원본 priest.c:804-827)
/// 제단이 파괴/변환되면 ispriest를 해제하고 isminion으로 전환
pub fn should_convert_to_roamer(altar_exists: bool, altar_alignment_matches: bool) -> bool {
    !altar_exists || !altar_alignment_matches
}

// =============================================================================
// 12. mstatusline_info — 몬스터 상태 정보 문자열 조립
// [v2.11.0] priest.c:933-1029 이식 (순수 문자열 생성)
// =============================================================================

/// 몬스터 상태 정보
#[derive(Debug, Clone)]
pub struct MonsterStatusInfo {
    pub is_tame: bool,
    pub is_peaceful: bool,
    pub is_cancelled: bool,
    pub is_confused: bool,
    pub is_blind: bool,
    pub is_stunned: bool,
    pub is_sleeping: bool,
    pub is_frozen: bool,
    pub is_fleeing: bool,
    pub is_trapped: bool,
    pub is_fast: bool,
    pub is_slow: bool,
    pub is_invisible: bool,
    pub is_shapechanger: bool,
    pub is_eating: bool,
    pub is_meditating: bool,
}

/// 몬스터 상태 문자열 조립 (원본 priest.c:933-1029)
/// 쉼표로 구분된 상태 태그 리스트 반환
pub fn mstatusline_tags(info: &MonsterStatusInfo) -> Vec<&'static str> {
    let mut tags = Vec::new();

    if info.is_tame {
        tags.push("tame");
    } else if info.is_peaceful {
        tags.push("peaceful");
    }

    if info.is_shapechanger {
        tags.push("shapechanger");
    }
    if info.is_eating {
        tags.push("eating");
    }
    if info.is_cancelled {
        tags.push("cancelled");
    }
    if info.is_confused {
        tags.push("confused");
    }
    if info.is_blind {
        tags.push("blind");
    }
    if info.is_stunned {
        tags.push("stunned");
    }
    if info.is_sleeping {
        tags.push("asleep");
    } else if info.is_frozen {
        tags.push("can't move");
    } else if info.is_meditating {
        tags.push("meditating");
    }
    if info.is_fleeing {
        tags.push("scared");
    }
    if info.is_trapped {
        tags.push("trapped");
    }
    if info.is_fast {
        tags.push("fast");
    } else if info.is_slow {
        tags.push("slow");
    }
    if info.is_invisible {
        tags.push("invisible");
    }

    tags
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    // ─── mon_aligntyp ─────────────────────────────────────────
    #[test]
    fn test_aligntyp_priest_lawful() {
        assert_eq!(mon_aligntyp(true, false, 1, 0, 0), Alignment::Lawful);
    }

    #[test]
    fn test_aligntyp_minion_chaotic() {
        assert_eq!(mon_aligntyp(false, true, 0, -1, 0), Alignment::Chaotic);
    }

    #[test]
    fn test_aligntyp_normal_neutral() {
        assert_eq!(mon_aligntyp(false, false, 0, 0, 0), Alignment::Neutral);
    }

    #[test]
    fn test_aligntyp_none() {
        assert_eq!(mon_aligntyp(true, false, -128, 0, 0), Alignment::None);
    }

    // ─── align_str ────────────────────────────────────────────
    #[test]
    fn test_align_str() {
        assert_eq!(align_str(Alignment::Lawful), "lawful");
        assert_eq!(align_str(Alignment::Chaotic), "chaotic");
        assert_eq!(align_str(Alignment::Neutral), "neutral");
        assert_eq!(align_str(Alignment::None), "unaligned");
    }

    // ─── piousness_str ────────────────────────────────────────
    #[test]
    fn test_piousness_pious() {
        assert_eq!(piousness_str(20, false), "piously");
    }

    #[test]
    fn test_piousness_nominally() {
        assert_eq!(piousness_str(0, false), "nominally");
    }

    #[test]
    fn test_piousness_strayed() {
        assert_eq!(piousness_str(-2, true), "strayed");
    }

    #[test]
    fn test_piousness_sinned() {
        assert_eq!(piousness_str(-5, true), "sinned");
    }

    #[test]
    fn test_piousness_transgressed() {
        assert_eq!(piousness_str(-10, true), "transgressed");
    }

    // ─── p_coaligned ──────────────────────────────────────────
    #[test]
    fn test_coaligned() {
        assert!(p_coaligned(Alignment::Lawful, Alignment::Lawful));
        assert!(!p_coaligned(Alignment::Lawful, Alignment::Chaotic));
    }

    // ─── priestname_result ────────────────────────────────────
    #[test]
    fn test_priestname_high_priest() {
        let result = priestname_result(
            true,
            false,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            "high priest",
            "Amaterasu Omikami",
            false,
            10,
            false,
        );
        assert!(result.name.contains("high"));
        assert!(result.name.contains("priest"));
        assert!(result.name.contains("Amaterasu Omikami"));
    }

    #[test]
    fn test_priestname_priestess() {
        let result = priestname_result(
            true,
            false,
            false,
            true,
            false,
            true,
            false,
            false,
            false,
            "aligned priest",
            "Quetzalcoatl",
            false,
            5,
            false,
        );
        assert!(result.name.contains("priestess"));
    }

    #[test]
    fn test_priestname_guardian_angel() {
        let result = priestname_result(
            false,
            true,
            false,
            false,
            false,
            false,
            true,
            false,
            false,
            "Angel",
            "Shan Lai Ching",
            false,
            3,
            false,
        );
        assert!(result.name.contains("guardian"));
    }

    // ─── in_your_sanctuary_check ──────────────────────────────
    #[test]
    fn test_sanctuary_valid() {
        assert!(in_your_sanctuary_check(
            false, 5, true, true, true, true, true
        ));
    }

    #[test]
    fn test_sanctuary_sinned() {
        assert!(!in_your_sanctuary_check(
            false, -4, true, true, true, true, true
        ));
    }

    #[test]
    fn test_sanctuary_minion() {
        assert!(!in_your_sanctuary_check(
            true, 10, true, true, true, true, true
        ));
    }

    // ─── priest_donation_result ───────────────────────────────
    #[test]
    fn test_donation_refused() {
        assert_eq!(
            priest_donation_result(0, 10, 1000, true, false, 0, false, 0, 0, 0),
            DonationResult::Refused
        );
    }

    #[test]
    fn test_donation_cheapskate() {
        // offer=100 < level(10)*200=2000, gold(5000)>200
        assert_eq!(
            priest_donation_result(100, 10, 5000, true, false, 0, false, 0, 0, 0),
            DonationResult::Cheapskate
        );
    }

    #[test]
    fn test_donation_protection() {
        // offer=4500 >= level(10)*400=4000, < 6000
        match priest_donation_result(4500, 10, 10000, true, false, 0, false, 0, 0, 3) {
            DonationResult::Protection { new_blessed } => assert!(new_blessed >= 2),
            other => panic!("기대값: Protection, 실제: {:?}", other),
        }
    }

    // ─── ghod_direction ───────────────────────────────────────
    #[test]
    fn test_ghod_direction_lined_up() {
        let dir = ghod_direction(5, 5, 5, 10, 1, 1, 10, 10, false, true, false, 0, 0);
        assert!(dir.valid);
        assert_eq!(dir.dx, 0);
        assert_eq!(dir.dy, 1);
    }

    // ─── move_special_pick ────────────────────────────────────
    #[test]
    fn test_move_special_approach() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let positions = vec![
            (3, 3, true, false),
            (5, 5, true, false),
            (2, 2, true, false),
        ];
        let result = move_special_pick(&positions, 1, 1, false, true, &mut rng);
        assert_eq!(result, Some((2, 2))); // (2,2)가 (1,1)에 가장 가까움
    }

    // ─── temple_entry_message ─────────────────────────────────
    #[test]
    fn test_temple_sanctum_infidel() {
        assert_eq!(
            temple_entry_message(true, true, true, true, true, 10),
            TempleEntryMessage::SanctumInfidel
        );
    }

    #[test]
    fn test_temple_peaceful() {
        assert_eq!(
            temple_entry_message(true, false, true, true, true, 14),
            TempleEntryMessage::Peaceful { unusual: false }
        );
    }

    #[test]
    fn test_temple_untended() {
        assert_eq!(
            temple_entry_message(false, false, false, false, false, 0),
            TempleEntryMessage::UntendedEerie
        );
    }

    // ─── should_convert_to_roamer ─────────────────────────────
    #[test]
    fn test_convert_no_altar() {
        assert!(should_convert_to_roamer(false, false));
    }

    #[test]
    fn test_no_convert_valid_altar() {
        assert!(!should_convert_to_roamer(true, true));
    }

    // ─── mstatusline_tags ─────────────────────────────────────
    #[test]
    fn test_status_tame_confused() {
        let info = MonsterStatusInfo {
            is_tame: true,
            is_peaceful: false,
            is_cancelled: false,
            is_confused: true,
            is_blind: false,
            is_stunned: false,
            is_sleeping: false,
            is_frozen: false,
            is_fleeing: false,
            is_trapped: false,
            is_fast: true,
            is_slow: false,
            is_invisible: false,
            is_shapechanger: false,
            is_eating: false,
            is_meditating: false,
        };
        let tags = mstatusline_tags(&info);
        assert!(tags.contains(&"tame"));
        assert!(tags.contains(&"confused"));
        assert!(tags.contains(&"fast"));
        assert!(!tags.contains(&"peaceful"));
    }

    #[test]
    fn test_status_sleeping_not_frozen() {
        let info = MonsterStatusInfo {
            is_tame: false,
            is_peaceful: true,
            is_cancelled: false,
            is_confused: false,
            is_blind: false,
            is_stunned: false,
            is_sleeping: true,
            is_frozen: true,
            is_fleeing: false,
            is_trapped: false,
            is_fast: false,
            is_slow: false,
            is_invisible: false,
            is_shapechanger: false,
            is_eating: false,
            is_meditating: false,
        };
        let tags = mstatusline_tags(&info);
        assert!(tags.contains(&"asleep"));
        assert!(!tags.contains(&"can't move")); // 수면이 우선
    }
}

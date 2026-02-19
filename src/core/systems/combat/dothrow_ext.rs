// =============================================================================
// AIHack - dothrow_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// [v2.11.0] dothrow.c 핵심 로직 순수 결과 패턴 이식
// 원본: nethack-3.6.7/src/dothrow.c (2,189줄)
//
// 이식된 로직:
//   1. multishot_count — 연사 횟수 계산 (스킬/역할/종족 보너스, dothrow.c:107-201)
//   2. throwing_weapon_check — 투척 무기 판정 (dothrow.c:1060-1071)
//   3. autoquiver_priority — 자동 화살집 우선순위 판정 (dothrow.c:286-348)
//   4. throw_range — 투척 사거리 계산 (dothrow.c:1206-1260)
//   5. walk_path_bresenham — Bresenham 직선 경로 (dothrow.c:452-531)
//   6. hurtle_blocked — 돌진 차단 판정 (dothrow.c:570-654)
//   7. slip_chance — 저주/기름 미끄러짐 확률 (dothrow.c:1108-1129)
//   8. toss_up_damage — 위로 던진 물건의 낙하 데미지 (dothrow.c:1003-1028)
//   9. omon_adj — 투척 명중 보정치 (dothrow.c:1446-1487)
//  10. gem_accept_result — 보석 선물 반응 (dothrow.c 하단)
// =============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// 1. multishot_count — 연사 횟수 계산
// [v2.11.0] dothrow.c:107-201 이식
// =============================================================================

/// 연사 판정에 필요한 입력 정보
#[derive(Debug, Clone)]
pub struct MultishotInfo {
    /// 아이템 수량
    pub quantity: i64,
    /// 탄약인지 여부 (is_ammo)
    pub is_ammo: bool,
    /// 발사대(활/석궁)와 매칭되는지 (ammo_and_launcher)
    pub matching_launcher: bool,
    /// 무기 클래스 여부 (WEAPON_CLASS)
    pub is_weapon_class: bool,
    /// 혼란 상태
    pub confused: bool,
    /// 기절 상태
    pub stunned: bool,
    /// 투척 스킬 레벨 (0=미숙련, 1=기본, 2=숙련, 3=전문)
    pub skill_level: i32,
    /// 역할 (예: "Caveman", "Ranger" 등)
    pub role: &'static str,
    /// 종족 (예: "Elf", "Orc" 등)
    pub race: &'static str,
    /// 무기 스킬 종류 (음수=발사대계열, 양수=근접계열)
    pub weapon_skill: i32,
    /// 손재주 (DEX)
    pub dexterity: i32,
    /// 더듬거림(Fumbling) 여부
    pub fumbling: bool,
    /// 석궁+탄약 조합인지
    pub is_crossbow_ammo: bool,
    /// 힘(STR)
    pub strength: i32,
    /// 노움 여부
    pub is_gnome: bool,
    /// 종족 전용 화살+활 매칭 (예: 엘프 화살+엘프 활)
    pub racial_ammo_match: bool,
    /// 사무라이 야+유미 조합
    pub samurai_ya_yumi: bool,
    /// 사용자 지정 발사 제한
    pub shot_limit: i32,
}

/// 연사 횟수 계산 (원본 dothrow.c:107-201)
/// 스킬 숙련도, 역할 보너스, 종족 보너스를 모두 적용
pub fn multishot_count(info: &MultishotInfo, rng: &mut NetHackRng) -> i32 {
    // 기본 1발
    let mut multishot = 1;

    // 수량이 1 이하이거나 조건 미충족 시 1발
    if info.quantity <= 1 {
        return 1;
    }
    // 탄약은 매칭 발사대 필요, 아니면 무기 클래스여야 함
    if !(if info.is_ammo {
        info.matching_launcher
    } else {
        info.is_weapon_class
    }) {
        return 1;
    }
    // 혼란/기절 시 1발
    if info.confused || info.stunned {
        return 1;
    }

    // 약한 연사: 마법사/사제/치료사(칼 제외)/관광객(다트 제외)/손재주 6 이하/더듬거림
    let weak = info.role == "Wizard"
        || info.role == "Priest"
        || (info.role == "Healer" && info.weapon_skill != 6) // P_KNIFE=6
        || (info.role == "Tourist" && info.weapon_skill != -7) // -P_DART=-7
        || info.fumbling
        || info.dexterity <= 6;

    // 스킬 보너스
    match info.skill_level {
        3 => {
            // 전문(Expert)
            multishot += 1;
            if !weak {
                multishot += 1;
            }
        }
        2 => {
            // 숙련(Skilled)
            if !weak {
                multishot += 1;
            }
        }
        _ => {} // 기본/미숙련: 보너스 없음
    }

    // 역할 보너스
    match info.role {
        "Caveman" => {
            // 투석기(-P_SLING) 또는 창(P_SPEAR) 보너스
            if info.weapon_skill == -1 || info.weapon_skill == 5 {
                multishot += 1;
            }
        }
        "Monk" => {
            if info.weapon_skill == -8 {
                // -P_SHURIKEN
                multishot += 1;
            }
        }
        "Ranger" => {
            if info.weapon_skill != 1 {
                // P_DAGGER=1 제외
                multishot += 1;
            }
        }
        "Rogue" => {
            if info.weapon_skill == 1 {
                // P_DAGGER
                multishot += 1;
            }
        }
        "Samurai" => {
            if info.samurai_ya_yumi {
                multishot += 1;
            }
        }
        _ => {}
    }

    // 종족 보너스 (약한 연사가 아닐 때만)
    if !weak && info.racial_ammo_match {
        multishot += 1;
    }

    // 석궁은 높은 힘이 필요 (비노움 18, 노움 16)
    if multishot > 1 && info.is_crossbow_ammo {
        let threshold = if info.is_gnome { 16 } else { 18 };
        if info.strength < threshold {
            multishot = rng.rnd(multishot);
        }
    }

    // 최종 랜덤화
    multishot = rng.rnd(multishot);

    // 수량 초과 방지
    if (multishot as i64) > info.quantity {
        multishot = info.quantity as i32;
    }
    // 사용자 제한
    if info.shot_limit > 0 && multishot > info.shot_limit {
        multishot = info.shot_limit;
    }

    multishot
}

// =============================================================================
// 2. throwing_weapon_check — 투척 무기 판정
// [v2.11.0] dothrow.c:1060-1071 이식
// =============================================================================

/// 투척 무기 판정 정보
#[derive(Debug, Clone)]
pub struct ThrowWeaponInfo {
    /// 미사일(다트, 수리검 등)
    pub is_missile: bool,
    /// 창 종류
    pub is_spear: bool,
    /// 도검류(blade)
    pub is_blade: bool,
    /// 검 (is_sword)
    pub is_sword: bool,
    /// PIERCE 방향 설정
    pub has_pierce: bool,
    /// 워해머
    pub is_war_hammer: bool,
    /// 아클리스 (밧줄 달린 무기)
    pub is_aklys: bool,
}

/// 투척용 무기인지 판정 (탄약 제외) (원본 dothrow.c:1060-1071)
pub fn throwing_weapon_check(info: &ThrowWeaponInfo) -> bool {
    info.is_missile
        || info.is_spear
        // 단검/나이프 (검 제외, 관통 방향)
        || (info.is_blade && !info.is_sword && info.has_pierce)
        // 특수 케이스
        || info.is_war_hammer
        || info.is_aklys
}

// =============================================================================
// 3. autoquiver_priority — 자동 화살집 우선순위 판정
// [v2.11.0] dothrow.c:286-348 이식
// =============================================================================

/// 화살집 후보 우선순위
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuiverPriority {
    /// 최우선: 장비된 발사대에 맞는 탄약
    MatchedAmmo = 4,
    /// 미사일 (다트, 수리검 등)
    Missile = 3,
    /// 보조 무기에 맞는 탄약
    AltAmmo = 2,
    /// 기타 투척 가능 아이템
    Misc = 1,
    /// 해당 없음
    None = 0,
}

/// 아이템의 화살집 우선순위 판정
pub fn autoquiver_priority(
    is_worn: bool,
    is_artifact: bool,
    is_identified: bool,
    is_ammo: bool,
    is_missile: bool,
    is_throwing: bool,
    is_dagger: bool,
    is_rock_or_flint_or_glass: bool,
    is_gem_class: bool,
    using_sling: bool,
    matches_primary_launcher: bool,
    matches_swap_launcher: bool,
) -> QuiverPriority {
    // 장비 중이거나 아티팩트이거나 미감정이면 제외
    if is_worn || is_artifact || !is_identified {
        return QuiverPriority::None;
    }

    // 돌/부싯돌/유리 → 투석기 사용 시 최우선, 아니면 보조/기타
    if is_rock_or_flint_or_glass {
        if using_sling {
            return QuiverPriority::MatchedAmmo;
        } else if matches_swap_launcher {
            return QuiverPriority::AltAmmo;
        } else {
            return QuiverPriority::Misc;
        }
    }

    // 보석류(비돌) → 무시
    if is_gem_class {
        return QuiverPriority::None;
    }

    // 탄약
    if is_ammo {
        if matches_primary_launcher {
            return QuiverPriority::MatchedAmmo;
        } else if matches_swap_launcher {
            return QuiverPriority::AltAmmo;
        } else {
            return QuiverPriority::Misc;
        }
    }

    // 미사일
    if is_missile {
        return QuiverPriority::Missile;
    }

    // 투척 무기 (단검 → 미사일 취급)
    if is_throwing {
        if is_dagger {
            return QuiverPriority::Missile;
        }
        return QuiverPriority::Misc;
    }

    QuiverPriority::None
}

// =============================================================================
// 4. throw_range — 투척 사거리 계산
// [v2.11.0] dothrow.c:1206-1260 이식
// =============================================================================

/// 투척 사거리 입력 정보
#[derive(Debug, Clone)]
pub struct ThrowRangeInfo {
    /// 힘 기반 사거리 (ACURRSTR/2, 석궁은 18/2=9)
    pub base_range: i32,
    /// 석궁+탄약 조합
    pub is_crossbow_ammo: bool,
    /// 아이템 무게 (owt)
    pub weight: i32,
    /// 쇠구슬 여부
    pub is_heavy_iron_ball: bool,
    /// 바위 여부
    pub is_boulder: bool,
    /// 묠니르 여부
    pub is_mjollnir: bool,
    /// 밧줄 달린 무기(아클리스) 여부
    pub is_tethered: bool,
    /// 수중 여부
    pub is_underwater: bool,
    /// 공중 레벨 또는 부유 중
    pub is_airborne: bool,
    /// 묶인 공(uball)이면서 잡고 있는 상태
    pub is_attached_ball: bool,
    /// 트랩에 갇힌 상태 (바닥 함몰)
    pub is_infloor_trap: bool,
    /// 탄약이고 발사대 매칭
    pub ammo_with_launcher: bool,
    /// 탄약이지만 보석 클래스가 아닌 경우
    pub is_mismatched_ammo: bool,
}

/// 투척 사거리 계산 (원본 dothrow.c:1206-1260)
/// 반환값: (투척 사거리, 반동 사거리)
pub fn throw_range(info: &ThrowRangeInfo) -> (i32, i32) {
    let urange = if info.is_crossbow_ammo {
        9
    } else {
        info.base_range
    };

    let mut range;
    if info.is_heavy_iron_ball {
        range = urange - info.weight / 100;
    } else {
        range = urange - info.weight / 40;
    }

    // 묶인 공 제한
    if info.is_attached_ball {
        if range >= 5 {
            range = 5;
        }
    }

    if range < 1 {
        range = 1;
    }

    // 탄약 보정
    if info.ammo_with_launcher {
        if info.is_crossbow_ammo {
            range = 8; // BOLT_LIM
        } else {
            range += 1;
        }
    } else if info.is_mismatched_ammo {
        range /= 2;
    }

    // 공중 반동
    let mut recoil = 0;
    if info.is_airborne {
        recoil = urange - range;
        if recoil < 1 {
            recoil = 1;
        }
        range -= recoil;
        if range < 1 {
            range = 1;
        }
    }

    // 특수 오버라이드
    if info.is_boulder {
        range = 20;
    } else if info.is_mjollnir {
        range = (range + 1) / 2;
    } else if info.is_tethered {
        range = range.min(4); // BOLT_LIM/2
    } else if info.is_attached_ball && info.is_infloor_trap {
        range = 1;
    }

    if info.is_underwater {
        range = 1;
    }

    (range, recoil)
}

// =============================================================================
// 5. walk_path_bresenham — Bresenham 직선 경로
// [v2.11.0] dothrow.c:452-531 이식
// =============================================================================

/// Bresenham 직선 알고리즘으로 경로 생성 (원본 dothrow.c:452-531)
/// 시작점에서 끝점까지 각 중간 지점을 반환 (시작점 제외)
pub fn walk_path_bresenham(src_x: i32, src_y: i32, dest_x: i32, dest_y: i32) -> Vec<(i32, i32)> {
    let mut path = Vec::new();
    let mut dx = dest_x - src_x;
    let mut dy = dest_y - src_y;
    let mut x = src_x;
    let mut y = src_y;

    let x_change = if dx < 0 {
        dx = -dx;
        -1
    } else {
        1
    };
    let y_change = if dy < 0 {
        dy = -dy;
        -1
    } else {
        1
    };

    let mut err = 0;

    if dx < dy {
        let mut i = 0;
        while i < dy {
            i += 1;
            y += y_change;
            err += dx << 1;
            if err > dy {
                x += x_change;
                err -= dy << 1;
            }
            path.push((x, y));
        }
    } else {
        let mut i = 0;
        while i < dx {
            i += 1;
            x += x_change;
            err += dy << 1;
            if err > dx {
                y += y_change;
                err -= dx << 1;
            }
            path.push((x, y));
        }
    }

    path
}

// =============================================================================
// 6. hurtle_blocked — 돌진 차단 판정
// [v2.11.0] dothrow.c:582-654에서 순수 판정 부분 이식
// =============================================================================

/// 돌진 차단 사유
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HurtleBlock {
    /// 범위 밖
    OutOfBounds,
    /// 벽/닫힌 문/비스듬한 열린 문
    Wall,
    /// 쇠창살
    IronBars,
    /// 바위
    Boulder,
    /// 벽 통과 불가 (우주의 끝)
    NoPasswall,
    /// 틈에 끼임 (대각선)
    Wedged,
    /// 몬스터에 충돌
    Monster,
    /// 범위 소진
    RangeExhausted,
    /// 차단 없음 (통과)
    Clear,
}

/// 돌진 충돌 데미지 (원본 dothrow.c:610-651)
/// range: 남은 범위 (충돌 강도에 영향)
pub fn hurtle_collision_damage(remaining_range: i32, rng: &mut NetHackRng) -> i32 {
    rng.rnd(2 + remaining_range)
}

// =============================================================================
// 7. slip_chance — 저주/기름 미끄러짐 확률
// [v2.11.0] dothrow.c:1108-1129 이식
// =============================================================================

/// 투척 시 미끄러짐 판정 (원본 dothrow.c:1108-1129)
/// 저주 또는 기름칠 무기는 1/7 확률로 미끄러짐
pub fn slip_chance(
    is_cursed: bool,
    is_greased: bool,
    dx: i32,
    dy: i32,
    is_ammo_with_launcher: bool,
    is_throwing_weapon: bool,
    rng: &mut NetHackRng,
) -> bool {
    if (!is_cursed && !is_greased) || (dx == 0 && dy == 0) {
        return false;
    }
    if rng.rn2(7) != 0 {
        return false;
    }
    // 발사대+탄약이면 항상 미끄러짐, 아니면 기름칠/투척무기만
    is_ammo_with_launcher || is_greased || is_throwing_weapon
}

// =============================================================================
// 8. toss_up_damage — 위로 던졌을 때 낙하 데미지
// [v2.11.0] dothrow.c:1003-1028 이식
// =============================================================================

/// 위로 던진 물건 낙하 데미지 계산 (원본 dothrow.c:1003-1028)
/// base_dmgval: dmgval() 결과
/// weight: 아이템 무게
/// has_metallic_helmet: 금속 헬멧 착용
/// damage_increase: 데미지 증가 보너스 (u.udaminc)
/// has_half_phys: Half_physical_damage
pub fn toss_up_damage(
    base_dmgval: i32,
    weight: i32,
    has_metallic_helmet: bool,
    damage_increase: i32,
    has_half_phys: bool,
) -> i32 {
    let mut dmg = if base_dmgval == 0 {
        // 무기가 아니면 무게 기반 데미지
        let d = weight / 100;
        d.clamp(1, 6)
    } else {
        base_dmgval
    };

    // 금속 헬멧은 데미지 1로 감소
    if dmg > 1 && has_metallic_helmet {
        dmg = 1;
    }

    // 데미지 증가 보너스
    if dmg > 0 {
        dmg += damage_increase;
    }
    if dmg < 0 {
        dmg = 0;
    }

    // 물리 절반
    if has_half_phys {
        dmg = (dmg + 1) / 2;
    }

    dmg
}

// =============================================================================
// 9. omon_adj — 투척 명중 보정치 (크기/수면/고정/아이템별)
// [v2.11.0] dothrow.c:1446-1487 이식
// =============================================================================

/// 투척 명중 보정 입력
#[derive(Debug, Clone)]
pub struct OmonAdjInfo {
    /// 몬스터 크기 (MZ_MEDIUM=2 기준, -2~+5)
    pub monster_size: i32,
    /// 몬스터 수면 중
    pub is_sleeping: bool,
    /// 몬스터 이동 불가
    pub is_immobile: bool,
    /// 아이템이 쇠구슬(묶이지 않은)
    pub is_loose_iron_ball: bool,
    /// 아이템이 바위
    pub is_boulder: bool,
    /// 아이템이 무기/보석 (hitval 적용 대상)
    pub is_weapon_or_gem: bool,
    /// hitval 결과 (무기/보석일 때)
    pub hit_bonus: i32,
}

/// 투척 명중 보정치 계산 (원본 dothrow.c:1446-1487)
pub fn omon_adj(info: &OmonAdjInfo) -> i32 {
    let mut tmp = 0;

    // 크기 보정 (MZ_MEDIUM=2)
    tmp += info.monster_size - 2;

    // 수면 보정 +2
    if info.is_sleeping {
        tmp += 2;
    }

    // 고정 보정 +4
    if info.is_immobile {
        tmp += 4;
    }

    // 아이템별 보정
    if info.is_loose_iron_ball {
        tmp += 2;
    } else if info.is_boulder {
        tmp += 6;
    } else if info.is_weapon_or_gem {
        tmp += info.hit_bonus;
    }

    tmp
}

// =============================================================================
// 10. gem_accept_result — 보석 선물 반응
// [v2.11.0] dothrow.c 하단 gem_accept() 로직 이식
// =============================================================================

/// 보석 선물 반응 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemAcceptResult {
    /// 유니콘이 보석을 받음 → 행운 변화
    Accepted { luck_change: i32 },
    /// 유리인 것을 알아챔 → 적대
    RejectedGlass,
    /// 회색 유니콘은 무시
    Ignored,
    /// 유니콘이 아님
    NotUnicorn,
}

/// 보석 선물 결과 판정 (원본 dothrow.c gem_accept)
/// is_unicorn: 대상이 유니콘인지
/// unicorn_color: "white", "gray", "black"
/// is_real_gem: 진짜 보석인지 (유리가 아닌)
/// same_alignment: 플레이어와 같은 성향
/// opposite_alignment: 반대 성향
pub fn gem_accept_result(
    is_unicorn: bool,
    unicorn_color: &str,
    is_real_gem: bool,
    same_alignment: bool,
    _opposite_alignment: bool,
) -> GemAcceptResult {
    if !is_unicorn {
        return GemAcceptResult::NotUnicorn;
    }
    if unicorn_color == "gray" {
        return GemAcceptResult::Ignored;
    }

    if is_real_gem {
        // 진짜 보석: 같은 성향이면 +5, 다르면 -5 (반대면 -7 식의 변형)
        let luck = if same_alignment { 5 } else { -5 };
        GemAcceptResult::Accepted { luck_change: luck }
    } else {
        // 유리: 거부 + 적대
        GemAcceptResult::RejectedGlass
    }
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    // ─── multishot_count ──────────────────────────────────────
    fn base_multishot_info() -> MultishotInfo {
        MultishotInfo {
            quantity: 20,
            is_ammo: true,
            matching_launcher: true,
            is_weapon_class: false,
            confused: false,
            stunned: false,
            skill_level: 0,
            role: "Valkyrie",
            race: "Human",
            weapon_skill: 0,
            dexterity: 12,
            fumbling: false,
            is_crossbow_ammo: false,
            strength: 18,
            is_gnome: false,
            racial_ammo_match: false,
            samurai_ya_yumi: false,
            shot_limit: 0,
        }
    }

    #[test]
    fn test_multishot_basic() {
        let mut rng = NetHackRng::new(42);
        let info = base_multishot_info();
        let count = multishot_count(&info, &mut rng);
        assert!(count >= 1);
    }

    #[test]
    fn test_multishot_confused() {
        let mut rng = NetHackRng::new(42);
        let mut info = base_multishot_info();
        info.confused = true;
        assert_eq!(multishot_count(&info, &mut rng), 1);
    }

    #[test]
    fn test_multishot_expert_ranger() {
        let mut rng = NetHackRng::new(42);
        let mut info = base_multishot_info();
        info.skill_level = 3; // Expert
        info.role = "Ranger";
        info.weapon_skill = 2; // 단검이 아닌 스킬
                               // 전문(+2) + 레인저(+1) = 4, rnd(4) ∈ [1,4]
        let count = multishot_count(&info, &mut rng);
        assert!(count >= 1 && count <= 4);
    }

    #[test]
    fn test_multishot_single_item() {
        let mut rng = NetHackRng::new(42);
        let mut info = base_multishot_info();
        info.quantity = 1;
        assert_eq!(multishot_count(&info, &mut rng), 1);
    }

    // ─── throwing_weapon_check ─────────────────────────────────
    #[test]
    fn test_throwing_spear() {
        let info = ThrowWeaponInfo {
            is_missile: false,
            is_spear: true,
            is_blade: false,
            is_sword: false,
            has_pierce: false,
            is_war_hammer: false,
            is_aklys: false,
        };
        assert!(throwing_weapon_check(&info));
    }

    #[test]
    fn test_throwing_sword_not_throwing() {
        let info = ThrowWeaponInfo {
            is_missile: false,
            is_spear: false,
            is_blade: true,
            is_sword: true,
            has_pierce: true,
            is_war_hammer: false,
            is_aklys: false,
        };
        assert!(!throwing_weapon_check(&info));
    }

    // ─── autoquiver_priority ──────────────────────────────────
    #[test]
    fn test_quiver_matched_ammo() {
        assert_eq!(
            autoquiver_priority(
                false, false, true, true, false, false, false, false, false, false, true, false
            ),
            QuiverPriority::MatchedAmmo
        );
    }

    #[test]
    fn test_quiver_worn_excluded() {
        assert_eq!(
            autoquiver_priority(
                true, false, true, true, false, false, false, false, false, false, true, false
            ),
            QuiverPriority::None
        );
    }

    // ─── throw_range ──────────────────────────────────────────
    #[test]
    fn test_range_normal() {
        let info = ThrowRangeInfo {
            base_range: 9,
            is_crossbow_ammo: false,
            weight: 40,
            is_heavy_iron_ball: false,
            is_boulder: false,
            is_mjollnir: false,
            is_tethered: false,
            is_underwater: false,
            is_airborne: false,
            is_attached_ball: false,
            is_infloor_trap: false,
            ammo_with_launcher: false,
            is_mismatched_ammo: false,
        };
        let (range, recoil) = throw_range(&info);
        assert_eq!(range, 8); // 9 - 40/40 = 8
        assert_eq!(recoil, 0);
    }

    #[test]
    fn test_range_boulder() {
        let info = ThrowRangeInfo {
            base_range: 12,
            is_crossbow_ammo: false,
            weight: 6000,
            is_heavy_iron_ball: false,
            is_boulder: true,
            is_mjollnir: false,
            is_tethered: false,
            is_underwater: false,
            is_airborne: false,
            is_attached_ball: false,
            is_infloor_trap: false,
            ammo_with_launcher: false,
            is_mismatched_ammo: false,
        };
        let (range, _) = throw_range(&info);
        assert_eq!(range, 20);
    }

    #[test]
    fn test_range_underwater() {
        let info = ThrowRangeInfo {
            base_range: 12,
            is_crossbow_ammo: false,
            weight: 10,
            is_heavy_iron_ball: false,
            is_boulder: false,
            is_mjollnir: false,
            is_tethered: false,
            is_underwater: true,
            is_airborne: false,
            is_attached_ball: false,
            is_infloor_trap: false,
            ammo_with_launcher: false,
            is_mismatched_ammo: false,
        };
        let (range, _) = throw_range(&info);
        assert_eq!(range, 1);
    }

    // ─── walk_path_bresenham ──────────────────────────────────
    #[test]
    fn test_bresenham_horizontal() {
        let path = walk_path_bresenham(0, 0, 5, 0);
        assert_eq!(path.len(), 5);
        assert_eq!(path[0], (1, 0));
        assert_eq!(path[4], (5, 0));
    }

    #[test]
    fn test_bresenham_diagonal() {
        let path = walk_path_bresenham(0, 0, 3, 3);
        assert_eq!(path.len(), 3);
        for (i, &(x, y)) in path.iter().enumerate() {
            assert_eq!(x, (i as i32) + 1);
            assert_eq!(y, (i as i32) + 1);
        }
    }

    #[test]
    fn test_bresenham_same_point() {
        let path = walk_path_bresenham(5, 5, 5, 5);
        assert!(path.is_empty());
    }

    // ─── hurtle_collision_damage ──────────────────────────────
    #[test]
    fn test_hurtle_damage() {
        let mut rng = NetHackRng::new(42);
        let dmg = hurtle_collision_damage(5, &mut rng);
        assert!(dmg >= 1 && dmg <= 7); // rnd(2+5) = rnd(7)
    }

    // ─── slip_chance ──────────────────────────────────────────
    #[test]
    fn test_slip_not_cursed() {
        let mut rng = NetHackRng::new(42);
        assert!(!slip_chance(false, false, 1, 0, false, false, &mut rng));
    }

    #[test]
    fn test_slip_no_direction() {
        let mut rng = NetHackRng::new(42);
        assert!(!slip_chance(true, false, 0, 0, false, false, &mut rng));
    }

    // ─── toss_up_damage ───────────────────────────────────────
    #[test]
    fn test_toss_damage_weapon() {
        assert_eq!(toss_up_damage(8, 100, false, 0, false), 8);
    }

    #[test]
    fn test_toss_damage_helmet() {
        assert_eq!(toss_up_damage(8, 100, true, 0, false), 1);
    }

    #[test]
    fn test_toss_damage_non_weapon() {
        assert_eq!(toss_up_damage(0, 300, false, 0, false), 3); // 300/100=3
    }

    #[test]
    fn test_toss_damage_half_phys() {
        assert_eq!(toss_up_damage(10, 100, false, 0, true), 5); // (10+1)/2
    }

    // ─── omon_adj ─────────────────────────────────────────────
    #[test]
    fn test_omon_adj_large_sleeping() {
        let info = OmonAdjInfo {
            monster_size: 5, // MZ_HUGE
            is_sleeping: true,
            is_immobile: false,
            is_loose_iron_ball: false,
            is_boulder: false,
            is_weapon_or_gem: false,
            hit_bonus: 0,
        };
        assert_eq!(omon_adj(&info), 3 + 2); // (5-2) + 2 = 5
    }

    #[test]
    fn test_omon_adj_boulder() {
        let info = OmonAdjInfo {
            monster_size: 2,
            is_sleeping: false,
            is_immobile: false,
            is_loose_iron_ball: false,
            is_boulder: true,
            is_weapon_or_gem: false,
            hit_bonus: 0,
        };
        assert_eq!(omon_adj(&info), 6);
    }

    // ─── gem_accept_result ────────────────────────────────────
    #[test]
    fn test_gem_same_alignment() {
        match gem_accept_result(true, "white", true, true, false) {
            GemAcceptResult::Accepted { luck_change } => assert_eq!(luck_change, 5),
            _ => panic!("기대값: Accepted"),
        }
    }

    #[test]
    fn test_gem_glass_rejected() {
        assert_eq!(
            gem_accept_result(true, "white", false, true, false),
            GemAcceptResult::RejectedGlass
        );
    }

    #[test]
    fn test_gem_gray_ignored() {
        assert_eq!(
            gem_accept_result(true, "gray", true, true, false),
            GemAcceptResult::Ignored
        );
    }

    #[test]
    fn test_gem_not_unicorn() {
        assert_eq!(
            gem_accept_result(false, "white", true, true, false),
            GemAcceptResult::NotUnicorn
        );
    }
}

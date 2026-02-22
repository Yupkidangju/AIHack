// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
//
// [v2.21.0 R9-4] 빔 반사 물리 엔진 확장 모듈 (zap_ext.rs)
//
// 원본 참조: NetHack 3.6.7 zap.c L800-1200 (bhit, buzz, zhitm, zap_over_floor)
//
// 구현 내용:
//   1. bhit() 정밀 궤적 계산 — 벽면 각도 기반 반사 벡터
//   2. 반사 소스 판별 시스템 (거울, 드래곤 비늘 갑옷, 실버 방패, 아뮬렉트)
//   3. 화염 빔 ↔ 폭발물(지뢰) 연쇄 상호작용
//   4. 빔 속성별 지형 상호작용 확장 (나무/문/얼음/용암)
// =============================================================================

use crate::core::entity::monster::DamageType;
use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 반사 소스 시스템 (원본: ureflects(), check_reflect(), zap.c + worn.c)
// =============================================================================

/// [v2.21.0 R9-4] 반사 원인 (원본: ureflects() 반환 문자열 분류)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReflectSource {
    /// 드래곤 비늘 갑옷(SILVER_DRAGON_SCALE_MAIL) 반사
    SilverDragonScaleMail,
    /// 실드 오브 리플렉션 (Shield of Reflection / Amulet of Reflection)
    ShieldOfReflection,
    /// 아뮬렛 오브 리플렉션 (Amulet of Reflection)
    AmuletOfReflection,
    /// 거울로 반사 (wand of nothing, zap 자가 반사 등)
    Mirror,
    /// 몬스터 고유 반사 (silver dragon 등)
    InnateMonstReflect,
    /// 알 수 없는 소스
    Unknown,
}

/// [v2.21.0 R9-4] 반사 소스의 사용자 출력 메시지 (원본: ureflects 문자열)
pub fn reflect_message(source: ReflectSource) -> &'static str {
    match source {
        ReflectSource::SilverDragonScaleMail => "It reflects off your silver dragon scale mail!",
        ReflectSource::ShieldOfReflection => "It reflects off your shield!",
        ReflectSource::AmuletOfReflection => "It reflects off your amulet!",
        ReflectSource::Mirror => "It reflects off your mirror!",
        ReflectSource::InnateMonstReflect => "It reflects off the monster!",
        ReflectSource::Unknown => "It is reflected!",
    }
}

/// [v2.21.0 R9-4] 반사 판정 컨텍스트
/// 장비 슬롯별 반사 아이템 존재 여부를 캡슐화
#[derive(Debug, Clone, Default)]
pub struct ReflectionContext {
    /// 실버 드래곤 비늘 갑옷 착용 여부
    pub has_silver_dragon_scale: bool,
    /// 리플렉션 쉴드 착용 여부
    pub has_reflection_shield: bool,
    /// 리플렉션 아뮬렛 착용 여부
    pub has_reflection_amulet: bool,
    /// 거울 소지 여부
    pub has_mirror: bool,
    /// 상태 REFLECTING 플래그 여부 (누적 효과)
    pub status_reflecting: bool,
}

/// [v2.21.0 R9-4] 반사 가능 여부 및 소스 판별
/// 원본: ureflects() → 장비 우선순위: 갑옷 > 방패 > 아뮬렛
pub fn check_reflect(ctx: &ReflectionContext) -> Option<ReflectSource> {
    // 우선순위: 갑옷 > 방패 > 아뮬렛 > 거울 > 상태 플래그
    if ctx.has_silver_dragon_scale {
        return Some(ReflectSource::SilverDragonScaleMail);
    }
    if ctx.has_reflection_shield {
        return Some(ReflectSource::ShieldOfReflection);
    }
    if ctx.has_reflection_amulet {
        return Some(ReflectSource::AmuletOfReflection);
    }
    if ctx.has_mirror {
        return Some(ReflectSource::Mirror);
    }
    if ctx.status_reflecting {
        return Some(ReflectSource::Unknown);
    }
    None
}

/// [v2.21.0 R9-4] 빔이 반사 가능한 속성인지 확인
/// 원본: zap.c — ray(레이) 계열만 반사 가능, bolt는 불가
pub fn is_beam_reflectable(dtype: DamageType) -> bool {
    matches!(
        dtype,
        DamageType::Magm  // 마법 미사일
        | DamageType::Fire  // 화염
        | DamageType::Cold  // 냉기
        | DamageType::Slee  // 수면
        | DamageType::Deth  // 죽음
        | DamageType::Disn  // 분해
        | DamageType::Elec  // 전격
        | DamageType::Acid  // 산
        | DamageType::Poly // 변이
    )
}

// =============================================================================
// [2] bhit 궤적 계산 엔진 (원본: bhit(), zap.c L800-950)
// =============================================================================

/// [v2.21.0 R9-4] 빔 경로 타일 단위 결과
#[derive(Debug, Clone)]
pub struct BeamTile {
    pub x: i32,
    pub y: i32,
    /// 해당 타일에서 발생한 이벤트
    pub event: BeamTileEvent,
}

/// [v2.21.0 R9-4] 타일에서 발생 가능한 빔 이벤트
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeamTileEvent {
    /// 아무 일도 없이 통과
    PassThrough,
    /// 벽에 부딪혀 반사
    WallBounce,
    /// 대상(플레이어/몬스터)에 의한 반사
    EntityReflect(ReflectSource),
    /// 대상에게 적중
    HitEntity,
    /// 지형 효과 발동
    TerrainInteract(super::zap::BeamTerrainEffect),
    /// 트랩 연쇄 (화염 → 지뢰 등)
    TrapChain(TrapChainEffect),
    /// 범위 밖 또는 최대 반사 도달로 소멸
    Dissipated,
}

/// [v2.21.0 R9-4] 벽 반사 방향 계산 (원본: zap.c bhit의 bounce 로직)
///
/// `(dx, dy)`: 현재 빔 방향
/// `wall_at_hori`: (prev_x, cy) 위치가 벽인지 (수평 성분 차단 확인)
/// `wall_at_vert`: (cx, prev_y) 위치가 벽인지 (수직 성분 차단 확인)
///
/// 반환: 반사 후의 새 (dx, dy)
pub fn calc_wall_bounce(
    dx: i32,
    dy: i32,
    wall_at_hori: bool,
    wall_at_vert: bool,
    rng: &mut NetHackRng,
) -> (i32, i32) {
    if dx != 0 && dy != 0 {
        // 대각선 입사
        if wall_at_hori && wall_at_vert {
            // 코너 — 완전 역방향 반사
            (-dx, -dy)
        } else if wall_at_hori {
            // 수평 벽(천장/바닥)에 부딪힘 → dx 반전
            let new_dx = -dx;
            // 원본: 25% 확률로 산란(scattering)
            let new_dy = if rng.rn2(4) == 0 { -dy } else { dy };
            (new_dx, new_dy)
        } else if wall_at_vert {
            // 수직 벽(좌/우)에 부딪힘 → dy 반전
            (-dx, -dy) // 원본에서는 양쪽 다 없으면 여기로 옴
        } else {
            // 두 벽 모두 없음 — 역방향 (원본 fallback)
            (-dx, -dy)
        }
    } else if dx != 0 {
        // 수평 진행 → 수평 반전
        (-dx, dy)
    } else {
        // 수직 진행 → 수직 반전
        (dx, -dy)
    }
}

/// [v2.21.0 R9-4] 엔티티 반사 시 방향 반전
/// 원본: 엔티티에 맞을 때 빔은 정확히 역방향으로 반사
pub fn calc_entity_reflect(dx: i32, dy: i32) -> (i32, i32) {
    (-dx, -dy)
}

// =============================================================================
// [3] 화염 빔 ↔ 폭발물/트랩 연쇄 시스템 (원본: zap_over_floor, zap.c L1100+)
// =============================================================================

/// [v2.21.0 R9-4] 트랩 연쇄 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapChainEffect {
    /// 화염 빔이 지뢰를 유발 (원본: fire hits landmine → explode)
    LandmineDetonation {
        x: i32,
        y: i32,
        explosion_radius: i32,
        base_damage: i32,
    },
    /// 냉기 빔이 화염 트랩 무력화 (원본: cold hits fire trap → disarm)
    FireTrapDisabled { x: i32, y: i32 },
    /// 전격 빔이 물 웅덩이에서 전도 (원본: shock on water pool → electrify)
    WaterElectrify { x: i32, y: i32, damage: i32 },
    /// 분해 빔이 트랩 자체를 소멸
    TrapDisintegrated { x: i32, y: i32 },
}

/// [v2.21.0 R9-4] 빔-트랩 연쇄 판정 (원본: zap_over_floor, trap.c + zap.c)
///
/// `beam_type`: 빔의 데미지 속성
/// `trap_name`: 트랩 종류 문자열 ("landmine", "fire", "pit" 등)
/// `tx, ty`: 트랩 위치
///
/// 반환: 연쇄 효과 (없으면 None)
pub fn check_beam_trap_chain(
    beam_type: DamageType,
    trap_name: &str,
    tx: i32,
    ty: i32,
    rng: &mut NetHackRng,
) -> Option<TrapChainEffect> {
    let tn = trap_name.to_lowercase();
    match beam_type {
        DamageType::Fire => {
            // 화염 빔이 지뢰에 닿으면 폭발 (원본: fire + landmine → boom)
            if tn.contains("landmine") || tn.contains("mine") {
                return Some(TrapChainEffect::LandmineDetonation {
                    x: tx,
                    y: ty,
                    explosion_radius: 1,
                    base_damage: rng.d(6, 6), // 6d6 피해
                });
            }
            // 화염 빔이 웹을 태움 (원본 존재)
            // 여기서는 TerrainInteract(BurnTree)와 유사하므로 None
        }
        DamageType::Cold => {
            // 냉기 빔이 화염 트랩 무력화 (원본 존재)
            if tn.contains("fire") {
                return Some(TrapChainEffect::FireTrapDisabled { x: tx, y: ty });
            }
        }
        DamageType::Elec => {
            // 전격 빔이 물 지형의 트랩으로 전도
            if tn.contains("pool") || tn.contains("water") || tn.contains("moat") {
                return Some(TrapChainEffect::WaterElectrify {
                    x: tx,
                    y: ty,
                    damage: rng.d(4, 6), // 4d6 전도 피해
                });
            }
        }
        DamageType::Disn => {
            // 분해 빔: 모든 트랩 소멸
            return Some(TrapChainEffect::TrapDisintegrated { x: tx, y: ty });
        }
        _ => {}
    }
    None
}

// =============================================================================
// [4] 빔-지형 정밀 상호작용 (원본: zap.c buzz() 내부 지형 효과 확장)
// =============================================================================

/// [v2.21.0 R9-4] 확장 지형 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtTerrainEffect {
    /// 화염 → 얼음 녹임 (→ 물웅덩이)
    MeltIce,
    /// 화염 → 나무 태움 (→ 빈 바닥)
    BurnTree,
    /// 화염 → 닫힌 문 태움 (→ 파괴)
    BurnDoor,
    /// 화염 → 물 증발 (→ 빈 바닥) (원본: evaporate_one)
    EvaporateWater,
    /// 냉기 → 물 얼림 (→ 얼음)
    FreezeWater,
    /// 냉기 → 용암 냉각 (→ 바닥) (원본: cool lava)
    CoolLava,
    /// 전격 → 물에 전기 전도 (주변 피해 확산)
    ElectrifyWater,
    /// 전격 → 닫힌 문 파괴
    ShockDoor,
    /// 분해 → 모든 타일 파괴 (→ 구멍)
    DisintegrateAny,
    /// 산 → 돌문 부식
    CorreDoor,
    /// 효과 없음
    None,
}

/// [v2.21.0 R9-4] 빔 속성 × 지형(TileType) 상호작용 매핑 (원본: buzz → 지형 분기)
pub fn beam_terrain_effect_ext(beam_type: DamageType, tile_name: &str) -> ExtTerrainEffect {
    let t = tile_name.to_lowercase();
    match beam_type {
        DamageType::Fire => {
            if t.contains("ice") {
                ExtTerrainEffect::MeltIce
            } else if t.contains("tree") {
                ExtTerrainEffect::BurnTree
            } else if t.contains("door") {
                ExtTerrainEffect::BurnDoor
            } else if t.contains("water") || t.contains("pool") {
                ExtTerrainEffect::EvaporateWater
            } else {
                ExtTerrainEffect::None
            }
        }
        DamageType::Cold => {
            if t.contains("lava") {
                ExtTerrainEffect::CoolLava
            } else if t.contains("water") || t.contains("pool") || t.contains("moat") {
                ExtTerrainEffect::FreezeWater
            } else {
                ExtTerrainEffect::None
            }
        }
        DamageType::Elec => {
            if t.contains("water") || t.contains("pool") || t.contains("moat") {
                ExtTerrainEffect::ElectrifyWater
            } else if t.contains("door") {
                ExtTerrainEffect::ShockDoor
            } else {
                ExtTerrainEffect::None
            }
        }
        DamageType::Disn => ExtTerrainEffect::DisintegrateAny,
        DamageType::Acid => {
            if t.contains("door") {
                ExtTerrainEffect::CorreDoor
            } else {
                ExtTerrainEffect::None
            }
        }
        _ => ExtTerrainEffect::None,
    }
}

// =============================================================================
// [5] 정밀 bhit 시뮬레이션 엔진 (순수 결과 패턴, ECS 비의존)
// =============================================================================

/// [v2.21.0 R9-4] bhit 물리 시뮬레이션 입력
#[derive(Debug, Clone)]
pub struct BhitInput {
    /// 빔 시작 위치
    pub origin: (i32, i32),
    /// 빔 방향 (dx, dy)
    pub direction: (i32, i32),
    /// 빔 속성
    pub damage_type: DamageType,
    /// 빔 사거리
    pub range: i32,
    /// 레이 여부 (반사 가능 여부 결정)
    pub is_ray: bool,
    /// 맵 가로 크기
    pub map_width: i32,
    /// 맵 세로 크기
    pub map_height: i32,
}

/// [v2.21.0 R9-4] bhit 물리 시뮬레이션 결과
#[derive(Debug, Clone)]
pub struct BhitTrace {
    /// 빔이 통과한 타일 목록 (순서대로)
    pub tiles: Vec<BeamTile>,
    /// 총 반사 횟수
    pub bounce_count: i32,
    /// 빔이 소멸된 이유
    pub termination: BhitTermination,
}

/// [v2.21.0 R9-4] 빔 소멸 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BhitTermination {
    /// 사거리 소진
    RangeExhausted,
    /// 최대 반사 횟수 초과
    MaxBounceExceeded,
    /// 범위 밖 이탈
    OutOfBounds,
    /// 비반사 벽 충돌 (볼트)
    WallAbsorbed,
    /// 대상 적중으로 소멸 (볼트)
    EntityAbsorbed,
}

/// [v2.21.0 R9-4] 순수 bhit 궤적 시뮬레이션 (ECS 비의존, 타일 콜백 기반)
///
/// `is_wall`: (x, y) -> 해당 좌표가 벽인지 콜백
///
/// 반환: 빔이 지나간 모든 타일과 발생한 이벤트 기록
pub fn simulate_bhit<F>(input: &BhitInput, mut is_wall: F, rng: &mut NetHackRng) -> BhitTrace
where
    F: FnMut(i32, i32) -> bool,
{
    let mut tiles = Vec::new();
    let mut bounce_count = 0;
    let max_bounce: i32 = if input.is_ray { 8 } else { 0 };
    let mut range_left = input.range;

    let (mut dx, mut dy) = input.direction;
    if dx == 0 && dy == 0 {
        return BhitTrace {
            tiles,
            bounce_count: 0,
            termination: BhitTermination::RangeExhausted,
        };
    }

    let (mut cx, mut cy) = input.origin;

    while range_left > 0 {
        range_left -= 1;
        let prev_x = cx;
        let prev_y = cy;
        cx += dx;
        cy += dy;

        // 범위 밖 체크
        if cx < 0 || cx >= input.map_width || cy < 0 || cy >= input.map_height {
            tiles.push(BeamTile {
                x: cx,
                y: cy,
                event: BeamTileEvent::Dissipated,
            });
            return BhitTrace {
                tiles,
                bounce_count,
                termination: BhitTermination::OutOfBounds,
            };
        }

        // 벽 충돌 체크
        if is_wall(cx, cy) {
            if input.is_ray && bounce_count < max_bounce {
                // 벽 반사 — 인접 타일 분석
                let wall_h = is_wall(prev_x, cy);
                let wall_v = is_wall(cx, prev_y);
                let (new_dx, new_dy) = calc_wall_bounce(dx, dy, wall_h, wall_v, rng);
                dx = new_dx;
                dy = new_dy;
                bounce_count += 1;

                tiles.push(BeamTile {
                    x: cx,
                    y: cy,
                    event: BeamTileEvent::WallBounce,
                });

                // 이전 위치로 복귀 후 새 방향 진행
                cx = prev_x;
                cy = prev_y;
                continue;
            } else {
                // 볼트 또는 최대 반사 → 벽에 흡수
                tiles.push(BeamTile {
                    x: cx,
                    y: cy,
                    event: BeamTileEvent::Dissipated,
                });
                return BhitTrace {
                    tiles,
                    bounce_count,
                    termination: if !input.is_ray {
                        BhitTermination::WallAbsorbed
                    } else {
                        BhitTermination::MaxBounceExceeded
                    },
                };
            }
        }

        // 벽이 아닌 경우 PassThrough 기록
        tiles.push(BeamTile {
            x: cx,
            y: cy,
            event: BeamTileEvent::PassThrough,
        });
    }

    BhitTrace {
        tiles,
        bounce_count,
        termination: BhitTermination::RangeExhausted,
    }
}

// =============================================================================
// [6] 빔 감쇠 모델 정밀화 (원본: zap.c damage reduction over distance)
// =============================================================================

/// [v2.21.0 R9-4] 반사 횟수에 따른 데미지 감쇠 계수
/// 원본에서는 반사 시 데미지 감소 없음이지만, 현실적 밸런스를 위해 선택적 적용
pub fn reflect_damage_factor(bounces: i32) -> f32 {
    match bounces {
        0 => 1.0,
        1 => 0.9,
        2 => 0.75,
        3 => 0.6,
        _ => 0.5,
    }
}

/// [v2.21.0 R9-4] 거리 + 반사 횟수 종합 데미지 계산
pub fn calc_beam_damage(
    base_dice: (i32, i32),
    distance: i32,
    bounces: i32,
    beam_type: DamageType,
    rng: &mut NetHackRng,
) -> i32 {
    let raw = if base_dice.0 > 0 {
        rng.d(base_dice.0, base_dice.1)
    } else {
        0
    };
    // 거리 감쇠
    let attenuated = super::zap::beam_attenuation(raw, distance, beam_type);
    // 반사 감쇠
    let factor = reflect_damage_factor(bounces);
    ((attenuated as f32) * factor).max(1.0) as i32
}

// =============================================================================
// [7] 거울 반사 특수 로직 (원본: use_mirror(), apply.c + zap.c)
// =============================================================================

/// [v2.21.0 R9-4] 거울 반사 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirrorReflectResult {
    /// 성공적으로 반사
    Reflected,
    /// 거울이 깨짐 (저주된 거울)
    BrokenMirror,
    /// 반사 불가 (빔 속성이 반사 불가)
    NotReflectable,
}

/// [v2.21.0 R9-4] 거울 반사 시도 (원본: use_mirror와 zap 연동)
///
/// 저주된 거울은 25% 확률로 깨짐
pub fn try_mirror_reflect(
    beam_type: DamageType,
    mirror_cursed: bool,
    rng: &mut NetHackRng,
) -> MirrorReflectResult {
    if !is_beam_reflectable(beam_type) {
        return MirrorReflectResult::NotReflectable;
    }
    // 저주된 거울: 25% 확률로 깨짐 (원본: rn2(4) == 0)
    if mirror_cursed && rng.rn2(4) == 0 {
        return MirrorReflectResult::BrokenMirror;
    }
    MirrorReflectResult::Reflected
}

// =============================================================================
// [8] 드래곤 비늘 갑옷 반사 (원본: worn.c, dragon scale 장비 효과)
// =============================================================================

/// [v2.21.0 R9-4] 드래곤 비늘 종류별 반사 여부
/// 원본: silver dragon scale mail → reflection 부여
pub fn dragon_scale_reflects(scale_color: &str) -> bool {
    let c = scale_color.to_lowercase();
    // 은색(silver) 드래곤 비늘 갑옷만 반사 부여
    c.contains("silver")
}

/// [v2.21.0 R9-4] 드래곤 비늘 종류별 저항 부여 (참고 데이터)
pub fn dragon_scale_resistance(scale_color: &str) -> Option<DamageType> {
    let c = scale_color.to_lowercase();
    if c.contains("red") {
        Some(DamageType::Fire)
    } else if c.contains("white") {
        Some(DamageType::Cold)
    } else if c.contains("blue") {
        Some(DamageType::Elec)
    } else if c.contains("orange") {
        Some(DamageType::Slee)
    } else if c.contains("black") {
        Some(DamageType::Disn)
    } else if c.contains("yellow") {
        Some(DamageType::Acid)
    } else if c.contains("green") {
        Some(DamageType::Drst)
    } else if c.contains("gray") || c.contains("grey") {
        None // 회색: 특수 효과 없음 (취소 방지 등)
    } else {
        None
    }
}

// =============================================================================
// [9] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::rng::NetHackRng;

    #[test]
    fn test_check_reflect_priority() {
        // 갑옷 > 방패 > 아뮬렛 > 거울
        let ctx = ReflectionContext {
            has_silver_dragon_scale: true,
            has_reflection_shield: true,
            has_reflection_amulet: true,
            has_mirror: true,
            status_reflecting: false,
        };
        assert_eq!(
            check_reflect(&ctx),
            Some(ReflectSource::SilverDragonScaleMail)
        );

        let ctx2 = ReflectionContext {
            has_silver_dragon_scale: false,
            has_reflection_shield: true,
            has_reflection_amulet: true,
            has_mirror: false,
            status_reflecting: false,
        };
        assert_eq!(
            check_reflect(&ctx2),
            Some(ReflectSource::ShieldOfReflection)
        );

        let ctx3 = ReflectionContext::default();
        assert_eq!(check_reflect(&ctx3), None);
    }

    #[test]
    fn test_is_beam_reflectable() {
        assert!(is_beam_reflectable(DamageType::Magm));
        assert!(is_beam_reflectable(DamageType::Fire));
        assert!(is_beam_reflectable(DamageType::Deth));
        // Phys(물리)는 반사 불가
        assert!(!is_beam_reflectable(DamageType::Phys));
        assert!(!is_beam_reflectable(DamageType::Drli));
    }

    #[test]
    fn test_calc_wall_bounce_straight() {
        let mut rng = NetHackRng::new(42);
        // 수평 진행 → 벽에 부딪히면 dx 반전
        let (ndx, ndy) = calc_wall_bounce(1, 0, false, false, &mut rng);
        assert_eq!(ndx, -1);
        assert_eq!(ndy, 0);
    }

    #[test]
    fn test_calc_wall_bounce_diagonal_corner() {
        let mut rng = NetHackRng::new(42);
        // 대각선 코너 → 완전 역방향
        let (ndx, ndy) = calc_wall_bounce(1, 1, true, true, &mut rng);
        assert_eq!(ndx, -1);
        assert_eq!(ndy, -1);
    }

    #[test]
    fn test_simulate_bhit_basic() {
        let mut rng = NetHackRng::new(42);

        let input = BhitInput {
            origin: (5, 5),
            direction: (1, 0),
            damage_type: DamageType::Magm,
            range: 10,
            is_ray: false,
            map_width: 80,
            map_height: 21,
        };

        // 벽 없음 → 사거리 소진
        let trace = simulate_bhit(&input, |_x, _y| false, &mut rng);
        assert_eq!(trace.tiles.len(), 10);
        assert_eq!(trace.termination, BhitTermination::RangeExhausted);
        assert_eq!(trace.bounce_count, 0);
    }

    #[test]
    fn test_simulate_bhit_wall_bounce() {
        let mut rng = NetHackRng::new(42);

        let input = BhitInput {
            origin: (5, 5),
            direction: (1, 0),
            damage_type: DamageType::Fire,
            range: 20,
            is_ray: true, // 반사 가능
            map_width: 80,
            map_height: 21,
        };

        // x=10 에 벽 배치
        let trace = simulate_bhit(&input, |x, _y| x == 10, &mut rng);
        // 반사가 발생했으므로 bounce_count > 0
        assert!(trace.bounce_count > 0);
        // WallBounce 이벤트가 기록되었는지 확인
        assert!(trace
            .tiles
            .iter()
            .any(|t| t.event == BeamTileEvent::WallBounce));
    }

    #[test]
    fn test_simulate_bhit_bolt_no_bounce() {
        let mut rng = NetHackRng::new(42);

        let input = BhitInput {
            origin: (5, 5),
            direction: (1, 0),
            damage_type: DamageType::Magm,
            range: 20,
            is_ray: false, // 볼트 → 반사 불가
            map_width: 80,
            map_height: 21,
        };

        // x=10 에 벽
        let trace = simulate_bhit(&input, |x, _y| x == 10, &mut rng);
        assert_eq!(trace.bounce_count, 0);
        assert_eq!(trace.termination, BhitTermination::WallAbsorbed);
    }

    #[test]
    fn test_beam_trap_chain_fire_landmine() {
        let mut rng = NetHackRng::new(42);
        let result = check_beam_trap_chain(DamageType::Fire, "landmine", 10, 10, &mut rng);
        assert!(result.is_some());
        match result.unwrap() {
            TrapChainEffect::LandmineDetonation {
                x,
                y,
                explosion_radius,
                base_damage,
            } => {
                assert_eq!(x, 10);
                assert_eq!(y, 10);
                assert_eq!(explosion_radius, 1);
                assert!(base_damage > 0);
            }
            _ => panic!("예상과 다른 트랩 연쇄 효과"),
        }
    }

    #[test]
    fn test_beam_trap_chain_cold_fire() {
        let mut rng = NetHackRng::new(42);
        let result = check_beam_trap_chain(DamageType::Cold, "fire trap", 5, 5, &mut rng);
        assert_eq!(
            result,
            Some(TrapChainEffect::FireTrapDisabled { x: 5, y: 5 })
        );
    }

    #[test]
    fn test_beam_trap_chain_disintegrate() {
        let mut rng = NetHackRng::new(42);
        let result = check_beam_trap_chain(DamageType::Disn, "bear trap", 3, 7, &mut rng);
        assert_eq!(
            result,
            Some(TrapChainEffect::TrapDisintegrated { x: 3, y: 7 })
        );
    }

    #[test]
    fn test_mirror_reflect_normal() {
        let mut rng = NetHackRng::new(42);
        let r = try_mirror_reflect(DamageType::Magm, false, &mut rng);
        assert_eq!(r, MirrorReflectResult::Reflected);
    }

    #[test]
    fn test_mirror_not_reflectable() {
        let mut rng = NetHackRng::new(42);
        // Phys는 반사 불가
        let r = try_mirror_reflect(DamageType::Phys, false, &mut rng);
        assert_eq!(r, MirrorReflectResult::NotReflectable);
    }

    #[test]
    fn test_dragon_scale_reflects() {
        assert!(dragon_scale_reflects("silver"));
        assert!(dragon_scale_reflects("Silver Dragon Scale Mail"));
        assert!(!dragon_scale_reflects("red"));
        assert!(!dragon_scale_reflects("blue"));
    }

    #[test]
    fn test_dragon_scale_resistance() {
        assert_eq!(dragon_scale_resistance("red"), Some(DamageType::Fire));
        assert_eq!(dragon_scale_resistance("white"), Some(DamageType::Cold));
        assert_eq!(dragon_scale_resistance("blue"), Some(DamageType::Elec));
        assert_eq!(dragon_scale_resistance("silver"), None); // 은색은 반사만
    }

    #[test]
    fn test_reflect_damage_factor() {
        assert!((reflect_damage_factor(0) - 1.0).abs() < f32::EPSILON);
        assert!((reflect_damage_factor(1) - 0.9).abs() < f32::EPSILON);
        assert!((reflect_damage_factor(4) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calc_beam_damage() {
        let mut rng = NetHackRng::new(42);
        let dmg0 = calc_beam_damage((6, 6), 0, 0, DamageType::Magm, &mut rng);
        assert!(dmg0 > 0);
        let dmg1 = calc_beam_damage((6, 6), 5, 2, DamageType::Fire, &mut rng);
        // 거리+반사 감쇠 적용 후에도 최소 1
        assert!(dmg1 >= 1);
    }

    #[test]
    fn test_beam_terrain_effect_ext() {
        assert_eq!(
            beam_terrain_effect_ext(DamageType::Fire, "ice"),
            ExtTerrainEffect::MeltIce
        );
        assert_eq!(
            beam_terrain_effect_ext(DamageType::Fire, "tree"),
            ExtTerrainEffect::BurnTree
        );
        assert_eq!(
            beam_terrain_effect_ext(DamageType::Cold, "lava_pool"),
            ExtTerrainEffect::CoolLava
        );
        assert_eq!(
            beam_terrain_effect_ext(DamageType::Elec, "pool"),
            ExtTerrainEffect::ElectrifyWater
        );
        assert_eq!(
            beam_terrain_effect_ext(DamageType::Disn, "anything"),
            ExtTerrainEffect::DisintegrateAny
        );
        assert_eq!(
            beam_terrain_effect_ext(DamageType::Acid, "door"),
            ExtTerrainEffect::CorreDoor
        );
        assert_eq!(
            beam_terrain_effect_ext(DamageType::Phys, "floor"),
            ExtTerrainEffect::None
        );
    }

    #[test]
    fn test_entity_reflect_reverses_direction() {
        assert_eq!(calc_entity_reflect(1, 0), (-1, 0));
        assert_eq!(calc_entity_reflect(-1, 1), (1, -1));
        assert_eq!(calc_entity_reflect(0, -1), (0, 1));
    }

    #[test]
    fn test_out_of_bounds() {
        let mut rng = NetHackRng::new(42);
        let input = BhitInput {
            origin: (78, 10),
            direction: (1, 0),
            damage_type: DamageType::Magm,
            range: 50,
            is_ray: false,
            map_width: 80,
            map_height: 21,
        };
        let trace = simulate_bhit(&input, |_x, _y| false, &mut rng);
        assert_eq!(trace.termination, BhitTermination::OutOfBounds);
    }
}

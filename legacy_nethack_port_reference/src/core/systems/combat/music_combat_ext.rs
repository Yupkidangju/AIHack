// ============================================================================
// [v2.29.0 R17-4] 음악 전투 확장 (music_combat_ext.rs)
// 원본: NetHack 3.6.7 music.c (643줄) 전투 효과 부분
// 악기별 전투 효과, 즉사 노래, 혼란 효과
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.29.0 R17-4] 악기 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instrument {
    WoodenFlute,
    MagicFlute,
    TooledHorn,
    FrostHorn,
    FireHorn,
    Harp,
    MagicHarp,
    Drum,
    DrumOfEarthquake,
    Bugle,
}

/// [v2.29.0 R17-4] 악기 전투 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MusicEffect {
    /// 수면 (magic flute)
    Sleep { radius: i32, turns: i32 },
    /// 냉기 데미지 (frost horn)
    ColdDamage(i32),
    /// 화염 데미지 (fire horn)
    FireDamage(i32),
    /// 혼란 (drum)
    Confuse { radius: i32, turns: i32 },
    /// 지진 (drum of earthquake)
    Earthquake { radius: i32 },
    /// 공포 유발 (magic harp)
    Fear { radius: i32, turns: i32 },
    /// 길들이기 (magic harp, 특정 대상)
    Charm { radius: i32 },
    /// 병사 소환 (bugle)
    SummonSoldiers(i32),
    /// 효과 없음 (일반 악기)
    NoEffect,
}

/// [v2.29.0 R17-4] 악기→효과 매핑 (원본: do_play_instrument)
pub fn play_effect(instrument: Instrument, skill_level: i32, rng: &mut NetHackRng) -> MusicEffect {
    match instrument {
        Instrument::MagicFlute => MusicEffect::Sleep {
            radius: 3 + skill_level,
            turns: rng.rn1(10, 5),
        },
        Instrument::FrostHorn => MusicEffect::ColdDamage(rng.rn1(6, 1) * skill_level.max(1)),
        Instrument::FireHorn => MusicEffect::FireDamage(rng.rn1(6, 1) * skill_level.max(1)),
        Instrument::MagicHarp => {
            if rng.rn2(2) == 0 {
                MusicEffect::Fear {
                    radius: 5,
                    turns: rng.rn1(8, 3),
                }
            } else {
                MusicEffect::Charm { radius: 3 }
            }
        }
        Instrument::Drum => MusicEffect::Confuse {
            radius: 5,
            turns: rng.rn1(5, 1),
        },
        Instrument::DrumOfEarthquake => MusicEffect::Earthquake { radius: 8 },
        Instrument::Bugle => MusicEffect::SummonSoldiers(rng.rn1(4, 1)),
        _ => MusicEffect::NoEffect,
    }
}

// 음악-공격 대상 판정: 몬스터가 범위 내에 있는지
pub fn in_music_range(
    player_x: i32,
    player_y: i32,
    target_x: i32,
    target_y: i32,
    radius: i32,
) -> bool {
    let dx = (player_x - target_x).abs();
    let dy = (player_y - target_y).abs();
    dx <= radius && dy <= radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_flute_sleep() {
        let mut rng = NetHackRng::new(42);
        let effect = play_effect(Instrument::MagicFlute, 2, &mut rng);
        assert!(matches!(effect, MusicEffect::Sleep { .. }));
    }

    #[test]
    fn test_fire_horn() {
        let mut rng = NetHackRng::new(42);
        let effect = play_effect(Instrument::FireHorn, 3, &mut rng);
        if let MusicEffect::FireDamage(d) = effect {
            assert!(d >= 3); // 최소 1*3
        } else {
            panic!("expected FireDamage");
        }
    }

    #[test]
    fn test_earthquake() {
        let mut rng = NetHackRng::new(42);
        let effect = play_effect(Instrument::DrumOfEarthquake, 1, &mut rng);
        assert_eq!(effect, MusicEffect::Earthquake { radius: 8 });
    }

    #[test]
    fn test_normal_no_effect() {
        let mut rng = NetHackRng::new(42);
        let effect = play_effect(Instrument::WoodenFlute, 0, &mut rng);
        assert_eq!(effect, MusicEffect::NoEffect);
    }

    #[test]
    fn test_range() {
        assert!(in_music_range(10, 10, 12, 12, 5));
        assert!(!in_music_range(10, 10, 20, 20, 5));
    }
}

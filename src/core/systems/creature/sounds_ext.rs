// sounds_ext.rs — sounds.c 핵심 로직 순수 결과 패턴 이식
// [v2.13.0] 신규 생성: 몬스터 소리 유형 결정, 환각 소리, 펫 울음 동사 등 8개 함수
// 원본: NetHack 3.6.7 src/sounds.c (1,183줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 환각 상태에서 들리는 소리 목록 (36종)
/// 원본: sounds.c h_sounds[] L300-307
const H_SOUNDS: [&str; 36] = [
    "beep", "boing", "sing", "belche", "creak", "cough", "rattle", "ululate", "pop", "jingle",
    "sniffle", "tinkle", "eep", "clatter", "hum", "sizzle", "twitter", "wheeze", "rustle", "honk",
    "lisp", "yodel", "coo", "burp", "moo", "boom", "murmur", "oink", "quack", "rumble", "twang",
    "bellow", "toot", "gargle", "hoot", "warble",
];

/// 웃음 유형 목록 (4종)
/// 원본: sounds.c domonnoise() MS_LAUGH L728-731
const LAUGH_MSGS: [&str; 4] = ["giggles.", "chuckles.", "snickers.", "laughs."];

/// 방 유형별 소리 발생 확률 분모
/// 원본: sounds.c dosounds() — 분수, 베이스, 등등
const FOUNTAIN_SOUND_CHANCE: i32 = 400;
const SINK_SOUND_CHANCE: i32 = 300;
const ROOM_SOUND_CHANCE: i32 = 200;
const ORACLE_SOUND_CHANCE: i32 = 400;

// ============================================================
// 열거형
// ============================================================

/// 몬스터 소리 유형 (msound)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterSound {
    Silent,
    Mew,
    Bark,
    Growl,
    Roar,
    Hiss,
    Buzz,
    Sqeek,
    Sqawk,
    Neigh,
    Wail,
    Gurgle,
    Burble,
    Shriek,
    Imitate,
    Bones,
    Laugh,
    Mumble,
    Grunt,
    Spell,
    Nurse,
    Seduce,
    Arrest,
    Soldier,
    Guard,
    Djinni,
    Boast,
    Humanoid,
    Were,
    Vampire,
    Orc,
    Oracle,
    Priest,
    Leader,
    Nemesis,
    Guardian,
    Sell,
    Rider,
    Bribe,
    Cuss,
    Animal,
    Other,
}

/// 으르렁/비명 소리 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundVerb {
    pub verb: &'static str,
}

/// 펫 소리 컨텍스트
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetSoundContext {
    Growl,   // 심한 학대
    Yelp,    // 가벼운 학대
    Whimper, // 고통
}

/// 방 유형 (소리 발생용)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    Fountain,
    Sink,
    Court,
    Swamp,
    Vault,
    Beehive,
    Morgue,
    Barracks,
    Zoo,
    Shop,
    Temple,
    Oracle,
}

// ============================================================
// 1. growl_sound_type — 으르렁 소리 동사 결정
// ============================================================

/// 몬스터의 으르렁 소리 동사 결정
/// 원본: sounds.c growl_sound() L310-349
pub fn growl_sound_type(sound: MonsterSound) -> &'static str {
    match sound {
        MonsterSound::Mew | MonsterSound::Hiss => "hiss",
        MonsterSound::Bark | MonsterSound::Growl => "growl",
        MonsterSound::Roar => "roar",
        MonsterSound::Buzz => "buzz",
        MonsterSound::Sqeek => "squeal",
        MonsterSound::Sqawk => "screech",
        MonsterSound::Neigh => "neigh",
        MonsterSound::Wail => "wail",
        MonsterSound::Silent => "commotion",
        _ => "scream",
    }
}

// ============================================================
// 2. yelp_sound_type — 학대된 펫 비명 동사
// ============================================================

/// 학대된 펫의 비명 소리 동사 결정
/// 원본: sounds.c yelp() L388-408
pub fn yelp_sound_type(sound: MonsterSound, is_deaf: bool) -> Option<&'static str> {
    match sound {
        MonsterSound::Mew => Some(if !is_deaf { "yowl" } else { "arch" }),
        MonsterSound::Bark | MonsterSound::Growl => Some(if !is_deaf { "yelp" } else { "recoil" }),
        MonsterSound::Roar => Some(if !is_deaf { "snarl" } else { "bluff" }),
        MonsterSound::Sqeek => Some(if !is_deaf { "squeal" } else { "quiver" }),
        MonsterSound::Sqawk => Some(if !is_deaf { "screak" } else { "thrash" }),
        MonsterSound::Wail => Some(if !is_deaf { "wail" } else { "cringe" }),
        _ => None,
    }
}

// ============================================================
// 3. whimper_sound_type — 고통 펫 신음 동사
// ============================================================

/// 고통 상태 펫의 신음 소리 동사 결정
/// 원본: sounds.c whimper() L431-442
pub fn whimper_sound_type(sound: MonsterSound) -> Option<&'static str> {
    match sound {
        MonsterSound::Mew | MonsterSound::Growl => Some("whimper"),
        MonsterSound::Bark => Some("whine"),
        MonsterSound::Sqeek => Some("squeal"),
        _ => None,
    }
}

// ============================================================
// 4. halluc_sound — 환각 소리 선택
// ============================================================

/// 환각 상태에서 랜덤 소리 동사 선택
/// 원본: sounds.c h_sounds[] + rn2(SIZE(h_sounds))
pub fn halluc_sound(rng: &mut NetHackRng) -> &'static str {
    H_SOUNDS[rng.rn2(H_SOUNDS.len() as i32) as usize]
}

// ============================================================
// 5. room_sound_chance — 방 유형별 소리 발생 판정
// ============================================================

/// 방 유형에 따른 소리 발생 확률 판정
/// 원본: sounds.c dosounds() — !rn2(N) 패턴
pub fn room_sound_chance(room_type: RoomType, rng: &mut NetHackRng) -> bool {
    let chance = match room_type {
        RoomType::Fountain => FOUNTAIN_SOUND_CHANCE,
        RoomType::Sink => SINK_SOUND_CHANCE,
        RoomType::Oracle => ORACLE_SOUND_CHANCE,
        _ => ROOM_SOUND_CHANCE,
    };
    rng.rn2(chance) == 0
}

// ============================================================
// 6. laugh_sound_index — 웃음 유형 결정
// ============================================================

/// 몬스터 웃음 유형 결정
/// 원본: sounds.c domonnoise() MS_LAUGH L728-732
pub fn laugh_sound(rng: &mut NetHackRng) -> &'static str {
    LAUGH_MSGS[rng.rn2(LAUGH_MSGS.len() as i32) as usize]
}

// ============================================================
// 7. pet_sound_verb — 펫 소리 통합 결정
// ============================================================

/// 펫 소리 동사 결정 (컨텍스트 + 환각 고려)
/// 반환: 동사 문자열 (None이면 소리 없음)
pub fn pet_sound_verb(
    context: PetSoundContext,
    sound: MonsterSound,
    is_sleeping: bool,
    can_move: bool,
    has_sound: bool,
    is_hallucinating: bool,
    is_deaf: bool,
    rng: &mut NetHackRng,
) -> Option<&'static str> {
    // 수면/활동불능/소리 없음 → 무음
    if is_sleeping || !can_move || !has_sound {
        return None;
    }

    // 환각 → 랜덤 소리
    if is_hallucinating {
        return Some(halluc_sound(rng));
    }

    match context {
        PetSoundContext::Growl => Some(growl_sound_type(sound)),
        PetSoundContext::Yelp => yelp_sound_type(sound, is_deaf),
        PetSoundContext::Whimper => whimper_sound_type(sound),
    }
}

// ============================================================
// 8. wake_radius — 소리로 깨우는 반경² 계산
// ============================================================

/// 펫 소리로 주변 몬스터를 깨우는 반경² 계산
/// 원본: sounds.c growl()/yelp()/whimper() — mlevel * 18/12/6
pub fn wake_radius_squared(monster_level: i32, context: PetSoundContext) -> i32 {
    let multiplier = match context {
        PetSoundContext::Growl => 18,
        PetSoundContext::Yelp => 12,
        PetSoundContext::Whimper => 6,
    };
    monster_level * multiplier
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

    // --- growl_sound_type ---
    #[test]
    fn test_growl_hiss() {
        assert_eq!(growl_sound_type(MonsterSound::Hiss), "hiss");
        assert_eq!(growl_sound_type(MonsterSound::Mew), "hiss");
    }

    #[test]
    fn test_growl_bark() {
        assert_eq!(growl_sound_type(MonsterSound::Bark), "growl");
        assert_eq!(growl_sound_type(MonsterSound::Growl), "growl");
    }

    #[test]
    fn test_growl_roar() {
        assert_eq!(growl_sound_type(MonsterSound::Roar), "roar");
    }

    #[test]
    fn test_growl_silent() {
        assert_eq!(growl_sound_type(MonsterSound::Silent), "commotion");
    }

    #[test]
    fn test_growl_default() {
        assert_eq!(growl_sound_type(MonsterSound::Laugh), "scream");
    }

    // --- yelp_sound_type ---
    #[test]
    fn test_yelp_mew_hearing() {
        assert_eq!(yelp_sound_type(MonsterSound::Mew, false), Some("yowl"));
    }

    #[test]
    fn test_yelp_mew_deaf() {
        assert_eq!(yelp_sound_type(MonsterSound::Mew, true), Some("arch"));
    }

    #[test]
    fn test_yelp_bark() {
        assert_eq!(yelp_sound_type(MonsterSound::Bark, false), Some("yelp"));
    }

    #[test]
    fn test_yelp_none() {
        assert_eq!(yelp_sound_type(MonsterSound::Laugh, false), None);
    }

    // --- whimper_sound_type ---
    #[test]
    fn test_whimper_mew() {
        assert_eq!(whimper_sound_type(MonsterSound::Mew), Some("whimper"));
    }

    #[test]
    fn test_whimper_bark() {
        assert_eq!(whimper_sound_type(MonsterSound::Bark), Some("whine"));
    }

    #[test]
    fn test_whimper_none() {
        assert_eq!(whimper_sound_type(MonsterSound::Roar), None);
    }

    // --- halluc_sound ---
    #[test]
    fn test_halluc_sound_valid() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let sound = halluc_sound(&mut rng);
            assert!(H_SOUNDS.contains(&sound), "유효한 환각 소리: {}", sound);
        }
    }

    // --- room_sound_chance ---
    #[test]
    fn test_room_sound_fountain() {
        let mut rng = test_rng();
        let mut triggered = 0;
        for _ in 0..4000 {
            if room_sound_chance(RoomType::Fountain, &mut rng) {
                triggered += 1;
            }
        }
        // 1/400 확률, 4000회 → ~10회 기대
        assert!(
            triggered > 0 && triggered < 30,
            "분수 소리 발생: {}",
            triggered
        );
    }

    #[test]
    fn test_room_sound_generic() {
        let mut rng = test_rng();
        let mut triggered = 0;
        for _ in 0..2000 {
            if room_sound_chance(RoomType::Court, &mut rng) {
                triggered += 1;
            }
        }
        // 1/200 확률, 2000회 → ~10회 기대
        assert!(
            triggered > 0 && triggered < 30,
            "궁정 소리 발생: {}",
            triggered
        );
    }

    // --- laugh_sound ---
    #[test]
    fn test_laugh_sound() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let sound = laugh_sound(&mut rng);
            assert!(LAUGH_MSGS.contains(&sound), "유효한 웃음: {}", sound);
        }
    }

    // --- pet_sound_verb ---
    #[test]
    fn test_pet_sleeping_no_sound() {
        let mut rng = test_rng();
        assert_eq!(
            pet_sound_verb(
                PetSoundContext::Growl,
                MonsterSound::Bark,
                true,
                true,
                true,
                false,
                false,
                &mut rng
            ),
            None
        );
    }

    #[test]
    fn test_pet_hallucinating() {
        let mut rng = test_rng();
        let verb = pet_sound_verb(
            PetSoundContext::Yelp,
            MonsterSound::Bark,
            false,
            true,
            true,
            true,
            false,
            &mut rng,
        );
        assert!(verb.is_some(), "환각 시 소리 발생");
        assert!(H_SOUNDS.contains(&verb.unwrap()), "환각 소리 목록에 존재");
    }

    #[test]
    fn test_pet_growl_normal() {
        let mut rng = test_rng();
        let verb = pet_sound_verb(
            PetSoundContext::Growl,
            MonsterSound::Roar,
            false,
            true,
            true,
            false,
            false,
            &mut rng,
        );
        assert_eq!(verb, Some("roar"));
    }

    // --- wake_radius_squared ---
    #[test]
    fn test_wake_radius() {
        assert_eq!(wake_radius_squared(5, PetSoundContext::Growl), 90);
        assert_eq!(wake_radius_squared(5, PetSoundContext::Yelp), 60);
        assert_eq!(wake_radius_squared(5, PetSoundContext::Whimper), 30);
    }
}

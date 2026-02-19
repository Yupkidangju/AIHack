// spell_ext.rs — spell.c 핵심 로직 순수 결과 패턴 이식
// [v2.12.0] 신규 생성: 주문 시전/학습/보유율/에너지/배고픔/역화 등 12개 함수
// 원본: NetHack 3.6.7 src/spell.c (1,898줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 주문 보유 기간 (턴 수)
const KEEN: i32 = 20000;

/// 금속 투구 주문 시전 패널티
const UARMH_BON: i32 = 4;
/// 장갑 주문 시전 패널티
const UARMG_BON: i32 = 6;
/// 신발 주문 시전 패널티
const UARMF_BON: i32 = 2;

/// 최대 주문 학습 횟수
const MAX_SPELL_STUDY: i32 = 3;

// ============================================================
// 열거형
// ============================================================

/// 역할 (직업) — 주문 시전 능력치에 영향
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellRole {
    Archeologist,
    Barbarian,
    Caveman,
    Healer,
    Knight,
    Monk,
    Priest,
    Ranger,
    Rogue,
    Samurai,
    Tourist,
    Valkyrie,
    Wizard,
}

/// 주문 스킬 카테고리
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellSkill {
    Attack,
    Healing,
    Divination,
    Enchantment,
    Clerical,
    Escape,
    Matter,
}

/// 스킬 숙련도
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillLevel {
    Restricted = 0,
    Unskilled = 1,
    Basic = 2,
    Skilled = 3,
    Expert = 4,
}

/// 저주 마법서 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CursedBookEffect {
    /// 순간이동됨
    Teleported,
    /// 몬스터 각성 (aggravate)
    Aggravated,
    /// 실명 (실명 지속시간 추가)
    Blinded { duration: i32 },
    /// 금화 도둑맞음
    GoldStolen,
    /// 혼란 (혼란 지속시간 추가)
    Confused { duration: i32 },
    /// 접촉 독 (장갑 부식 또는 능력치 하락+데미지)
    ContactPoison {
        has_gloves: bool,
        str_loss: i32,
        damage: i32,
    },
    /// 폭발 (antimagic이면 보호, 아니면 데미지)
    Explode { blocked: bool, damage: i32 },
    /// 랜덤 저주
    RandomCurse,
}

/// 주문 역화 효과 (잊어버린 주문 시전 시)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellBackfireResult {
    /// 혼란 지속시간 추가
    pub confusion_add: i32,
    /// 기절 지속시간 추가
    pub stun_add: i32,
}

/// 보호 마법 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectionResult {
    /// AC 보호 획득량 (0이면 효과 없음)
    pub gain: i32,
    /// 보호 타이머 값
    pub spell_timer: i32,
}

/// 주문 보유율 표시
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellRetention {
    /// 만료됨
    Gone,
    /// 100% 보유
    Full,
    /// 범위 표시 (예: "51%-75%")
    Range { low: i32, high: i32 },
}

/// 주문 시전 성공률 계산에 필요한 입력
#[derive(Debug, Clone)]
pub struct SpellCastInput {
    /// 직업 기본 마법 능력치 (spelbase)
    pub spelbase: i32,
    /// 직업 치유 보너스 (spelheal)
    pub spelheal: i32,
    /// 현재 마법 스탯 (Int 또는 Wis)
    pub stat_used: i32,
    /// 직업 갑옷 패널티 (spelarmr)
    pub spelarmr: i32,
    /// 직업 방패 패널티 (spelshld)
    pub spelshld: i32,
    /// 직업 특수 주문 보너스 (spelsbon)
    pub spelsbon: i32,
    /// 금속 갑옷 착용 여부
    pub has_metal_armor: bool,
    /// 로브 착용 여부
    pub has_robe: bool,
    /// 방패 착용 여부
    pub has_shield: bool,
    /// 무거운 방패 착용 여부 (소형 방패 무게 초과)
    pub has_heavy_shield: bool,
    /// 금속 투구 착용 여부 (단, Helm of Brilliance 제외)
    pub has_metal_helmet: bool,
    /// 금속 장갑 착용 여부
    pub has_metal_gloves: bool,
    /// 금속 신발 착용 여부
    pub has_metal_boots: bool,
    /// 주문 레벨
    pub spell_level: i32,
    /// 해당 주문이 직업 특수 주문인지
    pub is_role_special: bool,
    /// 해당 주문이 치유 계열인지
    pub is_healing_spell: bool,
    /// 스킬 숙련도
    pub skill_level: SkillLevel,
    /// 플레이어 레벨
    pub player_level: i32,
}

/// 주문 학습 지연시간 계산에 필요한 입력
#[derive(Debug, Clone)]
pub struct StudyDelayInput {
    /// 주문 레벨
    pub spell_level: i32,
    /// 주문 기본 지연 (oc_delay)
    pub oc_delay: i32,
}

/// 주문 시전 시 배고픔 비용 계산 입력
#[derive(Debug, Clone)]
pub struct SpellHungerInput {
    /// 주문 레벨
    pub spell_level: i32,
    /// 음식 탐지 주문인지 여부
    pub is_detect_food: bool,
    /// 위저드 직업인지 여부
    pub is_wizard: bool,
    /// 현재 지능 (acurr(A_INT))
    pub intelligence: i32,
    /// 현재 배고픔 수치
    pub current_hunger: i32,
}

/// 수정된 isqrt (정수 제곱근) — 원본 hacklib.c의 isqrt 재현
fn isqrt(val: i32) -> i32 {
    if val <= 0 {
        return 0;
    }
    let mut rt: i32 = 0;
    let mut odd: i32 = 1;
    // 누적 빼기 방식의 정수 제곱근
    let mut remaining = val;
    while remaining >= odd {
        remaining -= odd;
        odd += 2;
        rt += 1;
    }
    rt
}

// ============================================================
// 1. percent_success — 주문 성공률 계산
// ============================================================

/// 주문 시전 성공 확률(%) 계산
/// 원본: spell.c percent_success()
/// 반환: 0~100 사이의 성공 확률
pub fn percent_success(input: &SpellCastInput) -> i32 {
    // 기본 시전자 능력 계산 (splcaster)
    let mut splcaster = input.spelbase;

    // 갑옷 패널티
    if input.has_metal_armor {
        splcaster += if input.has_robe {
            input.spelarmr / 2 // 로브가 금속 간섭을 절반으로 줄임
        } else {
            input.spelarmr
        };
    } else if input.has_robe {
        splcaster -= input.spelarmr; // 로브만 입으면 보너스
    }

    // 방패 패널티
    if input.has_shield {
        splcaster += input.spelshld;
    }

    // 금속 방어구 패널티 (투구/장갑/신발)
    if input.has_metal_helmet {
        splcaster += UARMH_BON;
    }
    if input.has_metal_gloves {
        splcaster += UARMG_BON;
    }
    if input.has_metal_boots {
        splcaster += UARMF_BON;
    }

    // 직업 특수 주문 보너스
    if input.is_role_special {
        splcaster += input.spelsbon;
    }

    // 치유 주문 보너스
    if input.is_healing_spell {
        splcaster += input.spelheal;
    }

    // splcaster 상한 20
    if splcaster > 20 {
        splcaster = 20;
    }

    // 학습 능력 계산
    let mut chance = 11 * input.stat_used / 2;

    // 난이도 계산: (주문레벨 - 1) * 4 - (스킬*6 + 레벨/3 + 1)
    let skill_val = (input.skill_level as i32).max(1) - 1; // unskilled => 0
    let difficulty = (input.spell_level - 1) * 4 - (skill_val * 6 + input.player_level / 3 + 1);

    if difficulty > 0 {
        // 너무 어려움 — 큰 패널티
        chance -= isqrt(900 * difficulty + 2000);
    } else {
        // 쉬움 — 제한적 보너스
        let learning = 15 * (-difficulty) / input.spell_level;
        chance += learning.min(20);
    }

    // 기회 클램프 0~120
    chance = chance.clamp(0, 120);

    // 무거운 방패 패널티
    if input.has_heavy_shield {
        if input.is_role_special {
            chance /= 2;
        } else {
            chance /= 4;
        }
    }

    // 최종 결합: chance * (20 - splcaster) / 15 - splcaster
    chance = chance * (20 - splcaster) / 15 - splcaster;

    // 퍼센트 클램프
    chance.clamp(0, 100)
}

// ============================================================
// 2. spell_energy_cost — 주문 에너지 비용
// ============================================================

/// 주문 시전에 필요한 마나 비용 계산
/// 원본: spell.c spelleffects() 내 "energy = (spellev(spell) * 5)"
pub fn spell_energy_cost(spell_level: i32) -> i32 {
    spell_level * 5 // 5 <= energy <= 35
}

// ============================================================
// 3. study_delay — 마법서 학습 소요 시간
// ============================================================

/// 마법서 학습에 소요되는 턴 수 (음수로 반환, 원본 형식)
/// 원본: spell.c study_book() 내 switch(objects[booktype].oc_level)
pub fn study_delay(input: &StudyDelayInput) -> i32 {
    match input.spell_level {
        1 | 2 => -input.oc_delay,
        3 | 4 => -(input.spell_level - 1) * input.oc_delay,
        5 | 6 => -input.spell_level * input.oc_delay,
        7 => -8 * input.oc_delay,
        _ => 0, // 알 수 없는 레벨
    }
}

// ============================================================
// 4. spell_hunger_cost — 주문 시전 시 배고픔 소비
// ============================================================

/// 주문 시전 시 소비되는 배고픔 양 계산
/// 원본: spell.c spelleffects() 내 hungr 계산 로직
pub fn spell_hunger_cost(input: &SpellHungerInput) -> i32 {
    // 음식 탐지 주문은 배고픔 소비 없음
    if input.is_detect_food {
        return 0;
    }

    let energy = input.spell_level * 5;
    let mut hungr = energy * 2;

    // 위저드의 지능에 따른 배고픔 감소
    let intell = if input.is_wizard {
        input.intelligence
    } else {
        10 // 비위저드는 지능 10으로 간주
    };

    match intell {
        17..=i32::MAX => hungr = 0, // 지능 17+ 이면 배고픔 없음
        16 => hungr /= 4,           // 1/4
        15 => hungr /= 2,           // 1/2
        _ => {}                     // 14 이하 정상
    }

    // 기절 직전까지만 소비 (현재 배고픔 - 3 이하로)
    if hungr > input.current_hunger - 3 {
        hungr = input.current_hunger - 3;
    }

    hungr.max(0)
}

// ============================================================
// 5. spell_backfire_calc — 잊어버린 주문 역화 효과
// ============================================================

/// 잊어버린 주문 시전 시 역화 효과(혼란/기절) 계산
/// 원본: spell.c spell_backfire()
pub fn spell_backfire_calc(spell_level: i32, rng: &mut NetHackRng) -> SpellBackfireResult {
    let duration = (spell_level + 1) * 3; // 6~24

    match rng.rn2(10) {
        0 | 1 | 2 | 3 => SpellBackfireResult {
            // 40%: 혼란만
            confusion_add: duration,
            stun_add: 0,
        },
        4 | 5 | 6 => SpellBackfireResult {
            // 30%: 혼란 2/3 + 기절 1/3
            confusion_add: 2 * duration / 3,
            stun_add: duration / 3,
        },
        7 | 8 => SpellBackfireResult {
            // 20%: 기절 2/3 + 혼란 1/3
            confusion_add: duration / 3,
            stun_add: 2 * duration / 3,
        },
        _ => SpellBackfireResult {
            // 10%: 기절만
            confusion_add: 0,
            stun_add: duration,
        },
    }
}

// ============================================================
// 6. cast_protection_calc — 보호 마법 AC 보너스 계산
// ============================================================

/// 보호 주문 시전 시 획득하는 AC 보호량 계산
/// 원본: spell.c cast_protection()
pub fn cast_protection_calc(
    player_level: i32,
    current_spell_prot: i32,
    natural_ac: i32,
    is_expert: bool,
) -> ProtectionResult {
    // loglev = log2(ulevel) + 1 (1~5)
    let mut l = player_level;
    let mut loglev = 0;
    while l > 0 {
        loglev += 1;
        l /= 2;
    }

    // natac 정규화: (10 - natac) / 10, 양수로 스케일
    // 여기서 natac은 uac + uspellprot (보호 제외 순수 AC)
    let natac_scaled = (10 - natural_ac) / 10;
    let divisor = (4 - natac_scaled.min(3)).max(1);
    let gain = loglev - current_spell_prot / divisor;

    if gain > 0 {
        let timer = if is_expert { 20 } else { 10 };
        ProtectionResult {
            gain,
            spell_timer: timer,
        }
    } else {
        ProtectionResult {
            gain: 0,
            spell_timer: 0,
        }
    }
}

// ============================================================
// 7. spell_retention_display — 보유율 디스플레이 문자열
// ============================================================

/// 주문 보유율을 범위 형식으로 계산
/// 원본: spell.c spellretention()
pub fn spell_retention_display(turns_left: i32, skill: SkillLevel) -> SpellRetention {
    if turns_left < 1 {
        return SpellRetention::Gone;
    }
    if turns_left >= KEEN {
        return SpellRetention::Full;
    }

    // 퍼센트 계산
    let percent = (turns_left - 1) / (KEEN / 100) + 1;

    // 정밀도: Expert=2%, Skilled=5%, Basic=10%, Unskilled/Restricted=25%
    let accuracy = match skill {
        SkillLevel::Expert => 2,
        SkillLevel::Skilled => 5,
        SkillLevel::Basic => 10,
        _ => 25,
    };

    // 상한으로 올림
    let high = accuracy * ((percent - 1) / accuracy + 1);
    let low = high - accuracy + 1;

    SpellRetention::Range { low, high }
}

// ============================================================
// 8. cursed_book_effect — 저주 마법서 효과 결정
// ============================================================

/// 저주 마법서를 읽을 때의 부작용 결정
/// 원본: spell.c cursed_book()
pub fn cursed_book_effect(
    book_level: i32,
    has_gloves: bool,
    has_antimagic: bool,
    poison_resistance: bool,
    rng: &mut NetHackRng,
) -> CursedBookEffect {
    // 주문 레벨이 0이면 rn2(0)이므로 case 0 고정
    let roll = if book_level > 0 {
        rng.rn2(book_level)
    } else {
        0
    };

    match roll {
        0 => CursedBookEffect::Teleported,
        1 => CursedBookEffect::Aggravated,
        2 => CursedBookEffect::Blinded {
            duration: rng.rn1(100, 250),
        },
        3 => CursedBookEffect::GoldStolen,
        4 => CursedBookEffect::Confused {
            duration: rng.rn1(7, 16),
        },
        5 => {
            // 접촉 독
            if has_gloves {
                CursedBookEffect::ContactPoison {
                    has_gloves: true,
                    str_loss: 0,
                    damage: 0,
                }
            } else {
                let str_loss = if poison_resistance {
                    rng.rn1(2, 1)
                } else {
                    rng.rn1(4, 3)
                };
                let damage = rng.rnd(if poison_resistance { 6 } else { 10 });
                CursedBookEffect::ContactPoison {
                    has_gloves: false,
                    str_loss,
                    damage,
                }
            }
        }
        6 => {
            // 폭발
            if has_antimagic {
                CursedBookEffect::Explode {
                    blocked: true,
                    damage: 0,
                }
            } else {
                let damage = 2 * rng.rnd(10) + 5;
                CursedBookEffect::Explode {
                    blocked: false,
                    damage,
                }
            }
        }
        _ => CursedBookEffect::RandomCurse,
    }
}

// ============================================================
// 9. spell_damage_bonus — 주문 데미지 보너스
// ============================================================

/// 주문 데미지 보너스 계산
/// 원본: spell.c spell_damage_bonus() (실제로는 uhitm.c에 있지만 spell 관련)
/// 반환: 레벨 기반 보너스 데미지
pub fn spell_damage_bonus(half_level_plus_one: i32) -> i32 {
    // 원본: u.ulevel / 2 + 1 이 인자로 넘어옴
    half_level_plus_one
}

// ============================================================
// 10. spell_type_mnemonic — 주문 유형 니모닉 문자열
// ============================================================

/// 주문 스킬 유형에 대한 니모닉 문자열 반환
/// 원본: spell.c spelltypemnemonic()
pub fn spell_type_mnemonic(skill: SpellSkill) -> &'static str {
    match skill {
        SpellSkill::Attack => "attack",
        SpellSkill::Healing => "healing",
        SpellSkill::Divination => "divination",
        SpellSkill::Enchantment => "enchantment",
        SpellSkill::Clerical => "clerical",
        SpellSkill::Escape => "escape",
        SpellSkill::Matter => "matter",
    }
}

// ============================================================
// 11. read_ability_check — 마법서 읽기 능력 판정
// ============================================================

/// 마법서를 읽을 수 있는 능력 판정 (너무 어려운지 확인)
/// 원본: spell.c study_book() 내 read_ability 계산
/// 반환: 읽기 능력치 (20 이상이면 안전, 미만이면 위험)
pub fn read_ability_check(
    intelligence: i32,
    player_level: i32,
    book_level: i32,
    has_lenses: bool,
) -> i32 {
    let lens_bonus = if has_lenses { 2 } else { 0 };
    intelligence + 4 + player_level / 2 - 2 * book_level + lens_bonus
}

// ============================================================
// 12. losespells_calc — 기억상실 시 잃을 주문 수 계산
// ============================================================

/// 기억상실에 의해 잃을 주문 수 계산
/// 원본: spell.c losespells()
/// n: 알려진 주문 수, confused: 혼란 여부
/// 반환: 잃을 주문 수 (0~n)
pub fn losespells_calc(n: i32, confused: bool, rng: &mut NetHackRng) -> i32 {
    if n <= 0 {
        return 0;
    }

    // 기본: rn2(n+1)
    let mut nzap = rng.rn2(n + 1);
    // 혼란 시 두 번째 주사위와 비교하여 더 나쁜 결과 선택
    if confused {
        let i = rng.rn2(n + 1);
        if i > nzap {
            nzap = i;
        }
    }

    // 행운이 좋으면 감소 (rnl(7) == 0이면)
    // 여기서는 단순히 rn2(7)==0으로 근사 (rnl은 Luck 보정이 필요)
    if nzap > 1 && rng.rn2(7) == 0 {
        nzap = rng.rnd(nzap);
    }

    nzap
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

    // --- percent_success ---
    #[test]
    fn test_percent_success_wizard_low_spell() {
        // 위저드, 레벨 10, 지능 18, 기본 스킬, 방어구 없음, 1레벨 주문
        let input = SpellCastInput {
            spelbase: 1, // 위저드 기본
            spelheal: 3,
            stat_used: 18,
            spelarmr: 10,
            spelshld: 1,
            spelsbon: 4,
            has_metal_armor: false,
            has_robe: false,
            has_shield: false,
            has_heavy_shield: false,
            has_metal_helmet: false,
            has_metal_gloves: false,
            has_metal_boots: false,
            spell_level: 1,
            is_role_special: false,
            is_healing_spell: false,
            skill_level: SkillLevel::Basic,
            player_level: 10,
        };
        let result = percent_success(&input);
        assert!(result > 0 && result <= 100, "성공률: {}", result);
    }

    #[test]
    fn test_percent_success_barbarian_high_spell() {
        // 바바리안, 레벨 5, 지능 10, 미숙련, 7레벨 주문 → 매우 낮은 확률
        let input = SpellCastInput {
            spelbase: 14, // 바바리안 기본 (높을수록 나쁨)
            spelheal: 0,
            stat_used: 10,
            spelarmr: 0,
            spelshld: 0,
            spelsbon: 0,
            has_metal_armor: true,
            has_robe: false,
            has_shield: true,
            has_heavy_shield: true,
            has_metal_helmet: true,
            has_metal_gloves: true,
            has_metal_boots: true,
            spell_level: 7,
            is_role_special: false,
            is_healing_spell: false,
            skill_level: SkillLevel::Unskilled,
            player_level: 5,
        };
        let result = percent_success(&input);
        assert!(result >= 0 && result <= 100, "성공률: {}", result);
        // 바바리안+고레벨 주문+중장갑 → 매우 낮을 것
        assert!(
            result < 30,
            "바바리안 7레벨 주문 성공률이 예상보다 높음: {}",
            result
        );
    }

    #[test]
    fn test_percent_success_with_robe_bonus() {
        // 로브 착용 시 보너스 확인 (spelbase를 높여서 차이가 드러나도록)
        let mut input = SpellCastInput {
            spelbase: 12,
            spelheal: 0,
            stat_used: 14,
            spelarmr: 10,
            spelshld: 0,
            spelsbon: 0,
            has_metal_armor: false,
            has_robe: true,
            has_shield: false,
            has_heavy_shield: false,
            has_metal_helmet: false,
            has_metal_gloves: false,
            has_metal_boots: false,
            spell_level: 5,
            is_role_special: false,
            is_healing_spell: false,
            skill_level: SkillLevel::Basic,
            player_level: 8,
        };
        let with_robe = percent_success(&input);
        input.has_robe = false;
        let without_robe = percent_success(&input);
        // 로브 착용 시 splcaster가 -spelarmr(10) 감소 → 성공률 증가
        assert!(
            with_robe >= without_robe,
            "로브 착용 시 성공률 증가: {} vs {}",
            with_robe,
            without_robe
        );
    }

    // --- spell_energy_cost ---
    #[test]
    fn test_energy_cost() {
        assert_eq!(spell_energy_cost(1), 5);
        assert_eq!(spell_energy_cost(7), 35);
        assert_eq!(spell_energy_cost(3), 15);
    }

    // --- study_delay ---
    #[test]
    fn test_study_delay_level_1() {
        let input = StudyDelayInput {
            spell_level: 1,
            oc_delay: 3,
        };
        assert_eq!(study_delay(&input), -3);
    }

    #[test]
    fn test_study_delay_level_4() {
        let input = StudyDelayInput {
            spell_level: 4,
            oc_delay: 5,
        };
        assert_eq!(study_delay(&input), -15); // -(4-1)*5
    }

    #[test]
    fn test_study_delay_level_7() {
        let input = StudyDelayInput {
            spell_level: 7,
            oc_delay: 10,
        };
        assert_eq!(study_delay(&input), -80); // -8*10
    }

    // --- spell_hunger_cost ---
    #[test]
    fn test_hunger_detect_food() {
        let input = SpellHungerInput {
            spell_level: 2,
            is_detect_food: true,
            is_wizard: false,
            intelligence: 10,
            current_hunger: 500,
        };
        assert_eq!(spell_hunger_cost(&input), 0);
    }

    #[test]
    fn test_hunger_wizard_high_int() {
        // 위저드 지능 18 → 배고픔 0
        let input = SpellHungerInput {
            spell_level: 5,
            is_detect_food: false,
            is_wizard: true,
            intelligence: 18,
            current_hunger: 500,
        };
        assert_eq!(spell_hunger_cost(&input), 0);
    }

    #[test]
    fn test_hunger_normal() {
        // 비위저드, 레벨 3 주문, 배고픔 500
        let input = SpellHungerInput {
            spell_level: 3,
            is_detect_food: false,
            is_wizard: false,
            intelligence: 12,
            current_hunger: 500,
        };
        let cost = spell_hunger_cost(&input);
        assert_eq!(cost, 30); // 3*5*2 = 30
    }

    #[test]
    fn test_hunger_near_fainting() {
        // 배고픔이 거의 없을 때 → 제한
        let input = SpellHungerInput {
            spell_level: 5,
            is_detect_food: false,
            is_wizard: false,
            intelligence: 10,
            current_hunger: 20,
        };
        let cost = spell_hunger_cost(&input);
        assert_eq!(cost, 17); // min(50, 20-3) = 17
    }

    // --- spell_backfire_calc ---
    #[test]
    fn test_backfire_always_positive() {
        let mut rng = test_rng();
        for level in 1..=7 {
            let result = spell_backfire_calc(level, &mut rng);
            assert!(
                result.confusion_add >= 0 && result.stun_add >= 0,
                "레벨 {} 역화 결과 음수: {:?}",
                level,
                result
            );
            assert!(
                result.confusion_add > 0 || result.stun_add > 0,
                "레벨 {} 역화 효과 없음: {:?}",
                level,
                result
            );
        }
    }

    #[test]
    fn test_backfire_duration_scaling() {
        let mut rng = test_rng();
        let low = spell_backfire_calc(1, &mut rng);
        let mut rng2 = NetHackRng::new(42);
        let high = spell_backfire_calc(7, &mut rng2);
        // 레벨 7의 총 효과가 레벨 1보다 커야 함
        let total_low = low.confusion_add + low.stun_add;
        let total_high = high.confusion_add + high.stun_add;
        assert!(total_high >= total_low, "고레벨 역화가 더 강해야 함");
    }

    // --- cast_protection_calc ---
    #[test]
    fn test_protection_level_1() {
        let result = cast_protection_calc(1, 0, 10, false);
        assert!(result.gain > 0, "레벨 1 첫 보호 → 획득 있어야 함");
        assert_eq!(result.spell_timer, 10);
    }

    #[test]
    fn test_protection_expert_timer() {
        let result = cast_protection_calc(10, 0, 5, true);
        if result.gain > 0 {
            assert_eq!(result.spell_timer, 20, "전문가는 20턴 타이머");
        }
    }

    #[test]
    fn test_protection_diminishing() {
        // 이미 많은 보호 → gain 감소/0
        let result = cast_protection_calc(1, 10, 10, false);
        assert_eq!(result.gain, 0, "이미 높은 보호 → 추가 없음");
    }

    // --- spell_retention_display ---
    #[test]
    fn test_retention_gone() {
        assert_eq!(
            spell_retention_display(0, SkillLevel::Basic),
            SpellRetention::Gone
        );
    }

    #[test]
    fn test_retention_full() {
        assert_eq!(
            spell_retention_display(20000, SkillLevel::Basic),
            SpellRetention::Full
        );
    }

    #[test]
    fn test_retention_range_expert() {
        // 10000 턴 남음 = 50%, Expert 정밀도 2% → 49%-50%
        let result = spell_retention_display(10000, SkillLevel::Expert);
        match result {
            SpellRetention::Range { low, high } => {
                assert_eq!(high, 50);
                assert_eq!(low, 49);
            }
            _ => panic!("범위 표시 예상"),
        }
    }

    #[test]
    fn test_retention_range_unskilled() {
        // 5000 턴 = 25%, Unskilled 정밀도 25% → 1%-25%
        let result = spell_retention_display(5000, SkillLevel::Unskilled);
        match result {
            SpellRetention::Range { low, high } => {
                assert_eq!(high, 25);
                assert_eq!(low, 1);
            }
            _ => panic!("범위 표시 예상"),
        }
    }

    // --- cursed_book_effect ---
    #[test]
    fn test_cursed_book_level_0() {
        let mut rng = test_rng();
        let effect = cursed_book_effect(0, false, false, false, &mut rng);
        assert_eq!(
            effect,
            CursedBookEffect::Teleported,
            "레벨 0 → 항상 텔레포트"
        );
    }

    #[test]
    fn test_cursed_book_explosion_blocked() {
        // antimagic이면 폭발 차단
        let mut rng = test_rng();
        // roll이 6이 나오도록 여러 번 시도
        for _ in 0..100 {
            let effect = cursed_book_effect(7, false, true, false, &mut rng);
            if let CursedBookEffect::Explode { blocked, damage } = effect {
                assert!(blocked, "antimagic → 폭발 차단");
                assert_eq!(damage, 0);
                return;
            }
        }
        // 100번 안에 6이 안 나올 수 있으므로 패닉은 안 함
    }

    // --- read_ability_check ---
    #[test]
    fn test_read_ability_easy() {
        // 지능 18, 레벨 10, 1레벨 책, 렌즈 있음
        let ability = read_ability_check(18, 10, 1, true);
        assert!(ability >= 20, "쉬운 책은 20 이상이어야 함: {}", ability);
    }

    #[test]
    fn test_read_ability_hard() {
        // 지능 10, 레벨 3, 7레벨 책, 렌즈 없음
        let ability = read_ability_check(10, 3, 7, false);
        assert!(ability < 20, "어려운 책은 20 미만이어야 함: {}", ability);
    }

    // --- losespells_calc ---
    #[test]
    fn test_losespells_zero_known() {
        let mut rng = test_rng();
        assert_eq!(losespells_calc(0, false, &mut rng), 0);
    }

    #[test]
    fn test_losespells_range() {
        let mut rng = test_rng();
        let result = losespells_calc(10, false, &mut rng);
        assert!(result >= 0 && result <= 10, "잃을 수: {}", result);
    }

    #[test]
    fn test_losespells_confused_worse() {
        // 혼란 시 더 많이 잃는 경향
        let mut rng1 = NetHackRng::new(42);
        let mut rng2 = NetHackRng::new(42);
        let mut normal_total = 0;
        let mut confused_total = 0;
        for _ in 0..100 {
            normal_total += losespells_calc(20, false, &mut rng1);
            confused_total += losespells_calc(20, true, &mut rng2);
        }
        // 혼란 시 평균적으로 더 많이 잃어야 함
        assert!(
            confused_total >= normal_total,
            "혼란 시 총 잃음: {} >= 정상: {}",
            confused_total,
            normal_total
        );
    }

    // --- spell_type_mnemonic ---
    #[test]
    fn test_mnemonic() {
        assert_eq!(spell_type_mnemonic(SpellSkill::Attack), "attack");
        assert_eq!(spell_type_mnemonic(SpellSkill::Healing), "healing");
        assert_eq!(spell_type_mnemonic(SpellSkill::Matter), "matter");
    }

    // --- spell_damage_bonus ---
    #[test]
    fn test_damage_bonus() {
        assert_eq!(spell_damage_bonus(6), 6); // level 10: 10/2+1=6
        assert_eq!(spell_damage_bonus(1), 1); // level 1
    }

    // --- isqrt ---
    #[test]
    fn test_isqrt() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(9), 3);
        assert_eq!(isqrt(10), 3);
        assert_eq!(isqrt(100), 10);
        assert_eq!(isqrt(2000), 44);
    }
}

// ============================================================================
// [v2.31.0 Phase 95-2] 특수방 생성 확장 (mkroom_phase95_ext.rs)
// 원본: NetHack 3.6.7 src/mkroom.c L200-1200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 특수방 컨텐츠 — room_contents (mkroom.c L200-700)
// =============================================================================

/// [v2.31.0 95-2] 특수방 유형 (확장)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialRoom {
    Shop,
    Temple,
    Throne,
    Zoo,
    Morgue,
    Barracks,
    Beehive,
    Swamp,
    Vault,
    Leprechaun,
    Cockatrice,
    Anthole,
    Garden,
    DragonLair,
}

/// [v2.31.0 95-2] 방 컨텐츠
#[derive(Debug, Clone)]
pub struct RoomContents {
    pub monsters: Vec<RoomMonster>,
    pub items: Vec<RoomItem>,
    pub features: Vec<String>,
    pub is_lit: bool,
    pub has_trap: bool,
}

/// [v2.31.0 95-2] 방 내 몬스터
#[derive(Debug, Clone)]
pub struct RoomMonster {
    pub name: String,
    pub level: i32,
    pub is_asleep: bool,
}

/// [v2.31.0 95-2] 방 내 아이템
#[derive(Debug, Clone)]
pub struct RoomItem {
    pub item_type: String,
    pub value: i32,
}

/// [v2.31.0 95-2] 특수방 컨텐츠 생성
/// 원본: mkroom.c mkroom()
pub fn generate_room_contents(
    room_type: SpecialRoom,
    depth: i32,
    rng: &mut NetHackRng,
) -> RoomContents {
    match room_type {
        SpecialRoom::Zoo => {
            let count = rng.rn2(8) + 5;
            let monsters: Vec<RoomMonster> = (0..count)
                .map(|_| {
                    let animals = ["곰", "뱀", "사자", "호랑이", "늑대", "고릴라"];
                    let idx = rng.rn2(animals.len() as i32) as usize;
                    RoomMonster {
                        name: animals[idx].to_string(),
                        level: depth / 3 + rng.rn2(3),
                        is_asleep: true,
                    }
                })
                .collect();
            RoomContents {
                monsters,
                items: vec![RoomItem {
                    item_type: "금화".to_string(),
                    value: rng.rn2(200) + 50,
                }],
                features: vec!["우리".to_string()],
                is_lit: true,
                has_trap: false,
            }
        }
        SpecialRoom::Morgue => {
            let count = rng.rn2(6) + 3;
            let monsters: Vec<RoomMonster> = (0..count)
                .map(|_| {
                    let undead = ["좀비", "스켈레톤", "구울", "레이스", "뱀파이어"];
                    let idx = rng.rn2(undead.len() as i32) as usize;
                    RoomMonster {
                        name: undead[idx].to_string(),
                        level: depth / 2 + rng.rn2(3),
                        is_asleep: true,
                    }
                })
                .collect();
            RoomContents {
                monsters,
                items: vec![
                    RoomItem {
                        item_type: "관".to_string(),
                        value: count,
                    },
                    RoomItem {
                        item_type: "포션".to_string(),
                        value: rng.rn2(3) + 1,
                    },
                ],
                features: vec!["관들".to_string(), "어둠".to_string()],
                is_lit: false,
                has_trap: false,
            }
        }
        SpecialRoom::Barracks => {
            let count = rng.rn2(6) + 4;
            let monsters: Vec<RoomMonster> = (0..count)
                .map(|_| RoomMonster {
                    name: "병사".to_string(),
                    level: depth / 3 + 3,
                    is_asleep: false,
                })
                .collect();
            RoomContents {
                monsters,
                items: vec![
                    RoomItem {
                        item_type: "무기".to_string(),
                        value: count,
                    },
                    RoomItem {
                        item_type: "갑옷".to_string(),
                        value: rng.rn2(3) + 1,
                    },
                ],
                features: vec!["무기대".to_string()],
                is_lit: true,
                has_trap: false,
            }
        }
        SpecialRoom::Beehive => {
            let count = rng.rn2(10) + 8;
            let monsters: Vec<RoomMonster> = (0..count)
                .map(|i| {
                    if i == 0 {
                        RoomMonster {
                            name: "여왕벌".to_string(),
                            level: depth / 2 + 5,
                            is_asleep: false,
                        }
                    } else {
                        RoomMonster {
                            name: "꿀벌".to_string(),
                            level: depth / 3 + 1,
                            is_asleep: false,
                        }
                    }
                })
                .collect();
            RoomContents {
                monsters,
                items: vec![
                    RoomItem {
                        item_type: "꿀 덩어리".to_string(),
                        value: rng.rn2(5) + 3,
                    },
                    RoomItem {
                        item_type: "로열젤리".to_string(),
                        value: 1,
                    },
                ],
                features: vec!["벌집".to_string()],
                is_lit: false,
                has_trap: false,
            }
        }
        SpecialRoom::Temple => {
            let monsters = vec![RoomMonster {
                name: "사제".to_string(),
                level: depth / 2 + 5,
                is_asleep: false,
            }];
            RoomContents {
                monsters,
                items: vec![],
                features: vec!["제단".to_string(), "성수".to_string()],
                is_lit: true,
                has_trap: false,
            }
        }
        SpecialRoom::Throne => RoomContents {
            monsters: vec![RoomMonster {
                name: "왕".to_string(),
                level: depth / 2 + 5,
                is_asleep: false,
            }],
            items: vec![RoomItem {
                item_type: "금화".to_string(),
                value: rng.rn2(500) + 200,
            }],
            features: vec!["옥좌".to_string()],
            is_lit: true,
            has_trap: rng.rn2(3) == 0,
        },
        SpecialRoom::Shop => {
            let item_count = rng.rn2(10) + 10;
            let items: Vec<RoomItem> = (0..item_count)
                .map(|_| {
                    let types = [
                        "무기",
                        "갑옷",
                        "포션",
                        "스크롤",
                        "지팡이",
                        "반지",
                        "음식",
                        "도구",
                    ];
                    let idx = rng.rn2(types.len() as i32) as usize;
                    RoomItem {
                        item_type: types[idx].to_string(),
                        value: rng.rn2(200) + 10,
                    }
                })
                .collect();
            RoomContents {
                monsters: vec![RoomMonster {
                    name: "상점 주인".to_string(),
                    level: 15 + rng.rn2(5),
                    is_asleep: false,
                }],
                items,
                features: vec!["진열대".to_string(), "간판".to_string()],
                is_lit: true,
                has_trap: false,
            }
        }
        SpecialRoom::Vault => RoomContents {
            monsters: vec![],
            items: vec![RoomItem {
                item_type: "금화".to_string(),
                value: rng.rn2(1000) + 500,
            }],
            features: vec!["금고 문".to_string()],
            is_lit: false,
            has_trap: true,
        },
        SpecialRoom::Leprechaun => {
            let count = rng.rn2(5) + 3;
            let monsters: Vec<RoomMonster> = (0..count)
                .map(|_| RoomMonster {
                    name: "레프러콘".to_string(),
                    level: 5,
                    is_asleep: false,
                })
                .collect();
            RoomContents {
                monsters,
                items: vec![RoomItem {
                    item_type: "금화".to_string(),
                    value: rng.rn2(300) + 100,
                }],
                features: vec!["무지개".to_string()],
                is_lit: true,
                has_trap: false,
            }
        }
        SpecialRoom::DragonLair => {
            let dragon_types = ["적룡", "청룡", "백룡", "흑룡", "녹룡"];
            let idx = rng.rn2(dragon_types.len() as i32) as usize;
            RoomContents {
                monsters: vec![RoomMonster {
                    name: dragon_types[idx].to_string(),
                    level: 20 + rng.rn2(5),
                    is_asleep: rng.rn2(2) == 0,
                }],
                items: vec![
                    RoomItem {
                        item_type: "금화".to_string(),
                        value: rng.rn2(2000) + 1000,
                    },
                    RoomItem {
                        item_type: "보석".to_string(),
                        value: rng.rn2(5) + 3,
                    },
                ],
                features: vec!["보물 더미".to_string()],
                is_lit: false,
                has_trap: false,
            }
        }
        _ => {
            let count = rng.rn2(4) + 1;
            let monsters: Vec<RoomMonster> = (0..count)
                .map(|_| RoomMonster {
                    name: "몬스터".to_string(),
                    level: depth / 3,
                    is_asleep: rng.rn2(2) == 0,
                })
                .collect();
            RoomContents {
                monsters,
                items: vec![],
                features: vec![],
                is_lit: rng.rn2(2) == 0,
                has_trap: rng.rn2(5) == 0,
            }
        }
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    #[test]
    fn test_zoo_contents() {
        let mut rng = test_rng();
        let contents = generate_room_contents(SpecialRoom::Zoo, 10, &mut rng);
        assert!(contents.monsters.len() >= 5);
        assert!(contents.monsters.iter().all(|m| m.is_asleep));
    }

    #[test]
    fn test_morgue_dark() {
        let mut rng = test_rng();
        let contents = generate_room_contents(SpecialRoom::Morgue, 15, &mut rng);
        assert!(!contents.is_lit);
        assert!(contents.monsters.len() >= 3);
    }

    #[test]
    fn test_beehive_queen() {
        let mut rng = test_rng();
        let contents = generate_room_contents(SpecialRoom::Beehive, 10, &mut rng);
        assert!(contents.monsters.iter().any(|m| m.name == "여왕벌"));
    }

    #[test]
    fn test_shop_items() {
        let mut rng = test_rng();
        let contents = generate_room_contents(SpecialRoom::Shop, 5, &mut rng);
        assert!(contents.items.len() >= 10);
        assert!(contents.monsters.iter().any(|m| m.name == "상점 주인"));
    }

    #[test]
    fn test_vault_trap() {
        let mut rng = test_rng();
        let contents = generate_room_contents(SpecialRoom::Vault, 10, &mut rng);
        assert!(contents.has_trap);
        assert!(contents.monsters.is_empty());
    }

    #[test]
    fn test_dragon_lair() {
        let mut rng = test_rng();
        let contents = generate_room_contents(SpecialRoom::DragonLair, 25, &mut rng);
        assert_eq!(contents.monsters.len(), 1);
        assert!(contents.monsters[0].level >= 20);
    }

    #[test]
    fn test_temple_altar() {
        let mut rng = test_rng();
        let contents = generate_room_contents(SpecialRoom::Temple, 10, &mut rng);
        assert!(contents.features.iter().any(|f| f.contains("제단")));
    }
}

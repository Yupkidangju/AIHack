// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::entity::{Dialogue, Monster, PlayerTag, Position, Talkative};
use crate::core::game_state::Direction;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;
use serde::{Deserialize, Serialize};

/// 오라클 소문(Rumors) 에셋
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rumors {
    pub true_rumors: Vec<String>,
    pub false_rumors: Vec<String>,
    pub oracles: Vec<String>,
    pub epitaphs: Vec<String>,
    pub engravings: Vec<String>,
}

impl Rumors {
    pub fn new() -> Self {
        let mut r = Self {
            true_rumors: Vec::new(),
            false_rumors: Vec::new(),
            oracles: Vec::new(),
            epitaphs: Vec::new(),
            engravings: Vec::new(),
        };
        r.load_all();
        r
    }

    pub fn load_all(&mut self) {
        use std::fs;
        // 1. True Rumors
        if let Ok(content) = fs::read_to_string("assets/dat/rumors.tru") {
            self.true_rumors = content
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        // 2. False Rumors
        if let Ok(content) = fs::read_to_string("assets/dat/rumors.fal") {
            self.false_rumors = content
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        // 3. Oracles
        if let Ok(content) = fs::read_to_string("assets/dat/oracles.txt") {
            // Oracles are separated by "-----"
            self.oracles = content
                .split("-----")
                .map(|s| s.trim().replace("\n", " "))
                .filter(|s| !s.is_empty() && s.len() > 10)
                .collect();
        }
        // 4. Epitaphs
        if let Ok(content) = fs::read_to_string("assets/dat/epitaph.txt") {
            self.epitaphs = content
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        // 5. Engravings
        if let Ok(content) = fs::read_to_string("assets/dat/engrave.txt") {
            self.engravings = content
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // 폴백 데이터 (파일 로드 실패 시)
        if self.true_rumors.is_empty() {
            self.true_rumors.push(
                "They say the Amulet of Yendor is hidden deep within the dungeon.".to_string(),
            );
        }
    }

    pub fn get_random_rumor(&self, rng: &mut NetHackRng) -> &str {
        let is_true = rng.rn2(2) == 0;
        let list = if is_true {
            &self.true_rumors
        } else {
            &self.false_rumors
        };
        if list.is_empty() {
            return "The air is silent.";
        }
        &list[rng.rn2(list.len() as i32) as usize]
    }

    pub fn get_random_oracle(&self, rng: &mut NetHackRng) -> &str {
        if self.oracles.is_empty() {
            return "Seek the Amulet.";
        }
        &self.oracles[rng.rn2(self.oracles.len() as i32) as usize]
    }

    pub fn get_random_epitaph(&self, rng: &mut NetHackRng) -> &str {
        if self.epitaphs.is_empty() {
            return "Rest In Peace.";
        }
        &self.epitaphs[rng.rn2(self.epitaphs.len() as i32) as usize]
    }

    pub fn get_random_engraving(&self, rng: &mut NetHackRng) -> &str {
        if self.engravings.is_empty() {
            return "Elbereth";
        }
        &self.engravings[rng.rn2(self.engravings.len() as i32) as usize]
    }
}

/// 대화(Talk) 시스템
pub fn try_talk(
    world: &mut SubWorld,
    dir: Direction,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
    rumors: &Rumors,
) -> bool {
    let mut p_query = <&Position>::query().filter(component::<PlayerTag>());
    let p_pos = match p_query.iter(world).next() {
        Some(pos) => *pos,
        None => return false,
    };

    let (dx, dy) = dir.to_delta();
    let tx = p_pos.x + dx;
    let ty = p_pos.y + dy;

    // 해당 좌표의 대화 가능한 대상 찾기
    let mut target_query = <(
        Entity,
        &Position,
        Option<&Dialogue>,
        Option<&Monster>,
        Option<&crate::core::entity::ShopkeeperTag>,
        Option<&crate::core::entity::QuestLeader>,
    )>::query()
    .filter(component::<Talkative>());

    let mut talked = false;
    let mut target_data = None;

    for (ent, pos, diag, monster, shk, leader) in target_query.iter(world) {
        if pos.x == tx && pos.y == ty {
            target_data = Some((
                *ent,
                diag.cloned(),
                monster.cloned(),
                shk.is_some(),
                leader.is_some(),
            ));
            break;
        }
    }

    if let Some((_ent, diag, monster, is_shk, is_leader)) = target_data {
        talked = true;

        //
        if is_shk {
            // 상점 서비스 메뉴 (Phase 47.3)
            //
            if crate::core::systems::shop::try_identify_service(world, log, turn) {
                return true;
            }
            // 감정할 것이 없거나 돈이 없으면 일반 대사로 넘어감
        }

        // 0. 퀘스트 리더(QuestLeader) 특수 대사
        if is_leader {
            let mut p_query =
                <&crate::core::entity::player::Player>::query().filter(component::<PlayerTag>());
            if let Some(p) = p_query.iter(world).next() {
                if p.exp_level < 14 {
                    log.add(
                        "The Leader says: 'You are not yet strong enough for this quest.'",
                        turn,
                    );
                } else {
                    log.add(format!("The Leader says: 'Greetings, fellow {:?}. The time has come to fulfill your destiny.'", p.role), turn);
                }
                return true;
            }
        }

        // 1. 오라클(Oracle) 특수 대사 케이스
        if let Some(m) = monster {
            if m.kind == crate::generated::MonsterKind::Oracle {
                // 1/3 확률로 Oracle Advice, 2/3 확률로 Rumor
                if rng.rn2(3) == 0 {
                    log.add(
                        format!("The Oracle speaks: '{}'", rumors.get_random_oracle(rng)),
                        turn,
                    );
                } else {
                    log.add(
                        format!("The Oracle whispers: '{}'", rumors.get_random_rumor(rng)),
                        turn,
                    );
                }
                return true;
            }
        }

        //
        if let Some(d) = diag {
            if !d.messages.is_empty() {
                //
                let msg = &d.messages[rng.rn2(d.messages.len() as i32) as usize];
                log.add(format!("The inhabitant says: '{}'", msg), turn);
            } else {
                log.add("They have nothing to say to you.", turn);
            }
        } else {
            log.add("They don't seem interested in talking.", turn);
        }
    }

    if !talked {
        if dir == Direction::Here {
            log.add("You talk to yourself. It's a lonely dungeon.", turn);
        } else {
            log.add("There is no one there to talk to.", turn);
        }
    }

    talked
}

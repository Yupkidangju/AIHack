// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::entity::status::{StatusBundle, StatusFlags};
use crate::core::entity::{Health, PlayerTag};
use crate::core::events::{EventQueue, GameEvent};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;

#[legion::system]
#[write_component(StatusBundle)]
#[write_component(crate::core::entity::player::Player)]
#[write_component(Health)]
#[read_component(PlayerTag)]
pub fn status_tick(
    world: &mut SubWorld,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] rng: &mut NetHackRng,
    #[resource] event_queue: &mut EventQueue,
) {
    let mut query = <(
        Entity,
        &mut StatusBundle,
        &mut Health,
        Option<&mut crate::core::entity::player::Player>,
        Option<&PlayerTag>,
    )>::query();

    for (_ent, status, health, player, is_player) in query.iter_mut(world) {
        let expired = status.tick();

        if let Some(p) = player {
            // Player-specific timeouts
            // 1. Prayer cooldown
            if p.prayer_cooldown > 0 {
                p.prayer_cooldown -= 1;
            }

            // 2. Luck timeout (luck_tick)
            if p.luck != 0 {
                if p.luck_turns > 0 {
                    p.luck_turns -= 1;
                } else {
                    if p.luck > 0 {
                        p.luck -= 1;
                    } else if p.luck < 0 {
                        p.luck += 1;
                    }
                    p.luck_turns = 600;
                }
            }

            // 3. Attribute Recovery (timeout.c)
            if p.attribute_recovery_turns > 0 {
                p.attribute_recovery_turns -= 1;
            } else {
                let mut recovered = false;
                if p.str.base < p.str.max {
                    p.str.base += 1;
                    recovered = true;
                } else if p.dex.base < p.dex.max {
                    p.dex.base += 1;
                    recovered = true;
                } else if p.con.base < p.con.max {
                    p.con.base += 1;
                    recovered = true;
                } else if p.int.base < p.int.max {
                    p.int.base += 1;
                    recovered = true;
                } else if p.wis.base < p.wis.max {
                    p.wis.base += 1;
                    recovered = true;
                } else if p.cha.base < p.cha.max {
                    p.cha.base += 1;
                    recovered = true;
                }

                if recovered {
                    log.add("You feel your abilities returning.", *turn);
                }
                p.attribute_recovery_turns = 1500;
            }

            //
            //
            //
            //
            {
                let mut hunger_rate = 1i32;

                // [v2.0.0
                hunger_rate += p.equip_hunger_bonus;

                //
                if p.nutrition > 1000 {
                    hunger_rate += 1;
                }

                //
                //
                if hunger_rate > 0 {
                    p.nutrition -= hunger_rate;
                }
            }

            //
            let old_hunger = p.hunger;
            p.hunger = if p.nutrition > 1000 {
                crate::core::entity::player::HungerState::Satiated
            } else if p.nutrition > 150 {
                crate::core::entity::player::HungerState::NotHungry
            } else if p.nutrition > 50 {
                crate::core::entity::player::HungerState::Hungry
            } else if p.nutrition > 0 {
                crate::core::entity::player::HungerState::Weak
            } else {
                crate::core::entity::player::HungerState::Fainting
            };

            //
            if p.hunger != old_hunger {
                match p.hunger {
                    crate::core::entity::player::HungerState::Hungry => {
                        log.add("You are beginning to feel hungry.", *turn)
                    }
                    crate::core::entity::player::HungerState::Weak => {
                        log.add("You feel weak.", *turn)
                    }
                    crate::core::entity::player::HungerState::Fainting => {
                        log.add("You feel faint from lack of food.", *turn)
                    }
                    crate::core::entity::player::HungerState::Satiated => {
                        log.add("You feel stuffed.", *turn)
                    }
                    _ => {}
                }
            }

            // [v2.0.0
            //
            //
            if p.nutrition <= 0 {
                if p.nutrition < -200 {
                    //
                    log.add_colored("You die from starvation...", [255, 0, 0], *turn);
                    health.current = 0;
                } else if rng.rn2(20) == 0 && !status.flags().contains(StatusFlags::SLEEPING) {
                    //
                    log.add("You faint from lack of food.", *turn);
                    status.add(StatusFlags::SLEEPING, 10 + rng.rn2(10) as u32);

                    // [v2.0.0
                    event_queue.push(GameEvent::StatusApplied {
                        target: "player".to_string(),
                        status: StatusFlags::SLEEPING,
                        turns: 10,
                    });
                }
            }

            // 5. Periodic Status Damage
            let flags = status.flags();
            if flags.contains(StatusFlags::POISONED) && !flags.contains(StatusFlags::POISON_RES) {
                if *turn % 15 == 0 {
                    health.current -= 1;
                    log.add("You feel the poison coursing through your veins.", *turn);
                    if rng.rn2(10) == 0 && p.str.base > 3 {
                        p.str.base -= 1;
                        log.add("You feel much weaker!", *turn);
                    }
                }
            }
            if flags.contains(StatusFlags::SICK) {
                if *turn % 10 == 0 {
                    health.current -= 2;
                    log.add("You feel very ill.", *turn);
                }
            }
        }

        if is_player.is_some() {
            for flag in expired {
                // [v2.0.0
                event_queue.push(GameEvent::StatusExpired {
                    target: "player".to_string(),
                    status: flag,
                });

                match flag {
                    StatusFlags::BLIND => log.add("You can see again.", *turn),
                    StatusFlags::CONFUSED => log.add("You feel less confused.", *turn),
                    StatusFlags::STUNNED => log.add("You feel less staggered.", *turn),
                    StatusFlags::HALLUCINATING => log.add("Everything looks normal now.", *turn),
                    StatusFlags::LEVITATING => log.add("You float gently to the ground.", *turn),
                    StatusFlags::SLOW => log.add("You feel yourself speeding up.", *turn),
                    StatusFlags::FAST => log.add("You feel yourself slowing down.", *turn),
                    StatusFlags::SLEEPING => log.add("You wake up.", *turn),
                    StatusFlags::PARALYZED => log.add("You can move again.", *turn),
                    StatusFlags::PHASING => log.add("You feel solid again.", *turn),
                    StatusFlags::POISONED | StatusFlags::SICK => {
                        log.add("You feel much better now.", *turn)
                    }
                    StatusFlags::STONING => log.add("You feel limber again.", *turn),
                    StatusFlags::SLIMED => log.add("The slime disappears.", *turn),
                    StatusFlags::STRANGLED | StatusFlags::CHOKING => {
                        log.add("You can breathe again.", *turn)
                    }
                    _ => {}
                }
            }
        }
    }
}

// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::entity::status::{StatusBundle, StatusFlags};
use crate::core::entity::{Health, PlayerTag};
use crate::core::events::{EventQueue, GameEvent};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;

pub fn status_tick_system(ctx: &mut crate::core::context::GameContext) {
    let mut query = <(
        Entity,
        &mut StatusBundle,
        &mut Health,
        Option<&mut crate::core::entity::player::Player>,
        Option<&PlayerTag>,
    )>::query();

    for (_ent, status, health, player, is_player) in query.iter_mut(ctx.world) {
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
                    ctx.log.add("You feel your abilities returning.", ctx.turn);
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
                        ctx.log.add("You are beginning to feel hungry.", ctx.turn)
                    }
                    crate::core::entity::player::HungerState::Weak => {
                        ctx.log.add("You feel weak.", ctx.turn)
                    }
                    crate::core::entity::player::HungerState::Fainting => {
                        ctx.log.add("You feel faint from lack of food.", ctx.turn)
                    }
                    crate::core::entity::player::HungerState::Satiated => {
                        ctx.log.add("You feel stuffed.", ctx.turn)
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
                    ctx.log
                        .add_colored("You die from starvation...", [255, 0, 0], ctx.turn);
                    health.current = 0;
                } else if ctx.rng.rn2(20) == 0 && !status.flags().contains(StatusFlags::SLEEPING) {
                    //
                    ctx.log.add("You faint from lack of food.", ctx.turn);
                    status.add(StatusFlags::SLEEPING, 10 + ctx.rng.rn2(10) as u32);

                    // [v2.0.0
                    ctx.event_queue.push(GameEvent::StatusApplied {
                        target: "player".to_string(),
                        status: StatusFlags::SLEEPING,
                        turns: 10,
                    });
                }
            }

            // 5. Periodic Status Damage
            let flags = status.flags();
            if flags.contains(StatusFlags::POISONED) && !flags.contains(StatusFlags::POISON_RES) {
                if ctx.turn % 15 == 0 {
                    health.current -= 1;
                    ctx.log
                        .add("You feel the poison coursing through your veins.", ctx.turn);
                    if ctx.rng.rn2(10) == 0 && p.str.base > 3 {
                        p.str.base -= 1;
                        ctx.log.add("You feel much weaker!", ctx.turn);
                    }
                }
            }
            if flags.contains(StatusFlags::SICK) {
                if ctx.turn % 10 == 0 {
                    health.current -= 2;
                    ctx.log.add("You feel very ill.", ctx.turn);
                }
            }
        }

        if is_player.is_some() {
            for flag in expired {
                // [v2.0.0
                ctx.event_queue.push(GameEvent::StatusExpired {
                    target: "player".to_string(),
                    status: flag,
                });

                match flag {
                    StatusFlags::BLIND => ctx.log.add("You can see again.", ctx.turn),
                    StatusFlags::CONFUSED => ctx.log.add("You feel less confused.", ctx.turn),
                    StatusFlags::STUNNED => ctx.log.add("You feel less staggered.", ctx.turn),
                    StatusFlags::HALLUCINATING => {
                        ctx.log.add("Everything looks normal now.", ctx.turn)
                    }
                    StatusFlags::LEVITATING => {
                        ctx.log.add("You float gently to the ground.", ctx.turn)
                    }
                    StatusFlags::SLOW => ctx.log.add("You feel yourself speeding up.", ctx.turn),
                    StatusFlags::FAST => ctx.log.add("You feel yourself slowing down.", ctx.turn),
                    StatusFlags::SLEEPING => ctx.log.add("You wake up.", ctx.turn),
                    StatusFlags::PARALYZED => ctx.log.add("You can move again.", ctx.turn),
                    StatusFlags::PHASING => ctx.log.add("You feel solid again.", ctx.turn),
                    StatusFlags::POISONED | StatusFlags::SICK => {
                        ctx.log.add("You feel much better now.", ctx.turn)
                    }
                    StatusFlags::STONING => ctx.log.add("You feel limber again.", ctx.turn),
                    StatusFlags::SLIMED => ctx.log.add("The slime disappears.", ctx.turn),
                    StatusFlags::STRANGLED | StatusFlags::CHOKING => {
                        ctx.log.add("You can breathe again.", ctx.turn)
                    }
                    _ => {}
                }
            }
        }
    }
}

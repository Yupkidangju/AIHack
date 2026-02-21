// [v2.0.0 Phase R1] main.rs에서 분리된 게임 루프 로직
// 입력 처리  GameState 분기  턴 실행  시스템 스케줄  사망 체크
//
//

use crate::core::entity::{Health, Inventory, PlayerTag, Position};
use crate::core::game_state::{Direction, DirectionAction, GameState};
use crate::ui::input::Command;
use eframe::egui;
use legion::*;

impl super::NetHackApp {
    /// 게임 턴 처리  입력 분기 + 시스템 실행 + 후처리
    /// main.rs의 update()에서 AppState::Playing 일 때 호출됨
    pub(crate) fn process_game_turn(&mut self, ctx: &eframe::egui::Context) {
        // 입력 처리 (Bug #3: poll_input 사용)

        let (polled_cmd, polled_spell) = self.poll_input(ctx);
        self.input.last_cmd = polled_cmd;
        self.input.spell_key_input = polled_spell;
        //

        //
        self.game.resources.insert(self.input.game_state.clone());

        let mut _action_executed = false;
        match &self.input.game_state {
            GameState::More => {
                //
                if self.input.last_cmd != Command::Unknown {
                    self.input.game_state.reset();
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        log.needs_more = false;
                        log.turn_message_count = 0;
                        // "--More--" 메시지 제거 (가장 최근 것)
                        if let Some(last) = log.messages.last() {
                            if last.text == "--More--" {
                                log.messages.pop();
                            }
                        }
                    }
                    self.input.last_cmd = Command::Unknown; // 입력 소비
                }
            }
            GameState::WaitingForDirection { action } => {
                let action_copy = *action;
                if self.handle_direction_input(action_copy) {
                    _action_executed = true;
                }
            }
            GameState::WaitingForSpell
            | GameState::IdentifySelect { .. }
            | GameState::ConfirmRefill { .. } => {
                if self.handle_target_input(ctx) {
                    _action_executed = true;
                }
            }
            GameState::Normal => {
                if self.handle_normal_state(ctx) {
                    _action_executed = true;
                }
            }
            GameState::Inventory => {
                self.handle_inventory_action();
            }
            GameState::SelectOffhand
            | GameState::SelectEngraveTool
            | GameState::EngravingText { .. }
            | GameState::Help
            | GameState::Looting { .. }
            | GameState::Enhance => {
                self.handle_special_states();
            }
            _ => (/* Normal 등 */),
        }

        // 명령이 있을 경우 시스템 실행
        if self.input.last_cmd != crate::ui::input::Command::Unknown {
            //
            self.game.resources.insert(self.input.last_cmd);
            self.game.resources.insert(self.game.grid.clone());

            //
            if _action_executed {
                if let Some(mut turn) = self.game.resources.get_mut::<u64>() {
                    *turn += 1;
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        log.current_turn = *turn;
                    }
                }
            }

            // [v1.9.0
            let drained = self.drain_action_queue();
            _action_executed = _action_executed || drained;
            self.execute_turn_systems();

            self.post_turn_processing();
            self.handle_level_change();
        }
    }

    pub(crate) fn handle_normal_state(&mut self, ctx: &eframe::egui::Context) -> bool {
        let mut _action_executed = false;
        match self.input.last_cmd {
            Command::MoveN
            | Command::MoveS
            | Command::MoveE
            | Command::MoveW
            | Command::MoveNE
            | Command::MoveNW
            | Command::MoveSE
            | Command::MoveSW
            | Command::Wait => {
                _action_executed = true;
            }
            Command::Pickup => {
                _action_executed = true;
            }
            Command::Open => {
                self.input
                    .game_state
                    .request_direction(DirectionAction::Open);
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    log.add("In what direction?", turn);
                }
            }
            Command::Close => {
                self.input
                    .game_state
                    .request_direction(DirectionAction::Close);
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    log.add("In what direction?", turn);
                }
            }
            Command::Kick => {
                self.input
                    .game_state
                    .request_direction(DirectionAction::Kick);
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    log.add("In what direction?", turn);
                }
            }
            Command::Talk => {
                self.input
                    .game_state
                    .request_direction(DirectionAction::Talk);
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    log.add("In what direction?", turn);
                }
            }

            Command::CharacterSheet => {
                self.ui.show_character = !self.ui.show_character;
            }
            Command::Unknown => {
                // poll_input에서 직접 설정된 특수 처리 (Character toggle 등)
                //
            }
            Command::Throw => {
                //
                let mut query = <&Inventory>::query().filter(component::<PlayerTag>());
                let mut item_to_throw = None;
                for inv in query.iter(&self.game.world) {
                    if !inv.items.is_empty() {
                        item_to_throw = Some(inv.items[0]);
                    }
                }

                if let Some(item) = item_to_throw {
                    self.input
                        .game_state
                        .request_direction(DirectionAction::Throw { item });
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add("In what direction?", turn);
                    }
                } else {
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add("You have nothing to throw.", turn);
                    }
                }
            }
            Command::Cast => {
                self.input.game_state = GameState::WaitingForSpell;
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    log.add("Cast which spell?", turn);
                }
            }
            Command::Loot => {
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    log.add("In what direction?", turn);
                }
                self.input
                    .game_state
                    .request_direction(DirectionAction::Loot);
            }
            Command::Search => {
                let (mut subworld, _) =
                    self.game
                        .world
                        .split_for_query(&<(&Position, &mut crate::core::entity::Trap)>::query());
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    if let Some(mut rng) = self
                        .game
                        .resources
                        .get_mut::<crate::util::rng::NetHackRng>()
                    {
                        if let Some(rumors) = self
                            .game
                            .resources
                            .get::<crate::core::systems::talk::Rumors>()
                        {
                            crate::core::systems::search::try_search(
                                &mut subworld,
                                &mut self.game.grid,
                                &mut log,
                                turn,
                                &mut rng,
                                &rumors,
                            );
                        }
                        _action_executed = true;
                    }
                }
            }
            Command::Offer => {
                let mut p_pos = None;
                let mut query = <&Position>::query().filter(component::<PlayerTag>());
                for pos in query.iter(&self.game.world) {
                    p_pos = Some((pos.x, pos.y));
                }

                if let Some((px, py)) = p_pos {
                    if let Some(tile) = self.game.grid.get_tile(px as usize, py as usize) {
                        if tile.typ == crate::core::dungeon::tile::TileType::Altar {
                            self.input.game_state = GameState::OfferSelection;
                            if let Some(mut log) =
                                self.game.resources.get_mut::<crate::ui::log::GameLog>()
                            {
                                let turn =
                                    self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                                log.add("Which item do you want to offer?", turn);
                            }
                        } else {
                            if let Some(mut log) =
                                self.game.resources.get_mut::<crate::ui::log::GameLog>()
                            {
                                let turn =
                                    self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                                log.add("There is no altar here.", turn);
                            }
                        }
                    }
                }
            }
            Command::Pray => {
                let (mut subworld, _) = self.game.world.split_for_query(&<(
                    &mut crate::core::entity::player::Player,
                    &mut crate::core::entity::Health,
                    &Position,
                )>::query(
                ));
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    if let Some(mut rng) = self
                        .game
                        .resources
                        .get_mut::<crate::util::rng::NetHackRng>()
                    {
                        if let Some(provider) =
                            self.game
                                .resources
                                .get::<crate::core::systems::social::DefaultInteractionProvider>()
                        {
                            crate::core::systems::pray::try_pray(
                                &mut subworld,
                                &self.game.grid,
                                &mut log,
                                turn,
                                &mut rng,
                                &*provider,
                            );
                            _action_executed = true;
                        }
                    }
                }
            }
            Command::Sit => {
                let (mut subworld, _) = self.game.world.split_for_query(&<(
                    &mut crate::core::entity::player::Player,
                    &mut crate::core::entity::Health,
                    &Position,
                )>::query(
                ));
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    if let Some(mut rng) = self
                        .game
                        .resources
                        .get_mut::<crate::util::rng::NetHackRng>()
                    {
                        crate::core::systems::sit::try_sit(
                            &mut subworld,
                            &self.game.grid,
                            &mut log,
                            turn,
                            &mut rng,
                        );
                        _action_executed = true;
                    }
                }
            }
            Command::Zap => {
                let mut query = <&Inventory>::query().filter(component::<PlayerTag>());
                let mut wand_to_zap = None;
                for inv in query.iter(&self.game.world) {
                    for &item_ent in &inv.items {
                        if let Ok(entry) = self.game.world.entry_ref(item_ent) {
                            if let Ok(item) = entry.get_component::<crate::core::entity::Item>() {
                                if let Some(template) =
                                    self.game.assets.items.get_by_kind(item.kind)
                                {
                                    if template.class
                                        == crate::core::entity::object::ItemClass::Wand
                                    {
                                        wand_to_zap = Some(item_ent);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if wand_to_zap.is_some() {
                        break;
                    }
                }

                if let Some(item) = wand_to_zap {
                    self.input
                        .game_state
                        .request_direction(DirectionAction::Zap { item });
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add("In what direction?", turn);
                    }
                } else {
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add_colored(
                            "Search your pack... but you have no wand to zap.",
                            [255, 0, 0],
                            turn,
                        );
                    }
                }
            }
            Command::Apply => {
                //
                //
                self.input.game_state = GameState::Inventory;
            }
            Command::Help => {
                self.input.game_state = GameState::Help;
            }
            Command::Inventory => {
                self.input.game_state = GameState::Inventory;
            }
            Command::Enhance => {
                self.input.game_state = GameState::Enhance;
            }
            Command::TwoWeapon => {
                let mut query = <&mut crate::core::entity::player::Player>::query()
                    .filter(component::<PlayerTag>());
                let mut current_state = false;
                for player in query.iter_mut(&mut self.game.world) {
                    current_state = player.two_weapon;
                    if current_state {
                        player.two_weapon = false;
                    }
                }

                if current_state {
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add("You switch to your primary weapon only.", turn);
                    }
                } else {
                    self.input.game_state = GameState::SelectOffhand;
                }
            }
            Command::Swap => {
                let mut query =
                    <&mut crate::core::entity::Equipment>::query().filter(component::<PlayerTag>());
                for eq in query.iter_mut(&mut self.game.world) {
                    use crate::core::entity::EquipmentSlot;
                    let melee = eq.slots.remove(&EquipmentSlot::Melee);
                    let swap = eq.slots.remove(&EquipmentSlot::Swap);

                    if let Some(m) = melee {
                        eq.slots.insert(EquipmentSlot::Swap, m);
                    }
                    if let Some(s) = swap {
                        eq.slots.insert(EquipmentSlot::Melee, s);
                    }

                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add("You swap your weapons.", turn);
                    }
                    _action_executed = true;
                }
            }
            Command::Engrave => {
                self.input.game_state = GameState::SelectEngraveTool;
            }
            Command::Name => {
                self.input.game_state = GameState::Naming {
                    entity: None,
                    is_call: false,
                };
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    log.add("What do you want to name?", turn);
                }
            }
            Command::Invoke => {
                self.input.game_state = GameState::SelectInvoke;
                if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    log.add("Invoke which object?", turn);
                }
            }
            Command::Fire => {
                let mut query =
                    <&crate::core::entity::Equipment>::query().filter(component::<PlayerTag>());
                let mut item_to_fire = None;
                for eq in query.iter(&self.game.world) {
                    item_to_fire = eq
                        .slots
                        .get(&crate::core::entity::EquipmentSlot::Quiver)
                        .cloned();
                }

                if let Some(item) = item_to_fire {
                    self.input
                        .game_state
                        .request_direction(DirectionAction::Throw { item });
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add("In what direction?", turn);
                    }
                } else {
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add("You have nothing quivered.", turn);
                    }
                }
            }
            Command::Quiver => {
                self.input.game_state = GameState::SelectQuiver;
            }
            Command::LogHistory => {
                self.ui.show_log_history = !self.ui.show_log_history;
            }
            Command::Save => {
                if let Err(e) = crate::core::save::SaveManager::save(
                    "save/player.sav",
                    &self.game.world,
                    &self.game.resources,
                    &self.game.dungeon,
                ) {
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add(format!("Save failed: {}", e), turn);
                    }
                } else {
                    // 저장 성공 시 종료 (원본 NetHack 동작)
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            Command::Quit => {
                self.input.game_state = GameState::GameOver {
                    message: "You quit the game.".to_string(),
                };
            }
            _ => {
                // 기타 명령 처리 (상태 변화 없음)
            }
        }
        _action_executed
    }

    pub(crate) fn handle_direction_input(&mut self, action: DirectionAction) -> bool {
        let mut _action_executed = false;
        if self.input.last_cmd == Command::Cancel {
            // ESC: 취소
            self.input.game_state.reset();
            if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
                let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                log.add("Never mind.", turn);
            }
        } else if let Some(dir) = Direction::from_command(&self.input.last_cmd) {
            // 방향 입력 받음 - 액션 실행
            let action_copy = action;
            self.input.game_state.reset();

            let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);

            match action_copy {
                DirectionAction::Throw { item } => {
                    self.game
                        .resources
                        .insert(Some(crate::core::systems::throw::ThrowAction { item, dir }));
                    self.input.last_cmd = Command::Unknown; // 입력 소비 (이동 방지)
                    _action_executed = true;
                }
                DirectionAction::Cast { spell_key } => {
                    self.game
                        .resources
                        .insert(Some(crate::core::systems::spell::CastAction {
                            spell_key,
                            direction: Some(dir),
                        }));
                    self.input.last_cmd = Command::Unknown;
                    _action_executed = true;
                }
                DirectionAction::Zap { item } => {
                    self.game
                        .resources
                        .insert(Some(crate::core::systems::zap::ZapAction {
                            item_ent: Some(item),
                            spell_name: None,
                            direction: dir,
                        }));
                    self.input.last_cmd = Command::Unknown;
                    _action_executed = true;
                }
                DirectionAction::Apply { item } => {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    let mut cb = legion::systems::CommandBuffer::new(&self.game.world);

                    {
                        if let Some(mut log) =
                            self.game.resources.get_mut::<crate::ui::log::GameLog>()
                        {
                            crate::core::systems::apply::execute_apply_action(
                                item,
                                dir,
                                &mut self.game.world,
                                &self.game.assets,
                                &mut self.game.grid,
                                &mut log,
                                turn,
                                &mut cb,
                            );
                        }
                    }

                    cb.flush(&mut self.game.world, &mut self.game.resources);
                    self.game.resources.insert(self.game.grid.clone());
                    self.input.last_cmd = Command::Unknown;
                    _action_executed = true;
                }
                DirectionAction::Loot => {
                    let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                    let mut cb = legion::systems::CommandBuffer::new(&self.game.world);

                    {
                        if let Some(mut log) =
                            self.game.resources.get_mut::<crate::ui::log::GameLog>()
                        {
                            crate::core::systems::loot::try_loot(
                                dir,
                                &mut self.game.world,
                                &self.game.assets,
                                &mut log,
                                turn,
                                &mut cb,
                                &mut self.input.game_state,
                            );
                        }
                    }

                    cb.flush(&mut self.game.world, &mut self.game.resources);
                    self.game.resources.insert(self.game.grid.clone());
                    self.input.last_cmd = Command::Unknown;
                    _action_executed = true;
                }
                DirectionAction::Talk => {
                    let (mut subworld, _) = self.game.world.split_for_query(&<(
                        Entity,
                        &Position,
                        Option<&crate::core::entity::Dialogue>,
                        Option<&crate::core::entity::Monster>,
                    )>::query(
                    ));
                    // Resource borrow workaround: Extract using remove, perform action, then insert back.
                    if let Some(mut log) = self.game.resources.remove::<crate::ui::log::GameLog>() {
                        if let Some(mut rng) =
                            self.game.resources.remove::<crate::util::rng::NetHackRng>()
                        {
                            if let Some(rumors) = self
                                .game
                                .resources
                                .get::<crate::core::systems::talk::Rumors>()
                            {
                                if let Some(provider) = self
                                    .game
                                    .resources
                                    .get::<crate::core::systems::social::DefaultInteractionProvider>()
                                {
                                    let turn = log.current_turn;
                                    crate::core::systems::talk::try_talk(
                                        &mut subworld,
                                        dir,
                                        &mut log,
                                        turn,
                                        &mut rng,
                                        &rumors,
                                        &*provider,
                                    );
                                }
                            }
                            self.game.resources.insert(rng);
                        }
                        self.game.resources.insert(log);
                    }
                    self.input.last_cmd = Command::Unknown;
                    _action_executed = true;
                }
                _ => {
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        if let Some(mut rng) = self
                            .game
                            .resources
                            .get_mut::<crate::util::rng::NetHackRng>()
                        {
                            if let Some(provider) = self
                                .game
                                .resources
                                .get::<crate::core::systems::social::DefaultInteractionProvider>()
                            {
                                crate::core::systems::interaction::execute_direction_action(
                                    action_copy,
                                    dir,
                                    &mut self.game.world,
                                    &mut self.game.grid,
                                    &mut log,
                                    turn,
                                    &mut rng,
                                    &*provider,
                                );
                            }
                        }
                    }
                    _action_executed = true;
                }
            }
        }
        // 다른 입력은 무시
        _action_executed
    }

    pub(crate) fn handle_target_input(&mut self, ctx: &eframe::egui::Context) -> bool {
        let mut _action_executed = false;
        match &self.input.game_state {
            GameState::WaitingForSpell => {
                // 마법 주문 단축키 입력 대기 (a-z)
                if let Some(c) = self.input.spell_key_input {
                    // 주문 정보 확인 (방향 필요 여부 체크)
                    let mut needs_dir = false;
                    let mut query = <&crate::core::entity::SpellKnowledge>::query()
                        .filter(component::<crate::core::entity::PlayerTag>());
                    for knowledge in query.iter(&self.game.world) {
                        if let Some(spell) = knowledge.spells.get(&c) {
                            // 임시: Force Bolt는 방향 필요
                            let name = spell.name.to_lowercase();
                            if name.contains("force bolt") {
                                needs_dir = true;
                            }
                        }
                    }

                    if needs_dir {
                        self.input
                            .game_state
                            .request_direction(DirectionAction::Cast { spell_key: c });
                        if let Some(mut log) =
                            self.game.resources.get_mut::<crate::ui::log::GameLog>()
                        {
                            let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                            log.add("In what direction?", turn);
                        }
                    } else {
                        self.game
                            .resources
                            .insert(Some(crate::core::systems::spell::CastAction {
                                spell_key: c,
                                direction: None,
                            }));
                        self.input.game_state.reset();
                        _action_executed = true;
                    }
                }
            }
            GameState::IdentifySelect {
                scroll: scroll_ref,
                count: count_ref,
            } => {
                let scroll = *scroll_ref;
                let mut count = *count_ref;
                if let Some(c) = self.input.spell_key_input {
                    let mut item_to_ident = None;
                    let mut player_inv = None;

                    let mut inv_query = <&mut Inventory>::query().filter(component::<PlayerTag>());
                    if let Some(inv) = inv_query.iter_mut(&mut self.game.world).next() {
                        if let Some(ent) = inv.letter_map.get(&c) {
                            item_to_ident = Some(*ent);
                            player_inv = Some(inv);
                        }
                    }

                    if let (Some(target), Some(_inv)) = (item_to_ident, player_inv) {
                        if let Ok(mut entry) = self.game.world.entry_mut(target) {
                            if let Ok(item) = entry.get_component_mut::<crate::core::entity::Item>()
                            {
                                if !item.known || !item.bknown {
                                    item.known = true;
                                    item.bknown = true;
                                    if let Some(mut log) =
                                        self.game.resources.get_mut::<crate::ui::log::GameLog>()
                                    {
                                        let turn = self
                                            .game
                                            .resources
                                            .get::<u64>()
                                            .map(|t| *t)
                                            .unwrap_or(0);
                                        log.add("You identify an item.", turn);
                                    }
                                    if count != 100 {
                                        count -= 1;
                                    }
                                } else if let Some(mut log) =
                                    self.game.resources.get_mut::<crate::ui::log::GameLog>()
                                {
                                    let turn =
                                        self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                                    log.add("You already know that item.", turn);
                                }
                            }
                        }
                    }

                    if count == 0 {
                        self.input.game_state.reset();
                        // 스크롤 소모
                        let mut p_inv_query =
                            <&mut Inventory>::query().filter(component::<PlayerTag>());
                        if let Some(p_inv) = p_inv_query.iter_mut(&mut self.game.world).next() {
                            if let Some(pos) = p_inv.items.iter().position(|&e| e == scroll) {
                                p_inv.items.remove(pos);
                            }
                            p_inv.letter_map.retain(|_, &mut v| v != scroll);
                        }
                        self.game.world.remove(scroll);
                        if let Some(mut log) =
                            self.game.resources.get_mut::<crate::ui::log::GameLog>()
                        {
                            let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                            log.add("The scroll crumbles to dust.", turn);
                        }
                    } else {
                        self.input.game_state = GameState::IdentifySelect { scroll, count };
                    }
                } else if self.input.last_cmd == Command::Cancel {
                    self.input.game_state.reset();
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add("Never mind.", turn);
                    }
                }
            }
            GameState::ConfirmRefill { lamp, oil } => {
                let mut confirmed = false;
                let mut cancelled = false;

                // 'y' 또는 'n' 입력 체크
                if ctx.input(|i| i.key_pressed(egui::Key::Y)) {
                    confirmed = true;
                } else if ctx.input(|i| i.key_pressed(egui::Key::N))
                    || self.input.last_cmd == Command::Cancel
                {
                    cancelled = true;
                }

                if confirmed {
                    // 연료 보충 로직
                    use crate::core::entity::{Inventory, Item, PlayerTag};
                    let mut inv_q = <&mut Inventory>::query().filter(component::<PlayerTag>());
                    let mut oil_used = false;

                    if let Some(inv) = inv_q.iter_mut(&mut self.game.world).next() {
                        if let Some(pos) = inv.items.iter().position(|&e| e == *oil) {
                            inv.items.remove(pos);
                            oil_used = true;
                        }
                    }

                    if oil_used {
                        if let Ok(mut entry) = self.game.world.entry_mut(*lamp) {
                            if let Ok(item) = entry.get_component_mut::<Item>() {
                                item.age += 1000; // 보충량 (potion of oil)
                                if let Some(mut log) =
                                    self.game.resources.get_mut::<crate::ui::log::GameLog>()
                                {
                                    let turn =
                                        self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                                    log.add(format!("You refill the {}.", item.kind), turn);
                                }
                            }
                        }
                        // 기름 엔티티 삭제
                        self.game.world.remove(*oil);
                    }

                    _action_executed = true;
                    self.input.game_state.reset();
                } else if cancelled {
                    self.input.game_state.reset();
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add("Never mind.", turn);
                    }
                }
            }
            _ => {}
        }
        _action_executed
    }

    pub(crate) fn handle_inventory_action(&mut self) {
        if self.input.last_cmd == Command::Cancel || self.input.last_cmd == Command::Inventory {
            self.input.game_state.reset();
            self.input.last_cmd = Command::Unknown;
        }
    }

    pub(crate) fn handle_special_states(&mut self) {
        match &self.input.game_state {
            GameState::SelectOffhand
            | GameState::SelectEngraveTool
            | GameState::EngravingText { .. } => {
                if self.input.last_cmd == Command::Cancel
                    || self.input.last_cmd == Command::Inventory
                {
                    self.input.game_state.reset();
                    self.input.last_cmd = Command::Unknown;
                }
            }
            GameState::Help => {
                if self.input.last_cmd == Command::Cancel || self.input.last_cmd == Command::Help {
                    self.input.game_state.reset();
                }
            }
            GameState::Looting { .. } => {
                if self.input.last_cmd == Command::Cancel {
                    self.input.game_state.reset();
                }
            }
            GameState::Enhance => {
                if self.input.last_cmd == Command::Cancel || self.input.last_cmd == Command::Enhance
                {
                    self.input.game_state.reset();
                }
            }
            _ => {}
        }
    }

    fn execute_turn_systems(&mut self) {
        let mut schedule = Schedule::builder()
            .add_system(crate::core::systems::movement::movement_system())
            .flush()
            .add_system(crate::core::systems::ai::pet_hunger_system())
            .flush()
            .add_system(crate::core::systems::ai::monster_ai_system())
            .flush()
            .add_system(crate::core::systems::luck::luck_maintenance_system())
            .flush()
            .add_system(crate::core::systems::engrave::engrave_tick_system())
            .flush()
            .add_system(crate::core::systems::trap::trap_trigger_system())
            .flush()
            .add_system(crate::core::systems::death::death_system())
            .flush()
            .add_system(crate::core::systems::vision_system::vision_update_system())
            .flush()
            .add_system(crate::core::systems::vision_system::magic_map_effect_system())
            .flush()
            .add_system(crate::core::systems::inventory::autopickup_tick_system())
            .flush()
            .add_system(crate::core::systems::inventory::inventory_action_system())
            .flush()
            .add_system(crate::core::systems::item_use::item_input_system())
            .flush()
            .add_system(crate::core::systems::item_use::item_use_system())
            .flush()
            .add_system(crate::core::systems::equipment::equipment_system())
            .flush()
            .add_system(crate::core::systems::equipment::update_player_stats_system())
            .flush()
            .add_system(crate::core::systems::throw::throw_system())
            .flush()
            .add_system(crate::core::systems::zap::zap_system())
            .flush()
            .add_system(crate::core::systems::teleport::teleport_system())
            .flush()
            .add_system(crate::core::systems::spell::spell_cast_system())
            .flush()
            .add_system(crate::core::systems::stairs::stairs_system())
            .flush()
            .add_system(crate::core::systems::status::status_tick_system())
            .flush()
            .add_system(crate::core::systems::attrib::attrib_maintenance_system())
            .flush()
            .add_system(crate::core::systems::timeout::timeout_dialogue_system())
            .flush()
            .add_system(crate::core::systems::item_tick::item_tick_system())
            .flush()
            .add_system(crate::core::systems::regeneration::regeneration_system())
            .flush()
            .add_system(crate::core::systems::regeneration::monster_regeneration_system())
            .flush()
            .add_system(crate::core::systems::evolution::evolution_tick_system())
            .flush()
            .add_system(crate::core::systems::evolution::lycanthropy_tick_system())
            .flush()
            .add_system(crate::core::systems::shop::shopkeeper_update_system())
            .flush()
            .add_system(crate::core::systems::weight::update_encumbrance_system())
            .build();

        schedule.execute(&mut self.game.world, &mut self.game.resources);
    }

    fn handle_level_change(&mut self) {
        let mut level_change = None;
        if let Some(req) = self
            .game
            .resources
            .get::<Option<crate::core::dungeon::LevelChange>>()
        {
            level_change = *req;
        }

        if let Some(change) = level_change {
            use crate::core::dungeon::{LevelChange, LevelID};
            let next_level = match change {
                LevelChange::NextLevel => LevelID::new(
                    self.game.dungeon.current_level.branch,
                    self.game.dungeon.current_level.depth + 1,
                ),
                LevelChange::PrevLevel => LevelID::new(
                    self.game.dungeon.current_level.branch,
                    self.game.dungeon.current_level.depth - 1,
                ),
                LevelChange::Teleport { target, .. } => target,
            };

            if next_level.depth >= 1 {
                let log_msg;
                // 1. 현재 그리드 저장
                self.game
                    .dungeon
                    .set_level(self.game.dungeon.current_level, self.game.grid.clone());

                // 2. 새로운 층 로드 또는 생성
                let (new_grid, stairs_pos) = if let Some(existing) =
                    self.game.dungeon.get_level(next_level)
                {
                    log_msg = format!("Welcome back to level {}.", next_level.depth);
                    let pos = match change {
                        LevelChange::NextLevel | LevelChange::PrevLevel => {
                            let target_tile = if matches!(change, LevelChange::NextLevel) {
                                crate::core::dungeon::tile::TileType::StairsUp
                            } else {
                                crate::core::dungeon::tile::TileType::StairsDown
                            };
                            let mut p = (10, 10);
                            for x in 0..crate::core::dungeon::COLNO {
                                for y in 0..crate::core::dungeon::ROWNO {
                                    if existing.locations[x][y].typ == target_tile {
                                        p = (x as i32, y as i32);
                                        break;
                                    }
                                }
                            }
                            p
                        }
                        LevelChange::Teleport { landing, .. } => match landing {
                            crate::core::dungeon::LandingType::StairsUp => {
                                let mut p = (10, 10);
                                for x in 0..crate::core::dungeon::COLNO {
                                    for y in 0..crate::core::dungeon::ROWNO {
                                        if existing.locations[x][y].typ
                                            == crate::core::dungeon::tile::TileType::StairsUp
                                        {
                                            p = (x as i32, y as i32);
                                            break;
                                        }
                                    }
                                }
                                p
                            }
                            crate::core::dungeon::LandingType::StairsDown => {
                                let mut p = (10, 10);
                                for x in 0..crate::core::dungeon::COLNO {
                                    for y in 0..crate::core::dungeon::ROWNO {
                                        if existing.locations[x][y].typ
                                            == crate::core::dungeon::tile::TileType::StairsDown
                                        {
                                            p = (x as i32, y as i32);
                                            break;
                                        }
                                    }
                                }
                                p
                            }
                            crate::core::dungeon::LandingType::Coordinate(x, y) => (x, y),
                            crate::core::dungeon::LandingType::Random => {
                                let mut p = (10, 10);
                                if let Some(mut rng) = self
                                    .game
                                    .resources
                                    .get_mut::<crate::util::rng::NetHackRng>()
                                {
                                    for _ in 0..200 {
                                        let tx = rng.rn2(crate::core::dungeon::COLNO as i32);
                                        let ty = rng.rn2(crate::core::dungeon::ROWNO as i32);
                                        if !existing.locations[tx as usize][ty as usize]
                                            .typ
                                            .is_wall()
                                        {
                                            p = (tx, ty);
                                            break;
                                        }
                                    }
                                }
                                p
                            }
                            crate::core::dungeon::LandingType::Connection(source_level) => {
                                let mut p = (10, 10);
                                for (pos, target) in &existing.portals {
                                    if *target == source_level {
                                        p = *pos;
                                        break;
                                    }
                                }
                                p
                            }
                        },
                    };
                    (existing.clone(), pos)
                } else {
                    log_msg = format!("You enter level {}.", next_level.depth);
                    let mut out_gen_grid = crate::core::dungeon::Grid::new();
                    let mut out_pos = (10, 10);

                    if let Some(mut rng) = self
                        .game
                        .resources
                        .get_mut::<crate::util::rng::NetHackRng>()
                    {
                        let monster_templates: Vec<_> =
                            self.game.assets.monsters.templates.values().collect();
                        let ltype =
                            crate::core::dungeon::gen::LevelType::for_depth(next_level, &mut *rng);
                        let (gen_grid, up_pos, down_pos, _rooms) =
                            crate::core::dungeon::gen::MapGenerator::generate_improved(
                                &mut *rng,
                                next_level,
                                &mut self.game.world,
                                &self.game.assets.items,
                                &monster_templates,
                                ltype,
                            );
                        let pos = match change {
                            LevelChange::NextLevel => up_pos,
                            LevelChange::PrevLevel => down_pos,
                            LevelChange::Teleport { landing, .. } => match landing {
                                crate::core::dungeon::LandingType::StairsUp => up_pos,
                                crate::core::dungeon::LandingType::StairsDown => down_pos,
                                crate::core::dungeon::LandingType::Coordinate(x, y) => (x, y),
                                crate::core::dungeon::LandingType::Random => {
                                    let mut p = (10, 10);
                                    for _ in 0..200 {
                                        let tx = rng.rn2(crate::core::dungeon::COLNO as i32);
                                        let ty = rng.rn2(crate::core::dungeon::ROWNO as i32);
                                        if !gen_grid.locations[tx as usize][ty as usize]
                                            .typ
                                            .is_wall()
                                        {
                                            p = (tx, ty);
                                            break;
                                        }
                                    }
                                    p
                                }
                                crate::core::dungeon::LandingType::Connection(source_level) => {
                                    let mut p = (10, 10);
                                    for (pos, target) in &gen_grid.portals {
                                        if *target == source_level {
                                            p = *pos;
                                            break;
                                        }
                                    }
                                    p
                                }
                            },
                        };
                        out_gen_grid = gen_grid;
                        out_pos = pos;
                    }
                    (out_gen_grid, out_pos)
                };

                self.game.grid = new_grid;
                self.game.dungeon.current_level = next_level;

                //
                let mut query = <(Entity, &mut Position, &mut crate::core::entity::Level)>::query()
                    .filter(component::<PlayerTag>());
                for (_, pos, level) in query.iter_mut(&mut self.game.world) {
                    pos.x = stairs_pos.0;
                    pos.y = stairs_pos.1;
                    level.0 = next_level;

                    //
                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                        log.add(log_msg.clone(), turn);
                    }
                }

                // 4. 리소스 갱신
                self.game.resources.insert(self.game.grid.clone());
                self.game.resources.insert(self.game.dungeon.clone());
            }

            self.game
                .resources
                .insert(None::<crate::core::dungeon::LevelChange>);
        }
    }

    fn post_turn_processing(&mut self) {
        // Phase 48: 스폰 요청 처리 (Bag of Tricks 등)
        //
        // [v1.9.0
        if let Some(mut rng) = self
            .game
            .resources
            .get_mut::<crate::util::rng::NetHackRng>()
        {
            crate::core::systems::spawn_manager::run_spawn_requests(
                &mut self.game.world,
                &self.game.grid,
                &mut rng,
                &self.game.assets,
            );

            // [v2.0.0] 턴 기반 몬스터 리스폰 (원본: allmain.c — 1/50 확률)
            let current_turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
            crate::core::systems::spawn_manager::turn_respawn(
                &mut self.game.world,
                &self.game.grid,
                &mut rng,
                &self.game.assets,
                current_turn,
            );
        }

        // [v2.0.0] 몬스터 사망 후 시체/아이템 드롭 처리 (SubWorld 제약 우회)
        // death_system에서 DeathResults에 쌓인 요청을 &mut World로 실제 실행
        if let Some(death_res) = self
            .game
            .resources
            .get::<crate::core::systems::death::DeathResults>()
        {
            let corpse_reqs = death_res.corpse_requests.clone();
            let _drop_reqs = death_res.item_drop_requests.clone();
            drop(death_res); // 빌림 해제

            // 시체 엔티티 생성 (원본: mon.c:make_corpse)
            //
            let current_level = {
                let mut lvl_query = <&crate::core::entity::Level>::query()
                    .filter(legion::query::component::<crate::core::entity::PlayerTag>());
                lvl_query
                    .iter(&self.game.world)
                    .next()
                    .map(|l| l.0)
                    .unwrap_or(crate::core::dungeon::LevelID::new(
                        crate::core::dungeon::DungeonBranch::Main,
                        1,
                    ))
            };
            for req in &corpse_reqs {
                self.game.world.push((
                    crate::core::entity::ItemTag,
                    crate::core::entity::Position { x: req.x, y: req.y },
                    crate::core::entity::Renderable {
                        glyph: '%',
                        color: req.color,
                    },
                    crate::core::entity::Item {
                        kind: crate::generated::ItemKind::from_str(&format!(
                            "{} corpse",
                            req.monster_name
                        )),
                        weight: req.weight,
                        quantity: 1,
                        corpsenm: Some(req.monster_name.clone()),
                        age: req.corpse_age,
                        ..Default::default()
                    },
                    crate::core::entity::Level(current_level),
                ));
            }
        }

        // [v2.0.0 R5] 이벤트 소비: EventQueue → EventHistory 기록 후 clear
        //
        {
            let current_turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);

            // 1. EventQueue → EventHistory 기록
            if let Some(eq) = self.game.resources.get::<crate::core::events::EventQueue>() {
                if !eq.is_empty() {
                    //
                    let events: Vec<crate::core::events::GameEvent> = eq.iter().cloned().collect();
                    drop(eq); // EventQueue 빌림 해제

                    // EventHistory 및 GameLog에 기록
                    let mut log_msgs = Vec::new();

                    if let Some(mut history) = self
                        .game
                        .resources
                        .get_mut::<crate::core::events::EventHistory>()
                    {
                        for event in events.iter() {
                            history.record(current_turn, event.clone());
                            log_msgs.push(event.to_narrative());
                        }
                    }

                    if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>()
                    {
                        for msg in log_msgs {
                            log.add(msg, current_turn);
                        }
                    }
                }
            }

            // 2. EventQueue clear (다음 턴 준비)
            if let Some(mut eq) = self
                .game
                .resources
                .get_mut::<crate::core::events::EventQueue>()
            {
                eq.clear(current_turn + 1);
            }
        }

        // GameState 동기화 복구
        if let Some(st) = self.game.resources.get::<GameState>() {
            self.input.game_state = (*st).clone();
        }

        //
        if let Some(mut log) = self.game.resources.get_mut::<crate::ui::log::GameLog>() {
            if log.needs_more {
                self.input.game_state = GameState::More;
                let turn = self.game.resources.get::<u64>().map(|t| *t).unwrap_or(0);
                log.add_colored("--More--", [255, 255, 0], turn);
            }
        }

        // 동기화 복구 (More 상태 등 반영)
        self.game.resources.insert(self.input.game_state.clone());

        //
        let mut altar_update = None;
        if let Some(mut req) = self
            .game
            .resources
            .get_mut::<Option<crate::core::systems::pray::PendingAltarUpdate>>()
        {
            altar_update = req.take();
        }
        if let Some(update) = altar_update {
            if let Some(tile) = self
                .game
                .grid
                .get_tile_mut(update.pos.0 as usize, update.pos.1 as usize)
            {
                let mask = match update.new_align {
                    crate::core::entity::player::Alignment::Lawful => 1,
                    crate::core::entity::player::Alignment::Neutral => 2,
                    crate::core::entity::player::Alignment::Chaotic => 4,
                };
                tile.altarmask = mask;
                // 그리드 리소스 동기화
                self.game.resources.insert(self.game.grid.clone());
            }
        }

        // Level Change 요청 처리

        // Turn 종료 후 액션 초기화
        self.game
            .resources
            .insert(None::<crate::core::systems::item_use::ItemAction>);
        // 사망 체크
        let mut is_dead = false;
        {
            let mut q = <&Health>::query().filter(component::<PlayerTag>());
            if let Some(h) = q.iter(&self.game.world).next() {
                if h.current <= 0 {
                    is_dead = true;
                }
            }
        }
        if is_dead {
            self.input.game_state = GameState::GameOver {
                message: "You died...".to_string(),
            };
        }
    }
    pub(crate) fn drain_action_queue(&mut self) -> bool {
        let mut has_action = false;

        let mut queue_items = Vec::new();
        if let Some(mut queue) = self
            .game
            .resources
            .get_mut::<crate::core::action_queue::ActionQueue>()
        {
            while let Some(action) = queue.pop() {
                queue_items.push(action);
                has_action = true;
            }
        }

        for action in queue_items {
            match action {
                crate::core::action_queue::GameAction::Item(a) => {
                    self.game.resources.insert(Some(a))
                }
                crate::core::action_queue::GameAction::Throw(a) => {
                    self.game.resources.insert(Some(a))
                }
                crate::core::action_queue::GameAction::Cast(a) => {
                    self.game.resources.insert(Some(a))
                }
                crate::core::action_queue::GameAction::Zap(a) => {
                    self.game.resources.insert(Some(a))
                }
                crate::core::action_queue::GameAction::Teleport(a) => {
                    self.game.resources.insert(Some(a))
                }
                crate::core::action_queue::GameAction::LevelChange(a) => {
                    self.game.resources.insert(Some(a))
                }
            }
        }

        has_action
    }
}

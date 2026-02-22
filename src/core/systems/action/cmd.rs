// cmd.rs — NetHack 원본 cmd.c 1:1 디스패치 매핑
// [v2.21.0 R9-2] Command -> ActionQueue 매핑 및 매크로(반복키) 엔진
use crate::core::action_queue::{ActionQueue, GameAction};
use crate::core::entity::object::ItemManager;
use crate::core::game_state::GameState;
use crate::ui::input::Command;
use legion::World;

/// 확장 커맨드 처리 엔진 (do_ext_cmd 원본 매핑)
pub struct CommandDispatcher;

impl CommandDispatcher {
    /// 원본: cmd.c rhack(), do_ext_cmd()
    /// 입력 커맨드를 확인하고, 즉시 가능한 일은 Action에 넣거나
    /// 대상/아이템이 필요한 경우 GameState(Prompt)로 전환시킴.
    pub fn dispatch(
        cmd: Command,
        current_state: &mut GameState,
        action_queue: &mut ActionQueue,
        _world: &mut World,
    ) -> bool {
        match cmd {
            // ====================================
            // # Command 계층 처리 (do_ext_cmd)
            // ====================================
            Command::Pray => {
                // TODO: 기도(Pray) 상태 전환 혹은 즉시 액션
                *current_state = GameState::Normal;
                true
            }
            Command::Rub => {
                // TODO: 램프 등을 문지르기 위해 아이템 선별 모드 진입
                *current_state = GameState::Normal;
                true
            }
            Command::Dip => {
                // TODO: ItemSelection으로 먼저 포션(담그는 액체)을 선택하게 함
                *current_state = GameState::Normal;
                true
            }
            Command::Wipe => {
                *current_state = GameState::Normal;
                true
            }
            Command::Force => {
                // TODO: 방향을 묻는 디렉션 콜백으로 전환
                *current_state = GameState::Normal;
                true
            }

            // 기존 명령 체계 바이패스
            _ => false,
        }
    }

    /// 원본: cmd.c count_commands()
    /// "20s" 와 같이 횟수가 지정된 명령을 큐에 20번 전송함
    pub fn execute_macro(count: u32, action: GameAction, queue: &mut ActionQueue) {
        for _ in 0..count {
            queue.push(action.clone());
        }
    }
}

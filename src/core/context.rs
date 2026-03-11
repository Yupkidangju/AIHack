// [v3.0.0 Phase E1] GameContext — 모든 시스템 함수의 단일 진입점
// 결정 #41 (ENGINE-2): World + Resources + LLM Provider를 통합
// 기존 Legion의 #[system] 매크로 + SubWorld + Schedule을 대체하여
// AccessDenied 패닉을 구조적으로 영구 제거한다.
//
// 현재 Phase E1에서는 World+Resources 직접 접근 래퍼로 구현.
// Phase E2에서 각 시스템의 시그니처를 fn(ctx: &mut GameContext)로 전환.
// Phase E4에서 AIProvider 필드를 추가.
use crate::assets::AssetManager;
use crate::core::action_queue::ActionQueue;
use crate::core::dungeon::dungeon::Dungeon;
use crate::core::dungeon::{Grid, LevelChange};
use crate::core::events::EventQueue;
use crate::core::systems::vision::VisionSystem;
use crate::llm::LlmEngine; // [v3.0.0 E4] LLM 엔진
use crate::ui::input::Command;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::*;

/// [v3.0.0] 게임 전체 컨텍스트 — 모든 시스템 함수의 단일 진입점
///
/// Legion의 SubWorld + #[resource] 매크로를 대체한다.
/// &mut World에서는 entry_ref/entry_mut 호출 시 권한 검사가 없으므로
/// AccessDenied가 구조적으로 불가능하다.
///
/// ## 사용법
/// ```ignore
/// fn movement_system(ctx: &mut GameContext) {
///     // ctx.world — 전체 World 접근 (AccessDenied 불가능)
///     // ctx.log — 게임 로그
///     // ctx.rng — 난수 생성기
///     // ctx.grid — 현재 층 맵
/// }
/// ```
pub struct GameContext<'a> {
    // === ECS 데이터 ===
    /// Legion ECS World — 모든 엔티티의 컨테이너
    /// SubWorld와 달리 권한 제한 없이 모든 컴포넌트에 접근 가능
    pub world: &'a mut World,

    // === 게임 리소스 ===
    /// 현재 층의 던전 맵
    pub grid: &'a mut Grid,
    /// 게임 로그 (메시지 출력)
    pub log: &'a mut GameLog,
    /// NetHack 결정론적 난수 생성기
    pub rng: &'a mut NetHackRng,
    /// 현재 턴 번호
    pub turn: u64,
    /// 현재 플레이어 입력 커맨드
    pub cmd: Command,
    /// 에셋 매니저 (몬스터/아이템 템플릿)
    pub assets: &'a AssetManager,
    /// 이벤트 큐 (시스템 간 통신)
    pub event_queue: &'a mut EventQueue,
    /// 액션 큐 (장비/던지기/이동 등 사용자 액션)
    pub action_queue: &'a mut ActionQueue,
    /// 시야 시스템 (FOV/가시 영역 계산)
    pub vision: &'a mut VisionSystem,
    /// 레벨 변경 요청 (계단/텔레포트 등)
    pub level_req: &'a mut Option<LevelChange>,
    /// 던전 정보 (현재 레벨, 브랜치 등)
    pub dungeon: &'a Dungeon,
    /// 게임 상태 (플레이 중/게임 오버 등)
    pub game_state: &'a mut crate::core::game_state::GameState,
    // === AI 엔진 (Phase E4) ===
    /// [v3.0.0 E4] LLM 추론 엔진 (Option: 없으면 폴백 텍스트 사용)
    pub llm: Option<&'a LlmEngine>,
}

impl<'a> GameContext<'a> {
    /// GameContext 생성
    pub fn new(
        world: &'a mut World,
        grid: &'a mut Grid,
        log: &'a mut GameLog,
        rng: &'a mut NetHackRng,
        turn: u64,
        cmd: Command,
        assets: &'a AssetManager,
        event_queue: &'a mut EventQueue,
        action_queue: &'a mut ActionQueue,
        vision: &'a mut VisionSystem,
        level_req: &'a mut Option<LevelChange>,
        dungeon: &'a Dungeon,
        game_state: &'a mut crate::core::game_state::GameState,
        llm: Option<&'a LlmEngine>, // [v3.0.0 E4]
    ) -> Self {
        Self {
            world,
            grid,
            log,
            rng,
            turn,
            cmd,
            assets,
            event_queue,
            action_queue,
            vision,
            level_req,
            dungeon,
            game_state,
            llm,
        }
    }
}

// [v3.0.0 Phase E4 예약] AIProvider trait — LLM 교체 포인트
// 현재는 InteractionProvider(social/mod.rs)를 그대로 사용.
// Phase E4에서 이 파일에 AIProvider trait + DefaultAIProvider + Snapshot 구조체를 추가 예정.
// 결정 #41 ENGINE-3 참조.

/// `TurnRunner` — 시스템 스케줄링 및 순차 실행기
///
/// 기존 Legion의 Schedule 매크로 병렬 실행 모델을 대체하여
/// 단일 스레드 기반 턴 게임 로직에 최적화된 순차 실행을 보장한다.
pub struct TurnRunner {
    systems: Vec<Box<dyn Fn(&mut GameContext)>>,
}

impl TurnRunner {
    /// 새로운 TurnRunner 생성
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// 실행할 시스템 등록
    pub fn add_system<F>(mut self, system_fn: F) -> Self
    where
        F: Fn(&mut GameContext) + 'static,
    {
        self.systems.push(Box::new(system_fn));
        self
    }

    /// 등록된 모든 시스템을 순차적으로 실행
    pub fn execute(&self, ctx: &mut GameContext) {
        for system_fn in &self.systems {
            system_fn(ctx);
        }
    }
}

// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// GameContext가 World + 리소스 참조를 정상적으로 보유하는지 확인
    #[test]
    fn test_game_context_creation() {
        let mut world = World::default();
        let mut grid = Grid::new();
        let mut log = GameLog::new(100);
        let mut rng = NetHackRng::new(42);
        let mut event_queue = EventQueue::new();
        let mut resources = Resources::default();
        let assets = AssetManager::new();

        let mut action_queue = crate::core::action_queue::ActionQueue::new();
        let mut vision = crate::core::systems::vision::VisionSystem::new();
        let mut level_req: Option<LevelChange> = None;
        let dungeon = Dungeon::new();

        let mut game_state = crate::core::game_state::GameState::Normal;

        let mut ctx = GameContext::new(
            &mut world,
            &mut grid,
            &mut log,
            &mut rng,
            0,
            Command::Wait,
            &assets,
            &mut event_queue,
            &mut action_queue,
            &mut vision,
            &mut level_req,
            &dungeon,
            &mut game_state,
            None, // [v3.0.0 E4] LLM 없음 (테스트)
        );

        // GameContext가 정상 생성되면 turn/cmd 접근 가능
        assert_eq!(ctx.turn, 0);
        assert_eq!(ctx.cmd, Command::Wait);
    }
}

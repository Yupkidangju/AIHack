use aihack_runtime::GameClient;

fn accepts_adapter_client<C: GameClient>(_client: &mut C) {}

#[test]
fn game_client_is_an_adapter_facing_contract() {
    struct CompileOnly;

    impl GameClient for CompileOnly {
        fn observation(&self) -> aihack_ai_contract::Observation {
            unreachable!()
        }

        fn revision(&self) -> aihack_ai_contract::ClientRevision {
            unreachable!()
        }

        fn run_state(&self) -> aihack_core::run_state::RunState {
            unreachable!()
        }

        fn submit(
            &mut self,
            _intent: aihack_core::action::CommandIntent,
        ) -> aihack_core::turn::TurnOutcome {
            unreachable!()
        }
    }

    let mut client = CompileOnly;
    accepts_adapter_client(&mut client);
}

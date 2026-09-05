use std::path::{Path, PathBuf};

use aihack_ai_contract::{RunState, SnapshotHash};
use aihack_headless::{
    run_replay_to_turn, run_to_turn_with_trace, HeadlessPolicy, HeadlessRunError,
};
use aihack_runtime::{save::ArtifactStore, GameClient, GameSession};
use clap::Parser;
use serde::Serialize;

/// AIHack 결정론적 headless runner의 실행 인자다.
#[derive(Parser, Debug)]
struct Args {
    #[arg(long, value_parser = ["knight", "scout", "mage"], conflicts_with = "load")]
    role: Option<String>,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(
        long,
        default_value_t = 1000,
        value_parser = clap::value_parser!(u64).range(1..=1_000_000)
    )]
    turns: u64,
    #[arg(long, default_value = "survival-v1")]
    policy: String,
    #[arg(long)]
    save: Option<PathBuf>,
    #[arg(long)]
    load: Option<PathBuf>,
    #[arg(long)]
    replay_out: Option<PathBuf>,
    #[arg(long)]
    replay_in: Option<PathBuf>,
    #[arg(long)]
    report: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    if args.load.is_some() && args.seed.is_some() {
        eprintln!("cannot combine --load with --seed");
        std::process::exit(2);
    }
    let runtime_root = match runtime_root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let artifact_store = match ArtifactStore::open(&runtime_root) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let save_path = resolve_runtime_arg(&artifact_store, args.save.as_deref());
    let load_path = resolve_runtime_arg(&artifact_store, args.load.as_deref());
    let replay_in_path = resolve_runtime_arg(&artifact_store, args.replay_in.as_deref());
    let replay_out_path = resolve_runtime_arg(&artifact_store, args.replay_out.as_deref());
    let report_path = resolve_runtime_arg(&artifact_store, args.report.as_deref());
    if let (Some(input), Some(output)) = (&replay_in_path, &replay_out_path) {
        match artifact_store.paths_refer_to_same_artifact(input, output) {
            Ok(true) => {
                eprintln!("--replay-in and --replay-out must not resolve to the same path");
                std::process::exit(2);
            }
            Ok(false) => {}
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
    }
    let mut session = if let Some(path) = &load_path {
        match artifact_store.load_session(path) {
            Ok(session) => session,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
    } else {
        match if args.role.is_some() {
            GameSession::try_new(args.seed.unwrap_or(42))
        } else {
            GameSession::try_new_for_playing(args.seed.unwrap_or(42))
        } {
            Ok(session) => session,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
    };
    if let Some(role) = args.role.as_deref() {
        use aihack_ai_contract::{CommandIntent, Role};
        let role = match role {
            "knight" => Role::Knight,
            "scout" => Role::Scout,
            _ => Role::Mage,
        };
        if !session.submit(CommandIntent::Wait).accepted
            || !session
                .submit(CommandIntent::StartCampaign { role })
                .accepted
        {
            eprintln!("campaign initialization was rejected");
            std::process::exit(2);
        }
    }
    let initial_turn = session.revision().turn;
    let report_path = report_path.or_else(|| {
        resolve_runtime_arg(
            &artifact_store,
            Some(Path::new(&format!(
                "reports/long-run-{}.json",
                session.observation().seed
            ))),
        )
    });
    let policy = match args.policy.as_str() {
        "wait-v1" => HeadlessPolicy::wait_v1(),
        "survival-v1" => HeadlessPolicy::survival_v1(),
        "replay-file" => HeadlessPolicy::ReplayFile,
        other => {
            eprintln!("unknown policy: {other}");
            std::process::exit(2);
        }
    };
    let (report, trace) = if policy == HeadlessPolicy::ReplayFile {
        let Some(path) = &replay_in_path else {
            eprintln!("replay-file policy requires --replay-in");
            std::process::exit(2);
        };
        let trace = match artifact_store.read_replay_lines(path) {
            Ok(lines) => lines,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        };
        match run_replay_to_turn(&mut session, args.turns, &trace) {
            Ok(report) => (report, trace),
            Err(error) => {
                write_failure_report(
                    &artifact_store,
                    report_path.as_deref(),
                    &session,
                    policy,
                    args.turns,
                    initial_turn,
                    &error,
                );
                eprintln!("{error}");
                std::process::exit(runner_exit_code(&error));
            }
        }
    } else {
        if replay_in_path.is_some() {
            eprintln!("--replay-in requires --policy replay-file");
            std::process::exit(2);
        }
        match run_to_turn_with_trace(&mut session, args.turns, policy) {
            Ok(result) => result,
            Err(error) => {
                write_failure_report(
                    &artifact_store,
                    report_path.as_deref(),
                    &session,
                    policy,
                    args.turns,
                    initial_turn,
                    &error,
                );
                eprintln!("{error}");
                std::process::exit(runner_exit_code(&error));
            }
        }
    };
    if let Some(path) = &replay_out_path {
        if let Err(error) = artifact_store.append_replay_lines(path, &trace) {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }

    if let Some(path) = &save_path {
        if let Err(error) = artifact_store.save_session(&session, path) {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
    if let Some(path) = &report_path {
        if let Err(error) = write_report(&artifact_store, path, &report) {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }

    println!(
        "seed={} policy={} requested_turns={} accepted_turns={} submitted_commands={} final_state={:?} final_hash={}",
        report.seed,
        report.policy.id(),
        report.requested_turns,
        report.accepted_turns,
        report.submitted_commands,
        report.final_state,
        report.final_hash.0
    );
}

fn runtime_root() -> Result<PathBuf, String> {
    Ok(std::env::current_dir()
        .map_err(|error| error.to_string())?
        .join("runtime"))
}

fn runner_exit_code(error: &HeadlessRunError) -> i32 {
    match error {
        HeadlessRunError::TargetBeforeCurrent { .. } => 2,
        _ => 1,
    }
}

fn resolve_runtime_arg(store: &ArtifactStore, input: Option<&Path>) -> Option<PathBuf> {
    input.map(|path| match store.validate_path(path) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    })
}

fn write_report(store: &ArtifactStore, path: &Path, report: &impl Serialize) -> Result<(), String> {
    let body = serde_json::to_string_pretty(report).map_err(|error| error.to_string())?;
    store
        .write_atomic(path, body.as_bytes())
        .map_err(|error| error.to_string())
}

#[derive(Serialize)]
struct FailureReport<'a> {
    seed: u64,
    policy: &'static str,
    requested_turns: u64,
    accepted_turns: u64,
    submitted_commands: u64,
    final_state: RunState,
    final_hash: SnapshotHash,
    error: &'a HeadlessRunError,
}

fn write_failure_report(
    store: &ArtifactStore,
    path: Option<&Path>,
    session: &GameSession,
    policy: HeadlessPolicy,
    requested_turns: u64,
    initial_turn: u64,
    error: &HeadlessRunError,
) {
    let Some(path) = path else { return };
    let report = FailureReport {
        seed: session.observation().seed,
        policy: policy.id(),
        requested_turns,
        accepted_turns: session.revision().turn.saturating_sub(initial_turn),
        submitted_commands: error.submitted_commands(),
        final_state: session.run_state(),
        final_hash: session.revision().snapshot_hash,
        error,
    };
    if let Err(write_error) = write_report(store, path, &report) {
        eprintln!("{write_error}");
    }
}

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::Parser;

    #[test]
    fn implicit_policy_is_survival_and_turn_bounds_are_closed() {
        let defaults = Args::try_parse_from(["aihack-headless"]).unwrap();
        assert_eq!(defaults.policy, "survival-v1");
        assert_eq!(defaults.turns, 1_000);

        for turns in ["1", "1000000"] {
            assert!(Args::try_parse_from(["aihack-headless", "--turns", turns]).is_ok());
        }
        for turns in ["0", "1000001"] {
            assert!(Args::try_parse_from(["aihack-headless", "--turns", turns]).is_err());
        }
    }
}

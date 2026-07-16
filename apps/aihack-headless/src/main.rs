use std::{
    fs,
    path::{Path, PathBuf},
};

use aihack_ai_contract::{RunState, SnapshotHash};
use aihack_headless::{
    run_replay_to_turn, run_to_turn_with_trace, HeadlessPolicy, HeadlessRunError,
};
use aihack_runtime::{save, GameClient, GameSession};
use clap::Parser;
use serde::Serialize;

/// [v0.1.0] Phase 1 deterministic headless runner 인자다.
#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long, default_value_t = 1000)]
    turns: u64,
    #[arg(long, default_value = "wait-v1")]
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
    let save_path = resolve_runtime_arg(&runtime_root, args.save.as_deref());
    let load_path = resolve_runtime_arg(&runtime_root, args.load.as_deref());
    let replay_in_path = resolve_runtime_arg(&runtime_root, args.replay_in.as_deref());
    let replay_out_path = resolve_runtime_arg(&runtime_root, args.replay_out.as_deref());
    let report_path = resolve_runtime_arg(&runtime_root, args.report.as_deref());
    if replay_in_path.is_some() && replay_in_path == replay_out_path {
        eprintln!("--replay-in and --replay-out must not resolve to the same path");
        std::process::exit(2);
    }
    let mut session = if let Some(path) = &load_path {
        match save::load_session_from_path(path) {
            Ok(session) => session,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
    } else {
        match GameSession::try_new_for_playing(args.seed.unwrap_or(42)) {
            Ok(session) => session,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
    };
    let initial_turn = session.revision().turn;
    let report_path = report_path.or_else(|| {
        resolve_runtime_arg(
            &runtime_root,
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
        let trace = match save::read_replay_lines(path) {
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
                    report_path.as_deref(),
                    &session,
                    policy,
                    args.turns,
                    initial_turn,
                    &error,
                );
                eprintln!("{error}");
                std::process::exit(1);
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
                    report_path.as_deref(),
                    &session,
                    policy,
                    args.turns,
                    initial_turn,
                    &error,
                );
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    };
    if let Some(path) = &replay_out_path {
        for line in &trace {
            if let Err(error) = save::append_replay_line(path, line) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
    }

    if let Some(path) = &save_path {
        if let Err(error) = save::save_session_to_path(&session, path) {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
    if let Some(path) = &report_path {
        if let Err(error) = write_report(path, &report) {
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
    let root = std::env::current_dir()
        .map_err(|error| error.to_string())?
        .join("runtime");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(root)
}

fn resolve_runtime_arg(root: &Path, input: Option<&Path>) -> Option<PathBuf> {
    input.map(|path| match save::resolve_path_in_root(root, path) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    })
}

fn write_report(path: &Path, report: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let body = serde_json::to_string_pretty(report).map_err(|error| error.to_string())?;
    fs::write(path, body).map_err(|error| error.to_string())
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
    if let Err(write_error) = write_report(path, &report) {
        eprintln!("{write_error}");
    }
}

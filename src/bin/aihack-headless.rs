use std::path::PathBuf;

use aihack::core::{save, CommandIntent, GameSession, ReplayLineV1};
use clap::Parser;

/// [v0.1.0] Phase 1 deterministic headless runner 인자다.
#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long, default_value_t = 1000)]
    turns: u64,
    #[arg(long)]
    save: Option<PathBuf>,
    #[arg(long)]
    load: Option<PathBuf>,
    #[arg(long)]
    replay_out: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    if args.load.is_some() && args.seed.is_some() {
        eprintln!("cannot combine --load with --seed");
        std::process::exit(2);
    }
    let mut session = if let Some(path) = &args.load {
        match save::load_session_from_path(path) {
            Ok(session) => session,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
    } else {
        GameSession::new_for_playing(args.seed.unwrap_or(42))
    };

    while session.turn < args.turns {
        let turn_before = session.turn;
        let command = CommandIntent::Wait;
        let outcome = session.submit(command);
        if let Some(path) = &args.replay_out {
            let line = ReplayLineV1 {
                turn_before,
                command,
                snapshot_hash_after: outcome.snapshot_hash.clone(),
                outcome,
            };
            if let Err(error) = save::append_replay_line(path, &line) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        if matches!(session.state, aihack::core::RunState::GameOver { .. }) {
            break;
        }
    }

    if let Some(path) = &args.save {
        if let Err(error) = save::save_session_to_path(&session, path) {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }

    let final_hash = session.snapshot().stable_hash();
    println!(
        "seed={} turns={} final_turn={} final_hash={}",
        session.meta.seed, args.turns, session.turn, final_hash.0
    );
}

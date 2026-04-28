use aihack::core::{CommandIntent, GameSession};
use clap::Parser;

/// [v0.1.0] Phase 1 deterministic headless runner 인자다.
#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 1000)]
    turns: u64,
}

fn main() {
    let args = Args::parse();
    let mut session = GameSession::new(args.seed);

    for _ in 0..args.turns {
        session.submit(CommandIntent::Wait);
    }

    let final_hash = session.snapshot().stable_hash();
    println!(
        "seed={} turns={} final_turn={} final_hash={}",
        args.seed, args.turns, session.turn, final_hash.0
    );
}

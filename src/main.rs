use clap::Parser;

/// [v0.1.0] Phase 10 TUI adapter 진입 인자다.
#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 42)]
    seed: u64,
}

fn main() {
    let args = Args::parse();
    if let Err(error) = aihack::ui::tui::run_tui(args.seed) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

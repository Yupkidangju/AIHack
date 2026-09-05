use clap::Parser;

/// NetHack 3.6.7 호환 Rust 로그라이크의 TUI 실행 인자다.
#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long)]
    high_contrast: bool,
    #[arg(long)]
    reduced_motion: bool,
    #[arg(long, default_value = "runtime/tui")]
    save_dir: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();
    let ui_config = aihack_tui::tui::UiRuntimeConfig {
        high_contrast: args.high_contrast,
        reduced_motion: args.reduced_motion,
        ..Default::default()
    };
    if let Err(error) =
        aihack_tui::tui::run_tui_with_config_and_save_dir(args.seed, ui_config, &args.save_dir)
    {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessibility_flags_are_explicit_and_opt_in() {
        let args = Args::try_parse_from(["aihack", "--high-contrast", "--reduced-motion"]).unwrap();
        assert!(args.high_contrast);
        assert!(args.reduced_motion);
    }
}

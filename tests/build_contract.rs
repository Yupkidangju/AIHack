use std::fs;
use std::path::Path;

fn read_project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path}를 읽을 수 있어야 합니다: {error}"))
}

#[test]
fn manifest_pins_the_r1_toolchain_and_default_binary() {
    let manifest = read_project_file("Cargo.toml");

    assert!(manifest.contains("rust-version = \"1.94\""));
    assert!(manifest.contains("default-run = \"aihack\""));
    assert!(manifest.contains("ratatui = \"0.30\""));
    assert!(manifest.contains("crossterm = \"0.29\""));
}

#[test]
fn build_scripts_use_locked_commands_and_fail_on_missing_artifacts() {
    let linux = read_project_file("build.sh");
    let windows = read_project_file("build.bat");

    for (name, script) in [
        ("build.sh", linux.as_str()),
        ("build.bat", windows.as_str()),
    ] {
        assert!(
            script.contains("cargo test --locked --all-targets"),
            "{name}"
        );
        assert!(script.contains("cargo build --locked"), "{name}");
        assert!(!script.contains("|| true"), "{name}");
        assert!(script.contains("aihack-headless"), "{name}");
    }
}

#[test]
fn ci_and_dependency_policy_run_the_same_locked_gates() {
    let workflow = read_project_file(".github/workflows/ci.yml");
    let deny_config = read_project_file("deny.toml");

    for trigger in ["push:", "pull_request:"] {
        assert!(workflow.contains(trigger), "CI trigger: {trigger}");
    }
    for runner in ["ubuntu-latest", "windows-latest"] {
        assert!(workflow.contains(runner), "CI runner: {runner}");
    }
    for command in [
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets --locked -- -D warnings",
        "cargo test --workspace --all-targets --locked",
        "cargo build --workspace --release --locked",
        "cargo audit",
        "cargo deny check licenses bans sources",
        "git diff --exit-code -- Cargo.lock",
    ] {
        assert!(workflow.contains(command), "CI command: {command}");
    }

    assert!(deny_config.contains("[licenses]"));
    assert!(deny_config.contains("[bans]"));
    assert!(deny_config.contains("[sources]"));
    assert!(deny_config.contains("unknown-registry = \"deny\""));
    assert!(deny_config.contains("unknown-git = \"deny\""));
}

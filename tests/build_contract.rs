use std::fs;
use std::path::Path;

fn read_project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path}를 읽을 수 있어야 합니다: {error}"))
}

#[test]
fn manifest_pins_the_r1_toolchain_and_default_binary() {
    let root_manifest = read_project_file("Cargo.toml");
    let tui_manifest = read_project_file("apps/aihack-tui/Cargo.toml");

    assert!(root_manifest.contains("rust-version = \"1.94\""));
    assert!(root_manifest.contains("default-members = [\"apps/aihack-tui\"]"));
    assert!(!root_manifest.contains("default-run = \"aihack\""));
    assert!(tui_manifest.contains("default-run = \"aihack\""));
    assert!(tui_manifest.contains("ratatui = \"0.30\""));
    assert!(tui_manifest.contains("crossterm = \"0.29\""));
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
            script.contains("cargo test --workspace --locked --all-targets"),
            "{name}"
        );
        assert!(
            script.contains("cargo build --workspace --locked"),
            "{name}"
        );
        assert!(!script.contains("|| true"), "{name}");
        assert!(script.contains("aihack-headless"), "{name}");
    }
}

#[test]
fn workspace_path_dependencies_are_versioned_for_cargo_deny() {
    for manifest_path in [
        "Cargo.toml",
        "crates/aihack-ai-contract/Cargo.toml",
        "crates/aihack-content/Cargo.toml",
        "crates/aihack-llm/Cargo.toml",
        "crates/aihack-runtime/Cargo.toml",
        "apps/aihack-tui/Cargo.toml",
        "apps/aihack-headless/Cargo.toml",
    ] {
        let manifest = read_project_file(manifest_path);
        for line in manifest.lines().filter(|line| line.contains("{ path =")) {
            assert!(
                line.contains("version = \"0.1.0\""),
                "{manifest_path}의 내부 path dependency에는 현재 package version이 필요합니다: {line}"
            );
        }
    }
}

#[test]
fn audit_roadmap_uses_runnable_workspace_commands() {
    let roadmap = read_project_file("audit_roadmap.md");

    assert!(roadmap.contains("cargo metadata --locked --no-deps --format-version 1"));
    assert!(!roadmap.contains("cargo metadata --workspace"));
    assert!(roadmap.contains(
        "cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1"
    ));
}

#[test]
fn active_docs_select_the_intended_workspace_or_root_package() {
    for path in [
        "spec.md",
        "IMPLEMENTATION_SUMMARY.md",
        "audit_roadmap.md",
        "GAP_CLOSURE_ROADMAP.md",
        "BUILD_GUIDE.md",
        "README.md",
    ] {
        let document = read_project_file(path);
        for line in document.lines().map(str::trim) {
            for forbidden in [
                "cargo test --locked --test",
                "cargo test --workspace --locked --test",
                "cargo check --locked",
                "cargo build --locked",
            ] {
                assert!(
                    !line.starts_with(forbidden),
                    "{path}에 현재 workspace에서 범위가 잘못된 명령이 있습니다: {line}"
                );
            }
        }
    }
}

#[test]
fn active_task_file_owners_and_generated_output_policy_match_r5() {
    let summary = read_project_file("IMPLEMENTATION_SUMMARY.md");
    for line in summary
        .lines()
        .filter(|line| line.starts_with("**파일:**") || line.starts_with("**현재 파일:**"))
    {
        for removed_path in [
            "`src/data/schema.rs`",
            "`src/data/levels/main_2.toml`",
            "`src/bin/aihack-headless.rs`",
            "`src/ui/tui/",
        ] {
            assert!(
                !line.contains(removed_path),
                "현재 Task 책임표가 삭제된 monolith 경로를 가리킵니다: {line}"
            );
        }
    }

    let gitignore = read_project_file(".gitignore");
    assert!(gitignore.lines().any(|line| line.trim() == "/output/"));
}

#[test]
fn completed_task_file_counts_match_their_active_owner_lists() {
    let summary = read_project_file("IMPLEMENTATION_SUMMARY.md");
    let completed_tasks = summary
        .split("### Task R6-1:")
        .next()
        .expect("R1~R5 완료 Task 구간이 있어야 합니다");
    let mut active_file_list: Option<(&str, usize)> = None;

    for line in completed_tasks.lines() {
        if line.starts_with("**파일:**") || line.starts_with("**현재 파일:**") {
            active_file_list = Some((line, line.matches('`').count() / 2));
            continue;
        }

        if line.starts_with("**범위:**") {
            let Some((file_line, actual_count)) = active_file_list.take() else {
                continue;
            };
            let declared_count = line
                .split(',')
                .nth(1)
                .map(|count| {
                    count
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>()
                })
                .and_then(|count| count.parse::<usize>().ok())
                .unwrap_or_else(|| panic!("Task 범위 파일 수를 해석할 수 없습니다: {line}"));

            assert_eq!(
                actual_count, declared_count,
                "현재 파일 목록과 범위 수량이 일치해야 합니다: {file_line} / {line}"
            );
        }
    }

    assert!(
        active_file_list.is_none(),
        "마지막 완료 Task의 파일 목록에도 범위 수량이 있어야 합니다"
    );
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

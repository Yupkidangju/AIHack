use std::fs;
use std::path::Path;

use saphyr::{LoadableYamlNode, Yaml};

fn read_project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path}를 읽을 수 있어야 합니다: {error}"))
}

fn collect_action_uses(node: &Yaml<'_>, uses: &mut Vec<String>) -> Result<(), String> {
    match node {
        Yaml::Mapping(mapping) => {
            for (key, value) in mapping {
                if key.as_str() == Some("uses") {
                    uses.push(
                        value
                            .as_str()
                            .ok_or_else(|| "uses value must be a string".to_string())?
                            .to_string(),
                    );
                }
                collect_action_uses(value, uses)?;
            }
        }
        Yaml::Sequence(sequence) => {
            for value in sequence {
                collect_action_uses(value, uses)?;
            }
        }
        Yaml::Tagged(_, value) => collect_action_uses(value, uses)?,
        _ => {}
    }
    Ok(())
}

fn action_uses_from_yaml(source: &str) -> Result<Vec<String>, String> {
    let documents = Yaml::load_from_str(source).map_err(|error| error.to_string())?;
    let mut uses = Vec::new();
    for document in &documents {
        collect_action_uses(document, &mut uses)?;
    }
    Ok(uses)
}

fn github_yaml_files(root: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(root).unwrap_or_else(|error| panic!("{}: {error}", root.display())) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            github_yaml_files(&path, files);
        } else if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("yml" | "yaml")
        ) {
            files.push(path);
        }
    }
}

fn validate_remote_action_reference(action: &str) -> Result<(), String> {
    if action.starts_with("./") {
        return Ok(());
    }
    if action.starts_with("docker://") {
        let (_, digest) = action
            .rsplit_once("@sha256:")
            .ok_or_else(|| format!("container action digest missing: {action}"))?;
        if digest.len() != 64
            || !digest
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(format!(
                "container action digest must be full SHA-256: {action}"
            ));
        }
        return Ok(());
    }
    let (_, reference) = action
        .rsplit_once('@')
        .ok_or_else(|| format!("action ref missing: {action}"))?;
    if reference.len() != 40
        || !reference
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(format!("action ref must be a full SHA: {action}"));
    }
    Ok(())
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
    let attributes = read_project_file(".gitattributes");

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
        assert!(script.contains("candidate_date"), "{name}");
    }
    assert!(linux.contains("mktemp -d"));
    assert!(linux.contains("release-stage"));
    assert!(linux.contains("verify_release_bundle.sh\" \"$PACKAGE_DIR"));
    assert!(windows.contains("release_staging.ps1"));
    assert!(windows.contains("-Mode Promote"));
    assert!(read_project_file("scripts/release_staging.ps1").contains(".release-stage-"));

    assert!(
        attributes
            .lines()
            .any(|line| line == "legacy_nethack_port_reference export-ignore"),
        "legacy reference directory must be recursively excluded from git archive"
    );
    assert!(
        attributes.lines().any(|line| line == "*.bat text eol=crlf"),
        "Windows batch entrypoints require deterministic CRLF checkout"
    );
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
                line.contains("version = \"0.3.0\""),
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
        "cargo test --locked -p aihack --test dependency_exception_gate",
        "cargo test --locked -p aihack --test dependency_duplicate_gate",
        "cargo build --workspace --release --locked",
        "cargo audit",
        "cargo deny check licenses bans sources",
        "git diff --exit-code -- Cargo.lock",
    ] {
        assert!(workflow.contains(command), "CI command: {command}");
    }
    let mut yaml_files = Vec::new();
    github_yaml_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join(".github"),
        &mut yaml_files,
    );
    yaml_files.sort();
    let mut uses = Vec::new();
    for path in &yaml_files {
        let source = fs::read_to_string(path).unwrap();
        uses.extend(action_uses_from_yaml(&source).unwrap());
    }
    assert!(
        !uses.is_empty(),
        ".github YAML must contain action uses entries"
    );
    for action in uses {
        validate_remote_action_reference(&action).unwrap();
    }
    assert!(workflow.contains("# actions/checkout v4.4.0"));
    assert!(!workflow.contains("# actions/checkout v4.2.2"));
    assert!(!workflow.contains("actions/checkout@v4"));
    assert!(!workflow.contains("dtolnay/rust-toolchain@1.94.1"));

    for release_gate in [
        "if: runner.os == 'Linux'",
        "run: ./build.sh --release",
        "if: runner.os == 'Windows'",
        "run: cmd /c build.bat --release",
    ] {
        assert!(
            workflow.contains(release_gate),
            "CI actual release gate: {release_gate}"
        );
    }

    assert!(deny_config.contains("[licenses]"));
    assert!(deny_config.contains("[bans]"));
    assert!(deny_config.contains("[sources]"));
    assert!(deny_config.contains("unknown-registry = \"deny\""));
    assert!(deny_config.contains("unknown-git = \"deny\""));
}

#[test]
fn yaml_action_pin_gate_rejects_inline_spaced_nested_and_composite_mutable_refs() {
    let fixture = r#"
jobs:
  inline:
    steps:
      - { uses: actions/setup-node@v4 }
      - uses : actions/cache@v4
      - uses: docker://alpine:3.22
      - name: nested
        with:
          composite:
            uses: owner/action/path@main
"#;
    let uses = action_uses_from_yaml(fixture).unwrap();
    assert_eq!(uses.len(), 4);
    for action in uses {
        assert!(
            validate_remote_action_reference(&action).is_err(),
            "mutable ref가 거부되어야 합니다: {action}"
        );
    }

    let local_and_pinned = r#"
runs:
  steps:
    - uses: ./local-action
    - uses: docker://alpine@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
    - uses: owner/action@0123456789abcdef0123456789abcdef01234567
"#;
    for action in action_uses_from_yaml(local_and_pinned).unwrap() {
        validate_remote_action_reference(&action).unwrap();
    }
}

#[test]
fn winx_license_exception_is_version_scoped_and_time_bounded() {
    let deny_config = read_project_file("deny.toml");
    let guide = read_project_file("BUILD_GUIDE.md");
    let ledger = read_project_file("dependency-exceptions.json");

    for contract in [
        "[[licenses.exceptions]]",
        "name = \"winx\"",
        "version = \"=0.36.4\"",
        "allow = [\"Apache-2.0 WITH LLVM-exception\"]",
    ] {
        assert!(deny_config.contains(contract), "deny.toml 누락: {contract}");
    }
    for contract in [
        "Dependency owner / Release manager",
        "2026-10-31",
        "winx 0.36.4",
        "다른 crate에는 이 exception을 확장하지 않는다",
    ] {
        assert!(guide.contains(contract), "BUILD_GUIDE 누락: {contract}");
    }
    for contract in [
        "DEP-EXC-0001",
        "\"winx\": \"0.36.4\"",
        "\"expires_on\": \"2026-10-31\"",
        "\"cap-primitives\": \"4.0.2\"",
    ] {
        assert!(
            ledger.contains(contract),
            "exception ledger 누락: {contract}"
        );
    }
}

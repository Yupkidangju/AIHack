use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);

fn project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path} 읽기 실패: {error}"))
}

fn bash_program() -> PathBuf {
    #[cfg(windows)]
    {
        let output = Command::new("git")
            .arg("--exec-path")
            .output()
            .expect("Git exec path 조회 실패");
        assert!(output.status.success(), "Git exec path 조회 실패");
        let exec_path = String::from_utf8(output.stdout).expect("Git exec path UTF-8 변환 실패");
        for ancestor in Path::new(exec_path.trim()).ancestors() {
            let candidate = ancestor.join("bin/bash.exe");
            if candidate.is_file() {
                return candidate;
            }
        }
        panic!("Git Bash 실행 파일을 찾지 못했다: {}", exec_path.trim());
    }

    #[cfg(not(windows))]
    {
        PathBuf::from("bash")
    }
}

struct ReleaseFixture {
    root: PathBuf,
}

impl ReleaseFixture {
    fn approved() -> Self {
        let root = std::env::temp_dir().join(format!(
            "aihack-r8-checkpoint-{}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }

        for directory in [
            "scripts",
            "crates/aihack-core",
            "crates/aihack-content",
            "crates/aihack-ai-contract",
            "crates/aihack-llm",
            "crates/aihack-runtime",
            "apps/aihack-tui",
            "apps/aihack-headless",
            ".archive",
        ] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }

        fs::write(
            root.join("scripts/r8_checkpoint.sh"),
            project_file("scripts/r8_checkpoint.sh"),
        )
        .unwrap();
        fs::write(
            root.join("scripts/r7_checkpoint.sh"),
            "#!/usr/bin/env bash\nprintf '%s\\n' 'R7 CHECKPOINT: PASS'\n",
        )
        .unwrap();
        fs::write(
            root.join("scripts/verify_release_bundle.sh"),
            project_file("scripts/verify_release_bundle.sh"),
        )
        .unwrap();

        let manifests = [
            "Cargo.toml",
            "crates/aihack-core/Cargo.toml",
            "crates/aihack-content/Cargo.toml",
            "crates/aihack-ai-contract/Cargo.toml",
            "crates/aihack-llm/Cargo.toml",
            "crates/aihack-runtime/Cargo.toml",
            "apps/aihack-tui/Cargo.toml",
            "apps/aihack-headless/Cargo.toml",
        ];
        for (index, manifest) in manifests.into_iter().enumerate() {
            let dependency = if index == 0 {
                "\n[dependencies]\naihack-core = { path = \"crates/aihack-core\", version = \"0.3.0\" }\n"
            } else {
                ""
            };
            fs::write(
                root.join(manifest),
                format!(
                    "[package]\nname = \"fixture-{index}\"\nversion = \"0.3.0\"\nlicense = \"NGPL\"\npublish = false\n{dependency}"
                ),
            )
            .unwrap();
        }
        fs::write(root.join("Cargo.lock"), "# fixture lock\n").unwrap();

        let archive_pairs = [
            ("spec.md", "spec_archive.md"),
            ("designs.md", "designs_archive.md"),
            ("DESIGN_DECISIONS.md", "decisions_archive.md"),
            ("BUILD_GUIDE.md", "build_archive.md"),
            ("IMPLEMENTATION_SUMMARY.md", "implementation_archive.md"),
            ("GAP_CLOSURE_ROADMAP.md", "gap_archive.md"),
            ("audit_roadmap.md", "audit_archive.md"),
        ];
        for (document, archive) in archive_pairs {
            fs::write(
                root.join(document),
                format!("> Archive chain\n> - Latest: `.archive/{archive}`\n"),
            )
            .unwrap();
            fs::write(root.join(".archive").join(archive), "immutable fixture\n").unwrap();
        }

        fs::write(root.join("README.md"), "> Current code: Cargo 0.3.0\n").unwrap();
        fs::write(
            root.join("CHANGELOG.md"),
            "# Changelog\n\n## [0.3.0] - 2026-07-20\n",
        )
        .unwrap();
        fs::write(
            root.join("PROVENANCE.md"),
            "**외부 배포 판정: APPROVED — R8 distribution review complete**\n",
        )
        .unwrap();
        for file in [
            "LICENSE",
            "NOTICE",
            "MODIFICATIONS.md",
            "PROJECT_OWNER_LICENSE_APPROVAL.md",
            "RELEASE-METADATA",
            ".gitattributes",
            "build.sh",
            "build.bat",
        ] {
            fs::write(root.join(file), project_file(file)).unwrap();
        }

        Self { root }
    }

    fn replace(&self, path: &str, from: &str, to: &str) {
        let path = self.root.join(path);
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains(from), "fixture source missing: {from}");
        fs::write(path, content.replace(from, to)).unwrap();
    }

    fn replace_once(&self, path: &str, from: &str, to: &str) {
        let path = self.root.join(path);
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains(from), "fixture source missing: {from}");
        fs::write(path, content.replacen(from, to, 1)).unwrap();
    }

    fn remove(&self, path: &str) {
        fs::remove_file(self.root.join(path)).unwrap();
    }

    fn run(&self) -> std::process::Output {
        Command::new(bash_program())
            .arg(self.root.join("scripts/r8_checkpoint.sh"))
            .env_remove("AIHACK_R8_ROOT")
            .output()
            .unwrap()
    }

    fn run_with_root_override(&self, root_override: &str) -> std::process::Output {
        Command::new(bash_program())
            .arg(self.root.join("scripts/r8_checkpoint.sh"))
            .env("AIHACK_R8_ROOT", root_override)
            .output()
            .unwrap()
    }
}

impl Drop for ReleaseFixture {
    fn drop(&mut self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}

#[test]
fn r8_checkpoint_accepts_a_complete_release_fixture() {
    let fixture = ReleaseFixture::approved();

    let output = fixture.run();

    assert!(
        output.status.success(),
        "complete fixture가 거부됐다: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("R8 CHECKPOINT: PASS"));
}

#[test]
fn r8_checkpoint_holds_for_r7_version_license_and_distribution_approval() {
    let fixture = ReleaseFixture::approved();
    fixture.replace(
        "scripts/r7_checkpoint.sh",
        "R7 CHECKPOINT: PASS",
        "R7 CHECKPOINT: HOLD",
    );
    fixture.replace("scripts/r7_checkpoint.sh", "\n", "\nexit 1\n");
    fixture.replace("Cargo.toml", "version = \"0.3.0\"", "version = \"0.1.0\"");
    for manifest in [
        "crates/aihack-core/Cargo.toml",
        "crates/aihack-content/Cargo.toml",
        "crates/aihack-ai-contract/Cargo.toml",
        "crates/aihack-llm/Cargo.toml",
        "crates/aihack-runtime/Cargo.toml",
        "apps/aihack-tui/Cargo.toml",
        "apps/aihack-headless/Cargo.toml",
    ] {
        fixture.replace_once(manifest, "version = \"0.3.0\"", "version = \"0.1.0\"");
    }
    fixture.replace(
        "Cargo.toml",
        "license = \"NGPL\"",
        "license = \"UNLICENSED\"",
    );
    fixture.replace("README.md", "Cargo 0.3.0", "Cargo 0.1.0");
    fixture.replace("CHANGELOG.md", "## [0.3.0]", "## [Unreleased]");
    fixture.replace("PROVENANCE.md", "APPROVED", "BLOCKED");

    let output = fixture.run();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(1), "stdout={stdout}");
    assert!(stdout.contains("R8 CHECKPOINT: HOLD"));
    assert!(stdout.contains("R7 approval checkpoint pending"));
    assert!(stdout.contains("workspace release version must be 0.3.0"));
    assert!(stdout.contains("workspace distribution license must be NGPL"));
    assert!(stdout.contains("README release version pending"));
    assert!(stdout.contains("CHANGELOG 0.3.0 release entry pending"));
    assert!(stdout.contains("external distribution approval pending"));
}

#[test]
fn r8_checkpoint_rejects_path_dependency_version_drift() {
    let fixture = ReleaseFixture::approved();
    fixture.replace(
        "Cargo.toml",
        "version = \"0.3.0\" }",
        "version = \"0.1.0\" }",
    );

    let output = fixture.run();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(2), "stdout={stdout}");
    assert!(stdout.contains("R8 CHECKPOINT: FAIL"));
    assert!(stdout.contains("path dependency version drift"));
}

#[test]
fn r8_checkpoint_rejects_a_broken_archive_chain() {
    let fixture = ReleaseFixture::approved();
    fixture.remove(".archive/spec_archive.md");

    let output = fixture.run();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(2), "stdout={stdout}");
    assert!(stdout.contains("R8 CHECKPOINT: FAIL"));
    assert!(stdout.contains("archive target missing or empty"));
}

#[test]
fn r8_checkpoint_rejects_missing_modification_evidence() {
    let fixture = ReleaseFixture::approved();
    fixture.remove("MODIFICATIONS.md");

    let output = fixture.run();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(2), "stdout={stdout}");
    assert!(stdout.contains("R8 CHECKPOINT: FAIL"));
    assert!(stdout.contains("required file missing: MODIFICATIONS.md"));
}

#[test]
fn r8_checkpoint_rejects_a_verifier_without_authority_integrity_checks() {
    let fixture = ReleaseFixture::approved();
    fixture.replace(
        "scripts/verify_release_bundle.sh",
        "require_metadata_value",
        "metadata_validation_removed",
    );

    let output = fixture.run();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(2), "stdout={stdout}");
    assert!(stdout.contains("R8 CHECKPOINT: FAIL"));
    assert!(stdout.contains("release verifier reference-integrity missing"));
}

#[test]
fn r8_checkpoint_ignores_inherited_root_override() {
    let fixture = ReleaseFixture::approved();

    let output = fixture.run_with_root_override("/definitely/not/aihack");

    assert!(
        output.status.success(),
        "inherited root가 checkpoint를 redirect했다: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("/definitely/not/aihack"));
}

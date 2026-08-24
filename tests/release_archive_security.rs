use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct FixtureDir(PathBuf);

impl FixtureDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "aihack-r29-archive-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for FixtureDir {
    fn drop(&mut self) {
        if self.0.exists() {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
}

fn project_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn python() -> &'static str {
    for candidate in ["python", "python3"] {
        if Command::new(candidate)
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
        {
            return candidate;
        }
    }
    panic!("Python 3 interpreter is required for release archive verification");
}

fn generate(format: &str, case: &str, output: &Path, source: Option<&Path>) {
    let mut command = Command::new(python());
    command
        .arg(project_path("tests/support/archive_fixture.py"))
        .args(["--format", format, "--case", case, "--output"])
        .arg(output);
    if let Some(source) = source {
        command.arg("--source").arg(source);
    }
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "fixture generation failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn verify(archive: &Path, format: &str, validate_only: bool) -> Output {
    let mut command = Command::new(python());
    command
        .arg(project_path("scripts/verify_source_archive.py"))
        .arg("--archive")
        .arg(archive)
        .args(["--format", format]);
    if validate_only {
        command.arg("--validate-only");
    } else {
        let commit = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap();
        assert!(commit.status.success());
        command
            .arg("--repository-root")
            .arg(env!("CARGO_MANIFEST_DIR"))
            .arg("--expected-commit")
            .arg(String::from_utf8(commit.stdout).unwrap().trim());
    }
    command.output().unwrap()
}

#[test]
fn format_aware_validator_rejects_windows_components_collisions_prefixes_and_types() {
    let fixture = FixtureDir::new();
    for format in ["zip", "tar.gz"] {
        let normal = fixture.0.join(format!("{format}-normal.archive"));
        generate(format, "normal", &normal, None);
        let positive = verify(&normal, format, true);
        assert!(
            positive.status.success(),
            "normal format-aware fixture rejected: {}",
            String::from_utf8_lossy(&positive.stderr)
        );
        for case in [
            "forbidden_question",
            "forbidden_pipe",
            "forbidden_quote",
            "forbidden_angle",
            "superscript_com",
            "superscript_lpt",
            "console_in",
            "console_out",
            "raw_c0_control",
            "raw_c1_control",
            "unicode_collision",
            "prefix_conflict",
            "symlink",
        ] {
            let archive = fixture.0.join(format!("{format}-{case}.archive"));
            generate(format, case, &archive, None);
            assert!(
                !verify(&archive, format, true).status.success(),
                "validator accepted format={format} case={case}"
            );
        }
    }
    for case in ["hardlink", "device"] {
        let archive = fixture.0.join(format!("tar-{case}.archive"));
        generate("tar.gz", case, &archive, None);
        assert!(!verify(&archive, "tar.gz", true).status.success());
    }
}

#[test]
fn expected_commit_requires_the_complete_exported_archive_identity() {
    let fixture = FixtureDir::new();
    for format in ["zip", "tar.gz"] {
        let actual = fixture.0.join(format!("actual-{format}.archive"));
        let archive_format = if format == "zip" { "zip" } else { "tar.gz" };
        let status = Command::new("git")
            .args(["archive", &format!("--format={archive_format}"), "--output"])
            .arg(&actual)
            .arg("HEAD")
            .status()
            .unwrap();
        assert!(status.success());
        let positive = verify(&actual, format, false);
        assert!(
            positive.status.success(),
            "actual archive rejected: {}",
            String::from_utf8_lossy(&positive.stderr)
        );

        let docs_only = fixture.0.join(format!("{format}-docs-only.archive"));
        generate(format, "docs_only", &docs_only, None);
        assert!(
            !verify(&docs_only, format, false).status.success(),
            "ExpectedCommit accepted a document-only archive: format={format}"
        );

        for case in ["omission", "blob_changed", "safe_extra", "type_mutation"] {
            let mutated = fixture.0.join(format!("{format}-{case}.archive"));
            generate(format, case, &mutated, Some(&actual));
            assert!(
                !verify(&mutated, format, false).status.success(),
                "ExpectedCommit accepted format={format} case={case}"
            );
        }
    }
}

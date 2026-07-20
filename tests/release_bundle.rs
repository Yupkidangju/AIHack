#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);
const OWNER_APPROVAL_ID: &str = "AIHACK-OWNER-2026-07-20-NGPL-01";
const MODIFICATION_NOTICE_ID: &str = "AIHACK-MODIFICATIONS-2026-07-20-01";

fn project_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

struct BundleFixture {
    root: PathBuf,
    commit: String,
}

#[derive(Clone, Copy)]
enum BundleCase {
    Complete,
    MissingApprovalRecord,
    MissingOwnerMetadata,
    MismatchedOwnerRecord,
    MissingModificationMetadata,
    MismatchedModificationRecord,
    IncludedLegacy,
}

impl BundleFixture {
    fn new(case: BundleCase) -> Self {
        let root = std::env::temp_dir().join(format!(
            "aihack-release-bundle-{}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(root.join("output")).unwrap();

        for file in ["LICENSE", "NOTICE", "MODIFICATIONS.md", "RELEASE-METADATA"] {
            fs::copy(project_path(file), root.join(file)).unwrap();
        }
        if !matches!(case, BundleCase::MissingApprovalRecord) {
            fs::copy(
                project_path("PROJECT_OWNER_LICENSE_APPROVAL.md"),
                root.join("PROJECT_OWNER_LICENSE_APPROVAL.md"),
            )
            .unwrap();
        }
        if matches!(case, BundleCase::MismatchedOwnerRecord) {
            replace_in_file(
                &root.join("PROJECT_OWNER_LICENSE_APPROVAL.md"),
                OWNER_APPROVAL_ID,
                "AIHACK-OWNER-MISMATCH",
            );
        }
        if matches!(case, BundleCase::MismatchedModificationRecord) {
            replace_in_file(
                &root.join("MODIFICATIONS.md"),
                MODIFICATION_NOTICE_ID,
                "AIHACK-MODIFICATIONS-MISMATCH",
            );
        }
        if matches!(case, BundleCase::MissingOwnerMetadata) {
            remove_line_containing(&root.join("RELEASE-METADATA"), "owner_approval=");
        }
        if matches!(case, BundleCase::MissingModificationMetadata) {
            remove_line_containing(&root.join("RELEASE-METADATA"), "modification_notice=");
        }

        let attributes = if !matches!(case, BundleCase::IncludedLegacy) {
            "legacy_nethack_port_reference export-ignore\nRELEASE-METADATA export-subst\n"
        } else {
            "RELEASE-METADATA export-subst\n"
        };
        fs::write(root.join(".gitattributes"), attributes).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"aihack-bundle-fixture\"\nversion = \"0.3.0\"\nlicense = \"NGPL\"\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("legacy_nethack_port_reference")).unwrap();
        fs::write(
            root.join("legacy_nethack_port_reference/probe.txt"),
            "must not ship\n",
        )
        .unwrap();

        for args in [
            &["init", "-q"][..],
            &["config", "user.name", "AIHack release test"][..],
            &["config", "user.email", "release-test@invalid"][..],
            &["add", "."][..],
            &["commit", "-qm", "release fixture"][..],
        ] {
            let status = Command::new("git")
                .args(args)
                .current_dir(&root)
                .status()
                .unwrap();
            assert!(status.success(), "git fixture command 실패: {args:?}");
        }
        let commit = String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&root)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();

        let output = root.join("output");
        fs::write(output.join("aihack"), "fixture binary\n").unwrap();
        fs::write(output.join("aihack-headless"), "fixture binary\n").unwrap();
        for file in ["LICENSE", "NOTICE", "MODIFICATIONS.md"] {
            fs::copy(root.join(file), output.join(file)).unwrap();
        }
        if !matches!(case, BundleCase::MissingApprovalRecord) {
            fs::copy(
                root.join("PROJECT_OWNER_LICENSE_APPROVAL.md"),
                output.join("PROJECT_OWNER_LICENSE_APPROVAL.md"),
            )
            .unwrap();
        }
        let output_metadata = fs::read_to_string(root.join("RELEASE-METADATA"))
            .unwrap()
            .replace("$Format:%H$", &commit);
        fs::write(output.join("RELEASE-METADATA"), output_metadata).unwrap();
        let archive = output.join("aihack-0.3.0-source.tar.gz");
        let status = Command::new("git")
            .args([
                "archive",
                "--format=tar.gz",
                &format!("--output={}", archive.display()),
                "HEAD",
            ])
            .current_dir(&root)
            .status()
            .unwrap();
        assert!(status.success());

        let mut names = vec![
            "aihack",
            "aihack-headless",
            "LICENSE",
            "NOTICE",
            "MODIFICATIONS.md",
            "RELEASE-METADATA",
            "aihack-0.3.0-source.tar.gz",
        ];
        if !matches!(case, BundleCase::MissingApprovalRecord) {
            names.push("PROJECT_OWNER_LICENSE_APPROVAL.md");
        }
        let hashes = Command::new("sha256sum")
            .args(names)
            .current_dir(&output)
            .output()
            .unwrap();
        assert!(hashes.status.success());
        fs::write(output.join("SHA256SUMS"), hashes.stdout).unwrap();

        Self { root, commit }
    }

    fn verify(&self) -> std::process::Output {
        Command::new("bash")
            .arg(project_path("scripts/verify_release_bundle.sh"))
            .arg(self.root.join("output"))
            .arg(&self.commit)
            .output()
            .unwrap()
    }
}

fn replace_in_file(path: &Path, from: &str, to: &str) {
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains(from));
    fs::write(path, content.replace(from, to)).unwrap();
}

fn remove_line_containing(path: &Path, needle: &str) {
    let content = fs::read_to_string(path).unwrap();
    assert!(content.lines().any(|line| line.contains(needle)));
    let filtered = content
        .lines()
        .filter(|line| !line.contains(needle))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, format!("{filtered}\n")).unwrap();
}

impl Drop for BundleFixture {
    fn drop(&mut self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}

#[test]
fn verifier_accepts_commit_bound_bundle_with_notices_and_checksums() {
    let fixture = BundleFixture::new(BundleCase::Complete);
    let output = fixture.verify();
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(&fixture.commit));
}

#[test]
fn verifier_rejects_missing_or_mismatched_authority_references() {
    for (case, expected_error) in [
        (
            BundleCase::MissingApprovalRecord,
            "PROJECT_OWNER_LICENSE_APPROVAL.md",
        ),
        (BundleCase::MissingOwnerMetadata, "owner_approval"),
        (BundleCase::MismatchedOwnerRecord, "Approval ID"),
        (
            BundleCase::MissingModificationMetadata,
            "modification_notice",
        ),
        (BundleCase::MismatchedModificationRecord, "Notice ID"),
    ] {
        let fixture = BundleFixture::new(case);
        let output = fixture.verify();
        assert!(
            !output.status.success(),
            "누락·불일치 authority reference를 허용하면 안 됩니다: {expected_error}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected_error),
            "예상 오류가 필요합니다: {expected_error}; stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn verifier_rejects_a_source_archive_containing_the_blocked_legacy_tree() {
    let fixture = BundleFixture::new(BundleCase::IncludedLegacy);
    let output = fixture.verify();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("excluded path"));
}

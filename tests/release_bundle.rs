#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);
const OWNER_APPROVAL_ID: &str = "AIHACK-OWNER-2026-07-20-NGPL-01";
const MODIFICATION_NOTICE_ID: &str = "AIHACK-MODIFICATIONS-2026-08-24-01";
const CANDIDATE_DATE: &str = "2026-08-24";

fn project_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

struct BundleFixture {
    root: PathBuf,
    commit: String,
    candidate_date: String,
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
    SimilarLegacyName,
    MinimumYearCalendar,
    MaximumYearCalendar,
    StaleModificationPeriod,
}

#[derive(Clone, Copy, Debug)]
enum MetadataTarget {
    Archive,
    Output,
}

#[derive(Clone, Copy, Debug)]
enum MetadataFault {
    WrongOwner,
    SuffixedOwner,
    DuplicateOwner,
    WrongModification,
    SuffixedModification,
    DuplicateModification,
}

impl BundleFixture {
    fn new(case: BundleCase) -> Self {
        Self::build(case, None)
    }

    fn with_metadata_fault(target: MetadataTarget, fault: MetadataFault) -> Self {
        Self::build(BundleCase::Complete, Some((target, fault)))
    }

    fn build(case: BundleCase, metadata_fault: Option<(MetadataTarget, MetadataFault)>) -> Self {
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
        if matches!(case, BundleCase::StaleModificationPeriod) {
            replace_in_file(
                &root.join("MODIFICATIONS.md"),
                "Covered change period: `2025-05-20..2026-08-24`",
                "Covered change period: `2025-05-20..2026-08-23`",
            );
        }
        let candidate_date = match case {
            BundleCase::MinimumYearCalendar => "0001-06-15",
            BundleCase::MaximumYearCalendar => "9999-06-15",
            _ => CANDIDATE_DATE,
        };
        let edge_period = match case {
            BundleCase::MinimumYearCalendar => Some("0001-01-01..0001-12-31"),
            BundleCase::MaximumYearCalendar => Some("9999-01-01..9999-12-31"),
            _ => None,
        };
        if let Some(period) = edge_period {
            replace_in_file(
                &root.join("MODIFICATIONS.md"),
                "2025-05-20..2026-08-24",
                period,
            );
            replace_in_file(
                &root.join("RELEASE-METADATA"),
                "candidate_date=$Format:%cs$",
                &format!("candidate_date={candidate_date}"),
            );
        }
        if matches!(case, BundleCase::MissingOwnerMetadata) {
            remove_line_containing(&root.join("RELEASE-METADATA"), "owner_approval=");
        }
        if matches!(case, BundleCase::MissingModificationMetadata) {
            remove_line_containing(&root.join("RELEASE-METADATA"), "modification_notice=");
        }
        if let Some((MetadataTarget::Archive, fault)) = metadata_fault {
            mutate_metadata_file(&root.join("RELEASE-METADATA"), fault);
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
        if matches!(case, BundleCase::SimilarLegacyName) {
            fs::create_dir_all(root.join("legacy_nethack_port_reference_backup")).unwrap();
            fs::write(
                root.join("legacy_nethack_port_reference_backup/probe.txt"),
                "allowed similar name\n",
            )
            .unwrap();
        }

        for args in [
            &["init", "-q"][..],
            &["config", "user.name", "AIHack release test"][..],
            &["config", "user.email", "release-test@invalid"][..],
            &["add", "."][..],
        ] {
            let status = Command::new("git")
                .args(args)
                .current_dir(&root)
                .status()
                .unwrap();
            assert!(status.success(), "git fixture command 실패: {args:?}");
        }
        if matches!(case, BundleCase::IncludedLegacy) {
            let status = Command::new("git")
                .args(["add", "-f", "legacy_nethack_port_reference/probe.txt"])
                .current_dir(&root)
                .status()
                .unwrap();
            assert!(status.success(), "legacy negative fixture 강제 추적 실패");
        }
        let status = Command::new("git")
            .args(["commit", "-qm", "release fixture"])
            .env("GIT_AUTHOR_DATE", "2026-08-24T12:00:00+09:00")
            .env("GIT_COMMITTER_DATE", "2026-08-24T12:00:00+09:00")
            .current_dir(&root)
            .status()
            .unwrap();
        assert!(status.success(), "release fixture commit 실패");
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
        let mut output_metadata = if matches!(metadata_fault, Some((MetadataTarget::Archive, _))) {
            fs::read_to_string(project_path("RELEASE-METADATA")).unwrap()
        } else {
            fs::read_to_string(root.join("RELEASE-METADATA")).unwrap()
        };
        if let Some((MetadataTarget::Output, fault)) = metadata_fault {
            output_metadata = mutate_metadata(&output_metadata, fault);
        }
        output_metadata = output_metadata
            .replace("$Format:%H$", &commit)
            .replace("$Format:%cs$", candidate_date);
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
        if matches!(case, BundleCase::IncludedLegacy) {
            let listing = Command::new("tar")
                .args(["-tzf", archive.to_str().unwrap()])
                .output()
                .unwrap();
            assert!(listing.status.success());
            assert!(
                String::from_utf8_lossy(&listing.stdout)
                    .contains("legacy_nethack_port_reference/probe.txt"),
                "legacy negative fixture archive에 차단 경로가 실제 포함되어야 한다"
            );
        }

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

        Self {
            root,
            commit,
            candidate_date: candidate_date.to_owned(),
        }
    }

    fn verify(&self) -> std::process::Output {
        self.verify_with_candidate(&self.candidate_date)
    }

    fn verify_with_candidate(&self, candidate_date: &str) -> std::process::Output {
        Command::new("bash")
            .arg(project_path("scripts/verify_release_bundle.sh"))
            .arg(self.root.join("output"))
            .arg(&self.commit)
            .arg(candidate_date)
            .arg(&self.root)
            .output()
            .unwrap()
    }

    fn rewrite_archive_with_path_alias(&self, alias: &str) {
        let unpacked = self.root.join("archive-rewrite");
        fs::create_dir_all(&unpacked).unwrap();
        let archive = self.root.join("output/aihack-0.3.0-source.tar.gz");
        let extract = Command::new("tar")
            .args([
                "-xzf",
                archive.to_str().unwrap(),
                "-C",
                unpacked.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(extract.success());
        fs::write(unpacked.join("blocked-probe"), "blocked\n").unwrap();
        let transform = format!("s,^blocked-probe$,{},", alias.replace('\\', "\\\\"));
        let create = Command::new("tar")
            .args([
                "-czf",
                archive.to_str().unwrap(),
                "--transform",
                &transform,
                "-C",
                unpacked.to_str().unwrap(),
                "LICENSE",
                "NOTICE",
                "MODIFICATIONS.md",
                "PROJECT_OWNER_LICENSE_APPROVAL.md",
                "RELEASE-METADATA",
                "Cargo.toml",
                "blocked-probe",
            ])
            .status()
            .unwrap();
        assert!(create.success());
        rewrite_checksums(&self.root.join("output"));
    }

    fn rewrite_calendar(&self, candidate_date: &str, start: &str, end: &str) {
        let output = self.root.join("output");
        replace_in_file(
            &output.join("MODIFICATIONS.md"),
            "Covered change period: `2025-05-20..2026-08-24`",
            &format!("Covered change period: `{start}..{end}`"),
        );
        replace_in_file(
            &output.join("RELEASE-METADATA"),
            &format!("candidate_date={CANDIDATE_DATE}"),
            &format!("candidate_date={candidate_date}"),
        );
        let unpacked = self.root.join("calendar-rewrite");
        fs::create_dir_all(&unpacked).unwrap();
        let archive = output.join("aihack-0.3.0-source.tar.gz");
        assert!(Command::new("tar")
            .args([
                "-xzf",
                archive.to_str().unwrap(),
                "-C",
                unpacked.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success());
        replace_in_file(
            &unpacked.join("MODIFICATIONS.md"),
            "Covered change period: `2025-05-20..2026-08-24`",
            &format!("Covered change period: `{start}..{end}`"),
        );
        replace_in_file(
            &unpacked.join("RELEASE-METADATA"),
            &format!("candidate_date={CANDIDATE_DATE}"),
            &format!("candidate_date={candidate_date}"),
        );
        assert!(Command::new("tar")
            .args([
                "-czf",
                archive.to_str().unwrap(),
                "-C",
                unpacked.to_str().unwrap(),
                "LICENSE",
                "NOTICE",
                "MODIFICATIONS.md",
                "PROJECT_OWNER_LICENSE_APPROVAL.md",
                "RELEASE-METADATA",
                "Cargo.toml",
            ])
            .status()
            .unwrap()
            .success());
        rewrite_checksums(&output);
    }
}

fn mutate_metadata_file(path: &Path, fault: MetadataFault) {
    let content = fs::read_to_string(path).unwrap();
    fs::write(path, mutate_metadata(&content, fault)).unwrap();
}

fn mutate_metadata(content: &str, fault: MetadataFault) -> String {
    let (key, expected, replacement) = match fault {
        MetadataFault::WrongOwner => ("owner_approval", OWNER_APPROVAL_ID, "WRONG-OWNER".into()),
        MetadataFault::SuffixedOwner => (
            "owner_approval",
            OWNER_APPROVAL_ID,
            format!("{OWNER_APPROVAL_ID}-TAMPERED"),
        ),
        MetadataFault::DuplicateOwner => (
            "owner_approval",
            OWNER_APPROVAL_ID,
            format!("{OWNER_APPROVAL_ID}\nowner_approval={OWNER_APPROVAL_ID}"),
        ),
        MetadataFault::WrongModification => (
            "modification_notice",
            MODIFICATION_NOTICE_ID,
            "WRONG-MODIFICATION".into(),
        ),
        MetadataFault::SuffixedModification => (
            "modification_notice",
            MODIFICATION_NOTICE_ID,
            format!("{MODIFICATION_NOTICE_ID}-TAMPERED"),
        ),
        MetadataFault::DuplicateModification => (
            "modification_notice",
            MODIFICATION_NOTICE_ID,
            format!("{MODIFICATION_NOTICE_ID}\nmodification_notice={MODIFICATION_NOTICE_ID}"),
        ),
    };
    let expected_line = format!("{key}={expected}");
    assert!(content.lines().any(|line| line == expected_line));
    content.replacen(&expected_line, &format!("{key}={replacement}"), 1)
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

fn rewrite_checksums(output: &Path) {
    let names = [
        "aihack",
        "aihack-headless",
        "LICENSE",
        "NOTICE",
        "MODIFICATIONS.md",
        "PROJECT_OWNER_LICENSE_APPROVAL.md",
        "RELEASE-METADATA",
        "aihack-0.3.0-source.tar.gz",
    ];
    let hashes = Command::new("sha256sum")
        .args(names)
        .current_dir(output)
        .output()
        .unwrap();
    assert!(hashes.status.success());
    fs::write(output.join("SHA256SUMS"), hashes.stdout).unwrap();
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
    assert!(String::from_utf8_lossy(&output.stderr).contains("excluded"));
}

#[test]
fn verifier_rejects_canonical_aliases_of_blocked_archive_paths() {
    for alias in [
        "./legacy_nethack_port_reference/probe.txt",
        "././legacy_nethack_port_reference/probe.txt",
        "a/../legacy_nethack_port_reference/probe.txt",
        "/legacy_nethack_port_reference/probe.txt",
        "legacy_nethack_port_reference\\probe.txt",
        "LEGACY_NETHACK_PORT_REFERENCE/probe.txt",
        "Legacy_NetHack_Port_Reference/probe.txt",
        "legacy_nethack_port_reference./probe.txt",
        "legacy_nethack_port_reference /probe.txt",
        "CON/probe.txt",
        "license",
    ] {
        let fixture = BundleFixture::new(BundleCase::Complete);
        fixture.rewrite_archive_with_path_alias(alias);
        let output = fixture.verify();
        assert!(
            !output.status.success(),
            "blocked archive alias was accepted: alias={alias} stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn verifier_accepts_a_normal_similar_archive_name() {
    let fixture = BundleFixture::new(BundleCase::SimilarLegacyName);
    let output = fixture.verify();
    assert!(
        output.status.success(),
        "normal similar archive name was rejected: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn verifier_rejects_non_calendar_candidate_and_period_matrix() {
    for (candidate, start, end) in [
        ("2026-13-01", "2025-05-20", "2026-12-31"),
        ("2026-02-30", "2025-05-20", "2026-12-31"),
        ("2025-02-29", "2025-01-01", "2025-12-31"),
        ("2026-08-24", "2026-00-00", "2026-99-99"),
        ("2026-08-24", "2026-08-25", "2026-08-24"),
        ("0000-06-15", "0000-01-01", "0000-12-31"),
    ] {
        let fixture = BundleFixture::new(BundleCase::Complete);
        fixture.rewrite_calendar(candidate, start, end);
        let output = fixture.verify_with_candidate(candidate);
        assert!(
            !output.status.success(),
            "invalid calendar tuple was accepted: {start} <= {candidate} <= {end}; stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn verifier_accepts_minimum_and_maximum_supported_calendar_years() {
    for (case, candidate) in [
        (BundleCase::MinimumYearCalendar, "0001-06-15"),
        (BundleCase::MaximumYearCalendar, "9999-06-15"),
    ] {
        let fixture = BundleFixture::new(case);
        let output = fixture.verify();
        assert!(
            output.status.success(),
            "supported calendar edge rejected: {candidate}; stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn verifier_rejects_a_candidate_date_outside_the_bundled_modification_period() {
    let fixture = BundleFixture::new(BundleCase::StaleModificationPeriod);
    let output = fixture.verify();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("candidate date"));
}

#[test]
fn verifier_rejects_wrong_suffixed_or_duplicate_metadata_values_in_archive_and_output() {
    for target in [MetadataTarget::Archive, MetadataTarget::Output] {
        for fault in [
            MetadataFault::WrongOwner,
            MetadataFault::SuffixedOwner,
            MetadataFault::DuplicateOwner,
            MetadataFault::WrongModification,
            MetadataFault::SuffixedModification,
            MetadataFault::DuplicateModification,
        ] {
            let fixture = BundleFixture::with_metadata_fault(target, fault);
            let output = fixture.verify();
            assert!(
                !output.status.success(),
                "metadata exactness 우회를 허용하면 안 됩니다: {target:?} {fault:?}"
            );
        }
    }
}

#[test]
fn verifier_rejects_extra_file_directory_and_symbolic_link_entries() {
    use std::os::unix::fs::symlink;

    for name in [
        "UNTRACKED-UNSIGNED-PAYLOAD",
        "unexpected-directory",
        "linked-payload",
    ] {
        let fixture = BundleFixture::new(BundleCase::Complete);
        let output_dir = fixture.root.join("output");
        match name {
            "UNTRACKED-UNSIGNED-PAYLOAD" => {
                fs::write(output_dir.join(name), "unsigned\n").unwrap();
            }
            "unexpected-directory" => fs::create_dir(output_dir.join(name)).unwrap(),
            "linked-payload" => {
                symlink(output_dir.join("aihack"), output_dir.join(name)).unwrap();
            }
            _ => unreachable!(),
        }

        let output = fixture.verify();
        assert!(
            !output.status.success(),
            "release verifier accepted extra output entry: {name}"
        );
    }
}

#[test]
fn verifier_rejects_a_symbolic_link_output_root() {
    use std::os::unix::fs::symlink;

    let fixture = BundleFixture::new(BundleCase::Complete);
    let output = fixture.root.join("output");
    let real_output = fixture.root.join("redirected-output");
    fs::rename(&output, &real_output).unwrap();
    symlink(&real_output, &output).unwrap();

    let result = fixture.verify();
    assert!(
        !result.status.success(),
        "release verifier must reject an output-root symbolic link"
    );
}

#[test]
fn verifier_rejects_an_expected_name_hard_link_without_mutating_its_other_name() {
    let fixture = BundleFixture::new(BundleCase::Complete);
    let output = fixture.root.join("output");
    let victim = fixture.root.join("outside-victim");
    let original = fs::read(output.join("aihack")).unwrap();
    fs::write(&victim, &original).unwrap();
    fs::remove_file(output.join("aihack")).unwrap();
    fs::hard_link(&victim, output.join("aihack")).unwrap();
    rewrite_checksums(&output);

    let result = fixture.verify();
    assert!(
        !result.status.success(),
        "release verifier must reject an expected-name hard link"
    );
    assert_eq!(fs::read(victim).unwrap(), original);
}

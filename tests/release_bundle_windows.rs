#![cfg(windows)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const CANDIDATE_DATE: &str = "2026-08-24";

#[derive(Clone, Copy, Debug)]
enum Fault {
    None,
    IncludedLegacy,
    DuplicateMetadata,
    MismatchedRecord,
    WrongHash,
    EmptyArtifact,
    DuplicateChecksum,
    ExtraFile,
    ExtraDirectory,
    ExtraJunction,
    StaleModificationPeriod,
    DotLegacyAlias,
    RepeatedDotLegacyAlias,
    ParentLegacyAlias,
    AbsoluteLegacyAlias,
    BackslashLegacyAlias,
    UppercaseLegacyAlias,
    MixedCaseLegacyAlias,
    TrailingDotLegacyAlias,
    TrailingSpaceLegacyAlias,
    ReservedDeviceAlias,
    CaseCollisionAlias,
    SimilarLegacyName,
    InvalidCalendarPeriod,
    InvalidCalendarDay,
    InvalidLeapDay,
    ReverseCalendarPeriod,
    InvalidCandidateDate,
    YearZeroCalendar,
    MinimumYearCalendar,
    MaximumYearCalendar,
}

struct Fixture {
    root: PathBuf,
    candidate_date: String,
}

impl Fixture {
    fn new(fault: Fault) -> Self {
        let root = std::env::temp_dir().join(format!(
            "aihack-windows-bundle-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("source")).unwrap();
        fs::create_dir_all(root.join("output")).unwrap();
        let source = root.join("source");
        let output = root.join("output");
        for name in [
            "LICENSE",
            "NOTICE",
            "MODIFICATIONS.md",
            "PROJECT_OWNER_LICENSE_APPROVAL.md",
            "Cargo.toml",
        ] {
            fs::copy(project_path(name), source.join(name)).unwrap();
        }
        let candidate_date = match fault {
            Fault::InvalidCandidateDate => "2026-02-30",
            Fault::YearZeroCalendar => "0000-06-15",
            Fault::MinimumYearCalendar => "0001-06-15",
            Fault::MaximumYearCalendar => "9999-06-15",
            _ => CANDIDATE_DATE,
        };
        let metadata = format!(
            "product=AIHack\nversion=0.3.0\ncommit={COMMIT}\ncandidate_date={candidate_date}\nsource_license=NGPL\nmodification_notice=AIHACK-MODIFICATIONS-2026-08-24-01\nowner_approval=AIHACK-OWNER-2026-07-20-NGPL-01\n"
        );
        fs::write(source.join("RELEASE-METADATA"), &metadata).unwrap();
        if matches!(fault, Fault::StaleModificationPeriod) {
            let modifications = fs::read_to_string(source.join("MODIFICATIONS.md"))
                .unwrap()
                .replace(
                    "Covered change period: `2025-05-20..2026-08-24`",
                    "Covered change period: `2025-05-20..2026-08-23`",
                );
            fs::write(source.join("MODIFICATIONS.md"), modifications).unwrap();
        }
        if matches!(
            fault,
            Fault::InvalidCalendarPeriod
                | Fault::InvalidCalendarDay
                | Fault::InvalidLeapDay
                | Fault::ReverseCalendarPeriod
        ) {
            let replacement = match fault {
                Fault::InvalidCalendarPeriod => "Covered change period: `2026-00-00..2026-99-99`",
                Fault::InvalidCalendarDay => "Covered change period: `2026-02-30..2026-12-31`",
                Fault::InvalidLeapDay => "Covered change period: `2025-02-29..2026-08-24`",
                Fault::ReverseCalendarPeriod => "Covered change period: `2026-08-25..2026-08-24`",
                _ => unreachable!(),
            };
            let modifications = fs::read_to_string(source.join("MODIFICATIONS.md"))
                .unwrap()
                .replace(
                    "Covered change period: `2025-05-20..2026-08-24`",
                    replacement,
                );
            fs::write(source.join("MODIFICATIONS.md"), modifications).unwrap();
        }
        let edge_period = match fault {
            Fault::YearZeroCalendar => Some("Covered change period: `0000-01-01..0000-12-31`"),
            Fault::MinimumYearCalendar => Some("Covered change period: `0001-01-01..0001-12-31`"),
            Fault::MaximumYearCalendar => Some("Covered change period: `9999-01-01..9999-12-31`"),
            _ => None,
        };
        if let Some(edge_period) = edge_period {
            let modifications = fs::read_to_string(source.join("MODIFICATIONS.md"))
                .unwrap()
                .replace(
                    "Covered change period: `2025-05-20..2026-08-24`",
                    edge_period,
                );
            fs::write(source.join("MODIFICATIONS.md"), modifications).unwrap();
        }
        if matches!(fault, Fault::IncludedLegacy) {
            fs::create_dir_all(source.join("legacy_nethack_port_reference")).unwrap();
            fs::write(
                source.join("legacy_nethack_port_reference/probe.txt"),
                "blocked\n",
            )
            .unwrap();
        }

        let mut archive_names = vec![
            "LICENSE",
            "NOTICE",
            "MODIFICATIONS.md",
            "PROJECT_OWNER_LICENSE_APPROVAL.md",
            "RELEASE-METADATA",
            "Cargo.toml",
        ];
        if matches!(fault, Fault::IncludedLegacy) {
            archive_names.push("legacy_nethack_port_reference");
        }
        let archive = output.join("aihack-0.3.0-source.zip");
        let status = Command::new("tar")
            .args(["-a", "-cf", archive.to_str().unwrap()])
            .args(&archive_names)
            .current_dir(&source)
            .status()
            .unwrap();
        assert!(status.success());
        let archive_alias = match fault {
            Fault::DotLegacyAlias => Some("./legacy_nethack_port_reference/probe.txt"),
            Fault::RepeatedDotLegacyAlias => Some("././legacy_nethack_port_reference/probe.txt"),
            Fault::ParentLegacyAlias => Some("a/../legacy_nethack_port_reference/probe.txt"),
            Fault::AbsoluteLegacyAlias => Some("/legacy_nethack_port_reference/probe.txt"),
            Fault::BackslashLegacyAlias => Some("legacy_nethack_port_reference\\probe.txt"),
            Fault::UppercaseLegacyAlias => Some("LEGACY_NETHACK_PORT_REFERENCE/probe.txt"),
            Fault::MixedCaseLegacyAlias => Some("Legacy_NetHack_Port_Reference/probe.txt"),
            Fault::TrailingDotLegacyAlias => Some("legacy_nethack_port_reference./probe.txt"),
            Fault::TrailingSpaceLegacyAlias => Some("legacy_nethack_port_reference /probe.txt"),
            Fault::ReservedDeviceAlias => Some("CON/probe.txt"),
            Fault::CaseCollisionAlias => Some("license"),
            Fault::SimilarLegacyName => Some("legacy_nethack_port_reference_backup/probe.txt"),
            _ => None,
        };
        if let Some(archive_alias) = archive_alias {
            let script = r#"Add-Type -AssemblyName System.IO.Compression; Add-Type -AssemblyName System.IO.Compression.FileSystem; $zip=[System.IO.Compression.ZipFile]::Open($env:AIHACK_ARCHIVE,[System.IO.Compression.ZipArchiveMode]::Update); try { $entry=$zip.CreateEntry($env:AIHACK_ARCHIVE_ALIAS); $writer=[System.IO.StreamWriter]::new($entry.Open()); try { $writer.Write('blocked') } finally { $writer.Dispose() } } finally { $zip.Dispose() }"#;
            let status = Command::new("powershell")
                .args(["-NoProfile", "-Command", script])
                .env("AIHACK_ARCHIVE", &archive)
                .env("AIHACK_ARCHIVE_ALIAS", archive_alias)
                .status()
                .unwrap();
            assert!(status.success());
        }

        for name in [
            "LICENSE",
            "NOTICE",
            "MODIFICATIONS.md",
            "PROJECT_OWNER_LICENSE_APPROVAL.md",
        ] {
            fs::copy(source.join(name), output.join(name)).unwrap();
        }
        fs::write(output.join("RELEASE-METADATA"), &metadata).unwrap();
        fs::write(output.join("aihack.exe"), "fixture tui\n").unwrap();
        fs::write(output.join("aihack-headless.exe"), "fixture headless\n").unwrap();

        match fault {
            Fault::DuplicateMetadata => {
                fs::write(
                    output.join("RELEASE-METADATA"),
                    format!("{metadata}owner_approval=AIHACK-OWNER-2026-07-20-NGPL-01\n"),
                )
                .unwrap();
            }
            Fault::MismatchedRecord => {
                let approval = fs::read_to_string(output.join("PROJECT_OWNER_LICENSE_APPROVAL.md"))
                    .unwrap()
                    .replace("AIHACK-OWNER-2026-07-20-NGPL-01", "AIHACK-OWNER-MISMATCH");
                fs::write(output.join("PROJECT_OWNER_LICENSE_APPROVAL.md"), approval).unwrap();
            }
            Fault::EmptyArtifact => fs::write(output.join("aihack.exe"), []).unwrap(),
            _ => {}
        }
        write_checksums(&output);
        if matches!(fault, Fault::WrongHash) {
            fs::write(output.join("aihack.exe"), "tampered after checksum\n").unwrap();
        }
        if matches!(fault, Fault::DuplicateChecksum) {
            let sums = fs::read_to_string(output.join("SHA256SUMS")).unwrap();
            let first = sums.lines().next().unwrap();
            fs::write(output.join("SHA256SUMS"), format!("{sums}{first}\n")).unwrap();
        }
        if matches!(fault, Fault::ExtraFile) {
            fs::write(output.join("UNTRACKED-UNSIGNED-PAYLOAD.exe"), "unsigned\n").unwrap();
        }
        if matches!(fault, Fault::ExtraDirectory) {
            fs::create_dir_all(output.join("unexpected-directory")).unwrap();
        }
        if matches!(fault, Fault::ExtraJunction) {
            fs::create_dir_all(root.join("junction-target")).unwrap();
            let status = Command::new("cmd.exe")
                .args([
                    "/d",
                    "/c",
                    "mklink",
                    "/J",
                    output.join("unexpected-junction").to_str().unwrap(),
                    root.join("junction-target").to_str().unwrap(),
                ])
                .status()
                .unwrap();
            assert!(status.success());
        }
        Self {
            root,
            candidate_date: candidate_date.to_string(),
        }
    }

    fn verify(&self) -> std::process::Output {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                project_path("scripts/verify_release_bundle.ps1")
                    .to_str()
                    .unwrap(),
                "-OutputDir",
                self.root.join("output").to_str().unwrap(),
                "-ExpectedCommit",
                COMMIT,
                "-ExpectedCandidateDate",
                &self.candidate_date,
            ])
            .output()
            .unwrap()
    }

    fn staging(&self, mode: &str, stage: Option<&Path>) -> std::process::Output {
        let mut command = Command::new("powershell");
        command.args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            project_path("scripts/release_staging.ps1")
                .to_str()
                .unwrap(),
            "-Mode",
            mode,
            "-Root",
            self.root.to_str().unwrap(),
            "-OutputDir",
            "output",
        ]);
        if let Some(stage) = stage {
            command.arg("-Stage").arg(stage);
        }
        command.output().unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}

fn project_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn write_checksums(output: &Path) {
    let script = r#"function Hash([string]$path){$stream=[IO.File]::OpenRead($path);try{$sha=[Security.Cryptography.SHA256]::Create();try{return ([BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-','').ToLowerInvariant()}finally{$sha.Dispose()}}finally{$stream.Dispose()}}; $names=@('aihack.exe','aihack-headless.exe','LICENSE','NOTICE','MODIFICATIONS.md','PROJECT_OWNER_LICENSE_APPROVAL.md','RELEASE-METADATA','aihack-0.3.0-source.zip'); $lines=foreach($name in $names){(Hash (Join-Path $env:AIHACK_BUNDLE_OUTPUT $name))+'  '+$name}; Set-Content -Encoding Ascii (Join-Path $env:AIHACK_BUNDLE_OUTPUT 'SHA256SUMS') $lines"#;
    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .env("AIHACK_BUNDLE_OUTPUT", output)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn windows_verifier_accepts_complete_bundle_normal_similar_name_and_calendar_edges() {
    for fault in [
        Fault::None,
        Fault::SimilarLegacyName,
        Fault::MinimumYearCalendar,
        Fault::MaximumYearCalendar,
    ] {
        let fixture = Fixture::new(fault);
        let output = fixture.verify();
        assert!(
            output.status.success(),
            "positive fixture rejected: {fault:?}; stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn windows_verifier_rejects_legacy_metadata_record_hash_size_and_checksum_faults() {
    for fault in [
        Fault::IncludedLegacy,
        Fault::DuplicateMetadata,
        Fault::MismatchedRecord,
        Fault::WrongHash,
        Fault::EmptyArtifact,
        Fault::DuplicateChecksum,
        Fault::ExtraFile,
        Fault::ExtraDirectory,
        Fault::ExtraJunction,
        Fault::StaleModificationPeriod,
        Fault::DotLegacyAlias,
        Fault::RepeatedDotLegacyAlias,
        Fault::ParentLegacyAlias,
        Fault::AbsoluteLegacyAlias,
        Fault::BackslashLegacyAlias,
        Fault::UppercaseLegacyAlias,
        Fault::MixedCaseLegacyAlias,
        Fault::TrailingDotLegacyAlias,
        Fault::TrailingSpaceLegacyAlias,
        Fault::ReservedDeviceAlias,
        Fault::CaseCollisionAlias,
        Fault::InvalidCalendarPeriod,
        Fault::InvalidCalendarDay,
        Fault::InvalidLeapDay,
        Fault::ReverseCalendarPeriod,
        Fault::InvalidCandidateDate,
        Fault::YearZeroCalendar,
    ] {
        let fixture = Fixture::new(fault);
        let output = fixture.verify();
        assert!(
            !output.status.success(),
            "Windows verifier accepted negative fixture: {fault:?}"
        );
    }
}

#[test]
fn windows_verifier_rejects_an_output_root_junction() {
    let fixture = Fixture::new(Fault::None);
    let output = fixture.root.join("output");
    let target = fixture.root.join("redirected-output");
    fs::rename(&output, &target).unwrap();
    let status = Command::new("cmd.exe")
        .args([
            "/d",
            "/c",
            "mklink",
            "/J",
            output.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let result = fixture.verify();
    assert!(
        !result.status.success(),
        "Windows verifier must reject an output-root junction"
    );
}

#[test]
fn windows_release_staging_rejects_an_output_root_junction_before_writing() {
    let fixture = Fixture::new(Fault::None);
    let output = fixture.root.join("output");
    let target = fixture.root.join("redirected-output");
    fs::rename(&output, &target).unwrap();
    let marker = target.join("outside-marker");
    fs::write(&marker, "unchanged").unwrap();
    let status = Command::new("cmd.exe")
        .args([
            "/d",
            "/c",
            "mklink",
            "/J",
            output.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let result = fixture.staging("New", None);
    assert!(!result.status.success());
    assert_eq!(fs::read_to_string(marker).unwrap(), "unchanged");
}

#[test]
fn windows_release_staging_rejects_a_nested_output_junction_before_cleanup() {
    let fixture = Fixture::new(Fault::ExtraJunction);
    let marker = fixture.root.join("junction-target/outside-marker");
    fs::write(&marker, "unchanged").unwrap();

    let result = fixture.staging("New", None);
    assert!(!result.status.success());
    assert_eq!(fs::read_to_string(marker).unwrap(), "unchanged");
}

#[test]
fn windows_verifier_rejects_an_expected_name_hard_link_without_mutating_its_other_name() {
    let fixture = Fixture::new(Fault::None);
    let output = fixture.root.join("output");
    let victim = fixture.root.join("outside-victim.exe");
    let original = fs::read(output.join("aihack.exe")).unwrap();
    fs::write(&victim, &original).unwrap();
    fs::remove_file(output.join("aihack.exe")).unwrap();
    fs::hard_link(&victim, output.join("aihack.exe")).unwrap();
    write_checksums(&output);

    let result = fixture.verify();
    assert!(
        !result.status.success(),
        "Windows verifier must reject an expected-name hard link"
    );
    assert_eq!(fs::read(victim).unwrap(), original);
}

#[test]
fn windows_release_staging_promotes_a_fresh_directory_without_writing_a_preplaced_hard_link() {
    let fixture = Fixture::new(Fault::None);
    let output = fixture.root.join("output");
    let victim = fixture.root.join("outside-victim.exe");
    fs::write(&victim, "victim-must-not-change").unwrap();
    fs::remove_file(output.join("aihack.exe")).unwrap();
    fs::hard_link(&victim, output.join("aihack.exe")).unwrap();

    let created = fixture.staging("New", None);
    assert!(
        created.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&created.stderr)
    );
    let stage = PathBuf::from(String::from_utf8(created.stdout).unwrap().trim());
    fs::write(stage.join("aihack.exe"), "fresh-release-binary").unwrap();
    let promoted = fixture.staging("Promote", Some(&stage));
    assert!(
        promoted.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&promoted.stderr)
    );
    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        "victim-must-not-change"
    );
    assert_eq!(
        fs::read_to_string(output.join("aihack.exe")).unwrap(),
        "fresh-release-binary"
    );
}

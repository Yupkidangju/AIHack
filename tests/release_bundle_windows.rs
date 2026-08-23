#![cfg(windows)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

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
}

struct Fixture {
    root: PathBuf,
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
        let metadata = format!(
            "product=AIHack\nversion=0.3.0\ncommit={COMMIT}\nsource_license=NGPL\nmodification_notice=AIHACK-MODIFICATIONS-2026-08-23-02\nowner_approval=AIHACK-OWNER-2026-07-20-NGPL-01\n"
        );
        fs::write(source.join("RELEASE-METADATA"), &metadata).unwrap();
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
        Self { root }
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
            ])
            .output()
            .unwrap()
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
fn windows_verifier_accepts_the_complete_commit_bound_bundle() {
    let fixture = Fixture::new(Fault::None);
    let output = fixture.verify();
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
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
    ] {
        let fixture = Fixture::new(fault);
        let output = fixture.verify();
        assert!(
            !output.status.success(),
            "Windows verifier accepted negative fixture: {fault:?}"
        );
    }
}

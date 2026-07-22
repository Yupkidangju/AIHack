use std::{fs, path::Path, process::Command};

const OFFICIAL_LICENSE_SHA256: &str =
    "93a3ae2cb8dee482daddfaebe53bcffe5b114b603def19b4dca21621cbc5a747";

fn project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path} 읽기 실패: {error}"))
}

#[test]
fn workspace_declares_ngpl_v030_without_crates_io_publication() {
    for manifest in [
        "Cargo.toml",
        "crates/aihack-core/Cargo.toml",
        "crates/aihack-content/Cargo.toml",
        "crates/aihack-ai-contract/Cargo.toml",
        "crates/aihack-llm/Cargo.toml",
        "crates/aihack-runtime/Cargo.toml",
        "apps/aihack-tui/Cargo.toml",
        "apps/aihack-headless/Cargo.toml",
    ] {
        let content = project_file(manifest);
        assert!(content.contains("version = \"0.3.0\""), "{manifest}");
        assert!(content.contains("license = \"NGPL\""), "{manifest}");
        assert!(content.contains("publish = false"), "{manifest}");
        assert!(!content.contains("UNLICENSED"), "{manifest}");
        for dependency in content.lines().filter(|line| line.contains("{ path =")) {
            assert!(
                dependency.contains("version = \"0.3.0\""),
                "{manifest}: {dependency}"
            );
        }
    }
}

#[test]
fn root_uses_the_verified_official_ngpl_text_and_derivative_notice() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("sha256sum")
        .arg(root.join("LICENSE"))
        .output()
        .expect("sha256sum 실행 실패");
    assert!(output.status.success(), "LICENSE checksum 계산 실패");
    assert!(
        String::from_utf8_lossy(&output.stdout).starts_with(OFFICIAL_LICENSE_SHA256),
        "공식 NetHack 3.6.7 dat/license 원문과 일치해야 한다"
    );

    let notice = project_file("NOTICE");
    for phrase in [
        "NetHack 3.6.7",
        "derivative reimplementation",
        "AI-assisted semantic rewrite",
        "complete corresponding source",
        "AIHack contributors",
        "MODIFICATIONS.md",
        "PROJECT_OWNER_LICENSE_APPROVAL.md",
        "RELEASE-METADATA",
    ] {
        assert!(notice.contains(phrase), "NOTICE 필수 문구 누락: {phrase}");
    }
    assert!(!notice.contains("distributed Git history"));
}

#[test]
fn owner_approval_and_bundle_carried_modification_evidence_are_traceable() {
    let approval = project_file("PROJECT_OWNER_LICENSE_APPROVAL.md");
    for phrase in [
        "AIHACK-OWNER-2026-07-20-NGPL-01",
        "Project owner",
        "whole-work NGPL",
        "PROV-0001..PROV-0012",
        "NH367-C001..NH367-C010",
        "direct user instruction",
        "qualified legal opinion: not claimed",
    ] {
        assert!(approval.contains(phrase), "approval record 누락: {phrase}");
    }

    let modifications = project_file("MODIFICATIONS.md");
    for phrase in [
        "2025-05-20..2026-07-20",
        "src/**",
        "crates/**",
        "apps/**",
        "tests/**",
        "scripts/**",
        "does not depend on distributed Git history",
    ] {
        assert!(
            modifications.contains(phrase),
            "modification manifest 누락: {phrase}"
        );
    }

    let metadata = project_file("RELEASE-METADATA");
    assert!(metadata.contains("version=0.3.0"));
    assert!(metadata.contains("commit=$Format:%H$"));
    assert!(metadata.contains("owner_approval=AIHACK-OWNER-2026-07-20-NGPL-01"));
    assert!(metadata.contains("modification_notice=AIHACK-MODIFICATIONS-2026-07-20-01"));
}

#[test]
fn project_owner_approved_ngpl_provenance_and_source_review_records() {
    let provenance = project_file("PROVENANCE.md");
    let prov_0004 = provenance
        .lines()
        .find(|line| line.starts_with("| PROV-0004 |"))
        .expect("PROV-0004 record 누락");
    for field in [
        "| Approved | yes | Project owner | 2026-07-20 | `NGPL` |",
        "| true | true |",
        "AI-assisted semantic rewrite",
        "AIHACK-OWNER-2026-07-20-NGPL-01",
    ] {
        assert!(
            prov_0004.contains(field),
            "PROV-0004 승인 근거 누락: {field}"
        );
    }
    assert!(provenance.contains("외부 배포 판정: APPROVED"));

    let compatibility = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/compatibility");
    let mut approved = 0;
    for entry in fs::read_dir(compatibility).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("NH367-C") || !name.ends_with(".md") {
            continue;
        }
        let record = fs::read_to_string(&path).unwrap();
        for field in [
            "reference_seen: true",
            "provenance_status: Approved",
            "approval_reviewer: Project owner",
            "approval_reviewed_at: 2026-07-20",
            "license_id: NGPL",
            "notice_required: true",
            "modification_notice_required: true",
            "evidence: Project owner derivative classification",
            "AIHACK-OWNER-2026-07-20-NGPL-01",
        ] {
            assert!(record.contains(field), "{name} 승인 근거 누락: {field}");
        }
        approved += 1;
    }
    assert_eq!(approved, 10);
}

#[test]
fn release_packaging_includes_license_notice_and_complete_source() {
    let linux = project_file("build.sh");
    let windows = project_file("build.bat");
    let verifier = project_file("scripts/verify_release_bundle.sh");
    let attributes = project_file(".gitattributes");

    for required in [
        "LICENSE",
        "NOTICE",
        "MODIFICATIONS.md",
        "PROJECT_OWNER_LICENSE_APPROVAL.md",
        "RELEASE-METADATA",
        "SHA256SUMS",
        "git status --porcelain",
        "git archive",
        "aihack-0.3.0-source",
    ] {
        assert!(linux.contains(required), "build.sh 누락: {required}");
        assert!(windows.contains(required), "build.bat 누락: {required}");
    }
    assert!(attributes.contains("legacy_nethack_port_reference export-ignore"));
    assert!(attributes.contains("target export-ignore"));
    assert!(attributes.contains("output export-ignore"));
    assert!(attributes.contains("RELEASE-METADATA export-subst"));
    assert!(linux.contains("verify_release_bundle.sh"));
    for reference in [
        "owner_approval=AIHACK-OWNER-2026-07-20-NGPL-01",
        "modification_notice=AIHACK-MODIFICATIONS-2026-07-20-01",
    ] {
        assert!(linux.contains(reference), "build.sh 누락: {reference}");
        assert!(windows.contains(reference), "build.bat 누락: {reference}");
    }
    for document_id in ["Approval ID:", "Notice ID:"] {
        assert!(
            verifier.contains(document_id),
            "verifier 누락: {document_id}"
        );
        assert!(
            windows.contains(document_id),
            "build.bat 누락: {document_id}"
        );
    }
    assert!(
        windows.contains("$matches.Count -ne 1"),
        "build.bat은 metadata key가 정확히 한 번만 나타나는지 검사해야 합니다"
    );
}

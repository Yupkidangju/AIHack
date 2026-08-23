use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

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

fn compute_sha256(data: &[u8]) -> String {
    // sha256sum을 우선 사용하고, 사용할 수 없으면 자체 SHA-256 구현으로 검증한다.
    if let Ok(mut child) = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            if stdin.write_all(data).is_ok() {
                drop(stdin);
                if let Ok(output) = child.wait_with_output() {
                    if output.status.success() {
                        let text = String::from_utf8_lossy(&output.stdout);
                        if let Some(hash) = text.split_whitespace().next() {
                            return hash.to_string();
                        }
                    }
                }
            }
        }
    }

    // 외부 명령이 없는 환경에서도 같은 FIPS 180-4 SHA-256 결과를 계산한다.
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_val = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_val
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_val = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_val);
    }

    h.iter()
        .map(|val| format!("{:08x}", val))
        .collect::<Vec<_>>()
        .join("")
}

#[test]
fn root_uses_the_verified_official_ngpl_text_and_derivative_notice() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let license = fs::read(root.join("LICENSE")).expect("LICENSE 읽기 실패");
    let normalized = license
        .into_iter()
        .filter(|byte| *byte != b'\r')
        .collect::<Vec<_>>();
    let actual_hash = compute_sha256(&normalized);
    assert_eq!(
        actual_hash, OFFICIAL_LICENSE_SHA256,
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
        "2025-05-20..2026-08-23",
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
    assert!(metadata.contains("modification_notice=AIHACK-MODIFICATIONS-2026-08-23-02"));
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
    let windows_verifier = project_file("scripts/verify_release_bundle.ps1");
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
    assert!(
        attributes.lines().any(|line| line == "LICENSE text eol=lf"),
        "Windows checkout에서도 공식 LICENSE 바이트를 보존해야 한다"
    );
    assert!(
        attributes.lines().any(|line| line == "*.sh text eol=lf"),
        "Windows checkout에서도 Bash 검증 스크립트의 LF를 보존해야 한다"
    );
    assert!(linux.contains("verify_release_bundle.sh"));
    assert!(windows.contains("verify_release_bundle.ps1"));
    for contract in [
        "release source archive contains an excluded path",
        "metadata mismatch or duplicate key",
        "SHA256SUMS record count mismatch",
        "duplicate SHA256SUMS record",
        "release artifact is empty",
    ] {
        assert!(
            windows_verifier.contains(contract),
            "Windows verifier 누락: {contract}"
        );
    }
    assert!(
        windows.contains("git show HEAD:LICENSE"),
        "Windows binary bundle도 Git blob의 공식 LICENSE 바이트를 포함해야 한다"
    );
    for reference in [
        "owner_approval=AIHACK-OWNER-2026-07-20-NGPL-01",
        "modification_notice=AIHACK-MODIFICATIONS-2026-08-23-02",
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

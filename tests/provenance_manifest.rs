use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

const OFFICIAL_ARCHIVE_SHA256: &str =
    "98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2";
const OFFICIAL_LICENSE_SHA256: &str =
    "93a3ae2cb8dee482daddfaebe53bcffe5b114b603def19b4dca21621cbc5a747";
static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);

fn project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path} 읽기 실패: {error}"))
}

struct CheckpointFixture {
    root: PathBuf,
}

impl CheckpointFixture {
    fn complete_approved() -> Self {
        let root = std::env::temp_dir().join(format!(
            "aihack-r7-checkpoint-{}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(root.join("scripts")).unwrap();
        fs::create_dir_all(root.join("docs/compatibility")).unwrap();
        for directory in [
            "src",
            "crates/aihack-content/src/data/levels",
            "apps",
            "tests",
            "docs/provenance",
        ] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }

        fs::write(root.join("PROVENANCE.md"), project_file("PROVENANCE.md")).unwrap();
        fs::write(root.join("Cargo.toml"), project_file("Cargo.toml")).unwrap();
        fs::write(root.join("Cargo.lock"), project_file("Cargo.lock")).unwrap();
        fs::write(
            root.join("tests/nethack_367_compat.rs"),
            project_file("tests/nethack_367_compat.rs"),
        )
        .unwrap();
        fs::write(
            root.join("docs/provenance/r7-content.sha256"),
            project_file("docs/provenance/r7-content.sha256"),
        )
        .unwrap();
        for path in [
            "crates/aihack-content/src/data/items.toml",
            "crates/aihack-content/src/data/monsters.toml",
            "crates/aihack-content/src/data/levels/main_1.toml",
            "crates/aihack-content/src/data/levels/main_2.toml",
        ] {
            fs::write(root.join(path), project_file(path)).unwrap();
        }

        let compatibility = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/compatibility");
        for entry in fs::read_dir(compatibility).unwrap() {
            let path = entry.unwrap().path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !name.starts_with("NH367-C") || !name.ends_with(".md") {
                continue;
            }
            let record = fs::read_to_string(&path).unwrap();
            fs::write(root.join("docs/compatibility").join(name), record).unwrap();
        }

        let script = root.join("scripts/r7_checkpoint.sh");
        fs::write(&script, project_file("scripts/r7_checkpoint.sh")).unwrap();

        Self { root }
    }

    fn status_only_approved() -> Self {
        let fixture = Self::complete_approved();
        fixture.replace(
            "PROVENANCE.md",
            "| `NGPL` | whole AIHack derivative distribution | true | true | Project owner derivative classification; `AIHACK-OWNER-2026-07-20-NGPL-01`; AI-assisted semantic rewrite from NetHack 3.6.7 source |",
            "| pending | distribution scope unresolved | pending | pending | project owner or qualified reviewer approval required |",
        );
        for entry in fs::read_dir(fixture.root.join("docs/compatibility")).unwrap() {
            let path = entry.unwrap().path();
            let relative = path.strip_prefix(&fixture.root).unwrap().to_str().unwrap();
            fixture.replace(
                relative,
                "approval_reviewer: Project owner",
                "approval_reviewer: \"\"",
            );
            fixture.replace(
                relative,
                "approval_reviewed_at: 2026-07-20",
                "approval_reviewed_at: \"\"",
            );
            fixture.replace(relative, "license_id: NGPL", "license_id: pending");
            fixture.replace(
                relative,
                "license_scope: whole AIHack derivative distribution",
                "license_scope: pending",
            );
            fixture.replace(
                relative,
                "\n  notice_required: true\n",
                "\n  notice_required: pending\n",
            );
            fixture.replace(
                relative,
                "\n  modification_notice_required: true\n",
                "\n  modification_notice_required: pending\n",
            );
            fixture.replace(
                relative,
                "evidence: Project owner derivative classification; AIHACK-OWNER-2026-07-20-NGPL-01; AI-assisted semantic rewrite from NetHack 3.6.7 source",
                "evidence: \"\"",
            );
        }
        fixture
    }

    fn approved_with_evidence() -> Self {
        Self::complete_approved()
    }

    fn replace(&self, path: &str, from: &str, to: &str) {
        let path = self.root.join(path);
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains(from),
            "fixture replacement source missing: {from}"
        );
        fs::write(path, content.replace(from, to)).unwrap();
    }

    fn run(&self) -> std::process::Output {
        Command::new("bash")
            .arg(self.root.join("scripts/r7_checkpoint.sh"))
            .env_remove("AIHACK_R7_ROOT")
            .output()
            .unwrap()
    }

    fn run_with_root_override(&self, root_override: &str) -> std::process::Output {
        Command::new("bash")
            .arg(self.root.join("scripts/r7_checkpoint.sh"))
            .env("AIHACK_R7_ROOT", root_override)
            .output()
            .unwrap()
    }
}

impl Drop for CheckpointFixture {
    fn drop(&mut self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}

fn inventory_rows(document: &str) -> Vec<Vec<&str>> {
    let section = document
        .split("<!-- runtime-inventory:start -->")
        .nth(1)
        .and_then(|rest| rest.split("<!-- runtime-inventory:end -->").next())
        .expect("runtime inventory marker가 필요하다");

    section
        .lines()
        .filter(|line| line.starts_with('|'))
        .map(|line| {
            line.trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>()
        })
        .filter(|columns| {
            columns.len() == 13 && columns[0] != "ID" && !columns[0].starts_with("---")
        })
        .collect()
}

#[test]
fn r7_runtime_inventory_has_no_unknown_or_blocked_included_asset() {
    let provenance = project_file("PROVENANCE.md");
    let rows = inventory_rows(&provenance);

    assert!(rows.len() >= 8, "runtime/격리 자산 record가 충분하지 않다");
    for row in rows.iter().filter(|row| row[5] == "yes") {
        assert!(
            matches!(row[4], "Reviewed" | "Approved"),
            "runtime included {} 상태가 {}다",
            row[1],
            row[4]
        );
        assert_ne!(row[8], "", "runtime included {} license id 누락", row[1]);
        assert_ne!(row[9], "", "runtime included {} license scope 누락", row[1]);
        assert_ne!(
            row[11], "",
            "runtime included {} modification notice 누락",
            row[1]
        );
        assert_ne!(row[12], "", "runtime included {} evidence 누락", row[1]);
    }
}

#[test]
fn r7_records_verified_official_and_damaged_local_license_checksums() {
    let provenance = project_file("PROVENANCE.md");

    assert!(provenance.contains(OFFICIAL_ARCHIVE_SHA256));
    assert!(provenance.contains(OFFICIAL_LICENSE_SHA256));
    assert!(provenance.contains("5e3e7c0cd3be7f65f4d9b59b49820c303abfa92c95497c5eb8cff2b64e456bdf"));
    assert!(provenance.contains("33..35"));
}

#[test]
fn release_checkpoints_use_ci_portable_text_search_tools() {
    for script in ["scripts/r7_checkpoint.sh", "scripts/r8_checkpoint.sh"] {
        let content = project_file(script);
        assert!(
            !content.contains("rg "),
            "{script}가 GitHub runner 기본 이미지에 없는 rg에 의존한다"
        );
    }
}

fn assert_no_legacy_reference(path: &Path) {
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            assert_no_legacy_reference(&entry.unwrap().path());
        }
        return;
    }
    if !matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rs" | "toml")
    ) {
        return;
    }
    let content = fs::read_to_string(path).unwrap();
    assert!(
        !content.contains("legacy_nethack_port_reference"),
        "runtime source가 legacy path를 참조한다: {}",
        path.display()
    );
}

#[test]
fn r7_runtime_sources_do_not_import_the_blocked_legacy_tree() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for path in ["Cargo.toml", "src", "crates", "apps"] {
        assert_no_legacy_reference(&root.join(path));
    }
}

#[test]
fn r8_external_distribution_uses_ngpl_but_remains_subject_to_technical_audit() {
    let provenance = project_file("PROVENANCE.md");
    let manifest = project_file("Cargo.toml");

    assert!(manifest.contains("license = \"NGPL\""));
    assert!(manifest.contains("publish = false"));
    assert!(provenance.contains(
        "외부 배포 판정: APPROVED FOR NGPL-COMPLIANT PACKAGING — R8 technical audit pending"
    ));
}

#[test]
fn r7_checkpoint_rejects_status_only_approval_without_required_evidence() {
    let fixture = CheckpointFixture::status_only_approved();

    let output = fixture.run();

    assert!(
        !output.status.success(),
        "status 문자열만 Approved인 fixture가 checkpoint를 통과했다: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PROV-0004 Approved license_id missing"));
    assert!(stdout.contains("NH367-C001 Approved reviewer missing"));
}

#[test]
fn r7_checkpoint_accepts_complete_approval_fixture_with_unlicensed_root() {
    let fixture = CheckpointFixture::approved_with_evidence();
    fixture.replace(
        "Cargo.toml",
        "license = \"NGPL\"",
        "license = \"UNLICENSED\"",
    );

    let output = fixture.run();

    assert!(
        output.status.success(),
        "complete approval fixture 실패: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn r7_checkpoint_ignores_inherited_root_override() {
    let fixture = CheckpointFixture::approved_with_evidence();

    let output = fixture.run_with_root_override("/definitely/not/aihack");

    assert!(
        output.status.success(),
        "caller-selected root가 checkpoint를 바꿨다: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn r7_checkpoint_rejects_each_missing_scenario_approval_field() {
    for (field, valid, invalid, expected) in [
        (
            "reviewer",
            "approval_reviewer: Project owner",
            "approval_reviewer: \"\"",
            "Approved reviewer missing",
        ),
        (
            "reviewed_at",
            "approval_reviewed_at: 2026-07-20",
            "approval_reviewed_at: \"\"",
            "Approved reviewed_at invalid",
        ),
        (
            "license_id",
            "license_id: NGPL",
            "license_id: pending",
            "Approved license_id missing",
        ),
        (
            "license_scope",
            "license_scope: whole AIHack derivative distribution",
            "license_scope: pending",
            "Approved license_scope missing",
        ),
        (
            "notice",
            "\n  notice_required: true\n",
            "\n  notice_required: pending\n",
            "Approved notice_required invalid",
        ),
        (
            "modification_notice",
            "\n  modification_notice_required: true\n",
            "\n  modification_notice_required: pending\n",
            "Approved modification_notice_required invalid",
        ),
        (
            "evidence",
            "evidence: Project owner derivative classification; AIHACK-OWNER-2026-07-20-NGPL-01; AI-assisted semantic rewrite from NetHack 3.6.7 source",
            "evidence: \"\"",
            "Approved evidence missing",
        ),
    ] {
        let fixture = CheckpointFixture::approved_with_evidence();
        fixture.replace(
            "docs/compatibility/NH367-C001-wall-movement.md",
            valid,
            invalid,
        );

        let output = fixture.run();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!output.status.success(), "{field} 누락 fixture가 통과했다");
        assert!(
            stdout.contains(expected),
            "{field} 누락 사유가 없다: {stdout}"
        );
    }
}

#[test]
fn r7_checkpoint_rejects_checksum_drift_and_duplicate_scenario_id() {
    let checksum_fixture = CheckpointFixture::approved_with_evidence();
    checksum_fixture.replace(
        "crates/aihack-content/src/data/items.toml",
        "[[item]]",
        "# checksum drift\n[[item]]",
    );
    let checksum_output = checksum_fixture.run();
    assert!(!checksum_output.status.success());
    assert!(String::from_utf8_lossy(&checksum_output.stdout)
        .contains("runtime content checksum mismatch"));

    let coverage_fixture = CheckpointFixture::approved_with_evidence();
    let manifest = fs::read_to_string(
        coverage_fixture
            .root
            .join("docs/provenance/r7-content.sha256"),
    )
    .unwrap();
    let shortened_manifest = manifest.lines().skip(1).collect::<Vec<_>>().join("\n") + "\n";
    fs::write(
        coverage_fixture
            .root
            .join("docs/provenance/r7-content.sha256"),
        shortened_manifest,
    )
    .unwrap();
    let coverage_output = coverage_fixture.run();
    assert!(!coverage_output.status.success());
    assert!(String::from_utf8_lossy(&coverage_output.stdout)
        .contains("runtime content checksum coverage incomplete"));

    let duplicate_fixture = CheckpointFixture::approved_with_evidence();
    duplicate_fixture.replace(
        "docs/compatibility/NH367-C002-closed-door.md",
        "id: NH367-C002",
        "id: NH367-C001",
    );
    let duplicate_output = duplicate_fixture.run();
    assert!(!duplicate_output.status.success());
    assert!(String::from_utf8_lossy(&duplicate_output.stdout)
        .contains("duplicate scenario id: NH367-C001"));
}

#[test]
fn r7_checkpoint_rejects_invalid_trace_schema_and_function_link() {
    let locator_fixture = CheckpointFixture::approved_with_evidence();
    locator_fixture.replace(
        "docs/compatibility/NH367-C001-wall-movement.md",
        "locator: src/hack.c:test_move@713,domove@1352",
        "locator:",
    );
    let locator_output = locator_fixture.run();
    assert!(!locator_output.status.success());
    assert!(String::from_utf8_lossy(&locator_output.stdout).contains("NH367-C001 locator missing"));

    let function_fixture = CheckpointFixture::approved_with_evidence();
    function_fixture.replace(
        "docs/compatibility/NH367-C001-wall-movement.md",
        "function: nh367_c001_wall_movement_preserves_position_turn_and_hash",
        "function: missing_test_function",
    );
    let function_output = function_fixture.run();
    assert!(!function_output.status.success());
    assert!(String::from_utf8_lossy(&function_output.stdout)
        .contains("NH367-C001 test function link invalid"));
}

#[test]
fn r7_checkpoint_rejects_missing_or_ambiguous_runtime_coverage_and_blocked_include() {
    let missing_fixture = CheckpointFixture::approved_with_evidence();
    fs::write(missing_fixture.root.join("src/probe.rs"), "fn probe() {}\n").unwrap();
    missing_fixture.replace("PROVENANCE.md", "`src/**`", "`source/**`");
    let missing_output = missing_fixture.run();
    assert!(!missing_output.status.success());
    assert!(String::from_utf8_lossy(&missing_output.stdout)
        .contains("runtime coverage must resolve once: src/probe.rs (0)"));

    let ambiguous_fixture = CheckpointFixture::approved_with_evidence();
    fs::write(
        ambiguous_fixture.root.join("src/probe.rs"),
        "fn probe() {}\n",
    )
    .unwrap();
    ambiguous_fixture.replace("PROVENANCE.md", "`apps/**`", "`src/**`");
    let ambiguous_output = ambiguous_fixture.run();
    assert!(!ambiguous_output.status.success());
    assert!(String::from_utf8_lossy(&ambiguous_output.stdout)
        .contains("runtime coverage must resolve once: src/probe.rs (2)"));

    let blocked_fixture = CheckpointFixture::approved_with_evidence();
    fs::write(
        blocked_fixture.root.join("src/probe.rs"),
        "const BAD: &str = \"legacy_nethack_port_reference/src\";\n",
    )
    .unwrap();
    let blocked_output = blocked_fixture.run();
    assert!(!blocked_output.status.success());
    assert!(String::from_utf8_lossy(&blocked_output.stdout)
        .contains("Blocked/Unknown runtime reference found"));
}

#[test]
fn r7_checkpoint_rejects_abbreviated_scenario_checksum() {
    let fixture = CheckpointFixture::approved_with_evidence();
    fixture.replace(
        "docs/compatibility/NH367-C001-wall-movement.md",
        OFFICIAL_ARCHIVE_SHA256,
        "98cf67df...aacb2",
    );

    let output = fixture.run();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("NH367-C001 archive checksum invalid"));
}

use std::{fs, path::Path};

fn project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path} 읽기 실패: {error}"))
}

#[test]
fn r8_design_and_decision_docs_describe_the_ngpl_release_boundary() {
    let designs = project_file("designs.md");
    for phrase in [
        "현재 v0.3.0 구현",
        "R8 release contract",
        "## 14. R8 배포 표현 경계",
        "NGPL",
        "complete corresponding source",
    ] {
        assert!(designs.contains(phrase), "designs.md 누락: {phrase}");
    }

    let decisions = project_file("DESIGN_DECISIONS.md");
    assert!(decisions.contains("## ADR-0030:"));
    assert!(decisions.contains("whole-work NGPL"));

    let compatibility = project_file("docs/compatibility/README.md");
    assert!(compatibility.contains("project-owner NGPL approval recorded"));
    assert!(compatibility.contains("independent R8 audit pending"));
}

#[test]
fn compatibility_index_matches_all_ten_approved_records() {
    let index = project_file("docs/compatibility/README.md");
    let indexed_approved = index
        .lines()
        .filter(|line| line.starts_with("| NH367-C") && line.ends_with("| Approved |"))
        .count();
    assert_eq!(indexed_approved, 10, "compatibility index Approved count");

    for number in 1..=10 {
        let id = format!("NH367-C{number:03}");
        let record_name = index
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .and_then(|line| line.split('|').nth(3))
            .map(str::trim)
            .map(|name| name.trim_matches('`'))
            .unwrap_or_else(|| panic!("{id} index row 누락"));
        let record = project_file(&format!("docs/compatibility/{record_name}"));
        assert!(record.contains("provenance_status: Approved"), "{id}");
    }
}

#[test]
fn r8_documentation_self_check_is_current_without_rewriting_the_r0_audit() {
    let report = project_file("DOCUMENTATION_AUDIT_REPORT.md");
    assert!(report.contains("**R0 Documentation: PASS**"));
    assert!(report.contains("## 10. R8 문서 동기화 self-check (2026-07-20)"));
    assert!(report.contains("**R8 Documentation Self-check: PASS**"));
    assert!(report.contains("독립 R8 감사 pending"));

    for phrase in [
        "Cargo/README/CHANGELOG 0.3.0",
        "ADR-0030",
        "official LICENSE checksum",
        "complete corresponding source",
        "scripts/r8_checkpoint.sh",
        "scripts/r8_tui_core_flow.sh",
        "AI 구현 문서 표준 12개 항목",
    ] {
        assert!(
            report.contains(phrase),
            "R8 문서 self-check 근거 누락: {phrase}"
        );
    }

    let roadmap = project_file("audit_roadmap.md");
    for command in [
        "scripts/r8_tui_core_flow.sh",
        "scripts/r6_pty_matrix.sh",
        "scripts/r6_pending_exit_smoke.sh",
    ] {
        assert!(
            roadmap.contains(command),
            "R8 수동 검증 명령 누락: {command}"
        );
    }
}

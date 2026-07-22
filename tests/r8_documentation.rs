use std::{fs, path::Path};

fn project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path} 읽기 실패: {error}"))
}

fn markdown_section<'a>(document: &'a str, heading: &str, next_heading: &str) -> &'a str {
    let start = document
        .find(heading)
        .unwrap_or_else(|| panic!("section 시작 누락: {heading}"));
    let remainder = &document[start..];
    let end = remainder
        .find(next_heading)
        .unwrap_or_else(|| panic!("section 종료 누락: {next_heading}"));
    &remainder[..end]
}

fn gap_row<'a>(roadmap: &'a str, gap_id: &str) -> &'a str {
    roadmap
        .lines()
        .find(|line| line.starts_with(&format!("| {gap_id} |")))
        .unwrap_or_else(|| panic!("gap row 누락: {gap_id}"))
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
    assert!(compatibility
        .contains("report 20 active-state/false-green HOLD remediation pending re-audit"));
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
    assert!(report.contains("report 20: report 19 evidence verified"));

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

#[test]
fn active_r8_status_docs_share_the_same_audited_ci_and_hold_boundary() {
    let evidence = [
        "b9bd680200d82b20d7c9ba961a2758caa3d49e16",
        "29886410221",
        "https://github.com/Yupkidangju/AIHack/actions/runs/29886410221",
        "ubuntu-latest quality gate",
        "windows-latest quality gate",
        "2026-07-22",
        "audit_report_19.md",
        "audit_report_20.md",
    ];
    for document in [
        "README.md",
        "IMPLEMENTATION_SUMMARY.md",
        "audit_roadmap.md",
        "GAP_CLOSURE_ROADMAP.md",
        "BUILD_GUIDE.md",
        "DOCUMENTATION_AUDIT_REPORT.md",
        "DESIGN_DECISIONS.md",
    ] {
        let content = project_file(document);
        for phrase in evidence {
            assert!(
                content.contains(phrase),
                "{document} evidence 누락: {phrase}"
            );
        }
    }

    let summary = project_file("IMPLEMENTATION_SUMMARY.md");
    assert!(summary.contains("SC-BUILD-02 PASS"));
    assert!(summary.contains("report 20 active-state/false-green HOLD 시정 후 독립 재감사 대기"));

    let guide = project_file("BUILD_GUIDE.md");
    assert!(guide.contains("`audit_report_19.md`의 technical evidence는 Verified"));
    assert!(guide.contains("`audit_report_20.md`의 잔여 active-state/false-green HOLD"));
    assert!(!guide.contains("| CI | Linux/Windows workflow 구성, 원격 green 대기 |"));
}

#[test]
fn active_release_sections_reject_known_stale_statuses() {
    let summary = project_file("IMPLEMENTATION_SUMMARY.md");
    let baseline = markdown_section(&summary, "## 1. 현재 기준과 목표", "## 2. 전체 런타임 흐름");
    assert!(baseline.contains("R1~R7 engineering gate와 SC-BUILD-02를 완료했다"));
    assert!(baseline.contains("report 20 문서 HOLD 시정의 독립 재감사"));
    for stale in [
        "다음 release 범위는 아직 완료되지 않았다",
        "- Linux/Windows 원격 CI evidence",
        "- R6 독립 감사",
        "- NetHack 출처·호환성 trace",
    ] {
        assert!(
            !baseline.contains(stale),
            "summary stale 상태 잔존: {stale}"
        );
    }

    let gaps = project_file("GAP_CLOSURE_ROADMAP.md");
    let license = gap_row(&gaps, "G-LICENSE-001");
    assert!(license.contains("audit_report_19.md"));
    assert!(license.contains("b9bd680200d82b20d7c9ba961a2758caa3d49e16"));
    assert!(license.contains("29886410221"));
    assert!(license.ends_with("| Closed |"));

    let build = gap_row(&gaps, "G-BUILD-004");
    assert!(build.ends_with("| Closed |"));

    let documentation = gap_row(&gaps, "G-DOC-001");
    assert!(documentation.contains("audit_report_20.md"));
    assert!(documentation.ends_with("| Implemented / report 20 re-audit pending |"));

    let guide = project_file("BUILD_GUIDE.md");
    let workspace_test = guide
        .lines()
        .find(|line| {
            line.starts_with("- [x]")
                && line.contains("cargo test --workspace --all-targets --locked`")
        })
        .expect("BUILD_GUIDE workspace test checklist 누락");
    assert!(workspace_test.contains("전체 PASS"));
    assert!(!workspace_test.contains("342 tests"));
    assert!(!workspace_test.contains("343 tests"));
}

use std::{
    fs,
    path::{Path, PathBuf},
};

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
    assert!(compatibility.contains("NH367-C001..C010 engineering/provenance closed"));
    assert!(compatibility.contains("report 27 predecessor `ea7822a5/32683076204`"));
    assert!(compatibility.contains("audit report 28 remediation in progress"));
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
    assert!(report.contains("audit_report_21.md` 종결 상태"));
    assert!(report.contains("audit_report_23.md` 역사적 상태"));
    assert!(report.contains("audit_report_26.md` 역사적 최종 상태"));
    assert!(report.contains("audit_report_27.md` 역사적 predecessor 상태"));
    assert!(report.contains("audit_report_28.md` 현재 상태"));

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
    let evidence = ["2519bc8e0ede81c39f46b5778e62a41d4ca66901", "32107862171"];
    for document in [
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
        for phrase in ["ea7822a5", "32683076204"] {
            assert!(
                content.contains(phrase),
                "{document} report 27 evidence 누락: {phrase}"
            );
        }
    }

    for document in [
        "IMPLEMENTATION_SUMMARY.md",
        "audit_roadmap.md",
        "GAP_CLOSURE_ROADMAP.md",
        "BUILD_GUIDE.md",
        "DOCUMENTATION_AUDIT_REPORT.md",
        "DESIGN_DECISIONS.md",
    ] {
        let content = project_file(document);
        for phrase in ["b732c42d62f295f4d8be64480c1d0a5a440fe738", "32650404618"] {
            assert!(
                content.contains(phrase) || (phrase.len() == 40 && content.contains("b732c42d")),
                "{document} report 25 evidence 누락: {phrase}"
            );
        }
    }

    for document in [
        "README.md",
        "IMPLEMENTATION_SUMMARY.md",
        "audit_roadmap.md",
        "BUILD_GUIDE.md",
        "DOCUMENTATION_AUDIT_REPORT.md",
    ] {
        let content = project_file(document);
        assert!(
            content.contains("docs/audit/audit_report_28.md"),
            "{document} current authority 누락"
        );
    }

    let summary = project_file("IMPLEMENTATION_SUMMARY.md");
    assert!(summary.contains("report 23/24 finding"));
    assert!(summary.contains("현재 권위는 report 27 구현"));
    assert!(summary.contains("audit_report_28.md"));

    let guide = project_file("BUILD_GUIDE.md");
    assert!(guide.contains("report 20 문서 시정 독립 재감사 PASS"));
    assert!(guide.contains("docs/audit/audit_report_28.md` allocator/registry"));
    assert!(!guide.contains("| CI | Linux/Windows workflow 구성, 원격 green 대기 |"));
}

#[test]
fn active_release_sections_reject_known_stale_statuses() {
    let summary = project_file("IMPLEMENTATION_SUMMARY.md");
    let baseline = markdown_section(&summary, "## 1. 현재 기준과 목표", "## 2. 전체 런타임 흐름");
    assert!(baseline.contains("report 23/24 finding"));
    assert!(baseline.contains("audit_report_28.md"));
    assert!(baseline.contains("partial evidence"));
    for stale in [
        "다음 release 범위는 아직 완료되지 않았다",
        "- Linux/Windows 원격 CI evidence",
        "- R6 독립 감사",
        "- NetHack 출처·호환성 trace",
        "report 20 문서 HOLD 시정의 독립 재감사",
        "report 20 active-state/false-green HOLD 시정 후 독립 재감사 대기",
    ] {
        assert!(
            !baseline.contains(stale),
            "summary stale 상태 잔존: {stale}"
        );
    }

    let gaps = project_file("GAP_CLOSURE_ROADMAP.md");
    let license = gap_row(&gaps, "G-LICENSE-001");
    assert!(license.contains("audit_report_21.md"));
    assert!(license.contains("2519bc8e0ede81c39f46b5778e62a41d4ca66901"));
    assert!(license.contains("32107862171"));
    assert!(license.ends_with("| Closed |"));

    let build = gap_row(&gaps, "G-BUILD-004");
    assert!(build.contains("2519bc8e0ede81c39f46b5778e62a41d4ca66901"));
    assert!(build.contains("32107862171"));
    assert!(build.ends_with("| Closed |"));

    let documentation = gap_row(&gaps, "G-DOC-001");
    assert!(documentation.contains("audit_report_21.md"));
    assert!(documentation.ends_with("| Closed |"));
    for verified in ["G-BUILD-006", "G-TEST-003", "G-DOC-004", "G-SEC-001"] {
        assert!(
            gap_row(&gaps, verified).ends_with("| Verified |"),
            "{verified} 현재 상태 불일치"
        );
    }
    for in_progress in ["G-CORE-004", "G-DOC-006", "G-SEC-003", "G-UI-002"] {
        assert!(
            gap_row(&gaps, in_progress).ends_with("| In Progress |"),
            "{in_progress} 현재 상태 불일치"
        );
    }
    assert!(gap_row(&gaps, "G-FINAL-001").ends_with("| In Progress |"));
    for row in gaps.lines().filter(|line| line.starts_with("| G-")) {
        assert!(
            !row.contains("Closed /"),
            "compound closed 상태 금지: {row}"
        );
        assert!(
            !row.contains("Implemented /"),
            "compound implemented 상태 금지: {row}"
        );
    }

    let causal_report = project_file("docs/audit/audit_report_22.md");
    assert!(causal_report.contains("Initial Finding과 현재 시정 상태"));
    assert!(causal_report.contains("independent re-audit pending"));
    assert!(causal_report.contains("2026-10-31"));

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

#[test]
fn active_r9_status_requires_field_only_pairs_and_new_ci() {
    let spec = project_file("spec.md");
    let r9 = markdown_section(&spec, "## 19. R9 후속 목표", "### 19.1 목표");
    assert!(r9.contains("9종 모두 동일 flow에서 대상 field/state만 neutralize"));
    assert!(r9.contains("SHA `ea7822a5`/Actions `32683076204`"));
    assert!(!r9.contains("gold/no-gold production pair와 독립 negative matrix를 시정 중"));
    assert!(!r9.contains("actual producer-removal matrix는 시정 중"));
}

#[test]
fn implementation_summary_does_not_reopen_completed_report_27_ci() {
    let summary = project_file("IMPLEMENTATION_SUMMARY.md");
    let implementation_order = markdown_section(
        &summary,
        "## 10. 구현 시작 순서",
        "## 11. R9 콘텐츠 인과 폐쇄 Task",
    );
    let r9 = &summary[summary.find("## 11. R9 콘텐츠 인과 폐쇄 Task").unwrap()..];

    assert!(implementation_order.contains("audit_report_28.md"));
    assert!(!implementation_order
        .contains("audit_report_27.md` 시정의 전체 local gate와 새 clean same-SHA 양 OS CI"));
    assert!(!r9.contains("새 CI와 독립 재감사 전에는 R9 최종 PASS"));
}

#[test]
fn sc_cause_contract_ids_map_to_code_and_tests() {
    let spec = project_file("spec.md");
    let roadmap = project_file("audit_roadmap.md");
    let summary = project_file("IMPLEMENTATION_SUMMARY.md");
    let causal_source = project_file("crates/aihack-runtime/src/causal.rs");
    let causal_tests = project_file("tests/causal_content.rs");
    let long_run_tests = project_file("tests/long_run.rs");

    for id in [
        "SC-CAUSE-01",
        "SC-CAUSE-02",
        "SC-CAUSE-03",
        "SC-CAUSE-04",
        "SC-CAUSE-05",
        "SC-CAUSE-06",
        "SC-CAUSE-07",
    ] {
        assert!(spec.contains(id), "spec ID 누락: {id}");
        assert!(
            roadmap
                .lines()
                .any(|line| line.starts_with(&format!("| {id} |"))),
            "audit roadmap 매핑 누락: {id}"
        );
        assert!(
            summary
                .lines()
                .any(|line| line.starts_with(&format!("| {id} |"))),
            "implementation summary 매핑 누락: {id}"
        );
    }

    for symbol in [
        "pub enum CausalWitness",
        "pub const REQUIRED_CAUSAL_WITNESSES",
        "pub struct CausalProjection",
        "pub struct CausalSummary",
        "pub struct CausalWitnessRecord",
        "pub fn observe_monster_speed_pair",
        "pub fn observe_monster_ai_pair",
        "pub fn observe_item_nutrition_pair",
        "pub fn observe_armor_defense_pair",
        "pub fn observe_monster_passive_pair",
        "pub fn observe_monster_difficulty_pair",
        "pub fn observe_prayer_luck_pair",
        "pub fn observe_gold_score_pair",
        "pub fn validate_required",
    ] {
        assert!(
            causal_source.contains(symbol),
            "causal source 책임 누락: {symbol}"
        );
    }
    for test_name in [
        "monster_speed_content_changes_actual_turn_movement",
        "monster_ai_content_changes_actual_turn_intent",
        "monster_passive_content_changes_player_status",
        "armor_content_bonus_changes_player_defense_state",
        "eating_food_changes_nutrition_hunger_and_item_lifecycle",
        "jackal_death_creates_an_edible_corpse_that_changes_hunger",
        "item_base_price_changes_actual_game_over_score",
        "prayer_created_luck_changes_the_next_attack_roll",
    ] {
        assert!(
            causal_tests.contains(&format!("fn {test_name}")),
            "causal content test 누락: {test_name}"
        );
        assert!(
            roadmap.contains(test_name),
            "audit roadmap test 매핑 누락: {test_name}"
        );
    }
    for test_name in [
        "causal_fixture_covers_every_required_witness_for_each_seed",
        "causal_witness_multiset_and_final_hash_are_stable_across_three_runs",
        "causal_validator_rejects_event_only_and_turn_only_changes",
        "causal_field_only_ab_loses_exactly_one_witness_and_preserves_other_records",
        "gold_score_witness_uses_a_paired_production_score",
    ] {
        assert!(
            long_run_tests.contains(&format!("fn {test_name}")),
            "long-run test 누락: {test_name}"
        );
        assert!(
            roadmap.contains(test_name),
            "audit roadmap test 매핑 누락: {test_name}"
        );
    }
}

fn collect_markdown_files(root: &Path, directory: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let relative = path.strip_prefix(root).unwrap();
            if matches!(
                relative
                    .components()
                    .next()
                    .and_then(|part| part.as_os_str().to_str()),
                Some(".git" | ".archive" | "target" | "legacy_nethack_port_reference")
            ) {
                continue;
            }
            collect_markdown_files(root, &path, out);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

#[test]
fn active_markdown_relative_links_resolve_after_audit_report_relocation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_markdown_files(root, root, &mut files);
    let mut broken = Vec::new();

    for file in files {
        let content = fs::read_to_string(&file).unwrap();
        let mut check_target = |raw: &str| {
            let raw = raw.trim().trim_matches(['<', '>']);
            let target = raw.split('#').next().unwrap_or_default();
            if target.is_empty()
                || target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
            {
                return;
            }
            let resolved = file.parent().unwrap().join(target.replace("%20", " "));
            if !resolved.exists() {
                broken.push(format!(
                    "{} -> {target}",
                    file.strip_prefix(root).unwrap().display()
                ));
            }
        };
        for line in content.lines() {
            let mut remainder = line;
            while let Some(start) = remainder.find("](") {
                remainder = &remainder[start + 2..];
                let Some(end) = remainder.find(')') else {
                    break;
                };
                check_target(&remainder[..end]);
                remainder = &remainder[end + 1..];
            }
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                if let Some(separator) = trimmed.find("]:") {
                    if let Some(target) = trimmed[separator + 2..].split_whitespace().next() {
                        check_target(target);
                    }
                }
            }
        }
    }

    assert!(broken.is_empty(), "broken Markdown links: {broken:#?}");
}

#[test]
fn save_permission_contract_matches_platform_guarantees() {
    let spec = project_file("spec.md");
    let guide = project_file("BUILD_GUIDE.md");
    let decisions = project_file("DESIGN_DECISIONS.md");

    for document in [spec, guide, decisions] {
        assert!(document.contains("Unix"));
        assert!(document.contains("0600"));
        assert!(document.contains("Windows"));
        assert!(document.contains("parent directory DACL"));
        assert!(!document.contains("지원 OS 모두에서 owner-only"));
    }
}

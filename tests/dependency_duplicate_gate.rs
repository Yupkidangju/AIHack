use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};

fn project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn resolved_duplicate_families() -> BTreeMap<String, BTreeSet<String>> {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut versions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for package in metadata["packages"].as_array().unwrap() {
        versions
            .entry(package["name"].as_str().unwrap().to_string())
            .or_default()
            .insert(package["version"].as_str().unwrap().to_string());
    }
    versions.retain(|_, versions| versions.len() > 1);
    versions
}

fn validate_budget(
    budget: &serde_json::Value,
    actual: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), String> {
    if budget["schema_version"] != 2 {
        return Err("unsupported duplicate budget schema".to_string());
    }
    for field in ["owner", "reason", "shipped_scope", "reviewed_on"] {
        if budget[field].as_str().is_none_or(str::is_empty) {
            return Err(format!("missing duplicate budget metadata: {field}"));
        }
    }
    if budget["owner"] != "Dependency owner / Release manager"
        || budget["reviewed_on"] != "2026-08-24"
        || !budget["shipped_scope"]
            .as_str()
            .is_some_and(|scope| scope.contains("workspace --all-targets"))
    {
        return Err("duplicate budget ownership or review scope drift".to_string());
    }
    let triggers = budget["review_triggers"]
        .as_array()
        .ok_or("review_triggers")?
        .iter()
        .map(|value| value.as_str().ok_or("review trigger"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let expected_triggers = [
        "Cargo.lock change",
        "target matrix change",
        "dev dependency change",
        "toolchain change",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    if triggers != expected_triggers {
        return Err("duplicate budget review trigger drift".to_string());
    }
    let max = budget["max_duplicate_families"]
        .as_u64()
        .ok_or("max_duplicate_families")? as usize;
    let declared = budget["families"].as_object().ok_or("families")?;
    if declared.len() > max || actual.len() > max || declared.len() != actual.len() {
        return Err("duplicate family budget mismatch".to_string());
    }
    for (name, expected) in declared {
        let expected = expected
            .as_array()
            .ok_or("family versions")?
            .iter()
            .map(|version| version.as_str().ok_or("version"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        let actual = actual
            .get(name)
            .ok_or_else(|| format!("missing family: {name}"))?;
        if expected.len() < 2
            || actual.iter().map(String::as_str).collect::<BTreeSet<_>>() != expected
        {
            return Err(format!("duplicate version drift: {name}"));
        }
    }
    Ok(())
}

#[test]
fn resolved_duplicate_families_match_the_explicit_budget_exactly() {
    let budget: serde_json::Value =
        serde_json::from_str(&project_file("dependency-duplicate-budget.json")).unwrap();
    validate_budget(&budget, &resolved_duplicate_families()).unwrap();
}

#[test]
fn new_family_version_or_budget_reduction_fails_closed() {
    let actual = resolved_duplicate_families();
    let live: serde_json::Value =
        serde_json::from_str(&project_file("dependency-duplicate-budget.json")).unwrap();

    let mut reduced = live.clone();
    reduced["max_duplicate_families"] = serde_json::json!(23);
    assert!(validate_budget(&reduced, &actual).is_err());

    let mut version_drift = live.clone();
    version_drift["families"]["getrandom"] = serde_json::json!(["0.2.17", "0.4.4"]);
    assert!(validate_budget(&version_drift, &actual).is_err());

    let mut new_family = live;
    new_family["families"]["unexpected"] = serde_json::json!(["1.0.0", "2.0.0"]);
    assert!(validate_budget(&new_family, &actual).is_err());
}

#[test]
fn ownership_scope_and_review_metadata_are_required() {
    let actual = resolved_duplicate_families();
    let live: serde_json::Value =
        serde_json::from_str(&project_file("dependency-duplicate-budget.json")).unwrap();

    for field in [
        "owner",
        "reason",
        "shipped_scope",
        "reviewed_on",
        "review_triggers",
    ] {
        let mut missing = live.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(validate_budget(&missing, &actual).is_err(), "field={field}");
    }
}

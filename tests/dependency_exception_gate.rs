use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
struct DependencyGraph {
    versions: BTreeMap<String, BTreeSet<String>>,
    edges: BTreeSet<(String, String)>,
}

const EXPECTED_TRIGGER_VERSIONS: [(&str, &str); 5] = [
    ("cap-fs-ext", "4.0.2"),
    ("cap-primitives", "4.0.2"),
    ("cap-std", "4.0.2"),
    ("cap-tempfile", "4.0.2"),
    ("winx", "0.36.4"),
];

const EXPECTED_DEPENDENCY_PATH: [&str; 4] =
    ["aihack-runtime", "cap-fs-ext", "cap-primitives", "winx"];

fn project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn cargo_graph() -> DependencyGraph {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut by_id = BTreeMap::new();
    let mut versions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for package in metadata["packages"].as_array().unwrap() {
        let id = package["id"].as_str().unwrap().to_string();
        let name = package["name"].as_str().unwrap().to_string();
        let version = package["version"].as_str().unwrap().to_string();
        by_id.insert(id, name.clone());
        versions.entry(name).or_default().insert(version);
    }
    let mut edges = BTreeSet::new();
    for node in metadata["resolve"]["nodes"].as_array().unwrap() {
        let from = by_id.get(node["id"].as_str().unwrap()).unwrap().clone();
        for dependency in node["deps"].as_array().unwrap() {
            let to = by_id
                .get(dependency["pkg"].as_str().unwrap())
                .unwrap()
                .clone();
            edges.insert((from.clone(), to));
        }
    }
    DependencyGraph { versions, edges }
}

fn date_days(value: &str) -> Result<i64, String> {
    let mut parts = value.split('-');
    let year: i64 = parts.next().ok_or("year")?.parse().map_err(|_| "year")?;
    let month: i64 = parts.next().ok_or("month")?.parse().map_err(|_| "month")?;
    let day: i64 = parts.next().ok_or("day")?.parse().map_err(|_| "day")?;
    let leap = year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return Err("invalid month".to_string()),
    };
    if parts.next().is_some() || day < 1 || day > days_in_month {
        return Err("invalid date".to_string());
    }
    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Ok(era * 146_097 + day_of_era - 719_468)
}

fn validate_exception_gate(
    ledger: &serde_json::Value,
    deny: &str,
    graph: &DependencyGraph,
    today: i64,
) -> Result<(), String> {
    if ledger["schema_version"] != 1 {
        return Err("unsupported exception ledger schema".to_string());
    }
    let exceptions = ledger["exceptions"]
        .as_array()
        .ok_or("missing exceptions")?;
    let parsed_deny: toml::Value = toml::from_str(deny).map_err(|error| error.to_string())?;
    let deny_exceptions = parsed_deny
        .get("licenses")
        .and_then(|licenses| licenses.get("exceptions"))
        .and_then(toml::Value::as_array)
        .ok_or("missing parsed licenses.exceptions")?;
    if exceptions.len() != 1 || deny_exceptions.len() != 1 {
        return Err("exception count mismatch".to_string());
    }
    let exception = &exceptions[0];
    for field in ["id", "crate", "version", "license", "owner", "reason"] {
        if exception[field].as_str().is_none_or(str::is_empty) {
            return Err(format!("missing exception field: {field}"));
        }
    }
    if exception["id"] != "DEP-EXC-0001" {
        return Err("unexpected exception id".to_string());
    }
    let approved = date_days(exception["approved_on"].as_str().ok_or("approved_on")?)?;
    let expires = date_days(exception["expires_on"].as_str().ok_or("expires_on")?)?;
    if approved > today || expires < today || expires <= approved || expires - approved > 90 {
        return Err("exception expired or exceeds the 90-day budget".to_string());
    }

    let crate_name = exception["crate"].as_str().unwrap();
    let version = exception["version"].as_str().unwrap();
    let requirement = exception["deny_version_requirement"].as_str().unwrap();
    let license = exception["license"].as_str().unwrap();
    if requirement != format!("={version}") {
        return Err("deny version requirement is not exact".to_string());
    }
    let deny_exception = deny_exceptions[0]
        .as_table()
        .ok_or("deny exception must be a table")?;
    let deny_keys = deny_exception
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected_deny_keys = ["allow", "name", "version"]
        .into_iter()
        .collect::<BTreeSet<_>>();
    if deny_keys != expected_deny_keys
        || deny_exception.get("name").and_then(toml::Value::as_str) != Some(crate_name)
        || deny_exception.get("version").and_then(toml::Value::as_str) != Some(requirement)
    {
        return Err("deny.toml exception table drift".to_string());
    }
    let deny_allow = deny_exception
        .get("allow")
        .and_then(toml::Value::as_array)
        .ok_or("deny exception allow")?;
    if deny_allow.len() != 1 || deny_allow[0].as_str() != Some(license) {
        return Err("deny.toml exception license drift".to_string());
    }

    let trigger_versions = exception["trigger_versions"]
        .as_object()
        .ok_or("trigger_versions")?;
    let actual_trigger_keys = trigger_versions
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected_trigger_keys = EXPECTED_TRIGGER_VERSIONS
        .iter()
        .map(|(name, _)| *name)
        .collect::<BTreeSet<_>>();
    if actual_trigger_keys != expected_trigger_keys {
        return Err("dependency trigger key set drift".to_string());
    }
    for (name, expected) in EXPECTED_TRIGGER_VERSIONS {
        if trigger_versions
            .get(name)
            .and_then(serde_json::Value::as_str)
            != Some(expected)
        {
            return Err(format!("ledger trigger version drift: {name}"));
        }
        let actual = graph
            .versions
            .get(name)
            .ok_or_else(|| format!("missing {name}"))?;
        if actual.len() != 1 || !actual.contains(expected) {
            return Err(format!("dependency version drift: {name}"));
        }
    }

    let path = exception["dependency_path"]
        .as_array()
        .ok_or("dependency_path")?
        .iter()
        .map(|entry| entry.as_str().ok_or("dependency path entry"))
        .collect::<Result<Vec<_>, _>>()?;
    if path.as_slice() != EXPECTED_DEPENDENCY_PATH
        || path.windows(2).any(|pair| {
            !graph
                .edges
                .contains(&(pair[0].to_string(), pair[1].to_string()))
        })
    {
        return Err("dependency path drift".to_string());
    }
    Ok(())
}

#[test]
fn live_exception_is_unexpired_version_scoped_and_present_in_the_resolved_graph() {
    let ledger: serde_json::Value =
        serde_json::from_str(&project_file("dependency-exceptions.json")).unwrap();
    let deny = project_file("deny.toml");
    let graph = cargo_graph();
    let today = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 86_400) as i64;

    validate_exception_gate(&ledger, &deny, &graph, today).unwrap();
}

#[test]
fn expired_version_drift_and_unrelated_crate_fixtures_fail_closed() {
    let live: serde_json::Value =
        serde_json::from_str(&project_file("dependency-exceptions.json")).unwrap();
    let deny = project_file("deny.toml");
    let graph = cargo_graph();

    assert!(
        validate_exception_gate(&live, &deny, &graph, date_days("2026-11-01").unwrap()).is_err()
    );

    let mut drift = live.clone();
    drift["exceptions"][0]["trigger_versions"]["winx"] = serde_json::json!("0.36.5");
    assert!(
        validate_exception_gate(&drift, &deny, &graph, date_days("2026-08-23").unwrap()).is_err()
    );

    let mut unrelated = live;
    unrelated["exceptions"][0]["crate"] = serde_json::json!("serde");
    assert!(
        validate_exception_gate(&unrelated, &deny, &graph, date_days("2026-08-23").unwrap())
            .is_err()
    );
}

#[test]
fn comment_decoy_missing_trigger_and_invalid_calendar_date_fail_closed() {
    let live: serde_json::Value =
        serde_json::from_str(&project_file("dependency-exceptions.json")).unwrap();
    let graph = cargo_graph();
    let decoy = r#"
[licenses]
allow = ["MIT"]

# name = "winx"
# version = "=0.36.4"
# allow = ["Apache-2.0 WITH LLVM-exception"]
[[licenses.exceptions]]
name = "serde"
version = "=1.0.0"
allow = ["MIT"]
"#;
    assert!(
        validate_exception_gate(&live, decoy, &graph, date_days("2026-08-23").unwrap()).is_err(),
        "comment text must not satisfy the structural deny exception contract"
    );

    let mut missing_trigger = live.clone();
    missing_trigger["exceptions"][0]["trigger_versions"]
        .as_object_mut()
        .unwrap()
        .remove("cap-tempfile");
    assert!(validate_exception_gate(
        &missing_trigger,
        &project_file("deny.toml"),
        &graph,
        date_days("2026-08-23").unwrap(),
    )
    .is_err());

    let mut invalid_date = live;
    invalid_date["exceptions"][0]["approved_on"] = serde_json::json!("2026-02-31");
    assert!(validate_exception_gate(
        &invalid_date,
        &project_file("deny.toml"),
        &graph,
        date_days("2026-03-15").unwrap(),
    )
    .is_err());

    let mut future_approval: serde_json::Value =
        serde_json::from_str(&project_file("dependency-exceptions.json")).unwrap();
    future_approval["exceptions"][0]["approved_on"] = serde_json::json!("2026-09-01");
    future_approval["exceptions"][0]["expires_on"] = serde_json::json!("2026-10-31");
    assert!(
        validate_exception_gate(
            &future_approval,
            &project_file("deny.toml"),
            &graph,
            date_days("2026-08-24").unwrap(),
        )
        .is_err(),
        "감사일보다 미래인 approval date를 허용하면 안 됩니다"
    );
}

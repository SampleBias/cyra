use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vmcell"))
}

#[test]
fn a00_01_no_interpreter_path_or_credentials_required() {
    let output = binary()
        .env_clear()
        .env("PATH", "/nonexistent")
        .args(["--offline", "--format", "json", "version"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        result["result"]["expansion"],
        "Cybernetic Cellular Intelligence"
    );
}

#[test]
fn a00_05_all_structured_cli_paths_emit_one_clean_envelope() {
    let cases: Vec<(Vec<&str>, i32)> = vec![
        (vec!["version"], 0),
        (vec!["capabilities"], 0),
        (vec!["schema", "request"], 0),
        (vec!["schema", "absent"], 2),
        (vec!["intervene", "--model", "secret-file"], 3),
        (vec!["counterfactual"], 3),
        (vec!["simulate"], 3),
        (vec!["nonsense"], 2),
        (vec!["--nonsense", "secret-token"], 2),
        (vec!["--help"], 0),
    ];
    for (args, exit) in cases {
        let output = binary()
            .args(["--format", "json"])
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(exit), "{args:?}");
        assert!(output.stderr.is_empty(), "{args:?}");
        let text = String::from_utf8(output.stdout).unwrap();
        assert_eq!(text.lines().count(), 1);
        assert!(!text.contains('\u{1b}'));
        assert!(!text.contains("secret-token"));
        let response: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(response["exit_code"], exit);
    }
}

#[test]
fn a00_03_agent_recovers_and_reports_causal_unavailability() {
    let mut child = binary()
        .args(["agent", "--protocol", "jsonl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    writeln!(
        input,
        "{}",
        json!({"schema_version":"vmcell.protocol.v1","id":"a","operation":"version","arguments":{}})
    )
    .unwrap();
    writeln!(input, "bad JSON").unwrap();
    writeln!(input,"{}",json!({"schema_version":"vmcell.protocol.v1","id":"b","operation":"counterfactual","arguments":{}})).unwrap();
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let r: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(r.len(), 3);
    assert_eq!(r[1]["status"], "invalid_input");
    assert_eq!(r[2]["query_level"], "counterfactual");
    assert!(r[2]["result"].is_null());
}

#[test]
fn a00_05_human_output_names_project_and_maturity() {
    let output = binary().arg("version").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Cyra"));
    assert!(text.contains("Sprint 00"));
    assert!(text.contains("not") || text.contains("No biological simulator"));
}

fn fixture(
    dir: &std::path::Path,
) -> (
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let input = dir.join("rows.csv");
    let mapping = dir.join("mapping.json");
    let metadata = dir.join("metadata.json");
    let out = dir.join("dataset");
    std::fs::write(
        &input,
        "sample,analyte,value,unit,modality,missing_reason,censor_limit,qc_failure,time_amount,time_unit,time_origin,assay_id,protocol_version,compartment\nS1,GAPDH,0,count,transcript,,,,24,hour,stimulation_start,rna-1,v1,intracellular\nS1,ACTB,,tpm,transcript,not_measured,,,24,hour,stimulation_start,rna-1,v1,intracellular\n",
    )
    .unwrap();
    std::fs::write(
        &mapping,
        r#"{"schema_version":"cyra.import-mapping.v1","format":"csv","columns":{"sample_id":"sample","analyte_id":"analyte","value":"value","unit":"unit","modality":"modality","missing_reason":"missing_reason","censor_limit":"censor_limit","qc_failure":"qc_failure","time_amount":"time_amount","time_unit":"time_unit","time_origin":"time_origin","assay_id":"assay_id","protocol_version":"protocol_version","compartment":"compartment"},"analytes":{"GAPDH":{"modality":"transcript","unit":"count"},"ACTB":{"modality":"transcript","unit":"tpm"}},"conversions":[]}"#,
    )
    .unwrap();
    std::fs::write(
        &metadata,
        r#"{"schema_version":"cyra.dataset-metadata.v1","dataset_id":"fixture-cli","origin":"synthetic_fixture","tissue_context":"illustrative_whartons_jelly","accession":{"required":false,"value":null,"note":"synthetic fixture has no repository accession"},"licenses":[{"domain":"dataset","rights_basis":"synthetic_fixture_authorization","intended_use":"pipeline_validation","spdx":null,"note":"Pipeline fixture. Not a biological observation."},{"domain":"code","rights_basis":"unknown","intended_use":"not_applicable","spdx":null,"note":"This fixture records unknown code rights and is not the MIT license of the Cyra source."},{"domain":"model_weight","rights_basis":"unknown","intended_use":"not_applicable","spdx":null,"note":"No model weights belong to this dataset."}],"sources":[{"source_id":"src-fixture","kind":"synthetic_table","uri":null,"version":"1","content_role":"measurements","checksum_sha256":null}],"samples":[{"sample_id":"S1","donor_id":"D1","cell_id":"C1","population_id":"POP","replicate_id":"R1","replicate_role":"primary","parent_sample_id":null,"passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"}],"unavailable_fields":["donor_age","raw_file_uri"]}"#,
    )
    .unwrap();
    (input, mapping, metadata, out)
}

#[test]
fn a01_09_human_and_jsonl_provenance_match() {
    let dir = tempfile::tempdir().unwrap();
    let (input, mapping, metadata, out) = fixture(dir.path());
    let imported = binary()
        .args(["--format", "json", "import", "--input"])
        .arg(&input)
        .arg("--mapping")
        .arg(&mapping)
        .arg("--metadata")
        .arg(&metadata)
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();
    assert_eq!(
        imported.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&imported.stderr)
    );
    assert!(imported.stderr.is_empty());
    let imported: Value = serde_json::from_slice(&imported.stdout).unwrap();
    assert_eq!(imported["exit_code"], 0);
    assert_eq!(
        imported["result"]["scientific_computation_performed"],
        false
    );
    assert_eq!(imported["result"]["benchmark_plan_included"], false);
    assert_eq!(imported["result"]["origin"], "synthetic_fixture");
    let sha = imported["result"]["result_manifest_sha256"]
        .as_str()
        .unwrap()
        .to_owned();
    let request = dir.path().join("request.json");
    std::fs::write(
        &request,
        r#"{"schema_version":"cyra.observation-request.v1","kind":"measurement_retrieval","sample_ids":null,"analyte_ids":null,"modalities":null,"include_missing":true}"#,
    )
    .unwrap();
    let observed = binary()
        .args(["--format", "json", "observe", "--dataset"])
        .arg(&out)
        .arg("--request")
        .arg(&request)
        .output()
        .unwrap();
    assert!(observed.status.success());
    let observed: Value = serde_json::from_slice(&observed.stdout).unwrap();
    assert_eq!(observed["query_level"], "association");
    assert_eq!(
        observed["scientific_context"]["identification"]["status"],
        "not_applicable"
    );
    assert_eq!(
        observed["execution"]["scientific_computation_performed"],
        false
    );
    assert_eq!(observed["result"]["record_count"], 2);
    assert!(
        observed["result"]["text"]
            .as_str()
            .unwrap()
            .contains("not an association estimate")
    );
    assert!(
        observed["result"]["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["value"] == "0")
    );
    assert!(
        observed["result"]["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["value"].is_null() && record["missing_reason"] == "not_measured")
    );
    let human = binary()
        .args(["observe", "--dataset"])
        .arg(&out)
        .arg("--request")
        .arg(&request)
        .output()
        .unwrap();
    let human_text = String::from_utf8(human.stdout).unwrap();
    assert!(human.status.success());
    assert!(human_text.contains("fixture-cli"));
    assert!(human_text.contains("not an association estimate"));
    assert!(human_text.contains("donor_age"));
    let prov = binary()
        .args(["--format", "json", "provenance", "--artifact"])
        .arg(&out)
        .output()
        .unwrap();
    assert!(prov.status.success());
    let prov: Value = serde_json::from_slice(&prov.stdout).unwrap();
    assert_eq!(prov["result"]["result_manifest_sha256"], sha);
    assert_eq!(prov["result"]["origin"], "synthetic_fixture");
    assert_eq!(prov["query_level"], Value::Null);
    assert!(prov["result"]["licenses"].as_array().unwrap().len() == 3);
    let human_prov = binary()
        .args(["provenance", "--artifact"])
        .arg(out.join("manifest.json"))
        .output()
        .unwrap();
    let human_prov = String::from_utf8(human_prov.stdout).unwrap();
    assert!(human_prov.contains(&sha));
    assert!(human_prov.contains("synthetic_fixture"));
    assert!(human_prov.contains("No physiological claim"));

    let mut child = binary()
        .args(["agent", "--protocol", "jsonl"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for line in [
        json!({"schema_version":"vmcell.protocol.v1","id":"empty","operation":"observe","arguments":{}}),
        json!({"schema_version":"vmcell.protocol.v1","id":"assoc","operation":"observe","arguments":{"dataset":out,"request":request}}),
        json!({"schema_version":"vmcell.protocol.v1","id":"prov","operation":"provenance","arguments":{"artifact":out}}),
    ] {
        writeln!(stdin, "{line}").unwrap();
    }
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let lines: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["exit_code"], 3);
    assert_eq!(lines[0]["query_level"], "association");
    assert!(lines[0]["result"].is_null());
    assert_eq!(
        lines[0]["execution"]["scientific_computation_performed"],
        false
    );
    assert_eq!(lines[1]["result"]["result_manifest_sha256"].as_str(), None);
    assert_eq!(lines[1]["result"]["dataset_id"], "fixture-cli");
    assert_eq!(
        lines[1]["scientific_context"]["identification"]["reason"],
        "measurement_retrieval_is_not_an_estimand"
    );
    assert_eq!(lines[2]["result"]["result_manifest_sha256"], sha);
    assert!(lines[2]["result"]["human"].as_str().unwrap().contains(&sha));
}

#[test]
fn sprint_01_version_capabilities_and_schema_stay_honest() {
    let version = binary()
        .args(["--format", "json", "version"])
        .output()
        .unwrap();
    let version: Value = serde_json::from_slice(&version.stdout).unwrap();
    assert_eq!(version["result"]["stage"], "sprint_01_data_provenance");
    assert_eq!(version["result"]["simulator"], "not_implemented");
    assert_eq!(version["result"]["foundation"], "sprint_00");
    let human = String::from_utf8(binary().arg("version").output().unwrap().stdout).unwrap();
    assert!(human.contains("Sprint 00"));
    assert!(human.contains("Sprint 01"));
    assert!(human.contains("No biological simulator"));
    let caps = binary()
        .args(["--format", "json", "capabilities"])
        .output()
        .unwrap();
    let caps: Value = serde_json::from_slice(&caps.stdout).unwrap();
    let available = caps["result"]["available_operations"].as_array().unwrap();
    assert!(available.iter().any(|item| item == "import"));
    assert!(available.iter().any(|item| item == "observe"));
    let planned = caps["result"]["planned_operations"].as_array().unwrap();
    assert!(
        planned
            .iter()
            .all(|item| item != "import" && item != "observe" && item != "provenance")
    );
    assert!(planned.iter().any(|item| item == "simulate"));
    assert_eq!(
        caps["result"]["observe_scope"],
        "measurement_retrieval_only"
    );
    assert_eq!(caps["result"]["network_adapters"], json!([]));
    let schema = binary()
        .args(["--format", "json", "schema", "import-mapping"])
        .output()
        .unwrap();
    assert!(schema.status.success());
    let schema: Value = serde_json::from_slice(&schema.stdout).unwrap();
    assert_eq!(schema["result"]["title"], "ImportMapping");
}

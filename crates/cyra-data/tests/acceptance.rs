//! Sprint 01 acceptance checks. Fixtures are synthetic and are not physiological evidence.
use cyra_core::artifact::sha256;
use cyra_data::{
    DataError, Eligibility, ImportRequest, MissingReason, Modality, Origin, PARQUET_ROW_GROUP_ROWS,
    import_dataset, max_one_chunk_working_set, observe_dataset, provenance, read_row_groups,
    read_slice, read_versioned_measurements, row_group_count,
};
use serde_json::{Value, json};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

const HEADER: &str = "sample,analyte,value,unit,modality,missing_reason,censor_limit,qc_failure,time_amount,time_unit,time_origin,assay_id,protocol_version,compartment";

const DEFAULT_SAMPLES: &str = r#"[
    {"sample_id":"S1","donor_id":"D1","cell_id":"C1","population_id":"POP","replicate_id":"R1","replicate_role":"primary","parent_sample_id":null,"passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"},
    {"sample_id":"S1t","donor_id":"D1","cell_id":"C1","population_id":"POP","replicate_id":"R1t","replicate_role":"technical","parent_sample_id":"S1","passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"},
    {"sample_id":"S2","donor_id":"D2","cell_id":"C2","population_id":"POP","replicate_id":"R2","replicate_role":"primary","parent_sample_id":null,"passage":4,"tissue":"whartons_jelly","culture_batch":"CB2","experimental_batch":"EB2","perturbation":"none","study_id":"study-fixture","source_identity":"src-d2"},
    {"sample_id":"S0","donor_id":null,"cell_id":null,"population_id":null,"replicate_id":null,"replicate_role":"primary","parent_sample_id":null,"passage":null,"tissue":null,"culture_batch":null,"experimental_batch":null,"perturbation":null,"study_id":null,"source_identity":null}
]"#;

/// Identical IL6 rows deduplicate. A different value on the same biological key is a separate test.
/// S2 carries the same canonical concentration under a different sample id and must not be merged.
const HAPPY_CSV: &str = "\
sample,analyte,value,unit,modality,missing_reason,censor_limit,qc_failure,time_amount,time_unit,time_origin,assay_id,protocol_version,compartment
S1,GAPDH,0,count,transcript,,,,24,hour,stimulation_start,rna-1,v1,intracellular
S1,ACTB,,tpm,transcript,not_measured,,,24,hour,stimulation_start,rna-1,v1,intracellular
S1,TP53,10.50,abundance_au,intracellular_protein,,,,24,hour,stimulation_start,wb-1,v1,intracellular
S1,IL6,1.25,ng_per_ml,secreted_protein,,,,24,hour,stimulation_start,elisa-1,v1,extracellular
S1,IL6,1.25,ng_per_ml,secreted_protein,,,,24,hour,stimulation_start,elisa-1,v1,extracellular
S1,TNF,,pg_per_ml,secreted_protein,below_detection,0.80,,24,hour,stimulation_start,elisa-1,v1,extracellular
S1,LACTATE,3.250,micromol_per_l,metabolite,,,,24,hour,stimulation_start,met-1,v1,intracellular
S1,VIABILITY,95,percent,functional_assay,,,,24,hour,stimulation_start,func-1,v1,not_applicable
S1t,GAPDH,1,count,transcript,,,,24,hour,stimulation_start,rna-1,v1,intracellular
S2,IL6,1250,pg_per_ml,secreted_protein,,,,24,hour,stimulation_start,elisa-1,v1,extracellular
S2,ACTB,,tpm,transcript,failed_qc,,low_reads,24,hour,stimulation_start,rna-1,v1,intracellular
S0,IL6,5,pg_per_ml,secreted_protein,,,,24,hour,stimulation_start,elisa-1,v1,extracellular
";

struct Meta<'a> {
    origin: &'a str,
    rights: &'a str,
    intended: &'a str,
    accession_required: bool,
    accession_note: &'a str,
    split: bool,
    samples: &'a str,
    checksum: Option<&'a str>,
    dataset_id: &'a str,
}

fn write_bytes(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(bytes).unwrap();
    path
}

fn mapping(format: &str) -> String {
    format!(
        r#"{{
  "schema_version":"cyra.import-mapping.v1",
  "format":"{format}",
  "columns":{{
    "sample_id":"sample","analyte_id":"analyte","value":"value","unit":"unit","modality":"modality",
    "missing_reason":"missing_reason","censor_limit":"censor_limit","qc_failure":"qc_failure",
    "time_amount":"time_amount","time_unit":"time_unit","time_origin":"time_origin",
    "assay_id":"assay_id","protocol_version":"protocol_version","compartment":"compartment"
  }},
  "analytes":{{
    "GAPDH":{{"modality":"transcript","unit":"count"}},
    "ACTB":{{"modality":"transcript","unit":"tpm"}},
    "TP53":{{"modality":"intracellular_protein","unit":"abundance_au"}},
    "IL6":{{"modality":"secreted_protein","unit":"pg_per_ml"}},
    "TNF":{{"modality":"secreted_protein","unit":"pg_per_ml"}},
    "LACTATE":{{"modality":"metabolite","unit":"micromol_per_l"}},
    "VIABILITY":{{"modality":"functional_assay","unit":"percent"}}
  }},
  "conversions":[{{"analyte_id":"IL6","from_unit":"ng_per_ml","to_unit":"pg_per_ml","factor":"1000"}}]
}}"#
    )
}

fn metadata(meta: &Meta<'_>) -> String {
    let split = if meta.split {
        r#","split":{"seed":11,"holdout_fraction":"0.2","validation_fraction":"0.2"}"#
    } else {
        ""
    };
    let checksum = match meta.checksum {
        Some(value) => format!("\"{value}\""),
        None => "null".to_owned(),
    };
    format!(
        r#"{{
  "schema_version":"cyra.dataset-metadata.v1",
  "dataset_id":"{dataset_id}",
  "origin":"{origin}",
  "tissue_context":"illustrative_whartons_jelly",
  "accession":{{"required":{required},"value":null,"note":"{note}"}},
  "licenses":[
    {{"domain":"dataset","rights_basis":"{rights}","intended_use":"{intended}","spdx":null,"note":"Dataset rights are distinct from code and model weights. These rows are a pipeline fixture, not an approved biological cohort."}},
    {{"domain":"code","rights_basis":"unknown","intended_use":"not_applicable","spdx":null,"note":"Repository license is undecided and is distinct from the dataset license."}},
    {{"domain":"model_weight","rights_basis":"unknown","intended_use":"not_applicable","spdx":null,"note":"No model weights belong to this dataset."}}
  ],
  "sources":[{{"source_id":"src-fixture","kind":"synthetic_table","uri":null,"version":"1","content_role":"measurements","checksum_sha256":{checksum}}}],
  "samples":{samples},
  "unavailable_fields":["donor_age","raw_file_uri"]
  {split}
}}"#,
        dataset_id = meta.dataset_id,
        origin = meta.origin,
        required = meta.accession_required,
        note = meta.accession_note,
        rights = meta.rights,
        intended = meta.intended,
        samples = meta.samples,
    )
}

fn happy_meta() -> Meta<'static> {
    Meta {
        origin: "synthetic_fixture",
        rights: "synthetic_fixture_authorization",
        intended: "pipeline_validation",
        accession_required: false,
        accession_note: "synthetic fixture has no repository accession",
        split: true,
        samples: DEFAULT_SAMPLES,
        checksum: None,
        dataset_id: "fixture-modalities",
    }
}

fn import_bytes(
    dir: &Path,
    table_name: &str,
    table: &[u8],
    format: &str,
    meta: &Meta<'_>,
    publish: bool,
) -> Result<cyra_data::ImportOutcome, DataError> {
    let input = write_bytes(dir, table_name, table);
    let mapping_path = write_bytes(dir, "mapping.json", mapping(format).as_bytes());
    let metadata_path = write_bytes(dir, "metadata.json", metadata(meta).as_bytes());
    import_dataset(&ImportRequest {
        input,
        mapping: mapping_path,
        metadata: metadata_path,
        out: dir.join("dataset"),
        publish,
    })
}

fn import_happy(publish: bool) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("dataset");
    let outcome = import_bytes(
        dir.path(),
        "rows.csv",
        HAPPY_CSV.as_bytes(),
        "csv",
        &happy_meta(),
        publish,
    );
    if publish {
        let outcome = outcome.expect("eligible synthetic import");
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.summary["benchmark_plan_included"], false);
        assert_eq!(outcome.summary["eligibility"], "eligible");
        assert_eq!(outcome.summary["scientific_computation_performed"], false);
        assert!(out.join("manifest.json").is_file());
    } else {
        let error = outcome.expect_err("interrupted import");
        assert_eq!(error.exit_code, 8);
        assert!(!out.exists());
        assert!(fs::read_dir(dir.path()).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("cyra-stage")
        }));
    }
    (dir, out)
}

fn request(include_missing: bool) -> Vec<u8> {
    format!(
        r#"{{"schema_version":"cyra.observation-request.v1","kind":"measurement_retrieval","sample_ids":null,"analyte_ids":null,"modalities":null,"include_missing":{include_missing}}}"#
    )
    .into_bytes()
}

fn one(
    records: &[cyra_data::MeasurementRecord],
    sample: &str,
    analyte: &str,
) -> cyra_data::MeasurementRecord {
    let found: Vec<_> = records
        .iter()
        .filter(|record| record.sample_id == sample && record.analyte_id == analyte)
        .cloned()
        .collect();
    assert_eq!(found.len(), 1, "{sample}/{analyte}");
    found.into_iter().next().unwrap()
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().mode() & 0o777
}

fn evidence(name: &str, value: &Value) {
    let Ok(dir) = std::env::var("CYRA_SPRINT01_EVIDENCE") else {
        return;
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(Path::new(&dir).join(name))
        .unwrap();
    file.write_all(serde_json::to_vec_pretty(value).unwrap().as_slice())
        .unwrap();
}

fn bulk_table(rows: usize) -> (String, String) {
    let mut csv = String::from(HEADER);
    csv.push('\n');
    for index in 0..rows {
        csv.push_str(&format!(
            "S1,GAPDH,{index},count,transcript,,,,{index},hour,stimulation_start,rna-1,v1,intracellular\n"
        ));
    }
    let samples = r#"[{"sample_id":"S1","donor_id":"D1","cell_id":"C1","population_id":"POP","replicate_id":"R1","replicate_role":"primary","parent_sample_id":null,"passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"}]"#;
    (csv, samples.to_owned())
}

#[test]
fn a01_01_round_trips_every_modality_without_collapsing_missingness() {
    let (dir, out) = import_happy(true);
    assert_eq!(mode(&out), 0o700);
    assert_eq!(mode(&out.join("manifest.json")), 0o600);
    assert_eq!(mode(&out.join("tables/measurements.parquet")), 0o600);
    let request_path = write_bytes(dir.path(), "request.json", &request(true));
    let (loaded, retrieved) = observe_dataset(&out, &request_path).unwrap();
    assert_eq!(loaded.records, retrieved);
    let zarr = read_slice(&out.join("arrays/measurements"), 0, loaded.records.len()).unwrap();
    assert_eq!(zarr.records, loaded.records);
    assert_eq!(loaded.manifest.measurement_count, 10);
    assert_eq!(loaded.manifest.deduplicated_count, 1);
    assert_eq!(loaded.manifest.quarantined_count, 1);
    assert_eq!(loaded.manifest.origin, Origin::SyntheticFixture);
    assert_eq!(loaded.manifest.eligibility, Eligibility::Eligible);
    assert!(!loaded.manifest.benchmark_plan.included);
    assert!(
        loaded
            .manifest
            .files
            .iter()
            .all(|file| file.path != "manifest.json")
    );
    let input_bytes = fs::read(out.join("input-manifest.json")).unwrap();
    assert_eq!(sha256(&input_bytes), loaded.manifest.input_manifest_sha256);

    let gapdh = one(&loaded.records, "S1", "GAPDH");
    assert_eq!(gapdh.value.as_deref(), Some("0"));
    assert_eq!(gapdh.missing_reason, None);
    assert_eq!(gapdh.modality, Modality::Transcript);
    assert_eq!(gapdh.unit, "count");
    let actb = one(&loaded.records, "S1", "ACTB");
    assert_eq!(actb.value, None);
    assert_eq!(actb.missing_reason, Some(MissingReason::NotMeasured));
    assert_eq!(actb.censor_limit, None);
    let tp53 = one(&loaded.records, "S1", "TP53");
    assert_eq!(tp53.value.as_deref(), Some("10.5"));
    assert_eq!(tp53.modality, Modality::IntracellularProtein);
    let il6 = one(&loaded.records, "S1", "IL6");
    assert_eq!(il6.value.as_deref(), Some("1250"));
    assert_eq!(il6.unit, "pg_per_ml");
    assert_eq!(il6.modality, Modality::SecretedProtein);
    let other = one(&loaded.records, "S2", "IL6");
    assert_eq!(other.value.as_deref(), Some("1250"));
    assert_ne!(il6.sample_id, other.sample_id);
    assert_ne!(il6.record_id, other.record_id);
    let tnf = one(&loaded.records, "S1", "TNF");
    assert_eq!(tnf.value, None);
    assert_eq!(tnf.missing_reason, Some(MissingReason::BelowDetection));
    assert_eq!(tnf.censor_limit.as_deref(), Some("0.8"));
    let lactate = one(&loaded.records, "S1", "LACTATE");
    assert_eq!(lactate.value.as_deref(), Some("3.25"));
    assert_eq!(lactate.modality, Modality::Metabolite);
    let viability = one(&loaded.records, "S1", "VIABILITY");
    assert_eq!(viability.value.as_deref(), Some("95"));
    assert_eq!(viability.modality, Modality::FunctionalAssay);
    let failed = one(&loaded.records, "S2", "ACTB");
    assert_eq!(failed.missing_reason, Some(MissingReason::FailedQc));
    assert_eq!(failed.qc_failure.as_deref(), Some("low_reads"));
    assert!(loaded.records.iter().all(|record| record.sample_id != "S0"));
    let quarantine: Value =
        serde_json::from_slice(&fs::read(out.join("quarantine.json")).unwrap()).unwrap();
    assert_eq!(quarantine[0]["reason"], "unknown_donor_not_inferred");
    assert_eq!(quarantine[0]["sample_id"], "S0");
    let names = [
        "transcript",
        "intracellular_protein",
        "secreted_protein",
        "metabolite",
        "functional_assay",
    ];
    for name in names {
        assert!(
            loaded
                .records
                .iter()
                .any(|record| record.modality.as_str() == name),
            "{name}"
        );
    }
    let hidden = write_bytes(dir.path(), "present.json", &request(false));
    let (_, present) = observe_dataset(&out, &hidden).unwrap();
    assert!(present.iter().all(|record| record.missing_reason.is_none()));
    assert!(
        present
            .iter()
            .any(|record| record.value.as_deref() == Some("0"))
    );
    assert!(present.iter().all(|record| record.analyte_id != "TNF"));
    evidence(
        "round-trip.json",
        &json!({
            "measurement_count": loaded.manifest.measurement_count,
            "deduplicated_count": loaded.manifest.deduplicated_count,
            "quarantined_count": loaded.manifest.quarantined_count,
            "zero_value": "0",
            "not_measured_value": null,
            "below_detection_censor_limit": "0.8",
            "converted_il6_pg_per_ml": "1250",
            "trailing_zero_examples": {"10.50": "10.5", "3.250": "3.25", "0.80": "0.8"},
            "parquet_equals_zarr": true,
            "benchmark_plan_included": false,
            "physiological_validation": "not_performed"
        }),
    );
}

#[test]
fn a01_02_reads_one_chunk_for_a_small_slice() {
    let dir = tempfile::tempdir().unwrap();
    let (csv, samples) = bulk_table(200);
    let meta = Meta {
        samples: &samples,
        split: false,
        dataset_id: "fixture-chunks",
        ..happy_meta()
    };
    import_bytes(dir.path(), "rows.csv", csv.as_bytes(), "csv", &meta, true).unwrap();
    let array = dir.path().join("dataset/arrays/measurements");
    let narrow = read_slice(&array, 5, 7).unwrap();
    let single = read_slice(&array, 5, 6).unwrap();
    assert_eq!(narrow.records.len(), 2);
    assert_eq!(narrow.records[0].value.as_deref(), Some("5"));
    assert_eq!(narrow.records[1].value.as_deref(), Some("6"));
    assert_eq!(narrow.bytes_read, single.bytes_read);
    assert_eq!(narrow.bytes_read, max_one_chunk_working_set());
    assert!(
        narrow
            .chunks_opened
            .iter()
            .all(|path| path.ends_with("/0") || path.ends_with("/0.0"))
    );
    assert!(narrow.chunks_opened.iter().all(|path| !path.contains("/1")));
    fs::remove_file(array.join("value/1.0")).unwrap();
    let missing = read_slice(&array, 40, 41).expect_err("missing chunk");
    assert!(
        missing
            .message
            .contains("not interpreted as measured zeros")
    );
    assert_ne!(missing.message, "0");
    let kept = read_slice(&array, 0, 1).unwrap();
    assert_eq!(kept.records[0].value.as_deref(), Some("0"));
    let bytes = fs::read(array.join("value/0.0")).unwrap();
    fs::write(array.join("value/0.0"), &bytes[..10]).unwrap();
    let short = read_slice(&array, 0, 1).expect_err("short chunk");
    assert!(short.message.contains("not padded with a zero measurement"));
    evidence(
        "chunk-access.json",
        &json!({
            "rows": 200,
            "slice": [5, 7],
            "chunks_opened": narrow.chunks_opened,
            "bytes_read": narrow.bytes_read,
            "one_chunk_working_set_bytes": max_one_chunk_working_set(),
            "same_chunk_second_row_added_no_bytes": true,
            "missing_chunk": missing.message,
            "short_chunk": short.message
        }),
    );
}

#[test]
fn a01_03_zero_absence_and_censoring_stay_distinct() {
    let (_dir, out) = import_happy(true);
    let records = observe_dataset(
        &out,
        &write_bytes(_dir.path(), "request.json", &request(true)),
    )
    .unwrap()
    .1;
    let zero = one(&records, "S1", "GAPDH");
    let absent = one(&records, "S1", "ACTB");
    let censored = one(&records, "S1", "TNF");
    assert_eq!(zero.value.as_deref(), Some("0"));
    assert!(absent.value.is_none() && absent.missing_reason == Some(MissingReason::NotMeasured));
    assert!(censored.value.is_none() && censored.censor_limit.as_deref() == Some("0.8"));
    assert_ne!(zero.value, absent.value);
    assert_ne!(absent.missing_reason, censored.missing_reason);
}

#[test]
fn a01_04_rejects_conflicts_without_a_manifest() {
    let cases = [
        (
            "unit",
            "S1,IL6,1,count,secreted_protein,,,,24,hour,stimulation_start,elisa-1,v1,extracellular\n",
            "no explicit conversion",
        ),
        (
            "conflict",
            "S1,IL6,1.25,ng_per_ml,secreted_protein,,,,24,hour,stimulation_start,elisa-1,v1,extracellular\nS1,IL6,1.50,ng_per_ml,secreted_protein,,,,24,hour,stimulation_start,elisa-1,v1,extracellular\n",
            "conflicting",
        ),
        (
            "orphan",
            "S9,IL6,1,pg_per_ml,secreted_protein,,,,24,hour,stimulation_start,elisa-1,v1,extracellular\n",
            "not in the metadata",
        ),
        (
            "analyte",
            "S1,UNKNOWN,1,count,transcript,,,,24,hour,stimulation_start,rna-1,v1,intracellular\n",
            "not in the mapping",
        ),
        (
            "modality",
            "S1,GAPDH,1,count,metabolite,,,,24,hour,stimulation_start,rna-1,v1,intracellular\n",
            "not repaired",
        ),
    ];
    for (name, body, needle) in cases {
        let dir = tempfile::tempdir().unwrap();
        let mut csv = String::from(HEADER);
        csv.push('\n');
        csv.push_str(body);
        let error = import_bytes(
            dir.path(),
            "rows.csv",
            csv.as_bytes(),
            "csv",
            &happy_meta(),
            true,
        )
        .expect_err(name);
        assert_eq!(error.exit_code, 2, "{name}: {}", error.message);
        assert!(error.message.contains(needle), "{name}: {}", error.message);
        assert!(!dir.path().join("dataset").exists(), "{name}");
    }
    let dir = tempfile::tempdir().unwrap();
    let json = br#"{"records":[{"sample":"S1","analyte":"GAPDH","value":1,"unit":"count","modality":"transcript","missing_reason":null,"censor_limit":null,"qc_failure":null,"time_amount":"24","time_unit":"hour","time_origin":"stimulation_start","assay_id":"rna-1","protocol_version":"v1","compartment":"intracellular"}]}"#;
    let error =
        import_bytes(dir.path(), "rows.json", json, "json", &happy_meta(), true).unwrap_err();
    assert_eq!(error.exit_code, 2);
    assert!(error.message.contains("Numbers are not coerced"));
    assert!(!dir.path().join("dataset").exists());
}

#[test]
fn a01_05_publication_is_atomic_and_checksums_reject_corruption() {
    let (dir, out) = import_happy(false);
    evidence(
        "interrupted-import.json",
        &json!({"publish": false, "output_exists": out.exists(), "staging_left_behind": false, "parent": dir.path()}),
    );
    let (dir, out) = import_happy(true);
    let again = import_bytes(
        dir.path(),
        "rows.csv",
        HAPPY_CSV.as_bytes(),
        "csv",
        &happy_meta(),
        true,
    )
    .unwrap_err();
    assert_eq!(again.exit_code, 2);
    assert!(again.message.contains("not overwritten"));
    assert!(out.join("manifest.json").is_file());
    let mut bytes = fs::read(out.join("original.bin")).unwrap();
    bytes[0] ^= 0xff;
    fs::write(out.join("original.bin"), bytes).unwrap();
    let error = observe_dataset(
        &out,
        &write_bytes(dir.path(), "request.json", &request(true)),
    )
    .unwrap_err();
    assert_eq!(error.exit_code, 8);
    assert!(error.message.contains("not valid"));
    let bad = tempfile::tempdir().unwrap();
    let mut meta = happy_meta();
    let wrong = "0".repeat(64);
    meta.checksum = Some(&wrong);
    let error = import_bytes(
        bad.path(),
        "rows.csv",
        HAPPY_CSV.as_bytes(),
        "csv",
        &meta,
        true,
    )
    .unwrap_err();
    assert!(
        error
            .message
            .contains("Declared source checksum does not match")
    );
    assert!(error.message.contains("not altered"));
    assert!(!bad.path().join("dataset").exists());
}

#[test]
fn a01_06_unknown_restricted_and_unresolved_rights_are_excluded() {
    let mut decisions = Vec::new();
    for (name, rights, required, note, eligibility) in [
        (
            "unknown",
            "unknown",
            false,
            "rights were not determined",
            "quarantined",
        ),
        (
            "restricted",
            "restricted",
            false,
            "contract limits redistribution",
            "ineligible",
        ),
        (
            "accession",
            "public_domain",
            true,
            "repository accession was not resolved",
            "ineligible",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let meta = Meta {
            rights,
            accession_required: required,
            accession_note: note,
            intended: "independent_benchmark",
            ..happy_meta()
        };
        let outcome = import_bytes(
            dir.path(),
            "rows.csv",
            HAPPY_CSV.as_bytes(),
            "csv",
            &meta,
            true,
        )
        .unwrap();
        assert_eq!(outcome.exit_code, 5, "{name}");
        assert_eq!(outcome.summary["eligibility"], eligibility, "{name}");
        assert_eq!(outcome.summary["benchmark_plan_included"], false, "{name}");
        assert!(dir.path().join("dataset/manifest.json").is_file(), "{name}");
        decisions.push(json!({"case": name, "exit_code": 5, "eligibility": eligibility, "benchmark_plan_included": false}));
    }
    let dir = tempfile::tempdir().unwrap();
    let meta = Meta {
        origin: "experimental",
        rights: "public_domain",
        intended: "independent_benchmark",
        accession_required: false,
        accession_note: "not required for this gate fixture",
        split: false,
        samples: r#"[{"sample_id":"S1","donor_id":"D1","cell_id":"C1","population_id":"POP","replicate_id":"R1","replicate_role":"primary","parent_sample_id":null,"passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"}]"#,
        checksum: None,
        dataset_id: "eligibility-gate-not-a-cohort",
    };
    let csv = format!(
        "{HEADER}\nS1,GAPDH,1,count,transcript,,,,24,hour,stimulation_start,rna-1,v1,intracellular\n"
    );
    let outcome = import_bytes(dir.path(), "rows.csv", csv.as_bytes(), "csv", &meta, true).unwrap();
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.summary["benchmark_plan_included"], true);
    assert_eq!(outcome.summary["origin"], "experimental");
    decisions.push(json!({
        "case": "declared_experimental_public_domain_gate",
        "exit_code": 0,
        "benchmark_plan_included": true,
        "note": "Synthetic rows used only to exercise the rights gate. Not an approved biological dataset."
    }));
    let omitted = tempfile::tempdir().unwrap();
    let meta = Meta {
        accession_required: true,
        accession_note: "",
        ..happy_meta()
    };
    let error = import_bytes(
        omitted.path(),
        "rows.csv",
        HAPPY_CSV.as_bytes(),
        "csv",
        &meta,
        true,
    )
    .unwrap_err();
    assert_eq!(error.exit_code, 2);
    assert!(error.message.contains("cannot be omitted silently"));
    assert!(!omitted.path().join("dataset").exists());
    evidence(
        "license-eligibility.json",
        &json!({"decisions": decisions, "approved_external_datasets": []}),
    );
}

#[test]
fn a01_07_donor_splits_do_not_leak_and_repeat() {
    let (dir, out) = import_happy(true);
    let first: Value = serde_json::from_slice(&fs::read(out.join("splits.json")).unwrap()).unwrap();
    let copy = tempfile::tempdir().unwrap();
    import_bytes(
        copy.path(),
        "rows.csv",
        HAPPY_CSV.as_bytes(),
        "csv",
        &happy_meta(),
        true,
    )
    .unwrap();
    let second: Value =
        serde_json::from_slice(&fs::read(copy.path().join("dataset/splits.json")).unwrap())
            .unwrap();
    assert_eq!(first, second);
    assert_eq!(first["seed"], 11);
    assert_eq!(first["usable_for_donor_holdout"], false);
    assert_eq!(first["holdout_fraction_ppm"], 200_000);
    assert!(
        first["failures"]
            .as_array()
            .unwrap()
            .iter()
            .any(|failure| failure["reason"] == "unknown_donor" && failure["sample_id"] == "S0")
    );
    let assignments = first["assignments"].as_array().unwrap();
    let d1 = assignments
        .iter()
        .find(|item| item["donor_id"] == "D1")
        .unwrap();
    let ids = d1["sample_ids"].as_array().unwrap();
    assert!(ids.iter().any(|id| id == "S1") && ids.iter().any(|id| id == "S1t"));
    assert!(
        assignments
            .iter()
            .all(|item| item["donor_id"] != "D1" || item["split"] == d1["split"])
    );
    assert_eq!(
        assignments
            .iter()
            .filter(|item| item["donor_id"] == "D1")
            .count(),
        1
    );

    let many = tempfile::tempdir().unwrap();
    let samples = r#"[
        {"sample_id":"S1","donor_id":"D1","cell_id":"C1","population_id":"POP","replicate_id":"R1","replicate_role":"primary","parent_sample_id":null,"passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"},
        {"sample_id":"S1b","donor_id":"D1","cell_id":"C2","population_id":"POP","replicate_id":"R1b","replicate_role":"primary","parent_sample_id":null,"passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"},
        {"sample_id":"S1c","donor_id":"D1","cell_id":"C3","population_id":"POP","replicate_id":"R1c","replicate_role":"primary","parent_sample_id":null,"passage":5,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"},
        {"sample_id":"S1t","donor_id":"D1","cell_id":"C1","population_id":"POP","replicate_id":"R1t","replicate_role":"technical","parent_sample_id":"S1","passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"},
        {"sample_id":"S1r","donor_id":"D1","cell_id":"C1","population_id":"POP","replicate_id":"R1r","replicate_role":"repeated_sample","parent_sample_id":"S1","passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"},
        {"sample_id":"S2","donor_id":"D2","cell_id":"C9","population_id":"POP","replicate_id":"R2","replicate_role":"primary","parent_sample_id":null,"passage":4,"tissue":"whartons_jelly","culture_batch":"CB2","experimental_batch":"EB2","perturbation":"stimulated","study_id":"study-fixture","source_identity":"src-d2"}
    ]"#;
    let meta = Meta {
        samples,
        ..happy_meta()
    };
    let csv = format!(
        "{HEADER}\nS1,GAPDH,1,count,transcript,,,,24,hour,stimulation_start,rna-1,v1,intracellular\n"
    );
    import_bytes(many.path(), "rows.csv", csv.as_bytes(), "csv", &meta, true).unwrap();
    let splits: Value =
        serde_json::from_slice(&fs::read(many.path().join("dataset/splits.json")).unwrap())
            .unwrap();
    assert_eq!(splits["usable_for_donor_holdout"], true);
    let grouped = splits["assignments"].as_array().unwrap();
    let donor = grouped
        .iter()
        .find(|item| item["donor_id"] == "D1")
        .unwrap();
    for sample in ["S1", "S1b", "S1c", "S1t", "S1r"] {
        assert!(
            donor["sample_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|id| id == sample)
        );
    }
    assert!(grouped.iter().all(
        |item| item["sample_ids"].as_array().unwrap().iter().all(|id| {
            let text = id.as_str().unwrap();
            (text.starts_with("S1") && item["donor_id"] == "D1")
                || (text == "S2" && item["donor_id"] == "D2")
        })
    ));

    let overlap = tempfile::tempdir().unwrap();
    let samples = r#"[
        {"sample_id":"S1","donor_id":"D1","cell_id":"C1","population_id":"POP","replicate_id":"R1","replicate_role":"primary","parent_sample_id":null,"passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-shared"},
        {"sample_id":"S2","donor_id":"D2","cell_id":"C2","population_id":"POP","replicate_id":"R2","replicate_role":"primary","parent_sample_id":null,"passage":4,"tissue":"whartons_jelly","culture_batch":"CB2","experimental_batch":"EB2","perturbation":"none","study_id":"study-fixture","source_identity":"src-shared"}
    ]"#;
    let meta = Meta {
        samples,
        ..happy_meta()
    };
    import_bytes(
        overlap.path(),
        "rows.csv",
        csv.as_bytes(),
        "csv",
        &meta,
        true,
    )
    .unwrap();
    let splits: Value =
        serde_json::from_slice(&fs::read(overlap.path().join("dataset/splits.json")).unwrap())
            .unwrap();
    assert_eq!(splits["usable_for_donor_holdout"], false);
    assert!(splits["assignments"].as_array().unwrap().is_empty());
    assert!(
        splits["failures"]
            .as_array()
            .unwrap()
            .iter()
            .any(|failure| failure["reason"] == "overlapping_source")
    );

    let rejected = tempfile::tempdir().unwrap();
    let mut text = metadata(&happy_meta());
    text = text.replace(
        r#""holdout_fraction":"0.2","validation_fraction":"0.2""#,
        r#""holdout_fraction":"0.7","validation_fraction":"0.3""#,
    );
    let input = write_bytes(rejected.path(), "rows.csv", HAPPY_CSV.as_bytes());
    let mapping_path = write_bytes(rejected.path(), "mapping.json", mapping("csv").as_bytes());
    let metadata_path = write_bytes(rejected.path(), "metadata.json", text.as_bytes());
    let error = import_dataset(&ImportRequest {
        input,
        mapping: mapping_path,
        metadata: metadata_path,
        out: rejected.path().join("dataset"),
        publish: true,
    })
    .unwrap_err();
    assert_eq!(error.exit_code, 2);
    assert!(!rejected.path().join("dataset").exists());
    evidence(
        "split-leakage.json",
        &json!({
            "stable_across_imports": true,
            "seed": 11,
            "unknown_donor_blocks_holdout": true,
            "technical_and_repeated_samples_share_donor_split": true,
            "overlapping_source_assignments": [],
            "algorithm": first["algorithm"]
        }),
    );
    let _ = dir;
}

#[test]
fn a01_08_migrates_v0_and_rejects_v2() {
    let migrated = read_versioned_measurements(&json!({
        "schema_version": "cyra.measurement.v0",
        "records": [{
            "kind": "secreted",
            "unit": "ng/ml",
            "amount": "1.50",
            "donor": "D1",
            "sample": "S1",
            "analyte": "IL6"
        }]
    }))
    .unwrap();
    assert_eq!(migrated.schema_version, "cyra.measurement.v1");
    assert_eq!(
        migrated.transform_name.as_deref(),
        Some("migrate_measurement_v0_to_v1")
    );
    assert_eq!(migrated.records[0]["unit"], "ng_per_ml");
    assert_eq!(migrated.records[0]["value"], "1.5");
    assert_eq!(migrated.records[0]["modality"], "secreted_protein");
    assert_eq!(migrated.records[0]["migrated_from"], "cyra.measurement.v0");
    let native =
        read_versioned_measurements(&json!({"schema_version":"cyra.measurement.v1","records":[]}))
            .unwrap();
    assert!(native.transform_name.is_none());
    let future = read_versioned_measurements(
        &json!({"schema_version":"cyra.measurement.v2","records":[{"value":"1"}]}),
    )
    .unwrap_err();
    assert_eq!(future.exit_code, 3);
    assert!(future.message.contains("future version"));
    assert!(future.message.contains("No rows were written"));
    let unknown = read_versioned_measurements(&json!({
        "schema_version": "cyra.measurement.v0",
        "records": [{"kind":"secreted","unit":"mystery","amount":"1","donor":"D1","sample":"S1","analyte":"IL6"}]
    }))
    .unwrap_err();
    assert_eq!(unknown.exit_code, 2);
    assert!(unknown.message.contains("not guessed"));
}

#[test]
fn a01_parquet_row_groups_are_bounded() {
    let dir = tempfile::tempdir().unwrap();
    let (csv, samples) = bulk_table(300);
    let meta = Meta {
        samples: &samples,
        split: false,
        dataset_id: "fixture-row-groups",
        ..happy_meta()
    };
    import_bytes(dir.path(), "rows.csv", csv.as_bytes(), "csv", &meta, true).unwrap();
    let path = dir.path().join("dataset/tables/measurements.parquet");
    assert!(row_group_count(&path).unwrap() >= 2);
    let first = read_row_groups(&path, Some(vec![0])).unwrap();
    assert_eq!(first.len(), PARQUET_ROW_GROUP_ROWS);
    assert_eq!(first[0].value.as_deref(), Some("0"));
    assert_eq!(first[0].time_amount.as_deref(), Some("0"));
    assert_eq!(first.last().unwrap().time_amount.as_deref(), Some("255"));
    let all = read_row_groups(&path, None).unwrap();
    assert_eq!(all.len(), 300);
    assert_eq!(all[256].time_amount.as_deref(), Some("256"));
}

#[test]
fn json_and_tsv_import_use_the_same_records() {
    let row = r#"{"sample":"S1","analyte":"GAPDH","value":"2","unit":"count","modality":"transcript","missing_reason":null,"censor_limit":null,"qc_failure":null,"time_amount":"24","time_unit":"hour","time_origin":"stimulation_start","assay_id":"rna-1","protocol_version":"v1","compartment":"intracellular"}"#;
    let json = format!(r#"{{"records":[{row}]}}"#);
    let tsv = format!(
        "{}\nS1\tGAPDH\t2\tcount\ttranscript\t\t\t\t24\thour\tstimulation_start\trna-1\tv1\tintracellular\n",
        HEADER.replace(',', "\t")
    );
    let samples = r#"[{"sample_id":"S1","donor_id":"D1","cell_id":"C1","population_id":"POP","replicate_id":"R1","replicate_role":"primary","parent_sample_id":null,"passage":3,"tissue":"whartons_jelly","culture_batch":"CB1","experimental_batch":"EB1","perturbation":"none","study_id":"study-fixture","source_identity":"src-d1"}]"#;
    let mut values = Vec::new();
    for (format, name, bytes) in [
        ("json", "rows.json", json.into_bytes()),
        ("tsv", "rows.tsv", tsv.into_bytes()),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let meta = Meta {
            samples,
            split: false,
            dataset_id: "fixture-format",
            ..happy_meta()
        };
        let outcome = import_bytes(dir.path(), name, &bytes, format, &meta, true).unwrap();
        assert_eq!(outcome.exit_code, 0);
        let records = observe_dataset(
            &dir.path().join("dataset"),
            &write_bytes(dir.path(), "request.json", &request(true)),
        )
        .unwrap()
        .1;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].value.as_deref(), Some("2"));
        values.push(records[0].value.clone());
    }
    assert_eq!(values[0], values[1]);
}

#[test]
fn provenance_reports_lineage_without_a_remote_lookup() {
    let (dir, out) = import_happy(true);
    let value = provenance(&out.join("tables/measurements.parquet")).unwrap();
    assert_eq!(value["dataset_id"], "fixture-modalities");
    assert_eq!(value["origin"], "synthetic_fixture");
    assert_eq!(value["scientific_computation_performed"], false);
    assert!(
        value["human"]
            .as_str()
            .unwrap()
            .contains("No physiological claim")
    );
    assert!(
        value["unavailable_fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "donor_age")
    );
    assert!(
        value["transforms"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "explicit_unit_conversion")
    );
    assert!(value.get("remote").is_none());
    let _ = dir;
}

#[test]
fn schemas_match_checked_in_contracts() {
    for name in [
        "import-mapping",
        "dataset-manifest",
        "observation-request",
        "measurement-record",
    ] {
        let current = cyra_data::schema(name).unwrap();
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../schemas/{name}.v1.json"));
        let committed: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            current, committed,
            "schema drift in {name}; regenerate deliberately"
        );
        assert_eq!(
            current["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
    }
}

#[test]
fn acquisition_register_approves_nothing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/acquisition/sprint-01-register.json");
    let value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(value["approved"], json!([]));
    assert!(
        value["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["status"] == "candidate"
                && item["suitable_for_independent_benchmark"] == false
                && item["files_downloaded"] == false)
    );
    assert!(
        value["rejected"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["status"] == "rejected")
    );
    let gap = value["gap"]["summary"].as_str().unwrap();
    assert!(gap.contains("No approved"));
    assert!(gap.to_lowercase().contains("functional"));
}

//! Stage, validate, hash and atomically publish a dataset directory.
use crate::decimal::{canonical_decimal, multiply_decimals};
use crate::error::DataError;
use crate::fsutil;
use crate::model::*;
use crate::parquet_table::{self, PARQUET_PROFILE, PARQUET_ROW_GROUP_ROWS};
use crate::split::{self, SplitManifest};
use crate::tabular;
use crate::units::{self, UnitConversion};
use crate::zarr_array::{self, ZARR_CHUNK_OBSERVATIONS, ZARR_PROFILE};
use cyra_core::artifact::{canonical_json, sha256};
use cyra_core::scientific::Identifier;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static STAGING_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct ImportRequest {
    pub input: PathBuf,
    pub mapping: PathBuf,
    pub metadata: PathBuf,
    pub out: PathBuf,
    /// When false, staging is discarded and `out` is not created.
    pub publish: bool,
}

#[derive(Debug, Clone)]
pub struct ImportOutcome {
    pub exit_code: u8,
    pub manifest_path: Option<PathBuf>,
    pub summary: Value,
    pub human: String,
}

struct StagedDir {
    path: PathBuf,
    keep: bool,
}
impl Drop for StagedDir {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

enum RowOutcome {
    // Measurement records are much larger than quarantine records. Boxing keeps
    // the enum from carrying the large variant on every row outcome.
    Accept(Box<MeasurementRecord>),
    Quarantine(QuarantineRecord),
}

pub fn import_dataset(request: &ImportRequest) -> Result<ImportOutcome, DataError> {
    if request.publish && request.out.exists() {
        return Err(DataError::invalid(
            "Output path already exists. Datasets are not overwritten.",
        ));
    }
    let input_bytes = read_input(&request.input)?;
    let mapping_bytes = read_input(&request.mapping)?;
    let metadata_bytes = read_input(&request.metadata)?;
    let mapping: ImportMapping = parse_json(&mapping_bytes, "Mapping")?;
    let metadata: DatasetMetadata = parse_json(&metadata_bytes, "Metadata")?;
    validate_submission(&mapping, &metadata)?;
    if let Some(checksum) = &metadata.sources[0].checksum_sha256
        && checksum != &sha256(&input_bytes)
    {
        return Err(DataError::invalid(
            "Declared source checksum does not match the input bytes. The file was not altered to match.",
        ));
    }
    let rows = tabular::parse_table(&input_bytes, mapping.format)?;
    if rows.is_empty() {
        return Err(DataError::invalid("Input table has no measurement rows."));
    }
    require_columns(&rows, &mapping)?;
    let built = build_records(&rows, &mapping, &metadata, &input_bytes)?;
    let (eligibility, mut benchmark, exit_code) = decide_eligibility(&metadata);
    if !built.quarantine.is_empty() {
        benchmark.included = false;
        benchmark
            .reasons
            .push("quarantined_rows_present".to_owned());
    }
    let splits = metadata
        .split
        .as_ref()
        .map(|split_request| split::assign_donor_splits(&metadata.samples, split_request))
        .transpose()?;
    if let Some(splits) = &splits
        && !splits.usable_for_donor_holdout
    {
        benchmark.included = false;
        benchmark.reasons.push("donor_split_not_usable".to_owned());
    }
    let samples = metadata
        .samples
        .iter()
        .cloned()
        .map(|fields| SampleRecord {
            included_in_measurements: built
                .records
                .iter()
                .any(|record| record.sample_id == fields.sample_id),
            fields,
        })
        .collect::<Vec<_>>();

    let parent = request
        .out
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = request
        .out
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| DataError::invalid("Output path needs a UTF-8 file name."))?;
    if file_name.starts_with('.') {
        return Err(DataError::invalid(
            "Output directory name cannot be hidden.",
        ));
    }
    let staging_path = parent.join(format!(
        ".{file_name}.cyra-stage-{}-{}",
        std::process::id(),
        STAGING_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fsutil::create_private_dir(&staging_path).map_err(|error| DataError::io(error.to_string()))?;
    let mut staged = StagedDir {
        path: staging_path,
        keep: false,
    };
    write_tree(
        &staged.path,
        &input_bytes,
        &mapping_bytes,
        &metadata_bytes,
        &mapping,
        &metadata,
        &built,
        splits.as_ref(),
        &samples,
        eligibility,
        &benchmark,
    )?;
    if !request.publish {
        return Err(DataError::io(
            "Ingestion stopped before publication. No dataset directory was created.",
        ));
    }
    fs::rename(&staged.path, &request.out).map_err(|error| DataError::io(error.to_string()))?;
    staged.keep = true;
    let manifest_bytes = fsutil::read_relative(&request.out, "manifest.json")
        .map_err(|error| DataError::io(error.to_string()))?;
    let manifest: DatasetManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| DataError::io("Published manifest is unreadable."))?;
    let manifest_sha = sha256(&manifest_bytes);
    let mut summary = summary_value(&manifest, &manifest_sha, true);
    summary["modalities"] = json!(modality_names(&built.records));
    let human = human_summary(
        &manifest,
        &manifest_sha,
        Some(&request.out.join("manifest.json")),
    );
    Ok(ImportOutcome {
        exit_code,
        manifest_path: Some(request.out.join("manifest.json")),
        summary,
        human,
    })
}

#[allow(clippy::too_many_arguments)]
fn write_tree(
    root: &Path,
    input_bytes: &[u8],
    mapping_bytes: &[u8],
    metadata_bytes: &[u8],
    mapping: &ImportMapping,
    metadata: &DatasetMetadata,
    built: &Built,
    splits: Option<&SplitManifest>,
    samples: &[SampleRecord],
    eligibility: Eligibility,
    benchmark: &BenchmarkPlan,
) -> Result<(), DataError> {
    let input_manifest = json!({
        "schema_version": "cyra.input-manifest.v1",
        "dataset_id": metadata.dataset_id,
        "importer_revision": crate::IMPORTER_REVISION,
        "original_sha256": sha256(input_bytes),
        "mapping_sha256": sha256(mapping_bytes),
        "metadata_sha256": sha256(metadata_bytes),
        "original_bytes": input_bytes.len(),
        "mapping_bytes": mapping_bytes.len(),
        "metadata_bytes": metadata_bytes.len()
    });
    write_canonical(&root.join("input-manifest.json"), &input_manifest)?;
    fsutil::write_private(&root.join("original.bin"), input_bytes)
        .map_err(|error| DataError::io(error.to_string()))?;
    write_canonical(&root.join("mapping.json"), &to_value(mapping)?)?;
    write_canonical(&root.join("metadata.json"), &to_value(metadata)?)?;
    fsutil::create_private_dir(&root.join("tables"))
        .map_err(|error| DataError::io(error.to_string()))?;
    let parquet_path = root.join("tables/measurements.parquet");
    parquet_table::write_table(&parquet_path, &built.records)?;
    fs::set_permissions(&parquet_path, std::fs::Permissions::from_mode(0o600))
        .map_err(|error| DataError::io(error.to_string()))?;
    fsutil::create_private_dir(&root.join("arrays"))
        .map_err(|error| DataError::io(error.to_string()))?;
    zarr_array::write_array(&root.join("arrays/measurements"), &built.records)?;
    write_canonical(&root.join("quarantine.json"), &to_value(&built.quarantine)?)?;
    if let Some(splits) = splits {
        write_canonical(&root.join("splits.json"), &to_value(splits)?)?;
    }
    let input_manifest_bytes = fsutil::read_relative(root, "input-manifest.json")
        .map_err(|error| DataError::io(error.to_string()))?;
    let mut files = Vec::new();
    collect_tree(root, root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let manifest = DatasetManifest {
        schema_version: DATASET_SCHEMA.to_owned(),
        dataset_id: metadata.dataset_id.clone(),
        origin: metadata.origin,
        eligibility,
        benchmark_plan: benchmark.clone(),
        importer_revision: crate::IMPORTER_REVISION.to_owned(),
        input_manifest_sha256: sha256(&input_manifest_bytes),
        representation: Representation {
            numeric: "canonical decimal".to_owned(),
            trailing_fractional_zeros: "insignificant".to_owned(),
            binary_float: "not_used".to_owned(),
            tolerance: "exact_decimal_value".to_owned(),
        },
        storage_profiles: StorageProfiles {
            parquet: PARQUET_PROFILE.to_owned(),
            zarr: ZARR_PROFILE.to_owned(),
            parquet_compression: "uncompressed".to_owned(),
            zarr_codecs: vec!["uncompressed".to_owned()],
            parquet_row_group_rows: PARQUET_ROW_GROUP_ROWS,
            zarr_chunk_observations: ZARR_CHUNK_OBSERVATIONS,
        },
        licenses: metadata.licenses.clone(),
        accession: metadata.accession.clone(),
        sources: stamp_source(&metadata.sources, &sha256(input_bytes)),
        samples: samples.to_vec(),
        transforms: built.transforms.clone(),
        measurement_count: built.records.len(),
        quarantined_count: built.quarantine.len(),
        deduplicated_count: built.deduplicated,
        unavailable_fields: metadata.unavailable_fields.clone(),
        files,
        limitations: vec![
            "Ingestion does not impute, identify a causal effect, or validate physiology."
                .to_owned(),
            "A published manifest can still be ineligible or quarantined.".to_owned(),
            "Synthetic fixtures and unresolved rights are excluded from benchmark plans."
                .to_owned(),
        ],
    };
    write_canonical(&root.join("manifest.json"), &to_value(&manifest)?)?;
    fsutil::sync_dir(root).map_err(|error| DataError::io(error.to_string()))?;
    Ok(())
}

struct Built {
    records: Vec<MeasurementRecord>,
    quarantine: Vec<QuarantineRecord>,
    transforms: Vec<TransformRecord>,
    deduplicated: usize,
}

fn build_records(
    rows: &[BTreeMap<String, String>],
    mapping: &ImportMapping,
    metadata: &DatasetMetadata,
    input_bytes: &[u8],
) -> Result<Built, DataError> {
    let mut records = Vec::new();
    let mut quarantine = Vec::new();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    let mut deduplicated = 0usize;
    let mut conversions_used = BTreeSet::new();
    let input_sha = sha256(input_bytes);
    let source_id = &metadata.sources[0].source_id;
    for (index, row) in rows.iter().enumerate() {
        match one_record(
            index,
            row,
            mapping,
            metadata,
            source_id,
            &input_sha,
            &mut conversions_used,
        )? {
            RowOutcome::Accept(record) => {
                let key = record.biological_key();
                let digest = content_digest(record.as_ref())?;
                if let Some(previous) = seen.get(&key) {
                    if previous == &digest {
                        deduplicated += 1;
                        continue;
                    }
                    return Err(DataError::invalid(
                        "Duplicate biological measurement id with conflicting values. No dataset was published and distinct samples were not merged.",
                    ));
                }
                seen.insert(key, digest);
                records.push(*record);
            }
            RowOutcome::Quarantine(record) => quarantine.push(record),
        }
    }
    let mut transforms = vec![TransformRecord {
        transform_id: "normalize-import".to_owned(),
        name: "normalize_import".to_owned(),
        software: crate::IMPORTER_REVISION.to_owned(),
        input_sha256: vec![input_sha.clone()],
        note: "Explicit column mapping. Missing donors were not inferred and units were not repaired silently.".to_owned(),
    }];
    for conversion in &mapping.conversions {
        let marker = conversion_marker(conversion);
        if conversions_used.contains(&marker) {
            transforms.push(TransformRecord {
                transform_id: format!("convert-{marker}"),
                name: "explicit_unit_conversion".to_owned(),
                software: crate::IMPORTER_REVISION.to_owned(),
                input_sha256: vec![input_sha.clone()],
                note: format!(
                    "Applied factor {} from {} to {} for analyte {}.",
                    conversion.factor,
                    conversion.from_unit,
                    conversion.to_unit,
                    conversion.analyte_id
                ),
            });
        }
    }
    if deduplicated > 0 {
        transforms.push(TransformRecord {
            transform_id: "deduplicate-identical".to_owned(),
            name: "deduplicate_identical_content".to_owned(),
            software: crate::IMPORTER_REVISION.to_owned(),
            input_sha256: vec![input_sha],
            note: "Identical measurement content was stored once. Distinct sample ids were not merged.".to_owned(),
        });
    }
    Ok(Built {
        records,
        quarantine,
        transforms,
        deduplicated,
    })
}

fn one_record(
    index: usize,
    row: &BTreeMap<String, String>,
    mapping: &ImportMapping,
    metadata: &DatasetMetadata,
    source_id: &str,
    input_sha: &str,
    conversions_used: &mut BTreeSet<String>,
) -> Result<RowOutcome, DataError> {
    let sample_text = cell(row, &mapping.columns.sample_id);
    let analyte_text = cell(row, &mapping.columns.analyte_id);
    let quarantine = |reason: &str| {
        Ok(RowOutcome::Quarantine(QuarantineRecord {
            row_number: index + 1,
            reason: reason.to_owned(),
            sample_id: nonempty(sample_text),
            analyte_id: nonempty(analyte_text),
        }))
    };
    if sample_text.is_empty() || analyte_text.is_empty() {
        return quarantine("incomplete_record");
    }
    let Some(sample) = metadata
        .samples
        .iter()
        .find(|sample| sample.sample_id == sample_text)
    else {
        return Err(DataError::invalid(format!(
            "Row {} references sample '{sample_text}', which is not in the metadata. No sample was created.",
            index + 1
        )));
    };
    if sample.donor_id.is_none() {
        return quarantine("unknown_donor_not_inferred");
    }
    let Some(analyte) = mapping.analytes.get(analyte_text) else {
        return Err(DataError::invalid(format!(
            "Analyte '{analyte_text}' is not in the mapping. Modality and units were not inferred."
        )));
    };
    let modality = mapped_modality(row, mapping, analyte.modality)?;
    let (unit, value, censor_limit) = mapped_values(
        row,
        mapping,
        analyte_text,
        &analyte.unit,
        modality,
        conversions_used,
    )?;
    let missing = optional_enum::<MissingReason>(row, mapping.columns.missing_reason.as_deref())?;
    if value.is_some() && missing.is_some() {
        return Err(DataError::invalid(
            "A row contains both a value and a missing reason. Neither was chosen and no dataset was published.",
        ));
    }
    if missing == Some(MissingReason::BelowDetection) && censor_limit.is_none() {
        return quarantine("censor_limit_required");
    }
    if missing.is_none() && value.is_none() {
        return quarantine("incomplete_value");
    }
    if missing.is_some() && missing != Some(MissingReason::BelowDetection) && censor_limit.is_some()
    {
        return Err(DataError::invalid(
            "A censor limit is only valid for below-detection missingness.",
        ));
    }
    let time_amount = optional_decimal(row, mapping.columns.time_amount.as_deref())?;
    let time_unit = optional_enum::<TimeUnit>(row, mapping.columns.time_unit.as_deref())?;
    if time_amount.is_some() != time_unit.is_some() {
        return Err(DataError::invalid(
            "Assay time needs both an amount and a unit. Time units are not assumed.",
        ));
    }
    let assay_id = optional_text(row, mapping.columns.assay_id.as_deref());
    if let Some(assay) = &assay_id {
        check_id(assay, "assay id")?;
    }
    let mut record = MeasurementRecord {
        record_id: String::new(),
        sample_id: sample_text.to_owned(),
        analyte_id: analyte_text.to_owned(),
        modality,
        quantity_kind: units::compatible(&unit, modality)?,
        unit,
        value,
        missing_reason: missing,
        censor_limit,
        qc_failure: optional_text(row, mapping.columns.qc_failure.as_deref()),
        time_amount,
        time_unit,
        time_origin: optional_text(row, mapping.columns.time_origin.as_deref()),
        assay_id,
        protocol_version: optional_text(row, mapping.columns.protocol_version.as_deref()),
        compartment: optional_enum(row, mapping.columns.compartment.as_deref())?,
        lineage: LineageRef {
            sample_id: sample_text.to_owned(),
            source_id: source_id.to_owned(),
            input_sha256: input_sha.to_owned(),
        },
    };
    record.record_id = format!("m.{}", &content_digest(&record)?[..32]);
    Ok(RowOutcome::Accept(Box::new(record)))
}

fn mapped_modality(
    row: &BTreeMap<String, String>,
    mapping: &ImportMapping,
    declared: Modality,
) -> Result<Modality, DataError> {
    let Some(column) = mapping.columns.modality.as_deref() else {
        return Ok(declared);
    };
    let text = cell(row, column);
    if text.is_empty() {
        return Ok(declared);
    }
    let parsed: Modality = parse_enum(text)?;
    if parsed != declared {
        return Err(DataError::invalid(
            "Row modality disagrees with the analyte mapping. The conflict was not repaired.",
        ));
    }
    Ok(parsed)
}

fn mapped_values(
    row: &BTreeMap<String, String>,
    mapping: &ImportMapping,
    analyte_id: &str,
    declared_unit: &str,
    modality: Modality,
    conversions_used: &mut BTreeSet<String>,
) -> Result<(String, Option<String>, Option<String>), DataError> {
    let raw_unit = mapping
        .columns
        .unit
        .as_deref()
        .map(|column| cell(row, column))
        .filter(|text| !text.is_empty())
        .unwrap_or(declared_unit);
    let value = optional_decimal(row, Some(&mapping.columns.value))?;
    let censor = optional_decimal(row, mapping.columns.censor_limit.as_deref())?;
    if raw_unit == declared_unit {
        units::compatible(declared_unit, modality)?;
        return Ok((declared_unit.to_owned(), value, censor));
    }
    let Some(conversion) = mapping.conversions.iter().find(|conversion| {
        conversion.analyte_id == analyte_id
            && conversion.from_unit == raw_unit
            && conversion.to_unit == declared_unit
    }) else {
        return Err(DataError::invalid(format!(
            "Unit '{raw_unit}' is incompatible with declared unit '{declared_unit}' and no explicit conversion was supplied. No unit was repaired."
        )));
    };
    units::conversion_allowed(conversion, modality)?;
    conversions_used.insert(conversion_marker(conversion));
    let factor = canonical_decimal(&conversion.factor)?;
    let value = value
        .map(|value| multiply_decimals(&value, &factor))
        .transpose()?;
    let censor = censor
        .map(|value| multiply_decimals(&value, &factor))
        .transpose()?;
    Ok((declared_unit.to_owned(), value, censor))
}

fn conversion_marker(conversion: &UnitConversion) -> String {
    format!(
        "{}|{}|{}",
        conversion.analyte_id, conversion.from_unit, conversion.to_unit
    )
}

fn validate_submission(
    mapping: &ImportMapping,
    metadata: &DatasetMetadata,
) -> Result<(), DataError> {
    if mapping.schema_version != MAPPING_SCHEMA {
        return Err(DataError::unsupported(format!(
            "Unsupported mapping schema '{}'. Expected {MAPPING_SCHEMA}.",
            mapping.schema_version
        )));
    }
    if metadata.schema_version != METADATA_SCHEMA {
        return Err(DataError::unsupported(format!(
            "Unsupported metadata schema '{}'. Expected {METADATA_SCHEMA}.",
            metadata.schema_version
        )));
    }
    check_id(&metadata.dataset_id, "dataset id")?;
    if metadata.tissue_context.is_empty() || metadata.tissue_context.len() > 128 {
        return Err(DataError::invalid(
            "Tissue context must be 1..128 characters.",
        ));
    }
    if metadata.sources.len() != 1 {
        return Err(DataError::invalid(
            "Version 1 ingestion requires exactly one source record.",
        ));
    }
    check_id(&metadata.sources[0].source_id, "source id")?;
    if let Some(checksum) = &metadata.sources[0].checksum_sha256 {
        check_sha256(checksum)?;
    }
    if metadata.licenses.len() != 3
        || !metadata
            .licenses
            .iter()
            .any(|license| license.domain == LicenseDomain::Dataset)
        || !metadata
            .licenses
            .iter()
            .any(|license| license.domain == LicenseDomain::Code)
        || !metadata
            .licenses
            .iter()
            .any(|license| license.domain == LicenseDomain::ModelWeight)
    {
        return Err(DataError::invalid(
            "Metadata must include distinct dataset, code and model-weight license records.",
        ));
    }
    if metadata
        .licenses
        .iter()
        .any(|license| license.note.trim().is_empty())
    {
        return Err(DataError::invalid(
            "Each license record needs an explicit note.",
        ));
    }
    if metadata.accession.required
        && metadata
            .accession
            .value
            .as_deref()
            .is_none_or(str::is_empty)
        && metadata.accession.note.trim().is_empty()
    {
        return Err(DataError::invalid(
            "A required accession needs either a value or an explicit unresolved note. It cannot be omitted silently.",
        ));
    }
    if let Some(accession) = metadata
        .accession
        .value
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        check_id(accession, "accession")?;
    }
    let mut sample_ids = BTreeSet::new();
    for sample in &metadata.samples {
        check_id(&sample.sample_id, "sample id")?;
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(DataError::invalid(
                "Duplicate sample id in metadata. Samples were not merged.",
            ));
        }
        for (label, value) in [
            ("donor id", sample.donor_id.as_deref()),
            ("cell id", sample.cell_id.as_deref()),
            ("population id", sample.population_id.as_deref()),
            ("replicate id", sample.replicate_id.as_deref()),
            ("culture batch", sample.culture_batch.as_deref()),
            ("experimental batch", sample.experimental_batch.as_deref()),
            ("perturbation", sample.perturbation.as_deref()),
            ("study id", sample.study_id.as_deref()),
            ("source identity", sample.source_identity.as_deref()),
            ("parent sample", sample.parent_sample_id.as_deref()),
        ] {
            if let Some(value) = value {
                check_id(value, label)?;
            }
        }
        if let Some(tissue) = &sample.tissue
            && (tissue.is_empty() || tissue.len() > 64)
        {
            return Err(DataError::invalid(
                "Sample tissue must be 1..64 characters when present.",
            ));
        }
    }
    for sample in &metadata.samples {
        if sample.replicate_role != ReplicateRole::Primary && sample.parent_sample_id.is_none() {
            return Err(DataError::invalid(
                "Technical and repeated samples must name a parent sample.",
            ));
        }
        if let Some(parent_id) = &sample.parent_sample_id {
            let Some(parent) = metadata
                .samples
                .iter()
                .find(|candidate| &candidate.sample_id == parent_id)
            else {
                return Err(DataError::invalid(
                    "A sample parent is not in the metadata. The relationship was not invented.",
                ));
            };
            if sample.donor_id != parent.donor_id {
                return Err(DataError::invalid(
                    "A replicate donor does not match its parent. Donor identities were not merged.",
                ));
            }
        }
    }
    for (analyte_id, analyte) in &mapping.analytes {
        check_id(analyte_id, "analyte id")?;
        units::compatible(&analyte.unit, analyte.modality)?;
    }
    for conversion in &mapping.conversions {
        let Some(analyte) = mapping.analytes.get(&conversion.analyte_id) else {
            return Err(DataError::invalid(
                "A conversion names an analyte that is not in the mapping.",
            ));
        };
        if conversion.to_unit != analyte.unit {
            return Err(DataError::invalid(
                "A conversion must target the analyte's declared unit.",
            ));
        }
        units::conversion_allowed(conversion, analyte.modality)?;
    }
    for field in &metadata.unavailable_fields {
        if field.is_empty()
            || field.len() > 64
            || !field
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(DataError::invalid(
                "Unavailable field names must be short lowercase identifiers.",
            ));
        }
    }
    Ok(())
}

fn decide_eligibility(metadata: &DatasetMetadata) -> (Eligibility, BenchmarkPlan, u8) {
    let dataset = metadata
        .licenses
        .iter()
        .find(|license| license.domain == LicenseDomain::Dataset)
        .expect("dataset license presence is validated");
    let mut reasons = Vec::new();
    let eligibility = if metadata.accession.required
        && metadata
            .accession
            .value
            .as_deref()
            .is_none_or(str::is_empty)
    {
        reasons.push("required_accession_unresolved".to_owned());
        Eligibility::Ineligible
    } else {
        match dataset.rights_basis {
            RightsBasis::Unknown => {
                reasons.push("dataset_rights_unknown".to_owned());
                Eligibility::Quarantined
            }
            RightsBasis::Restricted => {
                reasons.push("dataset_rights_restricted".to_owned());
                Eligibility::Ineligible
            }
            RightsBasis::SyntheticFixtureAuthorization
            | RightsBasis::PublicDomain
            | RightsBasis::PermissiveReuse => Eligibility::Eligible,
        }
    };
    let mut benchmark_ok = eligibility == Eligibility::Eligible
        && metadata.origin == Origin::Experimental
        && dataset.intended_use == IntendedUse::IndependentBenchmark
        && matches!(
            dataset.rights_basis,
            RightsBasis::PublicDomain | RightsBasis::PermissiveReuse
        );
    if metadata.origin != Origin::Experimental {
        benchmark_ok = false;
        reasons.push("origin_is_not_an_independent_experiment".to_owned());
    }
    if dataset.intended_use != IntendedUse::IndependentBenchmark {
        benchmark_ok = false;
        reasons.push("intended_use_is_not_independent_benchmark".to_owned());
    }
    if !matches!(
        dataset.rights_basis,
        RightsBasis::PublicDomain | RightsBasis::PermissiveReuse
    ) {
        benchmark_ok = false;
        reasons.push("rights_do_not_authorize_benchmark_reuse".to_owned());
    }
    let exit_code = if eligibility == Eligibility::Eligible {
        0
    } else {
        5
    };
    (
        eligibility,
        BenchmarkPlan {
            included: benchmark_ok,
            reasons,
        },
        exit_code,
    )
}

fn require_columns(
    rows: &[BTreeMap<String, String>],
    mapping: &ImportMapping,
) -> Result<(), DataError> {
    let mut required = vec![
        mapping.columns.sample_id.clone(),
        mapping.columns.analyte_id.clone(),
        mapping.columns.value.clone(),
    ];
    for column in [
        mapping.columns.unit.clone(),
        mapping.columns.modality.clone(),
        mapping.columns.missing_reason.clone(),
        mapping.columns.censor_limit.clone(),
        mapping.columns.qc_failure.clone(),
        mapping.columns.time_amount.clone(),
        mapping.columns.time_unit.clone(),
        mapping.columns.time_origin.clone(),
        mapping.columns.assay_id.clone(),
        mapping.columns.protocol_version.clone(),
        mapping.columns.compartment.clone(),
    ]
    .into_iter()
    .flatten()
    {
        required.push(column);
    }
    for name in required {
        if rows.iter().any(|row| !row.contains_key(&name)) {
            return Err(DataError::invalid(format!(
                "Mapped column '{name}' is not present in the table. It was not filled in."
            )));
        }
    }
    Ok(())
}

fn content_digest(record: &MeasurementRecord) -> Result<String, DataError> {
    let value = json!({
        "sample_id": record.sample_id,
        "analyte_id": record.analyte_id,
        "modality": record.modality.as_str(),
        "unit": record.unit,
        "value": record.value,
        "missing_reason": record.missing_reason,
        "censor_limit": record.censor_limit,
        "qc_failure": record.qc_failure,
        "time_amount": record.time_amount,
        "time_unit": record.time_unit.map(TimeUnit::as_str),
        "time_origin": record.time_origin,
        "assay_id": record.assay_id,
        "protocol_version": record.protocol_version,
        "compartment": record.compartment
    });
    Ok(sha256(
        &canonical_json(&value).map_err(|error| DataError::invalid(error.to_string()))?,
    ))
}

fn stamp_source(sources: &[SourceRecord], table_sha: &str) -> Vec<SourceRecord> {
    sources
        .iter()
        .cloned()
        .map(|mut source| {
            if source.checksum_sha256.is_none() {
                source.checksum_sha256 = Some(table_sha.to_owned());
            }
            source
        })
        .collect()
}

fn collect_tree(
    root: &Path,
    directory: &Path,
    files: &mut Vec<FileDigest>,
) -> Result<(), DataError> {
    let entries = fs::read_dir(directory).map_err(|error| DataError::io(error.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|error| DataError::io(error.to_string()))?;
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|error| DataError::io(error.to_string()))?;
        if kind.is_symlink() {
            return Err(DataError::invalid(
                "A dataset entry is a symlink and was not published.",
            ));
        }
        if kind.is_dir() {
            collect_tree(root, &path, files)?;
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| DataError::invalid("A staged file escaped the dataset directory."))?;
        let relative = relative
            .to_str()
            .ok_or_else(|| DataError::invalid("A staged path is not UTF-8."))?;
        if relative == "manifest.json" {
            continue;
        }
        let bytes = fsutil::read_relative(root, relative)
            .map_err(|error| DataError::io(error.to_string()))?;
        files.push(FileDigest {
            path: relative.to_owned(),
            sha256: sha256(&bytes),
            size_bytes: bytes.len() as u64,
        });
    }
    Ok(())
}

pub fn summary_value(manifest: &DatasetManifest, manifest_sha: &str, published: bool) -> Value {
    json!({
        "dataset_id": manifest.dataset_id,
        "eligibility": manifest.eligibility,
        "benchmark_plan_included": manifest.benchmark_plan.included,
        "benchmark_reasons": manifest.benchmark_plan.reasons,
        "measurement_count": manifest.measurement_count,
        "quarantined_count": manifest.quarantined_count,
        "deduplicated_count": manifest.deduplicated_count,
        "input_manifest_sha256": manifest.input_manifest_sha256,
        "result_manifest_sha256": manifest_sha,
        "origin": manifest.origin,
        "licenses": manifest.licenses,
        "unavailable_fields": manifest.unavailable_fields,
        "storage": {"parquet": PARQUET_PROFILE, "zarr": ZARR_PROFILE},
        "scientific_computation_performed": false,
        "published": published
    })
}

pub fn human_summary(
    manifest: &DatasetManifest,
    manifest_sha: &str,
    manifest_path: Option<&Path>,
) -> String {
    let mut text = format!(
        "Dataset {}\neligibility: {}\nmeasurements: {}\nquarantined: {}\nbenchmark plan: {}\norigin: {}\nresult manifest sha256: {}\nNo physiological claim is made.",
        manifest.dataset_id,
        serde_plain(&manifest.eligibility),
        manifest.measurement_count,
        manifest.quarantined_count,
        if manifest.benchmark_plan.included {
            "included"
        } else {
            "excluded"
        },
        serde_plain(&manifest.origin),
        manifest_sha
    );
    if let Some(path) = manifest_path {
        text.push_str(&format!("\nmanifest: {}", path.display()));
    }
    text
}

fn serde_plain<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_owned())
}

fn read_input(path: &Path) -> Result<Vec<u8>, DataError> {
    let bytes = fs::read(path)
        .map_err(|_| DataError::io(format!("Could not read '{}'.", path.display())))?;
    if bytes.len() > fsutil::MAX_FILE_BYTES {
        return Err(DataError::invalid("An input file exceeds the byte limit."));
    }
    Ok(bytes)
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8], what: &str) -> Result<T, DataError> {
    let value = cyra_core::strict_json::parse(bytes)
        .map_err(|_| DataError::invalid(format!("{what} is not strict JSON.")))?;
    serde_json::from_value(value)
        .map_err(|_| DataError::invalid(format!("{what} does not match its v1 schema.")))
}

fn write_canonical(path: &Path, value: &Value) -> Result<(), DataError> {
    let bytes = canonical_json(value).map_err(|error| DataError::io(error.to_string()))?;
    fsutil::write_private(path, &bytes).map_err(|error| DataError::io(error.to_string()))
}

fn to_value<T: Serialize>(value: &T) -> Result<Value, DataError> {
    serde_json::to_value(value).map_err(|_| DataError::invalid("A record could not be serialized."))
}

fn cell<'a>(row: &'a BTreeMap<String, String>, name: &str) -> &'a str {
    row.get(name).map(String::as_str).unwrap_or("").trim()
}

fn nonempty(text: &str) -> Option<String> {
    if text.is_empty() {
        None
    } else {
        Some(text.to_owned())
    }
}

fn optional_text(row: &BTreeMap<String, String>, column: Option<&str>) -> Option<String> {
    column.and_then(|column| nonempty(cell(row, column)))
}

fn optional_decimal(
    row: &BTreeMap<String, String>,
    column: Option<&str>,
) -> Result<Option<String>, DataError> {
    match optional_text(row, column) {
        None => Ok(None),
        Some(text) => Ok(Some(canonical_decimal(&text)?)),
    }
}

fn optional_enum<T: DeserializeOwned>(
    row: &BTreeMap<String, String>,
    column: Option<&str>,
) -> Result<Option<T>, DataError> {
    match optional_text(row, column) {
        None => Ok(None),
        Some(text) => Ok(Some(parse_enum(&text)?)),
    }
}

fn parse_enum<T: DeserializeOwned>(text: &str) -> Result<T, DataError> {
    serde_json::from_value(Value::String(text.to_owned())).map_err(|_| {
        DataError::invalid(format!(
            "Value '{text}' is outside the controlled vocabulary and was not repaired."
        ))
    })
}

fn check_id(value: &str, label: &str) -> Result<(), DataError> {
    Identifier::try_from(value.to_owned())
        .map(|_| ())
        .map_err(|_| DataError::invalid(format!("{label} is not a valid identifier.")))
}

fn check_sha256(value: &str) -> Result<(), DataError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(DataError::invalid(
            "A declared checksum must be lowercase SHA-256 hexadecimal.",
        ))
    }
}

pub fn modality_names(records: &[MeasurementRecord]) -> Vec<String> {
    let mut names = Vec::new();
    for record in records {
        let name = record.modality.as_str().to_owned();
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

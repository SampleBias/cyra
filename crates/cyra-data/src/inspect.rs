//! Local provenance and measurement retrieval. No external service is contacted.
use crate::error::DataError;
use crate::fsutil;
use crate::ingest::{human_summary, summary_value};
use crate::model::{
    DatasetManifest, MeasurementRecord, OBSERVATION_SCHEMA, ObservationKind, ObservationRequest,
};
use crate::parquet_table;
use cyra_core::artifact::sha256;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LoadedDataset {
    pub root: PathBuf,
    pub manifest: DatasetManifest,
    pub manifest_sha256: String,
    pub records: Vec<MeasurementRecord>,
}

pub fn load_dataset(path: &Path) -> Result<LoadedDataset, DataError> {
    let (root, _matched) = resolve_root(path)?;
    if !fsutil::mode_is_private_dir(&root).map_err(|error| DataError::io(error.to_string()))? {
        return Err(DataError::io(
            "Dataset directory must be private (mode 0700) on the reference platform.",
        ));
    }
    let manifest_bytes = fsutil::read_relative(&root, "manifest.json")
        .map_err(|error| DataError::io(error.to_string()))?;
    let manifest: DatasetManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| DataError::invalid("Dataset manifest does not match cyra.dataset.v1."))?;
    if manifest.schema_version != crate::model::DATASET_SCHEMA {
        return Err(DataError::unsupported(
            "Unsupported dataset manifest schema. No migration was guessed.",
        ));
    }
    for file in &manifest.files {
        let bytes = fsutil::read_relative(&root, &file.path).map_err(|_| {
            DataError::io(format!(
                "Dataset file '{}' failed its integrity check. The dataset is not valid.",
                file.path
            ))
        })?;
        if bytes.len() as u64 != file.size_bytes || sha256(&bytes) != file.sha256 {
            return Err(DataError::io(format!(
                "Checksum mismatch for '{}'. The dataset is not valid and measurements were not returned as if it were.",
                file.path
            )));
        }
    }
    let records = parquet_table::read_table(&root.join("tables/measurements.parquet"))?;
    if records.len() != manifest.measurement_count {
        return Err(DataError::io(
            "Parquet row count does not match the dataset manifest.",
        ));
    }
    Ok(LoadedDataset {
        root,
        manifest_sha256: sha256(&manifest_bytes),
        manifest,
        records,
    })
}

pub fn observe_dataset(
    dataset: &Path,
    request_path: &Path,
) -> Result<(LoadedDataset, Vec<MeasurementRecord>), DataError> {
    let request_bytes = fs::read(request_path)
        .map_err(|_| DataError::io("Could not read the observation request."))?;
    let value = cyra_core::strict_json::parse(&request_bytes)
        .map_err(|_| DataError::invalid("Observation request is not strict JSON."))?;
    let request: ObservationRequest = serde_json::from_value(value)
        .map_err(|_| DataError::invalid("Observation request does not match its v1 schema."))?;
    if request.schema_version != OBSERVATION_SCHEMA {
        return Err(DataError::unsupported(
            "Unsupported observation request schema.",
        ));
    }
    if request.kind != ObservationKind::MeasurementRetrieval {
        return Err(DataError::unsupported(
            "Association estimands are not computed by measurement retrieval. No estimate was returned.",
        ));
    }
    let loaded = load_dataset(dataset)?;
    let records = loaded
        .records
        .iter()
        .filter(|record| allows(&request.sample_ids, &record.sample_id))
        .filter(|record| allows(&request.analyte_ids, &record.analyte_id))
        .filter(|record| {
            request
                .modalities
                .as_ref()
                .is_none_or(|modalities| modalities.contains(&record.modality))
        })
        .filter(|record| request.include_missing || record.missing_reason.is_none())
        .cloned()
        .collect();
    Ok((loaded, records))
}

pub fn provenance(path: &Path) -> Result<Value, DataError> {
    let loaded = load_dataset(path)?;
    let mut value = summary_value(&loaded.manifest, &loaded.manifest_sha256, true);
    value["transforms"] = json!(loaded.manifest.transforms);
    value["sources"] = json!(loaded.manifest.sources);
    value["accession"] = json!(loaded.manifest.accession);
    value["limitations"] = json!(loaded.manifest.limitations);
    value["human"] = json!(human_summary(
        &loaded.manifest,
        &loaded.manifest_sha256,
        Some(&loaded.root.join("manifest.json"))
    ));
    value["fields_not_stored"] = json!(loaded.manifest.unavailable_fields);
    Ok(value)
}

fn allows(filter: &Option<Vec<String>>, value: &str) -> bool {
    filter
        .as_ref()
        .is_none_or(|items| items.iter().any(|item| item == value))
}

fn resolve_root(path: &Path) -> Result<(PathBuf, Option<String>), DataError> {
    if path.is_dir() {
        return Ok((path.to_owned(), None));
    }
    if path.file_name().and_then(|name| name.to_str()) == Some("manifest.json") {
        let Some(parent) = path.parent() else {
            return Err(DataError::io("Manifest path has no parent directory."));
        };
        return Ok((parent.to_owned(), None));
    }
    let mut current = path.parent();
    let mut relative = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_owned();
    for _ in 0..4 {
        let Some(directory) = current else { break };
        if directory.join("manifest.json").is_file() {
            let bytes = fsutil::read_relative(directory, "manifest.json")
                .map_err(|error| DataError::io(error.to_string()))?;
            let manifest: DatasetManifest = serde_json::from_slice(&bytes)
                .map_err(|_| DataError::invalid("Dataset manifest is not readable."))?;
            if manifest.files.iter().any(|file| file.path == relative) {
                return Ok((directory.to_owned(), Some(relative)));
            }
        }
        let Some(name) = directory.file_name().and_then(|name| name.to_str()) else {
            break;
        };
        relative = format!("{name}/{relative}");
        current = directory.parent();
    }
    Err(DataError::io(
        "No dataset manifest claims this artifact. Provenance was not guessed.",
    ))
}

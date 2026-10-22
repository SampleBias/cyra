//! Donor-grouped splits. The assignment is a documented SHA-256 bucket, not a scientific RNG.
use crate::error::DataError;
use crate::model::{SampleFields, SplitRequest};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SPLIT_SCHEMA: &str = "cyra.donor-split.v1";
const SCALE: u32 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SplitName {
    Train,
    Validation,
    Holdout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DonorAssignment {
    pub donor_id: String,
    pub split: SplitName,
    pub bucket: u32,
    pub sample_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SplitFailure {
    pub reason: String,
    pub donor_id: Option<String>,
    pub sample_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SplitManifest {
    pub schema_version: String,
    pub seed: u64,
    pub algorithm: String,
    pub holdout_fraction_ppm: u32,
    pub validation_fraction_ppm: u32,
    pub usable_for_donor_holdout: bool,
    pub assignments: Vec<DonorAssignment>,
    pub failures: Vec<SplitFailure>,
    pub domains: SplitDomains,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SplitDomains {
    pub batches: Vec<String>,
    pub passages: Vec<String>,
    pub perturbations: Vec<String>,
    pub studies: Vec<String>,
}

pub fn assign_donor_splits(
    samples: &[SampleFields],
    request: &SplitRequest,
) -> Result<SplitManifest, DataError> {
    let holdout = fraction_ppm(&request.holdout_fraction)?;
    let validation = fraction_ppm(&request.validation_fraction)?;
    if holdout.saturating_add(validation) >= SCALE {
        return Err(DataError::invalid(
            "Holdout and validation fractions must leave a non-empty train assignment.",
        ));
    }
    let mut failures = Vec::new();
    let mut donors: Vec<String> = Vec::new();
    for sample in samples {
        let Some(donor) = sample.donor_id.as_deref() else {
            failures.push(SplitFailure {
                reason: "unknown_donor".to_owned(),
                donor_id: None,
                sample_id: Some(sample.sample_id.clone()),
            });
            continue;
        };
        if !donors.iter().any(|existing| existing == donor) {
            donors.push(donor.to_owned());
        }
    }
    let mut blocked = Vec::new();
    let identities: Vec<&str> = samples
        .iter()
        .filter_map(|sample| sample.source_identity.as_deref())
        .collect();
    for identity in identities {
        let mut owners = Vec::new();
        for sample in samples {
            if sample.source_identity.as_deref() == Some(identity)
                && let Some(donor) = sample.donor_id.as_deref()
                && !owners.iter().any(|existing: &String| existing == donor)
            {
                owners.push(donor.to_owned());
            }
        }
        if owners.len() > 1 {
            for donor in &owners {
                failures.push(SplitFailure {
                    reason: "overlapping_source".to_owned(),
                    donor_id: Some(donor.clone()),
                    sample_id: None,
                });
                blocked.push(donor.clone());
            }
        }
    }
    let mut assignments = Vec::new();
    for donor in donors {
        if blocked.iter().any(|blocked| blocked == &donor) {
            continue;
        }
        let bucket = donor_bucket(request.seed, &donor);
        let split = if bucket < holdout {
            SplitName::Holdout
        } else if bucket < holdout + validation {
            SplitName::Validation
        } else {
            SplitName::Train
        };
        let sample_ids = samples
            .iter()
            .filter(|sample| sample.donor_id.as_deref() == Some(donor.as_str()))
            .map(|sample| sample.sample_id.clone())
            .collect();
        assignments.push(DonorAssignment {
            donor_id: donor,
            split,
            bucket,
            sample_ids,
        });
    }
    assignments.sort_by(|left, right| left.donor_id.cmp(&right.donor_id));
    Ok(SplitManifest {
        schema_version: SPLIT_SCHEMA.to_owned(),
        seed: request.seed,
        algorithm: "sha256-mod-1000000 of cyra.donor-split.v1, seed and donor id".to_owned(),
        holdout_fraction_ppm: holdout,
        validation_fraction_ppm: validation,
        usable_for_donor_holdout: failures.is_empty(),
        assignments,
        failures,
        domains: domains(samples),
    })
}

pub fn donor_bucket(seed: u64, donor_id: &str) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(b"cyra.donor-split.v1\0");
    hasher.update(seed.to_le_bytes());
    hasher.update(donor_id.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    (u64::from_le_bytes(bytes) % u64::from(SCALE)) as u32
}

fn fraction_ppm(text: &str) -> Result<u32, DataError> {
    let canonical = crate::decimal::canonical_decimal(text)?;
    let (integer, fraction) = match canonical.split_once('.') {
        Some((integer, fraction)) => (integer, fraction),
        None => (canonical.as_str(), ""),
    };
    if integer != "0" && integer != "1" {
        return Err(DataError::invalid(
            "A split fraction must be between 0 and 1.",
        ));
    }
    if fraction.len() > 6 {
        return Err(DataError::invalid(
            "A split fraction supports at most 6 decimal places.",
        ));
    }
    if integer == "1" {
        if fraction.chars().any(|ch| ch != '0') && !fraction.is_empty() {
            return Err(DataError::invalid("A split fraction cannot exceed 1."));
        }
        return Ok(SCALE);
    }
    let mut digits = fraction.to_owned();
    while digits.len() < 6 {
        digits.push('0');
    }
    digits
        .parse()
        .map_err(|_| DataError::invalid("A split fraction is not a decimal."))
}

fn domains(samples: &[SampleFields]) -> SplitDomains {
    let mut batches = Vec::new();
    let mut passages = Vec::new();
    let mut perturbations = Vec::new();
    let mut studies = Vec::new();
    for sample in samples {
        push_unique(&mut batches, sample.experimental_batch.clone());
        push_unique(&mut batches, sample.culture_batch.clone());
        if let Some(passage) = sample.passage {
            push_unique(&mut passages, Some(passage.to_string()));
        }
        push_unique(&mut perturbations, sample.perturbation.clone());
        push_unique(&mut studies, sample.study_id.clone());
    }
    SplitDomains {
        batches,
        passages,
        perturbations,
        studies,
    }
}

fn push_unique(values: &mut Vec<String>, value: Option<String>) {
    if let Some(value) = value
        && !values.contains(&value)
    {
        values.push(value);
    }
}

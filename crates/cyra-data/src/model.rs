//! Versioned measurement, sample, source, license and manifest records.
use crate::units::UnitConversion;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const DATASET_SCHEMA: &str = "cyra.dataset.v1";
pub const MAPPING_SCHEMA: &str = "cyra.import-mapping.v1";
pub const METADATA_SCHEMA: &str = "cyra.dataset-metadata.v1";
pub const OBSERVATION_SCHEMA: &str = "cyra.observation-request.v1";
pub const MEASUREMENT_SCHEMA_V0: &str = "cyra.measurement.v0";
pub const MEASUREMENT_SCHEMA_V1: &str = "cyra.measurement.v1";
pub const MEASUREMENT_SCHEMA_V2: &str = "cyra.measurement.v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Transcript,
    IntracellularProtein,
    SecretedProtein,
    Metabolite,
    FunctionalAssay,
}
impl Modality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transcript => "transcript",
            Self::IntracellularProtein => "intracellular_protein",
            Self::SecretedProtein => "secreted_protein",
            Self::Metabolite => "metabolite",
            Self::FunctionalAssay => "functional_assay",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuantityKind {
    Count,
    RelativeExpression,
    Abundance,
    Activity,
    Concentration,
    Amount,
    SpecificAmount,
    Flux,
    FunctionalReadout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MissingReason {
    NotMeasured,
    BelowDetection,
    FailedQc,
    NotApplicable,
    Unknown,
}
impl MissingReason {
    pub fn code(self) -> u8 {
        match self {
            Self::NotMeasured => 1,
            Self::BelowDetection => 2,
            Self::FailedQc => 3,
            Self::NotApplicable => 4,
            Self::Unknown => 5,
        }
    }
    pub fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            0 => return None,
            1 => Self::NotMeasured,
            2 => Self::BelowDetection,
            3 => Self::FailedQc,
            4 => Self::NotApplicable,
            5 => Self::Unknown,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
    Second,
    Minute,
    Hour,
    Day,
}
impl TimeUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Second => "second",
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Compartment {
    Intracellular,
    Extracellular,
    Membrane,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReplicateRole {
    Primary,
    Technical,
    RepeatedSample,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Experimental,
    SyntheticFixture,
    Simulated,
    LiteraturePointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Eligibility {
    Eligible,
    Quarantined,
    Ineligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LicenseDomain {
    Dataset,
    Code,
    ModelWeight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RightsBasis {
    SyntheticFixtureAuthorization,
    PublicDomain,
    PermissiveReuse,
    Restricted,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntendedUse {
    PipelineValidation,
    IndependentBenchmark,
    MethodDevelopment,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LicenseRecord {
    pub domain: LicenseDomain,
    pub rights_basis: RightsBasis,
    pub intended_use: IntendedUse,
    pub spdx: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Accession {
    pub required: bool,
    pub value: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRecord {
    pub source_id: String,
    pub kind: String,
    pub uri: Option<String>,
    pub version: String,
    pub content_role: String,
    pub checksum_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SampleFields {
    pub sample_id: String,
    pub donor_id: Option<String>,
    pub cell_id: Option<String>,
    pub population_id: Option<String>,
    pub replicate_id: Option<String>,
    pub replicate_role: ReplicateRole,
    pub parent_sample_id: Option<String>,
    pub passage: Option<u32>,
    pub tissue: Option<String>,
    pub culture_batch: Option<String>,
    pub experimental_batch: Option<String>,
    pub perturbation: Option<String>,
    pub study_id: Option<String>,
    pub source_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SampleRecord {
    #[serde(flatten)]
    pub fields: SampleFields,
    pub included_in_measurements: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LineageRef {
    pub sample_id: String,
    pub source_id: String,
    pub input_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MeasurementRecord {
    pub record_id: String,
    pub sample_id: String,
    pub analyte_id: String,
    pub modality: Modality,
    pub unit: String,
    pub quantity_kind: QuantityKind,
    pub value: Option<String>,
    pub missing_reason: Option<MissingReason>,
    pub censor_limit: Option<String>,
    pub qc_failure: Option<String>,
    pub time_amount: Option<String>,
    pub time_unit: Option<TimeUnit>,
    pub time_origin: Option<String>,
    pub assay_id: Option<String>,
    pub protocol_version: Option<String>,
    pub compartment: Option<Compartment>,
    pub lineage: LineageRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransformRecord {
    pub transform_id: String,
    pub name: String,
    pub software: String,
    pub input_sha256: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuarantineRecord {
    pub row_number: usize,
    pub reason: String,
    pub sample_id: Option<String>,
    pub analyte_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileDigest {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPlan {
    pub included: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Representation {
    pub numeric: String,
    pub trailing_fractional_zeros: String,
    pub binary_float: String,
    pub tolerance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StorageProfiles {
    pub parquet: String,
    pub zarr: String,
    pub parquet_compression: String,
    pub zarr_codecs: Vec<String>,
    pub parquet_row_group_rows: usize,
    pub zarr_chunk_observations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DatasetManifest {
    pub schema_version: String,
    pub dataset_id: String,
    pub origin: Origin,
    pub eligibility: Eligibility,
    pub benchmark_plan: BenchmarkPlan,
    pub importer_revision: String,
    pub input_manifest_sha256: String,
    pub representation: Representation,
    pub storage_profiles: StorageProfiles,
    pub licenses: Vec<LicenseRecord>,
    pub accession: Accession,
    pub sources: Vec<SourceRecord>,
    pub samples: Vec<SampleRecord>,
    pub transforms: Vec<TransformRecord>,
    pub measurement_count: usize,
    pub quarantined_count: usize,
    pub deduplicated_count: usize,
    pub unavailable_fields: Vec<String>,
    pub files: Vec<FileDigest>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColumnMapping {
    pub sample_id: String,
    pub analyte_id: String,
    pub value: String,
    pub unit: Option<String>,
    pub modality: Option<String>,
    pub missing_reason: Option<String>,
    pub censor_limit: Option<String>,
    pub qc_failure: Option<String>,
    pub time_amount: Option<String>,
    pub time_unit: Option<String>,
    pub time_origin: Option<String>,
    pub assay_id: Option<String>,
    pub protocol_version: Option<String>,
    pub compartment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalyteMapping {
    pub modality: Modality,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportMapping {
    pub schema_version: String,
    pub format: TableFormat,
    pub columns: ColumnMapping,
    pub analytes: std::collections::BTreeMap<String, AnalyteMapping>,
    pub conversions: Vec<UnitConversion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TableFormat {
    Csv,
    Tsv,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SplitRequest {
    pub seed: u64,
    pub holdout_fraction: String,
    pub validation_fraction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DatasetMetadata {
    pub schema_version: String,
    pub dataset_id: String,
    pub origin: Origin,
    pub tissue_context: String,
    pub accession: Accession,
    pub licenses: Vec<LicenseRecord>,
    pub sources: Vec<SourceRecord>,
    pub samples: Vec<SampleFields>,
    pub split: Option<SplitRequest>,
    pub unavailable_fields: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObservationKind {
    MeasurementRetrieval,
    AssociationEstimand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservationRequest {
    pub schema_version: String,
    pub kind: ObservationKind,
    pub sample_ids: Option<Vec<String>>,
    pub analyte_ids: Option<Vec<String>>,
    pub modalities: Option<Vec<Modality>>,
    pub include_missing: bool,
}

impl MeasurementRecord {
    pub fn biological_key(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}",
            self.sample_id,
            self.analyte_id,
            self.modality.as_str(),
            self.time_amount.as_deref().unwrap_or(""),
            self.time_unit.map(TimeUnit::as_str).unwrap_or(""),
            self.time_origin.as_deref().unwrap_or("")
        )
    }
}

//! Measurement ingestion and provenance. No physiological model is evaluated here.
//! Storage profiles are uncompressed Parquet and a Zarr v2 subset; see ADR 0003.

mod decimal;
mod error;
mod fsutil;
mod ingest;
mod inspect;
mod migrate;
mod model;
mod parquet_table;
mod split;
mod tabular;
mod units;
mod zarr_array;

pub use error::DataError;
pub use ingest::{ImportOutcome, ImportRequest, import_dataset};
pub use inspect::{observe_dataset, provenance};
pub use migrate::read_versioned_measurements;
pub use model::{
    DatasetManifest, Eligibility, ImportMapping, MeasurementRecord, MissingReason, Modality,
    ObservationRequest, Origin, SampleRecord,
};
pub use parquet_table::{
    PARQUET_PROFILE, PARQUET_ROW_GROUP_ROWS, read_row_groups, row_group_count,
};
pub use split::assign_donor_splits;
pub use zarr_array::{
    ZARR_CHUNK_OBSERVATIONS, ZARR_PROFILE, max_one_chunk_working_set, read_slice,
};

pub const IMPORTER_REVISION: &str =
    concat!("cyra-data/", env!("CARGO_PKG_VERSION"), "+cyra.dataset.v1");
pub const MAX_IMPORT_ROWS: usize = 8192;

pub fn schema(name: &str) -> Option<serde_json::Value> {
    let schema = match name {
        "import-mapping" => schemars::schema_for!(ImportMapping),
        "dataset-manifest" => schemars::schema_for!(DatasetManifest),
        "observation-request" => schemars::schema_for!(ObservationRequest),
        "measurement-record" => schemars::schema_for!(MeasurementRecord),
        _ => return None,
    };
    serde_json::to_value(schema).ok()
}

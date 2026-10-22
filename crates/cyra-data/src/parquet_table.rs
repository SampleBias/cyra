//! Uncompressed Parquet profile `cyra.parquet.v1`.
//! Plain encoding, no dictionary, row groups of at most [`PARQUET_ROW_GROUP_ROWS`] rows.
//! Decimals are UTF-8 canonical strings. Null is a typed absence, not zero.
use crate::error::DataError;
use crate::model::{
    Compartment, LineageRef, MEASUREMENT_SCHEMA_V1, MeasurementRecord, MissingReason, QuantityKind,
    TimeUnit,
};
use arrow_array::{Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::basic::{Compression, Encoding};
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

pub const PARQUET_PROFILE: &str = "cyra.parquet.v1";
pub const PARQUET_ROW_GROUP_ROWS: usize = 256;
const PROFILE_KEY: &str = "cyra.measurement-table";
const CODEC_KEY: &str = "cyra.codec";

fn arrow_schema() -> Arc<Schema> {
    let utf8 = |name: &str, nullable: bool| Field::new(name, DataType::Utf8, nullable);
    Arc::new(Schema::new(vec![
        utf8("record_id", false),
        utf8("sample_id", false),
        utf8("analyte_id", false),
        utf8("modality", false),
        utf8("unit", false),
        utf8("quantity_kind", false),
        utf8("value", true),
        utf8("missing_reason", true),
        utf8("censor_limit", true),
        utf8("qc_failure", true),
        utf8("time_amount", true),
        utf8("time_unit", true),
        utf8("time_origin", true),
        utf8("assay_id", true),
        utf8("protocol_version", true),
        utf8("compartment", true),
        utf8("lineage_json", false),
    ]))
}

pub fn write_table(path: &Path, records: &[MeasurementRecord]) -> Result<usize, DataError> {
    let schema = arrow_schema();
    let properties = WriterProperties::builder()
        .set_compression(Compression::UNCOMPRESSED)
        .set_dictionary_enabled(false)
        .set_encoding(Encoding::PLAIN)
        .set_max_row_group_row_count(Some(PARQUET_ROW_GROUP_ROWS))
        .set_key_value_metadata(Some(vec![
            KeyValue {
                key: PROFILE_KEY.to_owned(),
                value: Some(PARQUET_PROFILE.to_owned()),
            },
            KeyValue {
                key: CODEC_KEY.to_owned(),
                value: Some("uncompressed".to_owned()),
            },
        ]))
        .build();
    let file = File::create(path).map_err(|error| DataError::io(error.to_string()))?;
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(properties))
        .map_err(|error| DataError::io(format!("Parquet writer rejected the profile: {error}")))?;
    for batch in records.chunks(PARQUET_ROW_GROUP_ROWS) {
        writer
            .write(&batch_from(schema.clone(), batch)?)
            .map_err(|error| DataError::io(format!("Parquet write failed: {error}")))?;
        writer
            .flush()
            .map_err(|error| DataError::io(format!("Parquet row group flush failed: {error}")))?;
    }
    let metadata = writer
        .close()
        .map_err(|error| DataError::io(format!("Parquet close failed: {error}")))?;
    let groups = metadata.row_groups().len();
    if records.len() > PARQUET_ROW_GROUP_ROWS && groups < 2 {
        return Err(DataError::io(
            "Parquet writer did not honor the bounded row-group profile.",
        ));
    }
    Ok(groups)
}

pub fn read_table(path: &Path) -> Result<Vec<MeasurementRecord>, DataError> {
    read_row_groups(path, None)
}

pub fn read_row_groups(
    path: &Path,
    only: Option<Vec<usize>>,
) -> Result<Vec<MeasurementRecord>, DataError> {
    let file = File::open(path).map_err(|error| DataError::io(error.to_string()))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|error| DataError::invalid(format!("Parquet file is not readable: {error}")))?;
    let metadata = builder.metadata().clone();
    check_profile(metadata.file_metadata().key_value_metadata())?;
    for group in metadata.row_groups() {
        if group.num_rows() > PARQUET_ROW_GROUP_ROWS as i64 {
            return Err(DataError::invalid(
                "Parquet row group exceeds the cyra.parquet.v1 bound.",
            ));
        }
        if group
            .columns()
            .iter()
            .any(|column| column.compression() != Compression::UNCOMPRESSED)
        {
            return Err(DataError::unsupported(
                "Only uncompressed Parquet is in the cyra.parquet.v1 profile.",
            ));
        }
    }
    let builder = if let Some(groups) = only {
        builder.with_row_groups(groups)
    } else {
        builder
    };
    let reader = builder
        .with_batch_size(PARQUET_ROW_GROUP_ROWS)
        .build()
        .map_err(|error| {
            DataError::invalid(format!("Parquet reader rejected the file: {error}"))
        })?;
    let mut records = Vec::new();
    for batch in reader {
        let batch =
            batch.map_err(|error| DataError::invalid(format!("Parquet batch failed: {error}")))?;
        records.extend(records_from(&batch)?);
    }
    Ok(records)
}

pub fn row_group_count(path: &Path) -> Result<usize, DataError> {
    let file = File::open(path).map_err(|error| DataError::io(error.to_string()))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|error| DataError::invalid(format!("Parquet file is not readable: {error}")))?;
    Ok(builder.metadata().num_row_groups())
}

fn check_profile(metadata: Option<&Vec<KeyValue>>) -> Result<(), DataError> {
    let Some(entries) = metadata else {
        return Err(DataError::invalid(
            "Parquet file is missing the Cyra schema profile.",
        ));
    };
    let profile = entries.iter().find(|entry| entry.key == PROFILE_KEY);
    let codec = entries.iter().find(|entry| entry.key == CODEC_KEY);
    match profile.and_then(|entry| entry.value.as_deref()) {
        Some(PARQUET_PROFILE) | Some(MEASUREMENT_SCHEMA_V1) => {}
        Some(other) if other.contains(".v0") => {
            return Err(DataError::unsupported(
                "Parquet profile v0 is not stored. Migrate cyra.measurement.v0 JSON explicitly; no silent upgrade was applied.",
            ));
        }
        Some(other) => {
            return Err(DataError::unsupported(format!(
                "Unsupported measurement table schema '{other}'. Supported native profile: {PARQUET_PROFILE}. Future schemas are rejected rather than guessed."
            )));
        }
        None => {
            return Err(DataError::invalid(
                "Parquet file does not declare cyra.parquet.v1.",
            ));
        }
    }
    if codec.and_then(|entry| entry.value.as_deref()) != Some("uncompressed") {
        return Err(DataError::unsupported(
            "Parquet codec is not the uncompressed cyra.parquet.v1 profile.",
        ));
    }
    Ok(())
}

fn batch_from(
    schema: Arc<Schema>,
    records: &[MeasurementRecord],
) -> Result<RecordBatch, DataError> {
    let string = |values: Vec<Option<String>>| Arc::new(StringArray::from(values));
    let required = |values: Vec<String>| string(values.into_iter().map(Some).collect());
    let lineage = records
        .iter()
        .map(|record| serde_json::to_string(&record.lineage))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| DataError::invalid("Lineage reference could not be serialized."))?;
    let batch = RecordBatch::try_new(
        schema,
        vec![
            required(
                records
                    .iter()
                    .map(|record| record.record_id.clone())
                    .collect(),
            ),
            required(
                records
                    .iter()
                    .map(|record| record.sample_id.clone())
                    .collect(),
            ),
            required(
                records
                    .iter()
                    .map(|record| record.analyte_id.clone())
                    .collect(),
            ),
            required(
                records
                    .iter()
                    .map(|record| record.modality.as_str().to_owned())
                    .collect(),
            ),
            required(records.iter().map(|record| record.unit.clone()).collect()),
            required(
                records
                    .iter()
                    .map(|record| quantity_name(record.quantity_kind))
                    .collect(),
            ),
            string(records.iter().map(|record| record.value.clone()).collect()),
            string(
                records
                    .iter()
                    .map(|record| record.missing_reason.map(serde_plain))
                    .collect(),
            ),
            string(
                records
                    .iter()
                    .map(|record| record.censor_limit.clone())
                    .collect(),
            ),
            string(
                records
                    .iter()
                    .map(|record| record.qc_failure.clone())
                    .collect(),
            ),
            string(
                records
                    .iter()
                    .map(|record| record.time_amount.clone())
                    .collect(),
            ),
            string(
                records
                    .iter()
                    .map(|record| record.time_unit.map(|unit| unit.as_str().to_owned()))
                    .collect(),
            ),
            string(
                records
                    .iter()
                    .map(|record| record.time_origin.clone())
                    .collect(),
            ),
            string(
                records
                    .iter()
                    .map(|record| record.assay_id.clone())
                    .collect(),
            ),
            string(
                records
                    .iter()
                    .map(|record| record.protocol_version.clone())
                    .collect(),
            ),
            string(
                records
                    .iter()
                    .map(|record| record.compartment.map(serde_plain_compartment))
                    .collect(),
            ),
            required(lineage),
        ],
    )
    .map_err(|error| DataError::io(format!("Arrow batch rejected measurements: {error}")))?;
    Ok(batch)
}

fn serde_plain(reason: MissingReason) -> String {
    serde_json::to_value(reason)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_owned())
}

fn serde_plain_compartment(compartment: Compartment) -> String {
    serde_json::to_value(compartment)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "not_applicable".to_owned())
}

fn records_from(batch: &RecordBatch) -> Result<Vec<MeasurementRecord>, DataError> {
    let column = |name: &str| {
        batch
            .column_by_name(name)
            .and_then(|array| array.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| {
                DataError::invalid(format!("Parquet column '{name}' is missing or not UTF-8."))
            })
    };
    let record_id = column("record_id")?;
    let sample_id = column("sample_id")?;
    let analyte_id = column("analyte_id")?;
    let modality = column("modality")?;
    let unit = column("unit")?;
    let quantity_kind = column("quantity_kind")?;
    let value = column("value")?;
    let missing_reason = column("missing_reason")?;
    let censor_limit = column("censor_limit")?;
    let qc_failure = column("qc_failure")?;
    let time_amount = column("time_amount")?;
    let time_unit = column("time_unit")?;
    let time_origin = column("time_origin")?;
    let assay_id = column("assay_id")?;
    let protocol_version = column("protocol_version")?;
    let compartment = column("compartment")?;
    let lineage_json = column("lineage_json")?;
    let mut records = Vec::with_capacity(batch.num_rows());
    for index in 0..batch.num_rows() {
        let text = |array: &StringArray| {
            if array.is_null(index) {
                None
            } else {
                Some(array.value(index).to_owned())
            }
        };
        let required = |array: &StringArray, name: &str| {
            text(array).ok_or_else(|| DataError::invalid(format!("Parquet row is missing {name}.")))
        };
        let lineage: LineageRef = serde_json::from_str(&required(lineage_json, "lineage")?)
            .map_err(|_| DataError::invalid("Parquet lineage JSON is not a lineage reference."))?;
        records.push(MeasurementRecord {
            record_id: required(record_id, "record_id")?,
            sample_id: required(sample_id, "sample_id")?,
            analyte_id: required(analyte_id, "analyte_id")?,
            modality: parse_enum(&required(modality, "modality")?)?,
            unit: required(unit, "unit")?,
            quantity_kind: parse_enum(&required(quantity_kind, "quantity_kind")?)?,
            value: text(value),
            missing_reason: text(missing_reason)
                .map(|reason| parse_enum::<MissingReason>(&reason))
                .transpose()?,
            censor_limit: text(censor_limit),
            qc_failure: text(qc_failure),
            time_amount: text(time_amount),
            time_unit: text(time_unit)
                .map(|unit| parse_enum::<TimeUnit>(&unit))
                .transpose()?,
            time_origin: text(time_origin),
            assay_id: text(assay_id),
            protocol_version: text(protocol_version),
            compartment: text(compartment)
                .map(|compartment| parse_enum::<Compartment>(&compartment))
                .transpose()?,
            lineage,
        });
    }
    Ok(records)
}

fn parse_enum<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, DataError> {
    serde_json::from_value(serde_json::Value::String(text.to_owned()))
        .map_err(|_| DataError::invalid(format!("Unrecognized stored enumeration '{text}'.")))
}

/// Quantity kind is stored with serde's snake_case, not Debug.
pub fn quantity_name(kind: QuantityKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_owned())
}

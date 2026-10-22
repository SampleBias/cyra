//! Explicit schema migration. Future versions are rejected and are not partially read.
use crate::error::DataError;
use crate::model::{MEASUREMENT_SCHEMA_V0, MEASUREMENT_SCHEMA_V1, MEASUREMENT_SCHEMA_V2, Modality};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigratedMeasurements {
    pub schema_version: String,
    pub records: Vec<Value>,
    pub transform_name: Option<String>,
    pub note: String,
}

pub fn read_versioned_measurements(value: &Value) -> Result<MigratedMeasurements, DataError> {
    let version = value
        .get("schema_version")
        .and_then(Value::as_str)
        .ok_or_else(|| DataError::invalid("Measurement document has no schema_version."))?;
    match version {
        MEASUREMENT_SCHEMA_V1 => Ok(MigratedMeasurements {
            schema_version: version.to_owned(),
            records: array(value)?,
            transform_name: None,
            note: "Native v1 document; no migration was required.".to_owned(),
        }),
        MEASUREMENT_SCHEMA_V0 => {
            let mut records = Vec::new();
            for (index, row) in array(value)?.into_iter().enumerate() {
                records.push(migrate_v0(index, &row)?);
            }
            Ok(MigratedMeasurements {
                schema_version: MEASUREMENT_SCHEMA_V1.to_owned(),
                records,
                transform_name: Some("migrate_measurement_v0_to_v1".to_owned()),
                note: "Closed v0 vocabulary was mapped to v1. Unknown v0 units are not repaired."
                    .to_owned(),
            })
        }
        MEASUREMENT_SCHEMA_V2 => Err(DataError::unsupported(
            "Schema cyra.measurement.v2 is a future version and was rejected. Supported inputs: cyra.measurement.v0 (explicit migration) and cyra.measurement.v1. No rows were written.",
        )),
        other => Err(DataError::unsupported(format!(
            "Unsupported measurement schema '{other}'. Supported inputs: {MEASUREMENT_SCHEMA_V0} and {MEASUREMENT_SCHEMA_V1}."
        ))),
    }
}

fn array(value: &Value) -> Result<Vec<Value>, DataError> {
    value
        .get("records")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| DataError::invalid("Measurement document requires a records array."))
}

fn migrate_v0(index: usize, row: &Value) -> Result<Value, DataError> {
    let text = |name: &str| {
        row.get(name)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| {
                DataError::invalid(format!(
                    "v0 row {} is missing string field {name}. It was not inferred.",
                    index + 1
                ))
            })
    };
    let modality = match text("kind")?.as_str() {
        "transcript" => Modality::Transcript,
        "intracellular_protein" => Modality::IntracellularProtein,
        "secreted" => Modality::SecretedProtein,
        "metabolite" => Modality::Metabolite,
        "functional_assay" => Modality::FunctionalAssay,
        other => {
            return Err(DataError::invalid(format!(
                "v0 modality '{other}' is outside the migration map."
            )));
        }
    };
    let unit = match text("unit")?.as_str() {
        "count" => "count",
        "tpm" => "tpm",
        "pg/ml" => "pg_per_ml",
        "ng/ml" => "ng_per_ml",
        "abundance_au" => "abundance_au",
        "uM" => "micromol_per_l",
        "percent" => "percent",
        other => {
            return Err(DataError::invalid(format!(
                "v0 unit '{other}' has no explicit migration. It was not guessed."
            )));
        }
    };
    let amount = row.get("amount");
    let value = match amount {
        None | Some(Value::Null) => Value::Null,
        Some(Value::String(text)) => Value::String(crate::decimal::canonical_decimal(text)?),
        _ => {
            return Err(DataError::invalid(
                "v0 amount must be a string or null so the decimal spelling is explicit.",
            ));
        }
    };
    Ok(serde_json::json!({
        "schema_version": MEASUREMENT_SCHEMA_V1,
        "donor_id": text("donor")?,
        "sample_id": text("sample")?,
        "analyte_id": text("analyte")?,
        "modality": modality.as_str(),
        "unit": unit,
        "value": value,
        "migrated_from": MEASUREMENT_SCHEMA_V0
    }))
}

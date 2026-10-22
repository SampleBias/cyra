//! Zarr v2 directory store, profile `cyra.zarr.v1`.
//! Codecs other than uncompressed `null`, and Zarr versions other than 2, are rejected.
//! A missing chunk is an error. It is never read back as a measured zero.
use crate::error::DataError;
use crate::fsutil;
use crate::model::{MeasurementRecord, MissingReason};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

pub const ZARR_PROFILE: &str = "cyra.zarr.v1";
pub const ZARR_CHUNK_OBSERVATIONS: usize = 32;
pub const ZARR_VALUE_BYTES: usize = 48;

#[derive(Debug)]
pub struct SliceRead {
    pub records: Vec<MeasurementRecord>,
    pub chunks_opened: Vec<String>,
    pub bytes_read: u64,
}

pub fn write_array(dir: &Path, records: &[MeasurementRecord]) -> Result<(), DataError> {
    fsutil::create_private_dir(dir).map_err(io_error)?;
    write_json(dir.join(".zgroup"), &json!({"zarr_format": 2}))?;
    write_json(dir.join(".zattrs"), &group_attrs(records))?;
    let n = records.len();
    write_bytes_array(dir, "value", n, &value_planes(records, false))?;
    write_bytes_array(dir, "censor_limit", n, &value_planes(records, true))?;
    write_u16_array(dir, "value_len", &lengths(records, false))?;
    write_u16_array(dir, "censor_limit_len", &lengths(records, true))?;
    write_u8_array(
        dir,
        "presence",
        &records.iter().map(presence).collect::<Vec<_>>(),
    )?;
    write_u8_array(
        dir,
        "missing_reason",
        &records
            .iter()
            .map(|record| record.missing_reason.map(MissingReason::code).unwrap_or(0))
            .collect::<Vec<_>>(),
    )?;
    write_u8_array(
        dir,
        "censor_flag",
        &records
            .iter()
            .map(|record| u8::from(record.missing_reason == Some(MissingReason::BelowDetection)))
            .collect::<Vec<_>>(),
    )?;
    Ok(())
}

pub fn read_slice(dir: &Path, start: usize, end: usize) -> Result<SliceRead, DataError> {
    let attrs = read_json(&dir.join(".zattrs"))?;
    if attrs.get("cyra_zarr_profile").and_then(Value::as_str) != Some(ZARR_PROFILE) {
        return Err(DataError::unsupported(
            "Array attributes are not cyra.zarr.v1. Unsupported Zarr variants are not interpreted.",
        ));
    }
    if attrs.get("missing_chunk_policy").and_then(Value::as_str) != Some("error_not_zero") {
        return Err(DataError::unsupported(
            "This Zarr array does not declare the missing-chunk error policy.",
        ));
    }
    let n = attrs
        .get("observation_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| DataError::invalid("Zarr attributes omit observation_count."))?
        as usize;
    if start > end || end > n {
        return Err(DataError::invalid(
            "Observation slice is outside the array.",
        ));
    }
    let mut cache = ChunkCache::default();
    let mut records = Vec::new();
    for index in start..end {
        let chunk = index / ZARR_CHUNK_OBSERVATIONS;
        let offset = index % ZARR_CHUNK_OBSERVATIONS;
        let value = read_padded(dir, "value", chunk, offset, &mut cache)?;
        let censor = read_padded(dir, "censor_limit", chunk, offset, &mut cache)?;
        let value_len = read_u16_at(dir, "value_len", chunk, offset, &mut cache)?;
        let censor_len = read_u16_at(dir, "censor_limit_len", chunk, offset, &mut cache)?;
        let presence = read_u8_at(dir, "presence", chunk, offset, &mut cache)?;
        let reason = read_u8_at(dir, "missing_reason", chunk, offset, &mut cache)?;
        let _censor_flag = read_u8_at(dir, "censor_flag", chunk, offset, &mut cache)?;
        if presence == 0 {
            return Err(DataError::invalid(
                "Stored presence is absent. This is not reported as a measured zero.",
            ));
        }
        let value = decode_text(&value, value_len)?;
        let censor_limit = decode_text(&censor, censor_len)?;
        records.push(record_from_attrs(
            &attrs,
            index,
            presence,
            reason,
            value,
            censor_limit,
        )?);
    }
    cache.opened.sort();
    cache.opened.dedup();
    Ok(SliceRead {
        records,
        chunks_opened: cache.opened,
        bytes_read: cache.bytes_read,
    })
}

#[derive(Default)]
struct ChunkCache {
    files: std::collections::HashMap<String, Vec<u8>>,
    opened: Vec<String>,
    bytes_read: u64,
}

pub fn max_one_chunk_working_set() -> u64 {
    let rows = ZARR_CHUNK_OBSERVATIONS as u64;
    rows * ZARR_VALUE_BYTES as u64 * 2 + rows * 2 * 2 + rows * 3
}

fn group_attrs(records: &[MeasurementRecord]) -> Value {
    let string_list = |values: Vec<Option<String>>| {
        values
            .into_iter()
            .map(|value| value.map(Value::String).unwrap_or(Value::Null))
            .collect::<Vec<_>>()
    };
    json!({
        "cyra_zarr_profile": ZARR_PROFILE,
        "missing_chunk_policy": "error_not_zero",
        "representation": "canonical decimal; trailing fractional zeros insignificant; binary float not used",
        "axes": [
            {"name": "observation", "type": "measurement"},
            {"name": "byte", "type": "utf8_padding"}
        ],
        "observation_count": records.len(),
        "record_ids": records.iter().map(|record| record.record_id.clone()).collect::<Vec<_>>(),
        "sample_ids": records.iter().map(|record| record.sample_id.clone()).collect::<Vec<_>>(),
        "analyte_ids": records.iter().map(|record| record.analyte_id.clone()).collect::<Vec<_>>(),
        "modalities": records.iter().map(|record| record.modality.as_str()).collect::<Vec<_>>(),
        "units": records.iter().map(|record| record.unit.clone()).collect::<Vec<_>>(),
        "quantity_kinds": records.iter().map(|record| crate::parquet_table::quantity_name(record.quantity_kind)).collect::<Vec<_>>(),
        "time_amounts": string_list(records.iter().map(|record| record.time_amount.clone()).collect()),
        "time_units": string_list(records.iter().map(|record| record.time_unit.map(|unit| unit.as_str().to_owned())).collect()),
        "time_origins": string_list(records.iter().map(|record| record.time_origin.clone()).collect()),
        "assay_ids": string_list(records.iter().map(|record| record.assay_id.clone()).collect()),
        "protocol_versions": string_list(records.iter().map(|record| record.protocol_version.clone()).collect()),
        "compartments": string_list(records.iter().map(|record| record.compartment.map(|compartment| {
            serde_json::to_value(compartment).ok().and_then(|value| value.as_str().map(str::to_owned)).unwrap_or_else(|| "not_applicable".to_owned())
        })).collect()),
        "qc_failures": string_list(records.iter().map(|record| record.qc_failure.clone()).collect()),
        "lineage": records.iter().map(|record| serde_json::to_value(&record.lineage).unwrap_or(Value::Null)).collect::<Vec<_>>(),
        "codecs": ["uncompressed"],
        "chunk_observations": ZARR_CHUNK_OBSERVATIONS
    })
}

fn value_planes(records: &[MeasurementRecord], censor: bool) -> Vec<[u8; ZARR_VALUE_BYTES]> {
    records
        .iter()
        .map(|record| {
            let text = if censor {
                record.censor_limit.as_deref().unwrap_or("")
            } else if record.value.is_some() {
                record.value.as_deref().unwrap_or("")
            } else {
                ""
            };
            let mut bytes = [0u8; ZARR_VALUE_BYTES];
            let encoded = text.as_bytes();
            bytes[..encoded.len()].copy_from_slice(encoded);
            bytes
        })
        .collect()
}

fn lengths(records: &[MeasurementRecord], censor: bool) -> Vec<u16> {
    records
        .iter()
        .map(|record| {
            let text = if censor {
                record.censor_limit.as_deref().unwrap_or("")
            } else {
                record.value.as_deref().unwrap_or("")
            };
            text.len() as u16
        })
        .collect()
}

fn presence(record: &MeasurementRecord) -> u8 {
    if record.value.is_some() { 1 } else { 2 }
}

fn write_bytes_array(
    dir: &Path,
    name: &str,
    n: usize,
    planes: &[[u8; ZARR_VALUE_BYTES]],
) -> Result<(), DataError> {
    let array = dir.join(name);
    fsutil::create_private_dir(&array).map_err(io_error)?;
    write_json(
        array.join(".zarray"),
        &zarray(
            vec![n, ZARR_VALUE_BYTES],
            vec![ZARR_CHUNK_OBSERVATIONS, ZARR_VALUE_BYTES],
            "|u1",
            0,
        ),
    )?;
    write_json(array.join(".zattrs"), &json!({"role": name}))?;
    for (chunk, plane) in planes.chunks(ZARR_CHUNK_OBSERVATIONS).enumerate() {
        let mut bytes = Vec::with_capacity(plane.len() * ZARR_VALUE_BYTES);
        for row in plane {
            bytes.extend_from_slice(row);
        }
        fsutil::write_private(&array.join(format!("{chunk}.0")), &bytes).map_err(io_error)?;
    }
    Ok(())
}

fn write_u8_array(dir: &Path, name: &str, values: &[u8]) -> Result<(), DataError> {
    let array = dir.join(name);
    fsutil::create_private_dir(&array).map_err(io_error)?;
    write_json(
        array.join(".zarray"),
        &zarray(
            vec![values.len()],
            vec![ZARR_CHUNK_OBSERVATIONS],
            "|u1",
            255,
        ),
    )?;
    write_json(array.join(".zattrs"), &json!({"role": name}))?;
    for (chunk, rows) in values.chunks(ZARR_CHUNK_OBSERVATIONS).enumerate() {
        fsutil::write_private(&array.join(chunk.to_string()), rows).map_err(io_error)?;
    }
    Ok(())
}

fn write_u16_array(dir: &Path, name: &str, values: &[u16]) -> Result<(), DataError> {
    let array = dir.join(name);
    fsutil::create_private_dir(&array).map_err(io_error)?;
    write_json(
        array.join(".zarray"),
        &zarray(
            vec![values.len()],
            vec![ZARR_CHUNK_OBSERVATIONS],
            "<u2",
            65535,
        ),
    )?;
    write_json(array.join(".zattrs"), &json!({"role": name}))?;
    for (chunk, rows) in values.chunks(ZARR_CHUNK_OBSERVATIONS).enumerate() {
        let mut bytes = Vec::with_capacity(rows.len() * 2);
        for value in rows {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        fsutil::write_private(&array.join(chunk.to_string()), &bytes).map_err(io_error)?;
    }
    Ok(())
}

fn zarray(shape: Vec<usize>, chunks: Vec<usize>, dtype: &str, fill: u64) -> Value {
    json!({
        "zarr_format": 2,
        "shape": shape,
        "chunks": chunks,
        "dtype": dtype,
        "compressor": null,
        "fill_value": fill,
        "order": "C",
        "filters": null
    })
}

fn write_json(path: PathBuf, value: &Value) -> Result<(), DataError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|_| DataError::io("JSON metadata could not be encoded."))?;
    fsutil::write_private(&path, &bytes).map_err(io_error)
}

fn read_json(path: &Path) -> Result<Value, DataError> {
    let bytes = fs::read(path).map_err(io_error)?;
    serde_json::from_slice(&bytes).map_err(|_| DataError::invalid("Zarr metadata JSON is invalid."))
}

fn require_zarray(path: &Path, dtype: &str) -> Result<Value, DataError> {
    let value = read_json(path)?;
    if value.get("zarr_format").and_then(Value::as_u64) != Some(2) {
        return Err(DataError::unsupported(
            "Only Zarr format 2 is supported. Other versions are rejected.",
        ));
    }
    if !value.get("compressor").is_some_and(Value::is_null)
        || !value.get("filters").is_some_and(Value::is_null)
    {
        return Err(DataError::unsupported(
            "Compressed or filtered Zarr codecs are outside cyra.zarr.v1.",
        ));
    }
    if value.get("order").and_then(Value::as_str) != Some("C") {
        return Err(DataError::unsupported(
            "Only C-order Zarr arrays are supported.",
        ));
    }
    if value.get("dtype").and_then(Value::as_str) != Some(dtype) {
        return Err(DataError::unsupported(
            "Zarr dtype does not match the cyra.zarr.v1 array being read.",
        ));
    }
    Ok(value)
}

fn read_chunk(
    dir: &Path,
    array: &str,
    chunk: usize,
    suffix: Option<usize>,
    cache: &mut ChunkCache,
) -> Result<Vec<u8>, DataError> {
    let relative = match suffix {
        Some(second) => format!("{array}/{chunk}.{second}"),
        None => format!("{array}/{chunk}"),
    };
    if let Some(bytes) = cache.files.get(&relative) {
        return Ok(bytes.clone());
    }
    let path = dir.join(&relative);
    if !path.is_file() {
        return Err(DataError::invalid(format!(
            "Zarr chunk '{relative}' is absent. Absent chunks are not interpreted as measured zeros."
        )));
    }
    let bytes = fsutil::read_relative(dir, &relative).map_err(io_error)?;
    cache.bytes_read += bytes.len() as u64;
    cache.opened.push(relative.clone());
    cache.files.insert(relative, bytes.clone());
    Ok(bytes)
}

fn read_padded(
    dir: &Path,
    array: &str,
    chunk: usize,
    offset: usize,
    cache: &mut ChunkCache,
) -> Result<Vec<u8>, DataError> {
    require_zarray(&dir.join(array).join(".zarray"), "|u1")?;
    let bytes = read_chunk(dir, array, chunk, Some(0), cache)?;
    let start = offset * ZARR_VALUE_BYTES;
    let end = start + ZARR_VALUE_BYTES;
    if end > bytes.len() {
        return Err(DataError::invalid(
            "Zarr value chunk is shorter than the requested observation. It was not padded with a zero measurement.",
        ));
    }
    Ok(bytes[start..end].to_vec())
}

fn read_u8_at(
    dir: &Path,
    array: &str,
    chunk: usize,
    offset: usize,
    cache: &mut ChunkCache,
) -> Result<u8, DataError> {
    require_zarray(&dir.join(array).join(".zarray"), "|u1")?;
    let bytes = read_chunk(dir, array, chunk, None, cache)?;
    bytes.get(offset).copied().ok_or_else(|| {
        DataError::invalid("Zarr mask chunk does not contain the requested observation.")
    })
}

fn read_u16_at(
    dir: &Path,
    array: &str,
    chunk: usize,
    offset: usize,
    cache: &mut ChunkCache,
) -> Result<u16, DataError> {
    require_zarray(&dir.join(array).join(".zarray"), "<u2")?;
    let bytes = read_chunk(dir, array, chunk, None, cache)?;
    let start = offset * 2;
    let end = start + 2;
    if end > bytes.len() {
        return Err(DataError::invalid("Zarr length chunk is truncated."));
    }
    Ok(u16::from_le_bytes([bytes[start], bytes[start + 1]]))
}

fn decode_text(bytes: &[u8], len: u16) -> Result<Option<String>, DataError> {
    if len == 0 {
        return Ok(None);
    }
    if usize::from(len) > bytes.len() {
        return Err(DataError::invalid(
            "Declared decimal length exceeds the chunk row.",
        ));
    }
    let text = std::str::from_utf8(&bytes[..usize::from(len)])
        .map_err(|_| DataError::invalid("Stored decimal bytes are not UTF-8."))?;
    Ok(Some(text.to_owned()))
}

fn record_from_attrs(
    attrs: &Value,
    index: usize,
    presence: u8,
    reason: u8,
    value: Option<String>,
    censor_limit: Option<String>,
) -> Result<MeasurementRecord, DataError> {
    let text = |name: &str| {
        attrs
            .get(name)
            .and_then(Value::as_array)
            .and_then(|items| items.get(index))
            .ok_or_else(|| {
                DataError::invalid(format!(
                    "Zarr attributes are missing {name} at this observation."
                ))
            })
    };
    let required = |name: &str| {
        text(name)?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| DataError::invalid(format!("Zarr attribute {name} is not a string.")))
    };
    let optional = |name: &str| -> Result<Option<String>, DataError> {
        let value = text(name)?;
        if value.is_null() {
            Ok(None)
        } else {
            value
                .as_str()
                .map(|text| Some(text.to_owned()))
                .ok_or_else(|| {
                    DataError::invalid(format!("Zarr attribute {name} is not a string or null."))
                })
        }
    };
    let missing_reason = if presence == 1 {
        if reason != 0 || value.is_none() {
            return Err(DataError::invalid(
                "A measured Zarr observation has no value. Zero was not substituted.",
            ));
        }
        None
    } else {
        if value.is_some() {
            return Err(DataError::invalid(
                "A missing Zarr observation also contains a value.",
            ));
        }
        Some(MissingReason::from_code(reason).ok_or_else(|| {
            DataError::invalid("Zarr missing-reason code is not in the v1 vocabulary.")
        })?)
    };
    let lineage = serde_json::from_value(text("lineage")?.clone())
        .map_err(|_| DataError::invalid("Zarr lineage attribute is not a lineage reference."))?;
    Ok(MeasurementRecord {
        record_id: required("record_ids")?,
        sample_id: required("sample_ids")?,
        analyte_id: required("analyte_ids")?,
        modality: parse_enum(&required("modalities")?)?,
        unit: required("units")?,
        quantity_kind: parse_enum(&required("quantity_kinds")?)?,
        value,
        missing_reason,
        censor_limit,
        qc_failure: optional("qc_failures")?,
        time_amount: optional("time_amounts")?,
        time_unit: optional("time_units")?
            .map(|unit| parse_enum(&unit))
            .transpose()?,
        time_origin: optional("time_origins")?,
        assay_id: optional("assay_ids")?,
        protocol_version: optional("protocol_versions")?,
        compartment: optional("compartments")?
            .map(|compartment| parse_enum(&compartment))
            .transpose()?,
        lineage,
    })
}

fn parse_enum<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, DataError> {
    serde_json::from_value(serde_json::Value::String(text.to_owned()))
        .map_err(|_| DataError::invalid(format!("Unrecognized stored enumeration '{text}'.")))
}

fn io_error(error: std::io::Error) -> DataError {
    DataError::io(error.to_string())
}

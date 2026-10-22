//! Explicit CSV, TSV and JSON tables. Values stay strings so numbers are not coerced.
use crate::error::DataError;
use crate::model::TableFormat;
use cyra_core::strict_json;
use serde_json::Value;
use std::collections::BTreeMap;

pub fn parse_table(
    bytes: &[u8],
    format: TableFormat,
) -> Result<Vec<BTreeMap<String, String>>, DataError> {
    if bytes.len() > crate::fsutil::MAX_FILE_BYTES {
        return Err(DataError::invalid("Input table exceeds the byte limit."));
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| DataError::invalid("Input table must be UTF-8."))?;
    match format {
        TableFormat::Csv => parse_delimited(text, b','),
        TableFormat::Tsv => parse_delimited(text, b'\t'),
        TableFormat::Json => parse_json(text),
    }
}

fn parse_json(text: &str) -> Result<Vec<BTreeMap<String, String>>, DataError> {
    let value = strict_json::parse(text.as_bytes())
        .map_err(|_| DataError::invalid("JSON table is invalid or contains duplicate keys."))?;
    let rows = if let Some(records) = value.get("records") {
        records
            .as_array()
            .ok_or_else(|| DataError::invalid("JSON field records must be an array."))?
    } else if let Some(rows) = value.as_array() {
        rows
    } else {
        return Err(DataError::invalid(
            "JSON table must be an array or an object with a records array.",
        ));
    };
    if rows.len() > crate::MAX_IMPORT_ROWS {
        return Err(DataError::invalid("Input table exceeds the row limit."));
    }
    rows.iter()
        .enumerate()
        .map(|(index, row)| json_row(index + 1, row))
        .collect()
}

fn json_row(row_number: usize, value: &Value) -> Result<BTreeMap<String, String>, DataError> {
    let Some(object) = value.as_object() else {
        return Err(DataError::invalid(format!(
            "JSON row {row_number} must be an object."
        )));
    };
    let mut row = BTreeMap::new();
    for (key, item) in object {
        let text = match item {
            Value::Null => String::new(),
            Value::String(text) => text.clone(),
            _ => {
                return Err(DataError::invalid(format!(
                    "JSON row {row_number} field '{key}' must be a string or null. Numbers are not coerced."
                )));
            }
        };
        row.insert(key.clone(), text);
    }
    Ok(row)
}

fn parse_delimited(text: &str, delimiter: u8) -> Result<Vec<BTreeMap<String, String>>, DataError> {
    let mut rows = Vec::new();
    let mut record = Vec::new();
    let mut field = Vec::new();
    let mut quoted = false;
    let mut chars = text.bytes().peekable();
    while let Some(byte) = chars.next() {
        if byte == b'"' {
            if quoted && chars.peek() == Some(&b'"') {
                chars.next();
                field.push(b'"');
            } else {
                quoted = !quoted;
            }
            continue;
        }
        if quoted {
            if byte == b'\n' || byte == b'\r' {
                return Err(DataError::invalid(
                    "Quoted newlines are not part of the v1 tabular profile.",
                ));
            }
            field.push(byte);
            continue;
        }
        if byte == delimiter {
            record.push(field_text(&field)?);
            field.clear();
            continue;
        }
        if byte == b'\n' || byte == b'\r' {
            if byte == b'\r' && chars.peek() == Some(&b'\n') {
                chars.next();
            }
            record.push(field_text(&field)?);
            field.clear();
            if !(record.len() == 1 && record[0].is_empty()) {
                rows.push(std::mem::take(&mut record));
            } else {
                record.clear();
            }
            continue;
        }
        field.push(byte);
    }
    if quoted {
        return Err(DataError::invalid("Unclosed quote in the input table."));
    }
    if !field.is_empty() || !record.is_empty() {
        record.push(field_text(&field)?);
        rows.push(record);
    }
    if rows.is_empty() {
        return Err(DataError::invalid("Input table has no header."));
    }
    let header = rows.remove(0);
    if header.iter().any(String::is_empty)
        || header.len()
            != header
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
    {
        return Err(DataError::invalid(
            "Table header names must be unique and non-empty.",
        ));
    }
    if rows.len() > crate::MAX_IMPORT_ROWS {
        return Err(DataError::invalid("Input table exceeds the row limit."));
    }
    rows.into_iter()
        .enumerate()
        .map(|(index, row)| {
            if row.len() != header.len() {
                return Err(DataError::invalid(format!(
                    "Row {} does not match the header width.",
                    index + 1
                )));
            }
            Ok(header.iter().cloned().zip(row).collect())
        })
        .collect()
}

fn field_text(bytes: &[u8]) -> Result<String, DataError> {
    String::from_utf8(bytes.to_vec()).map_err(|_| DataError::invalid("A table field is not UTF-8."))
}

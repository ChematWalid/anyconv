//! In-memory data converter for CSV, TSV, and JSON formats.

use crate::dispatch::Format;
use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Convert between CSV, TSV, and JSON formats purely in-memory.
pub fn convert_data(input: &Path, output: &Path, from: &Format, to: &Format) -> Result<()> {
    match (from, to) {
        (Format::Csv, Format::Json) => csv_or_tsv_to_json(input, output, b','),
        (Format::Tsv, Format::Json) => csv_or_tsv_to_json(input, output, b'\t'),
        (Format::Json, Format::Csv) => json_to_csv_or_tsv(input, output, b','),
        (Format::Json, Format::Tsv) => json_to_csv_or_tsv(input, output, b'\t'),
        (Format::Csv, Format::Tsv) => delimit_swap(input, output, b',', b'\t'),
        (Format::Tsv, Format::Csv) => delimit_swap(input, output, b'\t', b','),
        _ => bail!("Built-in data engine does not support {} -> {}", from, to),
    }
}

fn parse_value(s: &str) -> Value {
    let trimmed = s.trim();
    if trimmed.eq_ignore_ascii_case("null") {
        return Value::Null;
    }
    if trimmed.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if let Ok(i) = trimmed.parse::<i64>() {
        return json!(i);
    }
    if let Ok(f) = trimmed.parse::<f64>() {
        if f.is_finite() {
            return json!(f);
        }
    }
    Value::String(s.to_string())
}

fn csv_or_tsv_to_json(input: &Path, output: &Path, delimiter: u8) -> Result<()> {
    let file =
        File::open(input).with_context(|| format!("Failed to open input: {}", input.display()))?;
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .from_reader(BufReader::new(file));

    let headers = reader
        .headers()
        .with_context(|| "Failed to read headers")?
        .clone();

    let mut rows: Vec<Value> = Vec::new();

    for result in reader.records() {
        let record = result.with_context(|| "Failed to read tabular record")?;
        let mut map = Map::new();
        for (i, header) in headers.iter().enumerate() {
            let field = record.get(i).unwrap_or("");
            map.insert(header.to_string(), parse_value(field));
        }
        rows.push(Value::Object(map));
    }

    let out_file = File::create(output)
        .with_context(|| format!("Failed to create output: {}", output.display()))?;
    let mut writer = BufWriter::new(out_file);
    serde_json::to_writer_pretty(&mut writer, &rows)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn json_to_csv_or_tsv(input: &Path, output: &Path, delimiter: u8) -> Result<()> {
    let file =
        File::open(input).with_context(|| format!("Failed to open input: {}", input.display()))?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let val: Value =
        serde_json::from_str(&content).with_context(|| "Failed to parse JSON input")?;

    let out_file = File::create(output)
        .with_context(|| format!("Failed to create output: {}", output.display()))?;
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(BufWriter::new(out_file));

    match val {
        Value::Array(items) => {
            if items.is_empty() {
                return Ok(());
            }

            if items.iter().all(|i| i.is_object()) {
                let mut header_set = HashSet::new();
                let mut header_list = Vec::new();

                for item in &items {
                    if let Value::Object(map) = item {
                        for key in map.keys() {
                            if header_set.insert(key.clone()) {
                                header_list.push(key.clone());
                            }
                        }
                    }
                }

                writer.write_record(&header_list)?;

                for item in &items {
                    if let Value::Object(map) = item {
                        let mut row = Vec::with_capacity(header_list.len());
                        for key in &header_list {
                            let cell = match map.get(key) {
                                Some(Value::Null) | None => String::new(),
                                Some(Value::String(s)) => s.clone(),
                                Some(other) => other.to_string(),
                            };
                            row.push(cell);
                        }
                        writer.write_record(&row)?;
                    }
                }
            } else if items.iter().all(|i| i.is_array()) {
                for item in &items {
                    if let Value::Array(row) = item {
                        let row_strings: Vec<String> = row
                            .iter()
                            .map(|v| match v {
                                Value::Null => String::new(),
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            })
                            .collect();
                        writer.write_record(&row_strings)?;
                    }
                }
            } else {
                bail!("JSON to CSV conversion expects an array of objects or an array of arrays");
            }
        }
        Value::Object(map) => {
            let headers: Vec<&String> = map.keys().collect();
            writer.write_record(&headers)?;
            let row: Vec<String> = headers
                .iter()
                .map(|k| match map.get(*k) {
                    Some(Value::Null) | None => String::new(),
                    Some(Value::String(s)) => s.clone(),
                    Some(other) => other.to_string(),
                })
                .collect();
            writer.write_record(&row)?;
        }
        _ => {
            bail!("JSON input must be an array of objects or an object to convert to CSV/TSV");
        }
    }

    writer.flush()?;
    Ok(())
}

fn delimit_swap(input: &Path, output: &Path, from_delim: u8, to_delim: u8) -> Result<()> {
    let file =
        File::open(input).with_context(|| format!("Failed to open input: {}", input.display()))?;
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(from_delim)
        .has_headers(false)
        .from_reader(BufReader::new(file));

    let out_file = File::create(output)
        .with_context(|| format!("Failed to create output: {}", output.display()))?;
    let mut writer = csv::WriterBuilder::new()
        .delimiter(to_delim)
        .from_writer(BufWriter::new(out_file));

    for result in reader.records() {
        let record = result.with_context(|| "Failed reading record during delimiter swap")?;
        writer.write_record(&record)?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_to_json_roundtrip() {
        let csv_data = "name,age,active\nAlice,30,true\nBob,25,false\n";
        let csv_file = NamedTempFile::new().unwrap();
        std::fs::write(csv_file.path(), csv_data).unwrap();

        let json_file = NamedTempFile::new().unwrap();
        convert_data(
            csv_file.path(),
            json_file.path(),
            &Format::Csv,
            &Format::Json,
        )
        .unwrap();

        let json_content = std::fs::read_to_string(json_file.path()).unwrap();
        assert!(json_content.contains("\"name\": \"Alice\""));
        assert!(json_content.contains("\"age\": 30"));
        assert!(json_content.contains("\"active\": true"));

        let csv_out = NamedTempFile::new().unwrap();
        convert_data(
            json_file.path(),
            csv_out.path(),
            &Format::Json,
            &Format::Csv,
        )
        .unwrap();

        let converted_csv = std::fs::read_to_string(csv_out.path()).unwrap();
        assert!(converted_csv.contains("Alice"));
        assert!(converted_csv.contains("30"));
        assert!(converted_csv.contains("Bob"));
        assert!(converted_csv.contains("25"));
    }

    #[test]
    fn test_csv_to_tsv() {
        let csv_data = "id,name,role\n1,Admin,Superuser\n";
        let csv_file = NamedTempFile::new().unwrap();
        std::fs::write(csv_file.path(), csv_data).unwrap();

        let tsv_file = NamedTempFile::new().unwrap();
        convert_data(csv_file.path(), tsv_file.path(), &Format::Csv, &Format::Tsv).unwrap();

        let tsv_content = std::fs::read_to_string(tsv_file.path()).unwrap();
        assert_eq!(tsv_content, "id\tname\trole\n1\tAdmin\tSuperuser\n");
    }
}

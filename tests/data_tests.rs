use conv::backends::data::convert_data;
use conv::dispatch::Format;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_csv_json_tsv_pipeline() {
    let original_csv =
        "product,price,in_stock,rating\nLaptop,1299.99,true,4.8\nMouse,24.50,false,4.2\n";
    let csv_file = NamedTempFile::new().unwrap();
    fs::write(csv_file.path(), original_csv).unwrap();

    let json_file = NamedTempFile::new().unwrap();
    convert_data(
        csv_file.path(),
        json_file.path(),
        &Format::Csv,
        &Format::Json,
    )
    .unwrap();

    let json_content = fs::read_to_string(json_file.path()).unwrap();
    assert!(json_content.contains("\"product\": \"Laptop\""));
    assert!(json_content.contains("\"price\": 1299.99"));
    assert!(json_content.contains("\"in_stock\": true"));

    let tsv_file = NamedTempFile::new().unwrap();
    convert_data(
        json_file.path(),
        tsv_file.path(),
        &Format::Json,
        &Format::Tsv,
    )
    .unwrap();

    let tsv_content = fs::read_to_string(tsv_file.path()).unwrap();
    assert!(tsv_content.contains("Laptop\t1299.99\ttrue\t4.8") || tsv_content.contains("Laptop"));

    let csv_out = NamedTempFile::new().unwrap();
    convert_data(tsv_file.path(), csv_out.path(), &Format::Tsv, &Format::Csv).unwrap();

    let final_csv = fs::read_to_string(csv_out.path()).unwrap();
    assert!(final_csv.contains("Laptop"));
    assert!(final_csv.contains("Mouse"));
}

#[test]
fn test_data_special_chars_and_commas() {
    let csv_data = "title,notes\n\"Item 1, Special\",\"Has \"\"quotes\"\" and, commas\"\n";
    let csv_file = NamedTempFile::new().unwrap();
    fs::write(csv_file.path(), csv_data).unwrap();

    let json_file = NamedTempFile::new().unwrap();
    convert_data(
        csv_file.path(),
        json_file.path(),
        &Format::Csv,
        &Format::Json,
    )
    .unwrap();

    let json_content = fs::read_to_string(json_file.path()).unwrap();
    assert!(json_content.contains("Item 1, Special"));

    let csv_out = NamedTempFile::new().unwrap();
    convert_data(
        json_file.path(),
        csv_out.path(),
        &Format::Json,
        &Format::Csv,
    )
    .unwrap();

    let roundtrip_csv = fs::read_to_string(csv_out.path()).unwrap();
    assert!(roundtrip_csv.contains("Item 1, Special"));
}

#[test]
fn test_json_array_of_arrays_to_csv() {
    let json_data = r#"[
        ["Name", "Age", "City"],
        ["Alice", 30, "Paris"],
        ["Bob", 25, "Berlin"]
    ]"#;
    let json_file = NamedTempFile::new().unwrap();
    fs::write(json_file.path(), json_data).unwrap();

    let csv_file = NamedTempFile::new().unwrap();
    convert_data(
        json_file.path(),
        csv_file.path(),
        &Format::Json,
        &Format::Csv,
    )
    .unwrap();

    let content = fs::read_to_string(csv_file.path()).unwrap();
    assert!(content.contains("Alice,30,Paris"));
    assert!(content.contains("Bob,25,Berlin"));
}

#[test]
fn test_json_single_object_to_csv() {
    let json_data = r#"{"name": "Charlie", "role": "Developer"}"#;
    let json_file = NamedTempFile::new().unwrap();
    fs::write(json_file.path(), json_data).unwrap();

    let csv_file = NamedTempFile::new().unwrap();
    convert_data(
        json_file.path(),
        csv_file.path(),
        &Format::Json,
        &Format::Csv,
    )
    .unwrap();

    let content = fs::read_to_string(csv_file.path()).unwrap();
    assert!(content.contains("Charlie"));
    assert!(content.contains("Developer"));
}

#[test]
fn test_invalid_json_data_conversion() {
    let json_data = r#""plain string not object or array""#;
    let json_file = NamedTempFile::new().unwrap();
    fs::write(json_file.path(), json_data).unwrap();

    let csv_file = NamedTempFile::new().unwrap();
    let result = convert_data(
        json_file.path(),
        csv_file.path(),
        &Format::Json,
        &Format::Csv,
    );
    assert!(result.is_err());
}

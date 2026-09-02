use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_no_args_shows_help() {
    let mut cmd = Command::cargo_bin("conv").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: conv"));
}

#[test]
fn test_cli_list_command() {
    let mut cmd = Command::cargo_bin("conv").unwrap();
    cmd.arg("--list")
        .assert()
        .success()
        .stdout(predicate::str::contains("conv — Supported Format Matrix"))
        .stdout(predicate::str::contains("FFmpeg"))
        .stdout(predicate::str::contains("Pandoc"));
}

#[test]
fn test_cli_completions() {
    let mut cmd = Command::cargo_bin("conv").unwrap();
    cmd.args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -F _conv"));
}

#[test]
fn test_cli_dry_run() {
    let dir = tempdir().unwrap();
    let csv_file = dir.path().join("data.csv");
    fs::write(&csv_file, "a,b\n1,2\n").unwrap();

    let mut cmd = Command::cargo_bin("conv").unwrap();
    cmd.args(["--dry-run", csv_file.to_str().unwrap(), "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"))
        .stdout(predicate::str::contains("Built-in Data Engine"));

    let json_file = dir.path().join("data.json");
    assert!(!json_file.exists());
}

#[test]
fn test_cli_shorthand_execution() {
    let dir = tempdir().unwrap();
    let csv_file = dir.path().join("records.csv");
    fs::write(&csv_file, "id,name\n101,Tester\n").unwrap();

    let mut cmd = Command::cargo_bin("conv").unwrap();
    cmd.args([csv_file.to_str().unwrap(), "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "All 1 conversion(s) completed successfully!",
        ));

    let json_file = dir.path().join("records.json");
    assert!(json_file.exists());
    let content = fs::read_to_string(json_file).unwrap();
    assert!(content.contains("\"id\": 101"));
    assert!(content.contains("\"name\": \"Tester\""));
}

#[test]
fn test_cli_incompatible_format_error() {
    let dir = tempdir().unwrap();
    let csv_file = dir.path().join("data.csv");
    fs::write(&csv_file, "a,b\n1,2\n").unwrap();

    let mut cmd = Command::cargo_bin("conv").unwrap();
    cmd.args([csv_file.to_str().unwrap(), "mp3"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No converter available for 'csv' -> 'mp3'",
        ))
        .stderr(predicate::str::contains("tsv"))
        .stderr(predicate::str::contains("json"));
}

#[test]
fn test_cli_force_overwrite() {
    let dir = tempdir().unwrap();
    let csv_file = dir.path().join("items.csv");
    let json_file = dir.path().join("items.json");

    fs::write(&csv_file, "x,y\n1,2\n").unwrap();
    fs::write(&json_file, "old content").unwrap();

    let mut cmd_fail = Command::cargo_bin("conv").unwrap();
    cmd_fail
        .args([csv_file.to_str().unwrap(), "json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    let mut cmd_force = Command::cargo_bin("conv").unwrap();
    cmd_force
        .args(["--force", csv_file.to_str().unwrap(), "json"])
        .assert()
        .success();

    let content = fs::read_to_string(json_file).unwrap();
    assert!(content.contains("\"x\": 1"));
}

#[test]
fn test_cli_output_dir_flag() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("custom_out");
    let csv_file = dir.path().join("data.csv");
    fs::write(&csv_file, "col1,col2\nval1,val2\n").unwrap();

    let mut cmd = Command::cargo_bin("conv").unwrap();
    cmd.args([
        "-o",
        out_dir.to_str().unwrap(),
        csv_file.to_str().unwrap(),
        "json",
    ])
    .assert()
    .success();

    let expected_out = out_dir.join("data.json");
    assert!(expected_out.exists());
}

#[test]
fn test_cli_parallel_jobs_flag() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("f1.csv");
    let file2 = dir.path().join("f2.csv");
    fs::write(&file1, "a\n1\n").unwrap();
    fs::write(&file2, "b\n2\n").unwrap();

    let mut cmd = Command::cargo_bin("conv").unwrap();
    cmd.args([
        "-j",
        "2",
        file1.to_str().unwrap(),
        file2.to_str().unwrap(),
        "json",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "2 task(s) using 2 worker thread(s)",
    ));

    assert!(dir.path().join("f1.json").exists());
    assert!(dir.path().join("f2.json").exists());
}

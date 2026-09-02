use conv::config::Config;
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_config_from_file_full() {
    let toml_content = r#"
jobs = 8
force = true
keep_going = false
output_dir = "/tmp/anyconv_test"

[backend_paths]
ffmpeg = "/custom/bin/ffmpeg"
pandoc = "/custom/bin/pandoc"
"#;
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), toml_content).unwrap();

    let cfg = Config::from_file(file.path()).unwrap();
    assert_eq!(cfg.jobs, Some(8));
    assert_eq!(cfg.force, Some(true));
    assert_eq!(cfg.keep_going, Some(false));
    assert_eq!(cfg.output_dir, Some(PathBuf::from("/tmp/anyconv_test")));
    assert_eq!(
        cfg.backend_paths.get("ffmpeg"),
        Some(&PathBuf::from("/custom/bin/ffmpeg"))
    );
    assert_eq!(
        cfg.backend_paths.get("pandoc"),
        Some(&PathBuf::from("/custom/bin/pandoc"))
    );
}

#[test]
fn test_config_from_file_partial() {
    let toml_content = r#"
jobs = 2
"#;
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), toml_content).unwrap();

    let cfg = Config::from_file(file.path()).unwrap();
    assert_eq!(cfg.jobs, Some(2));
    assert_eq!(cfg.force, None);
    assert_eq!(cfg.keep_going, None);
    assert_eq!(cfg.output_dir, None);
    assert!(cfg.backend_paths.is_empty());
}

#[test]
fn test_config_invalid_toml() {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "invalid = [toml, syntax").unwrap();

    let result = Config::from_file(file.path());
    assert!(result.is_err());
}

#[test]
fn test_config_nonexistent_file() {
    let path = PathBuf::from("/nonexistent/path/to/config.toml");
    let result = Config::from_file(&path);
    assert!(result.is_err());
}

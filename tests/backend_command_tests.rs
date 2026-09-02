use conv::backends::document::{
    build_libreoffice_command, build_pandoc_command, build_pdftoppm_command,
};
use conv::backends::ebook::build_command as build_ebook_command;
use conv::backends::ffmpeg::build_command as build_ffmpeg_command;
use conv::backends::image::build_command as build_image_command;
use conv::backends::ConversionTask;
use conv::dispatch::{Backend, Format};
use std::path::PathBuf;

#[test]
fn test_ffmpeg_command_structure() {
    let cmd = build_ffmpeg_command(
        &PathBuf::from("video.mp4"),
        &PathBuf::from("audio.mp3"),
        &Format::Mp4,
        &Format::Mp3,
        false,
    );
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();

    assert!(args.contains(&"-n".to_string()));
    assert!(args.contains(&"-vn".to_string()));
    assert!(args.contains(&"video.mp4".to_string()));
    assert!(args.contains(&"audio.mp3".to_string()));
}

#[test]
fn test_ffmpeg_force_overwrite_flag() {
    let cmd = build_ffmpeg_command(
        &PathBuf::from("video.mp4"),
        &PathBuf::from("audio.mp3"),
        &Format::Mp4,
        &Format::Mp3,
        true,
    );
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();

    assert!(args.contains(&"-y".to_string()));
    assert!(!args.contains(&"-n".to_string()));
}

#[test]
fn test_pandoc_command_structure() {
    let cmd = build_pandoc_command(&PathBuf::from("doc.md"), &PathBuf::from("doc.docx"));
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();

    assert_eq!(args, vec!["-s", "doc.md", "-o", "doc.docx"]);
}

#[test]
fn test_libreoffice_command_structure() {
    let cmd = build_libreoffice_command(&PathBuf::from("slides.pptx"), &PathBuf::from("/tmp"));
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();

    assert_eq!(
        args,
        vec![
            "--headless",
            "--convert-to",
            "pdf",
            "slides.pptx",
            "--outdir",
            "/tmp"
        ]
    );
}

#[test]
fn test_pdftoppm_format_variants() {
    let cmd_png =
        build_pdftoppm_command(&PathBuf::from("doc.pdf"), &PathBuf::from("p"), &Format::Png);
    let args_png: Vec<String> = cmd_png
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(args_png.contains(&"-png".to_string()));

    let cmd_jpg =
        build_pdftoppm_command(&PathBuf::from("doc.pdf"), &PathBuf::from("p"), &Format::Jpg);
    let args_jpg: Vec<String> = cmd_jpg
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(args_jpg.contains(&"-jpeg".to_string()));

    let cmd_tiff = build_pdftoppm_command(
        &PathBuf::from("doc.pdf"),
        &PathBuf::from("p"),
        &Format::Tiff,
    );
    let args_tiff: Vec<String> = cmd_tiff
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(args_tiff.contains(&"-tiff".to_string()));
}

#[test]
fn test_image_backend_command_flags() {
    let cmd_ico = build_image_command(
        &PathBuf::from("icon.png"),
        &PathBuf::from("icon.ico"),
        &Format::Png,
        &Format::Ico,
    );
    let args_ico: Vec<String> = cmd_ico
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(args_ico.contains(&"-define".to_string()));

    let cmd_webp = build_image_command(
        &PathBuf::from("in.png"),
        &PathBuf::from("out.webp"),
        &Format::Png,
        &Format::Webp,
    );
    let args_webp: Vec<String> = cmd_webp
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(args_webp.contains(&"-quality".to_string()));
    assert!(args_webp.contains(&"90".to_string()));
}

#[test]
fn test_ebook_command_structure() {
    let cmd = build_ebook_command(&PathBuf::from("novel.epub"), &PathBuf::from("novel.mobi"));
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert_eq!(args, vec!["novel.epub", "novel.mobi"]);
}

#[test]
fn test_conversion_task_command_string() {
    let task = ConversionTask::new(
        PathBuf::from("file.csv"),
        PathBuf::from("file.json"),
        Format::Csv,
        Format::Json,
        Backend::BuiltinData,
        false,
    );
    assert_eq!(
        task.command_string(),
        "[Built-in Engine] file.csv -> file.json"
    );
}

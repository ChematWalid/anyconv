use conv::backends::ConversionTask;
use conv::dispatch::{self, Format};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_ffmpeg_mp4_to_mp3_and_gif() {
    if which::which("ffmpeg").is_err() {
        eprintln!("Skipping FFmpeg test (ffmpeg not installed on PATH)");
        return;
    }

    let dir = tempdir().unwrap();
    let mp4_file = dir.path().join("test_video.mp4");
    let mp3_file = dir.path().join("test_audio.mp3");
    let gif_file = dir.path().join("test_anim.gif");

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=1:size=320x240:rate=10",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            mp4_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to generate test mp4");

    if !status.status.success() {
        eprintln!("Skipping FFmpeg test: could not generate test video (missing codecs)");
        return;
    }

    let backend = dispatch::dispatch(&Format::Mp4, &Format::Mp3).unwrap();
    let task = ConversionTask::new(
        mp4_file.clone(),
        mp3_file.clone(),
        Format::Mp4,
        Format::Mp3,
        backend,
        true,
    );
    task.execute().expect("Failed converting MP4 to MP3");
    assert!(mp3_file.exists());
    assert!(fs::metadata(&mp3_file).unwrap().len() > 0);

    let backend = dispatch::dispatch(&Format::Mp4, &Format::Gif).unwrap();
    let task = ConversionTask::new(
        mp4_file,
        gif_file.clone(),
        Format::Mp4,
        Format::Gif,
        backend,
        true,
    );
    task.execute().expect("Failed converting MP4 to GIF");
    assert!(gif_file.exists());
    assert!(fs::metadata(&gif_file).unwrap().len() > 0);
}

#[test]
fn test_ffmpeg_wav_to_flac() {
    if which::which("ffmpeg").is_err() {
        eprintln!("Skipping FFmpeg WAV->FLAC test");
        return;
    }

    let dir = tempdir().unwrap();
    let wav_file = dir.path().join("sound.wav");
    let flac_file = dir.path().join("sound.flac");

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=1000:duration=1",
            wav_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to generate test wav");

    if !status.status.success() {
        return;
    }

    let backend = dispatch::dispatch(&Format::Wav, &Format::Flac).unwrap();
    let task = ConversionTask::new(
        wav_file,
        flac_file.clone(),
        Format::Wav,
        Format::Flac,
        backend,
        true,
    );
    task.execute().expect("Failed converting WAV to FLAC");
    assert!(flac_file.exists());
}

#[test]
fn test_magick_png_to_webp_and_ico() {
    let has_magick = which::which("magick").is_ok() || which::which("convert").is_ok();
    if !has_magick {
        eprintln!("Skipping ImageMagick test (magick/convert not installed)");
        return;
    }

    let dir = tempdir().unwrap();
    let png_file = dir.path().join("image.png");
    let webp_file = dir.path().join("image.webp");
    let ico_file = dir.path().join("image.ico");

    let bin = if which::which("magick").is_ok() {
        "magick"
    } else {
        "convert"
    };

    let _ = Command::new(bin)
        .args(["-size", "100x100", "xc:blue", png_file.to_str().unwrap()])
        .output();

    if !png_file.exists() {
        return;
    }

    let backend = dispatch::dispatch(&Format::Png, &Format::Webp).unwrap();
    let task = ConversionTask::new(
        png_file.clone(),
        webp_file.clone(),
        Format::Png,
        Format::Webp,
        backend,
        true,
    );
    task.execute().expect("Failed converting PNG to WebP");
    assert!(webp_file.exists());

    let backend = dispatch::dispatch(&Format::Png, &Format::Ico).unwrap();
    let task = ConversionTask::new(
        png_file,
        ico_file.clone(),
        Format::Png,
        Format::Ico,
        backend,
        true,
    );
    task.execute().expect("Failed converting PNG to ICO");
    assert!(ico_file.exists());
}

#[test]
fn test_pandoc_markdown_to_html_and_docx() {
    if which::which("pandoc").is_err() {
        eprintln!("Skipping Pandoc test (pandoc not installed)");
        return;
    }

    let dir = tempdir().unwrap();
    let md_file = dir.path().join("doc.md");
    let html_file = dir.path().join("doc.html");
    let docx_file = dir.path().join("doc.docx");

    fs::write(&md_file, "# Title\n\nThis is **bold** text in markdown.\n").unwrap();

    let backend = dispatch::dispatch(&Format::Md, &Format::Html).unwrap();
    let task = ConversionTask::new(
        md_file.clone(),
        html_file.clone(),
        Format::Md,
        Format::Html,
        backend,
        true,
    );
    task.execute().expect("Failed converting MD to HTML");
    assert!(html_file.exists());
    let html = fs::read_to_string(&html_file).unwrap();
    assert!(html.contains("<h1") || html.contains("Title"));

    let backend = dispatch::dispatch(&Format::Md, &Format::Docx).unwrap();
    let task = ConversionTask::new(
        md_file,
        docx_file.clone(),
        Format::Md,
        Format::Docx,
        backend,
        true,
    );
    task.execute().expect("Failed converting MD to DOCX");
    assert!(docx_file.exists());
}

#[test]
fn test_libreoffice_docx_to_pdf() {
    if which::which("soffice").is_err() || which::which("pandoc").is_err() {
        eprintln!("Skipping LibreOffice test");
        return;
    }

    let dir = tempdir().unwrap();
    let md_file = dir.path().join("sample.md");
    let docx_file = dir.path().join("sample.docx");
    let pdf_file = dir.path().join("sample.pdf");

    fs::write(
        &md_file,
        "# Document for LibreOffice\n\nTesting PDF export.\n",
    )
    .unwrap();
    let _ = Command::new("pandoc")
        .args([md_file.to_str().unwrap(), "-o", docx_file.to_str().unwrap()])
        .output();

    if !docx_file.exists() {
        return;
    }

    let backend = dispatch::dispatch(&Format::Docx, &Format::Pdf).unwrap();
    let task = ConversionTask::new(
        docx_file,
        pdf_file.clone(),
        Format::Docx,
        Format::Pdf,
        backend,
        true,
    );
    task.execute()
        .expect("Failed converting DOCX to PDF via LibreOffice");
    assert!(pdf_file.exists());
}

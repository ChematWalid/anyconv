use conv::dispatch::{self, Backend, Category, DispatchError, Format};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_audio_conversions() {
    let audios = [
        Format::Mp3,
        Format::Wav,
        Format::Flac,
        Format::Aac,
        Format::M4a,
        Format::Ogg,
        Format::Opus,
        Format::Wma,
    ];

    for &from in &audios {
        for &to in &audios {
            let backend = dispatch::dispatch(&from, &to).unwrap();
            assert_eq!(backend, Backend::Ffmpeg);
        }
    }
}

#[test]
fn test_video_conversions() {
    let videos = [
        Format::Mp4,
        Format::Mkv,
        Format::Avi,
        Format::Mov,
        Format::Webm,
        Format::Flv,
        Format::Wmv,
    ];

    for &from in &videos {
        for &to in &videos {
            let backend = dispatch::dispatch(&from, &to).unwrap();
            assert_eq!(backend, Backend::Ffmpeg);
        }
    }
}

#[test]
fn test_video_to_audio_and_gif() {
    assert_eq!(
        dispatch::dispatch(&Format::Mp4, &Format::Mp3).unwrap(),
        Backend::Ffmpeg
    );
    assert_eq!(
        dispatch::dispatch(&Format::Mkv, &Format::Flac).unwrap(),
        Backend::Ffmpeg
    );
    assert_eq!(
        dispatch::dispatch(&Format::Webm, &Format::Wav).unwrap(),
        Backend::Ffmpeg
    );
    assert_eq!(
        dispatch::dispatch(&Format::Mp4, &Format::Gif).unwrap(),
        Backend::Ffmpeg
    );
    assert_eq!(
        dispatch::dispatch(&Format::Gif, &Format::Mp4).unwrap(),
        Backend::Ffmpeg
    );
}

#[test]
fn test_image_conversions() {
    let images = [
        Format::Png,
        Format::Jpg,
        Format::Jpeg,
        Format::Webp,
        Format::Svg,
        Format::Ico,
        Format::Bmp,
        Format::Tiff,
        Format::Avif,
    ];

    for &from in &images {
        for &to in &images {
            let backend = dispatch::dispatch(&from, &to).unwrap();
            assert_eq!(backend, Backend::Magick);
        }
        assert_eq!(
            dispatch::dispatch(&from, &Format::Pdf).unwrap(),
            Backend::Magick
        );
    }
}

#[test]
fn test_document_and_markup_conversions() {
    assert_eq!(
        dispatch::dispatch(&Format::Md, &Format::Html).unwrap(),
        Backend::Pandoc
    );
    assert_eq!(
        dispatch::dispatch(&Format::Md, &Format::Docx).unwrap(),
        Backend::Pandoc
    );
    assert_eq!(
        dispatch::dispatch(&Format::Md, &Format::Tex).unwrap(),
        Backend::Pandoc
    );
    assert_eq!(
        dispatch::dispatch(&Format::Html, &Format::Md).unwrap(),
        Backend::Pandoc
    );
    assert_eq!(
        dispatch::dispatch(&Format::Md, &Format::Pdf).unwrap(),
        Backend::Pandoc
    );

    assert_eq!(
        dispatch::dispatch(&Format::Docx, &Format::Pdf).unwrap(),
        Backend::LibreOffice
    );
    assert_eq!(
        dispatch::dispatch(&Format::Pptx, &Format::Pdf).unwrap(),
        Backend::LibreOffice
    );
    assert_eq!(
        dispatch::dispatch(&Format::Xlsx, &Format::Pdf).unwrap(),
        Backend::LibreOffice
    );
    assert_eq!(
        dispatch::dispatch(&Format::Odt, &Format::Pdf).unwrap(),
        Backend::LibreOffice
    );

    assert_eq!(
        dispatch::dispatch(&Format::Pdf, &Format::Png).unwrap(),
        Backend::Pdftoppm
    );
    assert_eq!(
        dispatch::dispatch(&Format::Pdf, &Format::Jpg).unwrap(),
        Backend::Pdftoppm
    );
    assert_eq!(
        dispatch::dispatch(&Format::Pdf, &Format::Tiff).unwrap(),
        Backend::Pdftoppm
    );
}

#[test]
fn test_ebook_conversions() {
    assert_eq!(
        dispatch::dispatch(&Format::Epub, &Format::Mobi).unwrap(),
        Backend::EbookConvert
    );
    assert_eq!(
        dispatch::dispatch(&Format::Epub, &Format::Azw3).unwrap(),
        Backend::EbookConvert
    );
    assert_eq!(
        dispatch::dispatch(&Format::Mobi, &Format::Epub).unwrap(),
        Backend::EbookConvert
    );
    assert_eq!(
        dispatch::dispatch(&Format::Fb2, &Format::Epub).unwrap(),
        Backend::EbookConvert
    );
    assert_eq!(
        dispatch::dispatch(&Format::Epub, &Format::Htmlz).unwrap(),
        Backend::EbookConvert
    );
}

#[test]
fn test_unsupported_pair_error() {
    let result = dispatch::dispatch(&Format::Csv, &Format::Mp4);
    assert!(result.is_err());
    if let Err(DispatchError::UnsupportedConversion {
        from,
        to,
        available_targets,
    }) = result
    {
        assert_eq!(from, Format::Csv);
        assert_eq!(to, Format::Mp4);
        assert!(available_targets.contains(&Format::Json));
        assert!(available_targets.contains(&Format::Tsv));
    } else {
        panic!("Expected UnsupportedConversion");
    }
}

#[test]
fn test_format_from_ext_cases() {
    assert_eq!(Format::from_ext("MP4"), Some(Format::Mp4));
    assert_eq!(Format::from_ext(".png"), Some(Format::Png));
    assert_eq!(Format::from_ext("markdown"), Some(Format::Md));
    assert_eq!(Format::from_ext("htm"), Some(Format::Html));
    assert_eq!(Format::from_ext("jpeg"), Some(Format::Jpeg));
    assert_eq!(Format::from_ext("tif"), Some(Format::Tiff));
    assert_eq!(Format::from_ext("unknown_ext"), None);
}

#[test]
fn test_format_display() {
    for fmt in Format::all() {
        assert_eq!(format!("{}", fmt), fmt.ext());
    }
}

#[test]
fn test_format_categories() {
    assert_eq!(Format::Mp4.category(), Category::Video);
    assert_eq!(Format::Mp3.category(), Category::Audio);
    assert_eq!(Format::Png.category(), Category::Image);
    assert_eq!(Format::Epub.category(), Category::Ebook);
    assert_eq!(Format::Pdf.category(), Category::Document);
    assert_eq!(Format::Csv.category(), Category::Data);
}

#[test]
fn test_sniffing_text_formats() {
    let json_file = NamedTempFile::new().unwrap();
    fs::write(json_file.path(), "{\n  \"test\": true\n}\n").unwrap();
    assert_eq!(Format::sniff(json_file.path()), Some(Format::Json));

    let html_file = NamedTempFile::new().unwrap();
    fs::write(
        html_file.path(),
        "<!DOCTYPE html><html><body>Hi</body></html>",
    )
    .unwrap();
    assert_eq!(Format::sniff(html_file.path()), Some(Format::Html));

    let svg_file = NamedTempFile::new().unwrap();
    fs::write(
        svg_file.path(),
        "<svg viewBox=\"0 0 100 100\"><circle cx=\"50\" cy=\"50\" r=\"40\"/></svg>",
    )
    .unwrap();
    assert_eq!(Format::sniff(svg_file.path()), Some(Format::Svg));

    let md_file = NamedTempFile::new().unwrap();
    fs::write(md_file.path(), "# Header\n\nSome paragraph.\n").unwrap();
    assert_eq!(Format::sniff(md_file.path()), Some(Format::Md));

    let empty_file = NamedTempFile::new().unwrap();
    assert_eq!(Format::sniff(empty_file.path()), None);
}

#[test]
fn test_backend_metadata() {
    assert_eq!(Backend::Ffmpeg.binary_name(), Some("ffmpeg"));
    assert_eq!(Backend::EbookConvert.binary_name(), Some("ebook-convert"));
    assert_eq!(Backend::Pandoc.binary_name(), Some("pandoc"));
    assert_eq!(Backend::LibreOffice.binary_name(), Some("soffice"));
    assert_eq!(Backend::Pdftoppm.binary_name(), Some("pdftoppm"));
    assert_eq!(Backend::Magick.binary_name(), Some("magick"));
    assert_eq!(Backend::BuiltinData.binary_name(), None);

    for backend in [
        Backend::Ffmpeg,
        Backend::EbookConvert,
        Backend::Pandoc,
        Backend::LibreOffice,
        Backend::Pdftoppm,
        Backend::Magick,
        Backend::BuiltinData,
    ] {
        assert!(!backend.display_name().is_empty());
    }
}

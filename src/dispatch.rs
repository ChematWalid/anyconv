//! Pure format detection and backend routing module.

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

/// Supported file formats across all categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Mp4,
    Mkv,
    Avi,
    Mov,
    Webm,
    Flv,
    Wmv,

    Mp3,
    Wav,
    Flac,
    Aac,
    M4a,
    Ogg,
    Opus,
    Wma,

    Gif,

    Png,
    Jpg,
    Jpeg,
    Webp,
    Svg,
    Ico,
    Bmp,
    Tiff,
    Avif,

    Epub,
    Mobi,
    Azw3,
    Azw,
    Fb2,
    Htmlz,

    Pdf,
    Md,
    Html,
    Docx,
    Doc,
    Pptx,
    Ppt,
    Xlsx,
    Xls,
    Odt,
    Ods,
    Odp,
    Tex,
    Rst,
    Rtf,
    Txt,

    Csv,
    Tsv,
    Json,
}

/// High-level categorization of file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Video,
    Audio,
    Image,
    Ebook,
    Document,
    Data,
}

impl Format {
    /// Canonical file extension associated with the format.
    pub fn ext(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
            Self::Avi => "avi",
            Self::Mov => "mov",
            Self::Webm => "webm",
            Self::Flv => "flv",
            Self::Wmv => "wmv",
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Aac => "aac",
            Self::M4a => "m4a",
            Self::Ogg => "ogg",
            Self::Opus => "opus",
            Self::Wma => "wma",
            Self::Gif => "gif",
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Jpeg => "jpeg",
            Self::Webp => "webp",
            Self::Svg => "svg",
            Self::Ico => "ico",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Avif => "avif",
            Self::Epub => "epub",
            Self::Mobi => "mobi",
            Self::Azw3 => "azw3",
            Self::Azw => "azw",
            Self::Fb2 => "fb2",
            Self::Htmlz => "htmlz",
            Self::Pdf => "pdf",
            Self::Md => "md",
            Self::Html => "html",
            Self::Docx => "docx",
            Self::Doc => "doc",
            Self::Pptx => "pptx",
            Self::Ppt => "ppt",
            Self::Xlsx => "xlsx",
            Self::Xls => "xls",
            Self::Odt => "odt",
            Self::Ods => "ods",
            Self::Odp => "odp",
            Self::Tex => "tex",
            Self::Rst => "rst",
            Self::Rtf => "rtf",
            Self::Txt => "txt",
            Self::Csv => "csv",
            Self::Tsv => "tsv",
            Self::Json => "json",
        }
    }

    /// Category mapping for this format.
    pub fn category(&self) -> Category {
        match self {
            Self::Mp4 | Self::Mkv | Self::Avi | Self::Mov | Self::Webm | Self::Flv | Self::Wmv => {
                Category::Video
            }
            Self::Mp3
            | Self::Wav
            | Self::Flac
            | Self::Aac
            | Self::M4a
            | Self::Ogg
            | Self::Opus
            | Self::Wma => Category::Audio,
            Self::Png
            | Self::Jpg
            | Self::Jpeg
            | Self::Webp
            | Self::Svg
            | Self::Ico
            | Self::Bmp
            | Self::Tiff
            | Self::Avif => Category::Image,
            Self::Gif => Category::Image,
            Self::Epub | Self::Mobi | Self::Azw3 | Self::Azw | Self::Fb2 | Self::Htmlz => {
                Category::Ebook
            }
            Self::Pdf
            | Self::Md
            | Self::Html
            | Self::Docx
            | Self::Doc
            | Self::Pptx
            | Self::Ppt
            | Self::Xlsx
            | Self::Xls
            | Self::Odt
            | Self::Ods
            | Self::Odp
            | Self::Tex
            | Self::Rst
            | Self::Rtf
            | Self::Txt => Category::Document,
            Self::Csv | Self::Tsv | Self::Json => Category::Data,
        }
    }

    /// Resolves format from a string extension.
    pub fn from_ext(ext: &str) -> Option<Self> {
        let clean = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        match clean.as_str() {
            "mp4" | "m4v" => Some(Self::Mp4),
            "mkv" => Some(Self::Mkv),
            "avi" => Some(Self::Avi),
            "mov" => Some(Self::Mov),
            "webm" => Some(Self::Webm),
            "flv" => Some(Self::Flv),
            "wmv" => Some(Self::Wmv),

            "mp3" => Some(Self::Mp3),
            "wav" => Some(Self::Wav),
            "flac" => Some(Self::Flac),
            "aac" => Some(Self::Aac),
            "m4a" => Some(Self::M4a),
            "ogg" | "oga" => Some(Self::Ogg),
            "opus" => Some(Self::Opus),
            "wma" => Some(Self::Wma),

            "gif" => Some(Self::Gif),

            "png" => Some(Self::Png),
            "jpg" => Some(Self::Jpg),
            "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::Webp),
            "svg" => Some(Self::Svg),
            "ico" => Some(Self::Ico),
            "bmp" => Some(Self::Bmp),
            "tiff" | "tif" => Some(Self::Tiff),
            "avif" => Some(Self::Avif),

            "epub" => Some(Self::Epub),
            "mobi" => Some(Self::Mobi),
            "azw3" => Some(Self::Azw3),
            "azw" => Some(Self::Azw),
            "fb2" => Some(Self::Fb2),
            "htmlz" => Some(Self::Htmlz),

            "pdf" => Some(Self::Pdf),
            "md" | "markdown" => Some(Self::Md),
            "html" | "htm" => Some(Self::Html),
            "docx" => Some(Self::Docx),
            "doc" => Some(Self::Doc),
            "pptx" => Some(Self::Pptx),
            "ppt" => Some(Self::Ppt),
            "xlsx" => Some(Self::Xlsx),
            "xls" => Some(Self::Xls),
            "odt" => Some(Self::Odt),
            "ods" => Some(Self::Ods),
            "odp" => Some(Self::Odp),
            "tex" | "latex" => Some(Self::Tex),
            "rst" => Some(Self::Rst),
            "rtf" => Some(Self::Rtf),
            "txt" => Some(Self::Txt),

            "csv" => Some(Self::Csv),
            "tsv" => Some(Self::Tsv),
            "json" => Some(Self::Json),

            _ => None,
        }
    }

    /// Detect format from a file path by checking extension first and magic bytes as fallback.
    pub fn detect(path: &Path) -> Option<Self> {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(fmt) = Self::from_ext(ext) {
                return Some(fmt);
            }
        }
        Self::sniff(path)
    }

    /// Sniff magic bytes or header structure of a file.
    pub fn sniff(path: &Path) -> Option<Self> {
        if let Ok(Some(kind)) = infer::get_from_path(path) {
            if let Some(fmt) = Self::from_ext(kind.extension()) {
                return Some(fmt);
            }
        }

        let mut file = File::open(path).ok()?;
        let mut buffer = [0u8; 512];
        let bytes_read = file.read(&mut buffer).ok()?;
        if bytes_read == 0 {
            return None;
        }
        let text = std::str::from_utf8(&buffer[..bytes_read])
            .ok()?
            .trim_start();

        if text.starts_with('{') || text.starts_with('[') {
            return Some(Self::Json);
        }
        if text.starts_with("<?xml")
            || text.starts_with("<html")
            || text.starts_with("<!DOCTYPE html")
        {
            return Some(Self::Html);
        }
        if text.starts_with("<svg") || text.contains("<svg") {
            return Some(Self::Svg);
        }
        if text.starts_with("# ") || text.starts_with("---\n") {
            return Some(Self::Md);
        }

        None
    }

    /// Returns a slice of all supported formats.
    pub fn all() -> &'static [Format] {
        &[
            Self::Mp4,
            Self::Mkv,
            Self::Avi,
            Self::Mov,
            Self::Webm,
            Self::Flv,
            Self::Wmv,
            Self::Mp3,
            Self::Wav,
            Self::Flac,
            Self::Aac,
            Self::M4a,
            Self::Ogg,
            Self::Opus,
            Self::Wma,
            Self::Gif,
            Self::Png,
            Self::Jpg,
            Self::Jpeg,
            Self::Webp,
            Self::Svg,
            Self::Ico,
            Self::Bmp,
            Self::Tiff,
            Self::Avif,
            Self::Epub,
            Self::Mobi,
            Self::Azw3,
            Self::Azw,
            Self::Fb2,
            Self::Htmlz,
            Self::Pdf,
            Self::Md,
            Self::Html,
            Self::Docx,
            Self::Doc,
            Self::Pptx,
            Self::Ppt,
            Self::Xlsx,
            Self::Xls,
            Self::Odt,
            Self::Ods,
            Self::Odp,
            Self::Tex,
            Self::Rst,
            Self::Rtf,
            Self::Txt,
            Self::Csv,
            Self::Tsv,
            Self::Json,
        ]
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ext())
    }
}

/// Execution backend responsible for carrying out conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backend {
    Ffmpeg,
    EbookConvert,
    Pandoc,
    LibreOffice,
    Pdftoppm,
    Magick,
    BuiltinData,
}

impl Backend {
    /// Underlying binary name for external tools, or `None` for built-in engines.
    pub fn binary_name(&self) -> Option<&'static str> {
        match self {
            Self::Ffmpeg => Some("ffmpeg"),
            Self::EbookConvert => Some("ebook-convert"),
            Self::Pandoc => Some("pandoc"),
            Self::LibreOffice => Some("soffice"),
            Self::Pdftoppm => Some("pdftoppm"),
            Self::Magick => Some("magick"),
            Self::BuiltinData => None,
        }
    }

    /// User-facing descriptive name of the backend.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ffmpeg => "FFmpeg (Audio/Video)",
            Self::EbookConvert => "Calibre ebook-convert",
            Self::Pandoc => "Pandoc (Document markup)",
            Self::LibreOffice => "LibreOffice Headless",
            Self::Pdftoppm => "pdftoppm (PDF to Raster)",
            Self::Magick => "ImageMagick",
            Self::BuiltinData => "Built-in Data Engine (Rust)",
        }
    }

    /// Actionable installation hints across package managers.
    pub fn install_hint(&self) -> &'static str {
        match self {
            Self::Ffmpeg => "pacman -S ffmpeg (or apt install ffmpeg / brew install ffmpeg)",
            Self::EbookConvert => {
                "pacman -S calibre (or apt install calibre / brew install calibre)"
            }
            Self::Pandoc => "pacman -S pandoc-cli (or apt install pandoc / brew install pandoc)",
            Self::LibreOffice => "pacman -S libreoffice-fresh (or apt install libreoffice)",
            Self::Pdftoppm => {
                "pacman -S poppler (or apt install poppler-utils / brew install poppler)"
            }
            Self::Magick => {
                "pacman -S imagemagick (or apt install imagemagick / brew install imagemagick)"
            }
            Self::BuiltinData => "",
        }
    }
}

/// Errors returned when dispatch routing cannot resolve a suitable backend.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DispatchError {
    #[error("No converter available for '{from}' -> '{to}'. Supported targets for '{from}': {}", format_targets(.available_targets))]
    UnsupportedConversion {
        from: Format,
        to: Format,
        available_targets: Vec<Format>,
    },
    #[error("Unknown or unsupported format: '{0}'")]
    UnknownFormat(String),
}

fn format_targets(targets: &[Format]) -> String {
    if targets.is_empty() {
        "none".to_string()
    } else {
        let exts: Vec<&str> = targets.iter().map(|t| t.ext()).collect();
        exts.join(", ")
    }
}

/// Internal non-recursive routing logic.
pub fn dispatch_internal(from: &Format, to: &Format) -> Option<Backend> {
    if from == to {
        return match from.category() {
            Category::Audio | Category::Video => Some(Backend::Ffmpeg),
            Category::Image => Some(Backend::Magick),
            Category::Data => Some(Backend::BuiltinData),
            Category::Document | Category::Ebook => Some(Backend::EbookConvert),
        };
    }

    if is_data_format(from) && is_data_format(to) {
        return Some(Backend::BuiltinData);
    }

    if is_ffmpeg_pair(from, to) {
        return Some(Backend::Ffmpeg);
    }

    if *from == Format::Pdf && matches!(to, Format::Png | Format::Jpg | Format::Jpeg | Format::Tiff)
    {
        return Some(Backend::Pdftoppm);
    }

    if is_image_format(from) && (is_image_format(to) || *to == Format::Pdf) {
        return Some(Backend::Magick);
    }

    if is_office_document(from) && *to == Format::Pdf {
        return Some(Backend::LibreOffice);
    }

    if is_pandoc_pair(from, to) {
        return Some(Backend::Pandoc);
    }

    if is_ebook_convert_pair(from, to) {
        return Some(Backend::EbookConvert);
    }

    None
}

/// Pure dispatch function routing `(from, to)` to the optimal Backend or returning an actionable error.
pub fn dispatch(from: &Format, to: &Format) -> Result<Backend, DispatchError> {
    if let Some(backend) = dispatch_internal(from, to) {
        Ok(backend)
    } else {
        let available = get_supported_targets(from);
        Err(DispatchError::UnsupportedConversion {
            from: *from,
            to: *to,
            available_targets: available,
        })
    }
}

/// Retrieves all supported target formats for a given input format.
pub fn get_supported_targets(from: &Format) -> Vec<Format> {
    Format::all()
        .iter()
        .copied()
        .filter(|to| from != to && dispatch_internal(from, to).is_some())
        .collect()
}

fn is_data_format(f: &Format) -> bool {
    matches!(f, Format::Csv | Format::Tsv | Format::Json)
}

fn is_audio_format(f: &Format) -> bool {
    matches!(
        f,
        Format::Mp3
            | Format::Wav
            | Format::Flac
            | Format::Aac
            | Format::M4a
            | Format::Ogg
            | Format::Opus
            | Format::Wma
    )
}

fn is_video_format(f: &Format) -> bool {
    matches!(
        f,
        Format::Mp4
            | Format::Mkv
            | Format::Avi
            | Format::Mov
            | Format::Webm
            | Format::Flv
            | Format::Wmv
    )
}

fn is_image_format(f: &Format) -> bool {
    matches!(
        f,
        Format::Png
            | Format::Jpg
            | Format::Jpeg
            | Format::Webp
            | Format::Svg
            | Format::Ico
            | Format::Bmp
            | Format::Tiff
            | Format::Avif
            | Format::Gif
    )
}

fn is_ffmpeg_pair(from: &Format, to: &Format) -> bool {
    let from_video = is_video_format(from);
    let from_audio = is_audio_format(from);
    let to_video = is_video_format(to);
    let to_audio = is_audio_format(to);

    if (from_video && (to_video || to_audio)) || (from_audio && to_audio) {
        return true;
    }

    if from_video && *to == Format::Gif {
        return true;
    }

    if *from == Format::Gif && to_video {
        return true;
    }

    false
}

fn is_pandoc_pair(from: &Format, to: &Format) -> bool {
    let pandoc_formats = [
        Format::Md,
        Format::Html,
        Format::Docx,
        Format::Tex,
        Format::Rst,
        Format::Epub,
        Format::Txt,
    ];

    if pandoc_formats.contains(from) && pandoc_formats.contains(to) && from != to {
        return true;
    }

    if matches!(from, Format::Md | Format::Rst | Format::Html) && *to == Format::Pdf {
        return true;
    }

    false
}

fn is_office_document(f: &Format) -> bool {
    matches!(
        f,
        Format::Docx
            | Format::Doc
            | Format::Pptx
            | Format::Ppt
            | Format::Xlsx
            | Format::Xls
            | Format::Odt
            | Format::Ods
            | Format::Odp
            | Format::Rtf
    )
}

fn is_ebook_convert_pair(from: &Format, to: &Format) -> bool {
    let ebook_formats = [
        Format::Epub,
        Format::Mobi,
        Format::Azw3,
        Format::Azw,
        Format::Fb2,
        Format::Pdf,
        Format::Txt,
        Format::Rtf,
        Format::Htmlz,
    ];

    ebook_formats.contains(from) && ebook_formats.contains(to) && from != to
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffmpeg_routing() {
        assert_eq!(
            dispatch(&Format::Mp4, &Format::Mp3).unwrap(),
            Backend::Ffmpeg
        );
        assert_eq!(
            dispatch(&Format::Wav, &Format::Flac).unwrap(),
            Backend::Ffmpeg
        );
        assert_eq!(
            dispatch(&Format::Mkv, &Format::Mp4).unwrap(),
            Backend::Ffmpeg
        );
        assert_eq!(
            dispatch(&Format::Mp4, &Format::Gif).unwrap(),
            Backend::Ffmpeg
        );
    }

    #[test]
    fn test_image_routing() {
        assert_eq!(
            dispatch(&Format::Png, &Format::Webp).unwrap(),
            Backend::Magick
        );
        assert_eq!(
            dispatch(&Format::Jpg, &Format::Png).unwrap(),
            Backend::Magick
        );
        assert_eq!(
            dispatch(&Format::Svg, &Format::Png).unwrap(),
            Backend::Magick
        );
        assert_eq!(
            dispatch(&Format::Png, &Format::Ico).unwrap(),
            Backend::Magick
        );
        assert_eq!(
            dispatch(&Format::Png, &Format::Pdf).unwrap(),
            Backend::Magick
        );
    }

    #[test]
    fn test_data_routing() {
        assert_eq!(
            dispatch(&Format::Csv, &Format::Json).unwrap(),
            Backend::BuiltinData
        );
        assert_eq!(
            dispatch(&Format::Json, &Format::Csv).unwrap(),
            Backend::BuiltinData
        );
        assert_eq!(
            dispatch(&Format::Tsv, &Format::Json).unwrap(),
            Backend::BuiltinData
        );
        assert_eq!(
            dispatch(&Format::Csv, &Format::Tsv).unwrap(),
            Backend::BuiltinData
        );
    }

    #[test]
    fn test_document_and_ebook_routing() {
        assert_eq!(
            dispatch(&Format::Epub, &Format::Mobi).unwrap(),
            Backend::EbookConvert
        );
        assert_eq!(
            dispatch(&Format::Epub, &Format::Pdf).unwrap(),
            Backend::EbookConvert
        );
        assert_eq!(
            dispatch(&Format::Md, &Format::Html).unwrap(),
            Backend::Pandoc
        );
        assert_eq!(
            dispatch(&Format::Md, &Format::Docx).unwrap(),
            Backend::Pandoc
        );
        assert_eq!(
            dispatch(&Format::Docx, &Format::Pdf).unwrap(),
            Backend::LibreOffice
        );
        assert_eq!(
            dispatch(&Format::Pptx, &Format::Pdf).unwrap(),
            Backend::LibreOffice
        );
        assert_eq!(
            dispatch(&Format::Xlsx, &Format::Pdf).unwrap(),
            Backend::LibreOffice
        );
        assert_eq!(
            dispatch(&Format::Pdf, &Format::Png).unwrap(),
            Backend::Pdftoppm
        );
    }

    #[test]
    fn test_unsupported_pair_gives_clean_error() {
        let err = dispatch(&Format::Csv, &Format::Mp3).unwrap_err();
        match err {
            DispatchError::UnsupportedConversion {
                from,
                to,
                available_targets,
            } => {
                assert_eq!(from, Format::Csv);
                assert_eq!(to, Format::Mp3);
                assert!(available_targets.contains(&Format::Json));
                assert!(available_targets.contains(&Format::Tsv));
                assert!(!available_targets.contains(&Format::Mp3));
            }
            _ => panic!("Expected UnsupportedConversion error"),
        }
    }
}

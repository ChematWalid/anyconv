# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-02

### Added
- **Universal Multi-Engine Dispatcher**: Intelligently routes file conversions to the optimal backend engine based on input and target format.
- **Audio & Video Engine (`ffmpeg`)**:
  - Video $\leftrightarrow$ Video remux/transcoding (`mp4`, `mkv`, `avi`, `mov`, `webm`, `flv`, `wmv`)
  - Video $\rightarrow$ Audio extraction (`mp4` $\rightarrow$ `mp3`, `mkv` $\rightarrow$ `flac`, etc.)
  - Video $\rightarrow$ GIF animation with Lanczos scaling
  - Audio $\leftrightarrow$ Audio transcoding (`wav`, `mp3`, `flac`, `aac`, `m4a`, `ogg`, `opus`, `wma`)
- **E-Book Engine (`ebook-convert` / Calibre)**:
  - Bidirectional conversions between `epub`, `mobi`, `azw3`, `azw`, `fb2`, `pdf`, `txt`, `rtf`, `htmlz`
- **Document & Markup Engine (`pandoc`, `libreoffice`, `pdftoppm`)**:
  - Markdown $\leftrightarrow$ `html`, `docx`, `tex`, `rst`, `epub` via Pandoc
  - Office docs (`docx`, `pptx`, `xlsx`, `odt`, `ods`, `odp`) $\rightarrow$ `pdf` via headless LibreOffice
  - PDF $\rightarrow$ raster images (`png`, `jpg`, `tiff`) via `pdftoppm`
- **Image Engine (`magick` / ImageMagick)**:
  - High-performance conversion between `png`, `jpg`, `jpeg`, `webp`, `svg`, `ico`, `bmp`, `tiff`, `avif`
  - Multi-resolution Windows ICO generation
  - SVG rasterization with delegate detection
- **Built-in Pure Rust Data Engine**:
  - Sub-millisecond in-memory conversions between `csv`, `tsv`, and `json` with type preservation and robust escaping
- **CLI Features**:
  - Shorthand target extensions: `conv input.mp4 mp3`
  - Explicit targets: `conv input.mp4 output.mp3`
  - Shell & literal glob batch conversions: `conv *.png webp`
  - Multi-threaded parallel conversions via Rayon (`--jobs <N>`)
  - Dry-run mode (`--dry-run`)
  - Force overwrite protection (`--force`)
  - Fault-tolerant batch processing (`--keep-going`)
  - Target output directory routing (`--output-dir <DIR>`)
  - Complete format matrix table (`--list`)
  - Shell completion generation (`conv completions <shell>`)
- **Configuration Support**:
  - Optional `~/.config/conv/config.toml` file to customize default worker jobs, overwrite policy, and custom backend paths.

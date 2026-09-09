# conv (anyconv) 🚀

> **Universal Smart Converter CLI** written in Rust — blazingly fast, multi-threaded, and intelligent format routing.

`conv` (also aliased as `anyconv`) eliminates the friction of remembering dozens of CLI converter tools (`ffmpeg`, `pandoc`, `ebook-convert`, `libreoffice`, `magick`, `pdftoppm`). You simply pass your file and desired format, and `conv` inspects the file, detects the format, and delegates to the best-suited engine.

---

## Features

- **⚡ Blazing Fast (<1 ms Startup)**: Compiled native Rust binary with zero interpreter latency.
- **🧵 Multi-Threaded Parallel Conversions**: Automatically processes batches across CPU cores simultaneously with configurable concurrency (`--jobs`).
- **🧠 Format Sniffing**: Uses file extension first, with automatic magic byte header sniffing fallback if extension is missing.
- **📊 In-Memory Data Engine**: Sub-millisecond conversions between CSV, TSV, and JSON purely inside Rust memory (no subprocesses spawned).
- **🛡️ Safe by Default**: Refuses to overwrite files or overwrite inputs unless `--force` is specified.
- **✨ Flexible Syntax**: Supports explicit filenames (`conv in.mp4 out.mp3`), extension shorthand (`conv in.mp4 mp3`), and wildcards (`conv *.png webp`).
- **🌐 Shell Completions**: First-class shell auto-completion for `bash`, `zsh`, `fish`, `elvish`, and `powershell`.

---

## Non-Conflict with ImageMagick (`/usr/bin/convert`)

On Linux systems, ImageMagick provides a legacy utility named `/usr/bin/convert` (restricted to image transformations). 

To ensure **100% compatibility and zero system breakage**, this tool is named **`conv`** and symlinked as **`anyconv`** in `/usr/local/bin` or `~/.local/bin`. Your system's existing ImageMagick `convert` binary remains completely untouched.

---

## Conversion Matrix

| Category | Formats Handled | Backend Engine | Capabilities |
| :--- | :--- | :--- | :--- |
| **Audio** | `mp3`, `wav`, `flac`, `aac`, `m4a`, `ogg`, `opus`, `wma` | **FFmpeg** | Lossy/lossless audio transcoding, metadata retention |
| **Video** | `mp4`, `mkv`, `avi`, `mov`, `webm`, `flv`, `wmv` | **FFmpeg** | Video remuxing, video $\rightarrow$ audio, video $\rightarrow$ GIF (Lanczos filter) |
| **Images** | `png`, `jpg`, `jpeg`, `webp`, `svg`, `ico`, `bmp`, `tiff`, `avif` | **ImageMagick** | Raster/vector conversion, multi-resolution `.ico` generation, WebP compression |
| **E-Books** | `epub`, `mobi`, `azw3`, `azw`, `fb2`, `pdf`, `txt`, `rtf`, `htmlz` | **Calibre (`ebook-convert`)** | Full digital library format conversions with typography preserving |
| **Documents & Markup** | `md`, `html`, `docx`, `tex`, `rst`, `epub`, `pdf` | **Pandoc** | Markdown/HTML/LaTeX cross-conversion |
| **Office $\rightarrow$ PDF** | `docx`, `pptx`, `xlsx`, `odt`, `ods`, `odp` $\rightarrow$ `pdf` | **LibreOffice Headless** | Batch document/spreadsheet/presentation PDF rendering |
| **PDF $\rightarrow$ Raster** | `pdf` $\rightarrow$ `png`, `jpg`, `tiff` | **pdftoppm** | Page-by-page high DPI raster image extraction |
| **Data Tables** | `csv` $\leftrightarrow$ `json`, `tsv` $\leftrightarrow$ `json`, `csv` $\leftrightarrow$ `tsv` | **Rust Native Memory** | Instant in-memory structured data transformations |

---

## Installation

### Option 1: Quick Install Script
```bash
git clone https://github.com/ChematWalid/anyconv.git
cd anyconv
./install.sh
```

### Option 2: Build & Install with Cargo
```bash
cargo install --path .
```

### Option 3: Manual Installation
```bash
cargo build --release
sudo cp target/release/conv /usr/local/bin/conv
sudo ln -sf /usr/local/bin/conv /usr/local/bin/anyconv
```

---

## Usage Examples

### 1. Basic & Shorthand Conversion
```bash
# Explicit output name
conv video.mp4 audio.mp3

# Shorthand target format (automatically writes song.mp3 in the same directory)
conv song.wav mp3

# Convert markdown note to HTML
conv README.md README.html

# Convert EPUB to Kindle format
conv book.epub mobi
```

### 2. Batch & Wildcard Conversions
```bash
# Convert all PNG images in current directory to WebP
conv *.png webp

# Convert multiple specific files to MP3
conv intro.mp4 track1.wav voice.m4a mp3

# Parallel batch conversion with custom thread count (e.g. 8 threads)
conv -j 8 *.jpg webp

# Route all converted outputs to a specific directory
conv *.docx pdf --output-dir ./converted_pdfs/
```

### 3. Dry-Run & Force Overwrite
```bash
# Preview planned conversions and backend engines without executing
conv --dry-run *.mp4 gif

# Overwrite existing destination files without prompting
conv --force input.csv output.json

# Continue converting remaining files even if one file errors
conv --keep-going *.png webp
```

### 4. Viewing Format Matrix & Help
```bash
# View live status of installed backends and supported conversions
conv --list

# CLI help
conv --help
```

### 5. Generating Shell Completions
```bash
# Bash
conv completions bash > ~/.local/share/bash-completion/completions/conv

# Zsh
conv completions zsh > ~/.zsh/completions/_conv

# Fish
conv completions fish > ~/.config/fish/completions/conv.fish
```

---

## Configuration (`~/.config/conv/config.toml`)

You can optionally define default options in `~/.config/conv/config.toml`:

```toml
# Default parallel worker threads (default: min(4, CPU cores))
jobs = 4

# Overwrite existing files by default
force = false

# Continue on errors during batch processing
keep_going = false

# Default output directory
output_dir = "/home/walid/Downloads/Converted"

# Custom binary paths if installed in non-standard directories
[backend_paths]
ffmpeg = "/usr/bin/ffmpeg"
magick = "/usr/bin/magick"
```

---

## License

This project is licensed under the [MIT License](LICENSE).

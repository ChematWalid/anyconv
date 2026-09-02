//! Universal Smart Converter CLI entrypoint and argument parser.

use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::Colorize;
use conv::backends::ConversionTask;
use conv::config::Config;
use conv::dispatch::{self, Backend, Format};
use glob::glob;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// CLI specification for conv.
#[derive(Parser, Debug)]
#[command(
    name = "conv",
    author = "ACEECA1",
    version = env!("CARGO_PKG_VERSION"),
    about = "Universal Smart Converter CLI — automatically routes media, documents, ebooks, images, and data",
    long_about = "Universal Smart Converter CLI (conv / anyconv)\n\n\
    Intelligently converts files across audio, video, images, e-books, documents, and data formats\n\
    by automatically selecting the fastest and best-suited backend engine (FFmpeg, ImageMagick,\n\
    Pandoc, Calibre, LibreOffice, pdftoppm, or Rust native memory engine)."
)]
struct Cli {
    #[arg(
        help = "Input files and/or target extension/output filename (e.g. 'in.mp4 out.mp3', 'in.mp4 mp3', '*.png webp')"
    )]
    files: Vec<String>,

    #[arg(
        short = 'o',
        long = "output-dir",
        value_name = "DIR",
        help = "Directory to place converted files in"
    )]
    output_dir: Option<PathBuf>,

    #[arg(
        short = 'j',
        long = "jobs",
        value_name = "N",
        help = "Maximum parallel conversion jobs (default: min(4, CPU cores))"
    )]
    jobs: Option<usize>,

    #[arg(
        short = 'd',
        long = "dry-run",
        help = "Show planned backend and command per file without executing"
    )]
    dry_run: bool,

    #[arg(
        short = 'f',
        long = "force",
        help = "Overwrite existing output files without prompting"
    )]
    force: bool,

    #[arg(
        short = 'k',
        long = "keep-going",
        help = "Continue processing remaining files when a batch item fails"
    )]
    keep_going: bool,

    #[arg(
        short = 'l',
        long = "list",
        help = "List all supported file formats, engines, and conversion matrix"
    )]
    list: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Generate shell completions for bash, zsh, fish, etc.")]
    Completions {
        #[arg(value_enum, help = "The target shell")]
        shell: Shell,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{} {}", "error:".bold().red(), err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load();

    if let Some(Commands::Completions { shell }) = cli.command {
        let mut cmd = Cli::command();
        generate(shell, &mut cmd, "conv", &mut io::stdout());
        return Ok(());
    }

    if cli.list {
        print_format_matrix();
        return Ok(());
    }

    if cli.files.is_empty() {
        Cli::command().print_help()?;
        println!();
        return Ok(());
    }

    let force = cli.force || config.force.unwrap_or(false);
    let keep_going = cli.keep_going || config.keep_going.unwrap_or(false);
    let output_dir = cli.output_dir.or(config.output_dir);
    let default_jobs = std::cmp::min(4, num_cpus::get());
    let jobs = cli.jobs.or(config.jobs).unwrap_or(default_jobs).max(1);

    let tasks = resolve_conversion_tasks(&cli.files, output_dir.as_deref(), force)?;

    if tasks.is_empty() {
        bail!("No valid conversion tasks could be resolved from the provided arguments.");
    }

    if cli.dry_run {
        println!("{}", "Dry run - Planned conversions:".bold().cyan());
        for (i, task) in tasks.iter().enumerate() {
            println!(
                "  [{}/{}] {} ({})",
                i + 1,
                tasks.len(),
                task.backend.display_name().bold().yellow(),
                task.command_string()
            );
        }
        return Ok(());
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()
        .context("Failed to initialize thread pool")?;

    let total = tasks.len();
    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));
    let should_stop = Arc::new(AtomicBool::new(false));

    println!(
        "{} {} task(s) using {} worker thread(s)...",
        "Converting".bold().green(),
        total,
        jobs
    );

    pool.install(|| {
        tasks.par_iter().enumerate().for_each(|(idx, task)| {
            if should_stop.load(Ordering::Relaxed) {
                return;
            }

            let task_prefix = format!("[{}/{}]", idx + 1, total);
            println!(
                "{} {} -> {} ({})",
                task_prefix.dimmed(),
                task.input.display().to_string().cyan(),
                task.output.display().to_string().green(),
                task.backend.display_name().yellow()
            );

            match task.execute() {
                Ok(()) => {
                    success_count.fetch_add(1, Ordering::Relaxed);
                    println!(
                        "{} {} Done: {}",
                        task_prefix.dimmed(),
                        "✓".bold().green(),
                        task.output.display()
                    );
                }
                Err(err) => {
                    failure_count.fetch_add(1, Ordering::Relaxed);
                    eprintln!(
                        "{} {} Failed to convert {}: {:#}",
                        task_prefix.dimmed(),
                        "✗".bold().red(),
                        task.input.display(),
                        err
                    );

                    if !keep_going {
                        should_stop.store(true, Ordering::Relaxed);
                    }
                }
            }
        });
    });

    let succeeded = success_count.load(Ordering::Relaxed);
    let failed = failure_count.load(Ordering::Relaxed);

    if failed > 0 {
        eprintln!(
            "\n{} {} succeeded, {} failed",
            "Finished with errors:".bold().red(),
            succeeded,
            failed
        );
        std::process::exit(1);
    } else {
        println!(
            "\n{} All {} conversion(s) completed successfully!",
            "Success:".bold().green(),
            succeeded
        );
    }

    Ok(())
}

/// Resolves user CLI arguments into concrete ConversionTasks.
fn resolve_conversion_tasks(
    raw_args: &[String],
    output_dir: Option<&Path>,
    force: bool,
) -> Result<Vec<ConversionTask>> {
    if raw_args.is_empty() {
        return Ok(Vec::new());
    }

    if raw_args.len() == 2 && !is_glob(&raw_args[0]) {
        let input_path = PathBuf::from(&raw_args[0]);
        if input_path.exists() {
            let target_arg = &raw_args[1];
            let task = resolve_single_task(&input_path, target_arg, output_dir, force)?;
            return Ok(vec![task]);
        }
    }

    let target_arg = raw_args.last().unwrap();
    let target_format = parse_target_format(target_arg)?;

    let mut input_paths = Vec::new();
    let mut seen_inputs = HashSet::new();

    for arg in &raw_args[..raw_args.len() - 1] {
        if is_glob(arg) {
            let entries = glob(arg).with_context(|| format!("Invalid glob pattern: {}", arg))?;
            for entry in entries {
                let path = entry.with_context(|| "Failed to read glob entry")?;
                if path.is_file() && seen_inputs.insert(path.clone()) {
                    input_paths.push(path);
                }
            }
        } else {
            let path = PathBuf::from(arg);
            if !path.exists() {
                bail!("Input file not found: {}", path.display());
            }
            if path.is_file() && seen_inputs.insert(path.clone()) {
                input_paths.push(path);
            }
        }
    }

    if input_paths.is_empty() {
        bail!("No input files matched for conversion.");
    }

    let mut tasks = Vec::new();
    for input in input_paths {
        let from_format = match Format::detect(&input) {
            Some(fmt) => fmt,
            None => {
                bail!(
                    "Could not detect format for input file '{}'. Make sure the file extension is valid.",
                    input.display()
                );
            }
        };

        let out_filename = format!(
            "{}.{}",
            input.file_stem().unwrap_or_default().to_string_lossy(),
            target_format.ext()
        );

        let output_path = match output_dir {
            Some(dir) => dir.join(&out_filename),
            None => input
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(&out_filename),
        };

        validate_paths(&input, &output_path, force)?;

        let backend = dispatch::dispatch(&from_format, &target_format)?;
        tasks.push(ConversionTask::new(
            input,
            output_path,
            from_format,
            target_format,
            backend,
            force,
        ));
    }

    Ok(tasks)
}

fn resolve_single_task(
    input: &Path,
    target_arg: &str,
    output_dir: Option<&Path>,
    force: bool,
) -> Result<ConversionTask> {
    if !input.exists() {
        bail!("Input file not found: {}", input.display());
    }

    let from_format = Format::detect(input).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not determine format of '{}'. Please verify extension or file header.",
            input.display()
        )
    })?;

    let (output_path, to_format) = if target_arg.contains('.') && !target_arg.starts_with('.') {
        let target_path = PathBuf::from(target_arg);
        let ext = target_path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| {
                anyhow::anyhow!("Output filename '{}' lacks an extension", target_arg)
            })?;
        let to_fmt = Format::from_ext(ext)
            .ok_or_else(|| anyhow::anyhow!("Unsupported output file extension '.{}'", ext))?;

        let resolved_output = match output_dir {
            Some(dir) => dir.join(target_path.file_name().unwrap_or_default()),
            None => target_path,
        };

        (resolved_output, to_fmt)
    } else {
        let to_fmt = parse_target_format(target_arg)?;
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        let filename = format!("{}.{}", stem, to_fmt.ext());
        let resolved_output = match output_dir {
            Some(dir) => dir.join(&filename),
            None => input
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(&filename),
        };
        (resolved_output, to_fmt)
    };

    validate_paths(input, &output_path, force)?;

    let backend = dispatch::dispatch(&from_format, &to_format)?;
    Ok(ConversionTask::new(
        input.to_path_buf(),
        output_path,
        from_format,
        to_format,
        backend,
        force,
    ))
}

fn parse_target_format(arg: &str) -> Result<Format> {
    let clean = arg.trim_start_matches('.');
    Format::from_ext(clean).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown or unsupported target format '{}'. Run 'conv --list' to view supported formats.",
            arg
        )
    })
}

fn is_glob(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

fn validate_paths(input: &Path, output: &Path, force: bool) -> Result<()> {
    if let (Ok(in_canon), Ok(out_canon)) = (fs::canonicalize(input), fs::canonicalize(output)) {
        if in_canon == out_canon {
            if !force {
                bail!(
                    "Input path is identical to output path: {}\nRefusing to overwrite source file unless --force is specified.",
                    input.display()
                );
            } else {
                eprintln!(
                    "{} Input and output paths are identical: {}",
                    "warning:".bold().yellow(),
                    input.display()
                );
            }
        }
    } else if input == output && !force {
        bail!(
            "Input path is identical to output path: {}\nRefusing to overwrite source file unless --force is specified.",
            input.display()
        );
    }

    if output.exists() && !force {
        bail!(
            "Output file '{}' already exists. Use --force (-f) to overwrite.",
            output.display()
        );
    }

    Ok(())
}

fn print_format_matrix() {
    println!(
        "{}",
        "========================================================================================="
            .bold()
            .cyan()
    );
    println!("{}", "                            conv — Supported Format Matrix                                ".bold());
    println!(
        "{}",
        "========================================================================================="
            .bold()
            .cyan()
    );
    println!(
        "{:<15} | {:<35} | {:<25} | {:<10}",
        "Category".bold(),
        "Supported Formats".bold(),
        "Backend Engine".bold(),
        "Status".bold()
    );
    println!("----------------+-------------------------------------+---------------------------+-----------");

    let categories = [
        (
            "Audio",
            "mp3, wav, flac, aac, m4a, ogg, opus, wma",
            Backend::Ffmpeg,
        ),
        (
            "Video",
            "mp4, mkv, avi, mov, webm, flv, wmv, gif",
            Backend::Ffmpeg,
        ),
        (
            "Images",
            "png, jpg, jpeg, webp, svg, ico, bmp, tiff, avif",
            Backend::Magick,
        ),
        (
            "E-Books",
            "epub, mobi, azw3, azw, fb2, pdf, txt, rtf, htmlz",
            Backend::EbookConvert,
        ),
        (
            "Documents",
            "md, html, docx, tex, rst, epub, pdf",
            Backend::Pandoc,
        ),
        (
            "Office -> PDF",
            "docx, pptx, xlsx, odt, ods, odp -> pdf",
            Backend::LibreOffice,
        ),
        ("PDF -> Raster", "pdf -> png, jpg, tiff", Backend::Pdftoppm),
        (
            "Data Tables",
            "csv <-> json, tsv <-> json, csv <-> tsv",
            Backend::BuiltinData,
        ),
    ];

    for (cat, formats, backend) in categories {
        let status = if let Some(bin) = backend.binary_name() {
            if which::which(bin).is_ok() || (bin == "magick" && which::which("convert").is_ok()) {
                "Available".bold().green()
            } else {
                "Missing".bold().red()
            }
        } else {
            "Built-in".bold().blue()
        };

        println!(
            "{:<15} | {:<35} | {:<25} | {:<10}",
            cat,
            formats,
            backend.display_name(),
            status
        );
    }
    println!(
        "{}",
        "========================================================================================="
            .bold()
            .cyan()
    );
    println!("\n{}", "Common Usage Examples:".bold());
    println!("  conv video.mp4 audio.mp3          # Extract audio from video");
    println!("  conv song.wav mp3                 # Shorthand: converts to song.mp3");
    println!("  conv *.png webp                   # Batch convert all PNGs to WebP");
    println!("  conv doc.md doc.html              # Render markdown to HTML");
    println!("  conv book.epub mobi               # Convert EPUB to Kindle MOBI");
    println!("  conv table.csv json               # Convert CSV to JSON table");
    println!("  conv presentation.pptx pdf        # Convert PPTX slides to PDF");
    println!("  conv doc.pdf png                  # Extract PDF pages as PNG images");
    println!("  conv -j 8 *.jpg webp              # Convert in parallel using 8 threads");
    println!("  conv --dry-run *.mp4 gif          # Preview planned conversion commands\n");
}

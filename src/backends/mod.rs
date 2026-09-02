//! Conversion backends and task execution orchestration.

pub mod data;
pub mod document;
pub mod ebook;
pub mod ffmpeg;
pub mod image;

use crate::dispatch::{Backend, Format};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents a concrete file conversion task to be executed.
#[derive(Debug, Clone)]
pub struct ConversionTask {
    pub input: PathBuf,
    pub output: PathBuf,
    pub from: Format,
    pub to: Format,
    pub backend: Backend,
    pub force: bool,
}

impl ConversionTask {
    /// Creates a new conversion task instance.
    pub fn new(
        input: PathBuf,
        output: PathBuf,
        from: Format,
        to: Format,
        backend: Backend,
        force: bool,
    ) -> Self {
        Self {
            input,
            output,
            from,
            to,
            backend,
            force,
        }
    }

    /// Validates that the required external binary is available on PATH.
    pub fn check_prerequisites(&self) -> Result<()> {
        if let Some(bin) = self.backend.binary_name() {
            if which::which(bin).is_err() {
                if bin == "magick" && which::which("convert").is_ok() {
                    return Ok(());
                }
                bail!(
                    "Backend '{}' is required for {} -> {} conversion but was not found on PATH.\nTo install: {}",
                    bin,
                    self.from,
                    self.to,
                    self.backend.install_hint()
                );
            }
        }
        Ok(())
    }

    /// Formats the command string for display in dry-run mode or logs.
    pub fn command_string(&self) -> String {
        match self.backend {
            Backend::BuiltinData => format!(
                "[Built-in Engine] {} -> {}",
                self.input.display(),
                self.output.display()
            ),
            Backend::Ffmpeg => {
                let cmd = ffmpeg::build_command(
                    &self.input,
                    &self.output,
                    &self.from,
                    &self.to,
                    self.force,
                );
                format_command(&cmd)
            }
            Backend::EbookConvert => {
                let cmd = ebook::build_command(&self.input, &self.output);
                format_command(&cmd)
            }
            Backend::Pandoc => {
                let cmd = document::build_pandoc_command(&self.input, &self.output);
                format_command(&cmd)
            }
            Backend::LibreOffice => {
                let outdir = self.output.parent().unwrap_or_else(|| Path::new("."));
                let cmd = document::build_libreoffice_command(&self.input, outdir);
                format_command(&cmd)
            }
            Backend::Pdftoppm => {
                let stem = self.output.file_stem().unwrap_or_default();
                let parent = self.output.parent().unwrap_or_else(|| Path::new("."));
                let prefix = parent.join(stem);
                let cmd = document::build_pdftoppm_command(&self.input, &prefix, &self.to);
                format_command(&cmd)
            }
            Backend::Magick => {
                let cmd = image::build_command(&self.input, &self.output, &self.from, &self.to);
                format_command(&cmd)
            }
        }
    }

    /// Executes the conversion task.
    pub fn execute(&self) -> Result<()> {
        self.check_prerequisites()?;

        if let Some(parent) = self.output.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "Failed to create destination directory: {}",
                        parent.display()
                    )
                })?;
            }
        }

        match self.backend {
            Backend::BuiltinData => {
                data::convert_data(&self.input, &self.output, &self.from, &self.to)
            }
            Backend::Ffmpeg => {
                let mut cmd = ffmpeg::build_command(
                    &self.input,
                    &self.output,
                    &self.from,
                    &self.to,
                    self.force,
                );
                run_command(&mut cmd)
            }
            Backend::EbookConvert => {
                let mut cmd = ebook::build_command(&self.input, &self.output);
                run_command(&mut cmd)
            }
            Backend::Pandoc => {
                let mut cmd = document::build_pandoc_command(&self.input, &self.output);
                run_command(&mut cmd)
            }
            Backend::LibreOffice => document::execute_libreoffice(&self.input, &self.output),
            Backend::Pdftoppm => {
                let stem = self.output.file_stem().unwrap_or_default();
                let parent = self.output.parent().unwrap_or_else(|| Path::new("."));
                let prefix = parent.join(stem);
                let mut cmd = document::build_pdftoppm_command(&self.input, &prefix, &self.to);
                run_command(&mut cmd)
            }
            Backend::Magick => {
                let mut cmd = image::build_command(&self.input, &self.output, &self.from, &self.to);
                let output = cmd
                    .output()
                    .with_context(|| "Failed to spawn ImageMagick process")?;
                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr);
                    if self.from == Format::Svg
                        && (err.contains("delegate") || err.contains("librsvg"))
                    {
                        bail!(
                            "ImageMagick SVG conversion failed: missing SVG delegate.\nInstall librsvg via: pacman -S librsvg (or apt install librsvg2-bin / brew install librsvg)"
                        );
                    }
                    bail!(
                        "ImageMagick conversion failed (exit code {}):\n{}",
                        output.status,
                        err.trim()
                    );
                }
                Ok(())
            }
        }
    }
}

fn format_command(cmd: &Command) -> String {
    let prog = cmd.get_program().to_string_lossy();
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| {
            let s = a.to_string_lossy();
            if s.contains(' ') {
                format!("\"{}\"", s)
            } else {
                s.into_owned()
            }
        })
        .collect();
    format!("{} {}", prog, args.join(" "))
}

fn run_command(cmd: &mut Command) -> Result<()> {
    let output = cmd.output().with_context(|| {
        format!(
            "Failed to execute command: {}",
            cmd.get_program().to_string_lossy()
        )
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = if !stderr.is_empty() { stderr } else { stdout };
        bail!(
            "Command '{}' failed with exit status {}:\n{}",
            cmd.get_program().to_string_lossy(),
            output.status,
            msg.trim()
        );
    }
    Ok(())
}

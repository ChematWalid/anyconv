//! Document and markup backend builders (Pandoc, LibreOffice, pdftoppm).

use crate::dispatch::Format;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Builds a Pandoc execution command.
pub fn build_pandoc_command(input: &Path, output: &Path) -> Command {
    let mut cmd = Command::new("pandoc");
    cmd.arg("-s").arg(input).arg("-o").arg(output);
    cmd
}

/// Builds a headless LibreOffice PDF conversion command.
pub fn build_libreoffice_command(input: &Path, outdir: &Path) -> Command {
    let mut cmd = Command::new("soffice");
    cmd.arg("--headless")
        .arg("--convert-to")
        .arg("pdf")
        .arg(input)
        .arg("--outdir")
        .arg(outdir);
    cmd
}

/// Builds a pdftoppm rasterization command.
pub fn build_pdftoppm_command(input: &Path, prefix: &Path, format: &Format) -> Command {
    let mut cmd = Command::new("pdftoppm");
    match format {
        Format::Png => {
            cmd.arg("-png");
        }
        Format::Jpg | Format::Jpeg => {
            cmd.arg("-jpeg");
        }
        Format::Tiff => {
            cmd.arg("-tiff");
        }
        _ => {
            cmd.arg("-png");
        }
    }
    cmd.arg("-r").arg("150").arg(input).arg(prefix);
    cmd
}

/// Executes LibreOffice in headless mode and moves the generated output file to the target location.
pub fn execute_libreoffice(input: &Path, output: &Path) -> Result<()> {
    let outdir = output.parent().unwrap_or_else(|| Path::new("."));
    let mut cmd = build_libreoffice_command(input, outdir);
    let status = cmd
        .status()
        .with_context(|| "Failed to execute libreoffice (soffice)")?;
    if !status.success() {
        bail!("LibreOffice conversion failed with status: {}", status);
    }

    let stem = input.file_stem().unwrap_or_default();
    let produced = outdir.join(format!("{}.pdf", stem.to_string_lossy()));
    if produced.exists() && produced != output {
        fs::rename(&produced, output).with_context(|| {
            format!(
                "Failed to move {} to {}",
                produced.display(),
                output.display()
            )
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_pandoc_command() {
        let cmd = build_pandoc_command(&PathBuf::from("doc.md"), &PathBuf::from("doc.html"));
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["-s", "doc.md", "-o", "doc.html"]);
    }

    #[test]
    fn test_pdftoppm_command() {
        let cmd = build_pdftoppm_command(
            &PathBuf::from("doc.pdf"),
            &PathBuf::from("page"),
            &Format::Png,
        );
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["-png", "-r", "150", "doc.pdf", "page"]);
    }
}

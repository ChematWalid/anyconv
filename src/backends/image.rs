//! ImageMagick image conversion backend builder.

use crate::dispatch::Format;
use std::path::Path;
use std::process::Command;

/// Builds an ImageMagick execution command.
pub fn build_command(input: &Path, output: &Path, from: &Format, to: &Format) -> Command {
    let bin = if which::which("magick").is_ok() {
        "magick"
    } else {
        "convert"
    };

    let mut cmd = Command::new(bin);
    cmd.arg(input);

    if *to == Format::Ico {
        cmd.arg("-define")
            .arg("icon:auto-resize=256,128,64,48,32,16");
    }

    if *to == Format::Webp && *from != Format::Webp {
        cmd.arg("-quality").arg("90");
    }

    cmd.arg(output);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_image_command() {
        let cmd = build_command(
            &PathBuf::from("in.png"),
            &PathBuf::from("out.webp"),
            &Format::Png,
            &Format::Webp,
        );
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(args.contains(&"in.png".to_string()));
        assert!(args.contains(&"out.webp".to_string()));
    }

    #[test]
    fn test_ico_command() {
        let cmd = build_command(
            &PathBuf::from("icon.png"),
            &PathBuf::from("icon.ico"),
            &Format::Png,
            &Format::Ico,
        );
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(args.contains(&"-define".to_string()));
        assert!(args.contains(&"icon:auto-resize=256,128,64,48,32,16".to_string()));
    }
}

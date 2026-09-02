//! FFmpeg audio and video backend builder.

use crate::dispatch::Format;
use std::path::Path;
use std::process::Command;

/// Builds an FFmpeg execution command configured for the requested audio/video conversion.
pub fn build_command(
    input: &Path,
    output: &Path,
    from: &Format,
    to: &Format,
    force: bool,
) -> Command {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-hide_banner").arg("-loglevel").arg("error");

    if force {
        cmd.arg("-y");
    } else {
        cmd.arg("-n");
    }

    cmd.arg("-i").arg(input);

    if *to == Format::Gif && from.category() == crate::dispatch::Category::Video {
        cmd.arg("-vf").arg("fps=15,scale=480:-1:flags=lanczos");
    } else if from.category() == crate::dispatch::Category::Video
        && to.category() == crate::dispatch::Category::Audio
    {
        cmd.arg("-vn");
    }

    cmd.arg(output);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_ffmpeg_command_args() {
        let cmd = build_command(
            &PathBuf::from("in.mp4"),
            &PathBuf::from("out.mp3"),
            &Format::Mp4,
            &Format::Mp3,
            true,
        );
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();

        assert!(args.contains(&"-y".to_string()));
        assert!(args.contains(&"-vn".to_string()));
        assert_eq!(args.last().unwrap(), "out.mp3");
    }

    #[test]
    fn test_ffmpeg_gif_args() {
        let cmd = build_command(
            &PathBuf::from("in.mp4"),
            &PathBuf::from("out.gif"),
            &Format::Mp4,
            &Format::Gif,
            true,
        );
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();

        assert!(args.contains(&"-vf".to_string()));
    }
}

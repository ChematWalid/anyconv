//! Calibre ebook-convert backend builder.

use std::path::Path;
use std::process::Command;

/// Builds an ebook-convert execution command.
pub fn build_command(input: &Path, output: &Path) -> Command {
    let mut cmd = Command::new("ebook-convert");
    cmd.arg(input).arg(output);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_ebook_command() {
        let cmd = build_command(&PathBuf::from("book.epub"), &PathBuf::from("book.mobi"));
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["book.epub", "book.mobi"]);
    }
}

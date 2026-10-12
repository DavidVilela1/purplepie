//! The best score, kept as one number in a small text file next to the game's
//! executable (`purple-swarm-best.txt`). A missing or unreadable file counts
//! as 0, and a failed write only prints a warning: losing the best score must
//! never stop the game.

use std::path::{Path, PathBuf};
use std::{fs, io};

/// File name, next to the executable.
const FILE_NAME: &str = "purple-swarm-best.txt";

/// Where the best score lives: beside the running executable, so a shipped
/// game keeps it in its own folder (and `cargo run` keeps it under `target/`,
/// out of the repository). `None` if the executable's folder is unknown.
pub fn path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join(FILE_NAME))
}

/// Reads the best score: 0 if the file is missing, and 0 with a warning if it
/// does not hold a number.
pub fn read(path: &Path) -> u32 {
    match fs::read_to_string(path) {
        Ok(text) => text.trim().parse().unwrap_or_else(|_| {
            eprintln!(
                "swarm: ignoring {} (not a score: {:?})",
                path.display(),
                text.trim()
            );
            0
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => 0,
        Err(error) => {
            eprintln!("swarm: could not read {}: {error}", path.display());
            0
        }
    }
}

/// Writes `score` to a temporary file and renames it over `path`, so a crash
/// mid-write cannot leave a half-written file behind.
pub fn write(path: &Path, score: u32) -> io::Result<()> {
    let temporary = path.with_extension("txt.tmp");
    fs::write(&temporary, format!("{score}\n"))?;
    fs::rename(&temporary, path).inspect_err(|_| {
        let _ = fs::remove_file(&temporary);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, empty folder for one test.
    fn folder(test: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("purple-swarm-best-{}-{test}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("test folder");
        dir
    }

    #[test]
    fn a_missing_file_is_zero() {
        let dir = folder("missing");
        assert_eq!(read(&dir.join(FILE_NAME)), 0);
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn a_written_score_reads_back_and_leaves_no_temporary_file() {
        let dir = folder("roundtrip");
        let file = dir.join(FILE_NAME);
        write(&file, 415).expect("write");
        assert_eq!(read(&file), 415);
        write(&file, 1200).expect("overwrite");
        assert_eq!(read(&file), 1200);
        assert_eq!(fs::read_to_string(&file).expect("text"), "1200\n");
        let names: Vec<_> = fs::read_dir(&dir)
            .expect("list")
            .map(|e| e.expect("entry").file_name())
            .collect();
        assert_eq!(names, [FILE_NAME], "only the score file is left");
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn a_broken_file_is_zero_and_hand_edits_with_spaces_still_read() {
        let dir = folder("broken");
        let file = dir.join(FILE_NAME);
        for broken in ["", "lots", "-5", "99999999999", "12 points"] {
            fs::write(&file, broken).expect("write");
            assert_eq!(read(&file), 0, "{broken:?}");
        }
        fs::write(&file, "  250 \r\n").expect("write");
        assert_eq!(read(&file), 250, "Windows line ending and spaces");
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn a_failed_write_is_an_error_not_a_panic() {
        let dir = folder("unwritable");
        // A folder where the file should be: the rename cannot replace it.
        let file = dir.join(FILE_NAME);
        fs::create_dir(&file).expect("blocking folder");
        assert!(write(&file, 10).is_err());
        assert_eq!(read(&file), 0, "reading a folder is 0, not a panic");
        fs::remove_dir_all(dir).expect("cleanup");
    }
}

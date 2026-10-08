//! File change detection for asset hot reload (ADR-037), shared by the
//! texture, font and sound stores.
//!
//! A file is identified as changed when its modification time or length
//! differs from the values seen when it was last read. A file that is broken
//! at that moment is remembered too, so it is reported once and read again
//! only after its next change.

use std::path::Path;
use std::time::SystemTime;

use crate::error::BoxError;

/// Modification time and length (the length catches two writes within the
/// file system's time resolution).
type FileStamp = (SystemTime, u64);

fn file_stamp(path: &Path) -> Option<FileStamp> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// Hot-reload bookkeeping for one loaded file.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct FileWatch {
    /// The file as last read successfully.
    loaded: Option<FileStamp>,
    /// The file as last seen broken.
    failed: Option<FileStamp>,
}

impl FileWatch {
    /// Stamps `path` as it is now. Call it *before* reading the file, so a
    /// write that lands during the read is seen as a change on the next check.
    pub(crate) fn before_read(path: &Path) -> Self {
        Self {
            loaded: file_stamp(path),
            failed: None,
        }
    }
}

/// What one reload pass over a store did.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ReloadReport<Id> {
    /// Assets whose file changed and that now hold the new contents.
    pub(crate) reloaded: Vec<Id>,
    /// Assets whose file changed but could not be read or decoded; they keep
    /// their previous contents.
    pub(crate) failed: Vec<Id>,
}

impl<Id> Default for ReloadReport<Id> {
    fn default() -> Self {
        Self {
            reloaded: Vec::new(),
            failed: Vec::new(),
        }
    }
}

/// Reloads one asset if its file changed: `reload` reads the file and
/// replaces the asset's contents, returning a short description for the log
/// (or an error, which leaves the asset untouched). Missing files are ignored
/// (an editor may be replacing them); `kind` names the asset kind in log lines.
pub(crate) fn reload_if_changed<Id: Copy + std::fmt::Debug>(
    path: &Path,
    watch: &mut FileWatch,
    id: Id,
    kind: &str,
    report: &mut ReloadReport<Id>,
    reload: impl FnOnce() -> Result<String, BoxError>,
) {
    let Some(stamp) = file_stamp(path) else {
        return;
    };
    if Some(stamp) == watch.loaded || Some(stamp) == watch.failed {
        return;
    }
    match reload() {
        Ok(detail) => {
            *watch = FileWatch {
                loaded: Some(stamp),
                failed: None,
            };
            log::info!("reloaded {kind} {} ({detail}) as {id:?}", path.display());
            report.reloaded.push(id);
        }
        Err(error) => {
            watch.failed = Some(stamp);
            log::warn!(
                "{kind} {} changed but could not be reloaded (keeping the old one): {error}",
                path.display()
            );
            report.failed.push(id);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Writes `bytes` to `path` and moves its modification time `seconds`
    /// into the future, so the change is seen even on coarse file systems.
    pub(crate) fn write_changed(path: &Path, bytes: &[u8], seconds: u64) {
        std::fs::write(path, bytes).expect("write");
        let file = std::fs::File::options()
            .write(true)
            .open(path)
            .expect("open");
        let time = SystemTime::now() + std::time::Duration::from_secs(seconds);
        file.set_modified(time).expect("set mtime");
    }

    #[test]
    fn changes_are_seen_once_and_broken_files_are_reported_once() {
        let path = std::env::temp_dir().join(format!("purplepie-{}-watch.txt", std::process::id()));
        std::fs::write(&path, b"one").expect("write");
        let mut watch = FileWatch::before_read(&path);
        let mut report = ReloadReport::default();
        let mut reads = 0;
        let mut pass = |watch: &mut FileWatch, ok: bool, report: &mut ReloadReport<u8>| {
            reload_if_changed(&path, watch, 7_u8, "file", report, || {
                reads += 1;
                if ok {
                    Ok("fine".into())
                } else {
                    Err("broken".into())
                }
            });
        };
        pass(&mut watch, true, &mut report);
        assert_eq!(report, ReloadReport::default(), "unchanged: not read");
        write_changed(&path, b"two!", 10);
        pass(&mut watch, true, &mut report);
        pass(&mut watch, true, &mut report);
        assert_eq!(report.reloaded, [7], "read once per change");
        write_changed(&path, b"three", 20);
        pass(&mut watch, false, &mut report);
        pass(&mut watch, false, &mut report);
        assert_eq!(report.failed, [7], "a broken file is reported once");
        write_changed(&path, b"fixed!", 30);
        pass(&mut watch, true, &mut report);
        assert_eq!(report.reloaded, [7, 7]);
        std::fs::remove_file(&path).expect("remove");
        pass(&mut watch, true, &mut report);
        assert_eq!(
            (report.reloaded.len(), report.failed.len()),
            (2, 1),
            "missing: ignored"
        );
        assert_eq!(reads, 3);
    }
}

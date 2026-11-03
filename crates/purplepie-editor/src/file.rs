//! Scene paths typed into Open and Save As (PP-044a2, ADR-046): relative to
//! the project folder (the folder holding the asset folder), and inside the
//! asset folder, since a scene's texture and font paths are relative to it.

use std::path::{Component, Path, PathBuf};

/// The project folder: the one holding `assets`, or `fallback` (the working
/// folder) without an asset folder.
pub(crate) fn project_folder(assets: Option<&Path>, fallback: &Path) -> PathBuf {
    assets
        .and_then(Path::parent)
        .map_or_else(|| fallback.to_path_buf(), Path::to_path_buf)
}

/// The scene file for `typed`: a relative path is taken from `project`; `..`
/// and `.` are resolved; `.ron` is added when there is no extension. With an
/// asset folder, the file must be inside it. `Err` says why not, in words.
pub(crate) fn resolve(
    typed: &str,
    project: &Path,
    assets: Option<&Path>,
) -> Result<PathBuf, String> {
    let typed = typed.trim();
    if typed.is_empty() {
        return Err("type the scene file's path".to_owned());
    }
    let mut path = normalize(&project.join(typed));
    if path.file_name().is_none() || typed.ends_with('/') || typed.ends_with('\\') {
        return Err(format!("{typed} is a folder; add the file name"));
    }
    if path.extension().is_none() {
        path.set_extension("ron");
    }
    if let Some(assets) = assets {
        let assets = normalize(assets);
        if !path.starts_with(&assets) {
            return Err(format!(
                "{} is outside the asset folder ({}): a scene's image and font paths are relative to it",
                shown(&path, project),
                shown(&assets, project)
            ));
        }
    }
    Ok(path)
}

/// `path` as typed in the fields: relative to `project` when inside it, with
/// `/` between folders.
pub(crate) fn shown(path: &Path, project: &Path) -> String {
    match path.strip_prefix(project) {
        Ok(relative) => relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/"),
        Err(_) => path.display().to_string(),
    }
}

/// `path` with `.` dropped and `..` taken back, without touching the disk
/// (the file may not exist yet).
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> PathBuf {
        PathBuf::from("/games/heist")
    }

    fn assets() -> PathBuf {
        project().join("assets")
    }

    #[test]
    fn paths_are_taken_from_the_project_and_must_stay_in_the_asset_folder() {
        let resolve = |typed| resolve(typed, &project(), Some(&assets()));
        assert_eq!(
            resolve("assets/levels/level2.ron"),
            Ok(PathBuf::from("/games/heist/assets/levels/level2.ron"))
        );
        assert_eq!(
            resolve(" assets/levels/new "),
            Ok(PathBuf::from("/games/heist/assets/levels/new.ron")),
            "trimmed, .ron added"
        );
        assert_eq!(
            resolve("assets/levels/../scenes/./a.ron"),
            Ok(PathBuf::from("/games/heist/assets/scenes/a.ron"))
        );
        assert_eq!(
            resolve("/games/heist/assets/b.ron"),
            Ok(PathBuf::from("/games/heist/assets/b.ron")),
            "absolute paths are kept"
        );
        for outside in [
            "level.ron",
            "assets/../level.ron",
            "/tmp/level.ron",
            "../other/assets/x.ron",
        ] {
            let error = resolve(outside).expect_err(outside);
            assert!(
                error.contains("outside the asset folder"),
                "{outside}: {error}"
            );
        }
        assert!(resolve("").is_err());
        assert!(resolve("   ").is_err());
        assert!(
            resolve("assets/levels/")
                .expect_err("folder")
                .contains("folder")
        );
        // Without an asset folder any path is accepted.
        assert_eq!(
            super::resolve("x/level.ron", &project(), None),
            Ok(PathBuf::from("/games/heist/x/level.ron"))
        );
    }

    #[test]
    fn the_project_is_the_asset_folders_parent_and_paths_show_relative_to_it() {
        assert_eq!(
            project_folder(Some(&assets()), Path::new("/cwd")),
            project()
        );
        assert_eq!(
            project_folder(None, Path::new("/cwd")),
            PathBuf::from("/cwd")
        );
        assert_eq!(
            shown(
                Path::new("/games/heist/assets/levels/level1.ron"),
                &project()
            ),
            "assets/levels/level1.ron"
        );
        assert_eq!(shown(Path::new("/tmp/x.ron"), &project()), "/tmp/x.ron");
    }
}

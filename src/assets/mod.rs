//! Where game files are loaded from (ADR-025).
//!
//! Relative asset paths (e.g. `"textures/player.png"`) are resolved against one
//! **asset root**, chosen once when the engine starts:
//!
//! 1. [`EngineConfig::with_asset_root`](crate::EngineConfig::with_asset_root),
//!    if the game set one (a relative root is taken relative to the working
//!    directory at startup);
//! 2. otherwise the `assets` folder **next to the executable**, if it exists
//!    (how a shipped game is laid out);
//! 3. otherwise the `assets` folder in the **working directory**, if it exists
//!    (how `cargo run` from the project folder finds it);
//! 4. otherwise there is no root, and loading a relative path fails with an
//!    [`Error::Asset`] that lists where PurplePie looked.
//!
//! Absolute paths are used as they are.

pub(crate) mod watch;

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// The folder name looked for next to the executable and in the working directory.
pub(crate) const DEFAULT_FOLDER: &str = "assets";

/// The resolved asset root, or the places that were searched without success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AssetRoot {
    Found(PathBuf),
    NotFound { searched: Vec<PathBuf> },
}

impl AssetRoot {
    /// Chooses the root (see the module docs). `exe_dir` and `cwd` are passed in
    /// so the rules can be tested without touching the real process state.
    pub(crate) fn resolve(
        explicit: Option<&Path>,
        exe_dir: Option<&Path>,
        cwd: Option<&Path>,
    ) -> Self {
        if let Some(root) = explicit {
            let root = match cwd {
                Some(cwd) if root.is_relative() => cwd.join(root),
                _ => root.to_path_buf(),
            };
            return Self::Found(root);
        }
        let candidates: Vec<PathBuf> = [exe_dir, cwd]
            .into_iter()
            .flatten()
            .map(|dir| dir.join(DEFAULT_FOLDER))
            .collect();
        match candidates.iter().find(|dir| dir.is_dir()) {
            Some(found) => Self::Found(found.clone()),
            None => Self::NotFound {
                searched: candidates,
            },
        }
    }

    /// [`resolve`](Self::resolve) with the running process's executable folder
    /// and working directory.
    pub(crate) fn for_process(explicit: Option<&Path>) -> Self {
        let exe = std::env::current_exe().ok();
        let exe_dir = exe.as_deref().and_then(Path::parent);
        let cwd = std::env::current_dir().ok();
        Self::resolve(explicit, exe_dir, cwd.as_deref())
    }

    /// The root folder, if one was found or configured.
    pub(crate) fn path(&self) -> Option<&Path> {
        match self {
            Self::Found(root) => Some(root),
            Self::NotFound { .. } => None,
        }
    }

    /// The full path of the asset `path`: absolute paths unchanged, relative
    /// ones joined to the root. Fails with `Error::Asset` if a relative path is
    /// requested and no root exists.
    pub(crate) fn locate(&self, path: &Path) -> Result<PathBuf> {
        if path.is_absolute() {
            return Ok(path.to_path_buf());
        }
        match self {
            Self::Found(root) => Ok(root.join(path)),
            Self::NotFound { searched } => {
                let places: Vec<String> =
                    searched.iter().map(|p| p.display().to_string()).collect();
                Err(Error::Asset {
                    path: path.to_path_buf(),
                    source: format!(
                        "no asset folder found (looked for {}); create one or set \
                         EngineConfig::with_asset_root",
                        places.join(" and ")
                    )
                    .into(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh scratch directory for one test.
    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("purplepie-assets-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    #[test]
    fn the_folder_next_to_the_executable_wins_over_the_working_directory() {
        let base = scratch("order");
        let (exe_dir, cwd) = (base.join("bin"), base.join("work"));
        for dir in [&exe_dir, &cwd] {
            std::fs::create_dir_all(dir.join(DEFAULT_FOLDER)).expect("mkdir");
        }
        let root = AssetRoot::resolve(None, Some(&exe_dir), Some(&cwd));
        assert_eq!(root, AssetRoot::Found(exe_dir.join("assets")));

        std::fs::remove_dir_all(exe_dir.join(DEFAULT_FOLDER)).expect("rmdir");
        let root = AssetRoot::resolve(None, Some(&exe_dir), Some(&cwd));
        assert_eq!(
            root,
            AssetRoot::Found(cwd.join("assets")),
            "falls back to the working directory"
        );
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn a_file_named_assets_is_not_a_folder() {
        let base = scratch("file");
        std::fs::write(base.join(DEFAULT_FOLDER), b"not a folder").expect("write");
        let root = AssetRoot::resolve(None, Some(&base), None);
        assert_eq!(
            root,
            AssetRoot::NotFound {
                searched: vec![base.join("assets")]
            }
        );
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn an_explicit_root_is_used_even_if_missing_and_made_absolute() {
        let cwd = Path::new("/game");
        assert_eq!(
            AssetRoot::resolve(Some(Path::new("data")), None, Some(cwd)),
            AssetRoot::Found(PathBuf::from("/game/data"))
        );
        let absolute = std::env::temp_dir().join("purplepie-no-such-root");
        assert_eq!(
            AssetRoot::resolve(Some(&absolute), None, Some(cwd)),
            AssetRoot::Found(absolute.clone())
        );
    }

    #[test]
    fn relative_paths_join_the_root_and_absolute_paths_are_kept() {
        let root = AssetRoot::Found(PathBuf::from("/game/assets"));
        assert_eq!(
            root.locate(Path::new("textures/a.png")).expect("relative"),
            PathBuf::from("/game/assets/textures/a.png")
        );
        let absolute = std::env::temp_dir().join("x.png");
        assert_eq!(root.locate(&absolute).expect("absolute"), absolute);
    }

    #[test]
    fn without_a_root_relative_paths_fail_and_name_the_searched_folders() {
        let root = AssetRoot::NotFound {
            searched: vec![PathBuf::from("/bin/assets"), PathBuf::from("/work/assets")],
        };
        let err = root
            .locate(Path::new("textures/a.png"))
            .expect_err("no root");
        assert!(matches!(&err, Error::Asset { path, .. } if path == Path::new("textures/a.png")));
        let source = std::error::Error::source(&err)
            .map(ToString::to_string)
            .unwrap_or_default();
        assert!(
            source.contains("/bin/assets") && source.contains("/work/assets"),
            "{source}"
        );
        assert!(source.contains("with_asset_root"), "{source}");
        // Absolute paths still work without a root.
        let absolute = std::env::temp_dir().join("x.png");
        assert_eq!(root.locate(&absolute).expect("absolute"), absolute);
        assert_eq!(root.path(), None);
    }
}

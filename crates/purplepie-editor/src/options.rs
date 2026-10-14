//! Command-line options and the asset folder of a scene.

use std::path::{Path, PathBuf};

/// How to run the editor, usually from [`Options::from_args`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The scene file to open and save.
    pub scene: PathBuf,
    /// The asset folder the scene's texture and font paths are relative to;
    /// `None` means the nearest folder named `assets` that contains the scene
    /// ([`asset_folder_of`]).
    pub assets: Option<PathBuf>,
}

/// The command line, for `--help` and errors.
pub const USAGE: &str = "\
usage: purplepie-editor [--assets <folder>] <scene.ron>

Opens a PurplePie scene file to view and save it.
  --assets <folder>  the folder the scene's asset paths are relative to
                     (default: the nearest folder named `assets` holding the scene)";

impl Options {
    /// Reads the arguments after the program name. `Err` holds a message to
    /// print with [`USAGE`]; `Err(None)` means `--help` was asked for.
    pub fn from_args<I>(args: I) -> Result<Self, Option<String>>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let mut scene = None;
        let mut assets = None;
        let mut args = args.into_iter().map(Into::into);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => return Err(None),
                "--assets" => match args.next() {
                    Some(folder) => assets = Some(PathBuf::from(folder)),
                    None => return Err(Some("--assets needs a folder".to_owned())),
                },
                flag if flag.starts_with('-') => {
                    return Err(Some(format!("unknown option {flag}")));
                }
                _ if scene.is_some() => {
                    return Err(Some(format!("only one scene file, but also got {arg}")));
                }
                _ => scene = Some(PathBuf::from(arg)),
            }
        }
        match scene {
            Some(scene) => Ok(Self { scene, assets }),
            None => Err(Some("no scene file given".to_owned())),
        }
    }
}

/// The nearest folder named `assets` among the folders holding `scene`
/// (e.g. `game/assets` for `game/assets/scenes/level.ron`), or `None`.
pub fn asset_folder_of(scene: &Path) -> Option<PathBuf> {
    scene
        .ancestors()
        .skip(1)
        .find(|folder| folder.file_name().is_some_and(|name| name == "assets"))
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scene_and_an_optional_asset_folder() {
        assert_eq!(
            Options::from_args(["level.ron"]),
            Ok(Options {
                scene: "level.ron".into(),
                assets: None
            })
        );
        assert_eq!(
            Options::from_args(["--assets", "game/assets", "level.ron"]),
            Ok(Options {
                scene: "level.ron".into(),
                assets: Some("game/assets".into())
            })
        );
        assert_eq!(Options::from_args(["--help"]), Err(None));
        for bad in [
            &[][..],
            &["--assets"][..],
            &["--fast", "a.ron"][..],
            &["a.ron", "b.ron"][..],
        ] {
            let error = Options::from_args(bad.iter().copied()).expect_err("rejected");
            assert!(error.is_some(), "{bad:?} gets a message");
        }
    }

    #[test]
    fn the_asset_folder_is_the_nearest_one_named_assets() {
        assert_eq!(
            asset_folder_of(Path::new("/games/swarm/assets/scenes/level.ron")),
            Some(PathBuf::from("/games/swarm/assets"))
        );
        assert_eq!(
            asset_folder_of(Path::new("assets/a/assets/b/x.ron")),
            Some(PathBuf::from("assets/a/assets"))
        );
        assert_eq!(asset_folder_of(Path::new("/tmp/level.ron")), None);
        assert_eq!(
            asset_folder_of(Path::new("/x/assets")),
            None,
            "a file named assets is not its own folder"
        );
    }
}

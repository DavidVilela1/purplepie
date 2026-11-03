//! The asset folder's files: images and fonts offered when adding sprites and
//! texts (ADR-044, PP-038i2), and the project panel's tree of scenes, images,
//! fonts and sounds (PP-044b, ADR-046).

use std::path::Path;

/// Image files the engine loads as textures.
pub(crate) const IMAGES: &[&str] = &["png"];
/// Font files the engine loads.
pub(crate) const FONTS: &[&str] = &["ttf", "otf"];
/// Scene files.
pub(crate) const SCENES: &[&str] = &["ron"];
/// Sound files the engine loads.
pub(crate) const SOUNDS: &[&str] = &["wav", "ogg"];

/// What a file in the project panel is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Scene,
    Image,
    Font,
    Sound,
}

impl Kind {
    /// The kind of a file with this extension (any case), if the panel shows it.
    pub(crate) fn of(path: &Path) -> Option<Kind> {
        let ext = path.extension()?;
        [
            (SCENES, Kind::Scene),
            (IMAGES, Kind::Image),
            (FONTS, Kind::Font),
            (SOUNDS, Kind::Sound),
        ]
        .into_iter()
        .find(|(exts, _)| exts.iter().any(|e| ext.eq_ignore_ascii_case(e)))
        .map(|(_, kind)| kind)
    }

    /// A short word shown after the file name.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Kind::Scene => "scene",
            Kind::Image => "image",
            Kind::Font => "font",
            Kind::Sound => "sound",
        }
    }
}

/// A file in the tree: its name, its path relative to the asset folder (with
/// `/`), and its kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) kind: Kind,
}

/// A folder in the tree: its sub-folders, then its files, each sorted by name.
/// Folders with nothing to show are left out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Folder {
    pub(crate) name: String,
    pub(crate) folders: Vec<Folder>,
    pub(crate) files: Vec<Entry>,
}

/// The tree of `root` (the asset folder), and whether files were left out
/// because there were more than the panel lists (500). Hidden files and
/// folders (starting with `.`) and files of other kinds are skipped.
pub(crate) fn tree(root: &Path) -> (Folder, bool) {
    let mut count = 0;
    let folder = walk(root, "", String::new(), &mut count);
    (folder, count > MAX_FILES)
}

fn walk(folder: &Path, prefix: &str, name: String, count: &mut usize) -> Folder {
    let mut out = Folder {
        name,
        ..Folder::default()
    };
    let Ok(entries) = std::fs::read_dir(folder) else {
        return out;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let relative = format!("{prefix}{name}");
        if path.is_dir() {
            let sub = walk(&path, &format!("{relative}/"), name, count);
            if !sub.folders.is_empty() || !sub.files.is_empty() {
                out.folders.push(sub);
            }
        } else if let Some(kind) = Kind::of(&path) {
            *count += 1;
            if *count <= MAX_FILES {
                out.files.push(Entry {
                    name,
                    path: relative,
                    kind,
                });
            }
        }
    }
    out
}

/// At most this many files are listed (a huge folder stays usable).
const MAX_FILES: usize = 500;

/// Files under `root` whose extension is one of `extensions` (any case), as
/// paths relative to `root` with `/` between folders, sorted. Hidden files and
/// folders (starting with `.`) are skipped; unreadable folders are ignored.
pub(crate) fn files_under(root: &Path, extensions: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    collect(root, "", extensions, &mut found);
    found.sort();
    found.truncate(MAX_FILES);
    found
}

fn collect(folder: &Path, prefix: &str, extensions: &[&str], found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        if found.len() >= MAX_FILES {
            return;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let relative = format!("{prefix}{name}");
        let path = entry.path();
        if path.is_dir() {
            collect(&path, &format!("{relative}/"), extensions, found);
        } else if path.extension().is_some_and(|ext| {
            extensions
                .iter()
                .any(|wanted| ext.eq_ignore_ascii_case(wanted))
        }) {
            found.push(relative);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_and_fonts_are_listed_relative_to_the_asset_folder() {
        let root =
            std::env::temp_dir().join(format!("purplepie-editor-library-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for file in [
            "textures/b.png",
            "textures/A.PNG",
            "textures/sheets/c.png",
            "textures/notes.txt",
            ".hidden/x.png",
            "fonts/f.ttf",
            "fonts/g.otf",
            "fonts/OFL.txt",
        ] {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().expect("folder")).expect("folder");
            std::fs::write(&path, b"").expect("file");
        }
        assert_eq!(
            files_under(&root, IMAGES),
            ["textures/A.PNG", "textures/b.png", "textures/sheets/c.png"]
        );
        assert_eq!(files_under(&root, FONTS), ["fonts/f.ttf", "fonts/g.otf"]);
        assert!(files_under(&root.join("missing"), IMAGES).is_empty());
        std::fs::remove_dir_all(&root).expect("clean up");
    }

    #[test]
    fn the_project_tree_shows_scenes_images_fonts_and_sounds_by_folder() {
        let root =
            std::env::temp_dir().join(format!("purplepie-editor-tree-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for file in [
            "levels/level2.ron",
            "levels/level1.ron",
            "textures/a.png",
            "sounds/hit.WAV",
            "sounds/music.ogg",
            "fonts/f.ttf",
            "fonts/OFL.txt",
            "notes/readme.md",
            ".git/x.ron",
            "top.ron",
        ] {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().expect("folder")).expect("folder");
            std::fs::write(&path, b"").expect("file");
        }
        let (top, truncated) = tree(&root);
        assert!(!truncated);
        let names: Vec<&str> = top.folders.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            ["fonts", "levels", "sounds", "textures"],
            "no notes, no .git"
        );
        assert_eq!(top.files.len(), 1);
        assert_eq!(
            (top.files[0].path.as_str(), top.files[0].kind),
            ("top.ron", Kind::Scene)
        );
        let levels = &top.folders[1];
        let paths: Vec<&str> = levels.files.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, ["levels/level1.ron", "levels/level2.ron"]);
        let kinds: Vec<Kind> = top.folders[2].files.iter().map(|e| e.kind).collect();
        assert_eq!(kinds, [Kind::Sound, Kind::Sound], "any case");
        assert_eq!(top.folders[0].files.len(), 1, "OFL.txt is not shown");
        assert_eq!(Kind::Image.word(), "image");
        assert_eq!(tree(&root.join("missing")), (Folder::default(), false));
        std::fs::remove_dir_all(&root).expect("clean up");
    }

    #[test]
    fn a_huge_folder_is_cut_at_the_cap() {
        let root =
            std::env::temp_dir().join(format!("purplepie-editor-cap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("folder");
        for i in 0..(MAX_FILES + 3) {
            std::fs::write(root.join(format!("s{i:04}.ron")), b"").expect("file");
        }
        let (tree, truncated) = tree(&root);
        assert!(truncated);
        assert_eq!(tree.files.len(), MAX_FILES);
        std::fs::remove_dir_all(&root).expect("clean up");
    }
}

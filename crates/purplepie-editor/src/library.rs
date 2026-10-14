//! The asset folder's images and fonts, offered when adding sprites and texts
//! (ADR-044, PP-038i2).

use std::path::Path;

/// Image files the engine loads as textures.
pub(crate) const IMAGES: &[&str] = &["png"];
/// Font files the engine loads.
pub(crate) const FONTS: &[&str] = &["ttf", "otf"];

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
}

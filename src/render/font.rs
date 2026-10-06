//! Font handles and the CPU-side font store (ADR-027).
//!
//! Fonts follow the texture pattern (ADR-020): game code loads a TrueType or
//! OpenType file through [`Context::load_font`](crate::Context::load_font) and
//! gets back a [`FontId`]. Parsing happens immediately, so a missing or broken
//! file is reported to the caller. Glyphs are rasterized later, inside the
//! renderer, at the size they are drawn (see `atlas.rs`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ab_glyph::{Font as _, FontArc};

use crate::error::{BoxError, Error, Result};

/// Identifies a font loaded with [`Context::load_font`](crate::Context::load_font).
///
/// A small `Copy` value: store it in [`Text`](super::Text) components or
/// anywhere else. Fonts stay loaded until the engine stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontId(u32);

impl FontId {
    /// Position in the store.
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

/// A parsed font file.
#[derive(Clone)]
pub(crate) struct FontData {
    /// Where the font came from (file path), for messages.
    pub(crate) source: PathBuf,
    pub(crate) font: FontArc,
}

impl std::fmt::Debug for FontData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontData")
            .field("source", &self.source)
            .field("glyphs", &self.font.glyph_count())
            .finish()
    }
}

/// Parses TrueType / OpenType bytes (`.ttf`, `.otf`; the first font of a collection).
pub(crate) fn decode_font(bytes: Vec<u8>) -> std::result::Result<FontArc, BoxError> {
    let font = FontArc::try_from_vec(bytes)?;
    if font.units_per_em().is_none() {
        return Err("the font has no valid units-per-em value".into());
    }
    if font.height_unscaled() <= 0.0 {
        return Err("the font's ascent and descent give it no height".into());
    }
    Ok(font)
}

/// Every font loaded during this run, in load order. Owned by the engine's
/// runner and read by the renderer when it lays out text.
#[derive(Debug, Default)]
pub(crate) struct Fonts {
    entries: Vec<FontData>,
    /// Path → id, so loading the same path twice returns the same font.
    by_path: HashMap<PathBuf, FontId>,
}

impl Fonts {
    /// Loads and parses the font at `path` (normally a full path from the asset
    /// root; `Context::load_font` resolves it). A path that is already loaded
    /// (same spelling) returns the existing id without reading the file.
    pub(crate) fn load(&mut self, path: &Path) -> Result<FontId> {
        if let Some(&id) = self.by_path.get(path) {
            return Ok(id);
        }
        let asset_error = |source: BoxError| Error::Asset {
            path: path.to_path_buf(),
            source,
        };
        let bytes = std::fs::read(path).map_err(|e| asset_error(Box::new(e)))?;
        let font = decode_font(bytes).map_err(asset_error)?;
        let id = self.push(FontData {
            source: path.to_path_buf(),
            font,
        });
        self.by_path.insert(path.to_path_buf(), id);
        log::debug!("loaded font {} as {id:?}", path.display());
        Ok(id)
    }

    /// Adds an already parsed font (tests).
    pub(crate) fn push(&mut self, data: FontData) -> FontId {
        let index = u32::try_from(self.entries.len()).unwrap_or(u32::MAX);
        self.entries.push(data);
        FontId(index)
    }

    /// Number of fonts loaded so far.
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn get(&self, id: FontId) -> Option<&FontData> {
        self.entries.get(id.index())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::error::Error as _;

    /// The font shipped in `assets/fonts` (OFL-1.1, see `assets/fonts/OFL.txt`).
    pub(crate) const POPPINS: &[u8] = include_bytes!("../../assets/fonts/Poppins-Regular.ttf");

    /// A font store holding only the shipped font, and its id.
    pub(crate) fn poppins() -> (Fonts, FontId) {
        let mut fonts = Fonts::default();
        let id = fonts.push(FontData {
            source: PathBuf::from("Poppins-Regular.ttf"),
            font: decode_font(POPPINS.to_vec()).expect("shipped font parses"),
        });
        (fonts, id)
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("purplepie-{}-{name}", std::process::id()))
    }

    #[test]
    fn the_shipped_font_parses_and_covers_printable_ascii() {
        let font = decode_font(POPPINS.to_vec()).expect("parse");
        assert_eq!(font.units_per_em(), Some(1000.0));
        for c in ' '..='~' {
            assert_ne!(font.glyph_id(c).0, 0, "{c:?} has a glyph");
        }
    }

    #[test]
    fn load_assigns_ids_and_reuses_them_per_path() {
        let path = temp_path("font.ttf");
        std::fs::write(&path, POPPINS).expect("write");
        let mut fonts = Fonts::default();
        let a = fonts.load(&path).expect("load");
        let again = fonts.load(&path).expect("load again");
        std::fs::remove_file(&path).ok();
        assert_eq!(a, again);
        assert_eq!(a.index(), 0);
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts.get(a).expect("entry").source, path);
    }

    #[test]
    fn missing_font_is_an_asset_error_naming_the_path() {
        let path = temp_path("no-such-font.ttf");
        let mut fonts = Fonts::default();
        let err = fonts.load(&path).expect_err("missing file must fail");
        assert!(matches!(&err, Error::Asset { path: p, .. } if *p == path));
        assert!(err.to_string().contains("no-such-font.ttf"), "{err}");
        assert!(
            err.source()
                .and_then(|s| s.downcast_ref::<std::io::Error>())
                .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        );
        assert_eq!(fonts.len(), 0);
    }

    #[test]
    fn invalid_font_is_an_asset_error_and_is_not_cached() {
        let path = temp_path("broken.ttf");
        std::fs::write(&path, b"not a font at all").expect("write");
        let mut fonts = Fonts::default();
        let err = fonts.load(&path).expect_err("garbage must fail");
        assert!(matches!(err, Error::Asset { .. }));
        assert!(decode_font(Vec::new()).is_err());
        // Fixing the file and loading again works: failures are not remembered.
        std::fs::write(&path, POPPINS).expect("write");
        let id = fonts.load(&path).expect("now valid");
        std::fs::remove_file(&path).ok();
        assert_eq!(id.index(), 0);
    }
}

//! Texture handles and the CPU-side texture store (ADR-020).
//!
//! Game code loads a PNG through [`Context::load_texture`](crate::Context::load_texture)
//! and gets back a [`TextureId`]: plain, copyable data that can live in
//! components. Decoding happens immediately, so a missing or broken file is
//! reported to the caller. The GPU upload happens later, inside the
//! renderer, so no wgpu type ever reaches game code (ADR-009).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{BoxError, Error, Result};
use crate::math::Vec2;

/// Identifies a texture loaded with
/// [`Context::load_texture`](crate::Context::load_texture).
///
/// A small `Copy` value: store it in [`Sprite`](super::Sprite) components or
/// anywhere else. It can only be obtained from the engine, so it always refers
/// to a loaded texture. Textures stay loaded until the engine stops (there is
/// no unloading yet; see ADR-020).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextureId(u32);

impl TextureId {
    /// Position in the store, which is also the renderer's upload order.
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

/// Decoded pixels: RGBA, 8 bits per channel, sRGB, straight (not premultiplied) alpha,
/// rows top to bottom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextureData {
    /// Where the pixels came from (file path), for labels and error messages.
    pub(crate) source: PathBuf,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Vec<u8>,
}

/// Decodes a PNG file's bytes into RGBA8. Any PNG colour type and bit depth is
/// accepted: grey, palette and RGB gain an opaque alpha, 16-bit channels are
/// reduced to 8 bits.
pub(crate) fn decode_png(bytes: &[u8]) -> std::result::Result<(u32, u32, Vec<u8>), BoxError> {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)?;
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    if width == 0 || height == 0 {
        return Err("the image has no pixels".into());
    }
    Ok((width, height, rgba.into_raw()))
}

/// Every texture loaded during this run, in load order. Owned by the engine's
/// runner. The renderer uploads entries it has not seen yet before each frame,
/// and re-uploads all of them if it is recreated (e.g. after `suspended`).
#[derive(Debug, Default)]
pub(crate) struct Textures {
    entries: Vec<TextureData>,
    /// Path → id, so loading the same path twice returns the same texture.
    by_path: HashMap<PathBuf, TextureId>,
}

impl Textures {
    /// Loads and decodes the PNG at `path`. Relative paths are resolved against
    /// the process's current working directory. Loading a path that is already
    /// loaded (same spelling) returns the existing id without reading the file.
    pub(crate) fn load(&mut self, path: &Path) -> Result<TextureId> {
        if let Some(&id) = self.by_path.get(path) {
            return Ok(id);
        }
        let asset_error = |source: BoxError| Error::Asset {
            path: path.to_path_buf(),
            source,
        };
        let bytes = std::fs::read(path).map_err(|e| asset_error(Box::new(e)))?;
        let (width, height, pixels) = decode_png(&bytes).map_err(asset_error)?;
        let id = self.push(TextureData {
            source: path.to_path_buf(),
            width,
            height,
            pixels,
        });
        self.by_path.insert(path.to_path_buf(), id);
        log::debug!(
            "loaded texture {} ({width}×{height}) as {id:?}",
            path.display()
        );
        Ok(id)
    }

    fn push(&mut self, data: TextureData) -> TextureId {
        let index = u32::try_from(self.entries.len()).unwrap_or(u32::MAX);
        self.entries.push(data);
        TextureId(index)
    }

    /// Number of textures loaded so far.
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn get(&self, id: TextureId) -> Option<&TextureData> {
        self.entries.get(id.index())
    }

    /// Width and height in texels.
    pub(crate) fn size(&self, id: TextureId) -> Option<Vec2> {
        self.get(id)
            .map(|t| Vec2::new(t.width as f32, t.height as f32))
    }

    /// Entries from position `start` on, with their ids (the renderer's pending uploads).
    pub(crate) fn since(&self, start: usize) -> impl Iterator<Item = (TextureId, &TextureData)> {
        self.entries
            .iter()
            .enumerate()
            .skip(start)
            .map(|(i, data)| (TextureId(u32::try_from(i).unwrap_or(u32::MAX)), data))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::error::Error as _;

    /// Encodes `pixels` (RGBA8) as a PNG in memory.
    pub(crate) fn encode_png(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
        let mut out = std::io::Cursor::new(Vec::new());
        image::RgbaImage::from_raw(width, height, pixels.to_vec())
            .expect("pixel count matches the size")
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("PNG encoding");
        out.into_inner()
    }

    /// A unique scratch file path for one test.
    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("purplepie-{}-{name}", std::process::id()))
    }

    const PIXELS_2X2: [u8; 16] = [
        255, 0, 0, 255, /**/ 0, 255, 0, 255, // top row: red, green
        0, 0, 255, 255, /**/ 255, 255, 255, 0, // bottom row: blue, transparent white
    ];

    #[test]
    fn png_decodes_to_rgba_rows_top_to_bottom() {
        let (w, h, pixels) = decode_png(&encode_png(2, 2, &PIXELS_2X2)).expect("decode");
        assert_eq!((w, h), (2, 2));
        assert_eq!(pixels, PIXELS_2X2);
    }

    #[test]
    fn greyscale_and_rgb_pngs_gain_an_opaque_alpha() {
        let mut grey = std::io::Cursor::new(Vec::new());
        image::GrayImage::from_raw(2, 1, vec![0, 200])
            .expect("size")
            .write_to(&mut grey, image::ImageFormat::Png)
            .expect("encode");
        let (_, _, pixels) = decode_png(grey.get_ref()).expect("decode");
        assert_eq!(pixels, [0, 0, 0, 255, 200, 200, 200, 255]);

        let mut rgb = std::io::Cursor::new(Vec::new());
        image::RgbImage::from_raw(1, 1, vec![10, 20, 30])
            .expect("size")
            .write_to(&mut rgb, image::ImageFormat::Png)
            .expect("encode");
        let (_, _, pixels) = decode_png(rgb.get_ref()).expect("decode");
        assert_eq!(pixels, [10, 20, 30, 255]);
    }

    #[test]
    fn sixteen_bit_pngs_are_reduced_to_eight_bits() {
        let mut out = std::io::Cursor::new(Vec::new());
        image::ImageBuffer::<image::Rgba<u16>, _>::from_raw(1, 1, vec![65535_u16, 0, 32896, 65535])
            .expect("size")
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("encode");
        let (_, _, pixels) = decode_png(out.get_ref()).expect("decode");
        assert_eq!(pixels, [255, 0, 128, 255]);
    }

    #[test]
    fn non_png_bytes_are_rejected() {
        assert!(decode_png(b"definitely not a png").is_err());
        assert!(decode_png(&[]).is_err());
    }

    #[test]
    fn the_sandbox_texture_decodes_to_the_expected_quadrants() {
        let bytes = include_bytes!("../../assets/textures/sandbox_quadrants.png");
        let (w, h, pixels) = decode_png(bytes).expect("decode");
        assert_eq!((w, h), (16, 16));
        let at = |x: usize, y: usize| &pixels[(y * 16 + x) * 4..][..4];
        assert_eq!(at(0, 0), [0, 0, 0, 0], "1-texel transparent border");
        assert_eq!(at(3, 3), [0xE6, 0x39, 0x46, 255], "top-left");
        assert_eq!(at(12, 3), [0x2A, 0x9D, 0x8F, 255], "top-right");
        assert_eq!(at(3, 12), [0xF4, 0xA2, 0x61, 255], "bottom-left");
        assert_eq!(
            at(12, 12),
            [0x3A, 0x86, 0xFF, 128],
            "bottom-right, half transparent"
        );
    }

    #[test]
    fn load_assigns_sequential_ids_and_reuses_them_per_path() {
        let a = temp_path("a.png");
        let b = temp_path("b.png");
        std::fs::write(&a, encode_png(2, 2, &PIXELS_2X2)).expect("write");
        std::fs::write(&b, encode_png(1, 1, &[1, 2, 3, 4])).expect("write");

        let mut textures = Textures::default();
        let id_a = textures.load(&a).expect("load a");
        let id_b = textures.load(&b).expect("load b");
        let id_a_again = textures.load(&a).expect("load a again");
        std::fs::remove_file(&a).ok();
        std::fs::remove_file(&b).ok();

        assert_eq!(id_a.index(), 0);
        assert_eq!(id_b.index(), 1);
        assert_eq!(id_a_again, id_a, "same path, same texture");
        assert_eq!(textures.len(), 2);
        assert_eq!(textures.size(id_a), Some(Vec2::new(2.0, 2.0)));
        assert_eq!(textures.size(id_b), Some(Vec2::new(1.0, 1.0)));
        let data = textures.get(id_b).expect("entry");
        assert_eq!(data.pixels, [1, 2, 3, 4]);
        assert_eq!(data.source, b);
    }

    #[test]
    fn since_lists_only_entries_not_yet_seen() {
        let a = temp_path("since.png");
        std::fs::write(&a, encode_png(1, 1, &[0, 0, 0, 255])).expect("write");
        let mut textures = Textures::default();
        let first = textures.load(&a).expect("load");
        std::fs::remove_file(&a).ok();
        assert_eq!(
            textures.since(0).map(|(id, _)| id).collect::<Vec<_>>(),
            [first]
        );
        assert_eq!(textures.since(1).count(), 0);
    }

    #[test]
    fn missing_file_is_an_asset_error_naming_the_path() {
        let path = temp_path("does-not-exist.png");
        let mut textures = Textures::default();
        let err = textures.load(&path).expect_err("missing file must fail");
        assert!(matches!(&err, Error::Asset { path: p, .. } if *p == path));
        assert!(err.to_string().contains("does-not-exist.png"), "{err}");
        let source = err.source().expect("io error kept as source");
        assert!(
            source
                .downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        );
        assert_eq!(textures.len(), 0, "failed loads store nothing");
    }

    #[test]
    fn invalid_file_is_an_asset_error_and_is_not_cached() {
        let path = temp_path("broken.png");
        std::fs::write(&path, b"not a png").expect("write");
        let mut textures = Textures::default();
        let err = textures.load(&path).expect_err("garbage must fail");
        assert!(matches!(err, Error::Asset { .. }));
        // Fixing the file and loading again works: failures are not remembered.
        std::fs::write(&path, encode_png(1, 1, &[9, 9, 9, 255])).expect("write");
        let id = textures.load(&path).expect("now valid");
        std::fs::remove_file(&path).ok();
        assert_eq!(id.index(), 0);
    }
}

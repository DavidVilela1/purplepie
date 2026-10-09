//! Texture handles and the CPU-side texture store (ADR-020).
//!
//! Game code loads a PNG through [`Context::load_texture`](crate::Context::load_texture)
//! and gets back a [`TextureId`]: plain, copyable data that can live in
//! components. Decoding happens immediately, so a missing or broken file is
//! reported to the caller. The GPU upload happens later, inside the
//! renderer, so no wgpu type ever reaches game code (ADR-009). How a texture
//! is sampled ([`TextureFilter`]) is chosen per texture when it loads (ADR-034).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::assets::watch::{FileWatch, ReloadReport, reload_if_changed};
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

/// How a texture's texels are blended when it is drawn larger, smaller, rotated
/// or between whole pixels (ADR-034).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TextureFilter {
    /// Every pixel shows exactly one texel: crisp, blocky edges. Right for
    /// pixel art and exact at whole-number scales; scaled-down or rotated
    /// sprites shimmer. The default.
    #[default]
    Nearest,
    /// Blends the four nearest texels: smooth when scaled, rotated or moved by
    /// fractions of a pixel, slightly soft at large magnification. Without
    /// mipmaps, shrinking below half size still aliases. Fully transparent
    /// texels take on their neighbours' colour when the texture loads, so
    /// edges blend without dark fringes.
    Linear,
}

/// Options for [`Context::load_texture_with`](crate::Context::load_texture_with).
///
/// Build one from [`TextureOptions::NEAREST`] (the default, what
/// [`Context::load_texture`](crate::Context::load_texture) uses) or
/// [`TextureOptions::LINEAR`], or with [`with_filter`](Self::with_filter).
/// More options may be added later, so the struct cannot be built with a
/// literal.
///
/// ```
/// use purplepie::render::{TextureFilter, TextureOptions};
///
/// assert_eq!(TextureOptions::default(), TextureOptions::NEAREST);
/// assert_eq!(TextureOptions::LINEAR.filter, TextureFilter::Linear);
/// assert_eq!(TextureOptions::default().with_filter(TextureFilter::Linear), TextureOptions::LINEAR);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TextureOptions {
    /// How the texture is sampled. Default: [`TextureFilter::Nearest`].
    pub filter: TextureFilter,
}

impl TextureOptions {
    /// Nearest-texel sampling: crisp pixel art (the default).
    pub const NEAREST: Self = Self {
        filter: TextureFilter::Nearest,
    };
    /// Linear (bilinear) sampling: smooth scaled and rotated sprites.
    pub const LINEAR: Self = Self {
        filter: TextureFilter::Linear,
    };

    /// The same options with `filter`.
    pub const fn with_filter(mut self, filter: TextureFilter) -> Self {
        self.filter = filter;
        self
    }
}

/// Gives every fully transparent texel (alpha 0) the average colour of its
/// nearest non-transparent texels, spreading outwards ring by ring
/// (8-neighbourhood), and keeps its alpha at 0. Texels with any alpha are not
/// changed. Linear filtering mixes the colour of neighbouring texels, so
/// without this the (usually black) colour of transparent texels would darken
/// the edges of a sprite drawn with straight alpha. A texture with no visible
/// texel is left as it is.
pub(crate) fn bleed_transparent_texels(width: u32, height: u32, pixels: &mut [u8]) {
    let (w, h) = (width as usize, height as usize);
    debug_assert_eq!(pixels.len(), w * h * 4);
    let mut done: Vec<bool> = pixels.as_chunks::<4>().0.iter().map(|p| p[3] > 0).collect();
    if done.len() != w * h || !done.iter().any(|&d| d) {
        return;
    }
    let neighbours = |i: usize| {
        let (x, y) = ((i % w) as isize, (i / w) as isize);
        (-1..=1_isize)
            .flat_map(move |dy| (-1..=1_isize).map(move |dx| (x + dx, y + dy)))
            .filter(move |&(nx, ny)| {
                (nx, ny) != (x, y) && nx >= 0 && ny >= 0 && nx < w as isize && ny < h as isize
            })
            .map(move |(nx, ny)| ny as usize * w + nx as usize)
    };
    let mut queued = done.clone();
    let mut ring: Vec<usize> = Vec::new();
    for i in 0..w * h {
        if !done[i] && neighbours(i).any(|n| done[n]) {
            queued[i] = true;
            ring.push(i);
        }
    }
    while !ring.is_empty() {
        let colours: Vec<(usize, [u8; 3])> = ring
            .iter()
            .map(|&i| {
                let (mut sum, mut count) = ([0_u32; 3], 0_u32);
                for n in neighbours(i).filter(|&n| done[n]) {
                    for (c, total) in sum.iter_mut().enumerate() {
                        *total += u32::from(pixels[n * 4 + c]);
                    }
                    count += 1;
                }
                // `count` ≥ 1: every texel in the ring touches a finished one.
                let average = sum.map(|total| ((total + count / 2) / count.max(1)) as u8);
                (i, average)
            })
            .collect();
        for &(i, rgb) in &colours {
            pixels[i * 4..i * 4 + 3].copy_from_slice(&rgb);
            done[i] = true;
        }
        let mut next = Vec::new();
        for &(i, _) in &colours {
            for n in neighbours(i) {
                if !queued[n] {
                    queued[n] = true;
                    next.push(n);
                }
            }
        }
        ring = next;
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
    /// How the renderer samples it.
    pub(crate) filter: TextureFilter,
    /// Bumped each time hot reload replaces the pixels (ADR-037); the renderer
    /// re-uploads a texture whose revision differs from the one it uploaded.
    pub(crate) revision: u32,
}

/// Reads and decodes `path` for a texture sampled with `filter`.
fn read_texture(
    path: &Path,
    filter: TextureFilter,
) -> std::result::Result<(u32, u32, Vec<u8>), BoxError> {
    let bytes = std::fs::read(path)?;
    let (width, height, mut pixels) = decode_png(&bytes)?;
    if filter == TextureFilter::Linear {
        bleed_transparent_texels(width, height, &mut pixels);
    }
    Ok((width, height, pixels))
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
    /// (Path, options) → id, so loading the same path with the same options
    /// twice returns the same texture.
    by_path: HashMap<(PathBuf, TextureOptions), TextureId>,
    /// Parallel to `entries`: file stamps for hot reload.
    watches: Vec<FileWatch>,
}

impl Textures {
    /// Loads and decodes the PNG at `path`, normally a full path from the asset
    /// root (ADR-025; `Context::load_texture` resolves it), to be sampled as
    /// `options` say. Loading a path that is already loaded (same spelling)
    /// with the same options returns the existing id without reading the file;
    /// other options make a separate texture.
    pub(crate) fn load(&mut self, path: &Path, options: TextureOptions) -> Result<TextureId> {
        let key = (path.to_path_buf(), options);
        if let Some(&id) = self.by_path.get(&key) {
            return Ok(id);
        }
        let asset_error = |source: BoxError| Error::Asset {
            path: path.to_path_buf(),
            source,
        };
        // Stamp before reading: a write that lands during the read shows up
        // as a change on the next poll instead of being missed.
        let watch = FileWatch::before_read(path);
        let (width, height, pixels) = read_texture(path, options.filter).map_err(asset_error)?;
        let id = self.push(TextureData {
            source: path.to_path_buf(),
            width,
            height,
            pixels,
            filter: options.filter,
            revision: 0,
        });
        self.watches.push(watch);
        self.by_path.insert(key, id);
        log::debug!(
            "loaded texture {} ({width}×{height}, {:?}) as {id:?}",
            path.display(),
            options.filter
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

    /// Re-reads every texture whose file changed since it was last read
    /// (modification time or length), keeping its [`TextureId`], filter and
    /// `Linear` bleeding; bumps its revision so the renderer re-uploads it. A
    /// file that cannot be read or decoded leaves the texture as it was and is
    /// reported once until it changes again. Missing files are ignored (an
    /// editor may be replacing them). Used by hot reload (ADR-037).
    pub(crate) fn reload_changed(&mut self) -> ReloadReport<TextureId> {
        let mut report = ReloadReport::default();
        for (index, (data, watch)) in self.entries.iter_mut().zip(&mut self.watches).enumerate() {
            let id = TextureId(u32::try_from(index).unwrap_or(u32::MAX));
            let source = data.source.clone();
            reload_if_changed(&source, watch, id, "texture", &mut report, || {
                let (width, height, pixels) = read_texture(&source, data.filter)?;
                data.width = width;
                data.height = height;
                data.pixels = pixels;
                data.revision = data.revision.wrapping_add(1);
                Ok(format!("{width}×{height}"))
            });
        }
        report
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

    /// `N` distinct texture ids for tests that never touch pixels or the GPU.
    pub(crate) fn texture_ids<const N: usize>() -> [TextureId; N] {
        std::array::from_fn(|i| TextureId(u32::try_from(i).expect("small test index")))
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
        let id_a = textures.load(&a, TextureOptions::NEAREST).expect("load a");
        let id_b = textures.load(&b, TextureOptions::NEAREST).expect("load b");
        let id_a_again = textures
            .load(&a, TextureOptions::NEAREST)
            .expect("load a again");
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
    fn options_default_to_nearest_and_key_the_cache() {
        assert_eq!(TextureOptions::default(), TextureOptions::NEAREST);
        assert_eq!(TextureFilter::default(), TextureFilter::Nearest);
        assert_eq!(
            TextureOptions::NEAREST.with_filter(TextureFilter::Linear),
            TextureOptions::LINEAR
        );
        let path = temp_path("options.png");
        std::fs::write(&path, encode_png(2, 2, &PIXELS_2X2)).expect("write");
        let mut textures = Textures::default();
        let nearest = textures.load(&path, TextureOptions::NEAREST).expect("load");
        let linear = textures.load(&path, TextureOptions::LINEAR).expect("load");
        assert_eq!(
            textures.load(&path, TextureOptions::LINEAR).expect("again"),
            linear
        );
        std::fs::remove_file(&path).ok();
        assert_ne!(nearest, linear);
        assert_eq!(textures.len(), 2);
        let (n, l) = (
            textures.get(nearest).expect("n"),
            textures.get(linear).expect("l"),
        );
        assert_eq!(
            (n.filter, l.filter),
            (TextureFilter::Nearest, TextureFilter::Linear)
        );
        assert_eq!(n.pixels, PIXELS_2X2, "Nearest keeps the file's texels");
        // Linear: the transparent texel takes its neighbours' average colour, alpha stays 0.
        assert_eq!(l.pixels[..12], PIXELS_2X2[..12]);
        assert_eq!(l.pixels[12..], [85, 85, 85, 0]);
    }

    #[test]
    fn bleeding_fills_transparent_texels_ring_by_ring() {
        // 4×1: red, transparent, transparent, transparent.
        let mut row = [
            255, 0, 0, 255, /**/ 0, 0, 0, 0, /**/ 0, 0, 0, 0, /**/ 9, 9, 9, 0,
        ];
        bleed_transparent_texels(4, 1, &mut row);
        assert_eq!(
            row,
            [255, 0, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0]
        );
        // Between two colours the first ring averages them; texels with alpha > 0
        // (even 1) are sources and never change.
        let mut row = [200, 0, 0, 1, /**/ 0, 0, 0, 0, /**/ 0, 0, 100, 255];
        bleed_transparent_texels(3, 1, &mut row);
        assert_eq!(row, [200, 0, 0, 1, 100, 0, 50, 0, 0, 0, 100, 255]);
        // 3×3 with one opaque corner: every texel ends up with its colour.
        let mut grid = [0_u8; 36];
        grid[..4].copy_from_slice(&[10, 20, 30, 255]);
        bleed_transparent_texels(3, 3, &mut grid);
        for (i, texel) in grid.as_chunks::<4>().0.iter().enumerate() {
            let alpha = if i == 0 { 255 } else { 0 };
            assert_eq!(*texel, [10, 20, 30, alpha], "texel {i}");
        }
        // Nothing visible: unchanged.
        let mut empty = [7_u8, 7, 7, 0, 0, 0, 0, 0];
        bleed_transparent_texels(2, 1, &mut empty);
        assert_eq!(empty, [7, 7, 7, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn the_breakout_ball_has_no_dark_texels_after_bleeding() {
        let bytes = include_bytes!("../../assets/textures/breakout/ball.png");
        let (w, h, mut pixels) = decode_png(bytes).expect("decode");
        assert!(
            pixels.as_chunks::<4>().0.contains(&[0, 0, 0, 0]),
            "the file stores black transparent texels"
        );
        let alpha_before: Vec<u8> = pixels.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
        bleed_transparent_texels(w, h, &mut pixels);
        assert!(
            pixels
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| p[..3] == [255, 255, 255])
        );
        let alpha_after: Vec<u8> = pixels.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
        assert_eq!(alpha_after, alpha_before, "coverage is untouched");
    }

    use crate::assets::watch::tests::write_changed;

    #[test]
    fn changed_files_are_reloaded_under_the_same_id() {
        let path = temp_path("reload.png");
        std::fs::write(&path, encode_png(2, 2, &PIXELS_2X2)).expect("write");
        let mut textures = Textures::default();
        let nearest = textures.load(&path, TextureOptions::NEAREST).expect("load");
        let linear = textures.load(&path, TextureOptions::LINEAR).expect("load");
        assert_eq!(
            textures.reload_changed(),
            ReloadReport::default(),
            "unchanged"
        );

        // A different size and colour: both textures reload, ids and filters kept.
        let three = [10, 20, 30, 255, 0, 0, 0, 0, 40, 50, 60, 255];
        write_changed(&path, &encode_png(3, 1, &three), 10);
        let report = textures.reload_changed();
        assert_eq!(report.reloaded, [nearest, linear]);
        assert!(report.failed.is_empty());
        let n = textures.get(nearest).expect("nearest");
        assert_eq!((n.width, n.height, n.revision), (3, 1, 1));
        assert_eq!(n.pixels, three);
        assert_eq!(textures.size(nearest), Some(Vec2::new(3.0, 1.0)));
        let l = textures.get(linear).expect("linear");
        assert_eq!((l.filter, l.revision), (TextureFilter::Linear, 1));
        assert_eq!(
            l.pixels[4..8],
            [25, 35, 45, 0],
            "Linear bleeding is applied again"
        );
        assert_eq!(
            textures.reload_changed(),
            ReloadReport::default(),
            "seen once"
        );

        // A broken file keeps the old pixels and is reported once.
        write_changed(&path, b"half-written", 20);
        let report = textures.reload_changed();
        assert_eq!(report.failed, [nearest, linear]);
        assert!(report.reloaded.is_empty());
        assert_eq!(textures.get(nearest).expect("kept").pixels, three);
        assert_eq!(textures.get(nearest).expect("kept").revision, 1);
        assert_eq!(
            textures.reload_changed(),
            ReloadReport::default(),
            "reported once"
        );

        // Fixing it reloads; deleting it is ignored.
        write_changed(&path, &encode_png(1, 1, &[1, 2, 3, 255]), 30);
        assert_eq!(textures.reload_changed().reloaded, [nearest, linear]);
        assert_eq!(textures.get(nearest).expect("fixed").revision, 2);
        std::fs::remove_file(&path).expect("remove");
        assert_eq!(textures.reload_changed(), ReloadReport::default());
        assert_eq!(
            textures.get(nearest).expect("still there").pixels,
            [1, 2, 3, 255]
        );
    }

    #[test]
    fn since_lists_only_entries_not_yet_seen() {
        let a = temp_path("since.png");
        std::fs::write(&a, encode_png(1, 1, &[0, 0, 0, 255])).expect("write");
        let mut textures = Textures::default();
        let first = textures.load(&a, TextureOptions::NEAREST).expect("load");
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
        let err = textures
            .load(&path, TextureOptions::NEAREST)
            .expect_err("missing file must fail");
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
        let err = textures
            .load(&path, TextureOptions::NEAREST)
            .expect_err("garbage must fail");
        assert!(matches!(err, Error::Asset { .. }));
        // Fixing the file and loading again works: failures are not remembered.
        std::fs::write(&path, encode_png(1, 1, &[9, 9, 9, 255])).expect("write");
        let id = textures
            .load(&path, TextureOptions::NEAREST)
            .expect("now valid");
        std::fs::remove_file(&path).ok();
        assert_eq!(id.index(), 0);
    }
}

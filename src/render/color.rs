//! Game-facing colors (ADR-015).

/// An RGBA color with **sRGB** components in `0.0..=1.0` and straight
/// (non-premultiplied) alpha.
///
/// sRGB is what image editors and color pickers show, so `Color::hex(0x6A0DAD)`
/// looks the same on screen as `#6A0DAD` does in an editor. The renderer
/// converts to linear light where the GPU needs it.
///
/// ```
/// use purplepie::render::Color;
///
/// assert_eq!(Color::hex(0xFF8000), Color::rgb8(255, 128, 0));
/// assert_eq!(Color::PURPLEPIE, Color::hex(0x6A0DAD));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red, sRGB-encoded, `0.0..=1.0`.
    pub r: f32,
    /// Green, sRGB-encoded, `0.0..=1.0`.
    pub g: f32,
    /// Blue, sRGB-encoded, `0.0..=1.0`.
    pub b: f32,
    /// Alpha (opacity), `0.0..=1.0`. Not gamma-encoded.
    pub a: f32,
}

impl Color {
    /// PurplePie's signature purple, `#6A0DAD`. The default clear color.
    pub const PURPLEPIE: Self = Self::hex(0x6A0DAD);
    /// Opaque black.
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    /// Opaque white.
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// Opaque color from sRGB components in `0.0..=1.0`.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    /// Color from sRGB components and alpha in `0.0..=1.0`.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Opaque color from 8-bit sRGB components (`0..=255`).
    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba8(r, g, b, 255)
    }

    /// Color from 8-bit sRGB components and alpha (`0..=255`).
    pub const fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Opaque color from a `0xRRGGBB` value, as written in CSS or image editors.
    pub const fn hex(rgb: u32) -> Self {
        Self::rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    }

    /// The same color in linear light: `[r, g, b, a]`. Alpha is unchanged.
    pub fn to_linear(self) -> [f32; 4] {
        [
            srgb_to_linear(self.r),
            srgb_to_linear(self.g),
            srgb_to_linear(self.b),
            self.a,
        ]
    }

    /// Converts to the value the GPU must write so the color *appears* as
    /// specified. For an sRGB target format the GPU encodes linear → sRGB
    /// itself, so it must receive linear values. For a non-sRGB (`Unorm`) target it
    /// stores the value as-is, so it must receive the sRGB values.
    pub(crate) fn to_wgpu(self, target_is_srgb: bool) -> wgpu::Color {
        let [r, g, b, a] = if target_is_srgb {
            self.to_linear()
        } else {
            [self.r, self.g, self.b, self.a]
        };
        wgpu::Color {
            r: f64::from(r),
            g: f64::from(g),
            b: f64::from(b),
            a: f64::from(a),
        }
    }
}

impl Default for Color {
    /// [`Color::PURPLEPIE`].
    fn default() -> Self {
        Self::PURPLEPIE
    }
}

/// sRGB electro-optical transfer function (IEC 61966-2-1), one channel.
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn hex_and_rgb8_agree() {
        assert_eq!(Color::hex(0x6A0DAD), Color::rgb8(0x6A, 0x0D, 0xAD));
        assert_eq!(Color::hex(0xFFFFFF), Color::WHITE);
        assert_eq!(Color::hex(0x000000), Color::BLACK);
        assert_eq!(Color::PURPLEPIE.a, 1.0);
    }

    #[test]
    fn linear_conversion_matches_reference_values() {
        // Endpoints and the linear segment.
        assert_eq!(srgb_to_linear(0.0), 0.0);
        assert!(close(srgb_to_linear(1.0), 1.0));
        assert!(close(srgb_to_linear(0.04), 0.04 / 12.92));
        // Mid-grey: sRGB 0.5 ≈ 0.214 linear (standard reference value).
        assert!(close(srgb_to_linear(0.5), 0.214_041_14));
        // 8-bit 106 (0x6A) ≈ 0.14413 linear.
        assert!(close(srgb_to_linear(106.0 / 255.0), 0.144_128_4));
    }

    #[test]
    fn alpha_is_not_gamma_converted() {
        let [.., a] = Color::rgba(0.5, 0.5, 0.5, 0.5).to_linear();
        assert_eq!(a, 0.5);
    }

    #[test]
    fn wgpu_value_depends_on_target_format() {
        let c = Color::rgb(0.5, 0.5, 0.5);
        let srgb_target = c.to_wgpu(true);
        let unorm_target = c.to_wgpu(false);
        assert!((srgb_target.r - 0.214_041).abs() < 1e-5);
        assert_eq!(unorm_target.r, 0.5);
    }
}

# ADR-0008: Public colors are sRGB

**Status:** Proposed. Confirm in Stage 4/5.

## Context
`surface.get_default_config` usually picks an `*Srgb` format, and the GPU then
encodes linear values to sRGB. In the Stage 0 spike, clear color
`(0.35, 0.10, 0.55)` rendered as `#A059C4` (160, 89, 196), noticeably lighter
than the designer-intended `#591A8C`.

## Proposal
* `Color` stores **sRGB** components (what designers and color pickers use),
  with `Color::rgb(u8,u8,u8)`, `Color::hex(0x6A0DAD)`, and float constructors.
* The renderer converts to linear exactly once (`Color::to_linear()`) when the
  surface/texture format is sRGB. Sprite textures load as `Rgba8UnormSrgb`.
* PurplePie purple: pick the brand value in Stage 4 and add `Color::PURPLEPIE`.

## Consequences
* Colors look the same as in image editors. Blending is correct in linear space.

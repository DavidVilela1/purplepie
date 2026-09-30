# ADR-0009: World units, +Y up, centered camera

**Status:** Proposed. Confirm in Stage 7.

## Proposal
* World space: right-handed 2D, **+X right, +Y up**, units are "world units".
* `Camera2D { position: Vec2, zoom: f32, pixels_per_unit: f32 }` produces an
  orthographic projection centered on `position`. The window resize changes the
  visible area, not the scale.
* `Transform2D.rotation` is in radians, counter-clockwise.
* Draw order is a `z: f32` / layer on `Sprite`, sorted per batch. There is no depth buffer.
* Screen (cursor) coordinates are physical pixels with +Y down. `Camera2D::screen_to_world`
  converts them.
* Hi-DPI: the surface uses physical size, and `scale_factor` is exposed for UI.

## Consequences
* Game math is unaffected by resolution. Physics-like code uses meaningful units.

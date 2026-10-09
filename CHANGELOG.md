# Changelog

All notable changes to PurplePie are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow the policy in
[docs/RELEASING.md](docs/RELEASING.md): semantic versioning where, before 1.0, a minor version (0.**x**.0) may
break compatibility and a patch version (0.x.**y**) may not.

## [Unreleased]

### Added

- `math::Rng`: a seedable random number generator (PCG32): `new(seed)`, `from_entropy()`, `next_u32`, `f32`,
  `range_f32`, `range_u32`, `chance`, `pick`, `shuffle`, `unit_vec2`. The same seed gives the same numbers on every
  platform (ADR-041).
- `math::Rect` (centre + half size: `from_center_size`, `from_corners`, `min`, `max`, `size`, `contains`,
  `overlaps`, `intersection`, `closest_point`, `overlaps_circle`, `expand`) and `math::circles_overlap` for gameplay
  overlap tests (ADR-041).

## [0.1.0] - 2026-10-09

The first release. Depend on it with
`purplepie = { git = "https://github.com/DavidVilela1/purplepie", tag = "v0.1.0" }`.

### Added

- **App and loop:** `Engine` and `EngineConfig` (window size, title, fixed step, frame clamp, clear colour, asset
  root, audio, hot reload, console log); the `Game` trait with `init`, a deterministic fixed-rate `fixed_update`
  (60 Hz by default) and a per-frame `update`; `Context` as the game's single handle to the engine; `Time`.
- **ECS:** one `hecs` world per game (`ecs::World`, `ecs::Entity`, the full `ecs::hecs` re-export), `Velocity` and
  `integrate_velocity`.
- **Drawing:**
  - `Quad` and `Sprite` (PNG textures; per-texture `Nearest` or `Linear` sampling; tint; mirroring);
  - `Layer`, `Hidden` and `Camera2D` (pan, zoom, `fit`, screen ↔ world);
  - sprite sheets (`TextureRegion`, `SpriteGrid`) and frame animation (`SpriteAnimation`, `advance_animations`);
  - draw calls batched by texture.
- **Text:** TrueType/OpenType fonts drawn as `Text` with `TextAnchor`s and `\n` line breaks; `Context::measure_text`.
  One font ships with the engine (Poppins, SIL Open Font License).
- **HUD and UI:** `ScreenSpace` pins entities to a window anchor; `ui::Button` with `ui::update_buttons`.
- **Input:** keyboard (`KeyCode`, named by US-layout position), mouse buttons, cursor (window and world) and wheel.
  Each press is reported once to `fixed_update` and once to `update`.
- **Audio:** WAV and OGG Vorbis sounds; `play_sound`, `loop_sound` (seamless), `stop_sound`, per-playback and
  master volume. Games run silently without an audio device.
- **Assets:** an `assets/` folder next to the executable or in the working directory; typed `Error::Asset` errors
  carrying the full path; opt-in hot reload of textures, fonts and sounds.
- **Scenes:** `save_scene` / `load_scene` write and read entities as RON (format version 1), with all engine
  components and the game's own components registered with `register_scene_component`.
- **Errors and logs:** one `Error` type whose `Debug` output prints the message and its causes; an opt-in console
  logger (`EngineConfig::with_console_log`, level from `PURPLEPIE_LOG`).
- **Examples and docs:** the sandbox, `examples/breakout.rs`, `examples/scene.rs`; `docs/GUIDE.md` (every code
  block compiled by `cargo test`); `docs/CHECKLIST.md`.

### Changed (for code written against pre-release snapshots)

- `ScreenSpace::TOP` / `LEFT` / `RIGHT` / `BOTTOM` are now `TOP_CENTER` / `CENTER_LEFT` / `CENTER_RIGHT` /
  `BOTTOM_CENTER`, and `ScreenAnchor::Top` / … are now `TopCenter` / …. Scene files with the old names still load
  (ADR-040).
- `EngineConfig`, `Sprite`, `Quad`, `Text`, `TextMetrics`, `SpriteAnimation`, `AnimationMode`, `TextureFilter`,
  `Camera2D`, `ui::Button` and `ui::Pointer` are `#[non_exhaustive]`: build them with their constructors and
  builders (ADR-040).
- `Error`'s `Debug` output is the message plus a "Caused by:" list instead of a struct dump, and `Error::Save` holds
  the resolved file path (ADR-040).
- The sandbox's own logger moved into the engine as the opt-in console logger (ADR-040, amending ADR-016).

### Known limitations

These are listed in [docs/PROJECT_STATUS.md](docs/PROJECT_STATUS.md):

- no text wrapping, shaping or font fallback;
- no MP3/FLAC, streaming or fades;
- buttons are the only UI widget;
- one camera, without rotation;
- fatal GPU faults;
- rendering has been seen on Linux (software GPU) and on a Windows AMD integrated GPU; macOS is compiled and tested in
  CI but not yet seen running.

[Unreleased]: https://github.com/DavidVilela1/purplepie/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/DavidVilela1/purplepie/releases/tag/v0.1.0

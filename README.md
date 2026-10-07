# PurplePie

A small, modular, cross-platform **2D game engine** written in Rust, built on
`winit`, `wgpu`, `hecs` and `glam`.

> **Status: Stages 0–10 complete; growing past the portfolio scope: runtime essentials done (text, sprite sheets, animation, per-texture sampling, screen-space HUD, sound, OGG music loops and UI buttons); editor foundations started (scene files).**
> The engine runs a complete game: `cargo run --example breakout`.
> Rendering verified on Linux (Xvfb + software GPU, pixel-checked); CI builds and tests on Linux, Windows and macOS.
> Current state: [docs/PROJECT_STATUS.md](docs/PROJECT_STATUS.md). Next task: [docs/TASKS.md](docs/TASKS.md).

## What it does

- **App and loop:** `Engine` owns the window (winit) and the GPU (wgpu); your `Game` gets `init`, a fixed-rate
  `fixed_update` (60 Hz by default, deterministic) and a per-frame `update`, each with a `Context`.
- **ECS:** one `hecs::World` for your entities and components; you call your own systems.
- **2D rendering:** solid `Quad`s, textured `Sprite`s (PNG, tint, crisp `Nearest` or smooth `Linear` sampling per texture, sprite-sheet regions via `SpriteGrid`, frame animation via `SpriteAnimation`) and `Text` (TrueType/OpenType fonts, rasterized
  sharp at the on-screen size, anchored/aligned with `TextAnchor`, measured with `Context::measure_text`), ordered by `Layer`, hidden with `Hidden`, drawable in screen space for HUDs (`ScreenSpace`) with clickable `ui::Button`s, batched into instanced draw calls, seen
  through a `Camera2D` (pan, zoom, `fit`, screen ↔ world).
- **Input:** keyboard (`KeyCode`), mouse buttons, cursor (screen and world) and wheel, with each press reported exactly
  once per callback regardless of frame rate.
- **Audio:** WAV and OGG Vorbis sounds with `Context::load_sound`, `play_sound` / `loop_sound` (seamless loops), `stop_sound`, per-sound and
  master volume (mixed in software, any sample rate); games keep running silently without an audio device.
- **Assets:** textures, fonts and sounds loaded by paths relative to an `assets/` folder found next to the executable or in the
  project folder. One font ships with the engine: `assets/fonts/Poppins-Regular.ttf` (SIL Open Font License).
- **Scenes:** `Context::save_scene` / `load_scene` write and read the drawable entities (transforms, quads, sprites,
  text, layers, hidden, screen space) as human-readable RON, with textures and fonts referenced by asset path.
- **Errors:** one `Error` type; missing files, GPU loss and device failures end the game cleanly instead of panicking.
- Not included (yet): MP3/FLAC, streamed music and fades, text wrapping and shaping, UI layout, keyboard focus and text input, physics,
  animation/game components in scene files, an editor. See [docs/ROADMAP.md](docs/ROADMAP.md).

## Getting started

`examples/breakout.rs` is a complete game and the best reference. The smallest useful game looks like this
(game code never touches `wgpu` or `winit`):

```rust
// Coordinates: +X right, +Y up, origin at the window centre,
// 1 unit = 1 logical pixel at zoom 1 (ADR-018).
use purplepie::ecs::{self, Velocity};
use purplepie::input::KeyCode;
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Color, Layer, Quad, Sprite, Text, TextAnchor};
use purplepie::{Context, Engine, EngineConfig, Game};

struct MyGame;

impl Game for MyGame {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        // PNG only. A missing or broken file returns Error::Asset right here.
        // Relative to the asset root: `assets/` next to the executable, else
        // `assets/` in the working directory (ADR-025).
        let player = ctx.load_texture("textures/player.png")?;
        ctx.world_mut().spawn((
            Transform2D::from_position(Vec2::new(0.0, 100.0)),
            Sprite::new(player, Vec2::new(64.0, 64.0)),
            Velocity(Vec2::new(50.0, 0.0)), // 50 world units per second
        ));
        ctx.world_mut().spawn((
            Transform2D::from_position(Vec2::new(0.0, -100.0)),
            Quad::new(Vec2::new(200.0, 20.0), Color::WHITE),
            Layer(1), // drawn over layer-0 sprites (no Layer = layer 0)
        ));
        // Text: 24 = font size in world units; the anchor puts the text's top centre at the position.
        let font = ctx.load_font("fonts/Poppins-Regular.ttf")?;
        ctx.world_mut().spawn((
            Transform2D::from_position(Vec2::new(0.0, 340.0)),
            Text::new("Hello, PurplePie!", font, 24.0)
                .with_color(Color::hex(0xF1FAEE))
                .with_anchor(TextAnchor::TOP_CENTER),
        ));
        Ok(())
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        let dt = ctx.dt(); // fixed step: 1/60 s by default
        ecs::integrate_velocity(ctx.world_mut(), dt);
        // Arrow keys pan the camera; each press of Space zooms in once,
        // however many fixed steps this frame runs (ADR-024).
        let pan = ctx.input().axis(KeyCode::ArrowLeft, KeyCode::ArrowRight);
        let zoom_in = ctx.input().just_pressed(KeyCode::Space);
        ctx.camera_mut().position.x += pan * 200.0 * dt;
        if zoom_in {
            ctx.camera_mut().zoom *= 1.5;
        }
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        let title = format!("My Game - {:.0} s", ctx.time().elapsed());
        ctx.set_window_title(title);
    }
}

fn main() -> purplepie::Result<()> {
    let config = EngineConfig::new("My Game").with_size(1280, 720);
    // .with_clear_color(Color::hex(0x202030)) to change the background
    Engine::new(config)?.run(MyGame)
}
```

The sandbox (`cargo run`) is the engine's test bed: arrow keys pan, `=` / `-` or the wheel zoom, a left click stamps a
square at the cursor (the top-right "Reset camera" button resets the view), Escape quits; a text label at the bottom left lists these controls.

## Build

Requires Rust stable (edition 2024; `rust-version = 1.90`, developed with 1.95). On Linux, audio needs the ALSA headers
to build: `sudo apt install libasound2-dev` (Debian/Ubuntu).

```bash
cargo run            # opens the sandbox window (M toggles a music loop; Escape or close to quit)
cargo run --example breakout   # the example game: arrows/A-D/mouse move, Space/click launch, Escape quits
# PURPLEPIE_BREAKOUT_AUTOPLAY=win cargo run --example breakout   a bot plays a whole game (deterministic)
cargo run --example scene      # loads assets/scenes/demo.ron (PURPLEPIE_SCENE_EXAMPLE=build|save: build in code / write it)
# PURPLEPIE_SANDBOX_CAMERA=0,120,2 cargo run   start the sandbox with camera at (0,120), zoom 2
# PURPLEPIE_LOG=info cargo run    (PowerShell: $env:PURPLEPIE_LOG="info"; cargo run) shows GPU details
cargo test -- --ignored   # GPU-dependent tests (need a GPU or software Vulkan)
cargo test
cargo fmt --check && cargo clippy --all-targets
```

**Shipping a game:** copy the `assets/` folder next to the executable (for example
`target/release/assets/`). PurplePie looks there first, then in `assets/` in the working directory,
which is the project folder under `cargo run`. `EngineConfig::with_asset_root(path)` sets the folder explicitly.

CI: `.github/workflows/ci.yml` runs fmt, clippy, check and tests (Linux, Windows, macOS) on every push. All jobs passed on the first run (2026-10-01).

## Layout

```text
PurplePie/
├── Cargo.toml / Cargo.lock
├── LICENSE-MIT / LICENSE-APACHE
├── src/lib.rs        engine library (modules arrive stage by stage)
├── src/main.rs       `sandbox` binary: a game using only the public API
├── examples/         breakout.rs: a complete game on the public API; scene.rs: scene files
├── assets/           textures/ (sandbox_quadrants.png, sandbox_sheet.png, breakout/), fonts/ (Poppins-Regular.ttf + OFL.txt), sounds/ (blip, hit, lose, loop .wav; loop.ogg), scenes/ (demo.ron), shaders/
├── .github/workflows/ CI (fmt, clippy, check, test)
└── docs/
    ├── PROJECT_STATUS.md where we are, what works, validation log   ← start here
    ├── TASKS.md          task tracker (PP-xxx), the single next task
    ├── DEVELOPMENT.md    working protocol, Definition of Done, env setup, archives
    ├── ARCHITECTURE.md   modules, dependency rules, runtime flow (current vs planned)
    ├── DECISIONS.md      ADR log (ADR-001…) + pending decisions
    ├── ROADMAP.md        Stages 0–10, milestones M0–M10
    ├── RISKS.md          technical risks
    ├── TECH_STACK.md     verified versions and API notes
    └── spikes/           Stage 0 compatibility spike (reference only)
```

Windows needs the Visual Studio **"Desktop development with C++"** workload for
the MSVC linker. See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#9-environment-setup).

## Stack

| Crate | Version | Stage |
|---|---|---|
| winit | 0.30.13 | 1 |
| thiserror | 2.0 | 1 |
| hecs | 0.11.1 | 3 |
| glam | 0.33 | 3 |
| wgpu | 30.0.1 | 4 |
| pollster | 1.0.1 | 4 |
| bytemuck | 1.25 | 5 |
| log | 0.4 | 4 |
| image (PNG only) | 0.25.10 | 6 |
| ab_glyph | 0.2.32 | PP-018a |
| cpal | 0.18.2 | PP-022 |
| hound | 3.5.1 | PP-022 |
| lewton | 0.10.2 | PP-024b |
| ron | 0.12.2 | PP-026a |
| serde | 1.0.229 | PP-026a |

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Assets keep their own licences: `assets/fonts/Poppins-Regular.ttf` is under the SIL Open Font License 1.1
([assets/fonts/OFL.txt](assets/fonts/OFL.txt)); include that file if you ship the font with a game.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

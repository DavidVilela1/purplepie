# PurplePie

A small, modular, cross-platform **2D game engine** written in Rust, built on
`winit`, `wgpu`, `hecs` and `glam`.

> **Status: Stage 5 (first 2D primitive) complete.** `cargo run` opens a window
> where the engine draws ECS entities as coloured quads with wgpu: three reference
> shapes on PurplePie purple, one of them moving. Game logic runs in a 60 Hz
> fixed-timestep `fixed_update`. GPU failures end the game with a clean error. It is verified on
> Linux. Sprites/textures are next (Stage 6).
> Current state: [docs/PROJECT_STATUS.md](docs/PROJECT_STATUS.md). Next task: [docs/TASKS.md](docs/TASKS.md).

## Design in one paragraph (target; see PROJECT_STATUS for what exists)

The engine is the `purplepie` library. Games are separate binaries (starting
with `sandbox`) that implement a small `Game` trait and receive a `Context`
containing the ECS `World`, `Time` and `Input`. The engine owns the event loop,
window and GPU, runs a fixed 60 Hz simulation step plus a per-frame update, and
renders entities that carry `Transform2D` + `Sprite`. Game code never touches
`wgpu` or `winit`.

```rust
// Works today (Stage 5). Coordinates: +X right, +Y up, origin at the window
// centre, 1 unit = 1 logical pixel (ADR-018).
use purplepie::ecs::{self, Velocity};
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Color, Quad};
use purplepie::{Context, Engine, EngineConfig, Game};

struct Sandbox;

impl Game for Sandbox {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        ctx.world_mut().spawn((
            Transform2D::from_position(Vec2::new(0.0, 100.0)),
            Quad::new(Vec2::new(64.0, 64.0), Color::WHITE),
            Velocity(Vec2::new(50.0, 0.0)), // 50 logical px per second
        ));
        Ok(())
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        let dt = ctx.dt(); // fixed step: 1/60 s by default
        ecs::integrate_velocity(ctx.world_mut(), dt);
    }
}

fn main() -> purplepie::Result<()> {
    let config = EngineConfig::new("Sandbox").with_size(1280, 720);
    // .with_clear_color(Color::hex(0x202030)) to change the background
    Engine::new(config)?.run(Sandbox)
}
```

## Build

Requires Rust stable (edition 2024; `rust-version = 1.90`, developed with 1.95).

```bash
cargo run            # opens the sandbox window (Escape or close to quit)
# PURPLEPIE_LOG=info cargo run    (PowerShell: $env:PURPLEPIE_LOG="info"; cargo run) shows GPU details
cargo test -- --ignored   # GPU-dependent tests (need a GPU or software Vulkan)
cargo test
cargo fmt --check && cargo clippy --all-targets
```

CI: `.github/workflows/ci.yml` runs fmt, clippy, check and tests (Linux, Windows, macOS) on every push.

## Layout

```text
PurplePie/
├── Cargo.toml / Cargo.lock
├── src/lib.rs        engine library (modules arrive stage by stage)
├── src/main.rs       `sandbox` binary: a game using only the public API
├── assets/           textures/, fonts/, shaders/
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

## License

Not chosen yet. The project owner should decide (MIT OR Apache-2.0 is the Rust ecosystem norm).

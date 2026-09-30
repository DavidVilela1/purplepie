# PurplePie

A small, modular, cross-platform **2D game engine** written in Rust, built on
`winit`, `wgpu`, `hecs` and `glam`.

> **Status: Milestone M0 (Architecture Ready) verified. Stage 0 complete.**
> The repository contains the architecture, roadmap, decision log and task
> tracker, plus a compiling scaffold. There is no window or renderer yet.
> Current state: [docs/PROJECT_STATUS.md](docs/PROJECT_STATUS.md). Next task: [docs/TASKS.md](docs/TASKS.md).

## Design in one paragraph

The engine is the `purplepie` library. Games are separate binaries (starting
with `sandbox`) that implement a small `Game` trait and receive a `Context`
containing the ECS `World`, `Time` and `Input`. The engine owns the event loop,
window and GPU, runs a fixed 60 Hz simulation step plus a per-frame update, and
renders entities that carry `Transform2D` + `Sprite`. Game code never touches
`wgpu` or `winit`.

```rust
// Target API (Stage 1–10). Not implemented yet.
fn main() -> purplepie::Result<()> {
    let engine = Engine::new(EngineConfig::new("Sandbox").with_size(1280, 720))?;
    engine.run(Sandbox::default())
}
```

## Build

Requires Rust stable (edition 2024; `rust-version = 1.90`, developed with 1.95).

```bash
cargo run            # runs the sandbox binary
cargo test
cargo fmt --check && cargo clippy --all-targets
```

## Layout

```text
PurplePie/
├── Cargo.toml / Cargo.lock
├── src/lib.rs        engine library (modules arrive stage by stage)
├── src/main.rs       `sandbox` binary: a game using only the public API
├── assets/           textures/, fonts/, shaders/
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

# PurplePie

A small, modular, cross-platform **2D game engine** written in Rust, built on
`winit`, `wgpu`, `hecs` and `glam`.

> **Status: Stage 0 (Architecture & Planning) complete.** The repository
> contains the architecture, roadmap and decisions, plus a compiling scaffold.
> There is no window or renderer yet. See [docs/STATUS.md](docs/STATUS.md).

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
    ├── ARCHITECTURE.md   modules, dependency rules, ownership, frame lifecycle
    ├── TECH_STACK.md     verified versions and API notes
    ├── ROADMAP.md        Stages 0–10
    ├── STATUS.md         current state and validation log
    ├── RISKS.md          architectural risks
    ├── adr/              architecture decision records
    └── spikes/           Stage 0 compatibility spike (reference only)
```

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

# Project Status

| | |
|---|---|
| **Current stage** | Stage 0: Architecture & Planning. **Complete** |
| **Date** | 2026-09-30 |
| **Next stage** | Stage 1: Minimal Application (not started; begins on request) |
| **Archive** | `PurplePie-stage-0.zip` |

## Stage 0 deliverables

- [x] Workspace inspected. It was empty, so the project was created fresh as `PurplePie/`.
- [x] Crate versions researched on crates.io and verified against crate source ([TECH_STACK.md](TECH_STACK.md)).
- [x] Compatibility spike compiled, built and ran headless ([spikes/](spikes/stage-0-compat-spike.md)).
- [x] ECS chosen: **hecs 0.11.1** ([ADR-0002](adr/0002-ecs-hecs.md)).
- [x] Architecture: modules, dependency direction, ownership, frame lifecycle, rendering and error policy ([ARCHITECTURE.md](ARCHITECTURE.md)).
- [x] Target public API: `Engine` / `EngineConfig` / `Game` / `Context` ([ADR-0004](adr/0004-engine-game-api.md)).
- [x] Staged roadmap, Stages 0–10 ([ROADMAP.md](ROADMAP.md)).
- [x] Risks ([RISKS.md](RISKS.md)) and ADRs 0001–0010 ([adr/](adr/README.md)).
- [x] Scaffold: `purplepie` lib + `sandbox` bin, **no dependencies**, lints (`unsafe_code = forbid`, `clippy::unwrap_used = warn`), `assets/{textures,fonts,shaders}/`.

## Validation log (Stage 0, executed in the Cowork sandbox, Rust 1.95.0, Linux x86_64)

| Command | Result |
|---|---|
| `cargo fmt --check` | ✅ exit 0 |
| `cargo check --all-targets` | ✅ exit 0 |
| `cargo clippy --all-targets -- -D warnings` | ✅ exit 0 |
| `cargo test` | ✅ 1 unit test + 1 doctest passed |
| `cargo build` | ✅ exit 0 |
| `cargo doc --no-deps` | ✅ exit 0 |
| `cargo run` | ✅ prints `PurplePie sandbox v0.0.0 (Stage 0 scaffold …)` |

Not validated: Windows, macOS, Wayland, real GPUs, and toolchains other than 1.95.0.

## Known open decisions

| Decision | Stage |
|---|---|
| Whether the sandbox installs `env_logger` | 1 |
| Brand purple value and sRGB conversion ([ADR-0008](adr/0008-color-space.md)) | 4–5 |
| `bytemuck` / `image` versions and features | 5–6 |
| Coordinate system ([ADR-0009](adr/0009-coordinate-system.md)) | 7 |
| Input edge semantics ([ADR-0010](adr/0010-input-model.md)) | 8 |
| License | owner |

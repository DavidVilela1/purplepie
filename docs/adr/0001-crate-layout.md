# ADR-0001: Single package with an engine library and a sandbox binary

**Status:** Accepted (Stage 0). Revisit in Stage 10.

## Context
Engine code and game code must stay separate. Options:
1. One binary crate with `main.rs` + modules. Nothing stops game code from reaching engine internals.
2. One package with a `lib` target (engine) and a `bin` target (game).
3. A Cargo workspace with several crates (`purplepie_core`, `purplepie_render`, …).

## Decision
Option 2. Package `purplepie`: `src/lib.rs` is the engine and `src/main.rs` is the
`sandbox` binary. The binary is named `sandbox` to avoid the lib/bin name
collision in `cargo doc`.

## Consequences
* The compiler enforces the boundary: `main.rs` sees only `pub` engine items.
* One `Cargo.toml` and one lockfile. Nothing to coordinate between crates.
* Internal visibility (`pub(crate)`) hides wgpu/winit details.
* Later, more demo games go in `examples/`.
* A workspace split is deferred until build times or reuse justify it. The
  module dependency rules (ARCHITECTURE §4) already match a future crate split.

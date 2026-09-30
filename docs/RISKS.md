# PurplePie Technical Risks

Last reviewed: 2026-09-30 (end of Stage 0).

**Status:** `OPEN` (could happen), `MONITORING` (watched at a known trigger),
`MATERIALIZED` (happening now), `MITIGATED` (handled, may recur), `CLOSED`.
Likelihood and impact are qualitative: Low, Medium or High.

## Summary

| ID | Risk | Status | Likelihood | Impact | Stages |
|---|---|---|---|---|---|
| R-01 | wgpu API evolution | MONITORING | High | High | 4+ |
| R-02 | winit lifecycle changes | MONITORING | Medium | Medium | 1+ |
| R-03 | GPU API compatibility and outdated examples | OPEN | High | Medium | 1–7 |
| R-04 | Rust ownership/borrowing constraints in `Context` | OPEN | Medium | Medium | 1, 3, 10 |
| R-05 | Input edges under fixed timestep | OPEN | High | Medium | 8 |
| R-06 | Frame pacing / busy loop | OPEN | High | Low | 1–4 |
| R-07 | Spiral of death | OPEN | Medium | Medium | 2 |
| R-08 | Color-space errors | OPEN | High | Low | 4–6 |
| R-09 | Surface/device loss handling | OPEN | Medium | Medium | 4 |
| R-10 | Renderer overengineering / scope creep | OPEN | Medium | High | all |
| R-11 | Headless-only validation in Cowork | MATERIALIZED | High | Medium | 1+ |
| R-12 | Cross-platform window behavior | OPEN | Medium | Medium | 1, 4, 7 |
| R-13 | Future public API stability | OPEN | Medium | Medium | 10 |
| R-14 | Windows build environment (MSVC linker) | MITIGATED | — | High | 0 |
| R-15 | Project inside OneDrive | MATERIALIZED | Medium | Medium | 1+ |
| R-16 | Documentation drift | OPEN | Medium | Medium | all |
| R-17 | Resource and asset lifetime management | OPEN | Medium | Medium | 6, 9 |
| R-18 | Performance: per-frame allocations and draw calls | OPEN | Medium | Low | 5–6 |
| R-19 | Declared MSRV untested | OPEN | Low | Low | all |
| R-20 | Compile-time growth in a single crate | OPEN | High | Low | 4, 10 |
| R-21 | ECS integration complexity (no resources/scheduler in hecs) | OPEN | Low | Medium | 3 |

## Details

### R-01: wgpu API evolution
- **Trigger:** A new wgpu major release (roughly quarterly; 26 → 30 in about 14 months). In wgpu 30, acquire/present and instance creation changed.
- **Mitigation:** Pin via `Cargo.lock`. Confine wgpu to `render` (`pub(crate)`). Upgrade deliberately, one upgrade at a time.
- **Fallback:** Stay on 30.x until a quiet period. The public API does not expose wgpu, so games are unaffected.

### R-02: winit lifecycle changes
- **Trigger:** winit 0.31 stable is released (currently 0.31.0-beta.3). It changes the window/app model.
- **Mitigation:** winit only in `app` (ADR-003/004). Follow the 0.30 `ApplicationHandler` rules exactly (windows created in `resumed`).
- **Fallback:** Remain on 0.30.x, which is still maintained (0.30.13 released 2026-03).

### R-03: GPU API compatibility and outdated examples
- **Trigger:** Copying tutorial code written for wgpu ≤ 27 or winit ≤ 0.29.
- **Mitigation:** The Stage 0 spike and [TECH_STACK.md](TECH_STACK.md) are the reference. Verify against crate source before use.
- **Fallback:** Rebuild the spike for the exact API in question before touching `src/`.

### R-04: Rust ownership/borrowing constraints in `Context`
- **Trigger:** The game needs `&mut World` while the engine also needs a borrow of the same state.
- **Mitigation:** The renderer is never in `Context`. Build `Context` from disjoint field borrows. Rendering reads the world after updates.
- **Fallback:** Split the runner state into a sub-struct and borrow it wholesale. Do not introduce `RefCell`.

### R-05: Input edges under fixed timestep
- **Trigger:** A frame with 0 fixed steps misses `just_pressed`, or a frame with 2+ steps sees it twice.
- **Mitigation:** PD-03: latch edges until the first fixed step consumes them. `update()` always sees the frame's edges.
- **Fallback:** Document that edge checks belong in `update()` only.

### R-06: Frame pacing / busy loop
- **Trigger:** `ControlFlow::Poll` without a vsync swapchain (Stages 1–3) spins at 100% CPU.
- **Mitigation:** Stages 1–3 use `ControlFlow::WaitUntil(next frame)`. From Stage 4, `Fifo` present mode paces frames.
- **Fallback:** A frame-rate cap in `EngineConfig`.

### R-07: Spiral of death
- **Trigger:** Long frames (debugger, window drag, slow machine) cause ever more fixed steps.
- **Mitigation:** `MAX_FRAME_DT = 0.25 s`, `MAX_FIXED_STEPS = 5`, backlog clamp (ADR-010). Unit-tested in Stage 2.
- **Fallback:** Lower the caps via `EngineConfig`.

### R-08: Color-space errors
- **Trigger:** An sRGB surface encodes linear values. The spike showed intended `#591A8C` displayed as `#A059C4`.
- **Mitigation:** PD-01: public colors are sRGB and converted in one place. Textures use sRGB formats.
- **Fallback:** Choose a non-sRGB surface format explicitly.

### R-09: Surface/device loss handling
- **Trigger:** `CurrentSurfaceTexture::Lost`, GPU reset, or driver update. In wgpu 30, OOM and device loss arrive through callbacks, not through the acquire result.
- **Mitigation:** Recreate the surface on `Lost`. Capture `on_uncaptured_error` and `set_device_lost_callback` and turn them into `Error::Render`.
- **Fallback:** Exit cleanly with a descriptive error. Full device recreation is deferred.

### R-10: Renderer overengineering / scope creep
- **Trigger:** Adding render graphs, plugin systems or generic resource registries before a stage needs them.
- **Mitigation:** Priority order (ARCHITECTURE §2). Every new abstraction needs a current user. Structural changes need an ADR.
- **Fallback:** Remove unused abstractions during stage review.

### R-11: Headless-only validation in Cowork
- **Trigger:** Now. Cowork runs Linux without a GPU or display. Only Xvfb + Mesa lavapipe is available, which is proven to work.
- **Mitigation:** Smoke runs under Xvfb with pixel checks. The owner runs every windowed stage on Windows with a real GPU and reports the result.
- **Fallback:** Mark platform behavior as unverified in PROJECT_STATUS until the owner confirms it.

### R-12: Cross-platform window behavior
- **Trigger:** DPI scaling, Wayland vs X11, macOS main-thread rules, minimize reporting a 0×0 size.
- **Mitigation:** No platform-specific code. Physical sizes for the surface. Never configure 0×0.
- **Fallback:** Platform-specific workarounds behind `cfg`, each documented.

### R-13: Future public API stability
- **Trigger:** Stage 10 review finds `Game`/`Context` awkward in a real game.
- **Mitigation:** Version stays 0.x. API refinement is a planned stage. Changes are recorded in DECISIONS.
- **Fallback:** Accept breaking changes before 1.0.

### R-14: Windows build environment (MSVC linker)
- **Trigger (occurred 2026-09-30):** `cargo test` failed with `linker link.exe not found` on the owner's Windows laptop. `check` and `clippy` passed because they do not link.
- **Mitigation:** Install the Visual Studio "Desktop development with C++" workload. After that the owner's `cargo run` succeeded. Documented in [DEVELOPMENT.md](DEVELOPMENT.md#9-environment-setup).
- **Fallback:** Use "Developer PowerShell for VS". The GNU toolchain is not recommended.

### R-15: Project inside OneDrive
- **Trigger:** Now. The owner's copy is under `OneDrive\Ambiente de Trabalho\…`. Syncing `target/` (hundreds of MB from Stage 4 onward) causes slow builds and "Access denied" errors.
- **Mitigation:** Move the project outside OneDrive, or set `CARGO_TARGET_DIR` outside it.
- **Fallback:** Pause OneDrive sync while building.

### R-16: Documentation drift
- **Trigger:** Code changes without doc updates. Known duplication: the module table in `src/lib.rs` mirrors ARCHITECTURE §3.
- **Mitigation:** The DoD includes doc updates. Repository and validation outrank docs. The per-session start checklist is in DEVELOPMENT.md.
- **Fallback:** A full doc-vs-code review at each stage boundary.

### R-17: Resource and asset lifetime management
- **Trigger:** Textures referenced by components outlive or predate their GPU upload.
- **Mitigation:** Components hold handles, not GPU objects (ADR-009). The design is decided in PD-06.
- **Fallback:** Reference counting inside the asset store only.

### R-18: Performance: per-frame allocations and draw calls
- **Trigger:** Rebuilding vertex buffers or `Vec`s every frame, or one draw call per sprite.
- **Mitigation:** Reuse buffers and instanced batching (PD-05). Measure before optimizing.
- **Fallback:** Profile with a sprite-count benchmark in `examples/`.

### R-19: Declared MSRV untested
- **Trigger:** A user builds with Rust 1.90–1.94. `rust-version = "1.90"` comes from dependency metadata, and only 1.95 was run.
- **Mitigation:** Documented in TECH_STACK.
- **Fallback:** Raise `rust-version` to the lowest version actually verified.

### R-20: Compile-time growth in a single crate
- **Trigger:** wgpu is added (the spike took about 1m21s for a clean debug build in Cowork).
- **Mitigation:** Incremental builds. Measure at Stage 10.
- **Fallback:** Workspace split (ADR-002 revisit).

### R-21: ECS integration complexity
- **Trigger:** A need for resources, events or change detection that hecs lacks.
- **Mitigation:** Resources live in `Context`. Events are plain `Vec`s owned by the engine or game.
- **Fallback:** Revisit ADR-006.

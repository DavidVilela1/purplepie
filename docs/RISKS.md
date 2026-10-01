# PurplePie Technical Risks

Last reviewed: 2026-10-01 (Stage 6 complete, PP-015).

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
| R-06 | Frame pacing / busy loop | MITIGATED | — | Low | all |
| R-07 | Spiral of death | MITIGATED | — | Medium | 2 |
| R-08 | Color-space errors | MITIGATED | — | Low | 4–6 |
| R-09 | Surface/device loss and uncaptured GPU errors | MITIGATED | — | Medium | 4+ |
| R-10 | Renderer overengineering / scope creep | OPEN | Medium | High | all |
| R-11 | Headless-only validation in Cowork | MATERIALIZED | High | Medium | 1+ |
| R-12 | Cross-platform window behavior | OPEN | Medium | Medium | 1, 4, 7 |
| R-13 | Future public API stability | OPEN | Medium | Medium | 10 |
| R-14 | Windows build environment (MSVC linker) | MITIGATED | — | High | 0 |
| R-15 | Project inside OneDrive | MATERIALIZED | Medium | Medium | 1+ |
| R-16 | Documentation drift | MONITORING | Medium | Medium | all |
| R-17 | Resource and asset lifetime management | MONITORING | Medium | Medium | 6, 9 |
| R-18 | Performance: per-frame allocations and draw calls | MONITORING | Low | Low | 5–6 |
| R-19 | Declared MSRV untested | OPEN | Low | Low | all |
| R-20 | Compile-time growth in a single crate | OPEN | High | Low | 4, 10 |
| R-21 | ECS integration complexity (no resources/scheduler in hecs) | OPEN | Low | Medium | 3 |
| R-22 | Panics inside wgpu/wgpu-hal that PurplePie cannot intercept | MONITORING | Low | High | 4+ |
| R-23 | CI platform jobs never exercised yet | OPEN | Medium | Low | all |
| R-24 | Asset paths depend on the working directory | OPEN | Medium | Low | 6, 9 |

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
- **Observed (PP-005):** `integrate_velocity(ctx.world_mut(), ctx.dt())` does not compile, because the context is borrowed twice. The documented pattern is `let dt = ctx.dt();` first. This is acceptable ergonomics. Revisit in Stage 10 (ADR-008).
- **Fallback:** Split the runner state into a sub-struct and borrow it wholesale. Do not introduce `RefCell`.

### R-05: Input edges under fixed timestep
- **Trigger:** A frame with 0 fixed steps misses `just_pressed`, or a frame with 2+ steps sees it twice.
- **Mitigation:** PD-03: latch edges until the first fixed step consumes them. `update()` always sees the frame's edges.
- **Fallback:** Document that edge checks belong in `update()` only.

### R-06: Frame pacing / busy loop
- **Trigger:** A loop without a working throttle spins at 100% CPU.
- **Evidence (PP-006):** with a swapchain, `Fifo` + `Poll` still ran at **544 fps** under Xvfb + lavapipe. The default present mode there was `Immediate`, and `Occluded` returns immediately.
- **Mitigation (ADR-014):** keep `FramePacer` + `ControlFlow::WaitUntil` (60 Hz cap) and use `PresentMode::AutoVsync`. Measured: 300 frames in 5.1 s. Remaining idle CPU under lavapipe (~0.75 s per 3 s) is software GPU work, not spinning.
- **Fallback:** Lower the cap, or pause rendering while unfocused.

### R-07: Spiral of death
- **Trigger:** Long frames (debugger, window drag, slow machine) cause ever more fixed steps.
- **Mitigation (implemented in PP-004):** `max_frame_dt = 0.25 s`, `max_fixed_steps = 5`, backlog clamp (ADR-010). Unit-tested. A 1 s stall measured under Xvfb gave 5 steps and no burst afterwards.
- **Fallback:** Lower the caps via `EngineConfig`.

### R-08: Color-space errors
- **Trigger:** An sRGB surface encodes linear values, so the Stage 0 spike showed intended `#591A8C` as `#A059C4`.
- **Mitigation (ADR-015, implemented):** public colors are sRGB and converted once, per target format. Verified pixel-exact (`#6A0DAD`, 921,600/921,600 px). Textures (PP-008, ADR-020) use `Rgba8UnormSrgb` on sRGB surfaces and `Rgba8Unorm` otherwise, so they blend in the same space as quad colours; a 50% alpha texel matched the linear-space blend exactly.
- **Fallback:** Choose a non-sRGB surface format explicitly.

### R-09: Surface/device loss and uncaptured GPU errors
- **Trigger:** `CurrentSurfaceTexture::Lost`, a GPU reset or driver update, or any wgpu validation, OOM or internal error.
- **Mitigation (PP-014, ADR-017):** `FaultSlot` replaces wgpu's panicking uncaptured-error handler and captures device loss. All of these become `Error::Render` and a clean exit. Verified with the ignored GPU test, a control run (panics without the handler) and an end-to-end window-destroy test (exit 1, readable cause).
- **Residual:** no recovery. A transient `Lost` on a live window ends the game (ADR-017 revisit condition).
- **Fallback:** Recover from `Lost` only when the window is known to still exist, if real hardware shows transient losses.

### R-10: Renderer overengineering / scope creep
- **Trigger:** Adding render graphs, plugin systems or generic resource registries before a stage needs them.
- **Mitigation:** Priority order (ARCHITECTURE §2). Every new abstraction needs a current user. Structural changes need an ADR.
- **Fallback:** Remove unused abstractions during stage review.

### R-11: Headless-only validation in Cowork
- **Trigger:** Now. Cowork runs Linux without a GPU or display. Xvfb + Mesa lavapipe + xdotool is available, which is proven to work. On Windows with a real GPU, the owner confirmed the purple window by screenshot (pixel-checked). Vsync pacing and the GPU error paths are unverified on real hardware.
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
- **Trigger:** Code changes without doc updates.
- **Evidence:** PP-003 found one real drift. ADR-011/TECH_STACK claimed winit depends on `log`, but it uses `tracing`. Now corrected. The duplicated module table in `src/lib.rs` was removed.
- **Mitigation:** The DoD includes doc updates. Repository and validation outrank docs. The per-session start checklist is in DEVELOPMENT.md, and each loop run ends with a drift check.
- **Fallback:** A full doc-vs-code review at each stage boundary.

### R-17: Resource and asset lifetime management
- **Trigger:** Textures referenced by components outlive or predate their GPU upload.
- **Mitigation (PP-008, ADR-020):** components hold a plain `TextureId`, never GPU objects. The CPU store is the source of truth and outlives renderers, so a recreated renderer re-uploads everything; uploads happen before each frame, so a texture is never drawn before it exists. The rest (unloading, other asset kinds) is PD-06.
- **Remaining:** nothing is ever unloaded, and every texture keeps a CPU copy, so memory grows with every distinct texture loaded. Fine for small games; it matters for large or streamed content.
- **Fallback:** Reference counting inside the asset store only, or dropping CPU copies once uploaded (and reloading from disk on renderer recreation).

### R-18: Performance: per-frame allocations and draw calls
- **Trigger:** Rebuilding vertex buffers or `Vec`s every frame, or one draw call per sprite.
- **Mitigation (PP-007):** all quads go in one instanced draw call. The instance `Vec` is reused every frame, and the GPU buffer grows by powers of two only. The remaining per-frame cost is one matrix product per quad on the CPU (ADR-019).
- **PP-008:** sprites reuse the same scheme (reused `Vec`s, growable buffer). Consecutive sprites with the same texture share one draw call, but sprites alternating between textures in query order get one draw call each until PP-015 sorts by texture.
- **PP-015 (ADR-021):** one draw list sorted by (layer, material, entity) → one draw call per (layer, texture) run, one shared instance buffer, all `Vec`s reused. New cost: an O(n log n) sort every frame. Measured (release, Cowork CPU): ~0.08 ms / 1k, ~1.8 ms / 10k, ~7.9 ms / 50k drawables.
- **Fallback:** Move the model matrix to the GPU (uniform view-projection) if profiling shows the CPU cost matters; skip the sort when nothing changed. Add a sprite-count benchmark in `examples/` (PP-012).

### R-19: Declared MSRV untested
- **Trigger:** A user builds with Rust 1.90–1.94. `rust-version = "1.90"` comes from dependency metadata, and only 1.95 was run.
- **Mitigation:** Documented in TECH_STACK.
- **Fallback:** Raise `rust-version` to the lowest version actually verified.

### R-20: Compile-time growth in a single crate
- **Trigger:** wgpu was added in PP-006, bringing the tree to 116 unique normal dependencies on Linux (128 after `image` in PP-008). The spike took about 1m21s for a clean debug build in Cowork.
- **Mitigation:** Incremental builds. Measure at Stage 10.
- **Fallback:** Workspace split (ADR-002 revisit).

### R-21: ECS integration complexity
- **Trigger:** A need for resources, events or change detection that hecs lacks.
- **Mitigation:** Resources live in `Context`. Events are plain `Vec`s owned by the engine or game.
- **Fallback:** Revisit ADR-006.

### R-22: Panics inside wgpu/wgpu-hal that PurplePie cannot intercept
- **Trigger (occurred, PP-014):** recreating a surface for a destroyed X11 window panicked in wgpu-hal 30.0.1 (`vulkan/instance.rs:407`, an `expect`), not returning an error.
- **Mitigation:** avoid the triggering call (`Lost` is fatal, ADR-017). Keep the end-to-end fault tests (DEVELOPMENT §8) and re-run them after every wgpu upgrade (R-01).
- **Fallback:** Report upstream, or pin to a wgpu version without the panic.

### R-23: CI platform jobs never exercised yet
- **Trigger:** `.github/workflows/ci.yml` was added on 2026-09-30, but no run has happened yet. The Windows and macOS jobs have never executed anywhere.
- **Mitigation:** the owner's first push runs it. Record the results in PROJECT_STATUS. The workflow passed actionlint + shellcheck, and its exact commands passed locally on Linux.
- **Fallback:** Remove `macos-latest` from the matrix if runner minutes or macOS-specific failures become a burden.

### R-24: Asset paths depend on the working directory
- **Trigger:** `Context::load_texture("assets/...")` resolves relative paths against the process's current directory, so a game started from another folder (a shortcut, a double-click on the `.exe`, `cargo run` from a subfolder) cannot find its files.
- **Mitigation (PP-008):** the error is a clear `Error::Asset` naming the path, never a panic. The sandbox builds an absolute path from `CARGO_MANIFEST_DIR` at compile time.
- **Fallback:** decide an asset root in PD-06 / PP-011 (e.g. next to the executable, overridable in `EngineConfig`).

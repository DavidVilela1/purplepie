# Architectural Risks

| ID | Risk | Impact | Mitigation | Stage |
|---|---|---|---|---|
| R-01 | **wgpu churn.** Breaking release every ~3 months (26 → 30 in ~14 months); acquire/present/instance APIs changed recently. | High | Pin exact minor versions. Confine all wgpu use to `render` (`pub(crate)`). Upgrade deliberately, one ADR per upgrade. | 4+ |
| R-02 | **winit 0.31 migration.** 0.31 (beta) changes the application/window model. | Medium | Keep winit inside `app` only. Game code never sees winit types, so migration touches one module. | 1+ |
| R-03 | **Outdated tutorials.** Most online examples target wgpu ≤ 27 / winit ≤ 0.29. | Medium | Treat the Stage 0 spike and [TECH_STACK.md](TECH_STACK.md) as the API reference. Verify against crate source before use. | 1–7 |
| R-04 | **`Context` borrow conflicts.** Game wants `&mut World` and the renderer at once. | Medium | Renderer is never in `Context`. Rendering reads the `World` after updates. Disjoint field borrows in `Runner`. | 1, 10 |
| R-05 | **Fixed-step edge input.** `just_pressed` can be missed (0 fixed steps this frame) or seen twice (2+ steps). | Medium | ADR-0010: latch edges until consumed by the first fixed step, and expose frame-rate `update` for UI-style input. | 8 |
| R-06 | **CPU spin before vsync exists.** `ControlFlow::Poll` without a GPU swapchain busy-loops in Stages 1–3. | Low | Stages 1–3 use `ControlFlow::WaitUntil(next_frame)`. From Stage 4, `Fifo` present mode paces frames. | 1–4 |
| R-07 | **Spiral of death.** Long frames trigger ever more fixed steps. | Medium | `MAX_FRAME_DT` clamp + `MAX_FIXED_STEPS` + backlog drop (ADR-0005). | 2 |
| R-08 | **Color-space mistakes.** sRGB surface makes linear clear colors look lighter (seen in spike). | Low | ADR-0008: public colors are sRGB and are converted in one place. | 4–5 |
| R-09 | **Surface loss / device loss** not reported through the acquire result in wgpu 30. | Medium | Handle `Lost` by recreating the surface. Capture device-lost and uncaptured errors via callbacks and turn them into `Error::Render`. | 4 |
| R-10 | **Scope creep toward a framework** (schedulers, plugins, resources registry). | High | Priority order in ARCHITECTURE §1. Each new abstraction needs a concrete current use and, if structural, an ADR. | all |
| R-11 | **Headless validation limits.** Only Linux + Xvfb + lavapipe can run here. No Windows/macOS/real GPU. | Medium | Keep platform-specific code at zero. The user runs manual smoke tests on real hardware per rendering stage. | 1+ |
| R-12 | **MSRV claim untested.** `rust-version = 1.90` comes from metadata. Only 1.95 was run. | Low | Documented. Adjust if a lower toolchain fails. | all |
| R-13 | **Single crate compile time** grows once wgpu is in. | Low | Split into a workspace (e.g. `purplepie_render`) only when measured build times justify it (ADR-0001). | 10 |

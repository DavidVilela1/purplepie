# ADR-0005: Fixed-timestep loop driven by `RedrawRequested`

**Status:** Accepted (Stage 0). Implement in Stages 1–2.

## Context
Gameplay must be deterministic-ish and frame-rate independent. Rendering must
run as fast as the display allows. winit 0.30 recommends drawing in
`WindowEvent::RedrawRequested` and requesting redraws from `about_to_wait`.

## Decision
* One whole frame (tick time → 0..N fixed updates → variable update →
  render → end input frame) runs inside `RedrawRequested`.
* `about_to_wait` calls `window.request_redraw()`.
* Constants (configurable through `EngineConfig`):
  `FIXED_DT = 1/60 s` (f64), `MAX_FRAME_DT = 0.25 s`, `MAX_FIXED_STEPS = 5`.
* When the step cap is hit, the remaining backlog is clamped to less than one step,
  so simulation slows down instead of spiralling.
* `alpha = accumulator / FIXED_DT` is exposed for future interpolation. No
  interpolation is implemented until it is needed.
* `FixedTimestep` is a pure struct with no winit/wgpu dependency, so it is unit-testable.
* Stages 1–3 (no swapchain yet) use `ControlFlow::WaitUntil(next frame)` to
  avoid a busy loop. Stage 4 switches to `Poll` + `Fifo` vsync pacing.

## Consequences
* Rendering and simulation rates are decoupled.
* Pausing in a debugger does not cause a burst of hundreds of updates.
* Edge-triggered input needs care with 0 or N fixed steps per frame (ADR-0010).

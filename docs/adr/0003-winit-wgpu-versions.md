# ADR-0003: winit 0.30.13 and wgpu 30.0.1, pinned

**Status:** Accepted (Stage 0)

## Context
Latest on 2026-09-30: `wgpu 30.0.1` (stable) and `winit 0.31.0-beta.3`
(pre-release; latest stable is `0.30.13`). Both depend on `raw-window-handle 0.6`,
which is how wgpu obtains a surface from a winit window.

## Decision
* `winit = "0.30.13"` (latest **stable**). Pre-releases are not used for the foundation.
* `wgpu = "30.0.1"`.
* Commit `Cargo.lock`. Upgrade either crate only as a deliberate change with its own note or ADR.

## Evidence
A throwaway spike compiled and ran with exactly these versions (plus hecs,
glam, pollster, thiserror): one `raw-window-handle` in the tree, purple clear
rendered under Xvfb + lavapipe. See [../spikes/stage-0-compat-spike.md](../spikes/stage-0-compat-spike.md).

## Consequences
* winit types stay inside `app` and wgpu types inside `render`, which limits
  the size of the future winit 0.31 and wgpu 31+ migrations.
* The wgpu 30 API differs from most tutorials (see TECH_STACK.md). The spike is the reference.

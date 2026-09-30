# Architecture Decision Records

Format: Context → Decision → Consequences. Status is one of **Accepted**,
**Proposed** (direction set, to be confirmed in the listed stage), or
**Superseded**.

| ADR | Title | Status | Decide/confirm in |
|---|---|---|---|
| [0001](0001-crate-layout.md) | Single package: engine library + sandbox binary | Accepted | 0 (revisit 10) |
| [0002](0002-ecs-hecs.md) | ECS library: `hecs` | Accepted | 0 |
| [0003](0003-winit-wgpu-versions.md) | winit 0.30.13 + wgpu 30.0.1, pinned | Accepted | 0 |
| [0004](0004-engine-game-api.md) | `Game` trait + per-call `Context`; engine owns world | Accepted | 0 (refine 10) |
| [0005](0005-game-loop.md) | Fixed-timestep loop driven by `RedrawRequested` | Accepted | 0 (implement 2) |
| [0006](0006-errors-and-logging.md) | One `thiserror` error enum, `log` facade, no `unwrap` | Accepted | 0 |
| [0007](0007-async-pollster.md) | `pollster::block_on` instead of an async runtime | Accepted | 0 |
| [0008](0008-color-space.md) | Public colors are sRGB | Proposed | 4–5 |
| [0009](0009-coordinate-system.md) | World units, +Y up, centered camera | Proposed | 7 |
| [0010](0010-input-model.md) | Engine-owned key types; edge latching for fixed steps | Proposed | 8 |

Still to write when the stage arrives: asset handle design (Stage 9),
sprite batching strategy (Stage 6), and a workspace split if one is needed (Stage 10).

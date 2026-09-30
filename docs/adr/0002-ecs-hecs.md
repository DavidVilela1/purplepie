# ADR-0002: Use `hecs` as the ECS

**Status:** Accepted (Stage 0)

## Context
Exactly one ECS library. Candidates: `hecs 0.11.1` and `bevy_ecs 0.19.1`
(0.20 is in RC). Measured on 2026-09-30:

| Criterion | hecs 0.11.1 | bevy_ecs 0.19.1 |
|---|---|---|
| Unique normal deps (incl. itself) | **4** | **76** |
| MSRV | 1.81 | 1.95 |
| Release cadence | 0.10 → 0.11 took ~2.5 years | 0.17 → 0.18 → 0.19 → 0.20-rc within ~11 months, each breaking |
| Scope | archetypal storage + queries only | storage, queries, scheduler, system params, resources, events, observers, change detection, reflection hooks |
| Fit with an explicit game loop | plain functions over `&mut World` | best used with its own `Schedule`/`App` model |
| Independence from Bevy | fully independent | part of the Bevy release train |
| Performance | archetypal, cache-friendly iteration; more than enough for 2D (10⁴–10⁵ entities) | archetypal/table storage with parallel scheduling |

## Decision
Use **hecs**. Wrap nothing: re-export `hecs::World` and `hecs::Entity` from
`purplepie::ecs` so games use them directly.

## Why
* **Simplicity and ergonomics:** `world.spawn((a, b))`,
  `world.query_mut::<(&mut Transform2D, &Velocity)>()`, and a system is just `fn(&mut World)`.
* **Maturity and stability:** small API surface, rare breaking releases. This
  matters because wgpu already brings frequent breaking releases (RISKS R-01).
* **Independence:** no scheduler or app model conflicts with PurplePie's own loop.
* **Scalability:** archetypal storage scales for a 2D engine. If parallel
  systems are ever needed, they can be added at the engine level.

## Consequences
* No built-in resources, events, change detection, or scheduler. Resources such
  as `Time`/`Input` live in `Context`. System order is explicit function-call
  order in game code, which matches the "don't hide behavior" principle.
* No `Commands` buffer. Structural changes during iteration use
  `hecs::CommandBuffer` or collect-then-apply.
* Switching ECS later would be expensive, since game code uses hecs types directly. That cost is accepted.

# ADR-0006: One `thiserror` error enum, `log` facade, no `unwrap`

**Status:** Accepted (Stage 0)

## Decision
* `purplepie::Error` is a single `#[derive(thiserror::Error)]` enum, with
  `purplepie::Result<T> = std::result::Result<T, Error>`.
* Variants follow failure domains: event loop, window, surface creation,
  adapter, device, unsupported surface, render (device lost / OOM / repeated
  validation), asset, and `Game(Box<dyn std::error::Error + Send + Sync>)`.
* Third-party errors are wrapped with `#[from]`/`#[source]`, so their messages are
  kept but their types are not part of the public API contract.
* winit callbacks cannot return errors. The runner stores the first error,
  calls `event_loop.exit()`, and `Engine::run` returns it.
* Lints: `unsafe_code = "forbid"` and `clippy::unwrap_used = "warn"`. `expect`
  is allowed only for invariants, with a message explaining why.
* Diagnostics use the `log` crate, which is already a transitive dependency of
  winit and wgpu. The engine never installs a logger. The sandbox may add
  `env_logger` (decided in Stage 1).

## Consequences
* One error type to match on for games. Its variants grow with the stages.
* No logging framework, spans or tracing until there is a demonstrated need.

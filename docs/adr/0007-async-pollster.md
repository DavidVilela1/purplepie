# ADR-0007: `pollster::block_on` instead of an async runtime

**Status:** Accepted (Stage 0). Used from Stage 4.

## Context
`Instance::request_adapter` and `Adapter::request_device` return futures. On
native backends they complete immediately or after a short driver call. They
do no I/O that benefits from a scheduler.

## Decision
Use `pollster 1.0.1` (`pollster::block_on(...)`) during renderer creation
inside `resumed`. No Tokio, async-std or smol.

## Consequences
* One tiny, dependency-free crate. Initialization stays synchronous and easy to follow.
* Blocking happens once at startup, or on surface recreation, on the main thread. That is acceptable.
* A web target would need a different initialization path, since blocking is not
  allowed in the browser. The web is a non-goal, so this is accepted.

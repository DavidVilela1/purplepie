# ADR-0010: Engine-owned key types; edge latching for fixed steps

**Status:** Proposed. Confirm in Stage 8.

## Context
Game code must not depend on winit (ADR-0004). Edge events (`just_pressed`)
interact badly with fixed steps: a frame may run 0 fixed steps (the edge is
missed) or several (the edge is seen twice).

## Proposal
* `purplepie::input::{KeyCode, MouseButton}` are PurplePie enums covering what
  2D games need (letters, digits, arrows, space, enter, escape, tab,
  shift/ctrl/alt, F1–F12, mouse L/R/M). `app` maps winit physical key codes
  to them, and unmapped keys are ignored.
* `Input` tracks `pressed` (level) and per-frame `just_pressed`/`just_released`.
* Fixed-step semantics: edges are **latched** until the first fixed step of the
  frame in which they are observed, and then cleared for later steps in the same
  frame. `update()` always sees the frame's edges.
* Text input and gamepads are out of scope until requested.

## Consequences
* A small mapping table to maintain, in exchange for a stable game API across winit upgrades.

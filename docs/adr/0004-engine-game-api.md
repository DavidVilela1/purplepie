# ADR-0004: `Game` trait with a per-call `Context`; the engine owns the world

**Status:** Accepted (Stage 0). Refine in Stage 10.

## Context
Game code needs to mutate ECS state and read time and input. It must not see
wgpu/winit. winit 0.30 drives the application through
`ApplicationHandler` callbacks on `&mut self`, so the engine must own one
struct that holds everything.

Options considered:
1. **Closures** (`engine.run(|ctx| ...)`). Awkward for game state, and
   multiple callbacks (init/fixed/update) need several closures sharing state.
2. **Game owns the world** and the engine borrows it for rendering. The renderer
   then depends on game structure, and the borrow gymnastics leak into game code.
3. **`Game` trait, engine owns `World`/`Time`/`Input`, and the game gets a `Context<'_>`.**
4. A Bevy-style plugin/scheduler app. This is the "mini-Bevy" the brief rules out.

## Decision
Option 3.

```rust
pub trait Game {
    fn init(&mut self, ctx: &mut Context<'_>) -> Result<()> { Ok(()) }
    fn fixed_update(&mut self, ctx: &mut Context<'_>);
    fn update(&mut self, ctx: &mut Context<'_>) {}
}
pub struct Context<'a> {
    pub world: &'a mut World,
    pub time: &'a Time,
    pub input: &'a Input,
    /* private: exit flag */
}
impl Engine {
    pub fn new(config: EngineConfig) -> Result<Engine>;   // creates the EventLoop
    pub fn run<G: Game>(self, game: G) -> Result<()>;     // consumes; blocks until exit
}
```

The fields fill in stage by stage: `world` arrives in Stage 3 and `input` in Stage 8.
Until then, `Context` only carries what exists.

## Consequences
* **Ownership:** `Runner<G>` owns `game: G` and the engine state. Static
  dispatch, no `Box<dyn Game>`, no `Rc<RefCell<_>>`.
* **Borrowing:** `Context` is built from disjoint field borrows while `game`
  is borrowed separately, so it is sound with zero interior mutability.
* **Rendering access:** none from the game. The renderer extracts
  `Transform2D + Sprite` after updates. Custom draw hooks, if ever needed,
  will take a restricted `DrawContext`, not wgpu.
* **Events:** `app` translates raw window events into `Input` state. The game
  polls state instead of receiving winit events.
* **Lifetime:** `Engine::run` consumes the engine. Returning from it means the
  window, surface and device were dropped in order.
* **Testability:** game logic can be unit-tested by building a `Context`
  from a headless `World`, `Time` and `Input`.
* `Engine::new` is fallible because `EventLoop::new()` can fail, and winit
  permits one event loop per process.

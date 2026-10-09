# Second game: Purple Swarm

P3.5 item 6 (PP-036). This is the second game written as an outside crate on PurplePie's public API. Breakout is a
reflex game, and the trial game Pie Catch was "move left/right, catch things". Purple Swarm is deliberately
different: a top-down arena shooter, where the player moves freely, aims with the mouse and fights waves of enemies.
It decides which engine features are actually missing. Every gap below is either written in game code or turned into
a small engine task.

## The game

- **Arena:** 1600 × 1200 units, larger than the 960 × 640 window. The camera follows the player and stops at the
  arena edges. The floor is a tiled pattern and the walls are solid quads.
- **Player:** moves with WASD or the arrows at 260 units/s. It always faces the mouse cursor. Holding the left button
  shoots bullets toward the cursor, at most 8 per second. The player has 5 hit points and blinks with half a second of
  invulnerability after each hit.
- **Enemies:** spawn at random points along the arena edge, never within 300 units of the player. They come in waves
  that grow every 20 s.
  - *Drifter:* slow, walks straight at the player, 1 hit point.
  - *Dasher:* pauses, then dashes in a straight line, 2 hit points.

  Touching the player costs 1 hit point.
- **Pickups:** a defeated enemy drops a health orb with a 10 % chance. The orb disappears after 8 s.
- **Score:** 10 points per Drifter and 25 per Dasher. The best score is kept in a small text file next to the game.
- **Screens:**
  - title ("Click to start", best score);
  - play (HUD with hit points, score, wave and time);
  - pause (Escape; Escape again or Q quits);
  - game over (score, best, "Click to play again").
- **Sound:**
  - a shot, a hit and an enemy death (short WAVs);
  - a looped OGG during play, stopped on game over.
- **Assets:** about 6 small PNGs (player, two enemies, bullet, orb, floor tile), the bundled font, 3 WAVs and 1 OGG.
  The first version may reuse PurplePie's sample sounds.

## Engine features it uses (all in 0.1.0)

| Need | PurplePie 0.1.0 |
|---|---|
| Window, loop, fixed step | `Engine`, `Game`, `fixed_update` / `update` |
| Player, enemies, bullets as entities; spawning and despawning many per second | `ecs::World`, queries, despawn after collecting (guide §4) |
| Movement | `Velocity` + `integrate_velocity` |
| Facing the mouse | `Context::cursor_world`, `Transform2D::rotation` |
| Camera follow and clamp | `Context::camera_mut`, `Camera2D::visible_world_rect` |
| Sprites, tinting (hit flash), drawing order | `Sprite::with_tint`, `Layer`, `Hidden` (blinking) |
| HUD and buttons | `Text`, `ScreenSpace`, `ui::Button` |
| Sound effects and music | `play_sound`, `loop_sound`, `stop_sound` |
| Pause with Escape | `EngineConfig::with_exit_on_escape(false)`, `Context::request_exit` |
| Failure output and logs | `Error` debug output, `with_console_log` |

The skeleton (below) already uses the window and loop, entities and queries, facing the mouse, camera follow,
screen-space text and the console logger, from the published v0.1.0 tag.

## Gaps and decisions

| Gap | Where it goes | Why |
|---|---|---|
| **Random numbers** (spawn points, enemy types, drops; U-14) | **Engine: PP-036b** | Every game needs them, and the trial game and the guide each wrote an LCG by hand. A seedable generator in `math` keeps the fixed-step simulation reproducible (same seed, same game, like Breakout's autoplay). About 40 lines; no dependency. |
| **Overlap tests** (bullet ↔ enemy, enemy ↔ player, player ↔ orb; U-15) | **Engine: PP-036b** | Breakout, Pie Catch and the guide each wrote box overlap by hand. A small `math::Rect` (centre/size, `overlaps`, `contains`) plus circle overlap covers all three games. No physics: no resolution, no broad phase. |
| Many-to-many collision cost (≈ 200 bullets × 100 enemies at peak) | Game code | 20 000 overlap checks per step are well under a millisecond. Revisit with a spatial grid only if the game measures a problem. |
| Timers and cooldowns (fire rate, invulnerability, waves, orb lifetime) | Game code | One `f32` countdown each. An engine `Timer` type would save little; revisit if the game's code shows repeated patterns. |
| Switching screens: clearing the world | Game code + guide note | `ctx.world_mut().clear()` (hecs) already empties the world. Respawning the HUD and arena is the game's job. The guide should mention `clear()` together with `load_scene`. |
| Tiled floor (16 × 12 tiles of 100 units) | Game code | 192 sprite entities are cheap (batched by texture). A tilemap component pays off for bigger maps or an editor (P5), not here. |
| Camera clamp at the arena edge | Game code | A few lines with `viewport_size` and `zoom`. Revisit as a `Camera2D` helper if PP-036 writes it awkwardly. |
| Best score file | Game code | `std::fs` next to the executable is enough. An engine "save data" API is not justified by one number. |
| Screen shake on hit | Game code | A camera offset for a few frames. |
| Gamepad | Not in this game | Keyboard and mouse cover the design. Gamepad support stays a roadmap candidate until a game needs it. |

## Plan

1. **PP-036b:** `math::Rng` and `math::Rect` (+ circle overlap), with unit tests, docs, a guide update (§4 world
   clearing, §14 using `Rng`/`Rect` instead of hand-written ones) and a CHANGELOG entry under `[Unreleased]`.
2. **PP-036c:** the game's core: arena, player, shooting, enemies, waves, collisions, HUD, sounds. Also decide where
   the game lives:
   - **Recommendation:** in the repository as `games/purple-swarm/`, a workspace member with its own `assets/` and
     `purplepie = { path = "../.." }`, built by CI. It uses the public API only, like an outside crate. It keeps
     compiling as the engine changes, and the owner receives it with the repository.
   - **Alternative:** a separate repository.

   This changes the workspace layout and CI, so it gets an ADR in that task.
3. **PP-036d:** screens (title, pause, game over, restart via `World::clear`), best-score file, polish, and an
   autoplay mode for automated checks (like Breakout's). Its findings go into `docs/USABILITY.md`.

## Skeleton (PP-036a)

An empty crate outside the repository, depending on the published release:

```toml
[dependencies]
purplepie = { git = "https://github.com/DavidVilela1/purplepie", tag = "v0.1.0" }
```

About 80 lines:

- a 1600 × 1200 arena with a marker grid;
- a player quad that moves with WASD and turns toward the cursor;
- a camera that follows the player;
- a screen-space title.

It built with no warnings, and clippy was clean. Under Xvfb, the first frame showed the arena, the markers, the player
and the title. After one second of W+D with the mouse at the top right, the player had turned toward the cursor and
the grid had moved with the camera. The console logger reported the asset root and the GPU.

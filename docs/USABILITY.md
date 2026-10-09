# Usability findings

What a newcomer runs into when making a game with PurplePie without knowing its code. Phase P3.5 (ROADMAP) works
through this list. Each finding has an ID, a class (**doc fix**, **API issue**, **missing feature** or
**observation**) and where it went: fixed in the task that found it, or the follow-up task that owns it.

## PP-031: outside-crate trial (2026-10-08)

**Setup.** A new binary crate, `pie_catch`, in a scratch folder outside the PurplePie package. It depends on
`purplepie = { path = ... }`, and its own `assets/` folder holds a basket and a pie PNG, a WAV and the scene file. It
was written from the README and the generated API docs only.

**The game.** Pies fall at random positions. Left/Right or A/D move the basket. Each catch plays a sound and
raises the score, and three misses end the round; Enter restarts and Escape quits. The level is a scene file
(`assets/scenes/level.ron`) containing:

- a backdrop and ground quad;
- a screen-space title;
- the basket sprite, with a registered game component `Basket { speed }`;
- a score label, with a registered unit struct `ScoreLabel`.

`PIE_CATCH=save` rebuilds the level in code and writes the file. About 270 lines of game code.

**Checks, all under Xvfb with the software GPU:**

| Step | Result |
|---|---|
| README "smallest game", copied verbatim, `cargo build` | Built first time (1 min 45 s for the dependency tree), no warnings |
| Running it | Failed twice at start-up on missing assets (U-02, U-03); ran after adding them. Screenshot showed the sprite, the bar and the text. Space zoomed the view |
| Trial game: scene save, then load | 5 entities written and read back, including both registered components |
| Autoplay run, 6 s | Score 5, misses 1/3, HUD drawn. ALSA file capture contained the catch sounds; the capture was deleted afterwards |
| Game over, then Enter | "Game over" text after 3 misses; Enter reset the score to 0 |
| Shipping | `cargo build --release`, executable plus `assets/` copied into a new folder, run with the working directory `/`. The assets were found next to the executable and the frame matched |
| Logs | No engine messages at first (U-13). With `env_logger` added and `RUST_LOG=info`, the asset root (the crate's `assets/`), audio and GPU lines appeared |
| README fixes re-checked | A fresh `cargo new my_game` beside a PurplePie folder, following the fixed README word for word: built, ran, and the screenshot showed the sprite, the bar and "Hello, PurplePie!". `cargo doc -p purplepie --no-deps` took 1 min from a cold target directory |

### Findings

| ID | Class | Finding | Where it went |
|---|---|---|---|
| U-01 | doc fix | The README never said how a game depends on PurplePie: there was no `Cargo.toml` line, the crate is not on crates.io, and there is no repository URL to use with `git`. | **Fixed:** README "A new game crate" now gives the path form and a git placeholder, plus the folder layout. |
| U-02 | doc fix | The README example loads `textures/player.png`, which a new crate does not have. The first run ends with `Error::Asset`. | **Fixed:** the README says to put any PNG there. |
| U-03 | doc fix | "One font ships with the engine" suggested the font is available to every game. An outside crate's asset root is its own `assets/`, so the second run failed on `fonts/Poppins-Regular.ttf`. | **Fixed:** the README says to copy the font and `OFL.txt` into the game's `assets/fonts/`. |
| U-04 | doc fix | Scene components need `serde` (with `derive`) in the game's own `Cargo.toml`. The `register_scene_component` docs said so, but the README did not. `save_scene` also needs its folder to exist; the API docs said so, but the README did not. | **Fixed:** README dependency snippet and Scenes bullet. |
| U-05 | doc fix | hecs 0.11 queries yield only the components. To get the entity (for example to despawn it) `Entity` goes into the query, and nothing in the `ecs` docs showed that. | **Fixed:** new `ecs` module doctest covering `query::<(Entity, …)>()`, collecting before despawning, and `query_mut`. |
| U-06 | doc fix | `cargo doc` documents all of the roughly 140 dependencies, which took 6 minutes here. | **Fixed:** the README gives `cargo doc -p purplepie --no-deps --open` (1 min from cold). |
| U-07 | doc fix | The crate page pointed only to repository-internal docs. Module pages are written for engine developers: they describe crate-private parts (mixer, sound store) and cite ADR numbers. Only `ecs` and `ui` have examples; `audio`, `input`, `math` and `render` have none. | **Fixed:** the crate page now explains the game-crate layout and points to `Context` and `ecs`. The guide (`docs/GUIDE.md`, PP-032a) covers every feature with compiled examples. In PP-032b every public module page (`audio`, `ecs`, `input`, `math`, `render`, `ui`) gained a one-line summary for game authors, a paragraph with links to the guide, and a compiled example, with engine internals moved under "Engine notes". |
| U-08 | doc fix | Coming from winit, the first guess for letter keys is `KeyCode::KeyA`; PurplePie names them `KeyCode::A`. The compiler error does not suggest the right name. | **Fixed in PP-032a:** GUIDE.md section 8 lists the key names. |
| U-09 | API issue | `ScreenSpace::TOP` / `BOTTOM` / `LEFT` / `RIGHT` versus `TextAnchor::TOP_CENTER` / `BOTTOM_CENTER`. The natural guess `ScreenSpace::TOP_CENTER` does not exist, and the compiler suggests `CENTER`, which is wrong. | **Fixed in PP-034a (ADR-040):** the anchors are now `TOP_CENTER`, `CENTER_LEFT`, `CENTER_RIGHT` and `BOTTOM_CENTER`, named like `TextAnchor`'s; scene files written with the old names still load. |
| U-10 | API issue | `Error::Save` shows the path as given (`"scenes/level.ron"`), while `Error::Asset` shows the resolved full path. When the folder is missing, the user cannot see where the save was attempted. | **Fixed in PP-034a (ADR-040):** `Error::Save` carries the resolved path. In the trial, a save into a missing folder now names `…/pie_catch/assets/scenes/level.ron`. |
| U-11 | API issue | `fn main() -> purplepie::Result<()>` prints errors in `Debug` form (`Error: Asset { path: …, source: Os { code: 2, kind: NotFound, … } }`). It is readable, but raw for a player-facing failure. | **Fixed in PP-034a (ADR-040):** `Error`'s `Debug` prints the message and a "Caused by:" list, so `?` in `main` is readable. |
| U-12 | API issue | Registered game components are written on one line without spaces (`(speed:420.0)`), unlike the pretty-printed engine parts of the same scene file. Cosmetic. | **Decided in ADR-040, built in PP-034b.** |
| U-13 | doc fix + API issue | No engine message appeared in the trial: not the asset root, the GPU, the audio device, hot-reload results or warnings. PurplePie logs through the `log` crate and only the sandbox installs a backend; the README's `PURPLEPIE_LOG=info` works only for the sandbox, and the examples have no logger either. | **Doc part fixed:** README "Logs" bullet; `env_logger` + `RUST_LOG=info` was verified in the trial (the asset-root, audio and GPU lines appeared). **ADR-040** decides an opt-in console logger, **built in PP-034b**. |
| U-14 | missing feature | There are no random numbers; the trial wrote a 10-line LCG. | **PP-036** (second game). Alternatively the guide recommends a small crate such as `fastrand`. |
| U-15 | missing feature | There is no rectangle-overlap helper; the trial, like Breakout, checks boxes by hand. | **PP-036:** "collision helpers" is already on the P3.5 item 6 list. |
| U-16 | observation | A fresh outside crate resolves newer patch versions than the engine's `Cargo.lock` (hecs 0.11.2, glam 0.33.12, zerocopy 0.8.62, cc 1.6.0, …). It built and ran, but CI only tests the committed lock file. | **PP-035** (release): add a latest-dependencies CI job. |
| U-17 | observation | `Cargo.toml` has no `repository` field, so the README cannot give a git URL. | **PP-035:** owner input. |

**What worked without help:** the `Game` / `Context` / `EngineConfig` shape; sprites, quads, text and layers;
`ScreenSpace` HUD text; keyboard input; `play_sound`; `save_scene` / `load_scene` with two registered components
(one of them a unit struct); and shipping with `assets/` next to the executable. The game needed no engine changes.

## PP-032a: guide check (2026-10-08)

`docs/GUIDE.md` was written for newcomers: 14 sections from `cargo new` to shipping, with every Rust block compiled
by `cargo test` (12 blocks). It was then checked the way a newcomer would use it:

- A fresh `cargo new my_game` beside a PurplePie folder.
- `Cargo.toml` lines copied from section 1, and the three asset folders copied as section 1 says.
- The section 14 game pasted into `src/main.rs`.

Results:

- **Build:** with no warnings, and clippy was clean.
- **Under Xvfb:** the first frame showed the arena, three bricks, the HUD score, the Restart button and the animated corner sprite.
- **Play:** moving into a brick raised the score to 1, respawned the brick, and played the blip (captured through ALSA).
- **Restart:** clicking Restart reset the score to 0 and centred the ball.
- **Mutation check:** changing `KeyCode::A` to `KeyCode::KeyA` in the guide made `cargo test` fail on that block.

No new findings. The guide names its own workarounds: the game writes its own random numbers (U-14) and its own
rectangle overlap (U-15), and section 12 shows a `Display`-printing `main` (U-11).

## PP-032b: module pages and starter template (2026-10-09)

- **Module pages.** The crate page's module list now reads as a map for game authors:
  - audio: "Sound effects and music";
  - ecs: "Entities and components";
  - input: "Keyboard and mouse";
  - math: "Vectors and transforms";
  - render: "What gets drawn";
  - ui: "Clickable buttons".

  Each page opens with what the module is for, which `Context` methods to use and the matching guide section,
  followed by a compiled example. Doctests went from 43 to 47.
- **Starter template.** ADR-039: no template crate in the repository; the guide's section 1 and section 14 are the
  template. A trial template crate placed inside the repository failed `cargo check` ("believes it's in a
  workspace"), and a copied one needs its path edited anyway.

## PP-034a: second API review (2026-10-09)

ADR-040 reviewed every public item added since ADR-026 and fixed U-09, U-10 and U-11; U-12 and U-13 are decided and
go to PP-034b. Checked in the trial crate `pie_catch`:

- Its old `ScreenSpace::TOP` stopped compiling. This is the expected break; it is now `TOP_CENTER`.
- Its `level.ron`, written before the rename with `screen_space: Top`, still loads with the title centred.
- A missing `pie.png` now prints:

  ```text
  Error: failed to load asset `…/pie_catch/assets/textures/pie.png`

  Caused by:
      No such file or directory (os error 2)
  ```

- A save into a missing `scenes/` folder names the full resolved path.

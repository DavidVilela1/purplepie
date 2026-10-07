# PurplePie Development Protocol

How Claude Cowork (and any human contributor) works on PurplePie. This file is
persistent. Follow it in every session.

## 1. Source of truth

```text
Actual validated repository state      (code + commands actually executed)
        ↓ outranks
Project documentation                  (ARCHITECTURE, DECISIONS, PROJECT_STATUS, TASKS, RISKS)
        ↓ outranks
Roadmap                                (ROADMAP: intent, not fact)
        ↓ outranks
Conversation history
```

Conversation history never overrides evidence in the repository. If a
document contradicts the code, the code plus validation is the baseline, and the
document is corrected.

## 2. Session start checklist

1. Read [PROJECT_STATUS.md](PROJECT_STATUS.md), then [TASKS.md](TASKS.md), then the relevant parts of [DECISIONS.md](DECISIONS.md).
2. Get the current project: Claude works only in its own session workspace (§3,
   owner rule). If the workspace has no `PurplePie/` copy, or the owner may have
   changed files since the last delivered ZIP, ask the owner to attach a ZIP of the
   repo (without `target/`). Then inspect `Cargo.toml`, `src/`, tests and `docs/`.
   Do not assume docs are current.
3. Run the validation suite (§8) **before** changing anything, so pre-existing failures are known.
4. Confirm which single task the session is for. If none was requested, propose the task marked "next" in TASKS.md and wait.

## 3. Working rules

- **One verified increment at a time.** Work on one task, and a stage never starts automatically.
- **Inspect before modifying.** Read files before editing them.
- **Preserve working architecture.** Change it only deliberately (§7).
- **Implement only the requested scope.** No silent work on future stages.
- **Keep the sandbox runnable** after every task.
- **No new dependency** without ADR-013's checks (current version, MSRV, API verified against source) and a TECH_STACK entry.
- **Never claim a command passed unless it was executed in this session.** If
  something could not be executed, say: *"This code has been reviewed for
  consistency but has not been executed in this environment,"* and list what remains unverified.
- **Label owner-reported results** as owner-reported, never as executed.
- **Owner rule: never touch the owner's computer.** Claude does not read, write,
  stage or commit files in the owner's repo or any folder on the owner's computer,
  even when a folder is connected. All work happens in Claude's own workspace.
- **Owner rule: every delivery is a ZIP + an expand command, always.** At the end
  of every task, send the complete project as a ZIP (§10) together with the exact
  PowerShell `Expand-Archive` command that updates the owner's repo in place. The
  owner extracts it and commits it themselves.
- **Owner rule: deliver to the chat only.** Never save outputs on the owner's
  computer unless the owner specifically asks for it. While a folder from the
  owner's computer is connected to the session, the app copies every file sent
  in chat into `Claude outputs/` inside that folder. So before sending a file,
  confirm that no folder is connected. If one is, ask the owner to remove it
  from the session first.
- **Commit messages (owner rule):** never add AI attribution lines such as
  `Co-Authored-By: Claude …` or `Claude-Session: …`. Use conventional style
  (`feat:`, `fix:`, `docs:` …): a subject line, then a body listing what changed and why.
- **Stop after the requested task** and report.

## 4. Definition of Done

A task is `DONE` only when all of these hold:

- [ ] The implementation exists and matches the task's acceptance criteria.
- [ ] It builds: `cargo check --all-targets` and `cargo build` pass.
- [ ] Style and lints: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.
- [ ] Relevant tests exist for new logic and pass (`cargo test`).
- [ ] Behavior is checked where applicable: smoke run (Xvfb in Cowork) and owner run on Windows for windowed/GPU stages.
- [ ] Architecture stays coherent: the dependency rules in ARCHITECTURE §4 hold, and there are no upward imports.
- [ ] Docs are updated: ARCHITECTURE (if structure changed), DECISIONS (if a decision was made), TECH_STACK (if dependencies changed), RISKS (if risk changed), README (if usage changed).
- [ ] `Cargo.toml`/configuration is updated if needed. `Cargo.lock` is committed.
- [ ] No obvious regression: the sandbox still runs.
- [ ] Resumable: PROJECT_STATUS and TASKS reflect the new state.

If any item cannot be verified, the task stays `IN_PROGRESS` (or is marked
`DONE` with an explicit **Unverified** note agreed with the owner). The gap is
written in PROJECT_STATUS.

## 5. Tasks

- IDs: `PP-NNN`, sequential, never reused. Fields: ID, title, stage, priority, status, dependencies, acceptance criteria.
- Size: one meaningful increment that can be completed and validated in one session. Future stages keep one coarse task each. Split a task when its stage becomes current.
- Status transitions: `TODO → IN_PROGRESS → DONE`, or `→ BLOCKED` (record the blocker and what unblocks it) or `→ CANCELLED` (record why).
- Exactly one task is marked **next** in TASKS.md.

## 6. Architecture decisions

**Write an ADR** (in [DECISIONS.md](DECISIONS.md), next free number) when a
decision materially affects architecture, dependencies, the public API,
ownership, the ECS, the rendering architecture, threading, resource or asset
management, the engine/game split, module boundaries, persistence, or a
significant performance trade-off.

**Do not write an ADR** for naming, trivial helpers, formatting, ordinary bug
fixes, or implementation details with no architectural consequence.

Questions that are not yet decided go in **Pending Decisions** (PD-xx) with the
task that will decide them. They are not ADRs. When decided, a pending decision
becomes an ADR and its PD row is removed. Superseding an ADR means a new ADR
plus a `Superseded by ADR-xxx` status on the old one. Never delete an ADR.

## 7. Architectural drift

If the implementation diverges from ARCHITECTURE or ROADMAP:

1. Do not hide the divergence. State it in the session report.
2. Decide whether it is intentional.
3. If it is beneficial and significant, update ARCHITECTURE to match.
4. Record an ADR when §6 applies.
5. Update ROADMAP if sequencing or scope changed.
6. Update PROJECT_STATUS (Recent Changes) and TASKS.

A bad plan is not preserved merely because it was written first. An accidental
divergence is fixed in the code instead.

## 8. Validation commands

Run from the project root:

```bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
cargo run            # sandbox
```

Cowork has clippy 1.95, but the owner and CI use the latest stable, whose `clippy::chunks_exact_to_as_chunks` rejects
`chunks_exact(4)`-style calls with a constant size (R-19). Before delivering, this must print nothing; use
`as_chunks::<N>()` / `as_chunks_mut::<N>()` instead:

```bash
grep -rnE "chunks_exact(_mut)?\([0-9]" src examples
```

Windowed stages in Cowork (Linux, no display) use a virtual X server:

```bash
sudo apt-get install -y xvfb mesa-vulkan-drivers xdotool x11-utils imagemagick   # once per container if missing
pip install --break-system-packages python-xlib                     # to simulate the close button

# timed run (sandbox requests exit after N frames); expect exit 0 and ~N/60 s
PURPLEPIE_SANDBOX_EXIT_AFTER_FRAMES=120 xvfb-run -a cargo run
```

Interactive checks inside `xvfb-run` (there is no window manager):
- **Find the window:** `xdotool search --name "PurplePie Sandbox"`. Check attributes with `xwininfo -id <id>`.
- **Keys:** `xdotool windowfocus --sync <id>; xdotool key Escape`. XTEST input is
  required, because `xdotool key --window` sends synthetic events that winit (XInput2) ignores.
- **Close button:** send a `WM_DELETE_WINDOW` ClientMessage with python-xlib, which is what a WM does.
- **Busy-loop check:** read `utime+stime` from `/proc/<pid>/stat` after a few idle seconds.
- **Resize:** `xdotool windowsize <id> 640 360`.
- **Pixels (Stage 4+):** `import -window root shot.png; convert shot.png -format "%c" histogram:info:- | sort -rn | head`.
  The window area must be exactly the clear color, e.g. 921,600 px `#6A0DAD` for 1280×720.
  Needs `imagemagick` and the lavapipe Vulkan driver (`mesa-vulkan-drivers`).
- **No-GPU error path:** run with `VK_ICD_FILENAMES=/nonexistent.json VK_DRIVER_FILES=/nonexistent.json __EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent.json`. Expect `Error::Surface` and exit 1, no panic.

- **GPU fault end to end (Stage 4+):** destroy the window from another X client
  (python-xlib `window.destroy()`) while the sandbox runs. Expect `error: GPU rendering
  failed` + cause, exit 1, and **no panic**. Re-run after every wgpu upgrade (R-22).
- **GPU-dependent tests** are `#[ignore]` (CI has no GPU): `cargo test -- --ignored`.
- **Drawn shapes (Stage 5+):** screenshot after ~4 s (lavapipe compiles pipelines slowly) and analyse with PIL:
  per known colour, count pixels and take the bounding box relative to the window origin (`xwininfo`).
  The sandbox's reference quads give exact expectations: amber 200×100 at world (−300, 200) → 20,000 px
  centred at (W/2−300, H/2−200). Teal 100×100 rotated 45° → about 10,000 px, bbox ≈141×141. White 80×80 moving.
  Repeat after a resize: same pixel counts, positions relative to the new centre.
- **Sprites (Stage 6+):** the static sprite is 128×128 at world (0, 120), so its top-left is (W/2−64, H/2−184), and each
  of the 16×16 texels covers 8×8 px. Check per region: the 8-px border is exactly the clear colour (transparent texels);
  the inner quadrants (56×56 = 3,136 px each) are `#E63946`, `#2A9D8F`, `#F4A261`, and the 50% alpha `#3A86FF` blended
  in linear space over the clear colour (`#5662DB` on `#6A0DAD` with an sRGB surface). Tolerance ±2; lavapipe gives 0.
  Since PP-015 the two draw-order quads (next item) overlap two corners: the border shows yellow at the top-left and
  pink covers the bottom-right corner, so compare against a per-pixel expected image rather than whole-region colours.
  The tinted copy at (−300, −20) spins and is not pixel-checked.
- **Draw order (Stage 6+, ADR-021):** the yellow 48×48 quad (layer 0) sits on the sprite's top-left corner
  (screen x W/2−88..W/2−41, y H/2−208..H/2−161) and must be visible only through the sprite's transparent 8-px border:
  2,048 px yellow, the red texels on top. The pink 48×48 quad (layer 1) on the bottom-right corner
  (x W/2+40..W/2+87, y H/2−80..H/2−33) must be fully visible: 2,304 px pink, covering 256 px of the blended quadrant
  (2,880 blended px remain). Build the expected image per pixel over the 200×200 area around the sprite: 0 mismatches.
  Control: put the pink quad on `Layer(-1)` temporarily; it must go under the sprite (2,048 px pink).
- **Camera (Stage 7+, ADR-022):** run with `PURPLEPIE_SANDBOX_CAMERA=x,y,zoom` and compare the **whole frame** against
  a per-pixel model: map each pixel centre to the world with `world = camera + ((px + 0.5) − W/2, H/2 − (py + 0.5)) / zoom`,
  then paint the static scene in draw order (amber, yellow, sprite texels, pink). Exclude the rotating teal quad, the
  spinner and the moving quad's band. Expect 0 mismatches for (0,0,1), (0,120,2), (200,0,0.5), and again after a resize.
  Check sensitivity by evaluating the model one world unit off (thousands of mismatches expected).
  The timed exit line prints the logical viewport (check it after resizing mid-run) and the camera.
- **Keyboard (Stage 8+, ADR-024):** `xdotool windowfocus --sync <id>; sleep 0.5`, then XTEST keys
  (`xdotool keydown Right; sleep 0.5; xdotool keyup Right`, `xdotool key equal`). Run with `PURPLEPIE_LOG=debug` to see every
  key and focus event. The sandbox's timed exit prints the camera and how many `=` presses `fixed_update` and `update`
  saw (must be equal to the taps sent). Check the final frame against the per-pixel camera model at the printed camera.
  Focus loss: remove focus with python-xlib (`set_input_focus(X.NONE, …)`) while a key is held; the pan must stop then.
  The 0.5 s settle avoids a suspected focus race in WM-less Xvfb (R-11).
- **Mouse (Stage 8+, ADR-024):** after focusing, `xdotool mousemove X Y; xdotool click 1` (left), `click 4` / `click 5` (wheel).
  The timed exit prints clicks seen per callback and the cursor (screen + world). Model stamps (16×16) and the marker
  (10×10, follows the cursor) in the per-pixel camera check. Note: on a new Xvfb display the pointer starts at the screen
  centre, inside the window, so the marker is visible unless you move the pointer away. winit/X11 reports each XTEST wheel
  click twice.
- **Breakout example (Stage 10+):** `PURPLEPIE_BREAKOUT_AUTOPLAY=win` (≈2 min under lavapipe) and `=lose` (≈15 s) must print
  the same `autoplay finished: …` line on every run (it is deterministic: the simulation runs only in `fixed_update` with a
  seeded generator); current values are in TASKS (PP-012). Change them only with an explained gameplay or engine change.
  Since PP-018b the lose screen also shows text: compare it with an earlier screenshot and check that every changed
  pixel lies inside the rectangles `TextMetrics::bounds` predicts (compute them from the font's advance widths and
  ascent/descent at the `Camera2D::fit` zoom); the overlay colour `(161, 33, 47)` covers 339,277 px at 1024×768.
  Since PP-025 the ball texture is `Linear` (ADR-034): in a mid-game screenshot its edge pixels lie between the
  background `(16, 0, 43)` and white, and none is darker than the background (a darker ring would mean a fringe). The
  exact check is the ignored GPU test `linear_textures_blend_texels_and_nearest_ones_stay_exact`.
  Interactive: `xdotool mousemove 300 600` → paddle centred at x = 300; `click 1` launches. Don't `wait` without a PID in
  scripts that also started Xvfb (it waits for the server forever).
- **Scene files (PP-026a+, ADR-035):** run `examples/scene` under Xvfb three times with
  `PURPLEPIE_SCENE_EXAMPLE=save`, `=build` and unset (`load`), move the pointer out of the window first, screenshot each
  after ~3 s and expect pixel-identical frames. `save` rewrites `assets/scenes/demo.ron`; keep the file unchanged unless
  the demo deliberately changes (a unit test compares it byte-for-byte with what `save_scene` writes).
- **Sprite sheets (PP-019+):** the exact check is the ignored GPU test `sprite_regions_show_exactly_their_texels`.
  In the sandbox, four cells of `assets/textures/sandbox_sheet.png` (frames 0, 5, 6 mirrored, 3) are 32×32 world units
  centred at x = 400, 440, 480, 520, y = −250; add them to the whole-frame camera model (texel = 4×4 world units;
  mirrored: texel column 7 − column) and expect 0 mismatches at zoom 1, 1.5 and 2 (apart from the cursor marker).
- **Sprite animation (PP-020+, ADR-028):** the fifth cell at (580, −250) plays all 8 sheet frames at 4 fps. In a
  screenshot it must equal exactly one frame (exclude it from the static model and compare it with each frame). The
  timed exit prints `animated cell shows frame Some(n)`; expect n = ⌊fixed steps / 15⌋ mod 8.
- **Screen space (PP-021+, ADR-029):** the sandbox's HUD panel covers window pixels x 10..269, y 10..49 (`#1D3557`
  around its text) and the corner square x W−50..W−11, y H−50..H−11 (`#FF006E`). Screenshot at several cameras: both
  crops must be pixel-identical; after `xdotool windowsize` the square follows the bottom-right corner. Exclude these
  rectangles from the world model. The exact check is the ignored GPU test `screen_space_draws_on_top_at_fixed_window_pixels`.
- **Audio (PP-022+, ADR-030):** Cowork has no sound card. Without one the sandbox logs `audio disabled` and prints
  `audio output available: false`. To check the real output path, give ALSA a capture device in a scratch `HOME`:
  `.asoundrc` = `pcm.!default { type file; slave.pcm "null"; file "<dir>/out.raw"; format "raw" }`, run the sandbox
  with `PURPLEPIE_LOG=info` (logs the device format, e.g. `2 channels at 48000 Hz (f32)`), click once, and compare
  `out.raw` (interleaved samples) with `blip.wav` resampled linearly × 0.8: expect exactly one copy, identical channels.
  The null device runs faster than real time and the file grows by ~140 MB/s, so keep runs short and delete it after.
  For determinism with an active device use `pcm.!default { type null }`.
  Loops (PP-024a): press `m`, wait, press `m` again; model the mixer (f64 position += 22050/48000, wrap by subtracting
  the length, interpolate with the first frame after the last, × 0.5) and expect an exact match up to the last
  non-zero frame and only zeros after it. Align on the first non-zero frame minus one (the loop starts with a 0 sample).
  OGG (PP-024b+, ADR-033): the sandbox loop is `loop.ogg`, so model `lewton`'s decoded samples, not `loop.wav` (dump
  them from a temporary unit test that writes `decode(…).samples` to a scratch file; remove it after). Align on the
  first non-zero frame itself and expect max diff 0. ffmpeg (`libvorbis`) is available in Cowork for making test files;
  encode shipped assets with `-map_metadata -1 -fflags +bitexact -flags:a +bitexact` so they are reproducible.
- **UI buttons (PP-023+, ADR-031):** the sandbox's "Reset camera" button covers window x W−180..W−21, y 20..59. With
  XTEST (`mousemove`, `mousedown 1`, `mouseup 1`) and screenshots, a pixel inside it away from the text (e.g. W−170, 25)
  is `#3A86FF` idle, `#6FA8FF` hovered, `#1D5FCC` held. Start with `PURPLEPIE_SANDBOX_CAMERA=0,120,2`: after one click
  the exit lines show `camera at (0, 0) zoom 1` and `reset button clicks: 1`. Press on the button and release elsewhere:
  no click. Clicks on the button must not stamp. Exclude the button from the world model.
- **Text (PP-018a+, ADR-027):** the exact check is the ignored GPU test
  `gpu_text_matches_the_cpu_rasterization_pixel_for_pixel` (offscreen render read back and compared with the CPU
  rasterization, ≤ 1/255 per pixel). In the sandbox, the help label's baseline starts at world (−600, −300), 20 units em;
  its origin lands on `round(W/2 + (−600 − cx)·zoom), round(H/2 − (−300 − cy)·zoom)`. Exclude the label's box from the
  whole-frame camera model (0 mismatches elsewhere) and compare the box with FreeType (PIL `ImageFont.truetype`, same font,
  `anchor="ls"`): expect the same layout and an IoU around 0.6–0.9 (hinting and linear blending differ), not equality.
- **Asset errors end to end:** temporarily move or overwrite `assets/textures/sandbox_quadrants.png` (restore it after!).
  Expect `error: failed to load asset `…`` plus `caused by:` lines, exit 1, and no panic.
- **Asset root (Stage 9+, ADR-025):** run with `PURPLEPIE_LOG=info` and check the `asset root:` line in four layouts:
  from the project folder (working-directory fallback); a copy of the binary with `assets/` beside it started from an
  unrelated folder (executable folder); the same copy without `assets/` (exit 1, error lists both searched folders);
  the moved binary started from the project folder (working-directory fallback).
- **Window destroyed (re-run after any change that touches the runner or adds a per-frame window/surface call; use a fresh Xvfb display, see R-22):**
  PP-009 showed that a per-frame `Window::inner_size()` panics inside winit once the X11 window is gone (R-22).
- **Logs:** `PURPLEPIE_LOG=info cargo run` shows the GPU/backend/surface line.

The owner confirms every windowed stage on Windows with `cargo test` and `cargo run`.

**CI** (`.github/workflows/ci.yml`) runs on every push and PR: `cargo fmt --all -- --check`
and `cargo clippy --locked --all-targets --all-features -- -D warnings` on Linux, plus
`cargo check --locked --all-targets --all-features` and `cargo test --locked --all-features`
on Linux, Windows and macOS. It needs no GPU. Ignored GPU tests are not run.
Keep local commands consistent with CI: use `--locked` and keep `Cargo.lock` committed.

## 9. Environment setup

**All platforms:** Rust stable ≥ 1.90 (1.95 in use), plus the `rustfmt` and `clippy` components.
`rustup update stable && rustup component add rustfmt clippy`.

**Windows (owner's machine):**
- The MSVC linker is required. In Visual Studio Installer, add **"Desktop
  development with C++"** (MSVC x64/x86 build tools + a Windows SDK). Without
  it, `check`/`clippy` pass but `test`/`build`/`run` fail with `link.exe not found` (R-14).
- Keep the project, or at least `target/`, **outside OneDrive** (R-15). For example,
  move it to `C:\Users\<you>\dev\PurplePie`, or run `setx CARGO_TARGET_DIR C:\Users\<you>\cargo-target`.

**Linux:** development packages for X11/Wayland are loaded at runtime. A Vulkan
driver or Mesa is needed to run from Stage 4. Since PP-022 the ALSA headers are
needed to build: `sudo apt-get install -y libasound2-dev` (CI does the same).

## 10. Stage archives

Every completed stage ends with `PurplePie-stage-N.zip`. A documentation
update inside a stage re-issues that stage's archive.
After Stage 10 there are no numbered stages: each post-portfolio task ends with
`PurplePie-<task id>.zip` (for example `PurplePie-PP-018a.zip`).

```bash
cd <parent of PurplePie>
zip -r -X PurplePie-stage-N.zip PurplePie \
    -x 'PurplePie/target/*' '*.DS_Store' '*/Thumbs.db' '*.swp' '*/.idea/*' '*/.vscode/*'
unzip -tq PurplePie-stage-N.zip                      # integrity
unzip -Z1 PurplePie-stage-N.zip | grep -c target/    # must print 0
```

Before delivery, verify:
- the ZIP has a single `PurplePie/` root;
- its contents are identical to the workspace (`diff -r`, excluding `target/`);
- the extracted copy builds and tests.

Deliver the ZIP as a downloadable file, and report Created/Verified/Downloadable/Location factually.

**Delivery (owner rule, always):** send the ZIP as a downloadable file, and
give the exact commands to apply it to the owner's repo:

```powershell
# ALWAYS first (owner rule): delete the copy the app may drop into the repo
Remove-Item -Recurse -Force "C:\Users\35193\OneDrive\Ambiente de Trabalho\Programing\3-major-software-projects\PurplePie\PurplePie-stage-0\PurplePie\Claude outputs" -ErrorAction SilentlyContinue

# The owner sometimes edits on GitHub: bring those commits in before expanding
cd "C:\Users\35193\OneDrive\Ambiente de Trabalho\Programing\3-major-software-projects\PurplePie\PurplePie-stage-0\PurplePie"
git pull --rebase origin main

# ZIP root is PurplePie/, so the destination is the folder that CONTAINS the repo
Expand-Archive -Path "$HOME\Downloads\<zip name>" -DestinationPath "C:\Users\35193\OneDrive\Ambiente de Trabalho\Programing\3-major-software-projects\PurplePie\PurplePie-stage-0" -Force
```

Extracting never deletes files. If the task removed or renamed files, also give
the exact `Remove-Item` commands. A ZIP never contains `.git/` or `target/`, so
extracting leaves the owner's Git history and build cache untouched.

## 11. Session end report

Summarize: what changed, validation actually executed, docs updated, archive
status, the single next task, and anything unverified. Then stop.

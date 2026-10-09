# Platform checklist

A hands-on check of PurplePie on a real machine. It was written for the owner's **Windows** PC (PowerShell), with
macOS/Linux commands at the end. It takes about 30–40 minutes, mostly compile time. Claude ran every step on Linux
(Xvfb + software GPU), including the PowerShell blocks under PowerShell 7 for Linux, before handing it over (PP-033a).

**How to report.** Reply with:

1. the number of every step that did not look as described, and what you saw;
2. the console lines asked for in steps 1, 2, 3 and 5.

A screenshot helps when something looks wrong. Nothing in the checklist changes the repository except step 4, which
puts the file back itself; `git status` at the end must show no changes.

## 0. Setup

Open PowerShell (Windows Terminal or the plain console) and set the repository path once. Every later step uses `$pp`.

```powershell
$pp = "C:\Users\35193\OneDrive\Ambiente de Trabalho\Programing\3-major-software-projects\PurplePie\PurplePie-stage-0\PurplePie"
cd $pp
rustc --version
cargo --version
git status --short
```

**Expect:** a Rust version of 1.90 or newer, and an empty `git status`.

## 1. Unit tests and doctests

```powershell
cargo test
```

**Expect:** `test result: ok. 216 passed; 0 failed; 12 ignored` and `test result: ok. 47 passed`. The guide's code
blocks are part of the 47. **Report** the two `test result` lines.

## 2. GPU tests on your real graphics card

```powershell
cargo test -- --ignored
```

**Expect:** `test result: ok. 12 passed`. These tests render offscreen and compare pixels exactly. They have only
ever run on a software GPU, so a failure here is a finding about real hardware, not necessarily a bug. **Report** the
`test result` line, plus the names of any failed tests and their first error lines.

## 3. Sandbox: drawing, input, sound, window

```powershell
$env:PURPLEPIE_LOG = "info"
cargo run
```

**Report** the console line starting with `GPU:` (your adapter and backend) and the `audio:` line.

**Expect** in the window (1280 × 720, purple background):

- coloured rectangles (an orange one, a turned teal diamond, a white square and small yellow and pink ones);
- a four-colour square sprite in the middle, a faded copy of it slowly turning on the left, and five small
  sprite-sheet cells at the bottom right, one of them animating;
- a text label at the bottom left listing the controls, and a small green marker following the mouse cursor;
- a dark "Screen-space HUD" panel at the top left, a "Reset camera" button at the top right and a pink square in the
  bottom-right corner. The HUD, button and corner square stay put while the camera moves.

**Do:**

- **Camera:** arrow keys pan, and `=` / `-` or the mouse wheel zoom (one wheel notch is one zoom step, not two).
- **Clicks:** a left click in the world stamps a square at the cursor and plays a short blip. "Reset camera" resets
  the pan and zoom, and stamps nothing.
- **Music:** `M` starts a music loop and `M` again stops it. The loop must repeat with no gap or click at the seam.
- **Window:**
  - Resize the window: the picture stays centred and is not stretched.
  - Minimise and restore it: drawing resumes.
  - Escape quits, and so does the close button on a second run.

Then clear the log setting:

```powershell
Remove-Item Env:PURPLEPIE_LOG
```

## 4. Hot reload

The sandbox reloads changed assets while it runs. Start it again:

```powershell
cargo run
```

While the window is open, open a **second** PowerShell window and run:

```powershell
cd "C:\Users\35193\OneDrive\Ambiente de Trabalho\Programing\3-major-software-projects\PurplePie\PurplePie-stage-0\PurplePie"
Copy-Item assets\textures\sandbox_sheet.png assets\textures\sandbox_quadrants.png
```

**Expect:** within about one second both four-colour squares (the middle one and the turning one) show the sprite
sheet instead, without restarting.
Now put the original back. The sandbox switches back too:

```powershell
git checkout -- assets/textures/sandbox_quadrants.png
git status --short
```

**Expect:** the four colours return, and `git status` prints nothing. Close the sandbox with Escape.

## 5. Breakout

Play a round:

```powershell
cargo run --example breakout
```

**Expect:**

- arrows, A/D or the mouse move the paddle, and Space, ↑ or a left click launches the ball;
- bounces, broken bricks and lost balls play sounds;
- the HUD shows the score as text, lives as small balls and cleared bricks as a progress bar;
- losing all lives shows an end screen, and launching again restarts; Escape quits.

Then let the bot play. Each run ends by itself after printing a summary line:

```powershell
$env:PURPLEPIE_BREAKOUT_AUTOPLAY = "lose"
cargo run --example breakout
$env:PURPLEPIE_BREAKOUT_AUTOPLAY = "win"
cargo run --example breakout
Remove-Item Env:PURPLEPIE_BREAKOUT_AUTOPLAY
```

**Report** both summary lines. On Linux they are:

- `breakout: autoplay finished: Lost after 892 fixed steps; bricks 8/60; lives 0; score 14`
- `breakout: autoplay finished: Won after 7135 fixed steps; bricks 60/60; lives 3; score 220`

(The win takes about two minutes: the bot plays in real time.) The game uses `sin`/`cos`, whose last bits can differ between Windows and Linux
math libraries, so different numbers are worth reporting but are not a crash.

## 6. Scene files

```powershell
cargo run --example scene
```

**Expect:**

- the console prints `scene: loaded 19 entities from scenes/demo.ron`, then `scene: demo scene loaded 1 time(s)` and
  `scene: 2 spinning sprites`;
- the window shows coloured quads, sprites (two of them rotating), text and a "Click me" button at the top right;
- each click on the button prints `scene: button clicked (1 so far)`, then 2, and so on.

Escape quits. Then build the same scene in code; it must look the same:

```powershell
$env:PURPLEPIE_SCENE_EXAMPLE = "build"
cargo run --example scene
Remove-Item Env:PURPLEPIE_SCENE_EXAMPLE
```

## 7. A new game made from the guide

This copies the guide's complete game into a new crate **outside** the repository and outside OneDrive
(`C:\Users\35193\purplepie-check`), exactly as a newcomer would build it.

```powershell
$check = "$HOME\purplepie-check"
New-Item -ItemType Directory -Force $check | Out-Null
cd $check
cargo new my_game
cd my_game
$ppToml = $pp -replace '\\', '/'
Add-Content Cargo.toml "purplepie = { path = `"$ppToml`" }" -Encoding ascii
New-Item -ItemType Directory -Force assets | Out-Null
Copy-Item -Recurse "$pp\assets\fonts", "$pp\assets\sounds", "$pp\assets\textures" assets
$guide = Get-Content "$pp\docs\GUIDE.md" -Raw -Encoding UTF8
$section = $guide.Substring($guide.IndexOf('## 14. Putting it together'))
$code = [regex]::Match($section, '(?s)```rust no_run\r?\n(.*?)```').Groups[1].Value
[System.IO.File]::WriteAllText((Join-Path (Get-Location) 'src\main.rs'), $code)
cargo run
```

**Expect:**

- the first build compiles the dependencies (a few minutes) and shows no warnings;
- an 800 × 600 window titled "Collector" opens, with a white ball, three bricks, "Score 0" at the top left, a
  "Restart" button and an animated square at the bottom right;
- arrows or WASD move the ball, and touching a brick plays a blip, adds 1 to the score and puts a new brick elsewhere;
- "Restart" (or R) resets the score; Escape quits.

## 8. Shipping that game

```powershell
cargo build --release
$ship = "$check\ship"
New-Item -ItemType Directory -Force $ship | Out-Null
Copy-Item target\release\my_game.exe $ship
Copy-Item -Recurse assets $ship
cd $HOME
& "$ship\my_game.exe"
```

**Expect:** the same game, started from another folder: it finds `assets\` next to the `.exe`. Escape quits. Then
clean up (the check folder holds a few GB of build output) and confirm the repository is untouched:

```powershell
cd $pp
Remove-Item -Recurse -Force $check
git status --short
```

**Expect:** `git status` prints nothing.

## macOS or Linux

The same steps in Terminal (bash/zsh). Linux also needs `sudo apt install libasound2-dev` first.

```bash
pp="$HOME/path/to/PurplePie"         # 0. adjust
cd "$pp" && cargo test && cargo test -- --ignored        # 1, 2
PURPLEPIE_LOG=info cargo run                              # 3
cargo run                                                 # 4: in a second terminal:
#   cp assets/textures/sandbox_sheet.png assets/textures/sandbox_quadrants.png
#   git checkout -- assets/textures/sandbox_quadrants.png
cargo run --example breakout                              # 5
PURPLEPIE_BREAKOUT_AUTOPLAY=lose cargo run --example breakout
PURPLEPIE_BREAKOUT_AUTOPLAY=win cargo run --example breakout
cargo run --example scene                                 # 6
PURPLEPIE_SCENE_EXAMPLE=build cargo run --example scene
check="$HOME/purplepie-check"; mkdir -p "$check" && cd "$check"      # 7
cargo new my_game && cd my_game
echo "purplepie = { path = \"$pp\" }" >> Cargo.toml
mkdir -p assets && cp -R "$pp/assets/fonts" "$pp/assets/sounds" "$pp/assets/textures" assets/
python3 -c "import re,sys; g=open(sys.argv[1]).read(); s=g[g.index('## 14. Putting it together'):]; open('src/main.rs','w').write(re.search(r'\`\`\`rust no_run\n(.*?)\`\`\`', s, re.S).group(1))" "$pp/docs/GUIDE.md"
cargo run
cargo build --release && mkdir -p "$check/ship" && cp target/release/my_game "$check/ship/" && cp -R assets "$check/ship/"   # 8
cd "$HOME" && "$check/ship/my_game"
cd "$pp" && rm -rf "$check" && git status --short
```

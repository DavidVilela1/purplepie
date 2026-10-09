# PurplePie guide

From an empty folder to a shipped 2D game. Each section is short; the last one puts everything together in one
complete game. Every Rust block in this guide is compiled by `cargo test` (as a doctest of the `purplepie` crate),
so the code matches the engine you have.

The API reference is one command away: `cargo doc -p purplepie --no-deps --open`. Start at `Context`.

1. [Set up a game crate](#1-set-up-a-game-crate)
2. [The game loop](#2-the-game-loop)
3. [Coordinates and the camera](#3-coordinates-and-the-camera)
4. [Entities and components](#4-entities-and-components)
5. [Shapes and sprites](#5-shapes-and-sprites)
6. [Sprite sheets and animation](#6-sprite-sheets-and-animation)
7. [Text](#7-text)
8. [Keyboard and mouse](#8-keyboard-and-mouse)
9. [HUD and buttons](#9-hud-and-buttons)
10. [Sound and music](#10-sound-and-music)
11. [Scene files](#11-scene-files)
12. [Hot reload, errors and logs](#12-hot-reload-errors-and-logs)
13. [Shipping](#13-shipping)
14. [Putting it together](#14-putting-it-together)

## 1. Set up a game crate

PurplePie is not on crates.io yet. Put your game next to a PurplePie folder and depend on it by path:

```text
cargo new my_game
```

```toml
# my_game/Cargo.toml
[dependencies]
purplepie = { path = "../PurplePie" }
serde = { version = "1", features = ["derive"] }   # only for your own components in scene files (section 11)
```

Your game has its **own** `assets/` folder; every asset path in your code is relative to it. To follow this guide,
copy PurplePie's `assets/fonts`, `assets/sounds` and `assets/textures` folders into `my_game/assets/`:

```text
my_game/
├── Cargo.toml
├── src/main.rs
└── assets/
    ├── fonts/      Poppins-Regular.ttf, OFL.txt (the font's licence: ship it with the font)
    ├── sounds/     blip.wav, hit.wav, lose.wav, loop.wav, loop.ogg
    └── textures/   sandbox_quadrants.png, sandbox_sheet.png, breakout/ball.png, breakout/brick.png
```

The first `cargo run` compiles about 140 crates and takes a few minutes. On Linux install the ALSA headers first
(`sudo apt install libasound2-dev`); on Windows you need the Visual Studio "Desktop development with C++" workload.

## 2. The game loop

A game is a type that implements `Game`. `Engine::run` opens the window, then calls your game back with a `Context`,
your handle to everything: the world, assets, input, sound, the camera, time.

```rust no_run
use purplepie::{Context, Engine, EngineConfig, Game};

struct MyGame {
    seconds: f32,
}

impl Game for MyGame {
    // Once, after the window exists: load assets and spawn entities.
    fn init(&mut self, _ctx: &mut Context<'_>) -> purplepie::Result<()> {
        Ok(())
    }

    // At a fixed rate (60 per second by default): movement, collisions, rules.
    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        self.seconds += ctx.dt(); // the fixed step, 1/60 s
    }

    // Once per drawn frame: things that follow the screen, like buttons and the title.
    fn update(&mut self, ctx: &mut Context<'_>) {
        ctx.set_window_title(format!("My Game - {:.0} s", self.seconds));
    }
}

fn main() -> purplepie::Result<()> {
    let config = EngineConfig::new("My Game")
        .with_size(1280, 720) // logical pixels
        .with_clear_color(purplepie::render::Color::hex(0x202030));
    Engine::new(config)?.run(MyGame { seconds: 0.0 })
}
```

- All three methods are optional; leave out the ones you do not need.
- Escape quits by default (`EngineConfig::with_exit_on_escape(false)` hands Escape to your game). The game can quit
  itself with `ctx.request_exit()`.
- Put gameplay in `fixed_update`: it runs at the same rate on every machine, so the game behaves the same whatever
  the frame rate. Key presses are reported once to each callback, so both can react to them.

## 3. Coordinates and the camera

The origin is the centre of the window, +X is right and **+Y is up**. One unit is one logical pixel at zoom 1. An
entity's `Transform2D` gives its position (its centre), rotation in radians (counter-clockwise) and scale.

The camera is plain data: move it, zoom it, or fit a region of the world into the window.

```rust
use purplepie::Context;
use purplepie::math::Vec2;
use purplepie::render::Camera2D;

fn follow(ctx: &mut Context<'_>, target: Vec2) {
    ctx.camera_mut().position = target; // the world point at the window centre
    ctx.camera_mut().zoom = 2.0; // everything twice as big
}

fn show_level(ctx: &mut Context<'_>) {
    // Make the 800 × 600 area around (0, 0) fill the window, whatever its size.
    let viewport = ctx.viewport_size();
    *ctx.camera_mut() = Camera2D::fit(Vec2::ZERO, Vec2::new(800.0, 600.0), viewport);
}
```

## 4. Entities and components

Game state lives in one ECS world (`hecs`). An entity is a bundle of components; any `'static + Send + Sync` type of
yours can be a component. Systems are plain functions you call yourself, in the order you choose.

```rust
use purplepie::Context;
use purplepie::ecs::{self, Entity, Velocity};
use purplepie::math::{Transform2D, Vec2};

/// Your own components: plain structs.
struct Enemy {
    health: u32,
}
struct Bullet;

fn spawn_enemy(ctx: &mut Context<'_>, at: Vec2) -> Entity {
    ctx.world_mut().spawn((
        Transform2D::from_position(at),
        Velocity(Vec2::new(-40.0, 0.0)), // units per second
        Enemy { health: 3 },
    ))
}

fn step(ctx: &mut Context<'_>) {
    let dt = ctx.dt();
    // Built-in system: position += velocity × dt.
    ecs::integrate_velocity(ctx.world_mut(), dt);

    // Change components in place with `query_mut`.
    for (transform, enemy) in ctx.world_mut().query_mut::<(&mut Transform2D, &Enemy)>() {
        transform.scale = Vec2::splat(1.0 + enemy.health as f32 * 0.1);
    }

    // Queries yield components only; add `Entity` to get the entity too. Collect first,
    // then despawn: the world cannot change while a query borrows it.
    let gone: Vec<Entity> = ctx
        .world()
        .query::<(Entity, &Transform2D, &Bullet)>()
        .iter()
        .filter(|(_, t, _)| t.position.x.abs() > 1000.0)
        .map(|(e, _, _)| e)
        .collect();
    for e in gone {
        let _ = ctx.world_mut().despawn(e);
    }
}
```

`ctx.world().get::<&Enemy>(entity)` reads one entity's component; the full `hecs` API is re-exported as
`purplepie::ecs::hecs`.

## 5. Shapes and sprites

An entity with a `Transform2D` and a `Quad` is drawn as a solid rectangle; with a `Sprite`, as an image. `Layer`
orders drawing (higher on top, default 0) and `Hidden` skips an entity without removing it.

```rust
use purplepie::Context;
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Color, Hidden, Layer, Quad, Sprite, TextureOptions};

fn spawn_scenery(ctx: &mut Context<'_>) -> purplepie::Result<()> {
    // PNG files, relative to your assets folder. A missing file is an error right here.
    let crisp = ctx.load_texture("textures/sandbox_quadrants.png")?; // pixel art: Nearest (default)
    let smooth = ctx.load_texture_with("textures/breakout/ball.png", TextureOptions::LINEAR)?;

    let world = ctx.world_mut();
    world.spawn((
        Transform2D::from_position(Vec2::new(0.0, -200.0)),
        Quad::new(Vec2::new(600.0, 40.0), Color::hex(0x2A9D8F)),
        Layer(-1), // behind layer-0 things
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(-100.0, 0.0)).with_rotation(0.3),
        Sprite::new(crisp, Vec2::splat(96.0)), // drawn 96 × 96, whatever the file size
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(100.0, 0.0)).with_scale(Vec2::new(-1.0, 1.0)), // mirrored
        Sprite::new(smooth, Vec2::splat(48.0)).with_tint(Color::rgba(1.0, 0.5, 0.5, 0.8)),
    ));
    world.spawn((
        Transform2D::default(),
        Quad::new(Vec2::splat(20.0), Color::WHITE),
        Hidden, // remove the component to show it again
    ));
    Ok(())
}
```

## 6. Sprite sheets and animation

A sheet is one texture cut into equal cells. `SpriteGrid` names the cells, `Sprite::with_region` shows one of them,
and `SpriteAnimation` steps through a range of them.

```rust
use purplepie::Context;
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{self, Sprite, SpriteAnimation, SpriteGrid};

fn spawn_animated(ctx: &mut Context<'_>) -> purplepie::Result<()> {
    let sheet = ctx.load_texture("textures/sandbox_sheet.png")?; // 32 × 16 pixels
    let grid = SpriteGrid::new(8, 8, 4, 2); // 8 × 8 cells, 4 columns, 2 rows; frames 0..=7

    let still = grid.frame(5).expect("the sheet has 8 frames");
    ctx.world_mut().spawn((
        Transform2D::from_position(Vec2::new(-60.0, 0.0)),
        Sprite::new(sheet, Vec2::splat(64.0)).with_region(still),
    ));
    ctx.world_mut().spawn((
        Transform2D::from_position(Vec2::new(60.0, 0.0)),
        Sprite::new(sheet, Vec2::splat(64.0)),
        SpriteAnimation::new(grid, 0, 7, 8.0), // frames 0 to 7, 8 per second, looping (`.once()` stops at the end)
    ));
    Ok(())
}

fn animate(ctx: &mut Context<'_>) {
    // Call it from `fixed_update`: advances every animation and sets its sprite's region.
    let dt = ctx.dt();
    render::advance_animations(ctx.world_mut(), dt);
}
```

## 7. Text

Load a TrueType/OpenType font, then give entities a `Text`. The size is in world units, and the anchor says which point of
the text sits at the entity's position. To change what it says, change `content`.

```rust
use purplepie::Context;
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Color, Text, TextAnchor};

/// Marks the score label.
struct Score;

fn spawn_label(ctx: &mut Context<'_>) -> purplepie::Result<()> {
    let font = ctx.load_font("fonts/Poppins-Regular.ttf")?;
    let title = Text::new("Hello!\nTwo lines", font, 32.0)
        .with_color(Color::hex(0xF4A261))
        .with_anchor(TextAnchor::TOP_CENTER);
    // Measure before spawning, e.g. to put a panel behind it.
    if let Some(metrics) = ctx.measure_text(&title) {
        println!("{} × {} units", metrics.width, metrics.height);
    }
    ctx.world_mut()
        .spawn((Transform2D::from_position(Vec2::new(0.0, 300.0)), title));
    ctx.world_mut().spawn((
        Transform2D::default(),
        Text::new("Score 0", font, 24.0).with_anchor(TextAnchor::CENTER),
        Score,
    ));
    Ok(())
}

fn show_score(ctx: &mut Context<'_>, score: u32) {
    for (text, _) in ctx.world_mut().query_mut::<(&mut Text, &Score)>() {
        text.content = format!("Score {score}");
    }
}
```

Text supports `\n` line breaks; there is no automatic wrapping yet.

## 8. Keyboard and mouse

`ctx.input()` describes the keyboard and mouse. Keys are named by their position on a US keyboard: letters are
`KeyCode::A` … `KeyCode::Z`, digits `KeyCode::Digit0` … `Digit9`, plus `ArrowLeft`, `Space`, `Enter`, `Escape`,
`ShiftLeft`, `F1` and so on (`KeyCode::ALL` lists them).

```rust
use purplepie::Context;
use purplepie::input::{KeyCode, MouseButton};

fn controls(ctx: &mut Context<'_>) {
    let input = ctx.input();
    // -1, 0 or 1: arrows or A/D, clamped so both together are not faster.
    let x = (input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight) + input.axis(KeyCode::A, KeyCode::D))
        .clamp(-1.0, 1.0);
    let held = input.pressed(KeyCode::ShiftLeft); // down right now
    let jump = input.just_pressed(KeyCode::Space); // went down since the last callback
    let click = input.mouse_just_pressed(MouseButton::Left);
    let wheel = input.scroll().y; // wheel notches since the last callback (up is positive)
    // Where the cursor is in the world (it follows the camera); `None` outside the window.
    let cursor = ctx.cursor_world();
    println!("{x} {held} {jump} {click} {wheel} {cursor:?}");
}
```

## 9. HUD and buttons

Add `ScreenSpace` to an entity to pin it to the window instead of the world: its position is then measured from a
window anchor (corner, edge or centre) in logical pixels, +Y still up, and the camera does not move it. Screen-space
entities are drawn above the world.

A `ui::Button` makes an entity clickable; what it looks like is up to you (a quad, a sprite, a text).

```rust
use purplepie::Context;
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Color, FontId, Layer, Quad, ScreenSpace, Text, TextAnchor};
use purplepie::ui::{self, Button, Pointer};

/// Marks the restart button.
struct Restart;

fn spawn_hud(ctx: &mut Context<'_>, font: FontId) {
    let world = ctx.world_mut();
    // 16 pixels in from the top-left corner (down is -Y).
    world.spawn((
        Transform2D::from_position(Vec2::new(16.0, -16.0)),
        Text::new("Lives 3", font, 20.0).with_anchor(TextAnchor::TOP_LEFT),
        ScreenSpace::TOP_LEFT,
    ));
    // A button 100 pixels left of and 40 pixels below the top-right corner.
    let size = Vec2::new(140.0, 40.0);
    let at = Transform2D::from_position(Vec2::new(-100.0, -40.0));
    world.spawn((at, Quad::new(size, Color::hex(0xE76F51)), Button::new(size), ScreenSpace::TOP_RIGHT, Restart));
    world.spawn((
        at,
        Text::new("Restart", font, 18.0).with_anchor(TextAnchor::CENTER),
        ScreenSpace::TOP_RIGHT,
        Layer(1), // above the button's quad
    ));
}

/// Call from `update` (once per frame); returns `true` on the frame Restart was clicked.
fn restart_clicked(ctx: &mut Context<'_>) -> bool {
    let pointer = Pointer::from_input(ctx.input());
    let viewport = ctx.viewport_size();
    ui::update_buttons(ctx.world_mut(), pointer, viewport);
    ctx.world()
        .query::<(&Button, &Restart)>()
        .iter()
        .any(|(button, _)| button.clicked())
}
```

`ScreenSpace::TOP`, `BOTTOM`, `LEFT`, `RIGHT` and `CENTER` anchor to the middle of an edge or of the window.

## 10. Sound and music

WAV and OGG Vorbis files are loaded completely, then played as often as you like; sounds overlap. Without an audio
device the game runs silently.

```rust
use purplepie::Context;
use purplepie::audio::{PlaybackId, SoundId};

struct Sounds {
    blip: SoundId,
    music: Option<PlaybackId>,
}

fn load_sounds(ctx: &mut Context<'_>) -> purplepie::Result<Sounds> {
    let blip = ctx.load_sound("sounds/blip.wav")?;
    let music_file = ctx.load_sound("sounds/loop.ogg")?;
    let music = Some(ctx.loop_sound(music_file, 0.5)); // loops seamlessly until stopped
    Ok(Sounds { blip, music })
}

fn on_hit(ctx: &mut Context<'_>, sounds: &mut Sounds) {
    ctx.play_sound(sounds.blip, 1.0); // fire and forget; 1.0 = as recorded
    if let Some(music) = sounds.music.take() {
        ctx.stop_sound(music);
    }
    ctx.set_master_volume(0.8);
}
```

## 11. Scene files

`ctx.save_scene(path)` writes the drawable entities (with their transforms, quads, sprites, text, layers, screen
space, animations, velocities and buttons) to a readable RON file; `ctx.load_scene(path)` spawns them again. Textures
and fonts are stored by asset path, so the file keeps working when the game folder moves. Use it for levels and
menus that you build once and edit as text.

Your own components are saved too once you register them under a stable name. They need serde
(see section 1):

```rust
use purplepie::Context;
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Color, Quad};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Door {
    leads_to: String,
}

fn save_level(ctx: &mut Context<'_>) -> purplepie::Result<()> {
    // Register before saving or loading: files name the component.
    ctx.register_scene_component::<Door>("my_game::Door")?;
    ctx.world_mut().spawn((
        Transform2D::from_position(Vec2::new(200.0, 0.0)),
        Quad::new(Vec2::new(40.0, 80.0), Color::hex(0x8338EC)),
        Door { leads_to: "levels/two.ron".into() },
    ));
    // Relative to the assets folder; the folder `assets/levels/` must exist.
    ctx.save_scene("levels/one.ron")
}

fn load_level(ctx: &mut Context<'_>) -> purplepie::Result<()> {
    ctx.register_scene_component::<Door>("my_game::Door")?;
    let spawned = ctx.load_scene("levels/one.ron")?; // adds to the world; returns the new entities
    println!("{} entities", spawned.len());
    Ok(())
}
```

Saving writes every saveable entity in the world, so save from a world that holds only the level. Entities with
none of the components listed above (and no registered one) are skipped. Loading adds to the world; despawn the old
entities first to replace a level.

## 12. Hot reload, errors and logs

**Hot reload.** With `EngineConfig::with_hot_reload(true)`, saving a texture, font or sound file while the game runs
replaces it within about half a second, under the same id. Turn it off for shipped games.

**Errors.** Every engine failure is a `purplepie::Error`: `Asset` (a file is missing or broken, with the full path the
engine tried), `Save` (a scene could not be written), `Render` (the GPU failed), `Game` (your own error returned
from `init`) and a few start-up errors. `init` returning an error ends the game, and
`Engine::run` returns it, so `?` in `main` prints it. To show it in a friendlier form, print its `Display` text:

```rust no_run
use purplepie::{Engine, EngineConfig, Game};

struct MyGame;
impl Game for MyGame {}

fn main() {
    let result = Engine::new(EngineConfig::new("My Game")).and_then(|engine| engine.run(MyGame));
    if let Err(error) = result {
        eprintln!("my_game: {error}");
        std::process::exit(1);
    }
}
```

**Logs.** The engine reports what it is doing through the [`log`](https://docs.rs/log) crate: the asset folder it
chose, the GPU, the audio device, hot reloads and warnings. Nothing is printed until your game installs a logger. For
example add `env_logger = "0.11"` to `Cargo.toml`, call `env_logger::init();` first thing in `main`, and run with
`RUST_LOG=info cargo run` (PowerShell: `$env:RUST_LOG="info"; cargo run`).

## 13. Shipping

```text
cargo build --release
```

Copy `target/release/my_game` (`my_game.exe` on Windows) and your `assets/` folder side by side into a new folder
and ship that folder. PurplePie looks for `assets/` next to the executable first, then in the working directory (which
is what `cargo run` uses). `EngineConfig::with_asset_root(path)` sets the folder explicitly. Ship `fonts/OFL.txt`
with the font.

## 14. Putting it together

A complete little game: move the ball with the arrows or A/D, touch the bricks to score, and click Restart (or
press R) to start over; an animated sprite plays in the corner. It uses only files copied in section 1. Paste it into
`src/main.rs` and `cargo run`. Together with section 1, this is the starter template: grow your own game from it.

```rust no_run
use purplepie::audio::SoundId;
use purplepie::ecs::Entity;
use purplepie::input::KeyCode;
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{
    self, Color, Layer, Quad, ScreenSpace, Sprite, SpriteAnimation, SpriteGrid, Text, TextAnchor,
    TextureId, TextureOptions,
};
use purplepie::ui::{self, Button, Pointer};
use purplepie::{Context, Engine, EngineConfig, Game};

const ARENA: Vec2 = Vec2::new(760.0, 520.0);
const PLAYER_SIZE: f32 = 40.0;
const BRICK_SIZE: Vec2 = Vec2::new(54.0, 18.0);

struct Player {
    speed: f32,
}
struct Brick;
struct ScoreLabel;
struct RestartButton;

struct Collector {
    brick: Option<TextureId>,
    blip: Option<SoundId>,
    score: u32,
    seed: u32,
}

impl Collector {
    /// A tiny pseudo-random number in 0..1 (the engine has no random numbers yet).
    fn random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.seed >> 8) as f32 / (1u32 << 24) as f32
    }

    fn spawn_brick(&mut self, ctx: &mut Context<'_>) {
        let Some(texture) = self.brick else { return };
        let half = ARENA / 2.0 - BRICK_SIZE;
        let at = Vec2::new((self.random() * 2.0 - 1.0) * half.x, (self.random() * 2.0 - 1.0) * half.y);
        ctx.world_mut()
            .spawn((Transform2D::from_position(at), Sprite::new(texture, BRICK_SIZE), Brick));
    }

    fn set_score(&mut self, ctx: &mut Context<'_>, score: u32) {
        self.score = score;
        for (text, _) in ctx.world_mut().query_mut::<(&mut Text, &ScoreLabel)>() {
            text.content = format!("Score {score}");
        }
    }
}

impl Game for Collector {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        let ball = ctx.load_texture_with("textures/breakout/ball.png", TextureOptions::LINEAR)?;
        let sheet = ctx.load_texture("textures/sandbox_sheet.png")?;
        let font = ctx.load_font("fonts/Poppins-Regular.ttf")?;
        self.brick = Some(ctx.load_texture("textures/breakout/brick.png")?);
        self.blip = Some(ctx.load_sound("sounds/blip.wav")?);

        let world = ctx.world_mut();
        world.spawn((Transform2D::default(), Quad::new(ARENA, Color::hex(0x1D3557)), Layer(-10)));
        world.spawn((
            Transform2D::default(),
            Sprite::new(ball, Vec2::splat(PLAYER_SIZE)),
            Player { speed: 320.0 },
            Layer(1),
        ));
        world.spawn((
            Transform2D::from_position(Vec2::new(16.0, -16.0)),
            Text::new("Score 0", font, 24.0).with_anchor(TextAnchor::TOP_LEFT),
            ScreenSpace::TOP_LEFT,
            ScoreLabel,
        ));
        world.spawn((
            Transform2D::from_position(Vec2::new(-40.0, 40.0)),
            Sprite::new(sheet, Vec2::splat(48.0)),
            SpriteAnimation::new(SpriteGrid::new(8, 8, 4, 2), 0, 7, 6.0),
            ScreenSpace::BOTTOM_RIGHT,
        ));
        let button = Transform2D::from_position(Vec2::new(-90.0, -36.0));
        let size = Vec2::new(140.0, 40.0);
        world.spawn((button, Quad::new(size, Color::hex(0xE76F51)), Button::new(size), ScreenSpace::TOP_RIGHT, RestartButton));
        world.spawn((
            button,
            Text::new("Restart", font, 18.0).with_anchor(TextAnchor::CENTER),
            ScreenSpace::TOP_RIGHT,
            Layer(1),
        ));
        for _ in 0..3 {
            self.spawn_brick(ctx);
        }
        Ok(())
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        let dt = ctx.dt();
        let input = ctx.input();
        let direction = Vec2::new(
            input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight) + input.axis(KeyCode::A, KeyCode::D),
            input.axis(KeyCode::ArrowDown, KeyCode::ArrowUp) + input.axis(KeyCode::S, KeyCode::W),
        )
        .clamp(Vec2::splat(-1.0), Vec2::ONE);

        let mut player_at = Vec2::ZERO;
        let limit = (ARENA - Vec2::splat(PLAYER_SIZE)) / 2.0;
        for (transform, player) in ctx.world_mut().query_mut::<(&mut Transform2D, &Player)>() {
            transform.position = (transform.position + direction * player.speed * dt).clamp(-limit, limit);
            player_at = transform.position;
        }
        render::advance_animations(ctx.world_mut(), dt);

        // Bricks the player touches (rectangle overlap).
        let reach = (Vec2::splat(PLAYER_SIZE) + BRICK_SIZE) / 2.0;
        let touched: Vec<Entity> = ctx
            .world()
            .query::<(Entity, &Transform2D, &Brick)>()
            .iter()
            .filter(|(_, t, _)| {
                let d = (t.position - player_at).abs();
                d.x < reach.x && d.y < reach.y
            })
            .map(|(e, _, _)| e)
            .collect();
        for brick in touched {
            let _ = ctx.world_mut().despawn(brick);
            if let Some(blip) = self.blip {
                ctx.play_sound(blip, 0.8);
            }
            let score = self.score + 1;
            self.set_score(ctx, score);
            self.spawn_brick(ctx);
        }
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        let pointer = Pointer::from_input(ctx.input());
        let viewport = ctx.viewport_size();
        ui::update_buttons(ctx.world_mut(), pointer, viewport);
        let restart = ctx
            .world()
            .query::<(&Button, &RestartButton)>()
            .iter()
            .any(|(button, _)| button.clicked());
        if restart || ctx.input().just_pressed(KeyCode::R) {
            self.set_score(ctx, 0);
            for (transform, _) in ctx.world_mut().query_mut::<(&mut Transform2D, &Player)>() {
                transform.position = Vec2::ZERO;
            }
        }
    }
}

fn main() -> purplepie::Result<()> {
    let config = EngineConfig::new("Collector")
        .with_size(800, 600)
        .with_clear_color(Color::hex(0x10101A));
    Engine::new(config)?.run(Collector { brick: None, blip: None, score: 0, seed: 7 })
}
```

Where to go next: `examples/breakout.rs` in the PurplePie folder is a full game with levels, lives, a HUD and
sound; `examples/scene.rs` shows scene files with game components. The API reference
(`cargo doc -p purplepie --no-deps --open`) documents every type and method.

//! Breakout, built only on PurplePie's public API (PP-012).
//!
//! Run with `cargo run --example breakout` from the project folder (textures
//! are loaded from `assets/textures/breakout/`).
//!
//! Controls: ←/→ or A/D move the paddle, or move the mouse; Space, ↑ or a left
//! click launches the ball and restarts after a win or loss; Escape quits.
//!
//! Everything game-specific lives here: paddle and ball physics, collisions
//! (axis-aligned boxes), bricks, lives, score, and the win/lose screens. There
//! is no text rendering yet, so the HUD is drawn with sprites and quads (lives
//! as small balls, a progress bar for cleared bricks), the score and lives are
//! in the window title, and events are printed to the console.
//!
//! `PURPLEPIE_BREAKOUT_AUTOPLAY=win` lets a simple bot play until the board is
//! cleared; `=lose` parks the paddle until all lives are gone. Both print a
//! summary, keep the end screen up for two seconds and exit. The simulation runs only in `fixed_update` with a seeded
//! random generator, so an autoplay game is identical on every run.

use purplepie::ecs::{self, Entity, Velocity, World};
use purplepie::input::{KeyCode, MouseButton};
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Camera2D, Color, Hidden, Layer, Quad, Sprite, TextureId};
use purplepie::{Context, Engine, EngineConfig, Game};

// --- Playfield (world units; the camera is zoomed to fit it into the window) ---

/// The area the ball moves in: x in ±FIELD.x / 2, y in ±FIELD.y / 2.
const FIELD: Vec2 = Vec2::new(800.0, 600.0);
const WALL: f32 = 16.0;
/// Extra world space shown around the walls (HUD above the top wall).
const MARGIN: Vec2 = Vec2::new(40.0, 90.0);

const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 16.0);
const PADDLE_Y: f32 = -FIELD.y / 2.0 + 40.0;
const PADDLE_SPEED: f32 = 600.0;

const BALL_SIZE: f32 = 16.0;
const BALL_START_SPEED: f32 = 360.0;
const BALL_SPEED_UP: f32 = 6.0;
const BALL_MAX_SPEED: f32 = 720.0;
/// Largest launch / bounce angle from vertical, in radians.
const MAX_BOUNCE_ANGLE: f32 = 1.05; // ≈ 60°

const BRICK_SIZE: Vec2 = Vec2::new(72.0, 24.0);
const BRICK_GAP: f32 = 6.0;
const BRICK_COLUMNS: usize = 10;
const BRICK_ROWS: usize = 6;
const BRICK_TOP_Y: f32 = FIELD.y / 2.0 - 50.0;
/// Row colours and points, top row first.
const ROWS: [(u32, u32); BRICK_ROWS] = [
    (0xE63946, 7),
    (0xF4A261, 5),
    (0xE9C46A, 4),
    (0x2A9D8F, 3),
    (0x3A86FF, 2),
    (0x8338EC, 1),
];

const LIVES: u32 = 3;
const TOTAL_BRICKS: usize = BRICK_COLUMNS * BRICK_ROWS;
/// Autoplay gives up after this many fixed steps (10 simulated minutes).
const AUTOPLAY_STEP_LIMIT: u64 = 60 * 60 * 10;

// --- Components (plain data, owned by the ECS world) ---

/// Marks the paddle.
struct Paddle;
/// Marks the ball. Its motion is the engine's `Velocity`.
struct Ball;
/// A brick worth `points`.
struct Brick {
    points: u32,
}
/// Everything that belongs to one round and is removed on restart.
struct RoundEntity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// The ball rests on the paddle until launched.
    Serve,
    Playing,
    Won,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Autoplay {
    Off,
    Win,
    Lose,
}

/// A tiny deterministic random generator (no dependency needed).
struct Lcg(u64);

impl Lcg {
    /// A value in `[-1, 1)`.
    fn next_signed(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
    }
}

struct Textures {
    ball: TextureId,
    brick: TextureId,
}

struct Breakout {
    autoplay: Autoplay,
    rng: Lcg,
    textures: Option<Textures>,
    state: State,
    lives: u32,
    score: u32,
    bricks_left: usize,
    ball_speed: f32,
    /// Where the autoplay bot aims to hit the ball on the paddle (−1..1).
    bot_aim: f32,
    /// The cursor's world x when last seen, to follow the mouse only when it moves.
    last_cursor_x: Option<f32>,
    paddle: Option<Entity>,
    ball: Option<Entity>,
    hud: Vec<Entity>,
    progress: Option<Entity>,
    overlay: Option<Entity>,
    /// Autoplay: the fixed step at which the game ended.
    finished_at: Option<u64>,
    /// The last window title set, to update it only when it changes.
    title: Option<String>,
}

impl Breakout {
    fn new(autoplay: Autoplay) -> Self {
        Self {
            autoplay,
            rng: Lcg(0x5EED_1234),
            textures: None,
            state: State::Serve,
            lives: LIVES,
            score: 0,
            bricks_left: TOTAL_BRICKS,
            ball_speed: BALL_START_SPEED,
            bot_aim: 0.0,
            last_cursor_x: None,
            paddle: None,
            ball: None,
            hud: Vec::new(),
            progress: None,
            overlay: None,
            finished_at: None,
            title: None,
        }
    }

    /// Removes the previous round (if any) and spawns paddle, ball and bricks.
    fn start_round(&mut self, world: &mut World) {
        let Some(textures) = &self.textures else {
            return;
        };
        let old: Vec<Entity> = world
            .query::<(Entity, &RoundEntity)>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for entity in old {
            let _ = world.despawn(entity);
        }

        self.state = State::Serve;
        self.lives = LIVES;
        self.score = 0;
        self.bricks_left = TOTAL_BRICKS;
        self.ball_speed = BALL_START_SPEED;

        self.paddle = Some(world.spawn((
            Transform2D::from_position(Vec2::new(0.0, PADDLE_Y)),
            Quad::new(PADDLE_SIZE, Color::hex(0xF1FAEE)),
            Paddle,
            RoundEntity,
        )));
        self.ball = Some(world.spawn((
            Transform2D::default(),
            Sprite::new(textures.ball, Vec2::splat(BALL_SIZE)),
            Velocity(Vec2::ZERO),
            Ball,
            RoundEntity,
            Layer(1),
        )));

        let row_width = BRICK_COLUMNS as f32 * (BRICK_SIZE.x + BRICK_GAP) - BRICK_GAP;
        for (row, &(color, points)) in ROWS.iter().enumerate() {
            for column in 0..BRICK_COLUMNS {
                let x = -row_width / 2.0
                    + BRICK_SIZE.x / 2.0
                    + column as f32 * (BRICK_SIZE.x + BRICK_GAP);
                let y = BRICK_TOP_Y - row as f32 * (BRICK_SIZE.y + BRICK_GAP);
                world.spawn((
                    Transform2D::from_position(Vec2::new(x, y)),
                    Sprite::new(textures.brick, BRICK_SIZE).with_tint(Color::hex(color)),
                    Brick { points },
                    RoundEntity,
                ));
            }
        }
        self.place_ball_on_paddle(world);
    }

    fn position(world: &World, entity: Option<Entity>) -> Vec2 {
        entity
            .and_then(|e| world.get::<&Transform2D>(e).ok().map(|t| t.position))
            .unwrap_or_default()
    }

    fn set_position(world: &mut World, entity: Option<Entity>, position: Vec2) {
        if let Some(mut t) = entity.and_then(|e| world.get::<&mut Transform2D>(e).ok()) {
            t.position = position;
        }
    }

    fn set_ball_velocity(&self, world: &mut World, velocity: Vec2) {
        if let Some(mut v) = self.ball.and_then(|e| world.get::<&mut Velocity>(e).ok()) {
            v.0 = velocity;
        }
    }

    fn place_ball_on_paddle(&mut self, world: &mut World) {
        let paddle = Self::position(world, self.paddle);
        let on_top = Vec2::new(paddle.x, PADDLE_Y + (PADDLE_SIZE.y + BALL_SIZE) / 2.0 + 1.0);
        Self::set_position(world, self.ball, on_top);
        self.set_ball_velocity(world, Vec2::ZERO);
    }

    /// Velocity at `angle` from straight up (positive = to the right).
    fn velocity_at(&self, angle: f32) -> Vec2 {
        Vec2::new(angle.sin(), angle.cos()) * self.ball_speed
    }

    fn launch(&mut self, world: &mut World) {
        let angle = self.rng.next_signed() * MAX_BOUNCE_ANGLE * 0.5;
        let velocity = self.velocity_at(angle);
        self.set_ball_velocity(world, velocity);
        self.state = State::Playing;
        println!("breakout: launch at {:.1}°", angle.to_degrees());
    }

    /// Moves the paddle from keys, mouse or the bot, clamped to the field.
    fn move_paddle(&mut self, ctx: &mut Context<'_>, dt: f32) {
        let mut x = Self::position(ctx.world(), self.paddle).x;
        match self.autoplay {
            Autoplay::Off => {
                let input = ctx.input();
                let keys = input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight)
                    + input.axis(KeyCode::A, KeyCode::D);
                x += keys.clamp(-1.0, 1.0) * PADDLE_SPEED * dt;
                // The mouse takes over only while it moves, so keys keep working.
                let cursor_x = ctx.cursor_world().map(|c| c.x);
                if let Some(cx) = cursor_x
                    && cursor_x != self.last_cursor_x
                {
                    x = cx;
                }
                self.last_cursor_x = cursor_x;
            }
            Autoplay::Win => {
                let ball = Self::position(ctx.world(), self.ball);
                let target = ball.x - self.bot_aim * PADDLE_SIZE.x * 0.4;
                x += (target - x).clamp(-PADDLE_SPEED * dt, PADDLE_SPEED * dt);
            }
            Autoplay::Lose => x = -FIELD.x, // parked against the left wall
        }
        let limit = (FIELD.x - PADDLE_SIZE.x) / 2.0;
        Self::set_position(
            ctx.world_mut(),
            self.paddle,
            Vec2::new(x.clamp(-limit, limit), PADDLE_Y),
        );
    }

    /// Bounces the ball off walls, paddle and bricks after it has moved.
    fn collide(&mut self, world: &mut World) {
        let Some(ball) = self.ball else { return };
        let mut position = Self::position(world, Some(ball));
        let mut velocity = world
            .get::<&Velocity>(ball)
            .map(|v| v.0)
            .unwrap_or_default();
        let half = BALL_SIZE / 2.0;

        // Walls (left, right, top).
        let (left, right, top) = (
            -FIELD.x / 2.0 + half,
            FIELD.x / 2.0 - half,
            FIELD.y / 2.0 - half,
        );
        if position.x < left {
            position.x = left;
            velocity.x = velocity.x.abs();
        } else if position.x > right {
            position.x = right;
            velocity.x = -velocity.x.abs();
        }
        if position.y > top {
            position.y = top;
            velocity.y = -velocity.y.abs();
        }

        // Paddle: the bounce angle depends on where the ball hits it.
        let paddle = Self::position(world, self.paddle);
        if velocity.y < 0.0
            && overlap(position, Vec2::splat(BALL_SIZE), paddle, PADDLE_SIZE).is_some()
        {
            let offset = ((position.x - paddle.x) / (PADDLE_SIZE.x / 2.0)).clamp(-1.0, 1.0);
            velocity = self.velocity_at(offset * MAX_BOUNCE_ANGLE);
            position.y = PADDLE_Y + (PADDLE_SIZE.y + BALL_SIZE) / 2.0;
            self.bot_aim = self.rng.next_signed();
        }

        // Bricks: break the first one touched and reflect on the shallower axis.
        let hit = world
            .query::<(Entity, &Transform2D, &Brick)>()
            .iter()
            .find_map(|(entity, t, brick)| {
                overlap(position, Vec2::splat(BALL_SIZE), t.position, BRICK_SIZE)
                    .map(|depth| (entity, brick.points, depth))
            });
        if let Some((brick, points, depth)) = hit {
            if depth.x < depth.y {
                velocity.x = -velocity.x;
            } else {
                velocity.y = -velocity.y;
            }
            let _ = world.despawn(brick);
            self.score += points;
            self.bricks_left -= 1;
            self.ball_speed = (self.ball_speed + BALL_SPEED_UP).min(BALL_MAX_SPEED);
            velocity = velocity.normalize_or_zero() * self.ball_speed;
            if self.bricks_left == 0 {
                self.state = State::Won;
                velocity = Vec2::ZERO;
                println!("breakout: board cleared! score {}", self.score);
            }
        }

        Self::set_position(world, Some(ball), position);
        self.set_ball_velocity(world, velocity);

        // Fell past the paddle.
        if position.y < -FIELD.y / 2.0 - BALL_SIZE {
            self.lives -= 1;
            println!("breakout: ball lost, lives left: {}", self.lives);
            if self.lives == 0 {
                self.state = State::Lost;
                self.set_ball_velocity(world, Vec2::ZERO);
                Self::set_position(world, Some(ball), Vec2::new(0.0, -FIELD.y));
            } else {
                self.state = State::Serve;
                self.place_ball_on_paddle(world);
            }
        }
    }

    /// Lives as small balls, cleared bricks as a bar, and the win/lose overlay.
    fn update_hud(&mut self, world: &mut World) {
        let Some(textures) = &self.textures else {
            return;
        };
        let hud_y = FIELD.y / 2.0 + WALL + 30.0;
        while self.hud.len() < self.lives as usize {
            let i = self.hud.len() as f32;
            let x = -FIELD.x / 2.0 + 12.0 + i * 28.0;
            self.hud.push(world.spawn((
                Transform2D::from_position(Vec2::new(x, hud_y)),
                Sprite::new(textures.ball, Vec2::splat(20.0)),
                Layer(10),
            )));
        }
        while self.hud.len() > self.lives as usize {
            if let Some(entity) = self.hud.pop() {
                let _ = world.despawn(entity);
            }
        }

        let bar_width = 300.0;
        let done = (TOTAL_BRICKS - self.bricks_left) as f32 / TOTAL_BRICKS as f32;
        let bar = self.progress.get_or_insert_with(|| {
            world.spawn((
                Transform2D::default(),
                Quad::new(Vec2::ZERO, Color::hex(0xE9C46A)),
                Layer(10),
            ))
        });
        if let Ok((t, quad)) = world.query_one_mut::<(&mut Transform2D, &mut Quad)>(*bar) {
            quad.size = Vec2::new(bar_width * done, 12.0);
            t.position = Vec2::new(FIELD.x / 2.0 - bar_width + quad.size.x / 2.0, hud_y);
        }

        let overlay_color = match self.state {
            State::Won => Some(Color::rgba(0.2, 0.8, 0.4, 0.45)),
            State::Lost => Some(Color::rgba(0.9, 0.2, 0.2, 0.45)),
            State::Serve | State::Playing => None,
        };
        // One overlay entity, shown or hidden with the `Hidden` marker.
        let overlay = *self.overlay.get_or_insert_with(|| {
            world.spawn((
                Transform2D::default(),
                Quad::new(FIELD, Color::TRANSPARENT),
                Layer(20),
                Hidden,
            ))
        });
        match overlay_color {
            Some(color) => {
                if let Ok(mut quad) = world.get::<&mut Quad>(overlay) {
                    quad.color = color;
                }
                let _ = world.remove_one::<Hidden>(overlay);
            }
            None => {
                let _ = world.insert_one(overlay, Hidden);
            }
        }
    }

    fn summary(&self, ctx: &Context<'_>) -> String {
        format!(
            "{:?} after {} fixed steps; bricks {}/{}; lives {}; score {}",
            self.state,
            ctx.time().fixed_steps(),
            TOTAL_BRICKS - self.bricks_left,
            TOTAL_BRICKS,
            self.lives,
            self.score
        )
    }
}

/// How deep two centred boxes overlap on each axis, or `None` if they don't.
fn overlap(a: Vec2, a_size: Vec2, b: Vec2, b_size: Vec2) -> Option<Vec2> {
    let depth = (a_size + b_size) / 2.0 - (a - b).abs();
    (depth.x > 0.0 && depth.y > 0.0).then_some(depth)
}

impl Game for Breakout {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        self.textures = Some(Textures {
            ball: ctx.load_texture("textures/breakout/ball.png")?,
            brick: ctx.load_texture("textures/breakout/brick.png")?,
        });
        let world = ctx.world_mut();
        // Walls: left, right, top.
        let wall = Color::hex(0x5A189A);
        for (position, size) in [
            (
                Vec2::new(-(FIELD.x + WALL) / 2.0, 0.0),
                Vec2::new(WALL, FIELD.y + 2.0 * WALL),
            ),
            (
                Vec2::new((FIELD.x + WALL) / 2.0, 0.0),
                Vec2::new(WALL, FIELD.y + 2.0 * WALL),
            ),
            (
                Vec2::new(0.0, (FIELD.y + WALL) / 2.0),
                Vec2::new(FIELD.x, WALL),
            ),
        ] {
            world.spawn((Transform2D::from_position(position), Quad::new(size, wall)));
        }
        self.start_round(world);
        Ok(())
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        let dt = ctx.dt();
        let input = ctx.input();
        let action = input.just_pressed(KeyCode::Space)
            || input.just_pressed(KeyCode::ArrowUp)
            || input.mouse_just_pressed(MouseButton::Left)
            || self.autoplay != Autoplay::Off;

        match self.state {
            State::Serve => {
                self.move_paddle(ctx, dt);
                self.place_ball_on_paddle(ctx.world_mut());
                if action {
                    self.launch(ctx.world_mut());
                }
            }
            State::Playing => {
                self.move_paddle(ctx, dt);
                ecs::integrate_velocity(ctx.world_mut(), dt);
                self.collide(ctx.world_mut());
            }
            State::Won | State::Lost => {
                if self.autoplay == Autoplay::Off && action {
                    self.start_round(ctx.world_mut());
                    println!("breakout: new game");
                }
            }
        }
        self.update_hud(ctx.world_mut());

        if self.autoplay != Autoplay::Off {
            let finished = matches!(self.state, State::Won | State::Lost);
            let steps = ctx.time().fixed_steps();
            if (finished || steps >= AUTOPLAY_STEP_LIMIT) && self.finished_at.is_none() {
                println!("breakout: autoplay finished: {}", self.summary(ctx));
                self.finished_at = Some(steps);
            }
            // Keep the end screen up for two seconds, then quit.
            if self.finished_at.is_some_and(|at| steps >= at + 120) {
                ctx.request_exit();
            }
        }
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        // Fit the walls and the HUD into the window, whatever its size.
        let needed = FIELD + 2.0 * Vec2::splat(WALL) + 2.0 * MARGIN;
        let center = Vec2::new(0.0, MARGIN.y / 2.0 - 10.0);
        *ctx.camera_mut() = Camera2D::fit(center, needed, ctx.viewport_size());

        // No text rendering yet: show score and lives in the window title.
        let title = format!(
            "PurplePie Breakout - score {} - lives {}{}",
            self.score,
            self.lives,
            match self.state {
                State::Won => " - you win! (Space to play again)",
                State::Lost => " - game over (Space to play again)",
                State::Serve | State::Playing => "",
            }
        );
        if self.title.as_deref() != Some(title.as_str()) {
            ctx.set_window_title(title.clone());
            self.title = Some(title);
        }
    }
}

fn main() -> purplepie::Result<()> {
    let autoplay = match std::env::var("PURPLEPIE_BREAKOUT_AUTOPLAY").as_deref() {
        Ok("win") => Autoplay::Win,
        Ok("lose") => Autoplay::Lose,
        _ => Autoplay::Off,
    };
    let config = EngineConfig::new("PurplePie Breakout")
        .with_size(1024, 768)
        .with_clear_color(Color::hex(0x10002B));
    Engine::new(config)?.run(Breakout::new(autoplay))
}

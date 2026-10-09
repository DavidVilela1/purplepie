//! Purple Swarm: a top-down arena shooter built on PurplePie's public API only
//! (docs/GAME2.md). This is part 1 (PP-036c1): the arena, the player, the
//! camera and shooting. Enemies, waves and scoring follow in PP-036c2.
//!
//! Run it from this folder, so its own `assets/` is found:
//!
//! ```text
//! cd games/purple-swarm
//! cargo run
//! ```
//!
//! WASD or the arrows move, the mouse aims, the left button shoots, Escape
//! quits. `PURPLE_SWARM_AUTOPLAY=1` lets a bot circle the arena and shoot,
//! for automated checks.

use purplepie::audio::SoundId;
use purplepie::ecs::{self, Entity, Velocity};
use purplepie::input::{KeyCode, MouseButton};
use purplepie::math::{Rect, Transform2D, Vec2};
use purplepie::render::{
    Color, Layer, Quad, ScreenSpace, Sprite, Text, TextAnchor, TextureId, TextureOptions,
};
use purplepie::{Context, Engine, EngineConfig, Game};

/// The playing field, centred on the origin (world units).
const ARENA: Vec2 = Vec2::new(1600.0, 1200.0);
/// Size of one floor tile.
const TILE: f32 = 100.0;
/// Thickness of the walls drawn around the arena.
const WALL: f32 = 24.0;
const PLAYER_SIZE: f32 = 40.0;
const PLAYER_SPEED: f32 = 260.0;
const BULLET_SIZE: f32 = 12.0;
const BULLET_SPEED: f32 = 720.0;
/// Seconds between shots while the button is held (8 per second).
const FIRE_INTERVAL: f32 = 0.125;
const WINDOW: (u32, u32) = (960, 640);

/// The player's ship.
struct Player;

/// A bullet in flight.
struct Bullet;

/// The text that shows how many shots were fired.
struct ShotsLabel;

/// Textures and sounds, loaded once in `init`.
struct Assets {
    bullet: TextureId,
    shot: SoundId,
}

struct Swarm {
    assets: Option<Assets>,
    /// Seconds until the next shot is allowed.
    cooldown: f32,
    shots: u32,
    /// Simulated seconds (drives the autoplay bot).
    clock: f32,
    autoplay: bool,
}

impl Swarm {
    fn new(autoplay: bool) -> Self {
        Self {
            assets: None,
            cooldown: 0.0,
            shots: 0,
            clock: 0.0,
            autoplay,
        }
    }

    /// Movement direction and aim point for this step: from the keyboard and
    /// mouse, or from the bot.
    fn controls(&self, ctx: &Context<'_>, player_at: Vec2) -> (Vec2, Option<Vec2>, bool) {
        if self.autoplay {
            // Run around a circle of radius 350, aiming outward and firing.
            let t = self.clock * 0.6;
            let target = Vec2::new(t.cos(), t.sin()) * 350.0;
            let to_target = target - player_at;
            let direction = if to_target.length() > 8.0 {
                to_target.normalize()
            } else {
                Vec2::ZERO
            };
            let aim = player_at + Vec2::new((t * 3.0).cos(), (t * 3.0).sin()) * 200.0;
            return (direction, Some(aim), true);
        }
        let input = ctx.input();
        let direction = Vec2::new(
            input.axis(KeyCode::A, KeyCode::D)
                + input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight),
            input.axis(KeyCode::S, KeyCode::W) + input.axis(KeyCode::ArrowDown, KeyCode::ArrowUp),
        )
        .clamp(Vec2::splat(-1.0), Vec2::ONE)
        .normalize_or_zero();
        let firing = input.mouse_pressed(MouseButton::Left);
        (direction, ctx.cursor_world(), firing)
    }
}

/// Spawns the floor, the walls, the player and the HUD.
fn spawn_level(ctx: &mut Context<'_>) -> purplepie::Result<()> {
    let floor = ctx.load_texture("textures/floor.png")?;
    let player = ctx.load_texture_with("textures/player.png", TextureOptions::LINEAR)?;
    let font = ctx.load_font("fonts/Poppins-Regular.ttf")?;
    let world = ctx.world_mut();

    let (columns, rows) = ((ARENA.x / TILE) as u32, (ARENA.y / TILE) as u32);
    for ty in 0..rows {
        for tx in 0..columns {
            let at = -ARENA / 2.0 + (Vec2::new(tx as f32, ty as f32) + 0.5) * TILE;
            world.spawn((
                Transform2D::from_position(at),
                Sprite::new(floor, Vec2::splat(TILE)),
                Layer(-10),
            ));
        }
    }
    let wall_color = Color::hex(0x7B2CBF);
    let half = ARENA / 2.0 + WALL / 2.0;
    for (at, size) in [
        (
            Vec2::new(0.0, half.y),
            Vec2::new(ARENA.x + 2.0 * WALL, WALL),
        ),
        (
            Vec2::new(0.0, -half.y),
            Vec2::new(ARENA.x + 2.0 * WALL, WALL),
        ),
        (Vec2::new(half.x, 0.0), Vec2::new(WALL, ARENA.y)),
        (Vec2::new(-half.x, 0.0), Vec2::new(WALL, ARENA.y)),
    ] {
        world.spawn((
            Transform2D::from_position(at),
            Quad::new(size, wall_color),
            Layer(-5),
        ));
    }
    world.spawn((
        Transform2D::default(),
        Sprite::new(player, Vec2::splat(PLAYER_SIZE)),
        Player,
        Layer(5),
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(16.0, -16.0)),
        Text::new("PURPLE SWARM", font, 24.0)
            .with_color(Color::hex(0xC77DFF))
            .with_anchor(TextAnchor::TOP_LEFT),
        ScreenSpace::TOP_LEFT,
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(16.0, -48.0)),
        Text::new("Shots 0", font, 18.0).with_anchor(TextAnchor::TOP_LEFT),
        ScreenSpace::TOP_LEFT,
        ShotsLabel,
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(0.0, 16.0)),
        Text::new(
            "WASD move - mouse aim - hold left button to shoot - Esc quits",
            font,
            16.0,
        )
        .with_color(Color::rgba(1.0, 1.0, 1.0, 0.6))
        .with_anchor(TextAnchor::BOTTOM_CENTER),
        ScreenSpace::BOTTOM_CENTER,
    ));
    Ok(())
}

/// Keeps the camera on `target` but never shows space outside the arena
/// (when the arena is smaller than the view along an axis, it stays centred).
fn follow_camera(ctx: &mut Context<'_>, target: Vec2) {
    let viewport = ctx.viewport_size();
    let camera = ctx.camera_mut();
    let half_view = viewport / (2.0 * camera.effective_zoom());
    let room = (ARENA / 2.0 + WALL - half_view).max(Vec2::ZERO);
    camera.position = target.clamp(-room, room);
}

impl Game for Swarm {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        spawn_level(ctx)?;
        self.assets = Some(Assets {
            bullet: ctx.load_texture_with("textures/bullet.png", TextureOptions::LINEAR)?,
            shot: ctx.load_sound("sounds/shot.wav")?,
        });
        Ok(())
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        let dt = ctx.dt();
        self.clock += dt;
        self.cooldown = (self.cooldown - dt).max(0.0);

        // The player: move, stay inside the arena, face the aim point.
        let player_at = ctx
            .world()
            .query::<(&Transform2D, &Player)>()
            .iter()
            .map(|(t, _)| t.position)
            .next()
            .unwrap_or(Vec2::ZERO);
        let (direction, aim, firing) = self.controls(ctx, player_at);
        let limit = ARENA / 2.0 - Vec2::splat(PLAYER_SIZE / 2.0);
        let mut player_at = player_at;
        let mut facing = Vec2::X;
        for (t, _) in ctx.world_mut().query_mut::<(&mut Transform2D, &Player)>() {
            t.position = (t.position + direction * PLAYER_SPEED * dt).clamp(-limit, limit);
            if let Some(aim) = aim {
                let to_aim = aim - t.position;
                if to_aim.length_squared() > 1.0 {
                    t.rotation = to_aim.y.atan2(to_aim.x);
                }
            }
            player_at = t.position;
            facing = Vec2::from_angle(t.rotation);
        }
        follow_camera(ctx, player_at);

        // Shooting: one bullet per FIRE_INTERVAL while the button is held.
        if firing
            && self.cooldown <= 0.0
            && let Some(assets) = &self.assets
        {
            let (bullet, shot) = (assets.bullet, assets.shot);
            ctx.world_mut().spawn((
                Transform2D::from_position(player_at + facing * (PLAYER_SIZE / 2.0)),
                Sprite::new(bullet, Vec2::splat(BULLET_SIZE)),
                Velocity(facing * BULLET_SPEED),
                Bullet,
                Layer(4),
            ));
            ctx.play_sound(shot, 0.3);
            self.shots += 1;
            self.cooldown = FIRE_INTERVAL;
            let text = format!("Shots {}", self.shots);
            for (label, _) in ctx.world_mut().query_mut::<(&mut Text, &ShotsLabel)>() {
                label.content.clone_from(&text);
            }
        }

        // Move bullets; remove the ones that left the arena.
        ecs::integrate_velocity(ctx.world_mut(), dt);
        let arena = Rect::from_center_size(Vec2::ZERO, ARENA);
        let gone: Vec<Entity> = ctx
            .world()
            .query::<(Entity, &Transform2D, &Bullet)>()
            .iter()
            .filter(|(_, t, _)| !arena.contains(t.position))
            .map(|(e, _, _)| e)
            .collect();
        for e in gone {
            let _ = ctx.world_mut().despawn(e);
        }
    }
}

fn main() -> purplepie::Result<()> {
    let autoplay = std::env::var("PURPLE_SWARM_AUTOPLAY").is_ok_and(|v| v == "1");
    let config = EngineConfig::new("Purple Swarm")
        .with_size(WINDOW.0, WINDOW.1)
        .with_clear_color(Color::hex(0x0B0618))
        .with_console_log(true);
    Engine::new(config)?.run(Swarm::new(autoplay))
}

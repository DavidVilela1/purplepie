//! Purple Swarm: a top-down arena shooter built on PurplePie's public API only
//! (docs/GAME2.md). Parts 1–2 (PP-036c1/c2): arena, player, camera,
//! shooting, two enemy types in growing waves, collisions, hit points, score,
//! health orbs and sound. Title, pause and restart screens follow in PP-036d.
//!
//! Run it from this folder, so its own `assets/` is found:
//!
//! ```text
//! cd games/purple-swarm
//! cargo run
//! ```
//!
//! WASD or the arrows move, the mouse aims, the left button shoots, Escape
//! quits. `PURPLE_SWARM_AUTOPLAY=1` lets a bot play with a fixed random seed:
//! it prints a summary after 60 simulated seconds (or when it dies) and exits,
//! and two runs print the same line.

mod enemy;
mod level;

use purplepie::audio::{PlaybackId, SoundId};
use purplepie::ecs::{self, Entity, Velocity};
use purplepie::input::{KeyCode, MouseButton};
use purplepie::math::{Rng, Transform2D, Vec2, circles_overlap};
use purplepie::render::{Hidden, Layer, Sprite, TextureId, TextureOptions};
use purplepie::{Context, Engine, EngineConfig, Game};

use enemy::{Enemy, Kind};
use level::Hud;

const PLAYER_SIZE: f32 = 40.0;
const PLAYER_RADIUS: f32 = 15.0;
const PLAYER_SPEED: f32 = 260.0;
const MAX_HIT_POINTS: u32 = 5;
/// Seconds of invulnerability after being hit.
const INVULNERABLE: f32 = 0.5;
const BULLET_SIZE: f32 = 12.0;
const BULLET_SPEED: f32 = 720.0;
/// Seconds between shots while the button is held (8 per second).
const FIRE_INTERVAL: f32 = 0.125;
const ORB_SIZE: f32 = 20.0;
const ORB_LIFETIME: f32 = 8.0;
const ORB_DROP_CHANCE: f32 = 0.1;
/// Enemies never spawn closer than this to the player.
const SPAWN_DISTANCE: f32 = 300.0;
/// Fixed steps the autoplay bot plays before printing its summary (60 s).
const AUTOPLAY_STEPS: u64 = 60 * 60;
const AUTOPLAY_SEED: u64 = 2026;
/// How far the autoplay bot looks for targets.
const BOT_RANGE: f32 = 420.0;
const WINDOW: (u32, u32) = (960, 640);

/// The player's ship.
struct Player;

/// A bullet in flight.
struct Bullet;

/// A health pickup; disappears after `life` seconds.
struct Orb {
    life: f32,
}

/// Textures and sounds, loaded once in `init`.
struct Assets {
    bullet: TextureId,
    drifter: TextureId,
    dasher: TextureId,
    orb: TextureId,
    shot: SoundId,
    hit: SoundId,
    death: SoundId,
    game_over: SoundId,
    music: SoundId,
}

struct Swarm {
    assets: Option<Assets>,
    rng: Rng,
    autoplay: bool,
    /// Fixed steps simulated so far.
    steps: u64,
    /// Seconds until the next shot is allowed.
    cooldown: f32,
    hit_points: u32,
    /// Seconds of invulnerability left.
    invulnerable: f32,
    score: u32,
    kills: u32,
    shots: u32,
    wave: u32,
    /// Seconds until the next enemy spawns.
    spawn_timer: f32,
    over: bool,
    music: Option<PlaybackId>,
}

impl Swarm {
    fn new(autoplay: bool) -> Self {
        Self {
            assets: None,
            rng: if autoplay {
                Rng::new(AUTOPLAY_SEED)
            } else {
                Rng::from_entropy()
            },
            autoplay,
            steps: 0,
            cooldown: 0.0,
            hit_points: MAX_HIT_POINTS,
            invulnerable: 0.0,
            score: 0,
            kills: 0,
            shots: 0,
            wave: 1,
            spawn_timer: 1.0,
            over: false,
            music: None,
        }
    }

    fn seconds(&self, ctx: &Context<'_>) -> f32 {
        self.steps as f32 * ctx.dt()
    }

    /// Movement direction, aim point and trigger for this step: from the
    /// keyboard and mouse, or from the bot.
    fn controls(&self, ctx: &Context<'_>, player: Vec2) -> (Vec2, Option<Vec2>, bool) {
        if self.autoplay {
            // Run around a circle of radius 350 and shoot at the nearest enemy.
            let t = self.seconds(ctx) * 0.6;
            let target = Vec2::from_angle(t) * 350.0;
            let to_target = target - player;
            let direction = if to_target.length() > 8.0 {
                to_target.normalize()
            } else {
                Vec2::ZERO
            };
            // Like a player, it only sees enemies on screen (about BOT_RANGE away).
            let nearest = ctx
                .world()
                .query::<(&Transform2D, &Enemy)>()
                .iter()
                .map(|(t, _)| t.position)
                .filter(|at| at.distance(player) < BOT_RANGE)
                .min_by(|a, b| {
                    a.distance_squared(player)
                        .total_cmp(&b.distance_squared(player))
                });
            return match nearest {
                Some(aim) => (direction, Some(aim), true),
                None => (direction, Some(player + direction * 100.0), false),
            };
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

    fn summary(&self, ctx: &Context<'_>) -> String {
        format!(
            "{:.1} s; wave {}; score {}; kills {}; shots {}; hit points {}",
            self.seconds(ctx),
            self.wave,
            self.score,
            self.kills,
            self.shots,
            self.hit_points
        )
    }

    fn update_hud(&self, ctx: &mut Context<'_>) {
        let secs = self.seconds(ctx) as u32;
        let status = format!(
            "Score {}   Wave {}   {}:{:02}",
            self.score,
            self.wave,
            secs / 60,
            secs % 60
        );
        level::set_hud(ctx, Hud::Status, &status);
        let health = format!("HP {}/{}", self.hit_points, MAX_HIT_POINTS);
        level::set_hud(ctx, Hud::Health, &health);
    }

    /// Moves the player, turns it toward the aim point and fires.
    fn player_step(&mut self, ctx: &mut Context<'_>, dt: f32) -> Vec2 {
        let start = player_position(ctx);
        let (direction, aim, firing) = self.controls(ctx, start);
        let limit = level::ARENA / 2.0 - Vec2::splat(PLAYER_SIZE / 2.0);
        let (mut at, mut facing) = (start, Vec2::X);
        for (t, _) in ctx.world_mut().query_mut::<(&mut Transform2D, &Player)>() {
            t.position = (t.position + direction * PLAYER_SPEED * dt).clamp(-limit, limit);
            if let Some(aim) = aim {
                let to_aim = aim - t.position;
                if to_aim.length_squared() > 1.0 {
                    t.rotation = to_aim.y.atan2(to_aim.x);
                }
            }
            at = t.position;
            facing = Vec2::from_angle(t.rotation);
        }
        level::follow_camera(ctx, at);

        self.cooldown = (self.cooldown - dt).max(0.0);
        if firing
            && self.cooldown <= 0.0
            && let Some(assets) = &self.assets
        {
            let (bullet, shot) = (assets.bullet, assets.shot);
            ctx.world_mut().spawn((
                Transform2D::from_position(at + facing * (PLAYER_SIZE / 2.0)),
                Sprite::new(bullet, Vec2::splat(BULLET_SIZE)),
                Velocity(facing * BULLET_SPEED),
                Bullet,
                Layer(4),
            ));
            ctx.play_sound(shot, 0.25);
            self.shots += 1;
            self.cooldown = FIRE_INTERVAL;
        }
        at
    }

    /// Advances the wave clock and spawns enemies on the arena edge.
    fn spawn_step(&mut self, ctx: &mut Context<'_>, player: Vec2, dt: f32) {
        self.wave = 1 + (self.seconds(ctx) / enemy::WAVE_LENGTH) as u32;
        self.spawn_timer -= dt;
        if self.spawn_timer > 0.0 {
            return;
        }
        self.spawn_timer = enemy::spawn_interval(self.wave);
        let alive = ctx.world().query::<&Enemy>().iter().count();
        let Some(assets) = &self.assets else { return };
        if alive >= enemy::MAX_ENEMIES {
            return;
        }
        let kind = enemy::pick_kind(&mut self.rng, self.wave);
        let texture = match kind {
            Kind::Drifter => assets.drifter,
            Kind::Dasher => assets.dasher,
        };
        let at = level::edge_point(&mut self.rng, player, SPAWN_DISTANCE, kind.size());
        ctx.world_mut().spawn((
            Transform2D::from_position(at),
            Sprite::new(texture, Vec2::splat(kind.size())),
            Velocity(Vec2::ZERO),
            Enemy::new(kind),
            Layer(3),
        ));
    }

    /// Bullets against enemies: damage, kills, score and drops.
    fn bullet_hits(&mut self, ctx: &mut Context<'_>) {
        let bullets: Vec<(Entity, Vec2)> = ctx
            .world()
            .query::<(Entity, &Transform2D, &Bullet)>()
            .iter()
            .map(|(e, t, _)| (e, t.position))
            .collect();
        let mut spent = Vec::new();
        let mut dead = Vec::new();
        let mut wounded = false;
        for (enemy_entity, t, enemy) in ctx
            .world_mut()
            .query_mut::<(Entity, &Transform2D, &mut Enemy)>()
        {
            for (bullet, at) in &bullets {
                if enemy.hit_points == 0 || spent.contains(bullet) {
                    continue;
                }
                if circles_overlap(*at, BULLET_SIZE / 2.0, t.position, enemy.radius()) {
                    spent.push(*bullet);
                    enemy.hit_points -= 1;
                    enemy.flash = 0.1;
                    if enemy.hit_points == 0 {
                        dead.push((enemy_entity, t.position, enemy.kind));
                    } else {
                        wounded = true;
                    }
                }
            }
        }
        for bullet in spent {
            let _ = ctx.world_mut().despawn(bullet);
        }
        let Some(assets) = &self.assets else { return };
        let (death, orb) = (assets.death, assets.orb);
        if wounded {
            ctx.play_sound(assets.hit, 0.25);
        }
        for (entity, at, kind) in dead {
            let _ = ctx.world_mut().despawn(entity);
            self.score += kind.points();
            self.kills += 1;
            ctx.play_sound(death, 0.5);
            if self.rng.chance(ORB_DROP_CHANCE) {
                ctx.world_mut().spawn((
                    Transform2D::from_position(at),
                    Sprite::new(orb, Vec2::splat(ORB_SIZE)),
                    Orb { life: ORB_LIFETIME },
                    Layer(2),
                ));
            }
        }
    }

    /// Enemies touching the player cost a hit point; orbs heal.
    fn player_contacts(&mut self, ctx: &mut Context<'_>, player: Vec2, dt: f32) {
        self.invulnerable = (self.invulnerable - dt).max(0.0);
        let touched = ctx
            .world()
            .query::<(&Transform2D, &Enemy)>()
            .iter()
            .any(|(t, e)| circles_overlap(player, PLAYER_RADIUS, t.position, e.radius()));
        if touched && self.invulnerable <= 0.0 && self.hit_points > 0 {
            self.hit_points -= 1;
            self.invulnerable = INVULNERABLE;
            if let Some(assets) = &self.assets {
                ctx.play_sound(assets.hit, 1.0);
            }
        }

        let mut taken = Vec::new();
        for (e, t, orb) in ctx
            .world_mut()
            .query_mut::<(Entity, &Transform2D, &mut Orb)>()
        {
            orb.life -= dt;
            if circles_overlap(player, PLAYER_RADIUS, t.position, ORB_SIZE / 2.0) {
                taken.push((e, true));
            } else if orb.life <= 0.0 {
                taken.push((e, false));
            }
        }
        for (e, healed) in taken {
            let _ = ctx.world_mut().despawn(e);
            if healed {
                self.hit_points = (self.hit_points + 1).min(MAX_HIT_POINTS);
            }
        }

        // Blink while invulnerable: hidden for 0.06 s out of every 0.12 s.
        let blink = self.invulnerable > 0.0 && (self.invulnerable / 0.06) as u32 % 2 == 1;
        let player_entity = ctx
            .world()
            .query::<(Entity, &Player)>()
            .iter()
            .map(|(e, _)| e)
            .next();
        if let Some(e) = player_entity {
            let world = ctx.world_mut();
            let hidden = world.get::<&Hidden>(e).is_ok();
            if blink && !hidden {
                let _ = world.insert_one(e, Hidden);
            } else if !blink && hidden {
                let _ = world.remove_one::<Hidden>(e);
            }
        }
    }

    fn game_over(&mut self, ctx: &mut Context<'_>) {
        self.over = true;
        if let Some(music) = self.music.take() {
            ctx.stop_sound(music);
        }
        if let Some(assets) = &self.assets {
            ctx.play_sound(assets.game_over, 0.8);
        }
        let banner = format!("GAME OVER - score {} - Esc quits", self.score);
        level::set_hud(ctx, Hud::Banner, &banner);
        println!("swarm: game over at {}", self.summary(ctx));
    }
}

/// Where the player is (the origin if it is missing).
fn player_position(ctx: &Context<'_>) -> Vec2 {
    ctx.world()
        .query::<(&Transform2D, &Player)>()
        .iter()
        .map(|(t, _)| t.position)
        .next()
        .unwrap_or(Vec2::ZERO)
}

/// Removes bullets that left the arena.
fn remove_stray_bullets(ctx: &mut Context<'_>) {
    let arena = level::arena();
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

/// Keeps enemies inside the arena (dashes stop at the wall).
fn keep_enemies_inside(ctx: &mut Context<'_>) {
    let half = level::ARENA / 2.0;
    for (t, enemy) in ctx.world_mut().query_mut::<(&mut Transform2D, &Enemy)>() {
        let limit = half - Vec2::splat(enemy.kind.size() / 2.0);
        t.position = t.position.clamp(-limit, limit);
    }
}

impl Game for Swarm {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        level::spawn(ctx)?;
        level::spawn_player(ctx, PLAYER_SIZE)?;
        let assets = Assets {
            bullet: ctx.load_texture_with("textures/bullet.png", TextureOptions::LINEAR)?,
            drifter: ctx.load_texture_with("textures/drifter.png", TextureOptions::LINEAR)?,
            dasher: ctx.load_texture_with("textures/dasher.png", TextureOptions::LINEAR)?,
            orb: ctx.load_texture_with("textures/orb.png", TextureOptions::LINEAR)?,
            shot: ctx.load_sound("sounds/shot.wav")?,
            hit: ctx.load_sound("sounds/hit.wav")?,
            death: ctx.load_sound("sounds/death.wav")?,
            game_over: ctx.load_sound("sounds/gameover.wav")?,
            music: ctx.load_sound("sounds/music.ogg")?,
        };
        self.music = Some(ctx.loop_sound(assets.music, 0.35));
        self.assets = Some(assets);
        self.update_hud(ctx);
        Ok(())
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        if self.over {
            return;
        }
        let dt = ctx.dt();
        self.steps += 1;

        let player = self.player_step(ctx, dt);
        self.spawn_step(ctx, player, dt);
        enemy::steer(ctx.world_mut(), player, self.wave, dt);
        ecs::integrate_velocity(ctx.world_mut(), dt);
        keep_enemies_inside(ctx);
        remove_stray_bullets(ctx);
        self.bullet_hits(ctx);
        self.player_contacts(ctx, player, dt);
        enemy::show_flashes(ctx.world_mut());
        self.update_hud(ctx);

        if self.hit_points == 0 {
            self.game_over(ctx);
            if self.autoplay {
                ctx.request_exit();
            }
        } else if self.autoplay && self.steps == AUTOPLAY_STEPS {
            println!("swarm: autoplay finished: {}", self.summary(ctx));
            ctx.request_exit();
        }
    }
}

fn main() -> purplepie::Result<()> {
    let autoplay = std::env::var("PURPLE_SWARM_AUTOPLAY").is_ok_and(|v| v == "1");
    let config = EngineConfig::new("Purple Swarm")
        .with_size(WINDOW.0, WINDOW.1)
        .with_clear_color(purplepie::render::Color::hex(0x0B0618))
        .with_console_log(true);
    Engine::new(config)?.run(Swarm::new(autoplay))
}

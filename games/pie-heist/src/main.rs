//! Pie Heist: a top-down heist whose levels are made in the scene editor,
//! built on PurplePie's public API only (docs/GAME3.md).
//!
//! So far (PP-042a): a level is loaded from its scene file, the player appears
//! at the level's start, moves with WASD or the arrows, stops at walls and
//! slides along them, and the camera follows inside the level. Pies, keys,
//! doors, exits and guards come next (PP-042b, PP-042c).
//!
//! Run it from this folder, so its own `assets/` is found:
//!
//! ```text
//! cd games/pie-heist
//! cargo run                          # levels/level1.ron
//! cargo run -- levels/level1.ron     # any level, relative to assets/
//! ```
//!
//! Levels are drawn with the game's editor: `cargo run -p pie-heist-editor --
//! assets/levels/level1.ron`. `PIE_HEIST_EXIT_AFTER_FRAMES=N` closes the game
//! after N frames and prints where the player is (for automated checks).

use purplepie::ecs::Entity;
use purplepie::input::KeyCode;
use purplepie::math::{Rect, Transform2D, Vec2};
use purplepie::render::{Color, Hidden, Layer, Quad, Sprite};
use purplepie::{Context, Engine, EngineConfig, Game};

use pie_heist::{PlayerStart, Wall, camera_target, level_bounds, move_and_slide, wall_rect};

const WINDOW: (u32, u32) = (960, 640);
const PLAYER_SIZE: f32 = 28.0;
const PLAYER_SPEED: f32 = 220.0;
const FIRST_LEVEL: &str = "levels/level1.ron";

/// The player's entity.
struct Player;

struct PieHeist {
    level: String,
    player: Option<Entity>,
    /// The walls' areas, collected when the level loads (walls do not move).
    walls: Vec<Rect>,
    bounds: Option<Rect>,
    exit_after: Option<u32>,
    frames: u32,
}

impl PieHeist {
    /// Loads `self.level` and puts the player at its start.
    fn load_level(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        ctx.world_mut().clear();
        let entities = ctx.load_scene(&self.level)?;
        let world = ctx.world_mut();

        self.walls.clear();
        let mut tilted = 0;
        for (t, q, _) in world.query_mut::<(&Transform2D, &Quad, &Wall)>() {
            match wall_rect(t, q) {
                Some(rect) => self.walls.push(rect),
                None => tilted += 1,
            }
        }
        if tilted > 0 {
            eprintln!(
                "pie-heist: warning: {tilted} rotated wall(s) in {} are ignored: walls must be axis-aligned",
                self.level
            );
        }
        self.bounds = level_bounds(&self.walls);

        let mut starts: Vec<(Entity, Vec2)> = world
            .query_mut::<(Entity, &Transform2D, &PlayerStart)>()
            .into_iter()
            .map(|(e, t, _)| (e, t.position))
            .collect();
        starts.sort_by_key(|(e, _)| e.id());
        let start = match starts.first() {
            Some(&(_, at)) => at,
            None => {
                eprintln!(
                    "pie-heist: warning: {} has no player start; starting at (0, 0)",
                    self.level
                );
                Vec2::ZERO
            }
        };
        if starts.len() > 1 {
            eprintln!(
                "pie-heist: warning: {} has {} player starts; using the first",
                self.level,
                starts.len()
            );
        }
        for (entity, _) in starts {
            let _ = world.insert_one(entity, Hidden);
        }

        let texture = ctx.load_texture("textures/player.png")?;
        let player = ctx.world_mut().spawn((
            Transform2D::from_position(start),
            Sprite::new(texture, Vec2::splat(PLAYER_SIZE)),
            Layer(10),
            Player,
        ));
        self.player = Some(player);
        println!(
            "pie-heist: {}: {} entities, {} walls; player at ({:.1}, {:.1})",
            self.level,
            entities.len(),
            self.walls.len(),
            start.x,
            start.y
        );
        Ok(())
    }

    fn player_position(&self, ctx: &Context<'_>) -> Option<Vec2> {
        let player = self.player?;
        ctx.world()
            .get::<&Transform2D>(player)
            .ok()
            .map(|t| t.position)
    }
}

impl Game for PieHeist {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        pie_heist::register(ctx)?;
        self.load_level(ctx)
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        let input = ctx.input();
        let direction = Vec2::new(
            input.axis(KeyCode::A, KeyCode::D)
                + input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight),
            input.axis(KeyCode::S, KeyCode::W) + input.axis(KeyCode::ArrowDown, KeyCode::ArrowUp),
        )
        .clamp(Vec2::splat(-1.0), Vec2::ONE)
        .normalize_or_zero();
        let delta = direction * PLAYER_SPEED * ctx.dt();
        let Some(player) = self.player else {
            return;
        };
        if let Ok(mut t) = ctx.world_mut().get::<&mut Transform2D>(player) {
            t.position = move_and_slide(t.position, Vec2::splat(PLAYER_SIZE), delta, &self.walls);
        }
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        if let (Some(at), Some(bounds)) = (self.player_position(ctx), self.bounds) {
            let half_view = ctx.viewport_size() / (2.0 * ctx.camera().effective_zoom());
            ctx.camera_mut().position = camera_target(at, half_view, bounds);
        }
        self.frames += 1;
        if self.exit_after.is_some_and(|frames| self.frames >= frames) {
            if let Some(at) = self.player_position(ctx) {
                println!("pie-heist: player at ({:.1}, {:.1})", at.x, at.y);
            }
            ctx.request_exit();
        }
    }
}

fn main() -> purplepie::Result<()> {
    let level = std::env::args()
        .nth(1)
        .unwrap_or_else(|| FIRST_LEVEL.to_owned());
    let exit_after = std::env::var("PIE_HEIST_EXIT_AFTER_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok());
    let config = EngineConfig::new("Pie Heist")
        .with_size(WINDOW.0, WINDOW.1)
        .with_clear_color(Color::hex(0x14101E))
        .with_console_log(true);
    Engine::new(config)?.run(PieHeist {
        level,
        player: None,
        walls: Vec::new(),
        bounds: None,
        exit_after,
        frames: 0,
    })
}

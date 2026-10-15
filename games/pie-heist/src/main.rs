//! Pie Heist: a top-down heist whose levels are made in the scene editor,
//! built on PurplePie's public API only (docs/GAME3.md).
//!
//! So far: a level is loaded from its scene file, the player appears at the
//! level's start, moves with WASD or the arrows, stops at walls and slides
//! along them, and the camera follows inside the level (PP-042a). Pies are
//! collected by touching them; once every pie of the level is collected the
//! exit opens, and touching it loads the next level, or wins the game on the
//! last one (PP-042b1). A key opens every door of its colour; doors block
//! like walls until then (PP-042b2). Guards walk their patrol paths;
//! touching one restarts the level (pies, keys and doors come back) and counts
//! as caught (PP-042c1). The title, pause and win screens come next.
//!
//! Run it from this folder, so its own `assets/` is found:
//!
//! ```text
//! cd games/pie-heist
//! cargo run                          # levels/level1.ron
//! cargo run -- levels/level2.ron     # any level, relative to assets/
//! ```
//!
//! Levels are drawn with the game's editor: `cargo run -p pie-heist-editor --
//! assets/levels/level1.ron`. `PIE_HEIST_EXIT_AFTER_FRAMES=N` closes the game
//! after N frames and prints where the player is (for automated checks).

use purplepie::audio::SoundId;
use purplepie::ecs::Entity;
use purplepie::input::KeyCode;
use purplepie::math::{Rect, Transform2D, Vec2};
use purplepie::render::{
    Color, FontId, Hidden, Layer, Quad, ScreenSpace, Sprite, Text, TextAnchor,
};
use purplepie::{Context, Engine, EngineConfig, Game};

use pie_heist::{
    Door, Exit, Guard, Key, KeyColor, Pie, PlayerStart, Wall, blocking, camera_target, drawn_rect,
    level_bounds, level_title, move_and_slide, next_level, patrol_offset, touched, wall_rect,
};

const WINDOW: (u32, u32) = (960, 640);
const PLAYER_SIZE: f32 = 28.0;
const PLAYER_SPEED: f32 = 220.0;
const FIRST_LEVEL: &str = "levels/level1.ron";
/// How a locked exit looks: its own colours, faded.
const LOCKED_TINT: Color = Color::rgba(1.0, 1.0, 1.0, 0.3);

/// The player's entity.
struct Player;

/// Marks the HUD's text entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hud {
    Status,
    Banner,
}

/// Sounds, loaded once.
struct Sounds {
    pie: SoundId,
    exit: SoundId,
    key: SoundId,
    door: SoundId,
    caught: SoundId,
}

/// A guard of the level being played: where it started, and its patrol.
struct Patrol {
    entity: Entity,
    origin: Vec2,
    guard: Guard,
}

/// The exit of the level being played.
struct LevelExit {
    entity: Entity,
    next: Option<String>,
    open: bool,
}

struct PieHeist {
    level: String,
    player: Option<Entity>,
    /// The walls' areas, collected when the level loads (walls do not move).
    walls: Vec<Rect>,
    /// The level's doors: their colour, area and entity.
    doors: Vec<(KeyColor, Rect, Entity)>,
    /// The keys picked up in this level.
    keys: Vec<KeyColor>,
    /// What blocks the player now: the walls and the doors still closed.
    solid: Vec<Rect>,
    bounds: Option<Rect>,
    pies_total: usize,
    pies_left: usize,
    exit: Option<LevelExit>,
    guards: Vec<Patrol>,
    /// Seconds since the level was (re)started: where the guards are.
    level_time: f32,
    /// How many times a guard caught the player, over the whole game.
    caught: u32,
    /// The game time when the last level's exit was reached; the player
    /// stops, and the banner shows it.
    won: Option<f64>,
    font: Option<FontId>,
    sounds: Option<Sounds>,
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

        // Doors are quads that block until a key of their colour is held; keys
        // and doors are drawn in their colour, whatever the file says.
        self.doors.clear();
        self.keys.clear();
        for (e, t, q, door) in world.query_mut::<(Entity, &Transform2D, &mut Quad, &Door)>() {
            q.color = door.color.color();
            match wall_rect(t, q) {
                Some(rect) => self.doors.push((door.color, rect, e)),
                None => eprintln!(
                    "pie-heist: warning: a rotated door in {} is ignored",
                    self.level
                ),
            }
        }
        for (sprite, key) in world.query_mut::<(&mut Sprite, &Key)>() {
            sprite.tint = key.color.color();
        }
        self.update_solid();

        // Guards walk from where the level puts them; drawn above the floor
        // items, below the player.
        self.guards = world
            .query_mut::<(Entity, &Transform2D, &Guard)>()
            .into_iter()
            .map(|(entity, t, guard)| Patrol {
                entity,
                origin: t.position,
                guard: guard.clone(),
            })
            .collect();
        self.guards.sort_by_key(|p| p.entity.id());
        for patrol in &self.guards {
            let _ = world.insert_one(patrol.entity, Layer(5));
        }
        self.level_time = 0.0;
        self.move_guards(ctx);
        let world = ctx.world_mut();

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

        self.pies_total = world.query_mut::<&Pie>().into_iter().count();
        self.pies_left = self.pies_total;
        let mut exits: Vec<(Entity, Exit)> = world
            .query_mut::<(Entity, &Exit)>()
            .into_iter()
            .map(|(e, exit)| (e, exit.clone()))
            .collect();
        exits.sort_by_key(|(e, _)| e.id());
        if exits.len() > 1 {
            eprintln!(
                "pie-heist: warning: {} has {} exits; using the first",
                self.level,
                exits.len()
            );
        }
        self.exit = exits.into_iter().next().map(|(entity, exit)| LevelExit {
            entity,
            next: next_level(&exit).map(str::to_owned),
            open: false,
        });
        if self.exit.is_none() {
            eprintln!("pie-heist: warning: {} has no exit", self.level);
        }

        let texture = ctx.load_texture("textures/player.png")?;
        let player = ctx.world_mut().spawn((
            Transform2D::from_position(start),
            Sprite::new(texture, Vec2::splat(PLAYER_SIZE)),
            Layer(10),
            Player,
        ));
        self.player = Some(player);
        self.spawn_hud(ctx)?;
        self.update_exit(ctx);
        self.update_hud(ctx);
        println!(
            "pie-heist: {}: {} entities, {} walls, {} doors, {} pies, {} guards; player at ({:.1}, {:.1})",
            self.level,
            entities.len(),
            self.walls.len(),
            self.doors.len(),
            self.pies_total,
            self.guards.len(),
            start.x,
            start.y
        );
        Ok(())
    }

    fn spawn_hud(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        let font = match self.font {
            Some(font) => font,
            None => *self
                .font
                .insert(ctx.load_font("fonts/Poppins-Regular.ttf")?),
        };
        let world = ctx.world_mut();
        world.spawn((
            Transform2D::from_position(Vec2::new(16.0, -14.0)),
            Text::new("", font, 22.0).with_anchor(TextAnchor::TOP_LEFT),
            ScreenSpace::TOP_LEFT,
            Layer(20),
            Hud::Status,
        ));
        world.spawn((
            Transform2D::default(),
            Text::new("", font, 44.0)
                .with_color(Color::hex(0xFFD60A))
                .with_anchor(TextAnchor::CENTER),
            ScreenSpace::CENTER,
            Layer(21),
            Hud::Banner,
        ));
        Ok(())
    }

    fn set_hud(ctx: &mut Context<'_>, which: Hud, content: String) {
        for (text, hud) in ctx.world_mut().query_mut::<(&mut Text, &Hud)>() {
            if *hud == which && text.content != content {
                text.content.clone_from(&content);
            }
        }
    }

    fn update_hud(&self, ctx: &mut Context<'_>) {
        let exit = match &self.exit {
            Some(exit) if exit.open => "   Exit open!",
            _ => "",
        };
        let keys = if self.keys.is_empty() {
            String::new()
        } else {
            let names: Vec<String> = self.keys.iter().map(|k| format!("{k:?}")).collect();
            format!("   Keys: {}", names.join(", "))
        };
        let status = format!(
            "{}   Pies {}/{}   Caught {}{keys}{exit}",
            level_title(&self.level),
            self.pies_total - self.pies_left,
            self.pies_total,
            self.caught
        );
        Self::set_hud(ctx, Hud::Status, status);
        let banner = match self.won {
            Some(at) => format!("You win!  {at:.0} s, caught {}", self.caught),
            None => String::new(),
        };
        Self::set_hud(ctx, Hud::Banner, banner);
    }

    /// Opens the exit once every pie is collected, and shows whether it is open.
    fn update_exit(&mut self, ctx: &mut Context<'_>) {
        let Some(exit) = &mut self.exit else {
            return;
        };
        exit.open = self.pies_left == 0;
        let tint = if exit.open { Color::WHITE } else { LOCKED_TINT };
        if let Ok(mut sprite) = ctx.world_mut().get::<&mut Sprite>(exit.entity) {
            sprite.tint = tint;
        }
        if let Ok(mut quad) = ctx.world_mut().get::<&mut Quad>(exit.entity) {
            quad.color.a = tint.a;
        }
    }

    fn player_position(&self, ctx: &Context<'_>) -> Option<Vec2> {
        let player = self.player?;
        ctx.world()
            .get::<&Transform2D>(player)
            .ok()
            .map(|t| t.position)
    }

    /// The area an entity is drawn over (its sprite or quad), if any.
    fn area(ctx: &Context<'_>, entity: Entity) -> Option<Rect> {
        let world = ctx.world();
        let transform = *world.get::<&Transform2D>(entity).ok()?;
        let size = world
            .get::<&Sprite>(entity)
            .map(|s| s.size)
            .or_else(|_| world.get::<&Quad>(entity).map(|q| q.size))
            .ok()?;
        Some(drawn_rect(&transform, size))
    }

    fn play(&self, ctx: &mut Context<'_>, sound: fn(&Sounds) -> SoundId) {
        if let Some(sounds) = &self.sounds {
            ctx.play_sound(sound(sounds), 0.8);
        }
    }

    /// Puts every guard where its patrol is after `level_time` seconds.
    fn move_guards(&self, ctx: &mut Context<'_>) {
        for patrol in &self.guards {
            let at = patrol.origin
                + patrol_offset(&patrol.guard.path, patrol.guard.speed, self.level_time);
            if let Ok(mut t) = ctx.world_mut().get::<&mut Transform2D>(patrol.entity) {
                t.position = at;
            }
        }
    }

    /// Restarts the level when a guard touches the player: pies, keys, doors
    /// and guards are back where the level file puts them. `true` if caught.
    fn check_caught(&mut self, ctx: &mut Context<'_>, player: Rect) -> bool {
        let guards: Vec<Rect> = self
            .guards
            .iter()
            .filter_map(|p| Self::area(ctx, p.entity))
            .collect();
        if touched(player, &guards).is_empty() {
            return false;
        }
        self.caught += 1;
        self.play(ctx, |s| s.caught);
        println!(
            "pie-heist: caught by a guard ({} time(s)); restarting {}",
            self.caught, self.level
        );
        if let Err(error) = self.load_level(ctx) {
            eprintln!("pie-heist: could not reload {}: {error:?}", self.level);
            ctx.request_exit();
        }
        true
    }

    /// The walls and the doors no key opens yet.
    fn update_solid(&mut self) {
        let doors: Vec<(KeyColor, Rect)> = self.doors.iter().map(|(c, r, _)| (*c, *r)).collect();
        self.solid = blocking(&self.walls, &doors, &self.keys);
    }

    /// Picks up the keys the player touches; every door of their colour opens
    /// (it disappears and stops blocking).
    fn pick_up_keys(&mut self, ctx: &mut Context<'_>, player: Rect) {
        let keys: Vec<(Entity, KeyColor)> = ctx
            .world_mut()
            .query_mut::<(Entity, &Key)>()
            .into_iter()
            .map(|(e, k)| (e, k.color))
            .collect();
        let areas: Vec<(Entity, KeyColor, Rect)> = keys
            .into_iter()
            .filter_map(|(e, c)| Self::area(ctx, e).map(|r| (e, c, r)))
            .collect();
        let rects: Vec<Rect> = areas.iter().map(|(_, _, r)| *r).collect();
        for i in touched(player, &rects) {
            let (entity, color, _) = areas[i];
            let _ = ctx.world_mut().despawn(entity);
            if !self.keys.contains(&color) {
                self.keys.push(color);
            }
            self.play(ctx, |s| s.key);
            let opened: Vec<Entity> = self
                .doors
                .iter()
                .filter(|(c, _, _)| *c == color)
                .map(|(_, _, e)| *e)
                .collect();
            for door in &opened {
                let _ = ctx.world_mut().insert_one(*door, Hidden);
            }
            if !opened.is_empty() {
                self.play(ctx, |s| s.door);
            }
            println!(
                "pie-heist: {color:?} key picked up; {} door(s) open",
                opened.len()
            );
        }
        self.update_solid();
    }

    /// Collects the pies the player touches, then goes through an open exit.
    fn pick_up(&mut self, ctx: &mut Context<'_>, player: Rect) {
        self.pick_up_keys(ctx, player);
        let pies: Vec<Entity> = ctx
            .world_mut()
            .query_mut::<(Entity, &Pie)>()
            .into_iter()
            .map(|(e, _)| e)
            .collect();
        let areas: Vec<(Entity, Rect)> = pies
            .into_iter()
            .filter_map(|e| Self::area(ctx, e).map(|r| (e, r)))
            .collect();
        let rects: Vec<Rect> = areas.iter().map(|(_, r)| *r).collect();
        for i in touched(player, &rects) {
            let _ = ctx.world_mut().despawn(areas[i].0);
            self.pies_left = self.pies_left.saturating_sub(1);
            self.play(ctx, |s| s.pie);
            println!("pie-heist: pie collected ({} left)", self.pies_left);
            if self.pies_left == 0 {
                println!("pie-heist: exit open");
            }
        }
        self.update_exit(ctx);

        let Some(exit) = &self.exit else {
            return;
        };
        if !exit.open || Self::area(ctx, exit.entity).is_none_or(|r| !player.overlaps(r)) {
            return;
        }
        self.play(ctx, |s| s.exit);
        match exit.next.clone() {
            Some(next) => {
                println!("pie-heist: exit to {next}");
                self.level = next;
                if let Err(error) = self.load_level(ctx) {
                    eprintln!("pie-heist: could not load {}: {error:?}", self.level);
                    ctx.request_exit();
                }
            }
            None => {
                let at = ctx.time().elapsed();
                self.won = Some(at);
                println!("pie-heist: you win in {at:.1} s");
            }
        }
    }
}

impl Game for PieHeist {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        pie_heist::register(ctx)?;
        self.sounds = Some(Sounds {
            pie: ctx.load_sound("sounds/pie.wav")?,
            exit: ctx.load_sound("sounds/exit.wav")?,
            key: ctx.load_sound("sounds/key.wav")?,
            door: ctx.load_sound("sounds/door.wav")?,
            caught: ctx.load_sound("sounds/caught.wav")?,
        });
        self.load_level(ctx)
    }

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        if self.won.is_some() {
            return;
        }
        let input = ctx.input();
        let direction = Vec2::new(
            input.axis(KeyCode::A, KeyCode::D)
                + input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight),
            input.axis(KeyCode::S, KeyCode::W) + input.axis(KeyCode::ArrowDown, KeyCode::ArrowUp),
        )
        .clamp(Vec2::splat(-1.0), Vec2::ONE)
        .normalize_or_zero();
        let delta = direction * PLAYER_SPEED * ctx.dt();
        self.level_time += ctx.dt();
        self.move_guards(ctx);
        let Some(player) = self.player else {
            return;
        };
        let mut at = None;
        if let Ok(mut t) = ctx.world_mut().get::<&mut Transform2D>(player) {
            t.position = move_and_slide(t.position, Vec2::splat(PLAYER_SIZE), delta, &self.solid);
            at = Some(t.position);
        }
        if let Some(at) = at {
            let area = Rect::from_center_size(at, Vec2::splat(PLAYER_SIZE));
            if !self.check_caught(ctx, area) {
                self.pick_up(ctx, area);
            }
        }
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        if let (Some(at), Some(bounds)) = (self.player_position(ctx), self.bounds) {
            let half_view = ctx.viewport_size() / (2.0 * ctx.camera().effective_zoom());
            ctx.camera_mut().position = camera_target(at, half_view, bounds);
        }
        self.update_hud(ctx);
        self.frames += 1;
        if self.exit_after.is_some_and(|frames| self.frames >= frames) {
            if let Some(at) = self.player_position(ctx) {
                println!(
                    "pie-heist: {}; player at ({:.1}, {:.1})",
                    self.level, at.x, at.y
                );
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
        doors: Vec::new(),
        keys: Vec::new(),
        solid: Vec::new(),
        bounds: None,
        pies_total: 0,
        pies_left: 0,
        exit: None,
        guards: Vec::new(),
        level_time: 0.0,
        caught: 0,
        won: None,
        font: None,
        sounds: None,
        exit_after,
        frames: 0,
    })
}

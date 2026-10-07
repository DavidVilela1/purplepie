//! Scene files (ADR-035): the same picture built in code or loaded from
//! `assets/scenes/demo.ron`.
//!
//! ```text
//! cargo run --example scene                                   # load scenes/demo.ron
//! PURPLEPIE_SCENE_EXAMPLE=build cargo run --example scene     # build it in code instead
//! PURPLEPIE_SCENE_EXAMPLE=save  cargo run --example scene     # build in code and write scenes/demo.ron
//! ```
//!
//! `load` and `build` must look identical: that is the round-trip check. The
//! shipped `demo.ron` was written by `save`. Escape quits.

use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{
    Color, Hidden, Layer, Quad, ScreenSpace, Sprite, SpriteGrid, Text, TextAnchor, TextureOptions,
};
use purplepie::{Context, Engine, EngineConfig, Game};

/// The scene file, relative to the asset root.
const SCENE: &str = "scenes/demo.ron";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Load `SCENE` (the default).
    Load,
    /// Build the scene in code.
    Build,
    /// Build the scene in code, then save it to `SCENE`.
    Save,
}

struct SceneDemo {
    mode: Mode,
}

/// Spawns the demo scene: quads, sprites (whole textures, sheet cells,
/// mirrored, rotated, tinted, `Nearest` and `Linear`), world and screen-space
/// text, layers and a hidden quad.
fn build(ctx: &mut Context<'_>) -> purplepie::Result<()> {
    let sheet = ctx.load_texture("textures/sandbox_sheet.png")?;
    let quadrants = ctx.load_texture("textures/sandbox_quadrants.png")?;
    let smooth = ctx.load_texture_with("textures/sandbox_quadrants.png", TextureOptions::LINEAR)?;
    let ball = ctx.load_texture_with("textures/breakout/ball.png", TextureOptions::LINEAR)?;
    let font = ctx.load_font("fonts/Poppins-Regular.ttf")?;
    let grid = SpriteGrid::new(8, 8, 4, 2);
    let world = ctx.world_mut();

    // A floor panel behind everything, and a stack of overlapping quads.
    world.spawn((
        Transform2D::from_position(Vec2::new(0.0, -40.0)),
        Quad::new(Vec2::new(1000.0, 420.0), Color::hex(0x240046)),
        Layer(-10),
    ));
    for (i, color) in [0xFF006E, 0xFB5607, 0xFFBE0B].into_iter().enumerate() {
        let i = i as f32;
        world.spawn((
            Transform2D::from_position(Vec2::new(-380.0 + i * 30.0, 60.0 - i * 30.0)),
            Quad::new(Vec2::new(120.0, 80.0), Color::hex(color)),
            Layer(i as i32),
        ));
    }
    // A quad that exists but is not drawn.
    world.spawn((
        Transform2D::default(),
        Quad::new(Vec2::splat(2000.0), Color::WHITE),
        Layer(100),
        Hidden,
    ));
    // Sheet cells: plain, mirrored, rotated; whole textures nearest and linear.
    for (frame, x) in [(0, -120.0), (5, -60.0)] {
        if let Some(cell) = grid.frame(frame) {
            world.spawn((
                Transform2D::from_position(Vec2::new(x, 40.0)),
                Sprite::new(sheet, cell.size() * 6.0).with_region(cell),
            ));
        }
    }
    if let Some(cell) = grid.frame(6) {
        world.spawn((
            Transform2D::from_position(Vec2::new(0.0, 40.0)).with_scale(Vec2::new(-1.0, 1.0)),
            Sprite::new(sheet, cell.size() * 6.0).with_region(cell),
        ));
    }
    world.spawn((
        Transform2D::from_position(Vec2::new(150.0, 40.0)).with_rotation(0.3),
        Sprite::new(quadrants, Vec2::splat(96.0)),
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(290.0, 40.0)).with_rotation(0.3),
        Sprite::new(smooth, Vec2::splat(96.0)).with_tint(Color::rgba(1.0, 1.0, 1.0, 0.8)),
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(400.0, 40.0)),
        Sprite::new(ball, Vec2::splat(20.0)),
        Layer(2),
    ));
    // Text in the world and a screen-space HUD.
    world.spawn((
        Transform2D::from_position(Vec2::new(0.0, -140.0)),
        Text::new("Saved and loaded with scene files", font, 28.0)
            .with_anchor(TextAnchor::BASELINE_CENTER),
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(16.0, -16.0)),
        Text::new("SCENE DEMO", font, 18.0)
            .with_color(Color::hex(0xFFBE0B))
            .with_anchor(TextAnchor::TOP_LEFT),
        ScreenSpace::TOP_LEFT,
        Layer(20),
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(-30.0, 30.0)),
        Quad::new(Vec2::splat(40.0), Color::hex(0x3A86FF)),
        ScreenSpace::BOTTOM_RIGHT,
    ));
    Ok(())
}

impl Game for SceneDemo {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        match self.mode {
            Mode::Load => {
                let entities = ctx.load_scene(SCENE)?;
                println!("scene: loaded {} entities from {SCENE}", entities.len());
            }
            Mode::Build => {
                build(ctx)?;
                println!("scene: built {} entities in code", ctx.world().len());
            }
            Mode::Save => {
                build(ctx)?;
                ctx.save_scene(SCENE)?;
                println!("scene: built in code and saved to {SCENE}");
            }
        }
        Ok(())
    }
}

fn main() -> purplepie::Result<()> {
    let mode = match std::env::var("PURPLEPIE_SCENE_EXAMPLE").as_deref() {
        Ok("build") => Mode::Build,
        Ok("save") => Mode::Save,
        _ => Mode::Load,
    };
    let config = EngineConfig::new("PurplePie Scene").with_size(1024, 600);
    Engine::new(config)?.run(SceneDemo { mode })
}

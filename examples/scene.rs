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
//! shipped `demo.ron` was written by `save`. The animated cell starts part-way
//! through its animation and the small square drifts right; with
//! `PURPLEPIE_SCENE_FREEZE=1` everything stays still, so screenshots can be compared.
//! The "Click me" button counts clicks. The two rotated squares carry a game
//! component of this example, `Spin` (registered with
//! `register_scene_component`, ADR-036), so they keep turning after a load; a
//! `Visits` component on an entity with nothing to draw counts how often the
//! scene was loaded. Escape quits.

use purplepie::ecs::{self, Velocity};
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{
    self, Color, Hidden, Layer, Quad, ScreenSpace, Sprite, SpriteAnimation, SpriteGrid, Text,
    TextAnchor, TextureOptions,
};
use purplepie::ui::{self, Button, Pointer};
use purplepie::{Context, Engine, EngineConfig, Game};
use serde::{Deserialize, Serialize};

/// A game component: turns the entity at `speed` radians per second.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct Spin {
    speed: f32,
    direction: Turn,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Turn {
    Clockwise,
    CounterClockwise,
}

/// A game component on an entity without a drawable: how often this scene
/// file has been loaded (the `save` mode writes 0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Visits {
    label: String,
    count: u32,
}

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
    /// Skip animation and movement (for pixel comparisons).
    frozen: bool,
    clicks: u32,
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
        Spin {
            speed: 0.8,
            direction: Turn::Clockwise,
        },
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(290.0, 40.0)).with_rotation(0.3),
        Sprite::new(smooth, Vec2::splat(96.0)).with_tint(Color::rgba(1.0, 1.0, 1.0, 0.8)),
        Spin {
            speed: 0.8,
            direction: Turn::CounterClockwise,
        },
    ));
    world.spawn((Visits {
        label: "demo scene".into(),
        count: 0,
    },));
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
    // PP-026b: an animation saved part-way through (frame 5, mid-frame), a
    // drifting square, and a screen-space button with its label.
    let mut animation = SpriteAnimation::new(grid, 0, 7, 4.0);
    animation.advance(1.3);
    world.spawn((
        Transform2D::from_position(Vec2::new(-260.0, -60.0)),
        Sprite::new(sheet, Vec2::splat(48.0)),
        animation,
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(-420.0, -200.0)),
        Quad::new(Vec2::splat(12.0), Color::hex(0x8338EC)),
        Velocity(Vec2::new(40.0, 0.0)),
        Layer(1),
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(-100.0, -40.0)),
        Quad::new(Vec2::new(160.0, 40.0), Color::hex(0xFF006E)),
        Button::new(Vec2::new(160.0, 40.0)),
        ScreenSpace::TOP_RIGHT,
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(-100.0, -40.0)),
        Text::new("Click me", font, 18.0).with_anchor(TextAnchor::new(
            render::HorizontalAnchor::Center,
            render::VerticalAnchor::Middle,
        )),
        ScreenSpace::TOP_RIGHT,
        Layer(1),
    ));
    Ok(())
}

impl Game for SceneDemo {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        // Register before saving or loading: files name these components.
        ctx.register_scene_component::<Spin>("scene_demo::Spin")?;
        ctx.register_scene_component::<Visits>("scene_demo::Visits")?;
        match self.mode {
            Mode::Load => {
                let entities = ctx.load_scene(SCENE)?;
                println!("scene: loaded {} entities from {SCENE}", entities.len());
                let world = ctx.world_mut();
                for visits in world.query_mut::<&mut Visits>() {
                    visits.count += 1;
                    println!("scene: {} loaded {} time(s)", visits.label, visits.count);
                }
                let spinning = ctx.world().query::<&Spin>().iter().count();
                println!("scene: {spinning} spinning sprites");
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

    fn fixed_update(&mut self, ctx: &mut Context<'_>) {
        if self.frozen {
            return;
        }
        let dt = ctx.dt();
        render::advance_animations(ctx.world_mut(), dt);
        ecs::integrate_velocity(ctx.world_mut(), dt);
        for (transform, spin) in ctx.world_mut().query_mut::<(&mut Transform2D, &Spin)>() {
            let sign = match spin.direction {
                Turn::Clockwise => -1.0,
                Turn::CounterClockwise => 1.0,
            };
            transform.rotation += sign * spin.speed * dt;
        }
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        let pointer = Pointer::from_input(ctx.input());
        let viewport = ctx.viewport_size();
        ui::update_buttons(ctx.world_mut(), pointer, viewport);
        let clicked = ctx
            .world()
            .query::<&Button>()
            .iter()
            .any(|button| button.clicked());
        if clicked {
            self.clicks += 1;
            println!("scene: button clicked ({} so far)", self.clicks);
        }
    }
}

fn main() -> purplepie::Result<()> {
    let mode = match std::env::var("PURPLEPIE_SCENE_EXAMPLE").as_deref() {
        Ok("build") => Mode::Build,
        Ok("save") => Mode::Save,
        _ => Mode::Load,
    };
    let config = EngineConfig::new("PurplePie Scene").with_size(1024, 600);
    let frozen = std::env::var("PURPLEPIE_SCENE_FREEZE").is_ok_and(|v| v == "1");
    Engine::new(config)?.run(SceneDemo {
        mode,
        frozen,
        clicks: 0,
    })
}

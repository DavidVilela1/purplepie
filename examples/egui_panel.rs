//! The `Game::egui` hook (ADR-044): a game drawing its own egui panel.
//!
//! ```text
//! cargo run --example egui_panel --features egui
//! cargo run --example egui_panel --features debug-overlay    # plus the F12 overlay
//! ```
//!
//! The left panel changes the game while it runs: the square's size and
//! colour, the window title (typed into a text field), and a button that
//! removes the stamps. The game keeps its own input: ←/→/↑/↓ move the square
//! and a click in the scene stamps a dot there. Clicks on the panel and keys
//! typed into the title field do not reach the game. Escape quits.
//!
//! `PURPLEPIE_EGUI_EXAMPLE_FRAMES=N` exits after N frames and prints what
//! happened, for automated checks.

use purplepie::ecs::Entity;
use purplepie::egui;
use purplepie::input::{KeyCode, MouseButton};
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Color, Layer, Quad};
use purplepie::{Context, Engine, EngineConfig, Game};

/// How fast the square moves, in world units per second.
const SPEED: f32 = 240.0;

struct Panel {
    square: Option<Entity>,
    size: f32,
    color: [f32; 3],
    title: String,
    stamps: Vec<Entity>,
    clicks: u32,
    keys: u32,
    frames: u32,
    exit_after: Option<u32>,
}

impl Game for Panel {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        self.square = Some(ctx.world_mut().spawn((
            Transform2D::from_position(Vec2::new(100.0, 0.0)),
            Quad::new(Vec2::splat(self.size), Color::WHITE),
            Layer(1),
        )));
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        let input = ctx.input();
        let step = Vec2::new(
            input.axis(KeyCode::ArrowLeft, KeyCode::ArrowRight),
            input.axis(KeyCode::ArrowDown, KeyCode::ArrowUp),
        ) * SPEED
            * ctx.dt();
        // Every key press the game sees, so a check can tell typing apart.
        self.keys += input.pressed_keys().count() as u32;
        let click = input
            .mouse_just_pressed(MouseButton::Left)
            .then(|| ctx.cursor_world())
            .flatten();
        if let Some(square) = self.square
            && let Ok(mut transform) = ctx.world_mut().get::<&mut Transform2D>(square)
        {
            transform.position += step;
        }
        if let Some(at) = click {
            self.clicks += 1;
            let stamp = ctx.world_mut().spawn((
                Transform2D::from_position(at),
                Quad::new(Vec2::splat(10.0), Color::hex(0xFFB000)),
                Layer(2),
            ));
            self.stamps.push(stamp);
        }
        self.frames += 1;
        if self.exit_after.is_some_and(|n| self.frames >= n) {
            let position = self
                .square
                .and_then(|e| ctx.world().get::<&Transform2D>(e).ok().map(|t| t.position));
            println!(
                "egui_panel: clicks {}, key frames {}, stamps {}, size {:.0}, title {:?}, square at {:?}",
                self.clicks,
                self.keys,
                self.stamps.len(),
                self.size,
                self.title,
                position
            );
            ctx.request_exit();
        }
    }

    fn egui(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        egui::Panel::left("tuning")
            .resizable(false)
            .default_size(220.0)
            .show(ui, |ui| {
                ui.heading("Tuning");
                ui.add(egui::Slider::new(&mut self.size, 10.0..=200.0).text("size"));
                ui.horizontal(|ui| {
                    ui.label("colour");
                    ui.color_edit_button_rgb(&mut self.color);
                });
                ui.label("Window title");
                if ui.text_edit_singleline(&mut self.title).changed() {
                    ctx.set_window_title(self.title.clone());
                }
                if ui.button("Remove stamps").clicked() {
                    for stamp in self.stamps.drain(..) {
                        let _ = ctx.world_mut().despawn(stamp);
                    }
                }
                ui.separator();
                ui.label(format!("{} entities", ctx.world().len()));
                ui.label(format!("{} clicks reached the game", self.clicks));
                ui.small("Arrows move the square; click the scene to stamp.");
            });
        let [r, g, b] = self.color;
        if let Some(square) = self.square
            && let Ok(mut quad) = ctx.world_mut().get::<&mut Quad>(square)
        {
            quad.size = Vec2::splat(self.size);
            quad.color = Color::rgba(r, g, b, 1.0);
        }
    }
}

fn main() -> purplepie::Result<()> {
    let exit_after = std::env::var("PURPLEPIE_EGUI_EXAMPLE_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok());
    let config = EngineConfig::new("PurplePie egui panel")
        .with_size(1024, 600)
        .with_console_log(true)
        .with_egui(true)
        // With `--features debug-overlay`, F12 shows the overlay on top.
        .with_debug_overlay(cfg!(feature = "debug-overlay"));
    Engine::new(config)?.run(Panel {
        square: None,
        size: 80.0,
        color: [1.0, 1.0, 1.0],
        title: "PurplePie egui panel".to_owned(),
        stamps: Vec::new(),
        clicks: 0,
        keys: 0,
        frames: 0,
        exit_after,
    })
}

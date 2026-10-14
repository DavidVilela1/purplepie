//! The editor as a PurplePie [`Game`] (ADR-044): the engine draws the scene,
//! and egui panels around it list the entities, save and report.

use std::path::PathBuf;

use purplepie::egui;
use purplepie::math::Vec2;
use purplepie::{Context, Game};

use crate::summary::{entity_ids, summary};
use crate::view;

/// One line of feedback at the bottom of the window.
struct Status {
    text: String,
    error: bool,
}

/// The editor's state: the scene file and the view of it.
pub(crate) struct Editor {
    /// The scene file (opened and saved).
    file: PathBuf,
    /// Fit the camera to the scene in the next frame (after opening, or asked
    /// for), when the panels' sizes are known.
    fit_pending: bool,
    status: Status,
    /// The file exists but could not be opened (unreadable, broken, another
    /// version): saving would replace it with what is shown, so it is refused.
    keep_file: bool,
    /// Frames to run before exiting, for automated checks.
    exit_after: Option<u32>,
    frames: u32,
}

impl Editor {
    pub(crate) fn new(file: PathBuf, exit_after: Option<u32>) -> Self {
        Self {
            file,
            fit_pending: true,
            status: Status {
                text: String::new(),
                error: false,
            },
            keep_file: false,
            exit_after,
            frames: 0,
        }
    }

    fn file_name(&self) -> String {
        self.file.file_name().map_or_else(
            || self.file.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
    }

    /// Shows `text` in the status line and prints it for the terminal.
    fn report(&mut self, text: String, error: bool) {
        if error {
            eprintln!("editor: {text}");
        } else {
            println!("editor: {text}");
        }
        self.status = Status { text, error };
    }

    fn save(&mut self, ctx: &mut Context<'_>) {
        if self.keep_file {
            let text = format!(
                "not saved: {} could not be opened, and saving would replace it",
                self.file_name()
            );
            self.report(text, true);
            return;
        }
        match ctx.save_scene(&self.file) {
            Ok(()) => {
                let text = format!("saved {}", self.file.display());
                self.report(text, false);
            }
            Err(error) => self.report(format!("could not save: {}", explain(&error)), true),
        }
    }

    /// The left panel: the file, save and fit buttons, and every entity.
    fn entities_panel(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        egui::Panel::left("purplepie editor entities")
            .default_size(260.0)
            .min_size(160.0)
            .show(ui, |ui| {
                ui.heading(self.file_name());
                ui.horizontal(|ui| {
                    if ui.button("Save").on_hover_text("Ctrl+S").clicked() {
                        self.save(ctx);
                    }
                    if ui.button("Fit view").on_hover_text("F").clicked() {
                        self.fit_pending = true;
                    }
                });
                ui.separator();
                let world = ctx.world();
                let ids = entity_ids(world);
                ui.label(format!("{} entities", ids.len()));
                let row_height = ui.spacing().interact_size.y;
                egui::ScrollArea::vertical()
                    .id_salt("purplepie editor entity list")
                    .auto_shrink([false, false])
                    .show_rows(ui, row_height, ids.len(), |ui, range| {
                        for &entity in &ids[range] {
                            ui.label(summary(world, entity));
                        }
                    });
            });
    }

    /// The bottom line: the last message, red for errors.
    fn status_line(&self, ui: &mut egui::Ui) {
        egui::Panel::bottom("purplepie editor status").show(ui, |ui| {
            let text = egui::RichText::new(&self.status.text);
            ui.label(if self.status.error {
                text.color(ui.visuals().error_fg_color)
            } else {
                text
            });
        });
    }

    /// The free area over the scene: pan with the middle or right button,
    /// zoom with the wheel around the cursor, fit when asked.
    fn viewport(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let rect = ui.max_rect();
                let viewport = ctx.viewport_size();
                if self.fit_pending {
                    self.fit_pending = false;
                    let bounds = view::scene_bounds_in(ctx);
                    *ctx.camera_mut() = view::fit(
                        bounds,
                        Vec2::new(rect.min.x, rect.min.y),
                        Vec2::new(rect.max.x, rect.max.y),
                        viewport,
                    );
                }
                let response = ui.interact(
                    rect,
                    ui.id().with("purplepie editor viewport"),
                    egui::Sense::click_and_drag(),
                );
                if response.dragged_by(egui::PointerButton::Middle)
                    || response.dragged_by(egui::PointerButton::Secondary)
                {
                    let delta = response.drag_delta();
                    *ctx.camera_mut() = view::pan(*ctx.camera(), Vec2::new(delta.x, delta.y));
                }
                if let Some(at) = response.hover_pos() {
                    let notches: f32 = ui.input(|i| {
                        i.events
                            .iter()
                            .filter_map(|event| match event {
                                egui::Event::MouseWheel { unit, delta, .. } => Some(match unit {
                                    egui::MouseWheelUnit::Point => delta.y / 50.0,
                                    egui::MouseWheelUnit::Line => delta.y,
                                    egui::MouseWheelUnit::Page => delta.y * 10.0,
                                }),
                                _ => None,
                            })
                            .sum()
                    });
                    if notches != 0.0 {
                        *ctx.camera_mut() = view::zoom_at(
                            *ctx.camera(),
                            viewport,
                            Vec2::new(at.x, at.y),
                            view::ZOOM_STEP.powf(notches),
                        );
                    }
                }
            });
    }
}

/// An error with the reasons behind it: "failed to load asset `x`: No such
/// file or directory (os error 2)".
fn explain(error: &purplepie::Error) -> String {
    let mut text = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        text.push_str(&format!(": {cause}"));
        source = cause.source();
    }
    text
}

impl Game for Editor {
    fn init(&mut self, ctx: &mut Context<'_>) -> purplepie::Result<()> {
        ctx.set_window_title(format!("{} - PurplePie Editor", self.file_name()));
        // A file that cannot be opened is reported, not fatal: the window
        // stays open to show why.
        match ctx.load_scene_keeping_unknown(&self.file) {
            Ok(entities) => {
                let text = format!(
                    "opened {}: {} entities",
                    self.file.display(),
                    entities.len()
                );
                self.report(text, false);
            }
            Err(error) => {
                // A missing file is a new scene; an existing one is kept safe.
                self.keep_file = self.file.exists();
                self.report(format!("could not open: {}", explain(&error)), true);
            }
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        self.frames += 1;
        if self.exit_after.is_some_and(|frames| self.frames >= frames) {
            ctx.request_exit();
        }
    }

    fn egui(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        // Shortcuts (each key press with its own modifiers).
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            self.save(ctx);
        }
        if !ui.ctx().text_edit_focused()
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F))
        {
            self.fit_pending = true;
        }
        // Side and bottom panels first, so the viewport is what they leave.
        self.entities_panel(ctx, ui);
        self.status_line(ui);
        self.viewport(ctx, ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_are_explained_with_their_causes() {
        let error = purplepie::Error::Asset {
            path: "scenes/x.ron".into(),
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not there",
            )),
        };
        let text = explain(&error);
        assert!(
            text.contains("scenes/x.ron") && text.ends_with(": not there"),
            "{text}"
        );
    }
}

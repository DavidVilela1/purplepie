//! The editor as a PurplePie [`Game`] (ADR-044): the engine draws the scene,
//! and egui panels around it list the entities, inspect the selection, save
//! and report. Selecting, moving and undoing work on the engine's world
//! through [`Context`]; the selection outline is drawn by egui, never as an
//! entity, so it is never saved.

use std::path::PathBuf;

use purplepie::ecs::Entity;
use purplepie::egui;
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Hidden, Layer, ScreenSpace, Text};
use purplepie::{Context, Game};

use crate::history::{Edit, History};
use crate::select;
use crate::summary::{details, entity_ids, summary};
use crate::view;

/// One line of feedback at the bottom of the window.
struct Status {
    text: String,
    error: bool,
}

/// A drag that moves an entity: where it was and where the pointer went down.
#[derive(Debug, Clone, Copy)]
struct Drag {
    entity: Entity,
    /// The entity's transform when the drag started.
    from: Transform2D,
    origin: egui::Pos2,
}

/// Arrow-key steps on the same entity within this many seconds are undone
/// together.
const NUDGE_MERGE_SECONDS: f64 = 1.0;

/// The outline drawn around the selection.
const OUTLINE: egui::Color32 = egui::Color32::from_rgb(255, 200, 40);

/// The editor's state: the scene file, the selection, the history and the
/// view of the scene.
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
    selected: Option<Entity>,
    /// Scroll the entity list to the selection (it was chosen in the view).
    reveal_selection: bool,
    history: History,
    drag: Option<Drag>,
    /// The entity and time of the last arrow-key step, to merge a burst.
    last_nudge: Option<(Entity, f64)>,
    /// An inspector edit in progress (a field being dragged or typed into):
    /// its value before the edit; recorded as one step when the field is let go.
    pending: Option<Edit>,
    /// Whether the window title shows the unsaved-changes marker.
    title_dirty: Option<bool>,
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
            selected: None,
            reveal_selection: false,
            history: History::new(),
            drag: None,
            last_nudge: None,
            pending: None,
            title_dirty: None,
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
        self.commit_pending(ctx);
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
                self.history.mark_saved();
                let text = format!("saved {}", self.file.display());
                self.report(text, false);
            }
            Err(error) => self.report(format!("could not save: {}", explain(&error)), true),
        }
    }

    /// Selects `entity` (or nothing) and says so in the status line.
    fn select(&mut self, ctx: &Context<'_>, entity: Option<Entity>, reveal: bool) {
        if self.selected != entity {
            self.commit_pending(ctx);
            self.selected = entity;
            self.reveal_selection = reveal && entity.is_some();
            match entity {
                Some(entity) => {
                    self.report(format!("selected {}", summary(ctx.world(), entity)), false);
                }
                None => self.report("nothing selected".to_owned(), false),
            }
        }
    }

    /// Reverts the last edit (`undo`) or applies the last reverted one again.
    fn step_history(&mut self, ctx: &mut Context<'_>, undo: bool) {
        // An inspector edit still in progress becomes the last step first.
        self.commit_pending(ctx);
        let edit = if undo {
            self.history.undo()
        } else {
            self.history.redo()
        };
        let Some(edit) = edit else {
            return;
        };
        self.last_nudge = None;
        let word = if undo { "undid" } else { "redid" };
        let entity = edit.entity();
        if edit.apply(ctx.world_mut(), !undo) {
            self.select(ctx, Some(entity), true);
            self.report(format!("{word} {}", edit.describe()), false);
        } else {
            self.report(format!("{word}: #{} no longer exists", entity.id()), true);
        }
    }

    /// Starts an inspector edit of the component `before` describes (its
    /// value now), unless one of the same kind on the same entity is already
    /// in progress; a different one in progress is recorded first.
    fn begin_edit(&mut self, ctx: &Context<'_>, before: Edit) {
        if let Some(pending) = self.pending {
            if std::mem::discriminant(&pending) == std::mem::discriminant(&before)
                && pending.entity() == before.entity()
            {
                return;
            }
            self.commit_pending(ctx);
        }
        self.pending = Some(before);
        self.last_nudge = None;
    }

    /// Records the inspector edit in progress, if any, as one step.
    fn commit_pending(&mut self, ctx: &Context<'_>) {
        if let Some(pending) = self.pending.take() {
            let edit = pending.ending_now(ctx.world());
            if !edit.changes_nothing() {
                self.history.record(edit, false);
            }
        }
    }

    /// Whether the scene differs from the file (including an edit in progress).
    fn is_dirty(&self, ctx: &Context<'_>) -> bool {
        self.history.is_dirty()
            || self
                .pending
                .is_some_and(|p| !p.ending_now(ctx.world()).changes_nothing())
    }

    /// The selection's transform, if it can be moved in the view (it has a
    /// transform and is not in screen space, whose moves come later).
    fn movable_transform(ctx: &Context<'_>, entity: Entity) -> Option<Transform2D> {
        let world = ctx.world();
        if world.get::<&ScreenSpace>(entity).is_ok() {
            return None;
        }
        world.get::<&Transform2D>(entity).ok().map(|t| *t)
    }

    fn set_position(ctx: &mut Context<'_>, entity: Entity, position: Vec2) {
        if let Ok(mut transform) = ctx.world_mut().get::<&mut Transform2D>(entity) {
            transform.position = position;
        }
    }

    /// Keyboard shortcuts: save, fit, undo, redo, deselect, arrow steps.
    fn shortcuts(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        use egui::{Key, Modifiers};
        if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::S)) {
            self.save(ctx);
        }
        if ui.ctx().text_edit_focused() {
            return;
        }
        // Most specific first: Ctrl+Shift+Z before Ctrl+Z.
        if ui.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)
                || i.consume_key(Modifiers::COMMAND, Key::Y)
        }) {
            self.step_history(ctx, false);
        }
        if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Z)) {
            self.step_history(ctx, true);
        }
        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F)) {
            self.fit_pending = true;
        }
        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.select(ctx, None, false);
        }
        let mut step = Vec2::ZERO;
        ui.input_mut(|i| {
            for (key, direction) in [
                (Key::ArrowLeft, Vec2::new(-1.0, 0.0)),
                (Key::ArrowRight, Vec2::new(1.0, 0.0)),
                (Key::ArrowUp, Vec2::new(0.0, 1.0)),
                (Key::ArrowDown, Vec2::new(0.0, -1.0)),
            ] {
                let shifted = i.count_and_consume_key(Modifiers::SHIFT, key) as f32;
                let plain = i.count_and_consume_key(Modifiers::NONE, key) as f32;
                step += select::key_step(direction, true) * shifted
                    + select::key_step(direction, false) * plain;
            }
        });
        if step == Vec2::ZERO {
            return;
        }
        let Some(entity) = self.selected else {
            return;
        };
        let Some(before) = Self::movable_transform(ctx, entity) else {
            self.report(
                format!(
                    "#{} cannot be moved (screen space or no transform)",
                    entity.id()
                ),
                true,
            );
            return;
        };
        self.commit_pending(ctx);
        let to = before.position + step;
        Self::set_position(ctx, entity, to);
        let now = ctx.time().elapsed();
        let merge = self
            .last_nudge
            .is_some_and(|(e, at)| e == entity && now - at < NUDGE_MERGE_SECONDS);
        self.history.record(Edit::moved(entity, before, to), merge);
        self.last_nudge = Some((entity, now));
    }

    /// The left panel: file, buttons, the entities and the inspector.
    fn entities_panel(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        egui::Panel::left("purplepie editor entities")
            .default_size(280.0)
            .min_size(180.0)
            .show(ui, |ui| {
                ui.heading(self.file_name());
                ui.horizontal(|ui| {
                    if ui.button("Save").on_hover_text("Ctrl+S").clicked() {
                        self.save(ctx);
                    }
                    if ui.button("Fit view").on_hover_text("F").clicked() {
                        self.fit_pending = true;
                    }
                    if ui
                        .add_enabled(self.history.can_undo(), egui::Button::new("Undo"))
                        .on_hover_text("Ctrl+Z")
                        .clicked()
                    {
                        self.step_history(ctx, true);
                    }
                    if ui
                        .add_enabled(self.history.can_redo(), egui::Button::new("Redo"))
                        .on_hover_text("Ctrl+Shift+Z or Ctrl+Y")
                        .clicked()
                    {
                        self.step_history(ctx, false);
                    }
                });
                ui.separator();
                let ids = entity_ids(ctx.world());
                ui.label(format!("{} entities", ids.len()));
                let row_height = ui.spacing().interact_size.y;
                let row_stride = row_height + ui.spacing().item_spacing.y;
                let mut list = egui::ScrollArea::vertical()
                    .id_salt("purplepie editor entity list")
                    .max_height((ui.available_height() * 0.55).max(120.0))
                    .auto_shrink([false, true]);
                if std::mem::take(&mut self.reveal_selection)
                    && let Some(index) =
                        self.selected.and_then(|s| ids.iter().position(|&e| e == s))
                {
                    list = list.vertical_scroll_offset((index as f32 - 2.0).max(0.0) * row_stride);
                }
                let mut clicked = None;
                list.show_rows(ui, row_height, ids.len(), |ui, range| {
                    for &entity in &ids[range] {
                        let selected = self.selected == Some(entity);
                        if ui
                            .selectable_label(selected, summary(ctx.world(), entity))
                            .clicked()
                        {
                            clicked = Some(entity);
                        }
                    }
                });
                if clicked.is_some() {
                    self.select(ctx, clicked, false);
                }
                ui.separator();
                self.inspector(ctx, ui);
            });
    }

    /// The selection's components: transform, layer and hidden editable
    /// (each change one undoable step when its field is let go), the rest
    /// read-only.
    fn inspector(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        let Some(entity) = self.selected else {
            ui.label("Click an entity in the view or the list to select it.");
            return;
        };
        if !ctx.world().contains(entity) {
            ui.label(format!("#{} no longer exists.", entity.id()));
            return;
        }
        ui.strong(format!("#{}", entity.id()));
        let mut active = false;
        let mut track = |response: &egui::Response| {
            active |= response.dragged() || response.has_focus();
            response.changed()
        };
        let transform = ctx.world().get::<&Transform2D>(entity).ok().map(|t| *t);
        let layer = ctx.world().get::<&Layer>(entity).ok().map(|l| *l);
        let hidden = ctx.world().get::<&Hidden>(entity).is_ok();
        let mut new_transform = transform;
        let mut new_layer = layer;
        let mut new_hidden = hidden;
        egui::Grid::new("purplepie editor fields")
            .num_columns(2)
            .show(ui, |ui| {
                if let Some(t) = &mut new_transform {
                    ui.label("Position");
                    ui.horizontal(|ui| {
                        track(
                            &ui.add(
                                egui::DragValue::new(&mut t.position.x)
                                    .speed(1.0)
                                    .prefix("x "),
                            ),
                        );
                        track(
                            &ui.add(
                                egui::DragValue::new(&mut t.position.y)
                                    .speed(1.0)
                                    .prefix("y "),
                            ),
                        );
                    });
                    ui.end_row();
                    ui.label("Rotation");
                    let mut degrees = t.rotation.to_degrees();
                    if track(&ui.add(egui::DragValue::new(&mut degrees).speed(1.0).suffix("°"))) {
                        t.rotation = degrees.to_radians();
                    }
                    ui.end_row();
                    ui.label("Scale");
                    ui.horizontal(|ui| {
                        track(
                            &ui.add(
                                egui::DragValue::new(&mut t.scale.x)
                                    .speed(0.01)
                                    .prefix("x "),
                            ),
                        );
                        track(
                            &ui.add(
                                egui::DragValue::new(&mut t.scale.y)
                                    .speed(0.01)
                                    .prefix("y "),
                            ),
                        );
                    });
                    ui.end_row();
                }
                ui.label("Layer");
                ui.horizontal(|ui| {
                    let mut has_layer = new_layer.is_some();
                    let mut value = new_layer.map_or(0, |l| l.0);
                    track(&ui.checkbox(&mut has_layer, ""));
                    track(&ui.add_enabled(has_layer, egui::DragValue::new(&mut value)));
                    new_layer = has_layer.then_some(Layer(value));
                });
                ui.end_row();
                ui.label("Hidden");
                track(&ui.checkbox(&mut new_hidden, ""));
                ui.end_row();
            });
        if let (Some(before), Some(after)) = (transform, new_transform)
            && before != after
        {
            self.begin_edit(
                ctx,
                Edit::Transform {
                    entity,
                    before,
                    after: before,
                },
            );
            if let Ok(mut t) = ctx.world_mut().get::<&mut Transform2D>(entity) {
                *t = after;
            }
        }
        if new_layer != layer {
            self.begin_edit(
                ctx,
                Edit::Layer {
                    entity,
                    before: layer,
                    after: layer,
                },
            );
            Edit::Layer {
                entity,
                before: layer,
                after: new_layer,
            }
            .apply(ctx.world_mut(), true);
        }
        if new_hidden != hidden {
            self.begin_edit(
                ctx,
                Edit::Hidden {
                    entity,
                    before: hidden,
                    after: hidden,
                },
            );
            Edit::Hidden {
                entity,
                before: hidden,
                after: new_hidden,
            }
            .apply(ctx.world_mut(), true);
        }
        if !active {
            self.commit_pending(ctx);
        }

        ui.separator();
        let editable = ["Position", "Rotation", "Scale", "Layer", "Hidden"];
        let rows: Vec<(String, String)> = details(ctx.world(), entity)
            .into_iter()
            .filter(|(label, _)| !editable.contains(&label.as_str()))
            .collect();
        egui::ScrollArea::vertical()
            .id_salt("purplepie editor inspector")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                egui::Grid::new("purplepie editor inspector grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        for (label, value) in rows {
                            ui.label(label);
                            ui.add(egui::Label::new(egui::RichText::new(value).monospace()).wrap());
                            ui.end_row();
                        }
                    });
            });
    }

    /// The bottom line: the last message (red for errors) and whether there
    /// are unsaved changes.
    fn status_line(&self, ctx: &Context<'_>, ui: &mut egui::Ui) {
        let dirty = self.is_dirty(ctx);
        egui::Panel::bottom("purplepie editor status").show(ui, |ui| {
            ui.horizontal(|ui| {
                if dirty {
                    ui.strong("* unsaved changes");
                    ui.separator();
                }
                let text = egui::RichText::new(&self.status.text);
                ui.label(if self.status.error {
                    text.color(ui.visuals().error_fg_color)
                } else {
                    text
                });
            });
        });
    }

    /// The free area over the scene: select and move with the left button,
    /// pan with the middle or right button, zoom with the wheel around the
    /// cursor, fit when asked; the selection is outlined.
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
                self.select_and_move(ctx, ui, &response, viewport);
                if response.dragged_by(egui::PointerButton::Middle)
                    || response.dragged_by(egui::PointerButton::Secondary)
                {
                    let delta = response.drag_delta();
                    *ctx.camera_mut() = view::pan(*ctx.camera(), Vec2::new(delta.x, delta.y));
                }
                if let Some(at) = response.hover_pos() {
                    let notches = wheel_notches(ui);
                    if notches != 0.0 {
                        *ctx.camera_mut() = view::zoom_at(
                            *ctx.camera(),
                            viewport,
                            Vec2::new(at.x, at.y),
                            view::ZOOM_STEP.powf(notches),
                        );
                    }
                }
                if let Some(entity) = self.selected {
                    let camera = *ctx.camera();
                    let corners = view::outline(ctx.world(), entity, &camera, viewport, |text| {
                        text_bounds(ctx, text)
                    });
                    if let Some(corners) = corners {
                        let points = corners.iter().map(|c| egui::pos2(c.x, c.y)).collect();
                        ui.painter().add(egui::Shape::closed_line(
                            points,
                            egui::Stroke::new(2.0, OUTLINE),
                        ));
                    }
                }
            });
    }

    /// Left click selects (again on the same spot: the next one below); a
    /// left drag moves the selection, or the topmost entity under the
    /// pointer, and is one undoable move.
    fn select_and_move(
        &mut self,
        ctx: &mut Context<'_>,
        ui: &egui::Ui,
        response: &egui::Response,
        viewport: Vec2,
    ) {
        let camera = *ctx.camera();
        let picked_at = |ctx: &Context<'_>, p: egui::Pos2| {
            ctx.entities_at(camera.screen_to_world(Vec2::new(p.x, p.y), viewport))
        };
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(p) = response.interact_pointer_pos()
        {
            let picked = picked_at(ctx, p);
            self.select(ctx, select::after_click(&picked, self.selected), true);
        }
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
        {
            let picked = picked_at(ctx, origin);
            let target = select::drag_target(&picked, self.selected);
            self.select(ctx, target, true);
            self.commit_pending(ctx);
            self.drag = target.and_then(|entity| {
                Self::movable_transform(ctx, entity).map(|from| Drag {
                    entity,
                    from,
                    origin,
                })
            });
            self.last_nudge = None;
        }
        if let Some(drag) = self.drag {
            if response.dragged_by(egui::PointerButton::Primary)
                && let Some(pointer) = ui.input(|i| i.pointer.interact_pos())
            {
                let offset = select::drag_to_world(
                    Vec2::new(pointer.x - drag.origin.x, pointer.y - drag.origin.y),
                    camera.effective_zoom(),
                );
                Self::set_position(ctx, drag.entity, drag.from.position + offset);
            }
            if response.drag_stopped_by(egui::PointerButton::Primary) {
                self.drag = None;
                if let Some(now) = Self::movable_transform(ctx, drag.entity) {
                    let to = now.position;
                    self.history
                        .record(Edit::moved(drag.entity, drag.from, to), false);
                    if to != drag.from.position {
                        self.report(
                            format!("moved #{} to ({:.1}, {:.1})", drag.entity.id(), to.x, to.y),
                            false,
                        );
                    }
                }
            }
        }
    }

    /// Keeps the window title in step with unsaved changes ("* name").
    fn update_title(&mut self, ctx: &mut Context<'_>) {
        let dirty = self.is_dirty(ctx);
        if self.title_dirty != Some(dirty) {
            self.title_dirty = Some(dirty);
            let mark = if dirty { "* " } else { "" };
            ctx.set_window_title(format!("{mark}{} - PurplePie Editor", self.file_name()));
        }
    }
}

/// A text's local rectangle, measured with the loaded fonts.
fn text_bounds(ctx: &Context<'_>, text: &Text) -> Option<(Vec2, Vec2)> {
    ctx.measure_text(text).map(|m| m.bounds(text.anchor))
}

/// Wheel notches this frame (up is positive), whatever unit the platform used.
fn wheel_notches(ui: &egui::Ui) -> f32 {
    ui.input(|i| {
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
    })
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
        self.update_title(ctx);
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        self.frames += 1;
        if self.exit_after.is_some_and(|frames| self.frames >= frames) {
            ctx.request_exit();
        }
    }

    fn egui(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        self.shortcuts(ctx, ui);
        // Side and bottom panels first, so the viewport is what they leave.
        self.entities_panel(ctx, ui);
        self.status_line(ctx, ui);
        self.viewport(ctx, ui);
        self.update_title(ctx);
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

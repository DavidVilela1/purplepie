//! New, Open and Save As within one run (PP-044a2, ADR-046): the File row,
//! the path field they ask for, and opening a scene (also what `init` does).

use std::path::PathBuf;

use purplepie::Context;
use purplepie::egui;

use super::{Editor, explain};
use crate::file;
use crate::history::History;
use crate::library;
use crate::question::Leave;

/// Which path the field asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PromptKind {
    /// The scene to open.
    Open,
    /// Where to save the scene, then do `then` (a question that was waiting
    /// for a new scene to be saved).
    SaveAs { then: Option<Leave> },
}

/// The Open or Save As path field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathPrompt {
    kind: PromptKind,
    text: String,
    problem: Option<String>,
    /// Save As onto this existing file was asked once; asking again replaces it.
    replace: Option<PathBuf>,
}

impl Editor {
    /// The folder typed paths are relative to: the one holding the asset folder.
    fn project(ctx: &Context<'_>) -> PathBuf {
        let cwd = std::env::current_dir().unwrap_or_default();
        file::project_folder(ctx.asset_root(), &cwd)
    }

    /// New, Open…, Save and Save As… buttons.
    pub(super) fn file_row(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("New").on_hover_text("Ctrl+N").clicked() {
                self.leave(ctx, Leave::New);
            }
            if ui.button("Open…").on_hover_text("Ctrl+O").clicked() {
                self.start_open(ctx);
            }
            if ui.button("Save").on_hover_text("Ctrl+S").clicked() {
                self.save(ctx);
            }
            if ui
                .button("Save As…")
                .on_hover_text("Ctrl+Shift+S")
                .clicked()
            {
                self.start_save_as(ctx, None);
            }
        });
    }

    /// Opens the path field for Open, filled with the current scene's folder.
    pub(super) fn start_open(&mut self, ctx: &Context<'_>) {
        let project = Self::project(ctx);
        let folder = self
            .file
            .as_ref()
            .and_then(|f| f.parent())
            .map(|f| format!("{}/", file::shown(f, &project)))
            .unwrap_or_default();
        self.prompt = Some(PathPrompt {
            kind: PromptKind::Open,
            text: folder,
            problem: None,
            replace: None,
        });
    }

    /// Opens the path field for Save As, filled with the current file; `then`
    /// happens after a successful save.
    pub(super) fn start_save_as(&mut self, ctx: &mut Context<'_>, then: Option<Leave>) {
        self.finish_edits(ctx);
        let project = Self::project(ctx);
        let text = match &self.file {
            Some(f) => file::shown(f, &project),
            None => ctx
                .asset_root()
                .map(|a| format!("{}/", file::shown(a, &project)))
                .unwrap_or_default(),
        };
        self.prompt = Some(PathPrompt {
            kind: PromptKind::SaveAs { then },
            text,
            problem: None,
            replace: None,
        });
    }

    /// The path field, over everything while it is open.
    pub(super) fn path_prompt_window(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        let Some(mut prompt) = self.prompt.take() else {
            return;
        };
        let (title, action) = match prompt.kind {
            PromptKind::Open => ("Open a scene", "Open"),
            PromptKind::SaveAs { .. } => ("Save the scene as", "Save"),
        };
        let mut chosen = false;
        let mut cancelled = false;
        let modal = egui::Modal::new(egui::Id::new("purplepie editor path")).show(ui.ctx(), |ui| {
            ui.heading(title);
            ui.label("Path from the project folder (the folder holding the asset folder):");
            let field = ui.add(
                egui::TextEdit::singleline(&mut prompt.text)
                    .desired_width(420.0)
                    .id(egui::Id::new("purplepie editor path field")),
            );
            if !field.has_focus() && !field.lost_focus() {
                field.request_focus();
            }
            if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                chosen = true;
            }
            if let Some(problem) = &prompt.problem {
                ui.colored_label(egui::Color32::from_rgb(255, 120, 120), problem);
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button(action).clicked() {
                    chosen = true;
                }
                if ui.button("Cancel").clicked() {
                    cancelled = true;
                }
            });
        });
        if cancelled || (!chosen && modal.should_close()) {
            return; // the prompt is dropped
        }
        if !chosen {
            self.prompt = Some(prompt);
            return;
        }
        let project = Self::project(ctx);
        let path = match file::resolve(&prompt.text, &project, ctx.asset_root()) {
            Ok(path) => path,
            Err(problem) => {
                prompt.problem = Some(problem);
                self.prompt = Some(prompt);
                return;
            }
        };
        match prompt.kind.clone() {
            PromptKind::Open => self.leave(ctx, Leave::Open(path)),
            PromptKind::SaveAs { then } => {
                let replacing = path.exists() && self.file.as_ref() != Some(&path);
                if replacing && prompt.replace.as_ref() != Some(&path) {
                    prompt.problem = Some(format!(
                        "{} already exists; click Save again to replace it",
                        file::shown(&path, &project)
                    ));
                    prompt.replace = Some(path);
                    self.prompt = Some(prompt);
                    return;
                }
                self.save_as(ctx, path, then);
            }
        }
    }

    /// Saves to `path`, which becomes the scene's file; then does `then`.
    fn save_as(&mut self, ctx: &mut Context<'_>, path: PathBuf, then: Option<Leave>) {
        if self.file.as_ref() != Some(&path) {
            // A new file is the editor's own: the old one's protection (it could
            // not be opened) does not apply to it.
            self.keep_file = false;
            self.file = Some(path);
            self.title_dirty = None;
        }
        self.save(ctx);
        if !self.is_dirty(ctx)
            && let Some(then) = then
        {
            self.go(ctx, then);
        }
    }

    /// Replaces the scene with `file` (or an empty one): everything about the
    /// old scene is forgotten (selection, history, edits in progress).
    pub(super) fn open_scene(&mut self, ctx: &mut Context<'_>, file: Option<PathBuf>) {
        ctx.world_mut().clear();
        self.file = file;
        self.keep_file = false;
        self.selected = None;
        self.reveal_selection = false;
        self.history = History::new();
        self.drag = None;
        self.last_nudge = None;
        self.pending = None;
        self.kept_draft = None;
        self.fit_pending = true;
        self.title_dirty = None;
        self.load(ctx);
    }

    /// Loads the scene file into the (empty) world and reports what was found.
    /// A file that cannot be opened is reported, not fatal: the window stays
    /// open to show why.
    pub(super) fn load(&mut self, ctx: &mut Context<'_>) {
        match self.file.clone() {
            None => self.report(
                "new scene: Save (Ctrl+S) asks where to save it".to_owned(),
                false,
            ),
            Some(path) => match ctx.load_scene_keeping_unknown(&path) {
                Ok(entities) => {
                    let mut text =
                        format!("opened {}: {} entities", path.display(), entities.len());
                    let (checked, problems) = self.check_game_components(ctx);
                    if checked > 0 {
                        text.push_str(&format!(
                            "; {checked} game component value(s) checked against the game's types"
                        ));
                    }
                    match problems.first() {
                        None => self.report(text, false),
                        Some(first) => {
                            for problem in &problems {
                                eprintln!("editor: {problem}");
                            }
                            let count = problems.len();
                            text.push_str(&format!(
                                "; {count} do(es) not fit (in red in the inspector), first: {first}"
                            ));
                            self.report(text, true);
                        }
                    }
                }
                Err(error) => {
                    // A missing file is a new scene; an existing one is kept safe.
                    self.keep_file = path.exists();
                    if self.keep_file {
                        self.report(format!("could not open: {}", explain(&error)), true);
                    } else {
                        let text = format!(
                            "new scene: {} does not exist yet; saving (Ctrl+S) creates it",
                            path.display()
                        );
                        self.report(text, false);
                    }
                }
            },
        }
        if let Some(root) = ctx.asset_root() {
            self.images = library::files_under(root, library::IMAGES);
            self.fonts = library::files_under(root, library::FONTS);
        }
        self.update_title(ctx);
    }
}

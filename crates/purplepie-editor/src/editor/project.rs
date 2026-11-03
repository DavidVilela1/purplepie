//! The project panel (PP-044b, ADR-046): the asset folder as a tree of scenes,
//! images, fonts and sounds; a click on a scene opens it.

use std::path::Path;

use purplepie::Context;
use purplepie::egui;

use super::Editor;
use crate::library::{self, Folder, Kind};
use crate::question::Leave;

/// How often the asset folder is read again, in seconds, so files added or
/// removed outside the editor show up.
const REFRESH_SECONDS: f64 = 1.0;

/// The asset folder as last read.
#[derive(Debug, Default)]
pub(crate) struct ProjectFiles {
    tree: Folder,
    truncated: bool,
    /// When it was read (game time), `None` to read it in the next frame.
    read_at: Option<f64>,
}

impl ProjectFiles {
    /// Read the folder again in the next frame (after a save, say).
    pub(crate) fn stale(&mut self) {
        self.read_at = None;
    }
}

/// `path` relative to `root`, with `/` between folders, as the tree has it.
fn relative(path: &Path, root: &Path) -> Option<String> {
    let rest = path.strip_prefix(root).ok()?;
    Some(
        rest.components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/"),
    )
}

impl Editor {
    /// Reads the asset folder again when it is due: the tree, and the images
    /// and fonts the Add menus offer.
    fn refresh_files(&mut self, ctx: &Context<'_>) {
        let now = ctx.time().elapsed();
        if self
            .project
            .read_at
            .is_some_and(|at| now - at < REFRESH_SECONDS && now >= at)
        {
            return;
        }
        self.project.read_at = Some(now);
        let Some(root) = ctx.asset_root() else {
            return;
        };
        let (tree, truncated) = library::tree(root);
        self.project.tree = tree;
        self.project.truncated = truncated;
        self.images = library::files_under(root, library::IMAGES);
        self.fonts = library::files_under(root, library::FONTS);
    }

    /// The panel at the far left.
    pub(super) fn project_panel(&mut self, ctx: &mut Context<'_>, ui: &mut egui::Ui) {
        self.refresh_files(ctx);
        let root = ctx.asset_root().map(Path::to_path_buf);
        let current = match (&self.file, &root) {
            (Some(file), Some(root)) => relative(file, root),
            _ => None,
        };
        let mut clicked: Option<String> = None;
        egui::Panel::left("purplepie editor project")
            .default_size(200.0)
            .min_size(140.0)
            .show(ui, |ui| {
                ui.heading("Project");
                let Some(root) = &root else {
                    ui.weak("No asset folder: scenes and assets cannot be listed.");
                    return;
                };
                let folder_name = root.file_name().map_or_else(
                    || root.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                );
                ui.weak(format!("{folder_name}/"))
                    .on_hover_text(root.display().to_string());
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        show_folder(ui, &self.project.tree, "", current.as_deref(), &mut clicked);
                        if self.project.truncated {
                            ui.weak("… more files than the panel lists");
                        }
                    });
            });
        if let (Some(path), Some(root)) = (clicked, root) {
            self.leave(ctx, Leave::Open(root.join(path)));
        }
    }
}

/// One folder's sub-folders (collapsible, open at first) and files. A click on
/// a scene other than `current` is put in `clicked`.
fn show_folder(
    ui: &mut egui::Ui,
    folder: &Folder,
    prefix: &str,
    current: Option<&str>,
    clicked: &mut Option<String>,
) {
    for sub in &folder.folders {
        let path = format!("{prefix}{}/", sub.name);
        egui::CollapsingHeader::new(&sub.name)
            .id_salt(("purplepie editor folder", &path))
            .default_open(true)
            .show(ui, |ui| show_folder(ui, sub, &path, current, clicked));
    }
    for file in &folder.files {
        if file.kind == Kind::Scene {
            let open = current == Some(file.path.as_str());
            let response = ui
                .selectable_label(open, &file.name)
                .on_hover_text(format!("{} (click to open)", file.path));
            if response.clicked() && !open {
                *clicked = Some(file.path.clone());
            }
        } else {
            ui.label(egui::RichText::new(format!("{}  ·  {}", file.name, file.kind.word())).weak())
                .on_hover_text(&file.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_shown_relative_to_the_asset_folder() {
        let root = Path::new("/game/assets");
        assert_eq!(
            relative(Path::new("/game/assets/levels/one.ron"), root),
            Some("levels/one.ron".to_owned())
        );
        assert_eq!(relative(Path::new("/elsewhere/one.ron"), root), None);
    }
}

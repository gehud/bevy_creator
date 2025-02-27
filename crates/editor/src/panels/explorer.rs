use std::path::PathBuf;

use bevy::ecs::world::World;
use bevy_egui::egui::{TextBuffer, Ui};

use crate::{editor::SelectedProject, panel::Panel};

#[derive(Default)]
pub struct ExplorerPanel;

impl Panel for ExplorerPanel {
    fn name(&self) -> String {
        "Explorer".into()
    }

    fn ui(&mut self, world: &mut World, ui: &mut Ui) {
        let project_dir = world
            .get_resource::<SelectedProject>()
            .unwrap()
            .dir
            .clone()
            .unwrap();

        self.draw_recursive(&project_dir, ui);
    }
}

const EXPLORER_FILE_EXCLUDE: &'static [&'static str] =
    &[".bevy", ".cargo", "Cargo.toml", "Cargo.lock", "target"];

const EXPLORER_EXT_EXCLUDE: &'static [&'static str] = &[
    "meta"
];

impl ExplorerPanel {
    fn draw_recursive(&self, path: &PathBuf, ui: &mut Ui) {
        if EXPLORER_FILE_EXCLUDE.contains(&path.file_name().unwrap().to_string_lossy().as_str()) {
            return;
        }

        if let Some(extension) = path.extension() {
            if EXPLORER_EXT_EXCLUDE.contains(&extension.to_string_lossy().as_str()) {
                return;
            }
        }

        if !path.is_dir() {
            ui.label(path.file_name().unwrap().to_string_lossy());
            return;
        }

        let entries = std::fs::read_dir(path).unwrap();

        ui.collapsing(path.file_name().unwrap().to_string_lossy(), |ui| {
            for entry in entries {
                if let Ok(entry) = entry {
                    self.draw_recursive(&entry.path(), ui);
                }
            }
        })
        .header_response
        .context_menu(|ui| {
            self.dir_context_menu(ui);
        });
    }

    fn dir_context_menu(&self, ui: &mut Ui) {
        if ui.button("New Folder...").clicked() {
            ui.close_menu();
        }
    }
}

use std::path::PathBuf;

use bevy::ecs::world::World;
use bevy_egui::egui::Ui;

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

impl ExplorerPanel {
    fn draw_recursive(&self, path: &PathBuf, ui: &mut Ui) {
        if path.file_name().unwrap() == ".bevy" {
            return;
        }

        if path.is_file() {
            ui.label(path.file_name().unwrap().to_string_lossy());
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

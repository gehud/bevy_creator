use std::path::PathBuf;

use bevy::{asset::AssetServer, ecs::world::World, tasks::block_on};
use bevy_assets::AssetRefPayload;
use bevy_egui::egui::{Id, TextBuffer, Ui};

use crate::{editor::SelectedProject, panel::Panel, PROJECT_ASSETS_DIR};

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

        self.dir_ui(&world, &project_dir, ui);
    }
}

const EXPLORER_FILE_EXCLUDE: &'static [&'static str] =
    &[".bevy", ".cargo", "Cargo.toml", "Cargo.lock", "target"];

const EXPLORER_EXT_EXCLUDE: &'static [&'static str] = &["meta"];

impl ExplorerPanel {
    fn dir_ui(&self, world: &World, path: &PathBuf, ui: &mut Ui) {
        if EXPLORER_FILE_EXCLUDE.contains(&path.file_name().unwrap().to_string_lossy().as_str()) {
            return;
        }

        if let Some(extension) = path.extension() {
            if EXPLORER_EXT_EXCLUDE.contains(&extension.to_string_lossy().as_str()) {
                return;
            }
        }

        if !path.is_dir() {
            self.file_ui(world, path, ui);
            return;
        }

        let entries = std::fs::read_dir(path).unwrap();

        ui.collapsing(path.file_name().unwrap().to_string_lossy(), |ui| {
            for entry in entries {
                if let Ok(entry) = entry {
                    self.dir_ui(&world, &entry.path(), ui);
                }
            }
        })
        .header_response
        .context_menu(|ui| {
            self.dir_context_menu_ui(ui);
        });
    }

    fn file_ui(&self, world: &World, path: &PathBuf, ui: &mut Ui) {
        let mut project_assets_dir = world.resource::<SelectedProject>().dir.clone().unwrap();
        project_assets_dir.push(PROJECT_ASSETS_DIR);

        if let Ok(asset_path) = path.strip_prefix(project_assets_dir) {
            let asset_path = asset_path.to_string_lossy().to_string();

            let asset_server = world.resource::<AssetServer>();

            if let Ok(_) = block_on(asset_server.load_untyped_async(&asset_path)) {
                if let Some(labeled_assets) = asset_server.get_living_labeled_assets(&asset_path) {
                    ui.collapsing(path.file_name().unwrap().to_string_lossy(), |ui| {
                        let payload = AssetRefPayload(asset_path.clone());
                        let id = Id::new(format!("AssetRefPayload({})", &asset_path));
                        ui.dnd_drag_source(id, payload, |ui| {
                            ui.label("Self");
                        });

                        for label in labeled_assets {
                            let labeled_path = format!("{}#{}", &asset_path, &label);
                            let payload = AssetRefPayload(labeled_path.clone());
                            let id = Id::new(format!("AssetRefPayload({})", &labeled_path));
                            ui.dnd_drag_source(id, payload, |ui| {
                                ui.label(label);
                            });
                        }
                    });
                } else {
                    let payload = AssetRefPayload(asset_path.clone());
                    let id = Id::new(format!("AssetRefPayload({})", &asset_path));
                    ui.dnd_drag_source(id, payload, |ui| {
                        ui.label(path.file_name().unwrap().to_string_lossy());
                    });
                }
            } else {
                ui.label(path.file_name().unwrap().to_string_lossy());
            }
        } else {
            ui.label(path.file_name().unwrap().to_string_lossy());
        }
    }

    fn dir_context_menu_ui(&self, ui: &mut Ui) {
        if ui.button("New Folder...").clicked() {
            ui.close_menu();
        }
    }
}

use bevy::ecs::schedule::SystemSet;
use bevy_config::define_app_config;

mod asset;
mod camera;
mod codegen;
mod dock;
mod editor;
mod editor_config;
mod egui_config;
mod egui_picking;
mod grid;
mod panel;
mod panels;
mod scene;
mod selection;
mod transform_gizmo_ext;
mod window_config;

pub use editor::EditorPlugin;

mod project_dir;
pub use project_dir::ProjectDir;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum EditorSet {
    Egui,
}

const PROJECT_CACHE_DIR: &'static str = ".bevy";
const PROJECT_ASSET_DIR: &'static str = "assets";
const PROJECT_IMPORTED_ASSET_DIR: &'static str = "imported";

define_app_config!();

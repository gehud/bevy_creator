use std::env;

use asset::EditorAssetPlugin;
use bevy::app::{App, PluginGroup, PreUpdate};
use bevy::asset::{AssetMode, AssetPlugin};
use bevy::ecs::schedule::{IntoSystemSetConfigs, SystemSet};
use bevy::picking::{mesh_picking::MeshPickingPlugin, PickSet};
use bevy::utils::default;
use bevy::window::{PresentMode, Window, WindowPlugin};
use bevy::DefaultPlugins;
use bevy_config::define_app_config;
use bevy_egui::{EguiPlugin, EguiPreUpdateSet};
use bevy_helper::winit::WindowIconPlugin;
use editor::{EditorPlugin, SelectedProject};
use egui_picking::EguiPickingPlugin;
use selection::SelectionPlugin;

mod asset;
mod dock;
mod editor;
mod egui_config;
mod egui_picking;
mod panel;
mod panels;
mod scene;
mod selection;
mod transform_gizmo_ext;
mod window_config;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum EditorSet {
    Egui,
}

define_app_config!();

const PROJECT_CACHE_DIR: &'static str = "/.bevy";
const PROJECT_ASSETS_DIR: &'static str = "/assets";
const PROJECT_PROCESSED_ASSET_DIR: &'static str = "/imported";

fn main() {
    let Some(project_dir) = env::args().nth(1) else {
        bevy::log::error!("Project directory expected as first argument");
        return;
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoNoVsync,
                        title: "BevyEditor".into(),
                        resolution: (1280., 720.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    mode: AssetMode::Processed,
                    file_path: project_dir.clone() + PROJECT_ASSETS_DIR,
                    processed_file_path: project_dir.clone()
                        + PROJECT_CACHE_DIR
                        + PROJECT_PROCESSED_ASSET_DIR,
                    watch_for_changes_override: Some(true),
                    ..default()
                }),
        )
        .add_plugins(WindowIconPlugin)
        .configure_sets(
            PreUpdate,
            EditorSet::Egui
                .after(EguiPreUpdateSet::BeginPass)
                .before(PickSet::Backend),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EguiPickingPlugin)
        .add_plugins(SelectionPlugin)
        .add_plugins(EditorPlugin)
        .add_plugins(EditorAssetPlugin)
        .insert_resource(SelectedProject {
            dir: Some(project_dir.into()),
            ..default()
        })
        .run();
}

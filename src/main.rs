use asset::EditorAssetPlugin;
use bevy::app::{App, PluginGroup, PreUpdate};
use bevy::asset::{AssetMode, AssetPlugin};
use bevy::ecs::schedule::{IntoSystemSetConfigs, SystemSet};
use bevy::picking::{mesh_picking::MeshPickingPlugin, PickSet};
use bevy::state::{app::AppExtStates, state::States};
use bevy::utils::default;
use bevy::window::{PresentMode, Window, WindowPlugin};
use bevy::DefaultPlugins;
use bevy_egui::{EguiPlugin, EguiPreUpdateSet};
use editor::EditorPlugin;
use egui_picking::EguiPickingPlugin;
use projects::ProjectsPlugin;
use selection::SelectionPlugin;

mod asset;
mod config;
mod demo_scene;
mod dock;
mod editor;
mod egui_config;
mod egui_picking;
mod panel;
mod panels;
mod projects;
mod selection;
mod transform_gizmo_ext;
mod util;
mod window_config;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    Projects,
    Editor,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum AppSet {
    Egui,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoNoVsync,
                        title: "BevyEditor".into(),
                        resolution: (640., 360.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    mode: AssetMode::Processed,
                    watch_for_changes_override: Some(true),
                    ..default()
                }),
        )
        .init_state::<AppState>()
        .configure_sets(
            PreUpdate,
            AppSet::Egui
                .after(EguiPreUpdateSet::BeginPass)
                .before(PickSet::Backend),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EguiPickingPlugin)
        .add_plugins(SelectionPlugin)
        .add_plugins(ProjectsPlugin)
        .add_plugins(EditorPlugin)
        .add_plugins(EditorAssetPlugin)
        .run();
}

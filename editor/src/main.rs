use bevy::{picking::PickSet, prelude::*, window::PresentMode};
use bevy_egui::{EguiPlugin, EguiPreUpdateSet};
use editor::EditorPlugin;
use egui_picking::EguiPickingPlugin;
use projects::ProjectsPlugin;
use selection::SelectionPlugin;

mod config;
mod demo_scene;
mod editor;
mod egui_config;
mod egui_picking;
mod panel;
mod projects;
mod selection;
mod transform_gizmo_ext;
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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                title: "BevyEditor".into(),
                resolution: (640., 360.).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .configure_sets(
            PreUpdate,
            AppSet::Egui
                .after(EguiPreUpdateSet::BeginPass)
                .after(bevy_egui::begin_pass_system)
                .before(PickSet::Backend),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EguiPickingPlugin)
        .add_plugins(SelectionPlugin)
        .add_plugins(ProjectsPlugin)
        .add_plugins(EditorPlugin)
        .run();
}

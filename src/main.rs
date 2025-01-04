use bevy::{
    app::App,
    prelude::{MeshPickingPlugin, PluginGroup},
    utils::default,
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use demo_scene::DemoScenePlugin;
use editor::EditorPlugin;
use selection::SelectionPlugin;
use window_config::WindowConfigPlugin;

mod config;
mod demo_scene;
mod editor;
mod egui_config;
mod egui_picking;
mod selection;
mod window_config;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                title: String::from("BevyCreator"),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WindowConfigPlugin)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(SelectionPlugin)
        .add_plugins(DemoScenePlugin)
        .add_plugins(EditorPlugin)
        .run();
}

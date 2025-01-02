use bevy::{prelude::*, window::PresentMode};

mod egui_picking_plugin;
pub use egui_picking_plugin::EguiPickingPlugin;

mod selection;
pub use selection::SelectionPlugin;

pub mod file_io;

mod window_persistence;
use window_persistence::WindowPersistencePlugin;

mod ui_plugin;
use ui_plugin::UiPlugin;

mod demo_scene;
pub use demo_scene::DemoScenePlugin;

pub struct GameEditorPlugin;

impl Plugin for GameEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                title: String::from("BevyCreator"),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EguiPickingPlugin)
        .add_plugins(SelectionPlugin)
        .add_plugins(WindowPersistencePlugin)
        .add_plugins(UiPlugin)
        .add_plugins(DemoScenePlugin)
        .register_type::<Option<Handle<Image>>>()
        .register_type::<AlphaMode>();
    }
}

fn main() {
    App::new().add_plugins(GameEditorPlugin).run();
}

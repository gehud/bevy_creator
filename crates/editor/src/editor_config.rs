use bevy::app::{App, Plugin, PreUpdate, Startup};
use bevy::ecs::event::EventReader;
use bevy::ecs::system::Res;
use bevy::ecs::world::World;
use bevy::window::WindowCloseRequested;
use bevy_config::app_config;

use crate::editor::{load_last_scene, SelectedScene};

const CONFIG_NAME: &str = "editor";

pub struct EditorConfigPlugin;

impl Plugin for EditorConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedScene>()
            .add_systems(Startup, restore_editor_config)
            .add_systems(PreUpdate, on_before_close);
    }
}

pub fn restore_editor_config(world: &mut World) {
    if let Some(config) = app_config!().load::<SelectedScene>(CONFIG_NAME) {
        *world.resource_mut::<SelectedScene>() = config;
    }

    load_last_scene(world);
}

fn on_before_close(
    mut ev_window_will_close: EventReader<WindowCloseRequested>,
    state: Res<SelectedScene>,
) {
    for _ in ev_window_will_close.read() {
        app_config!().save(CONFIG_NAME, state.as_ref());
    }
}

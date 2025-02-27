use bevy::app::{App, Plugin, PreUpdate, Startup};
use bevy::ecs::{
    event::EventReader,
    system::{Res, ResMut},
};
use bevy::window::WindowCloseRequested;
use bevy_config::app_config;

use crate::{dock::EditorDockState, editor::EditorState};

const CONFIG_NAME: &str = "egui";

pub struct EguiConfigPlugin;

impl Plugin for EguiConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, restore_panel_state)
            .add_systems(PreUpdate, on_before_close);
    }
}

fn restore_panel_state(mut editor_state: ResMut<EditorState>) {
    if let Some(state) = load_panel_config() {
        bevy::log::info!("Loaded \"egui\" config file");
        editor_state.docking = state;
    } else {
        bevy::log::info!("Could not load \"egui\" config file. Setting to default");
    }
}

fn on_before_close(
    editor_state: Res<EditorState>,
    mut ev_window_will_close: EventReader<WindowCloseRequested>,
) {
    for _ in ev_window_will_close.read() {
        save_panel_config(&editor_state.docking);
    }
}

fn load_panel_config() -> Option<EditorDockState> {
    app_config!().load::<EditorDockState>(CONFIG_NAME)
}

fn save_panel_config(state: &EditorDockState) {
    app_config!().save(CONFIG_NAME, state);
}

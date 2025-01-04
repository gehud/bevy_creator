use bevy::prelude::*;
use egui_dock::DockState;

use crate::{
    config::{read_json_config, save_json_config},
    editor::{EguiWindow, EditorState},
};

const FILE_NAME: &str = "egui_config";

pub struct EguiConfigPlugin;

impl Plugin for EguiConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, restore_panel_state)
            .add_systems(PreUpdate, on_before_close);
    }
}

fn restore_panel_state(mut editor_state: ResMut<EditorState>) {
    let Some(state) = load_panel_config() else {
        bevy::log::info!("Could not load egui panel config file");
        return;
    };

    editor_state.docking = state;
}

fn on_before_close(
    editor_state: Res<EditorState>,
    mut ev_window_will_close: EventReader<bevy::window::WindowCloseRequested>,
) {
    for _ in ev_window_will_close.read() {
        save_panel_config(&editor_state.docking);
    }
}

fn load_panel_config() -> Option<DockState<EguiWindow>> {
    let Ok(config) = read_json_config(FILE_NAME) else {
        return None;
    };

    let Ok(state) = serde_json::from_str::<DockState<EguiWindow>>(config.as_str()) else {
        return None;
    };

    Some(state)
}

fn save_panel_config(state: &DockState<EguiWindow>) {
    let serialized = serde_json::to_string(state).unwrap();
    save_json_config(FILE_NAME, serialized);
}

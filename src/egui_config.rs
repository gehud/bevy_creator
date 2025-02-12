use bevy::prelude::*;

use crate::{
    config::{read_json_config, save_json_config}, dock::EditorDockState, editor::EditorState, AppState
};

const CONFIG_NAME: &str = "egui";

pub struct EguiConfigPlugin;

impl Plugin for EguiConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Editor), restore_panel_state)
            .add_systems(
                PreUpdate,
                on_before_close.run_if(in_state(AppState::Editor)),
            );
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
    mut ev_window_will_close: EventReader<bevy::window::WindowCloseRequested>,
) {
    for _ in ev_window_will_close.read() {
        save_panel_config(&editor_state.docking);
    }
}

fn load_panel_config() -> Option<EditorDockState> {
    let Ok(config) = read_json_config(CONFIG_NAME) else {
        return None;
    };

    let Ok(state) = serde_json::from_str::<EditorDockState>(config.as_str()) else {
        return None;
    };

    Some(state)
}

fn save_panel_config(state: &EditorDockState) {
    let serialized = serde_json::to_string(state).unwrap();
    save_json_config(CONFIG_NAME, serialized);
}

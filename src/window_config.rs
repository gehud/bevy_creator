use crate::{
    config::{read_json_config, save_json_config},
    AppState,
};
use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    query::With,
    schedule::IntoSystemConfigs,
    system::{NonSend, Query},
};
use bevy_state::{condition::in_state, state::OnEnter};
use bevy_window::{
    PrimaryWindow, Window, WindowCloseRequested, WindowMode, WindowPosition, WindowResolution,
};
use bevy_winit::WinitWindows;
use serde::{Deserialize, Serialize};

const CONFIG_NAME: &str = "window";

pub struct WindowConfigPlugin;

impl Plugin for WindowConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Editor), restore_window_state)
            .add_systems(
                PreUpdate,
                on_before_close.run_if(in_state(AppState::Editor)),
            );
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MainWindowConfig {
    pub mode: WindowMode,
    pub position: WindowPosition,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
    pub scale_factor: f32,
}

fn restore_window_state(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = primary_window.single_mut();

    if let Some(config) = load_window_config() {
        bevy_log::info!("Loaded \"window\" config file");
        window.resolution = WindowResolution::new(
            config.width as f32 / config.scale_factor as f32,
            config.height as f32 / config.scale_factor as f32,
        );
        window.mode = config.mode;
        window.position = config.position;
        window.set_maximized(config.maximized);
    } else {
        bevy_log::info!("Could not load \"window\" config file. Setting to default");
        window.set_maximized(true);
    };
}

fn on_before_close(
    primary_window: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut ev_window_will_close: EventReader<WindowCloseRequested>,
    winit_window: NonSend<WinitWindows>,
) {
    for _ in ev_window_will_close.read() {
        let Ok((entity, window)) = primary_window.get_single() else {
            return;
        };

        let Some(winit_window) = winit_window.get_window(entity) else {
            return;
        };

        save_window_state(&window, winit_window.is_maximized());
    }
}

fn load_window_config() -> Option<MainWindowConfig> {
    let Ok(config) = read_json_config(CONFIG_NAME) else {
        return None;
    };

    let Ok(state) = serde_json::from_str::<MainWindowConfig>(config.as_str()) else {
        return None;
    };

    Some(state)
}

fn save_window_state(window: &Window, maximized: bool) {
    let config = MainWindowConfig {
        mode: window.mode.clone(),
        position: window.position.clone(),
        width: window.physical_width(),
        height: window.physical_height(),
        scale_factor: window.resolution.base_scale_factor(),
        maximized,
    };

    let serialized = serde_json::to_string(&config).unwrap();
    save_json_config(CONFIG_NAME, serialized);
}

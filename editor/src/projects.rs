use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowCloseRequested, WindowResolution},
};
use bevy_egui::{
    egui::{
        panel::{Side, TopBottomSide},
        CentralPanel, Id, Layout, RichText, SidePanel, TopBottomPanel,
    },
    EguiContexts,
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

use crate::{
    config::{read_json_config, save_json_config},
    AppSet, AppState,
};

const CONFIG_NAME: &str = "projects";

#[derive(Default, Resource, Serialize, Deserialize)]
struct ProjectsState {
    projects: Vec<PathBuf>,
}

pub struct ProjectsPlugin;

impl Plugin for ProjectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectsState>()
            .add_systems(
                OnEnter(AppState::Projects),
                (load_config, setup_window).chain(),
            )
            .add_systems(
                PreUpdate,
                (save_on_exit, ui)
                    .chain()
                    .in_set(AppSet::Egui)
                    .run_if(in_state(AppState::Projects)),
            );
    }
}

fn setup_window(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = windows.single_mut();
    window.title = "BevyEditor - Projects".into();
    window.resolution = (640., 360.).into();
    window.position = WindowPosition::Centered(MonitorSelection::Current);
}

fn load_config(mut state: ResMut<ProjectsState>) {
    *state = if let Ok(config) = read_json_config(CONFIG_NAME) {
        match serde_json::from_str::<ProjectsState>(config.as_str()) {
            Ok(state) => state,
            Err(error) => {
                bevy::log::error!("Cannot load config: {}", error);
                ProjectsState::default()
            }
        }
    } else {
        ProjectsState::default()
    }
}

fn save_config(state: &ProjectsState) {
    let serialized = serde_json::to_string(state).unwrap();
    save_json_config(CONFIG_NAME, serialized);
}

fn save_on_exit(
    state: Res<ProjectsState>,
    mut window_close_requests: EventReader<WindowCloseRequested>,
) {
    for _ in window_close_requests.read() {
        save_config(&state);
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut state: ResMut<ProjectsState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Some(ctx) = contexts.try_ctx_mut() {
        TopBottomPanel::new(TopBottomSide::Top, Id::new("top")).show(ctx, |ui| {
            ui.label(RichText::new("Bevy Projects").size(36.));
        });

        SidePanel::new(Side::Right, Id::new("right")).show(ctx, |_ui| {});

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    if let Some(project_dir) = FileDialog::new()
                        .set_can_create_directories(true)
                        .pick_folder()
                    {
                        if !state.projects.contains(&project_dir)
                            && create_project(project_dir.clone())
                        {
                            state.projects.push(project_dir.clone());
                        }
                    }
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                for project_dir in &state.projects {
                    if ui
                        .button(project_dir.file_name().unwrap().to_string_lossy())
                        .clicked()
                    {
                        open_project(&state, &mut next_state, project_dir.clone());
                    }
                }
            });
        });
    }
}

fn check_dir<P: AsRef<Path>>(dir_path: P) -> bool {
    if dir_path.as_ref().exists() {
        true
    } else {
        fs::create_dir(dir_path)
            .inspect_err(|error| bevy::log::error!("{}", error))
            .is_ok()
    }
}

fn create_project(project_dir: PathBuf) -> bool {
    let mut cache_dir = project_dir.clone();
    cache_dir.push(".bevy");
    if !check_dir(cache_dir) {
        return false;
    }

    let mut assets_dir = project_dir.clone();
    assets_dir.push("assets");
    if !check_dir(assets_dir) {
        return false;
    }

    true
}

fn open_project(
    state: &ProjectsState,
    next_state: &mut ResMut<NextState<AppState>>,
    project_dir: PathBuf,
) {
    save_config(state);
    next_state.set(AppState::Editor);
}

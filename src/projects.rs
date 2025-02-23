use std::{
    io::Cursor,
    path::PathBuf,
};

use bevy::app::{App, Plugin, PreUpdate};
use bevy::ecs::{
    event::EventReader,
    query::With,
    schedule::IntoSystemConfigs,
    system::{NonSend, Query, Res, ResMut, Resource},
};
use bevy_egui::{
    egui::{
        panel::{Side, TopBottomSide},
        CentralPanel, Id, RichText, SidePanel, TopBottomPanel,
    },
    EguiContexts,
};
use bevy::state::{
    condition::in_state,
    state::{NextState, OnEnter},
};
use bevy::window::{MonitorSelection, PrimaryWindow, Window, WindowCloseRequested, WindowPosition};
use bevy::winit::WinitWindows;
use image::ImageReader;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use winit::window::Icon;

use crate::{
    config::{read_json_config, save_json_config}, editor::SelectedProject, util::copy_dir_all, AppSet, AppState
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

fn setup_window(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    winit: NonSend<WinitWindows>,
) {
    let mut window = windows.single_mut();
    window.title = "BevyEditor - Projects".into();
    window.resolution = (640., 360.).into();
    window.position = WindowPosition::Centered(MonitorSelection::Current);

    let (icon_rgba, icon_width, icon_height) = {
        let image = ImageReader::new(Cursor::new(include_bytes!("../assets/icon.png")))
            .with_guessed_format()
            .expect("Unexpected image format")
            .decode()
            .expect("Failed to decode image")
            .into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    for window in winit.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
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
    };

    validate_projects(&mut state);
}

fn validate_projects(state: &mut ProjectsState) {
    state.projects.retain(|path| path.exists());
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
    mut selected_project: ResMut<SelectedProject>,
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
                        open_project(
                            &state,
                            &mut next_state,
                            &mut selected_project,
                            project_dir.clone(),
                        );
                    }
                }
            });
        });
    }
}

fn create_project(project_dir: PathBuf) -> bool {
    copy_dir_all("templates/project", project_dir).is_ok()
}

fn open_project(
    state: &ResMut<ProjectsState>,
    next_state: &mut ResMut<NextState<AppState>>,
    selected_prokect: &mut ResMut<SelectedProject>,
    project_dir: PathBuf,
) {
    save_config(state);
    selected_prokect.dir = Some(project_dir);
    next_state.set(AppState::Editor);
}

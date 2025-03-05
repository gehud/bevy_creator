use std::{path::PathBuf, process::Command};

use bevy::{
    app::{App, AppExit, Plugin, PluginGroup, Startup, Update},
    ecs::{
        event::{EventReader, EventWriter},
        query::With,
        system::{Query, Res, ResMut, Resource},
    },
    utils::default,
    window::{
        MonitorSelection, PrimaryWindow, Window, WindowCloseRequested, WindowPlugin, WindowPosition,
    },
    DefaultPlugins,
};
use bevy_config::app_config;
use bevy_config::define_app_config;
use bevy_egui::{
    egui::{panel::TopBottomSide, CentralPanel, Id, RichText, TopBottomPanel},
    EguiContexts, EguiPlugin,
};
use bevy_helper::{fs::copy_dir_all, winit::WindowIconPlugin};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

define_app_config!();

const CONFIG_NAME: &str = "projects";

const DEFAULT_RESOLUTION: (f32, f32) = (640., 360.);

#[derive(Default, Resource, Serialize, Deserialize)]
struct ProjectDirs(Vec<PathBuf>);

#[derive(Default, Resource)]
struct ProjectsToRemove(Vec<usize>);

fn load_config(mut project_dirs: ResMut<ProjectDirs>) {
    *project_dirs = if let Some(project_dirs) = app_config!().load::<ProjectDirs>(CONFIG_NAME) {
        project_dirs
    } else {
        ProjectDirs::default()
    };

    validate_projects(&mut project_dirs);
}

fn validate_projects(project_dirs: &mut ProjectDirs) {
    project_dirs.0.retain(|path| path.exists());
}

fn save_config(project_dirs: &ProjectDirs) {
    app_config!().save(CONFIG_NAME, project_dirs);
}

fn save_on_exit(
    project_dirs: Res<ProjectDirs>,
    mut window_close_requests: EventReader<WindowCloseRequested>,
) {
    for _ in window_close_requests.read() {
        save_config(&project_dirs);
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut project_dirs: ResMut<ProjectDirs>,
    mut projects_to_remove: ResMut<ProjectsToRemove>,
    mut exit: EventWriter<AppExit>,
) {
    if let Some(ctx) = contexts.try_ctx_mut() {
        TopBottomPanel::new(TopBottomSide::Top, Id::new("top")).show(ctx, |ui| {
            ui.label(RichText::new("Projects").size(36.));
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    if let Some(project_dir) = FileDialog::new()
                        .set_can_create_directories(true)
                        .pick_folder()
                    {
                        if !project_dirs.0.contains(&project_dir)
                            && create_project(project_dir.clone())
                        {
                            project_dirs.0.push(project_dir.clone());
                        }
                    }
                }

                if ui.button("Add").clicked() {
                    if let Some(project_dir) = FileDialog::new()
                        .set_can_create_directories(true)
                        .pick_folder()
                    {
                        if !project_dirs.0.contains(&project_dir)
                            && add_project(project_dir.clone())
                        {
                            project_dirs.0.push(project_dir.clone());
                        }
                    }
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                for (i, project_dir) in project_dirs.0.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.button("X").clicked() {
                            projects_to_remove.0.push(i);
                        }

                        if ui
                            .selectable_label(
                                false,
                                RichText::new(project_dir.to_string_lossy()).size(16.),
                            )
                            .clicked()
                        {
                            open_project(&project_dirs, project_dir.clone());
                            exit.send(AppExit::Success);
                        }
                    });
                }
            });
        });
    }
}

fn create_project(project_dir: PathBuf) -> bool {
    copy_dir_all("templates/project", project_dir).is_ok()
}

fn add_project(mut project_dir: PathBuf) -> bool {
    project_dir.push(".bevy");
    project_dir.exists()
}

fn open_project(project_dirs: &ResMut<ProjectDirs>, project_dir: PathBuf) {
    save_config(project_dirs);

    Command::new("BevyCreator")
        .arg(project_dir)
        .spawn()
        .expect("failed to execute process");
}

fn center_window(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = windows.single_mut();
    window.position = WindowPosition::Centered(MonitorSelection::Current);
}

fn validate_project_dirs(
    mut projects_to_remove: ResMut<ProjectsToRemove>,
    mut project_dirs: ResMut<ProjectDirs>,
) {
    for i in &projects_to_remove.0 {
        project_dirs.0.remove(*i);
    }

    projects_to_remove.0.clear();

    project_dirs.0.retain(|path| path.exists());
}

pub struct HubPlugin;

impl Plugin for HubPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectDirs>()
            .init_resource::<ProjectsToRemove>()
            .add_systems(Startup, (load_config, center_window))
            .add_systems(Update, (save_on_exit, ui, validate_project_dirs));
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "BevyHub".into(),
                resolution: DEFAULT_RESOLUTION.into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WindowIconPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(HubPlugin)
        .run();
}

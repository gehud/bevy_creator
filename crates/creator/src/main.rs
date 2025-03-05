use std::{env, path::PathBuf, process::exit};

use bevy::{
    app::{App, PluginGroup, PreUpdate},
    asset::{AssetMode, AssetPlugin},
    utils::default,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};

use bevy_editor::{EditorPlugin, PROJECT_ASSET_DIR, PROJECT_CACHE_DIR, PROJECT_IMPORTED_ASSET_DIR};

fn main() {
    let project_dir = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Project directory expected as first argument");
        exit(1);
    });

    let mut asset_dir = PathBuf::from(project_dir.clone());
    asset_dir.push(PROJECT_ASSET_DIR);

    let mut imported_asset_dir = PathBuf::from(project_dir.clone());
    imported_asset_dir.push(PROJECT_CACHE_DIR);
    imported_asset_dir.push(PROJECT_IMPORTED_ASSET_DIR);

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "BevyCreator".into(),
                        resolution: (1280., 720.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    mode: AssetMode::Processed,
                    file_path: asset_dir.to_string_lossy().to_string(),
                    processed_file_path: imported_asset_dir.to_string_lossy().to_string(),
                    watch_for_changes_override: true.into(),
                    ..default()
                }),
        )
        .add_plugins(EditorPlugin {
            project_dir: project_dir.into(),
        })
        .run();
}

use std::{env, process::exit};

use bevy::app::App;

use bevy_editor::EditorPlugin;

fn main() {
    let project_dir = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Project directory expected as first argument");
        exit(1);
    });

    App::new()
        .add_plugins(EditorPlugin {
            project_dir: project_dir.into(),
        })
        .run();
}

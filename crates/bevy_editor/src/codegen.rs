use std::{
    env::{consts::DLL_EXTENSION, current_dir},
    fs,
    path::PathBuf,
};

use bevy::utils::HashMap;

use crate::editor::EDITOR_PROJECT_DEPENDENCIES;

fn write_template(project_dir: &PathBuf, from: &str, to: &str, overrides: HashMap<&str, &str>) {
    let mut text = match fs::read_to_string(from) {
        Ok(cargo_manifest_text) => cargo_manifest_text,
        Err(err) => {
            bevy::log::error!("Failed to read file: {err}");
            return;
        }
    };

    for item in overrides {
        text = text.replace(item.0, item.1);
    }

    match fs::write(project_dir.join(to), text) {
        Err(err) => {
            bevy::log::error!("Failed to write file: {err}");
        }
        _ => {}
    }
}

pub fn setup_compilation(dependencies: &[&str], project_dir: &PathBuf) {
    write_template(
        project_dir,
        "templates/project/Cargo.toml",
        "Cargo.toml",
        [("{{dependencies}}", "")].into(),
    );

    let mut rustflags: String = "  \"-L\",\n  \"all={{lib_dir}}/deps\",\n".into();

    for dependency in dependencies
        .iter()
        .chain(EDITOR_PROJECT_DEPENDENCIES.iter())
    {
        let mut flag =
            String::from("  \"{{dependency}}={{lib_dir}}/{{dependency}}.{{dll_extension}}\",\n");

        flag = flag
            .replace("{{dependency}}", &dependency)
            .replace("{{dll_extension}}", DLL_EXTENSION);

        rustflags.push_str("  \"--extern\",\n");
        rustflags.push_str(&flag);
    }

    rustflags = rustflags.replace("{{lib_dir}}", &get_lib_dir());

    write_template(
        project_dir,
        "templates/project/.cargo/config.toml",
        ".cargo/config.toml",
        [("{{rustflags}}", rustflags.as_str())].into(),
    );
}

fn get_lib_dir() -> String {
    current_dir()
        .unwrap()
        .to_string_lossy()
        .to_string()
        .replace('\\', &"/")
}

pub fn setup_editing(dependencies: &[&str], project_dir: &PathBuf) {
    let mut text = String::default();

    for dependency in dependencies {
        text.push_str(
            &String::from("{{dependency}} = { git = \"https://github.com/gehud/bevy.git\", branch = \"dynamic\" }\n").replace("{{dependency}}", &dependency),
        );
    }

    for dependency in EDITOR_PROJECT_DEPENDENCIES {
        text.push_str(
            &String::from("{{dependency}} = { path = \"{{lib_dir}}/crates/{{dependency}}\" }")
                .replace("{{dependency}}", &dependency)
                .replace("{{lib_dir}}", &get_lib_dir()),
        );
    }

    write_template(
        project_dir,
        "templates/project/Cargo.toml",
        "Cargo.toml",
        [("{{dependencies}}", text.as_str())].into(),
    );

    write_template(
        project_dir,
        "templates/project/.cargo/config.toml",
        ".cargo/config.toml",
        [("{{rustflags}}", "")].into(),
    );
}

use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use ron::{from_str, ser::to_string_pretty};
use serde::{Deserialize, Serialize};

use std::io::Result as IoResult;

const PROJECT_QUALIFIER: &'static str = "com";
const PROJECT_ORGANIZATION: &'static str = "Example";

pub struct AppConfig {
    name: &'static str,
}

impl AppConfig {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub fn save<C: Serialize>(&self, file: &str, config: C) {
        let Ok(_) = self.ensure_config_dir() else {
            bevy_log::error!("Could not create config directory");
            return;
        };

        let Ok(text) = to_string_pretty(&config, Default::default()) else {
            bevy_log::error!("Could not serialize config");
            return;
        };

        match self.write(file, text) {
            Err(e) => bevy_log::error!("Could not write to '{}' file: {}", file, e.kind()),
            _ => {}
        }
    }

    pub fn load<C: for<'a> Deserialize<'a>>(&self, file: &str) -> Option<C> {
        let mut input = match File::open(self.get_config_path(file)) {
            Ok(file) => Some(file),
            Err(err) => {
                bevy_log::error!("Could not open '{}' file: {}", file, err);
                None
            }
        }?;

        let mut text = String::new();

        match input.read_to_string(&mut text) {
            Err(err) => {
                bevy_log::error!("Could not read from '{}' file: {}", file, err);
                return None;
            }
            _ => {}
        }

        let config = match from_str::<C>(&text) {
            Ok(config) => Some(config),
            Err(_) => {
                bevy_log::error!("Could not deserialize config");
                None
            }
        }?;

        Some(config)
    }

    fn project_dirs(&self) -> ProjectDirs {
        ProjectDirs::from(PROJECT_QUALIFIER, PROJECT_ORGANIZATION, &self.name).unwrap()
    }

    fn ensure_config_dir(&self) -> IoResult<()> {
        let proj_dirs = self.project_dirs();
        fs::create_dir_all(proj_dirs.config_local_dir())?;
        Ok(())
    }

    fn get_config_path(&self, file: &str) -> PathBuf {
        self.project_dirs()
            .config_local_dir()
            .join(file.to_owned() + ".ron")
    }

    fn write(&self, file: &str, contents: String) -> IoResult<()> {
        let mut output = File::create(self.get_config_path(file))?;
        output.write_all(contents.as_bytes())?;
        Ok(())
    }
}

#[macro_export]
macro_rules! define_app_config {
    () => {
        const APP_CONFIG: $crate::AppConfig = $crate::AppConfig::new(env!("CARGO_PKG_NAME"));
    };
}

#[macro_export]
macro_rules! app_config {
    () => {
        crate::APP_CONFIG
    };
}

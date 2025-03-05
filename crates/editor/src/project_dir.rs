use std::{
    ffi::OsStr,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use bevy::ecs::system::Resource;

#[derive(Resource, Clone)]
pub struct ProjectDir(pub PathBuf);

impl Deref for ProjectDir {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProjectDir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<PathBuf> for ProjectDir {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<Path> for ProjectDir {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl AsRef<OsStr> for ProjectDir {
    fn as_ref(&self) -> &OsStr {
        self.0.as_os_str()
    }
}

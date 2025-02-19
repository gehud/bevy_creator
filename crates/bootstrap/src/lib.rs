use bevy::app::App;

pub trait SystemConfig {
    fn add_system(app: &mut App);
}

pub use bevy_bootstrap_macros::*;

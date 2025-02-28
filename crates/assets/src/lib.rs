use bevy_app::{Plugin, Update};
use render::MeshRenderer;

mod render;

pub struct CustomAssetsPlugin;

impl Plugin for CustomAssetsPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.register_type::<MeshRenderer>()
            .add_systems(Update, render::renderer_system);
    }
}

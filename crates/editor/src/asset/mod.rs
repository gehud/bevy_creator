use bevy::app::Plugin;
use processors::StandardMaterialAssetPlugin;

mod processors;

pub struct EditorAssetPlugin;

impl Plugin for EditorAssetPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(StandardMaterialAssetPlugin);
    }
}

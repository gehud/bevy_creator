use std::io::Cursor;

use bevy_app::{Plugin, Startup};
use bevy_ecs::system::NonSend;
use bevy_winit::WinitWindows;
use image::ImageReader;
use winit::window::Icon;

fn setup_window_icon(windows: NonSend<WinitWindows>) {
    let (icon_rgba, icon_width, icon_height) = {
        let image = ImageReader::new(Cursor::new(include_bytes!(
            "../../../assets/branding/icon.png"
        )))
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

    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}

#[derive(Default)]
pub struct WindowIconPlugin;

impl Plugin for WindowIconPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(Startup, setup_window_icon);
    }
}

use bevy::prelude::World;
use bevy_egui::egui::Ui;

pub trait Panel: Send + Sync {
    fn name(&self) -> String;

    #[allow(unused)]
    fn setup(&mut self, world: &mut World) {}
    
    #[allow(unused)]
    fn draw(&mut self, world: &mut World, ui: &mut Ui) {}
    
    fn clear_background(&self) -> bool { true }
}
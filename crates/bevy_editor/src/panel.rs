use bevy::ecs::world::World;
use bevy_egui::egui::Ui;

pub trait Panel: Send + Sync {
    fn name(&self) -> String;

    #[allow(unused)]
    fn setup(&mut self, world: &mut World) {}

    #[allow(unused)]
    fn ui(&mut self, world: &mut World, ui: &mut Ui) {}

    fn clear_background(&self) -> bool {
        true
    }
}

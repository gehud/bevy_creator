use bevy::ecs::world::{Mut, World};
use bevy_egui::egui::Ui;
use bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui;

use crate::{
    editor::{InspectorSelection, InspectorState},
    panel::Panel,
};

#[derive(Default)]
pub struct HierarchyPanel;

impl Panel for HierarchyPanel {
    fn name(&self) -> String {
        "Hierarchy".into()
    }

    fn draw(&mut self, world: &mut World, ui: &mut Ui) {
        world.resource_scope(|world, mut state: Mut<InspectorState>| {
            let selected = hierarchy_ui(world, ui, &mut state.selected_entities);
            if selected {
                state.selection = InspectorSelection::Entities;
            }
        });
    }
}

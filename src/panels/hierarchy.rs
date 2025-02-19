use bevy::ecs::{
    query::With,
    reflect::AppTypeRegistry,
    world::{Mut, World},
};
use bevy_egui::egui::Ui;
use bevy_inspector_egui::bevy_inspector::hierarchy::Hierarchy;
use bevy::scene::SceneInstance;

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

    fn ui(&mut self, world: &mut World, ui: &mut Ui) {
        world.resource_scope(|world, mut state: Mut<InspectorState>| {
            let type_registry = world.resource::<AppTypeRegistry>().clone();
            let type_registry = type_registry.read();

            let selected = Hierarchy {
                world,
                type_registry: &type_registry,
                selected: &mut state.selected_entities,
                context_menu: None,
                shortcircuit_entity: None,
                extra_state: &mut (),
            }
            .show::<With<SceneInstance>>(ui);

            if selected {
                state.selection = InspectorSelection::Entities;
            }
        });
    }
}

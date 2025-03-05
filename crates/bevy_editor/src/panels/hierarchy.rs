use bevy::ecs::query::Without;
use bevy::ecs::{
    reflect::AppTypeRegistry,
    world::{Mut, World},
};
use bevy_egui::egui::Ui;
use bevy_inspector_egui::bevy_inspector::hierarchy::Hierarchy;

use crate::scene::EditorEntity;
use crate::selection::PickSelection;
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
        if ui.button("Create Entity").clicked() {
            world.spawn_empty();
        }

        if ui.button("Delete Selected").clicked() {
            world.resource_scope(|world, mut state: Mut<InspectorState>| {
                for entity in state.selected_entities.iter() {
                    world.despawn(entity);
                }

                state.selected_entities.clear();
            });
        }

        ui.separator();

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
            .show_with_default_filter::<Without<EditorEntity>>(ui);

            if selected {
                state.selection = InspectorSelection::Entities;
            }

            for entity in state.selected_entities.iter() {
                world
                    .entity_mut(entity)
                    .entry::<PickSelection>()
                    .and_modify(|mut selectoin| {
                        selectoin.is_selected = true;
                    });
            }
        });
    }
}

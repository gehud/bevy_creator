use bevy::ecs::{
    reflect::AppTypeRegistry,
    world::{Mut, World},
};
use bevy_egui::egui::Ui;
use bevy_inspector_egui::bevy_inspector::{
    self, ui_for_entities_shared_components, ui_for_entity_with_children,
};

use crate::{
    editor::{InspectorSelection, InspectorState},
    panel::Panel,
};

#[derive(Default)]
pub struct InspectorPanel;

impl Panel for InspectorPanel {
    fn name(&self) -> String {
        "Inspector".into()
    }

    fn draw(&mut self, world: &mut World, ui: &mut Ui) {
        world.resource_scope(|world, state: Mut<InspectorState>| {
            let binding = world.resource::<AppTypeRegistry>().clone();
            let type_registry = binding.read();

            match state.selection {
                InspectorSelection::Entities => match state.selected_entities.as_slice() {
                    &[entity] => ui_for_entity_with_children(world, entity, ui),
                    entities => ui_for_entities_shared_components(world, entities, ui),
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_resource(
                        world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
            }
        });
    }
}

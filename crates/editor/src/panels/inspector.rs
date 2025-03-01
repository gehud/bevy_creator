use bevy::{
    ecs::{
        reflect::{AppTypeRegistry, ReflectComponent},
        world::{Mut, World},
    },
    reflect::prelude::ReflectDefault,
};
use bevy_egui::egui::{Align, Layout, ScrollArea, Ui};
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

    fn ui(&mut self, world: &mut World, ui: &mut Ui) {
        world.resource_scope(|world, mut state: Mut<InspectorState>| {
            let binding = world.resource::<AppTypeRegistry>().clone();
            let type_registry = binding.read();

            match state.selection {
                InspectorSelection::Entities => entity_ui(&mut state, world, ui),
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

fn entity_ui(state: &mut InspectorState, world: &mut World, ui: &mut Ui) {
    match state.selected_entities.as_slice() {
        &[entity] => ui_for_entity_with_children(world, entity, ui),
        entities => ui_for_entities_shared_components(world, entities, ui),
    };

    ui.with_layout(
        Layout::top_down(Align::Center).with_cross_justify(true),
        |ui| {
            ui.menu_button("Add Component", |ui| {
                ui.text_edit_singleline(&mut state.component_filter);

                ScrollArea::new([false, true])
                    .min_scrolled_height(256.0)
                    .max_height(256.0)
                    .show(ui, |ui| {
                        let type_registry = world.resource::<AppTypeRegistry>().clone();
                        let type_registry = type_registry.read();

                        let components = type_registry
                            .iter()
                            .map(|registration| {
                                (
                                    registration.data::<ReflectComponent>(),
                                    registration.data::<ReflectDefault>(),
                                )
                            })
                            .filter(|(component, default)| component.is_some() && default.is_some())
                            .map(|(component, default)| {
                                (
                                    component.unwrap().register_component(world),
                                    component.unwrap(),
                                    default.unwrap(),
                                )
                            })
                            .collect::<Vec<_>>();

                        let components = components
                            .iter()
                            .map(|(id, component, default)| {
                                (
                                    world.components().get_info(*id).unwrap().name().to_string(),
                                    component,
                                    default,
                                )
                            })
                            .filter(|(info, _, _)| {
                                info.to_lowercase()
                                    .contains(&state.component_filter.to_lowercase())
                            })
                            .collect::<Vec<_>>();

                        for (name, component, default) in components {
                            if ui.button(name).clicked() {
                                for entity in state.selected_entities.iter() {
                                    let mut entity = world.entity_mut(entity);
                                    let data = default.default();
                                    component.insert(
                                        &mut entity,
                                        data.as_partial_reflect(),
                                        &type_registry,
                                    );
                                }

                                ui.close_menu();
                            }
                        }
                    });
            });
        },
    );
}

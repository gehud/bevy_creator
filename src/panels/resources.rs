use bevy::ecs::{
        reflect::{AppTypeRegistry, ReflectResource},
        world::{Mut, World},
    };
use bevy_egui::egui::Ui;

use crate::{
    editor::{InspectorSelection, InspectorState},
    panel::Panel,
};

#[derive(Default)]
pub struct ResourcesPanel;

impl Panel for ResourcesPanel {
    fn name(&self) -> String {
        "Resources".into()
    }

    fn draw(&mut self, world: &mut World, ui: &mut Ui) {
        world.resource_scope(|world, mut state: Mut<InspectorState>| {
            let binding = world.resource::<AppTypeRegistry>().clone();
            let type_registry = binding.read();

            let mut resources: Vec<_> = type_registry
                .iter()
                .filter(|registration| registration.data::<ReflectResource>().is_some())
                .map(|registration| {
                    (
                        registration.type_info().type_path_table().short_path(),
                        registration.type_id(),
                    )
                })
                .collect();
            
            resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

            for (resource_name, type_id) in resources {
                let selected = match state.selection {
                    InspectorSelection::Resource(selected, _) => selected == type_id,
                    _ => false,
                };

                if ui.selectable_label(selected, resource_name).clicked() {
                    state.selection =
                        InspectorSelection::Resource(type_id, resource_name.to_string());
                }
            }
        });
    }
}

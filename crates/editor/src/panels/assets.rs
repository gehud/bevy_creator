use bevy::asset::ReflectAsset;
use bevy::ecs::{
    reflect::AppTypeRegistry,
    world::{Mut, World},
};
use bevy_egui::egui::Ui;

use crate::{
    editor::{InspectorSelection, InspectorState},
    panel::Panel,
};

#[derive(Default)]
pub struct AssetsPanel;

impl Panel for AssetsPanel {
    fn name(&self) -> String {
        "Assets".into()
    }

    fn ui(&mut self, world: &mut World, ui: &mut Ui) {
        world.resource_scope(|world, mut state: Mut<InspectorState>| {
            let binding = world.resource::<AppTypeRegistry>().clone();
            let type_registry = binding.read();

            let mut assets: Vec<_> = type_registry
                .iter()
                .filter_map(|registration| {
                    let reflect_asset = registration.data::<ReflectAsset>()?;
                    Some((
                        registration.type_info().type_path_table().short_path(),
                        registration.type_id(),
                        reflect_asset,
                    ))
                })
                .collect();

            assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

            for (asset_name, asset_type_id, reflect_asset) in assets {
                let handles: Vec<_> = reflect_asset.ids(world).collect();

                ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
                    for handle in handles {
                        let selected = match state.selection {
                            InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                            _ => false,
                        };

                        if ui
                            .selectable_label(selected, format!("{:?}", handle))
                            .clicked()
                        {
                            state.selection = InspectorSelection::Asset(
                                asset_type_id,
                                asset_name.to_string(),
                                handle,
                            );
                        }
                    }
                });
            }
        });
    }
}

//! A raycasting backend for [`bevy_egui`]. This backend simply ensures that egui blocks other
//! entities from being picked.

use bevy::picking::backend::{HitData, PointerHits};
use bevy::picking::pointer::{PointerId, PointerLocation};
use bevy::picking::PickSet;
use bevy::prelude::*;
use bevy::render::camera::NormalizedRenderTarget;

use bevy_egui::EguiContext;

use crate::selection::NoDeselect;

/// Adds picking support for [`bevy_egui`], by ensuring that egui blocks other entities from being
/// picked.
#[derive(Clone)]
pub struct EguiPickingPlugin;
impl Plugin for EguiPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            egui_picking.in_set(PickSet::Backend),
        )
        .insert_resource(EguiPickingSettings::default())
        .register_type::<EguiPickingSettings>()
        .add_systems(First, update_settings);
    }
}

/// Settings for the [`EguiPickingPlugin`].
#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource, Default)]
pub struct EguiPickingSettings {
    /// When set to true, clicking on egui will deselect other entities
    pub allow_deselect: bool,
}

/// Marks the entity used as the pseudo egui pointer.
#[derive(Component, Reflect)]
pub struct EguiPointer;

pub fn update_settings(
    mut commands: Commands,
    settings: Res<EguiPickingSettings>,
    egui_context: Query<Entity, With<EguiContext>>,
) {
    if settings.is_added() || settings.is_changed() {
        for entity in &egui_context {
            match settings.allow_deselect {
                true => commands
                    .entity(entity)
                    .remove::<NoDeselect>(),
                false => commands
                    .entity(entity)
                    .try_insert(NoDeselect),
            };
        }
    }
}

/// If egui in the current window is reporting that the pointer is over it, we report a hit.
pub fn egui_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    mut egui_context: Query<(Entity, &mut EguiContext)>,
    mut output: EventWriter<PointerHits>,
) {
    for (pointer, location) in pointers
        .iter()
        .filter_map(|(i, p)| p.location.as_ref().map(|l| (i, l)))
    {
        if let NormalizedRenderTarget::Window(id) = location.target {
            if let Ok((entity, mut ctx)) = egui_context.get_mut(id.entity()) {
                if ctx.get_mut().wants_pointer_input() {
                    let entry = (entity, HitData::new(entity, 0.0, None, None));
                    let order = 1_000_000f32; // Assume egui should be on top of everything else.
                    output.send(PointerHits::new(*pointer, Vec::from([entry]), order));
                }
            }
        }
    }
}

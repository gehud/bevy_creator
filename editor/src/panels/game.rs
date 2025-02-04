use bevy::{
    ecs::{
        query::With,
        system::{In, Query, RunSystemOnce},
        world::{Mut, World},
    },
    math::{Quat, UVec2, Vec3},
    render::camera::{Camera, CameraProjection, Projection, Viewport},
    transform::components::{GlobalTransform, Transform},
    utils::default,
    window::{PrimaryWindow, Window},
};
use bevy_egui::{
    egui::{Rect, Ui},
    EguiSettings,
};
use transform_gizmo_egui::{math::Transform as GizmoTransform, GizmoConfig, GizmoOrientation};

use crate::{
    editor::{GizmoState, InspectorState, MainCamera},
    panel::Panel,
    transform_gizmo_ext::GizmoNewExt,
};

#[derive(Default)]
pub struct GamePanel;

impl Panel for GamePanel {
    fn name(&self) -> String {
        "Game".into()
    }

    fn clear_background(&self) -> bool {
        false
    }

    fn draw(&mut self, world: &mut World, ui: &mut Ui) {
        world
            .run_system_once_with(ui.clip_rect(), set_camera_viewport)
            .unwrap();

        world.resource_scope(|world, mut gizmo_state: Mut<GizmoState>| {
            let transform_entities = {
                let inspector_state = world.resource::<InspectorState>();

                if inspector_state.selected_entities.len() == 0 {
                    None
                } else {
                    Some(
                        inspector_state
                            .selected_entities
                            .iter()
                            .take_while(|item| world.get::<Transform>(*item).is_some())
                            .collect::<Vec<_>>(),
                    )
                }
            };

            let Some(transform_entities) = transform_entities else {
                return;
            };

            if transform_entities.is_empty() {
                return;
            }

            let selections = transform_entities
                .iter()
                .map(|item| {
                    let transform = world.get::<Transform>(*item).unwrap();
                    GizmoTransform::from_scale_rotation_translation(
                        transform.scale.as_dvec3(),
                        transform.rotation.as_dquat(),
                        transform.translation.as_dvec3(),
                    )
                })
                .collect::<Vec<_>>();

            let (cam_transform, projection) = world
                .query_filtered::<(&GlobalTransform, &Projection), With<MainCamera>>()
                .single(world);

            let view_matrix = cam_transform.compute_matrix().inverse();
            let projection_matrix = projection.get_clip_from_view();

            let modes = gizmo_state.gizmo_modes.clone();
            gizmo_state.gizmo.update_config(GizmoConfig {
                view_matrix: view_matrix.as_dmat4().into(),
                projection_matrix: projection_matrix.as_dmat4().into(),
                orientation: GizmoOrientation::Global,
                modes,
                ..default()
            });

            if let Some(result) = gizmo_state
                .gizmo
                .interact_new(ui, &selections)
                .map(|(_, res)| res)
            {
                for (entity, data) in transform_entities.iter().zip(result.iter()) {
                    let mut transform = world.get_mut::<Transform>(*entity).unwrap();
                    transform.translation = Vec3::new(
                        data.translation.x as f32,
                        data.translation.y as f32,
                        data.translation.z as f32,
                    );
                    transform.rotation = Quat::from_xyzw(
                        data.rotation.v.x as f32,
                        data.rotation.v.y as f32,
                        data.rotation.v.z as f32,
                        data.rotation.s as f32,
                    );
                    transform.scale = Vec3::new(
                        data.scale.x as f32,
                        data.scale.y as f32,
                        data.scale.z as f32,
                    );
                }
            };
        });
    }
}

fn set_camera_viewport(
    In(viewport_rect): In<Rect>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Query<&EguiSettings>,
    mut cameras: Query<&mut Camera, With<MainCamera>>,
) {
    let mut cam = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let scale_factor = window.scale_factor() * egui_settings.single().scale_factor;

    let viewport_pos = viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = viewport_rect.size() * scale_factor;

    let physical_position = UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32);
    let physical_size = UVec2::new(viewport_size.x as u32, viewport_size.y as u32);

    // The desired viewport rectangle at its offset in "physical pixel space"
    let rect = physical_position + physical_size;

    let window_size = window.physical_size();
    // wgpu will panic if trying to set a viewport rect which has coordinates extending
    // past the size of the render target, i.e. the physical window in our case.
    // Typically this shouldn't happen- but during init and resizing etc. edge cases might occur.
    // Simply do nothing in those cases.
    if rect.x <= window_size.x && rect.y <= window_size.y {
        cam.viewport = Some(Viewport {
            physical_position,
            physical_size,
            depth: 0.0..1.0,
        });
    }
}

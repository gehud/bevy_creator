use std::f32::consts::PI;

use bevy::{
    app::{Plugin, PostUpdate, Update},
    core_pipeline::core_3d::graph::input,
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        reflect::ReflectComponent,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut},
        world::Mut,
    },
    hierarchy::Parent,
    input::{
        keyboard::{KeyCode, KeyboardInput},
        mouse::{MouseButton, MouseButtonInput},
        ButtonInput, ButtonState,
    },
    math::{Dir3, EulerRot, Quat},
    picking::{
        events::Pointer,
        pointer::{PointerAction, PointerButton, PointerInput, PressDirection},
    },
    reflect::Reflect,
    render::camera::Camera,
    time::Time,
    transform::{
        components::{GlobalTransform, Transform},
        TransformSystem,
    },
    utils::default,
    window::PrimaryWindow,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EditorCamera {
    is_pan: bool,
    is_fly: bool,
    pan_sensitivity: f32,
    speed: f32,
    rotation_sensitivity: f32,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            is_pan: false,
            is_fly: false,
            pan_sensitivity: 0.01,
            speed: 10.,
            rotation_sensitivity: 8.,
        }
    }
}

const ANGLE_LIMIT: f32 = 85.;

#[derive(Default)]
pub struct EditorCameraPlugin;

impl Plugin for EditorCameraPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.register_type::<EditorCamera>().add_systems(
            PostUpdate,
            camera_movement_system.before(TransformSystem::TransformPropagate),
        );
    }
}

fn camera_movement_system(
    time: Res<Time>,
    primary_windows: Query<Entity, With<PrimaryWindow>>,
    mut editor_cameras: Query<(
        &Camera,
        &mut EditorCamera,
        &mut Transform,
        &GlobalTransform,
        &Parent,
    )>,
    mut pointer_inputs: EventReader<PointerInput>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let Ok(primary_window) = primary_windows.get_single() else {
        return;
    };

    let Ok((camera, mut editor_camera, mut transform, global_transform, root)) =
        editor_cameras.get_single_mut()
    else {
        return;
    };

    let mut was_moved = false;

    for input in pointer_inputs.read() {
        let matches_target =
            input.location.target == camera.target.normalize(Some(primary_window)).unwrap();

        match input.action {
            PointerAction::Moved { delta } => {
                if !was_moved {
                    if editor_camera.is_pan {
                        let right = global_transform.right();
                        let up = global_transform.up();
                        let offset = up * delta.y * editor_camera.pan_sensitivity
                            - right * delta.x * editor_camera.pan_sensitivity;
                        commands.entity(root.get()).entry::<Transform>().and_modify(
                            move |mut transform| {
                                transform.translation += offset;
                            },
                        );
                    } else if editor_camera.is_fly {
                        let mut pitch = transform.rotation.to_euler(EulerRot::XYZ).0 / PI * 180.;
                        pitch = (pitch - delta.y * editor_camera.rotation_sensitivity * time.delta_secs())
                            .clamp(-ANGLE_LIMIT, ANGLE_LIMIT);

                        transform.rotation =
                            Quat::from_euler(EulerRot::XYZ, pitch * PI / 180., 0., 0.);

                        let yaw_delta = -delta.x * editor_camera.rotation_sensitivity * time.delta_secs();
                        commands.entity(root.get()).entry::<Transform>().and_modify(
                            move |mut transform| {
                                let dir = transform.up();
                                transform.rotate_local_axis(dir, yaw_delta * PI / 180.);
                            },
                        );
                    }

                    was_moved = true;
                }
            }
            PointerAction::Pressed { direction, button } => {
                if button == PointerButton::Middle {
                    if matches_target && direction == PressDirection::Down {
                        editor_camera.is_pan = true;
                    } else if direction == PressDirection::Up {
                        editor_camera.is_pan = false;
                    }
                } else if button == PointerButton::Secondary {
                    if matches_target && direction == PressDirection::Down {
                        editor_camera.is_fly = true;
                    } else if direction == PressDirection::Up {
                        editor_camera.is_fly = false;
                    }
                }
            }
            _ => {}
        }
    }

    if editor_camera.is_fly {
        if keyboard_input.pressed(KeyCode::KeyW) {
            translate_dir(
                &mut commands,
                root.get(),
                &editor_camera,
                global_transform.forward(),
                time.delta_secs(),
            );
        } else if keyboard_input.pressed(KeyCode::KeyS) {
            translate_dir(
                &mut commands,
                root.get(),
                &editor_camera,
                global_transform.back(),
                time.delta_secs(),
            );
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            translate_dir(
                &mut commands,
                root.get(),
                &editor_camera,
                global_transform.left(),
                time.delta_secs(),
            );
        } else if keyboard_input.pressed(KeyCode::KeyD) {
            translate_dir(
                &mut commands,
                root.get(),
                &editor_camera,
                global_transform.right(),
                time.delta_secs(),
            );
        }

        if keyboard_input.pressed(KeyCode::KeyQ) {
            translate_dir(
                &mut commands,
                root.get(),
                &editor_camera,
                global_transform.down(),
                time.delta_secs(),
            );
        } else if keyboard_input.pressed(KeyCode::KeyE) {
            translate_dir(
                &mut commands,
                root.get(),
                &editor_camera,
                global_transform.up(),
                time.delta_secs(),
            );
        }
    }
}

fn translate_dir(
    commands: &mut Commands,
    root: Entity,
    editor_camera: &EditorCamera,
    dir: Dir3,
    delta_secs: f32,
) {
    let offset = dir * editor_camera.speed * delta_secs;
    commands
        .entity(root)
        .entry::<Transform>()
        .and_modify(move |mut transform| {
            transform.translation += offset;
        });
}

use std::f32::consts::PI;

use bevy::{
    app::{Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        reflect::ReflectComponent,
        system::{Commands, Query, Res},
    },
    hierarchy::Parent,
    input::{
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion},
        ButtonInput,
    },
    math::{Dir3, EulerRot, Quat},
    picking::pointer::PointerInput,
    reflect::Reflect,
    render::camera::Camera,
    time::Time,
    transform::components::{GlobalTransform, Transform},
    window::{CursorGrabMode, PrimaryWindow, Window},
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
        app.register_type::<EditorCamera>()
            .add_systems(Update, camera_movement_system);
    }
}

fn camera_movement_system(
    time: Res<Time>,
    mut primary_windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut editor_cameras: Query<(
        &Camera,
        &mut EditorCamera,
        &mut Transform,
        &GlobalTransform,
        &Parent,
    )>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motions: EventReader<MouseMotion>,
    mut pointer_inputs: EventReader<PointerInput>,
    mut commands: Commands,
) {
    let Ok((window_entity, mut window)) = primary_windows.get_single_mut() else {
        return;
    };

    let Ok((camera, mut editor_camera, mut transform, global_transform, root)) =
        editor_cameras.get_single_mut()
    else {
        return;
    };

    let camera_target = camera.target.normalize(Some(window_entity)).unwrap();

    let matches_target = pointer_inputs
        .read()
        .any(|input| input.location.target == camera_target);

    if matches_target && mouse_input.just_pressed(MouseButton::Middle) {
        editor_camera.is_pan = true;
    } else if mouse_input.just_released(MouseButton::Middle) {
        editor_camera.is_pan = false;
    }

    if matches_target && mouse_input.just_pressed(MouseButton::Right) {
        editor_camera.is_fly = true;
    } else if mouse_input.just_released(MouseButton::Right) {
        editor_camera.is_fly = false;
    }

    if editor_camera.is_pan || editor_camera.is_fly {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    } else {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }

    for motion in mouse_motions.read() {
        if editor_camera.is_pan {
            let right = global_transform.right().normalize();
            let up = global_transform.up().normalize();
            let offset = up * motion.delta.y * editor_camera.pan_sensitivity
                - right * motion.delta.x * editor_camera.pan_sensitivity;
            commands
                .entity(root.get())
                .entry::<Transform>()
                .and_modify(move |mut transform| {
                    transform.translation += offset;
                });
        } else if editor_camera.is_fly {
            let mut pitch = transform.rotation.to_euler(EulerRot::XYZ).0 / PI * 180.;
            pitch = (pitch
                - motion.delta.y * editor_camera.rotation_sensitivity * time.delta_secs())
            .clamp(-ANGLE_LIMIT, ANGLE_LIMIT);

            transform.rotation = Quat::from_euler(EulerRot::XYZ, pitch * PI / 180., 0., 0.);

            let yaw_delta =
                -motion.delta.x * editor_camera.rotation_sensitivity * time.delta_secs();
            commands
                .entity(root.get())
                .entry::<Transform>()
                .and_modify(move |mut transform| {
                    transform.rotate_local_y(yaw_delta * PI / 180.);
                });
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
    let offset = dir.normalize() * editor_camera.speed * delta_secs;
    commands
        .entity(root)
        .entry::<Transform>()
        .and_modify(move |mut transform| {
            transform.translation += offset;
        });
}

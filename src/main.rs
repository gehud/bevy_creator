use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Pointer},
    picking_core::Pickable,
    prelude::EntityEvent,
    selection::Deselect,
    *,
};
use ui_plugin::{MainCamera, UiPlugin, UiState};
use window_persistence::WindowPersistencePlugin;

pub mod file_io;
mod ui_plugin;
mod window_persistence;

pub struct GameEditorPlugin;

impl Plugin for GameEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Game Editor"),
                resolution: (1920., 1080.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(UiPlugin)
        .add_plugins(WindowPersistencePlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, auto_add_raycast_target)
        .add_systems(Update, handle_pick_events)
        .register_type::<Option<Handle<Image>>>()
        .register_type::<AlphaMode>();
    }
}

fn auto_add_raycast_target(
    mut commands: Commands,
    query: Query<Entity, (Without<Pickable>, With<Handle<Mesh>>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(PickableBundle::default());
    }
}

fn handle_pick_events(
    mut ui_state: ResMut<UiState>,
    mut click_events: EventReader<Pointer<Click>>,
    mut deselect_events: EventReader<Pointer<Deselect>>,
    input: Res<Input<KeyCode>>,
) {
    for click in click_events.read() {
        let do_add = input.any_pressed([
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
        ]);

        ui_state
            .selected_entities
            .select_maybe_add(click.target(), do_add);
    }

    for _ in deselect_events.read() {
        ui_state.selected_entities.clear();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let box_size = 2.0;
    let box_thickness = 0.15;
    let box_offset = (box_size + box_thickness) / 2.0;

    // left - red
    let mut transform = Transform::from_xyz(-box_offset, box_offset, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            box_size,
            box_thickness,
            box_size,
        ))),
        transform,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.63, 0.065, 0.05),
            ..Default::default()
        }),
        ..Default::default()
    });
    // right - green
    let mut transform = Transform::from_xyz(box_offset, box_offset, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            box_size,
            box_thickness,
            box_size,
        ))),
        transform,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.14, 0.45, 0.091),
            ..Default::default()
        }),
        ..Default::default()
    });
    // bottom - white
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size,
        ))),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.725, 0.71, 0.68),
            ..Default::default()
        }),
        ..Default::default()
    });
    // top - white
    let transform = Transform::from_xyz(0.0, 2.0 * box_offset, 0.0);
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size,
        ))),
        transform,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.725, 0.71, 0.68),
            ..Default::default()
        }),
        ..Default::default()
    });
    // back - white
    let mut transform = Transform::from_xyz(0.0, box_offset, -box_offset);
    transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size + 2.0 * box_thickness,
        ))),
        transform,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.725, 0.71, 0.68),
            ..Default::default()
        }),
        ..Default::default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });
    // top light
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(0.4))),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_x(std::f32::consts::PI),
                Vec3::new(0.0, box_size + 0.5 * box_thickness, 0.0),
            )),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: Color::WHITE * 100.0,
                ..Default::default()
            }),
            ..Default::default()
        })
        .with_children(|builder| {
            builder.spawn(PointLightBundle {
                point_light: PointLight {
                    color: Color::WHITE,
                    intensity: 25.0,
                    ..Default::default()
                },
                transform: Transform::from_translation((box_thickness + 0.05) * Vec3::Y),
                ..Default::default()
            });
        });
    // directional light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
        ..Default::default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            transform: Transform::from_xyz(0.0, box_offset, 4.0)
                .looking_at(Vec3::new(0.0, box_offset, 0.0), Vec3::Y),
            ..Default::default()
        },
        MainCamera,
        UiCameraConfig { show_ui: false },
        // PickRaycastSource,
    ));
}

fn main() {
    App::new().add_plugins(GameEditorPlugin).run();
}

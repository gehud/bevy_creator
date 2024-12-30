// use bevy_mod_picking::{
//     events::{Click, Pointer},
//     picking_core::Pickable,
//     prelude::EntityEvent,
//     selection::Deselect,
//     *,
// };
// use backends::raycast::bevy_mod_raycast::prelude::RaycastSource;

use bevy::app::{App, Plugin, PluginGroup, Startup, Update};
use bevy::asset::{Assets, Handle};
use bevy::color::{Color, LinearRgba};
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::change_detection::Res;
use bevy::ecs::event::EventReader;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{With, Without};
use bevy::ecs::system::{Commands, ResMut, Query};
use bevy::hierarchy::{BuildChildren, ChildBuild};
use bevy::image::Image;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::math::{
    prelude::{Cuboid, Plane3d},
    Mat4, Quat, Vec2, Vec3,
};
use bevy::pbr::{AmbientLight, MeshMaterial3d, PbrBundle, PointLight, StandardMaterial};
use bevy::picking::events::{Cancel, Click, Pointer};
use bevy::picking::mesh_picking::RayCastPickable;
use bevy::prelude::MeshPickingPlugin;
use bevy::render::{
    alpha::AlphaMode,
    camera::Camera,
    mesh::{Mesh, Mesh3d},
};
use bevy::transform::components::Transform;
use bevy::utils::default;
use bevy::window::{PresentMode, Window, WindowPlugin};
use bevy::DefaultPlugins;

pub mod file_io;

mod ui_plugin;
use transform_gizmo_bevy::GizmoCamera;
use ui_plugin::{MainCamera, UiPlugin, UiState};

mod window_persistence;
use window_persistence::WindowPersistencePlugin;

pub struct GameEditorPlugin;

impl Plugin for GameEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                title: String::from("BevyCreator"),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(WindowPersistencePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_pick_events)
        .register_type::<Option<Handle<Image>>>()
        .register_type::<AlphaMode>();
    }
}

fn handle_pick_events(
    mut ui_state: ResMut<UiState>,
    mut click_events: EventReader<Pointer<Click>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for click in click_events.read() {
        bevy::log::info!("{:?}", click.target);
        let do_add = input.any_pressed([
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
        ]);

        ui_state
            .selected_entities
            .select_maybe_add(click.target, do_add);
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
    commands.spawn((
        transform,
        Mesh3d(meshes.add(Cuboid::new(box_size, box_thickness, box_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.63, 0.065, 0.05),
            ..Default::default()
        })),
    ));

    // right - green
    let mut transform = Transform::from_xyz(box_offset, box_offset, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    commands.spawn((
        transform,
        Mesh3d(meshes.add(Cuboid::new(box_size, box_thickness, box_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.45, 0.091),
            ..Default::default()
        })),
    ));

    // bottom - white
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
    ));

    // top - white
    let transform = Transform::from_xyz(0.0, 2.0 * box_offset, 0.0);
    commands.spawn((
        transform,
        Mesh3d(meshes.add(Cuboid::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
    ));

    // back - white
    let mut transform = Transform::from_xyz(0.0, box_offset, -box_offset);
    transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands.spawn((
        transform,
        Mesh3d(meshes.add(Cuboid::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size + 2.0 * box_thickness,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
    ));

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 50.0,
    });

    // top light
    commands
        .spawn((
            Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_x(std::f32::consts::PI),
                Vec3::new(0.0, box_size + 0.5 * box_thickness, 0.0),
            )),
            Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::ONE * 0.4))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::WHITE * 100.0,
                ..Default::default()
            })),
        ))
        .with_children(|builder| {
            builder.spawn((
                Transform::from_translation((box_thickness + 0.05) * Vec3::Y),
                PointLight {
                    color: Color::WHITE,
                    intensity: 4000.0,
                    ..Default::default()
                },
            ));
        });

    // camera
    commands.spawn((
        Transform::from_xyz(0.0, box_offset, 4.0)
            .looking_at(Vec3::new(0.0, box_offset, 0.0), Vec3::Y),
        Camera {
            order: 0,
            ..default()
        },
        Camera3d::default(),
        MainCamera,
        GizmoCamera
    ));
}

fn main() {
    App::new().add_plugins(GameEditorPlugin).run();
}

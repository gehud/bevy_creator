use bevy::{asset::RenderAssetUsages, prelude::*, render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages}, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use crate::{editor::MainCamera, selection::PickSelection, AppState};

pub struct DemoScenePlugin;

impl Plugin for DemoScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Editor), setup_scene);
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Query<&Window, With<PrimaryWindow>>,
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
        PickSelection::default(),
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
        PickSelection::default(),
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
        PickSelection::default(),
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
        PickSelection::default(),
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
        PickSelection::default(),
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
            PickSelection::default(),
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

    let window = windows.single();

    let size = Extent3d {
        width: window.physical_width(),
        height: window.physical_height(),
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    // camera
    commands.spawn((
        Transform::from_xyz(0.0, box_offset, 4.0)
            .looking_at(Vec3::new(0.0, box_offset, 0.0), Vec3::Y),
        Camera3d::default(),
        Camera {
            target: image_handle.into(),
            clear_color: ClearColorConfig::Custom(Color::linear_rgb(0.1, 0.1, 0.1)),
            ..default()
        },
        MainCamera,
    ));
}

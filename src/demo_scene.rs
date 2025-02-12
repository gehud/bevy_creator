use bevy::{
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat, TextureUsages},
};

use crate::{editor::MainCamera, selection::PickSelection, AppState};

pub struct DemoScenePlugin;

impl Plugin for DemoScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Editor), (setup_scene, add_editor_camera));
    }
}

const BOX_SIZE: f32 = 2.0;
const BOX_THICKNESS: f32 = 0.15;
const BOX_OFFSET: f32 = (BOX_SIZE + BOX_THICKNESS) / 2.0;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scenes: ResMut<Assets<DynamicScene>>,
    app_type_registry: Res<AppTypeRegistry>,
) {
    let mut scene_world = World::new();

    scene_world.insert_resource(app_type_registry.clone());

    // left - red
    let mut transform = Transform::from_xyz(-BOX_OFFSET, BOX_OFFSET, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    scene_world.spawn((
        transform,
        Mesh3d(meshes.add(Cuboid::new(BOX_SIZE, BOX_THICKNESS, BOX_SIZE))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.63, 0.065, 0.05),
            ..Default::default()
        })),
        PickSelection::default(),
    ));

    // right - green
    let mut transform = Transform::from_xyz(BOX_OFFSET, BOX_OFFSET, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    scene_world.spawn((
        transform,
        Mesh3d(meshes.add(Cuboid::new(BOX_SIZE, BOX_THICKNESS, BOX_SIZE))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.45, 0.091),
            ..Default::default()
        })),
        PickSelection::default(),
    ));

    // bottom - white
    scene_world.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            BOX_SIZE + 2.0 * BOX_THICKNESS,
            BOX_THICKNESS,
            BOX_SIZE,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
        PickSelection::default(),
    ));

    // top - white
    let transform = Transform::from_xyz(0.0, 2.0 * BOX_OFFSET, 0.0);
    scene_world.spawn((
        transform,
        Mesh3d(meshes.add(Cuboid::new(
            BOX_SIZE + 2.0 * BOX_THICKNESS,
            BOX_THICKNESS,
            BOX_SIZE,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
        PickSelection::default(),
    ));

    // back - white
    let mut transform = Transform::from_xyz(0.0, BOX_OFFSET, -BOX_OFFSET);
    transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    scene_world.spawn((
        transform,
        Mesh3d(meshes.add(Cuboid::new(
            BOX_SIZE + 2.0 * BOX_THICKNESS,
            BOX_THICKNESS,
            BOX_SIZE + 2.0 * BOX_THICKNESS,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
        PickSelection::default(),
    ));

    // ambient light
    scene_world.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 50.0,
    });

    // top light
    scene_world
        .spawn((
            Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_x(std::f32::consts::PI),
                Vec3::new(0.0, BOX_SIZE + 0.5 * BOX_THICKNESS, 0.0),
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
                Transform::from_translation((BOX_THICKNESS + 0.05) * Vec3::Y),
                PointLight {
                    color: Color::WHITE,
                    intensity: 4000.0,
                    ..Default::default()
                },
            ));
        });

    let scene = DynamicScene::from_world(&scene_world);
    commands.spawn(DynamicSceneRoot(scenes.add(scene)));
}

fn add_editor_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        default(),
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );

    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    commands.spawn((
        Transform::from_xyz(0.0, BOX_OFFSET, 4.0)
            .looking_at(Vec3::new(0.0, BOX_OFFSET, 0.0), Vec3::Y),
        Camera3d::default(),
        Camera {
            target: image_handle.into(),
            clear_color: ClearColorConfig::Custom(Color::linear_rgb(0.1, 0.1, 0.1)),
            ..default()
        },
        MainCamera,
    ));
}

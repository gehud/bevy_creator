use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::color::{Gray, LinearRgba};
use bevy::core::Name;
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::component::Component;
use bevy::ecs::reflect::ReflectComponent;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::{
    reflect::AppTypeRegistry,
    system::{Commands, Res, ResMut},
    world::World,
};
use bevy::gizmos::config::{GizmoConfigGroup, GizmoConfigStore};
use bevy::gizmos::gizmos::Gizmos;
use bevy::gizmos::AppGizmoBuilder;
use bevy::image::Image;
use bevy::math::UVec2;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::reflect::Reflect;
use bevy::render::{
    camera::Camera,
    mesh::{Mesh, Mesh3d},
    render_resource::{TextureDimension, TextureFormat, TextureUsages},
};
use bevy::scene::{DynamicScene, DynamicSceneRoot};
use bevy::state::condition::in_state;
use bevy::state::state::OnEnter;
use bevy::tasks::block_on;
use bevy::transform::components::Transform;
use bevy::utils::default;
use std::f32::consts::PI;

use crate::{editor::MainCamera, selection::PickSelection, AppState};

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct EditorGizmosGroup;

pub struct DemoScenePlugin;

impl Plugin for DemoScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<EditorGizmosGroup>()
            .add_systems(
                OnEnter(AppState::Editor),
                (setup_editor_scene, init_scene, setup_gizmos).chain(),
            )
            .add_systems(Update, draw_gizmo.run_if(in_state(AppState::Editor)));
    }
}

const BOX_SIZE: f32 = 2.0;
const BOX_THICKNESS: f32 = 0.15;
const BOX_OFFSET: f32 = (BOX_SIZE + BOX_THICKNESS) / 2.0;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct EditorMateralTarget;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct EditorMeshTarget;

fn init_scene(
    mut commands: Commands,
    mut scenes: ResMut<Assets<DynamicScene>>,
    app_type_registry: Res<AppTypeRegistry>,
    asset_server: Res<AssetServer>,
) {
    let mut scene_world = World::new();

    scene_world.spawn((
        Mesh3d(asset_server.load::<Mesh>("models/cube.glb#Mesh0/Primitive0")),
        MeshMaterial3d(asset_server.load::<StandardMaterial>("materials/test.mat")),
        EditorMateralTarget,
        EditorMeshTarget,
        PickSelection::default(),
    ));

    scene_world.insert_resource(app_type_registry.clone());
    let scene = DynamicScene::from_world(&scene_world);
    commands.spawn((DynamicSceneRoot(scenes.add(scene)), Name::new("Untitled")));
}

fn setup_editor_scene(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
            ..default()
        },
        MainCamera,
    ));
}

fn setup_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<EditorGizmosGroup>();
    config.line_width = 0.5;
}

fn draw_gizmo(mut gizmos: Gizmos<EditorGizmosGroup>) {
    gizmos.grid(
        Quat::from_rotation_x(PI / 2.),
        UVec2::splat(20),
        Vec2::new(2., 2.),
        // Light gray
        LinearRgba::gray(0.65),
    );
}

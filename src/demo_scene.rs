use std::f32::consts::PI;
use std::ops::Deref;
use std::sync::{Arc, RwLockReadGuard};

use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::processor::{self, AssetProcessor, LoadAndSave, LoadTransformAndSave};
use bevy::asset::saver::AssetSaver;
use bevy::asset::transformer::IdentityAssetTransformer;
use bevy::asset::{
    Asset, AssetApp, AssetId, AssetLoader, AssetServer, Assets, AsyncWriteExt, Handle, StrongHandle,
};
use bevy::color::{Color, Gray, LinearRgba};
use bevy::core::Name;
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::reflect::ReflectComponent;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::Query;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::ecs::{
    reflect::AppTypeRegistry,
    system::{Commands, Res, ResMut},
    world::World,
};
use bevy::gizmos::config::{GizmoConfigGroup, GizmoConfigStore};
use bevy::gizmos::gizmos::Gizmos;
use bevy::gizmos::AppGizmoBuilder;
use bevy::hierarchy::{BuildChildren, ChildBuild};
use bevy::image::Image;
use bevy::math::primitives::Capsule3d;
use bevy::math::UVec2;
use bevy::math::{
    primitives::{Cuboid, Plane3d},
    Mat4, Quat, Vec2, Vec3,
};
use bevy::pbr::{AmbientLight, Material, MeshMaterial3d, PointLight, StandardMaterial};
use bevy::reflect::erased_serde::deserialize;
use bevy::reflect::serde::{ReflectDeserializer, ReflectSerializeWithRegistry, ReflectSerializer};
use bevy::reflect::{
    FromReflect, PartialReflect, Reflect, ReflectDeserialize, ReflectSerialize, TypeRegistry,
};
use bevy::render::{
    camera::Camera,
    mesh::{Mesh, Mesh3d},
    render_resource::{TextureDimension, TextureFormat, TextureUsages},
};
use bevy::scene::ron::ser::{to_string_pretty, PrettyConfig};
use bevy::scene::ron::{to_string, Deserializer};
use bevy::scene::{DynamicScene, DynamicSceneRoot};
use bevy::state::condition::in_state;
use bevy::state::state::OnEnter;
use bevy::tasks::block_on;
use bevy::transform::components::Transform;
use bevy::utils::default;
use serde::de::DeserializeSeed;
use uuid::Uuid;

use crate::assets::EditorAsset;
use crate::{editor::MainCamera, selection::PickSelection, AppState};

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct EditorGizmosGroup;

pub struct DemoScenePlugin;

impl Plugin for DemoScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<EditorGizmosGroup>()
            .register_type::<EditorMateralTarget>()
            .add_systems(
                OnEnter(AppState::Editor),
                (setup_editor_scene, init_scene, setup_gizmos).chain(),
            )
            .add_systems(
                Update,
                (draw_gizmo, asset_assign).run_if(in_state(AppState::Editor)),
            );
    }
}

const BOX_SIZE: f32 = 2.0;
const BOX_THICKNESS: f32 = 0.15;
const BOX_OFFSET: f32 = (BOX_SIZE + BOX_THICKNESS) / 2.0;

#[derive(Component)]
struct EditorMaterialAssign {
    handle: Handle<EditorAsset<StandardMaterial>>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct EditorMateralTarget;

fn init_scene(
    mut commands: Commands,
    mut scenes: ResMut<Assets<DynamicScene>>,
    mut meshes: ResMut<Assets<Mesh>>,
    app_type_registry: Res<AppTypeRegistry>,
    asset_server: Res<AssetServer>,
) {
    let mut scene_world = World::new();

    scene_world.spawn((
        Mesh3d(meshes.add(Capsule3d::default())),
        MeshMaterial3d(Handle::<StandardMaterial>::default()),
        EditorMateralTarget,
        PickSelection::default(),
    ));

    commands.spawn(EditorMaterialAssign {
        handle: asset_server.load::<EditorAsset<StandardMaterial>>("materials/test.std.mat"),
    });

    scene_world.insert_resource(app_type_registry.clone());
    let scene = DynamicScene::from_world(&scene_world);
    commands.spawn((DynamicSceneRoot(scenes.add(scene)), Name::new("Untitled")));
}

fn asset_assign(
    mut material_assigns: Query<(Entity, &EditorMaterialAssign)>,
    mut material_targets: Query<
        (Entity, &mut MeshMaterial3d<StandardMaterial>),
        With<EditorMateralTarget>,
    >,
    mut editor_materials: ResMut<Assets<EditorAsset<StandardMaterial>>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for (entity, assign) in material_assigns.iter_mut() {
        if let Some(editor_material) = editor_materials.remove(&assign.handle) {
            materials.insert(editor_material.uuid.clone(), editor_material.asset);

            for (target, mut mesh_material) in material_targets.iter_mut() {
                mesh_material.0 = Handle::Weak(AssetId::Uuid {
                    uuid: editor_material.uuid.clone(),
                });

                commands.entity(target).remove::<EditorMateralTarget>();
            }

            commands.entity(entity).despawn();
        }
    }
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

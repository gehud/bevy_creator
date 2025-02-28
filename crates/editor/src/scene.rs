use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::color::{Gray, LinearRgba};
use bevy::core::Name;
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::Query;
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
use bevy::reflect::Reflect;
use bevy::render::mesh::Mesh;
use bevy::render::{
    camera::Camera,
    render_resource::{TextureDimension, TextureFormat, TextureUsages},
};
use bevy::scene::{DynamicScene, DynamicSceneRoot};
use bevy::transform::components::Transform;
use bevy::utils::default;
use std::f32::consts::PI;

use crate::editor::MainCamera;

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct EditorGizmosGroup;

pub struct EditorScenePlugin;

impl Plugin for EditorScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<EditorGizmosGroup>()
            .add_systems(
                Startup,
                (setup_editor_scene, mark_editor_entities, setup_gizmos).chain(),
            )
            .add_systems(Update, draw_gizmo);
    }
}

const BOX_SIZE: f32 = 2.0;
const BOX_THICKNESS: f32 = 0.15;
const BOX_OFFSET: f32 = (BOX_SIZE + BOX_THICKNESS) / 2.0;

fn setup_editor_scene(asset_server: Res<AssetServer>, mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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

#[derive(Component)]
pub struct EditorEntity;

fn mark_editor_entities(entities: Query<Entity>, mut commands: Commands) {
    for entity in entities.iter() {
        commands.entity(entity).insert(EditorEntity);
    }
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

use bevy::app::{App, Plugin, Startup};
use bevy::asset::Assets;
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::Query;
use bevy::ecs::system::{Commands, ResMut};
use bevy::gizmos::config::GizmoConfigGroup;
use bevy::gizmos::AppGizmoBuilder;
use bevy::hierarchy::{BuildChildren, ChildBuild};
use bevy::image::Image;
use bevy::math::Vec3;
use bevy::reflect::Reflect;
use bevy::render::view::InheritedVisibility;
use bevy::render::{
    camera::Camera,
    render_resource::{TextureDimension, TextureFormat, TextureUsages},
};
use bevy::transform::components::Transform;
use bevy::utils::default;

use crate::camera::{EditorCamera, EditorCameraPlugin};
use crate::grid::{InfiniteGridBundle, InfiniteGridPlugin};

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct EditorGizmosGroup;

pub struct EditorScenePlugin;

impl Plugin for EditorScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InfiniteGridPlugin)
            .add_plugins(EditorCameraPlugin)
            .init_gizmo_group::<EditorGizmosGroup>()
            .add_systems(Startup, (setup_editor_scene, mark_editor_entities).chain());
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

    commands
        .spawn((
            Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            InheritedVisibility::default(),
        ))
        .with_children(|builder| {
            builder.spawn((
                Camera3d::default(),
                Camera {
                    target: image_handle.into(),
                    ..default()
                },
                EditorCamera::default(),
            ));
        });

    commands.spawn(InfiniteGridBundle::default());
}

#[derive(Component)]
pub struct EditorEntity;

fn mark_editor_entities(entities: Query<Entity>, mut commands: Commands) {
    for entity in entities.iter() {
        commands.entity(entity).insert(EditorEntity);
    }
}

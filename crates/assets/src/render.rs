use bevy_asset::{AssetServer, Handle};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::Changed,
    reflect::ReflectComponent,
    system::{Commands, Query, Res},
};
use bevy_pbr::{MeshMaterial3d, StandardMaterial};
use bevy_reflect::{prelude::ReflectDefault, Reflect};
use bevy_render::mesh::Mesh3d;

#[derive(Default, Component, Reflect)]
#[reflect(Default, Component)]
pub struct MeshRenderer {
    mesh: String,
    material: String,
}

pub fn renderer_system(
    renderers: Query<(Entity, &MeshRenderer), Changed<MeshRenderer>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (entity, renderer) in renderers.iter() {
        let mesh_handle = asset_server.load(renderer.mesh.clone());
        let material_handle = if renderer.material.is_empty() {
            Default::default()
        } else {
            asset_server.load(renderer.material.clone())
        };

        commands
            .entity(entity)
            .entry::<Mesh3d>()
            .or_insert(Mesh3d(mesh_handle.clone()))
            .and_modify(|mut mesh| {
                mesh.0 = mesh_handle;
            });

        commands
            .entity(entity)
            .entry::<MeshMaterial3d<StandardMaterial>>()
            .or_insert(MeshMaterial3d(material_handle.clone()))
            .and_modify(|mut material| {
                material.0 = material_handle;
            });
    }
}

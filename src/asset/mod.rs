use std::{convert::Infallible, ops::Deref, path::Path};

use bevy::{
    app::{Plugin, Startup, Update},
    asset::{
        meta::Settings,
        processor::{AssetProcessor, LoadTransformAndSave},
        saver::AssetSaver,
        transformer::{AssetTransformer, IdentityAssetTransformer, TransformedAsset},
        Asset, AssetApp, AssetEvent, AssetLoader, AssetServer, AsyncWriteExt, Handle,
    },
    ecs::{
        event::EventReader,
        reflect::AppTypeRegistry,
        schedule::{IntoSystemConfigs, IntoSystemSetConfigs, SystemSet},
        system::Res,
    },
    pbr::StandardMaterial,
    reflect::{
        serde::{ReflectDeserializer, ReflectSerializer},
        FromReflect, PartialReflect, Reflect, TypePath,
    },
    render::mesh::Mesh,
    scene::ron::{self, ser::to_string_pretty, Deserializer},
    utils::default,
};
use processors::{MeshAssetPlugin, StandardMaterialAssetPlugin};
use serde::{de::DeserializeSeed, Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

mod processors;

#[derive(Asset, TypePath, Debug)]
pub struct EditorAsset<A: Asset + Reflect> {
    pub uuid: Uuid,
    pub asset: A,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EditorAssetSettings<S: Settings = ()> {
    pub uuid: Uuid,
    pub settings: S,
}

impl<S: Settings + Default> Default for EditorAssetSettings<S> {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            settings: Default::default(),
        }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditorAssetSet {
    ProcessorRegistration,
}

pub struct EditorAssetPlugin;

impl Plugin for EditorAssetPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.configure_sets(
            Startup,
            EditorAssetSet::ProcessorRegistration.before(AssetProcessor::start),
        )
        .add_plugins(StandardMaterialAssetPlugin)
        .add_plugins(MeshAssetPlugin);
    }
}

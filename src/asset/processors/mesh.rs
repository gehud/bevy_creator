use bevy::{
    app::{Plugin, Startup},
    asset::{
        processor::{AssetProcessor, LoadTransformAndSave},
        saver::AssetSaver,
        transformer::IdentityAssetTransformer,
        AssetApp, AssetLoader, AssetServer, AsyncWriteExt,
    },
    ecs::{reflect::AppTypeRegistry, schedule::IntoSystemConfigs, system::Res},
    reflect::{
        serde::{ReflectDeserializer, ReflectSerializer},
        FromReflect,
    },
    render::mesh::Mesh,
    scene::ron::{self, ser::to_string_pretty, Deserializer},
    utils::default,
};

use crate::asset::{EditorAsset, EditorAssetSet, EditorAssetSettings};

use serde::de::DeserializeSeed;

use thiserror::Error;

pub struct MeshAssetPlugin;

impl Plugin for MeshAssetPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_asset::<EditorAsset<Mesh>>()
            .add_systems(Startup, start.in_set(EditorAssetSet::ProcessorRegistration));
    }
}

fn start(
    asset_server: Res<AssetServer>,
    asset_processor: Option<Res<AssetProcessor>>,
    app_type_registry: Res<AppTypeRegistry>,
) {
    asset_server.register_loader(ModelLoader);

    if let Some(asset_processor) = asset_processor {
        asset_processor.register_processor(ModelProcessor::new(
            default(),
            ModelSaver {
                type_registry: app_type_registry.clone(),
            },
        ));

        asset_processor.set_default_processor::<ModelProcessor>("glb");
    }
}

#[derive(Debug, Error)]
enum ModelLoaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RonSpannedError(#[from] ron::error::SpannedError),
    #[error(transparent)]
    LoadDirectError(#[from] bevy::asset::LoadDirectError),
}

struct ModelLoader;

impl AssetLoader for ModelLoader {
    type Asset = EditorAsset<Mesh>;

    type Settings = EditorAssetSettings;

    type Error = ModelLoaderError;

    async fn load(
        &self,
        _reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let path = load_context.path().to_path_buf().clone();
        let mesh = load_context.loader().immediate().load::<Mesh>(path).await?;
        Ok(EditorAsset::<Mesh> {
            uuid: settings.uuid,
            asset: mesh.take(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["glb"]
    }
}

struct MeshLoader {
    type_registry: AppTypeRegistry,
}

impl AssetLoader for MeshLoader {
    type Asset = EditorAsset<Mesh>;

    type Settings = EditorAssetSettings;

    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let type_registry = self.type_registry.read();
        let reflect_deserializer = ReflectDeserializer::new(&type_registry);
        let mut deserializer = Deserializer::from_bytes(&bytes).unwrap();
        let output = reflect_deserializer.deserialize(&mut deserializer).unwrap();

        Ok(EditorAsset::<Mesh> {
            uuid: settings.uuid,
            asset: Mesh::from_reflect(output.as_partial_reflect()).unwrap(),
        })
    }
}

struct ModelSaver {
    type_registry: AppTypeRegistry,
}

impl AssetSaver for ModelSaver {
    type Asset = EditorAsset<Mesh>;

    type Settings = ();

    type OutputLoader = MeshLoader;

    type Error = std::io::Error;

    async fn save(
        &self,
        writer: &mut bevy::asset::io::Writer,
        asset: bevy::asset::saver::SavedAsset<'_, Self::Asset>,
        _settings: &Self::Settings,
    ) -> Result<<Self::OutputLoader as AssetLoader>::Settings, Self::Error> {
        let text = {
            let type_registry = self.type_registry.read();
            let reflect_serializer = ReflectSerializer::new(&asset.asset, &type_registry);
            to_string_pretty(&reflect_serializer, default()).unwrap()
        };

        writer.write_all(text.as_bytes()).await?;

        Ok(EditorAssetSettings {
            uuid: asset.uuid,
            settings: (),
        })
    }
}

type ModelProcessor = LoadTransformAndSave<
    ModelLoader,
    IdentityAssetTransformer<<ModelLoader as AssetLoader>::Asset>,
    ModelSaver,
>;

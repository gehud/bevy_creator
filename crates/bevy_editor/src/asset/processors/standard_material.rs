use bevy::{
    app::{Plugin, Startup},
    asset::{
        processor::{AssetProcessor, LoadTransformAndSave},
        saver::AssetSaver,
        transformer::IdentityAssetTransformer,
        AssetLoader, AssetServer, AsyncWriteExt,
    },
    ecs::{reflect::AppTypeRegistry, schedule::IntoSystemConfigs, system::Res},
    pbr::StandardMaterial,
    reflect::{
        serde::{ReflectDeserializer, ReflectSerializer},
        FromReflect, PartialReflect,
    },
    scene::ron::{ser::to_string_pretty, Deserializer},
    utils::default,
};

use serde::de::DeserializeSeed;

pub struct StandardMaterialAssetPlugin;

impl Plugin for StandardMaterialAssetPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, start.before(AssetProcessor::start));
    }
}

fn start(
    asset_server: Res<AssetServer>,
    asset_processor: Option<Res<AssetProcessor>>,
    app_type_registry: Res<AppTypeRegistry>,
) {
    asset_server.register_loader(StandardMaterialLoader {
        type_registry: app_type_registry.clone(),
    });

    if let Some(asset_processor) = asset_processor {
        asset_processor.register_processor(StandardMaterialProcessor::new(
            default(),
            StandardMaterialSaver {
                type_registry: app_type_registry.clone(),
            },
        ));

        asset_processor.set_default_processor::<StandardMaterialProcessor>("mat");
    }
}

struct StandardMaterialLoader {
    type_registry: AppTypeRegistry,
}

impl AssetLoader for StandardMaterialLoader {
    type Asset = StandardMaterial;

    type Settings = ();

    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let type_registry = self.type_registry.read();
        let reflect_deserializer = ReflectDeserializer::new(&type_registry);
        let mut deserializer = Deserializer::from_bytes(&bytes).unwrap();
        let output = reflect_deserializer.deserialize(&mut deserializer).unwrap();

        Ok(StandardMaterial::from_reflect(output.as_partial_reflect()).unwrap())
    }

    fn extensions(&self) -> &[&str] {
        &["mat"]
    }
}

struct StandardMaterialSaver {
    type_registry: AppTypeRegistry,
}

impl AssetSaver for StandardMaterialSaver {
    type Asset = StandardMaterial;

    type Settings = ();

    type OutputLoader = StandardMaterialLoader;

    type Error = std::io::Error;

    async fn save(
        &self,
        writer: &mut bevy::asset::io::Writer,
        asset: bevy::asset::saver::SavedAsset<'_, Self::Asset>,
        _settings: &Self::Settings,
    ) -> Result<<Self::OutputLoader as AssetLoader>::Settings, Self::Error> {
        let text = {
            let type_registry = self.type_registry.read();
            let reflect_serializer =
                ReflectSerializer::new(asset.as_partial_reflect(), &type_registry);
            to_string_pretty(&reflect_serializer, default()).unwrap()
        };

        writer.write_all(text.as_bytes()).await?;

        Ok(())
    }
}

type StandardMaterialProcessor = LoadTransformAndSave<
    StandardMaterialLoader,
    IdentityAssetTransformer<<StandardMaterialLoader as AssetLoader>::Asset>,
    StandardMaterialSaver,
>;

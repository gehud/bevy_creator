use std::{any::TypeId, marker::PhantomData};

use bevy_app::{Plugin, Update};
use bevy_asset::{Asset, AssetPath, AssetServer, Handle, LoadState};
use bevy_inspector_egui::{
    egui::{self, Color32, Frame, Layout, Margin, Sense, Stroke},
    inspector_egui_impls::{InspectorEguiImpl, InspectorPrimitive},
};
use bevy_pbr::StandardMaterial;
use bevy_reflect::Reflect;
use bevy_render::mesh::Mesh;
use bevy_tasks::block_on;
use render::MeshRenderer;
use serde::{Deserialize, Serialize};

mod render;

#[derive(Serialize, Deserialize, Reflect)]
pub struct AssetRef<A: Asset>(String, #[reflect(ignore)] PhantomData<fn() -> Handle<A>>);

impl<A: Asset> Default for AssetRef<A> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

#[derive(Debug)]
pub struct AssetRefPayload(pub String);

impl<A: Asset> AssetRef<A> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, asset_server: &AssetServer) -> Handle<A> {
        if self.is_empty() {
            return Default::default();
        } else {
            asset_server.load(&self.0)
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl<A: Asset> InspectorPrimitive for AssetRef<A> {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _options: &dyn std::any::Any,
        _id: egui::Id,
        env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) -> bool {
        let mut changed = false;

        let frame = Frame::default().inner_margin(4.);

        let (_, payload) = ui.dnd_drop_zone::<AssetRefPayload, ()>(frame, |ui| {
            if self.is_empty() {
                ui.label("None");
            } else {
                ui.label(&self.0);
            }
        });

        if let Some(payload) = payload {
            self.0 = payload.0.clone();
            changed = true;
        }

        let is_valid = env.context.world.as_ref().is_none_or(|world| {
            let asset_server = unsafe { world.world().get_resource::<AssetServer>().unwrap() };
            match block_on(asset_server.load_untyped_async(&self.0)) {
                Ok(handle) => handle.type_id() == TypeId::of::<A>(),
                Err(_) => false,
            }
        });

        if !is_valid {
            self.0 = String::new();
        }

        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _options: &dyn std::any::Any,
        _id: egui::Id,
        _env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) {
        if self.is_empty() {
            ui.label("None");
        } else {
            ui.label(&self.0);
        }
    }
}

pub struct CustomAssetsPlugin;

impl Plugin for CustomAssetsPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.register_type::<AssetRef<Mesh>>()
            .register_type_data::<AssetRef<Mesh>, InspectorEguiImpl>()
            .register_type::<AssetRef<StandardMaterial>>()
            .register_type_data::<AssetRef<StandardMaterial>, InspectorEguiImpl>()
            .register_type::<MeshRenderer>()
            .add_systems(Update, render::renderer_system);
    }
}

use std::any::TypeId;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command};

use bevy::ecs::component::ComponentId;
use egui_dock::DockArea;
use libloading::{library_filename, Library, Symbol};
use rfd::FileDialog;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};

use bevy::app::{App, AppLabel, Plugin, PluginGroup, PreUpdate, Startup, SubApp};
use bevy::asset::{AssetMode, AssetPlugin, UntypedAssetId};
use bevy::ecs::entity::{Entity, EntityHashMap};
use bevy::ecs::event::{EventReader, EventRegistry};
use bevy::ecs::query::{With, Without};
use bevy::ecs::reflect::AppTypeRegistry;
use bevy::ecs::schedule::{IntoSystemConfigs, IntoSystemSetConfigs};
use bevy::ecs::system::{Commands, Query, ResMut, Resource};
use bevy::ecs::world::{Mut, World};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::picking::events::Pointer;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::picking::PickSet;
use bevy::reflect::TypeRegistry;
use bevy::render::mesh::Mesh3d;
use bevy::render::view::Visibility;
use bevy::scene::ron::Deserializer;
use bevy::scene::serde::SceneDeserializer;
use bevy::scene::DynamicSceneBuilder;
use bevy::time::Time;
use bevy::utils::default;
use bevy::utils::hashbrown::HashMap;
use bevy::window::{PrimaryWindow, Window, WindowPlugin};
use bevy::DefaultPlugins;
use bevy_assets::CustomAssetsPlugin;
use bevy_egui::egui::panel::TopBottomSide;
use bevy_egui::egui::{Id, Modal, TopBottomPanel};
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiPreUpdateSet};
use bevy_helper::winit::WindowIconPlugin;
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use transform_gizmo_egui::{EnumSet, Gizmo, GizmoMode};

use crate::asset::EditorAssetPlugin;
use crate::codegen::{setup_compilation, setup_editing};
use crate::dock::{EditorDockState, PanelViewer, StandardEditorDockStateTemplate};
use crate::editor_config::EditorConfigPlugin;
use crate::egui_config::EguiConfigPlugin;
use crate::egui_picking::EguiPickingPlugin;
use crate::panel::Panel;
use crate::panels::assets::AssetsPanel;
use crate::panels::explorer::ExplorerPanel;
use crate::panels::hierarchy::HierarchyPanel;
use crate::panels::inspector::InspectorPanel;
use crate::panels::resources::ResourcesPanel;
use crate::panels::scene::ScenePanel;
use crate::scene::{EditorEntity, EditorScenePlugin};
use crate::selection::{Deselect, PickSelection, Select, SelectionPlugin};
use crate::window_config::WindowConfigPlugin;
use crate::{
    EditorSet, ProjectDir, PROJECT_ASSET_DIR, PROJECT_CACHE_DIR, PROJECT_IMPORTED_ASSET_DIR,
};

const DEFAULT_PROJECT_DEPENDENCIES: &'static [&'static str] = &[
    "bevy",
    "bevy_a11y",
    "bevy_app",
    "bevy_core",
    "bevy_derive",
    "bevy_diagnostic",
    "bevy_ecs",
    "bevy_state",
    "bevy_hierarchy",
    "bevy_input",
    "bevy_log",
    "bevy_math",
    "bevy_ptr",
    "bevy_reflect",
    "bevy_time",
    "bevy_transform",
    "bevy_utils",
    "bevy_window",
    "bevy_tasks",
    "bevy_animation",
    "bevy_asset",
    "bevy_audio",
    "bevy_color",
    "bevy_core_pipeline",
    "bevy_dev_tools",
    "bevy_gilrs",
    "bevy_gizmos",
    "bevy_gltf",
    "bevy_image",
    "bevy_pbr",
    "bevy_picking",
    "bevy_remote",
    "bevy_render",
    "bevy_scene",
    "bevy_sprite",
    "bevy_text",
    "bevy_ui",
    "bevy_winit",
];

pub const EDITOR_PROJECT_DEPENDENCIES: &'static [&'static str] = &["bevy_bootstrap"];

#[derive(Eq, PartialEq)]
pub enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

pub struct EditorPlugin {
    pub project_dir: PathBuf,
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        let mut asset_dir = PathBuf::from(self.project_dir.clone());
        asset_dir.push(PROJECT_ASSET_DIR);

        let mut imported_asset_dir = PathBuf::from(self.project_dir.clone());
        imported_asset_dir.push(PROJECT_CACHE_DIR);
        imported_asset_dir.push(PROJECT_IMPORTED_ASSET_DIR);

        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "BevyCreator".into(),
                        resolution: (1280., 720.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    mode: AssetMode::Processed,
                    file_path: asset_dir.to_string_lossy().to_string(),
                    processed_file_path: imported_asset_dir.to_string_lossy().to_string(),
                    watch_for_changes_override: true.into(),
                    ..default()
                }),
        )
        .add_plugins(WindowIconPlugin)
        .configure_sets(
            PreUpdate,
            EditorSet::Egui
                .after(EguiPreUpdateSet::BeginPass)
                .before(PickSet::Backend),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EguiPickingPlugin)
        .add_plugins(SelectionPlugin)
        .add_plugins(EditorAssetPlugin)
        .add_plugins(CustomAssetsPlugin)
        .add_plugins(WindowConfigPlugin)
        .add_plugins(EguiConfigPlugin)
        .add_plugins(EditorScenePlugin)
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_plugins(EditorConfigPlugin)
        .insert_resource(EditorState::new())
        .insert_resource(InspectorState::new())
        .insert_resource(GizmoState::new())
        .insert_resource(ProjectDir(self.project_dir.clone()))
        .add_systems(Startup, init_panels)
        .add_systems(
            PreUpdate,
            (handle_selection, show_ui, add_selection)
                .chain()
                .in_set(EditorSet::Egui),
        );
    }
}

#[derive(Resource)]
pub struct EditorState {
    pub docking: EditorDockState,
    pub panels: HashMap<String, Box<dyn Panel>>,
    pub lib: Option<Library>,
    registration_type_ids: Vec<TypeId>,
    registration_component_ids: Vec<ComponentId>,
    pub compilation_process: Option<Child>,
    compilation_animation_time: f32,
}

#[derive(Default, Resource, Serialize, Deserialize)]
pub struct SelectedScene {
    pub active_scene_path: Option<PathBuf>,
}

impl EditorState {
    fn new() -> Self {
        Self {
            docking: EditorDockState::standard(),
            panels: default(),
            lib: default(),
            registration_type_ids: default(),
            registration_component_ids: default(),
            compilation_process: default(),
            compilation_animation_time: default(),
        }
    }

    fn insert_panel<P: Panel + 'static>(&mut self, panel: P) {
        self.panels.insert(panel.name(), Box::new(panel));
    }

    fn init_panel<P: Panel + Default + 'static>(&mut self) {
        self.insert_panel(P::default());
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        TopBottomPanel::new(TopBottomSide::Top, Id::new("menu")).show(ctx, |ui| {
            draw_menu(self, world, ui);
        });

        let mut panel_viewer = PanelViewer {
            world,
            panels: &mut self.panels,
        };

        let dock_style = egui_dock::Style::from_egui(ctx.style().as_ref());

        DockArea::new(&mut self.docking)
            .style(dock_style)
            .show_leaf_collapse_buttons(false)
            .show(ctx, &mut panel_viewer);

        ctx.request_repaint();
    }

    pub fn compile(&mut self, world: &mut World) {
        unload_scene(world);

        {
            let type_registry = world.resource_mut::<AppTypeRegistry>();
            let mut type_registry = type_registry.write();

            for type_id in &self.registration_type_ids {
                type_registry.remove(*type_id);
            }

            self.registration_type_ids.clear();
        }

        {
            for component_id in &self.registration_component_ids {
                world.unregister_component(*component_id);
            }

            self.registration_component_ids.clear();
        }

        if let Some(lib) = self.lib.take() {
            match lib.close() {
                Err(err) => {
                    bevy::log::error!("Failed to unload library: {err}");
                }
                _ => {}
            }
        }

        let project_dir = world.resource::<ProjectDir>().clone();

        setup_compilation(DEFAULT_PROJECT_DEPENDENCIES, &project_dir);

        self.compilation_process = Command::new("cargo")
            .current_dir(project_dir)
            .arg("build")
            .spawn()
            .ok();
    }

    pub fn launch(&mut self, _world: &mut World) {}
}

#[derive(Resource)]
pub struct InspectorState {
    pub selected_entities: SelectedEntities,
    pub selection: InspectorSelection,
    pub component_filter: String,
}

impl InspectorState {
    pub fn new() -> Self {
        Self {
            selection: InspectorSelection::Entities,
            selected_entities: SelectedEntities::default(),
            component_filter: default(),
        }
    }
}

#[derive(Resource)]
pub struct GizmoState {
    pub gizmo: Gizmo,
    pub gizmo_modes: EnumSet<GizmoMode>,
}

impl GizmoState {
    pub fn new() -> Self {
        Self {
            gizmo: Gizmo::default(),
            gizmo_modes: GizmoMode::all(),
        }
    }
}

fn init_panels(mut state: ResMut<EditorState>) {
    state.init_panel::<AssetsPanel>();
    state.init_panel::<ExplorerPanel>();
    state.init_panel::<HierarchyPanel>();
    state.init_panel::<InspectorPanel>();
    state.init_panel::<ResourcesPanel>();
    state.init_panel::<ScenePanel>();
}

fn handle_selection(
    mut inspector_state: ResMut<InspectorState>,
    mut deselect_events: EventReader<Pointer<Deselect>>,
    mut select_events: EventReader<Pointer<Select>>,
) {
    for e in deselect_events.read() {
        inspector_state.selected_entities.remove(e.target);
    }

    for e in select_events.read() {
        inspector_state
            .selected_entities
            .select_maybe_add(e.target, true);
    }
}

fn add_selection(
    query: Query<Entity, (Without<PickSelection>, With<Visibility>)>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(PickSelection::default());
    }
}

const COMPILATION_ANIMATION_DURATION: f32 = 0.5;
const COMPILATION_ANIMATION_DOTS: u32 = 3;

fn show_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };

    let mut egui_context = egui_context.clone();

    world.resource_scope::<EditorState, _>(|world, mut editor_state| {
        editor_state.ui(world, egui_context.get_mut())
    });

    compilation(egui_context.get_mut(), world);
}

fn compilation(egui_ctx: &mut egui::Context, world: &mut World) {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let project_dir = world.resource::<ProjectDir>().clone();
    let delta_seconds = world.resource::<Time>().delta_secs();
    world.resource_scope(|world, mut state: Mut<EditorState>| {
        if let Some(compilation_process) = state.compilation_process.as_mut() {
            match compilation_process.try_wait() {
                Ok(status) => {
                    if let Some(status) = status {
                        setup_editing(DEFAULT_PROJECT_DEPENDENCIES, &project_dir);

                        state.compilation_process = None;
                        state.compilation_animation_time = 0.;
                        if status.success() {
                            let mut lib_path = project_dir.clone();
                            lib_path.push("target/debug/bevy_project");

                            unsafe {
                                match Library::new(library_filename(lib_path)) {
                                    Ok(lib) => {
                                        state.lib = Some(lib);
                                    }
                                    Err(err) => {
                                        bevy::log::error!("Failed to load library: {err}");
                                    }
                                }

                                let func = if let Some(lib) = &mut state.lib {
                                    match lib.get::<Symbol<
                                        unsafe extern "C" fn(
                                            &mut World,
                                            &mut TypeRegistry,
                                            &mut Vec<TypeId>,
                                            &mut Vec<ComponentId>,
                                        ),
                                    >>(
                                        b"register_types"
                                    ) {
                                        Ok(func) => Some(func),
                                        Err(err) => {
                                            bevy::log::error!("Failed to get symbol: {err}");
                                            None
                                        }
                                    }
                                } else {
                                    None
                                };

                                if let Some(func) = func {
                                    let mut type_registry = type_registry.write();
                                    let mut registration_type_ids = Vec::new();
                                    let mut registration_component_ids = Vec::new();

                                    func(
                                        world,
                                        &mut type_registry,
                                        &mut registration_type_ids,
                                        &mut registration_component_ids,
                                    );

                                    state.registration_type_ids = registration_type_ids;
                                    state.registration_component_ids = registration_component_ids;
                                }
                            }

                            load_last_scene(world);
                        }
                    } else {
                        state.compilation_animation_time += delta_seconds;

                        let mut dots = (state.compilation_animation_time
                            / COMPILATION_ANIMATION_DURATION)
                            as u32;

                        if dots > COMPILATION_ANIMATION_DOTS {
                            state.compilation_animation_time = 0.;
                            dots = 0;
                        }

                        let mut message = String::from("Compilation");
                        for _ in 0..dots {
                            message.push('.');
                        }

                        Modal::new("compilation_process".into()).show(egui_ctx, |ui| {
                            ui.set_width(250.0);
                            ui.heading(message);
                        });
                    }
                }
                Err(_) => {
                    state.compilation_process = None;
                    state.compilation_animation_time = 0.;
                }
            }
        }
    });
}

fn load_scene_from(world: &mut World, path: &PathBuf) {
    let mut text = String::new();
    let Ok(_) = File::open(path.clone()).and_then(|mut file| file.read_to_string(&mut text)) else {
        bevy::log::error!("Error while reading from scene file");
        return;
    };

    let dynamic_scene = {
        let type_registry = world.resource::<AppTypeRegistry>();
        let type_registry = type_registry.read();

        let scene_deserializer = SceneDeserializer {
            type_registry: &type_registry,
        };

        let mut deserializer = Deserializer::from_str(&text).unwrap();

        scene_deserializer.deserialize(&mut deserializer)
    };

    let dynamic_scene = match dynamic_scene {
        Ok(dynamic_scene) => dynamic_scene,
        Err(err) => {
            bevy::log::error!("Failed to deserialize scene: {err}");
            return;
        }
    };

    let mut entity_map = EntityHashMap::default();
    let Ok(_) = dynamic_scene.write_to_world(world, &mut entity_map) else {
        bevy::log::error!("Failed load scene");
        return;
    };

    world.resource_mut::<SelectedScene>().active_scene_path = Some(path.clone());
}

pub fn unload_scene(world: &mut World) {
    world
        .resource_mut::<InspectorState>()
        .selected_entities
        .clear();

    let mut entities = world.query_filtered::<Entity, Without<EditorEntity>>();
    let entities = entities.iter(world).collect::<Vec<_>>();

    for entity in entities {
        world.despawn(entity);
    }
}

pub fn load_last_scene(world: &mut World) {
    let project_dir = world.resource::<ProjectDir>().clone();

    if world
        .resource::<SelectedScene>()
        .active_scene_path
        .as_ref()
        .is_none_or(|path| !path.exists() || path.strip_prefix(&project_dir).is_err())
    {
        return;
    }

    load_scene_from(
        world,
        &world
            .resource::<SelectedScene>()
            .active_scene_path
            .clone()
            .unwrap(),
    );
}

fn load_scene(world: &mut World) {
    let project_dir = world.resource::<ProjectDir>().clone();

    let path = FileDialog::new()
        .add_filter("Bevy Scene", &["scn"])
        .set_directory(project_dir)
        .pick_file();

    let Some(path) = path else {
        return;
    };

    load_scene_from(world, &path);
}

fn save_scene(world: &mut World) {
    let project_dir = world.resource::<ProjectDir>().clone();

    if world
        .resource::<SelectedScene>()
        .active_scene_path
        .as_ref()
        .is_none_or(|path| !path.exists() || path.strip_prefix(&project_dir).is_err())
    {
        save_scene_as(world);
        return;
    }

    let path = world
        .resource::<SelectedScene>()
        .active_scene_path
        .clone()
        .unwrap();

    save_scene_to(world, &path);
}

fn save_scene_to(world: &mut World, path: &PathBuf) {
    let entities = world
        .query_filtered::<Entity, Without<EditorEntity>>()
        .iter(world)
        .collect::<Vec<_>>();

    let scene = DynamicSceneBuilder::from_world(&world)
        .deny_component::<Mesh3d>()
        .deny_component::<MeshMaterial3d<StandardMaterial>>()
        .deny_component::<PickSelection>()
        .extract_entities(entities.iter().cloned())
        .build();

    {
        let type_registry = world.resource::<AppTypeRegistry>();
        let type_registry = type_registry.read();

        match scene.serialize(&type_registry) {
            Ok(serialized_scene) => {
                File::create(path.clone())
                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                    .expect("Error while writing scene to file");
            }
            Err(err) => {
                bevy::log::error!("Scene saving error: {}", err);
            }
        };
    }

    world.resource_mut::<SelectedScene>().active_scene_path = Some(path.clone());
}

fn save_scene_as(world: &mut World) {
    let project_dir = world.resource::<ProjectDir>().clone();

    let path = FileDialog::new()
        .add_filter("Bevy Scene", &["scn"])
        .set_directory(project_dir)
        .save_file();

    let Some(path) = path else {
        return;
    };

    save_scene_to(world, &path);
}

fn draw_menu(editor_state: &mut EditorState, world: &mut World, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                load_scene(world);
                ui.close_menu();
            }

            if ui.button("Save").clicked() {
                save_scene(world);
                ui.close_menu();
            }

            if ui.button("Save As...").clicked() {
                save_scene_as(world);
                ui.close_menu();
            }
        });

        ui.menu_button("View", |ui| {
            for name in editor_state.panels.keys() {
                if ui.button(name).clicked() {
                    editor_state.docking.add_window(vec![name.clone()]);
                    ui.close_menu();
                }
            }
        });

        if ui.button("Compile").clicked() {
            editor_state.compile(world);
        }

        if ui.button("Launch").clicked() {
            editor_state.launch(world);
        }
    });
}

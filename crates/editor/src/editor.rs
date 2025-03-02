use std::any::TypeId;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::dock::{EditorDockState, PanelViewer, StandardEditorDockStateTemplate};
use crate::editor_config::EditorConfigPlugin;
use crate::panel::Panel;
use crate::panels::assets::AssetsPanel;
use crate::panels::explorer::ExplorerPanel;
use crate::panels::hierarchy::HierarchyPanel;
use crate::panels::inspector::InspectorPanel;
use crate::panels::resources::ResourcesPanel;
use crate::panels::scene::ScenePanel;
use crate::scene::{EditorEntity, EditorScenePlugin};
use crate::window_config::WindowConfigPlugin;
use crate::EditorSet;
use bevy::app::{App, Plugin, PreUpdate, Startup};
use bevy::asset::UntypedAssetId;
use bevy::ecs::entity::{Entity, EntityHashMap};
use bevy::ecs::event::EventReader;
use bevy::ecs::query::{With, Without};
use bevy::ecs::reflect::AppTypeRegistry;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, ResMut, Resource};
use bevy::ecs::world::World;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::picking::events::Pointer;
use bevy::reflect::TypeRegistry;
use bevy::render::mesh::Mesh3d;
use bevy::render::view::Visibility;
use bevy::scene::ron::Deserializer;
use bevy::scene::serde::SceneDeserializer;
use bevy::scene::DynamicSceneBuilder;
use bevy::utils::default;
use bevy::utils::hashbrown::HashMap;
use bevy::window::PrimaryWindow;
use bevy_egui::egui::panel::TopBottomSide;
use bevy_egui::egui::{Id, TopBottomPanel};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use egui_dock::DockArea;
use libloading::{Library, Symbol};
use rfd::FileDialog;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};
use transform_gizmo_egui::{EnumSet, Gizmo, GizmoMode};

use crate::egui_config::EguiConfigPlugin;
use crate::selection::{Deselect, PickSelection, Select};

#[derive(Eq, PartialEq)]
pub enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Default, Resource)]
pub struct SelectedProject {
    pub dir: Option<PathBuf>,
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WindowConfigPlugin)
            .add_plugins(EguiConfigPlugin)
            .add_plugins(EditorScenePlugin)
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(EditorConfigPlugin)
            .insert_resource(EditorState::new())
            .insert_resource(InspectorState::new())
            .insert_resource(GizmoState::new())
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
}

#[derive(Default, Resource, Serialize, Deserialize)]
pub struct SelectedScene {
    pub active_scene_path: Option<PathBuf>,
}

impl EditorState {
    fn new() -> Self {
        Self {
            docking: EditorDockState::standard(),
            panels: HashMap::default(),
            lib: default(),
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

    fn compile(&mut self, world: &mut World) -> Result<(), Box<dyn Error>> {
        let selected_project = world.resource::<SelectedProject>();
        let mut path = selected_project.dir.clone().unwrap();
        path.push("target/debug/bevy_project");

        unsafe {
            self.lib = Some(Library::new(path)?);

            if let Some(lib) = &mut self.lib {
                let func =
                    lib.get::<Symbol<unsafe extern "C" fn(&mut TypeRegistry)>>(b"register_types")?;
                let type_registry = world.resource::<AppTypeRegistry>().clone();
                let mut type_registry = type_registry.write();
                func(&mut type_registry);
            }

            Ok(())
        }
    }
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

    let Ok(dynamic_scene) = dynamic_scene else {
        bevy::log::error!("Failed to deserialize scene");
        return;
    };

    let mut entity_map = EntityHashMap::default();
    let Ok(_) = dynamic_scene.write_to_world(world, &mut entity_map) else {
        bevy::log::error!("Failed load scene");
        return;
    };

    world.resource_mut::<SelectedScene>().active_scene_path = Some(path.clone());
}

pub fn load_last_scene(world: &mut World) {
    let project_dir = world.resource::<SelectedProject>().dir.clone().unwrap();

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
    let project_dir = world.resource::<SelectedProject>().dir.clone().unwrap();

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
    let project_dir = world.resource::<SelectedProject>().dir.clone().unwrap();

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
    let project_dir = world.resource::<SelectedProject>().dir.clone().unwrap();

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
            bevy::log::info!("{:?}", editor_state.compile(world));
        }
    });
}

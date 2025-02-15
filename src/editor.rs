use std::any::TypeId;
use std::path::PathBuf;

use bevy::app::{App, Plugin, PreUpdate};
use bevy::asset::UntypedAssetId;
use bevy::ecs::component::Component;
use bevy::input::ButtonInput;
use bevy::prelude::{in_state, EventReader, KeyCode, OnEnter, Pointer, Query, Res, With, World};
use bevy::prelude::{IntoSystemConfigs, ResMut, Resource};
use bevy::utils::default;
use bevy::utils::hashbrown::HashMap;
use rfd::FileDialog;

use crate::demo_scene::DemoScenePlugin;
use crate::dock::{EditorDockState, PanelViewer, StandardEditorDockStateTemplate};
use crate::panel::Panel;
use crate::panels::assets::AssetsPanel;
use crate::panels::explorer::ExplorerPanel;
use crate::panels::hierarchy::HierarchyPanel;
use crate::panels::inspector::InspectorPanel;
use crate::panels::resources::ResourcesPanel;
use crate::panels::scene::ScenePanel;
use crate::window_config::WindowConfigPlugin;
use crate::{AppSet, AppState};
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::egui::panel::TopBottomSide;
use bevy_egui::egui::{Id, TopBottomPanel};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use egui_dock::DockArea;
use transform_gizmo_egui::{EnumSet, Gizmo, GizmoMode};

use crate::egui_config::EguiConfigPlugin;
use crate::selection::{Deselect, Select};

#[derive(Component)]
pub struct MainCamera;

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
            .add_plugins(DemoScenePlugin)
            .add_plugins(DefaultInspectorConfigPlugin)
            .insert_resource(EditorState::new())
            .insert_resource(InspectorState::new())
            .insert_resource(GizmoState::new())
            .init_resource::<SelectedProject>()
            .add_systems(OnEnter(AppState::Editor), (setup_window, init_panels))
            .add_systems(
                PreUpdate,
                (handle_selection, set_gizmo_mode, show_ui)
                    .chain()
                    .in_set(AppSet::Egui)
                    .run_if(in_state(AppState::Editor)),
            );
    }
}

#[derive(Resource)]
pub struct EditorState {
    pub docking: EditorDockState,
    pub panels: HashMap<String, Box<dyn Panel>>,
}

impl EditorState {
    fn new() -> Self {
        Self {
            docking: EditorDockState::standard(),
            panels: HashMap::default(),
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

        DockArea::new(&mut self.docking)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut panel_viewer);

        ctx.request_repaint();
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
            component_filter: default()
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

fn setup_window(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = windows.single_mut();
    window.title = "BevyEditor".into();
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

fn set_gizmo_mode(input: Res<ButtonInput<KeyCode>>, mut gizmo_state: ResMut<GizmoState>) {
    let keybinds = [
        (KeyCode::KeyR, GizmoMode::all_rotate()),
        (KeyCode::KeyT, GizmoMode::all_translate()),
        (KeyCode::KeyS, GizmoMode::all_scale()),
    ];

    for (key, mode) in keybinds {
        if input.just_pressed(key) {
            gizmo_state.gizmo_modes = mode;
        }
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

fn save_scene(world: &mut World) {
    let project_dir = world.resource::<SelectedProject>().dir.clone().unwrap();
    
    let path = FileDialog::new()
        .add_filter("Bevy Scene", &["scn"])
        .set_directory(project_dir)
        .save_file();

    
}

fn draw_menu(editor_state: &mut EditorState, world: &mut World, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Save As...").clicked() {
                save_scene(world);
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
    });
}

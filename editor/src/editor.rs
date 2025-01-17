use std::any::TypeId;
use std::path::PathBuf;

use bevy::app::{App, Plugin, PreUpdate};
use bevy::asset::{ReflectAsset, UntypedAssetId};
use bevy::input::ButtonInput;
use bevy::math::UVec2;
use bevy::prelude::{
    in_state, AppTypeRegistry, Camera, Component, EventReader, GlobalTransform, KeyCode, OnEnter,
    Pointer, Projection, Query, ReflectResource, Res, Transform, With, World,
};
use bevy::prelude::{IntoSystemConfigs, ResMut, Resource};
use bevy::utils::hashbrown::HashMap;

use crate::demo_scene::DemoScenePlugin;
use crate::panel::Panel;
use crate::transform_gizmo_ext::GizmoNewExt;
use crate::window_config::WindowConfigPlugin;
use crate::{AppSet, AppState};
use bevy::reflect::TypeRegistry;
use bevy::render::camera::{CameraProjection, Viewport};
use bevy::utils::default;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::egui::panel::TopBottomSide;
use bevy_egui::egui::{Id, TopBottomPanel};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::bevy_inspector::hierarchy::{hierarchy_ui, SelectedEntities};
use bevy_inspector_egui::bevy_inspector::{
    self, ui_for_entities_shared_components, ui_for_entity_with_children,
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use egui_dock::{DockArea, DockState, NodeIndex};
use transform_gizmo_egui::{
    math::{Quat, Transform as GizmoTransform, Vec3},
    EnumSet, Gizmo, GizmoConfig, GizmoMode, GizmoOrientation,
};

use crate::egui_config::EguiConfigPlugin;
use crate::selection::{Deselect, Select};

pub type EditorTab = String;
pub type EditorDockState = DockState<EditorTab>;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WindowConfigPlugin)
            .add_plugins(EguiConfigPlugin)
            .add_plugins(DemoScenePlugin)
            .add_plugins(DefaultInspectorConfigPlugin)
            .insert_resource(EditorState::new())
            .init_resource::<SelectedProject>()
            .add_systems(OnEnter(AppState::Editor), setup_window)
            .add_systems(
                PreUpdate,
                (
                    handle_selection,
                    set_gizmo_mode,
                    show_ui,
                    set_camera_viewport,
                )
                    .chain()
                    .in_set(AppSet::Egui)
                    .run_if(in_state(AppState::Editor)),
            );
    }
}

fn setup_window(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = windows.single_mut();
    window.title = "BevyEditor".into();
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum EguiWindow {
    Game,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
}

#[derive(Default, Resource)]
pub struct SelectedProject {
    pub dir: Option<PathBuf>,
}

#[derive(Resource)]
pub struct EditorState {
    pub docking: EditorDockState,
    panels: HashMap<String, Box<dyn Panel>>,

    viewport_rect: egui::Rect,
    pub selected_entities: SelectedEntities,
    selection: InspectorSelection,

    gizmo: Gizmo,
    gizmo_modes: EnumSet<GizmoMode>,
}

impl EditorState {
    fn new() -> Self {
        let mut state = EditorDockState::new(vec![String::from("Game")]);
        let tree = state.main_surface_mut();
        let [game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.75, vec![String::from("Inspector")]);
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![String::from("Hierarchy")]);
        let [_game, _bottom] = tree.split_below(
            game,
            0.8,
            vec![String::from("Resources"), String::from("Assets")],
        );

        Self {
            docking: state,
            panels: HashMap::default(),
            selection: InspectorSelection::Entities,
            selected_entities: SelectedEntities::default(),
            viewport_rect: egui::Rect::NOTHING,
            gizmo: Gizmo::default(),
            gizmo_modes: GizmoMode::all(),
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        TopBottomPanel::new(TopBottomSide::Top, Id::new("menu")).show(ctx, |ui| {
            draw_menu(&mut self.docking, ui);
        });

        let mut tab_viewer = TabViewer {
            world,
            panels: &mut self.panels,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            gizmo_modes: &mut self.gizmo_modes,
            gizmo: &mut self.gizmo,
        };

        DockArea::new(&mut self.docking)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);

        ctx.request_repaint();
    }
}

fn handle_selection(
    mut editor_state: ResMut<EditorState>,
    mut deselect_events: EventReader<Pointer<Deselect>>,
    mut select_events: EventReader<Pointer<Select>>,
) {
    for e in deselect_events.read() {
        editor_state.selected_entities.remove(e.target);
    }

    for e in select_events.read() {
        editor_state
            .selected_entities
            .select_maybe_add(e.target, true);
    }
}

fn set_gizmo_mode(input: Res<ButtonInput<KeyCode>>, mut editor_state: ResMut<EditorState>) {
    let keybinds = [
        (KeyCode::KeyR, GizmoMode::all_rotate()),
        (KeyCode::KeyT, GizmoMode::all_translate()),
        (KeyCode::KeyS, GizmoMode::all_scale()),
    ];

    for (key, mode) in keybinds {
        if input.just_pressed(key) {
            editor_state.gizmo_modes = mode;
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

fn set_camera_viewport(
    editor_state: Res<EditorState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Query<&bevy_egui::EguiContextSettings>,
    mut cameras: Query<&mut Camera, With<MainCamera>>,
) {
    let mut cam = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let scale_factor = window.scale_factor() * egui_settings.single().scale_factor;

    let viewport_pos = editor_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = editor_state.viewport_rect.size() * scale_factor;

    let physical_position = UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32);
    let physical_size = UVec2::new(viewport_size.x as u32, viewport_size.y as u32);

    // The desired viewport rectangle at its offset in "physical pixel space"
    let rect = physical_position + physical_size;

    let window_size = window.physical_size();
    // wgpu will panic if trying to set a viewport rect which has coordinates extending
    // past the size of the render target, i.e. the physical window in our case.
    // Typically this shouldn't happen- but during init and resizing etc. edge cases might occur.
    // Simply do nothing in those cases.
    if rect.x <= window_size.x && rect.y <= window_size.y {
        cam.viewport = Some(Viewport {
            physical_position,
            physical_size,
            depth: 0.0..1.0,
        });
    }
}

struct TabViewer<'a> {
    world: &'a mut World,
    panels: &'a mut HashMap<String, Box<dyn Panel>>,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
    gizmo_modes: &'a mut EnumSet<GizmoMode>,
    gizmo: &'a mut Gizmo,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EditorTab;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        if let Some(panel) = self.panels.get_mut(window) {
            panel.draw(self.world, ui);
        } else if window == "Game" {
            *self.viewport_rect = ui.clip_rect();
            draw_gizmo(
                self.world,
                self.selected_entities,
                self.gizmo,
                *self.gizmo_modes,
                ui,
            );
        } else if window == "Hierarchy" {
            let selected = hierarchy_ui(self.world, ui, self.selected_entities);
            if selected {
                *self.selection = InspectorSelection::Entities;
            }
        } else if window == "Resources" {
            select_resource(ui, &type_registry, self.selection)
        } else if window == "Assets" {
            select_asset(ui, &type_registry, self.world, self.selection)
        } else if window == "Inspector" {
            match *self.selection {
                InspectorSelection::Entities => match self.selected_entities.as_slice() {
                    &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                    entities => ui_for_entities_shared_components(self.world, entities, ui),
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_resource(
                        self.world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        self.world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
            }
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        format!("{window:?}").into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        window != "Game"
    }
}

fn draw_gizmo(
    world: &mut World,
    selected_entities: &mut SelectedEntities,
    gizmo: &mut Gizmo,
    gizmo_modes: EnumSet<GizmoMode>,
    ui: &mut egui::Ui,
) {
    if selected_entities.len() == 0 {
        return;
    }

    let transform_entities = selected_entities
        .iter()
        .take_while(|item| world.get::<Transform>(*item).is_some())
        .collect::<Vec<_>>();

    if transform_entities.len() == 0 {
        return;
    }

    let selections = transform_entities
        .iter()
        .map(|item| {
            let transform = world.get::<Transform>(*item).unwrap();
            GizmoTransform::from_scale_rotation_translation(
                transform.scale.as_dvec3(),
                transform.rotation.as_dquat(),
                transform.translation.as_dvec3(),
            )
        })
        .collect::<Vec<_>>();

    let (cam_transform, projection) = world
        .query_filtered::<(&GlobalTransform, &Projection), With<MainCamera>>()
        .single(world);

    let view_matrix = cam_transform.compute_matrix().inverse();
    let projection_matrix = projection.get_clip_from_view();

    gizmo.update_config(GizmoConfig {
        view_matrix: view_matrix.as_dmat4().into(),
        projection_matrix: projection_matrix.as_dmat4().into(),
        orientation: GizmoOrientation::Global,
        modes: gizmo_modes,
        ..default()
    });

    if let Some(result) = gizmo.interact_new(ui, &selections).map(|(_, res)| res) {
        for (entity, data) in transform_entities.iter().zip(result.iter()) {
            let mut transform = world.get_mut::<Transform>(*entity).unwrap();
            transform.translation = Vec3::new(
                data.translation.x as f32,
                data.translation.y as f32,
                data.translation.z as f32,
            );
            transform.rotation = Quat::from_xyzw(
                data.rotation.v.x as f32,
                data.rotation.v.y as f32,
                data.rotation.v.z as f32,
                data.rotation.s as f32,
            );
            transform.scale = Vec3::new(
                data.scale.x as f32,
                data.scale.y as f32,
                data.scale.z as f32,
            );
        }
    };
}

fn draw_menu(dock_state: &mut EditorDockState, ui: &mut egui::Ui) {
    ui.menu_button("View", |ui| {
        if ui.button("Game").clicked() {
            dock_state.add_window(vec![String::from("Game")]);
            ui.close_menu();
        }
    });
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name.to_string());
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &World,
    selection: &mut InspectorSelection,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let handles: Vec<_> = reflect_asset.ids(world).collect();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match *selection {
                    InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{:?}", handle))
                    .clicked()
                {
                    *selection =
                        InspectorSelection::Asset(asset_type_id, asset_name.to_string(), handle);
                }
            }
        });
    }
}

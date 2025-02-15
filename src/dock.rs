use bevy::ecs::world::World;
use bevy::utils::HashMap;
use bevy_egui::egui::{Ui, WidgetText};
use egui_dock::{DockState, NodeIndex, TabViewer};

use crate::panel::Panel;

pub type EditorTab = String;
pub type EditorDockState = DockState<EditorTab>;

pub trait StandardEditorDockStateTemplate {
    fn standard() -> EditorDockState;
}

impl StandardEditorDockStateTemplate for EditorDockState {
    fn standard() -> Self {
        let mut state = EditorDockState::new(vec![String::from("Scene")]);
        let tree = state.main_surface_mut();
        let [game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.75, vec![String::from("Inspector")]);
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![String::from("Hierarchy")]);
        let [_game, _bottom] = tree.split_below(
            game,
            0.8,
            vec![String::from("Resources"), String::from("Assets")],
        );

        state
    }
}

pub struct PanelViewer<'a> {
    pub world: &'a mut World,
    pub panels: &'a mut HashMap<String, Box<dyn Panel>>,
}

impl TabViewer for PanelViewer<'_> {
    type Tab = EditorTab;

    fn ui(&mut self, ui: &mut Ui, window: &mut Self::Tab) {
        if let Some(panel) = self.panels.get_mut(window) {
            panel.ui(self.world, ui);
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> WidgetText {
        window.as_str().into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        match self.panels.get(window) {
            Some(panel) => panel.clear_background(),
            None => true,
        }
    }
}

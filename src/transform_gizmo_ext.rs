use std::default;

use bevy::utils::default;
use bevy_egui::egui::{
    self,
    epaint::{RectShape, Vertex},
    Mesh, PointerButton, Pos2, Rect, Rgba, Rounding, Sense, Stroke, TextureId, Ui, Vec2,
};
use transform_gizmo_egui::{
    math::Transform, Color32, Gizmo, GizmoConfig, GizmoInteraction, GizmoResult,
};

pub trait GizmoNewExt {
    fn interact_new(
        &mut self,
        ui: &Ui,
        targets: &[Transform],
    ) -> Option<(GizmoResult, Vec<Transform>)>;
}

impl GizmoNewExt for Gizmo {
    fn interact_new(
        &mut self,
        ui: &Ui,
        targets: &[Transform],
    ) -> Option<(GizmoResult, Vec<Transform>)> {
        let cursor_pos = ui
            .input(|input| input.pointer.hover_pos())
            .unwrap_or_default();

        let mut viewport = self.config().viewport;
        if !viewport.is_finite() {
            viewport = ui.clip_rect();
        }

        self.update_config(GizmoConfig {
            viewport,
            pixels_per_point: ui.ctx().pixels_per_point(),
            ..*self.config()
        });

        let gizmo_result = self.update(
            GizmoInteraction {
                cursor_pos: (cursor_pos.x, cursor_pos.y),
                hovered: true,
                drag_started: ui
                    .input(|input| input.pointer.button_pressed(PointerButton::Primary)),
                dragging: ui.input(|input| input.pointer.button_down(PointerButton::Primary)),
            },
            targets,
        );

        let draw_data = self.draw();

        let mesh = Mesh {
            indices: draw_data.indices,
            vertices: draw_data
                .vertices
                .into_iter()
                .zip(draw_data.colors)
                .map(|(pos, [r, g, b, a])| Vertex {
                    pos: pos.into(),
                    uv: Pos2::default(),
                    color: Rgba::from_rgba_premultiplied(r, g, b, a).into(),
                })
                .collect(),
            ..Default::default()
        };

        let bounds = mesh.calc_bounds();

        ui.painter().add(mesh);

        // Egui can interact only with rectangles.
        ui.interact(bounds, ui.id(), Sense::drag());

        gizmo_result
    }
}

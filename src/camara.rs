use eframe::egui;

pub struct Camara {
    pub zoom: f32,
    pub offset: egui::Vec2,
}

impl Default for Camara {
    fn default() -> Self {
        Self { zoom: 1.0, offset: egui::Vec2::ZERO }
    }
}

impl Camara {
    pub fn interactuar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let resp = ui.allocate_rect(rect, egui::Sense::drag());

        if resp.dragged() {
            self.offset += resp.drag_delta();
        }

        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                if let Some(cursor) = resp.hover_pos() {
                    let centro = rect.center();
                    let zoom_viejo = self.zoom;
                    let zoom_nuevo = (zoom_viejo * (scroll * 0.001).exp()).clamp(0.01, 100.0);
                    let mundo = (cursor - centro - self.offset) / zoom_viejo;
                    self.offset = cursor - centro - mundo * zoom_nuevo;
                    self.zoom = zoom_nuevo;
                }
            }
        }
    }

    pub fn origen(&self, centro: egui::Pos2) -> egui::Pos2 {
        centro + self.offset
    }

    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.offset = egui::Vec2::ZERO;
    }
}

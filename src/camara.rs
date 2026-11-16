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
    pub fn procesar_zoom(&mut self, ui: &egui::Ui, centro_pantalla: egui::Pos2) -> bool {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll == 0.0 {
            return false;
        }
        let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) else {
            return false;
        };
        let factor = (scroll * 0.001).exp();
        let zoom_viejo = self.zoom;
        let zoom_nuevo = (zoom_viejo * factor).clamp(0.01, 100.0);
        let mundo = (cursor - centro_pantalla - self.offset) / zoom_viejo;
        self.offset = cursor - centro_pantalla - mundo * zoom_nuevo;
        self.zoom = zoom_nuevo;
        true
    }

    pub fn procesar_pan(&mut self, ui: &egui::Ui) -> bool {
        let delta = ui.input(|i| i.pointer.delta());
        if ui.input(|i| i.pointer.primary_down()) && delta != egui::Vec2::ZERO {
            self.offset += delta;
            return true;
        }
        false
    }

    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.offset = egui::Vec2::ZERO;
    }
}

pub mod arbol_pitagoras;
pub mod arbol_pitagoras_paralelo;
pub mod copo_koch;
pub mod copo_koch_paralelo;
pub mod romanesco;
pub mod dragon;     // <-- NUEVO

use eframe::egui;

pub trait Fractal {
    fn nombre(&self) -> &'static str;
    fn ui_controles(&mut self, ui: &mut egui::Ui);
    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect);
}
